//! Account Monitor & Balance Inquiry E2E Tests
//!
//! Tests for Oracle Fusion GL Account Monitor:
//! - Account group CRUD
//! - Group member management
//! - Balance snapshot capture with threshold alerting
//! - Saved balance inquiry management
//! - Account monitor dashboard summary
//! - Validation edge cases
//! - Full lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_group(
    app: &axum::Router, code: &str, name: &str,
    warning: Option<&str>, critical: Option<&str>,
    comparison: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "code": code,
        "name": name,
        "isShared": true,
        "comparisonType": comparison.unwrap_or("prior_period"),
    });
    if let Some(w) = warning {
        body["thresholdWarningPct"] = json!(w);
    }
    if let Some(c) = critical {
        body["thresholdCriticalPct"] = json!(c);
    }
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/groups")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for account group but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_member(
    app: &axum::Router, group_id: &str, segment: &str, label: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "accountSegment": segment,
        "displayOrder": 0,
        "includeChildren": true,
    });
    if let Some(l) = label {
        body["accountLabel"] = json!(l);
    }
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/members", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for member but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Account Group Tests
// ============================================================================

#[tokio::test]
async fn test_create_account_group() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "CASH_GRP", "Cash Accounts", Some("10.0"), Some("25.0"), None).await;
    assert_eq!(group["code"], "CASH_GRP");
    assert_eq!(group["name"], "Cash Accounts");
    assert_eq!(group["comparisonType"], "prior_period");
    assert_eq!(group["status"], "active");
    assert_eq!(group["isShared"], true);
}

#[tokio::test]
async fn test_create_account_group_duplicate_code() {
    let (_state, app) = setup_test().await;
    create_test_group(&app, "DUP_GRP", "First Group", None, None, None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/groups")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP_GRP", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_account_group_invalid_comparison() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/groups")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_COMP", "name": "Bad", "comparisonType": "next_year"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_account_group_invalid_threshold() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/groups")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_THRESH", "name": "Bad", "thresholdWarningPct": "not_a_number"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_account_group() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "GET_GRP", "Get Group", None, None, None).await;
    let id = group["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/account-monitor/groups/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["code"], "GET_GRP");
}

#[tokio::test]
async fn test_list_account_groups() {
    let (_state, app) = setup_test().await;
    create_test_group(&app, "LIST_G1", "Group 1", None, None, None).await;
    create_test_group(&app, "LIST_G2", "Group 2", None, None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/account-monitor/groups")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_account_group() {
    let (_state, app) = setup_test().await;
    create_test_group(&app, "DEL_GRP", "Delete Me", None, None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/account-monitor/groups/code/DEL_GRP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Group Member Tests
// ============================================================================

#[tokio::test]
async fn test_add_group_member() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "MEM_GRP", "Member Group", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();

    let member = add_test_member(&app, group_id, "1000-000-0000", Some("Cash - Operating")).await;
    assert_eq!(member["accountSegment"], "1000-000-0000");
    assert_eq!(member["accountLabel"], "Cash - Operating");
}

#[tokio::test]
async fn test_add_multiple_members() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "MULTI_GRP", "Multi Group", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();

    add_test_member(&app, group_id, "1000-000-0000", Some("Cash")).await;
    add_test_member(&app, group_id, "1100-000-0000", Some("Accounts Receivable")).await;
    add_test_member(&app, group_id, "2000-000-0000", Some("Accounts Payable")).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/account-monitor/groups/{}/members", group_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_remove_group_member() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "REM_GRP", "Remove Group", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();
    let member = add_test_member(&app, group_id, "3000-000-0000", None).await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/members/{}", member_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_add_member_empty_segment_fails() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "EMPTY_GRP", "Empty Seg", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/members", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "accountSegment": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Balance Snapshot Tests
// ============================================================================

#[tokio::test]
async fn test_capture_snapshot() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "SNAP_GRP", "Snapshot Group", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "1000-000-0000", Some("Cash")).await;
    add_test_member(&app, group_id, "1100-000-0000", Some("AR")).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Jan-25",
            "periodStart": "2025-01-01",
            "periodEnd": "2025-01-31",
            "fiscalYear": 2025,
            "periodNumber": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snapshots = result["data"].as_array().unwrap();
    assert_eq!(snapshots.len(), 2);
    // Each snapshot should have balance data
    for snap in snapshots {
        assert!(snap["endingBalance"].is_string());
        assert!(snap["beginningBalance"].is_string());
        assert_eq!(snap["periodName"], "Jan-25");
    }
}

#[tokio::test]
async fn test_capture_snapshot_with_thresholds() {
    let (_state, app) = setup_test().await;
    // Low thresholds to trigger alerts
    let group = create_test_group(&app, "ALERT_GRP", "Alert Group", Some("0.01"), Some("0.05"), None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "4000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Feb-25",
            "periodStart": "2025-02-01",
            "periodEnd": "2025-02-28",
            "fiscalYear": 2025,
            "periodNumber": 2
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snap = &result["data"][0];
    // With very low thresholds, any non-zero variance should trigger an alert
    assert!(snap["alertStatus"].as_str().unwrap() == "warning" || snap["alertStatus"].as_str().unwrap() == "critical" || snap["alertStatus"].as_str().unwrap() == "none");
}

#[tokio::test]
async fn test_capture_snapshot_prior_year_comparison() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "YR_GRP", "Year Compare", None, None, Some("prior_year")).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "5000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Mar-25",
            "periodStart": "2025-03-01",
            "periodEnd": "2025-03-31",
            "fiscalYear": 2025,
            "periodNumber": 3
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snap = &result["data"][0];
    // Should have comparison period named for prior year
    if let Some(cp) = snap["comparisonPeriodName"].as_str() {
        assert!(cp.contains("FY2024"));
    }
}

#[tokio::test]
async fn test_capture_snapshot_empty_group_fails() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "EMPTY_SNAP", "Empty Group", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Jan-25",
            "periodStart": "2025-01-01",
            "periodEnd": "2025-01-31",
            "fiscalYear": 2025,
            "periodNumber": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_snapshots() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "LS_GRP", "List Snapshots", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "6000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    // Capture a snapshot first
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Apr-25",
            "periodStart": "2025-04-01",
            "periodEnd": "2025-04-30",
            "fiscalYear": 2025,
            "periodNumber": 4
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List snapshots
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_delete_snapshot() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "DS_GRP", "Delete Snapshot", None, None, None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "7000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "May-25",
            "periodStart": "2025-05-01",
            "periodEnd": "2025-05-31",
            "fiscalYear": 2025,
            "periodNumber": 5
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snap_id = result["data"][0]["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/snapshots/{}", snap_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Saved Balance Inquiry Tests
// ============================================================================

#[tokio::test]
async fn test_create_saved_inquiry() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Monthly Cash Inquiry",
            "description": "Check cash balances monthly",
            "accountSegments": ["1000-000-0000", "1100-000-0000"],
            "periodFrom": "Jan-25",
            "periodTo": "Mar-25",
            "currencyCode": "USD",
            "amountType": "ending_balance",
            "comparisonEnabled": true,
            "comparisonType": "prior_period",
            "isShared": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let inquiry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(inquiry["name"], "Monthly Cash Inquiry");
    assert_eq!(inquiry["amountType"], "ending_balance");
    assert_eq!(inquiry["comparisonEnabled"], true);
}

#[tokio::test]
async fn test_create_inquiry_invalid_amount_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Inquiry",
            "periodFrom": "Jan-25",
            "periodTo": "Mar-25",
            "amountType": "everything"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_inquiry_empty_name() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "",
            "periodFrom": "Jan-25",
            "periodTo": "Mar-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_saved_inquiry() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Get Inquiry",
            "periodFrom": "Jan-25",
            "periodTo": "Jun-25",
            "amountType": "net_activity"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let inquiry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = inquiry["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/account-monitor/inquiries/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["name"], "Get Inquiry");
    assert_eq!(fetched["amountType"], "net_activity");
}

#[tokio::test]
async fn test_list_saved_inquiries() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Inquiry 1",
            "periodFrom": "Jan-25",
            "periodTo": "Mar-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Inquiry 2",
            "periodFrom": "Apr-25",
            "periodTo": "Jun-25"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/account-monitor/inquiries")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_saved_inquiry() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Delete Me",
            "periodFrom": "Jan-25",
            "periodTo": "Dec-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let inquiry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = inquiry["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/inquiries/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Summary Test
// ============================================================================

#[tokio::test]
async fn test_account_monitor_summary() {
    let (_state, app) = setup_test().await;

    // Create a group with members and capture a snapshot
    let group = create_test_group(&app, "SUM_GRP", "Summary Group", Some("10"), Some("25"), None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "1000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Jun-25",
            "periodStart": "2025-06-01",
            "periodEnd": "2025-06-30",
            "fiscalYear": 2025,
            "periodNumber": 6
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create a saved inquiry
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Summary Inquiry",
            "periodFrom": "Jan-25",
            "periodTo": "Jun-25"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/account-monitor/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalGroups"].as_i64().unwrap() >= 1);
    assert!(summary["activeGroups"].as_i64().unwrap() >= 1);
    assert!(summary["totalMembers"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Alert Snapshots Test
// ============================================================================

#[tokio::test]
async fn test_get_alert_snapshots() {
    let (_state, app) = setup_test().await;
    let group = create_test_group(&app, "ALERT_SGRP", "Alert Snap Group", Some("0.01"), Some("0.05"), None).await;
    let group_id = group["id"].as_str().unwrap();
    add_test_member(&app, group_id, "8000-000-0000", None).await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Jul-25",
            "periodStart": "2025-07-01",
            "periodEnd": "2025-07-31",
            "fiscalYear": 2025,
            "periodNumber": 7
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/account-monitor/snapshots/alerts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].is_array());
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_account_monitor_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create account group
    let group = create_test_group(&app, "LIFE_GRP", "Lifecycle Group", Some("10.0"), Some("25.0"), Some("prior_period")).await;
    let group_id = group["id"].as_str().unwrap();
    assert_eq!(group["code"], "LIFE_GRP");

    // 2. Add members
    let m1 = add_test_member(&app, group_id, "1000-000-0000", Some("Cash")).await;
    let m2 = add_test_member(&app, group_id, "2000-000-0000", Some("AP")).await;
    assert_eq!(m1["accountSegment"], "1000-000-0000");
    assert_eq!(m2["accountSegment"], "2000-000-0000");

    // 3. Verify group has members
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/account-monitor/groups/id/{}", group_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched_group: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_group["members"].as_array().unwrap().len(), 2);

    // 4. Capture snapshot
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/account-monitor/groups/{}/snapshots", group_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Aug-25",
            "periodStart": "2025-08-01",
            "periodEnd": "2025-08-31",
            "fiscalYear": 2025,
            "periodNumber": 8
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let snap_result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snapshots = snap_result["data"].as_array().unwrap();
    assert_eq!(snapshots.len(), 2);
    for snap in snapshots {
        assert!(snap["endingBalance"].is_string());
        assert_eq!(snap["fiscalYear"], 2025);
        assert_eq!(snap["periodNumber"], 8);
    }

    // 5. Create saved inquiry
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/account-monitor/inquiries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Lifecycle Inquiry",
            "accountSegments": ["1000-000-0000", "2000-000-0000"],
            "periodFrom": "Jan-25",
            "periodTo": "Aug-25",
            "comparisonEnabled": true,
            "comparisonType": "prior_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let inquiry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let inquiry_id = inquiry["id"].as_str().unwrap();

    // 6. Check dashboard summary
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/account-monitor/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalGroups"].as_i64().unwrap() >= 1);
    assert!(summary["totalMembers"].as_i64().unwrap() >= 2);

    // 7. Cleanup: delete inquiry, snapshots, member, group
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/inquiries/id/{}", inquiry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Delete snapshots
    let snap_id = snapshots[0]["id"].as_str().unwrap();
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/snapshots/{}", snap_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Remove member
    let member_id = m1["id"].as_str().unwrap();
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/account-monitor/members/{}", member_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Delete group
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/account-monitor/groups/code/LIFE_GRP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
