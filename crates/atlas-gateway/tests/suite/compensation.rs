//! Compensation Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud HCM Compensation Workbench:
//! - Plan CRUD and component management
//! - Compensation cycle lifecycle (draft → active → allocation → review → completed)
//! - Budget pool management
//! - Worksheet creation, line management, and approval workflow
//! - Compensation statement generation and publishing
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_compensation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_plan(app: &axum::Router, code: &str, plan_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "plan_code": code,
            "plan_name": format!("{} Plan", code),
            "plan_type": plan_type,
            "eligibility_criteria": {"min_grade": 5}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_cycle(app: &axum::Router, name: &str, cycle_type: &str, budget: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/cycles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cycle_name": name,
            "cycle_type": cycle_type,
            "start_date": "2026-01-01",
            "end_date": "2026-12-31",
            "total_budget": budget,
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Transition cycle to allocation phase
async fn transition_cycle_to_allocation(app: &axum::Router, cycle_id: &str) {
    let (k, v) = auth_header(&admin_claims());
    // draft → active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    // active → allocation
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "allocation"})).unwrap())).unwrap()
    ).await.unwrap();
}

// ============================================================================
// Plan Tests
// ============================================================================

#[tokio::test]
async fn test_create_plan() {
    let (_state, app) = setup_compensation_test().await;
    let plan = create_test_plan(&app, "SAL-001", "salary").await;
    assert_eq!(plan["planCode"], "SAL-001");
    assert_eq!(plan["planType"], "salary");
    assert_eq!(plan["status"], "active");
    assert!(plan["id"].is_string());
}

#[tokio::test]
async fn test_create_plan_invalid_type() {
    let (_state, app) = setup_compensation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "plan_code": "BAD", "plan_name": "Bad Plan", "plan_type": "nonexistent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_plan_duplicate() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "DUP-01", "salary").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "plan_code": "DUP-01", "plan_name": "Dup Plan", "plan_type": "salary"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_plan() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "GET-01", "bonus").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/plans/GET-01")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(plan["planCode"], "GET-01");
    assert_eq!(plan["planType"], "bonus");
}

#[tokio::test]
async fn test_list_plans() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "LIST-A", "salary").await;
    create_test_plan(&app, "LIST-B", "bonus").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/plans")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_plan() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "DEL-01", "equity").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/compensation/plans/DEL-01")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/plans/DEL-01")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Component Tests
// ============================================================================

#[tokio::test]
async fn test_create_component() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "COMP-01", "mixed").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/compensation/plans/COMP-01/components")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "component_name": "Base Salary Component",
            "component_type": "salary",
            "is_recurring": true,
            "frequency": "annual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let comp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(comp["componentName"], "Base Salary Component");
    assert_eq!(comp["componentType"], "salary");
}

#[tokio::test]
async fn test_list_components() {
    let (_state, app) = setup_compensation_test().await;
    create_test_plan(&app, "LCOMP", "mixed").await;
    let (k, v) = auth_header(&admin_claims());
    // Add 2 components
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/compensation/plans/LCOMP/components")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "component_name": "Merit", "component_type": "merit"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/compensation/plans/LCOMP/components")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "component_name": "Bonus Pool", "component_type": "bonus"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/plans/LCOMP/components")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Cycle Tests
// ============================================================================

#[tokio::test]
async fn test_create_cycle() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "2026 Annual Comp", "annual", "5000000").await;
    assert_eq!(cycle["cycleName"], "2026 Annual Comp");
    assert_eq!(cycle["cycleType"], "annual");
    assert_eq!(cycle["status"], "draft");
    assert_eq!(cycle["totalBudget"], "5000000.00");
    assert_eq!(cycle["currencyCode"], "USD");
}

#[tokio::test]
async fn test_create_cycle_invalid_type() {
    let (_state, app) = setup_compensation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/cycles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cycle_name": "Bad", "cycle_type": "weekly",
            "start_date": "2026-01-01", "end_date": "2026-12-31", "total_budget": "1000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_cycle_invalid_dates() {
    let (_state, app) = setup_compensation_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/compensation/cycles")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cycle_name": "Bad Dates", "cycle_type": "annual",
            "start_date": "2026-12-31", "end_date": "2026-01-01", "total_budget": "1000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cycle_full_lifecycle() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Lifecycle Cycle", "annual", "1000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();
    assert_eq!(cycle["status"], "draft");

    let (k, v) = auth_header(&admin_claims());

    // draft → active
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "active");

    // active → allocation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "allocation"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // allocation → review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "review"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // review → completed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
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
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Bad Transition", "annual", "500000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    // draft → completed (skipping steps) should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_cycles_filtered() {
    let (_state, app) = setup_compensation_test().await;
    create_test_cycle(&app, "Filter Cycle 1", "annual", "1000000").await;
    create_test_cycle(&app, "Filter Cycle 2", "mid_year", "500000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/cycles?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_cycle() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Delete Cycle", "annual", "100000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/compensation/cycles/{}", cycle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_active_cycle_rejected() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Active Cycle", "annual", "100000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Transition to active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/transition", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // Try to delete active cycle
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/compensation/cycles/{}", cycle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Budget Pool Tests
// ============================================================================

#[tokio::test]
async fn test_create_budget_pool() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Pool Cycle", "annual", "2000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/pools", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "pool_name": "Engineering Merit Pool",
            "pool_type": "merit",
            "manager_name": "John Manager",
            "department_name": "Engineering",
            "total_budget": "500000",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(pool["poolName"], "Engineering Merit Pool");
    assert_eq!(pool["poolType"], "merit");
    assert_eq!(pool["totalBudget"], "500000.00");
    assert_eq!(pool["remainingBudget"], "500000.00");
}

#[tokio::test]
async fn test_list_budget_pools() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Pool List", "annual", "2000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/pools", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "pool_name": "Merit Pool", "pool_type": "merit", "total_budget": "300000"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/compensation/cycles/{}/pools", cycle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Worksheet & Line Tests
// ============================================================================

#[tokio::test]
async fn test_worksheet_full_workflow() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "WS Cycle", "annual", "1000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    transition_cycle_to_allocation(&app, cycle_id).await;

    let (k, v) = auth_header(&admin_claims());
    let manager_id = "00000000-0000-0000-0000-000000000100";

    // Create worksheet
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/worksheets", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "manager_id": manager_id,
            "manager_name": "Jane Manager"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ws: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let ws_id = ws["id"].as_str().unwrap();
    assert_eq!(ws["status"], "draft");

    // Add employee line
    let emp1 = "00000000-0000-0000-0000-000000000201";
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/lines", ws_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": emp1,
            "employee_name": "Alice Smith",
            "job_title": "Senior Engineer",
            "current_base_salary": "120000",
            "proposed_base_salary": "132000",
            "merit_amount": "12000",
            "bonus_amount": "15000",
            "equity_amount": "20000",
            "performance_rating": "4",
            "manager_comments": "Strong performer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["status"], "pending");
    // Verify calculated fields
    let change: f64 = line["salaryChangeAmount"].as_str().unwrap().parse().unwrap();
    assert!((change - 12000.0).abs() < 1.0);
    let total_comp: f64 = line["totalCompensation"].as_str().unwrap().parse().unwrap();
    assert!((total_comp - 179000.0).abs() < 1.0);

    // Add second employee
    let emp2 = "00000000-0000-0000-0000-000000000202";
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/lines", ws_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": emp2,
            "employee_name": "Bob Jones",
            "current_base_salary": "100000",
            "proposed_base_salary": "108000",
            "merit_amount": "8000",
            "bonus_amount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/compensation/worksheets/{}/lines", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines_resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines_resp["data"].as_array().unwrap().len(), 2);

    // Submit worksheet
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/submit", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve worksheet
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/approve", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_worksheet_cannot_create_in_draft_cycle() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Draft WS", "annual", "500000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let manager_id = "00000000-0000-0000-0000-000000000100";

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/worksheets", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "manager_id": manager_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    // Should fail because cycle is in draft status
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_empty_worksheet_rejected() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Empty WS", "annual", "500000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    transition_cycle_to_allocation(&app, cycle_id).await;

    let (k, v) = auth_header(&admin_claims());
    let manager_id = "00000000-0000-0000-0000-000000000100";

    // Create worksheet (no lines)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/worksheets", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "manager_id": manager_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ws: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let ws_id = ws["id"].as_str().unwrap();

    // Try to submit empty worksheet
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/submit", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_worksheet() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Reject WS", "annual", "500000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    transition_cycle_to_allocation(&app, cycle_id).await;

    let (k, v) = auth_header(&admin_claims());
    let manager_id = "00000000-0000-0000-0000-000000000100";

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/worksheets", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "manager_id": manager_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ws: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let ws_id = ws["id"].as_str().unwrap();

    // Add line
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/lines", ws_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000300",
            "current_base_salary": "90000", "proposed_base_salary": "95000"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/submit", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/reject", ws_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_update_worksheet_line() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Update Line", "annual", "500000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    transition_cycle_to_allocation(&app, cycle_id).await;

    let (k, v) = auth_header(&admin_claims());
    let manager_id = "00000000-0000-0000-0000-000000000100";

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/worksheets", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "manager_id": manager_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ws: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let ws_id = ws["id"].as_str().unwrap();

    // Add line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/worksheets/{}/lines", ws_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000400",
            "current_base_salary": "100000", "proposed_base_salary": "105000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Update line
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/compensation/lines/{}", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "proposed_base_salary": "115000",
            "merit_amount": "10000",
            "bonus_amount": "5000",
            "equity_amount": "0",
            "manager_comments": "Updated after review"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let total: f64 = updated["totalCompensation"].as_str().unwrap().parse().unwrap();
    assert!((total - 130000.0).abs() < 1.0);
}

// ============================================================================
// Statement Tests
// ============================================================================

#[tokio::test]
async fn test_generate_and_publish_statement() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "Stmt Cycle", "annual", "1000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let emp_id = "00000000-0000-0000-0000-000000000500";

    // Generate statement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/cycles/{}/statements", cycle_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": emp_id,
            "employee_name": "Test Employee",
            "base_salary": "120000",
            "merit_increase": "8000",
            "bonus": "15000",
            "equity": "20000",
            "benefits_value": "25000",
            "currency_code": "USD",
            "components": [{"name": "Health Insurance", "value": 12000}]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let stmt: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let stmt_id = stmt["id"].as_str().unwrap();
    assert_eq!(stmt["status"], "draft");

    // Verify total calculation: 120000 + 8000 + 15000 + 20000 + 25000 = 188000
    let total: f64 = stmt["totalCompensation"].as_str().unwrap().parse().unwrap();
    assert!((total - 188000.0).abs() < 1.0);
    let direct: f64 = stmt["totalDirectCompensation"].as_str().unwrap().parse().unwrap();
    assert!((direct - 163000.0).abs() < 1.0);
    let indirect: f64 = stmt["totalIndirectCompensation"].as_str().unwrap().parse().unwrap();
    assert!((indirect - 25000.0).abs() < 1.0);

    // Publish statement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/compensation/statements/{}/publish", stmt_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let published: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(published["status"], "published");
}

#[tokio::test]
async fn test_list_statements() {
    let (_state, app) = setup_compensation_test().await;
    let cycle = create_test_cycle(&app, "List Stmt", "annual", "2000000").await;
    let cycle_id = cycle["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate 2 statements
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/compensation/cycles/{}/statements", cycle_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "employee_id": format!("00000000-0000-0000-0000-{:012}", 600 + i),
                "employee_name": format!("Employee {}", 600 + i),
                "base_salary": "100000"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/compensation/cycles/{}/statements", cycle_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_compensation_dashboard() {
    let (_state, app) = setup_compensation_test().await;

    create_test_plan(&app, "DASH-01", "salary").await;
    create_test_cycle(&app, "Dash Cycle", "annual", "1000000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/compensation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["activePlans"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeCycles"].as_i64().unwrap() >= 0);
}
