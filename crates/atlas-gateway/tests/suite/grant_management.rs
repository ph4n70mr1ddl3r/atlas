//! Grant Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Grants Management:
//! - Sponsor CRUD
//! - Indirect cost rate CRUD
//! - Award CRUD and lifecycle (activate, suspend, complete, terminate)
//! - Budget line management
//! - Expenditure recording and approval
//! - Expenditure reversal
//! - Sponsor billing lifecycle (create, submit, approve, pay)
//! - Compliance reporting lifecycle (create, submit, approve)
//! - Dashboard summary
//! - Validation edge cases
//! - Indirect cost calculation verification

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_grant_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    sqlx::query(include_str!("../../../../migrations/036_grant_management.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_sponsor(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "sponsor_code": code,
        "name": name,
        "sponsor_type": "government",
        "contact_name": "Dr. Smith",
        "contact_email": "smith@nih.gov",
        "billing_frequency": "monthly",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/grants/sponsors")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create sponsor");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_award(
    app: &axum::Router,
    award_number: &str,
    sponsor_id: Uuid,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "award_number": award_number,
        "award_title": "Cancer Research Grant",
        "sponsor_id": sponsor_id,
        "award_type": "research",
        "start_date": "2025-01-01",
        "end_date": "2025-12-31",
        "total_award_amount": "500000",
        "direct_costs_total": "400000",
        "indirect_costs_total": "100000",
        "currency_code": "USD",
        "indirect_cost_rate": "50",
        "billing_frequency": "monthly",
        "billing_basis": "cost",
        "principal_investigator_name": "Dr. Jane Doe",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/grants/awards")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create award");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Sponsor CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_sponsor_crud() {
    let (_state, app) = setup_grant_test().await;

    // Create
    let sponsor = create_test_sponsor(&app, "NIH", "National Institutes of Health").await;
    assert_eq!(sponsor["sponsor_code"], "NIH");
    assert_eq!(sponsor["name"], "National Institutes of Health");
    assert_eq!(sponsor["sponsor_type"], "government");

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/sponsors/NIH")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(got["sponsor_code"], "NIH");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/sponsors")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/grants/sponsors/NIH")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Indirect Cost Rate Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_indirect_cost_rate_crud() {
    let (_state, app) = setup_grant_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Create
    let payload = json!({
        "rate_name": "FY2025 Negotiated Rate",
        "rate_type": "negotiated",
        "rate_percentage": "52.5",
        "base_type": "modified_total_direct_costs",
        "effective_from": "2025-01-01",
        "effective_to": "2025-12-31",
        "negotiated_by": "DHHS"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/grants/indirect-cost-rates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let rate: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(rate["rate_name"], "FY2025 Negotiated Rate");
    assert_eq!(rate["rate_type"], "negotiated");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/indirect-cost-rates")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Award Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_award_lifecycle() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create sponsor first
    let sponsor = create_test_sponsor(&app, "NSF", "National Science Foundation").await;
    let sponsor_id = sponsor["id"].as_str().unwrap();

    // Create award
    let award = create_test_award(&app, "AWD-2025-001", Uuid::parse_str(sponsor_id).unwrap()).await;
    assert_eq!(award["award_number"], "AWD-2025-001");
    assert_eq!(award["status"], "draft");
    assert_eq!(award["award_type"], "research");

    let award_id = award["id"].as_str().unwrap();

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let activated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(activated["status"], "active");

    // Suspend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/suspend", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let suspended: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(suspended["status"], "suspended");

    // Terminate (from suspended)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/terminate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let terminated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(terminated["status"], "terminated");
}

#[tokio::test]
#[ignore]
async fn test_award_activate_only_from_draft() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "DOD", "Dept of Defense").await;
    let award = create_test_award(&app, "AWD-ERR-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Try to activate again (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Budget Line Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_budget_lines() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "HHMI", "Howard Hughes Medical Institute").await;
    let mut award = create_test_award(&app, "AWD-BUD-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;

    // Activate award first
    let award_id = award["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    award = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();

    // Create budget lines
    let categories = vec![
        ("personnel", "200000"),
        ("fringe", "50000"),
        ("travel", "15000"),
        ("equipment", "30000"),
        ("supplies", "25000"),
        ("indirect", "100000"),
    ];

    for (cat, amount) in &categories {
        let payload = json!({
            "budget_category": cat,
            "description": format!("{} costs", cat),
            "budget_amount": amount,
        });
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/grants/awards/{}/budget-lines", award_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED, "Failed to create budget line for {}", cat);
    }

    // List budget lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/grants/awards/{}/budget-lines", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(list["data"].as_array().unwrap().len(), 6);
}

// ============================================================================
// Expenditure Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_expenditure_lifecycle() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "DOE", "Dept of Energy").await;
    let mut award = create_test_award(&app, "AWD-EXP-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    award = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();

    // Create expenditure
    let payload = json!({
        "expenditure_type": "actual",
        "expenditure_date": "2025-03-15",
        "description": "Lab supplies purchase",
        "budget_category": "supplies",
        "amount": "5000",
        "vendor_name": "Fisher Scientific",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/expenditures", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let exp: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(exp["status"], "pending");
    assert_eq!(exp["amount"], "5000");
    // Verify indirect cost calculation: 5000 * 50% = 2500, total = 7500
    let indirect: f64 = exp["indirect_cost_amount"].as_str().unwrap().parse().unwrap();
    let total: f64 = exp["total_amount"].as_str().unwrap().parse().unwrap();
    assert!((indirect - 2500.0).abs() < 1.0, "Expected indirect cost ~2500, got {}", indirect);
    assert!((total - 7500.0).abs() < 1.0, "Expected total ~7500, got {}", total);

    let exp_id = exp["id"].as_str().unwrap();

    // Approve expenditure
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/expenditures/{}/approve", exp_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(approved["status"], "approved");

    // List expenditures
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/grants/awards/{}/expenditures", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
#[ignore]
async fn test_expenditure_reversal() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "NASA", "National Aeronautics and Space Administration").await;
    let mut award = create_test_award(&app, "AWD-REV-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create expenditure
    let payload = json!({
        "expenditure_type": "actual",
        "expenditure_date": "2025-04-01",
        "description": "Equipment rental",
        "budget_category": "equipment",
        "amount": "10000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/expenditures", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let exp: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    let exp_id = exp["id"].as_str().unwrap();

    // Approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/expenditures/{}/approve", exp_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/expenditures/{}/reverse", exp_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let reversed: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(reversed["status"], "reversed");
}

// ============================================================================
// Billing Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_billing_lifecycle() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "EPA", "Environmental Protection Agency").await;
    let mut award = create_test_award(&app, "AWD-BIL-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create and approve expenditure
    let payload = json!({
        "expenditure_type": "actual",
        "expenditure_date": "2025-02-15",
        "description": "Field research supplies",
        "budget_category": "supplies",
        "amount": "8000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/expenditures", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let exp: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    let exp_id = exp["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/expenditures/{}/approve", exp_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create billing
    let payload = json!({
        "period_start": "2025-02-01",
        "period_end": "2025-02-28",
        "notes": "February billing"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/billings", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let billing: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(billing["status"], "draft");
    let billing_id = billing["id"].as_str().unwrap();

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/billings/{}/submit", billing_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let submitted: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/billings/{}/approve", billing_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(approved["status"], "approved");

    // Pay
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/billings/{}/pay", billing_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let paid: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(paid["status"], "paid");
}

// ============================================================================
// Compliance Report Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_compliance_report_lifecycle() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "CDC", "Centers for Disease Control").await;
    let mut award = create_test_award(&app, "AWD-RPT-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create compliance report
    let payload = json!({
        "report_type": "federal_financial_report_sf425",
        "report_title": "Q1 2025 SF-425",
        "reporting_period_start": "2025-01-01",
        "reporting_period_end": "2025-03-31",
        "due_date": "2025-04-30",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/reports", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let report: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(report["status"], "draft");
    assert_eq!(report["report_type"], "federal_financial_report_sf425");
    let report_id = report["id"].as_str().unwrap();

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/reports/{}/submit", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/reports/{}/approve", report_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert_eq!(approved["status"], "approved");
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_grant_dashboard() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create sponsor and active award
    let sponsor = create_test_sponsor(&app, "NIH-DASH", "NIH for Dashboard Test").await;
    let award = create_test_award(&app, "AWD-DASH-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/activate", award_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dashboard: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert!(dashboard["total_active_awards"].as_i64().unwrap() >= 1);
    assert!(dashboard["total_sponsors"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_expenditure_on_inactive_award_fails() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "VA", "Veterans Affairs").await;
    let award = create_test_award(&app, "AWD-VAL-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    let award_id = award["id"].as_str().unwrap();
    // Award is in 'draft' status - expenditures should fail

    let payload = json!({
        "expenditure_type": "actual",
        "expenditure_date": "2025-03-15",
        "amount": "1000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/grants/awards/{}/expenditures", award_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_list_awards_filtering() {
    let (_state, app) = setup_grant_test().await;
    let (k, v) = auth_header(&admin_claims());

    let sponsor = create_test_sponsor(&app, "NEH", "National Endowment for Humanities").await;

    // Create two awards
    create_test_award(&app, "AWD-FLT-001", Uuid::parse_str(sponsor["id"].as_str().unwrap()).unwrap()).await;
    create_test_award(&app, "AWD-FLT-002", Uuid::parse_str(sponsor["id"].as_str().unwrap()).await).await;

    // Filter by status=draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/awards?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap().to_vec().into();
    assert!(list["data"].as_array().unwrap().len() >= 2);

    // Filter by non-existent status
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/grants/awards?status=nonexistent")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
