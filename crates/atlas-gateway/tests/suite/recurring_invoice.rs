//! Recurring Invoice Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Recurring Invoices:
//! - Template CRUD
//! - Full lifecycle (draft → active → suspended → completed → cancelled)
//! - Template lines (add/remove)
//! - Invoice generation from templates
//! - Generation history
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Clean recurring invoice test data
    sqlx::query("DELETE FROM _atlas.recurring_invoice_generation_log").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_invoice_generations").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_invoice_template_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_invoice_templates").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/130_recurring_invoice_management.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run recurring invoice migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_template(
    app: &axum::Router,
    template_number: &str,
    template_name: &str,
    supplier_name: &str,
    recurrence_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateNumber": template_number,
        "templateName": template_name,
        "supplierName": supplier_name,
        "invoiceType": "standard",
        "invoiceCurrencyCode": "USD",
        "amountType": "fixed",
        "recurrenceType": recurrence_type,
        "recurrenceInterval": 1,
        "paymentDueDays": 30,
        "effectiveFrom": "2024-01-01",
        "effectiveTo": "2024-12-31",
        "holdForReview": true,
        "glDateBasis": "generation_date",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-invoices")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE TEMPLATE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create template: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    template_id: Uuid,
    amount: f64,
    line_type: &str,
    gl_account: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineType": line_type,
        "description": format!("Test {} line", line_type),
        "amount": amount,
        "quantity": 1.0,
        "glAccountCode": gl_account,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/lines", template_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD LINE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-001", "Monthly Rent", "Landlord Corp", "monthly").await;

    assert_eq!(tmpl["templateNumber"], "RI-001");
    assert_eq!(tmpl["templateName"], "Monthly Rent");
    assert_eq!(tmpl["supplierName"], "Landlord Corp");
    assert_eq!(tmpl["recurrenceType"], "monthly");
    assert_eq!(tmpl["amountType"], "fixed");
    assert_eq!(tmpl["status"], "draft");
    assert_eq!(tmpl["invoiceCurrencyCode"], "USD");
}

#[tokio::test]
async fn test_get_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-001", "Monthly Rent", "Landlord Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}", tmpl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], tmpl["id"]);
    assert_eq!(body["templateName"], "Monthly Rent");
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_test().await;
    create_template(&app, "RI-001", "Monthly Rent", "Landlord A", "monthly").await;
    create_template(&app, "RI-002", "Insurance Premium", "Insure Co", "quarterly").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-invoices")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_templates_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_template(&app, "RI-001", "Monthly Rent", "Landlord A", "monthly").await;
    let tmpl2 = create_template(&app, "RI-002", "Insurance Premium", "Insure Co", "quarterly").await;
    let tmpl2_id: Uuid = tmpl2["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate the second template
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl2_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Filter by draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-invoices?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|t| t["status"] == "draft"));

    // Filter by active
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-invoices?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|t| t["status"] == "active"));
    assert!(data.len() >= 1);
}

#[tokio::test]
async fn test_delete_draft_template() {
    let (_state, app) = setup_test().await;
    create_template(&app, "RI-DEL", "To Delete", "Delete Corp", "monthly").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/recurring-invoices/number/RI-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_template_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-LC", "Lifecycle Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Draft → Active
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
    // When activated, next generation date should be set
    assert!(body["nextGenerationDate"].is_string());

    // Active → Suspended
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "suspended"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "suspended");

    // Suspended → Active (reactivate)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // Active → Completed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "completed");
}

#[tokio::test]
async fn test_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-CAN", "Cancel Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_invalid_transition_completed_from_draft() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-INV", "Invalid Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_transition_suspended_from_draft() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-INV2", "Invalid Test 2", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "suspended"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_active_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-NO-DEL", "No Delete", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to delete active template
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/recurring-invoices/number/RI-NO-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_lines() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-LN", "Lines Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, tmpl_id, 5000.00, "item", "5200").await;
    let line2 = add_line(&app, tmpl_id, 450.00, "tax", "2300").await;

    assert_eq!(line1["lineType"], "item");
    let amt1: f64 = line1["amount"].as_f64().unwrap();
    assert!((amt1 - 5000.0).abs() < 0.01);

    assert_eq!(line2["lineType"], "tax");
    let amt2: f64 = line2["amount"].as_f64().unwrap();
    assert!((amt2 - 450.0).abs() < 0.01);
}

#[tokio::test]
async fn test_list_lines() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-LIST", "List Lines", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, tmpl_id, 3000.00, "item", "5200").await;
    add_line(&app, tmpl_id, 1500.00, "item", "5210").await;
    add_line(&app, tmpl_id, 450.00, "tax", "2300").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}/lines", tmpl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_remove_line() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-REM", "Remove Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, tmpl_id, 5000.00, "item", "5200").await;
    add_line(&app, tmpl_id, 450.00, "tax", "2300").await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/recurring-invoices/{}/lines/{}", tmpl_id, line1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify only one line remains
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}/lines", tmpl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_add_line_to_active_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-ACT", "Active Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to add line to active template
    let payload = json!({"lineType": "item", "amount": 100.0, "glAccountCode": "5200"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/lines", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Invoice Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_invoice() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-GEN", "Generate Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    // Add lines: item amount 5000, tax amount 450 on the same line
    add_line(&app, tmpl_id, 5000.00, "item", "5200").await;
    // Add tax as a separate line using the tax amount field
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineType": "tax",
        "description": "Tax line",
        "amount": 0.0,
        "quantity": 1.0,
        "glAccountCode": "2300",
        "taxAmount": 450.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/lines", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let _: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate invoice
    let payload = json!({
        "invoiceDate": "2024-02-01",
        "periodName": "FEB-2024",
        "fiscalYear": 2024,
        "periodNumber": 2,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Verify amounts: 5000 + 450 = 5450
    let inv_amt: f64 = body["invoiceAmount"].as_f64().unwrap();
    assert!((inv_amt - 5000.0).abs() < 0.01, "Expected invoice amount 5000, got {}", inv_amt);

    let tax_amt: f64 = body["taxAmount"].as_f64().unwrap();
    assert!((tax_amt - 450.0).abs() < 0.01, "Expected tax amount 450, got {}", tax_amt);

    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 5450.0).abs() < 0.01, "Expected total 5450, got {}", total);

    assert_eq!(body["generationNumber"], 1);
    assert_eq!(body["periodName"], "FEB-2024");
    assert_eq!(body["fiscalYear"], 2024);
    assert_eq!(body["generationStatus"], "generated");
}

#[tokio::test]
async fn test_generate_updates_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-GEN2", "Generate Test 2", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, tmpl_id, 3000.00, "item", "5200").await;

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate invoice
    let payload = json!({"invoiceDate": "2024-03-01"});
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Check template was updated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}", tmpl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["generationCount"], 1);
    let total: f64 = body["totalGeneratedAmount"].as_f64().unwrap();
    assert!((total - 3000.0).abs() < 0.01, "Expected total generated 3000, got {}", total);
    assert!(body["lastGenerationDate"].is_string());
    assert!(body["nextGenerationDate"].is_string());
}

#[tokio::test]
async fn test_generate_from_draft_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-GEN-ERR", "Gen Error Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, tmpl_id, 1000.00, "item", "5200").await;

    let (k, v) = auth_header(&admin_claims());

    // Try to generate from draft template (not active)
    let payload = json!({"invoiceDate": "2024-02-01"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_no_lines_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-GEN-NOL", "No Lines Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate (no lines added)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to generate without lines
    let payload = json!({"invoiceDate": "2024-02-01"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Generation History Tests
// ============================================================================

#[tokio::test]
async fn test_list_generations() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-HIST", "History Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, tmpl_id, 5000.00, "item", "5200").await;

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate two invoices
    for i in 1..=2 {
        let payload = json!({"invoiceDate": format!("2024-0{}-01", i)});
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // List generations
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-invoices/generations")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let gens = body["data"].as_array().unwrap();
    assert!(gens.len() >= 2);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_template(&app, "RI-D1", "Dashboard 1", "Supplier A", "monthly").await;
    create_template(&app, "RI-D2", "Dashboard 2", "Supplier B", "quarterly").await;

    let tmpl3 = create_template(&app, "RI-D3", "Dashboard 3", "Supplier C", "annual").await;
    let tmpl3_id: Uuid = tmpl3["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate one
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl3_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-invoices/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalTemplates").is_some());
    assert!(body.get("activeTemplates").is_some());
    assert!(body.get("suspendedTemplates").is_some());
    assert!(body.get("totalGenerations").is_some());
    assert!(body.get("totalGeneratedAmount").is_some());
    assert!(body.get("upcomingThisMonth").is_some());
    assert!(body.get("byRecurrenceType").is_some());
    assert!(body.get("bySupplier").is_some());

    assert!(body["totalTemplates"].as_i64().unwrap() >= 3);
    assert!(body["activeTemplates"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_template_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateNumber": "",
        "templateName": "No Number",
        "recurrenceType": "monthly",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-invoices")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_recurrence() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateNumber": "RI-BAD",
        "templateName": "Bad Recurrence",
        "recurrenceType": "biennial",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-invoices")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_amount_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateNumber": "RI-BAD2",
        "templateName": "Bad Amount Type",
        "recurrenceType": "monthly",
        "amountType": "indexed",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-invoices")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_interval() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateNumber": "RI-BAD3",
        "templateName": "Bad Interval",
        "recurrenceType": "monthly",
        "recurrenceInterval": 0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-invoices")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_status_in_transition() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-STAT", "Status Test", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "invalid_status"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_different_recurrence_types() {
    let (_state, app) = setup_test().await;

    // Weekly
    let tmpl = create_template(&app, "RI-WK", "Weekly Cleaning", "Clean Corp", "weekly").await;
    assert_eq!(tmpl["recurrenceType"], "weekly");

    // Quarterly
    let tmpl = create_template(&app, "RI-QTR", "Quarterly Insurance", "Insure Co", "quarterly").await;
    assert_eq!(tmpl["recurrenceType"], "quarterly");

    // Annual
    let tmpl = create_template(&app, "RI-ANN", "Annual License", "Software Co", "annual").await;
    assert_eq!(tmpl["recurrenceType"], "annual");
}

#[tokio::test]
async fn test_generate_multiple_increments_count() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "RI-MULTI", "Multi Generate", "Test Corp", "monthly").await;
    let tmpl_id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, tmpl_id, 1000.00, "item", "5200").await;

    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-invoices/{}/transition", tmpl_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate 3 invoices
    let months = ["2024-01-01", "2024-02-01", "2024-03-01"];
    for date in &months {
        let payload = json!({"invoiceDate": *date});
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/recurring-invoices/{}/generate", tmpl_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Verify template generation count
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-invoices/{}", tmpl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["generationCount"], 3);

    let total: f64 = body["totalGeneratedAmount"].as_f64().unwrap();
    assert!((total - 3000.0).abs() < 0.01, "Expected total 3000, got {}", total);
}
