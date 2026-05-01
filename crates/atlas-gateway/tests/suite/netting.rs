//! Netting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Netting:
//! - Netting agreement CRUD
//! - Agreement activation
//! - Netting batch lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_netting_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

const PARTNER_ID: &str = "00000000-0000-0000-0000-000000000300";

async fn create_test_agreement(
    app: &axum::Router,
    agreement_number: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/netting/agreements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "agreement_number": agreement_number,
            "name": format!("Netting Agreement {}", agreement_number),
            "partner_id": PARTNER_ID,
            "partner_name": "Partner Corp",
            "currency_code": "USD",
            "netting_direction": "both",
            "settlement_method": "automatic",
            "minimum_netting_amount": "0.00",
            "auto_select_transactions": true,
            "selection_criteria": {},
            "approval_required": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create netting agreement");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_create_netting_agreement() {
    let (_state, app) = setup_netting_test().await;

    let agreement = create_test_agreement(&app, "NET-001").await;

    assert_eq!(agreement["agreement_number"], "NET-001");
    assert_eq!(agreement["currency_code"], "USD");
}

#[tokio::test]
#[ignore]
async fn test_list_netting_agreements() {
    let (_state, app) = setup_netting_test().await;

    create_test_agreement(&app, "NET-010").await;
    create_test_agreement(&app, "NET-011").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/netting/agreements")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_get_netting_agreement() {
    let (_state, app) = setup_netting_test().await;

    let agreement = create_test_agreement(&app, "NET-020").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/netting/agreements/{}", agreement_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_activate_netting_agreement() {
    let (_state, app) = setup_netting_test().await;

    let agreement = create_test_agreement(&app, "NET-030").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/netting/agreements/{}/activate", agreement_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_duplicate_agreement() {
    let (_state, app) = setup_netting_test().await;

    create_test_agreement(&app, "NET-DUP").await;

    // Try duplicate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/netting/agreements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "agreement_number": "NET-DUP",
            "name": "Duplicate",
            "partner_id": PARTNER_ID,
            "partner_name": "Partner Corp",
            "currency_code": "USD",
            "netting_direction": "both",
            "settlement_method": "automatic",
            "minimum_netting_amount": "0.00",
            "auto_select_transactions": true,
            "selection_criteria": {},
            "approval_required": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore]
async fn test_netting_dashboard() {
    let (_state, app) = setup_netting_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/netting/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
