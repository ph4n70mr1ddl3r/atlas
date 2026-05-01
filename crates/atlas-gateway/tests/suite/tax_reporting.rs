//! Tax Reporting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Tax Reporting:
//! - Tax return template CRUD
//! - Tax return creation and lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_tax_reporting_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_template(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax-reporting/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("VAT Return {}", code),
            "tax_type": "vat",
            "jurisdiction_code": "US-CA",
            "filing_frequency": "quarterly",
            "return_form_number": "VAT-100",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_create_tax_template() {
    let (_state, app) = setup_tax_reporting_test().await;

    let template = create_test_template(&app, "VAT-Q1").await;
    assert_eq!(template["code"], "VAT-Q1");
    assert_eq!(template["tax_type"], "vat");
    assert_eq!(template["filing_frequency"], "quarterly");
}

#[tokio::test]
#[ignore]
async fn test_list_tax_templates() {
    let (_state, app) = setup_tax_reporting_test().await;

    create_test_template(&app, "VAT-L1").await;
    create_test_template(&app, "VAT-L2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/tax-reporting/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_create_tax_return() {
    let (_state, app) = setup_tax_reporting_test().await;

    let template = create_test_template(&app, "VAT-RET").await;
    let template_id = template["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax-reporting/returns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_id": template_id,
            "filing_period_start": "2024-01-01",
            "filing_period_end": "2024-03-31",
            "filing_due_date": "2024-04-30",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tax_return: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(tax_return["status"], "draft");
    assert_eq!(tax_return["template_name"], "VAT Return VAT-RET");
}

#[tokio::test]
#[ignore]
async fn test_list_tax_returns() {
    let (_state, app) = setup_tax_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/tax-reporting/returns")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_tax_reporting_dashboard() {
    let (_state, app) = setup_tax_reporting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/tax-reporting/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
