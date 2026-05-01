//! Inflation Adjustment E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Inflation Adjustment (IAS 29):
//! - Inflation index CRUD
//! - Index rate management
//! - Adjustment run lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_inflation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
#[ignore]
async fn test_create_inflation_index() {
    let (_state, app) = setup_inflation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/inflation/indices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CPI-ARG",
            "name": "Argentina Consumer Price Index",
            "country_code": "ARG",
            "currency_code": "ARS",
            "index_type": "cpi",
            "is_hyperinflationary": true,
            "hyperinflationary_start_date": "2018-07-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let idx: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(idx["code"], "CPI-ARG");
    assert_eq!(idx["country_code"], "ARG");
    assert_eq!(idx["is_hyperinflationary"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_inflation_indices() {
    let (_state, app) = setup_inflation_test().await;

    let (k, v) = auth_header(&admin_claims());
    // Create two indices
    for code in &["CPI-A", "CPI-B"] {
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/inflation/indices")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code, "name": format!("Index {}", code),
                "country_code": "USA", "currency_code": "USD",
                "index_type": "cpi", "is_hyperinflationary": false,
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/inflation/indices")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_get_inflation_index() {
    let (_state, app) = setup_inflation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/inflation/indices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CPI-GET", "name": "Get Test Index",
            "country_code": "USA", "currency_code": "USD",
            "index_type": "cpi", "is_hyperinflationary": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let idx: serde_json::Value = serde_json::from_slice(&b).unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/inflation/indices/{}", idx["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_create_adjustment_run() {
    let (_state, app) = setup_inflation_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Create index first
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/inflation/indices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CPI-RUN", "name": "Run Test Index",
            "country_code": "VEN", "currency_code": "VES",
            "index_type": "cpi", "is_hyperinflationary": true,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let idx: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let index_id = idx["id"].as_str().unwrap();

    // Create run
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/inflation/runs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "index_id": index_id,
            "name": "Q4 2024 Inflation Adjustment",
            "from_period": "2024-01-01",
            "to_period": "2024-12-31",
            "adjustment_method": "historical",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(run["adjustment_method"], "historical");
    assert_eq!(run["status"], "draft");
}

#[tokio::test]
#[ignore]
async fn test_inflation_dashboard() {
    let (_state, app) = setup_inflation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/inflation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
