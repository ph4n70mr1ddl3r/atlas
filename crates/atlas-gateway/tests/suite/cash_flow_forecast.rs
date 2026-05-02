//! Cash Flow Forecasting E2E Tests

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

#[tokio::test]
async fn test_create_forecast() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-flow-forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "forecast_number": "CF-E2E-001",
            "name": "Q2 2026 Forecast",
            "forecast_horizon": "monthly",
            "periods_out": 3,
            "start_date": "2026-04-01",
            "end_date": "2026-06-30",
            "base_currency_code": "USD",
            "opening_balance": "500000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let f: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(f["forecast_number"], "CF-E2E-001");
    assert_eq!(f["status"], "draft");
}

#[tokio::test]
async fn test_create_forecast_invalid_horizon() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-flow-forecasts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "forecast_number": "CF-BAD",
            "name": "Bad",
            "forecast_horizon": "yearly",
            "periods_out": 1,
            "start_date": "2026-01-01",
            "end_date": "2026-12-31",
            "base_currency_code": "USD",
            "opening_balance": "1000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_forecasts() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-flow-forecasts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-flow-forecasts/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
