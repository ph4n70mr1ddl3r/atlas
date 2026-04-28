//! Risk Management & Internal Controls E2E Tests
//!
//! Tests for Oracle Fusion GRC-inspired risk management:
//! - Risk category CRUD
//! - Risk register CRUD, assessment & scoring
//! - Control registry CRUD & effectiveness
//! - Risk-control mappings
//! - Control testing lifecycle
//! - Issue & remediation lifecycle
//! - Risk dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_risk_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_category(
    app: &axum::Router, code: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for category but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_risk(
    app: &axum::Router, risk_number: &str, title: &str,
    risk_source: &str, likelihood: i32, impact: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/risks")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskNumber": risk_number,
            "title": title,
            "riskSource": risk_source,
            "likelihood": likelihood,
            "impact": impact,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for risk but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_control(
    app: &axum::Router, control_number: &str, title: &str,
    control_type: &str, control_nature: &str, frequency: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/controls")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "controlNumber": control_number,
            "title": title,
            "controlType": control_type,
            "controlNature": control_nature,
            "frequency": frequency,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for control but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Category Tests
// ============================================================================

#[tokio::test]
async fn test_create_category() {
    let (_state, app) = setup_risk_test().await;
    let cat = create_test_category(&app, "FIN", "Financial Risks").await;
    assert_eq!(cat["code"], "FIN");
    assert_eq!(cat["name"], "Financial Risks");
    assert_eq!(cat["isActive"], true);
}

#[tokio::test]
async fn test_create_category_duplicate_conflict() {
    let (_state, app) = setup_risk_test().await;
    create_test_category(&app, "DUP", "First").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_categories() {
    let (_state, app) = setup_risk_test().await;
    create_test_category(&app, "OPS", "Operational").await;
    create_test_category(&app, "STR", "Strategic").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/risk/categories")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_category() {
    let (_state, app) = setup_risk_test().await;
    create_test_category(&app, "DEL", "Delete Me").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/risk/categories/code/DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Risk Register Tests
// ============================================================================

#[tokio::test]
async fn test_create_risk() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "R-001", "Data Breach", "technology", 4, 5).await;
    assert_eq!(risk["riskNumber"], "R-001");
    assert_eq!(risk["title"], "Data Breach");
    assert_eq!(risk["riskSource"], "technology");
    assert_eq!(risk["likelihood"], 4);
    assert_eq!(risk["impact"], 5);
    assert_eq!(risk["riskScore"], 20);
    assert_eq!(risk["riskLevel"], "critical");
    assert_eq!(risk["status"], "identified");
}

#[tokio::test]
async fn test_create_risk_medium_level() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "R-002", "Supplier Delay", "operational", 3, 2).await;
    assert_eq!(risk["riskScore"], 6);
    assert_eq!(risk["riskLevel"], "medium");
}

#[tokio::test]
async fn test_create_risk_high_level() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "R-003", "Revenue Loss", "financial", 3, 4).await;
    assert_eq!(risk["riskScore"], 12);
    assert_eq!(risk["riskLevel"], "high");
}

#[tokio::test]
async fn test_create_risk_low_level() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "R-004", "Minor Delay", "operational", 1, 2).await;
    assert_eq!(risk["riskScore"], 2);
    assert_eq!(risk["riskLevel"], "low");
}

#[tokio::test]
async fn test_create_risk_duplicate_conflict() {
    let (_state, app) = setup_risk_test().await;
    create_test_risk(&app, "DUP-R", "First", "operational", 3, 3).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/risks")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskNumber": "DUP-R", "title": "Duplicate", "riskSource": "operational",
            "likelihood": 3, "impact": 3
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_risk_invalid_likelihood() {
    let (_state, app) = setup_risk_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/risks")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskNumber": "BAD", "title": "Bad", "riskSource": "operational",
            "likelihood": 6, "impact": 3
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_risk() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "GET-R", "Get Risk", "financial", 2, 3).await;
    let id = risk["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/risk/risks/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["riskNumber"], "GET-R");
}

#[tokio::test]
async fn test_list_risks() {
    let (_state, app) = setup_risk_test().await;
    create_test_risk(&app, "LIST-1", "Risk One", "operational", 3, 3).await;
    create_test_risk(&app, "LIST-2", "Risk Two", "financial", 4, 4).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/risk/risks")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_assess_risk() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "ASS-R", "Assess Risk", "operational", 3, 3).await;
    let id = risk["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/risks/id/{}/assess", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "likelihood": 2, "impact": 2,
            "residualLikelihood": 1, "residualImpact": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["likelihood"], 2);
    assert_eq!(updated["impact"], 2);
    assert_eq!(updated["riskScore"], 4);
    assert_eq!(updated["riskLevel"], "low");
    assert_eq!(updated["status"], "assessed");
}

#[tokio::test]
async fn test_update_risk_status() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "STAT-R", "Status Risk", "compliance", 3, 3).await;
    let id = risk["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/risks/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "mitigated"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "mitigated");
}

#[tokio::test]
async fn test_delete_risk() {
    let (_state, app) = setup_risk_test().await;
    create_test_risk(&app, "DEL-R", "Delete Risk", "operational", 2, 2).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/risk/risks/number/DEL-R")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Control Registry Tests
// ============================================================================

#[tokio::test]
async fn test_create_control() {
    let (_state, app) = setup_risk_test().await;
    let ctrl = create_test_control(&app, "CTL-001", "Segregation of Duties", "preventive", "automated", "daily").await;
    assert_eq!(ctrl["controlNumber"], "CTL-001");
    assert_eq!(ctrl["title"], "Segregation of Duties");
    assert_eq!(ctrl["controlType"], "preventive");
    assert_eq!(ctrl["controlNature"], "automated");
    assert_eq!(ctrl["frequency"], "daily");
    assert_eq!(ctrl["effectiveness"], "not_tested");
    assert_eq!(ctrl["status"], "draft");
}

#[tokio::test]
async fn test_create_control_duplicate_conflict() {
    let (_state, app) = setup_risk_test().await;
    create_test_control(&app, "DUP-C", "First", "preventive", "manual", "monthly").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/controls")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "controlNumber": "DUP-C", "title": "Duplicate",
            "controlType": "preventive", "controlNature": "manual", "frequency": "monthly"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_control_status_and_effectiveness() {
    let (_state, app) = setup_risk_test().await;
    let ctrl = create_test_control(&app, "EFF-C", "Effective Control", "detective", "manual", "quarterly").await;
    let id = ctrl["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/controls/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Mark effective
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/controls/id/{}/effectiveness", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"effectiveness": "effective"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["effectiveness"], "effective");
    assert_eq!(updated["status"], "active");
}

// ============================================================================
// Risk-Control Mapping Tests
// ============================================================================

#[tokio::test]
async fn test_create_mapping() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "MAP-R", "Mapping Risk", "operational", 4, 4).await;
    let ctrl = create_test_control(&app, "MAP-C", "Mapping Control", "preventive", "manual", "monthly").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/mappings")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskId": risk["id"], "controlId": ctrl["id"],
            "mitigationEffectiveness": "partial"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let mapping: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(mapping["mitigationEffectiveness"], "partial");
    assert_eq!(mapping["status"], "active");
}

#[tokio::test]
async fn test_list_risk_mappings() {
    let (_state, app) = setup_risk_test().await;
    let risk = create_test_risk(&app, "LMAP-R", "List Mappings Risk", "financial", 3, 3).await;
    let ctrl = create_test_control(&app, "LMAP-C", "List Mappings Control", "detective", "automated", "daily").await;
    let risk_id = risk["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/mappings")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskId": risk["id"], "controlId": ctrl["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/risk/risks/id/{}/mappings", risk_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Control Test Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_complete_control_test() {
    let (_state, app) = setup_risk_test().await;
    let ctrl = create_test_control(&app, "TEST-C", "Test Control", "preventive", "manual", "monthly").await;
    let ctrl_id = ctrl["id"].as_str().unwrap();

    // Create test
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/tests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "controlId": ctrl_id,
            "testNumber": "T-001",
            "testPlan": "Verify all approvals are documented",
            "testPeriodStart": "2024-01-01",
            "testPeriodEnd": "2024-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let test: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(test["testNumber"], "T-001");
    assert_eq!(test["status"], "planned");
    let test_id = test["id"].as_str().unwrap();

    // Start test
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/start", test_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Complete test with pass
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/complete", test_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "result": "pass",
            "findings": "All approvals properly documented",
            "sampleSize": 25,
            "sampleExceptions": 0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["result"], "pass");
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["sampleSize"], 25);
}

#[tokio::test]
async fn test_control_test_fail_with_deficiency() {
    let (_state, app) = setup_risk_test().await;
    let ctrl = create_test_control(&app, "FAIL-C", "Fail Control", "detective", "manual", "quarterly").await;

    let (k, v) = auth_header(&admin_claims());
    // Create and start test
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/tests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "controlId": ctrl["id"], "testNumber": "T-FAIL",
            "testPlan": "Check segregation of duties",
            "testPeriodStart": "2024-01-01", "testPeriodEnd": "2024-06-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let test: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let test_id = test["id"].as_str().unwrap();

    // Start
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/start", test_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Complete with failure
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/complete", test_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "result": "fail",
            "findings": "3 exceptions found in approval chain",
            "deficiencySeverity": "significant",
            "sampleSize": 30,
            "sampleExceptions": 3
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["result"], "fail");
    assert_eq!(completed["deficiencySeverity"], "significant");
}

// ============================================================================
// Issue & Remediation Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_resolve_issue() {
    let (_state, app) = setup_risk_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create issue
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/issues")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "issueNumber": "ISS-001",
            "title": "Missing approval documentation",
            "description": "Three purchase orders found without proper approval signatures",
            "source": "control_test",
            "severity": "high",
            "priority": "urgent",
            "remediationPlan": "Implement automated approval workflow",
            "remediationDueDate": "2024-06-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let issue: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(issue["issueNumber"], "ISS-001");
    assert_eq!(issue["severity"], "high");
    assert_eq!(issue["status"], "open");
    let issue_id = issue["id"].as_str().unwrap();

    // Start remediation
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/issues/id/{}/status", issue_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "remediation_in_progress"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Resolve
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/issues/id/{}/resolve", issue_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rootCause": "Manual process bypass",
            "correctiveActions": "Implemented automated approval workflow"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let resolved: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["rootCause"], "Manual process bypass");
}

#[tokio::test]
async fn test_list_issues_filtered() {
    let (_state, app) = setup_risk_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two issues
    for (num, sev) in [("ISS-F1", "critical"), ("ISS-F2", "low")] {
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/issues")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "issueNumber": num, "title": num, "description": "Test",
                "severity": sev
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Filter by severity
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/risk/issues?severity=critical")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let issues = list["data"].as_array().unwrap();
    assert!(issues.iter().all(|i| i["severity"] == "critical"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_risk_dashboard() {
    let (_state, app) = setup_risk_test().await;

    // Create some data
    create_test_risk(&app, "DASH-R1", "Critical Risk", "technology", 5, 5).await;
    create_test_risk(&app, "DASH-R2", "Medium Risk", "operational", 3, 2).await;
    create_test_control(&app, "DASH-C1", "Dashboard Control", "preventive", "manual", "monthly").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/risk/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalRisks"].as_i64().unwrap() >= 2);
    assert!(dashboard["totalControls"].as_i64().unwrap() >= 1);
    assert!(dashboard["openRisks"].as_i64().unwrap() >= 2);
    assert!(dashboard["criticalRisks"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_risk_management_full_lifecycle() {
    let (_state, app) = setup_risk_test().await;

    // 1. Create risk category
    let _cat = create_test_category(&app, "CYBER", "Cybersecurity").await;

    // 2. Create risk
    let risk = create_test_risk(&app, "LIFE-R", "Ransomware Attack", "technology", 4, 5).await;
    let risk_id = risk["id"].as_str().unwrap();
    assert_eq!(risk["riskLevel"], "critical");

    // 3. Create control
    let ctrl = create_test_control(&app, "LIFE-C", "Endpoint Detection & Response", "detective", "automated", "daily").await;
    let ctrl_id = ctrl["id"].as_str().unwrap();

    // 4. Activate control
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/controls/id/{}/status", ctrl_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // 5. Map risk to control
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/mappings")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskId": risk_id, "controlId": ctrl_id,
            "mitigationEffectiveness": "partial"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 6. Create and run control test
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/tests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "controlId": ctrl_id, "testNumber": "LIFE-T",
            "testPlan": "Verify EDR detects and isolates threats",
            "testPeriodStart": "2024-01-01", "testPeriodEnd": "2024-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let test: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let test_id = test["id"].as_str().unwrap();

    // Start & complete
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/start", test_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/tests/id/{}/complete", test_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "result": "fail", "deficiencySeverity": "significant",
            "findings": "2 endpoints without EDR agent",
            "sampleSize": 100, "sampleExceptions": 2
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 7. Create issue from failed test
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/risk/issues")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "issueNumber": "LIFE-I", "title": "Missing EDR agents",
            "description": "2 endpoints lack EDR coverage",
            "source": "control_test", "controlTestId": test_id,
            "severity": "high", "priority": "urgent",
            "remediationPlan": "Deploy EDR to all endpoints"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 8. Assess risk (re-score with residual)
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/risk/risks/id/{}/assess", risk_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "likelihood": 4, "impact": 5,
            "residualLikelihood": 2, "residualImpact": 3
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 9. Verify dashboard
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/risk/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalRisks"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalControls"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalTests"].as_i64().unwrap() >= 1);
    assert!(dashboard["failedTests"].as_i64().unwrap() >= 1);
    assert!(dashboard["openIssues"].as_i64().unwrap() >= 1);
}
