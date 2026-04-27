//! Goal Management E2E Tests
//!
//! Tests for Oracle Fusion HCM Goal Management:
//! - Library category and template CRUD
//! - Goal plan lifecycle (draft → active → closed)
//! - Goal CRUD with cascading hierarchy
//! - Goal progress updates
//! - Goal alignment management
//! - Goal notes (comments, feedback, check-ins)
//! - Dashboard summary
//! - Validation edge cases
//! - Full lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_category(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/library/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "displayOrder": 0
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for category but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_template(app: &axum::Router, code: &str, name: &str, goal_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/library/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "goalType": goal_type,
            "successCriteria": "Achieve target", "targetMetric": "Revenue",
            "targetValue": "1000000", "suggestedWeight": "25.0"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for template but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_plan(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "planType": "performance",
            "reviewPeriodStart": "2025-01-01", "reviewPeriodEnd": "2025-12-31",
            "goalCreationDeadline": "2025-03-31",
            "allowSelfGoals": true, "allowTeamGoals": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for plan but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_goal(app: &axum::Router, name: &str, goal_type: &str, plan_id: Option<&str>) -> serde_json::Value {
    let mut body = json!({
        "name": name,
        "goalType": goal_type,
        "priority": "high",
        "weight": "30.0",
        "targetValue": "500000",
        "startDate": "2025-01-01",
        "targetDate": "2025-06-30"
    });
    if let Some(pid) = plan_id {
        body["planId"] = json!(pid);
    }
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for goal but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Library Category Tests
// ============================================================================

#[tokio::test]
async fn test_create_library_category() {
    let (_state, app) = setup_test().await;
    let cat = create_test_category(&app, "LEADERSHIP", "Leadership Goals").await;
    assert_eq!(cat["code"], "LEADERSHIP");
    assert_eq!(cat["name"], "Leadership Goals");
    assert_eq!(cat["status"], "active");
}

#[tokio::test]
async fn test_create_category_duplicate_code() {
    let (_state, app) = setup_test().await;
    create_test_category(&app, "DUP_CAT", "First").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/library/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP_CAT", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_library_categories() {
    let (_state, app) = setup_test().await;
    create_test_category(&app, "CAT_A", "Category A").await;
    create_test_category(&app, "CAT_B", "Category B").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/library/categories")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_library_category() {
    let (_state, app) = setup_test().await;
    create_test_category(&app, "DEL_CAT", "Delete Me").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/library/categories/DEL_CAT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Library Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_library_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "REV_GROWTH", "Revenue Growth Target", "organization").await;
    assert_eq!(tmpl["code"], "REV_GROWTH");
    assert_eq!(tmpl["name"], "Revenue Growth Target");
    assert_eq!(tmpl["goalType"], "organization");
    assert_eq!(tmpl["targetMetric"], "Revenue");
}

#[tokio::test]
async fn test_create_template_invalid_goal_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/library/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_TYPE", "name": "Bad", "goalType": "invalid"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_library_templates() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "TMPL_1", "Template 1", "individual").await;
    create_test_template(&app, "TMPL_2", "Template 2", "team").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/library/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_templates_filtered_by_type() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "FILT_I", "Individual", "individual").await;
    create_test_template(&app, "FILT_T", "Team", "team").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/library/templates?goal_type=individual")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let templates = list["data"].as_array().unwrap();
    assert!(templates.iter().all(|t| t["goalType"] == "individual"));
}

#[tokio::test]
async fn test_delete_library_template() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "DEL_TMPL", "Delete Me", "individual").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/library/templates/DEL_TMPL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Goal Plan Tests
// ============================================================================

#[tokio::test]
async fn test_create_goal_plan() {
    let (_state, app) = setup_test().await;
    let plan = create_test_plan(&app, "FY25_PERF", "FY2025 Performance Review").await;
    assert_eq!(plan["code"], "FY25_PERF");
    assert_eq!(plan["name"], "FY2025 Performance Review");
    assert_eq!(plan["status"], "draft");
    assert_eq!(plan["planType"], "performance");
    assert_eq!(plan["allowSelfGoals"], true);
}

#[tokio::test]
async fn test_create_plan_duplicate_code() {
    let (_state, app) = setup_test().await;
    create_test_plan(&app, "DUP_PLAN", "First").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP_PLAN", "name": "Duplicate",
            "reviewPeriodStart": "2025-01-01", "reviewPeriodEnd": "2025-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_plan_invalid_dates() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_DATES", "name": "Bad Dates",
            "reviewPeriodStart": "2025-12-31", "reviewPeriodEnd": "2025-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_goal_plan() {
    let (_state, app) = setup_test().await;
    let plan = create_test_plan(&app, "GET_PLAN", "Get Plan").await;
    let id = plan["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/goal-management/plans/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["code"], "GET_PLAN");
}

#[tokio::test]
async fn test_list_goal_plans() {
    let (_state, app) = setup_test().await;
    create_test_plan(&app, "LIST_P1", "Plan 1").await;
    create_test_plan(&app, "LIST_P2", "Plan 2").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/plans")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_plan_status_transition() {
    let (_state, app) = setup_test().await;
    let plan = create_test_plan(&app, "STATUS_PLAN", "Status Plan").await;
    let id = plan["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Draft → Active
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/plans/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "active");

    // Active → Closed
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/plans/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "closed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_plan_invalid_status_transition() {
    let (_state, app) = setup_test().await;
    let plan = create_test_plan(&app, "BAD_TRANS", "Bad Transition").await;
    let id = plan["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Draft → Closed (invalid)
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/plans/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "closed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_goal_plan() {
    let (_state, app) = setup_test().await;
    create_test_plan(&app, "DEL_PLAN", "Delete Plan").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/plans/code/DEL_PLAN")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Goal Tests
// ============================================================================

#[tokio::test]
async fn test_create_goal() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Increase Revenue by 20%", "individual", None).await;
    assert_eq!(goal["name"], "Increase Revenue by 20%");
    assert_eq!(goal["goalType"], "individual");
    assert_eq!(goal["status"], "not_started");
    assert_eq!(goal["priority"], "high");
}

#[tokio::test]
async fn test_create_goal_with_plan() {
    let (_state, app) = setup_test().await;
    // Create and activate a plan
    let plan = create_test_plan(&app, "GOAL_PLAN", "Goal Plan").await;
    let plan_id = plan["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/plans/id/{}/status", plan_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let goal = create_test_goal(&app, "Plan Goal", "individual", Some(plan_id)).await;
    assert_eq!(goal["planId"], plan_id);
}

#[tokio::test]
async fn test_create_goal_invalid_plan_status() {
    let (_state, app) = setup_test().await;
    // Create plan but leave as draft
    let plan = create_test_plan(&app, "DRAFT_PLAN", "Draft Plan").await;
    let plan_id = plan["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Goal", "goalType": "individual",
            "planId": plan_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_goal_invalid_priority() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Priority", "goalType": "individual", "priority": "urgent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_cascading_goals() {
    let (_state, app) = setup_test().await;
    // Org goal
    let org_goal = create_test_goal(&app, "Company Revenue Target", "organization", None).await;
    let parent_id = org_goal["id"].as_str().unwrap();

    // Team goal cascaded from org
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Sales Team Revenue", "goalType": "team",
            "parentGoalId": parent_id, "weight": "40.0"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let team_goal: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(team_goal["parentGoalId"], parent_id);
}

#[tokio::test]
async fn test_get_goal() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Get This Goal", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/goal-management/goals/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["name"], "Get This Goal");
}

#[tokio::test]
async fn test_list_goals() {
    let (_state, app) = setup_test().await;
    create_test_goal(&app, "Goal A", "individual", None).await;
    create_test_goal(&app, "Goal B", "team", None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/goals")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_goals_filtered_by_type() {
    let (_state, app) = setup_test().await;
    create_test_goal(&app, "Filtered Ind", "individual", None).await;
    create_test_goal(&app, "Filtered Team", "team", None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/goals?goal_type=team")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let goals = list["data"].as_array().unwrap();
    assert!(goals.iter().all(|g| g["goalType"] == "team"));
}

#[tokio::test]
async fn test_update_goal_progress() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Progress Goal", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualValue": "250000", "progressPct": "50.0", "status": "in_progress"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "in_progress");
}

#[tokio::test]
async fn test_update_goal_complete() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Complete Me", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "progressPct": "100.0", "status": "completed"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "completed");
    assert!(updated["completedDate"].is_string());
}

#[tokio::test]
async fn test_update_progress_invalid_pct() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Bad Pct", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "progressPct": "150.0"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_goal() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Delete Me", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/goals/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Goal Alignment Tests
// ============================================================================

#[tokio::test]
async fn test_create_goal_alignment() {
    let (_state, app) = setup_test().await;
    let g1 = create_test_goal(&app, "Align Source", "individual", None).await;
    let g2 = create_test_goal(&app, "Align Target", "team", None).await;
    let g1_id = g1["id"].as_str().unwrap();
    let g2_id = g2["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": g1_id, "alignedToGoalId": g2_id,
            "alignmentType": "supports", "description": "Supports team objective"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let alignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(alignment["alignmentType"], "supports");
    assert_eq!(alignment["sourceGoalId"], g1_id);
    assert_eq!(alignment["alignedToGoalId"], g2_id);
}

#[tokio::test]
async fn test_alignment_self_reference_fails() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Self Ref", "individual", None).await;
    let id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": id, "alignedToGoalId": id, "alignmentType": "supports"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_alignment_invalid_type() {
    let (_state, app) = setup_test().await;
    let g1 = create_test_goal(&app, "Inv Type 1", "individual", None).await;
    let g2 = create_test_goal(&app, "Inv Type 2", "individual", None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": g1["id"], "alignedToGoalId": g2["id"],
            "alignmentType": "blocks"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_goal_alignments() {
    let (_state, app) = setup_test().await;
    let g1 = create_test_goal(&app, "List Align 1", "individual", None).await;
    let g2 = create_test_goal(&app, "List Align 2", "team", None).await;
    let g1_id = g1["id"].as_str().unwrap();
    let g2_id = g2["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": g1_id, "alignedToGoalId": g2_id,
            "alignmentType": "cascaded_from"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/goal-management/alignments/goal/{}", g1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_delete_goal_alignment() {
    let (_state, app) = setup_test().await;
    let g1 = create_test_goal(&app, "Del Align 1", "individual", None).await;
    let g2 = create_test_goal(&app, "Del Align 2", "individual", None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": g1["id"], "alignedToGoalId": g2["id"],
            "alignmentType": "supports"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let alignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let align_id = alignment["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/alignments/id/{}", align_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Goal Notes Tests
// ============================================================================

#[tokio::test]
async fn test_create_goal_note() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Note Goal", "individual", None).await;
    let goal_id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "noteType": "check_in", "content": "On track for Q1 target",
            "visibility": "manager"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let note: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(note["noteType"], "check_in");
    assert_eq!(note["content"], "On track for Q1 target");
    assert_eq!(note["visibility"], "manager");
}

#[tokio::test]
async fn test_create_note_empty_content_fails() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Empty Note", "individual", None).await;
    let goal_id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "content": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_note_invalid_type() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Bad Note Type", "individual", None).await;
    let goal_id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "content": "Hello", "noteType": "gossip"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_goal_notes() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Multi Notes", "individual", None).await;
    let goal_id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add two notes
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "content": "First note", "noteType": "comment"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "content": "Second note", "noteType": "feedback", "visibility": "public"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_goal_note() {
    let (_state, app) = setup_test().await;
    let goal = create_test_goal(&app, "Del Note Goal", "individual", None).await;
    let goal_id = goal["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "content": "Delete me"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let note: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let note_id = note["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/notes/id/{}", note_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Summary Test
// ============================================================================

#[tokio::test]
async fn test_goal_management_summary() {
    let (_state, app) = setup_test().await;

    // Create goals in various states
    let g1 = create_test_goal(&app, "Summary Goal 1", "individual", None).await;
    let g2 = create_test_goal(&app, "Summary Goal 2", "team", None).await;

    // Update one to in_progress
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", g1["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "progressPct": "50.0", "status": "in_progress"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Complete one
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", g2["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "progressPct": "100.0", "status": "completed"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalGoals"].as_i64().unwrap() >= 2);
    assert!(summary["goalsCompleted"].as_i64().unwrap() >= 1);
    assert!(summary["goalsInProgress"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_goal_management_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create library category
    let cat = create_test_category(&app, "LIFE_CAT", "Lifecycle Category").await;
    assert_eq!(cat["code"], "LIFE_CAT");

    // 2. Create library template
    let tmpl = create_test_template(&app, "LIFE_TMPL", "Lifecycle Template", "individual").await;
    assert_eq!(tmpl["code"], "LIFE_TMPL");

    // 3. Create goal plan
    let plan = create_test_plan(&app, "LIFE_PLAN", "Lifecycle Plan").await;
    let plan_id = plan["id"].as_str().unwrap();
    assert_eq!(plan["status"], "draft");

    // 4. Activate plan
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/plans/id/{}/status", plan_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Create organization-level goal
    let org_goal = create_test_goal(&app, "Company Goal: $10M Revenue", "organization", Some(plan_id)).await;
    let org_goal_id = org_goal["id"].as_str().unwrap();
    assert_eq!(org_goal["status"], "not_started");

    // 6. Create cascaded team goal
    let (k2, v2) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k2, &v2)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Sales Team: $5M Revenue",
            "goalType": "team",
            "planId": plan_id,
            "parentGoalId": org_goal_id,
            "weight": "50.0",
            "targetValue": "5000000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let team_goal: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let team_goal_id = team_goal["id"].as_str().unwrap();

    // 7. Create individual goal cascaded from team
    let (k3, v3) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/goals")
        .header("Content-Type", "application/json").header(&k3, &v3)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Individual: Close $1M deals",
            "goalType": "individual",
            "planId": plan_id,
            "parentGoalId": team_goal_id,
            "weight": "20.0",
            "targetValue": "1000000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ind_goal: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let ind_goal_id = ind_goal["id"].as_str().unwrap();

    // 8. Create alignment
    let (k4, v4) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/goal-management/alignments")
        .header("Content-Type", "application/json").header(&k4, &v4)
        .body(Body::from(serde_json::to_string(&json!({
            "sourceGoalId": ind_goal_id,
            "alignedToGoalId": org_goal_id,
            "alignmentType": "cascaded_from",
            "description": "Individual goal cascaded from company goal"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let alignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let alignment_id = alignment["id"].as_str().unwrap();

    // 9. Add notes to the individual goal
    let (k5, v5) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/{}/notes", ind_goal_id))
        .header("Content-Type", "application/json").header(&k5, &v5)
        .body(Body::from(serde_json::to_string(&json!({
            "noteType": "check_in", "content": "Q1 progress: 60% of target achieved",
            "visibility": "manager"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let note: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let note_id = note["id"].as_str().unwrap();

    // 10. Update individual goal progress
    let (k6, v6) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/goal-management/goals/id/{}/progress", ind_goal_id))
        .header("Content-Type", "application/json").header(&k6, &v6)
        .body(Body::from(serde_json::to_string(&json!({
            "actualValue": "600000", "progressPct": "60.0", "status": "on_track"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "on_track");

    // 11. Check dashboard summary
    let (k7, v7) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/goal-management/dashboard")
        .header(&k7, &v7).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalGoals"].as_i64().unwrap() >= 3);
    assert!(summary["totalPlans"].as_i64().unwrap() >= 1);
    assert!(summary["activePlans"].as_i64().unwrap() >= 1);
    assert!(summary["totalAlignments"].as_i64().unwrap() >= 1);

    // 12. Cleanup: delete note, alignment, goals, plan, template, category
    let (k8, v8) = auth_header(&admin_claims());
    // Delete note
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/notes/id/{}", note_id))
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete alignment
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/alignments/id/{}", alignment_id))
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete goals (child first due to FK)
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/goals/id/{}", ind_goal_id))
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/goals/id/{}", team_goal_id))
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/goal-management/goals/id/{}", org_goal_id))
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete plan
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/plans/code/LIFE_PLAN")
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete template
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/library/templates/LIFE_TMPL")
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Delete category
    let _ = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/goal-management/library/categories/LIFE_CAT")
        .header(&k8, &v8).body(Body::empty()).unwrap()
    ).await.unwrap();
}
