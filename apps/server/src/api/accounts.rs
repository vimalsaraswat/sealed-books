//! Accounts API handlers with multi-tenant isolation and role enforcement.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::api::error::ApiError;
use crate::db::repository::account::Account;

/// Sub-router for account resources mounted under `/api/accounts`.
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_accounts).post(post_account))
}

/// Request payload for creating a new chart of accounts code.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub account_type: String,
}

/// Lists all accounts for the user's active organization ordered by account code.
pub async fn list_accounts(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<Account>>, ApiError> {
    let conn = state.db.lock();
    let accounts = Account::list_by_org(&conn, &auth.active_organization.id)?;
    Ok(Json(accounts))
}

/// Creates a new account in the chart of accounts for the user's active organization.
pub async fn post_account(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<Account>), ApiError> {
    if auth.role == "auditor" {
        return Err(ApiError::Forbidden(
            "Auditors have read-only access and cannot modify the chart of accounts".into(),
        ));
    }

    let valid_types = ["asset", "liability", "equity", "revenue", "expense"];
    let account_type = payload.account_type.trim().to_lowercase();
    if !valid_types.contains(&account_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid account_type '{}'. Must be one of: asset, liability, equity, revenue, expense",
            payload.account_type
        )));
    }

    let code = payload.code.trim().to_string();
    if code.is_empty() {
        return Err(ApiError::BadRequest("Account code cannot be empty".into()));
    }
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("Account name cannot be empty".into()));
    }

    let id = format!("acc_{code}");
    let account = Account {
        id,
        organization_id: auth.active_organization.id,
        code,
        name,
        account_type,
    };

    {
        let conn = state.db.lock();
        account.insert(&conn)?;
    }

    Ok((StatusCode::CREATED, Json(account)))
}
