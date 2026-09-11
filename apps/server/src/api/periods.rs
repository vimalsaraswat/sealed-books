//! Periods API handlers and close statement generator.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::api::AppState;
use crate::api::entries;
use crate::api::error::ApiError;
use crate::db::repository::entry::EntryExt;
use crate::db::repository::period::PeriodRecord;
use sealed_books_core::hash::statement_hash;
use sealed_books_core::merkle::build_statement;
use sealed_books_core::types::{Entry, SealStatement};

/// Sub-router for period resources mounted under `/api/periods`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_periods))
        .route("/:id", get(get_period))
        .route("/:id/statement", get(get_period_statement))
        .nest("/:id/entries", entries::router())
}

/// Response containing the deterministic period close statement and calculated hashes.
#[derive(Debug, Clone, Serialize)]
pub struct StatementResponse {
    pub statement: SealStatement,
    pub statement_hash_hex: String,
    pub ledger_root_hex: String,
    pub human_readable: String,
}

/// Lists all accounting periods ordered chronologically by start date.
pub async fn list_periods(
    State(state): State<AppState>,
) -> Result<Json<Vec<PeriodRecord>>, ApiError> {
    let conn = state.db.lock();
    let periods = PeriodRecord::list_all(&conn)?;
    Ok(Json(periods))
}

/// Retrieves a specific accounting period by its unique ID.
pub async fn get_period(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<PeriodRecord>, ApiError> {
    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;
    Ok(Json(period))
}

/// Generates the deterministic close statement and Merkle root for a period.
pub async fn get_period_statement(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<StatementResponse>, ApiError> {
    let conn = state.db.lock();

    // 1. Fetch period record and map to core domain
    let period_record = PeriodRecord::find_by_id(&conn, &period_id)?;
    let core_period = period_record.to_core();

    // 2. Fetch all entries for this period
    let entries = <Entry as EntryExt>::find_by_period(&conn, &period_id)?;

    // 3. Build canonical statement and Merkle tree root using crates/core
    let statement = build_statement(&core_period, &entries)?;
    let hash = statement_hash(&statement);
    let human_readable = statement.to_human_readable();
    let ledger_root_hex = hex::encode(statement.ledger_root);
    let statement_hash_hex = hex::encode(hash);

    Ok(Json(StatementResponse {
        statement,
        statement_hash_hex,
        ledger_root_hex,
        human_readable,
    }))
}
