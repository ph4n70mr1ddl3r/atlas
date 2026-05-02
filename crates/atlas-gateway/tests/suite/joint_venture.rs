//! Joint Venture Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Joint Venture Management:
//! - Joint venture CRUD and lifecycle (create, activate, close)
//! - Partner management
//! - AFE management and approval workflow
//! - Cost distributions
//! - Dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/081_joint_venture_management.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_venture(app: &axum::Router, number: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/joint-venture/ventures")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "venture_number": number,
            "name": name,
            "currency_code": "USD",
            "accounting_method": "proportional",
            "billing_cycle": "monthly",
            "start_date": "2025-01-01",
            "operator_name": "Operator Corp",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Joint Venture CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_joint_venture() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-001", "Gulf Coast Drilling").await;
    assert_eq!(venture["ventureNumber"], "JV-001");
    assert_eq!(venture["name"], "Gulf Coast Drilling");
    assert_eq!(venture["currencyCode"], "USD");
    assert_eq!(venture["accountingMethod"], "proportional");
    assert_eq!(venture["billingCycle"], "monthly");
    assert_eq!(venture["status"], "draft");
}

#[tokio::test]
async fn test_create_venture_invalid_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/joint-venture/ventures")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "venture_number": "BAD",
            "name": "Bad",
            "currency_code": "USD",
            "accounting_method": "invalid",
            "billing_cycle": "monthly",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_ventures() {
    let (_state, app) = setup_test().await;
    create_test_venture(&app, "JV-L1", "Venture 1").await;
    create_test_venture(&app, "JV-L2", "Venture 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/joint-venture/ventures")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_venture() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-GET", "Get Test").await;
    let id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/joint-venture/ventures/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v["ventureNumber"], "JV-GET");
}

#[tokio::test]
async fn test_get_venture_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/joint-venture/ventures/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_activate_venture() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-ACT", "Activate Test").await;
    let id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v["status"], "active");
}

#[tokio::test]
async fn test_close_venture() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-CLOSE", "Close Test").await;
    let id = venture["id"].as_str().unwrap();

    // Must activate first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/close", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v["status"], "closed");
}

// ============================================================================
// Partner Tests
// ============================================================================

#[tokio::test]
async fn test_add_partner() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-PART", "Partner Test").await;
    let venture_id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/partners", venture_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partner_id": uuid::Uuid::new_v4().to_string(),
            "partner_name": "Partner A Corp",
            "partner_type": "non_operator",
            "ownership_percentage": "40.00",
            "role": "partner",
            "effective_from": "2025-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let partner: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(partner["partnerName"], "Partner A Corp");
    assert_eq!(partner["partnerType"], "non_operator");
}

#[tokio::test]
async fn test_list_partners() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-LP", "List Partners").await;
    let venture_id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/partners", venture_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// AFE Tests
// ============================================================================

#[tokio::test]
async fn test_create_afe() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-AFE", "AFE Test").await;
    let venture_id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/afes", venture_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "afe_number": "AFE-001",
            "title": "Well Drilling Phase 1",
            "estimated_cost": "5000000.00",
            "currency_code": "USD",
            "effective_from": "2025-01-01",
            "effective_to": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let afe: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(afe["afeNumber"], "AFE-001");
    assert_eq!(afe["status"], "draft");
}

#[tokio::test]
async fn test_afe_submit_and_approve() {
    let (_state, app) = setup_test().await;
    let venture = create_test_venture(&app, "JV-AFE-AP", "AFE Approve Test").await;
    let venture_id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create AFE
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/afes", venture_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "afe_number": "AFE-AP-01",
            "title": "Pipeline Extension",
            "estimated_cost": "2500000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let afe: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let afe_id = afe["id"].as_str().unwrap();

    // Submit AFE
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/afes/{}/submit", afe_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let afe: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(afe["status"], "submitted");

    // Approve AFE
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/afes/{}/approve", afe_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let afe: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(afe["status"], "approved");
}

// ============================================================================
// Cost Distribution Tests
// ============================================================================

#[tokio::test]
async fn test_list_cost_distributions() {
    let (_state, app) = setup_test().await;
    let venture = create_and_activate_venture(&app, "JV-CDIST", "Cost Dist Test").await;
    let venture_id = venture["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/cost-distributions", venture_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().is_empty());
}

// ============================================================================
async fn create_and_activate_venture(app: &axum::Router, number: &str, name: &str) -> serde_json::Value {
    let venture = create_test_venture(app, number, name).await;
    let id = venture["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_and_activate_venture_with_partner(app: &axum::Router, number: &str, name: &str) -> serde_json::Value {
    let venture = create_and_activate_venture(app, number, name).await;
    let venture_id = venture["id"].as_str().unwrap();

    // Add a partner
    let (k, v) = auth_header(&admin_claims());
    let partner_resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/partners", venture_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partner_id": uuid::Uuid::new_v4().to_string(),
            "partner_name": "Operator Corp",
            "partner_type": "operator",
            "ownership_percentage": "60.00",
            "revenue_interest_pct": "60.00",
            "cost_bearing_pct": "60.00",
            "role": "operator",
            "billing_contact": "John Doe",
            "billing_email": "john@operator.com",
            "billing_address": "123 Main St",
            "effective_from": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    if partner_resp.status() != StatusCode::CREATED {
        let s = partner_resp.status();
        let b = axum::body::to_bytes(partner_resp.into_body(), usize::MAX).await.unwrap();
        panic!("Failed to add partner: {} - {}", s, String::from_utf8_lossy(&b));
    }

    // Add second partner
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/joint-venture/ventures/{}/partners", venture_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partner_id": uuid::Uuid::new_v4().to_string(),
            "partner_name": "Partner Corp",
            "partner_type": "non_operator",
            "ownership_percentage": "40.00",
            "revenue_interest_pct": "40.00",
            "cost_bearing_pct": "40.00",
            "role": "partner",
            "billing_contact": "Jane Smith",
            "billing_email": "jane@partner.com",
            "billing_address": "456 Oak Ave",
            "effective_from": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();

    venture
}
// ============================================================================

#[tokio::test]
async fn test_get_joint_venture_dashboard() {
    let (_state, app) = setup_test().await;
    create_test_venture(&app, "JV-DASH", "Dashboard Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/joint-venture/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Filter Tests
// ============================================================================

#[tokio::test]
async fn test_list_ventures_by_status() {
    let (_state, app) = setup_test().await;
    create_test_venture(&app, "JV-FILT1", "Filter Test 1").await;
    create_test_venture(&app, "JV-FILT2", "Filter Test 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/joint-venture/ventures?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}
