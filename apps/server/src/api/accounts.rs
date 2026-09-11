//! Accounts API handlers.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::api::AppState;
use crate::api::error::ApiError;
use crate::db::repository::account::Account;

/// Sub-router for account resources mounted under `/api/accounts`.
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_accounts))
}

/// Lists all accounts in the chart of accounts ordered by account code.
pub async fn list_accounts(State(state): State<AppState>) -> Result<Json<Vec<Account>>, ApiError> {
    let conn = state.db.lock();
    let accounts = Account::list_all(&conn)?;
    Ok(Json(accounts))
}
