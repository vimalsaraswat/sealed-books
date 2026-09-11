//! HTTP API error types and JSON response mapping.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

use crate::db::DbError;
use sealed_books_core::CoreError;

/// High-level API errors representing HTTP semantics.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    Unprocessable(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::SerializationError(msg) => ApiError::BadRequest(msg),
            // Accounting domain invariant violations (unbalanced entries, out-of-range, etc.)
            _ => ApiError::Unprocessable(err.to_string()),
        }
    }
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::AccountNotFound(id) => ApiError::NotFound(format!("Account '{id}' not found")),
            DbError::PeriodNotFound(id) => ApiError::NotFound(format!("Period '{id}' not found")),
            DbError::EntryNotFound(id) => ApiError::NotFound(format!("Entry '{id}' not found")),
            DbError::SealNotFound(id) => {
                ApiError::NotFound(format!("Seal for period '{id}' not found"))
            }
            DbError::PeriodSealed(id) => {
                ApiError::Conflict(format!("Period '{id}' is sealed and cannot be modified"))
            }
            DbError::SealAlreadyExists(id) => {
                ApiError::Conflict(format!("Seal for period '{id}' already exists"))
            }
            DbError::Core(core_err) => core_err.into(),
            DbError::Sqlite(e) => {
                tracing::error!("Database SQLite error: {:?}", e);
                ApiError::Internal("Internal database error".to_string())
            }
        }
    }
}
