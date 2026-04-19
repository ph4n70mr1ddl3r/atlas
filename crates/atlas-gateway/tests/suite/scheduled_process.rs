//! Scheduled Processes E2E tests
//!
//! Oracle Fusion Cloud ERP: Navigator > Tools > Scheduled Processes

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

use super::common::helpers::*;

// ============================================================================
// Helpers
// ============================================================================

async fn setup() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_template(app: &axum::Router, k: &str, v: &str, code: &str) -> serde_json::Value {
    let body = json!({
        "code": code,
        "name": format!("{} Template", code),
        "description": "Test process template",
        "process_type": "report",
        "executor_type": "built_in",
        "executor_config": {},
        "parameters": [],
        "default_parameters": {},
        "timeout_minutes": 60,
        "max_retries": 0,
        "retry_delay_minutes": 5,
        "requires_approval": false
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes/templates")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Template creation failed");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn submit_process(app: &axum::Router, k: &str, v: &str, name: &str) -> serde_json::Value {
    let body = json!({
        "process_name": name,
        "process_type": "report",
        "priority": "normal",
        "parameters": {}
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Process submission failed");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Template Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_template() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let template = create_template(&app, &k, &v, "TEST-RPT-001").await;
    assert_eq!(template["code"], "TEST-RPT-001");
    assert_eq!(template["name"], "TEST-RPT-001 Template");
    assert_eq!(template["process_type"], "report");
    assert_eq!(template["is_active"], true);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_templates() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "LIST-RPT-001").await;
    create_template(&app, &k, &v, "LIST-RPT-002").await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/scheduled-processes/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["total"].as_i64().unwrap() >= 2);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_template() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "GET-RPT-001").await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/scheduled-processes/templates/GET-RPT-001")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let template: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(template["code"], "GET-RPT-001");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_deactivate_template() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let template = create_template(&app, &k, &v, "DEACT-RPT-001").await;
    let id = template["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/templates/{}/deactivate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["is_active"], false);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_template() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "DEL-RPT-001").await;

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/scheduled-processes/templates/DEL-RPT-001")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Process Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_submit_process() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let process = submit_process(&app, &k, &v, "Test Report Run").await;
    assert_eq!(process["process_name"], "Test Report Run");
    assert_eq!(process["status"], "pending");
    assert_eq!(process["priority"], "normal");
    assert_eq!(process["progress_percent"], 0);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_process_full_lifecycle() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let process = submit_process(&app, &k, &v, "Lifecycle Test").await;
    let id = process["id"].as_str().unwrap();

    // Start the process
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/start", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let started: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(started["status"], "running");

    // Update progress
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/progress", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"progress_percent": 50}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Complete the process
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/complete", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"result_summary": "Report generated successfully", "log_output": "All steps completed"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["progress_percent"], 100);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_cancel_process() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let process = submit_process(&app, &k, &v, "Cancel Test").await;
    let id = process["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/cancel", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"reason": "No longer needed"}"#)).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["cancel_reason"], "No longer needed");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_processes() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    submit_process(&app, &k, &v, "List Test 1").await;
    submit_process(&app, &k, &v, "List Test 2").await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/scheduled-processes?status=pending")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["total"].as_i64().unwrap() >= 2);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_submit_scheduled_process() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());

    let scheduled_time = chrono::Utc::now() + chrono::Duration::hours(1);
    let body = json!({
        "process_name": "Scheduled Report",
        "process_type": "report",
        "priority": "low",
        "scheduled_start_at": scheduled_time.to_rfc3339(),
        "parameters": {}
    });

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let process: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(process["status"], "scheduled");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_process_logs() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    let process = submit_process(&app, &k, &v, "Log Test").await;
    let id = process["id"].as_str().unwrap();

    // Start process so it's running
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/start", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await;

    // Add log entry
    let log_body = json!({
        "log_level": "info",
        "message": "Processing step 1 completed",
        "step_name": "data_extraction",
        "duration_ms": 1500
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/logs", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&log_body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List logs
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/scheduled-processes/{}/logs", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let logs: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(logs["total"].as_i64().unwrap() >= 1);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_submit_with_template() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "TMPL-PROC-001").await;

    let body = json!({
        "template_code": "TMPL-PROC-001",
        "process_name": "Template-based Report",
        "process_type": "report",
        "priority": "normal",
        "parameters": {}
    });

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let process: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(process["template_code"], "TMPL-PROC-001");
    assert_eq!(process["status"], "pending");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_dashboard() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    submit_process(&app, &k, &v, "Dashboard Test 1").await;
    submit_process(&app, &k, &v, "Dashboard Test 2").await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/scheduled-processes/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["total_processes"].as_i64().unwrap() >= 2);
    assert!(dashboard["pending_processes"].as_i64().unwrap() >= 2);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_create_recurrence() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "REC-TMPL-001").await;

    let body = json!({
        "name": "Daily Sales Report",
        "description": "Runs every day at 8am",
        "template_code": "REC-TMPL-001",
        "parameters": {},
        "recurrence_type": "daily",
        "recurrence_config": { "time": "08:00" },
        "start_date": "2024-01-01"
    });

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes/recurrences")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let recurrence: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(recurrence["name"], "Daily Sales Report");
    assert_eq!(recurrence["recurrence_type"], "daily");
    assert_eq!(recurrence["is_active"], true);
    assert_eq!(recurrence["run_count"], 0);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_template_requires_approval() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());

    // Create a template that requires approval
    let body = json!({
        "code": "APPROVAL-RPT-001",
        "name": "Approval Required Report",
        "process_type": "batch",
        "executor_type": "built_in",
        "timeout_minutes": 60,
        "requires_approval": true
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Submit a process using this template
    let body = json!({
        "template_code": "APPROVAL-RPT-001",
        "process_name": "Needs Approval",
        "process_type": "batch",
        "priority": "normal",
        "parameters": {}
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let process: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(process["status"], "waiting_for_approval");

    // Approve the process
    let id = process["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/scheduled-processes/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "pending");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_duplicate_template_rejected() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());
    create_template(&app, &k, &v, "DUP-RPT-001").await;

    // Try to create a duplicate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP-RPT-001",
            "name": "Duplicate",
            "process_type": "report"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_invalid_process_type_rejected() {
    let (state, app) = setup().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/scheduled-processes/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID-TYPE-001",
            "name": "Invalid Type",
            "process_type": "invalid_type"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}
