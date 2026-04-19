//! Budget Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Budget Management:
//! - Budget definition CRUD
//! - Budget version lifecycle (create → add lines → submit → approve → activate → close)
//! - Budget line management
//! - Budget transfers
//! - Budget vs actuals variance reporting
//! - Budget control checks
//! - Workflow state transitions and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_budget_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_definition(
    app: &axum::Router, code: &str, name: &str,
    budget_type: &str, control_level: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/budget/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "budget_type": budget_type,
            "control_level": control_level,
            "allow_transfers": true,
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_version(
    app: &axum::Router, budget_code: &str, label: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/definitions/{}/versions", budget_code))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "label": label,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_line(
    app: &axum::Router, version_id: &str, account_code: &str,
    period_name: Option<&str>, amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "account_code": account_code,
        "budget_amount": amount,
    });
    if let Some(pn) = period_name {
        payload["period_name"] = json!(pn);
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/lines", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Budget Definition Tests
// ============================================================================

#[tokio::test]
async fn test_create_budget_definition() {
    let (_state, app) = setup_budget_test().await;

    let def = create_test_definition(&app, "FY2024_OPEX", "FY2024 Operating Budget", "operating", "advisory").await;

    assert_eq!(def["code"], "FY2024_OPEX");
    assert_eq!(def["name"], "FY2024 Operating Budget");
    assert_eq!(def["budget_type"], "operating");
    assert_eq!(def["control_level"], "advisory");
    assert_eq!(def["is_active"], true);
}

#[tokio::test]
async fn test_list_budget_definitions() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "BUDGET1", "Budget One", "operating", "none").await;
    create_test_definition(&app, "BUDGET2", "Budget Two", "capital", "absolute").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/budget/definitions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_budget_definition() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024", "FY2024 Budget", "operating", "none").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/budget/definitions/FY2024")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(def["code"], "FY2024");
}

#[tokio::test]
async fn test_delete_budget_definition() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "TO_DELETE", "To Delete", "operating", "none").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/budget/definitions/TO_DELETE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Should no longer be retrievable
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/budget/definitions/TO_DELETE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_definition_invalid_type() {
    let (_state, app) = setup_budget_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/budget/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Budget",
            "budget_type": "invalid_type",
            "control_level": "none",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Budget Version Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_budget_version_lifecycle() {
    let (_state, app) = setup_budget_test().await;

    // Create definition
    create_test_definition(&app, "FY2024_LC", "Lifecycle Test", "operating", "none").await;

    // Create version
    let version = create_test_version(&app, "FY2024_LC", "Original").await;
    let version_id = version["id"].as_str().unwrap();
    assert_eq!(version["status"], "draft");
    assert_eq!(version["version_number"], 1);

    // Add lines
    add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "5000.00").await;
    add_test_line(&app, version_id, "6100-OFFICE", Some("Jan-2024"), "2000.00").await;

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(active["status"], "active");

    // Close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/close", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let closed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(closed["status"], "closed");
}

#[tokio::test]
async fn test_submit_empty_version_fails() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_EMPTY", "Empty Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_EMPTY", "Empty").await;
    let version_id = version["id"].as_str().unwrap();

    // Submit without any lines should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_approve_draft_fails() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_AD", "Approve Draft Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_AD", "Draft").await;
    let version_id = version["id"].as_str().unwrap();

    // Approve a draft should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_reject_submitted_version() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_REJ", "Reject Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_REJ", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    add_test_line(&app, version_id, "6000-TEST", None, "1000.00").await;

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/reject", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Budget too high"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejected_reason"], "Budget too high");
}

// ============================================================================
// Budget Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_budget_lines_and_totals() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_LINES", "Lines Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_LINES", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    let line1 = add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "5000.00").await;
    assert_eq!(line1["account_code"], "6000-TRAVEL");
    assert_eq!(line1["budget_amount"], "5000.00");
    assert_eq!(line1["line_number"], 1);

    let line2 = add_test_line(&app, version_id, "6100-OFFICE", Some("Jan-2024"), "3000.00").await;
    assert_eq!(line2["line_number"], 2);

    // Check version totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/budget/versions/{}", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Total should be 5000 + 3000 = 8000
    let total: f64 = updated["totalBudgetAmount"].as_str().unwrap().parse().unwrap();
    assert!((total - 8000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_add_line_to_non_draft_fails() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_ND", "Non-Draft Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_ND", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    // Add a line and submit + approve + activate
    add_test_line(&app, version_id, "6000-TEST", None, "1000.00").await;
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add a line to active version - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/lines", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "6200-SUPPLIES",
            "budget_amount": "500.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_duplicate_line_fails() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_DUP", "Duplicate Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_DUP", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "5000.00").await;

    // Same account + period should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/lines", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "6000-TRAVEL",
            "period_name": "Jan-2024",
            "budget_amount": "3000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_budget_line() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_DEL", "Delete Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_DEL", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    let line = add_test_line(&app, version_id, "6000-DEL", None, "1000.00").await;
    let line_id = line["id"].as_str().unwrap();

    // Delete the line
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/budget/versions/{}/lines/{}", version_id, line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Budget Transfer Tests
// ============================================================================

#[tokio::test]
async fn test_budget_transfer_workflow() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_XFER", "Transfer Test", "operating", "advisory").await;
    let version = create_test_version(&app, "FY2024_XFER", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    // Add two lines
    add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "10000.00").await;
    add_test_line(&app, version_id, "6100-OFFICE", Some("Jan-2024"), "5000.00").await;

    // Submit → Approve → Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create transfer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/transfers", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_account_code": "6000-TRAVEL",
            "from_period_name": "Jan-2024",
            "to_account_code": "6100-OFFICE",
            "to_period_name": "Jan-2024",
            "amount": "2000.00",
            "description": "Transfer surplus to office supplies",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let transfer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(transfer["status"], "pending");
    assert_eq!(transfer["amount"], "2000.00");
    let transfer_id = transfer["id"].as_str().unwrap();

    // Approve transfer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/transfers/{}/approve", transfer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_transfer_insufficient_budget_fails() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_XFAIL", "Transfer Fail Test", "operating", "advisory").await;
    let version = create_test_version(&app, "FY2024_XFAIL", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    add_test_line(&app, version_id, "6000-SMALL", Some("Jan-2024"), "1000.00").await;
    add_test_line(&app, version_id, "6100-BIG", Some("Jan-2024"), "5000.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to transfer more than available
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/transfers", version_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_account_code": "6000-SMALL",
            "from_period_name": "Jan-2024",
            "to_account_code": "6100-BIG",
            "to_period_name": "Jan-2024",
            "amount": "5000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Budget Variance Report Test
// ============================================================================

#[tokio::test]
async fn test_budget_variance_report() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_VAR", "Variance Test", "operating", "none").await;
    let version = create_test_version(&app, "FY2024_VAR", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "5000.00").await;
    add_test_line(&app, version_id, "6100-OFFICE", Some("Jan-2024"), "3000.00").await;

    // Get variance report (works on draft too)
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/budget/versions/{}/variance", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let report: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(report["definitionCode"], "FY2024_VAR");
    assert_eq!(report["versionLabel"], "Original");
    let lines = report["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2);

    let total: f64 = report["totalBudget"].as_str().unwrap().parse().unwrap();
    assert!((total - 8000.0).abs() < 0.01);
}

// ============================================================================
// Budget Control Check Test
// ============================================================================

#[tokio::test]
async fn test_budget_control_check() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_CTRL", "Control Test", "operating", "absolute").await;
    let version = create_test_version(&app, "FY2024_CTRL", "Original").await;
    let version_id = version["id"].as_str().unwrap();

    add_test_line(&app, version_id, "6000-TRAVEL", Some("Jan-2024"), "5000.00").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", version_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check within budget
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/budget/definitions/FY2024_CTRL/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "6000-TRAVEL",
            "period_name": "Jan-2024",
            "proposed_amount": 3000.00,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let check: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(check["withinBudget"], true);

    // Check over budget
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/budget/definitions/FY2024_CTRL/check")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "6000-TRAVEL",
            "period_name": "Jan-2024",
            "proposed_amount": 6000.00,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let check: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(check["withinBudget"], false);
}

// ============================================================================
// Budget Version Auto-Deactivation Test
// ============================================================================

#[tokio::test]
async fn test_activating_new_version_closes_old() {
    let (_state, app) = setup_budget_test().await;

    create_test_definition(&app, "FY2024_REPLACE", "Replace Test", "operating", "none").await;

    // Create and activate first version
    let v1 = create_test_version(&app, "FY2024_REPLACE", "Original").await;
    let v1_id = v1["id"].as_str().unwrap();
    add_test_line(&app, v1_id, "6000-TEST", None, "1000.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", v1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", v1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", v1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create and activate second version
    let v2 = create_test_version(&app, "FY2024_REPLACE", "Revised").await;
    let v2_id = v2["id"].as_str().unwrap();
    add_test_line(&app, v2_id, "6000-TEST", None, "2000.00").await;

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/submit", v2_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/approve", v2_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/budget/versions/{}/activate", v2_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // v1 should be closed
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/budget/versions/{}", v1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let v1_status: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v1_status["status"], "closed");

    // v2 should be active
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/budget/versions/{}", v2_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let v2_status: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v2_status["status"], "active");
}
