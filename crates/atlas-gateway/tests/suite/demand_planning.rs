//! Demand Planning / Demand Management E2E Tests
//!
//! Tests for Oracle Fusion SCM Demand Management:
//! - Forecast method CRUD
//! - Demand schedule lifecycle (create → submit → approve → activate → close)
//! - Schedule line management
//! - Demand history recording
//! - Forecast consumption
//! - Accuracy measurement
//! - Dashboard analytics

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_demand_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_method(app: &axum::Router, code: &str, method_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/methods")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Method", code),
            "method_type": method_type,
            "parameters": {"window_size": 3}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_schedule(app: &axum::Router, schedule_number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": schedule_number,
            "name": format!("Forecast {}", schedule_number),
            "schedule_type": "monthly",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
            "currency_code": "USD",
            "confidence_level": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Forecast Method Tests
// ============================================================================

#[tokio::test]
async fn test_create_forecast_method() {
    let (_state, app) = setup_demand_test().await;
    let method = create_test_method(&app, "MA", "moving_average").await;
    assert_eq!(method["code"], "MA");
    assert_eq!(method["name"], "MA Method");
    assert_eq!(method["methodType"], "moving_average");
    assert!(method["id"].is_string());
}

#[tokio::test]
async fn test_create_method_duplicate() {
    let (_state, app) = setup_demand_test().await;
    create_test_method(&app, "DUP", "manual").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/methods")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP",
            "name": "Duplicate",
            "method_type": "manual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_method_invalid_type() {
    let (_state, app) = setup_demand_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/methods")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID",
            "name": "Invalid",
            "method_type": "crystal_ball"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_methods() {
    let (_state, app) = setup_demand_test().await;
    create_test_method(&app, "LS-A", "manual").await;
    create_test_method(&app, "LS-B", "moving_average").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/demand/methods").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_method() {
    let (_state, app) = setup_demand_test().await;
    let method = create_test_method(&app, "GET-M", "manual").await;
    let method_id = method["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/methods/{}", method_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "GET-M");
}

#[tokio::test]
async fn test_delete_method() {
    let (_state, app) = setup_demand_test().await;
    create_test_method(&app, "DEL-M", "manual").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/demand/methods-by-code/DEL-M").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Demand Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_create_schedule() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-001").await;
    assert_eq!(schedule["scheduleNumber"], "FC-001");
    assert_eq!(schedule["name"], "Forecast FC-001");
    assert_eq!(schedule["status"], "draft");
    assert_eq!(schedule["scheduleType"], "monthly");
    assert_eq!(schedule["confidenceLevel"], "medium");
    assert!(schedule["id"].is_string());
}

#[tokio::test]
async fn test_create_schedule_duplicate() {
    let (_state, app) = setup_demand_test().await;
    create_test_schedule(&app, "FC-DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "FC-DUP",
            "name": "Duplicate",
            "schedule_type": "monthly",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_schedule_invalid_dates() {
    let (_state, app) = setup_demand_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "FC-INV",
            "name": "Invalid dates",
            "schedule_type": "monthly",
            "start_date": "2025-12-31",
            "end_date": "2025-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_schedules() {
    let (_state, app) = setup_demand_test().await;
    create_test_schedule(&app, "FC-LA").await;
    create_test_schedule(&app, "FC-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/demand/schedules?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_schedule() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-GET").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/schedules/{}", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["scheduleNumber"], "FC-GET");
}

#[tokio::test]
async fn test_schedule_lifecycle() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-LC").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/submit", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/approve", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedAt"].is_string());

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/activate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(active["status"], "active");

    // Close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/close", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let closed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(closed["status"], "closed");
}

#[tokio::test]
async fn test_submit_non_draft_rejected() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-ND").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Submit once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/submit", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to submit again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/submit", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_schedule() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-CNL").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/cancel", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Schedule Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_schedule_line() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-LN").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-100",
            "item_name": "Widget A",
            "item_category": "Electronics",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "1000",
            "unit_price": "25.50",
            "confidence_pct": "85"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["itemCode"], "SKU-100");
    assert_eq!(line["itemName"], "Widget A");
    // forecast_value = 1000 * 25.50 = 25500
    assert!(line["forecastValue"].as_str().unwrap().starts_with("25500"));
    // remaining = forecast_quantity initially
    assert!(line["remainingQuantity"].as_str().unwrap().starts_with("1000"));
    assert!(line["lineNumber"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_schedule_lines() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-LL").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add two lines
    for (item, qty) in [("SKU-A", "500"), ("SKU-B", "300")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "item_code": item,
                "period_start": "2025-01-01",
                "period_end": "2025-01-31",
                "forecast_quantity": qty,
                "unit_price": "10"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_schedule_line() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-DL").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-DEL",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "100",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/demand/schedule-lines/{}", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_add_line_to_non_draft_rejected() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-NDA").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Submit the schedule
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/submit", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try adding a line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-LATE",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Demand History Tests
// ============================================================================

#[tokio::test]
async fn test_create_demand_history() {
    let (_state, app) = setup_demand_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/history")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-100",
            "item_name": "Widget A",
            "actual_date": "2025-01-15",
            "actual_quantity": "950",
            "actual_value": "24225",
            "source_type": "sales_order"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let history: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(history["itemCode"], "SKU-100");
    assert!(history["actualQuantity"].as_str().unwrap().starts_with("950"));
    assert_eq!(history["sourceType"], "sales_order");
    assert!(history["id"].is_string());
}

#[tokio::test]
async fn test_list_demand_history() {
    let (_state, app) = setup_demand_test().await;
    let (k, v) = auth_header(&admin_claims());
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/history")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "item_code": format!("SKU-H{}", i),
                "actual_date": "2025-01-15",
                "actual_quantity": "100",
                "source_type": "manual"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/demand/history?start_date=2025-01-01&end_date=2025-12-31")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_demand_history() {
    let (_state, app) = setup_demand_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/history")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-DELH",
            "actual_date": "2025-01-15",
            "actual_quantity": "100",
            "source_type": "manual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let history: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let history_id = history["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/demand/history/{}", history_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Forecast Consumption Tests
// ============================================================================

#[tokio::test]
async fn test_consume_forecast() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-CON").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add a line with forecast_quantity = 1000
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-CONS",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "1000",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Consume 400
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/consumption")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_line_id": line_id,
            "consumed_quantity": "400",
            "consumed_date": "2025-01-20",
            "source_type": "auto",
            "notes": "Partial consumption from sales orders"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let consumption: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(consumption["consumedQuantity"].as_str().unwrap().starts_with("400"));

    // Verify remaining was updated by listing lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    let updated_line = lines.iter().find(|l| l["itemCode"] == "SKU-CONS").unwrap();
    assert!(updated_line["consumedQuantity"].as_str().unwrap().starts_with("400"));
    assert!(updated_line["remainingQuantity"].as_str().unwrap().starts_with("600"));
}

#[tokio::test]
async fn test_consume_over_remaining_rejected() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-OVR").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add line with forecast_quantity = 100
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-OVR",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "100",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Try to consume more than remaining
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/consumption")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_line_id": line_id,
            "consumed_quantity": "200",
            "consumed_date": "2025-01-20"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_consumption() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-LCO").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add a line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-LCO",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "500",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Add consumption
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/consumption")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_line_id": line_id,
            "consumed_quantity": "100",
            "consumed_date": "2025-01-10"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List consumption
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/consumption/{}", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Accuracy Measurement Tests
// ============================================================================

#[tokio::test]
async fn test_measure_accuracy() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-ACC").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add line with forecast_quantity = 1000
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-ACC",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "1000",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Measure accuracy with actual = 900
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/accuracy")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_line_id": line_id,
            "actual_quantity": "900"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let accuracy: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(accuracy["itemCode"], "SKU-ACC");
    // forecast = 1000, actual = 900, error = 100, MAPE = 11.11%, bias = 100
    assert!(accuracy["absoluteError"].as_str().unwrap().starts_with("100"));
    assert!(accuracy["bias"].as_str().unwrap().starts_with("100"));
    let mape: f64 = accuracy["absolutePctError"].as_str().unwrap().parse().unwrap();
    assert!((mape - 11.11).abs() < 0.5);
}

#[tokio::test]
async fn test_list_accuracy() {
    let (_state, app) = setup_demand_test().await;
    let schedule = create_test_schedule(&app, "FC-ALI").await;
    let schedule_id = schedule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add a line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/demand/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-ALI",
            "period_start": "2025-01-01",
            "period_end": "2025-01-31",
            "forecast_quantity": "500",
            "unit_price": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Measure accuracy
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/demand/accuracy")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_line_id": line_id,
            "actual_quantity": "450"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List accuracy for the schedule
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/demand/schedules/{}/accuracy", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_demand_planning_dashboard() {
    let (_state, app) = setup_demand_test().await;
    create_test_schedule(&app, "FC-DB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/demand/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalSchedules"].as_i64().unwrap() >= 1);
}
