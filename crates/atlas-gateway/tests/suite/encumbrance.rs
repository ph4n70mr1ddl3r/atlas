//! Encumbrance Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Encumbrance Management:
//! - Encumbrance type CRUD
//! - Encumbrance entry lifecycle (create → add lines → activate → liquidate → fully liquidated)
//! - Encumbrance line management
//! - Liquidation (partial and full) and reversal
//! - Entry cancellation
//! - Year-end carry-forward
//! - Dashboard summary
//! - Error cases and validation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_encumbrance_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    // Run the encumbrance migration
    let migration_sql = include_str!("../../../../migrations/020_encumbrance_management.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_type(
    app: &axum::Router, code: &str, name: &str, category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "category": category,
            "allow_manual_entry": true,
            "allow_carry_forward": true,
            "priority": 10,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_entry(
    app: &axum::Router, type_code: &str, amount: &str,
    source_type: Option<&str>, source_number: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "encumbrance_type_code": type_code,
        "encumbrance_date": "2024-03-15",
        "amount": amount,
        "currency_code": "USD",
        "fiscal_year": 2024,
    });
    if let Some(st) = source_type {
        payload["source_type"] = json!(st);
    }
    if let Some(sn) = source_number {
        payload["source_number"] = json!(sn);
    }
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_line(
    app: &axum::Router, entry_id: &str, account_code: &str, amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/entries/{}/lines", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": account_code,
            "account_description": format!("Account {}", account_code),
            "amount": amount,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn activate_entry(app: &axum::Router, entry_id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/entries/{}/activate", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Encumbrance Type Tests
// ============================================================================

#[tokio::test]
async fn test_create_encumbrance_type() {
    let (_state, app) = setup_encumbrance_test().await;

    let etype = create_test_type(&app, "PURCHASE_ORDER", "Purchase Order", "commitment").await;

    assert_eq!(etype["code"], "PURCHASE_ORDER");
    assert_eq!(etype["name"], "Purchase Order");
    assert_eq!(etype["category"], "commitment");
    assert_eq!(etype["is_enabled"], true);
    assert_eq!(etype["allow_carry_forward"], true);
}

#[tokio::test]
async fn test_list_encumbrance_types() {
    let (_state, app) = setup_encumbrance_test().await;

    create_test_type(&app, "PO", "Purchase Order", "commitment").await;
    create_test_type(&app, "REQ", "Requisition", "preliminary").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/types")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let types: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(types.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_encumbrance_type() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "CONTRACT", "Contract", "obligation").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/types/CONTRACT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let etype: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(etype["code"], "CONTRACT");
    assert_eq!(etype["name"], "Contract");
}

#[tokio::test]
async fn test_delete_encumbrance_type() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "TEMP_TYPE", "Temporary", "commitment").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/encumbrance/types/TEMP_TYPE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone (soft-deleted)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/types/TEMP_TYPE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_encumbrance_type_invalid_category() {
    let (_state, app) = setup_encumbrance_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Category",
            "category": "invalid_category",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Encumbrance Entry Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_entry_lifecycle() {
    let (_state, app) = setup_encumbrance_test().await;

    // 1. Create encumbrance type
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    // 2. Create entry (draft)
    let entry = create_test_entry(&app, "PO", "50000.00", Some("purchase_order"), Some("PO-001")).await;
    let entry_id = entry["id"].as_str().unwrap();
    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["original_amount"].as_str().unwrap().replace("\"", ""), "50000.00");

    // 3. Add lines
    let line1 = add_test_line(&app, entry_id, "5000-01", "30000.00").await;
    let line2 = add_test_line(&app, entry_id, "5000-02", "20000.00").await;
    assert_eq!(line1["account_code"], "5000-01");
    assert_eq!(line2["account_code"], "5000-02");

    // 4. Activate entry
    let activated = activate_entry(&app, entry_id).await;
    assert_eq!(activated["status"], "active");

    // 5. List entries - should show active
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/entries?status=active")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let entries: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(entries.as_array().unwrap().len() >= 1);

    // 6. Partial liquidation
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/liquidations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_entry_id": entry_id,
            "liquidation_type": "partial",
            "liquidation_amount": "30000.00",
            "source_type": "invoice",
            "source_number": "INV-001",
            "liquidation_date": "2024-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 7. Verify entry status updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/encumbrance/entries/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "partially_liquidated");

    // 8. Final liquidation (remaining amount)
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/liquidations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_entry_id": entry_id,
            "liquidation_type": "final",
            "liquidation_amount": "20000.00",
            "source_type": "invoice",
            "source_number": "INV-002",
            "liquidation_date": "2024-04-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 9. Verify fully liquidated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/encumbrance/entries/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let final_entry: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(final_entry["status"], "fully_liquidated");
}

#[tokio::test]
async fn test_liquidation_reversal() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "10000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Liquidate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/liquidations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_entry_id": entry_id,
            "liquidation_type": "partial",
            "liquidation_amount": "5000.00",
            "liquidation_date": "2024-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let liq: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let liq_id = liq["id"].as_str().unwrap();

    // Reverse the liquidation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/liquidations/{}/reverse", liq_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Invoice was cancelled",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify entry is back to active
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/encumbrance/entries/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(entry["status"], "active");
}

#[tokio::test]
async fn test_cancel_entry() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "25000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();

    // Activate
    activate_entry(&app, entry_id).await;

    // Cancel
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/entries/{}/cancel", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Order was cancelled by supplier",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert!(cancelled["cancellation_reason"].is_string());
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[tokio::test]
async fn test_activate_non_draft_entry_fails() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "10000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Try to activate again - should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/entries/{}/activate", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_liquidate_exceeds_amount_fails() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "5000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Try to liquidate more than the current amount
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/liquidations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_entry_id": entry_id,
            "liquidation_type": "full",
            "liquidation_amount": "99999.00",
            "liquidation_date": "2024-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_entry_invalid_type_fails() {
    let (_state, app) = setup_encumbrance_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_type_code": "NONEXISTENT",
            "encumbrance_date": "2024-03-15",
            "amount": "1000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cancel_without_reason_fails() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "10000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/encumbrance/entries/{}/cancel", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Encumbrance Lines Tests
// ============================================================================

#[tokio::test]
async fn test_list_entry_lines() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "60000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    add_test_line(&app, entry_id, "5000-01", "40000.00").await;
    add_test_line(&app, entry_id, "5000-02", "20000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/encumbrance/entries/{}/lines", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_line_from_draft_entry() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "30000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    let line = add_test_line(&app, entry_id, "5000-01", "30000.00").await;
    let line_id = line["id"].as_str().unwrap();

    // Delete line (entry is still draft)
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/encumbrance/lines/{}", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Carry-Forward Tests
// ============================================================================

#[tokio::test]
async fn test_process_carry_forward() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    // Create and activate an entry in FY 2024
    let entry = create_test_entry(&app, "PO", "25000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Process carry-forward
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/encumbrance/carry-forward")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_fiscal_year": 2024,
            "to_fiscal_year": 2025,
            "description": "Year-end carry-forward 2024→2025",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cf: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cf["status"], "completed");
    assert!(cf["entry_count"].as_i64().unwrap() >= 1);
    assert_eq!(cf["from_fiscal_year"], 2024);
    assert_eq!(cf["to_fiscal_year"], 2025);
}

// ============================================================================
// Dashboard Summary Tests
// ============================================================================

#[tokio::test]
async fn test_encumbrance_summary() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    // Create and activate entries
    let entry = create_test_entry(&app, "PO", "50000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Get summary
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/summary")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(summary["active_entry_count"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Listing & Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_list_liquidations() {
    let (_state, app) = setup_encumbrance_test().await;
    create_test_type(&app, "PO", "Purchase Order", "commitment").await;

    let entry = create_test_entry(&app, "PO", "10000.00", None, None).await;
    let entry_id = entry["id"].as_str().unwrap();
    activate_entry(&app, entry_id).await;

    // Create liquidation
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/encumbrance/liquidations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "encumbrance_entry_id": entry_id,
            "liquidation_type": "partial",
            "liquidation_amount": "5000.00",
            "liquidation_date": "2024-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List liquidations
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/encumbrance/liquidations?status=processed")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let liqs: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(liqs.as_array().unwrap().len() >= 1);
}
