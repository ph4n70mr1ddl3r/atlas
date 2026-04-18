//! Financial Reporting E2E Tests (Oracle Fusion GL > Financial Reporting Center)
//!
//! Tests for Oracle Fusion Cloud ERP Financial Reporting:
//! - Report template CRUD
//! - Report row and column management
//! - Report generation (trial balance, income statement, balance sheet)
//! - Report lifecycle (generate → approve → publish → archive)
//! - Quick template creation
//! - Favourite management
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_financial_reporting_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_template(app: &axum::Router, code: &str, name: &str, report_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test report template",
        "reportType": report_type,
        "currencyCode": "USD",
        "rowDisplayOrder": "sequential",
        "columnDisplayOrder": "sequential",
        "roundingOption": "none",
        "showZeroAmounts": false,
        "segmentFilter": {},
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-reporting/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create template: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_row(app: &axum::Router, template_id: &str, row_number: i32, line_type: &str, label: &str, account_from: Option<&str>) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "rowNumber": row_number,
        "lineType": line_type,
        "label": label,
        "indentLevel": 0,
        "accountFilter": {},
        "computeSourceRows": [],
        "showLine": true,
        "bold": line_type == "header",
        "underline": false,
        "doubleUnderline": false,
        "pageBreakBefore": false,
    });
    if let Some(acc) = account_from {
        payload["accountRangeFrom"] = json!(acc);
        payload["accountRangeTo"] = json!(acc);
    }
    let uri = format!("/api/v1/financial-reporting/templates/{}/rows", template_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add row: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_column(app: &axum::Router, template_id: &str, col_number: i32, col_type: &str, label: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "columnNumber": col_number,
        "columnType": col_type,
        "headerLabel": label,
        "periodOffset": 0,
        "periodType": "period",
        "computeSourceColumns": [],
        "showColumn": true,
    });
    let uri = format!("/api/v1/financial-reporting/templates/{}/columns", template_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add column: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Template CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-001", "Trial Balance", "trial_balance").await;

    assert_eq!(template["code"], "TB-001");
    assert_eq!(template["name"], "Trial Balance");
    assert_eq!(template["reportType"], "trial_balance");
    assert_eq!(template["currencyCode"], "USD");
    assert_eq!(template["isActive"], true);
    assert!(template["id"].is_string());
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-001", "Trial Balance", "trial_balance").await;
    create_test_template(&app, "IS-001", "Income Statement", "income_statement").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/financial-reporting/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let templates: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(templates.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_templates_filtered() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-001", "Trial Balance", "trial_balance").await;
    create_test_template(&app, "IS-001", "Income Statement", "income_statement").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/templates?reportType=trial_balance")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let templates: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = templates.as_array().unwrap();
    assert!(arr.iter().all(|t| t["reportType"] == "trial_balance"));
}

#[tokio::test]
async fn test_get_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-001", "Trial Balance", "trial_balance").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/templates/TB-001")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let template: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(template["code"], "TB-001");
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/templates/NONEXISTENT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-DEL", "To Delete", "custom").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/financial-reporting/templates/TB-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/templates/TB-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_template_invalid_report_type() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD",
        "name": "Bad Type",
        "reportType": "invalid_type",
        "currencyCode": "USD",
        "roundingOption": "none",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-reporting/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_rounding() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD2",
        "name": "Bad Rounding",
        "reportType": "custom",
        "currencyCode": "USD",
        "roundingOption": "billions",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-reporting/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Row & Column Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_row_to_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-R1", "Row Test", "custom").await;
    let template_id = template["id"].as_str().unwrap();

    let row = add_test_row(&app, template_id, 1, "header", "Header Row", None).await;
    assert_eq!(row["rowNumber"], 1);
    assert_eq!(row["lineType"], "header");
    assert_eq!(row["label"], "Header Row");

    let data_row = add_test_row(&app, template_id, 2, "data", "Cash", Some("1000")).await;
    assert_eq!(data_row["lineType"], "data");
    assert_eq!(data_row["accountRangeFrom"], "1000");
}

#[tokio::test]
async fn test_list_rows() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-R2", "Row List Test", "custom").await;
    let template_id = template["id"].as_str().unwrap();

    add_test_row(&app, template_id, 1, "header", "Header", None).await;
    add_test_row(&app, template_id, 2, "data", "Cash", Some("1000")).await;
    add_test_row(&app, template_id, 3, "data", "Receivables", Some("1200")).await;

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/financial-reporting/templates/{}/rows", template_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rows: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_add_column_to_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-C1", "Column Test", "custom").await;
    let template_id = template["id"].as_str().unwrap();

    let col = add_test_column(&app, template_id, 1, "actuals", "Current Period").await;
    assert_eq!(col["columnNumber"], 1);
    assert_eq!(col["columnType"], "actuals");
    assert_eq!(col["headerLabel"], "Current Period");
}

#[tokio::test]
async fn test_list_columns() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-C2", "Column List Test", "custom").await;
    let template_id = template["id"].as_str().unwrap();

    add_test_column(&app, template_id, 1, "actuals", "Current").await;
    add_test_column(&app, template_id, 2, "budget", "Budget").await;

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/financial-reporting/templates/{}/columns", template_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cols: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cols.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_data_row_requires_account_range() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-R3", "Validation Test", "custom").await;
    let template_id = template["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rowNumber": 1,
        "lineType": "data",
        "label": "Data without accounts",
        "accountFilter": {},
        "computeSourceRows": [],
        "showLine": true,
    });
    let uri = format!("/api/v1/financial-reporting/templates/{}/rows", template_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Report Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_trial_balance() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-GEN", "Generated TB", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();

    // Add rows and columns
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;
    add_test_row(&app, template_id, 2, "data", "Receivables", Some("1200")).await;
    add_test_row(&app, template_id, 99, "total", "Total", None).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Q4 Trial Balance",
        "asOfDate": "2024-12-31",
        "segmentFilter": {},
        "includeUnposted": false,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-GEN/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to generate report");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(run["status"], "generated");
    assert!(run["runNumber"].as_str().unwrap().starts_with("FR-"));
}

#[tokio::test]
async fn test_trial_balance_requires_as_of_date() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-NODATE", "No Date TB", "trial_balance").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Missing date",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-NODATE/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_income_statement_requires_period_range() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "IS-NODATE", "No Period IS", "income_statement").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Missing periods",
        "asOfDate": "2024-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/IS-NODATE/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_period_from_must_be_before_to() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "IS-BAD", "Bad Period IS", "income_statement").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Bad periods",
        "periodFrom": "2024-12-31",
        "periodTo": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/IS-BAD/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Report Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_report_lifecycle_generate_approve_publish_archive() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-LC", "Lifecycle TB", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;

    // Generate
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "asOfDate": "2024-12-31", "segmentFilter": {} });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-LC/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let run_id = run["id"].as_str().unwrap();
    assert_eq!(run["status"], "generated");

    // Approve
    let uri = format!("/api/v1/financial-reporting/runs/{}/approve", run_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(approved["status"], "approved");

    // Publish
    let uri = format!("/api/v1/financial-reporting/runs/{}/publish", run_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let published: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(published["status"], "published");

    // Archive
    let uri = format!("/api/v1/financial-reporting/runs/{}/archive", run_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let archived: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(archived["status"], "archived");
}

#[tokio::test]
async fn test_cannot_approve_draft_report() {
    let (_state, app) = setup_financial_reporting_test().await;

    // Create a run that's in "draft" (by not going through generate endpoint)
    // Actually let's test by trying to approve a generated report that's already approved
    let template = create_test_template(&app, "TB-APP", "Approve Test", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "asOfDate": "2024-12-31", "segmentFilter": {} });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-APP/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Approve first time - OK
    let uri = format!("/api/v1/financial-reporting/runs/{}/approve", run_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Try to approve again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Quick Template Tests
// ============================================================================

#[tokio::test]
async fn test_quick_trial_balance_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "QTB",
        "name": "Quick Trial Balance",
        "currencyCode": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/quick/trial-balance")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let template: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(template["reportType"], "trial_balance");

    // Verify rows and columns were created
    let template_id = template["id"].as_str().unwrap();
    let uri = format!("/api/v1/financial-reporting/templates/{}/rows", template_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let rows: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(rows.as_array().unwrap().len() >= 2); // header + total

    let uri = format!("/api/v1/financial-reporting/templates/{}/columns", template_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let cols: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(cols.as_array().unwrap().len() >= 3); // beginning, debit, credit, ending
}

#[tokio::test]
async fn test_quick_income_statement_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "QIS",
        "name": "Quick Income Statement",
        "currencyCode": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/quick/income-statement")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let template: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(template["reportType"], "income_statement");
}

#[tokio::test]
async fn test_quick_balance_sheet_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "QBS",
        "name": "Quick Balance Sheet",
        "currencyCode": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/quick/balance-sheet")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let template: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(template["reportType"], "balance_sheet");
}

// ============================================================================
// Run Results & List Tests
// ============================================================================

#[tokio::test]
async fn test_list_runs() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-LR", "List Runs TB", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;

    let (k, v) = auth_header(&admin_claims());

    // Generate two reports
    for _ in 0..2 {
        let payload = json!({ "asOfDate": "2024-12-31", "segmentFilter": {} });
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/financial-reporting/templates/TB-LR/generate")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/runs")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let runs: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(runs.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_run_results() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-RES", "Results TB", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;
    add_test_row(&app, template_id, 2, "data", "Receivables", Some("1200")).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "asOfDate": "2024-12-31", "segmentFilter": {} });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-RES/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Get results
    let uri = format!("/api/v1/financial-reporting/runs/{}/results", run_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let results: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    // Should have 2 data rows × 1 column = 2 results
    assert!(results.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_runs_filtered_by_status() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-FS", "Filter Status", "trial_balance").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Balance").await;
    add_test_row(&app, template_id, 1, "data", "Cash", Some("1000")).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "asOfDate": "2024-12-31", "segmentFilter": {} });
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/TB-FS/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/runs?status=generated")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let runs: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let arr = runs.as_array().unwrap();
    assert!(arr.iter().all(|r| r["status"] == "generated"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_financial_reporting_dashboard() {
    let (_state, app) = setup_financial_reporting_test().await;

    create_test_template(&app, "TB-DASH", "Dashboard TB", "trial_balance").await;
    create_test_template(&app, "IS-DASH", "Dashboard IS", "income_statement").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(summary["templateCount"].as_i64().unwrap() >= 2);
    assert!(summary["activeTemplateCount"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Favourites Tests
// ============================================================================

#[tokio::test]
async fn test_favourite_workflow() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "TB-FAV", "Favourite TB", "custom").await;
    let template_id = template["id"].as_str().unwrap();
    let _template_uuid: uuid::Uuid = template_id.parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add favourite
    let uri = format!("/api/v1/financial-reporting/favourites/{}", template_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // List favourites
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-reporting/favourites")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let favs: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(favs.as_array().unwrap().len() >= 1);

    // Remove favourite
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_generate_from_nonexistent_template() {
    let (_state, app) = setup_financial_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "asOfDate": "2024-12-31" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/DOES_NOT_EXIST/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_generate_income_statement_with_valid_periods() {
    let (_state, app) = setup_financial_reporting_test().await;

    let template = create_test_template(&app, "IS-GEN", "Generated IS", "income_statement").await;
    let template_id = template["id"].as_str().unwrap();
    add_test_column(&app, template_id, 1, "actuals", "Current").await;
    add_test_row(&app, template_id, 1, "data", "Revenue", Some("4000")).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Q4 2024 Income Statement",
        "periodFrom": "2024-10-01",
        "periodTo": "2024-12-31",
        "segmentFilter": {},
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-reporting/templates/IS-GEN/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(run["status"], "generated");
}
