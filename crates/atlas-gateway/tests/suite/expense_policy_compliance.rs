//! Expense Policy Compliance E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Expense Policy Compliance:
//! - Policy rule CRUD (create, get, list, delete)
//! - Rule lifecycle (active → inactive)
//! - Compliance audit creation and evaluation
//! - Violation tracking and resolution
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    sqlx::query(include_str!("../../../../migrations/120_expense_policy_compliance.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    // Clean up expense compliance test data
    sqlx::query("DELETE FROM fin_expense_compliance_violations").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM fin_expense_compliance_audits").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM fin_expense_policy_rules").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_rule(
    app: &axum::Router,
    code: &str,
    name: &str,
    rule_type: &str,
    threshold: Option<&str>,
    maximum: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "rule_code": code,
        "name": name,
        "description": "Test expense policy rule",
        "rule_type": rule_type,
        "expense_category": "all",
        "severity": "warning",
        "evaluation_scope": "per_line",
        "threshold_days": 0,
        "requires_receipt": false,
        "requires_justification": false,
    });
    if let Some(t) = threshold {
        payload["threshold_amount"] = json!(t);
    }
    if let Some(m) = maximum {
        payload["maximum_amount"] = json!(m);
    }

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE RULE RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create rule: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_audit(
    app: &axum::Router,
    report_id: &str,
    trigger: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "report_id": report_id,
        "report_number": format!("EXP-{}", Uuid::new_v4().as_simple()),
        "employee_id": "11111111-1111-1111-1111-111111111111",
        "employee_name": "Test Employee",
        "audit_trigger": trigger,
        "audit_date": "2024-06-15",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/audits")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE AUDIT RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create audit");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Policy Rule CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_rule() {
    let (_state, app) = setup_test().await;
    let rule = create_rule(&app, "RULE-001", "Max Amount Rule", "amount_limit", Some("100"), Some("500")).await;

    assert_eq!(rule["ruleCode"], "RULE-001");
    assert_eq!(rule["name"], "Max Amount Rule");
    assert_eq!(rule["ruleType"], "amount_limit");
    assert_eq!(rule["severity"], "warning");
    assert_eq!(rule["status"], "active");
    assert_eq!(rule["isActive"], true);
}

#[tokio::test]
async fn test_get_rule() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-GET", "Get Test Rule", "receipt_required", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/rules/RULE-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["ruleCode"], "RULE-GET");
    assert_eq!(body["name"], "Get Test Rule");
}

#[tokio::test]
async fn test_list_rules() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-LIST1", "List Rule 1", "amount_limit", None, None).await;
    create_rule(&app, "RULE-LIST2", "List Rule 2", "daily_limit", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/rules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_rules_with_status_filter() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-FILTER", "Filter Test", "amount_limit", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/rules?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_rules_with_type_filter() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-TYPE1", "Type Amount", "amount_limit", None, None).await;
    create_rule(&app, "RULE-TYPE2", "Type Receipt", "receipt_required", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/rules?rule_type=amount_limit")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let rules = body["data"].as_array().unwrap();
    assert!(rules.len() >= 1);
    for rule in rules {
        assert_eq!(rule["ruleType"], "amount_limit");
    }
}

// ============================================================================
// Rule Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_deactivate_activate_rule() {
    let (_state, app) = setup_test().await;
    let rule = create_rule(&app, "RULE-DA", "Deactivate Activate", "amount_limit", None, None).await;
    let rule_id: Uuid = rule["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/rules/{}/deactivate", rule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");
    assert_eq!(body["isActive"], false);

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/rules/{}/activate", rule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
    assert_eq!(body["isActive"], true);
}

#[tokio::test]
async fn test_delete_inactive_rule() {
    let (_state, app) = setup_test().await;
    let rule = create_rule(&app, "RULE-DEL", "Delete Test", "amount_limit", None, None).await;
    let rule_id: Uuid = rule["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Must deactivate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/rules/{}/deactivate", rule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/expense-compliance/rules/RULE-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Compliance Audit Tests
// ============================================================================

#[tokio::test]
async fn test_create_audit() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;

    assert!(audit["auditNumber"].as_str().unwrap().starts_with("ECA-"));
    assert_eq!(audit["status"], "pending");
    assert_eq!(audit["auditTrigger"], "automatic");
    assert!(audit["reportNumber"].as_str().unwrap_or_default().contains("EXP-"));
}

#[tokio::test]
async fn test_get_audit() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "manual").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/expense-compliance/audits/{}", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["auditNumber"], audit["auditNumber"]);
}

#[tokio::test]
async fn test_list_audits() {
    let (_state, app) = setup_test().await;
    create_audit(&app, &Uuid::new_v4().to_string(), "automatic").await;
    create_audit(&app, &Uuid::new_v4().to_string(), "manual").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/audits")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_audits_with_status_filter() {
    let (_state, app) = setup_test().await;
    create_audit(&app, &Uuid::new_v4().to_string(), "automatic").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/audits?status=pending")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Compliance Evaluation Tests
// ============================================================================

#[tokio::test]
async fn test_evaluate_compliance_no_rules() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    // With no rules, compliance score should be perfect
    assert_eq!(body["complianceScore"].as_str().unwrap_or_default(), "100.00");
    assert_eq!(body["riskLevel"], "low");
}

#[tokio::test]
async fn test_evaluate_compliance_with_violations() {
    let (_state, app) = setup_test().await;
    // Create a rule that will trigger: threshold of 100
    create_rule(&app, "RULE-EVAL", "Evaluation Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "policy_violation").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    // Should have some score less than 100 because simulated expense (250) exceeds threshold (100)
    let score_str = body["complianceScore"].as_str().unwrap_or("100");
    let score: f64 = score_str.parse().unwrap();
    assert!(score < 100.0, "Score should be < 100 when violations found, got {}", score);
}

#[tokio::test]
async fn test_evaluate_compliance_creates_violations() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-VIOLATIONS", "Violations Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List violations
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/expense-compliance/audits/{}/violations", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let violations = body["data"].as_array().unwrap();
    assert!(!violations.is_empty(), "Should have violations when expense exceeds threshold");
}

// ============================================================================
// Audit Review Tests
// ============================================================================

#[tokio::test]
async fn test_complete_audit_review() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Complete review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/complete", audit_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_notes": "All violations reviewed and justified"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "completed");
}

#[tokio::test]
async fn test_escalate_audit() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "high_amount").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Escalate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/escalate", audit_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_notes": "Requires further investigation"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "escalated");
}

// ============================================================================
// Violation Resolution Tests
// ============================================================================

#[tokio::test]
async fn test_resolve_violation() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-RESOLVE", "Resolve Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get violations
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/expense-compliance/audits/{}/violations", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let violations: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let violation_id = violations["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Resolve the violation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/violations/{}/resolve", violation_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution_status": "justified",
            "justification": "Approved by VP"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["resolutionStatus"], "justified");
}

#[tokio::test]
async fn test_list_open_violations() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-OPEN", "Open Violations Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List open violations
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/violations/open")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_expense_compliance_dashboard() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-DASH", "Dashboard Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate to create some data
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/expense-compliance/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalActiveRules").is_some());
    assert!(body.get("totalAuditsPeriod").is_some());
    assert!(body.get("totalViolationsPeriod").is_some());
    assert!(body.get("totalWarningsPeriod").is_some());
    assert!(body.get("totalBlocksPeriod").is_some());
    assert!(body.get("avgComplianceScore").is_some());
    assert!(body.get("totalFlaggedAmount").is_some());
    assert!(body.get("highRiskAudits").is_some());
    assert!(body.get("openViolations").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_rule_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "",
        "name": "No Code",
        "rule_type": "amount_limit",
        "expense_category": "all",
        "severity": "warning",
        "evaluation_scope": "per_line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_empty_name_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-NO-NAME",
        "name": "",
        "rule_type": "amount_limit",
        "expense_category": "all",
        "severity": "warning",
        "evaluation_scope": "per_line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-BAD-TYPE",
        "name": "Bad Type",
        "rule_type": "invalid_type",
        "expense_category": "all",
        "severity": "warning",
        "evaluation_scope": "per_line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_severity_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-BAD-SEV",
        "name": "Bad Severity",
        "rule_type": "amount_limit",
        "expense_category": "all",
        "severity": "catastrophic",
        "evaluation_scope": "per_line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_category_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-BAD-CAT",
        "name": "Bad Category",
        "rule_type": "amount_limit",
        "expense_category": "nonexistent",
        "severity": "warning",
        "evaluation_scope": "per_line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_audit_invalid_trigger_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "report_id": Uuid::new_v4().to_string(),
        "audit_trigger": "invalid_trigger",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/expense-compliance/audits")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_active_rule_fails() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-DEL-ACT", "Delete Active Test", "amount_limit", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/expense-compliance/rules/RULE-DEL-ACT")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_evaluate_non_pending_audit_fails() {
    let (_state, app) = setup_test().await;
    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate once (changes status from pending)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Complete the review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/complete", audit_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"review_notes": "Done"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to evaluate again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_resolve_already_resolved_violation_fails() {
    let (_state, app) = setup_test().await;
    create_rule(&app, "RULE-RESOLVED", "Already Resolved Rule", "amount_limit", Some("100"), None).await;

    let report_id = Uuid::new_v4().to_string();
    let audit = create_audit(&app, &report_id, "automatic").await;
    let audit_id: Uuid = audit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Evaluate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/audits/{}/evaluate", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get violations
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/expense-compliance/audits/{}/violations", audit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let violations: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let violation_id = violations["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Resolve once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/violations/{}/resolve", violation_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution_status": "justified",
            "justification": "OK"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to resolve again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/expense-compliance/violations/{}/resolve", violation_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution_status": "upheld"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
