//! Hashing functions for journal entries and seal statements.
//!
//! Provides:
//! - `hash_entry`: Validates entry invariants and computes SHA-256 over its canonical encoding.
//! - `statement_hash`: Computes SHA-256 over the canonical encoding of a `SealStatement`.
//!   This 32-byte digest is what approvers sign with secp256k1 via Privy.

use crate::encode::{encode_entry, encode_statement};
use crate::error::CoreError;
use crate::types::{Entry, SealStatement};
use sha2::{Digest, Sha256};

/// Computes the 32-byte SHA-256 cryptographic leaf hash of a validated journal entry.
///
/// Returns `Err(CoreError)` if the entry violates accounting invariants
/// (e.g. fewer than 2 lines, unbalanced debits/credits, zero amount lines).
pub fn hash_entry(entry: &Entry) -> Result<[u8; 32], CoreError> {
    entry.validate()?;
    Ok(hash_entry_unchecked(entry))
}

/// Computes the 32-byte SHA-256 cryptographic leaf hash of a journal entry without validation.
///
/// Useful internally or during verification when inspecting potentially tampered entries.
pub fn hash_entry_unchecked(entry: &Entry) -> [u8; 32] {
    let canonical_bytes = encode_entry(entry);
    let digest = Sha256::digest(&canonical_bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

/// Computes the 32-byte SHA-256 digest of a `SealStatement`.
///
/// This is the prehash passed to Privy's `secp256k1_sign` RPC and verified on Hedera.
pub fn statement_hash(statement: &SealStatement) -> [u8; 32] {
    let canonical_bytes = encode_statement(statement);
    let digest = Sha256::digest(&canonical_bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::CANONICAL_VERSION_V1;
    use crate::types::{Direction, Line};

    fn sample_valid_entry() -> Entry {
        Entry {
            id: "ent_hash_01".into(),
            date: "2026-08-20".into(),
            description: "Server hosting payment".into(),
            created_at: "2026-08-20T14:30:00Z".into(),
            lines: vec![
                Line::new(
                    "l1",
                    "acc_hosting_expense",
                    Direction::Debit,
                    25000,
                    Some("AWS Invoice".into()),
                )
                .unwrap(),
                Line::new("l2", "acc_bank_checking", Direction::Credit, 25000, None).unwrap(),
            ],
        }
    }

    #[test]
    fn test_same_entry_produces_identical_hash() {
        let entry1 = sample_valid_entry();
        let entry2 = sample_valid_entry();

        let hash1 = hash_entry(&entry1).expect("hash entry 1");
        let hash2 = hash_entry(&entry2).expect("hash entry 2");

        assert_eq!(
            hash1, hash2,
            "Identical entries must yield identical hashes"
        );
    }

    #[test]
    fn test_unbalanced_entry_fails_hash() {
        let bad_entry = Entry {
            id: "ent_bad".into(),
            date: "2026-08-20".into(),
            description: "Unbalanced".into(),
            created_at: "2026-08-20T14:30:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_exp", Direction::Debit, 1000, None).unwrap(),
                Line::new("l2", "acc_cash", Direction::Credit, 999, None).unwrap(),
            ],
        };

        let err = hash_entry(&bad_entry).unwrap_err();
        assert_eq!(
            err,
            CoreError::UnbalancedEntry {
                entry_id: "ent_bad".into(),
                debits_minor: 1000,
                credits_minor: 999,
            }
        );
    }

    #[test]
    fn test_permuted_lines_yield_identical_hash() {
        let entry1 = sample_valid_entry();
        let mut entry2 = sample_valid_entry();
        entry2.lines.reverse();

        let hash1 = hash_entry(&entry1).unwrap();
        let hash2 = hash_entry(&entry2).unwrap();

        assert_eq!(
            hash1, hash2,
            "Permuted lines must produce identical hash due to canonical line sorting"
        );
    }

    #[test]
    fn test_any_field_change_changes_hash() {
        let base = sample_valid_entry();
        let base_hash = hash_entry(&base).unwrap();

        // Alter description
        let mut altered_desc = base.clone();
        altered_desc.description = "Server hosting payment (updated)".into();
        assert_ne!(base_hash, hash_entry(&altered_desc).unwrap());

        // Alter date
        let mut altered_date = base.clone();
        altered_date.date = "2026-08-21".into();
        assert_ne!(base_hash, hash_entry(&altered_date).unwrap());

        // Alter created_at
        let mut altered_created = base.clone();
        altered_created.created_at = "2026-08-20T14:30:01Z".into();
        assert_ne!(base_hash, hash_entry(&altered_created).unwrap());

        // Alter amount by 1 minor unit (keeping balanced)
        let mut altered_amount = base.clone();
        altered_amount.lines[0].amount_minor = 25001;
        altered_amount.lines[1].amount_minor = 25001;
        assert_ne!(base_hash, hash_entry(&altered_amount).unwrap());
    }

    #[test]
    fn test_statement_hash_determinism() {
        let stmt1 = SealStatement {
            version: CANONICAL_VERSION_V1,
            entity: "Acme Corp".into(),
            period_start: "2026-08-01".into(),
            period_end: "2026-08-31".into(),
            entry_count: 50,
            total_debits_minor: 500000,
            total_credits_minor: 500000,
            ledger_root: [0x11; 32],
        };

        let stmt2 = stmt1.clone();
        assert_eq!(statement_hash(&stmt1), statement_hash(&stmt2));

        let mut altered_root_stmt = stmt1.clone();
        altered_root_stmt.ledger_root = [0x22; 32];
        assert_ne!(statement_hash(&stmt1), statement_hash(&altered_root_stmt));
    }
}
