//! Performance Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud HCM Performance Management:
//! - Rating model CRUD
//! - Review cycle lifecycle (create → planning → goal_setting → self_eval → manager_eval → complete)
//! - Competency CRUD
//! - Performance document lifecycle with full workflow
//! - Goal management (create → rate → complete)
//! - Competency assessments
//! - Feedback lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_performance_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

fn test_rating_scale() -> serde_json::Value {
    json!([
        {"value": 1, "label": "Below Expectations"},
        {"value": 2, "label": "Needs Improvement"},
        {"value": 3, "label": "Meets Expectations"},
        {"value": 4, "label": "Exceeds Expectations"},
        {"value": 5, "label": "Outstanding"}
    ])
}

async fn create_test_rating_model(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/rating-models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Rating Model", code),
            "rating_scale": test_rating_scale()
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_cycle(app: &axum::Router, name: &str, cycle_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/cycles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": name,
            "cycle_type": cycle_type,
            "start_date": "2026-01-01",
            "end_date": "2026-12-31",
            "goal_setting_start": "2026-01-15",
            "goal_setting_end": "2026-02-15",
            "self_evaluation_start": "2026-10-01",
            "self_evaluation_end": "2026-10-31",
            "manager_evaluation_start": "2026-11-01",
            "manager_evaluation_end": "2026-11-30"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_competency(app: &axum::Router, code: &str, category: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/competencies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Competency", code),
            "category": category,
            "behavioral_indicators": [{"level": 1, "description": "Basic"}]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Rating Model Tests
// ============================================================================

#[tokio::test]
async fn test_create_rating_model() {
    let (_state, app) = setup_performance_test().await;
    let model = create_test_rating_model(&app, "RM-001").await;
    assert_eq!(model["code"], "RM-001");
    assert_eq!(model["isActive"], true);
    assert!(model["id"].is_string());
}

#[tokio::test]
async fn test_create_rating_model_invalid_scale() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/rating-models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Model",
            "rating_scale": []
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_rating_model() {
    let (_state, app) = setup_performance_test().await;
    create_test_rating_model(&app, "GET-RM").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/performance/rating-models/GET-RM")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let model: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(model["code"], "GET-RM");
}

#[tokio::test]
async fn test_list_rating_models() {
    let (_state, app) = setup_performance_test().await;
    create_test_rating_model(&app, "LIST-A").await;
    create_test_rating_model(&app, "LIST-B").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/performance/rating-models")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_rating_model() {
    let (_state, app) = setup_performance_test().await;
    create_test_rating_model(&app, "DEL-RM").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/performance/rating-models/DEL-RM")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Review Cycle Tests
// ============================================================================

#[tokio::test]
async fn test_create_review_cycle() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "2026 Annual Review", "annual").await;
    assert_eq!(cycle["name"], "2026 Annual Review");
    assert_eq!(cycle["cycleType"], "annual");
    assert_eq!(cycle["status"], "draft");
    assert_eq!(cycle["requireGoals"], true);
}

#[tokio::test]
async fn test_create_cycle_invalid_type() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/cycles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Cycle",
            "cycle_type": "biweekly",
            "start_date": "2026-01-01",
            "end_date": "2026-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cycle_full_lifecycle() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "Lifecycle Cycle", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();
    assert_eq!(cycle["status"], "draft");

    // draft → planning
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "planning");

    // planning → goal_setting
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // goal_setting → self_evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "self_evaluation"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // self_evaluation → manager_evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "manager_evaluation"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // manager_evaluation → completed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "completed");
}

#[tokio::test]
async fn test_cycle_invalid_transition() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "Invalid Transition", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    // draft → completed (skipping steps) should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_cycles_filtered() {
    let (_state, app) = setup_performance_test().await;
    create_test_cycle(&app, "Filter Cycle 1", "annual").await;
    create_test_cycle(&app, "Filter Cycle 2", "quarterly").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/performance/cycles?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Competency Tests
// ============================================================================

#[tokio::test]
async fn test_create_competency() {
    let (_state, app) = setup_performance_test().await;
    let comp = create_test_competency(&app, "COMM", "core").await;
    assert_eq!(comp["code"], "COMM");
    assert_eq!(comp["category"], "core");
    assert_eq!(comp["isActive"], true);
}

#[tokio::test]
async fn test_create_competency_invalid_category() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/competencies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Comp",
            "category": "nonexistent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_competencies_by_category() {
    let (_state, app) = setup_performance_test().await;
    create_test_competency(&app, "LC-CORE", "core").await;
    create_test_competency(&app, "LC-TECH", "technical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/performance/competencies?category=core")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let comps = resp["data"].as_array().unwrap();
    assert!(comps.len() >= 1);
    assert!(comps.iter().all(|c| c["category"] == "core"));
}

#[tokio::test]
async fn test_delete_competency() {
    let (_state, app) = setup_performance_test().await;
    create_test_competency(&app, "DEL-COMP", "core").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/performance/competencies/DEL-COMP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/performance/competencies/DEL-COMP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Performance Document Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_document_full_lifecycle() {
    let (_state, app) = setup_performance_test().await;

    // Create cycle and transition to goal_setting
    let cycle = create_test_cycle(&app, "Doc Lifecycle", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Transition to planning, then goal_setting
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create document
    let employee_id = "00000000-0000-0000-0000-000000000100";
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": employee_id,
            "employee_name": "John Doe",
            "manager_name": "Jane Manager"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let doc_id = doc["id"].as_str().unwrap();
    assert_eq!(doc["status"], "not_started");

    // Transition document: not_started → goal_setting
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/transition", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Add goals
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "document_id": doc_id,
            "employee_id": employee_id,
            "goal_name": "Increase revenue by 10%",
            "goal_category": "performance",
            "weight": "60.00",
            "target_metric": "Revenue target: $1.1M"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "document_id": doc_id,
            "employee_id": employee_id,
            "goal_name": "Complete leadership training",
            "goal_category": "development",
            "weight": "40.00"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List goals
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/performance/documents/{}/goals", doc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let goals_resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(goals_resp["data"].as_array().unwrap().len(), 2);

    // goal_setting → self_evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/transition", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "self_evaluation"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Self-evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/self-evaluation", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "overall_rating": "4",
            "comments": "I achieved most of my goals"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // self_evaluation → manager_evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/transition", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "manager_evaluation"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Manager evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/manager-evaluation", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "overall_rating": "4",
            "comments": "Good performance this year"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Finalize
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/documents/{}/finalize", doc_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "final_rating": "4",
            "final_comments": "Solid year"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let final_doc: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(final_doc["status"], "completed");
}

#[tokio::test]
async fn test_document_duplicate_prevented() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "Dup Test", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Transition cycle to goal_setting
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    let employee_id = "00000000-0000-0000-0000-000000000201";

    // First doc succeeds
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": employee_id,
            "employee_name": "Test"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Second doc for same employee+cycle should fail
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": employee_id,
            "employee_name": "Test"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Goal Tests
// ============================================================================

#[tokio::test]
async fn test_goal_rate_and_complete() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "Goal Rate", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    let employee_id = "00000000-0000-0000-0000-000000000300";
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": employee_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let doc_id = doc["id"].as_str().unwrap();

    // Add goal
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/goals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "document_id": doc_id,
            "employee_id": employee_id,
            "goal_name": "Test Goal",
            "weight": "100.00"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let goal: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let goal_id = goal["id"].as_str().unwrap();

    // Self-rate the goal
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/goals/{}/rate", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rating_type": "self",
            "rating": "4",
            "comments": "Did well"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Manager-rate the goal
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/goals/{}/rate", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rating_type": "manager",
            "rating": "5",
            "comments": "Excellent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Complete the goal
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/goals/{}/complete", goal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actual_result": "Achieved 12% increase"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
}

// ============================================================================
// Competency Assessment Tests
// ============================================================================

#[tokio::test]
async fn test_competency_assessment() {
    let (_state, app) = setup_performance_test().await;
    let comp = create_test_competency(&app, "ASSESS", "leadership").await;
    let comp_id = comp["id"].as_str().unwrap();

    let cycle = create_test_cycle(&app, "Assess Cycle", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    let employee_id = "00000000-0000-0000-0000-000000000400";
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": employee_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let doc_id = doc["id"].as_str().unwrap();

    // Self-assessment
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/assessments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "document_id": doc_id,
            "employee_id": employee_id,
            "competency_id": comp_id,
            "rating_type": "self",
            "rating": "4",
            "comments": "Good leadership"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Manager assessment (upsert)
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/assessments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "document_id": doc_id,
            "employee_id": employee_id,
            "competency_id": comp_id,
            "rating_type": "manager",
            "rating": "5",
            "comments": "Excellent leadership"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // List assessments
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/performance/documents/{}/assessments", doc_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Feedback Tests
// ============================================================================

#[tokio::test]
async fn test_feedback_lifecycle() {
    let (_state, app) = setup_performance_test().await;

    let (k, v) = auth_header(&admin_claims());
    let employee_id = "00000000-0000-0000-0000-000000000500";

    // Create feedback
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/feedback")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": employee_id,
            "feedback_type": "peer",
            "subject": "Great teamwork",
            "content": "John showed excellent collaboration skills during Q1.",
            "visibility": "manager_and_employee"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fb: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let fb_id = fb["id"].as_str().unwrap();
    assert_eq!(fb["status"], "draft");
    assert_eq!(fb["feedbackType"], "peer");

    // Submit feedback
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/feedback/{}/submit", fb_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");
}

#[tokio::test]
async fn test_feedback_invalid_type() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/feedback")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000600",
            "feedback_type": "invalid_type",
            "content": "Test content"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_feedback_empty_content() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/feedback")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000601",
            "feedback_type": "peer",
            "content": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_performance_dashboard() {
    let (_state, app) = setup_performance_test().await;

    let cycle = create_test_cycle(&app, "Dashboard Cycle", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create document
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": "00000000-0000-0000-0000-000000000700",
            "employee_name": "Dashboard Employee"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/performance/dashboard/{}", cycle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalDocuments"].as_i64().unwrap() >= 1);
    assert!(dashboard["notStartedCount"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Document Creation Validation Tests
// ============================================================================

#[tokio::test]
async fn test_document_draft_cycle_rejected() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "Draft Cycle", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    // Try to create document in draft cycle - should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": "00000000-0000-0000-0000-000000000800"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_document_nonexistent_cycle() {
    let (_state, app) = setup_performance_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_cycle_id = Uuid::new_v4().to_string();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": fake_cycle_id,
            "employee_id": "00000000-0000-0000-0000-000000000801"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_documents_by_employee() {
    let (_state, app) = setup_performance_test().await;
    let cycle = create_test_cycle(&app, "List Docs", "annual").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "planning"})).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/performance/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "goal_setting"})).unwrap())).unwrap()
    ).await.unwrap();

    let emp = "00000000-0000-0000-0000-000000000900";
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/performance/documents")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "review_cycle_id": cycle_id,
            "employee_id": emp,
            "employee_name": "List Test Employee"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/performance/documents?employee_id={}", emp))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}
