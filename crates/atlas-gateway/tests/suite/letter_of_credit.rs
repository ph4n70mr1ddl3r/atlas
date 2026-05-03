//! Letter of Credit Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Letter of Credit Management:
//! - LC CRUD (create, get, list, delete)
//! - Full LC lifecycle (draft → issued → advised → confirmed → accepted → paid)
//! - LC cancellation and expiration processing
//! - LC amendments (create, approve, reject)
//! - Required document management
//! - Shipment management
//! - Presentation management (create, accept, pay, reject)
//! - Presentation documents
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
    sqlx::query(include_str!("../../../../migrations/122_letter_of_credit.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_lc(
    app: &axum::Router,
    lc_number: &str,
    lc_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": lc_number,
        "lcType": lc_type,
        "lcForm": "irrevocable",
        "description": "Test letter of credit",
        "applicantName": "Global Import Corp",
        "applicantBankName": "First National Bank",
        "beneficiaryName": "Overseas Export Ltd",
        "beneficiaryBankName": "International Trade Bank",
        "lcAmount": "50000.00",
        "currencyCode": "USD",
        "availableBy": "payment",
        "expiryDate": "2025-12-31",
        "portOfLoading": "Shanghai",
        "portOfDischarge": "Los Angeles",
        "goodsDescription": "Electronic components",
        "incoterms": "CIF",
        "bankCharges": "beneficiary",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create LC: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn issue_lc(app: &axum::Router, lc_id: Uuid) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/issue", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "issueDate": "2024-06-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("ISSUE RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::OK, "Failed to issue LC");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// LC CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_lc() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-001", "import").await;

    assert_eq!(lc["lcNumber"], "LC-001");
    assert_eq!(lc["lcType"], "import");
    assert_eq!(lc["lcForm"], "irrevocable");
    assert_eq!(lc["status"], "draft");
    assert!(lc["lcAmount"].as_str().unwrap().contains("50000"));
    assert_eq!(lc["currencyCode"], "USD");
}

#[tokio::test]
async fn test_get_lc() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-GET", "import").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit/LC-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["lcNumber"], "LC-GET");
}

#[tokio::test]
async fn test_list_lcs() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-LIST1", "import").await;
    create_test_lc(&app, "LC-LIST2", "export").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_lcs_with_status_filter() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-FILTER", "import").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_lcs_with_type_filter() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-TYPE-FILTER", "export").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit?lc_type=export")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let lcs = body["data"].as_array().unwrap();
    assert!(lcs.len() >= 1);
    assert!(lcs.iter().all(|lc| lc["lcType"] == "export"));
}

#[tokio::test]
async fn test_delete_draft_lc() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-DEL", "import").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/letters-of-credit/LC-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// LC Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_lc_lifecycle() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-LIFE", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Issue
    let r = issue_lc(&app, lc_id).await;
    assert_eq!(r["status"], "issued");
    assert!(r["issueDate"].is_string());

    // Advise
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/advise", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "advised");

    // Confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/confirm", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "confirmed");

    // Accept
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/accept", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "accepted");

    // Pay
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/pay", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "paid");
}

#[tokio::test]
async fn test_cancel_lc() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-CANCEL", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/cancel", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_process_expired_lcs() {
    let (_state, app) = setup_test().await;

    // Create LC with already-past expiry date
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": "LC-EXPIRE",
        "lcType": "import",
        "lcForm": "irrevocable",
        "applicantName": "Test Corp",
        "applicantBankName": "Test Bank",
        "beneficiaryName": "Test Beneficiary",
        "lcAmount": "25000.00",
        "expiryDate": "2024-01-01",  // Past date
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let lc: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Issue it so it's not draft
    issue_lc(&app, lc_id).await;

    // Process expired
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit/process-expired")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "asOfDate": "2025-06-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["expired_count"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Amendment Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_approve_amendment() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-AMD", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Issue first so it can be amended
    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_increase",
            "previousAmount": "50000.00",
            "newAmount": "75000.00",
            "reason": "Increase order quantity"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let amendment: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(amendment["amendmentType"], "amount_increase");
    assert_eq!(amendment["status"], "draft");
    let amendment_id: Uuid = amendment["id"].as_str().unwrap().parse().unwrap();

    // Approve amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/amendments/{}/approve", amendment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Verify LC was updated
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit/LC-AMD")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let lc_updated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(lc_updated["amendmentCount"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_reject_amendment() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-REJ-AMD", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "expiry_extension",
            "newExpiryDate": "2026-06-30",
            "reason": "Extend delivery timeline"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let amendment: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let amendment_id: Uuid = amendment["id"].as_str().unwrap().parse().unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/amendments/{}/reject", amendment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "rejected");
}

#[tokio::test]
async fn test_list_amendments() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-AMD-LIST", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create two amendments
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_increase",
            "newAmount": "60000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "terms_change",
            "newTerms": "Revised payment terms"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Required Documents Tests
// ============================================================================

#[tokio::test]
async fn test_required_documents() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-DOCS", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add required document
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/required-documents", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "Bill of Lading",
            "documentCode": "BOL",
            "description": "Full set of clean on board bills of lading",
            "originalCopies": 3,
            "copyCount": 1,
            "isMandatory": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let doc: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(doc["documentType"], "Bill of Lading");
    assert_eq!(doc["originalCopies"], 3);
    assert_eq!(doc["isMandatory"], true);

    // List required documents
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/letters-of-credit/{}/required-documents", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);

    // Delete required document
    let doc_id = doc["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/letters-of-credit/required-documents/{}", doc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Shipment Tests
// ============================================================================

#[tokio::test]
async fn test_shipment_management() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-SHIP", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create shipment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/shipments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipmentNumber": "SHP-001",
            "vesselName": "MV Pacific Star",
            "billOfLadingNumber": "BOL-2024-001",
            "portOfLoading": "Shanghai",
            "portOfDischarge": "Los Angeles",
            "shipmentDate": "2024-07-15",
            "expectedArrivalDate": "2024-08-15",
            "goodsDescription": "Electronic components - batch 1",
            "quantity": "1000",
            "unitPrice": "50.00",
            "shipmentAmount": "50000.00",
            "currencyCode": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let shipment: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(shipment["shipmentNumber"], "SHP-001");
    assert_eq!(shipment["vesselName"], "MV Pacific Star");
    assert_eq!(shipment["status"], "pending");
    let shipment_id: Uuid = shipment["id"].as_str().unwrap().parse().unwrap();

    // List shipments
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/letters-of-credit/{}/shipments", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);

    // Update shipment status
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/shipments/{}/shipped", shipment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "shipped");
}

// ============================================================================
// Presentation Tests
// ============================================================================

#[tokio::test]
async fn test_presentation_lifecycle() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-PRES", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Issue the LC first
    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-001",
            "presentationDate": "2024-08-01",
            "presentingBankName": "International Trade Bank",
            "totalAmount": "50000.00",
            "currencyCode": "USD",
            "discrepant": false
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let pres: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(pres["presentationNumber"], "PRES-001");
    assert_eq!(pres["status"], "submitted");
    assert_eq!(pres["discrepant"], false);
    let pres_id: Uuid = pres["id"].as_str().unwrap().parse().unwrap();

    // Accept presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/presentations/{}/accept", pres_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "accepted");

    // Pay presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/presentations/{}/pay", pres_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "paidAmount": "50000.00",
            "paymentDate": "2024-08-15"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "paid");
}

#[tokio::test]
async fn test_discrepant_presentation() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-DISC", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create discrepant presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-DISC",
            "presentationDate": "2024-08-01",
            "totalAmount": "50000.00",
            "discrepant": true,
            "discrepancies": "Late presentation; Missing insurance certificate"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let pres: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(pres["discrepant"], true);
    let pres_id: Uuid = pres["id"].as_str().unwrap().parse().unwrap();

    // Reject the discrepant presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/presentations/{}/reject", pres_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "rejected");
}

#[tokio::test]
async fn test_list_presentations() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-PRES-LIST", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create two presentations
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-LIST1",
            "presentationDate": "2024-08-01",
            "totalAmount": "30000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-LIST2",
            "presentationDate": "2024-09-01",
            "totalAmount": "20000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Presentation Documents Tests
// ============================================================================

#[tokio::test]
async fn test_presentation_documents() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-PDOC", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());

    // Create presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-PDOC",
            "presentationDate": "2024-08-01",
            "totalAmount": "50000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let pres: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let pres_id: Uuid = pres["id"].as_str().unwrap().parse().unwrap();

    // Add document to presentation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/presentations/{}/documents", pres_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "documentType": "Bill of Lading",
            "documentReference": "BOL-2024-001",
            "description": "Clean on board bill of lading",
            "originalCopies": 3,
            "copyCount": 1,
            "isCompliant": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let doc: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(doc["documentType"], "Bill of Lading");
    assert_eq!(doc["isCompliant"], true);

    // List presentation documents
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/letters-of-credit/presentations/{}/documents", pres_id))
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
async fn test_lc_dashboard() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-DASH1", "import").await;
    create_test_lc(&app, "LC-DASH2", "export").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalActiveLcs").is_some());
    assert!(body.get("totalLcAmount").is_some());
    assert!(body.get("totalPendingAmendments").is_some());
    assert!(body.get("totalPresentationsPending").is_some());
    assert!(body.get("totalDiscrepantPresentations").is_some());
    assert!(body.get("expiringWithin30Days").is_some());
    assert!(body.get("expiringWithin90Days").is_some());
    assert!(body.get("byType").is_some());
    assert!(body.get("byCurrency").is_some());
    assert!(body.get("byStatus").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_lc_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": "",
        "lcType": "import",
        "applicantName": "Test Corp",
        "applicantBankName": "Test Bank",
        "beneficiaryName": "Test Beneficiary",
        "lcAmount": "10000.00",
        "expiryDate": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_lc_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": "LC-BAD-TYPE",
        "lcType": "invalid_type",
        "applicantName": "Test Corp",
        "applicantBankName": "Test Bank",
        "beneficiaryName": "Test Beneficiary",
        "lcAmount": "10000.00",
        "expiryDate": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_lc_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": "LC-ZERO",
        "lcType": "import",
        "applicantName": "Test Corp",
        "applicantBankName": "Test Bank",
        "beneficiaryName": "Test Beneficiary",
        "lcAmount": "0",
        "expiryDate": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_lc_duplicate_number_fails() {
    let (_state, app) = setup_test().await;
    create_test_lc(&app, "LC-DUP", "import").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lcNumber": "LC-DUP",
        "lcType": "import",
        "applicantName": "Test Corp",
        "applicantBankName": "Test Bank",
        "beneficiaryName": "Test Beneficiary",
        "lcAmount": "10000.00",
        "expiryDate": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/letters-of-credit")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_issue_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-ISS-FAIL", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Issue once
    issue_lc(&app, lc_id).await;

    // Try to issue again
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/issue", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "issueDate": "2024-07-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_pay_non_accepted_lc_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-PAY-FAIL", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Issue but don't accept
    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/pay", lc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_paid_lc_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-CANCEL-PAID", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Full lifecycle to paid
    issue_lc(&app, lc_id).await;
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/advise", lc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/confirm", lc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/accept", lc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/pay", lc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel a paid LC
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/cancel", lc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_issued_lc_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-DEL-ISS", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    issue_lc(&app, lc_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/letters-of-credit/LC-DEL-ISS")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete returns NOT_FOUND for non-draft LCs (deleted 0 rows)
    assert!(r.status() == StatusCode::BAD_REQUEST || r.status() == StatusCode::NOT_FOUND, "Expected 400 or 404, got {}", r.status());
}

#[tokio::test]
async fn test_amend_draft_lc_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-AMD-DRAFT", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Don't issue - try to amend directly
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/amendments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_increase",
            "newAmount": "60000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_presentation_for_draft_lc_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-PRES-DRAFT", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    // Don't issue - try to create presentation directly
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/presentations", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "presentationNumber": "PRES-DRAFT-FAIL",
            "presentationDate": "2024-08-01",
            "totalAmount": "50000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_shipment_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let lc = create_test_lc(&app, "LC-SHIP-ZERO", "import").await;
    let lc_id: Uuid = lc["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/letters-of-credit/{}/shipments", lc_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipmentNumber": "SHP-ZERO",
            "shipmentAmount": "0"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_lc_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/letters-of-credit/NONEXISTENT")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
