//! Lead and Opportunity Management E2E Tests
//!
//! Tests for Oracle Fusion CX Sales:
//! - Lead source CRUD
//! - Lead lifecycle (create → score → qualify → convert)
//! - Opportunity stage configuration
//! - Opportunity lifecycle (create → stage progression → win/lose)
//! - Opportunity line items
//! - Sales activities
//! - Pipeline dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_sales_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_lead_source(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/lead-sources")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Source", code),
            "description": "Test lead source"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_stage(app: &axum::Router, code: &str, name: &str, probability: &str, order: i32, is_won: bool, is_lost: bool) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/opportunity-stages")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "probability": probability,
            "display_order": order,
            "is_won": is_won,
            "is_lost": is_lost
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_lead(app: &axum::Router, lead_number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/leads")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "lead_number": lead_number,
            "first_name": "John",
            "last_name": "Doe",
            "company": "Acme Corp",
            "email": "john@acme.com",
            "phone": "+1-555-0100",
            "industry": "technology",
            "estimated_value": "50000",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_opportunity(app: &axum::Router, opp_number: &str, amount: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/opportunities")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "opportunity_number": opp_number,
            "name": format!("Opportunity {}", opp_number),
            "amount": amount,
            "currency_code": "USD",
            "probability": "25",
            "customer_name": "Test Customer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Lead Source Tests
// ============================================================================

#[tokio::test]
async fn test_create_lead_source() {
    let (_state, app) = setup_sales_test().await;
    let src = create_test_lead_source(&app, "WEBSITE").await;
    assert_eq!(src["code"], "WEBSITE");
    assert_eq!(src["name"], "WEBSITE Source");
    assert!(src["id"].is_string());
}

#[tokio::test]
async fn test_create_lead_source_duplicate() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead_source(&app, "DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/lead-sources")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP",
            "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_lead_sources() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead_source(&app, "LS-A").await;
    create_test_lead_source(&app, "LS-B").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sales/lead-sources").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_lead_source() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead_source(&app, "DEL-LS").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/sales/lead-sources/DEL-LS").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lead Tests
// ============================================================================

#[tokio::test]
async fn test_create_lead() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-001").await;
    assert_eq!(lead["leadNumber"], "LD-001");
    assert_eq!(lead["firstName"], "John");
    assert_eq!(lead["lastName"], "Doe");
    assert_eq!(lead["company"], "Acme Corp");
    assert_eq!(lead["status"], "new");
    assert_eq!(lead["leadRating"], "cold");
    // estimated_value comes back as estimatedValue via camelCase
    let ev = lead["estimatedValue"].as_str().unwrap();
    let ev_val: f64 = ev.parse().unwrap();
    assert!(ev_val > 0.0, "estimated_value should be positive, got: {}", ev);
}

#[tokio::test]
async fn test_create_lead_duplicate() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead(&app, "LD-DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/leads")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "lead_number": "LD-DUP",
            "first_name": "Jane"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_lead() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-GET").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sales/leads/{}", lead_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["leadNumber"], "LD-GET");
}

#[tokio::test]
async fn test_list_leads() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead(&app, "LD-LA").await;
    create_test_lead(&app, "LD-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sales/leads?status=new").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_lead_status() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-STS").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/status", lead_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "contacted"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "contacted");
}

#[tokio::test]
async fn test_update_lead_score() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-SCR").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/score", lead_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "score": "85",
            "rating": "hot"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["leadRating"], "hot");
    assert!(updated["leadScore"].as_str().unwrap().starts_with("85"));
}

#[tokio::test]
async fn test_update_lead_score_invalid_rating() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-INV").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/score", lead_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"score": "50", "rating": "frozen"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_lead() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-DEL").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/sales/leads/{}", lead_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Opportunity Stage Tests
// ============================================================================

#[tokio::test]
async fn test_create_opportunity_stages() {
    let (_state, app) = setup_sales_test().await;
    let stage = create_test_stage(&app, "PROSPECT", "Prospecting", "10", 1, false, false).await;
    assert_eq!(stage["code"], "PROSPECT");
    assert_eq!(stage["name"], "Prospecting");
    assert!(stage["probability"].as_str().unwrap().starts_with("10"));
    assert_eq!(stage["isWon"], false);
    assert_eq!(stage["isLost"], false);
}

#[tokio::test]
async fn test_create_won_lost_stages() {
    let (_state, app) = setup_sales_test().await;
    let won = create_test_stage(&app, "WON", "Closed Won", "100", 99, true, false).await;
    assert_eq!(won["isWon"], true);
    let lost = create_test_stage(&app, "LOST", "Closed Lost", "0", 100, false, true).await;
    assert_eq!(lost["isLost"], true);
}

#[tokio::test]
async fn test_list_opportunity_stages() {
    let (_state, app) = setup_sales_test().await;
    create_test_stage(&app, "QUAL", "Qualification", "25", 2, false, false).await;
    create_test_stage(&app, "PROP", "Proposal", "50", 3, false, false).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sales/opportunity-stages").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Opportunity Tests
// ============================================================================

#[tokio::test]
async fn test_create_opportunity() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-001", "100000").await;
    assert_eq!(opp["opportunityNumber"], "OPP-001");
    assert_eq!(opp["name"], "Opportunity OPP-001");
    assert_eq!(opp["status"], "open");
    assert!(opp["amount"].as_str().unwrap().starts_with("100000"));
    // weighted = 100000 * 25 / 100 = 25000
    assert!(opp["weightedAmount"].as_str().unwrap().starts_with("25000"));
    assert!(opp["id"].is_string());
}

#[tokio::test]
async fn test_create_opportunity_duplicate() {
    let (_state, app) = setup_sales_test().await;
    create_test_opportunity(&app, "OPP-DUP", "50000").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/opportunities")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "opportunity_number": "OPP-DUP",
            "name": "Duplicate",
            "amount": "1000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_opportunities() {
    let (_state, app) = setup_sales_test().await;
    create_test_opportunity(&app, "OPP-LA", "50000").await;
    create_test_opportunity(&app, "OPP-LB", "75000").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sales/opportunities?status=open").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_opportunity_stage() {
    let (_state, app) = setup_sales_test().await;
    let stage = create_test_stage(&app, "NEGOT", "Negotiation", "75", 4, false, false).await;
    let stage_id = stage["id"].as_str().unwrap();
    let opp = create_test_opportunity(&app, "OPP-STG", "200000").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/stage", opp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "stage_id": stage_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["stageName"], "Negotiation");
    // probability should be 75
    assert!(updated["probability"].as_str().unwrap().starts_with("75"));
    // weighted = 200000 * 75 / 100 = 150000
    assert!(updated["weightedAmount"].as_str().unwrap().starts_with("150000"));
}

#[tokio::test]
async fn test_close_opportunity_won() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-WIN", "150000").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/win", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let won: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(won["status"], "won");
    assert!(won["probability"].as_str().unwrap().starts_with("100"));
}

#[tokio::test]
async fn test_close_opportunity_lost() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-LOSE", "80000").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/lose", opp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "lost_reason": "Customer chose competitor"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lost: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lost["status"], "lost");
    assert_eq!(lost["lostReason"], "Customer chose competitor");
    assert!(lost["probability"].as_str().unwrap().starts_with("0"));
}

#[tokio::test]
async fn test_close_won_twice_rejected() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-TWICE", "50000").await;
    let opp_id = opp["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Close as won
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/win", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to close again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/win", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_stage_history() {
    let (_state, app) = setup_sales_test().await;
    let stage = create_test_stage(&app, "QUAL-STG", "Qualification", "40", 2, false, false).await;
    let stage_id = stage["id"].as_str().unwrap();
    let opp = create_test_opportunity(&app, "OPP-HIST", "100000").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Update stage
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/stage", opp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"stage_id": stage_id})).unwrap())).unwrap()
    ).await.unwrap();

    // Check history
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sales/opportunities/{}/history", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let history = resp["data"].as_array().unwrap();
    assert!(history.len() >= 1);
    assert_eq!(history[0]["toStage"], "Qualification");
}

#[tokio::test]
async fn test_delete_opportunity() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-DEL", "30000").await;
    let opp_id = opp["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/sales/opportunities/{}", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Opportunity Line Tests
// ============================================================================

#[tokio::test]
async fn test_opportunity_lines() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-LINES", "0").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add line items
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/opportunities/{}/lines", opp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "product_name": "Enterprise License",
            "product_code": "EL-100",
            "quantity": "10",
            "unit_price": "5000",
            "discount_percent": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["productName"], "Enterprise License");
    // 10 * 5000 * (1 - 10/100) = 45000
    assert!(line["lineAmount"].as_str().unwrap().starts_with("45000"));

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sales/opportunities/{}/lines", opp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Lead Conversion Tests
// ============================================================================

#[tokio::test]
async fn test_convert_lead_to_opportunity() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-CVT").await;
    let lead_id = lead["id"].as_str().unwrap();

    // Qualify the lead first
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/status", lead_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "qualified"})).unwrap())).unwrap()
    ).await.unwrap();

    // Convert
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/convert", lead_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();

    // Verify lead is converted
    assert_eq!(result["lead"]["status"], "converted");
    assert!(result["lead"]["convertedAt"].is_string());

    // Verify opportunity was created
    assert!(result["opportunity"]["id"].is_string());
    assert_eq!(result["opportunity"]["leadId"], lead_id);
    assert!(result["opportunity"]["name"].as_str().unwrap().contains("LD-CVT"));
}

#[tokio::test]
async fn test_convert_lead_twice_rejected() {
    let (_state, app) = setup_sales_test().await;
    let lead = create_test_lead(&app, "LD-2CVT").await;
    let lead_id = lead["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Convert once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/convert", lead_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/leads/{}/convert", lead_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Sales Activity Tests
// ============================================================================

#[tokio::test]
async fn test_activity_lifecycle() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-ACT", "50000").await;
    let opp_id = opp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create activity
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/activities")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "subject": "Demo call with customer",
            "activity_type": "demo",
            "priority": "high",
            "opportunity_id": opp_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let act: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let act_id = act["id"].as_str().unwrap();
    assert_eq!(act["subject"], "Demo call with customer");
    assert_eq!(act["activityType"], "demo");
    assert_eq!(act["status"], "planned");
    assert_eq!(act["priority"], "high");

    // Complete activity
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/activities/{}/complete", act_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "outcome": "Customer was impressed, wants proposal"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["outcome"], "Customer was impressed, wants proposal");
}

#[tokio::test]
async fn test_activity_cancel() {
    let (_state, app) = setup_sales_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/activities")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "subject": "Meeting to cancel",
            "activity_type": "meeting",
            "priority": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let act: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let act_id = act["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/sales/activities/{}/cancel", act_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_activities() {
    let (_state, app) = setup_sales_test().await;
    let opp = create_test_opportunity(&app, "OPP-LACT", "50000").await;
    let opp_id = opp["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/sales/activities")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "subject": format!("Activity {}", i),
                "activity_type": "call",
                "opportunity_id": opp_id
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/sales/activities?opportunity_id={}", opp_id))
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
async fn test_sales_pipeline_dashboard() {
    let (_state, app) = setup_sales_test().await;
    create_test_lead(&app, "LD-DASH").await;
    create_test_opportunity(&app, "OPP-DASH", "100000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/sales/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalLeads"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalOpportunities"].as_i64().unwrap() >= 1);
    assert!(dashboard["openOpportunities"].as_i64().unwrap() >= 1);
    assert!(dashboard["newLeads"].as_i64().unwrap() >= 1);
}
