//! HTTP API routing, state management, and CORS configuration.

pub mod accounts;
pub mod entries;
pub mod error;
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
use crate::privy::PrivyClient;

/// Shared application state accessible across all Axum request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub publisher: PublisherClient,
    pub mirror: MirrorClient,
    pub privy: Option<PrivyClient>,
    pub topic_id: String,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        let publisher =
            PublisherClient::live_from_env().unwrap_or_else(|_| PublisherClient::mock());
        let mirror = MirrorClient::live_from_env();
        let privy = PrivyClient::new_from_env().ok();
        let topic_id =
            std::env::var("HEDERA_TOPIC_ID").unwrap_or_else(|_| "0.0.10462941".to_string());
        Self {
            db,
            publisher,
            mirror,
            privy,
            topic_id,
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
mod tests {
    use super::*;
    use crate::db::repository::period::PeriodRecord;
    use crate::seed::seed_demo_data;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::rand_core::OsRng;
    use tower::ServiceExt;

    fn setup_test_app() -> (Router, Database) {
        let db = Database::open_in_memory().unwrap();
        {
            let mut conn = db.lock();
            seed_demo_data(&mut conn).unwrap();
        }
        let app = create_router(AppState::new_mock(db.clone()));
        (app, db)
    }

    #[tokio::test]
    async fn test_health_check() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/accounts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let accounts: Vec<Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(accounts.len(), 13);
    }

    #[tokio::test]
    async fn test_list_periods_and_get_period() {
        let (app, _) = setup_test_app();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/periods")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let periods: Vec<Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0]["id"], "per_2026_08");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_2026_08")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let period: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(period["id"], "per_2026_08");
        assert_eq!(period["entity"], "Acme Trading Pvt Ltd");
    }

    #[tokio::test]
    async fn test_get_nonexistent_period_returns_404() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_entries() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_2026_08/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let entries: Vec<Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.len(), 16);
    }

    #[tokio::test]
    async fn test_post_balanced_entry_succeeds() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "date": "2026-08-25",
            "description": "Consulting expense",
            "lines": [
                { "account_id": "acc_5040", "direction": "debit", "amount_minor": 50000 },
                { "account_id": "acc_1010", "direction": "credit", "amount_minor": 50000 }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let created: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(created["description"], "Consulting expense");
    }

    #[tokio::test]
    async fn test_post_unbalanced_entry_rejected_with_422() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "date": "2026-08-25",
            "description": "Broken transaction",
            "lines": [
                { "account_id": "acc_5040", "direction": "debit", "amount_minor": 50000 },
                { "account_id": "acc_1010", "direction": "credit", "amount_minor": 40000 }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_post_out_of_period_range_entry_rejected_with_422() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "date": "2026-09-05",
            "description": "September entry in August period",
            "lines": [
                { "account_id": "acc_5040", "direction": "debit", "amount_minor": 10000 },
                { "account_id": "acc_1010", "direction": "credit", "amount_minor": 10000 }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_post_entry_to_sealed_period_returns_409() {
        let db = Database::open_in_memory().unwrap();
        {
            let conn = db.lock();
            let period = PeriodRecord {
                id: "per_sealed".into(),
                entity: "Acme Corp".into(),
                start_date: "2026-08-01".into(),
                end_date: "2026-08-31".into(),
                status: "sealed".into(),
            };
            period.insert(&conn).unwrap();
        }
        let app = create_router(AppState::new_mock(db));

        let payload = json!({
            "date": "2026-08-15",
            "description": "Should fail",
            "lines": [
                { "account_id": "acc_1010", "direction": "debit", "amount_minor": 1000 },
                { "account_id": "acc_5040", "direction": "credit", "amount_minor": 1000 }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_sealed/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_get_period_statement() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_2026_08/statement")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let resp_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp_json["statement"]["entity"], "Acme Trading Pvt Ltd");
        assert_eq!(resp_json["statement"]["entry_count"], 16);
        assert!(
            resp_json["statement_hash_hex"]
                .as_str()
                .unwrap()
                .starts_with("0x")
        );
        assert!(
            resp_json["ledger_root_hex"]
                .as_str()
                .unwrap()
                .starts_with("0x")
        );
    }

    #[tokio::test]
    async fn test_seal_propose_returns_201_and_statement() {
        let (app, _) = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/propose")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["period_id"], "per_2026_08");
        assert_eq!(json["statement"]["entry_count"], 16);
        assert!(json["statement_hash"].as_str().unwrap().starts_with("0x"));
    }

    #[tokio::test]
    async fn test_seal_publish_without_quorum_rejected_with_422() {
        let (app, _) = setup_test_app();

        // Propose first
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/propose")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Attempt publish without approvals
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/publish")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_seal_approve_and_publish_e2e_flow() {
        let (app, db) = setup_test_app();

        // 1. Propose seal
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/propose")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let propose_json: Value = serde_json::from_slice(&body).unwrap();
        let st_hash_hex = propose_json["statement_hash"].as_str().unwrap();
        let st_hash_bytes = hex::decode(st_hash_hex.trim_start_matches("0x")).unwrap();

        // 2. Approver 1 signs
        let key1 = SigningKey::random(&mut OsRng);
        let vk1 = key1.verifying_key();
        let pk1_hex = format!("0x{}", hex::encode(vk1.to_encoded_point(true).as_bytes()));
        let (sig1, rec1) = key1.sign_prehash_recoverable(&st_hash_bytes).unwrap();
        let mut sig1_bytes = sig1.to_bytes().to_vec();
        sig1_bytes.push(rec1.to_byte());
        let sig1_hex = format!("0x{}", hex::encode(sig1_bytes));

        let app_req_1 = json!({
            "approver_pubkey": pk1_hex,
            "signature": sig1_hex,
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&app_req_1).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let app_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(app_json["approvals_collected"], 1);
        assert_eq!(app_json["quorum_met"], false);

        // 3. Approver 2 signs
        let key2 = SigningKey::random(&mut OsRng);
        let vk2 = key2.verifying_key();
        let pk2_hex = format!("0x{}", hex::encode(vk2.to_encoded_point(true).as_bytes()));
        let (sig2, rec2) = key2.sign_prehash_recoverable(&st_hash_bytes).unwrap();
        let mut sig2_bytes = sig2.to_bytes().to_vec();
        sig2_bytes.push(rec2.to_byte());
        let sig2_hex = format!("0x{}", hex::encode(sig2_bytes));

        let app_req_2 = json!({
            "approver_pubkey": pk2_hex,
            "signature": sig2_hex,
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&app_req_2).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let app_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(app_json["approvals_collected"], 2);
        assert_eq!(app_json["quorum_met"], true);

        // 4. Publish to Hedera
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/publish")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let pub_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(pub_json["status"], "sealed");
        assert_eq!(pub_json["sequence_number"], 1);
        assert!(pub_json["payload_bytes_len"].as_u64().unwrap() < 400);

        // 5. Verify Period status is now sealed in database
        {
            let conn = db.lock();
            let p = PeriodRecord::find_by_id(&conn, "per_2026_08").unwrap();
            assert_eq!(p.status, "sealed");
        }

        // 6. Verify that posting a new entry is now permanently rejected with 409 Conflict
        let new_entry_payload = json!({
            "date": "2026-08-28",
            "description": "Post-seal attempt",
            "lines": [
                { "account_id": "acc_5040", "direction": "debit", "amount_minor": 1000 },
                { "account_id": "acc_1010", "direction": "credit", "amount_minor": 1000 }
            ]
        });

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&new_entry_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_seal_duplicate_approval_rejected_with_409() {
        let (app, _) = setup_test_app();

        // Propose
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/propose")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let propose_json: Value = serde_json::from_slice(&body).unwrap();
        let st_hash_hex = propose_json["statement_hash"].as_str().unwrap();
        let st_hash_bytes = hex::decode(st_hash_hex.trim_start_matches("0x")).unwrap();

        let key = SigningKey::random(&mut OsRng);
        let vk = key.verifying_key();
        let pk_hex = format!("0x{}", hex::encode(vk.to_encoded_point(true).as_bytes()));
        let (sig, rec) = key.sign_prehash_recoverable(&st_hash_bytes).unwrap();
        let mut sig_bytes = sig.to_bytes().to_vec();
        sig_bytes.push(rec.to_byte());
        let sig_hex = format!("0x{}", hex::encode(sig_bytes));

        let app_req = json!({
            "approver_pubkey": pk_hex,
            "signature": sig_hex,
        });

        // First approval succeeds
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&app_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Same approver attempting second approval fails with 409 Conflict
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&app_req).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }
}
