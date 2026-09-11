//! HTTP API routing, state management, and CORS configuration.

pub mod accounts;
pub mod entries;
pub mod error;
pub mod periods;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::db::Database;

/// Shared application state accessible across all Axum request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

/// Creates the complete Axum router with nested sub-routers, CORS, and logging middleware.
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = Router::new()
        .nest("/accounts", accounts::router())
        .nest("/periods", periods::router());

    Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Basic health check endpoint.
async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "sealed-books-server" }))
}
