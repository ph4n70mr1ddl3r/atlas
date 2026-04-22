//! Procurement Contracts E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Procurement Contracts:
//! - Contract type CRUD
//! - Contract lifecycle (create → add lines → submit → approve → record spend → close)
//! - Contract line management
//! - Milestone management
//! - Contract renewal
//! - Spend tracking
//! - Contract termination
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_procurement_contracts_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Run migration for procurement contracts tables
    sqlx::query(include_str!("../../../../migrations/029_procurement_contracts.sql"))
        .execute(&state.db_pool)
        .await
        .ok(); // Ignore errors if tables already exist
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_contract_type(
    app: &axum::Router,
    code: &str,
    name: &str,
    classification: &str,
    requires_approval: bool,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "contract_classification": classification,
        "requires_approval": requires_approval,
        "allow_amount_commitment": true,
        "allow_quantity_commitment": true,
        "allow_line_additions": true,
        "allow_price_adjustment": false,
        "allow_renewal": true,
        "allow_termination": true,
        "max_renewals": 3,
        "default_currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts/types")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create contract type");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_contract(
    app: &axum::Router,
    title: &str,
    supplier_id: &str,
    classification: &str,
    start_date: &str,
    end_date: &str,
    committed_amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "title": title,
        "contract_classification": classification,
        "supplier_id": supplier_id,
        "supplier_name": "Test Supplier Corp",
        "supplier_number": "SUP-001",
        "buyer_name": "Procurement Officer",
        "start_date": start_date,
        "end_date": end_date,
        "total_committed_amount": committed_amount,
        "currency_code": "USD",
        "payment_terms_code": "NET30",
        "price_type": "fixed",
        "max_renewals": 3,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create contract");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_contract_line(
    app: &axum::Router,
    contract_id: &str,
    description: &str,
    quantity: &str,
    unit_price: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_description": description,
        "item_code": "ITEM-001",
        "category": "Office Supplies",
        "uom": "EA",
        "quantity_committed": quantity,
        "unit_price": unit_price,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/lines", contract_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add contract line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_milestone(
    app: &axum::Router,
    contract_id: &str,
    name: &str,
    milestone_type: &str,
    target_date: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": name,
        "milestone_type": milestone_type,
        "target_date": target_date,
        "amount": amount,
        "percent_of_total": "50.0",
        "deliverable": "Delivery of goods",
        "is_billable": true,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/milestones", contract_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add milestone");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Contract Type Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_type_crud() {
    let (_state, app) = setup_procurement_contracts_test().await;

    // Create
    let ct = create_test_contract_type(&app, "BPA", "Blanket Purchase Agreement", "blanket", true).await;
    assert_eq!(ct["code"], "BPA");
    assert_eq!(ct["name"], "Blanket Purchase Agreement");
    assert_eq!(ct["contract_classification"], "blanket");
    assert_eq!(ct["requires_approval"], true);
    assert_eq!(ct["is_active"], true);

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/procurement-contracts/types/BPA")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "BPA");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/procurement-contracts/types")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/procurement-contracts/types/BPA")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/procurement-contracts/types/BPA")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_contract_type_validation() {
    let (_state, app) = setup_procurement_contracts_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Empty code
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Test",
            "contract_classification": "blanket",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid classification
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST",
            "name": "Test",
            "contract_classification": "invalid_type",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Contract Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_full_lifecycle() {
    let (_state, app) = setup_procurement_contracts_test().await;

    // Step 1: Create a contract type that requires approval
    create_test_contract_type(&app, "STD", "Standard Contract", "blanket", true).await;

    // Step 2: Create a contract
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "IT Services Agreement 2026",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "50000.00",
    ).await;

    assert_eq!(contract["title"], "IT Services Agreement 2026");
    assert_eq!(contract["status"], "draft");
    assert_eq!(contract["contract_classification"], "blanket");
    assert_eq!(contract["currency_code"], "USD");
    assert_eq!(contract["price_type"], "fixed");
    assert!(contract["contract_number"].as_str().unwrap().starts_with("PC-"));
    let contract_id = contract["id"].as_str().unwrap().to_string();

    // Step 3: Add lines
    let line1 = add_test_contract_line(&app, &contract_id, "Laptop Computers", "10", "1200.00").await;
    assert_eq!(line1["item_description"], "Laptop Computers");
    assert_eq!(line1["line_number"], 1);
    let line_amount_1: f64 = line1["line_amount"].as_str().unwrap().parse().unwrap();
    assert!((line_amount_1 - 12000.0).abs() < 0.01);

    let line2 = add_test_contract_line(&app, &contract_id, "Desktop Monitors", "20", "350.00").await;
    assert_eq!(line2["line_number"], 2);

    // Verify contract total was recalculated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let updated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    // total_committed_amount should reflect line amounts (12000 + 7000 = 19000)
    let committed: f64 = updated["total_committed_amount"].as_str().unwrap().parse().unwrap();
    assert!((committed - 19000.0).abs() < 0.01, "Expected 19000, got {}", committed);

    // Step 4: Add a milestone
    let milestone = add_test_milestone(
        &app, &contract_id,
        "First Delivery", "delivery",
        "2026-03-31", "9500.00",
    ).await;
    assert_eq!(milestone["name"], "First Delivery");
    assert_eq!(milestone["status"], "pending");
    assert_eq!(milestone["milestone_type"], "delivery");

    // Step 5: Submit for approval
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let submitted: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(submitted["status"], "pending_approval");

    // Step 6: Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/approve", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(approved["status"], "active");

    // Step 7: Record spend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/spend", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "source_type": "purchase_order",
            "source_number": "PO-2026-001",
            "transaction_date": "2026-02-15",
            "amount": "5000.00",
            "description": "Partial delivery - laptops"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Step 8: Update milestone
    let milestone_id = milestone["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/procurement-contracts/milestones/{}", milestone_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "completed",
            "actual_date": "2026-03-28",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let ms: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(ms["status"], "completed");

    // Step 9: Close contract
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/close", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let closed: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(closed["status"], "closed");
}

// ============================================================================
// Contract Rejection Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_rejection() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Rejected Contract",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "10000.00",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/reject", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Budget constraints - try next quarter"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let rejected: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(rejected["status"], "draft");
    assert_eq!(rejected["rejection_reason"], "Budget constraints - try next quarter");

    // Can re-submit after rejection (goes back to draft)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Contract Termination Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_termination() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "service", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Service Contract - Terminated",
        supplier_id, "service",
        "2026-01-01", "2026-12-31", "25000.00",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/approve", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    // Terminate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/terminate", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Supplier failed to meet SLA requirements"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let terminated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(terminated["status"], "terminated");
    assert_eq!(terminated["termination_reason"], "Supplier failed to meet SLA requirements");
}

// ============================================================================
// Contract Renewal Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_renewal() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "BPA", "Blanket Purchase Agreement", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Annual Software License",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "75000.00",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/approve", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    // Renew
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/renewals", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "new_end_date": "2027-12-31",
            "renewal_type": "negotiated",
            "terms_changed": "5% price increase accepted",
            "notes": "Annual renewal with price adjustment"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let renewal: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(renewal["renewal_number"], 1);
    assert_eq!(renewal["previous_end_date"], "2026-12-31");
    assert_eq!(renewal["new_end_date"], "2027-12-31");
    assert_eq!(renewal["renewal_type"], "negotiated");

    // Verify contract was updated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let updated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(updated["end_date"], "2027-12-31");
    assert_eq!(updated["renewal_count"], 1);
    assert_eq!(updated["status"], "active");

    // List renewals
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}/renewals", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let renewals: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(renewals["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Spend Tracking Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_spend_tracking() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Office Supplies Contract",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "100000.00",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/approve", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    // Record multiple spend entries
    for i in 0..3 {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/procurement-contracts/{}/spend", contract_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "source_type": "invoice",
                "source_number": format!("INV-2026-{:03}", i + 1),
                "transaction_date": "2026-02-15",
                "amount": "10000.00",
                "description": format!("Monthly delivery #{}", i + 1)
            })).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // Verify total released was updated (3 * 10000 = 30000)
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let updated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let released: f64 = updated["total_released_amount"].as_str().unwrap().parse().unwrap();
    assert!((released - 30000.0).abs() < 0.01, "Expected 30000, got {}", released);

    // List spend entries
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}/spend", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let spend: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(spend["data"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_dashboard_summary() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";

    // Create multiple contracts
    create_test_contract(&app, "Contract A", supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "50000.00").await;
    create_test_contract(&app, "Contract B", supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "75000.00").await;

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/procurement-contracts/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(summary["total_contracts"], 2);
    assert_eq!(summary["active_contracts"], 0); // Both still in draft
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_contract_validation_errors() {
    let (_state, app) = setup_procurement_contracts_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Empty title
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "",
            "supplier_id": "00000000-0000-0000-0000-000000000100",
            "contract_classification": "blanket",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid classification
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Test",
            "supplier_id": "00000000-0000-0000-0000-000000000100",
            "contract_classification": "nonexistent",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // End date before start date
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Test Date Validation",
            "supplier_id": "00000000-0000-0000-0000-000000000100",
            "contract_classification": "blanket",
            "currency_code": "USD",
            "start_date": "2026-12-31",
            "end_date": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_record_spend_on_draft_contract() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Draft Contract - No Spend",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "10000.00",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to record spend on a draft contract - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/spend", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "source_type": "invoice",
            "transaction_date": "2026-02-15",
            "amount": "5000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_auto_approve_without_approval_requirement() {
    let (_state, app) = setup_procurement_contracts_test().await;

    // Create a contract type that does NOT require approval
    create_test_contract_type(&app, "AUTO", "Auto-Approved Contract", "blanket", false).await;

    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/procurement-contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Auto-Approved Contract",
            "contract_type_code": "AUTO",
            "contract_classification": "blanket",
            "supplier_id": supplier_id,
            "supplier_name": "Quick Supplier",
            "currency_code": "USD",
            "price_type": "fixed",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let contract: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let contract_id = contract["id"].as_str().unwrap();

    // Submit should auto-activate since type doesn't require approval
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/procurement-contracts/{}/submit", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let submitted: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(submitted["status"], "active"); // Auto-activated!
}

#[tokio::test]
#[ignore]
async fn test_contract_line_deletion() {
    let (_state, app) = setup_procurement_contracts_test().await;

    create_test_contract_type(&app, "STD", "Standard", "blanket", true).await;
    let supplier_id = "00000000-0000-0000-0000-000000000100";
    let contract = create_test_contract(
        &app, "Contract With Line Deletion",
        supplier_id, "blanket",
        "2026-01-01", "2026-12-31", "0",
    ).await;
    let contract_id = contract["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add two lines
    let line1 = add_test_contract_line(&app, &contract_id, "Item A", "5", "100.00").await;
    let _line2 = add_test_contract_line(&app, &contract_id, "Item B", "10", "200.00").await;

    let line1_id = line1["id"].as_str().unwrap();

    // Delete first line
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/procurement-contracts/lines/{}", line1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify remaining lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/procurement-contracts/{}/lines", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 1);
    assert_eq!(lines["data"][0]["item_description"], "Item B");
}
