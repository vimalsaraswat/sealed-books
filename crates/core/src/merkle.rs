//! Merkle tree construction and period seal statement builder.
//!
//! Provides:
//! - `merkle_root`: Deterministic binary Merkle root calculation with odd-leaf duplication.
//! - `build_statement`: Validates period range and entries, sorts entries canonically,
//!   computes leaf hashes and the Merkle root, and generates a `SealStatement`.

use crate::encode::CANONICAL_VERSION_V1;
use crate::error::CoreError;
use crate::hash::hash_entry;
use crate::types::{Direction, Entry, Period, SealStatement};
use sha2::{Digest, Sha256};

/// Computes the 32-byte Merkle root over a slice of 32-byte leaf hashes.
///
/// Rules:
/// - 0 leaves: returns `[0u8; 32]`.
/// - 1 leaf: returns that leaf directly.
/// - Odd leaves at any layer: duplicates the last leaf to complete the pair (standard RFC 6962 / Bitcoin style).
/// - Intermediate node: `SHA-256(left_child || right_child)`.
pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_layer: Vec<[u8; 32]> = leaves.to_vec();

    while current_layer.len() > 1 {
        let mut next_layer = Vec::with_capacity(current_layer.len().div_ceil(2));

        for chunk in current_layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };

            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);
            let digest = hasher.finalize();

            let mut parent = [0u8; 32];
            parent.copy_from_slice(&digest);
            next_layer.push(parent);
        }

        current_layer = next_layer;
    }

    current_layer[0]
}

/// Sorts a slice of entry references into canonical order: `(date, created_at, id)`.
///
/// This guarantees that Merkle tree leaves and statement aggregates are deterministic
/// regardless of the order entries were stored or retrieved.
pub fn sort_entries_canonically<'a>(entries: &'a [Entry]) -> Vec<&'a Entry> {
    let mut sorted: Vec<&'a Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| (&a.date, &a.created_at, &a.id).cmp(&(&b.date, &b.created_at, &b.id)));
    sorted
}

/// Constructs a `SealStatement` for a given `Period` and set of `Entry` records.
///
/// Validates:
/// 1. Period range is valid (`start_date <= end_date`).
/// 2. Every entry satisfies accounting invariants (`entry.validate()`).
/// 3. Every entry falls within the period's date window (`start_date <= entry.date <= end_date`).
/// 4. Canonical entry sorting ensures byte-for-byte deterministic Merkle root.
pub fn build_statement(period: &Period, entries: &[Entry]) -> Result<SealStatement, CoreError> {
    period.validate()?;

    let mut total_debits: u64 = 0;
    let mut total_credits: u64 = 0;

    // Validate entries and check period boundaries
    for entry in entries {
        entry.validate()?;

        if entry.date < period.start_date || entry.date > period.end_date {
            return Err(CoreError::EntryOutOfPeriodRange {
                entry_id: entry.id.clone(),
                entry_date: entry.date.clone(),
                period_start: period.start_date.clone(),
                period_end: period.end_date.clone(),
            });
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

    // Sort entries canonically
    let sorted_entries = sort_entries_canonically(entries);

    // Compute leaf hashes
    let mut leaf_hashes = Vec::with_capacity(sorted_entries.len());
    for entry in sorted_entries {
        leaf_hashes.push(hash_entry(entry)?);
    }

    let root = merkle_root(&leaf_hashes);

    Ok(SealStatement {
        version: CANONICAL_VERSION_V1,
        entity: period.entity.clone(),
        period_start: period.start_date.clone(),
        period_end: period.end_date.clone(),
        entry_count: entries.len() as u64,
        total_debits_minor: total_debits,
        total_credits_minor: total_credits,
        ledger_root: root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Line;

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

    #[test]
    fn test_merkle_root_empty_and_single() {
        assert_eq!(merkle_root(&[]), [0u8; 32]);

        let single = [0xabu8; 32];
        assert_eq!(merkle_root(&[single]), single);
    }

    #[test]
    fn test_merkle_root_odd_leaves() {
        let l1 = [1u8; 32];
        let l2 = [2u8; 32];
        let l3 = [3u8; 32];

        // 3 leaves: pairs are (l1, l2) and (l3, l3)
        let root = merkle_root(&[l1, l2, l3]);
        assert_ne!(root, [0u8; 32]);
        assert_ne!(root, l1);
    }

    #[test]
    fn test_build_statement_determinism_regardless_of_entry_order() {
        let period = Period {
            id: "p_aug".into(),
            entity: "Acme Inc".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
        };

        let e1 = make_entry("e1", "2026-08-05", 1000);
        let e2 = make_entry("e2", "2026-08-10", 2000);
        let e3 = make_entry("e3", "2026-08-20", 3000);

        // Feed entries in order [e1, e2, e3]
        let stmt1 = build_statement(&period, &[e1.clone(), e2.clone(), e3.clone()]).unwrap();

        // Feed entries in permuted order [e3, e1, e2]
        let stmt2 = build_statement(&period, &[e3.clone(), e1.clone(), e2.clone()]).unwrap();

        assert_eq!(
            stmt1.ledger_root, stmt2.ledger_root,
            "Merkle root must be identical regardless of input order"
        );
        assert_eq!(stmt1.entry_count, 3);
        assert_eq!(stmt1.total_debits_minor, 6000);
        assert_eq!(stmt1.total_credits_minor, 6000);
    }

    #[test]
    fn test_entry_outside_period_rejected() {
        let period = Period {
            id: "p_aug".into(),
            entity: "Acme Inc".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
        };

        let e_out = make_entry("e_sep", "2026-09-01", 1000);
        let err = build_statement(&period, &[e_out]).unwrap_err();

        assert_eq!(
            err,
            CoreError::EntryOutOfPeriodRange {
                entry_id: "e_sep".into(),
                entry_date: "2026-09-01".into(),
                period_start: "2026-08-01".into(),
                period_end: "2026-08-31".into(),
            }
        );
    }

    #[test]
    fn test_changing_one_entry_changes_merkle_root() {
        let period = Period {
            id: "p_aug".into(),
            entity: "Acme Inc".into(),
            start_date: "2026-08-01".into(),
            end_date: "2026-08-31".into(),
        };

        // Create 100 entries
        let mut entries: Vec<Entry> = (1..=100)
            .map(|i| make_entry(&format!("e_{i:03}"), "2026-08-15", 10000 + i))
            .collect();

        let stmt_original = build_statement(&period, &entries).unwrap();

        // Tamper with entry #42: change amount by 1 cent
        entries[41].lines[0].amount_minor += 1;
        entries[41].lines[1].amount_minor += 1;

        let stmt_tampered = build_statement(&period, &entries).unwrap();

        assert_ne!(
            stmt_original.ledger_root, stmt_tampered.ledger_root,
            "Altering a single entry among 100 must alter the Merkle root"
        );
    }
}
