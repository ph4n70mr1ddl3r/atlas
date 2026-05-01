//! Workplace Health & Safety (EHS) E2E Tests
//!
//! Tests for Oracle Fusion Environment, Health, and Safety:
//! - Safety incident CRUD, status transitions, investigation, and closure
//! - Hazard identification, risk assessment, residual risk
//! - Safety inspections, completion with findings
//! - Corrective and Preventive Actions (CAPA)
//! - Health & Safety dashboard
//! - Validation edge cases and error handling

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_health_safety_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_incident(
    app: &axum::Router,
    number: &str,
    title: &str,
    incident_type: &str,
    severity: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/incidents")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "incidentNumber": number,
                        "title": title,
                        "description": "Test incident description",
                        "incidentType": incident_type,
                        "severity": severity,
                        "priority": "high",
                        "incidentDate": "2024-06-15",
                        "incidentTime": "09:30",
                        "location": "Building A, Floor 2",
                        "oshaRecordable": false,
                        "bodyPart": "back",
                        "injurySource": "wet_floor",
                        "eventType": "slip_trip_fall",
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
            "Expected CREATED for incident but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_hazard(
    app: &axum::Router,
    code: &str,
    title: &str,
    category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/hazards")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "hazardCode": code,
                        "title": title,
                        "description": "Test hazard description",
                        "hazardCategory": category,
                        "likelihood": "likely",
                        "consequence": "major",
                        "location": "Chemical Storage Room",
                        "identifiedDate": "2024-06-01",
                        "mitigationMeasures": [{"measure": "Install ventilation", "status": "planned"}],
                        "reviewDate": "2024-09-01",
                        "ownerName": "Safety Manager",
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
            "Expected CREATED for hazard but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_inspection(
    app: &axum::Router,
    number: &str,
    title: &str,
    inspection_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "inspectionNumber": number,
                        "title": title,
                        "description": "Test inspection description",
                        "inspectionType": inspection_type,
                        "priority": "high",
                        "scheduledDate": "2024-07-01",
                        "location": "All Buildings",
                        "inspectorName": "Inspector Bob",
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
            "Expected CREATED for inspection but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_capa(
    app: &axum::Router,
    number: &str,
    title: &str,
    action_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/corrective-actions")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionNumber": number,
                        "title": title,
                        "description": "Test corrective action",
                        "actionType": action_type,
                        "priority": "high",
                        "sourceType": "incident",
                        "sourceNumber": "INC-001",
                        "rootCause": "Wet floor without warning signs",
                        "correctiveActionPlan": "Install non-slip mats",
                        "assignedToName": "Facilities Manager",
                        "dueDate": "2024-07-15",
                        "estimatedCost": 5000.0,
                        "currencyCode": "USD",
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
            "Expected CREATED for CAPA but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Incident Tests
// ============================================================================

#[tokio::test]
async fn test_create_incident() {
    let (_state, app) = setup_health_safety_test().await;
    let inc = create_test_incident(&app, "INC-E2E-001", "Slip and Fall", "injury", "medium").await;

    assert_eq!(inc["incidentNumber"], "INC-E2E-001");
    assert_eq!(inc["title"], "Slip and Fall");
    assert_eq!(inc["incidentType"], "injury");
    assert_eq!(inc["severity"], "medium");
    assert_eq!(inc["status"], "reported");
    assert_eq!(inc["priority"], "high");
}

#[tokio::test]
async fn test_create_incident_duplicate_number() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_incident(&app, "INC-DUP-001", "First", "injury", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/incidents")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "incidentNumber": "INC-DUP-001",
                        "title": "Second",
                        "incidentType": "injury",
                        "severity": "medium",
                        "priority": "high",
                        "incidentDate": "2024-06-15",
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
async fn test_create_incident_validation_bad_type() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/incidents")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "incidentNumber": "INC-BAD-001",
                        "title": "Test",
                        "incidentType": "explosion",
                        "severity": "medium",
                        "priority": "high",
                        "incidentDate": "2024-06-15",
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
async fn test_create_incident_validation_bad_severity() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/incidents")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "incidentNumber": "INC-BAD-002",
                        "title": "Test",
                        "incidentType": "injury",
                        "severity": "super_critical",
                        "priority": "high",
                        "incidentDate": "2024-06-15",
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
async fn test_get_incident() {
    let (_state, app) = setup_health_safety_test().await;
    let inc = create_test_incident(&app, "INC-GET-001", "Get Test", "injury", "high").await;
    let id = inc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/health-safety/incidents/id/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["incidentNumber"], "INC-GET-001");
}

#[tokio::test]
async fn test_list_incidents() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_incident(&app, "INC-LIST-001", "First Incident", "injury", "medium").await;
    create_test_incident(&app, "INC-LIST-002", "Second Incident", "near_miss", "low").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/incidents")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_list_incidents_with_filter() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_incident(&app, "INC-FILTER-001", "Injury", "injury", "high").await;
    create_test_incident(&app, "INC-FILTER-002", "Near Miss", "near_miss", "low").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/incidents?severity=high")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|d| d["severity"] == "high"));
}

#[tokio::test]
async fn test_update_incident_status() {
    let (_state, app) = setup_health_safety_test().await;
    let inc = create_test_incident(&app, "INC-STATUS-001", "Status Test", "injury", "medium").await;
    let id = inc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/incidents/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "under_investigation"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "under_investigation");
}

#[tokio::test]
async fn test_update_incident_investigation() {
    let (_state, app) = setup_health_safety_test().await;
    let inc = create_test_incident(&app, "INV-001", "Investigation Test", "injury", "medium").await;
    let id = inc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/incidents/id/{}/investigation", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "rootCause": "Wet floor due to leaking pipe",
                        "immediateAction": "Fixed pipe and placed warning signs",
                        "daysAwayFromWork": 3,
                        "daysRestricted": 5,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["rootCause"], "Wet floor due to leaking pipe");
    assert_eq!(body["daysAwayFromWork"], 3);
}

#[tokio::test]
async fn test_close_incident() {
    let (_state, app) = setup_health_safety_test().await;
    let inc = create_test_incident(&app, "CLOSE-001", "Close Test", "injury", "low").await;
    let id = inc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/incidents/id/{}/close", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "closed");
    assert!(body["closedDate"].is_string());
}

#[tokio::test]
async fn test_delete_incident() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_incident(&app, "DEL-INC-001", "Delete Test", "injury", "low").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health-safety/incidents/number/DEL-INC-001")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_incident_not_found() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health-safety/incidents/number/NONEXISTENT")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Hazard Tests
// ============================================================================

#[tokio::test]
async fn test_create_hazard() {
    let (_state, app) = setup_health_safety_test().await;
    let haz = create_test_hazard(&app, "HAZ-E2E-001", "Chemical Spill Risk", "chemical").await;

    assert_eq!(haz["hazardCode"], "HAZ-E2E-001");
    assert_eq!(haz["title"], "Chemical Spill Risk");
    assert_eq!(haz["hazardCategory"], "chemical");
    assert_eq!(haz["riskScore"], 16); // likely(4) * major(4)
    assert_eq!(haz["riskLevel"], "extreme");
    assert_eq!(haz["status"], "identified");
}

#[tokio::test]
async fn test_create_hazard_duplicate_code() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_hazard(&app, "HAZ-DUP-001", "First", "chemical").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/hazards")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "hazardCode": "HAZ-DUP-001",
                        "title": "Second",
                        "hazardCategory": "physical",
                        "likelihood": "possible",
                        "consequence": "minor",
                        "identifiedDate": "2024-06-01",
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
async fn test_create_hazard_validation_bad_category() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/hazards")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "hazardCode": "HAZ-BAD-001",
                        "title": "Bad Category",
                        "hazardCategory": "nuclear",
                        "likelihood": "possible",
                        "consequence": "minor",
                        "identifiedDate": "2024-06-01",
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
async fn test_create_hazard_validation_bad_likelihood() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/hazards")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "hazardCode": "HAZ-BAD-002",
                        "title": "Bad Likelihood",
                        "hazardCategory": "physical",
                        "likelihood": "frequent",
                        "consequence": "minor",
                        "identifiedDate": "2024-06-01",
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
async fn test_list_hazards() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_hazard(&app, "HAZ-LIST-001", "First", "chemical").await;
    create_test_hazard(&app, "HAZ-LIST-002", "Second", "physical").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/hazards")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_update_hazard_status() {
    let (_state, app) = setup_health_safety_test().await;
    let haz = create_test_hazard(&app, "HAZ-STATUS-001", "Status Test", "chemical").await;
    let id = haz["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/hazards/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "assessed"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "assessed");
}

#[tokio::test]
async fn test_assess_residual_risk() {
    let (_state, app) = setup_health_safety_test().await;
    let haz = create_test_hazard(&app, "HAZ-RES-001", "Residual Test", "chemical").await;
    let id = haz["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/hazards/id/{}/residual-risk", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "residualLikelihood": "unlikely",
                        "residualConsequence": "minor",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["residualRiskScore"], 4); // unlikely(2) * minor(2)
    assert_eq!(body["residualRiskLevel"], "medium");
}

#[tokio::test]
async fn test_delete_hazard() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_hazard(&app, "DEL-HAZ-001", "Delete Test", "ergonomic").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health-safety/hazards/code/DEL-HAZ-001")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Inspection Tests
// ============================================================================

#[tokio::test]
async fn test_create_inspection() {
    let (_state, app) = setup_health_safety_test().await;
    let ins = create_test_inspection(&app, "INS-E2E-001", "Q2 Fire Safety Audit", "periodic").await;

    assert_eq!(ins["inspectionNumber"], "INS-E2E-001");
    assert_eq!(ins["title"], "Q2 Fire Safety Audit");
    assert_eq!(ins["inspectionType"], "periodic");
    assert_eq!(ins["status"], "scheduled");
    assert_eq!(ins["priority"], "high");
}

#[tokio::test]
async fn test_create_inspection_duplicate_number() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_inspection(&app, "INS-DUP-001", "First", "routine").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "inspectionNumber": "INS-DUP-001",
                        "title": "Second",
                        "inspectionType": "routine",
                        "priority": "medium",
                        "scheduledDate": "2024-08-01",
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
async fn test_create_inspection_validation_bad_type() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "inspectionNumber": "INS-BAD-001",
                        "title": "Bad Type",
                        "inspectionType": "surprise",
                        "priority": "medium",
                        "scheduledDate": "2024-08-01",
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
async fn test_complete_inspection() {
    let (_state, app) = setup_health_safety_test().await;
    let ins = create_test_inspection(&app, "INS-COMP-001", "Complete Test", "routine").await;
    let id = ins["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/inspections/id/{}/complete", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "findingsSummary": "3 findings total",
                        "findings": [
                            {"type": "critical", "description": "Fire exit blocked"},
                            {"type": "nc", "description": "Expired fire extinguisher"},
                            {"type": "observation", "description": "Signage needs improvement"}
                        ],
                        "criticalFindings": 1,
                        "nonConformities": 1,
                        "observations": 1,
                        "score": 85.0,
                        "maxScore": 100.0,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "completed");
    assert_eq!(body["totalFindings"], 3);
    assert_eq!(body["criticalFindings"], 1);
    assert_eq!(body["score"], 85.0);
    assert_eq!(body["scorePct"], 85.0);
}

#[tokio::test]
async fn test_list_inspections() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_inspection(&app, "INS-LIST-001", "First", "routine").await;
    create_test_inspection(&app, "INS-LIST-002", "Second", "periodic").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/inspections")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_delete_inspection() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_inspection(&app, "DEL-INS-001", "Delete Test", "routine").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health-safety/inspections/number/DEL-INS-001")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// CAPA Tests
// ============================================================================

#[tokio::test]
async fn test_create_corrective_action() {
    let (_state, app) = setup_health_safety_test().await;
    let capa = create_test_capa(&app, "CAPA-E2E-001", "Install Non-Slip Mats", "corrective").await;

    assert_eq!(capa["actionNumber"], "CAPA-E2E-001");
    assert_eq!(capa["title"], "Install Non-Slip Mats");
    assert_eq!(capa["actionType"], "corrective");
    assert_eq!(capa["status"], "open");
    assert_eq!(capa["priority"], "high");
    assert_eq!(capa["estimatedCost"], 5000.0);
    assert_eq!(capa["currencyCode"], "USD");
}

#[tokio::test]
async fn test_create_capa_duplicate_number() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_capa(&app, "CAPA-DUP-001", "First", "corrective").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/corrective-actions")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionNumber": "CAPA-DUP-001",
                        "title": "Second",
                        "actionType": "preventive",
                        "priority": "medium",
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
async fn test_create_capa_validation_bad_type() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/corrective-actions")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionNumber": "CAPA-BAD-001",
                        "title": "Bad Type",
                        "actionType": "emergency",
                        "priority": "medium",
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
async fn test_create_capa_validation_negative_cost() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/corrective-actions")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionNumber": "CAPA-NEG-001",
                        "title": "Negative Cost",
                        "actionType": "corrective",
                        "priority": "medium",
                        "estimatedCost": -100.0,
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
async fn test_update_capa_status() {
    let (_state, app) = setup_health_safety_test().await;
    let capa = create_test_capa(&app, "CAPA-STATUS-001", "Status Test", "corrective").await;
    let id = capa["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/corrective-actions/id/{}/status", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"status": "in_progress"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "in_progress");
}

#[tokio::test]
async fn test_complete_capa() {
    let (_state, app) = setup_health_safety_test().await;
    let capa = create_test_capa(&app, "CAPA-COMP-001", "Complete Test", "corrective").await;
    let id = capa["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/corrective-actions/id/{}/complete", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "effectiveness": "effective",
                        "actualCost": 4500.0,
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "completed");
    assert_eq!(body["effectiveness"], "effective");
    assert_eq!(body["actualCost"], 4500.0);
}

#[tokio::test]
async fn test_complete_capa_bad_effectiveness() {
    let (_state, app) = setup_health_safety_test().await;
    let capa = create_test_capa(&app, "CAPA-BAD-EFF-001", "Bad Effectiveness", "corrective").await;
    let id = capa["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/health-safety/corrective-actions/id/{}/complete", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"effectiveness": "perfect"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_corrective_actions() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_capa(&app, "CAPA-LIST-001", "First", "corrective").await;
    create_test_capa(&app, "CAPA-LIST-002", "Second", "preventive").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/corrective-actions")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_delete_corrective_action() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_capa(&app, "DEL-CAPA-001", "Delete Test", "corrective").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/health-safety/corrective-actions/number/DEL-CAPA-001")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_health_safety_dashboard() {
    let (_state, app) = setup_health_safety_test().await;
    create_test_incident(&app, "INC-DASH-001", "Dash Incident", "injury", "high").await;
    create_test_hazard(&app, "HAZ-DASH-001", "Dash Hazard", "chemical").await;
    create_test_inspection(&app, "INS-DASH-001", "Dash Inspection", "routine").await;
    create_test_capa(&app, "CAPA-DASH-001", "Dash CAPA", "corrective").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    assert!(body["totalIncidents"].as_i64().unwrap() >= 1);
    assert!(body["totalHazards"].as_i64().unwrap() >= 1);
    assert!(body["totalInspections"].as_i64().unwrap() >= 1);
    assert!(body["totalCapa"].as_i64().unwrap() >= 1);
    assert!(body["openIncidents"].as_i64().unwrap() >= 1);
    assert!(body["openCapa"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Cross-cutting / Auth Tests
// ============================================================================

#[tokio::test]
async fn test_incident_requires_auth() {
    let (_state, app) = setup_health_safety_test().await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/health-safety/incidents")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_hazard_requires_auth() {
    let (_state, app) = setup_health_safety_test().await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/hazards")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dashboard_requires_auth() {
    let (_state, app) = setup_health_safety_test().await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health-safety/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_incident_not_found() {
    let (_state, app) = setup_health_safety_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_id = uuid::Uuid::new_v4();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/health-safety/incidents/id/{}", fake_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
