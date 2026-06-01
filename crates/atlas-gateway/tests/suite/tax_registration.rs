//! Tax Registration Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Tax > Tax Registrations:
//! - CRUD (create, get, list, get by number)
//! - Full lifecycle (pending → active → suspended → active → deregistered)
//! - Registration number validation per country/type
//! - Duplicate detection
//! - Compliance gap detection
//! - Dashboard summary
//! - Validation edge cases

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Clean tax registration test data
    sqlx::query("DELETE FROM _atlas.tax_registration_activities")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query("DELETE FROM _atlas.tax_registrations")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!(
        "../../../../migrations/134_tax_registration.sql"
    ))
    .execute(&state.db_pool)
    .await
    .expect("Failed to run tax registration migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_us_ein(app: &axum::Router, number: &str, party_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": number,
        "registrationType": "ein",
        "taxPurpose": "both",
        "partyType": party_type,
        "partyId": "00000000-0000-0000-0000-000000000010",
        "partyName": "Test Legal Entity",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
        "isDefault": true,
        "reportingName": "Test Corp",
        "source": "manual",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE REGISTRATION status={}: {}", status, body_str);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create registration: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

async fn create_gb_vat(app: &axum::Router, number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": number,
        "registrationType": "vat",
        "taxPurpose": "output_tax",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "GB-VAT",
        "countryCode": "GB",
        "effectiveFrom": "2025-01-01",
        "source": "manual",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE GB VAT status={}: {}", status, body_str);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create registration: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_first_party_registration() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;

    assert_eq!(reg["registrationNumber"], "12-3456789");
    assert_eq!(reg["registrationType"], "ein");
    assert_eq!(reg["taxPurpose"], "both");
    assert_eq!(reg["partyType"], "first_party");
    assert_eq!(reg["jurisdictionCode"], "US-FED");
    assert_eq!(reg["countryCode"], "US");
    assert_eq!(reg["status"], "active");
    assert_eq!(reg["isDefault"], true);
    assert_eq!(reg["source"], "manual");
}

#[tokio::test]
async fn test_create_third_party_registration() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "GB987654321",
        "registrationType": "vat",
        "taxPurpose": "input_tax",
        "partyType": "third_party",
        "partyName": "UK Supplier Ltd",
        "jurisdictionCode": "GB-VAT",
        "countryCode": "GB",
        "effectiveFrom": "2025-03-01",
        "effectiveTo": "2026-12-31",
        "isDefault": false,
        "source": "import",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["partyType"], "third_party");
    assert_eq!(body["effectiveTo"], "2026-12-31");
    assert_eq!(body["source"], "import");
}

#[tokio::test]
async fn test_get_registration() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/tax-registrations/{}", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["id"], reg["id"]);
    assert_eq!(body["registrationNumber"], "12-3456789");
}

#[tokio::test]
async fn test_get_registration_by_number() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations/number/12-3456789")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["registrationNumber"], "12-3456789");
}

#[tokio::test]
async fn test_list_registrations() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;
    create_gb_vat(&app, "GB123456789").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_registrations_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations?status=active")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|r| r["status"] == "active"));
}

#[tokio::test]
async fn test_list_registrations_filter_by_country() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;
    create_gb_vat(&app, "GB123456789").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations?countryCode=GB")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|r| r["countryCode"] == "GB"));
    assert!(data.len() >= 1);
}

#[tokio::test]
async fn test_list_registrations_filter_by_party_type() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations?partyType=first_party")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|r| r["partyType"] == "first_party"));
}

#[tokio::test]
async fn test_get_registration_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/tax-registrations/{}", Uuid::new_v4()))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_registration_by_number_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations/number/NONEXISTENT-999")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_active_to_deregistered() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Suspend
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/suspend", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "suspended");

    // Reactivate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/reactivate", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "active");

    // Deregister
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/deregister", reg_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "deregistrationDate": "2025-12-31",
                        "reason": "Business closure"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "deregistered");
}

#[tokio::test]
async fn test_suspend_from_suspended_fails() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Suspend once
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/suspend", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to suspend again
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/suspend", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_deregister_from_suspended() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Suspend first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/suspend", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Deregister from suspended should work
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/deregister", reg_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "deregistered");
}

#[tokio::test]
async fn test_deregister_from_deregistered_fails() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deregister
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/deregister", reg_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to deregister again
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/deregister", reg_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reactivate_from_active_fails() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Reactivate from active should fail
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/reactivate", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_validate_registration() {
    let (_state, app) = setup_test().await;
    let reg = create_us_ein(&app, "12-3456789", "first_party").await;
    let reg_id: Uuid = reg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/tax-registrations/{}/validate", reg_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["registrationNumber"], "12-3456789");
}

#[tokio::test]
async fn test_create_empty_registration_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "",
        "registrationType": "ein",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_registration_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345",
        "registrationType": "passport",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_party_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345",
        "registrationType": "tin",
        "partyType": "government",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_first_party_without_party_id_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345",
        "registrationType": "tin",
        "partyType": "first_party",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_us_ein_format_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12-345678",  // Only 8 digits
        "registrationType": "ein",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_country_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345",
        "registrationType": "tin",
        "partyType": "third_party",
        "jurisdictionCode": "XX-FED",
        "countryCode": "XXX",  // Must be 2 chars
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_date_range_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12-3456789",
        "registrationType": "ein",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-12-31",
        "effectiveTo": "2025-01-01",  // before effectiveFrom
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_source_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12-3456789",
        "registrationType": "ein",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
        "source": "unknown_source",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Multi-country Registration Tests
// ============================================================================

#[tokio::test]
async fn test_create_eu_vat_de() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "DE123456789",
        "registrationType": "vat",
        "taxPurpose": "both",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "EU-DE",
        "countryCode": "DE",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_in_gst_valid() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "29AAACR5055K1Z4",
        "registrationType": "gst",
        "taxPurpose": "both",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "IN-GST",
        "countryCode": "IN",
        "effectiveFrom": "2025-04-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_in_gst_invalid_format_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "INVALID-GST",  // Not 15 chars
        "registrationType": "gst",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "IN-GST",
        "countryCode": "IN",
        "effectiveFrom": "2025-04-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_au_abn_valid() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "53004085616",
        "registrationType": "gst",
        "taxPurpose": "both",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "AU-GST",
        "countryCode": "AU",
        "effectiveFrom": "2025-07-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_au_abn_invalid_checksum_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345678901",  // Valid length, bad checksum
        "registrationType": "gst",
        "partyType": "first_party",
        "partyId": "00000000-0000-0000-0000-000000000010",
        "jurisdictionCode": "AU-GST",
        "countryCode": "AU",
        "effectiveFrom": "2025-07-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_us_ein(&app, "12-3456789", "first_party").await;
    create_gb_vat(&app, "GB123456789").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tax-registrations/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    assert!(body.get("totalRegistrations").is_some());
    assert!(body.get("activeRegistrations").is_some());
    assert!(body.get("suspendedRegistrations").is_some());
    assert!(body.get("expiredRegistrations").is_some());
    assert!(body.get("pendingRegistrations").is_some());
    assert!(body.get("firstPartyCount").is_some());
    assert!(body.get("thirdPartyCount").is_some());
    assert!(body.get("jurisdictionsCovered").is_some());
}

// ============================================================================
// Third-party without party_id is allowed
// ============================================================================

#[tokio::test]
async fn test_create_third_party_without_party_id() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12-3456789",
        "registrationType": "ein",
        "partyType": "third_party",
        "partyName": "Acme Corp",
        "jurisdictionCode": "US-FED",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_empty_jurisdiction_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "registrationNumber": "12345",
        "registrationType": "tin",
        "partyType": "third_party",
        "jurisdictionCode": "",
        "countryCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tax-registrations")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
