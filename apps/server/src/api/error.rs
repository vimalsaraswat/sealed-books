//! HTTP API error handling and Axum response conversion.

use crate::db::error::DbError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sealed_books_core::CoreError;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        ApiError::BadRequest(err.to_string())
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
            DbError::EntityNotFound(msg) => ApiError::NotFound(msg),
            DbError::Core(core_err) => core_err.into(),
            DbError::Sqlite(e) => {
                tracing::error!("Database SQLite error: {:?}", e);
                ApiError::Internal("Internal database error".to_string())
            }
        }
    }
}
