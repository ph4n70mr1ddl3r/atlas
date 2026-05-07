//! Profitability Analysis E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Financials > Profitability Analysis:
//! - Segment CRUD
//! - Analysis run lifecycle (draft → calculated → reviewed → completed)
//! - Run lines (add/remove with automatic margin calculation)
//! - Templates
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Clean profitability test data
    sqlx::query("DELETE FROM _atlas.profitability_run_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.profitability_runs").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.profitability_segments").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.profitability_templates").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/129_profitability_analysis.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run profitability analysis migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_segment(
    app: &axum::Router,
    code: &str,
    name: &str,
    segment_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "segmentCode": code,
        "segmentName": name,
        "segmentType": segment_type,
        "description": format!("Test {} segment", segment_type),
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/segments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE SEGMENT status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create segment: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_run(
    app: &axum::Router,
    run_number: &str,
    run_name: &str,
    analysis_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "runNumber": run_number,
        "runName": run_name,
        "analysisType": analysis_type,
        "periodFrom": "2024-01-01",
        "periodTo": "2024-12-31",
        "currencyCode": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE RUN status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create run: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    run_id: Uuid,
    segment_code: &str,
    segment_name: &str,
    revenue: f64,
    cogs: f64,
    opex: f64,
    line_number: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "segmentCode": segment_code,
        "segmentName": segment_name,
        "segmentType": "product",
        "lineNumber": line_number,
        "revenue": revenue,
        "costOfGoodsSold": cogs,
        "operatingExpenses": opex,
        "otherIncome": 0.0,
        "otherExpense": 0.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/lines", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD LINE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Segment Tests
// ============================================================================

#[tokio::test]
async fn test_create_segment() {
    let (_state, app) = setup_test().await;
    let seg = create_segment(&app, "PROD-001", "Widget A", "product").await;

    assert_eq!(seg["segmentCode"], "PROD-001");
    assert_eq!(seg["segmentName"], "Widget A");
    assert_eq!(seg["segmentType"], "product");
    assert_eq!(seg["isActive"], true);
}

#[tokio::test]
async fn test_list_segments() {
    let (_state, app) = setup_test().await;
    create_segment(&app, "PROD-001", "Widget A", "product").await;
    create_segment(&app, "CUST-001", "Customer A", "customer").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/profitability/segments")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_segments_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_segment(&app, "PROD-001", "Widget A", "product").await;
    create_segment(&app, "CUST-001", "Customer A", "customer").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/profitability/segments?segment_type=product")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|s| s["segmentType"] == "product"));
}

#[tokio::test]
async fn test_get_segment() {
    let (_state, app) = setup_test().await;
    let seg = create_segment(&app, "PROD-001", "Widget A", "product").await;
    let seg_id: Uuid = seg["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/segments/{}", seg_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["segmentCode"], "PROD-001");
}

#[tokio::test]
async fn test_delete_segment() {
    let (_state, app) = setup_test().await;
    create_segment(&app, "DEL-001", "To Delete", "product").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/profitability/segments/code/DEL-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_create_segment_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "segmentCode": "BAD-001",
        "segmentName": "Bad",
        "segmentType": "invalid_type",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/segments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Run Tests
// ============================================================================

#[tokio::test]
async fn test_create_run() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-001", "Q1 Analysis", "standard").await;

    assert_eq!(run["runNumber"], "RUN-001");
    assert_eq!(run["runName"], "Q1 Analysis");
    assert_eq!(run["analysisType"], "standard");
    assert_eq!(run["status"], "draft");
    assert_eq!(run["currencyCode"], "USD");
}

#[tokio::test]
async fn test_list_runs() {
    let (_state, app) = setup_test().await;
    create_run(&app, "RUN-001", "Q1 Analysis", "standard").await;
    create_run(&app, "RUN-002", "Q2 Analysis", "comparison").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/profitability/runs")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_run_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-LC", "Lifecycle Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Draft -> Calculated
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "calculated"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "calculated");

    // Calculated -> Reviewed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "reviewed"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Reviewed -> Completed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "completed");
}

#[tokio::test]
async fn test_run_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-CAN", "Cancel Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_invalid_transition_completed_from_draft() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-INV", "Invalid Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "completed"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_draft_run() {
    let (_state, app) = setup_test().await;
    create_run(&app, "RUN-DEL", "Delete Test", "standard").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/profitability/runs/number/RUN-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_create_run_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "runNumber": "RUN-BAD",
        "runName": "Bad Type",
        "analysisType": "invalid",
        "periodFrom": "2024-01-01",
        "periodTo": "2024-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Run Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_lines_and_margins() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-LN", "Lines Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Revenue: 10000, COGS: 6000, OpEx: 1500
    let line = add_line(&app, run_id, "PROD-A", "Product A", 10000.0, 6000.0, 1500.0, 1).await;

    // Gross margin = 10000 - 6000 = 4000 (40%)
    let gm: f64 = line["grossMargin"].as_f64().unwrap();
    assert!((gm - 4000.0).abs() < 0.01, "Expected gross margin 4000, got {}", gm);
    let gm_pct: f64 = line["grossMarginPct"].as_f64().unwrap();
    assert!((gm_pct - 40.0).abs() < 0.01, "Expected gross margin % 40, got {}", gm_pct);

    // Operating margin = 4000 - 1500 = 2500 (25%)
    let om: f64 = line["operatingMargin"].as_f64().unwrap();
    assert!((om - 2500.0).abs() < 0.01, "Expected operating margin 2500, got {}", om);

    // Net margin = 2500 + 0 - 0 = 2500 (25%)
    let nm: f64 = line["netMargin"].as_f64().unwrap();
    assert!((nm - 2500.0).abs() < 0.01, "Expected net margin 2500, got {}", nm);
}

#[tokio::test]
async fn test_run_totals_recalculated() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-TOT", "Totals Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Line 1: Revenue 10000, COGS 6000, OpEx 1500
    add_line(&app, run_id, "PROD-A", "Product A", 10000.0, 6000.0, 1500.0, 1).await;
    // Line 2: Revenue 5000, COGS 2000, OpEx 800
    add_line(&app, run_id, "PROD-B", "Product B", 5000.0, 2000.0, 800.0, 2).await;

    // Verify run totals: Revenue=15000, COGS=8000, GM=7000, OpEx=2300, OM=4700, NM=4700
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/runs/{}", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total_rev: f64 = body["totalRevenue"].as_f64().unwrap();
    assert!((total_rev - 15000.0).abs() < 0.01, "Expected total revenue 15000, got {}", total_rev);

    let total_gm: f64 = body["totalGrossMargin"].as_f64().unwrap();
    assert!((total_gm - 7000.0).abs() < 0.01, "Expected total gross margin 7000, got {}", total_gm);

    let total_nm: f64 = body["totalNetMargin"].as_f64().unwrap();
    assert!((total_nm - 4700.0).abs() < 0.01, "Expected total net margin 4700, got {}", total_nm);

    let gm_pct: f64 = body["grossMarginPct"].as_f64().unwrap();
    assert!((gm_pct - 46.666666666666664).abs() < 0.01, "Expected GM% ~46.67, got {}", gm_pct);
}

#[tokio::test]
async fn test_remove_line_and_recalc() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-REM", "Remove Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, run_id, "PROD-A", "Product A", 10000.0, 6000.0, 1500.0, 1).await;
    let _line2 = add_line(&app, run_id, "PROD-B", "Product B", 5000.0, 2000.0, 800.0, 2).await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    // Remove first line
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/profitability/runs/{}/lines/{}", run_id, line1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify totals recalculated (only line 2 left)
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/runs/{}", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total_rev: f64 = body["totalRevenue"].as_f64().unwrap();
    assert!((total_rev - 5000.0).abs() < 0.01, "Expected total revenue 5000, got {}", total_rev);
}

#[tokio::test]
async fn test_list_run_lines() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-LIST", "List Lines Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, run_id, "PROD-A", "Product A", 5000.0, 3000.0, 500.0, 1).await;
    add_line(&app, run_id, "PROD-B", "Product B", 3000.0, 1500.0, 400.0, 2).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/runs/{}/lines", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_add_line_to_completed_run_fails() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-CPT", "Completed Test", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Transition to completed
    for status in &["calculated", "reviewed", "completed"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/profitability/runs/{}/transition", run_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": *status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Try to add line to completed run
    let payload = json!({"segmentCode": "PROD-X", "revenue": 100.0, "costOfGoodsSold": 50.0});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/profitability/runs/{}/lines", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "TMPL-PROD",
        "templateName": "Product Profitability",
        "segmentType": "product",
        "description": "Standard product-level analysis",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["templateCode"], "TMPL-PROD");
    assert_eq!(body["segmentType"], "product");
    assert_eq!(body["includesCogs"], true);
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    for code in &["TMPL-1", "TMPL-2"] {
        let payload = json!({
            "templateCode": code,
            "templateName": format!("Template {}", code),
            "segmentType": "product",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/profitability/templates")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/profitability/templates")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_template() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "TMPL-DEL",
        "templateName": "To Delete",
        "segmentType": "customer",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/profitability/templates/TMPL-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_segment(&app, "PROD-D1", "Product D1", "product").await;
    create_segment(&app, "CUST-D1", "Customer D1", "customer").await;
    let run = create_run(&app, "RUN-DASH", "Dashboard Run", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();
    add_line(&app, run_id, "PROD-D1", "Product D1", 5000.0, 3000.0, 500.0, 1).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/profitability/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalSegments").is_some());
    assert!(body.get("activeSegments").is_some());
    assert!(body.get("totalRuns").is_some());
    assert!(body.get("completedRuns").is_some());
    assert!(body.get("totalTemplates").is_some());
    assert!(body.get("latestRun").is_some());
    assert!(body.get("topMarginSegments").is_some());
    assert!(body.get("bottomMarginSegments").is_some());
    assert!(body.get("bySegmentType").is_some());

    assert!(body["totalSegments"].as_i64().unwrap() >= 2);
    assert!(body["totalRuns"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_segment_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "segmentCode": "",
        "segmentName": "No Code",
        "segmentType": "product",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/profitability/segments")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_segment_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/segments/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_run_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/runs/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_negative_margin_calculation() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-NEG", "Negative Margin", "standard").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Revenue: 5000, COGS: 7000 (loss-making product), OpEx: 1000
    let line = add_line(&app, run_id, "PROD-LOSS", "Loss Product", 5000.0, 7000.0, 1000.0, 1).await;

    // Gross margin = 5000 - 7000 = -2000 (-40%)
    let gm: f64 = line["grossMargin"].as_f64().unwrap();
    assert!((gm - (-2000.0)).abs() < 0.01);
    let gm_pct: f64 = line["grossMarginPct"].as_f64().unwrap();
    assert!((gm_pct - (-40.0)).abs() < 0.01);

    // Net margin = -2000 - 1000 = -3000 (-60%)
    let nm: f64 = line["netMargin"].as_f64().unwrap();
    assert!((nm - (-3000.0)).abs() < 0.01);
    let nm_pct: f64 = line["netMarginPct"].as_f64().unwrap();
    assert!((nm_pct - (-60.0)).abs() < 0.01);
}

#[tokio::test]
async fn test_contribution_percentages() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "RUN-CONTRIB", "Contribution Test", "contribution").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Line 1: 60% of revenue (10000 out of 16667)
    add_line(&app, run_id, "PROD-A", "Product A", 10000.0, 6000.0, 1000.0, 1).await;
    // Line 2: 40% of revenue (6667 out of 16667)
    add_line(&app, run_id, "PROD-B", "Product B", 6667.0, 3000.0, 500.0, 2).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/profitability/runs/{}/lines", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let lines = body["data"].as_array().unwrap();
    let line1 = lines.iter().find(|l| l["segmentCode"] == "PROD-A").unwrap();
    let rev_contrib: f64 = line1["revenueContributionPct"].as_f64().unwrap();

    // Line 1 revenue / total revenue * 100 = 10000/16667 * 100 ≈ 60%
    assert!(rev_contrib > 59.0 && rev_contrib < 61.0,
        "Expected ~60% contribution, got {}", rev_contrib);
}
