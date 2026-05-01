//! Impairment Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Impairment Management (IAS 36/ASC 360):
//! - Impairment indicator CRUD
//! - Impairment test lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_impairment_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
#[ignore]
async fn test_create_impairment_indicator() {
    let (_state, app) = setup_impairment_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/impairment/indicators")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "MKT-DECL",
            "name": "Significant Market Decline",
            "indicator_type": "market",
            "severity": "high",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ind: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(ind["code"], "MKT-DECL");
    assert_eq!(ind["severity"], "high");
}

#[tokio::test]
#[ignore]
async fn test_list_impairment_indicators() {
    let (_state, app) = setup_impairment_test().await;

    let (k, v) = auth_header(&admin_claims());
    for code in &["IND-1", "IND-2"] {
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/impairment/indicators")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code, "name": format!("Indicator {}", code),
                "indicator_type": "internal", "severity": "medium",
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/impairment/indicators")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_create_impairment_test() {
    let (_state, app) = setup_impairment_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/impairment/tests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Annual Goodwill Impairment Test",
            "test_type": "cash_generating_unit",
            "test_method": "value_in_use",
            "test_date": "2024-12-31",
            "carrying_amount": "5000000",
            "discount_rate": "0.10",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let test: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(test["test_method"], "value_in_use");
    assert_eq!(test["status"], "draft");
}

#[tokio::test]
#[ignore]
async fn test_impairment_dashboard() {
    let (_state, app) = setup_impairment_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/impairment/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
