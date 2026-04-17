//! Fixed Assets Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Fixed Assets:
//! - Asset category CRUD
//! - Asset book management
//! - Asset lifecycle (create → acquire → place in service → depreciate → retire)
//! - Depreciation calculation (straight-line, declining balance, sum-of-years-digits)
//! - Asset transfers
//! - Asset retirements with gain/loss

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_fa_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_category(
    app: &axum::Router, code: &str, name: &str,
    depreciation_method: &str, useful_life_months: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "default_depreciation_method": depreciation_method,
        "default_useful_life_months": useful_life_months,
        "default_salvage_value_percent": "10",
        "default_asset_account_code": "1500",
        "default_accum_depr_account_code": "1510",
        "default_depr_expense_account_code": "6200",
        "default_gain_loss_account_code": "8100",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/fixed-assets/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create category");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_book(
    app: &axum::Router, code: &str, name: &str, book_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "book_type": book_type,
        "auto_depreciation": true,
        "depreciation_calendar": "monthly",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/fixed-assets/books")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create book");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_asset(
    app: &axum::Router, asset_number: &str, asset_name: &str,
    original_cost: &str, category_code: Option<&str>, book_code: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "asset_number": asset_number,
        "asset_name": asset_name,
        "asset_type": "tangible",
        "original_cost": original_cost,
        "salvage_value": "0",
        "salvage_value_percent": "0",
        "location": "HQ Office",
        "department_name": "IT",
    });
    if let Some(cc) = category_code {
        payload["category_code"] = json!(cc);
    }
    if let Some(bc) = book_code {
        payload["book_code"] = json!(bc);
    }
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/fixed-assets/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create asset");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Asset Category Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_asset_category() {
    let (_state, app) = setup_fa_test().await;

    let cat = create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;

    assert_eq!(cat["code"], "IT_EQUIP");
    assert_eq!(cat["name"], "IT Equipment");
    assert_eq!(cat["default_depreciation_method"], "straight_line");
    assert_eq!(cat["default_useful_life_months"], 60);
    assert_eq!(cat["is_active"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_asset_categories() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;
    create_test_category(&app, "VEHICLES", "Vehicles", "straight_line", 48).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/fixed-assets/categories")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_get_asset_category() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "BUILDINGS", "Buildings", "straight_line", 360).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/fixed-assets/categories/BUILDINGS")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cat: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cat["name"], "Buildings");
    assert_eq!(cat["default_useful_life_months"], 360);
}

#[tokio::test]
#[ignore]
async fn test_delete_asset_category() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "FURNITURE", "Furniture", "straight_line", 36).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri("/api/v1/fixed-assets/categories/FURNITURE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Asset Book Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_asset_book() {
    let (_state, app) = setup_fa_test().await;

    let book = create_test_book(&app, "CORPORATE", "Corporate Book", "corporate").await;

    assert_eq!(book["code"], "CORPORATE");
    assert_eq!(book["name"], "Corporate Book");
    assert_eq!(book["book_type"], "corporate");
    assert_eq!(book["auto_depreciation"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_asset_books() {
    let (_state, app) = setup_fa_test().await;

    create_test_book(&app, "CORPORATE", "Corporate Book", "corporate").await;
    create_test_book(&app, "TAX_US", "US Tax Book", "tax").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/fixed-assets/books")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Asset Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_fixed_asset() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;
    create_test_book(&app, "CORPORATE", "Corporate Book", "corporate").await;

    let asset = create_test_asset(&app, "FA-001", "MacBook Pro", "2500.00", Some("IT_EQUIP"), Some("CORPORATE")).await;

    assert_eq!(asset["asset_number"], "FA-001");
    assert_eq!(asset["asset_name"], "MacBook Pro");
    assert_eq!(asset["status"], "draft");
    assert_eq!(asset["original_cost"], "2500.00");
    assert_eq!(asset["category_code"], "IT_EQUIP");
    assert_eq!(asset["book_code"], "CORPORATE");
}

#[tokio::test]
#[ignore]
async fn test_asset_full_lifecycle() {
    let (_state, app) = setup_fa_test().await;

    // Setup
    create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;
    create_test_book(&app, "CORPORATE", "Corporate Book", "corporate").await;

    // 1. Create asset
    let asset = create_test_asset(&app, "FA-100", "Server Rack", "10000.00", Some("IT_EQUIP"), Some("CORPORATE")).await;
    let asset_id = asset["id"].as_str().unwrap();
    assert_eq!(asset["status"], "draft");

    // 2. Acquire asset
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/acquire", asset_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let acquired: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(acquired["status"], "acquired");

    // 3. Place in service
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/place-in-service", asset_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let in_service: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(in_service["status"], "in_service");

    // 4. Calculate depreciation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/depreciate", asset_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fiscal_year": 2026,
            "period_number": 1,
            "period_name": "APR-2026",
            "depreciation_date": "2026-04-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dep_result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Straight-line: 10000 / 60 = 166.67
    let dep_amount: f64 = dep_result["depreciation_amount"].as_str().unwrap().parse().unwrap();
    assert!((dep_amount - 166.67).abs() < 1.0);

    // 5. Verify depreciation history
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/depreciation-history", asset_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let history: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(history["data"].as_array().unwrap().len(), 1);

    // 6. Retire asset
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/fixed-assets/retirements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asset_id": asset_id,
            "retirement_type": "sale",
            "retirement_date": "2026-06-30",
            "proceeds": "8500.00",
            "removal_cost": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let retirement: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let retirement_id = retirement["id"].as_str().unwrap();
    assert_eq!(retirement["retirement_type"], "sale");
    assert_eq!(retirement["status"], "pending");
    // Gain/Loss = 8500 - 9833.33 - 0 = -1333.33 (loss since NBV ~ 9833.33)
    let gain_loss: f64 = retirement["gain_loss_amount"].as_str().unwrap().parse().unwrap();
    assert!(gain_loss > 0.0, "Should have a gain or loss amount");

    // 7. Approve retirement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/retirements/{}/approve", retirement_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "completed");
}

// ============================================================================
// Asset Listing & Filtering Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_assets_by_status() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;

    let a1 = create_test_asset(&app, "FA-010", "Laptop 1", "1500.00", Some("IT_EQUIP"), None).await;
    let _a2 = create_test_asset(&app, "FA-011", "Laptop 2", "1500.00", Some("IT_EQUIP"), None).await;

    // Acquire first asset
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/acquire", a1["id"].as_str().unwrap()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List only draft assets
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/fixed-assets/assets?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Asset Transfer Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_asset_transfer() {
    let (_state, app) = setup_fa_test().await;

    create_test_category(&app, "IT_EQUIP", "IT Equipment", "straight_line", 60).await;

    let asset = create_test_asset(&app, "FA-200", "Projector", "3000.00", Some("IT_EQUIP"), None).await;
    let asset_id = asset["id"].as_str().unwrap();

    // Acquire and place in service
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/acquire", asset_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/place-in-service", asset_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create transfer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/fixed-assets/transfers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asset_id": asset_id,
            "to_department_name": "Marketing",
            "to_location": "Marketing Office",
            "transfer_date": "2026-05-01",
            "reason": "Department reorganization"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let transfer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let transfer_id = transfer["id"].as_str().unwrap();
    assert_eq!(transfer["status"], "pending");
    assert_eq!(transfer["to_department_name"], "Marketing");

    // Approve transfer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/transfers/{}/approve", transfer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "completed");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_depreciate_draft_asset() {
    let (_state, app) = setup_fa_test().await;

    let asset = create_test_asset(&app, "FA-300", "Monitor", "500.00", None, None).await;
    let asset_id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/fixed-assets/assets/{}/depreciate", asset_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fiscal_year": 2026,
            "period_number": 1,
            "depreciation_date": "2026-04-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_transfer_draft_asset() {
    let (_state, app) = setup_fa_test().await;

    let asset = create_test_asset(&app, "FA-301", "Monitor", "500.00", None, None).await;
    let asset_id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/fixed-assets/transfers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asset_id": asset_id,
            "to_department_name": "HR",
            "transfer_date": "2026-05-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_asset_with_negative_cost() {
    let (_state, app) = setup_fa_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/fixed-assets/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asset_number": "FA-400",
            "asset_name": "Bad Asset",
            "asset_type": "tangible",
            "original_cost": "-1000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_asset_with_invalid_type() {
    let (_state, app) = setup_fa_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/fixed-assets/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asset_number": "FA-401",
            "asset_name": "Bad Type",
            "asset_type": "nonexistent",
            "original_cost": "1000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
