//! Authentication and tenancy module for Sealed Books.

pub mod extractor;
pub mod handlers;
pub mod otp;
pub mod service;
pub mod types;

pub use extractor::resolve_auth_context;
pub use types::*;

use crate::api::AppState;
use axum::Router;
use axum::routing::{get, post};

/// Router for authentication and session management endpoints.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(handlers::get_me))
        .route("/login", post(handlers::post_login))
        .route("/register", post(handlers::post_register))
        .route("/logout", post(handlers::post_logout))
        .route("/switch-org", post(handlers::post_switch_org))
        .route("/otp/send", post(handlers::post_send_otp))
        .route("/otp/verify", post(handlers::post_verify_otp))
}
