//! Period seal workflow API handlers (propose, approve, publish, verify).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::api::error::ApiError;
use crate::crypto;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use crate::db::repository::seal::SealRecord;
use crate::verify::VerificationReport;
use sealed_books_core::types::{Entry, SealPayload, SealStatement};
use sealed_books_core::{build_statement, encode_seal_payload, statement_hash};

/// Request payload for submitting an approval signature.
#[derive(Debug, Clone, Deserialize)]
pub struct ApproveRequest {
    /// 33-byte compressed secp256k1 public key as hex (optional if using wallet_id).
    pub approver_pubkey: Option<String>,
    /// 64-byte (r||s) or 65-byte (r||s||v) signature as hex (optional if using wallet_id).
    pub signature: Option<String>,
    /// Privy server wallet ID to sign via attached Privy policy.
    pub wallet_id: Option<String>,
}

/// Request payload for dispatching a proposed seal to an external auditor.
#[derive(Debug, Clone, Deserialize)]
pub struct DispatchSealRequest {
    pub auditor_id: Option<String>,
}

/// Request payload for an auditor rejecting a seal proposal with notes.
#[derive(Debug, Clone, Deserialize)]
pub struct RejectSealRequest {
    pub notes: String,
}

/// Response returned when a seal is proposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeResponse {
    pub period_id: String,
    pub statement: SealStatement,
    pub statement_hash: String,
    pub human_readable: String,
}

/// Response returned when an approval is recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveResponse {
    pub period_id: String,
    pub approver_pubkey: String,
    pub approvals_collected: usize,
    pub quorum_met: bool,
}

/// Response returned when a seal is published to Hedera Consensus Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub status: String,
    pub topic_id: String,
    pub sequence_number: u64,
    pub consensus_timestamp: String,
    pub transaction_id: String,
    pub hashscan_url: String,
    pub payload_bytes_len: usize,
}

/// Response representing the current seal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealStateResponse {
    pub period_id: String,
    pub period_status: String,
    pub seal: Option<SealRecord>,
    pub approvals_collected: usize,
    pub quorum_met: bool,
}

/// Creates the sub-router for period seal operations.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_seal_state))
        .route("/propose", post(propose_seal))
        .route("/dispatch", post(dispatch_seal))
        .route("/reject", post(reject_seal))
        .route("/approve", post(approve_seal))
        .route("/publish", post(publish_seal))
        .route("/verify", get(verify_seal))
}

/// Retrieves the current seal progress or sealed status for a period.
async fn get_seal_state(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<SealStateResponse>, ApiError> {
    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;
    let seal = SealRecord::find_by_period_id(&conn, &period_id).ok();

    let approvals_collected = match &seal {
        Some(s) => {
            let mut count = 0;
            if s.approver_1_pubkey.is_some() && s.approver_1_sig.is_some() {
                count += 1;
            }
            if s.approver_2_pubkey.is_some() && s.approver_2_sig.is_some() {
                count += 1;
            }
            count
        }
        None => 0,
    };

    let quorum_met = approvals_collected >= 2;

    Ok(Json(SealStateResponse {
        period_id,
        period_status: period.status,
        seal,
        approvals_collected,
        quorum_met,
    }))
}

/// Proposes a seal for an open period: builds the canonical statement and hash.
async fn propose_seal(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(period_id): Path<String>,
) -> Result<(StatusCode, Json<ProposeResponse>), ApiError> {
    if auth.role == "auditor" || auth.role == "staff" {
        return Err(ApiError::Forbidden(
            "Only Organization Owners, Admins, or Controllers can propose a period close".into(),
        ));
    }

    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;
    if period.status == "sealed" {
        return Err(ApiError::Conflict(
            "Period is already sealed and cannot be reproposed".into(),
        ));
    }

    let entries = Entry::find_by_period(&conn, &period_id)?;
    let core_period = period.to_core();
    let statement = build_statement(&core_period, &entries)?;

    let statement_hash_bytes = statement_hash(&statement);
    let statement_hash_hex = format!("0x{}", hex::encode(statement_hash_bytes));
    let ledger_root_hex = format!("0x{}", hex::encode(statement.ledger_root));
    let human_readable = statement.to_human_readable();

    let seal_id = format!("seal_{period_id}");
    let seal = SealRecord {
        id: seal_id,
        period_id: period_id.clone(),
        root: ledger_root_hex,
        statement_hash: statement_hash_hex.clone(),
        topic_id: None,
        sequence_number: None,
        consensus_timestamp: None,
        approver_1_pubkey: None,
        approver_1_sig: None,
        approver_2_pubkey: None,
        approver_2_sig: None,
        dispatch_status: "draft".into(),
        auditor_id: None,
        auditor_notes: None,
        created_at: Utc::now().to_rfc3339(),
    };

    seal.upsert(&conn)?;

    Ok((
        StatusCode::CREATED,
        Json(ProposeResponse {
            period_id,
            statement,
            statement_hash: statement_hash_hex,
            human_readable,
        }),
    ))
}

/// Dispatches a proposed seal to an auditor for review.
async fn dispatch_seal(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(period_id): Path<String>,
    Json(payload): Json<DispatchSealRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if auth.role == "auditor" || auth.role == "staff" {
        return Err(ApiError::Forbidden(
            "Only Organization Owners, Admins, or Controllers can dispatch a seal to auditors"
                .into(),
        ));
    }

    let auditor_id = payload.auditor_id.unwrap_or_else(|| "usr_bob".into());

    let conn = state.db.lock();
    SealRecord::dispatch_to_auditor(&conn, &period_id, &auditor_id)?;

    Ok(Json(serde_json::json!({
        "status": "dispatched",
        "period_id": period_id,
        "auditor_id": auditor_id,
        "dispatch_status": "pending_auditor"
    })))
}

/// Rejects an audit proposal and returns notes to the controller.
async fn reject_seal(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(period_id): Path<String>,
    Json(payload): Json<RejectSealRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if auth.role != "auditor" && auth.role != "owner" && auth.role != "admin" {
        return Err(ApiError::Forbidden(
            "Only the assigned Auditor or Admin can reject a proposed close".into(),
        ));
    }

    let conn = state.db.lock();
    SealRecord::reject_audit(&conn, &period_id, &payload.notes)?;

    Ok(Json(serde_json::json!({
        "status": "rejected",
        "period_id": period_id,
        "notes": payload.notes,
        "dispatch_status": "rejected"
    })))
}

/// Records one approver's cryptographic signature against the proposed seal.
async fn approve_seal(
    State(state): State<AppState>,
    _auth: AuthContext,
    Path(period_id): Path<String>,
    Json(payload): Json<ApproveRequest>,
) -> Result<Json<ApproveResponse>, ApiError> {
    let mut seal = {
        let conn = state.db.lock();
        let p = PeriodRecord::find_by_id(&conn, &period_id)?;
        if p.status == "sealed" {
            return Err(ApiError::Conflict("Period is already sealed".into()));
        }
        SealRecord::find_by_period_id(&conn, &period_id)
            .map_err(|_| ApiError::BadRequest("Seal must be proposed before approving".into()))?
    };

    let (approver_pubkey_hex, signature_hex) = match (&payload.approver_pubkey, &payload.signature)
    {
        (Some(pk), Some(sig)) => {
            let pk_clean = pk.trim_start_matches("0x");
            let sig_clean = sig.trim_start_matches("0x");

            let pk_bytes = hex::decode(pk_clean)
                .map_err(|_| ApiError::BadRequest("Invalid hex in approver_pubkey".into()))?;
            let sig_bytes = hex::decode(sig_clean)
                .map_err(|_| ApiError::BadRequest("Invalid hex in signature".into()))?;

            let st_hash_clean = seal.statement_hash.trim_start_matches("0x");
            let st_hash_bytes = hex::decode(st_hash_clean)
                .map_err(|_| ApiError::Internal("Invalid statement hash in db".into()))?;

            crypto::verify_with_pubkey(&st_hash_bytes, &sig_bytes, &pk_bytes)
                .map_err(|e| ApiError::BadRequest(format!("Signature verification failed: {e}")))?;

            (format!("0x{pk_clean}"), format!("0x{sig_clean}"))
        }
        _ => match &payload.wallet_id {
            Some(wid) => {
                let privy = state.privy.as_ref().ok_or_else(|| {
                    ApiError::Internal("Privy client is not configured on this server".into())
                })?;

                let st_hash_clean = seal.statement_hash.trim_start_matches("0x");
                let st_hash_vec = hex::decode(st_hash_clean)
                    .map_err(|_| ApiError::Internal("Invalid statement hash in db".into()))?;
                let mut st_hash_bytes = [0u8; 32];
                st_hash_bytes.copy_from_slice(&st_hash_vec);

                let (sig_64, recovered_info) = privy
                    .sign_hash(wid, &st_hash_bytes)
                    .await
                    .map_err(|e| ApiError::Internal(format!("Privy signing error: {e}")))?;

                (
                    recovered_info.compressed_pubkey_hex,
                    format!("0x{}", hex::encode(sig_64)),
                )
            }
            None => {
                return Err(ApiError::BadRequest(
                    "Must provide either (approver_pubkey, signature) or wallet_id".into(),
                ));
            }
        },
    };

    // Four-Eyes Principle / Separation of Duties (Anti-Self-Approval)
    if seal
        .approver_1_pubkey
        .as_deref()
        .is_some_and(|p| p.eq_ignore_ascii_case(&approver_pubkey_hex))
    {
        return Err(ApiError::Conflict(
            "Separation of Duties violation: The same approver cannot sign both Approver 1 and Approver 2".into(),
        ));
    }

    if seal
        .approver_2_pubkey
        .as_deref()
        .is_some_and(|p| p.eq_ignore_ascii_case(&approver_pubkey_hex))
    {
        return Err(ApiError::Conflict(
            "This approver has already signed the proposed seal".into(),
        ));
    }

    if seal.approver_1_pubkey.is_none() {
        seal.approver_1_pubkey = Some(approver_pubkey_hex.clone());
        seal.approver_1_sig = Some(signature_hex);
    } else if seal.approver_2_pubkey.is_none() {
        seal.approver_2_pubkey = Some(approver_pubkey_hex.clone());
        seal.approver_2_sig = Some(signature_hex);
    } else {
        return Err(ApiError::Conflict(
            "Both required approvals have already been collected".into(),
        ));
    }

    {
        let conn = state.db.lock();
        seal.upsert(&conn)?;
    }

    let approvals_collected = match (
        seal.approver_1_pubkey.is_some(),
        seal.approver_2_pubkey.is_some(),
    ) {
        (true, true) => 2,
        (true, false) => 1,
        _ => 0,
    };

    Ok(Json(ApproveResponse {
        period_id,
        approver_pubkey: approver_pubkey_hex,
        approvals_collected,
        quorum_met: approvals_collected >= 2,
    }))
}

/// Publishes the dual-signed seal statement to Hedera Consensus Service.
async fn publish_seal(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<PublishResponse>, ApiError> {
    let (period, mut seal, entries) = {
        let conn = state.db.lock();
        let p = PeriodRecord::find_by_id(&conn, &period_id)?;
        if p.status == "sealed" {
            return Err(ApiError::Conflict(
                "Period is already sealed on Hedera HCS".into(),
            ));
        }
        let s = SealRecord::find_by_period_id(&conn, &period_id)
            .map_err(|_| ApiError::BadRequest("No proposed seal found for period".into()))?;
        let e = Entry::find_by_period(&conn, &period_id)?;
        (p, s, e)
    };

    let p1_key = seal
        .approver_1_pubkey
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Missing approver 1 signature".into()))?;
    let p1_sig = seal
        .approver_1_sig
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Missing approver 1 signature".into()))?;
    let p2_key = seal
        .approver_2_pubkey
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Missing approver 2 signature".into()))?;
    let p2_sig = seal
        .approver_2_sig
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Missing approver 2 signature".into()))?;

    let p1_key_bytes: [u8; 33] = hex::decode(p1_key.trim_start_matches("0x"))
        .map_err(|_| ApiError::Internal("Invalid hex in approver_1_pubkey".into()))?
        .try_into()
        .map_err(|_| ApiError::Internal("Invalid length for approver_1_pubkey".into()))?;

    let p1_sig_bytes: [u8; 64] = hex::decode(p1_sig.trim_start_matches("0x"))
        .map_err(|_| ApiError::Internal("Invalid hex in approver_1_sig".into()))?
        .try_into()
        .map_err(|_| ApiError::Internal("Invalid length for approver_1_sig".into()))?;

    let p2_key_bytes: [u8; 33] = hex::decode(p2_key.trim_start_matches("0x"))
        .map_err(|_| ApiError::Internal("Invalid hex in approver_2_pubkey".into()))?
        .try_into()
        .map_err(|_| ApiError::Internal("Invalid length for approver_2_pubkey".into()))?;

    let p2_sig_bytes: [u8; 64] = hex::decode(p2_sig.trim_start_matches("0x"))
        .map_err(|_| ApiError::Internal("Invalid hex in approver_2_sig".into()))?
        .try_into()
        .map_err(|_| ApiError::Internal("Invalid length for approver_2_sig".into()))?;

    let core_period = period.to_core();
    let statement = build_statement(&core_period, &entries)?;

    let payload = SealPayload {
        statement,
        approver_1_pubkey: p1_key_bytes,
        approver_1_sig: p1_sig_bytes,
        approver_2_pubkey: p2_key_bytes,
        approver_2_sig: p2_sig_bytes,
    };

    let encoded_bytes = encode_seal_payload(&payload);

    let receipt = state
        .publisher
        .publish(&state.topic_id, &encoded_bytes)
        .await
        .map_err(|e| ApiError::Internal(format!("Hedera consensus broadcast failed: {e}")))?;

    // Persist baseline leaf hashes and mark period immutable
    {
        let conn = state.db.lock();
        let leaves: Vec<(String, [u8; 32])> = entries
            .iter()
            .map(|e| {
                let h = sealed_books_core::hash_entry_unchecked(e);
                (e.id.clone(), h)
            })
            .collect();

        SealRecord::save_leaves(&conn, &period_id, &leaves)?;

        seal.topic_id = Some(receipt.topic_id.clone());
        seal.sequence_number = Some(receipt.sequence_number as i64);
        seal.consensus_timestamp = Some(receipt.consensus_timestamp.clone());
        seal.dispatch_status = "sealed".into();
        seal.upsert(&conn)?;

        PeriodRecord::mark_sealed(&conn, &period_id)?;
    }

    let hashscan_url = if !receipt.consensus_timestamp.is_empty() {
        format!("https://hashscan.io/testnet/transaction/{}", receipt.consensus_timestamp)
    } else {
        format!("https://hashscan.io/testnet/topic/{}", receipt.topic_id)
    };

    Ok(Json(PublishResponse {
        status: "published".into(),
        topic_id: receipt.topic_id,
        sequence_number: receipt.sequence_number,
        consensus_timestamp: receipt.consensus_timestamp,
        transaction_id: receipt.transaction_id,
        hashscan_url,
        payload_bytes_len: encoded_bytes.len(),
    }))
}

/// Runs independent verification for an accounting period against Hedera mirror node.
async fn verify_seal(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<VerificationReport>, ApiError> {
    let report = crate::verify::verify_period(&state.db, &state.mirror, &period_id)
        .await
        .map_err(ApiError::BadRequest)?;
    Ok(Json(report))
}
