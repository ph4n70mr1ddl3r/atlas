//! Segregation of Duties (SoD) E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Advanced Access Control:
//! - SoD rule CRUD and lifecycle
//! - Role assignment with conflict detection
//! - Preventive blocking of conflicting assignments
//! - Detective mode violation detection
//! - Mitigating control management
//! - Dashboard summary
//! - Error cases and validation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_sod_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SoD Rule CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_sod_rule() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VENDOR_AP_SPLIT",
            "name": "Vendor Creation vs AP Approval",
            "description": "Users who create vendors should not approve payments to them",
            "first_duties": ["create_vendor", "edit_vendor"],
            "second_duties": ["approve_payment", "authorize_payment"],
            "enforcement_mode": "preventive",
            "risk_level": "high"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["code"], "VENDOR_AP_SPLIT");
    assert_eq!(rule["name"], "Vendor Creation vs AP Approval");
    assert_eq!(rule["enforcement_mode"], "preventive");
    assert_eq!(rule["risk_level"], "high");
    assert_eq!(rule["is_active"], true);
}

#[tokio::test]
async fn test_create_sod_rule_with_detective_mode() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "JE_CREATE_POST",
            "name": "Journal Entry Creation vs Posting",
            "first_duties": ["create_journal_entry"],
            "second_duties": ["post_journal"],
            "enforcement_mode": "detective",
            "risk_level": "medium"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["enforcement_mode"], "detective");
    assert_eq!(rule["risk_level"], "medium");
}

#[tokio::test]
async fn test_create_sod_rule_overlap_rejected() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "OVERLAP_RULE",
            "name": "Overlapping duties",
            "first_duties": ["create_vendor"],
            "second_duties": ["create_vendor"],
            "enforcement_mode": "preventive",
            "risk_level": "high"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_sod_rule() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create first
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GET_TEST_RULE",
            "name": "Get Test Rule",
            "first_duties": ["duty_a"],
            "second_duties": ["duty_b"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // Get it back
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/rules/GET_TEST_RULE")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["code"], "GET_TEST_RULE");
}

#[tokio::test]
async fn test_list_sod_rules() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two rules
    for (code, mode) in &[("RULE_LIST_1", "preventive"), ("RULE_LIST_2", "detective")] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/sod/rules")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("{} Rule", code),
                "first_duties": ["duty_x"],
                "second_duties": ["duty_y"],
                "enforcement_mode": mode,
                "risk_level": "medium"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // List all
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/rules")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_deactivate_sod_rule() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create a rule
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACT_DEACT_RULE",
            "name": "Activate/Deactivate Test",
            "first_duties": ["duty_a"],
            "second_duties": ["duty_b"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rule_id = rule["id"].as_str().unwrap();

    // Deactivate
    let deact_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/sod/rules/{}/deactivate", rule_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(deact_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(deact_resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["is_active"], false);

    // Reactivate
    let act_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/sod/rules/{}/activate", rule_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(act_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(act_resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["is_active"], true);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Role Assignment Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_assign_role_no_conflict() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let user_id = "00000000-0000-0000-0000-000000000100";

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "Vendor Manager",
            "duty_code": "create_vendor"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let assignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(assignment["duty_code"], "create_vendor");
    assert_eq!(assignment["role_name"], "Vendor Manager");
    assert_eq!(assignment["is_active"], true);
}

#[tokio::test]
async fn test_assign_role_preventive_blocked() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let user_id = "00000000-0000-0000-0000-000000000200";

    // Create a preventive SoD rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "PREVENT_TEST",
            "name": "Preventive Test Rule",
            "first_duties": ["create_vendor"],
            "second_duties": ["approve_payment"],
            "enforcement_mode": "preventive",
            "risk_level": "high"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Assign first duty
    let resp1 = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "Vendor Clerk",
            "duty_code": "create_vendor"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to assign conflicting duty - should be blocked
    let resp2 = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "AP Manager",
            "duty_code": "approve_payment"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(err["error"].as_str().unwrap().contains("blocked"));
}

#[tokio::test]
async fn test_assign_role_detective_allowed() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let user_id = "00000000-0000-0000-0000-000000000300";

    // Create a detective SoD rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DETECT_TEST",
            "name": "Detective Test Rule",
            "first_duties": ["create_journal"],
            "second_duties": ["post_journal"],
            "enforcement_mode": "detective",
            "risk_level": "medium"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Assign first duty
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "JE Clerk",
            "duty_code": "create_journal"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Assign conflicting duty - should be ALLOWED (detective)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "JE Poster",
            "duty_code": "post_journal"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Conflict Detection Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_check_conflict_endpoint() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let user_id = "00000000-0000-0000-0000-000000000400";

    // Create rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CHECK_CONFLICT",
            "name": "Conflict Check Rule",
            "first_duties": ["create_po"],
            "second_duties": ["approve_po"],
            "enforcement_mode": "preventive",
            "risk_level": "high"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Assign first duty
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/assignments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "role_name": "Buyer",
            "duty_code": "create_po"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Check conflict for second duty
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/check")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "duty_code": "approve_po"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["has_conflicts"], true);
    assert_eq!(result["would_be_blocked"], true);
    assert!(result["conflicts"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_check_no_conflict() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let user_id = "00000000-0000-0000-0000-000000000500";

    // Check conflict with no assignments
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/check")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "user_id": user_id,
            "duty_code": "view_reports"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["has_conflicts"], false);
    assert_eq!(result["would_be_blocked"], false);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Violation & Mitigation Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_list_violations() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/violations")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].is_array());
}

#[tokio::test]
async fn test_resolve_violation() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Try to resolve a non-existent violation
    let fake_id = uuid::Uuid::new_v4().to_string();
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/sod/violations/{}/resolve", fake_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_sod_dashboard() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["total_rules"].is_number());
    assert!(summary["active_rules"].is_number());
    assert!(summary["total_violations"].is_number());
    assert!(summary["open_violations"].is_number());
    assert!(summary["mitigated_violations"].is_number());
    assert!(summary["violations_by_risk_level"].is_object());
}

#[tokio::test]
async fn test_sod_dashboard_after_creating_rules() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create a rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DASH_RULE",
            "name": "Dashboard Test Rule",
            "first_duties": ["duty_alpha"],
            "second_duties": ["duty_beta"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Check dashboard
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["total_rules"].as_i64().unwrap() >= 1);
    assert!(summary["active_rules"].as_i64().unwrap() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation & Error Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_rule_empty_code_rejected() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Empty Code",
            "first_duties": ["a"],
            "second_duties": ["b"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_empty_duties_rejected() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "NO_DUTIES",
            "name": "No Duties",
            "first_duties": [],
            "second_duties": ["b"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_enforcement_mode() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID_MODE",
            "name": "Invalid Mode",
            "first_duties": ["a"],
            "second_duties": ["b"],
            "enforcement_mode": "invalid_mode",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_sod_rule() {
    let (_state, app) = setup_sod_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/sod/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DELETE_ME",
            "name": "To be deleted",
            "first_duties": ["x"],
            "second_duties": ["y"],
            "enforcement_mode": "detective",
            "risk_level": "low"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Delete
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri("/api/v1/sod/rules/DELETE_ME")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it's gone
    let get_resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/sod/rules/DELETE_ME")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}
