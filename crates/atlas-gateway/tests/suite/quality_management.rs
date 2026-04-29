//! Quality Management E2E Tests
//!
//! Tests for Oracle Fusion Quality Management:
//! - Inspection plan CRUD and validation
//! - Plan criteria management
//! - Quality inspection lifecycle (create → start → complete)
//! - Inspection result recording
//! - Non-conformance reports (NCRs) with full lifecycle
//! - Corrective & preventive actions (CAPA) with verification
//! - Quality holds with release workflow
//! - Quality dashboard
//! - Validation edge cases and error handling

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_quality_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_plan(
    app: &axum::Router,
    code: &str,
    name: &str,
    plan_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/plans")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planCode": code,
                        "name": name,
                        "planType": plan_type,
                        "inspectionTrigger": "every_receipt",
                        "samplingMethod": "full",
                        "frequency": "per_lot"
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
            "Expected CREATED for plan but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_inspection(
    app: &axum::Router,
    plan_id: &str,
    qty_inspected: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planId": plan_id,
                        "sourceType": "receiving",
                        "itemCode": "ITEM-001",
                        "itemDescription": "Test Item",
                        "lotNumber": "LOT-001",
                        "quantityInspected": qty_inspected,
                        "quantityAccepted": qty_inspected,
                        "quantityRejected": "0",
                        "unitOfMeasure": "pcs",
                        "inspectionDate": "2024-06-15"
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

async fn create_test_ncr(
    app: &axum::Router,
    title: &str,
    ncr_type: &str,
    severity: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/ncrs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "title": title,
                        "ncrType": ncr_type,
                        "severity": severity,
                        "origin": "inspection",
                        "itemCode": "ITEM-001",
                        "detectedDate": "2024-06-15",
                        "detectedBy": "QA Inspector",
                        "responsibleParty": "Supplier A"
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
            "Expected CREATED for NCR but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_hold(
    app: &axum::Router,
    reason: &str,
    hold_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/holds")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": reason,
                        "holdType": hold_type,
                        "itemCode": "ITEM-001"
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
            "Expected CREATED for hold but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Inspection Plan Tests
// ============================================================================

#[tokio::test]
async fn test_create_plan() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "QP-001", "Receiving Inspection", "receiving").await;
    assert_eq!(plan["plan_code"], "QP-001");
    assert_eq!(plan["name"], "Receiving Inspection");
    assert_eq!(plan["plan_type"], "receiving");
    assert_eq!(plan["inspection_trigger"], "every_receipt");
    assert_eq!(plan["sampling_method"], "full");
    assert_eq!(plan["frequency"], "per_lot");
    assert_eq!(plan["is_active"], true);
}

#[tokio::test]
async fn test_create_plan_invalid_type() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/plans")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planCode": "BAD-TYPE",
                        "name": "Bad Type Plan",
                        "planType": "space_inspection",
                        "inspectionTrigger": "every_receipt",
                        "samplingMethod": "full",
                        "frequency": "per_lot"
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
async fn test_create_plan_duplicate_conflict() {
    let (_state, app) = setup_quality_test().await;
    create_test_plan(&app, "DUP-PLAN", "First Plan", "receiving").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/plans")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planCode": "DUP-PLAN",
                        "name": "Duplicate Plan",
                        "planType": "receiving",
                        "inspectionTrigger": "every_receipt",
                        "samplingMethod": "full",
                        "frequency": "per_lot"
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
async fn test_get_plan() {
    let (_state, app) = setup_quality_test().await;
    create_test_plan(&app, "GET-PLAN", "Get Plan", "in_process").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/plans/GET-PLAN")
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
    let plan: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(plan["plan_code"], "GET-PLAN");
    assert_eq!(plan["plan_type"], "in_process");
}

#[tokio::test]
async fn test_get_plan_not_found() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/plans/NONEXISTENT")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_plans() {
    let (_state, app) = setup_quality_test().await;
    create_test_plan(&app, "LIST-P1", "Plan One", "receiving").await;
    create_test_plan(&app, "LIST-P2", "Plan Two", "final").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/plans")
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
async fn test_delete_plan() {
    let (_state, app) = setup_quality_test().await;
    create_test_plan(&app, "DEL-PLAN", "Delete Me", "audit").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/quality/plans/DEL-PLAN")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Plan Criteria Tests
// ============================================================================

#[tokio::test]
async fn test_create_criterion() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "CRIT-PLAN", "Criteria Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionNumber": 1,
                        "name": "Dimension Check",
                        "characteristic": "length",
                        "measurementType": "numeric",
                        "targetValue": "10.00",
                        "lowerSpecLimit": "9.90",
                        "upperSpecLimit": "10.10",
                        "unitOfMeasure": "mm",
                        "isMandatory": true,
                        "weight": "1.0",
                        "criticality": "critical"
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
    let criterion: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(criterion["name"], "Dimension Check");
    assert_eq!(criterion["measurement_type"], "numeric");
    assert_eq!(criterion["criticality"], "critical");
}

#[tokio::test]
async fn test_create_criterion_invalid_measurement_type() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "CRIT-BAD", "Bad Crit Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionNumber": 1,
                        "name": "Bad Criterion",
                        "characteristic": "test",
                        "measurementType": "xray_scan",
                        "weight": "1.0",
                        "criticality": "major"
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
async fn test_list_criteria() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "LIST-CRIT", "List Crit Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create 2 criteria
    for (i, name) in ["Length Check", "Visual Check"].iter().enumerate() {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "criterionNumber": i + 1,
                            "name": name,
                            "characteristic": "test",
                            "measurementType": "pass_fail",
                            "weight": "1.0",
                            "criticality": "major"
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
                .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
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
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_criterion() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "DEL-CRIT", "Del Crit Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionNumber": 1,
                        "name": "To Delete",
                        "characteristic": "test",
                        "measurementType": "pass_fail",
                        "weight": "1.0",
                        "criticality": "minor"
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
    let criterion: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let crit_id = criterion["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/quality/criteria/{}", crit_id))
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
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "INSP-PLAN", "Inspection Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    let inspection = create_test_inspection(&app, plan_id, "100").await;
    assert!(inspection["inspection_number"].as_str().unwrap().starts_with("QI-"));
    assert_eq!(inspection["status"], "planned");
    assert_eq!(inspection["verdict"], "pending");
}

#[tokio::test]
async fn test_create_inspection_invalid_quantities() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "INSP-BAD", "Bad Qty Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();

    // Negative quantity
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planId": plan_id,
                        "sourceType": "receiving",
                        "quantityInspected": "-10",
                        "quantityAccepted": "0",
                        "quantityRejected": "0",
                        "inspectionDate": "2024-06-15"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Accepted + Rejected > Inspected
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/inspections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "planId": plan_id,
                        "sourceType": "receiving",
                        "quantityInspected": "10",
                        "quantityAccepted": "8",
                        "quantityRejected": "5",
                        "inspectionDate": "2024-06-15"
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
async fn test_inspection_lifecycle() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "LIFE-PLAN", "Lifecycle Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();
    let inspection = create_test_inspection(&app, plan_id, "50").await;
    let id = inspection["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Start inspection
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/start", id))
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
    let started: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(started["status"], "in_progress");

    // Record a result
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/results", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionName": "Visual Inspection",
                        "characteristic": "appearance",
                        "measurementType": "pass_fail",
                        "resultStatus": "pass",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Complete inspection
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/complete", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "verdict": "pass",
                        "notes": "All criteria met"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["verdict"], "pass");
}

#[tokio::test]
async fn test_cancel_inspection() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "CANCEL-PLAN", "Cancel Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();
    let inspection = create_test_inspection(&app, plan_id, "25").await;
    let id = inspection["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/cancel", id))
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
    let cancelled: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_inspections() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "LIST-INS", "List Inspections Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();
    create_test_inspection(&app, plan_id, "10").await;
    create_test_inspection(&app, plan_id, "20").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/inspections")
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
async fn test_inspection_results_list() {
    let (_state, app) = setup_quality_test().await;
    let plan = create_test_plan(&app, "RES-PLAN", "Results Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();
    let inspection = create_test_inspection(&app, plan_id, "30").await;
    let insp_id = inspection["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Start first
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/start", insp_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Record results
    for i in 0..3 {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/quality/inspections/{}/results", insp_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "criterionName": format!("Check {}", i + 1),
                            "characteristic": "test",
                            "measurementType": "pass_fail",
                            "resultStatus": "pass",
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
                .uri(format!("/api/v1/quality/inspections/{}/results", insp_id))
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

// ============================================================================
// Non-Conformance Report Tests
// ============================================================================

#[tokio::test]
async fn test_create_ncr() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "Scratched surface", "defect", "major").await;
    assert!(ncr["ncr_number"].as_str().unwrap().starts_with("NCR-"));
    assert_eq!(ncr["title"], "Scratched surface");
    assert_eq!(ncr["ncr_type"], "defect");
    assert_eq!(ncr["severity"], "major");
    assert_eq!(ncr["origin"], "inspection");
    assert_eq!(ncr["status"], "open");
}

#[tokio::test]
async fn test_create_ncr_invalid_type() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/ncrs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "title": "Bad NCR",
                        "ncrType": "alien_invasion",
                        "severity": "critical",
                        "origin": "inspection",
                        "detectedDate": "2024-06-15"
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
async fn test_create_ncr_invalid_severity() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/ncrs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "title": "Bad Severity",
                        "ncrType": "defect",
                        "severity": "catastrophic",
                        "origin": "inspection",
                        "detectedDate": "2024-06-15"
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
async fn test_ncr_lifecycle() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "Damaged goods", "damage", "critical").await;
    let id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Investigate
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/investigate", id))
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
    let ncr: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ncr["status"], "under_investigation");

    // Start corrective action phase
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/corrective-action", id))
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
    let ncr: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ncr["status"], "corrective_action");

    // Resolve
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/resolve", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "resolutionDescription": "Supplier will replace damaged items",
                        "resolutionType": "return_to_supplier",
                        "resolvedBy": "QA Manager"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let ncr: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ncr["status"], "resolved");

    // Close
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/close", id))
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
    let ncr: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ncr["status"], "closed");
}

#[tokio::test]
async fn test_ncr_investigate_wrong_status() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "Test NCR", "defect", "minor").await;
    let id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to investigate a second time (already open, ok to investigate once)
    // First investigate - ok
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/investigate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Second investigate - should fail (now under_investigation)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/investigate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_ncrs() {
    let (_state, app) = setup_quality_test().await;
    create_test_ncr(&app, "NCR 1", "defect", "major").await;
    create_test_ncr(&app, "NCR 2", "damage", "critical").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/ncrs")
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

// ============================================================================
// Corrective Action Tests
// ============================================================================

#[tokio::test]
async fn test_create_corrective_action() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "CAPA Test NCR", "defect", "major").await;
    let ncr_id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionType": "corrective",
                        "title": "Fix the root cause",
                        "description": "Investigate and fix the production process",
                        "rootCause": "Incorrect machine calibration",
                        "assignedTo": "John Smith",
                        "dueDate": "2024-07-15",
                        "priority": "high"
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(action["action_number"].as_str().unwrap().starts_with("CAPA-"));
    assert_eq!(action["action_type"], "corrective");
    assert_eq!(action["title"], "Fix the root cause");
    assert_eq!(action["status"], "open");
    assert_eq!(action["priority"], "high");
}

#[tokio::test]
async fn test_corrective_action_lifecycle() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "CAPA Lifecycle NCR", "specification", "major").await;
    let ncr_id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionType": "both",
                        "title": "Complete overhaul",
                        "priority": "critical"
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let action_id = action["id"].as_str().unwrap();
    assert_eq!(action["status"], "open");

    // Start action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/start", action_id))
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(action["status"], "in_progress");

    // Complete action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/complete", action_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "effectivenessRating": 4
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(action["status"], "completed");

    // Verify action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/verify", action_id))
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(action["status"], "verified");
}

#[tokio::test]
async fn test_list_corrective_actions() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "CAPA List NCR", "defect", "minor").await;
    let ncr_id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create 2 actions
    for i in 0..2 {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "actionType": "corrective",
                            "title": format!("Action {}", i + 1),
                            "priority": "medium"
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
                .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
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
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_corrective_action_invalid_effectiveness_rating() {
    let (_state, app) = setup_quality_test().await;
    let ncr = create_test_ncr(&app, "CAPA Rating NCR", "defect", "minor").await;
    let ncr_id = ncr["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create and start action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionType": "corrective",
                        "title": "Rating Test",
                        "priority": "medium"
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let action_id = action["id"].as_str().unwrap();

    // Start it
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/start", action_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try with invalid rating (out of range)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/complete", action_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "effectivenessRating": 10
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Quality Hold Tests
// ============================================================================

#[tokio::test]
async fn test_create_hold() {
    let (_state, app) = setup_quality_test().await;
    let hold = create_test_hold(&app, "Failed incoming inspection", "item").await;
    assert!(hold["hold_number"].as_str().unwrap().starts_with("QH-"));
    assert_eq!(hold["reason"], "Failed incoming inspection");
    assert_eq!(hold["hold_type"], "item");
    assert_eq!(hold["status"], "active");
}

#[tokio::test]
async fn test_create_hold_invalid_type() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/holds")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Bad hold type",
                        "holdType": "country"
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
async fn test_create_hold_empty_reason() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/quality/holds")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "",
                        "holdType": "item"
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
async fn test_release_hold() {
    let (_state, app) = setup_quality_test().await;
    let hold = create_test_hold(&app, "Quality concern", "lot").await;
    let id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/holds/{}/release", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "releaseNotes": "Issue resolved, lot cleared"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let released: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(released["status"], "released");
}

#[tokio::test]
async fn test_release_hold_already_released() {
    let (_state, app) = setup_quality_test().await;
    let hold = create_test_hold(&app, "Double release", "item").await;
    let id = hold["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // First release - ok
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/holds/{}/release", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Second release - should fail
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/holds/{}/release", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_holds() {
    let (_state, app) = setup_quality_test().await;
    create_test_hold(&app, "Hold 1", "item").await;
    create_test_hold(&app, "Hold 2", "lot").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/holds")
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

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_quality_dashboard() {
    let (_state, app) = setup_quality_test().await;

    // Create some data
    create_test_plan(&app, "DASH-PLAN", "Dashboard Plan", "receiving").await;
    create_test_ncr(&app, "Dashboard NCR", "defect", "major").await;
    create_test_hold(&app, "Dashboard Hold", "item").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/dashboard")
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
    assert!(dashboard["total_active_plans"].as_i64().unwrap() >= 1);
    assert!(dashboard["total_open_ncrs"].as_i64().unwrap() >= 1);
    assert!(dashboard["total_active_holds"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_quality_full_lifecycle() {
    let (_state, app) = setup_quality_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create an inspection plan
    let plan = create_test_plan(&app, "LIFE-QP", "Full Lifecycle Plan", "receiving").await;
    let plan_id = plan["id"].as_str().unwrap();
    assert_eq!(plan["plan_code"], "LIFE-QP");

    // 2. Add criteria to the plan
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/plans/{}/criteria", plan_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionNumber": 1,
                        "name": "Weight Check",
                        "characteristic": "weight",
                        "measurementType": "numeric",
                        "targetValue": "100",
                        "lowerSpecLimit": "99",
                        "upperSpecLimit": "101",
                        "unitOfMeasure": "g",
                        "isMandatory": true,
                        "weight": "2.0",
                        "criticality": "critical"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 3. Create an inspection from the plan
    let inspection = create_test_inspection(&app, plan_id, "50").await;
    let insp_id = inspection["id"].as_str().unwrap();
    assert_eq!(inspection["status"], "planned");

    // 4. Start the inspection
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/start", insp_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Record a failing result
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/results", insp_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "criterionName": "Weight Check",
                        "characteristic": "weight",
                        "measurementType": "numeric",
                        "observedValue": "95",
                        "targetValue": "100",
                        "lowerSpecLimit": "99",
                        "upperSpecLimit": "101",
                        "unitOfMeasure": "g",
                        "resultStatus": "fail",
                        "deviation": "-5.0000"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 6. Complete the inspection with fail verdict
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/inspections/{}/complete", insp_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "verdict": "fail",
                        "notes": "Weight out of specification"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 7. Create an NCR from the failed inspection
    let ncr = create_test_ncr(&app, "Weight out of spec - LOT-001", "specification", "major").await;
    let ncr_id = ncr["id"].as_str().unwrap();
    assert_eq!(ncr["status"], "open");

    // 8. Create a quality hold
    let hold = create_test_hold(&app, "Weight out of specification", "lot").await;
    let hold_id = hold["id"].as_str().unwrap();
    assert_eq!(hold["status"], "active");

    // 9. Investigate the NCR
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/investigate", ncr_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 10. Move to corrective action phase
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/corrective-action", ncr_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 11. Create a corrective action
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/actions", ncr_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "actionType": "both",
                        "title": "Recalibrate weighing equipment",
                        "rootCause": "Scale drift due to temperature changes",
                        "assignedTo": "Maintenance Team",
                        "priority": "high"
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
    let action: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let action_id = action["id"].as_str().unwrap();

    // 12. Start and complete the action
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/start", action_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/complete", action_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({ "effectivenessRating": 5 })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 13. Verify the action
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/actions/{}/verify", action_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 14. Resolve the NCR
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/resolve", ncr_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "resolutionDescription": "Equipment recalibrated and verified",
                        "resolutionType": "rework",
                        "resolvedBy": "QA Manager"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 15. Close the NCR
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/ncrs/{}/close", ncr_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 16. Release the hold
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/quality/holds/{}/release", hold_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "releaseNotes": "Quality issue resolved"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 17. Verify dashboard reflects the full lifecycle
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/quality/dashboard")
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
    assert!(dashboard["total_active_plans"].as_i64().unwrap() >= 1);
    assert!(dashboard["total_failed_inspections"].as_i64().unwrap() >= 1);
    // Dashboard is an aggregate - corrective actions may or may not be reflected
    // depending on whether counter maintenance runs synchronously
    assert!(dashboard["total_ncrs"].as_i64().unwrap() >= 1);
}
