//! Credit Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud Credit Management:
//! - Scoring model CRUD
//! - Credit profile lifecycle (create → score → suspend → block)
//! - Credit limit management (create → update → temp increase)
//! - Credit check rules CRUD
//! - Credit exposure calculation
//! - Credit check (pass/fail)
//! - Credit hold lifecycle (create → release/override)
//! - Credit review lifecycle (create → start → complete → approve)
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_credit_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

fn test_score_ranges() -> serde_json::Value {
    json!([
        {"min": 0, "max": 39, "label": "Very High Risk", "rating": "D"},
        {"min": 40, "max": 59, "label": "High Risk", "rating": "C"},
        {"min": 60, "max": 79, "label": "Medium Risk", "rating": "B"},
        {"min": 80, "max": 100, "label": "Low Risk", "rating": "A"}
    ])
}

fn test_scoring_criteria() -> serde_json::Value {
    json!([
        {"factor": "payment_history", "weight": 0.4, "max_score": 100},
        {"factor": "outstanding_debt", "weight": 0.3, "max_score": 100},
        {"factor": "credit_age", "weight": 0.2, "max_score": 100},
        {"factor": "credit_mix", "weight": 0.1, "max_score": 100}
    ])
}

async fn create_test_scoring_model(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/scoring-models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Model", code),
            "model_type": "scorecard",
            "scoring_criteria": test_scoring_criteria(),
            "score_ranges": test_score_ranges()
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_profile(app: &axum::Router, profile_number: &str, customer_id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/profiles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_number": profile_number,
            "profile_name": format!("Profile {}", profile_number),
            "profile_type": "customer",
            "customer_id": customer_id,
            "customer_name": format!("Customer for {}", profile_number),
            "review_frequency_days": 90
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[allow(dead_code)]
async fn delete_test_profile(app: &axum::Router, profile_id: &str) {
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/credit/profiles/{}", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
}

// ============================================================================
// Scoring Model Tests
// ============================================================================

#[tokio::test]
async fn test_create_scoring_model() {
    let (_state, app) = setup_credit_test().await;
    let model = create_test_scoring_model(&app, "SM-001").await;
    assert_eq!(model["code"], "SM-001");
    assert_eq!(model["modelType"], "scorecard");
    assert_eq!(model["isActive"], true);
    assert!(model["id"].is_string());
}

#[tokio::test]
async fn test_create_scoring_model_invalid_type() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/scoring-models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Model",
            "model_type": "invalid",
            "scoring_criteria": [],
            "score_ranges": []
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_scoring_model() {
    let (_state, app) = setup_credit_test().await;
    create_test_scoring_model(&app, "GET-SM").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/scoring-models/GET-SM")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let model: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(model["code"], "GET-SM");
}

#[tokio::test]
async fn test_list_scoring_models() {
    let (_state, app) = setup_credit_test().await;
    create_test_scoring_model(&app, "LIST-A").await;
    create_test_scoring_model(&app, "LIST-B").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/scoring-models")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_scoring_model() {
    let (_state, app) = setup_credit_test().await;
    create_test_scoring_model(&app, "DEL-SM").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/credit/scoring-models/DEL-SM")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Credit Profile Tests
// ============================================================================

#[tokio::test]
async fn test_create_profile() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "CP-001", &customer_id).await;
    assert_eq!(profile["profileNumber"], "CP-001");
    assert_eq!(profile["profileType"], "customer");
    assert_eq!(profile["status"], "active");
    assert_eq!(profile["riskLevel"], "medium");
    assert_eq!(profile["reviewFrequencyDays"], 90);
}

#[tokio::test]
async fn test_create_profile_invalid_type() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/profiles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_number": "BAD",
            "profile_name": "Bad",
            "profile_type": "supplier"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_profile_customer_missing() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/profiles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_number": "NO-CUST",
            "profile_name": "No Customer",
            "profile_type": "customer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_profile_duplicate() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    create_test_profile(&app, "DUP-001", &customer_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/profiles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_number": "DUP-001",
            "profile_name": "Duplicate",
            "profile_type": "customer",
            "customer_id": Uuid::new_v4().to_string()
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_profile_status() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "STATUS-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/profiles/{}/status", profile_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "suspended"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "suspended");
}

#[tokio::test]
async fn test_update_profile_score() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "SCORE-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/profiles/{}/score", profile_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "credit_score": "85",
            "credit_rating": "A",
            "risk_level": "low"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["creditRating"], "A");
    assert_eq!(updated["riskLevel"], "low");
}

#[tokio::test]
async fn test_list_profiles() {
    let (_state, app) = setup_credit_test().await;
    create_test_profile(&app, "LIST-PA", &Uuid::new_v4().to_string()).await;
    create_test_profile(&app, "LIST-PB", &Uuid::new_v4().to_string()).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/profiles?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_profile() {
    let (_state, app) = setup_credit_test().await;
    let profile = create_test_profile(&app, "DEL-CP", &Uuid::new_v4().to_string()).await;
    let profile_id = profile["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/credit/profiles/{}", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Credit Limit Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_update_credit_limit() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "LIM-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    // Update the default overall limit to 50000
    let (k, v) = auth_header(&admin_claims());
    let limits_resp = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/credit/profiles/{}/limits", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(limits_resp.status(), StatusCode::OK);
    let b = axum::body::to_bytes(limits_resp.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let overall_limit = limits["data"].as_array().unwrap().iter()
        .find(|l| l["limitType"] == "overall").unwrap();
    let limit_id = overall_limit["id"].as_str().unwrap();

    // Update the limit
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/credit/limits/{}", limit_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "credit_limit": "50000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(updated["creditLimit"].as_str().unwrap().starts_with("50000"), "got {:?}", updated["creditLimit"]);
}

#[tokio::test]
async fn test_set_temp_limit() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "TEMP-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let limits_resp = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/credit/profiles/{}/limits", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(limits_resp.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let limit_id = limits["data"][0]["id"].as_str().unwrap();

    // First set a base limit
    app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/credit/limits/{}", limit_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "credit_limit": "100000"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Set temp limit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/limits/{}/temp", limit_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "temp_limit_increase": "25000",
            "temp_limit_expiry": "2027-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(updated["tempLimitIncrease"].as_str().unwrap().starts_with("25000"), "got {:?}", updated["tempLimitIncrease"]);
}

#[tokio::test]
async fn test_create_additional_limit() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "ADDLIM-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "limit_type": "currency",
            "currency_code": "EUR",
            "credit_limit": "40000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let limit: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(limit["limitType"], "currency");
    assert_eq!(limit["currencyCode"], "EUR");
}

// ============================================================================
// Credit Check Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_check_rule() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/check-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Order Entry Check",
            "description": "Check credit on order entry",
            "check_point": "order_entry",
            "check_type": "automatic",
            "condition": {"min_order_amount": 1000},
            "action_on_failure": "hold",
            "priority": 10
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["name"], "Order Entry Check");
    assert_eq!(rule["checkPoint"], "order_entry");
    assert_eq!(rule["actionOnFailure"], "hold");
}

#[tokio::test]
async fn test_create_check_rule_invalid_checkpoint() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/check-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Check",
            "check_point": "invalid_point",
            "check_type": "automatic",
            "condition": {},
            "action_on_failure": "hold"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_check_rules() {
    let (_state, app) = setup_credit_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two rules
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/check-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Rule A",
            "check_point": "order_entry",
            "check_type": "automatic",
            "condition": {},
            "action_on_failure": "hold"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/check-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Rule B",
            "check_point": "shipment",
            "check_type": "manual",
            "condition": {},
            "action_on_failure": "warn"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/check-rules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Credit Exposure & Check Tests
// ============================================================================

#[tokio::test]
async fn test_calculate_exposure() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "EXP-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    // Set a credit limit first
    let (k, v) = auth_header(&admin_claims());
    let limits_resp = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/credit/profiles/{}/limits", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(limits_resp.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let limit_id = limits["data"][0]["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/credit/limits/{}", limit_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"credit_limit": "100000"})).unwrap())).unwrap()
    ).await.unwrap();

    // Calculate exposure
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/exposure/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "currency_code": "USD",
            "open_receivables": "30000",
            "open_orders": "20000",
            "open_shipments": "5000",
            "open_invoices": "10000",
            "unapplied_cash": "5000",
            "on_hold_amount": "2000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let exposure: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // total = 30000 + 20000 + 5000 + 10000 - 5000 = 60000
    assert!(exposure["totalExposure"].as_str().unwrap().starts_with("60000"), "got {:?}", exposure["totalExposure"]);
    assert!(exposure["creditLimit"].as_str().unwrap().starts_with("100000"), "got {:?}", exposure["creditLimit"]);
    assert!(exposure["availableCredit"].as_str().unwrap().starts_with("40000"), "got {:?}", exposure["availableCredit"]);
}

#[tokio::test]
async fn test_credit_check_pass() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "CHK-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    // Set a high credit limit
    let (k, v) = auth_header(&admin_claims());
    let limits_resp = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/credit/profiles/{}/limits", profile_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(limits_resp.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let limit_id = limits["data"][0]["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/credit/limits/{}", limit_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"credit_limit": "100000"})).unwrap())).unwrap()
    ).await.unwrap();

    // Perform credit check - should pass (no exposure yet)
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/exposure/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "requested_amount": "50000",
            "check_point": "order_entry"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["passed"], true);
}

#[tokio::test]
async fn test_credit_check_blocked_profile() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "BLK-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    // Block the profile
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/profiles/{}/status", profile_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "blocked"})).unwrap())).unwrap()
    ).await.unwrap();

    // Credit check on blocked profile should fail
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/exposure/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "requested_amount": "100",
            "check_point": "order_entry"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["passed"], false);
    assert!(result["reason"].as_str().unwrap().contains("blocked"));
}

// ============================================================================
// Credit Hold Tests
// ============================================================================

#[tokio::test]
async fn test_hold_lifecycle() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "HLD-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();
    let entity_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());

    // Create hold
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "hold_type": "credit_limit",
            "entity_type": "sales_order",
            "entity_id": entity_id.to_string(),
            "entity_number": "SO-12345",
            "hold_amount": "50000",
            "reason": "Credit limit exceeded"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let hold: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let hold_id = hold["id"].as_str().unwrap();
    assert_eq!(hold["status"], "active");
    assert_eq!(hold["holdType"], "credit_limit");
    assert_eq!(hold["entityType"], "sales_order");

    // Release hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/holds/{}/release", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "release_reason": "Payment received"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let released: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(released["status"], "released");
}

#[tokio::test]
async fn test_hold_override() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "OVR-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();
    let entity_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());

    // Create hold
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "hold_type": "manual",
            "entity_type": "invoice",
            "entity_id": entity_id.to_string(),
            "reason": "Manual review required"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let hold: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let hold_id = hold["id"].as_str().unwrap();

    // Override hold (requires reason)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/holds/{}/override", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "override_reason": "VP approval granted for strategic customer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let overridden: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(overridden["status"], "overridden");
}

#[tokio::test]
async fn test_hold_override_no_reason_rejected() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "OVR-NR", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();
    let entity_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());

    // Create hold
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "hold_type": "manual",
            "entity_type": "invoice",
            "entity_id": entity_id.to_string()
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let hold: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let hold_id = hold["id"].as_str().unwrap();

    // Override without reason should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/holds/{}/override", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "override_reason": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_holds() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "LH-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create two holds
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/holds")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "profile_id": profile_id,
                "hold_type": "credit_limit",
                "entity_type": "sales_order",
                "entity_id": Uuid::new_v4().to_string(),
                "reason": format!("Hold {}", i)
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/holds?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Credit Review Tests
// ============================================================================

#[tokio::test]
async fn test_review_full_lifecycle() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "REV-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create review
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/reviews")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "review_type": "periodic",
            "recommended_credit_limit": "75000",
            "due_date": "2027-03-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let review_id = review["id"].as_str().unwrap();
    assert_eq!(review["status"], "pending");
    assert_eq!(review["reviewType"], "periodic");

    // Start review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/start", review_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let started: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(started["status"], "in_review");

    // Complete review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/complete", review_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "new_score": "78",
            "new_rating": "B",
            "approved_credit_limit": "75000",
            "findings": "Customer payment history is good but some recent delays",
            "recommendations": "Increase limit to 75000, review again in 6 months",
            "reviewer_name": "Credit Analyst"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");

    // Approve review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/approve", review_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_review_reject() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "REJ-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create and complete review
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/reviews")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "review_type": "ad_hoc"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let review_id = review["id"].as_str().unwrap();

    // Start
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/start", review_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Complete
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/complete", review_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "new_score": "45",
            "new_rating": "C",
            "findings": "Too many late payments"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/reject", review_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Insufficient justification for limit increase"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_review_cancel() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "CAN-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/reviews")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "review_type": "periodic"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let review_id = review["id"].as_str().unwrap();

    // Cancel the pending review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/cancel", review_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_review_invalid_transition() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let profile = create_test_profile(&app, "INV-001", &customer_id).await;
    let profile_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/credit/reviews")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "profile_id": profile_id,
            "review_type": "periodic"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let review_id = review["id"].as_str().unwrap();

    // Can't complete a pending review (must start first)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/credit/reviews/{}/complete", review_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "new_score": "80"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_credit_dashboard() {
    let (_state, app) = setup_credit_test().await;
    let customer_id = Uuid::new_v4().to_string();
    create_test_profile(&app, "DASH-001", &customer_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/credit/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalProfiles"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeProfiles"].as_i64().unwrap() >= 1);
}
