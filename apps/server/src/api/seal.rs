//! Period seal workflow API handlers (propose, approve, publish).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::api::error::ApiError;
use crate::crypto;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use crate::db::repository::seal::SealRecord;
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

/// Response returned when a seal is published to Hedera.
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
        .route("/approve", post(approve_seal))
        .route("/publish", post(publish_seal))
}

/// Retrieves the current seal progress or sealed status for a period.
async fn get_seal_state(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<SealStateResponse>, ApiError> {
    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;
    let seal = SealRecord::find_by_period_id(&conn, &period_id).ok();

    let approvals_collected = seal
        .as_ref()
        .map(|s| {
            let mut count = 0;
            if s.approver_1_pubkey.is_some() && s.approver_1_sig.is_some() {
                count += 1;
            }
            if s.approver_2_pubkey.is_some() && s.approver_2_sig.is_some() {
                count += 1;
            }
            count
        })
        .unwrap_or(0);

    Ok(Json(SealStateResponse {
        period_id,
        period_status: period.status,
        approvals_collected,
        quorum_met: approvals_collected >= 2,
        seal,
    }))
}

/// Proposes a seal for an open period by computing its canonical statement and hash.
async fn propose_seal(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<(StatusCode, Json<ProposeResponse>), ApiError> {
    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;

    if period.status == "sealed" {
        return Err(ApiError::Conflict(
            "Period is already sealed; cannot re-propose".into(),
        ));
    }

    let entries = Entry::find_by_period(&conn, &period_id)?;
    let core_period = period.to_core();

    let statement = build_statement(&core_period, &entries)?;
    let st_hash = statement_hash(&statement);
    let st_hash_hex = format!("0x{}", hex::encode(st_hash));
    let root_hex = format!("0x{}", hex::encode(statement.ledger_root));
    let human_readable = statement.to_human_readable();

    let seal_record = SealRecord {
        id: format!("seal_{period_id}"),
        period_id: period_id.clone(),
        root: root_hex,
        statement_hash: st_hash_hex.clone(),
        topic_id: None,
        sequence_number: None,
        consensus_timestamp: None,
        approver_1_pubkey: None,
        approver_1_sig: None,
        approver_2_pubkey: None,
        approver_2_sig: None,
        created_at: Utc::now().to_rfc3339(),
    };

    seal_record.upsert(&conn)?;

    Ok((
        StatusCode::CREATED,
        Json(ProposeResponse {
            period_id,
            statement,
            statement_hash: st_hash_hex,
            human_readable,
        }),
    ))
}

/// Records one approver's cryptographic signature against the proposed seal.
async fn approve_seal(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
    Json(payload): Json<ApproveRequest>,
) -> Result<Json<ApproveResponse>, ApiError> {
    let mut seal = {
        let conn = state.db.lock();
        let period = PeriodRecord::find_by_id(&conn, &period_id)?;
        if period.status == "sealed" {
            return Err(ApiError::Conflict("Period is already sealed".into()));
        }
        SealRecord::find_by_period_id(&conn, &period_id).map_err(|_| {
            ApiError::NotFound("No seal proposal found for this period. Call /propose first".into())
        })?
    };

    let hash_bytes_vec = hex::decode(seal.statement_hash.trim_start_matches("0x"))
        .map_err(|e| ApiError::Internal(format!("Invalid statement hash in DB: {e}")))?;
    if hash_bytes_vec.len() != 32 {
        return Err(ApiError::Internal(
            "Stored statement hash is not 32 bytes".into(),
        ));
    }
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&hash_bytes_vec);

    let (pubkey_hex, sig_hex) = if let Some(wallet_id) = payload.wallet_id {
        let privy_client = state.privy.as_ref().ok_or_else(|| {
            ApiError::Unprocessable(
                "Privy client is not configured for server-assisted signing".into(),
            )
        })?;

        let (sig_64, recovered) = privy_client
            .sign_hash(&wallet_id, &hash_bytes)
            .await
            .map_err(ApiError::Unprocessable)?;

        (
            recovered.compressed_pubkey_hex,
            format!("0x{}", hex::encode(sig_64)),
        )
    } else if let (Some(pk_raw), Some(sig_raw)) = (payload.approver_pubkey, payload.signature) {
        let pk_clean = pk_raw.trim_start_matches("0x");
        let pk_bytes = hex::decode(pk_clean)
            .map_err(|e| ApiError::BadRequest(format!("Invalid public key hex: {e}")))?;
        if pk_bytes.len() != 33 {
            return Err(ApiError::BadRequest(format!(
                "Expected 33-byte compressed public key, got {} bytes",
                pk_bytes.len()
            )));
        }

        let sig_clean = sig_raw.trim_start_matches("0x");
        let sig_bytes = hex::decode(sig_clean)
            .map_err(|e| ApiError::BadRequest(format!("Invalid signature hex: {e}")))?;

        let (sig_64, v_opt) =
            crypto::split_signature(&sig_bytes).map_err(|e| ApiError::BadRequest(e))?;

        if let Some(v_byte) = v_opt {
            let recovered =
                crypto::verify_and_recover(&hash_bytes, sig_64, v_byte).map_err(|e| {
                    ApiError::Unprocessable(format!("Cryptographic verification failed: {e}"))
                })?;
            if !recovered
                .compressed_pubkey_hex
                .eq_ignore_ascii_case(&format!("0x{pk_clean}"))
            {
                return Err(ApiError::Unprocessable(
                    "Recovered public key does not match claimed approver public key".into(),
                ));
            }
        } else {
            crypto::verify_with_pubkey(&hash_bytes, sig_64, &pk_bytes).map_err(|e| {
                ApiError::Unprocessable(format!("Signature verification failed: {e}"))
            })?;
        }

        (
            format!("0x{pk_clean}"),
            format!("0x{}", hex::encode(sig_64)),
        )
    } else {
        return Err(ApiError::BadRequest(
            "Either wallet_id or (approver_pubkey, signature) must be provided".into(),
        ));
    };

    if seal.approver_1_pubkey.as_deref() == Some(&pubkey_hex)
        || seal.approver_2_pubkey.as_deref() == Some(&pubkey_hex)
    {
        return Err(ApiError::Conflict(
            "This approver has already signed the seal proposal".into(),
        ));
    }

    if seal.approver_1_pubkey.is_none() {
        seal.approver_1_pubkey = Some(pubkey_hex.clone());
        seal.approver_1_sig = Some(sig_hex);
    } else if seal.approver_2_pubkey.is_none() {
        seal.approver_2_pubkey = Some(pubkey_hex.clone());
        seal.approver_2_sig = Some(sig_hex);
    } else {
        return Err(ApiError::Conflict(
            "Both required approver slots are already filled".into(),
        ));
    }

    let approvals_collected = {
        let mut c = 0;
        if seal.approver_1_pubkey.is_some() {
            c += 1;
        }
        if seal.approver_2_pubkey.is_some() {
            c += 1;
        }
        c
    };

    {
        let conn = state.db.lock();
        seal.upsert(&conn)?;
    }

    Ok(Json(ApproveResponse {
        period_id,
        approver_pubkey: pubkey_hex,
        approvals_collected,
        quorum_met: approvals_collected >= 2,
    }))
}

/// Publishes a fully approved seal payload to Hedera Consensus Service and marks the period sealed.
async fn publish_seal(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<PublishResponse>, ApiError> {
    let (period, mut seal) = {
        let conn = state.db.lock();
        let period = PeriodRecord::find_by_id(&conn, &period_id)?;
        if period.status == "sealed" {
            return Err(ApiError::Conflict("Period is already sealed".into()));
        }
        let seal = SealRecord::find_by_period_id(&conn, &period_id).map_err(|_| {
            ApiError::NotFound(
                "No seal proposal found for this period. Call /propose and /approve first".into(),
            )
        })?;
        (period, seal)
    };

    let p1_hex = seal
        .approver_1_pubkey
        .as_deref()
        .ok_or_else(|| ApiError::Unprocessable("Approver 1 signature is missing".into()))?;
    let s1_hex = seal
        .approver_1_sig
        .as_deref()
        .ok_or_else(|| ApiError::Unprocessable("Approver 1 signature is missing".into()))?;
    let p2_hex = seal.approver_2_pubkey.as_deref().ok_or_else(|| {
        ApiError::Unprocessable(
            "Approver 2 signature is missing; minimum 2 distinct approvers required".into(),
        )
    })?;
    let s2_hex = seal
        .approver_2_sig
        .as_deref()
        .ok_or_else(|| ApiError::Unprocessable("Approver 2 signature is missing".into()))?;

    if p1_hex.eq_ignore_ascii_case(p2_hex) {
        return Err(ApiError::Unprocessable(
            "Approver 1 and Approver 2 must be distinct cryptographic identities".into(),
        ));
    }

    let p1_bytes = hex::decode(p1_hex.trim_start_matches("0x"))
        .map_err(|e| ApiError::Internal(format!("Invalid stored pubkey 1: {e}")))?;
    let s1_bytes = hex::decode(s1_hex.trim_start_matches("0x"))
        .map_err(|e| ApiError::Internal(format!("Invalid stored sig 1: {e}")))?;
    let p2_bytes = hex::decode(p2_hex.trim_start_matches("0x"))
        .map_err(|e| ApiError::Internal(format!("Invalid stored pubkey 2: {e}")))?;
    let s2_bytes = hex::decode(s2_hex.trim_start_matches("0x"))
        .map_err(|e| ApiError::Internal(format!("Invalid stored sig 2: {e}")))?;

    let mut approver_1_pubkey = [0u8; 33];
    let mut approver_1_sig = [0u8; 64];
    let mut approver_2_pubkey = [0u8; 33];
    let mut approver_2_sig = [0u8; 64];

    approver_1_pubkey.copy_from_slice(&p1_bytes);
    approver_1_sig.copy_from_slice(&s1_bytes);
    approver_2_pubkey.copy_from_slice(&p2_bytes);
    approver_2_sig.copy_from_slice(&s2_bytes);

    let current_statement = {
        let conn = state.db.lock();
        let entries = Entry::find_by_period(&conn, &period_id)?;
        let core_period = period.to_core();
        build_statement(&core_period, &entries)?
    };

    let current_hash_hex = format!("0x{}", hex::encode(statement_hash(&current_statement)));
    if !current_hash_hex.eq_ignore_ascii_case(&seal.statement_hash) {
        return Err(ApiError::Conflict(
            "Ledger entries were modified after seal approval; proposal is invalid".into(),
        ));
    }

    let payload = SealPayload {
        statement: current_statement,
        approver_1_pubkey,
        approver_1_sig,
        approver_2_pubkey,
        approver_2_sig,
    };

    let payload_bytes = encode_seal_payload(&payload);

    let receipt = state
        .publisher
        .publish(&state.topic_id, &payload_bytes)
        .await
        .map_err(|e| ApiError::Internal(format!("Hedera HCS submission failed: {e}")))?;

    {
        let conn = state.db.lock();
        seal.topic_id = Some(receipt.topic_id.clone());
        seal.sequence_number = Some(receipt.sequence_number as i64);
        seal.consensus_timestamp = Some(receipt.consensus_timestamp.clone());
        seal.upsert(&conn)?;

        PeriodRecord::mark_sealed(&conn, &period_id)?;
    }

    let hashscan_url = format!("https://hashscan.io/testnet/topic/{}", receipt.topic_id);

    Ok(Json(PublishResponse {
        status: "sealed".to_string(),
        topic_id: receipt.topic_id,
        sequence_number: receipt.sequence_number,
        consensus_timestamp: receipt.consensus_timestamp,
        transaction_id: receipt.transaction_id,
        hashscan_url,
        payload_bytes_len: payload_bytes.len(),
    }))
}
