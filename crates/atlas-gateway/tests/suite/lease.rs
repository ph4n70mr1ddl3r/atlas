//! Lease Accounting E2E Tests (ASC 842 / IFRS 16)
//!
//! Tests for Oracle Fusion Cloud ERP Lease Management:
//! - Lease contract CRUD
//! - Lease activation and amortization schedule generation
//! - Payment processing
//! - Lease modifications
//! - ROU asset impairment
//! - Lease termination with gain/loss
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_lease_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_lease(
    app: &axum::Router,
    title: &str,
    classification: &str,
    annual_payment: &str,
    term_months: i32,
    discount_rate: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "title": title,
        "classification": classification,
        "asset_description": "Office Space - Floor 3",
        "location": "Corporate HQ",
        "department_name": "Operations",
        "commencement_date": "2024-01-01",
        "end_date": "2026-12-31",
        "lease_term_months": term_months,
        "discount_rate": discount_rate,
        "currency_code": "USD",
        "payment_frequency": "monthly",
        "annual_payment_amount": annual_payment,
        "rou_asset_account_code": "1600",
        "rou_depreciation_account_code": "1610",
        "lease_liability_account_code": "2200",
        "lease_expense_account_code": "6300",
        "interest_expense_account_code": "6310",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lease/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create lease: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Lease Contract Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_lease_contract() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease - Building A", "operating", "120000.00", 36, "0.05").await;

    assert_eq!(lease["title"], "Office Lease - Building A");
    assert_eq!(lease["classification"], "operating");
    assert_eq!(lease["status"], "draft");
    assert_eq!(lease["payment_frequency"], "monthly");
    assert!(lease["initial_lease_liability"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
    assert!(lease["initial_rou_asset_value"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
    assert_eq!(lease["lease_term_months"], 36);
}

#[tokio::test]
#[ignore]
async fn test_create_finance_lease() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Equipment Finance Lease", "finance", "60000.00", 24, "0.04").await;

    assert_eq!(lease["classification"], "finance");
    assert_eq!(lease["status"], "draft");
}

#[tokio::test]
#[ignore]
async fn test_list_leases() {
    let (_state, app) = setup_lease_test().await;

    create_test_lease(&app, "Office A", "operating", "120000.00", 36, "0.05").await;
    create_test_lease(&app, "Equipment B", "finance", "60000.00", 24, "0.04").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/lease/contracts")
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
async fn test_list_leases_by_classification() {
    let (_state, app) = setup_lease_test().await;

    create_test_lease(&app, "Office A", "operating", "120000.00", 36, "0.05").await;
    create_test_lease(&app, "Equipment B", "finance", "60000.00", 24, "0.04").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/lease/contracts?classification=operating")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    assert_eq!(result["data"][0]["classification"], "operating");
}

#[tokio::test]
#[ignore]
async fn test_get_lease() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Warehouse Lease", "operating", "96000.00", 48, "0.06").await;
    let lease_id = lease["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}", lease_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["title"], "Warehouse Lease");
}

// ============================================================================
// Lease Activation & Amortization Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_activate_lease_generates_schedule() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease", "operating", "120000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();
    assert_eq!(lease["status"], "draft");

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");

    // Check payment schedule was generated (12 months = 12 payments)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}/payments", lease_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let payments: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let payment_list = payments["data"].as_array().unwrap();
    assert_eq!(payment_list.len(), 12);

    // Verify first payment has correct structure
    let first = &payment_list[0];
    assert_eq!(first["period_number"], 1);
    assert_eq!(first["status"], "scheduled");
    assert!(!first["is_paid"].as_bool().unwrap());

    // Interest portion should be positive
    let interest: f64 = first["interest_amount"].as_str().unwrap().parse().unwrap();
    assert!(interest > 0.0, "First period interest should be positive");

    // Principal portion should be positive
    let principal: f64 = first["principal_amount"].as_str().unwrap().parse().unwrap();
    assert!(principal > 0.0, "First period principal should be positive");

    // Total payment ≈ interest + principal
    let total: f64 = first["payment_amount"].as_str().unwrap().parse().unwrap();
    assert!((total - interest - principal).abs() < 0.01);
}

// ============================================================================
// Payment Processing Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_process_lease_payment() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease", "operating", "120000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Process payment for period 1
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/payments", lease_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "period_number": 1,
            "payment_reference": "PAY-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let payment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(payment["status"], "paid");
    assert_eq!(payment["is_paid"], true);
    assert_eq!(payment["payment_reference"], "PAY-001");
}

// ============================================================================
// Lease Modification Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_lease_modification() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease", "operating", "120000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create modification
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/modifications", lease_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modification_type": "term_extension",
            "description": "Extended lease by 6 months",
            "effective_date": "2025-01-01",
            "new_term_months": 18,
            "new_end_date": "2025-06-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let modification: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(modification["modification_type"], "term_extension");
    assert_eq!(modification["new_term_months"], 18);
    assert_eq!(modification["modification_number"], 1);

    // Verify lease status changed to modified
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "modified");
}

// ============================================================================
// Impairment Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_record_lease_impairment() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease", "operating", "120000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Record impairment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/impairment", lease_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "impairment_amount": "5000.00",
            "impairment_date": "2025-03-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let impaired: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(impaired["status"], "impaired");
}

// ============================================================================
// Termination Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_terminate_lease() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Office Lease", "operating", "120000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Terminate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/terminate", lease_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "termination_type": "mutual_agreement",
            "termination_date": "2025-06-30",
            "termination_penalty": "5000.00",
            "reason": "Business relocation"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let termination: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(termination["termination_type"], "mutual_agreement");
    assert_eq!(termination["status"], "pending");
    assert!(termination["gain_loss_amount"].as_str().unwrap().parse::<f64>().unwrap() >= 0.0);

    // Verify lease status changed to terminated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "terminated");

    // Verify termination is listed
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}/terminations", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let terminations: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(terminations["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_lease_full_lifecycle() {
    let (_state, app) = setup_lease_test().await;

    // 1. Create
    let lease = create_test_lease(&app, "Full Lifecycle Lease", "operating", "60000.00", 6, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();
    assert_eq!(lease["status"], "draft");

    // 2. Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let activated: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(activated["status"], "active");

    // 3. Process payments for periods 1 and 2
    for period in 1..=2 {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/lease/contracts/{}/payments", lease_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "period_number": period,
                "payment_reference": format!("PAY-{}", period)
            })).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

    // 4. Verify lease balances updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/lease/contracts/{}", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["periods_elapsed"], 2);
    let payments_made: f64 = updated["total_payments_made"].as_str().unwrap().parse().unwrap();
    assert!(payments_made > 0.0, "Should have recorded payments");

    // 5. Terminate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/terminate", lease_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "termination_type": "early",
            "termination_date": "2024-03-01",
            "termination_penalty": "0",
            "reason": "Early exit clause exercised"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_activate_non_draft_lease() {
    let (_state, app) = setup_lease_test().await;

    let lease = create_test_lease(&app, "Test", "operating", "60000.00", 12, "0.05").await;
    let lease_id = lease["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to activate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lease_with_invalid_classification() {
    let (_state, app) = setup_lease_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lease/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Bad Lease",
            "classification": "invalid",
            "commencement_date": "2024-01-01",
            "end_date": "2026-12-31",
            "lease_term_months": 36,
            "discount_rate": "0.05",
            "annual_payment_amount": "120000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lease_with_empty_title() {
    let (_state, app) = setup_lease_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lease/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "",
            "classification": "operating",
            "commencement_date": "2024-01-01",
            "end_date": "2026-12-31",
            "lease_term_months": 36,
            "discount_rate": "0.05",
            "annual_payment_amount": "120000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lease_with_invalid_dates() {
    let (_state, app) = setup_lease_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lease/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Bad Dates Lease",
            "classification": "operating",
            "commencement_date": "2026-12-31",
            "end_date": "2024-01-01",
            "lease_term_months": 36,
            "discount_rate": "0.05",
            "annual_payment_amount": "120000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_lease_with_invalid_discount_rate() {
    let (_state, app) = setup_lease_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lease/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Bad Rate Lease",
            "classification": "operating",
            "commencement_date": "2024-01-01",
            "end_date": "2026-12-31",
            "lease_term_months": 36,
            "discount_rate": "1.5",
            "annual_payment_amount": "120000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_lease_dashboard() {
    let (_state, app) = setup_lease_test().await;

    // Create and activate leases
    let lease1 = create_test_lease(&app, "Office A", "operating", "120000.00", 12, "0.05").await;
    let lease2 = create_test_lease(&app, "Equipment B", "finance", "60000.00", 12, "0.04").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease1["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/lease/contracts/{}/activate", lease2["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/lease/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["total_active_leases"], 2);
    assert_eq!(summary["operating_lease_count"], 1);
    assert_eq!(summary["finance_lease_count"], 1);
    assert!(summary["total_lease_liability"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
}
