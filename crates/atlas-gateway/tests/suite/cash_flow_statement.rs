//! Cash Flow Statement E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP GL > Cash Flow Statements:
//! - Statement CRUD (create, list, get)
//! - Statement lifecycle (draft → calculated → reviewed → published → archived)
//! - Line management (add, list, remove)
//! - Validation: method, period type, category, line type, status transitions
//! - Calculation: operating + investing + financing = net change
//! - Dashboard summary
//! - Edge cases and error handling

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    sqlx::query("DELETE FROM financials.cash_flow_statement_lines")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query("DELETE FROM financials.cash_flow_statements")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS financials")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!(
        "../../../../migrations/138_cash_flow_statement.sql"
    ))
    .execute(&state.db_pool)
    .await
    .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_statement(
    app: &axum::Router,
    statement_number: &str,
    method: &str,
    period_type: &str,
    period_start: &str,
    period_end: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": statement_number,
        "method": method,
        "periodType": period_type,
        "periodStart": period_start,
        "periodEnd": period_end,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&b);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create statement: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    statement_id: &Uuid,
    line_number: i32,
    category: &str,
    line_type: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineNumber": line_number,
        "category": category,
        "description": format!("{} line", category),
        "lineType": line_type,
        "amount": amount,
        "displayOrder": line_number,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/lines",
                    statement_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&b);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to add line: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Statement CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_cash_flow_statement_indirect() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-2026-Q1",
        "indirect",
        "quarterly",
        "2026-01-01",
        "2026-03-31",
    )
    .await;

    assert_eq!(stmt["statementNumber"], "CFS-2026-Q1");
    assert_eq!(stmt["method"], "indirect");
    assert_eq!(stmt["periodType"], "quarterly");
    assert_eq!(stmt["status"], "draft");
    assert_eq!(stmt["openingCashBalance"], "0");
}

#[tokio::test]
async fn test_create_cash_flow_statement_direct() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-DIR-01",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;

    assert_eq!(stmt["method"], "direct");
    assert_eq!(stmt["periodType"], "monthly");
}

#[tokio::test]
async fn test_create_yearly_statement() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-2026",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;

    assert_eq!(stmt["periodType"], "yearly");
}

#[tokio::test]
async fn test_create_statement_duplicate_number_fails() {
    let (_state, app) = setup_test().await;
    create_statement(
        &app,
        "CFS-DUP",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": "CFS-DUP",
        "method": "indirect",
        "periodType": "monthly",
        "periodStart": "2026-02-01",
        "periodEnd": "2026-02-28",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_statement_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": "",
        "method": "direct",
        "periodType": "monthly",
        "periodStart": "2026-01-01",
        "periodEnd": "2026-01-31",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_invalid_method_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": "CFS-BAD",
        "method": "hybrid",
        "periodType": "monthly",
        "periodStart": "2026-01-01",
        "periodEnd": "2026-01-31",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_invalid_period_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": "CFS-BAD-PT",
        "method": "direct",
        "periodType": "weekly",
        "periodStart": "2026-01-01",
        "periodEnd": "2026-01-07",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_end_before_start_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statementNumber": "CFS-BAD-DT",
        "method": "direct",
        "periodType": "monthly",
        "periodStart": "2026-12-31",
        "periodEnd": "2026-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-flow-statements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_cash_flow_statement() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-GET",
        "indirect",
        "quarterly",
        "2026-01-01",
        "2026-03-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["id"], stmt["id"]);
    assert_eq!(body["statementNumber"], "CFS-GET");
}

#[tokio::test]
async fn test_get_nonexistent_statement_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_id = Uuid::new_v4();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}", fake_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_cash_flow_statements() {
    let (_state, app) = setup_test().await;
    create_statement(
        &app,
        "CFS-LIST-A",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    create_statement(
        &app,
        "CFS-LIST-B",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cash-flow-statements")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_list_statements_filter_by_method() {
    let (_state, app) = setup_test().await;
    create_statement(
        &app,
        "CFS-FLT-D",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    create_statement(
        &app,
        "CFS-FLT-I",
        "indirect",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cash-flow-statements?method=indirect")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["method"], "indirect");
}

#[tokio::test]
async fn test_list_statements_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_statement(
        &app,
        "CFS-STS-D",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cash-flow-statements?status=draft")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty());
    for item in data {
        assert_eq!(item["status"], "draft");
    }
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_statement_lines() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-LINE",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, &stmt_id, 1, "operating", "detail", "100000.00").await;
    assert_eq!(line1["category"], "operating");
    assert_eq!(line1["amount"], "100000.00");
    assert_eq!(line1["lineNumber"], 1);

    let line2 = add_line(&app, &stmt_id, 2, "investing", "detail", "-25000.00").await;
    assert_eq!(line2["category"], "investing");
    assert_eq!(line2["lineNumber"], 2);

    let line3 = add_line(&app, &stmt_id, 3, "financing", "detail", "50000.00").await;
    assert_eq!(line3["category"], "financing");
}

#[tokio::test]
async fn test_add_line_with_account_ranges() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-ACCT",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineNumber": 1,
        "category": "operating",
        "description": "Cash from customers",
        "lineType": "detail",
        "amount": "75000.00",
        "accountRangeFrom": "4000",
        "accountRangeTo": "4999",
        "displayOrder": 1,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let line: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(line["accountRangeFrom"], "4000");
    assert_eq!(line["accountRangeTo"], "4999");
}

#[tokio::test]
async fn test_add_non_cash_line() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-NC",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineNumber": 1,
        "category": "operating",
        "description": "Depreciation & amortization",
        "lineType": "detail",
        "amount": "15000.00",
        "isNonCash": true,
        "displayOrder": 1,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let line: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(line["isNonCash"], true);
}

#[tokio::test]
async fn test_add_line_invalid_category_fails() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-BAD-CAT",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineNumber": 1,
        "category": "extraordinary",
        "lineType": "detail",
        "amount": "1000.00",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_invalid_line_type_fails() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-BAD-LT",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineNumber": 1,
        "category": "operating",
        "lineType": "summary",
        "amount": "1000.00",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_statement_lines() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-LL",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &stmt_id, 1, "operating", "detail", "50000.00").await;
    add_line(&app, &stmt_id, 2, "investing", "detail", "-10000.00").await;
    add_line(&app, &stmt_id, 3, "financing", "detail", "20000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_remove_statement_line() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-RL",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, &stmt_id, 1, "operating", "detail", "50000.00").await;
    add_line(&app, &stmt_id, 2, "investing", "detail", "-10000.00").await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Remove line
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cash-flow-statements/lines/{}", line1_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify only one line remains
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Workflow / Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_calculate_statement() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-CALC",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &stmt_id, 1, "operating", "detail", "100000.00").await;
    add_line(&app, &stmt_id, 2, "investing", "detail", "-25000.00").await;
    add_line(&app, &stmt_id, 3, "financing", "detail", "50000.00").await;

    // Calculate
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    assert_eq!(body["status"], "calculated");
    // Net change = operating + investing + financing = 100000 - 25000 + 50000 = 125000
    let operating: f64 = body["operatingCashFlow"].as_str().unwrap().parse().unwrap();
    let investing: f64 = body["investingCashFlow"].as_str().unwrap().parse().unwrap();
    let financing: f64 = body["financingCashFlow"].as_str().unwrap().parse().unwrap();
    let net_change: f64 = body["netChangeInCash"].as_str().unwrap().parse().unwrap();
    let closing: f64 = body["closingCashBalance"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    assert!((operating - 100000.0).abs() < 0.01);
    assert!((investing - (-25000.0)).abs() < 0.01);
    assert!((financing - 50000.0).abs() < 0.01);
    assert!((net_change - 125000.0).abs() < 0.01);
    assert!((closing - 125000.0).abs() < 0.01); // opening (0) + net_change (125000)
}

#[tokio::test]
async fn test_calculate_skips_non_detail_lines() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-HDR",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    // Add header and subtotal lines (should be skipped in calculation)
    add_line(&app, &stmt_id, 1, "operating", "header", "0.00").await;
    add_line(&app, &stmt_id, 2, "operating", "detail", "50000.00").await;
    add_line(&app, &stmt_id, 3, "operating", "subtotal", "50000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "calculated");
    let operating: f64 = body["operatingCashFlow"].as_str().unwrap().parse().unwrap();
    assert!((operating - 50000.0).abs() < 0.01); // Only detail line counted
}

#[tokio::test]
async fn test_full_lifecycle_draft_to_archived() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-LC",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(stmt["status"], "draft");

    // Add lines
    add_line(&app, &stmt_id, 1, "operating", "detail", "200000.00").await;
    add_line(&app, &stmt_id, 2, "investing", "detail", "-50000.00").await;
    add_line(&app, &stmt_id, 3, "financing", "detail", "30000.00").await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "calculated");

    // Review
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/review", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "reviewed");

    // Publish
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/publish", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "published");

    // Archive
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/archive", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "archived");
}

#[tokio::test]
async fn test_cannot_review_draft_statement() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-REV-D",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/review", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cannot_publish_without_review() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-PUB-NR",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &stmt_id, 1, "operating", "detail", "10000.00").await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate first (required before publish)
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to publish without review
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/publish", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cannot_archive_unpublished() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-ARC-UP",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/archive", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cannot_calculate_twice() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-CALC2",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &stmt_id, 1, "operating", "detail", "50000.00").await;

    let (k, v) = auth_header(&admin_claims());

    // First calculate - OK
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Second calculate - should fail
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cannot_add_lines_to_calculated_statement() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-ADD-CALC",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &stmt_id, 1, "operating", "detail", "10000.00").await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to add line to calculated statement
    let payload = json!({
        "lineNumber": 2,
        "category": "investing",
        "lineType": "detail",
        "amount": "5000.00",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_cash_flow_statement_dashboard() {
    let (_state, app) = setup_test().await;
    create_statement(
        &app,
        "CFS-DASH-1",
        "direct",
        "monthly",
        "2026-01-01",
        "2026-01-31",
    )
    .await;
    create_statement(
        &app,
        "CFS-DASH-2",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cash-flow-statements/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    assert!(body.get("totalStatements").is_some());
    assert!(body.get("draftStatements").is_some());
    assert!(body.get("publishedStatements").is_some());
    assert!(body.get("totalOperating").is_some());
    assert!(body.get("totalInvesting").is_some());
    assert!(body.get("totalFinancing").is_some());

    let total: i32 = body["totalStatements"].as_i64().unwrap() as i32;
    assert!(
        total >= 2,
        "Expected at least 2 total statements, got {}",
        total
    );
}

// ============================================================================
// Line Types: header, detail, subtotal, total
// ============================================================================

#[tokio::test]
async fn test_all_line_types() {
    let (_state, app) = setup_test().await;
    let stmt = create_statement(
        &app,
        "CFS-TYPES",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let header = add_line(&app, &stmt_id, 1, "operating", "header", "0.00").await;
    assert_eq!(header["lineType"], "header");

    let detail = add_line(&app, &stmt_id, 2, "operating", "detail", "75000.00").await;
    assert_eq!(detail["lineType"], "detail");

    let subtotal = add_line(&app, &stmt_id, 3, "operating", "subtotal", "75000.00").await;
    assert_eq!(subtotal["lineType"], "subtotal");

    let total = add_line(&app, &stmt_id, 4, "operating", "total", "75000.00").await;
    assert_eq!(total["lineType"], "total");
}

// ============================================================================
// Full Integration Test
// ============================================================================

#[tokio::test]
async fn test_full_cash_flow_statement_integration() {
    let (_state, app) = setup_test().await;

    // 1. Create an indirect method yearly statement
    let stmt = create_statement(
        &app,
        "CFS-INT-2026",
        "indirect",
        "yearly",
        "2026-01-01",
        "2026-12-31",
    )
    .await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(stmt["status"], "draft");

    // 2. Add operating lines (indirect method starts with net income)
    add_line(&app, &stmt_id, 1, "operating", "header", "0").await; // Header: Operating Activities
    add_line(&app, &stmt_id, 2, "operating", "detail", "250000.00").await; // Net Income
    add_line(&app, &stmt_id, 3, "operating", "detail", "15000.00").await; // Depreciation
    add_line(&app, &stmt_id, 4, "operating", "detail", "-5000.00").await; // Changes in Working Capital
    add_line(&app, &stmt_id, 5, "operating", "subtotal", "260000.00").await;

    // 3. Add investing lines
    add_line(&app, &stmt_id, 6, "investing", "header", "0").await;
    add_line(&app, &stmt_id, 7, "investing", "detail", "-80000.00").await; // Equipment Purchase
    add_line(&app, &stmt_id, 8, "investing", "detail", "10000.00").await; // Asset Sale
    add_line(&app, &stmt_id, 9, "investing", "subtotal", "-70000.00").await;

    // 4. Add financing lines
    add_line(&app, &stmt_id, 10, "financing", "header", "0").await;
    add_line(&app, &stmt_id, 11, "financing", "detail", "50000.00").await; // Loan Proceeds
    add_line(&app, &stmt_id, 12, "financing", "detail", "-20000.00").await; // Dividend Payment
    add_line(&app, &stmt_id, 13, "financing", "subtotal", "30000.00").await;

    // 5. Verify all lines
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}/lines", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 13);

    // 6. Calculate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-flow-statements/{}/calculate",
                    stmt_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let calculated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(calculated["status"], "calculated");

    // Verify calculation: operating=260000, investing=-70000, financing=30000
    let op: f64 = calculated["operatingCashFlow"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let inv: f64 = calculated["investingCashFlow"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let fin: f64 = calculated["financingCashFlow"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let net: f64 = calculated["netChangeInCash"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((op - 260000.0).abs() < 0.01);
    assert!((inv - (-70000.0)).abs() < 0.01);
    assert!((fin - 30000.0).abs() < 0.01);
    assert!((net - 220000.0).abs() < 0.01);

    // 7. Review
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/review", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 8. Publish
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/publish", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 9. Verify published statement via GET
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/cash-flow-statements/{}", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let final_stmt: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(final_stmt["status"], "published");
    assert_eq!(final_stmt["statementNumber"], "CFS-INT-2026");

    // 10. Dashboard should reflect the published statement
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/cash-flow-statements/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let dashboard: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let published: i32 = dashboard["publishedStatements"].as_i64().unwrap() as i32;
    assert!(published >= 1);

    // 11. Archive
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-flow-statements/{}/archive", stmt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let archived: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(archived["status"], "archived");
}
