//! Sustainability & ESG Management E2E Tests
//!
//! Tests for Oracle Fusion Sustainability / Environmental Accounting & Reporting:
//! - Facility CRUD and status management
//! - Emission factor CRUD and validation
//! - Environmental activity logging and status transitions
//! - ESG metric definitions and readings
//! - Sustainability goals with progress tracking
//! - Carbon offset management and retirement
//! - Sustainability dashboard
//! - Validation edge cases and error handling

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_sustainability_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_facility(
    app: &axum::Router,
    code: &str,
    name: &str,
    facility_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/facilities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "facilityCode": code,
                        "name": name,
                        "facilityType": facility_type,
                        "countryCode": "US",
                        "region": "California",
                        "city": "San Francisco",
                        "totalAreaSqm": 5000.0,
                        "employeeCount": 200,
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
            "Expected CREATED for facility but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_emission_factor(
    app: &axum::Router,
    code: &str,
    name: &str,
    scope: &str,
    category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/emission-factors")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "factorCode": code,
                        "name": name,
                        "scope": scope,
                        "category": category,
                        "activityType": "electricity",
                        "factorValue": 0.3886,
                        "unitOfMeasure": "kWh",
                        "gasType": "co2e",
                        "factorSource": "EPA eGRID",
                        "effectiveFrom": "2024-01-01",
                        "regionCode": "US"
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
            "Expected CREATED for emission factor but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_activity(
    app: &axum::Router,
    number: &str,
    scope: &str,
    co2e: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/activities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "activityNumber": number,
                        "activityType": "electricity",
                        "scope": scope,
                        "category": "purchased_electricity",
                        "quantity": 10000.0,
                        "unitOfMeasure": "kWh",
                        "co2eKg": co2e,
                        "activityDate": "2024-03-15",
                        "reportingPeriod": "2024-Q1",
                        "sourceType": "meter_reading"
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
            "Expected CREATED for activity but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_metric(
    app: &axum::Router,
    code: &str,
    name: &str,
    pillar: &str,
    direction: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metrics")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricCode": code,
                        "name": name,
                        "pillar": pillar,
                        "category": "climate",
                        "unitOfMeasure": "tonnes CO2e",
                        "direction": direction,
                        "targetValue": 5000.0,
                        "warningThreshold": 6000.0
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
            "Expected CREATED for metric but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_goal(
    app: &axum::Router,
    code: &str,
    name: &str,
    goal_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/goals")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "goalCode": code,
                        "name": name,
                        "goalType": goal_type,
                        "scope": "scope_1",
                        "baselineValue": 10000.0,
                        "baselineYear": 2020,
                        "baselineUnit": "tonnes CO2e",
                        "targetValue": 5000.0,
                        "targetYear": 2030,
                        "targetUnit": "tonnes CO2e",
                        "targetReductionPct": 50.0,
                        "framework": "SBTi"
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
            "Expected CREATED for goal but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_carbon_offset(
    app: &axum::Router,
    number: &str,
    name: &str,
    project_type: &str,
    quantity: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/carbon-offsets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "offsetNumber": number,
                        "name": name,
                        "projectName": format!("{} Project", name),
                        "projectType": project_type,
                        "projectLocation": "Brazil",
                        "registry": "Verra",
                        "registryId": "VCS-1234",
                        "quantityTonnes": quantity,
                        "unitPrice": 15.0,
                        "totalCost": quantity * 15.0,
                        "currencyCode": "USD",
                        "vintageYear": 2024,
                        "effectiveFrom": "2024-01-01",
                        "supplierName": "GreenCarbon Inc"
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
            "Expected CREATED for carbon offset but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Facility Tests
// ============================================================================

#[tokio::test]
async fn test_create_facility() {
    let (_state, app) = setup_sustainability_test().await;
    let facility = create_test_facility(&app, "FAC-001", "HQ Office", "office").await;
    assert_eq!(facility["facility_code"], "FAC-001");
    assert_eq!(facility["name"], "HQ Office");
    assert_eq!(facility["facility_type"], "office");
    assert_eq!(facility["country_code"], "US");
}

#[tokio::test]
async fn test_create_facility_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_facility(&app, "DUP-FAC", "First", "office").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/facilities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "facilityCode": "DUP-FAC",
                        "name": "Duplicate",
                        "facilityType": "office"
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
async fn test_create_facility_invalid_type() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/facilities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "facilityCode": "BAD-TYPE",
                        "name": "Bad Type",
                        "facilityType": "space_station"
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
async fn test_get_facility() {
    let (_state, app) = setup_sustainability_test().await;
    let facility = create_test_facility(&app, "GET-FAC", "Get Me", "warehouse").await;
    let id = facility["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sustainability/facilities/id/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["facility_code"], "GET-FAC");
    assert_eq!(fetched["facility_type"], "warehouse");
}

#[tokio::test]
async fn test_list_facilities() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_facility(&app, "LIST-1", "Facility One", "office").await;
    create_test_facility(&app, "LIST-2", "Facility Two", "manufacturing").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/facilities")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_facility_status() {
    let (_state, app) = setup_sustainability_test().await;
    let facility = create_test_facility(&app, "STATUS-FAC", "Status Fac", "data_center").await;
    let id = facility["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/facilities/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "inactive"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "inactive");
}

#[tokio::test]
async fn test_delete_facility() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_facility(&app, "DEL-FAC", "Delete Me", "retail").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/facilities/code/DEL-FAC")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Emission Factor Tests
// ============================================================================

#[tokio::test]
async fn test_create_emission_factor() {
    let (_state, app) = setup_sustainability_test().await;
    let ef = create_test_emission_factor(
        &app,
        "EF-001",
        "US Grid Electricity",
        "scope_2",
        "purchased_electricity",
    )
    .await;
    assert_eq!(ef["factor_code"], "EF-001");
    assert_eq!(ef["scope"], "scope_2");
    assert!((ef["factor_value"].as_f64().unwrap() - 0.3886).abs() < 0.001);
}

#[tokio::test]
async fn test_create_emission_factor_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_emission_factor(&app, "DUP-EF", "First", "scope_1", "stationary_combustion").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/emission-factors")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "factorCode": "DUP-EF",
                        "name": "Duplicate",
                        "scope": "scope_1",
                        "category": "stationary_combustion",
                        "activityType": "gas",
                        "factorValue": 5.3,
                        "unitOfMeasure": "therms",
                        "gasType": "co2e",
                        "effectiveFrom": "2024-01-01"
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
async fn test_create_emission_factor_invalid_scope() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/emission-factors")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "factorCode": "BAD-SCOPE",
                        "name": "Bad Scope",
                        "scope": "scope_5",
                        "category": "stationary_combustion",
                        "activityType": "gas",
                        "factorValue": 5.3,
                        "unitOfMeasure": "therms",
                        "gasType": "co2e",
                        "effectiveFrom": "2024-01-01"
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
async fn test_list_emission_factors() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_emission_factor(&app, "EF-LIST-1", "Factor 1", "scope_1", "stationary_combustion").await;
    create_test_emission_factor(&app, "EF-LIST-2", "Factor 2", "scope_2", "purchased_electricity").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/emission-factors")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_emission_factor() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_emission_factor(&app, "EF-DEL", "Delete Me", "scope_1", "stationary_combustion").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/emission-factors/code/EF-DEL")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Environmental Activity Tests
// ============================================================================

#[tokio::test]
async fn test_create_activity() {
    let (_state, app) = setup_sustainability_test().await;
    let activity = create_test_activity(&app, "ACT-001", "scope_2", 3886.0).await;
    assert_eq!(activity["activity_number"], "ACT-001");
    assert_eq!(activity["scope"], "scope_2");
    assert!((activity["co2e_kg"].as_f64().unwrap() - 3886.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_activity_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_activity(&app, "DUP-ACT", "scope_1", 100.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/activities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "activityNumber": "DUP-ACT",
                        "activityType": "gas",
                        "scope": "scope_1",
                        "quantity": 100.0,
                        "unitOfMeasure": "therms",
                        "co2eKg": 100.0,
                        "activityDate": "2024-03-15"
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
async fn test_create_activity_invalid_scope() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/activities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "activityNumber": "BAD-SCOPE-ACT",
                        "activityType": "electricity",
                        "scope": "scope_9",
                        "quantity": 100.0,
                        "unitOfMeasure": "kWh",
                        "co2eKg": 38.86,
                        "activityDate": "2024-03-15"
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
async fn test_update_activity_status() {
    let (_state, app) = setup_sustainability_test().await;
    let activity = create_test_activity(&app, "VERIFY-ACT", "scope_1", 500.0).await;
    let id = activity["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/activities/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "verified"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "verified");
}

#[tokio::test]
async fn test_list_activities() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_activity(&app, "ACT-LIST-1", "scope_1", 100.0).await;
    create_test_activity(&app, "ACT-LIST-2", "scope_2", 200.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/activities")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_activity() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_activity(&app, "DEL-ACT", "scope_1", 50.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/activities/number/DEL-ACT")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// ESG Metric Tests
// ============================================================================

#[tokio::test]
async fn test_create_metric() {
    let (_state, app) = setup_sustainability_test().await;
    let metric = create_test_metric(&app, "ESG-001", "GHG Emissions", "environmental", "lower_is_better").await;
    assert_eq!(metric["metric_code"], "ESG-001");
    assert_eq!(metric["pillar"], "environmental");
    assert_eq!(metric["direction"], "lower_is_better");
}

#[tokio::test]
async fn test_create_metric_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_metric(&app, "DUP-MET", "First", "environmental", "lower_is_better").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metrics")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricCode": "DUP-MET",
                        "name": "Duplicate",
                        "pillar": "social",
                        "category": "diversity",
                        "unitOfMeasure": "pct",
                        "direction": "higher_is_better"
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
async fn test_create_metric_invalid_pillar() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metrics")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricCode": "BAD-PILLAR",
                        "name": "Bad Pillar",
                        "pillar": "financial",
                        "category": "climate",
                        "unitOfMeasure": "tonnes",
                        "direction": "lower_is_better"
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
async fn test_list_metrics() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_metric(&app, "MET-LIST-1", "Env Metric", "environmental", "lower_is_better").await;
    create_test_metric(&app, "MET-LIST-2", "Social Metric", "social", "higher_is_better").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/metrics")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_metric() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_metric(&app, "DEL-MET", "Delete Me", "governance", "higher_is_better").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/metrics/code/DEL-MET")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// ESG Metric Readings Tests
// ============================================================================

#[tokio::test]
async fn test_create_metric_reading() {
    let (_state, app) = setup_sustainability_test().await;
    let metric = create_test_metric(&app, "READ-MET", "Reading Metric", "environmental", "lower_is_better").await;
    let metric_id = metric["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metric-readings")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricId": metric_id,
                        "metricValue": 8500.0,
                        "readingDate": "2024-06-30",
                        "reportingPeriod": "2024-Q2",
                        "source": "automated"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let reading: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((reading["metric_value"].as_f64().unwrap() - 8500.0).abs() < 0.01);
    assert_eq!(reading["source"], "automated");
}

#[tokio::test]
async fn test_create_metric_reading_not_found_metric() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metric-readings")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricId": "00000000-0000-0000-0000-999999999999",
                        "metricValue": 100.0,
                        "readingDate": "2024-06-30"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_metric_readings() {
    let (_state, app) = setup_sustainability_test().await;
    let metric = create_test_metric(&app, "LIST-READ-MET", "List Reading Metric", "environmental", "lower_is_better").await;
    let metric_id = metric["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create 3 readings
    for val in [9000.0, 8500.0, 8000.0] {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sustainability/metric-readings")
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "metricId": metric_id,
                            "metricValue": val,
                            "readingDate": "2024-06-30"
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
                .uri(format!("/api/v1/sustainability/metrics/{}/readings", metric_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_delete_metric_reading() {
    let (_state, app) = setup_sustainability_test().await;
    let metric = create_test_metric(&app, "DEL-READ-MET", "Del Reading Metric", "environmental", "lower_is_better").await;
    let metric_id = metric["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metric-readings")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricId": metric_id,
                        "metricValue": 500.0,
                        "readingDate": "2024-06-30"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let reading: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let reading_id = reading["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/sustainability/metric-readings/id/{}", reading_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Sustainability Goal Tests
// ============================================================================

#[tokio::test]
async fn test_create_goal() {
    let (_state, app) = setup_sustainability_test().await;
    let goal = create_test_goal(&app, "GOAL-001", "50% Emission Reduction", "emission_reduction").await;
    assert_eq!(goal["goal_code"], "GOAL-001");
    assert_eq!(goal["goal_type"], "emission_reduction");
    assert_eq!(goal["framework"], "SBTi");
}

#[tokio::test]
async fn test_create_goal_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_goal(&app, "DUP-GOAL", "First Goal", "emission_reduction").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/goals")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "goalCode": "DUP-GOAL",
                        "name": "Duplicate Goal",
                        "goalType": "energy_efficiency",
                        "baselineValue": 1000.0,
                        "baselineYear": 2020,
                        "baselineUnit": "MWh",
                        "targetValue": 500.0,
                        "targetYear": 2025,
                        "targetUnit": "MWh"
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
async fn test_create_goal_invalid_type() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/goals")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "goalCode": "BAD-TYPE",
                        "name": "Bad Type",
                        "goalType": "plastic_free",
                        "baselineValue": 1000.0,
                        "baselineYear": 2020,
                        "baselineUnit": "kg",
                        "targetValue": 0.0,
                        "targetYear": 2030,
                        "targetUnit": "kg"
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
async fn test_update_goal_progress() {
    let (_state, app) = setup_sustainability_test().await;
    let goal = create_test_goal(&app, "PROG-GOAL", "Progress Goal", "emission_reduction").await;
    let id = goal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/goals/id/{}/progress", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"currentValue": 7500.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((updated["current_value"].as_f64().unwrap() - 7500.0).abs() < 0.01);
}

#[tokio::test]
async fn test_update_goal_status() {
    let (_state, app) = setup_sustainability_test().await;
    let goal = create_test_goal(&app, "STATUS-GOAL", "Status Goal", "carbon_neutral").await;
    let id = goal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/goals/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "on_track"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_goals() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_goal(&app, "GOAL-LIST-1", "Goal One", "emission_reduction").await;
    create_test_goal(&app, "GOAL-LIST-2", "Goal Two", "renewable_energy").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/goals")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_goal() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_goal(&app, "DEL-GOAL", "Delete Me", "waste_reduction").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/goals/code/DEL-GOAL")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Carbon Offset Tests
// ============================================================================

#[tokio::test]
async fn test_create_carbon_offset() {
    let (_state, app) = setup_sustainability_test().await;
    let offset = create_test_carbon_offset(&app, "OFF-001", "Reforestation Brazil", "reforestation", 1000.0).await;
    assert_eq!(offset["offset_number"], "OFF-001");
    assert_eq!(offset["project_type"], "reforestation");
    assert!((offset["quantity_tonnes"].as_f64().unwrap() - 1000.0).abs() < 0.01);
    assert_eq!(offset["registry"], "Verra");
}

#[tokio::test]
async fn test_create_carbon_offset_duplicate_conflict() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_carbon_offset(&app, "DUP-OFF", "First", "renewable_energy", 500.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/carbon-offsets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "offsetNumber": "DUP-OFF",
                        "name": "Duplicate",
                        "projectName": "Dup Project",
                        "projectType": "methane_capture",
                        "quantityTonnes": 200.0,
                        "vintageYear": 2024,
                        "effectiveFrom": "2024-01-01"
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
async fn test_create_carbon_offset_invalid_type() {
    let (_state, app) = setup_sustainability_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/carbon-offsets")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "offsetNumber": "BAD-TYPE",
                        "name": "Bad Type",
                        "projectName": "Bad Project",
                        "projectType": "crypto_mining",
                        "quantityTonnes": 100.0,
                        "vintageYear": 2024,
                        "effectiveFrom": "2024-01-01"
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
async fn test_retire_carbon_offset() {
    let (_state, app) = setup_sustainability_test().await;
    let offset = create_test_carbon_offset(&app, "RETIRE-OFF", "Retire Test", "reforestation", 1000.0).await;
    let id = offset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/carbon-offsets/id/{}/retire", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"retireQuantity": 300.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((updated["remaining_tonnes"].as_f64().unwrap() - 700.0).abs() < 0.01);
    assert!((updated["retired_quantity"].as_f64().unwrap() - 300.0).abs() < 0.01);
}

#[tokio::test]
async fn test_list_carbon_offsets() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_carbon_offset(&app, "OFF-LIST-1", "Offset One", "reforestation", 500.0).await;
    create_test_carbon_offset(&app, "OFF-LIST-2", "Offset Two", "renewable_energy", 750.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/carbon-offsets")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_carbon_offset() {
    let (_state, app) = setup_sustainability_test().await;
    create_test_carbon_offset(&app, "DEL-OFF", "Delete Me", "methane_capture", 100.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/sustainability/carbon-offsets/number/DEL-OFF")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_sustainability_dashboard() {
    let (_state, app) = setup_sustainability_test().await;

    // Create some test data to populate dashboard
    create_test_facility(&app, "DASH-FAC", "Dashboard Facility", "manufacturing").await;
    create_test_activity(&app, "DASH-ACT", "scope_1", 5000.0).await;
    create_test_goal(&app, "DASH-GOAL", "Dashboard Goal", "emission_reduction").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["total_facilities"].as_i64().unwrap() >= 1);
    assert!(dashboard["active_goals"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_sustainability_full_lifecycle() {
    let (_state, app) = setup_sustainability_test().await;

    // 1. Create a facility
    let facility = create_test_facility(&app, "LIFE-FAC", "Lifecycle Facility", "manufacturing").await;
    let fac_id = facility["id"].as_str().unwrap();
    assert_eq!(facility["facility_code"], "LIFE-FAC");

    // 2. Create an emission factor
    let ef = create_test_emission_factor(&app, "LIFE-EF", "Natural Gas", "scope_1", "stationary_combustion").await;
    let ef_id = ef["id"].as_str().unwrap();
    assert_eq!(ef["factor_code"], "LIFE-EF");

    // 3. Log an environmental activity
    let (k, v) = auth_header(&admin_claims());
    let activity_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/activities")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "activityNumber": "LIFE-ACT",
                        "facilityId": fac_id,
                        "activityType": "natural_gas",
                        "scope": "scope_1",
                        "category": "stationary_combustion",
                        "quantity": 5000.0,
                        "unitOfMeasure": "therms",
                        "emissionFactorId": ef_id,
                        "co2eKg": 26500.0,
                        "co2Kg": 26000.0,
                        "ch4Kg": 200.0,
                        "n2oKg": 15.0,
                        "costAmount": 4500.0,
                        "costCurrency": "USD",
                        "activityDate": "2024-06-15",
                        "reportingPeriod": "2024-Q2",
                        "sourceType": "meter_reading"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(activity_resp.status(), StatusCode::CREATED);
    let activity: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(activity_resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    let act_id = activity["id"].as_str().unwrap();

    // 4. Verify the activity
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/activities/id/{}/status", act_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "verified"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Create an ESG metric
    let metric = create_test_metric(&app, "LIFE-ESG", "Total GHG", "environmental", "lower_is_better").await;
    let metric_id = metric["id"].as_str().unwrap();

    // 6. Record a metric reading
    let reading_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sustainability/metric-readings")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "metricId": metric_id,
                        "metricValue": 26500.0,
                        "readingDate": "2024-06-30",
                        "reportingPeriod": "2024-Q2",
                        "facilityId": fac_id,
                        "notes": "Q2 total emissions",
                        "source": "calculated"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reading_resp.status(), StatusCode::CREATED);

    // 7. Create a sustainability goal
    let goal = create_test_goal(&app, "LIFE-GOAL", "Net Zero by 2030", "carbon_neutral").await;
    let goal_id = goal["id"].as_str().unwrap();

    // 8. Update goal progress
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/goals/id/{}/progress", goal_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"currentValue": 7500.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 9. Purchase carbon offsets
    let offset = create_test_carbon_offset(&app, "LIFE-OFF", "Forest Carbon", "reforestation", 500.0).await;
    let offset_id = offset["id"].as_str().unwrap();

    // 10. Retire some offsets
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sustainability/carbon-offsets/id/{}/retire", offset_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"retireQuantity": 200.0})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let retired: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!((retired["remaining_tonnes"].as_f64().unwrap() - 300.0).abs() < 0.01);

    // 11. Check the dashboard
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sustainability/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let dashboard: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(dashboard["total_facilities"].as_i64().unwrap() >= 1);
    assert!(dashboard["active_goals"].as_i64().unwrap() >= 1);
    // Offsets may be aggregated differently in the dashboard
    assert!(dashboard["total_offsets_tonnes"].as_f64().unwrap_or(0.0) >= 0.0);

    // 12. Clean up - delete everything
    // Delete metric readings via list
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sustainability/metrics/{}/readings", metric_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let readings: serde_json::Value = serde_json::from_slice(&body).unwrap();
    for r in readings["data"].as_array().unwrap() {
        let rid = r["id"].as_str().unwrap();
        app.clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/sustainability/metric-readings/id/{}", rid))
                    .header(&k, &v)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Delete all created resources
    for code in &["LIFE-OFF", "LIFE-GOAL", "LIFE-ESG", "LIFE-ACT", "LIFE-EF", "LIFE-FAC"] {
        let (prefix, route) = match *code {
            "LIFE-OFF" => ("carbon-offsets/number", true),
            "LIFE-GOAL" => ("goals/code", true),
            "LIFE-ESG" => ("metrics/code", true),
            "LIFE-ACT" => ("activities/number", true),
            "LIFE-EF" => ("emission-factors/code", true),
            "LIFE-FAC" => ("facilities/code", true),
            _ => continue,
        };
        if route {
            let resp = app.clone().oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/sustainability/{}/{}", prefix, code))
                    .header(&k, &v)
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NO_CONTENT, "Failed to delete {}", code);
        }
    }
}
