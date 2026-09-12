//! Periods API handlers, period creation, tampering simulator, and close statement generator.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::api::entries;
use crate::api::error::ApiError;
use crate::api::seal;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use crate::verify::VerificationReport;
use sealed_books_core::hash::statement_hash;
use sealed_books_core::merkle::build_statement;
use sealed_books_core::types::{Entry, Period as CorePeriod, SealStatement};

/// Sub-router for period resources mounted under `/api/periods`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_periods).post(post_period))
        .route("/:id", get(get_period))
        .route("/:id/statement", get(get_period_statement))
        .route("/:id/verify", get(verify_period_handler))
        .route("/:id/tamper", post(tamper_period_handler))
        .nest("/:id/entries", entries::router())
        .nest("/:id/seal", seal::router())
}

/// Request payload for creating a new accounting period.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePeriodRequest {
    pub id: Option<String>,
    pub entity: String,
    pub start_date: String,
    pub end_date: String,
}

/// Request payload for deliberately modifying a ledger record to simulate tampering.
#[derive(Debug, Clone, Deserialize)]
pub struct TamperEntryRequest {
    pub entry_id: String,
    pub new_description: Option<String>,
}

/// Response containing the deterministic period close statement and calculated hashes.
#[derive(Debug, Clone, Serialize)]
pub struct StatementResponse {
    pub statement: SealStatement,
    pub statement_hash_hex: String,
    pub ledger_root_hex: String,
    pub human_readable: String,
}

/// Lists all accounting periods for the user's active organization ordered chronologically.
pub async fn list_periods(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<PeriodRecord>>, ApiError> {
    let conn = state.db.conn();
    let periods = PeriodRecord::list_by_org(conn, &auth.active_organization.id).await?;
    Ok(Json(periods))
}

/// Creates a brand new accounting period for the user's active organization.
pub async fn post_period(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreatePeriodRequest>,
) -> Result<(StatusCode, Json<PeriodRecord>), ApiError> {
    if auth.role == "auditor" || auth.role == "staff" {
        return Err(ApiError::Forbidden(
            "Only Organization Owners, Admins, or Controllers can create new accounting periods"
                .into(),
        ));
    }

    let entity = payload.entity.trim().to_string();
    if entity.is_empty() {
        return Err(ApiError::BadRequest("Entity name cannot be empty".into()));
    }

    let start_date = payload.start_date.trim().to_string();
    let end_date = payload.end_date.trim().to_string();

    let id = payload.id.unwrap_or_else(|| {
        let clean_start = start_date.replace('-', "_");
        format!(
            "per_{clean_start}_{}",
            &Uuid::new_v4().simple().to_string()[..6]
        )
    });

    let core_check = CorePeriod {
        id: id.clone(),
        entity: entity.clone(),
        start_date: start_date.clone(),
        end_date: end_date.clone(),
    };
    core_check.validate()?;

    let record = PeriodRecord {
        id,
        organization_id: auth.active_organization.id,
        entity,
        start_date,
        end_date,
        status: "open".into(),
    };

    let conn = state.db.conn();
    record.insert(conn).await?;

    Ok((StatusCode::CREATED, Json(record)))
}

/// Retrieves a specific accounting period by its unique ID.
pub async fn get_period(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<PeriodRecord>, ApiError> {
    let conn = state.db.conn();
    let period = PeriodRecord::find_by_id(conn, &period_id).await?;
    Ok(Json(period))
}

/// Computes the cryptographic seal statement and Merkle root for an accounting period.
pub async fn get_period_statement(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<StatementResponse>, ApiError> {
    let conn = state.db.conn();
    let period = PeriodRecord::find_by_id(conn, &period_id).await?;
    let entries = Entry::find_by_period(conn, &period_id).await?;

    let core_period = period.to_core();
    let statement = build_statement(&core_period, &entries)?;
    let statement_hash_bytes = statement_hash(&statement);

    let statement_hash_hex = format!("0x{}", hex::encode(statement_hash_bytes));
    let ledger_root_hex = format!("0x{}", hex::encode(statement.ledger_root));
    let human_readable = statement.to_human_readable();

    Ok(Json(StatementResponse {
        statement,
        statement_hash_hex,
        ledger_root_hex,
        human_readable,
    }))
}

/// Runs independent verification for an accounting period against Hedera mirror node.
pub async fn verify_period_handler(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<VerificationReport>, ApiError> {
    let report = crate::verify::verify_period(&state.db, &state.mirror, &period_id)
        .await
        .map_err(ApiError::BadRequest)?;
    Ok(Json(report))
}

/// Tamper simulator handler: alters an entry directly in SQLite to demonstrate fraud detection.
pub async fn tamper_period_handler(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
    Json(payload): Json<TamperEntryRequest>,
) -> Result<Json<Value>, ApiError> {
    let conn = state.db.conn();
    // Verify period exists
    let _ = PeriodRecord::find_by_id(conn, &period_id).await?;

    let new_desc = payload
        .new_description
        .unwrap_or_else(|| "Fraudulent alteration via SQLite bypass".to_string());

    let updated_rows = conn
        .execute(
            "UPDATE entries SET description = ?1 WHERE id = ?2 AND period_id = ?3;",
            params![
                new_desc.as_str(),
                payload.entry_id.as_str(),
                period_id.as_str()
            ],
        )
        .await
        .map_err(crate::db::error::DbError::Sqlite)?;

    if updated_rows == 0 {
        return Err(ApiError::NotFound(format!(
            "Entry '{}' not found in period '{}'",
            payload.entry_id, period_id
        )));
    }

    Ok(Json(json!({
        "status": "tampered",
        "period_id": period_id,
        "entry_id": payload.entry_id,
        "new_description": new_desc
    })))
}
