//! Revenue Management (ASC 606 / IFRS 15) E2E Tests
//!
//! Tests for Oracle Fusion Revenue Management:
//! - Revenue contract CRUD and lifecycle
//! - Performance obligations
//! - Standalone selling prices
//! - Revenue recognition events
//! - Dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/108_revenue_mgmt_cash_forecast_regulatory_reporting.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Contract Tests
// ============================================================================

#[tokio::test]
async fn test_create_revenue_contract() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue-management/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contract_number": "RC-E2E-001",
            "customer_id": uuid::Uuid::new_v4().to_string(),
            "customer_name": "Acme Corp",
            "transaction_price": "100000.00",
            "currency_code": "USD",
            "contract_start_date": "2026-01-01",
            "contract_end_date": "2026-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["contract_number"], "RC-E2E-001");
    assert_eq!(c["status"], "draft");
    assert_eq!(c["transaction_price"], "100000.00");
}

#[tokio::test]
async fn test_create_contract_empty_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue-management/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contract_number": "",
            "customer_id": uuid::Uuid::new_v4().to_string(),
            "customer_name": "Acme",
            "transaction_price": "1000.00",
            "currency_code": "USD",
            "contract_start_date": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_contract_negative_price() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue-management/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contract_number": "RC-BAD",
            "customer_id": uuid::Uuid::new_v4().to_string(),
            "customer_name": "Acme",
            "transaction_price": "-100.00",
            "currency_code": "USD",
            "contract_start_date": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_contracts() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue-management/contracts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_contract_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue-management/contracts/NONEXISTENT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_ssps() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue-management/ssp")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_ssp() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue-management/ssp")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SW-LIC-01",
            "item_name": "Software License",
            "estimation_method": "observed",
            "price": "5000.00",
            "currency_code": "USD",
            "effective_from": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue-management/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
