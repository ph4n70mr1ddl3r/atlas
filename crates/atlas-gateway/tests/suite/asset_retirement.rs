//! Asset Retirement E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Asset Retirement/Disposal:
//! - Create retirement requests (sale, scrap, donation, transfer)
//! - Workflow lifecycle: approve, complete, reverse, cancel
//! - Gain/loss calculation validation
//! - List with filtering by status and type
//! - Dashboard summary statistics
//! - Validation edge cases

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/146_asset_retirement.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_retirement(
    app: &axum::Router,
    asset_id: &str,
    retirement_type: &str,
    cost: &str,
    accumulated_depreciation: &str,
    proceeds: &str,
    removal_cost: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "assetId": asset_id,
        "assetNumber": format!("AST-{}", &asset_id[..4]),
        "assetDescription": "Test Asset",
        "retirementType": retirement_type,
        "retirementDate": "2025-06-15",
        "cost": cost,
        "accumulatedDepreciation": accumulated_depreciation,
        "proceeds": proceeds,
        "removalCost": removal_cost,
        "gainLossAccount": "GL_GAIN_LOSS",
        "assetAccount": "GL_FIXED_ASSETS",
        "depreciationAccount": "GL_ACC_DEPRECIATION",
        "proceedsAccount": "GL_CASH",
        "removalCostAccount": "GL_REMOVAL_EXPENSE",
        "buyerName": "Test Buyer Corp",
        "reason": "End of useful life"
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/asset-retirements")
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
    eprintln!("CREATE RETIREMENT RESPONSE status={}: {}", status, body_str);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create retirement: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_sale_retirement() {
    let (_state, app) = setup_test().await;
    let asset_id = "11111111-1111-1111-1111-111111111111";
    let ret = create_retirement(&app, asset_id, "sale", "30000", "18000", "15000", "500").await;

    assert_eq!(ret["retirementType"], "sale");
    assert_eq!(ret["status"], "pending");
    assert_eq!(ret["cost"], "30000");
    assert_eq!(ret["accumulatedDepreciation"], "18000");
    assert_eq!(ret["netBookValue"], "12000");
    assert_eq!(ret["proceeds"], "15000");
    assert_eq!(ret["removalCost"], "500");
    assert_eq!(ret["gainLossAmount"], "2500"); // 15000 - 12000 - 500
    assert!(ret["retirementNumber"]
        .as_str()
        .unwrap()
        .starts_with("RET-"));
}

#[tokio::test]
async fn test_create_scrap_retirement_loss() {
    let (_state, app) = setup_test().await;
    let asset_id = "22222222-2222-2222-2222-222222222222";
    let ret = create_retirement(&app, asset_id, "scrap", "5000", "3000", "0", "0").await;

    assert_eq!(ret["retirementType"], "scrap");
    assert_eq!(ret["netBookValue"], "2000");
    assert_eq!(ret["gainLossAmount"], "-2000"); // 0 - 2000 - 0 (loss)
}

#[tokio::test]
async fn test_create_donation_retirement() {
    let (_state, app) = setup_test().await;
    let asset_id = "33333333-3333-3333-3333-333333333333";
    let ret = create_retirement(&app, asset_id, "donation", "10000", "10000", "0", "0").await;

    assert_eq!(ret["retirementType"], "donation");
    assert_eq!(ret["netBookValue"], "0");
    assert_eq!(ret["gainLossAmount"], "0"); // Fully depreciated, no gain/loss
}

#[tokio::test]
async fn test_create_invalid_retirement_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "assetId": "00000000-0000-0000-0000-000000000099",
        "retirementType": "invalid",
        "retirementDate": "2025-06-15",
        "cost": "1000",
        "accumulatedDepreciation": "500",
        "proceeds": "0",
        "removalCost": "0",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/asset-retirements")
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
async fn test_create_negative_cost_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "assetId": "00000000-0000-0000-0000-000000000099",
        "retirementType": "sale",
        "retirementDate": "2025-06-15",
        "cost": "-1000",
        "accumulatedDepreciation": "500",
        "proceeds": "0",
        "removalCost": "0",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/asset-retirements")
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
async fn test_create_depreciation_exceeds_cost_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "assetId": "00000000-0000-0000-0000-000000000099",
        "retirementType": "sale",
        "retirementDate": "2025-06-15",
        "cost": "1000",
        "accumulatedDepreciation": "2000",
        "proceeds": "0",
        "removalCost": "0",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/asset-retirements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Get / List Tests
// ============================================================================

#[tokio::test]
async fn test_get_retirement() {
    let (_state, app) = setup_test().await;
    let asset_id = "44444444-4444-4444-4444-444444444444";
    let ret = create_retirement(&app, asset_id, "sale", "10000", "5000", "6000", "0").await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/asset-retirements/{}", ret_id))
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
    assert_eq!(body["id"], ret_id);
    assert_eq!(body["retirementType"], "sale");
}

#[tokio::test]
async fn test_get_retirement_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements/00000000-0000-0000-0000-000000000999")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_retirements() {
    let (_state, app) = setup_test().await;
    create_retirement(
        &app,
        "55555555-5555-5555-5555-555555555555",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    create_retirement(
        &app,
        "66666666-6666-6666-6666-666666666666",
        "scrap",
        "8000",
        "8000",
        "0",
        "0",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements")
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
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_retirements_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_retirement(
        &app,
        "77777777-7777-7777-7777-777777777771",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements?status=pending")
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
    let items = body["data"].as_array().unwrap();
    assert!(items.len() >= 1);
    assert!(items.iter().all(|i| i["status"] == "pending"));
}

#[tokio::test]
async fn test_list_retirements_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_retirement(
        &app,
        "77777777-7777-7777-7777-777777777772",
        "scrap",
        "5000",
        "3000",
        "0",
        "0",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements?retirementType=scrap")
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
    let items = body["data"].as_array().unwrap();
    assert!(items.len() >= 1);
    assert!(items.iter().all(|i| i["retirementType"] == "scrap"));
}

// ============================================================================
// Workflow Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_approve_retirement() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
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
    assert_eq!(body["status"], "approved");
    assert!(body["approvedBy"].is_string());
}

#[tokio::test]
async fn test_approve_non_pending_fails() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Approve first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to approve again
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_complete_retirement() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "sale",
        "20000",
        "12000",
        "10000",
        "200",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Approve first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Complete with GL batch
    let payload = json!({ "glBatchId": "dddddddd-dddd-dddd-dddd-dddddddddddd" });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/complete", ret_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "completed");
    assert_eq!(body["postedToGl"], true);
    assert_eq!(body["glBatchId"], "dddddddd-dddd-dddd-dddd-dddddddddddd");
}

#[tokio::test]
async fn test_complete_not_approved_fails() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({});
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/complete", ret_id))
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
async fn test_reverse_retirement() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Approve then complete
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let payload = json!({});
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/complete", ret_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Reverse
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/reverse", ret_id))
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
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_retirement() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "10101010-1010-1010-1010-101010101010",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/cancel", ret_id))
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
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_approved_retirement_fails() {
    let (_state, app) = setup_test().await;
    let ret = create_retirement(
        &app,
        "11111111-2222-3333-4444-555555555555",
        "sale",
        "10000",
        "5000",
        "6000",
        "0",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Approve first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to cancel approved retirement
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/cancel", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_retirement_dashboard() {
    let (_state, app) = setup_test().await;
    create_retirement(
        &app,
        "aaaa0000-0000-0000-0000-000000000001",
        "sale",
        "50000",
        "30000",
        "25000",
        "1000",
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements/dashboard")
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

    assert!(body.get("totalRetirements").is_some());
    assert!(body.get("pendingRetirements").is_some());
    assert!(body.get("completedRetirements").is_some());
    assert!(body.get("totalProceeds").is_some());
    assert!(body.get("totalGain").is_some());
    assert!(body.get("totalLoss").is_some());
    assert!(body.get("byType").is_some());
    assert!(body["totalRetirements"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Gain/Loss Calculation Tests
// ============================================================================

#[tokio::test]
async fn test_gain_on_sale() {
    let (_state, app) = setup_test().await;
    // Cost 50000, Acc Dep 30000, NBV 20000, Proceeds 25000, Removal 1000
    // Gain = 25000 - 20000 - 1000 = 4000
    let ret = create_retirement(
        &app,
        "b0000000-0000-0000-0000-000000000001",
        "sale",
        "50000",
        "30000",
        "25000",
        "1000",
    )
    .await;
    assert_eq!(ret["gainLossAmount"], "4000");
}

#[tokio::test]
async fn test_loss_on_scrap() {
    let (_state, app) = setup_test().await;
    // Cost 10000, Acc Dep 6000, NBV 4000, Proceeds 0, Removal 500
    // Loss = 0 - 4000 - 500 = -4500
    let ret = create_retirement(
        &app,
        "b0000000-0000-0000-0000-000000000002",
        "scrap",
        "10000",
        "6000",
        "0",
        "500",
    )
    .await;
    assert_eq!(ret["gainLossAmount"], "-4500");
}

#[tokio::test]
async fn test_no_gain_loss_fully_depreciated() {
    let (_state, app) = setup_test().await;
    // Cost 10000, Acc Dep 10000, NBV 0, Proceeds 0, Removal 0
    // Gain/Loss = 0 - 0 - 0 = 0
    let ret = create_retirement(
        &app,
        "b0000000-0000-0000-0000-000000000003",
        "donation",
        "10000",
        "10000",
        "0",
        "0",
    )
    .await;
    assert_eq!(ret["gainLossAmount"], "0");
}

// ============================================================================
// End-to-End Full Flow Test
// ============================================================================

#[tokio::test]
async fn test_end_to_end_retirement_flow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create a sale retirement
    let ret = create_retirement(
        &app,
        "e2e00000-0000-0000-0000-000000000001",
        "sale",
        "80000",
        "50000",
        "35000",
        "2000",
    )
    .await;
    let ret_id = ret["id"].as_str().unwrap();

    // Verify initial state
    assert_eq!(ret["status"], "pending");
    assert_eq!(ret["netBookValue"], "30000"); // 80000 - 50000
    assert_eq!(ret["gainLossAmount"], "3000"); // 35000 - 30000 - 2000 (gain)
    assert_eq!(ret["buyerName"], "Test Buyer Corp");
    assert!(ret["retirementNumber"]
        .as_str()
        .unwrap()
        .starts_with("RET-"));

    // 2. Get the retirement by ID
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/asset-retirements/{}", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let fetched: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(fetched["id"], ret_id);
    assert_eq!(fetched["retirementType"], "sale");

    // 3. List all retirements - should include our new one
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // 4. Approve the retirement
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/approve", ret_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(approved["status"], "approved");

    // 5. Complete the retirement with a GL batch
    let gl_batch_id = "e2e00000-0000-0000-0000-000000000099";
    let payload = json!({ "glBatchId": gl_batch_id });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/complete", ret_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let completed: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["postedToGl"], true);
    assert_eq!(completed["glBatchId"], gl_batch_id);

    // 6. Check dashboard has the completed retirement
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dash: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(dash["totalRetirements"].as_i64().unwrap() >= 1);
    assert!(dash["completedRetirements"].as_i64().unwrap() >= 1);

    // 7. List filtered by completed status
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements?status=completed")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let filtered: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let completed_items: Vec<_> = filtered["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["id"] == ret_id)
        .collect();
    assert_eq!(completed_items.len(), 1);

    // 8. Now create another retirement and cancel it
    let ret2 = create_retirement(
        &app,
        "e2e00000-0000-0000-0000-000000000002",
        "scrap",
        "5000",
        "3000",
        "0",
        "0",
    )
    .await;
    let ret2_id = ret2["id"].as_str().unwrap();
    assert_eq!(ret2["gainLossAmount"], "-2000");

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/asset-retirements/{}/cancel", ret2_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let cancelled: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(cancelled["status"], "cancelled");

    // 9. Verify dashboard counts are correct
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/asset-retirements/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let final_dash: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(final_dash["totalRetirements"].as_i64().unwrap() >= 2);
    assert!(final_dash["completedRetirements"].as_i64().unwrap() >= 1);
}
