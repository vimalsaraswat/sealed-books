//! Independent verification engine for Sealed Books.
//!
//! Fetches on-chain consensus seal payloads from the public Hedera mirror node
//! via plain HTTP, cryptographically verifies approver signatures, recomputes
//! the Merkle root and totals from SQLite as it stands right now, and either
//! confirms intact integrity (green) or pinpoints the altered entry (red).

use k256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::crypto;
use crate::db::Database;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use crate::db::repository::seal::SealRecord;
use crate::mirror::MirrorClient;
use sealed_books_core::hash::statement_hash;
use sealed_books_core::merkle::merkle_root;
use sealed_books_core::types::{Entry, VerifyResult};
use sealed_books_core::{
    decode_seal_payload, hash_entry_unchecked, sort_entries_canonically, verify,
};

/// High-level audit status of the period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// Database matches on-chain consensus seal byte-for-byte; all signatures valid.
    Verified,
    /// Database records have been modified, injected, or deleted post-sealing.
    Tampered,
    /// On-chain approver signatures failed cryptographic verification.
    SignatureInvalid,
}

/// Details of a verified approver who signed the close statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedApprover {
    pub compressed_pubkey: String,
    pub eth_address: String,
    pub signature_valid: bool,
}

/// Summary of the statement published to Hedera Consensus Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealStatementSummary {
    pub entity: String,
    pub period_start: String,
    pub period_end: String,
    pub entry_count: u64,
    pub total_debits_minor: u64,
    pub total_credits_minor: u64,
    pub ledger_root: String,
    pub statement_hash: String,
}

/// Summary of current database state computed right now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSummary {
    pub entry_count: u64,
    pub total_debits_minor: u64,
    pub total_credits_minor: u64,
    pub ledger_root: String,
}

/// Pinpoints the exact discrepancy between on-chain proof and database state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscrepancyDetail {
    pub kind: String,
    pub offending_entry_id: Option<String>,
    pub message: String,
    pub expected_hash: Option<String>,
    pub actual_hash: Option<String>,
}

/// Complete verification audit report returned by the verification engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub period_id: String,
    pub status: VerificationStatus,
    pub is_intact: bool,
    pub topic_id: String,
    pub sequence_number: u64,
    pub consensus_timestamp: String,
    pub on_chain_statement: SealStatementSummary,
    pub database_summary: DatabaseSummary,
    pub approver_1: VerifiedApprover,
    pub approver_2: VerifiedApprover,
    pub discrepancy: Option<DiscrepancyDetail>,
    pub human_readable_statement: String,
    pub hashscan_url: String,
}

/// Runs full independent verification for an accounting period against Hedera mirror node.
pub async fn verify_period(
    db: &Database,
    mirror_client: &MirrorClient,
    period_id: &str,
) -> Result<VerificationReport, String> {
    // 1. Retrieve local seal record to get topic ID and sequence number
    let (_period, seal) = {
        let conn = db.conn();
        let p = PeriodRecord::find_by_id(conn, period_id)
            .await
            .map_err(|e| format!("Period '{period_id}' not found: {e}"))?;
        let s = SealRecord::find_by_period_id(conn, period_id)
            .await
            .map_err(|e| format!("No seal found for period '{period_id}': {e}"))?;
        (p, s)
    };

    let topic_id = seal
        .topic_id
        .ok_or_else(|| format!("Period '{period_id}' has not been published to Hedera"))?;
    let sequence_number = seal
        .sequence_number
        .ok_or_else(|| format!("Period '{period_id}' is missing sequence number"))?
        as u64;

    // 2. Fetch consensus message from mirror node via plain HTTP (no SDK)
    let (consensus_timestamp, payload_bytes) = mirror_client
        .fetch_message(&topic_id, sequence_number)
        .await
        .map_err(|e| format!("Mirror node fetch failed: {e}"))?;

    // 3. Decode binary SealPayload
    let payload = decode_seal_payload(&payload_bytes)
        .map_err(|e| format!("Failed to decode canonical binary seal payload: {e}"))?;

    // 4. Cryptographically verify both approver signatures against statement hash
    let st_hash = statement_hash(&payload.statement);
    let st_hash_hex = format!("0x{}", hex::encode(st_hash));

    let pk1_hex = format!("0x{}", hex::encode(payload.approver_1_pubkey));
    let pk2_hex = format!("0x{}", hex::encode(payload.approver_2_pubkey));

    let sig1_valid = crypto::verify_with_pubkey(
        &st_hash,
        &payload.approver_1_sig,
        &payload.approver_1_pubkey,
    )
    .is_ok();
    let sig2_valid = crypto::verify_with_pubkey(
        &st_hash,
        &payload.approver_2_sig,
        &payload.approver_2_pubkey,
    )
    .is_ok();

    let eth_addr_1 = VerifyingKey::from_sec1_bytes(&payload.approver_1_pubkey)
        .map(|vk| crypto::derive_eth_address(&vk))
        .unwrap_or_else(|_| "unknown".into());

    let eth_addr_2 = VerifyingKey::from_sec1_bytes(&payload.approver_2_pubkey)
        .map(|vk| crypto::derive_eth_address(&vk))
        .unwrap_or_else(|_| "unknown".into());

    let approver_1 = VerifiedApprover {
        compressed_pubkey: pk1_hex.clone(),
        eth_address: eth_addr_1,
        signature_valid: sig1_valid,
    };

    let approver_2 = VerifiedApprover {
        compressed_pubkey: pk2_hex.clone(),
        eth_address: eth_addr_2,
        signature_valid: sig2_valid,
    };

    let signatures_intact =
        sig1_valid && sig2_valid && !payload.approver_1_pubkey.eq(&payload.approver_2_pubkey);

    // 5. Query current database state right now
    let (current_entries, baseline_leaves) = {
        let conn = db.conn();
        let entries = Entry::find_by_period(conn, period_id)
            .await
            .map_err(|e| format!("Failed to load current entries from DB: {e}"))?;
        let leaves = SealRecord::get_leaves(conn, period_id)
            .await
            .unwrap_or_default();
        (entries, leaves)
    };

    // 6. Recompute current database summary
    let (current_debits, current_credits, current_root) =
        compute_database_totals_and_root(&current_entries);

    let database_summary = DatabaseSummary {
        entry_count: current_entries.len() as u64,
        total_debits_minor: current_debits,
        total_credits_minor: current_credits,
        ledger_root: format!("0x{}", hex::encode(current_root)),
    };

    let on_chain_statement = SealStatementSummary {
        entity: payload.statement.entity.clone(),
        period_start: payload.statement.period_start.clone(),
        period_end: payload.statement.period_end.clone(),
        entry_count: payload.statement.entry_count,
        total_debits_minor: payload.statement.total_debits_minor,
        total_credits_minor: payload.statement.total_credits_minor,
        ledger_root: format!("0x{}", hex::encode(payload.statement.ledger_root)),
        statement_hash: st_hash_hex,
    };

    let human_readable_statement = payload.statement.to_human_readable();
    let hashscan_url = if !consensus_timestamp.is_empty() {
        format!("https://hashscan.io/testnet/transaction/{consensus_timestamp}")
    } else {
        format!("https://hashscan.io/testnet/topic/{topic_id}")
    };

    // 7. Verify integrity against on-chain statement
    if !signatures_intact {
        return Ok(VerificationReport {
            period_id: period_id.to_string(),
            status: VerificationStatus::SignatureInvalid,
            is_intact: false,
            topic_id,
            sequence_number,
            consensus_timestamp,
            on_chain_statement,
            database_summary,
            approver_1,
            approver_2,
            discrepancy: Some(DiscrepancyDetail {
                kind: "signature_invalid".to_string(),
                offending_entry_id: None,
                message:
                    "One or more approver signatures on the consensus payload failed verification"
                        .into(),
                expected_hash: None,
                actual_hash: None,
            }),
            human_readable_statement,
            hashscan_url,
        });
    }

    let verify_result = if !baseline_leaves.is_empty() {
        sealed_books_core::verify::verify_with_known_hashes(
            &payload.statement,
            &baseline_leaves,
            &current_entries,
        )
    } else {
        verify(&payload.statement, &current_entries)
    };

    let (status, is_intact, discrepancy) = match verify_result {
        VerifyResult::Intact => (VerificationStatus::Verified, true, None),
        VerifyResult::EntryTampered {
            entry_id,
            expected_hash,
            actual_hash,
            reason,
        } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "entry_tampered".into(),
                offending_entry_id: Some(entry_id.clone()),
                message: format!("Entry '{entry_id}' was altered after period close: {reason}"),
                expected_hash: expected_hash.map(|h| format!("0x{}", hex::encode(h))),
                actual_hash: Some(format!("0x{}", hex::encode(actual_hash))),
            }),
        ),
        VerifyResult::EntryMissing { entry_id } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "entry_missing".into(),
                offending_entry_id: Some(entry_id.clone()),
                message: format!("Sealed entry '{entry_id}' was deleted from the database"),
                expected_hash: None,
                actual_hash: None,
            }),
        ),
        VerifyResult::EntryInjected { entry_id } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "entry_injected".into(),
                offending_entry_id: Some(entry_id.clone()),
                message: format!("Unsealed entry '{entry_id}' was injected into the sealed period"),
                expected_hash: None,
                actual_hash: None,
            }),
        ),
        VerifyResult::EntryCountMismatch {
            statement_count,
            actual_count,
        } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "entry_count_mismatch".into(),
                offending_entry_id: None,
                message: format!(
                    "Entry count mismatch: on-chain statement has {statement_count}, database has {actual_count}"
                ),
                expected_hash: None,
                actual_hash: None,
            }),
        ),
        VerifyResult::TotalsMismatch {
            statement_debits,
            actual_debits,
            statement_credits,
            actual_credits,
        } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "totals_mismatch".into(),
                offending_entry_id: None,
                message: format!(
                    "Financial totals altered: debits ({actual_debits} vs expected {statement_debits}), credits ({actual_credits} vs expected {statement_credits})"
                ),
                expected_hash: None,
                actual_hash: None,
            }),
        ),
        VerifyResult::RootMismatch {
            expected_root,
            actual_root,
        } => (
            VerificationStatus::Tampered,
            false,
            Some(DiscrepancyDetail {
                kind: "root_mismatch".into(),
                offending_entry_id: None,
                message: "Recomputed Merkle root does not match on-chain consensus root".into(),
                expected_hash: Some(format!("0x{}", hex::encode(expected_root))),
                actual_hash: Some(format!("0x{}", hex::encode(actual_root))),
            }),
        ),
    };

    Ok(VerificationReport {
        period_id: period_id.to_string(),
        status,
        is_intact,
        topic_id,
        sequence_number,
        consensus_timestamp,
        on_chain_statement,
        database_summary,
        approver_1,
        approver_2,
        discrepancy,
        human_readable_statement,
        hashscan_url,
    })
}

fn compute_database_totals_and_root(entries: &[Entry]) -> (u64, u64, [u8; 32]) {
    let mut total_debits = 0u64;
    let mut total_credits = 0u64;

    for entry in entries {
        for line in &entry.lines {
            match line.direction {
                sealed_books_core::types::Direction::Debit => {
                    total_debits = total_debits.saturating_add(line.amount_minor);
                }
                sealed_books_core::types::Direction::Credit => {
                    total_credits = total_credits.saturating_add(line.amount_minor);
                }
            }
        }
    }

    let sorted = sort_entries_canonically(entries);
    let leaves: Vec<[u8; 32]> = sorted.iter().map(|e| hash_entry_unchecked(e)).collect();
    let root = merkle_root(&leaves);

    (total_debits, total_credits, root)
}
