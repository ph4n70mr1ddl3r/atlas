//! Project Billing E2E Tests (Oracle Fusion Cloud ERP)
//!
//! Tests for Oracle Fusion Cloud ERP Project Billing:
//! - Bill Rate Schedule CRUD and lifecycle (draft → active → inactive)
//! - Bill Rate Lines per schedule with date-effective lookups
//! - Project Billing Configurations per project
//! - Billing Events (milestone, progress, completion, retention_release)
//! - Project Invoices with full lifecycle (draft → submitted → approved → posted)
//! - Retention calculations
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_project_billing_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_schedule(
    app: &axum::Router,
    number: &str,
    name: &str,
    schedule_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "scheduleNumber": number,
        "name": name,
        "scheduleType": schedule_type,
        "currencyCode": "USD",
        "effectiveStart": "2024-01-01",
        "defaultMarkupPct": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create schedule: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_billing_config(
    app: &axum::Router,
    project_id: &str,
    billing_method: &str,
    contract_amount: f64,
    retention_pct: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": project_id,
        "billingMethod": billing_method,
        "contractAmount": contract_amount,
        "currencyCode": "USD",
        "invoiceFormat": "detailed",
        "billingCycle": "monthly",
        "paymentTermsDays": 30,
        "retentionPct": retention_pct,
        "retentionAmountCap": 0.0,
        "customerName": "Acme Corp",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/configs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create billing config: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_billing_event(
    app: &axum::Router,
    project_id: &str,
    event_number: &str,
    event_type: &str,
    amount: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": project_id,
        "eventNumber": event_number,
        "eventName": format!("Event {}", event_number),
        "eventType": event_type,
        "billingAmount": amount,
        "currencyCode": "USD",
        "completionPct": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/events")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create billing event: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_invoice(
    app: &axum::Router,
    invoice_number: &str,
    project_id: &str,
    invoice_type: &str,
    lines: Vec<serde_json::Value>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceNumber": invoice_number,
        "projectId": project_id,
        "invoiceType": invoice_type,
        "customerName": "Acme Corp",
        "lines": lines,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/invoices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create invoice: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

fn test_invoice_line(bill_amount: f64, role: &str) -> serde_json::Value {
    json!({
        "lineSource": "expenditure_item",
        "roleName": role,
        "expenditureType": "labor",
        "quantity": 40.0,
        "unitOfMeasure": "hours",
        "billRate": bill_amount / 40.0,
        "rawCostAmount": bill_amount * 0.8,
        "billAmount": bill_amount,
        "markupAmount": 0.0,
        "taxAmount": 0.0,
        "transactionDate": "2024-06-15",
    })
}

// ============================================================================
// Bill Rate Schedule Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_bill_rate_schedule() {
    let (_state, app) = setup_project_billing_test().await;

    let schedule = create_test_schedule(&app, "BRS-001", "Standard Rate Schedule", "standard").await;

    assert_eq!(schedule["schedule_number"], "BRS-001");
    assert_eq!(schedule["name"], "Standard Rate Schedule");
    assert_eq!(schedule["schedule_type"], "standard");
    assert_eq!(schedule["status"], "draft");
    assert_eq!(schedule["currency_code"], "USD");
}

#[tokio::test]
#[ignore]
async fn test_create_schedule_types() {
    let (_state, app) = setup_project_billing_test().await;

    let standard = create_test_schedule(&app, "BRS-STD", "Standard", "standard").await;
    assert_eq!(standard["schedule_type"], "standard");

    let overtime = create_test_schedule(&app, "BRS-OT", "Overtime", "overtime").await;
    assert_eq!(overtime["schedule_type"], "overtime");

    let holiday = create_test_schedule(&app, "BRS-HOL", "Holiday", "holiday").await;
    assert_eq!(holiday["schedule_type"], "holiday");

    let custom = create_test_schedule(&app, "BRS-CUS", "Custom", "custom").await;
    assert_eq!(custom["schedule_type"], "custom");
}

#[tokio::test]
#[ignore]
async fn test_list_schedules() {
    let (_state, app) = setup_project_billing_test().await;

    create_test_schedule(&app, "BRS-LIST-1", "Schedule 1", "standard").await;
    create_test_schedule(&app, "BRS-LIST-2", "Schedule 2", "overtime").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-billing/schedules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_activate_and_deactivate_schedule() {
    let (_state, app) = setup_project_billing_test().await;

    let schedule = create_test_schedule(&app, "BRS-ACT", "Activatable", "standard").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    assert_eq!(schedule["status"], "draft");

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/activate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(active["status"], "active");

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/deactivate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let inactive: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(inactive["status"], "inactive");
}

#[tokio::test]
#[ignore]
async fn test_delete_schedule() {
    let (_state, app) = setup_project_billing_test().await;

    create_test_schedule(&app, "BRS-DEL", "To Delete", "standard").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/project-billing/schedules/number/BRS-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-billing/schedules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let numbers: Vec<&str> = result["data"].as_array().unwrap().iter()
        .filter_map(|s| s["schedule_number"].as_str())
        .collect();
    assert!(!numbers.contains(&"BRS-DEL"));
}

#[tokio::test]
#[ignore]
async fn test_duplicate_schedule_rejected() {
    let (_state, app) = setup_project_billing_test().await;

    create_test_schedule(&app, "BRS-DUP", "First", "standard").await;

    // Try creating duplicate
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "scheduleNumber": "BRS-DUP",
        "name": "Duplicate",
        "scheduleType": "standard",
        "currencyCode": "USD",
        "effectiveStart": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Bill Rate Line Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_rate_lines() {
    let (_state, app) = setup_project_billing_test().await;

    let schedule = create_test_schedule(&app, "BRS-RL", "Rate Lines Test", "standard").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add Senior Developer rate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/rate-lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "roleName": "Senior Developer",
            "billRate": 150.0,
            "unitOfMeasure": "hours",
            "effectiveStart": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Add Project Manager rate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/rate-lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "roleName": "Project Manager",
            "billRate": 175.0,
            "unitOfMeasure": "hours",
            "effectiveStart": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/schedules/{}/rate-lines", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_find_rate_for_role() {
    let (_state, app) = setup_project_billing_test().await;

    let schedule = create_test_schedule(&app, "BRS-FIND", "Find Rate Test", "standard").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add a rate line
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/rate-lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "roleName": "Architect",
            "billRate": 200.0,
            "unitOfMeasure": "hours",
            "effectiveStart": "2024-01-01",
            "effectiveEnd": "2024-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Find rate
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/schedules/{}/find-rate/Architect?date=2024-06-15", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rate: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rate["role_name"], "Architect");
    let bill_rate: f64 = rate["bill_rate"].as_str().unwrap().parse().unwrap();
    assert!((bill_rate - 200.0).abs() < 0.01);

    // Find rate outside date range
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/schedules/{}/find-rate/Architect?date=2025-06-15", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Project Billing Config Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_billing_config() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let config = create_test_billing_config(&app, &project_id, "time_and_materials", 100000.0, 10.0).await;

    assert_eq!(config["billing_method"], "time_and_materials");
    assert_eq!(config["status"], "draft");
    let contract: f64 = config["contract_amount"].as_str().unwrap().parse().unwrap();
    assert!((contract - 100000.0).abs() < 0.01);
    let ret_pct: f64 = config["retention_pct"].as_str().unwrap().parse().unwrap();
    assert!((ret_pct - 10.0).abs() < 0.01);
}

#[tokio::test]
#[ignore]
async fn test_billing_config_methods() {
    let (_state, app) = setup_project_billing_test().await;

    // Time & Materials
    let project_a = uuid::Uuid::new_v4().to_string();
    let schedule = create_test_schedule(&app, "BRS-TM", "T&M Rates", "standard").await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": project_a,
        "billingMethod": "time_and_materials",
        "billRateScheduleId": schedule["id"],
        "contractAmount": 50000.0,
        "currencyCode": "USD",
        "invoiceFormat": "detailed",
        "billingCycle": "monthly",
        "paymentTermsDays": 30,
        "retentionPct": 5.0,
        "retentionAmountCap": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/configs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Fixed Price
    let project_b = uuid::Uuid::new_v4().to_string();
    let config_b = create_test_billing_config(&app, &project_b, "fixed_price", 250000.0, 0.0).await;
    assert_eq!(config_b["billing_method"], "fixed_price");

    // Milestone
    let project_c = uuid::Uuid::new_v4().to_string();
    let config_c = create_test_billing_config(&app, &project_c, "milestone", 500000.0, 15.0).await;
    assert_eq!(config_c["billing_method"], "milestone");
}

#[tokio::test]
#[ignore]
async fn test_activate_and_cancel_billing_config() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let config = create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;
    let config_id = config["id"].as_str().unwrap();
    assert_eq!(config["status"], "draft");

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/configs/{}/activate", config_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(active["status"], "active");

    // Cancel (create a new one to cancel)
    let project_id2 = uuid::Uuid::new_v4().to_string();
    let config2 = create_test_billing_config(&app, &project_id2, "cost_plus", 75000.0, 5.0).await;
    let config2_id = config2["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/configs/{}/cancel", config2_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
#[ignore]
async fn test_get_billing_config_by_project() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4();

    create_test_billing_config(&app, &project_id.to_string(), "fixed_price", 100000.0, 0.0).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/configs/project/{}", project_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let config: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(config["billing_method"], "fixed_price");
}

#[tokio::test]
#[ignore]
async fn test_duplicate_billing_config_rejected() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    // Try creating duplicate
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": project_id,
        "billingMethod": "milestone",
        "contractAmount": 200000.0,
        "currencyCode": "USD",
        "invoiceFormat": "summary",
        "billingCycle": "milestone",
        "paymentTermsDays": 30,
        "retentionPct": 0.0,
        "retentionAmountCap": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/configs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Billing Event Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_billing_event() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let event = create_test_billing_event(&app, &project_id, "BE-001", "milestone", 25000.0).await;

    assert_eq!(event["event_number"], "BE-001");
    assert_eq!(event["event_type"], "milestone");
    assert_eq!(event["status"], "planned");
    let amount: f64 = event["billing_amount"].as_str().unwrap().parse().unwrap();
    assert!((amount - 25000.0).abs() < 0.01);
}

#[tokio::test]
#[ignore]
async fn test_create_event_types() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let milestone = create_test_billing_event(&app, &project_id, "BE-MS", "milestone", 25000.0).await;
    assert_eq!(milestone["event_type"], "milestone");

    let progress = create_test_billing_event(&app, &project_id, "BE-PRG", "progress", 15000.0).await;
    assert_eq!(progress["event_type"], "progress");

    let completion = create_test_billing_event(&app, &project_id, "BE-CMP", "completion", 50000.0).await;
    assert_eq!(completion["event_type"], "completion");

    let retention = create_test_billing_event(&app, &project_id, "BE-RET", "retention_release", 10000.0).await;
    assert_eq!(retention["event_type"], "retention_release");
}

#[tokio::test]
#[ignore]
async fn test_complete_billing_event() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let event = create_test_billing_event(&app, &project_id, "BE-COMP", "milestone", 25000.0).await;
    let event_id = event["id"].as_str().unwrap();
    assert_eq!(event["status"], "planned");

    // Complete the event
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/events/{}/complete", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualDate": "2024-06-15",
            "completionPct": 100.0,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "ready");
    let pct: f64 = completed["completion_pct"].as_str().unwrap().parse().unwrap();
    assert!((pct - 100.0).abs() < 0.01);
}

#[tokio::test]
#[ignore]
async fn test_cancel_billing_event() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let event = create_test_billing_event(&app, &project_id, "BE-CANC", "milestone", 5000.0).await;
    let event_id = event["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/events/{}/cancel", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
#[ignore]
async fn test_list_billing_events_by_project() {
    let (_state, app) = setup_project_billing_test().await;
    let project_a = uuid::Uuid::new_v4();
    let project_b = uuid::Uuid::new_v4();

    create_test_billing_event(&app, &project_a.to_string(), "BE-LA1", "milestone", 10000.0).await;
    create_test_billing_event(&app, &project_a.to_string(), "BE-LA2", "progress", 5000.0).await;
    create_test_billing_event(&app, &project_b.to_string(), "BE-LB1", "milestone", 20000.0).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/events?project_id={}", project_a))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Invoice Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    // Create billing config first
    create_test_billing_config(&app, &project_id, "time_and_materials", 100000.0, 10.0).await;

    let invoice = create_test_invoice(&app, "INV-001", &project_id, "t_and_m", vec![
        test_invoice_line(10000.0, "Senior Developer"),
    ]).await;

    assert_eq!(invoice["invoice_number"], "INV-001");
    assert_eq!(invoice["invoice_type"], "t_and_m");
    assert_eq!(invoice["status"], "draft");
}

#[tokio::test]
#[ignore]
async fn test_invoice_with_retention() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    // 10% retention
    create_test_billing_config(&app, &project_id, "time_and_materials", 100000.0, 10.0).await;

    let invoice = create_test_invoice(&app, "INV-RET", &project_id, "t_and_m", vec![
        test_invoice_line(10000.0, "Senior Developer"),
    ]).await;

    // Retention should be 10% of 10000 = 1000
    let retention: f64 = invoice["retention_held"].as_str().unwrap().parse().unwrap();
    assert!((retention - 1000.0).abs() < 1.0, "Retention should be ~1000, got {}", retention);

    // Total = invoice_amount - retention
    let total: f64 = invoice["total_amount"].as_str().unwrap().parse().unwrap();
    assert!((total - 9000.0).abs() < 1.0, "Total should be ~9000, got {}", total);
}

#[tokio::test]
#[ignore]
async fn test_invoice_lifecycle_draft_to_posted() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-LC", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();
    assert_eq!(invoice["status"], "draft");

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/approve", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");

    // Post to GL
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/post", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "posted");
    assert_eq!(posted["gl_posted_flag"], true);
}

#[tokio::test]
#[ignore]
async fn test_reject_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-REJ", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/reject", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Incorrect billing amount"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejected_reason"], "Incorrect billing amount");
}

#[tokio::test]
#[ignore]
async fn test_cancel_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-CANC", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/cancel", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
#[ignore]
async fn test_cannot_cancel_posted_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-POST-CANC", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit → Approve → Post
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/approve", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/post", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel posted invoice
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/cancel", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_get_invoice_lines() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "time_and_materials", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-LINES", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Senior Developer"),
        test_invoice_line(3000.0, "Junior Developer"),
        test_invoice_line(2000.0, "QA Engineer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    // Get lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/invoices/{}/lines", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
#[ignore]
async fn test_list_invoices_by_project() {
    let (_state, app) = setup_project_billing_test().await;
    let project_a = uuid::Uuid::new_v4();
    let project_b = uuid::Uuid::new_v4();

    create_test_billing_config(&app, &project_a.to_string(), "time_and_materials", 100000.0, 0.0).await;
    create_test_billing_config(&app, &project_b.to_string(), "time_and_materials", 200000.0, 0.0).await;

    create_test_invoice(&app, "INV-PA1", &project_a.to_string(), "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    create_test_invoice(&app, "INV-PA2", &project_a.to_string(), "t_and_m", vec![
        test_invoice_line(3000.0, "Developer"),
    ]).await;
    create_test_invoice(&app, "INV-PB1", &project_b.to_string(), "t_and_m", vec![
        test_invoice_line(10000.0, "Architect"),
    ]).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/invoices?project_id={}", project_a))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_create_schedule_with_invalid_type() {
    let (_state, app) = setup_project_billing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "scheduleNumber": "BRS-INVALID",
        "name": "Invalid Type",
        "scheduleType": "nonexistent",
        "currencyCode": "USD",
        "effectiveStart": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_config_with_invalid_method() {
    let (_state, app) = setup_project_billing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": uuid::Uuid::new_v4().to_string(),
        "billingMethod": "invalid_method",
        "contractAmount": 100000.0,
        "currencyCode": "USD",
        "invoiceFormat": "detailed",
        "billingCycle": "monthly",
        "paymentTermsDays": 30,
        "retentionPct": 0.0,
        "retentionAmountCap": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/configs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_event_with_invalid_type() {
    let (_state, app) = setup_project_billing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "projectId": uuid::Uuid::new_v4().to_string(),
        "eventNumber": "BE-INV",
        "eventName": "Invalid Event",
        "eventType": "nonexistent",
        "billingAmount": 5000.0,
        "currencyCode": "USD",
        "completionPct": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/events")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_submit_non_draft_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-NS", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit first
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to submit again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_approve_non_submitted_invoice() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_billing_config(&app, &project_id, "fixed_price", 100000.0, 0.0).await;

    let invoice = create_test_invoice(&app, "INV-NA", &project_id, "t_and_m", vec![
        test_invoice_line(5000.0, "Developer"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    // Try to approve without submitting
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/approve", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_complete_non_planned_event() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let event = create_test_billing_event(&app, &project_id, "BE-NCP", "milestone", 5000.0).await;
    let event_id = event["id"].as_str().unwrap();

    // Cancel it first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/events/{}/cancel", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to complete a cancelled event
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/events/{}/complete", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualDate": "2024-06-15",
            "completionPct": 100.0,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_invoice_without_lines() {
    let (_state, app) = setup_project_billing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceNumber": "INV-EMPTY",
        "projectId": uuid::Uuid::new_v4().to_string(),
        "invoiceType": "t_and_m",
        "lines": [],
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-billing/invoices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_project_billing_dashboard() {
    let (_state, app) = setup_project_billing_test().await;
    let project_a = uuid::Uuid::new_v4();
    let project_b = uuid::Uuid::new_v4();

    // Create billing configs
    create_test_billing_config(&app, &project_a.to_string(), "time_and_materials", 100000.0, 10.0).await;
    create_test_billing_config(&app, &project_b.to_string(), "fixed_price", 200000.0, 0.0).await;

    // Create invoices
    create_test_invoice(&app, "INV-D1", &project_a.to_string(), "t_and_m", vec![
        test_invoice_line(10000.0, "Developer"),
    ]).await;
    create_test_invoice(&app, "INV-D2", &project_b.to_string(), "t_and_m", vec![
        test_invoice_line(25000.0, "Architect"),
    ]).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-billing/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert!(dashboard["total_projects_billable"].as_i64().unwrap() >= 2);
    assert!(dashboard["total_invoices"].as_i64().unwrap() >= 2);
    assert!(dashboard["total_contract_value"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
    assert!(dashboard["total_billed"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
}

// ============================================================================
// Full Lifecycle Integration Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_project_billing_full_lifecycle() {
    let (_state, app) = setup_project_billing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    // 1. Create a bill rate schedule
    let schedule = create_test_schedule(&app, "BRS-LC", "Lifecycle Schedule", "standard").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    // 2. Add rate lines
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/rate-lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "roleName": "Consultant",
            "billRate": 200.0,
            "unitOfMeasure": "hours",
            "effectiveStart": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 3. Activate schedule
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/schedules/{}/activate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // 4. Create billing config with retention
    let config = create_test_billing_config(&app, &project_id, "time_and_materials", 100000.0, 10.0).await;
    let config_id = config["id"].as_str().unwrap();

    // 5. Activate billing config
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/configs/{}/activate", config_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // 6. Create billing events
    let event1 = create_test_billing_event(&app, &project_id, "BE-LC1", "milestone", 25000.0).await;
    let event1_id = event1["id"].as_str().unwrap();

    // 7. Complete a billing event
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/events/{}/complete", event1_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualDate": "2024-03-15",
            "completionPct": 100.0,
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 8. Create an invoice
    let invoice = create_test_invoice(&app, "INV-LC1", &project_id, "t_and_m", vec![
        test_invoice_line(10000.0, "Consultant"),
        test_invoice_line(5000.0, "Analyst"),
    ]).await;
    let invoice_id = invoice["id"].as_str().unwrap();
    assert_eq!(invoice["status"], "draft");

    // Verify retention on invoice (10% of 15000 = 1500)
    let retention: f64 = invoice["retention_held"].as_str().unwrap().parse().unwrap();
    assert!((retention - 1500.0).abs() < 1.0);

    // 9. Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/submit", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 10. Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/approve", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 11. Post to GL
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-billing/invoices/{}/post", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "posted");
    assert_eq!(posted["gl_posted_flag"], true);

    // 12. Verify invoice lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-billing/invoices/{}/lines", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines_result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines_result["data"].as_array().unwrap().len(), 2);

    // 13. Verify dashboard reflects our work
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-billing/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["total_projects_billable"].as_i64().unwrap() >= 1);
    assert!(dashboard["posted_invoices"].as_i64().unwrap() >= 1);
}
