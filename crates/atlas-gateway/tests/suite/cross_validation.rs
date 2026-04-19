//! Cross-Validation Rules (CVR) E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Cross-Validation Rules:
//! - Rule CRUD and lifecycle
//! - Rule lines with pattern matching
//! - Combination validation (deny and allow rules)
//! - Dashboard summary
//! - Error cases and edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_cvr_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_deny_rule() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_CASH_MARKETING",
            "name": "Block Cash with Marketing",
            "description": "Cash accounts cannot be used by Marketing department",
            "rule_type": "deny",
            "error_message": "Cash accounts (1000) cannot be combined with Marketing department",
            "priority": 10,
            "segment_names": ["company", "department", "account"],
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["code"], "CVR_CASH_MARKETING");
    assert_eq!(rule["name"], "Block Cash with Marketing");
    assert_eq!(rule["ruleType"], "deny");
    assert_eq!(rule["isEnabled"], true);
    assert_eq!(rule["priority"], 10);
    assert_eq!(rule["segmentNames"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_create_allow_rule() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_ALLOW_EXEC",
            "name": "Allow Executive Overrides",
            "rule_type": "allow",
            "error_message": "N/A",
            "priority": 5,
            "segment_names": ["company", "department", "account"],
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["ruleType"], "allow");
    assert_eq!(rule["priority"], 5);
}

#[tokio::test]
async fn test_list_rules() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two rules
    for code in &["CVR_RULE_1", "CVR_RULE_2"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/cross-validation/rules")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Rule {}", code),
                "rule_type": "deny",
                "error_message": format!("Error for {}", code),
                "segment_names": ["company", "account"],
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/cross-validation/rules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_get_rule() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create first
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_TEST",
            "name": "Test Rule",
            "rule_type": "deny",
            "error_message": "Blocked",
            "segment_names": ["segment1"],
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Get it
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/cross-validation/rules/CVR_TEST")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["code"], "CVR_TEST");
}

#[tokio::test]
async fn test_enable_disable_rule() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create rule
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_TOGGLE",
            "name": "Toggle Test",
            "rule_type": "deny",
            "error_message": "Toggle error",
            "segment_names": ["seg1"],
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let rule_id = rule["id"].as_str().unwrap();

    // Disable
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/cross-validation/rules/{}/disable", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["isEnabled"], false);

    // Re-enable
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/cross-validation/rules/{}/enable", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["isEnabled"], true);
}

#[tokio::test]
async fn test_delete_rule() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_DEL",
            "name": "To Delete",
            "rule_type": "deny",
            "error_message": "Delete me",
            "segment_names": ["seg"],
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Delete
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri("/api/v1/cross-validation/rules/CVR_DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/cross-validation/rules/CVR_DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_duplicate_rule_code_rejected() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let payload = serde_json::to_string(&json!({
        "code": "CVR_DUP",
        "name": "First",
        "rule_type": "deny",
        "error_message": "Error",
        "segment_names": ["seg"],
    })).unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(payload.clone())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(payload)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule Line Tests
// ═══════════════════════════════════════════════════════════════════════════════

async fn setup_rule_with_lines(app: &axum::Router) {
    let (k, v) = auth_header(&admin_claims());

    // Create rule with 3 segments
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_CASH_DEPT",
            "name": "Block Cash with certain departments",
            "rule_type": "deny",
            "error_message": "Cash account 1000 cannot be used with department {department}",
            "priority": 10,
            "segment_names": ["company", "department", "account"],
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add "from" line: company=1000, any department, any account
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "from",
            "patterns": ["1000", "%", "%"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add "to" line: any company, department=MARKETING, account=5000
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "to",
            "patterns": ["%", "MARKETING", "5000"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
}

#[tokio::test]
async fn test_create_rule_lines() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // List lines
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let lines: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(lines.len(), 2);

    // Verify from line
    let from_line = lines.iter().find(|l| l["lineType"] == "from").unwrap();
    assert_eq!(from_line["patterns"].as_array().unwrap().len(), 3);
    assert_eq!(from_line["patterns"][0], "1000");
    assert_eq!(from_line["patterns"][1], "%");
    assert_eq!(from_line["patterns"][2], "%");

    // Verify to line
    let to_line = lines.iter().find(|l| l["lineType"] == "to").unwrap();
    assert_eq!(to_line["patterns"][0], "%");
    assert_eq!(to_line["patterns"][1], "MARKETING");
    assert_eq!(to_line["patterns"][2], "5000");
}

#[tokio::test]
async fn test_delete_rule_line() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // Get lines
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let lines: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Delete first line
    let line_id = lines[0]["id"].as_str().unwrap();
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/cross-validation/lines/{}", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify one less
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let lines: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(lines.len(), 1);
}

#[tokio::test]
async fn test_rule_line_wrong_pattern_count_rejected() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;
    let (k, v) = auth_header(&admin_claims());

    // Try to add a line with wrong number of patterns
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "from",
            "patterns": ["1000"],  // only 1, rule has 3 segments
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_validate_combination_blocked() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // This combination should be blocked:
    // from matches: company=1000, dept=%, account=% ✓
    // to matches: company=%, dept=MARKETING, account=5000 ✓
    // Both match → deny rule triggered
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "MARKETING", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], false);
    assert!(result["violatedRules"].as_array().unwrap().len() > 0);
    assert!(result["errorMessages"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_validate_combination_allowed_different_company() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // from does NOT match: company=2000 ≠ 1000
    // So the deny rule doesn't fire
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["2000", "MARKETING", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);
}

#[tokio::test]
async fn test_validate_combination_allowed_different_department() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // to does NOT match: dept=ENGINEERING ≠ MARKETING
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "ENGINEERING", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);
}

#[tokio::test]
async fn test_validate_no_rules_all_valid() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // No rules → everything is valid
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "MARKETING", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);
}

#[tokio::test]
async fn test_validate_disabled_rule_not_checked() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;
    let (k, v) = auth_header(&admin_claims());

    // Get rule and disable it
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let rule_id = rule["id"].as_str().unwrap();

    // Disable
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/cross-validation/rules/{}/disable", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now the previously blocked combination should be valid
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "MARKETING", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dashboard_with_data() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(summary["totalRules"], 1);
    assert_eq!(summary["enabledRules"], 1);
    assert_eq!(summary["denyRules"], 1);
    assert_eq!(summary["allowRules"], 0);
    assert_eq!(summary["totalLines"], 2);
}

#[tokio::test]
async fn test_dashboard_empty() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(summary["totalRules"], 0);
    assert_eq!(summary["enabledRules"], 0);
    assert_eq!(summary["denyRules"], 0);
    assert_eq!(summary["allowRules"], 0);
    assert_eq!(summary["totalLines"], 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_rule_invalid_type_rejected() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_BAD",
            "name": "Bad Type",
            "rule_type": "invalid",
            "error_message": "Error",
            "segment_names": ["seg"],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_empty_code_rejected() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "No Code",
            "rule_type": "deny",
            "error_message": "Error",
            "segment_names": ["seg"],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_empty_segments_rejected() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_NOSEG",
            "name": "No Segments",
            "rule_type": "deny",
            "error_message": "Error",
            "segment_names": [],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_line_invalid_type_rejected() {
    let (_state, app) = setup_cvr_test().await;
    setup_rule_with_lines(&app).await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASH_DEPT/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "invalid",
            "patterns": ["%", "%", "%"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_line_nonexistent_rule_rejected() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/NO_SUCH_RULE/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "from",
            "patterns": ["%"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Full Workflow Integration Test
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_full_cvr_workflow() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Step 1: Create a deny rule
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_REVENUE_RANDD",
            "name": "Block Revenue with R&D",
            "description": "Revenue accounts (4000-4999) cannot be used by R&D department",
            "rule_type": "deny",
            "error_message": "Revenue accounts cannot be combined with R&D department",
            "priority": 10,
            "segment_names": ["company", "department", "account"],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 2: Add from pattern: any company, any department, account in 4000-4999 range
    // We use "4000" as exact match for simplicity
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_REVENUE_RANDD/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "from",
            "patterns": ["%", "%", "4000"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 3: Add to pattern: any company, R&D department, any account
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_REVENUE_RANDD/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "to",
            "patterns": ["%", "R_AND_D", "%"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 4: Validate blocked combination (account=4000, dept=R_AND_D)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "R_AND_D", "4000"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], false);
    assert!(result["violatedRules"].as_array().unwrap().contains(&json!("CVR_REVENUE_RANDD")));

    // Step 5: Validate allowed combination (account=4000, dept=MARKETING)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "MARKETING", "4000"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);

    // Step 6: Validate allowed combination (account=5000, dept=R_AND_D)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["1000", "R_AND_D", "5000"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], true);

    // Step 7: Check dashboard
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/cross-validation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary["totalRules"], 1);
    assert_eq!(summary["enabledRules"], 1);
    assert_eq!(summary["totalLines"], 2);
}

#[tokio::test]
async fn test_case_insensitive_pattern_matching() {
    let (_state, app) = setup_cvr_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create rule with lowercase department
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CVR_CASE",
            "name": "Case Test",
            "rule_type": "deny",
            "error_message": "Case mismatch",
            "segment_names": ["dept", "account"],
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add lines with lowercase
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASE/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "from",
            "patterns": ["marketing", "%"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/cross-validation/rules/CVR_CASE/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "to",
            "patterns": ["%", "1000"],
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Validate with uppercase - should still match (case-insensitive)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/cross-validation/validate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_values": ["MARKETING", "1000"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["isValid"], false); // Blocked due to case-insensitive match
}
