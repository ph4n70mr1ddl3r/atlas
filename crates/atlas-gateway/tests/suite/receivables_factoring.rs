//! Receivables Factoring E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Receivables Factoring:
//! - Factor company CRUD and lifecycle
//! - Factoring agreement CRUD and lifecycle (draft → active → suspended → terminated)
//! - Factoring request full lifecycle (draft → submitted → approved → funded → settled)
//! - Request lines with automatic fee/advance/reserve calculation
//! - Settlement processing
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
    let migration_sql = include_str!("../../../../migrations/143_receivables_factoring.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_factor_company(
    app: &axum::Router,
    code: &str,
    name: &str,
    advance_rate: &str,
    fee_rate: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test factor company",
        "advance_rate": advance_rate,
        "fee_rate": fee_rate,
        "recourse_type": "recourse",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/factor-companies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE FACTOR COMPANY RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create factor company: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_agreement(
    app: &axum::Router,
    agreement_number: &str,
    factor_company_id: Uuid,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "agreement_number": agreement_number,
        "factor_company_id": factor_company_id.to_string(),
        "agreement_name": format!("Agreement {}", agreement_number),
        "agreement_type": "spot",
        "recourse_type": "recourse",
        "advance_rate": "0.8000",
        "factoring_fee_rate": "0.0150",
        "late_fee_rate": "0.0050",
        "reserve_rate": "0.0500",
        "minimum_fee": "0.00",
        "currency_code": "USD",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "credit_limit": "500000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/agreements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE AGREEMENT RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create agreement");
    serde_json::from_slice(&b).unwrap()
}

async fn activate_agreement(app: &axum::Router, id: Uuid) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/agreements/{}/activate", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("ACTIVATE AGREEMENT RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::OK, "Failed to activate agreement");
    serde_json::from_slice(&b).unwrap()
}

async fn create_request(
    app: &axum::Router,
    request_number: &str,
    agreement_id: Uuid,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "request_number": request_number,
        "agreement_id": agreement_id.to_string(),
        "request_date": "2026-05-01",
        "recourse_type": "recourse",
        "currency_code": "USD",
        "notes": "Test factoring request",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE REQUEST RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create request");
    serde_json::from_slice(&b).unwrap()
}

async fn add_request_line(
    app: &axum::Router,
    request_id: Uuid,
    line_number: i32,
    invoice_amount: &str,
    eligible_amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_number": line_number,
        "transaction_number": format!("INV-{:04}", line_number),
        "customer_name": "Acme Corp",
        "customer_number": "CUST-001",
        "invoice_date": "2026-04-15",
        "invoice_due_date": "2026-05-15",
        "invoice_amount": invoice_amount,
        "eligible_amount": eligible_amount,
        "days_outstanding": 30,
        "days_overdue": 0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/lines", request_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("ADD LINE RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to add request line");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Factor Company Tests
// ============================================================================

#[tokio::test]
async fn test_create_factor_company() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "FACTOR-01", "First Factor Bank", "0.8500", "0.0200").await;

    assert_eq!(fc["code"], "FACTOR-01");
    assert_eq!(fc["name"], "First Factor Bank");
    assert_eq!(fc["defaultRecourseType"], "recourse");
    assert_eq!(fc["isActive"], true);
}

#[tokio::test]
async fn test_get_factor_company() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "GET-FC", "Get Factor", "0.8000", "0.0150").await;
    let fc_id = fc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/factoring/factor-companies/{}", fc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "GET-FC");
}

#[tokio::test]
async fn test_get_factor_company_by_code() {
    let (_state, app) = setup_test().await;
    create_factor_company(&app, "CODE-FC", "Code Factor", "0.8000", "0.0150").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/factor-companies/code/CODE-FC")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "CODE-FC");
}

#[tokio::test]
async fn test_list_factor_companies() {
    let (_state, app) = setup_test().await;
    create_factor_company(&app, "LIST-FC1", "List Factor 1", "0.8000", "0.0150").await;
    create_factor_company(&app, "LIST-FC2", "List Factor 2", "0.7500", "0.0200").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/factor-companies")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_deactivate_activate_factor_company() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LC-FC", "Lifecycle Factor", "0.8000", "0.0150").await;
    let fc_id = fc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/factor-companies/{}/deactivate", fc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isActive"], false);

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/factor-companies/{}/activate", fc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isActive"], true);
}

#[tokio::test]
async fn test_create_factor_company_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "",
        "name": "No Code Factor",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/factor-companies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_factor_company_invalid_recourse_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD-REC",
        "name": "Bad Recourse",
        "recourse_type": "invalid_type",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/factor-companies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Agreement Tests
// ============================================================================

#[tokio::test]
async fn test_create_agreement() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "AGR-FC", "Agreement Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();

    let agreement = create_agreement(&app, "AGR-001", fc_id).await;

    assert_eq!(agreement["agreementNumber"], "AGR-001");
    assert_eq!(agreement["agreementType"], "spot");
    assert_eq!(agreement["recourseType"], "recourse");
    assert_eq!(agreement["status"], "draft");
}

#[tokio::test]
async fn test_agreement_lifecycle() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LC-AGR-FC", "LC Agreement Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "LC-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();

    // Activate
    let activated = activate_agreement(&app, agr_id).await;
    assert_eq!(activated["status"], "active");

    // Suspend
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/agreements/{}/suspend", agr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "suspended");

    // Terminate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/agreements/{}/terminate", agr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "terminated");
}

#[tokio::test]
async fn test_agreement_with_inactive_factor_fails() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "INACT-FC", "Inactive Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();

    // Deactivate the factor company
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/factor-companies/{}/deactivate", fc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to create agreement with inactive factor
    let payload = json!({
        "agreement_number": "INACT-AGR",
        "factor_company_id": fc_id.to_string(),
        "agreement_name": "Inactive Factor Agreement",
        "start_date": "2026-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/agreements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_agreements_filter_by_status() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LIST-AGR-FC", "List AGR Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let a1 = create_agreement(&app, "LIST-AGR1", fc_id).await;
    let _a2 = create_agreement(&app, "LIST-AGR2", fc_id).await;

    // Activate a1
    let a1_id: Uuid = a1["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, a1_id).await;

    // Filter by active
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/agreements?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let agreements = body["data"].as_array().unwrap();
    assert!(agreements.iter().all(|a| a["status"] == "active"));
}

// ============================================================================
// Request Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_create_factoring_request() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "REQ-FC", "Request Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "REQ-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "REQ-001", agr_id).await;

    assert_eq!(request["requestNumber"], "REQ-001");
    assert_eq!(request["status"], "draft");
    assert_eq!(request["recourseType"], "recourse");
    assert_eq!(request["currencyCode"], "USD");
}

#[tokio::test]
async fn test_request_with_inactive_agreement_fails() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "INACT-AGR-FC", "Inact AGR Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "INACT-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    // Don't activate the agreement

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "request_number": "INACT-REQ",
        "agreement_id": agr_id.to_string(),
        "request_date": "2026-05-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_full_request_lifecycle() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "FULL-FC", "Full Lifecycle Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "FULL-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "FULL-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/submit", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/approve", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Fund
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/fund", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "funded");

    // Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/settle", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "settled");
}

#[tokio::test]
async fn test_cancel_request() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "CANCEL-FC", "Cancel Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "CANCEL-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "CANCEL-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/cancel", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_funded_request_cannot_be_cancelled() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "NCAN-FC", "No Cancel Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "NCAN-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "NCAN-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit → Approve → Fund
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/submit", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/approve", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/fund", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel funded request
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/cancel", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Request Lines & Calculation Tests
// ============================================================================

#[tokio::test]
async fn test_add_request_line_with_calculations() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LINE-FC", "Line Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "LINE-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "LINE-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    // Add line: invoice 10000, eligible 10000
    // Advance rate = 0.80, Fee rate = 0.015, Reserve rate = 0.05
    // Advance = 10000 * 0.80 = 8000, Fee = 10000 * 0.015 = 150, Reserve = 10000 * 0.05 = 500
    let line = add_request_line(&app, req_id, 1, "10000.00", "10000.00").await;

    let advance: f64 = line["advanceAmount"].as_str().unwrap_or("0").parse().unwrap();
    let fee: f64 = line["factoringFeeAmount"].as_str().unwrap_or("0").parse().unwrap();
    let reserve: f64 = line["reserveAmount"].as_str().unwrap_or("0").parse().unwrap();

    assert!((advance - 8000.0).abs() < 1.0, "Expected advance ~8000, got {}", advance);
    assert!((fee - 150.0).abs() < 1.0, "Expected fee ~150, got {}", fee);
    assert!((reserve - 500.0).abs() < 1.0, "Expected reserve ~500, got {}", reserve);
    assert_eq!(line["status"], "pending");
    assert_eq!(line["isEligible"], true);
}

#[tokio::test]
async fn test_request_totals_updated() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "TOTALS-FC", "Totals Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "TOTALS-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "TOTALS-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    // Add two lines
    add_request_line(&app, req_id, 1, "10000.00", "10000.00").await;
    add_request_line(&app, req_id, 2, "20000.00", "20000.00").await;

    // Get the request to check totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/factoring/requests/{}", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total_invoice: f64 = body["totalInvoiceAmount"].as_str().unwrap_or("0").parse().unwrap();
    let advance: f64 = body["advanceAmount"].as_str().unwrap_or("0").parse().unwrap();
    let fee: f64 = body["factoringFeeAmount"].as_str().unwrap_or("0").parse().unwrap();
    let reserve: f64 = body["reserveAmount"].as_str().unwrap_or("0").parse().unwrap();

    assert!((total_invoice - 30000.0).abs() < 1.0, "Expected total ~30000, got {}", total_invoice);
    assert!((advance - 24000.0).abs() < 1.0, "Expected advance ~24000, got {}", advance);
    assert!((fee - 450.0).abs() < 1.0, "Expected fee ~450, got {}", fee);
    assert!((reserve - 1500.0).abs() < 1.0, "Expected reserve ~1500, got {}", reserve);
}

#[tokio::test]
async fn test_list_request_lines() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LIST-LINE-FC", "List Line Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "LIST-LINE-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "LIST-LINE-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    add_request_line(&app, req_id, 1, "5000.00", "5000.00").await;
    add_request_line(&app, req_id, 2, "7000.00", "7000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/factoring/requests/{}/lines", req_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_cannot_add_line_to_submitted_request() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "SUB-LINE-FC", "Submit Line Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "SUB-LINE-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "SUB-LINE-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    // Submit the request
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/submit", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add a line
    let payload = json!({
        "line_number": 1,
        "invoice_amount": "10000.00",
        "eligible_amount": "10000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/lines", req_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_line_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "NEG-FC", "Neg Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "NEG-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let request = create_request(&app, "NEG-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_number": 1,
        "invoice_amount": "-5000.00",
        "eligible_amount": "-5000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/lines", req_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Settlement Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_process_settlement() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "SETTLE-FC", "Settlement Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "SETTLE-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create settlement
    let payload = json!({
        "settlement_number": "SETTLE-001",
        "agreement_id": agr_id.to_string(),
        "settlement_date": "2026-06-01",
        "currency_code": "USD",
        "notes": "Test settlement",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let settlement: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(settlement["status"], "draft");
    let settlement_id: Uuid = settlement["id"].as_str().unwrap().parse().unwrap();

    // Process settlement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/settlements/{}/process", settlement_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "processed");
}

#[tokio::test]
async fn test_list_settlements() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "LIST-SET-FC", "List Settlement Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "LIST-SET-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let (k, v) = auth_header(&admin_claims());
    let payload1 = json!({
        "settlement_number": "LIST-SET-1",
        "agreement_id": agr_id.to_string(),
        "settlement_date": "2026-06-01",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload1).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/settlements")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_factoring_dashboard() {
    let (_state, app) = setup_test().await;
    let fc = create_factor_company(&app, "DASH-FC", "Dashboard Factor", "0.8000", "0.0150").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();
    let agreement = create_agreement(&app, "DASH-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalFactorCompanies").is_some());
    assert!(body.get("activeFactorCompanies").is_some());
    assert!(body.get("totalAgreements").is_some());
    assert!(body.get("activeAgreements").is_some());
    assert!(body.get("totalRequests").is_some());
    assert!(body.get("totalInvoicesFactored").is_some());

    // We created 1 active factor company
    assert!(body["totalFactorCompanies"].as_i64().unwrap() >= 1);
}

// ============================================================================
// End-to-End Full Flow Test
// ============================================================================

#[tokio::test]
async fn test_end_to_end_factoring_flow() {
    let (_state, app) = setup_test().await;

    // 1. Create factor company
    let fc = create_factor_company(&app, "E2E-FC", "E2E Factor Bank", "0.8500", "0.0200").await;
    let fc_id: Uuid = fc["id"].as_str().unwrap().parse().unwrap();

    // 2. Create and activate agreement
    let agreement = create_agreement(&app, "E2E-AGR", fc_id).await;
    let agr_id: Uuid = agreement["id"].as_str().unwrap().parse().unwrap();
    activate_agreement(&app, agr_id).await;

    // 3. Create factoring request
    let request = create_request(&app, "E2E-REQ", agr_id).await;
    let req_id: Uuid = request["id"].as_str().unwrap().parse().unwrap();

    // 4. Add invoice lines
    let line1 = add_request_line(&app, req_id, 1, "15000.00", "15000.00").await;
    let line2 = add_request_line(&app, req_id, 2, "25000.00", "25000.00").await;

    // Verify line calculations (advance_rate=0.80, fee_rate=0.015, reserve_rate=0.05)
    let l1_advance: f64 = line1["advanceAmount"].as_str().unwrap().parse().unwrap();
    let l1_fee: f64 = line1["factoringFeeAmount"].as_str().unwrap().parse().unwrap();
    assert!((l1_advance - 12000.0).abs() < 1.0);
    assert!((l1_fee - 225.0).abs() < 1.0);

    let l2_advance: f64 = line2["advanceAmount"].as_str().unwrap().parse().unwrap();
    let l2_fee: f64 = line2["factoringFeeAmount"].as_str().unwrap().parse().unwrap();
    assert!((l2_advance - 20000.0).abs() < 1.0);
    assert!((l2_fee - 375.0).abs() < 1.0);

    // 5. Submit → Approve → Fund
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/submit", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/approve", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/fund", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let funded: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(funded["status"], "funded");

    // Verify funded request totals
    let total_invoice: f64 = funded["totalInvoiceAmount"].as_str().unwrap_or("0").parse().unwrap();
    assert!((total_invoice - 40000.0).abs() < 1.0, "Expected 40000, got {}", total_invoice);

    // 6. Create settlement
    let payload = json!({
        "settlement_number": "E2E-SETTLE-001",
        "agreement_id": agr_id.to_string(),
        "request_id": req_id.to_string(),
        "settlement_date": "2026-06-15",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/factoring/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let settlement: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let settle_id: Uuid = settlement["id"].as_str().unwrap().parse().unwrap();

    // Process settlement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/settlements/{}/process", settle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 7. Settle the request
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/factoring/requests/{}/settle", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let settled: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(settled["status"], "settled");

    // 8. Verify dashboard
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/factoring/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dashboard: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(dashboard["totalFactorCompanies"].as_i64().unwrap() >= 1);
    assert!(dashboard["settledRequests"].as_i64().unwrap() >= 1);
}
