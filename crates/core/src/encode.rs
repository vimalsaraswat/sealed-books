//! Canonical binary serializer and deserializer for Sealed Books.
//!
//! Enforces the 5 canonical encoding rules:
//! 1. Fixed field order.
//! 2. Length-prefixed strings and slices (protects against concatenation collisions).
//! 3. Money represented strictly as u64 big-endian minor units (never floats).
//! 4. Timestamps as deterministic strings (UTC RFC 3339).
//! 5. A version byte (`0x01`) leading every encoded structure.
//!
//! In addition, journal lines within an entry are canonically sorted before encoding
//! so that identical entries created with different line insertion orders serialize
//! to byte-for-byte identical output.

use crate::error::CoreError;
use crate::types::{Entry, Line, SealPayload, SealStatement};

pub const CANONICAL_VERSION_V1: u8 = 0x01;

/// Appends a single byte.
#[inline]
fn write_u8(buf: &mut Vec<u8>, val: u8) {
    buf.push(val);
}

/// Appends a 32-bit big-endian integer.
#[inline]
fn write_u32_be(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

/// Appends a 64-bit big-endian integer.
#[inline]
fn write_u64_be(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_be_bytes());
}

/// Appends a length-prefixed UTF-8 string: [u32 length in BE] + [raw bytes].
#[inline]
fn write_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32_be(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

/// Appends an optional string: 0x00 for None, or 0x01 + length-prefixed string for Some.
#[inline]
fn write_opt_str(buf: &mut Vec<u8>, opt: Option<&str>) {
    match opt {
        Some(s) => {
            write_u8(buf, 0x01);
            write_str(buf, s);
        }
        None => {
            write_u8(buf, 0x00);
        }
    }
}

/// Encodes a single journal line.
fn encode_line(buf: &mut Vec<u8>, line: &Line) {
    write_str(buf, &line.id);
    write_str(buf, &line.account_id);
    write_u8(buf, line.direction.as_byte());
    write_u64_be(buf, line.amount_minor);
    write_opt_str(buf, line.description.as_deref());
}

/// Canonically encodes a journal entry into a byte vector.
///
/// Sorting rule: Lines are sorted deterministically by `(direction, account_id, amount_minor, id)`
/// prior to encoding. This guarantees that entry encoding is invariant under line permutations.
pub fn encode_entry(entry: &Entry) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);

    // 1. Version byte
    write_u8(&mut buf, CANONICAL_VERSION_V1);

    // 2. Entry header fields (fixed order)
    write_str(&mut buf, &entry.id);
    write_str(&mut buf, &entry.date);
    write_str(&mut buf, &entry.description);
    write_str(&mut buf, &entry.created_at);

    // 3. Lines (canonically sorted)
    let mut sorted_lines: Vec<&Line> = entry.lines.iter().collect();
    sorted_lines.sort_by(|a, b| {
        (a.direction.as_byte(), &a.account_id, a.amount_minor, &a.id).cmp(&(
            b.direction.as_byte(),
            &b.account_id,
            b.amount_minor,
            &b.id,
        ))
    });

    write_u32_be(&mut buf, sorted_lines.len() as u32);
    for line in sorted_lines {
        encode_line(&mut buf, line);
    }

    buf
}

/// Canonically encodes a SealStatement into a byte vector.
pub fn encode_statement(statement: &SealStatement) -> Vec<u8> {
    let mut buf = Vec::with_capacity(160);

    // 1. Version byte
    write_u8(&mut buf, statement.version);

    // 2. Statement fields (fixed order)
    write_str(&mut buf, &statement.entity);
    write_str(&mut buf, &statement.period_start);
    write_str(&mut buf, &statement.period_end);
    write_u64_be(&mut buf, statement.entry_count);
    write_u64_be(&mut buf, statement.total_debits_minor);
    write_u64_be(&mut buf, statement.total_credits_minor);

    // 3. Ledger root (raw 32 bytes)
    buf.extend_from_slice(&statement.ledger_root);

    buf
}

/// Canonically encodes a full on-chain SealPayload into a compact binary message (~430 bytes).
pub fn encode_seal_payload(payload: &SealPayload) -> Vec<u8> {
    let mut buf = encode_statement(&payload.statement);
    buf.extend_from_slice(&payload.approver_1_pubkey);
    buf.extend_from_slice(&payload.approver_1_sig);
    buf.extend_from_slice(&payload.approver_2_pubkey);
    buf.extend_from_slice(&payload.approver_2_sig);
    buf
}

/// Internal cursor for zero-copy binary reading.
struct ByteReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, CoreError> {
        if self.pos < self.buf.len() {
            let b = self.buf[self.pos];
            self.pos += 1;
            Ok(b)
        } else {
            Err(CoreError::SerializationError(
                "Unexpected end of buffer while reading u8".into(),
            ))
        }
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], CoreError> {
        if self.pos + len <= self.buf.len() {
            let slice = &self.buf[self.pos..self.pos + len];
            self.pos += len;
            Ok(slice)
        } else {
            Err(CoreError::SerializationError(format!(
                "Unexpected end of buffer: requested {len} bytes, {} available",
                self.buf.len().saturating_sub(self.pos)
            )))
        }
    }

    fn read_u32_be(&mut self) -> Result<u32, CoreError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn read_u64_be(&mut self) -> Result<u64, CoreError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn read_exact<const N: usize>(&mut self) -> Result<[u8; N], CoreError> {
        let bytes = self.read_bytes(N)?;
        let mut arr = [0u8; N];
        arr.copy_from_slice(bytes);
        Ok(arr)
    }

    fn read_str(&mut self) -> Result<String, CoreError> {
        let len = self.read_u32_be()? as usize;
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| CoreError::SerializationError(format!("Invalid UTF-8 string: {e}")))
    }

    fn ensure_empty(&self) -> Result<(), CoreError> {
        if self.pos == self.buf.len() {
            Ok(())
        } else {
            Err(CoreError::SerializationError(format!(
                "Trailing bytes remaining: read {} of {} bytes",
                self.pos,
                self.buf.len()
            )))
        }
    }
}

/// Decodes a `SealStatement` from a reader.
fn decode_statement_from_reader(reader: &mut ByteReader) -> Result<SealStatement, CoreError> {
    let version = reader.read_u8()?;
    if version != CANONICAL_VERSION_V1 {
        return Err(CoreError::SerializationError(format!(
            "Unsupported statement version: expected {CANONICAL_VERSION_V1}, got {version}"
        )));
    }
    let entity = reader.read_str()?;
    let period_start = reader.read_str()?;
    let period_end = reader.read_str()?;
    let entry_count = reader.read_u64_be()?;
    let total_debits_minor = reader.read_u64_be()?;
    let total_credits_minor = reader.read_u64_be()?;
    let ledger_root = reader.read_exact::<32>()?;

    Ok(SealStatement {
        version,
        entity,
        period_start,
        period_end,
        entry_count,
        total_debits_minor,
        total_credits_minor,
        ledger_root,
    })
}

/// Decodes a canonically encoded `SealStatement` from binary bytes.
pub fn decode_statement(bytes: &[u8]) -> Result<SealStatement, CoreError> {
    let mut reader = ByteReader::new(bytes);
    let statement = decode_statement_from_reader(&mut reader)?;
    reader.ensure_empty()?;
    Ok(statement)
}

/// Decodes a canonically encoded `SealPayload` from binary bytes.
pub fn decode_seal_payload(bytes: &[u8]) -> Result<SealPayload, CoreError> {
    let mut reader = ByteReader::new(bytes);
    let statement = decode_statement_from_reader(&mut reader)?;
    let approver_1_pubkey = reader.read_exact::<33>()?;
    let approver_1_sig = reader.read_exact::<64>()?;
    let approver_2_pubkey = reader.read_exact::<33>()?;
    let approver_2_sig = reader.read_exact::<64>()?;
    reader.ensure_empty()?;

    Ok(SealPayload {
        statement,
        approver_1_pubkey,
        approver_1_sig,
        approver_2_pubkey,
        approver_2_sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Direction;

    fn sample_entry() -> Entry {
        Entry {
            id: "ent_01".into(),
            date: "2026-08-15".into(),
            description: "Consulting Payment".into(),
            created_at: "2026-08-15T12:00:00Z".into(),
            lines: vec![
                Line::new(
                    "l1",
                    "acc_bank",
                    Direction::Debit,
                    500000,
                    Some("Wire transfer".into()),
                )
                .unwrap(),
                Line::new("l2", "acc_revenue", Direction::Credit, 500000, None).unwrap(),
            ],
        }
    }

    fn sample_statement() -> SealStatement {
        SealStatement {
            version: CANONICAL_VERSION_V1,
            entity: "Acme Trading Pvt Ltd".into(),
            period_start: "2026-08-01".into(),
            period_end: "2026-08-31".into(),
            entry_count: 16,
            total_debits_minor: 12185000,
            total_credits_minor: 12185000,
            ledger_root: [0x42; 32],
        }
    }

    #[test]
    fn test_encoding_starts_with_version_byte() {
        let entry = sample_entry();
        let bytes = encode_entry(&entry);
        assert_eq!(bytes[0], CANONICAL_VERSION_V1);
    }

    #[test]
    fn test_length_prefixing_prevents_concatenation_collisions() {
        let mut buf1 = Vec::new();
        write_str(&mut buf1, "ab");
        write_str(&mut buf1, "c");

        let mut buf2 = Vec::new();
        write_str(&mut buf2, "a");
        write_str(&mut buf2, "bc");

        assert_ne!(
            buf1, buf2,
            "Length prefixing must prevent concatenation collision"
        );
    }

    #[test]
    fn test_line_order_invariance() {
        let entry1 = sample_entry();
        let mut entry2 = sample_entry();

        // Swap order of lines in entry2
        entry2.lines.reverse();

        let bytes1 = encode_entry(&entry1);
        let bytes2 = encode_entry(&entry2);

        assert_eq!(
            bytes1, bytes2,
            "Entries with permuted lines must encode to identical bytes"
        );
    }

    #[test]
    fn test_any_field_change_changes_encoding() {
        let original = sample_entry();
        let original_bytes = encode_entry(&original);

        // 1. Change id
        let mut mod_id = original.clone();
        mod_id.id = "ent_02".into();
        assert_ne!(original_bytes, encode_entry(&mod_id));

        // 2. Change date
        let mut mod_date = original.clone();
        mod_date.date = "2026-08-16".into();
        assert_ne!(original_bytes, encode_entry(&mod_date));

        // 3. Change description
        let mut mod_desc = original.clone();
        mod_desc.description = "Consulting Payment.".into();
        assert_ne!(original_bytes, encode_entry(&mod_desc));

        // 4. Change amount
        let mut mod_amt = original.clone();
        mod_amt.lines[0].amount_minor = 500001;
        mod_amt.lines[1].amount_minor = 500001;
        assert_ne!(original_bytes, encode_entry(&mod_amt));
    }

    #[test]
    fn test_statement_encoding() {
        let statement = sample_statement();
        let bytes = encode_statement(&statement);
        assert_eq!(bytes[0], CANONICAL_VERSION_V1);
        assert_eq!(&bytes[bytes.len() - 32..], &[0x42; 32]);
    }

    #[test]
    fn test_statement_encoding_and_decoding_roundtrip() {
        let statement = sample_statement();
        let encoded = encode_statement(&statement);
        let decoded = decode_statement(&encoded).expect("statement roundtrip decode");
        assert_eq!(statement, decoded);
    }

    #[test]
    fn test_seal_payload_encoding_and_decoding_roundtrip() {
        let payload = SealPayload {
            statement: sample_statement(),
            approver_1_pubkey: [0x02; 33],
            approver_1_sig: [0x11; 64],
            approver_2_pubkey: [0x03; 33],
            approver_2_sig: [0x22; 64],
        };

        let encoded = encode_seal_payload(&payload);
        // Size must be compact: statement (~100 bytes) + 33 + 64 + 33 + 64 = ~294 bytes
        assert!(encoded.len() < 400);
        assert!(encoded.len() > 200);

        let decoded = decode_seal_payload(&encoded).expect("seal payload roundtrip decode");
        assert_eq!(payload, decoded);
    }

    #[test]
    fn test_decode_seal_payload_rejects_truncated_bytes() {
        let payload = SealPayload {
            statement: sample_statement(),
            approver_1_pubkey: [0x02; 33],
            approver_1_sig: [0x11; 64],
            approver_2_pubkey: [0x03; 33],
            approver_2_sig: [0x22; 64],
        };

        let encoded = encode_seal_payload(&payload);
        // Truncate last 10 bytes
        let truncated = &encoded[..encoded.len() - 10];
        let res = decode_seal_payload(truncated);
        assert!(res.is_err());
    }

    #[test]
    fn test_decode_seal_payload_rejects_trailing_junk() {
        let payload = SealPayload {
            statement: sample_statement(),
            approver_1_pubkey: [0x02; 33],
            approver_1_sig: [0x11; 64],
            approver_2_pubkey: [0x03; 33],
            approver_2_sig: [0x22; 64],
        };

        let mut corrupted = encode_seal_payload(&payload);
        corrupted.push(0xff); // trailing extra byte
        let res = decode_seal_payload(&corrupted);
        assert!(res.is_err());
    }
}
