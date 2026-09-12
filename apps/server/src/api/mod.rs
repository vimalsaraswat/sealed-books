//! HTTP API routing, state management, and CORS configuration.

pub mod accounts;
pub mod auth;
pub mod entries;
pub mod error;
pub mod organizations;
pub mod periods;
pub mod seal;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::db::Database;
use crate::hedera::PublisherClient;
use crate::mirror::{MirrorClient, MockMessageStore};
use crate::email::EmailService;
use crate::privy::PrivyClient;
use std::sync::Arc;

/// Shared application state accessible across all Axum request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub publisher: PublisherClient,
    pub mirror: MirrorClient,
    pub privy: Option<PrivyClient>,
    pub topic_id: String,
    pub email: Arc<EmailService>,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        let (publisher, mirror) = match PublisherClient::live_from_env() {
            Ok(live_pub) => {
                tracing::info!(
                    "Hedera operator credentials detected: using live Hedera testnet publisher and mirror node"
                );
                (live_pub, MirrorClient::live_from_env())
            }
            Err(err) => {
                tracing::warn!(
                    "Hedera live publisher not configured ({err}); using connected mock store for local development"
                );
                let store = MockMessageStore::new();
                (
                    PublisherClient::mock_with_store(store.clone()),
                    MirrorClient::mock(store),
                )
            }
        };
        let privy = PrivyClient::new_from_env().ok();
        let topic_id =
            std::env::var("HEDERA_TOPIC_ID").unwrap_or_else(|_| "0.0.10462941".to_string());
        Self {
            db,
            publisher,
            mirror,
            privy,
            topic_id,
            email: Arc::new(EmailService::from_env()),
        }
    }

    pub fn new_mock(db: Database) -> Self {
        let store = MockMessageStore::new();
        Self {
            db,
            publisher: PublisherClient::mock_with_store(store.clone()),
            mirror: MirrorClient::mock(store),
            privy: None,
            topic_id: "0.0.10462941".to_string(),
            email: Arc::new(EmailService::default()),
        }
    }

    pub fn with_publisher(mut self, publisher: PublisherClient) -> Self {
        self.publisher = publisher;
        self
    }

    pub fn with_mirror(mut self, mirror: MirrorClient) -> Self {
        self.mirror = mirror;
        self
    }
}

/// Creates the complete Axum router with nested sub-routers, CORS, and logging middleware.
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = Router::new()
        .nest("/auth", auth::router())
        .nest("/organizations", organizations::router())
        .nest("/auditor", organizations::auditor_router())
        .nest("/accounts", accounts::router())
        .nest("/periods", periods::router());

    Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Simple health check handler.
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "sealed-books-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}


#[cfg(test)]
mod tests;
