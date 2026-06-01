//! Dunning Letter Management E2E Tests
//!
//! Tests for dunning letter set management, customer profiles, dunning runs,
//! result tracking, and complete workflow.
//! Oracle Fusion: Financials > Receivables > Dunning Letters

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/151_dunning_letter_management.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Letter Set CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_letter_set() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "Standard Dunning",
                        "description": "Standard 3-level dunning letter set",
                        "number_of_levels": 3,
                        "minimum_overdue_days": 30,
                        "currency_code": "USD",
                        "aging_basis": "days_overdue",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["set_name"], "Standard Dunning");
    assert_eq!(d["status"], "draft");
    assert_eq!(d["number_of_levels"], 3);
}

#[tokio::test]
async fn test_create_letter_set_empty_name_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_letter_set_invalid_aging_basis_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "Bad Set",
                        "aging_basis": "invalid",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_letter_set_name() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "set_name": "UNIQUE-SET",
    }))
    .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_letter_sets() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/dunning/letter-sets")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["data"].is_array());
}

#[tokio::test]
async fn test_get_letter_set_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/dunning/letter-sets/{}",
                    uuid::Uuid::new_v4()
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_activate_letter_set_without_lines_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "NO-LINES-SET",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/activate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Letter Set Line Tests
// ========================================================================

#[tokio::test]
async fn test_add_letter_set_line() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "LINE-TEST-SET",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let set_id = d["id"].as_str().unwrap();

    // Add line
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "level_number": 1,
                        "level_name": "Friendly Reminder",
                        "min_days_overdue": 30,
                        "max_days_overdue": 59,
                        "minimum_amount": "100.00",
                        "delivery_method": "email",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["level_name"], "Friendly Reminder");
    assert_eq!(d["delivery_method"], "email");
}

#[tokio::test]
async fn test_add_line_to_active_set_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "ACTIVE-SET-NO-LINE-ADD",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let set_id = d["id"].as_str().unwrap();

    // Add a line first so we can activate
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "level_number": 1,
                        "level_name": "Level 1",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Activate
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/activate", set_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to add line to active set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "level_number": 2,
                        "level_name": "Level 2",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_letter_set_lines() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "LIST-LINES-SET",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let set_id = d["id"].as_str().unwrap();

    // Add two lines
    for level in 1..=2 {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "level_number": level,
                            "level_name": format!("Level {}", level),
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["data"].as_array().unwrap().len(), 2);
}

// ========================================================================
// Dunning Profile Tests
// ========================================================================

#[tokio::test]
async fn test_create_profile() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "customer_name": "Acme Corporation",
                        "minimum_overdue_amount": "500.00",
                        "contact_name": "John Doe",
                        "contact_email": "john@acme.com",
                        "preferred_delivery_method": "email",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["customer_name"], "Acme Corporation");
    assert_eq!(d["dunning_status"], "enabled");
}

#[tokio::test]
async fn test_duplicate_profile_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "customer_id": "00000000-0000-0000-0000-000000000200",
        "customer_name": "Dup Customer",
    }))
    .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_profiles() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/dunning/profiles")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_hold_and_enable_profile() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create profile
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000300",
                        "customer_name": "Hold Test Customer",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();
    assert_eq!(d["dunning_status"], "enabled");

    // Hold
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/profiles/{}/hold", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Customer dispute in progress",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["dunning_status"], "hold");
    assert_eq!(d["hold_reason"], "Customer dispute in progress");

    // Re-enable
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/profiles/{}/enable", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["dunning_status"], "enabled");
}

#[tokio::test]
async fn test_disable_profile() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000400",
                        "customer_name": "Disable Test Customer",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/profiles/{}/disable", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Customer closed account",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["dunning_status"], "disabled");
}

// ========================================================================
// Dunning Run Tests
// ========================================================================

#[tokio::test]
async fn test_create_run() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/runs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "run_number": "DUN-RUN-001",
                        "description": "Monthly dunning run",
                        "run_date": "2025-03-01",
                        "currency_code": "USD",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["run_number"], "DUN-RUN-001");
    assert_eq!(d["status"], "draft");
}

#[tokio::test]
async fn test_create_run_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/runs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "run_number": "",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_runs() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/dunning/runs")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cancel_run() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/runs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "run_number": "DUN-CANCEL-01",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/runs/{}/cancel", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

// ========================================================================
// Full Workflow Test: Set → Lines → Activate → Profile → Run → Submit → Results → Complete
// ========================================================================

#[tokio::test]
async fn test_full_workflow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create letter set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/letter-sets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "set_name": "WF-DUNNING-SET",
                        "description": "Workflow test dunning set",
                        "number_of_levels": 3,
                        "minimum_overdue_days": 15,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let set_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // 2. Add lines
    for (level, name, min_days) in [
        (1, "Friendly Reminder", 15),
        (2, "First Notice", 45),
        (3, "Final Demand", 90),
    ] {
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "level_number": level,
                            "level_name": name,
                            "min_days_overdue": min_days,
                            "delivery_method": "both",
                            "apply_credit_hold": level == 3,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // 3. Verify lines
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/lines", set_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = d["data"].as_array().unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["level_name"], "Friendly Reminder");
    assert_eq!(lines[2]["apply_credit_hold"], true);

    // 4. Activate letter set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/letter-sets/{}/activate", set_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");

    // 5. Create dunning profile for customer
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000500",
                        "customer_name": "Workflow Test Customer",
                        "letter_set_id": set_id,
                        "minimum_overdue_amount": "100.00",
                        "contact_name": "Jane Smith",
                        "contact_email": "jane@wftest.com",
                        "preferred_delivery_method": "email",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let profile_id = d["id"].as_str().unwrap();
    assert_eq!(d["dunning_status"], "enabled");
    assert_eq!(d["letter_set_id"], set_id);

    // 6. Create dunning run
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/runs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "run_number": "DUN-WF-001",
                        "description": "Workflow test run",
                        "letter_set_id": set_id,
                        "run_date": "2025-04-01",
                        "currency_code": "USD",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let run_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // 7. Add run result for the customer
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/runs/{}/results", run_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000500",
                        "customer_name": "Workflow Test Customer",
                        "profile_id": profile_id,
                        "dunning_level": 2,
                        "level_name": "First Notice",
                        "number_of_overdue_items": 3,
                        "total_overdue_amount": "15000.00",
                        "days_overdue": 50,
                        "delivery_method": "email",
                        "status": "pending",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let result_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "pending");
    assert_eq!(d["dunning_level"], 2);

    // 8. Submit run
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/runs/{}/submit", run_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "submitted");

    // 9. Verify run stats were updated
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/dunning/runs/{}", run_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_customers"], 1);
    assert_eq!(d["total_letters_generated"], 1);
    assert_eq!(d["total_overdue_amount"], "15000.00");

    // 10. Mark result as sent
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/results/{}/send", result_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "delivery_confirmation": "EMAIL-CONF-001",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "sent");
    assert_eq!(d["delivery_confirmation"], "EMAIL-CONF-001");

    // 11. Verify profile was updated with dunning tracking
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/dunning/profiles/{}", profile_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["last_dunning_level"], 2);
    assert_eq!(d["dunning_letter_count"], 1);
    assert!(d["last_dunning_date"].is_string());

    // 12. Complete run
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/runs/{}/complete", run_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "completed");

    // 13. Deactivate the letter set
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/dunning/letter-sets/{}/deactivate",
                    set_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "inactive");
}

// ========================================================================
// Run Result Failure Test
// ========================================================================

#[tokio::test]
async fn test_mark_result_failed() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create run
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/runs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "run_number": "DUN-FAIL-RUN",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let run_id = d["id"].as_str().unwrap();

    // Create profile
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dunning/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000600",
                        "customer_name": "Fail Test Customer",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let profile_id = d["id"].as_str().unwrap();

    // Add result
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/runs/{}/results", run_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "customer_id": "00000000-0000-0000-0000-000000000600",
                        "customer_name": "Fail Test Customer",
                        "profile_id": profile_id,
                        "dunning_level": 1,
                        "level_name": "Reminder",
                        "status": "pending",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let result_id = d["id"].as_str().unwrap();

    // Mark as failed
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/dunning/results/{}/fail", result_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Email delivery failed - bounced",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "failed");
    assert_eq!(d["reason"], "Email delivery failed - bounced");
}

// ========================================================================
// Dashboard Test
// ========================================================================

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/dunning/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Verify dashboard has expected keys (values may vary due to parallel tests)
    assert!(d.get("total_letter_sets").is_some());
    assert!(d.get("total_profiles").is_some());
    assert!(d.get("total_runs").is_some());
    assert!(d.get("active_letter_sets").is_some());
    assert!(d.get("enabled_profiles").is_some());
    assert!(d.get("letters_sent").is_some());
    assert!(d.get("letters_failed").is_some());
}
