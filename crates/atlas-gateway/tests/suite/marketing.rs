//! Marketing Campaign Management E2E Tests
//!
//! Tests for Oracle Fusion CX Marketing:
//! - Campaign type CRUD
//! - Campaign lifecycle (create → activate → pause → complete)
//! - Campaign cancellation
//! - Campaign members (add → update status)
//! - Campaign responses (create, list, delete)
//! - Marketing dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);
fn unique_id() -> u64 { COUNTER.fetch_add(1, Ordering::Relaxed) }

async fn setup_marketing_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean marketing test data
    sqlx::query("DELETE FROM _atlas.campaign_responses").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.campaign_members").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.marketing_campaigns").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.campaign_types").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_campaign_type(app: &axum::Router, code: &str, channel: &str) -> serde_json::Value {
    let uid = unique_id();
    let code = format!("{}-{}", code, uid);
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaign-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Type", code),
            "description": "Test campaign type",
            "channel": channel
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 for campaign type, got {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_campaign(app: &axum::Router, campaign_number: &str, channel: &str, budget: &str) -> serde_json::Value {
    let uid = unique_id();
    let campaign_number = format!("{}-{}", campaign_number, uid);
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": campaign_number,
            "name": format!("Campaign {}", campaign_number),
            "channel": channel,
            "budget": budget,
            "currency_code": "USD",
            "expected_responses": 100,
            "expected_revenue": "50000",
            "start_date": "2025-01-01",
            "end_date": "2025-03-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Campaign Type Tests
// ============================================================================

#[tokio::test]
async fn test_create_campaign_type() {
    let (_state, app) = setup_marketing_test().await;
    let ct = create_test_campaign_type(&app, "EMAIL_BLAST", "email").await;
    assert!(ct["code"].as_str().unwrap().starts_with("EMAIL_BLAST"));
    assert!(ct["name"].as_str().unwrap().contains("Type"));
    assert_eq!(ct["channel"], "email");
    assert!(ct["id"].is_string());
}

#[tokio::test]
async fn test_create_campaign_type_invalid_channel() {
    let (_state, app) = setup_marketing_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaign-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Type",
            "channel": "fax"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_campaign_type_duplicate() {
    let (_state, app) = setup_marketing_test().await;
    let uid = unique_id();
    let code = format!("DUP_CT-{}", uid);
    // Create first one directly (without extra uid)
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaign-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Type", code),
            "channel": "email"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    // Try same code again
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaign-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_campaign_types() {
    let (_state, app) = setup_marketing_test().await;
    create_test_campaign_type(&app, "CT-A", "email").await;
    create_test_campaign_type(&app, "CT-B", "social").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/marketing/campaign-types").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_campaign_type() {
    let (_state, app) = setup_marketing_test().await;
    let ct = create_test_campaign_type(&app, "DEL-CT", "email").await;
    let code = ct["code"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/marketing/campaign-types/{}", code)).header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Campaign Tests
// ============================================================================

#[tokio::test]
async fn test_create_campaign() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-001", "email", "10000").await;
    assert!(campaign["campaignNumber"].as_str().unwrap().starts_with("MKT-001"));
    assert!(campaign["name"].as_str().unwrap().contains("MKT-001"));
    assert_eq!(campaign["status"], "draft");
    assert_eq!(campaign["channel"], "email");
    assert!(campaign["budget"].as_str().unwrap().contains("10000"));
    assert!(campaign["id"].is_string());
}

#[tokio::test]
async fn test_create_campaign_invalid_channel() {
    let (_state, app) = setup_marketing_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": "MKT-BAD",
            "name": "Bad Campaign",
            "channel": "fax"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_campaign_duplicate() {
    let (_state, app) = setup_marketing_test().await;
    let uid = unique_id();
    let cnum = format!("MKT-DUP-{}", uid);
    // Create first directly
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": cnum,
            "name": format!("Campaign {}", cnum),
            "channel": "email",
            "budget": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    // Try same number again
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": cnum,
            "name": "Duplicate",
            "channel": "email"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_campaign() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-GET", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let cnum = campaign["campaignNumber"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/marketing/campaigns/{}", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["campaignNumber"], cnum);
}

#[tokio::test]
async fn test_list_campaigns() {
    let (_state, app) = setup_marketing_test().await;
    create_test_campaign(&app, "MKT-LA", "email", "5000").await;
    create_test_campaign(&app, "MKT-LB", "social", "7500").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/marketing/campaigns?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Campaign Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_campaign_lifecycle_activate_pause_complete() {
    let (_state, app) = setup_marketing_test().await;
    let uid = unique_id();
    let cnum = format!("MKT-LC-{}", uid);
    let (k, v) = auth_header(&admin_claims());
    // Create directly to ensure we control the ID
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": cnum,
            "name": format!("Campaign {}", cnum),
            "channel": "email",
            "budget": "20000",
            "currency_code": "USD",
            "expected_responses": 100,
            "expected_revenue": "50000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create campaign");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let campaign: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = campaign["id"].as_str().unwrap();

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(active["status"], "active");
    assert!(active["activatedAt"].is_string());

    // Pause
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/pause", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let paused: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(paused["status"], "paused");

    // Resume (activate again from paused)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK, "Expected OK on resume from paused, got {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resumed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resumed["status"], "active");

    // Complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/complete", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK, "Expected OK on complete, got {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
    assert!(completed["completedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_campaign() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-CANCEL", "email", "5000").await;
    let id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert!(cancelled["cancelledAt"].is_string());
}

#[tokio::test]
async fn test_activate_completed_campaign_rejected() {
    let (_state, app) = setup_marketing_test().await;
    let uid = unique_id();
    let cnum = format!("MKT-NOR-{}", uid);
    // Create directly to avoid uid double-suffix
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/marketing/campaigns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "campaign_number": cnum,
            "name": format!("Campaign {}", cnum),
            "channel": "email",
            "budget": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let campaign: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = campaign["id"].as_str().unwrap();

    // Activate then complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK, "Expected OK on activate, got {}", r.status());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/complete", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK, "Expected OK on complete, got {}", r.status());

    // Try to activate completed campaign
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_campaign() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-DEL", "email", "3000").await;
    let id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/marketing/campaigns/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Campaign Member Tests
// ============================================================================

#[tokio::test]
async fn test_add_campaign_member() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-MEM", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/members", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contact_name": "Jane Smith",
            "contact_email": "jane@example.com"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let member: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(member["contactName"], "Jane Smith");
    assert_eq!(member["contactEmail"], "jane@example.com");
    assert_eq!(member["status"], "invited");
}

#[tokio::test]
async fn test_list_campaign_members() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-LMEM", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add two members
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/marketing/campaigns/{}/members", campaign_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "contact_name": format!("Member {}", i),
                "contact_email": format!("member{}@example.com", i)
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/marketing/campaigns/{}/members", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_member_status() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-MSTS", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add member
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/members", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contact_name": "Bob Johnson",
            "contact_email": "bob@example.com"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let member: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let member_id = member["id"].as_str().unwrap();

    // Update to responded
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/members/{}/status", member_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "responded",
            "response": "interested"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "responded");
    assert_eq!(updated["response"], "interested");
    assert!(updated["respondedAt"].is_string());
}

#[tokio::test]
async fn test_delete_campaign_member() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-DMEM", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/members", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contact_name": "Delete Me",
            "contact_email": "delete@example.com"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let member: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let member_id = member["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/marketing/members/{}", member_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Campaign Response Tests
// ============================================================================

#[tokio::test]
async fn test_create_campaign_response() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-RESP", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // First activate campaign
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Record response
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/responses", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "response_type": "clicked",
            "contact_name": "Click User",
            "contact_email": "click@example.com",
            "description": "Clicked email link",
            "value": "0",
            "source_url": "https://example.com/landing"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(response["responseType"], "clicked");
    assert_eq!(response["contactName"], "Click User");
    assert_eq!(response["sourceUrl"], "https://example.com/landing");
    assert!(response["id"].is_string());
}

#[tokio::test]
async fn test_create_response_updates_campaign_actuals() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-ACT", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Record purchased response with value
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/responses", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "response_type": "purchased",
            "contact_name": "Buyer",
            "value": "5000",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Verify campaign actuals were updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/marketing/campaigns/{}", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["actualResponses"].as_i64().unwrap(), 1);
    assert!(updated["actualRevenue"].as_str().unwrap().contains("5000"));
}

#[tokio::test]
async fn test_list_campaign_responses() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-LRSP", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create responses
    for rt in ["opened", "clicked"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/marketing/campaigns/{}/responses", campaign_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "response_type": rt,
                "contact_name": format!("{} User", rt),
                "value": "0"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/marketing/campaigns/{}/responses", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_campaign_response() {
    let (_state, app) = setup_marketing_test().await;
    let campaign = create_test_campaign(&app, "MKT-DRSP", "email", "10000").await;
    let campaign_id = campaign["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/activate", campaign_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/marketing/campaigns/{}/responses", campaign_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "response_type": "opened",
            "contact_name": "To Delete",
            "value": "0"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let response_id = response["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/marketing/responses/{}", response_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_marketing_dashboard() {
    let (_state, app) = setup_marketing_test().await;
    create_test_campaign(&app, "MKT-DASH", "email", "10000").await;
    create_test_campaign(&app, "MKT-DASH2", "social", "20000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/marketing/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalCampaigns"].as_i64().unwrap() >= 2);
    assert!(dashboard["campaignsByStatus"].is_array());
    assert!(dashboard["campaignsByChannel"].is_array());
}
