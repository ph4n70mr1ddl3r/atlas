//! Transaction Calendar E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Transaction Calendars:
//! - Calendar CRUD and lifecycle (create, list, get, activate, deactivate, delete)
//! - Exception management (holidays, non-working days, special working days)
//! - Business day calculations (is business day, next/previous business day, add business days)
//! - Audit trail of calculations
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/145_transaction_calendar.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_calendar(
    app: &axum::Router,
    code: &str,
    name: &str,
    working_days: Option<&serde_json::Value>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "code": code,
        "name": name,
    });
    if let Some(wd) = working_days {
        payload["workingDays"] = wd.clone();
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/transaction-calendars")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE CALENDAR RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create calendar: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_exception(
    app: &axum::Router,
    calendar_id: &str,
    exception_date: &str,
    exception_type: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "exceptionDate": exception_date,
        "exceptionType": exception_type,
        "name": name,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", calendar_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE EXCEPTION RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create exception");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Calendar CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_calendar() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "STD-CAL", "Standard Calendar", None).await;

    assert_eq!(cal["code"], "STD-CAL");
    assert_eq!(cal["name"], "Standard Calendar");
    assert_eq!(cal["status"], "active");
    // Default working days: Mon-Fri
    let wd = cal["workingDays"].as_array().unwrap();
    assert_eq!(wd.len(), 5);
    assert!(wd.contains(&serde_json::Value::from(1)));
    assert!(wd.contains(&serde_json::Value::from(5)));
}

#[tokio::test]
async fn test_create_calendar_custom_working_days() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "SUN-THU", "Sun-Thu Calendar", Some(&json!([7, 1, 2, 3, 4]))).await;

    let wd = cal["workingDays"].as_array().unwrap();
    assert_eq!(wd.len(), 5);
    assert!(wd.contains(&serde_json::Value::from(7))); // Sunday
    assert!(wd.contains(&serde_json::Value::from(4))); // Thursday
}

#[tokio::test]
async fn test_get_calendar() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "GET-CAL", "Get Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/transaction-calendars/{}", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "GET-CAL");
}

#[tokio::test]
async fn test_get_calendar_by_code() {
    let (_state, app) = setup_test().await;
    create_calendar(&app, "CODE-CAL", "Code Calendar", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars/code/CODE-CAL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "CODE-CAL");
}

#[tokio::test]
async fn test_list_calendars() {
    let (_state, app) = setup_test().await;
    create_calendar(&app, "LIST-CAL1", "List Calendar 1", None).await;
    create_calendar(&app, "LIST-CAL2", "List Calendar 2", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_deactivate_calendar() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "LC-CAL", "Lifecycle Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/deactivate", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/activate", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_delete_calendar() {
    let (_state, app) = setup_test().await;
    create_calendar(&app, "DEL-CAL", "Delete Calendar", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transaction-calendars/code/DEL-CAL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars/code/DEL-CAL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_calendar_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "",
        "name": "No Code Calendar",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/transaction-calendars")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_calendar_invalid_working_days_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD-WD",
        "name": "Bad Working Days",
        "workingDays": [0, 1, 2],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/transaction-calendars")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_calendar_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_calendar(&app, "DUP-CAL", "First", None).await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "DUP-CAL",
        "name": "Duplicate",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/transaction-calendars")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_delete_calendar_with_exceptions_fails() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "DEL-EXC-CAL", "Calendar With Exception", None).await;
    let cal_id = cal["id"].as_str().unwrap();
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transaction-calendars/code/DEL-EXC-CAL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Exception Tests
// ============================================================================

#[tokio::test]
async fn test_create_exception() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "EXC-CAL", "Exception Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let exc = create_exception(&app, cal_id, "2025-12-25", "holiday", "Christmas Day").await;

    assert_eq!(exc["exceptionType"], "holiday");
    assert_eq!(exc["name"], "Christmas Day");
    assert_eq!(exc["exceptionDate"], "2025-12-25");
}

#[tokio::test]
async fn test_list_exceptions() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "LIST-EXC-CAL", "List Exception Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year's Day").await;
    create_exception(&app, cal_id, "2025-12-25", "holiday", "Christmas Day").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let items = body["data"].as_array().unwrap();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn test_delete_exception() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "DEL-EXC2-CAL", "Delete Exception Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();
    let exc = create_exception(&app, cal_id, "2025-07-04", "holiday", "Independence Day").await;
    let exc_id = exc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/transaction-calendars/exceptions/{}", exc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify exception is gone
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_exception_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "INV-EXC-CAL", "Invalid Exception Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "exceptionDate": "2025-01-01",
        "exceptionType": "invalid_type",
        "name": "Bad Exception",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_duplicate_exception_fails() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "DUP-EXC-CAL", "Dup Exception Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "exceptionDate": "2025-01-01",
        "exceptionType": "non_working",
        "name": "Another New Year",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Business Day Calculation Tests
// ============================================================================

#[tokio::test]
async fn test_is_business_day_weekday() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "BIZ-CAL", "Business Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // 2025-01-06 is a Monday
    let payload = json!({ "date": "2025-01-06" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], true);
}

#[tokio::test]
async fn test_is_business_day_weekend() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "BIZ2-CAL", "Business Calendar 2", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // 2025-01-04 is a Saturday
    let payload = json!({ "date": "2025-01-04" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], false);
}

#[tokio::test]
async fn test_is_business_day_holiday() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "HOL-CAL", "Holiday Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    // 2025-01-01 is a Wednesday, but we'll make it a holiday
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year's Day").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "date": "2025-01-01" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], false, "Holiday Wednesday should not be a business day");
}

#[tokio::test]
async fn test_is_business_day_special_working() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "SW-CAL", "Special Working Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    // 2025-01-04 is a Saturday, but we'll make it a special working day
    create_exception(&app, cal_id, "2025-01-04", "special_working", "Year-End Closing").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "date": "2025-01-04" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], true, "Special working Saturday should be a business day");
}

#[tokio::test]
async fn test_next_business_day() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "NEXT-CAL", "Next Business Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Friday Jan 3, 2025 -> next business day should be Monday Jan 6
    let payload = json!({ "date": "2025-01-03" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/next-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["nextBusinessDay"], "2025-01-06");
}

#[tokio::test]
async fn test_next_business_day_with_holiday() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "NEXT-HOL-CAL", "Next Holiday Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    // Make Monday Jan 6, 2025 a holiday
    create_exception(&app, cal_id, "2025-01-06", "holiday", "Epiphany").await;

    let (k, v) = auth_header(&admin_claims());
    // Friday Jan 3, 2025 -> next business day should skip weekend and holiday Monday -> Tuesday Jan 7
    let payload = json!({ "date": "2025-01-03" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/next-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["nextBusinessDay"], "2025-01-07");
}

#[tokio::test]
async fn test_previous_business_day() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "PREV-CAL", "Previous Business Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Monday Jan 6, 2025 -> previous business day should be Friday Jan 3
    let payload = json!({ "date": "2025-01-06" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/previous-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["previousBusinessDay"], "2025-01-03");
}

#[tokio::test]
async fn test_add_business_days() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "ADD-CAL", "Add Business Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Monday Jan 6 + 5 business days = Monday Jan 13
    let payload = json!({
        "startDate": "2025-01-06",
        "days": 5,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/add-business-days", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["resultDate"], "2025-01-13");
}

#[tokio::test]
async fn test_add_business_days_with_holiday() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "ADD-HOL-CAL", "Add Holiday Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    // Wednesday Jan 1 is a holiday, Thursday Jan 2 is normal
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year").await;

    let (k, v) = auth_header(&admin_claims());
    // Dec 30 (Monday) + 3 business days:
    // Dec 30 (Mon) +1 -> Dec 31 (Tue) +2 -> Jan 1 (Wed, holiday, skip) -> Jan 2 (Thu) +3 -> Jan 3 (Fri)
    let payload = json!({
        "startDate": "2024-12-30",
        "days": 3,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/add-business-days", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["resultDate"], "2025-01-03");
}

#[tokio::test]
async fn test_add_business_days_negative_fails() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "NEG-CAL", "Negative Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "startDate": "2025-01-06",
        "days": -1,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/add-business-days", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Audit Trail Tests
// ============================================================================

#[tokio::test]
async fn test_audit_trail_from_calculations() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "AUDIT-CAL", "Audit Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();

    // Perform some calculations
    let (k, v) = auth_header(&admin_claims());

    let payload = json!({ "date": "2025-01-06" });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let payload = json!({ "date": "2025-01-03" });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/next-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List audit entries
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars/calculations")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let entries = body["data"].as_array().unwrap();
    assert!(entries.len() >= 2, "Expected at least 2 calculation entries");

    // Verify operations are present
    let operations: Vec<&str> = entries.iter()
        .map(|e| e["operation"].as_str().unwrap_or(""))
        .collect();
    assert!(operations.contains(&"is_business_day"));
    assert!(operations.contains(&"next_business_day"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_transaction_calendar_dashboard() {
    let (_state, app) = setup_test().await;
    let cal = create_calendar(&app, "DASH-CAL", "Dashboard Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year").await;

    // Perform a calculation
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "date": "2025-01-06" });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalCalendars").is_some());
    assert!(body.get("activeCalendars").is_some());
    assert!(body.get("totalExceptions").is_some());
    assert!(body.get("totalCalculations").is_some());
    assert!(body.get("recentCalculations").is_some());

    assert!(body["totalCalendars"].as_i64().unwrap() >= 1);
    assert!(body["totalExceptions"].as_i64().unwrap() >= 1);
    assert!(body["totalCalculations"].as_i64().unwrap() >= 1);
}

// ============================================================================
// End-to-End Full Flow Test
// ============================================================================

#[tokio::test]
async fn test_end_to_end_transaction_calendar_flow() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());

    // 1. Create a standard Mon-Fri calendar
    let cal = create_calendar(&app, "E2E-CAL", "E2E Standard Calendar", None).await;
    let cal_id = cal["id"].as_str().unwrap();
    assert_eq!(cal["code"], "E2E-CAL");
    assert_eq!(cal["status"], "active");

    // 2. Add holidays for a typical US calendar
    create_exception(&app, cal_id, "2025-01-01", "holiday", "New Year's Day").await;
    create_exception(&app, cal_id, "2025-01-20", "holiday", "MLK Day").await;
    create_exception(&app, cal_id, "2025-02-17", "holiday", "Presidents' Day").await;
    create_exception(&app, cal_id, "2025-05-26", "holiday", "Memorial Day").await;
    create_exception(&app, cal_id, "2025-07-04", "holiday", "Independence Day").await;
    create_exception(&app, cal_id, "2025-09-01", "holiday", "Labor Day").await;

    // 3. Add a special working Saturday (year-end closing)
    create_exception(&app, cal_id, "2025-12-27", "special_working", "Year-End Close").await;

    // 4. Verify the Saturday is a business day (special_working)
    let payload = json!({ "date": "2025-12-27" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], true, "Special working Saturday should be a business day");

    // 5. Verify July 4th is not a business day
    let payload = json!({ "date": "2025-07-04" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/is-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isBusinessDay"], false, "July 4th holiday should not be a business day");

    // 6. Calculate next business day after July 4th (Friday July 4 is a holiday -> skip to Mon July 7)
    // Actually July 4 2025 is a Friday. With the holiday, next business day after July 3 (Thu) is July 7 (Mon)
    let payload = json!({ "date": "2025-07-03" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/next-business-day", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["nextBusinessDay"], "2025-07-07", "Next business day after Thu Jul 3 should skip holiday Fri Jul 4 and weekend to Mon Jul 7");

    // 7. Add 10 business days to Jan 1, 2025 (Jan 1 is a holiday Wednesday)
    // Jan 1 holiday, so starting from Dec 31, 2024 (Tue):
    let payload = json!({
        "startDate": "2024-12-31",
        "days": 10,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/add-business-days", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    // Dec 31 (Tue) +1-> Jan 2 (Thu, skip Jan 1 holiday) +2-> Jan 3 (Fri)
    // +3-> Jan 6 (Mon) +4-> Jan 7 +5-> Jan 8 +6-> Jan 9 +7-> Jan 10
    // +8-> Jan 13 +9-> Jan 14 +10-> Jan 15
    assert_eq!(body["resultDate"], "2025-01-15");

    // 8. Verify audit trail
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/transaction-calendars/calculations?calendar_id={}", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let audit_body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let audit_entries = audit_body["data"].as_array().unwrap();
    assert!(audit_entries.len() >= 4, "Expected at least 4 calculation entries, got {}", audit_entries.len());

    // 9. Check dashboard
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/transaction-calendars/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dashboard: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(dashboard["totalCalendars"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalExceptions"].as_i64().unwrap() >= 6);
    assert!(dashboard["totalCalculations"].as_i64().unwrap() >= 4);

    // 10. List all exceptions
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let exc_body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(exc_body["data"].as_array().unwrap().len(), 7); // 6 holidays + 1 special_working

    // 11. Deactivate and verify it can't have new exceptions
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/deactivate", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let payload = json!({
        "exceptionDate": "2025-11-27",
        "exceptionType": "holiday",
        "name": "Thanksgiving",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/exceptions", cal_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST, "Should not be able to add exceptions to inactive calendar");

    // 12. Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transaction-calendars/{}/activate", cal_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}
