//! Verification and tamper detection engine for Sealed Books.
//!
//! Provides:
//! - `verify`: Checks a set of entries against a `SealStatement`.
//! - `verify_with_baseline`: Pinpoints the exact entry that was altered, deleted,
//!   or injected by comparing against the original sealed entries.
//! - `verify_with_known_hashes`: Pinpoints the exact entry using known sealed leaf hashes.

use crate::hash::{hash_entry, hash_entry_unchecked};
use crate::merkle::{merkle_root, sort_entries_canonically};
use crate::types::{Direction, Entry, SealStatement, VerifyResult};
use std::collections::{HashMap, HashSet};

/// Verifies whether `entries` match the published `SealStatement`.
///
/// Performs the following checks:
/// 1. Invariant integrity: Checks if any entry is internally unbalanced or malformed.
/// 2. Period boundaries: Checks if any entry has a date outside `period_start..=period_end`.
/// 3. Entry count: Compares against `statement.entry_count`.
/// 4. Aggregate totals: Compares total debits and credits against `statement`.
/// 5. Merkle root: Reconstructs the canonical Merkle tree and matches against `statement.ledger_root`.
pub fn verify(statement: &SealStatement, entries: &[Entry]) -> VerifyResult {
    let mut total_debits: u64 = 0;
    let mut total_credits: u64 = 0;

    for entry in entries {
        // 1. Check if entry satisfies double-entry accounting invariants
        if let Err(err) = entry.validate() {
            return VerifyResult::EntryTampered {
                entry_id: entry.id.clone(),
                expected_hash: None,
                actual_hash: hash_entry_unchecked(entry),
                reason: err.to_string(),
            };
        }

        // 2. Check if entry is outside the sealed period range
        if entry.date < statement.period_start || entry.date > statement.period_end {
            return VerifyResult::EntryInjected {
                entry_id: entry.id.clone(),
            };
        }

        for line in &entry.lines {
            match line.direction {
                Direction::Debit => {
                    total_debits = total_debits.saturating_add(line.amount_minor);
                }
                Direction::Credit => {
                    total_credits = total_credits.saturating_add(line.amount_minor);
                }
            }
        }
    }

    // 3. Check entry count
    if entries.len() as u64 != statement.entry_count {
        return VerifyResult::EntryCountMismatch {
            statement_count: statement.entry_count,
            actual_count: entries.len() as u64,
        };
    }

    // 4. Check aggregate totals
    if total_debits != statement.total_debits_minor
        || total_credits != statement.total_credits_minor
    {
        return VerifyResult::TotalsMismatch {
            statement_debits: statement.total_debits_minor,
            actual_debits: total_debits,
            statement_credits: statement.total_credits_minor,
            actual_credits: total_credits,
        };
    }

    // 5. Reconstruct canonical Merkle root
    let sorted_entries = sort_entries_canonically(entries);
    let mut leaf_hashes = Vec::with_capacity(sorted_entries.len());
    for entry in sorted_entries {
        match hash_entry(entry) {
            Ok(h) => leaf_hashes.push(h),
            Err(err) => {
                return VerifyResult::EntryTampered {
                    entry_id: entry.id.clone(),
                    expected_hash: None,
                    actual_hash: hash_entry_unchecked(entry),
                    reason: err.to_string(),
                };
            }
        }
    }

    let reconstructed_root = merkle_root(&leaf_hashes);
    if reconstructed_root == statement.ledger_root {
        VerifyResult::Intact
    } else {
        VerifyResult::RootMismatch {
            expected_root: statement.ledger_root,
            actual_root: reconstructed_root,
        }
    }
}

/// Verifies current entries against a `SealStatement`, using baseline entries
/// to pinpoint the exact entry that was modified, added, or deleted.
pub fn verify_with_baseline(
    statement: &SealStatement,
    baseline_entries: &[Entry],
    current_entries: &[Entry],
) -> VerifyResult {
    let baseline_hashes: Vec<(String, [u8; 32])> = baseline_entries
        .iter()
        .map(|e| (e.id.clone(), hash_entry_unchecked(e)))
        .collect();

    verify_with_known_hashes(statement, &baseline_hashes, current_entries)
}

/// Verifies current entries against a `SealStatement` using known sealed leaf hashes.
///
/// If any entry was altered (even if amounts were kept balanced and totals didn't change),
/// this function detects the hash difference and names the exact entry ID that moved.
pub fn verify_with_known_hashes(
    statement: &SealStatement,
    baseline_hashes: &[(String, [u8; 32])],
    current_entries: &[Entry],
) -> VerifyResult {
    let baseline_map: HashMap<&str, [u8; 32]> = baseline_hashes
        .iter()
        .map(|(id, h)| (id.as_str(), *h))
        .collect();

    let current_ids: HashSet<&str> = current_entries.iter().map(|e| e.id.as_str()).collect();

    // 1. Check for missing entries (present in baseline, missing from current)
    for (id, _) in baseline_hashes {
        if !current_ids.contains(id.as_str()) {
            return VerifyResult::EntryMissing {
                entry_id: id.clone(),
            };
        }
    }

    // 2. Check for injected entries (present in current, missing from baseline)
    for entry in current_entries {
        if !baseline_map.contains_key(entry.id.as_str()) {
            return VerifyResult::EntryInjected {
                entry_id: entry.id.clone(),
            };
        }
    }

    // 3. Check for modified entries (hash mismatch)
    for entry in current_entries {
        let current_hash = hash_entry_unchecked(entry);
        if let Some(&expected_hash) = baseline_map.get(entry.id.as_str()) {
            if current_hash != expected_hash {
                let reason = if let Err(err) = entry.validate() {
                    err.to_string()
                } else {
                    "Entry content was altered after sealing".into()
                };

                return VerifyResult::EntryTampered {
                    entry_id: entry.id.clone(),
                    expected_hash: Some(expected_hash),
                    actual_hash: current_hash,
                    reason,
                };
            }
        }
    }

    // 4. Standard validation as safety net (checks statement totals & roots)
    verify(statement, current_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::build_statement;
    use crate::types::{Line, Period};

    fn make_entry(id: &str, date: &str, amount: u64) -> Entry {
        Entry {
            id: id.into(),
            date: date.into(),
            description: format!("Transaction {id}"),
            created_at: format!("{date}T10:00:00Z"),
            lines: vec![
                Line::new(format!("{id}_d"), "acc_exp", Direction::Debit, amount, None).unwrap(),
                Line::new(
                    format!("{id}_c"),
                    "acc_cash",
                    Direction::Credit,
                    amount,
                    None,
                )
                .unwrap(),
            ],
        }
    }

    fn sample_period() -> Period {
        Period {
            id: "p_aug".into(),
            entity: "Acme Trading Pvt Ltd".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
        }
    }

    #[test]
    fn test_verify_unmodified_period_is_intact() {
        let period = sample_period();
        let entries: Vec<Entry> = (1..=20)
            .map(|i| make_entry(&format!("e_{i:02}"), "2026-08-15", 5000 + i * 10))
            .collect();

        let statement = build_statement(&period, &entries).unwrap();
        let result = verify(&statement, &entries);

        assert_eq!(result, VerifyResult::Intact);
    }

    #[test]
    fn test_verify_unbalanced_tampered_entry_identified() {
        let period = sample_period();
        let mut entries: Vec<Entry> = (1..=20)
            .map(|i| make_entry(&format!("e_{i:02}"), "2026-08-15", 5000 + i * 10))
            .collect();

        let statement = build_statement(&period, &entries).unwrap();

        // Tamper with entry e_07: alter debit line only by 1 cent
        entries[6].lines[0].amount_minor += 1;

        let result = verify(&statement, &entries);

        match result {
            VerifyResult::EntryTampered {
                entry_id, reason, ..
            } => {
                assert_eq!(entry_id, "e_07");
                assert!(reason.contains("unbalanced"));
            }
            other => panic!("Expected EntryTampered for e_07, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_with_baseline_names_subtly_altered_entry() {
        let period = sample_period();
        let original_entries: Vec<Entry> = (1..=50)
            .map(|i| make_entry(&format!("e_{i:02}"), "2026-08-15", 10000))
            .collect();

        let statement = build_statement(&period, &original_entries).unwrap();

        // Subtly tamper with entry e_42 description only (amounts stay identical, totals match)
        let mut current_entries = original_entries.clone();
        current_entries[41].description = "Altered description (tampered)".into();

        let result = verify_with_baseline(&statement, &original_entries, &current_entries);

        match result {
            VerifyResult::EntryTampered {
                entry_id,
                expected_hash,
                actual_hash,
                ..
            } => {
                assert_eq!(entry_id, "e_42");
                assert!(expected_hash.is_some());
                assert_ne!(expected_hash.unwrap(), actual_hash);
            }
            other => panic!("Expected EntryTampered naming e_42, got {other:?}"),
        }
    }

    #[test]
    fn test_verify_detects_deleted_entry() {
        let period = sample_period();
        let original_entries: Vec<Entry> = (1..=10)
            .map(|i| make_entry(&format!("e_{i:02}"), "2026-08-15", 1000))
            .collect();

        let statement = build_statement(&period, &original_entries).unwrap();

        // Delete entry e_05
        let mut current_entries = original_entries.clone();
        current_entries.retain(|e| e.id != "e_05");

        let result = verify_with_baseline(&statement, &original_entries, &current_entries);
        assert_eq!(
            result,
            VerifyResult::EntryMissing {
                entry_id: "e_05".into()
            }
        );
    }

    #[test]
    fn test_verify_detects_injected_entry() {
        let period = sample_period();
        let original_entries: Vec<Entry> = (1..=10)
            .map(|i| make_entry(&format!("e_{i:02}"), "2026-08-15", 1000))
            .collect();

        let statement = build_statement(&period, &original_entries).unwrap();

        // Inject new entry e_99
        let mut current_entries = original_entries.clone();
        current_entries.push(make_entry("e_99", "2026-08-20", 500));

        let result = verify_with_baseline(&statement, &original_entries, &current_entries);
        assert_eq!(
            result,
            VerifyResult::EntryInjected {
                entry_id: "e_99".into()
            }
        );
    }
}
