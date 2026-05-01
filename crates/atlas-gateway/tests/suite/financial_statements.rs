//! Financial Statements E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Financial Statements:
//! - Balance Sheet generation
//! - Income Statement generation
//! - Cash Flow Statement generation
//! - Statement listing

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_fs_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
#[ignore]
async fn test_generate_balance_sheet() {
    let (_state, app) = setup_fs_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "balance_sheet",
            "as_of_date": "2026-04-15",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let statement: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(statement["report_type"], "balance_sheet");
}

#[tokio::test]
#[ignore]
async fn test_generate_income_statement() {
    let (_state, app) = setup_fs_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "income_statement",
            "as_of_date": "2026-04-15",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let statement: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(statement["report_type"], "income_statement");
}

#[tokio::test]
#[ignore]
async fn test_generate_cash_flow_statement() {
    let (_state, app) = setup_fs_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "cash_flow",
            "as_of_date": "2026-04-15",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_list_financial_statements() {
    let (_state, app) = setup_fs_test().await;

    // Generate a statement first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "balance_sheet",
            "as_of_date": "2026-04-15",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List statements
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-statements")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_get_financial_statement() {
    let (_state, app) = setup_fs_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "balance_sheet",
            "as_of_date": "2026-04-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let statement: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let statement_id = statement["id"].as_str().unwrap();

    // Get the statement
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/financial-statements/{}", statement_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["report_type"], "balance_sheet");
}

#[tokio::test]
#[ignore]
async fn test_invalid_report_type() {
    let (_state, app) = setup_fs_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/financial-statements/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "report_type": "invalid_type",
            "as_of_date": "2026-04-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
