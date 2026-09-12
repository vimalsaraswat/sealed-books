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
    async fn test_post_account_valid() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "code": "1050",
            "name": "Petty Cash",
            "account_type": "asset"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/accounts")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let account: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(account["id"], "acc_1050");
        assert_eq!(account["code"], "1050");
        assert_eq!(account["name"], "Petty Cash");
        assert_eq!(account["account_type"], "asset");
    }

    #[tokio::test]
    async fn test_post_account_invalid_type() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "code": "9999",
            "name": "Invalid Account",
            "account_type": "magical_money"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/accounts")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_periods() {
        let (app, _) = setup_test_app();

        let response = app
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
    }

    #[tokio::test]
    async fn test_post_period() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "id": "per_2026_09",
            "entity": "Acme Trading Pvt Ltd",
            "start_date": "2026-09-01",
            "end_date": "2026-09-30"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let period: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(period["id"], "per_2026_09");
        assert_eq!(period["status"], "open");
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
    async fn test_post_entry_balanced() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "date": "2026-08-15",
            "description": "Test wire transaction",
            "lines": [
                { "account_id": "acc_1010", "direction": "debit", "amount_minor": 5000 },
                { "account_id": "acc_5040", "direction": "credit", "amount_minor": 5000 }
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
        let entry: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(entry["description"], "Test wire transaction");
        assert_eq!(entry["lines"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_post_entry_unbalanced_returns_400() {
        let (app, _) = setup_test_app();

        let payload = json!({
            "date": "2026-08-15",
            "description": "Unbalanced wire",
            "lines": [
                { "account_id": "acc_1010", "direction": "debit", "amount_minor": 5000 },
                { "account_id": "acc_5040", "direction": "credit", "amount_minor": 4000 }
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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_post_entry_to_sealed_period_returns_409() {
        let db = Database::open_in_memory().unwrap();
        {
            let mut conn = db.lock();
            seed_demo_data(&mut conn).unwrap();
            let period = PeriodRecord {
                id: "per_sealed".into(),
                organization_id: "org_acme".into(),
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
    async fn test_full_seal_workflow_propose_approve_publish() {
        let (app, _) = setup_test_app();

        // 1. Propose seal
        let propose_res = app
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

        assert_eq!(propose_res.status(), StatusCode::CREATED);
        let prop_body = propose_res.into_body().collect().await.unwrap().to_bytes();
        let prop_json: Value = serde_json::from_slice(&prop_body).unwrap();
        let st_hash_hex = prop_json["statement_hash"].as_str().unwrap().to_string();
        let st_hash_bytes = hex::decode(st_hash_hex.trim_start_matches("0x")).unwrap();

        // 2. Approver 1 signs
        let key1 = SigningKey::random(&mut OsRng);
        let pk1_bytes = key1.verifying_key().to_encoded_point(true);
        let (sig1, _) = key1.sign_prehash_recoverable(&st_hash_bytes).unwrap();
        let mut sig1_64 = [0u8; 64];
        sig1_64[..32].copy_from_slice(&sig1.r().to_bytes());
        sig1_64[32..].copy_from_slice(&sig1.s().to_bytes());

        let app1_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "approver_pubkey": format!("0x{}", hex::encode(pk1_bytes.as_bytes())),
                            "signature": format!("0x{}", hex::encode(sig1_64))
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(app1_res.status(), StatusCode::OK);
        let app1_body = app1_res.into_body().collect().await.unwrap().to_bytes();
        let app1_json: Value = serde_json::from_slice(&app1_body).unwrap();
        assert_eq!(app1_json["approvals_collected"], 1);
        assert_eq!(app1_json["quorum_met"], false);

        // 3. Approver 2 signs
        let key2 = SigningKey::random(&mut OsRng);
        let pk2_bytes = key2.verifying_key().to_encoded_point(true);
        let (sig2, _) = key2.sign_prehash_recoverable(&st_hash_bytes).unwrap();
        let mut sig2_64 = [0u8; 64];
        sig2_64[..32].copy_from_slice(&sig2.r().to_bytes());
        sig2_64[32..].copy_from_slice(&sig2.s().to_bytes());

        let app2_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/approve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "approver_pubkey": format!("0x{}", hex::encode(pk2_bytes.as_bytes())),
                            "signature": format!("0x{}", hex::encode(sig2_64))
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(app2_res.status(), StatusCode::OK);
        let app2_body = app2_res.into_body().collect().await.unwrap().to_bytes();
        let app2_json: Value = serde_json::from_slice(&app2_body).unwrap();
        assert_eq!(app2_json["approvals_collected"], 2);
        assert_eq!(app2_json["quorum_met"], true);

        // 4. Publish to Hedera
        let pub_res = app
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

        assert_eq!(pub_res.status(), StatusCode::OK);
        let pub_body = pub_res.into_body().collect().await.unwrap().to_bytes();
        let pub_json: Value = serde_json::from_slice(&pub_body).unwrap();
        assert_eq!(pub_json["status"], "published");
        assert_eq!(pub_json["sequence_number"], 1);

        // 5. Query verification
        let ver_res = app
            .oneshot(
                Request::builder()
                    .uri("/api/periods/per_2026_08/seal/verify")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(ver_res.status(), StatusCode::OK);
        let ver_body = ver_res.into_body().collect().await.unwrap().to_bytes();
        let ver_json: Value = serde_json::from_slice(&ver_body).unwrap();
        assert_eq!(ver_json["status"], "verified");
        assert_eq!(ver_json["is_intact"], true);
    }

    #[tokio::test]
    async fn test_login_and_auth_context_switch_org() {
        let (app, _) = setup_test_app();

        // 1. Login as Bob (Auditor in Acme, Owner in Deloitte)
        let login_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "user_id": "usr_bob" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(login_res.status(), StatusCode::OK);
        let body = login_res.into_body().collect().await.unwrap().to_bytes();
        let res: Value = serde_json::from_slice(&body).unwrap();
        let token = res["token"].as_str().unwrap();
        assert_eq!(res["user"]["id"], "usr_bob");
        assert_eq!(res["active_organization"]["id"], "org_acme");
        assert_eq!(res["role"], "auditor");

        // 2. Bob switches active organization to Deloitte
        let switch_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/switch-org")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "organization_id": "org_deloitte" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(switch_res.status(), StatusCode::OK);
        let sw_body = switch_res.into_body().collect().await.unwrap().to_bytes();
        let sw_res: Value = serde_json::from_slice(&sw_body).unwrap();
        assert_eq!(sw_res["active_organization"]["id"], "org_deloitte");
        assert_eq!(sw_res["role"], "owner");
    }

    #[tokio::test]
    async fn test_auditor_cannot_post_account_or_entry() {
        let (app, _) = setup_test_app();

        // 1. Auditor attempts to add an account -> 403 Forbidden
        let acc_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/accounts")
                    .header("authorization", "Bearer token_bob")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "code": "1999",
                            "name": "Unauthorized Account",
                            "account_type": "asset"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(acc_res.status(), StatusCode::FORBIDDEN);

        // 2. Auditor attempts to post an entry -> 403 Forbidden
        let ent_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/entries")
                    .header("authorization", "Bearer token_bob")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "date": "2026-08-15",
                            "description": "Auditor rogue entry",
                            "lines": [
                                { "account_id": "acc_1010", "direction": "debit", "amount_minor": 1000 },
                                { "account_id": "acc_5040", "direction": "credit", "amount_minor": 1000 }
                            ]
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(ent_res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_controller_dispatches_to_auditor_and_auditor_rejects_with_notes() {
        let (app, _) = setup_test_app();

        // 1. Alice (Controller) proposes seal
        let prop_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/propose")
                    .header("authorization", "Bearer token_alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(prop_res.status(), StatusCode::CREATED);

        // 2. Alice dispatches to Auditor Bob
        let dispatch_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/dispatch")
                    .header("authorization", "Bearer token_alice")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "auditor_id": "usr_bob" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(dispatch_res.status(), StatusCode::OK);

        // 3. Bob queries pending audits -> sees per_2026_08
        let pending_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/auditor/pending")
                    .header("authorization", "Bearer token_bob")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pending_res.status(), StatusCode::OK);
        let pending_body = pending_res.into_body().collect().await.unwrap().to_bytes();
        let queue: Vec<Value> = serde_json::from_slice(&pending_body).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0]["period_id"], "per_2026_08");
        assert_eq!(queue[0]["dispatch_status"], "pending_auditor");

        // 4. Bob reviews and rejects with notes
        let reject_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/periods/per_2026_08/seal/reject")
                    .header("authorization", "Bearer token_bob")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "notes": "Line item l_14 needs supporting invoice document attached."
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(reject_res.status(), StatusCode::OK);
        let rej_body = reject_res.into_body().collect().await.unwrap().to_bytes();
        let rej_val: Value = serde_json::from_slice(&rej_body).unwrap();
        assert_eq!(rej_val["status"], "rejected");
        assert_eq!(rej_val["dispatch_status"], "rejected");
    }
