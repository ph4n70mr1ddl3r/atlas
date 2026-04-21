//! Approval Delegation E2E Tests
//!
//! Tests for Oracle Fusion Cloud BPM Worklist > Rules > Configure Delegation:
//! - Create delegation rules (all types: all, by_category, by_role, by_entity)
//! - Get/list delegation rules
//! - Activate scheduled rules
//! - Cancel delegation rules
//! - Process scheduled rules (auto-activate / auto-expire)
//! - Find delegate for approver
//! - Record delegation history
//! - Delegation dashboard
//! - Validation edge cases (self-delegation, bad dates, bad types)

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_delegation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_rule(
    app: &axum::Router,
    delegate_to: &str,
    rule_name: &str,
    start_date: &str,
    end_date: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate_to,
            "rule_name": rule_name,
            "description": "Vacation delegation",
            "delegation_type": "all",
            "start_date": start_date,
            "end_date": end_date
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating rule");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Create Delegation Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_delegation_rule_all() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "Vacation Rule", "2026-06-01", "2026-06-15").await;
    assert_eq!(rule["ruleName"], "Vacation Rule");
    assert_eq!(rule["delegationType"], "all");
    assert_eq!(rule["delegateToId"], delegate.to_string());
    assert_eq!(rule["status"], "scheduled"); // start date is in future
    assert!(rule["id"].is_string());
}

#[tokio::test]
async fn test_create_rule_auto_activates_today() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Active Now",
            "delegation_type": "all",
            "start_date": "2020-01-01",
            "end_date": "2030-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["status"], "active");
    assert!(rule["activatedAt"].is_string());
}

#[tokio::test]
async fn test_create_rule_by_entity() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Entity Rule",
            "delegation_type": "by_entity",
            "entity_types": ["purchase_orders", "expense_reports"],
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["delegationType"], "by_entity");
    let entities = rule["entityTypes"].as_array().unwrap();
    assert_eq!(entities.len(), 2);
}

#[tokio::test]
async fn test_create_rule_by_role() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Role Rule",
            "delegation_type": "by_role",
            "roles": ["finance_manager", "procurement_manager"],
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["delegationType"], "by_role");
    let roles = rule["roles"].as_array().unwrap();
    assert_eq!(roles.len(), 2);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_rule_self_delegation_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let claims = admin_claims();
    let user_id = &claims.sub;
    let (k, v) = auth_header(&claims);
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": user_id,
            "rule_name": "Self Delegate",
            "delegation_type": "all",
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_dates_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Bad Dates",
            "delegation_type": "all",
            "start_date": "2026-06-15",
            "end_date": "2026-06-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_type_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Bad Type",
            "delegation_type": "invalid",
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_by_entity_missing_entities_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Missing Entities",
            "delegation_type": "by_entity",
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_by_role_missing_roles_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": delegate.to_string(),
            "rule_name": "Missing Roles",
            "delegation_type": "by_role",
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_delegate_id_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/approval-delegation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "delegate_to_id": "not-a-uuid",
            "rule_name": "Bad Delegate",
            "delegation_type": "all",
            "start_date": "2026-06-01",
            "end_date": "2026-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Get / List Rule Tests
// ============================================================================

#[tokio::test]
async fn test_get_delegation_rule() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "GET-Test", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/approval-delegation/rules/{}", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["ruleName"], "GET-Test");
}

#[tokio::test]
async fn test_list_delegation_rules() {
    let (_state, app) = setup_delegation_test().await;
    let delegate1 = Uuid::new_v4();
    let delegate2 = Uuid::new_v4();
    create_test_rule(&app, &delegate1.to_string(), "LIST-A", "2026-06-01", "2026-06-15").await;
    create_test_rule(&app, &delegate2.to_string(), "LIST-B", "2026-07-01", "2026-07-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-delegation/rules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let rules = resp.as_array().unwrap();
    assert!(rules.len() >= 2, "Expected at least 2 rules, got {}", rules.len());
}

#[tokio::test]
async fn test_list_rules_by_status() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    create_test_rule(&app, &delegate.to_string(), "STAT-001", "2026-06-01", "2026-06-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-delegation/rules?status=scheduled")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rules: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = rules.as_array().unwrap();
    assert!(arr.len() >= 1);
    for rule in arr {
        assert_eq!(rule["status"], "scheduled");
    }
}

#[tokio::test]
async fn test_list_my_rules() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    create_test_rule(&app, &delegate.to_string(), "MY-001", "2026-06-01", "2026-06-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-delegation/rules/my")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rules: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = rules.as_array().unwrap();
    assert!(arr.len() >= 1);
    // All should belong to the admin user
    for rule in arr {
        assert_eq!(rule["delegatorId"], "00000000-0000-0000-0000-000000000002");
    }
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_activate_scheduled_rule() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "ACT-001", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();
    assert_eq!(rule["status"], "scheduled");

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
    assert!(activated["activatedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_rule() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "CAN-001", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/cancel", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Plans changed"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["isActive"], false);
}

#[tokio::test]
async fn test_cancel_already_cancelled_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "DBL-CAN", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // First cancel
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/cancel", rule_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({"reason": "first"})).unwrap())).unwrap()
    ).await.unwrap();

    // Second cancel should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/cancel", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "second"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_active_rule_rejected() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "ACT-DBL", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/activate", rule_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Second activation should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/approval-delegation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_rule() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    let rule = create_test_rule(&app, &delegate.to_string(), "DEL-001", "2026-06-01", "2026-06-15").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/approval-delegation/rules/{}", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/approval-delegation/rules/{}", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Process Scheduled Rules Test
// ============================================================================

#[tokio::test]
async fn test_process_scheduled_rules() {
    let (_state, app) = setup_delegation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/approval-delegation/process-scheduled")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Should have activated and expired counts even if 0
    assert!(result["activated"].is_number());
    assert!(result["expired"].is_number());
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_delegation_dashboard() {
    let (_state, app) = setup_delegation_test().await;
    let delegate = Uuid::new_v4();
    create_test_rule(&app, &delegate.to_string(), "DASH-001", "2026-06-01", "2026-06-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-delegation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalScheduledRules"].as_i64().unwrap() >= 1);
    assert!(dashboard["delegationsByType"].is_object());
    assert!(dashboard["recentDelegations"].is_array());
}

// ============================================================================
// History Test
// ============================================================================

#[tokio::test]
async fn test_list_delegation_history() {
    let (_state, app) = setup_delegation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/approval-delegation/history?limit=10")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let history: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(history.is_array());
}
