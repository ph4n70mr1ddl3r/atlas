//! Loyalty Management E2E Tests
//!
//! Tests for Oracle Fusion CX > Loyalty Management:
//! - Loyalty program CRUD and lifecycle (draft -> active -> suspended -> closed)
//! - Loyalty tier management with bonus percentages
//! - Member enrollment and lifecycle (active -> suspended -> reactivated)
//! - Point accrual with tier bonus calculation
//! - Point adjustment (positive and negative)
//! - Transaction reversal
//! - Reward catalog CRUD
//! - Reward redemption lifecycle (pending -> fulfilled / cancelled)
//! - Dashboard analytics
//! - Validation edge cases and error handling
//! - Full end-to-end lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_loyalty_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_program(
    app: &axum::Router,
    number: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/loyalty/programs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "programNumber": number,
                        "name": name,
                        "programType": "points",
                        "startDate": "2024-01-01",
                        "endDate": "2025-12-31",
                        "accrualRate": 1.0,
                        "pointsExpiryDays": 365,
                        "autoUpgrade": true,
                        "allowRedemption": true
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&b);
        panic!(
            "Expected CREATED for program but got {}: {}",
            status,
            body_str
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_tier(
    app: &axum::Router,
    program_id: &str,
    tier_code: &str,
    tier_name: &str,
    min_points: f64,
    max_points: Option<f64>,
    bonus_pct: f64,
    is_default: bool,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut body = json!({
        "tierCode": tier_code,
        "tierName": tier_name,
        "tierLevel": 0,
        "minimumPoints": min_points,
        "accrualBonusPercentage": bonus_pct,
        "isDefault": is_default
    });
    if let Some(max) = max_points {
        body["maximumPoints"] = json!(max);
    }
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loyalty/programs/{}/tiers", program_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for tier but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn enroll_test_member(
    app: &axum::Router,
    program_id: &str,
    member_number: &str,
    customer_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loyalty/programs/{}/members", program_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "memberNumber": member_number,
                        "customerName": customer_name,
                        "customerEmail": format!("{}@test.com", member_number.to_lowercase())
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for member but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn setup_active_program_with_tiers(app: &axum::Router) -> serde_json::Value {
    let program = create_test_program(app, "LP-ACT", "Active Program").await;
    let program_id = program["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/activate", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    // Add tiers
    create_test_tier(app, program_id, "BRONZE", "Bronze", 0.0, Some(1000.0), 0.0, true).await;
    create_test_tier(app, program_id, "SILVER", "Silver", 1000.0, Some(5000.0), 10.0, false).await;
    create_test_tier(app, program_id, "GOLD", "Gold", 5000.0, None, 25.0, false).await;

    program
}

// ============================================================================
// Program Tests
// ============================================================================

#[tokio::test]
async fn test_create_program() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "LP-001", "Test Loyalty Program").await;
    assert_eq!(program["programNumber"], "LP-001");
    assert_eq!(program["name"], "Test Loyalty Program");
    assert_eq!(program["programType"], "points");
    assert_eq!(program["status"], "draft");
    assert_eq!(program["accrualRate"], 1.0);
}

#[tokio::test]
async fn test_create_program_duplicate_conflict() {
    let (_state, app) = setup_loyalty_test().await;
    create_test_program(&app, "DUP-LP", "First").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/loyalty/programs")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "programNumber": "DUP-LP",
                    "name": "Duplicate",
                    "programType": "points",
                    "startDate": "2024-01-01"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_program_invalid_type() {
    let (_state, app) = setup_loyalty_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/loyalty/programs")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "programNumber": "BAD-TYPE",
                    "name": "Bad Type",
                    "programType": "invalid",
                    "startDate": "2024-01-01"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_program() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "GET-LP", "Get Me").await;
    let id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["programNumber"], "GET-LP");
}

#[tokio::test]
async fn test_list_programs() {
    let (_state, app) = setup_loyalty_test().await;
    create_test_program(&app, "LIST-1", "Program One").await;
    create_test_program(&app, "LIST-2", "Program Two").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/loyalty/programs")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_program() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "ACT-LP", "Activate Me").await;
    let id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/activate", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "active");
}

#[tokio::test]
async fn test_suspend_and_close_program() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "SUS-LP", "Suspend Me").await;
    let id = program["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/activate", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    // Suspend
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/suspend", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "suspended");

    // Close
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/close", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_program() {
    let (_state, app) = setup_loyalty_test().await;
    create_test_program(&app, "DEL-LP", "Delete Me").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("DELETE")
            .uri("/api/v1/loyalty/programs/number/DEL-LP")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Tier Tests
// ============================================================================

#[tokio::test]
async fn test_create_tier() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "TIER-LP", "Tier Program").await;
    let program_id = program["id"].as_str().unwrap();

    let tier = create_test_tier(&app, program_id, "SILVER", "Silver Tier", 1000.0, Some(5000.0), 10.0, false).await;
    assert_eq!(tier["tierCode"], "SILVER");
    assert_eq!(tier["tierName"], "Silver Tier");
    assert_eq!(tier["accrualBonusPercentage"], 10.0);
}

#[tokio::test]
async fn test_list_tiers() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "TIER-LIST", "List Tiers").await;
    let program_id = program["id"].as_str().unwrap();

    create_test_tier(&app, program_id, "B", "Bronze", 0.0, Some(1000.0), 0.0, true).await;
    create_test_tier(&app, program_id, "S", "Silver", 1000.0, Some(5000.0), 10.0, false).await;
    create_test_tier(&app, program_id, "G", "Gold", 5000.0, None, 25.0, false).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/tiers", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_delete_tier() {
    let (_state, app) = setup_loyalty_test().await;
    let program = create_test_program(&app, "TIER-DEL", "Del Tier").await;
    let program_id = program["id"].as_str().unwrap();
    let tier = create_test_tier(&app, program_id, "DEL", "Delete Me", 0.0, None, 0.0, false).await;
    let tier_id = tier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/loyalty/tiers/{}", tier_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Member Tests
// ============================================================================

#[tokio::test]
async fn test_enroll_member() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "MEM-001", "John Doe").await;
    assert_eq!(member["memberNumber"], "MEM-001");
    assert_eq!(member["customerName"], "John Doe");
    assert_eq!(member["status"], "active");
    // Should have been assigned the default tier (BRONZE)
    assert_eq!(member["tierCode"], "BRONZE");
    assert!(member["currentPoints"].as_f64().unwrap().abs() < 0.01);
}

#[tokio::test]
async fn test_enroll_member_duplicate_conflict() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    enroll_test_member(&app, program_id, "DUP-MEM", "First").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/members", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberNumber": "DUP-MEM",
                    "customerName": "Duplicate"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_suspend_and_reactivate_member() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "SUS-MEM", "Suspend Me").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Suspend
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/members/{}/suspend", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "suspended");

    // Reactivate
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/members/{}/reactivate", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "active");
}

#[tokio::test]
async fn test_delete_member() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    enroll_test_member(&app, program_id, "DEL-MEM", "Delete Me").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("DELETE")
            .uri("/api/v1/loyalty/members/number/DEL-MEM")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_members() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    enroll_test_member(&app, program_id, "ML-1", "Member One").await;
    enroll_test_member(&app, program_id, "ML-2", "Member Two").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/members", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Point Transaction Tests
// ============================================================================

#[tokio::test]
async fn test_accrue_points() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "ACC-MEM", "Accrue Points").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "TXN-001",
                    "sourceType": "sales_order",
                    "sourceNumber": "SO-12345",
                    "referenceAmount": 500.0,
                    "referenceCurrency": "USD",
                    "description": "Purchase reward"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(txn["transactionType"], "accrual");
    assert_eq!(txn["status"], "posted");
    // 500 * 1.0 = 500 points, bronze has 0% bonus
    assert!((txn["points"].as_f64().unwrap() - 500.0).abs() < 0.01);
}

#[tokio::test]
async fn test_accrue_points_with_tier_bonus() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "BONUS-MEM", "Bonus Member").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // First accrue 1000 points to upgrade to Silver (1000 threshold)
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "UPG-TXN",
                    "referenceAmount": 1000.0,
                    "description": "First purchase"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Check member was upgraded to Silver
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/members/{}", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated_member: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_member["tierCode"], "SILVER");

    // Now accrue more points - should get 10% bonus
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "BONUS-TXN",
                    "referenceAmount": 500.0,
                    "description": "Second purchase"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // 500 * 1.0 = 500 base + 10% bonus = 50 = 550 total
    assert!((txn["points"].as_f64().unwrap() - 550.0).abs() < 0.01);
    assert!((txn["tierBonusApplied"].as_f64().unwrap() - 50.0).abs() < 0.01);
}

#[tokio::test]
async fn test_adjust_points() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "ADJ-MEM", "Adjust Points").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/adjust", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "ADJ-001",
                    "points": 250.0,
                    "description": "Welcome bonus"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(txn["transactionType"], "adjustment");
    assert!((txn["points"].as_f64().unwrap() - 250.0).abs() < 0.01);
}

#[tokio::test]
async fn test_reverse_transaction() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "REV-MEM", "Reverse Me").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Accrue first
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "REV-TXN",
                    "referenceAmount": 100.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let txn_id = txn["id"].as_str().unwrap();

    // Reverse
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/transactions/{}/reverse", txn_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"reason": "Order cancelled"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let reversed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reversed["status"], "reversed");
}

#[tokio::test]
async fn test_list_transactions() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let member = enroll_test_member(&app, program_id, "LST-MEM", "List Txns").await;
    let member_id = member["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    for num in ["LTXN-1", "LTXN-2"] {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "memberId": member_id,
                        "transactionNumber": num,
                        "referenceAmount": 100.0
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/members/{}/transactions", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Reward Tests
// ============================================================================

#[tokio::test]
async fn test_create_reward() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "rewardCode": "RWD-001",
                    "name": "Free Coffee",
                    "rewardType": "voucher",
                    "pointsRequired": 500.0,
                    "cashValue": 5.0,
                    "quantityAvailable": 100
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let reward: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reward["rewardCode"], "RWD-001");
    assert_eq!(reward["name"], "Free Coffee");
    assert!((reward["pointsRequired"].as_f64().unwrap() - 500.0).abs() < 0.01);
    assert!(reward["isActive"].as_bool().unwrap());
}

#[tokio::test]
async fn test_list_rewards() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    for (code, name, pts) in [("R1", "Reward 1", 100.0), ("R2", "Reward 2", 500.0)] {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "rewardCode": code, "name": name,
                        "rewardType": "merchandise", "pointsRequired": pts
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_reward() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "rewardCode": "DEL-RWD", "name": "Delete Me",
                    "rewardType": "merchandise", "pointsRequired": 100.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let resp = app.clone().oneshot(
        Request::builder()
            .method("DELETE")
            .uri("/api/v1/loyalty/rewards/code/DEL-RWD")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Redemption Tests
// ============================================================================

#[tokio::test]
async fn test_redeem_reward() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create reward
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "rewardCode": "COFFEE", "name": "Free Coffee",
                    "rewardType": "voucher", "pointsRequired": 500.0,
                    "quantityAvailable": 100
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    // Get reward ID
    let reward_resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let reward_body = axum::body::to_bytes(reward_resp.into_body(), usize::MAX).await.unwrap();
    let rewards: serde_json::Value = serde_json::from_slice(&reward_body).unwrap();
    let reward_id = rewards["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Enroll member and accrue points
    let member = enroll_test_member(&app, program_id, "RED-MEM", "Redeem Points").await;
    let member_id = member["id"].as_str().unwrap();

    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "RED-ACQ",
                    "referenceAmount": 1000.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Redeem
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/redeem", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "rewardId": reward_id,
                    "redemptionNumber": "RED-001",
                    "quantity": 1
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let redemption: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(redemption["status"], "pending");
    assert!((redemption["pointsSpent"].as_f64().unwrap() - 500.0).abs() < 0.01);
}

#[tokio::test]
async fn test_fulfill_and_cancel_redemption() {
    let (_state, app) = setup_loyalty_test().await;
    let program = setup_active_program_with_tiers(&app).await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create reward
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "rewardCode": "FULFILL", "name": "Test Reward",
                    "rewardType": "merchandise", "pointsRequired": 200.0,
                    "quantityAvailable": 50
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let reward_resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let reward_body = axum::body::to_bytes(reward_resp.into_body(), usize::MAX).await.unwrap();
    let rewards: serde_json::Value = serde_json::from_slice(&reward_body).unwrap();
    let reward_id = rewards["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Test fulfillment
    let member1 = enroll_test_member(&app, program_id, "FUL-MEM", "Fulfill Me").await;
    let member1_id = member1["id"].as_str().unwrap();

    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member1_id,
                    "transactionNumber": "FUL-ACQ",
                    "referenceAmount": 500.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/redeem", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member1_id,
                    "rewardId": reward_id,
                    "redemptionNumber": "FUL-R01",
                    "quantity": 1
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let red: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let red_id = red["id"].as_str().unwrap();

    // Fulfill
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/redemptions/{}/fulfill", red_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "fulfilled");

    // Test cancellation with different member
    let member2 = enroll_test_member(&app, program_id, "CNC-MEM", "Cancel Me").await;
    let member2_id = member2["id"].as_str().unwrap();

    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member2_id,
                    "transactionNumber": "CNC-ACQ",
                    "referenceAmount": 500.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/redeem", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member2_id,
                    "rewardId": reward_id,
                    "redemptionNumber": "CNC-R01",
                    "quantity": 1
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let red: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let red_id = red["id"].as_str().unwrap();

    // Cancel (should refund points)
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/redemptions/{}/cancel", red_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"reason": "Customer request"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "cancelled");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_loyalty_dashboard() {
    let (_state, app) = setup_loyalty_test().await;
    create_test_program(&app, "DASH-LP1", "Dashboard Program 1").await;
    create_test_program(&app, "DASH-LP2", "Dashboard Program 2").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/loyalty/dashboard")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalPrograms"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Full End-to-End Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_loyalty_full_lifecycle() {
    let (_state, app) = setup_loyalty_test().await;

    let (k, v) = auth_header(&admin_claims());

    // 1. Create loyalty program
    let program = create_test_program(&app, "LIFE-LP", "Full Lifecycle Program").await;
    let program_id = program["id"].as_str().unwrap();
    assert_eq!(program["status"], "draft");

    // 2. Add tiers
    create_test_tier(&app, program_id, "BASIC", "Basic", 0.0, Some(500.0), 0.0, true).await;
    create_test_tier(&app, program_id, "PLUS", "Plus", 500.0, Some(2000.0), 15.0, false).await;
    create_test_tier(&app, program_id, "PREMIUM", "Premium", 2000.0, None, 30.0, false).await;

    // 3. Activate program
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/activate", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Create reward catalog
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "rewardCode": "TSHIRT", "name": " branded T-Shirt",
                    "rewardType": "merchandise", "pointsRequired": 300.0,
                    "quantityAvailable": 50, "maxPerMember": 2
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Get reward ID
    let reward_resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/programs/{}/rewards", program_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let reward_body = axum::body::to_bytes(reward_resp.into_body(), usize::MAX).await.unwrap();
    let rewards: serde_json::Value = serde_json::from_slice(&reward_body).unwrap();
    let reward_id = rewards["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // 5. Enroll member
    let member = enroll_test_member(&app, program_id, "LIFE-MEM", "Lifecycle User").await;
    let member_id = member["id"].as_str().unwrap();
    assert_eq!(member["status"], "active");
    assert_eq!(member["tierCode"], "BASIC");

    // 6. Accrue points through purchases
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "LIFE-TXN1",
                    "sourceType": "sales_order",
                    "sourceNumber": "SO-1001",
                    "referenceAmount": 750.0,
                    "description": "First purchase"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 7. Verify tier upgrade (750 lifetime -> should be PLUS tier >= 500)
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/members/{}", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["tierCode"], "PLUS");
    assert!((updated["currentPoints"].as_f64().unwrap() - 750.0).abs() < 0.01);
    assert!((updated["lifetimePoints"].as_f64().unwrap() - 750.0).abs() < 0.01);

    // 8. Accrue more with Plus bonus (15%)
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "LIFE-TXN2",
                    "referenceAmount": 1000.0,
                    "description": "Second purchase"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn2: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // 1000 base + 15% bonus = 1150 total
    assert!((txn2["points"].as_f64().unwrap() - 1150.0).abs() < 0.01);
    assert!((txn2["tierBonusApplied"].as_f64().unwrap() - 150.0).abs() < 0.01);

    // 9. Verify PREMIUM upgrade (750 + 1150 = 1900 lifetime... not quite 2000)
    // Let's do another accrual to push over 2000
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/accrue", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "transactionNumber": "LIFE-TXN3",
                    "referenceAmount": 500.0,
                    "description": "Third purchase"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Verify PREMIUM upgrade
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/loyalty/members/{}", member_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Total lifetime should be > 2000 now
    assert!(updated["lifetimePoints"].as_f64().unwrap() >= 2000.0);

    // 10. Redeem reward
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/programs/{}/redeem", program_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "memberId": member_id,
                    "rewardId": reward_id,
                    "redemptionNumber": "LIFE-RED1",
                    "quantity": 1
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let redemption: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let redemption_id = redemption["id"].as_str().unwrap();
    assert_eq!(redemption["status"], "pending");

    // 11. Fulfill redemption
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/loyalty/redemptions/{}/fulfill", redemption_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["status"], "fulfilled");

    // 12. Verify dashboard
    let resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/loyalty/dashboard")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalPrograms"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalMembers"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalRedemptions"].as_i64().unwrap() >= 1);
}
