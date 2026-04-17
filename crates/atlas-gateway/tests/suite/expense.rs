//! Expense Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Expense Management:
//! - Expense category CRUD
//! - Expense policy management
//! - Expense report lifecycle (create → add lines → submit → approve → reimburse)
//! - Expense line management
//! - Per-diem calculation
//! - Mileage calculation
//! - Policy validation
//! - Workflow state transitions

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_expense_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_category(
    app: &axum::Router, code: &str, name: &str,
    receipt_required: bool, receipt_threshold: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "code": code,
        "name": name,
        "receipt_required": receipt_required,
    });
    if let Some(threshold) = receipt_threshold {
        payload["receipt_threshold"] = json!(threshold);
    }
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/expense/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_policy(
    app: &axum::Router, name: &str, category_code: Option<&str>, max_amount: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "name": name,
        "violation_action": "warn",
    });
    if let Some(cc) = category_code {
        payload["category_code"] = json!(cc);
    }
    if let Some(max) = max_amount {
        payload["max_amount"] = json!(max);
    }
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/expense/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_report(
    app: &axum::Router, report_number: &str, title: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/expense/reports")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_number": report_number,
            "title": title,
            "employee_id": "00000000-0000-0000-0000-000000000002",
            "employee_name": "Test Employee",
            "currency_code": "USD",
            "purpose": "Business trip"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_line(
    app: &axum::Router, report_id: &str, amount: &str,
    expense_type: &str, category_code: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "expense_type": expense_type,
        "expense_date": "2026-04-15",
        "amount": amount,
        "description": format!("Test {} expense", expense_type),
    });
    if let Some(cc) = category_code {
        payload["category_code"] = json!(cc);
    }
    let uri = format!("/api/v1/expense/reports/{}/lines", report_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Expense Category Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_expense_category() {
    let (_state, app) = setup_expense_test().await;

    let category = create_test_category(&app, "TRAVEL", "Travel Expenses", true, Some("75.00")).await;

    assert_eq!(category["code"], "TRAVEL");
    assert_eq!(category["name"], "Travel Expenses");
    assert_eq!(category["receipt_required"], true);
    assert_eq!(category["is_active"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_expense_categories() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;
    create_test_category(&app, "MEALS", "Meals", false, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/expense/categories")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_get_expense_category() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "LODGING", "Lodging", true, Some("150.00")).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/expense/categories/LODGING")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let category: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(category["name"], "Lodging");
}

#[tokio::test]
#[ignore]
async fn test_delete_expense_category() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "MISC", "Miscellaneous", false, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri("/api/v1/expense/categories/MISC")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Expense Policy Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_expense_policy() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "MEALS", "Meals", false, None).await;
    let policy = create_test_policy(&app, "Meal Limit", Some("MEALS"), Some("100.00")).await;

    assert_eq!(policy["name"], "Meal Limit");
    assert_eq!(policy["violation_action"], "warn");
    assert_eq!(policy["is_active"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_expense_policies() {
    let (_state, app) = setup_expense_test().await;

    create_test_policy(&app, "General Policy", None, Some("5000.00")).await;
    create_test_policy(&app, "Travel Policy", None, Some("3000.00")).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/expense/policies")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Expense Report Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_expense_report_full_lifecycle() {
    let (_state, app) = setup_expense_test().await;

    // 1. Create category
    create_test_category(&app, "TRAVEL", "Travel", true, Some("75.00")).await;

    // 2. Create report
    let report = create_test_report(&app, "EXP-001", "Business Trip to NYC").await;
    let report_id = report["id"].as_str().unwrap();
    assert_eq!(report["status"], "draft");
    assert_eq!(report["total_amount"], "0");

    // 3. Add expense lines
    let line1 = add_test_line(&app, report_id, "150.00", "expense", Some("TRAVEL")).await;
    assert_eq!(line1["amount"], "150.00");
    assert_eq!(line1["expense_type"], "expense");
    assert_eq!(line1["line_number"], 1);

    let line2 = add_test_line(&app, report_id, "45.50", "expense", Some("TRAVEL")).await;
    assert_eq!(line2["line_number"], 2);

    // 4. Verify report totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/expense/reports/{}", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_report: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Total should be 150.00 + 45.50 = 195.50
    let total: f64 = updated_report["total_amount"].as_str().unwrap().parse().unwrap();
    assert!((total - 195.50).abs() < 0.01);

    // 5. Submit report
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/submit", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // 6. Approve report
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/approve", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approved_by"].is_string());

    // 7. Reimburse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/reimburse", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reimbursed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reimbursed["status"], "reimbursed");
    assert!(reimbursed["reimbursed_at"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_reject_expense_report() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;
    let report = create_test_report(&app, "EXP-002", "Trip Report").await;
    let report_id = report["id"].as_str().unwrap();

    // Add line and submit
    add_test_line(&app, report_id, "50.00", "expense", Some("TRAVEL")).await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/submit", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/reject", report_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Missing receipts"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejection_reason"], "Missing receipts");
}

#[tokio::test]
#[ignore]
async fn test_cannot_submit_empty_report() {
    let (_state, app) = setup_expense_test().await;

    let report = create_test_report(&app, "EXP-003", "Empty Report").await;
    let report_id = report["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/submit", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    // Should fail because no lines
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_add_line_to_submitted_report() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;
    let report = create_test_report(&app, "EXP-004", "Test Report").await;
    let report_id = report["id"].as_str().unwrap();

    // Add line and submit
    add_test_line(&app, report_id, "25.00", "expense", Some("TRAVEL")).await;
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/submit", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add another line - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/lines", report_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expense_type": "expense",
            "expense_date": "2026-04-16",
            "amount": "30.00"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Expense Line Management Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_expense_lines() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;
    let report = create_test_report(&app, "EXP-005", "Lines Test").await;
    let report_id = report["id"].as_str().unwrap();

    add_test_line(&app, report_id, "100.00", "expense", Some("TRAVEL")).await;
    add_test_line(&app, report_id, "50.00", "expense", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/expense/reports/{}/lines", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_delete_expense_line() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;
    let report = create_test_report(&app, "EXP-006", "Delete Line Test").await;
    let report_id = report["id"].as_str().unwrap();

    let line = add_test_line(&app, report_id, "75.00", "expense", Some("TRAVEL")).await;
    let line_id = line["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/expense/reports/{}/lines/{}", report_id, line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Listing & Filtering Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_expense_reports_by_status() {
    let (_state, app) = setup_expense_test().await;

    create_test_category(&app, "TRAVEL", "Travel", false, None).await;

    let report1 = create_test_report(&app, "EXP-010", "Report One").await;
    let report2 = create_test_report(&app, "EXP-011", "Report Two").await;

    // Submit report1
    let rid1 = report1["id"].as_str().unwrap();
    add_test_line(&app, rid1, "50.00", "expense", Some("TRAVEL")).await;
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense/reports/{}/submit", rid1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List only draft reports - should have report2
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/expense/reports?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);

    // List submitted - should have report1
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/expense/reports?status=submitted")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
}
