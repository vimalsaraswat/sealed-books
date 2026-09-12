//! Domain types and invariants for Sealed Books Core.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Direction of a double-entry ledger line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Debit,
    Credit,
}

impl Direction {
    /// Canonical single-byte representation.
    pub const fn as_byte(&self) -> u8 {
        match self {
            Direction::Debit => 0x01,
            Direction::Credit => 0x02,
        }
    }
}

/// An individual debit or credit line in a double-entry journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    pub id: String,
    pub account_id: String,
    pub direction: Direction,
    /// Amount in integer minor units (e.g. 10000 = $100.00 / ₹100.00). Must be > 0.
    pub amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Line {
    pub fn new(
        id: impl Into<String>,
        account_id: impl Into<String>,
        direction: Direction,
        amount_minor: u64,
        description: Option<String>,
    ) -> Result<Self, CoreError> {
        let id = id.into();
        if amount_minor == 0 {
            return Err(CoreError::ZeroAmountLine { line_id: id });
        }
        Ok(Self {
            id,
            account_id: account_id.into(),
            direction,
            amount_minor,
            description,
        })
    }
}

/// A balanced double-entry journal transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    /// Calendar date in ISO 8601 format: YYYY-MM-DD.
    pub date: String,
    pub description: String,
    /// Timestamp in UTC RFC 3339 format.
    pub created_at: String,
    pub lines: Vec<Line>,
}

impl Entry {
    /// Validates the accounting invariants of this journal entry:
    /// 1. Must contain at least 2 lines.
    /// 2. Every line must have an amount > 0.
    /// 3. Total debits must equal total credits exactly (double-entry equation).
    /// 4. Date must follow YYYY-MM-DD format.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.lines.len() < 2 {
            return Err(CoreError::InvalidLineCount {
                entry_id: self.id.clone(),
                line_count: self.lines.len(),
            });
        }

        let mut total_debits: u64 = 0;
        let mut total_credits: u64 = 0;

        for line in &self.lines {
            if line.amount_minor == 0 {
                return Err(CoreError::ZeroAmountLine {
                    line_id: line.id.clone(),
                });
            }
            match line.direction {
                Direction::Debit => {
                    total_debits =
                        total_debits.checked_add(line.amount_minor).ok_or_else(|| {
                            CoreError::SerializationError("Debit minor units overflowed u64".into())
                        })?;
                }
                Direction::Credit => {
                    total_credits =
                        total_credits
                            .checked_add(line.amount_minor)
                            .ok_or_else(|| {
                                CoreError::SerializationError(
                                    "Credit minor units overflowed u64".into(),
                                )
                            })?;
                }
            }
        }

        if total_debits != total_credits {
            return Err(CoreError::UnbalancedEntry {
                entry_id: self.id.clone(),
                debits_minor: total_debits,
                credits_minor: total_credits,
            });
        }

        Self::validate_date_format(&self.date)?;

        Ok(())
    }

    /// Sum of all debit lines in this entry.
    pub fn total_debits(&self) -> u64 {
        self.lines
            .iter()
            .filter(|l| l.direction == Direction::Debit)
            .map(|l| l.amount_minor)
            .sum()
    }

    /// Sum of all credit lines in this entry.
    pub fn total_credits(&self) -> u64 {
        self.lines
            .iter()
            .filter(|l| l.direction == Direction::Credit)
            .map(|l| l.amount_minor)
            .sum()
    }

    /// Validates YYYY-MM-DD date string.
    pub fn validate_date_format(date_str: &str) -> Result<(), CoreError> {
        if date_str.len() != 10 {
            return Err(CoreError::InvalidDateFormat(date_str.to_string()));
        }
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() != 3 {
            return Err(CoreError::InvalidDateFormat(date_str.to_string()));
        }
        let year: u32 = parts[0]
            .parse()
            .map_err(|_| CoreError::InvalidDateFormat(date_str.to_string()))?;
        let month: u32 = parts[1]
            .parse()
            .map_err(|_| CoreError::InvalidDateFormat(date_str.to_string()))?;
        let day: u32 = parts[2]
            .parse()
            .map_err(|_| CoreError::InvalidDateFormat(date_str.to_string()))?;

        if !(1000..=9999).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day)
        {
            return Err(CoreError::InvalidDateFormat(date_str.to_string()));
        }
        Ok(())
    }
}

/// An accounting period with inclusive calendar boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    pub id: String,
    /// Legal entity name (e.g. "Acme Trading Pvt Ltd")
    pub entity: String,
    /// Period start date: YYYY-MM-DD
    pub start_date: String,
    /// Period end date: YYYY-MM-DD
    pub end_date: String,
}

impl Period {
    pub fn validate(&self) -> Result<(), CoreError> {
        Entry::validate_date_format(&self.start_date)?;
        Entry::validate_date_format(&self.end_date)?;
        if self.start_date > self.end_date {
            return Err(CoreError::InvalidPeriodRange {
                start: self.start_date.clone(),
                end: self.end_date.clone(),
            });
        }
        Ok(())
    }

    /// Checks if a date string falls inclusively within this period.
    pub fn contains_date(&self, date_str: &str) -> Result<(), CoreError> {
        Entry::validate_date_format(date_str)?;
        if date_str < self.start_date.as_str() || date_str > self.end_date.as_str() {
            return Err(CoreError::EntryOutOfPeriodRange {
                entry_id: String::new(),
                entry_date: date_str.to_string(),
                period_start: self.start_date.clone(),
                period_end: self.end_date.clone(),
            });
        }
        Ok(())
    }
}

/// The seal statement that binds the period, entry totals, and Merkle root.
/// This statement is what human approvers read and sign with their keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealStatement {
    pub version: u8,
    pub entity: String,
    pub period_start: String,
    pub period_end: String,
    pub entry_count: u64,
    pub total_debits_minor: u64,
    pub total_credits_minor: u64,
    #[serde(with = "hex_32")]
    pub ledger_root: [u8; 32],
}

impl SealStatement {
    /// Formats the statement into human-readable text as prescribed by PLAN.md Section 3.2.
    pub fn to_human_readable(&self) -> String {
        format!(
            "Sealed Books — Period Close\n\
             Entity:        {}\n\
             Period:        {} -> {}\n\
             Entries:       {}\n\
             Total debits:  {} (minor units)\n\
             Total credits: {} (minor units)\n\
             Ledger root:   {}",
            self.entity,
            self.period_start,
            self.period_end,
            self.entry_count,
            self.total_debits_minor,
            self.total_credits_minor,
            hex::encode(self.ledger_root)
        )
    }
}

/// The published on-chain consensus record combining the period close statement
/// and the two distinct approver signatures required for sealing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealPayload {
    pub statement: SealStatement,
    #[serde(with = "hex_33")]
    pub approver_1_pubkey: [u8; 33],
    #[serde(with = "hex_64")]
    pub approver_1_sig: [u8; 64],
    #[serde(with = "hex_33")]
    pub approver_2_pubkey: [u8; 33],
    #[serde(with = "hex_64")]
    pub approver_2_sig: [u8; 64],
}

/// Result of verifying a set of entries against a sealed statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    /// The ledger entries match the sealed statement and Merkle root byte-for-byte.
    Intact,

    /// An entry was altered after sealing.
    EntryTampered {
        entry_id: String,
        expected_hash: Option<[u8; 32]>,
        actual_hash: [u8; 32],
        reason: String,
    },

    /// An entry was deleted that was present when the seal was created.
    EntryMissing { entry_id: String },

    /// A new entry was injected into this period after it was sealed.
    EntryInjected { entry_id: String },

    /// The aggregate totals do not match the statement.
    TotalsMismatch {
        statement_debits: u64,
        actual_debits: u64,
        statement_credits: u64,
        actual_credits: u64,
    },

    /// Entry count does not match the statement.
    EntryCountMismatch {
        statement_count: u64,
        actual_count: u64,
    },

    /// The calculated Merkle root does not match the sealed statement root.
    RootMismatch {
        expected_root: [u8; 32],
        actual_root: [u8; 32],
    },
}

mod hex_32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let clean = s.trim_start_matches("0x");
        let vec = hex::decode(clean).map_err(serde::de::Error::custom)?;
        if vec.len() != 32 {
            return Err(serde::de::Error::custom(format!(
                "expected 32 bytes for hex hash, got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

mod hex_33 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 33], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 33], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let clean = s.trim_start_matches("0x");
        let vec = hex::decode(clean).map_err(serde::de::Error::custom)?;
        if vec.len() != 33 {
            return Err(serde::de::Error::custom(format!(
                "expected 33 bytes for compressed public key, got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 33];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

mod hex_64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let clean = s.trim_start_matches("0x");
        let vec = hex::decode(clean).map_err(serde::de::Error::custom)?;
        if vec.len() != 64 {
            return Err(serde::de::Error::custom(format!(
                "expected 64 bytes for signature (r||s), got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_entry_validation() {
        let entry = Entry {
            id: "entry_1".into(),
            date: "2026-08-15".into(),
            description: "Office Supplies".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_supplies", Direction::Debit, 5000, None).unwrap(),
                Line::new("l2", "acc_cash", Direction::Credit, 5000, None).unwrap(),
            ],
        };

        assert!(entry.validate().is_ok());
        assert_eq!(entry.total_debits(), 5000);
        assert_eq!(entry.total_credits(), 5000);
    }

    #[test]
    fn test_unbalanced_entry_rejected() {
        let entry = Entry {
            id: "entry_bad".into(),
            date: "2026-08-15".into(),
            description: "Unbalanced purchase".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![
                Line::new("l1", "acc_supplies", Direction::Debit, 5000, None).unwrap(),
                Line::new("l2", "acc_cash", Direction::Credit, 4999, None).unwrap(),
            ],
        };

        let err = entry.validate().unwrap_err();
        assert_eq!(
            err,
            CoreError::UnbalancedEntry {
                entry_id: "entry_bad".into(),
                debits_minor: 5000,
                credits_minor: 4999,
            }
        );
    }

    #[test]
    fn test_zero_amount_line_rejected() {
        let err = Line::new("l1", "acc_cash", Direction::Debit, 0, None).unwrap_err();
        assert_eq!(
            err,
            CoreError::ZeroAmountLine {
                line_id: "l1".into()
            }
        );
    }

    #[test]
    fn test_fewer_than_two_lines_rejected() {
        let entry = Entry {
            id: "entry_single".into(),
            date: "2026-08-15".into(),
            description: "Single leg".into(),
            created_at: "2026-08-15T10:00:00Z".into(),
            lines: vec![Line::new("l1", "acc_cash", Direction::Debit, 1000, None).unwrap()],
        };

        let err = entry.validate().unwrap_err();
        assert_eq!(
            err,
            CoreError::InvalidLineCount {
                entry_id: "entry_single".into(),
                line_count: 1,
            }
        );
    }

    #[test]
    fn test_period_validation() {
        let good_period = Period {
            id: "p1".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
        };
        assert!(good_period.validate().is_ok());

        let bad_period = Period {
            id: "p2".into(),
            entity: "Acme Corp".into(),
            start_date: "2026-09-01".into(),
            end_date: "2026-08-31".into(),
        };
        assert!(bad_period.validate().is_err());
    }

    #[test]
    fn test_human_readable_statement() {
        let statement = SealStatement {
            version: 1,
            entity: "Acme Trading Pvt Ltd".into(),
            period_start: "2026-08-01".into(),
            period_end: "2026-08-31".into(),
            entry_count: 1247,
            total_debits_minor: 8432150,
            total_credits_minor: 8432150,
            ledger_root: [0xaa; 32],
        };

        let text = statement.to_human_readable();
        assert!(text.contains("Acme Trading Pvt Ltd"));
        assert!(text.contains("2026-08-01 -> 2026-08-31"));
        assert!(text.contains("Entries:       1247"));
        assert!(text.contains("Total debits:  8432150 (minor units)"));
        assert!(text.contains("Total credits: 8432150 (minor units)"));
        assert!(text.contains("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    }
}
