//! Approval Authority Limits E2E Tests
//!
//! Tests for Oracle Fusion Cloud BPM > Document Approval Limits:
//! - Create limits (user-type, role-type)
//! - Get/list limits with filters
//! - Activate/deactivate limits
//! - Delete limits
//! - Authority checking (approved, denied, no-limit)
//! - Check audit trail
//! - Dashboard
//! - Validation edge cases (missing fields, bad types, duplicate codes, etc.)

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

/// Helper: create a user-type authority limit
async fn create_user_limit(
    app: &axum::Router,
    code: &str,
    user_id: &Uuid,
    doc_type: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": code,
            "name": format!("{} limit", code),
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": doc_type,
            "approvalLimitAmount": amount,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating user limit");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Helper: create a role-type authority limit
async fn create_role_limit(
    app: &axum::Router,
    code: &str,
    role: &str,
    doc_type: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": code,
            "name": format!("{} limit", code),
            "ownerType": "role",
            "roleName": role,
            "documentType": doc_type,
            "approvalLimitAmount": amount,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating role limit");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Create Limit Tests
// ============================================================================

#[tokio::test]
async fn test_create_user_limit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "UL-001", &user_id, "purchase_order", "10000").await;
    assert_eq!(limit["limitCode"], "UL-001");
    assert_eq!(limit["ownerType"], "user");
    assert_eq!(limit["userId"], user_id.to_string());
    assert_eq!(limit["documentType"], "purchase_order");
    assert_eq!(limit["approvalLimitAmount"], "10000");
    assert_eq!(limit["status"], "active");
    assert!(limit["id"].is_string());
}

#[tokio::test]
async fn test_create_role_limit() {
    let (_state, app) = setup_test().await;
    let limit = create_role_limit(&app, "RL-001", "finance_manager", "invoice", "50000").await;
    assert_eq!(limit["limitCode"], "RL-001");
    assert_eq!(limit["ownerType"], "role");
    assert_eq!(limit["roleName"], "finance_manager");
    assert_eq!(limit["documentType"], "invoice");
    assert_eq!(limit["approvalLimitAmount"], "50000");
}

#[tokio::test]
async fn test_create_limit_with_business_unit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let bu_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "BU-001",
            "name": "BU scoped limit",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "purchase_order",
            "approvalLimitAmount": "25000",
            "currencyCode": "USD",
            "businessUnitId": bu_id.to_string()
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let limit: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(limit["businessUnitId"], bu_id.to_string());
}

#[tokio::test]
async fn test_create_limit_with_effective_dates() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "EFF-001",
            "name": "Dated limit",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "expense_report",
            "approvalLimitAmount": "5000",
            "currencyCode": "USD",
            "effectiveFrom": "2026-01-01",
            "effectiveTo": "2026-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let limit: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(limit["effectiveFrom"], "2026-01-01");
    assert_eq!(limit["effectiveTo"], "2026-12-31");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_limit_empty_code_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "",
            "name": "Bad code",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "purchase_order",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_invalid_owner_type_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "BAD-OT",
            "name": "Bad owner type",
            "ownerType": "department",
            "documentType": "purchase_order",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_user_type_missing_user_id_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "NO-UID",
            "name": "Missing user id",
            "ownerType": "user",
            "documentType": "purchase_order",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_role_type_missing_role_name_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "NO-RN",
            "name": "Missing role name",
            "ownerType": "role",
            "documentType": "purchase_order",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_invalid_document_type_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "BAD-DT",
            "name": "Bad doc type",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "nonexistent_type",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_negative_amount_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "NEG-AMT",
            "name": "Negative amount",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "purchase_order",
            "approvalLimitAmount": "-100",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_limit_duplicate_code_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    create_user_limit(&app, "DUP-001", &user_id, "purchase_order", "10000").await;

    // Second create with same code
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "DUP-001",
            "name": "Duplicate",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "purchase_order",
            "approvalLimitAmount": "5000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_limit_invalid_dates_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-authority/limits")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "limitCode": "BAD-DATES",
            "name": "Bad dates",
            "ownerType": "user",
            "userId": user_id.to_string(),
            "documentType": "purchase_order",
            "approvalLimitAmount": "1000",
            "currencyCode": "USD",
            "effectiveFrom": "2027-01-01",
            "effectiveTo": "2026-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Get / List Limit Tests
// ============================================================================

#[tokio::test]
async fn test_get_authority_limit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "GET-001", &user_id, "purchase_order", "10000").await;
    let limit_id = limit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/approval-authority/limits/{}", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["limitCode"], "GET-001");
}

#[tokio::test]
async fn test_list_authority_limits() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    create_user_limit(&app, "LIST-A", &user_id, "purchase_order", "10000").await;
    create_role_limit(&app, "LIST-B", "manager", "invoice", "25000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/limits")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let limits = resp.as_array().unwrap();
    assert!(limits.len() >= 2, "Expected at least 2 limits, got {}", limits.len());
}

#[tokio::test]
async fn test_list_limits_filter_by_status() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    create_user_limit(&app, "STAT-001", &user_id, "purchase_order", "10000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/limits?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = limits.as_array().unwrap();
    assert!(arr.len() >= 1);
    for lim in arr {
        assert_eq!(lim["status"], "active");
    }
}

#[tokio::test]
async fn test_list_limits_filter_by_owner_type() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    create_user_limit(&app, "OT-USER", &user_id, "purchase_order", "10000").await;
    create_role_limit(&app, "OT-ROLE", "manager", "invoice", "25000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/limits?owner_type=role")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let limits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = limits.as_array().unwrap();
    assert!(arr.len() >= 1);
    for lim in arr {
        assert_eq!(lim["ownerType"], "role");
    }
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_deactivate_limit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "DEACT-001", &user_id, "purchase_order", "10000").await;
    let limit_id = limit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-authority/limits/{}/deactivate", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let deactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(deactivated["status"], "inactive");
}

#[tokio::test]
async fn test_activate_limit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "ACT-001", &user_id, "purchase_order", "10000").await;
    let limit_id = limit["id"].as_str().unwrap();

    // Deactivate first
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-authority/limits/{}/deactivate", limit_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Then activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-authority/limits/{}/activate", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

#[tokio::test]
async fn test_deactivate_already_inactive_rejected() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "DBL-DEACT", &user_id, "purchase_order", "10000").await;
    let limit_id = limit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // First deactivate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-authority/limits/{}/deactivate", limit_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Second deactivate should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-authority/limits/{}/deactivate", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_limit() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    let limit = create_user_limit(&app, "DEL-001", &user_id, "purchase_order", "10000").await;
    let limit_id = limit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/approval-authority/limits/{}", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/approval-authority/limits/{}", limit_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Authority Check Tests
// ============================================================================

#[tokio::test]
async fn test_check_authority_approved() {
    let (_state, app) = setup_test().await;
    let claims = admin_claims();
    let user_id = Uuid::parse_str(&claims.sub).unwrap();

    // Create a user limit for POs at $10,000
    create_user_limit(&app, "CHK-APPROVE", &user_id, "purchase_order", "10000").await;

    // Check for $5,000 PO – should be approved
    let (k, v) = auth_header(&claims);
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["result"], "approved");
    assert_eq!(result["checkedUserId"], user_id.to_string());
}

#[tokio::test]
async fn test_check_authority_denied_amount_exceeds() {
    let (_state, app) = setup_test().await;
    let claims = admin_claims();
    let user_id = Uuid::parse_str(&claims.sub).unwrap();

    // Create a user limit for POs at $10,000
    create_user_limit(&app, "CHK-DENY", &user_id, "purchase_order", "10000").await;

    // Check for $15,000 PO – should be denied
    let (k, v) = auth_header(&claims);
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "15000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["result"], "denied");
}

#[tokio::test]
async fn test_check_authority_denied_no_limit() {
    let (_state, app) = setup_test().await;

    // Don't create any limit for this user/role on this doc type
    let claims = user_claims();
    let (k, v) = auth_header(&claims);
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["result"], "denied");
}

#[tokio::test]
async fn test_check_authority_exact_limit() {
    let (_state, app) = setup_test().await;
    let claims = admin_claims();
    let user_id = Uuid::parse_str(&claims.sub).unwrap();

    // Create a limit at exactly $10,000
    create_user_limit(&app, "CHK-EXACT", &user_id, "purchase_order", "10000").await;

    // Check for exactly $10,000 – should be approved (<=)
    let (k, v) = auth_header(&claims);
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["result"], "approved");
}

#[tokio::test]
async fn test_check_authority_role_fallback() {
    let (_state, app) = setup_test().await;
    // User has "user" role which should match the role-based limit
    let claims = user_claims();
    let (k, v) = auth_header(&claims.clone());

    // Create a role limit for "user" role
    create_role_limit(&app, "RL-FALLBACK", "user", "expense_report", "5000").await;

    // Check – should use the role limit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "expense_report",
            "amount": "3000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["result"], "approved");
}

#[tokio::test]
async fn test_check_authority_invalid_amount_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "not-a-number"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Audit Trail Tests
// ============================================================================

#[tokio::test]
async fn test_check_audits_recorded() {
    let (_state, app) = setup_test().await;
    let claims = admin_claims();
    let user_id = Uuid::parse_str(&claims.sub).unwrap();

    create_user_limit(&app, "AUD-001", &user_id, "purchase_order", "10000").await;

    // Perform a check
    let (k, v) = auth_header(&claims);
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-authority/check")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "purchase_order",
            "amount": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List audits
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/audits")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let audits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = audits.as_array().unwrap();
    assert!(arr.len() >= 1, "Expected at least 1 audit entry");
    let found = arr.iter().any(|a| a["documentType"] == "purchase_order" && a["result"] == "approved");
    assert!(found, "Expected to find an approved audit for purchase_order");
}

#[tokio::test]
async fn test_list_audits_filter_by_result() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/audits?result=denied")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let audits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for a in audits.as_array().unwrap() {
        assert_eq!(a["result"], "denied");
    }
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_authority_dashboard() {
    let (_state, app) = setup_test().await;
    let user_id = Uuid::new_v4();
    create_user_limit(&app, "DASH-001", &user_id, "purchase_order", "10000").await;
    create_role_limit(&app, "DASH-002", "manager", "invoice", "50000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-authority/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalLimits"].as_i64().unwrap() >= 2);
    assert!(dashboard["activeLimits"].as_i64().unwrap() >= 2);
    assert!(dashboard["limitsByDocumentType"].is_object());
    assert!(dashboard["limitsByOwnerType"].is_object());
    assert!(dashboard["recentChecks"].is_array());
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_limit() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/approval-authority/limits/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
