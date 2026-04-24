//! Data Archiving and Retention Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ILM (Information Lifecycle Management):
//! - Create/list/get/activate/deactivate/delete retention policies
//! - Create/list/get/release/delete legal holds
//! - Add/remove/list legal hold items
//! - Check legal hold status
//! - Execute archive batches
//! - Restore archived records
//! - Purge archived records
//! - Archive audit trail
//! - Dashboard
//! - Validation edge cases

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

/// Setup with pre-inserted old records for archival tests
async fn setup_test_with_old_records() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Insert old test records that will qualify for archival
    let org_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let old_date = chrono::Utc::now() - chrono::Duration::days(400);
    sqlx::query(
        "INSERT INTO test_items (organization_id, name, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)"
    )
    .bind(org_id).bind("Old record 1").bind("closed").bind(old_date)
    .execute(&state.db_pool).await.unwrap();
    sqlx::query(
        "INSERT INTO test_items (organization_id, name, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)"
    )
    .bind(org_id).bind("Old record 2").bind("closed").bind(old_date)
    .execute(&state.db_pool).await.unwrap();
    let app = build_router(state.clone());
    (state, app)
}

/// Helper: create a retention policy
async fn create_policy(
    app: &axum::Router,
    code: &str,
    entity_type: &str,
    retention_days: i32,
    action_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyCode": code,
            "name": format!("{} policy", code),
            "entityType": entity_type,
            "retentionDays": retention_days,
            "actionType": action_type,
            "purgeAfterDays": 30
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating policy");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Helper: create a legal hold
async fn create_legal_hold(
    app: &axum::Router,
    number: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/legal-holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "holdNumber": number,
            "name": name,
            "reason": "Litigation hold",
            "caseReference": "CASE-2026-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating legal hold");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Retention Policy Tests
// ============================================================================

#[tokio::test]
async fn test_create_retention_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "RP-001", "test_items", 365, "archive_then_purge").await;
    assert_eq!(policy["policyCode"], "RP-001");
    assert_eq!(policy["entityType"], "test_items");
    assert_eq!(policy["retentionDays"], 365);
    assert_eq!(policy["actionType"], "archive_then_purge");
    assert_eq!(policy["status"], "active");
    assert!(policy["id"].is_string());
}

#[tokio::test]
async fn test_create_policy_archive_only() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "RP-ARCH", "test_items", 180, "archive").await;
    assert_eq!(policy["actionType"], "archive");
    assert_eq!(policy["purgeAfterDays"], 30);
}

#[tokio::test]
async fn test_create_policy_purge_only() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "RP-PURGE", "test_items", 90, "purge").await;
    assert_eq!(policy["actionType"], "purge");
}

#[tokio::test]
async fn test_create_policy_zero_days_allowed() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "RP-ZERO", "test_items", 0, "archive").await;
    assert_eq!(policy["retentionDays"], 0);
}

#[tokio::test]
async fn test_create_policy_empty_code_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyCode": "",
            "name": "Bad",
            "entityType": "test_items",
            "retentionDays": 365,
            "actionType": "archive"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_negative_days_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyCode": "NEG-DAYS",
            "name": "Bad",
            "entityType": "test_items",
            "retentionDays": -1,
            "actionType": "archive"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_invalid_action_type_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyCode": "BAD-AT",
            "name": "Bad",
            "entityType": "test_items",
            "retentionDays": 365,
            "actionType": "destroy"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_duplicate_code_rejected() {
    let (_state, app) = setup_test().await;
    create_policy(&app, "DUP-RP", "test_items", 365, "archive").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyCode": "DUP-RP",
            "name": "Duplicate",
            "entityType": "test_items",
            "retentionDays": 365,
            "actionType": "archive"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_retention_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "GET-RP", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/policies/{}", policy_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["policyCode"], "GET-RP");
}

#[tokio::test]
async fn test_list_retention_policies() {
    let (_state, app) = setup_test().await;
    create_policy(&app, "LIST-A", "test_items", 365, "archive").await;
    create_policy(&app, "LIST-B", "invoices", 730, "archive_then_purge").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/policies")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let policies = resp.as_array().unwrap();
    assert!(policies.len() >= 2);
}

#[tokio::test]
async fn test_list_policies_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_policy(&app, "STAT-RP", "test_items", 365, "archive").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/policies?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let policies: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for p in policies.as_array().unwrap() {
        assert_eq!(p["status"], "active");
    }
}

#[tokio::test]
async fn test_activate_deactivate_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "ACT-RP", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    // Deactivate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/policies/{}/deactivate", policy_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let deactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(deactivated["status"], "inactive");

    // Activate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/policies/{}/activate", policy_id))
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
    let policy = create_policy(&app, "DBL-DEACT", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/policies/{}/deactivate", policy_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/policies/{}/deactivate", policy_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_retention_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_policy(&app, "DEL-RP", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/data-archiving/policies/{}", policy_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/policies/{}", policy_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Legal Hold Tests
// ============================================================================

#[tokio::test]
async fn test_create_legal_hold() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "LH-001", "Litigation Hold 1").await;
    assert_eq!(hold["holdNumber"], "LH-001");
    assert_eq!(hold["name"], "Litigation Hold 1");
    assert_eq!(hold["status"], "active");
    assert_eq!(hold["reason"], "Litigation hold");
    assert_eq!(hold["caseReference"], "CASE-2026-001");
    assert!(hold["id"].is_string());
}

#[tokio::test]
async fn test_create_legal_hold_empty_number_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/legal-holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "holdNumber": "",
            "name": "Bad"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_legal_hold_duplicate_number_rejected() {
    let (_state, app) = setup_test().await;
    create_legal_hold(&app, "DUP-LH", "Hold 1").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/legal-holds")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "holdNumber": "DUP-LH",
            "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_legal_hold() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "GET-LH", "Hold for get").await;
    let hold_id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}", hold_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["holdNumber"], "GET-LH");
}

#[tokio::test]
async fn test_list_legal_holds() {
    let (_state, app) = setup_test().await;
    create_legal_hold(&app, "LIST-LH1", "Hold 1").await;
    create_legal_hold(&app, "LIST-LH2", "Hold 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/legal-holds")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let holds: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(holds.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_holds_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_legal_hold(&app, "ST-LH", "Active Hold").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/legal-holds?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let holds: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for h in holds.as_array().unwrap() {
        assert_eq!(h["status"], "active");
    }
}

#[tokio::test]
async fn test_release_legal_hold() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "REL-LH", "Release Hold").await;
    let hold_id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/release", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Case settled"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let released: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(released["status"], "released");
}

#[tokio::test]
async fn test_release_already_released_rejected() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "DBL-REL", "Double Release").await;
    let hold_id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/release", hold_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({"reason": "Settled"})).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/release", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Again"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_legal_hold() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "DEL-LH", "Delete Hold").await;
    let hold_id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}", hold_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Legal Hold Items Tests
// ============================================================================

#[tokio::test]
async fn test_add_legal_hold_items() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "ITEM-LH", "Items Hold").await;
    let hold_id = hold["id"].as_str().unwrap();

    let record_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/items", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "items": [
                {"entityType": "test_items", "recordId": record_id.to_string()},
                {"entityType": "test_items", "recordId": Uuid::new_v4().to_string()}
            ]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let items: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(items.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_legal_hold_items() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "LIST-ITEM", "List Items").await;
    let hold_id = hold["id"].as_str().unwrap();

    let record_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/items", hold_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "items": [{"entityType": "test_items", "recordId": record_id.to_string()}]
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/items", hold_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let items: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(items.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_check_legal_hold() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "CHK-LH", "Check Hold").await;
    let hold_id = hold["id"].as_str().unwrap();

    let record_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Add item
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/items", hold_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "items": [{"entityType": "test_items", "recordId": record_id.to_string()}]
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Check hold - should be true
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/holds/check?entityType=test_items&recordId={}", record_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["isUnderHold"], true);

    // Check non-held record - should be false
    let other_id = Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/holds/check?entityType=test_items&recordId={}", other_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["isUnderHold"], false);
}

#[tokio::test]
async fn test_remove_legal_hold_item() {
    let (_state, app) = setup_test().await;
    let hold = create_legal_hold(&app, "RM-LH", "Remove Item").await;
    let hold_id = hold["id"].as_str().unwrap();

    let record_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}/items", hold_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "items": [{"entityType": "test_items", "recordId": record_id.to_string()}]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let items: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let item_id = items[0]["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/data-archiving/legal-holds/items/{}", item_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Archive Operations Tests
// ============================================================================

#[tokio::test]
async fn test_execute_archive() {
    let (_state, app) = setup_test_with_old_records().await;

    // Policy with 365 days retention - the old records (400 days old) should qualify
    let policy = create_policy(&app, "ARCH-RP", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    // Execute archive
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(batch["batchNumber"], "BATCH-001");
    assert_eq!(batch["status"], "completed");
    assert!(batch["archivedRecords"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_archived_records() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "LIST-ARCH", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-LIST"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/archived")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let records: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(records.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_restore_archived_record() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "REST-ARCH", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-REST"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Get archived records to find our record
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/archived?status=archived")
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let records: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = records.as_array().unwrap();
    assert!(arr.len() >= 1, "Expected at least 1 archived record");
    let archived_id = arr[0]["id"].as_str().unwrap();

    // Restore
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/archived/{}/restore", archived_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let restored: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(restored["status"], "restored");
}

#[tokio::test]
async fn test_purge_archived_record() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "PURGE-ARCH", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-PURGE"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/archived?status=archived")
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let records: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = records.as_array().unwrap();
    assert!(arr.len() >= 1, "Expected at least 1 archived record");
    let archived_id = arr[0]["id"].as_str().unwrap();

    // Purge
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/archived/{}/purge", archived_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let purged: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(purged["status"], "purged");
}

#[tokio::test]
async fn test_purge_already_restored_rejected() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "PR-REST", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-PRR"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Get archived and restore
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/archived?status=archived")
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let records: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = records.as_array().unwrap();
    assert!(arr.len() >= 1);
    let archived_id = arr[0]["id"].as_str().unwrap();

    // Restore first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/archived/{}/restore", archived_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to purge restored record - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/data-archiving/archived/{}/purge", archived_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Archive Batches
// ============================================================================

#[tokio::test]
async fn test_list_archive_batches() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "BATCH-LIST", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-LIST-001"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/batches")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batches: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(batches.as_array().unwrap().len() >= 1);
}

// ============================================================================
// Audit Trail
// ============================================================================

#[tokio::test]
async fn test_archive_audit_trail() {
    let (_state, app) = setup_test_with_old_records().await;
    let policy = create_policy(&app, "AUD-RP", "test_items", 365, "archive").await;
    let policy_id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/data-archiving/archive")
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "policyId": policy_id,
            "batchNumber": "BATCH-AUD"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Check audit entries
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/audit")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let audits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = audits.as_array().unwrap();
    assert!(arr.len() >= 1, "Expected audit entries");

    let found_archive = arr.iter().any(|a| a["operation"] == "archive" && a["result"] == "success");
    assert!(found_archive, "Expected an archive success audit entry");
}

#[tokio::test]
async fn test_list_audit_filter_by_operation() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/audit?operation=archive")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let audits: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for a in audits.as_array().unwrap() {
        assert_eq!(a["operation"], "archive");
    }
}

// ============================================================================
// Dashboard
// ============================================================================

#[tokio::test]
async fn test_data_archiving_dashboard() {
    let (_state, app) = setup_test().await;
    create_policy(&app, "DASH-RP", "test_items", 365, "archive").await;
    create_legal_hold(&app, "DASH-LH", "Dashboard Hold").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/data-archiving/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalPolicies"].as_i64().unwrap() >= 1);
    assert!(dashboard["activePolicies"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalLegalHolds"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeLegalHolds"].as_i64().unwrap() >= 1);
    assert!(dashboard["policiesByEntityType"].is_object());
    assert!(dashboard["recentAuditEntries"].is_array());
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_policy() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/policies/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_legal_hold() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/data-archiving/legal-holds/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
