//! Withholding Tax Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Withholding Tax:
//! - Tax Code CRUD
//! - Tax Group management with members
//! - Supplier assignment with exemption handling
//! - Withholding computation (core calculation)
//! - Certificate lifecycle (draft → issued → cancelled)
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
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/027_withholding_tax.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run withholding tax migration");
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Tax Code CRUD Tests
// ============================================================================

async fn create_tax_code(
    app: &axum::Router,
    code: &str,
    name: &str,
    tax_type: &str,
    rate_percentage: &str,
    threshold_amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "taxType": tax_type,
        "ratePercentage": rate_percentage,
        "thresholdAmount": threshold_amount,
        "thresholdIsCumulative": false,
        "withholdingAccountCode": "2200",
        "expenseAccountCode": "6500",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/codes")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE TAX CODE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create tax code: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
async fn test_create_tax_code() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "WHT-10", "Income Tax 10%", "income_tax", "10.00", "1000").await;

    assert_eq!(tc["code"], "WHT-10");
    assert_eq!(tc["name"], "Income Tax 10%");
    assert_eq!(tc["taxType"], "income_tax");
    assert_eq!(tc["isActive"], true);
}

#[tokio::test]
async fn test_create_tax_code_uppercased() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "wht-vat", "VAT Withholding", "vat", "5.00", "500").await;
    // Code should be uppercased by engine
    assert_eq!(tc["code"], "WHT-VAT");
}

#[tokio::test]
async fn test_get_tax_code() {
    let (_state, app) = setup_test().await;
    create_tax_code(&app, "WHT-GET", "Get Test", "income_tax", "10.00", "0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/codes/WHT-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "WHT-GET");
    assert_eq!(body["name"], "Get Test");
}

#[tokio::test]
async fn test_list_tax_codes() {
    let (_state, app) = setup_test().await;
    create_tax_code(&app, "WHT-L1", "List 1", "income_tax", "10.00", "0").await;
    create_tax_code(&app, "WHT-L2", "List 2", "vat", "5.00", "0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/codes")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_tax_codes_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_tax_code(&app, "WHT-F1", "Filter 1", "income_tax", "10.00", "0").await;
    create_tax_code(&app, "WHT-F2", "Filter 2", "vat", "5.00", "0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/codes?tax_type=vat")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|c| c["taxType"] == "vat"));
}

#[tokio::test]
async fn test_delete_tax_code() {
    let (_state, app) = setup_test().await;
    create_tax_code(&app, "WHT-DEL", "Delete Me", "income_tax", "10.00", "0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/withholding-tax/codes/WHT-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone (soft delete)
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/codes/WHT-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_tax_code_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD",
        "name": "Bad Type",
        "taxType": "crypto",
        "ratePercentage": "10.00",
        "thresholdAmount": "0",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/codes")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_tax_code_rate_over_100() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "OVER100",
        "name": "Over 100",
        "taxType": "income_tax",
        "ratePercentage": "150.00",
        "thresholdAmount": "0",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/codes")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_tax_code_negative_threshold() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "NEGTHR",
        "name": "Negative Threshold",
        "taxType": "income_tax",
        "ratePercentage": "10.00",
        "thresholdAmount": "-100",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/codes")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Tax Group Tests
// ============================================================================

async fn create_tax_group(
    app: &axum::Router,
    code: &str,
    name: &str,
    tax_code_ids: &[Uuid],
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let ids: Vec<String> = tax_code_ids.iter().map(|id| id.to_string()).collect();
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test tax group",
        "taxCodeIds": ids,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/groups")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE TAX GROUP status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create tax group: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
async fn test_create_tax_group() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "GRP-IT", "Income Tax", "income_tax", "10.00", "0").await;
    let tc_id: Uuid = tc["id"].as_str().unwrap().parse().unwrap();

    let group = create_tax_group(&app, "STD-WHT", "Standard Withholding", &[tc_id]).await;

    assert_eq!(group["code"], "STD-WHT");
    assert_eq!(group["name"], "Standard Withholding");
    assert!(group["taxCodes"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_create_tax_group_multiple_codes() {
    let (_state, app) = setup_test().await;
    let tc1 = create_tax_code(&app, "GRP-A", "Tax A", "income_tax", "10.00", "0").await;
    let tc2 = create_tax_code(&app, "GRP-B", "Tax B", "vat", "5.00", "0").await;
    let tc1_id: Uuid = tc1["id"].as_str().unwrap().parse().unwrap();
    let tc2_id: Uuid = tc2["id"].as_str().unwrap().parse().unwrap();

    let group = create_tax_group(&app, "MULTI-WHT", "Multi Withholding", &[tc1_id, tc2_id]).await;

    let members = group["taxCodes"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
async fn test_get_tax_group() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "GET-GRP", "Get Group TC", "income_tax", "10.00", "0").await;
    let tc_id: Uuid = tc["id"].as_str().unwrap().parse().unwrap();
    create_tax_group(&app, "GET-GRP", "Get Group Test", &[tc_id]).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/groups/GET-GRP")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "GET-GRP");
}

#[tokio::test]
async fn test_list_tax_groups() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "LIST-G", "List Groups TC", "income_tax", "10.00", "0").await;
    let tc_id: Uuid = tc["id"].as_str().unwrap().parse().unwrap();
    create_tax_group(&app, "LG-1", "List Group 1", &[tc_id]).await;
    create_tax_group(&app, "LG-2", "List Group 2", &[tc_id]).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/groups")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_tax_group() {
    let (_state, app) = setup_test().await;
    let tc = create_tax_code(&app, "DEL-G", "Delete Group TC", "income_tax", "10.00", "0").await;
    let tc_id: Uuid = tc["id"].as_str().unwrap().parse().unwrap();
    create_tax_group(&app, "DEL-GRP", "Delete Me Group", &[tc_id]).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/withholding-tax/groups/DEL-GRP")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_create_tax_group_empty_codes_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "EMPTY",
        "name": "Empty Group",
        "taxCodeIds": [],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/groups")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_tax_group_invalid_code_id_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "INVGRP",
        "name": "Invalid Group",
        "taxCodeIds": [Uuid::new_v4().to_string()],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/groups")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Supplier Assignment Tests
// ============================================================================

async fn setup_group_with_codes(
    app: &axum::Router,
) -> (Uuid, String) {
    let tc = create_tax_code(&app, "SUP-IT", "Supplier Income Tax", "income_tax", "10.00", "1000").await;
    let tc_id: Uuid = tc["id"].as_str().unwrap().parse().unwrap();
    let group = create_tax_group(app, "SUP-GRP", "Supplier Group", &[tc_id]).await;
    let group_code = group["code"].as_str().unwrap().to_string();
    (tc_id, group_code)
}

#[tokio::test]
async fn test_assign_supplier() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierNumber": "SUP-001",
        "supplierName": "Acme Corp",
        "taxGroupCode": group_code,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["supplierId"], supplier_id.to_string());
    assert_eq!(body["isExempt"], false);
    assert_eq!(body["taxGroupCode"], group_code);
}

#[tokio::test]
async fn test_assign_supplier_exempt() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierNumber": "SUP-002",
        "supplierName": "Exempt Corp",
        "taxGroupCode": group_code,
        "isExempt": true,
        "exemptionReason": "Government entity",
        "exemptionCertificate": "GOV-EX-001",
        "exemptionValidUntil": "2099-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isExempt"], true);
    assert_eq!(body["exemptionReason"], "Government entity");
}

#[tokio::test]
async fn test_assign_exempt_without_reason_fails() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "taxGroupCode": group_code,
        "isExempt": true,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_supplier_assignment() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Get Test Supplier",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/withholding-tax/suppliers/{}", supplier_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["supplierId"], supplier_id.to_string());
}

#[tokio::test]
async fn test_list_supplier_assignments() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let (k, v) = auth_header(&admin_claims());
    for i in 0..3 {
        let payload = json!({
            "supplierId": Uuid::new_v4().to_string(),
            "supplierName": format!("Supplier {}", i),
            "taxGroupCode": group_code,
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/withholding-tax/suppliers/assign")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/suppliers")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test_remove_supplier_assignment() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Remove Me",
        "taxGroupCode": group_code,
    });
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let assignment: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let assignment_id: Uuid = assignment["id"].as_str().unwrap().parse().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/withholding-tax/suppliers/{}", assignment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Withholding Computation Tests
// ============================================================================

#[tokio::test]
async fn test_compute_withholding_basic() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Assign supplier to group
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Compute Test Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Compute withholding - invoice_amount = 10000, rate = 10%, threshold = 1000
    let compute_payload = json!({
        "supplierId": supplier_id.to_string(),
        "invoiceAmount": 10000.0,
        "invoiceId": Uuid::new_v4().to_string(),
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/compute")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&compute_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    eprintln!("Compute result: {:?}", body);
    assert_eq!(body["is_exempt"], false);
    assert_eq!(body["taxGroupCode"], group_code);
    let lines = body["lines"].as_array().unwrap();
    assert!(!lines.is_empty());
    // 10000 * 10% = 1000 withheld
    let withheld: f64 = body["totalWithheldAmount"].as_f64().unwrap();
    assert!((withheld - 1000.0).abs() < 0.01, "Expected 1000.0, got {}", withheld);
    // Net = 10000 - 1000 = 9000
    let net: f64 = body["netPaymentAmount"].as_f64().unwrap();
    assert!((net - 9000.0).abs() < 0.01, "Expected 9000.0, got {}", net);
}

#[tokio::test]
async fn test_compute_withholding_exempt_supplier() {
    let (_state, app) = setup_test().await;
    let (_tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Assign exempt supplier
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Exempt Corp",
        "taxGroupCode": group_code,
        "isExempt": true,
        "exemptionReason": "Tax treaty",
        "exemptionCertificate": "TREATY-001",
        "exemptionValidUntil": "2099-12-31",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Compute - should be exempt
    let compute_payload = json!({
        "supplierId": supplier_id.to_string(),
        "invoiceAmount": 10000.0,
        "invoiceId": Uuid::new_v4().to_string(),
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/compute")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&compute_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["is_exempt"], true);
    assert_eq!(body["lines"].as_array().unwrap().len(), 0);
    let net: f64 = body["netPaymentAmount"].as_f64().unwrap();
    assert!((net - 10000.0).abs() < 0.01, "Exempt: net should equal invoice");
}

#[tokio::test]
async fn test_compute_withholding_no_assignment() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());
    let compute_payload = json!({
        "supplierId": Uuid::new_v4().to_string(),
        "invoiceAmount": 10000.0,
        "invoiceId": Uuid::new_v4().to_string(),
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/compute")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&compute_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    // No assignment = no withholding
    assert_eq!(body["is_exempt"], false);
    assert_eq!(body["lines"].as_array().unwrap().len(), 0);
    let net: f64 = body["netPaymentAmount"].as_f64().unwrap();
    assert!((net - 10000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_compute_withholding_below_threshold() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Assign supplier
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Threshold Test Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Invoice below threshold (1000) = invoice amount 500
    let compute_payload = json!({
        "supplierId": supplier_id.to_string(),
        "invoiceAmount": 500.0,
        "invoiceId": Uuid::new_v4().to_string(),
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/compute")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&compute_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Below threshold - no withholding
    let withheld: f64 = body["totalWithheldAmount"].as_f64().unwrap();
    assert!((withheld - 0.0).abs() < 0.01, "Below threshold: should be 0, got {}", withheld);
    let lines = body["lines"].as_array().unwrap();
    assert!(lines[0]["thresholdApplied"].as_bool().unwrap());
}

// ============================================================================
// Certificate Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_certificate_lifecycle() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Assign supplier
    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Cert Lifecycle Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate certificate
    let cert_payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Cert Lifecycle Corp",
        "taxCodeId": tc_id.to_string(),
        "periodStart": "2024-01-01",
        "periodEnd": "2024-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/certificates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let cert: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let cert_id: Uuid = cert["id"].as_str().unwrap().parse().unwrap();

    assert_eq!(cert["status"], "draft");
    assert!(cert["certificateNumber"].as_str().unwrap().starts_with("WHT-CERT-"));

    // Issue the certificate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/withholding-tax/certificates/{}/issue", cert_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "issued");
    assert!(body["issuedAt"].is_string());

    // Get by number
    let number = cert["certificateNumber"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/withholding-tax/certificates/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_certificate_cancel() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Cancel Cert Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let cert_payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Cancel Cert Corp",
        "taxCodeId": tc_id.to_string(),
        "periodStart": "2024-01-01",
        "periodEnd": "2024-03-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/certificates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let cert: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let cert_id: Uuid = cert["id"].as_str().unwrap().parse().unwrap();

    // Cancel
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/withholding-tax/certificates/{}/cancel", cert_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_issue_already_issued_fails() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Double Issue Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let cert_payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "Double Issue Corp",
        "taxCodeId": tc_id.to_string(),
        "periodStart": "2024-01-01",
        "periodEnd": "2024-06-30",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/certificates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let cert: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let cert_id: Uuid = cert["id"].as_str().unwrap().parse().unwrap();

    // Issue once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/withholding-tax/certificates/{}/issue", cert_id))
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();

    // Issue again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/withholding-tax/certificates/{}/issue", cert_id))
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_certificate_invalid_period_fails() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let cert_payload = json!({
        "supplierId": supplier_id.to_string(),
        "taxCodeId": tc_id.to_string(),
        "periodStart": "2024-12-31",
        "periodEnd": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/certificates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_certificates() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "List Cert Corp",
        "taxGroupCode": group_code,
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/suppliers/assign")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let cert_payload = json!({
        "supplierId": supplier_id.to_string(),
        "supplierName": "List Cert Corp",
        "taxCodeId": tc_id.to_string(),
        "periodStart": "2024-01-01",
        "periodEnd": "2024-03-31",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/withholding-tax/certificates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/certificates")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_certificates_filter_by_supplier() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let supplier1 = Uuid::new_v4();
    let supplier2 = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    for sid in &[supplier1, supplier2] {
        let payload = json!({
            "supplierId": sid.to_string(),
            "supplierName": format!("Filter Corp {}", sid),
            "taxGroupCode": group_code,
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/withholding-tax/suppliers/assign")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();

        let cert_payload = json!({
            "supplierId": sid.to_string(),
            "supplierName": format!("Filter Corp {}", sid),
            "taxCodeId": tc_id.to_string(),
            "periodStart": "2024-01-01",
            "periodEnd": "2024-06-30",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/withholding-tax/certificates")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&cert_payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/withholding-tax/certificates?supplier_id={}", supplier1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let certs = body["data"].as_array().unwrap();
    assert!(certs.iter().all(|c| c["supplierId"] == supplier1.to_string()));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_withholding_dashboard() {
    let (_state, app) = setup_test().await;
    let (tc_id, group_code) = setup_group_with_codes(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // Create some assignments
    for i in 0..2 {
        let payload = json!({
            "supplierId": Uuid::new_v4().to_string(),
            "supplierName": format!("Dash Corp {}", i),
            "taxGroupCode": group_code,
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/withholding-tax/suppliers/assign")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("activeTaxCodeCount").is_some());
    assert!(body.get("taxGroupCount").is_some());
    assert!(body.get("assignedSupplierCount").is_some());
    assert!(body.get("exemptSupplierCount").is_some());
    assert!(body.get("totalWithheldAmount").is_some());
    assert!(body.get("certificatesIssued").is_some());

    assert!(body["activeTaxCodeCount"].as_i64().unwrap() >= 1);
    assert!(body["taxGroupCount"].as_i64().unwrap() >= 1);
    assert!(body["assignedSupplierCount"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_tax_code_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/withholding-tax/codes/NONEXISTENT")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_supplier_assignment_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/withholding-tax/suppliers/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_certificate_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/withholding-tax/certificates/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
