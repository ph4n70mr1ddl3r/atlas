//! Project Resource Management E2E Tests
//!
//! Tests for Oracle Fusion Project Management > Resource Management:
//! - Resource profile CRUD and availability management
//! - Resource request lifecycle (draft -> submitted -> fulfilled -> cancelled)
//! - Resource assignment management with planned/actual hours
//! - Utilization entry recording, approval, and rejection
//! - Dashboard analytics
//! - Validation edge cases and error handling
//! - Full end-to-end lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

/// Helper to extract a numeric value from JSON, handling both f64 and i64 representations
fn jf64(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| v.as_i64().unwrap_or(0) as f64)
}

async fn setup_resource_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_profile(
    app: &axum::Router,
    number: &str,
    name: &str,
    resource_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "resourceNumber": number,
                        "name": name,
                        "email": format!("{}@test.com", number.to_lowercase()),
                        "resourceType": resource_type,
                        "department": "Engineering",
                        "jobTitle": "Senior Developer",
                        "skills": "Rust, SQL, Cloud",
                        "costRate": 150.0,
                        "billRate": 250.0
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for profile but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_request(
    app: &axum::Router,
    number: &str,
    role: &str,
    priority: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/requests")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "requestNumber": number,
                        "projectName": "Atlas Development",
                        "projectNumber": "PRJ-001",
                        "requestedRole": role,
                        "requiredSkills": "Rust, SQL",
                        "priority": priority,
                        "startDate": "2024-01-15",
                        "endDate": "2024-06-30",
                        "hoursPerWeek": 40,
                        "totalPlannedHours": 960
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for request but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Profile Tests
// ============================================================================

#[tokio::test]
async fn test_create_profile() {
    let (_state, app) = setup_resource_test().await;
    let profile = create_test_profile(&app, "RES-001", "Alice Johnson", "employee").await;
    assert_eq!(profile["resourceNumber"], "RES-001");
    assert_eq!(profile["name"], "Alice Johnson");
    assert_eq!(profile["resourceType"], "employee");
    assert_eq!(profile["availabilityStatus"], "available");
    assert!((jf64(&profile["costRate"]) - 150.0).abs() < 0.01);
    assert!((jf64(&profile["billRate"]) - 250.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_profile_duplicate_conflict() {
    let (_state, app) = setup_resource_test().await;
    create_test_profile(&app, "DUP-RES", "First", "employee").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "resourceNumber": "DUP-RES",
                        "name": "Duplicate",
                        "resourceType": "employee"
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
async fn test_create_profile_invalid_type() {
    let (_state, app) = setup_resource_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "resourceNumber": "BAD-TYPE",
                        "name": "Bad Type",
                        "resourceType": "vendor"
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
async fn test_get_profile() {
    let (_state, app) = setup_resource_test().await;
    let profile = create_test_profile(&app, "GET-RES", "Bob Smith", "contractor").await;
    let id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/resource/profiles/id/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["resourceNumber"], "GET-RES");
    assert_eq!(fetched["resourceType"], "contractor");
}

#[tokio::test]
async fn test_list_profiles() {
    let (_state, app) = setup_resource_test().await;
    create_test_profile(&app, "LIST-1", "Carol White", "employee").await;
    create_test_profile(&app, "LIST-2", "Dave Brown", "contractor").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resource/profiles")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_availability() {
    let (_state, app) = setup_resource_test().await;
    let profile = create_test_profile(&app, "AVAIL-RES", "Eve Davis", "employee").await;
    let id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/resource/profiles/id/{}/availability", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"availabilityStatus": "fully_allocated"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["availabilityStatus"], "fully_allocated");
}

#[tokio::test]
async fn test_delete_profile() {
    let (_state, app) = setup_resource_test().await;
    create_test_profile(&app, "DEL-RES", "Frank Miller", "employee").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/resource/profiles/number/DEL-RES")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Request Tests
// ============================================================================

#[tokio::test]
async fn test_create_request() {
    let (_state, app) = setup_resource_test().await;
    let request = create_test_request(&app, "REQ-001", "Senior Developer", "high").await;
    assert_eq!(request["requestNumber"], "REQ-001");
    assert_eq!(request["requestedRole"], "Senior Developer");
    assert_eq!(request["priority"], "high");
    assert_eq!(request["status"], "draft");
    assert!((jf64(&request["hoursPerWeek"]) - 40.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_request_invalid_dates() {
    let (_state, app) = setup_resource_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/requests")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "requestNumber": "BAD-DATE",
                        "requestedRole": "Developer",
                        "priority": "medium",
                        "startDate": "2025-01-01",
                        "endDate": "2024-01-01"
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
async fn test_request_lifecycle() {
    let (_state, app) = setup_resource_test().await;
    let request = create_test_request(&app, "LIFE-REQ", "Tech Lead", "critical").await;
    let id = request["id"].as_str().unwrap();
    assert_eq!(request["status"], "draft");

    let (k, v) = auth_header(&admin_claims());

    // Submit
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/requests/id/{}/submit", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "submitted");

    // Fulfill
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/requests/id/{}/fulfill", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fulfilled: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fulfilled["status"], "fulfilled");
    assert!(fulfilled["fulfilledBy"].is_string());
}

#[tokio::test]
async fn test_cancel_request() {
    let (_state, app) = setup_resource_test().await;
    let request = create_test_request(&app, "CNL-REQ", "Developer", "low").await;
    let id = request["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/requests/id/{}/cancel", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_requests() {
    let (_state, app) = setup_resource_test().await;
    create_test_request(&app, "LIST-REQ1", "Developer", "medium").await;
    create_test_request(&app, "LIST-REQ2", "Designer", "high").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resource/requests")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_request() {
    let (_state, app) = setup_resource_test().await;
    create_test_request(&app, "DEL-REQ", "QA Engineer", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/resource/requests/number/DEL-REQ")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Assignment Tests
// ============================================================================

async fn setup_resource_with_assignment(app: &axum::Router) -> (serde_json::Value, serde_json::Value) {
    let profile = create_test_profile(app, "ASGN-RES", "Grace Lee", "employee").await;
    let resource_id = profile["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/assignments")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentNumber": "ASGN-001",
                    "resourceId": resource_id,
                    "projectName": "Atlas Development",
                    "projectNumber": "PRJ-001",
                    "role": "Senior Developer",
                    "startDate": "2024-02-01",
                    "endDate": "2024-06-30",
                    "plannedHours": 800
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for assignment but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    let assignment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    (profile, assignment)
}

#[tokio::test]
async fn test_create_assignment() {
    let (_state, app) = setup_resource_test().await;
    let (_, assignment) = setup_resource_with_assignment(&app).await;
    assert_eq!(assignment["assignmentNumber"], "ASGN-001");
    assert_eq!(assignment["role"], "Senior Developer");
    assert_eq!(assignment["status"], "planned");
    assert!((jf64(&assignment["plannedHours"]) - 800.0).abs() < 0.01);
    assert!((jf64(&assignment["actualHours"]) - 0.0).abs() < 0.01);
}

#[tokio::test]
async fn test_assignment_lifecycle() {
    let (_state, app) = setup_resource_test().await;
    let (_, assignment) = setup_resource_with_assignment(&app).await;
    let id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/assignments/id/{}/activate", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(active["status"], "active");

    // Complete
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/assignments/id/{}/complete", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["status"], "completed");
}

#[tokio::test]
async fn test_cancel_assignment() {
    let (_state, app) = setup_resource_test().await;
    let (_, assignment) = setup_resource_with_assignment(&app).await;
    let id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/assignments/id/{}/cancel", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_assignments() {
    let (_state, app) = setup_resource_test().await;
    let _ = setup_resource_with_assignment(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resource/assignments")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Utilization Tests
// ============================================================================

#[tokio::test]
async fn test_create_utilization_entry() {
    let (_state, app) = setup_resource_test().await;
    let (profile, assignment) = setup_resource_with_assignment(&app).await;
    let resource_id = profile["id"].as_str().unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/utilization")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentId": assignment_id,
                    "resourceId": resource_id,
                    "entryDate": "2024-02-15",
                    "hoursWorked": 8.0,
                    "description": "Feature development",
                    "billable": true
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((jf64(&entry["hoursWorked"]) - 8.0).abs() < 0.01);
    assert_eq!(entry["status"], "submitted");
    assert_eq!(entry["billable"], true);
}

#[tokio::test]
async fn test_utilization_approval_workflow() {
    let (_state, app) = setup_resource_test().await;
    let (profile, assignment) = setup_resource_with_assignment(&app).await;
    let resource_id = profile["id"].as_str().unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create entry
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/utilization")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentId": assignment_id,
                    "resourceId": resource_id,
                    "entryDate": "2024-02-20",
                    "hoursWorked": 7.5,
                    "description": "Bug fixes"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entry_id = entry["id"].as_str().unwrap();

    // Approve
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/utilization/id/{}/approve", entry_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedBy"].is_string());
}

#[tokio::test]
async fn test_utilization_rejection() {
    let (_state, app) = setup_resource_test().await;
    let (profile, assignment) = setup_resource_with_assignment(&app).await;
    let resource_id = profile["id"].as_str().unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create entry
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/utilization")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentId": assignment_id,
                    "resourceId": resource_id,
                    "entryDate": "2024-02-25",
                    "hoursWorked": 4.0,
                    "description": "Questionable hours"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entry_id = entry["id"].as_str().unwrap();

    // Reject
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/utilization/id/{}/reject", entry_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_utilization_updates_assignment_hours() {
    let (_state, app) = setup_resource_test().await;
    let (profile, assignment) = setup_resource_with_assignment(&app).await;
    let resource_id = profile["id"].as_str().unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Log 8 hours
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/utilization")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentId": assignment_id,
                    "resourceId": resource_id,
                    "entryDate": "2024-02-10",
                    "hoursWorked": 8.0,
                    "description": "Sprint work"
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Check assignment has updated hours
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/resource/assignments/id/{}", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((jf64(&updated["actualHours"]) - 8.0).abs() < 0.01);
    // planned was 800, actual is 8, utilization = 8/800*100 = 1.0
    assert!((jf64(&updated["utilizationPercentage"]) - 1.0).abs() < 0.1);
}

#[tokio::test]
async fn test_list_utilization_entries() {
    let (_state, app) = setup_resource_test().await;
    let (profile, assignment) = setup_resource_with_assignment(&app).await;
    let resource_id = profile["id"].as_str().unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    for day in 1..=3 {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/utilization")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "assignmentId": assignment_id,
                        "resourceId": resource_id,
                        "entryDate": format!("2024-03-{:02}", day),
                        "hoursWorked": 8.0,
                        "description": format!("Day {} work", day)
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/resource/utilization?assignmentId={}", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_resource_dashboard() {
    let (_state, app) = setup_resource_test().await;
    create_test_profile(&app, "DASH-1", "Dashboard User 1", "employee").await;
    create_test_profile(&app, "DASH-2", "Dashboard User 2", "contractor").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resource/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalResources"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Full End-to-End Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_resource_full_lifecycle() {
    let (_state, app) = setup_resource_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create a resource profile
    let profile = create_test_profile(&app, "LIFE-RES", "Lifecycle Resource", "employee").await;
    let resource_id = profile["id"].as_str().unwrap();
    assert_eq!(profile["availabilityStatus"], "available");

    // 2. Create a resource request
    let request = create_test_request(&app, "LIFE-REQ", "Senior Developer", "high").await;
    let request_id = request["id"].as_str().unwrap();
    assert_eq!(request["status"], "draft");

    // 3. Submit the request
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/requests/id/{}/submit", request_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Create an assignment linking resource to the request
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/resource/assignments")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "assignmentNumber": "LIFE-ASGN",
                    "resourceId": resource_id,
                    "projectName": "Atlas Development",
                    "projectNumber": "PRJ-001",
                    "requestId": request_id,
                    "role": "Senior Developer",
                    "startDate": "2024-02-01",
                    "endDate": "2024-06-30",
                    "plannedHours": 800
                }))
                .unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let assignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let assignment_id = assignment["id"].as_str().unwrap();
    assert_eq!(assignment["status"], "planned");

    // 5. Fulfill the request
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/requests/id/{}/fulfill", request_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Activate the assignment
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/assignments/id/{}/activate", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 7. Log utilization
    for day in 10..=14 {
        let resp = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resource/utilization")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "assignmentId": assignment_id,
                        "resourceId": resource_id,
                        "entryDate": format!("2024-02-{}", day),
                        "hoursWorked": 8.0,
                        "description": format!("Week work day {}", day)
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // 8. Approve utilization entries
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/resource/utilization?assignmentId={}&status=submitted", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entries: serde_json::Value = serde_json::from_slice(&body).unwrap();
    for entry in entries["data"].as_array().unwrap() {
        let entry_id = entry["id"].as_str().unwrap();
        let resp = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/resource/utilization/id/{}/approve", entry_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // 9. Verify assignment hours are updated (5 days * 8 hours = 40 hours)
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/resource/assignments/id/{}", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated_assignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!((jf64(&updated_assignment["actualHours"]) - 40.0).abs() < 0.01);
    // 40/800*100 = 5.0%
    assert!((jf64(&updated_assignment["utilizationPercentage"]) - 5.0).abs() < 0.1);

    // 10. Complete the assignment
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/assignments/id/{}/complete", assignment_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 11. Mark resource as available again
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/resource/profiles/id/{}/availability", resource_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"availabilityStatus": "available"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 12. Check dashboard
    let resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/resource/dashboard")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalResources"].as_i64().unwrap() >= 1);
}
