//! Service Request Management E2E Tests
//!
//! Tests for Oracle Fusion CX Service:
//! - Service category CRUD
//! - Service request lifecycle (open → in_progress → resolved → closed)
//! - Request assignment and escalation
//! - Communications/updates
//! - Resolution workflow
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_service_request_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_category(app: &axum::Router, code: &str, sla_hours: Option<i32>) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "code": code,
        "name": format!("{} Category", code),
        "default_priority": "medium",
    });
    if let Some(hours) = sla_hours {
        payload["default_sla_hours"] = json!(hours);
    }
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_request(app: &axum::Router, number: &str, priority: &str, req_type: &str, channel: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": number,
            "title": format!("{} Issue", number),
            "description": "Customer reports an issue",
            "priority": priority,
            "request_type": req_type,
            "channel": channel,
            "customer_id": "00000000-0000-0000-0000-000000000100",
            "customer_name": "Test Customer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Category Tests
// ============================================================================

#[tokio::test]
async fn test_create_category() {
    let (_state, app) = setup_service_request_test().await;
    let cat = create_test_category(&app, "HW", Some(24)).await;
    assert_eq!(cat["code"], "HW");
    assert_eq!(cat["name"], "HW Category");
    assert_eq!(cat["defaultSlaHours"], 24);
    assert_eq!(cat["defaultPriority"], "medium");
    assert!(cat["id"].is_string());
}

#[tokio::test]
async fn test_create_category_invalid_priority() {
    let (_state, app) = setup_service_request_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD", "name": "Bad Category", "default_priority": "urgent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_category() {
    let (_state, app) = setup_service_request_test().await;
    create_test_category(&app, "SW", Some(48)).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/categories/SW")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cat: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cat["code"], "SW");
}

#[tokio::test]
async fn test_list_categories() {
    let (_state, app) = setup_service_request_test().await;
    create_test_category(&app, "LIST-A", None).await;
    create_test_category(&app, "LIST-B", None).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/categories")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_category() {
    let (_state, app) = setup_service_request_test().await;
    create_test_category(&app, "DEL", None).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/service/categories/DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/categories/DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Service Request Tests
// ============================================================================

#[tokio::test]
async fn test_create_service_request() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "SR-001", "high", "incident", "phone").await;
    assert_eq!(req["requestNumber"], "SR-001");
    assert_eq!(req["title"], "SR-001 Issue");
    assert_eq!(req["priority"], "high");
    assert_eq!(req["requestType"], "incident");
    assert_eq!(req["channel"], "phone");
    assert_eq!(req["status"], "open");
    assert_eq!(req["customerName"], "Test Customer");
    assert!(req["id"].is_string());
}

#[tokio::test]
async fn test_create_request_invalid_priority() {
    let (_state, app) = setup_service_request_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": "BAD-1", "title": "Bad", "priority": "super_high",
            "request_type": "incident", "channel": "web"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_request_invalid_type() {
    let (_state, app) = setup_service_request_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": "BAD-2", "title": "Bad", "priority": "medium",
            "request_type": "invalid_type", "channel": "web"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_request_invalid_channel() {
    let (_state, app) = setup_service_request_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": "BAD-3", "title": "Bad", "priority": "medium",
            "request_type": "incident", "channel": "telegraph"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_request() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "GET-001", "medium", "service_request", "web").await;
    let id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["requestNumber"], "GET-001");
}

#[tokio::test]
async fn test_get_request_by_number() {
    let (_state, app) = setup_service_request_test().await;
    create_test_request(&app, "NUM-001", "low", "problem", "email").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/requests/number/NUM-001")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["requestNumber"], "NUM-001");
}

#[tokio::test]
async fn test_list_requests() {
    let (_state, app) = setup_service_request_test().await;
    create_test_request(&app, "LIST-1", "high", "incident", "web").await;
    create_test_request(&app, "LIST-2", "low", "service_request", "email").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/requests")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_requests_filtered_by_status() {
    let (_state, app) = setup_service_request_test().await;
    create_test_request(&app, "FILT-1", "medium", "incident", "web").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/requests?status=open")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Status Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_request_full_lifecycle() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "LC-001", "high", "incident", "phone").await;
    let id = req["id"].as_str().unwrap();
    assert_eq!(req["status"], "open");

    let (k, v) = auth_header(&admin_claims());

    // open → in_progress
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "in_progress");

    // in_progress → pending_customer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "pending_customer"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // pending_customer → resolved (via resolve endpoint)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Root cause identified and fixed",
            "resolution_code": "resolved"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "resolved");
    assert_eq!(c["resolutionCode"], "resolved");
    assert!(c["resolvedAt"].is_string());

    // resolved → closed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "closed"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "closed");
    assert!(c["closedAt"].is_string());
}

#[tokio::test]
async fn test_invalid_transition() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "INV-001", "medium", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    // open → resolved (skipping steps) should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "resolved"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_from_open() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "CANC-001", "low", "service_request", "email").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "cancelled");
}

#[tokio::test]
async fn test_reopen_from_resolved() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "REOP-001", "high", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // open → in_progress
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();

    // Resolve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Fixed", "resolution_code": "resolved"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // resolved → in_progress (reopen)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["status"], "in_progress");
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[tokio::test]
async fn test_assign_request() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "ASGN-001", "high", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let agent_id = "00000000-0000-0000-0000-000000000500";

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/assign", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assigned_to": agent_id,
            "assigned_to_name": "Agent Smith",
            "assigned_group": "Tier 2 Support",
            "assignment_type": "initial"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let assignment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(assignment["assignedTo"], agent_id);
    assert_eq!(assignment["assignedToName"], "Agent Smith");
    assert_eq!(assignment["assignedGroup"], "Tier 2 Support");

    // Verify request is now in_progress (auto-transition on assignment)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "in_progress");
    assert_eq!(updated["assignedToName"], "Agent Smith");
}

#[tokio::test]
async fn test_list_assignments() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "ASGN-LIST", "medium", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Assign
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/assign", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assigned_to": "00000000-0000-0000-0000-000000000500",
            "assigned_to_name": "Agent 1",
            "assignment_type": "initial"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Escalate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/assign", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assigned_to": "00000000-0000-0000-0000-000000000600",
            "assigned_to_name": "Agent 2",
            "assignment_type": "escalation"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List assignments
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}/assignments", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let assignments = resp["data"].as_array().unwrap();
    assert!(assignments.len() >= 1, "Expected at least 1 assignment, got {}", assignments.len());
    // Verify we have at least one with escalation type
    let escalation_count = assignments.iter().filter(|a| a["assignmentType"] == "escalation").count();
    assert!(escalation_count >= 1, "Expected at least 1 escalation assignment");
}

// ============================================================================
// Updates / Communications Tests
// ============================================================================

#[tokio::test]
async fn test_add_and_list_updates() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "UPD-001", "medium", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add public comment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/updates", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "update_type": "comment",
            "body": "We are looking into this issue.",
            "is_internal": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let update: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(update["updateType"], "comment");
    assert_eq!(update["body"], "We are looking into this issue.");
    assert_eq!(update["isInternal"], false);

    // Add internal note
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/updates", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "update_type": "note",
            "body": "Internal note: customer seems frustrated",
            "is_internal": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List updates (excluding internal)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}/updates?include_internal=false", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let updates = resp["data"].as_array().unwrap();
    assert_eq!(updates.len(), 1); // Only public comment

    // List updates (including internal)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}/updates?include_internal=true", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Resolution Tests
// ============================================================================

#[tokio::test]
async fn test_resolve_with_workaround() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "RES-001", "critical", "incident", "phone").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to in_progress
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();

    // Resolve with workaround
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Provided workaround: restart the service after config change",
            "resolution_code": "workaround"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resolved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["resolutionCode"], "workaround");
}

#[tokio::test]
async fn test_resolve_invalid_code() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "RES-INV", "medium", "incident", "web").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to in_progress
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Fixed", "resolution_code": "invalid_code"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_resolve_closed() {
    let (_state, app) = setup_service_request_test().await;
    let req = create_test_request(&app, "RES-CLSD", "low", "service_request", "email").await;
    let id = req["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Cancel it
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap())).unwrap()
    ).await.unwrap();

    // Try to resolve cancelled
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Should fail", "resolution_code": "resolved"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_service_request_dashboard() {
    let (_state, app) = setup_service_request_test().await;

    create_test_request(&app, "DASH-1", "high", "incident", "web").await;
    create_test_request(&app, "DASH-2", "low", "service_request", "email").await;
    create_test_request(&app, "DASH-3", "critical", "incident", "phone").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalOpen"].as_i64().unwrap() >= 3);
    assert!(dashboard["byPriority"].is_object());
    assert!(dashboard["byStatus"].is_object());
    assert!(dashboard["byChannel"].is_object());
}

// ============================================================================
// Category Association Test
// ============================================================================

#[tokio::test]
async fn test_request_with_category() {
    let (_state, app) = setup_service_request_test().await;
    let cat = create_test_category(&app, "NET", Some(8)).await;
    let cat_id = cat["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": "CAT-001",
            "title": "Network connectivity issue",
            "category_id": cat_id,
            "priority": "high",
            "request_type": "incident",
            "channel": "phone"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(req["categoryId"], cat_id);
    assert_eq!(req["categoryName"], "NET Category");
    assert!(req["slaDueDate"].is_string()); // SLA set from category
}

// ============================================================================
// Comprehensive Workflow Test
// ============================================================================

#[tokio::test]
async fn test_full_customer_service_workflow() {
    let (_state, app) = setup_service_request_test().await;

    // 1. Create category
    let cat = create_test_category(&app, "SUPPORT", Some(24)).await;

    // 2. Customer reports issue
    let (k, v) = auth_header(&admin_claims());
    let cat_id = cat["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/service/requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "request_number": "WF-001",
            "title": "Cannot access portal after password reset",
            "description": "Customer reset password but still cannot log in",
            "category_id": cat_id,
            "priority": "high",
            "request_type": "incident",
            "channel": "phone",
            "customer_id": "00000000-0000-0000-0000-000000000200",
            "customer_name": "Jane Doe"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let req: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let req_id = req["id"].as_str().unwrap();
    assert_eq!(req["status"], "open");

    // 3. Assign to agent (auto-transitions to in_progress)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/assign", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assigned_to": "00000000-0000-0000-0000-000000000300",
            "assigned_to_name": "Support Agent Alice",
            "assigned_group": "L1 Support",
            "assignment_type": "initial"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 4. Agent adds internal note
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/updates", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "update_type": "note",
            "body": "Account locked, need to unlock and reset MFA",
            "is_internal": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 5. Agent communicates with customer
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/updates", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "update_type": "email",
            "subject": "Regarding your login issue",
            "body": "We are working on your issue. Please verify your identity.",
            "is_internal": false
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 6. Escalate to L2
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/assign", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assigned_to": "00000000-0000-0000-0000-000000000400",
            "assigned_to_name": "Senior Agent Bob",
            "assigned_group": "L2 Support",
            "assignment_type": "escalation"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 7. Resolve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/resolve", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Unlocked account and reset MFA enrollment. Customer confirmed access restored.",
            "resolution_code": "resolved"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resolved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["assignedToName"], "Senior Agent Bob");

    // 8. Close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/service/requests/{}/status", req_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "closed"})).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let closed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(closed["status"], "closed");

    // 9. Verify dashboard reflects the completed request
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/service/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalClosed"].as_i64().unwrap() >= 1);
    // Verify by_status includes entries
    assert!(dashboard["byStatus"].is_object());

    // 10. Verify assignment history has 2 entries
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/service/requests/{}/assignments", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let assignments: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(assignments["data"].as_array().unwrap().len() >= 2);
}
