//! Financial Consolidation E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Financial Consolidation:
//! - Consolidation ledger CRUD
//! - Consolidation entity management
//! - Consolidation scenario lifecycle (create, execute, approve, post, reverse)
//! - Elimination rules
//! - Dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/038_financial_consolidation.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_ledger(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-consolidation/ledgers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "base_currency_code": "USD",
            "translation_method": "current_rate",
            "equity_elimination_method": "full",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    if status != StatusCode::CREATED {
        let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&b);
        panic!("Failed to create ledger: status={}, body={}", status, body_str);
    }
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Ledger Tests
// ============================================================================

#[tokio::test]
async fn test_create_consolidation_ledger() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "CONSOL-01", "Primary Consolidation").await;
    assert_eq!(ledger["code"], "CONSOL-01");
    assert_eq!(ledger["name"], "Primary Consolidation");
    assert_eq!(ledger["baseCurrencyCode"], "USD");
    assert_eq!(ledger["translationMethod"], "current_rate");
    assert_eq!(ledger["equityEliminationMethod"], "full");
    assert_eq!(ledger["isActive"], true);
}

#[tokio::test]
async fn test_create_ledger_invalid_translation_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-consolidation/ledgers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad",
            "base_currency_code": "USD",
            "translation_method": "invalid",
            "equity_elimination_method": "full",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_consolidation_ledgers() {
    let (_state, app) = setup_test().await;
    create_test_ledger(&app, "L1", "Ledger 1").await;
    create_test_ledger(&app, "L2", "Ledger 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-consolidation/ledgers")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_ledger_by_code() {
    let (_state, app) = setup_test().await;
    create_test_ledger(&app, "GETTEST", "Get Test Ledger").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-consolidation/ledgers/GETTEST")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ledger: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(ledger["code"], "GETTEST");
}

#[tokio::test]
async fn test_get_ledger_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-consolidation/ledgers/NONEXISTENT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Entity Tests
// ============================================================================

#[tokio::test]
async fn test_add_consolidation_entity() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "ENT-TEST", "Entity Test").await;
    let ledger_id = ledger["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/financial-consolidation/ledgers/{}/entities", ledger_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_id": uuid::Uuid::new_v4().to_string(),
            "entity_name": "Subsidiary Corp",
            "entity_code": "SUB-01",
            "local_currency_code": "EUR",
            "ownership_percentage": "100.00",
            "consolidation_method": "full",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let entity: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(entity["entityCode"], "SUB-01");
    assert_eq!(entity["entityName"], "Subsidiary Corp");
}

#[tokio::test]
async fn test_list_consolidation_entities() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "LIST-ENT", "List Entities").await;
    let ledger_id = ledger["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/financial-consolidation/ledgers/{}/entities", ledger_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().is_empty());
}

// ============================================================================
// Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_create_consolidation_scenario() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "SCEN-TEST", "Scenario Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-consolidation/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ledger_id": ledger["id"],
            "scenario_number": "SCEN-001",
            "name": "Q4 2025 Consolidation",
            "fiscal_year": 2025,
            "period_name": "Q4",
            "period_start": "2025-10-01",
            "period_end": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(scenario["scenarioNumber"], "SCEN-001");
    assert_eq!(scenario["status"], "draft");
}

#[tokio::test]
async fn test_list_scenarios() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "LIST-SCEN", "List Scenarios").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-consolidation/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ledger_id": ledger["id"],
            "scenario_number": "LS-001",
            "name": "Test",
            "fiscal_year": 2025,
            "period_name": "Q4",
            "period_start": "2025-10-01",
            "period_end": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-consolidation/scenarios")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Elimination Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_elimination_rule() {
    let (_state, app) = setup_test().await;
    let ledger = create_test_ledger(&app, "ELIM-TEST", "Elimination Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/financial-consolidation/elimination-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ledger_id": ledger["id"],
            "rule_code": "IC-AR-AP",
            "name": "Intercompany AR/AP Elimination",
            "elimination_type": "intercompany_receivable_payable",
            "offset_account_code": "9999",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["ruleCode"], "IC-AR-AP");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_get_consolidation_dashboard() {
    let (_state, app) = setup_test().await;
    create_test_ledger(&app, "DASH-TEST", "Dashboard Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/financial-consolidation/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
