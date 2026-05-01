//! Funds Reservation & Budgetary Control E2E Tests
//!
//! Tests for Oracle Fusion Budgetary Control / Funds Reservation:
//! - Fund reservation CRUD (create, get, list, delete)
//! - Fund consumption (partial and full)
//! - Fund release (partial release back to budget)
//! - Reservation cancellation
//! - Fund availability checking
//! - Reservation lines management
//! - Budgetary control dashboard
//! - Validation edge cases and error handling

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_funds_reservation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_reservation(
    app: &axum::Router,
    number: &str,
    budget_code: &str,
    amount: f64,
    control_level: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let budget_id = "00000000-0000-0000-0000-000000000099";
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/funds-reservation/reservations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reservationNumber": number,
                        "budgetId": budget_id,
                        "budgetCode": budget_code,
                        "reservedAmount": amount,
                        "currencyCode": "USD",
                        "reservationDate": "2024-03-01",
                        "expiryDate": "2024-12-31",
                        "controlLevel": control_level,
                        "fiscalYear": 2024,
                        "periodName": "Q1-2024",
                        "description": format!("Test reservation {}", number),
                        "departmentName": "IT Department"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for reservation but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Fund Reservation CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_reservation() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-001", "BUD-OPS-2024", 50000.0, "advisory").await;
    assert_eq!(reservation["reservation_number"], "FR-001");
    assert_eq!(reservation["budget_code"], "BUD-OPS-2024");
    assert_eq!(reservation["control_level"], "advisory");
    assert!((reservation["reserved_amount"].as_f64().unwrap() - 50000.0).abs() < 0.01);
    assert_eq!(reservation["status"], "active");
    assert_eq!(reservation["currency_code"], "USD");
}

#[tokio::test]
async fn test_create_reservation_with_source_reference() {
    let (_state, app) = setup_funds_reservation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/funds-reservation/reservations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reservationNumber": "FR-SRC-001",
                        "budgetId": "00000000-0000-0000-0000-000000000099",
                        "budgetCode": "BUD-OPS-2024",
                        "reservedAmount": 25000.0,
                        "currencyCode": "USD",
                        "reservationDate": "2024-04-01",
                        "controlLevel": "advisory",
                        "sourceType": "purchase_requisition",
                        "sourceNumber": "PR-00123",
                        "description": "IT hardware procurement"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let reservation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reservation["source_type"], "purchase_requisition");
    assert_eq!(reservation["source_number"], "PR-00123");
}

#[tokio::test]
async fn test_create_reservation_duplicate_conflict() {
    let (_state, app) = setup_funds_reservation_test().await;
    create_test_reservation(&app, "FR-DUP", "BUD-001", 5000.0, "advisory").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/funds-reservation/reservations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reservationNumber": "FR-DUP",
                        "budgetId": "00000000-0000-0000-0000-000000000099",
                        "budgetCode": "BUD-001",
                        "reservedAmount": 3000.0,
                        "currencyCode": "USD",
                        "reservationDate": "2024-03-01",
                        "controlLevel": "advisory"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_reservation_invalid_control_level() {
    let (_state, app) = setup_funds_reservation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/funds-reservation/reservations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reservationNumber": "FR-BAD",
                        "budgetId": "00000000-0000-0000-0000-000000000099",
                        "budgetCode": "BUD-001",
                        "reservedAmount": 5000.0,
                        "currencyCode": "USD",
                        "reservationDate": "2024-03-01",
                        "controlLevel": "permissive"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_reservation_negative_amount() {
    let (_state, app) = setup_funds_reservation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/funds-reservation/reservations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reservationNumber": "FR-NEG",
                        "budgetId": "00000000-0000-0000-0000-000000000099",
                        "budgetCode": "BUD-001",
                        "reservedAmount": -5000.0,
                        "currencyCode": "USD",
                        "reservationDate": "2024-03-01",
                        "controlLevel": "advisory"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_reservation() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-GET", "BUD-001", 10000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["reservation_number"], "FR-GET");
    assert_eq!(fetched["budget_code"], "BUD-001");
}

#[tokio::test]
async fn test_get_reservation_by_number() {
    let (_state, app) = setup_funds_reservation_test().await;
    create_test_reservation(&app, "FR-BYNUM", "BUD-001", 15000.0, "absolute").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/funds-reservation/reservations/number/FR-BYNUM")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["reservation_number"], "FR-BYNUM");
    assert_eq!(fetched["control_level"], "absolute");
}

#[tokio::test]
async fn test_list_reservations() {
    let (_state, app) = setup_funds_reservation_test().await;
    create_test_reservation(&app, "FR-LIST-1", "BUD-001", 5000.0, "advisory").await;
    create_test_reservation(&app, "FR-LIST-2", "BUD-002", 8000.0, "absolute").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/funds-reservation/reservations")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_reservation() {
    let (_state, app) = setup_funds_reservation_test().await;
    create_test_reservation(&app, "FR-DEL", "BUD-001", 3000.0, "advisory").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/funds-reservation/reservations/number/FR-DEL")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Fund Consumption Tests
// ============================================================================

#[tokio::test]
async fn test_consume_reservation_partial() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-CONSUME", "BUD-001", 50000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"consumeAmount": 20000.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((updated["consumed_amount"].as_f64().unwrap() - 20000.0).abs() < 0.01);
    assert!((updated["remaining_amount"].as_f64().unwrap() - 30000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_consume_reservation_full() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-FULL", "BUD-001", 25000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"consumeAmount": 25000.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "fully_consumed");
    assert!((updated["remaining_amount"].as_f64().unwrap()).abs() < 0.01);
}

#[tokio::test]
async fn test_consume_exceeds_remaining() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-OVER", "BUD-001", 10000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"consumeAmount": 15000.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Fund Release Tests
// ============================================================================

#[tokio::test]
async fn test_release_reservation_partial() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-RELEASE", "BUD-001", 40000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/release", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"releaseAmount": 15000.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((updated["released_amount"].as_f64().unwrap() - 15000.0).abs() < 0.01);
    assert!((updated["remaining_amount"].as_f64().unwrap() - 25000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_release_reservation_full() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-REL-FULL", "BUD-001", 20000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/release", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"releaseAmount": 20000.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "released");
}

// ============================================================================
// Cancellation Tests
// ============================================================================

#[tokio::test]
async fn test_cancel_reservation() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-CANCEL", "BUD-001", 30000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/cancel", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"reason": "Project cancelled by management"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "cancelled");
    assert_eq!(updated["cancellation_reason"], "Project cancelled by management");
    assert!((updated["released_amount"].as_f64().unwrap() - 30000.0).abs() < 0.01);
    assert!((updated["remaining_amount"].as_f64().unwrap()).abs() < 0.01);
}

// ============================================================================
// Reservation Lines Tests
// ============================================================================

#[tokio::test]
async fn test_create_reservation_line() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-LINE", "BUD-001", 50000.0, "advisory").await;
    let reservation_id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/lines", reservation_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "lineNumber": 1,
                        "accountCode": "1000-100-1000",
                        "accountDescription": "IT Hardware Budget",
                        "reservedAmount": 30000.0,
                        "costCenter": "CC-IT"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(line["account_code"], "1000-100-1000");
    assert!((line["reserved_amount"].as_f64().unwrap() - 30000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_list_reservation_lines() {
    let (_state, app) = setup_funds_reservation_test().await;
    let reservation = create_test_reservation(&app, "FR-LINES", "BUD-001", 60000.0, "advisory").await;
    let reservation_id = reservation["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create two lines
    for (i, (account, amount)) in [("1000-100", 35000.0), ("1000-200", 25000.0)].iter().enumerate() {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/funds-reservation/reservations/id/{}/lines", reservation_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "lineNumber": i + 1,
                            "accountCode": account,
                            "reservedAmount": amount
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/funds-reservation/reservations/id/{}/lines", reservation_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Fund Availability Check Tests
// ============================================================================

#[tokio::test]
async fn test_check_fund_availability() {
    let (_state, app) = setup_funds_reservation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/funds-reservation/fund-availability?budgetId=00000000-0000-0000-0000-000000000099&accountCode=1000-100&asOfDate=2024-03-15&fiscalYear=2024")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let availability: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Should contain these fields regardless of budget data
    assert!(availability["account_code"].is_string());
    assert!(availability["budget_amount"].is_number());
    assert!(availability["available_balance"].is_number());
    assert!(availability["check_passed"].is_boolean());
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_budgetary_control_dashboard() {
    let (_state, app) = setup_funds_reservation_test().await;

    // Create some reservations to populate dashboard
    create_test_reservation(&app, "FR-DASH-1", "BUD-001", 25000.0, "advisory").await;
    create_test_reservation(&app, "FR-DASH-2", "BUD-002", 40000.0, "absolute").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/funds-reservation/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["total_reservations"].as_i64().unwrap() >= 2);
    assert!(dashboard["active_reservations"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_reservation_full_lifecycle() {
    let (_state, app) = setup_funds_reservation_test().await;

    // 1. Create a reservation
    let reservation = create_test_reservation(&app, "FR-LIFE", "BUD-OPS-2024", 100000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();
    assert_eq!(reservation["status"], "active");
    assert!((reservation["reserved_amount"].as_f64().unwrap() - 100000.0).abs() < 0.01);

    // 2. Add reservation lines
    let (k, v) = auth_header(&admin_claims());
    let line_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/lines", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "lineNumber": 1,
                    "accountCode": "6000-100",
                    "accountDescription": "Software Licenses",
                    "reservedAmount": 60000.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(line_resp.status(), StatusCode::CREATED);

    let line_resp2 = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/lines", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "lineNumber": 2,
                    "accountCode": "6000-200",
                    "accountDescription": "Hardware",
                    "reservedAmount": 40000.0
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(line_resp2.status(), StatusCode::CREATED);

    // 3. Partially consume (first invoice received)
    let consume_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"consumeAmount": 45000.0})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(consume_resp.status(), StatusCode::OK);
    let consumed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(consume_resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!((consumed["consumed_amount"].as_f64().unwrap() - 45000.0).abs() < 0.01);
    assert!((consumed["remaining_amount"].as_f64().unwrap() - 55000.0).abs() < 0.01);

    // 4. Release unneeded funds
    let release_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/release", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"releaseAmount": 20000.0})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(release_resp.status(), StatusCode::OK);
    let released: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(release_resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!((released["released_amount"].as_f64().unwrap() - 20000.0).abs() < 0.01);
    assert!((released["remaining_amount"].as_f64().unwrap() - 35000.0).abs() < 0.01);

    // 5. Consume remaining
    let final_consume = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"consumeAmount": 35000.0})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(final_consume.status(), StatusCode::OK);
    let final_state: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(final_consume.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(final_state["status"], "fully_consumed");
    assert!((final_state["consumed_amount"].as_f64().unwrap() - 80000.0).abs() < 0.01);
    assert!((final_state["released_amount"].as_f64().unwrap() - 20000.0).abs() < 0.01);
}

// ============================================================================
// Lifecycle Test with Cancellation
// ============================================================================

#[tokio::test]
async fn test_reservation_create_then_cancel() {
    let (_state, app) = setup_funds_reservation_test().await;

    // 1. Create a reservation
    let reservation = create_test_reservation(&app, "FR-CAN-LIFE", "BUD-001", 75000.0, "advisory").await;
    let id = reservation["id"].as_str().unwrap();

    // 2. Partially consume
    let (k, v) = auth_header(&admin_claims());
    let consume_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/consume", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"consumeAmount": 25000.0})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(consume_resp.status(), StatusCode::OK);

    // 3. Cancel remaining
    let cancel_resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/funds-reservation/reservations/id/{}/cancel", id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"reason": "Budget reallocation"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(cancel_resp.status(), StatusCode::OK);
    let cancelled: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(cancel_resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert!((cancelled["consumed_amount"].as_f64().unwrap() - 25000.0).abs() < 0.01);
    // Released amount should be the un-consumed portion
    assert!((cancelled["released_amount"].as_f64().unwrap() - 50000.0).abs() < 0.01);
}
