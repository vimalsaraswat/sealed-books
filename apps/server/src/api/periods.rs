//! Periods API handlers and close statement generator.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::api::AppState;
use crate::api::entries;
use crate::api::error::ApiError;
use crate::api::seal;
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
        .nest("/:id/seal", seal::router())
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

/// Computes the cryptographic seal statement and Merkle root for an accounting period.
pub async fn get_period_statement(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<StatementResponse>, ApiError> {
    let conn = state.db.lock();
    let period = PeriodRecord::find_by_id(&conn, &period_id)?;
    let entries = Entry::find_by_period(&conn, &period_id)?;

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
