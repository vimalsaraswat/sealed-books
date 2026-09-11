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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::period::PeriodRecord;
    use crate::seed::seed_demo_data;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn setup_test_app() -> Router {
        let db = Database::open_in_memory().unwrap();
        {
            let mut conn = db.lock();
            seed_demo_data(&mut conn).unwrap();
        }
        create_router(AppState::new(db))
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = setup_test_app();

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
        let app = setup_test_app();

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
        let accounts: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(accounts.as_array().unwrap().len(), 13);
    }

    #[tokio::test]
    async fn test_list_periods_and_get_period() {
        let app = setup_test_app();

        // 1. List periods
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
        let periods: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(periods.as_array().unwrap().len(), 1);
        assert_eq!(periods[0]["id"], "per_2026_08");

        // 2. Get specific period
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
        assert_eq!(period["entity"], "Acme Trading Pvt Ltd");
    }

    #[tokio::test]
    async fn test_get_nonexistent_period_returns_404() {
        let app = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let err_json: Value = serde_json::from_slice(&body).unwrap();
        assert!(err_json["error"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_period_statement() {
        let app = setup_test_app();

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
        let res: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(res["statement"]["entry_count"], 16);
        assert_eq!(res["statement"]["total_debits_minor"], 12185000);
        assert_eq!(res["statement"]["total_credits_minor"], 12185000);
        assert_eq!(res["statement_hash_hex"].as_str().unwrap().len(), 64);
        assert_eq!(res["ledger_root_hex"].as_str().unwrap().len(), 64);
        assert!(
            res["human_readable"]
                .as_str()
                .unwrap()
                .contains("Acme Trading Pvt Ltd")
        );
    }

    #[tokio::test]
    async fn test_list_entries() {
        let app = setup_test_app();

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
        let entries: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.as_array().unwrap().len(), 16);
    }

    #[tokio::test]
    async fn test_post_balanced_entry_succeeds() {
        let app = setup_test_app();

        let new_entry_payload = json!({
            "date": "2026-08-20",
            "description": "Consulting advisory services",
            "lines": [
                {
                    "account_id": "acc_1010",
                    "direction": "debit",
                    "amount_minor": 100000,
                    "description": "Advisory retainer"
                },
                {
                    "account_id": "acc_4010",
                    "direction": "credit",
                    "amount_minor": 100000,
                    "description": "Fee revenue"
                }
            ]
        });

        let response = app
            .clone()
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

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify entry count is now 17
        let list_res = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_2026_08/entries")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = list_res.into_body().collect().await.unwrap().to_bytes();
        let entries: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.as_array().unwrap().len(), 17);
    }

    #[tokio::test]
    async fn test_post_unbalanced_entry_rejected_with_422() {
        let app = setup_test_app();

        let unbalanced_payload = json!({
            "date": "2026-08-20",
            "description": "Tampered entry",
            "lines": [
                {
                    "account_id": "acc_1010",
                    "direction": "debit",
                    "amount_minor": 100000,
                    "description": "Advisory retainer"
                },
                {
                    "account_id": "acc_4010",
                    "direction": "credit",
                    "amount_minor": 99000,
                    "description": "Unbalanced credit"
                }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&unbalanced_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let err_json: Value = serde_json::from_slice(&body).unwrap();
        assert!(err_json["error"].as_str().unwrap().contains("unbalanced"));
    }

    #[tokio::test]
    async fn test_post_out_of_period_range_entry_rejected_with_422() {
        let app = setup_test_app();

        let out_of_range_payload = json!({
            "date": "2026-09-05",
            "description": "Future entry",
            "lines": [
                {
                    "account_id": "acc_1010",
                    "direction": "debit",
                    "amount_minor": 10000,
                    "description": "Test"
                },
                {
                    "account_id": "acc_4010",
                    "direction": "credit",
                    "amount_minor": 10000,
                    "description": "Test"
                }
            ]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&out_of_range_payload).unwrap(),
                    ))
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
            let mut conn = db.lock();
            seed_demo_data(&mut conn).unwrap();
            // Transition period to sealed
            PeriodRecord::mark_sealed(&mut conn, "per_2026_08").unwrap();
        }
        let app = create_router(AppState::new(db));

        let payload = json!({
            "date": "2026-08-20",
            "description": "Attempt to mutate sealed period",
            "lines": [
                {
                    "account_id": "acc_1010",
                    "direction": "debit",
                    "amount_minor": 5000,
                    "description": "Debit"
                },
                {
                    "account_id": "acc_4010",
                    "direction": "credit",
                    "amount_minor": 5000,
                    "description": "Credit"
                }
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

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let err_json: Value = serde_json::from_slice(&body).unwrap();
        assert!(err_json["error"].as_str().unwrap().contains("sealed"));
    }
}
