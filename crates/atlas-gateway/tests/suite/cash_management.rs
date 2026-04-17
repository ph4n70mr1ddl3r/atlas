//! Cash Position & Cash Forecasting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Treasury Management:
//! - Cash position CRUD and summary
//! - Forecast template CRUD
//! - Forecast source CRUD
//! - Cash forecast generation lifecycle
//! - Forecast lines listing
//! - Forecast approval
//! - Error cases and validation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_cash_management_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    // Run the cash management migration
    let migration_sql = include_str!("../../../../migrations/021_cash_management.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_position(
    app: &axum::Router, bank_account_id: &str, account_number: &str,
    account_name: &str, book_balance: &str, available_balance: &str,
    position_date: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/positions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": bank_account_id,
            "account_number": account_number,
            "account_name": account_name,
            "currency_code": "USD",
            "book_balance": book_balance,
            "available_balance": available_balance,
            "float_amount": "0",
            "one_day_float": "0",
            "two_day_float": "0",
            "position_date": position_date,
            "projected_inflows": "5000",
            "projected_outflows": "3000",
            "projected_net": "2000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_template(
    app: &axum::Router, code: &str, name: &str, bucket_type: &str, periods: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "bucket_type": bucket_type,
            "number_of_periods": periods,
            "start_offset_days": 0,
            "is_default": false,
            "columns": [],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_source(
    app: &axum::Router, template_code: &str, code: &str, name: &str,
    source_type: &str, direction: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/sources")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": template_code,
            "code": code,
            "name": name,
            "source_type": source_type,
            "cash_flow_direction": direction,
            "display_order": 10,
            "lead_time_days": 0,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Cash Position Tests
// ============================================================================

#[tokio::test]
async fn test_create_cash_position() {
    let (_state, app) = setup_cash_management_test().await;

    let bank_account_id = uuid::Uuid::new_v4().to_string();
    let pos = create_test_position(
        &app, &bank_account_id, "ACC-001", "Operating Account",
        "100000.00", "95000.00", "2025-04-18",
    ).await;

    assert_eq!(pos["account_number"], "ACC-001");
    assert_eq!(pos["account_name"], "Operating Account");
    assert_eq!(pos["currency_code"], "USD");
    assert_eq!(pos["is_reconciled"], false);
}

#[tokio::test]
async fn test_list_cash_positions() {
    let (_state, app) = setup_cash_management_test().await;

    let ba1 = uuid::Uuid::new_v4().to_string();
    let ba2 = uuid::Uuid::new_v4().to_string();
    create_test_position(&app, &ba1, "ACC-001", "Operating Account", "100000.00", "95000.00", "2025-04-18").await;
    create_test_position(&app, &ba2, "ACC-002", "Savings Account", "50000.00", "48000.00", "2025-04-18").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/positions?position_date=2025-04-18")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let positions: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(positions.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_cash_position_summary() {
    let (_state, app) = setup_cash_management_test().await;

    let ba1 = uuid::Uuid::new_v4().to_string();
    let ba2 = uuid::Uuid::new_v4().to_string();
    create_test_position(&app, &ba1, "ACC-001", "Operating", "100000.00", "95000.00", "2025-04-18").await;
    create_test_position(&app, &ba2, "ACC-002", "Savings", "50000.00", "48000.00", "2025-04-18").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/positions/summary?position_date=2025-04-18")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(summary["account_count"].as_i64().unwrap() >= 2);
    // Total book balance should be 100000 + 50000 = 150000
    let total: f64 = summary["total_book_balance"].as_str().unwrap().parse().unwrap();
    assert!((total - 150000.0).abs() < 1.0);
}

#[tokio::test]
async fn test_cash_position_upsert_updates_existing() {
    let (_state, app) = setup_cash_management_test().await;

    let ba = uuid::Uuid::new_v4().to_string();
    let pos1 = create_test_position(&app, &ba, "ACC-001", "Operating", "100000.00", "95000.00", "2025-04-18").await;

    // Upsert same account/date with different balance
    let pos2 = create_test_position(&app, &ba, "ACC-001", "Operating", "120000.00", "115000.00", "2025-04-18").await;

    // Should have same ID (upserted)
    assert_eq!(pos1["id"], pos2["id"]);
}

// ============================================================================
// Forecast Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_forecast_template() {
    let (_state, app) = setup_cash_management_test().await;

    let tmpl = create_test_template(&app, "WEEKLY_CF", "Weekly Cash Forecast", "weekly", 13).await;

    assert_eq!(tmpl["code"], "WEEKLY_CF");
    assert_eq!(tmpl["name"], "Weekly Cash Forecast");
    assert_eq!(tmpl["bucket_type"], "weekly");
    assert_eq!(tmpl["number_of_periods"], 13);
    assert_eq!(tmpl["is_active"], true);
}

#[tokio::test]
async fn test_list_forecast_templates() {
    let (_state, app) = setup_cash_management_test().await;

    create_test_template(&app, "TMPL_1", "Template 1", "monthly", 12).await;
    create_test_template(&app, "TMPL_2", "Template 2", "weekly", 4).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let templates: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(templates.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_forecast_template() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "MONTHLY", "Monthly Forecast", "monthly", 12).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/templates/MONTHLY")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tmpl: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(tmpl["code"], "MONTHLY");
}

#[tokio::test]
async fn test_delete_forecast_template() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "TEMP_T", "Temporary", "daily", 7).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/cash-management/templates/TEMP_T")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify soft-deleted
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/templates/TEMP_T")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_template_invalid_bucket_type() {
    let (_state, app) = setup_cash_management_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Template",
            "bucket_type": "quarterly",
            "number_of_periods": 4,
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Forecast Source Tests
// ============================================================================

#[tokio::test]
async fn test_create_forecast_source() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "MAIN", "Main Forecast", "monthly", 12).await;

    let source = create_test_source(&app, "MAIN", "AR", "Accounts Receivable", "accounts_receivable", "inflow").await;

    assert_eq!(source["code"], "AR");
    assert_eq!(source["name"], "Accounts Receivable");
    assert_eq!(source["source_type"], "accounts_receivable");
    assert_eq!(source["cash_flow_direction"], "inflow");
    assert_eq!(source["is_active"], true);
}

#[tokio::test]
async fn test_list_forecast_sources() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "MAIN", "Main Forecast", "monthly", 12).await;
    create_test_source(&app, "MAIN", "AR", "Accounts Receivable", "accounts_receivable", "inflow").await;
    create_test_source(&app, "MAIN", "AP", "Accounts Payable", "accounts_payable", "outflow").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/sources/MAIN")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let sources: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(sources.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_forecast_source() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "MAIN", "Main Forecast", "monthly", 12).await;
    create_test_source(&app, "MAIN", "TEMP", "Temporary Source", "manual", "both").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/cash-management/sources/MAIN/TEMP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_create_source_invalid_type() {
    let (_state, app) = setup_cash_management_test().await;
    create_test_template(&app, "MAIN", "Main Forecast", "monthly", 12).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/sources")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "MAIN",
            "code": "BAD",
            "name": "Bad Source",
            "source_type": "nonexistent_type",
            "cash_flow_direction": "inflow",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Cash Forecast Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_cash_forecast() {
    let (_state, app) = setup_cash_management_test().await;

    // Setup template and sources
    create_test_template(&app, "MAIN", "Main Forecast", "monthly", 3).await;
    create_test_source(&app, "MAIN", "AR", "Accounts Receivable", "accounts_receivable", "inflow").await;
    create_test_source(&app, "MAIN", "AP", "Accounts Payable", "accounts_payable", "outflow").await;

    // Generate forecast
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "MAIN",
            "name": "Q2 2025 Cash Forecast",
            "description": "Quarterly cash flow projection",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let forecast: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(forecast["status"], "generated");
    assert_eq!(forecast["name"], "Q2 2025 Cash Forecast");
    assert_eq!(forecast["template_name"], "Main Forecast");
    assert!(forecast["is_latest"].as_bool().unwrap());

    let forecast_id = forecast["id"].as_str().unwrap();

    // Verify forecast lines were created
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/cash-management/forecasts/{}/lines", forecast_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Should have 3 periods × 2 sources = 6 lines
    assert!(lines.as_array().unwrap().len() >= 6);
}

#[tokio::test]
async fn test_approve_cash_forecast() {
    let (_state, app) = setup_cash_management_test().await;

    create_test_template(&app, "APPROVE_TEST", "Test", "monthly", 3).await;
    create_test_source(&app, "APPROVE_TEST", "AR", "AR", "accounts_receivable", "inflow").await;

    // Generate forecast
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "APPROVE_TEST",
            "name": "Test Forecast",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let forecast: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let forecast_id = forecast["id"].as_str().unwrap();

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-management/forecasts/{}/approve", forecast_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_list_forecasts() {
    let (_state, app) = setup_cash_management_test().await;

    create_test_template(&app, "LIST_TEST", "List Test", "monthly", 3).await;
    create_test_source(&app, "LIST_TEST", "AR", "AR", "accounts_receivable", "inflow").await;

    // Generate a forecast
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "LIST_TEST",
            "name": "Forecast 1",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List forecasts
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-management/forecasts?status=generated")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let forecasts: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(forecasts.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_generate_forecast_no_sources_fails() {
    let (_state, app) = setup_cash_management_test().await;

    // Create template without sources
    create_test_template(&app, "EMPTY", "Empty Template", "monthly", 3).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "EMPTY",
            "name": "Should Fail",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_forecast_nonexistent_template_fails() {
    let (_state, app) = setup_cash_management_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "NONEXISTENT",
            "name": "Should Fail",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_approve_already_approved_fails() {
    let (_state, app) = setup_cash_management_test().await;

    create_test_template(&app, "DOUBLE_APPROVE", "Test", "monthly", 3).await;
    create_test_source(&app, "DOUBLE_APPROVE", "AR", "AR", "accounts_receivable", "inflow").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_code": "DOUBLE_APPROVE",
            "name": "Test Forecast",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let forecast: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let forecast_id = forecast["id"].as_str().unwrap();

    // Approve once
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-management/forecasts/{}/approve", forecast_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    // Try to approve again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-management/forecasts/{}/approve", forecast_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Position Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_position_missing_fields_fails() {
    let (_state, app) = setup_cash_management_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-management/positions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": uuid::Uuid::new_v4().to_string(),
            "account_number": "",
            "account_name": "Test",
            "currency_code": "USD",
            "book_balance": "1000",
            "available_balance": "1000",
            "position_date": "2025-04-18",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
