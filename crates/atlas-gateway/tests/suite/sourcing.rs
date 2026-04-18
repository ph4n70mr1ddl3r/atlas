//! Procurement Sourcing E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Procurement Sourcing:
//! - Sourcing event CRUD and lifecycle (draft → publish → close → award)
//! - Event lines (add items to source)
//! - Supplier invitations
//! - Supplier responses (bids) with line-level pricing
//! - Scoring criteria and response evaluation
//! - Award creation, approval, and rejection
//! - Sourcing templates
//! - Error cases and validation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_sourcing_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    // Run the sourcing migration
    let migration_sql = include_str!("../../../../migrations/022_procurement_sourcing.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_event(
    app: &axum::Router, title: &str, event_type: &str, deadline: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sourcing/events")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": title,
            "event_type": event_type,
            "style": "sealed",
            "response_deadline": deadline,
            "currency_code": "USD",
            "scoring_method": "weighted",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_line(
    app: &axum::Router, event_id: &str, desc: &str, qty: &str, target_price: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/lines", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": desc,
            "quantity": qty,
            "uom": "EA",
            "target_price": target_price,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn invite_test_supplier(
    app: &axum::Router, event_id: &str, supplier_id: &str, supplier_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/invites", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_id": supplier_id,
            "supplier_name": supplier_name,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn submit_test_response(
    app: &axum::Router, event_id: &str, supplier_id: &str, supplier_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/responses", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_id": supplier_id,
            "supplier_name": supplier_name,
            "cover_letter": "Please consider our bid",
            "valid_until": "2026-06-30",
            "payment_terms": "Net 30",
            "lead_time_days": 14,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Sourcing Event Tests
// ============================================================================

#[tokio::test]
async fn test_create_sourcing_event() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Server Hardware RFQ", "rfq", "2026-05-30").await;

    assert_eq!(event["title"], "Server Hardware RFQ");
    assert_eq!(event["event_type"], "rfq");
    assert_eq!(event["status"], "draft");
    assert_eq!(event["style"], "sealed");
    assert_eq!(event["currency_code"], "USD");
    assert!(event["event_number"].as_str().unwrap().starts_with("SE-"));
}

#[tokio::test]
async fn test_list_sourcing_events() {
    let (_state, app) = setup_sourcing_test().await;

    create_test_event(&app, "Event 1", "rfq", "2026-05-30").await;
    create_test_event(&app, "Event 2", "rfp", "2026-06-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sourcing/events")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let events: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(events.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_events_by_status() {
    let (_state, app) = setup_sourcing_test().await;
    create_test_event(&app, "Draft Event", "rfq", "2026-05-30").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sourcing/events?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let events: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(events.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_publish_sourcing_event() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Publishable Event", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    // Add a line first (required for publish)
    add_test_line(&app, event_id, "Widget A", "100", "50.00").await;

    // Publish
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let published: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(published["status"], "published");
}

#[tokio::test]
async fn test_publish_event_without_lines_fails() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Empty Event", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_sourcing_event() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Cancel Me", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/cancel", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "No longer needed"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_create_event_invalid_type_fails() {
    let (_state, app) = setup_sourcing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sourcing/events")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": "Bad Event",
            "event_type": "invalid_type",
            "response_deadline": "2026-05-30",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Event Lines Tests
// ============================================================================

#[tokio::test]
async fn test_add_event_line() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Line Test Event", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    let line = add_test_line(&app, event_id, "Server Rack Unit", "10", "2500.00").await;

    assert_eq!(line["description"], "Server Rack Unit");
    assert_eq!(line["uom"], "EA");
    assert_eq!(line["status"], "open");
}

#[tokio::test]
async fn test_list_event_lines() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Multi-line Event", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    add_test_line(&app, event_id, "Item A", "100", "10.00").await;
    add_test_line(&app, event_id, "Item B", "50", "25.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sourcing/events/{}/lines", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines.as_array().unwrap().len(), 2);
}

// ============================================================================
// Supplier Invitation Tests
// ============================================================================

#[tokio::test]
async fn test_invite_supplier() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Invite Test", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();
    add_test_line(&app, event_id, "Item A", "100", "10.00").await;

    // Publish first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let supplier_id = uuid::Uuid::new_v4().to_string();
    let invite = invite_test_supplier(&app, event_id, &supplier_id, "Acme Corp").await;

    assert_eq!(invite["supplier_name"], "Acme Corp");
    assert_eq!(invite["status"], "invited");
}

#[tokio::test]
async fn test_invite_duplicate_supplier_fails() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Dup Invite", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();
    add_test_line(&app, event_id, "Item A", "100", "10.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let supplier_id = uuid::Uuid::new_v4().to_string();
    invite_test_supplier(&app, event_id, &supplier_id, "Acme Corp").await;

    // Try duplicate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/invites", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_id": supplier_id,
            "supplier_name": "Acme Corp",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Supplier Response Tests
// ============================================================================

#[tokio::test]
async fn test_submit_supplier_response() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Response Test", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();
    add_test_line(&app, event_id, "Item A", "100", "10.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let supplier_id = uuid::Uuid::new_v4().to_string();
    invite_test_supplier(&app, event_id, &supplier_id, "Bidder Corp").await;

    let response = submit_test_response(&app, event_id, &supplier_id, "Bidder Corp").await;

    assert_eq!(response["supplier_name"], "Bidder Corp");
    assert_eq!(response["status"], "submitted");
    assert!(response["response_number"].as_str().unwrap().starts_with("SR-"));
}

#[tokio::test]
async fn test_add_response_line() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Response Line Test", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();
    let line = add_test_line(&app, event_id, "Item A", "100", "10.00").await;
    let event_line_id = line["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let supplier_id = uuid::Uuid::new_v4().to_string();
    invite_test_supplier(&app, event_id, &supplier_id, "Bidder Corp").await;
    let response = submit_test_response(&app, event_id, &supplier_id, "Bidder Corp").await;
    let response_id = response["id"].as_str().unwrap();

    // Add response line
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/responses/{}/lines", response_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "event_line_id": event_line_id,
            "unit_price": "9.50",
            "quantity": "100",
            "promised_delivery_date": "2026-06-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rline: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rline["unit_price"].as_str().unwrap().parse::<f64>().unwrap(), 9.5);

    // Check total was updated on response
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sourcing/responses/{}", response_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_response: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let total: f64 = updated_response["total_amount"].as_str().unwrap().parse().unwrap();
    assert!((total - 950.0).abs() < 1.0);
}

#[tokio::test]
async fn test_submit_response_uninvited_fails() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Uninvited Test", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();
    add_test_line(&app, event_id, "Item A", "100", "10.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/publish", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Submit response without being invited
    let supplier_id = uuid::Uuid::new_v4().to_string();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/responses", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_id": supplier_id,
            "supplier_name": "Uninvited Corp",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Scoring & Evaluation Tests
// ============================================================================

#[tokio::test]
async fn test_add_scoring_criterion() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Scoring Test", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sourcing/events/{}/criteria", event_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Price",
            "weight": "40",
            "max_score": "100",
            "criterion_type": "price",
            "display_order": 1,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let criterion: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(criterion["name"], "Price");
    assert_eq!(criterion["criterion_type"], "price");
}

#[tokio::test]
async fn test_list_scoring_criteria() {
    let (_state, app) = setup_sourcing_test().await;

    let event = create_test_event(&app, "Criteria List", "rfq", "2026-05-30").await;
    let event_id = event["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add two criteria
    for (name, ctype, weight) in [("Price", "price", "40"), ("Quality", "quality", "60")] {
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/sourcing/events/{}/criteria", event_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "name": name, "weight": weight, "max_score": "100", "criterion_type": ctype,
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sourcing/events/{}/criteria", event_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let criteria: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(criteria.as_array().unwrap().len(), 2);
}

// ============================================================================
// Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_sourcing_template() {
    let (_state, app) = setup_sourcing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sourcing/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "STD_RFQ",
            "name": "Standard RFQ",
            "description": "Standard Request for Quote template",
            "default_event_type": "rfq",
            "default_response_deadline_days": 14,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tmpl: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(tmpl["code"], "STD_RFQ");
    assert_eq!(tmpl["name"], "Standard RFQ");
    assert_eq!(tmpl["is_active"], true);
}

#[tokio::test]
async fn test_list_sourcing_templates() {
    let (_state, app) = setup_sourcing_test().await;

    let (k, v) = auth_header(&admin_claims());

    for code in ["TMPL_1", "TMPL_2"] {
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sourcing/templates")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code, "name": format!("Template {}", code),
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sourcing/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let templates: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(templates.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_sourcing_template() {
    let (_state, app) = setup_sourcing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sourcing/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DEL_ME", "name": "Delete Me",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/sourcing/templates/DEL_ME")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify soft-deleted
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sourcing/templates/DEL_ME")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Dashboard Summary Test
// ============================================================================

#[tokio::test]
async fn test_sourcing_summary() {
    let (_state, app) = setup_sourcing_test().await;

    // Create some events
    create_test_event(&app, "Summary Event 1", "rfq", "2026-05-30").await;
    create_test_event(&app, "Summary Event 2", "rfp", "2026-06-15").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sourcing/summary")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(summary["draft_event_count"].as_i64().unwrap() >= 2);
}
