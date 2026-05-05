//! Cash Concentration / Pooling E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Treasury > Cash Pooling:
//! - Cash pool CRUD and lifecycle (draft → active → suspended → closed)
//! - Pool participant management
//! - Sweep rule management
//! - Sweep execution with various sweep types
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
    // Clean cash concentration test data
    sqlx::query("DELETE FROM _atlas.cash_pool_sweep_run_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_pool_sweep_runs").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_pool_sweep_rules").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_pool_participants").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_pools").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!("../../../../migrations/125_cash_concentration_pooling.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_pool(
    app: &axum::Router,
    pool_code: &str,
    pool_name: &str,
    pool_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "pool_code": pool_code,
        "pool_name": pool_name,
        "pool_type": pool_type,
        "currency_code": "USD",
        "sweep_frequency": "daily",
        "sweep_time": "18:00",
        "concentration_account_name": "Master Account",
        "description": "Test cash pool",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/cash-pooling/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE POOL status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create pool: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_participant(
    app: &axum::Router,
    pool_id: Uuid,
    participant_code: &str,
    current_balance: &str,
    minimum_balance: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "participant_code": participant_code,
        "bank_account_name": format!("{} Account", participant_code),
        "participant_type": "source",
        "sweep_direction": "to_concentration",
        "current_balance": current_balance,
        "minimum_balance": minimum_balance,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/participants", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("ADD PARTICIPANT status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to add participant");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Cash Pool CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_pool() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-001", "Main Concentration Pool", "physical").await;

    assert_eq!(pool["poolCode"], "POOL-001");
    assert_eq!(pool["poolName"], "Main Concentration Pool");
    assert_eq!(pool["poolType"], "physical");
    assert_eq!(pool["currencyCode"], "USD");
    assert_eq!(pool["status"], "draft");
    assert_eq!(pool["sweepFrequency"], "daily");
}

#[tokio::test]
async fn test_create_notional_pool() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "NPOOL-001", "Notional Pool", "notional").await;

    assert_eq!(pool["poolType"], "notional");
    assert_eq!(pool["status"], "draft");
}

#[tokio::test]
async fn test_get_pool() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-002", "Test Pool", "physical").await;
    let pool_code = pool["poolCode"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/cash-pooling/pools/{}", pool_code))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["poolCode"], "POOL-002");
}

#[tokio::test]
async fn test_list_pools() {
    let (_state, app) = setup_test().await;
    create_pool(&app, "POOL-A", "Pool A", "physical").await;
    create_pool(&app, "POOL-B", "Pool B", "notional").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/cash-pooling/pools")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_pools_with_type_filter() {
    let (_state, app) = setup_test().await;
    create_pool(&app, "POOL-P1", "Physical Pool", "physical").await;
    create_pool(&app, "POOL-N1", "Notional Pool", "notional").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/cash-pooling/pools?pool_type=physical")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 1);
    for p in data {
        assert_eq!(p["poolType"], "physical");
    }
}

// ============================================================================
// Pool Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_pool_lifecycle() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-LC", "Lifecycle Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // Suspend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/suspend", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "suspended");

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // Close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/close", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "closed");
}

#[tokio::test]
async fn test_delete_pool_draft() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-DEL", "Delete Pool", "physical").await;
    let pool_code = pool["poolCode"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/cash-pooling/pools/{}", pool_code))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_active_pool_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-DACT", "Active Delete Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();
    let pool_code = pool["poolCode"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to delete active pool
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/cash-pooling/pools/{}", pool_code))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Participant Tests
// ============================================================================

#[tokio::test]
async fn test_add_participant() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-PART", "Participant Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let participant = add_participant(&app, pool_id, "SUB-001", "50000.00", "10000.00").await;

    assert_eq!(participant["participantCode"], "SUB-001");
    assert_eq!(participant["participantType"], "source");
    assert_eq!(participant["sweepDirection"], "to_concentration");
    assert_eq!(participant["currentBalance"], "50000.00");
    assert_eq!(participant["status"], "active");
}

#[tokio::test]
async fn test_list_participants() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-LP", "List Participants Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    add_participant(&app, pool_id, "SUB-A", "30000.00", "5000.00").await;
    add_participant(&app, pool_id, "SUB-B", "75000.00", "10000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/cash-pooling/pools/{}/participants", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove_participant() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-RP", "Remove Participant Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    add_participant(&app, pool_id, "SUB-R1", "25000.00", "5000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/participants/SUB-R1", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Sweep Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_sweep_rule() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SR", "Sweep Rule Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-ZB",
        "rule_name": "Zero Balance Sweep",
        "sweep_type": "zero_balance",
        "direction": "to_concentration",
        "priority": 1,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["ruleCode"], "RULE-ZB");
    assert_eq!(body["sweepType"], "zero_balance");
    assert_eq!(body["isActive"], true);
}

#[tokio::test]
async fn test_list_sweep_rules() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SRL", "Sweep Rule List Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create two rules
    for (code, stype) in &[("RULE-A", "zero_balance"), ("RULE-B", "target_balance")] {
        let payload = json!({
            "rule_code": code,
            "rule_name": format!("{} Rule", code),
            "sweep_type": stype,
            "direction": "to_concentration",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_sweep_rule() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SRD", "Sweep Rule Delete Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-DEL",
        "rule_name": "Delete Rule",
        "sweep_type": "zero_balance",
        "direction": "to_concentration",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules/RULE-DEL", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Sweep Execution Tests
// ============================================================================

#[tokio::test]
async fn test_execute_sweep_zero_balance() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SW1", "Sweep Pool 1", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate pool
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Add participants with balances
    add_participant(&app, pool_id, "SUB-01", "50000.00", "10000.00").await;
    add_participant(&app, pool_id, "SUB-02", "75000.00", "5000.00").await;

    // Execute sweep
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "run_type": "manual",
            "run_date": "2024-06-15",
            "notes": "Test sweep"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("EXECUTE SWEEP status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED);

    let run: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(run["status"], "completed");
    assert!(run["runNumber"].as_str().unwrap().starts_with("SWEEP-"));

    let total: f64 = run["totalSweptAmount"].as_str().unwrap().parse().unwrap();
    // SUB-01: 50000 - 10000 = 40000, SUB-02: 75000 - 5000 = 70000 => total 110000
    assert!((total - 110000.0).abs() < 1.0, "Expected 110000, got {}", total);
    assert_eq!(run["successfulTransactions"], 2);
}

#[tokio::test]
async fn test_sweep_run_lines() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SWL", "Sweep Lines Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate pool
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Add participant
    add_participant(&app, pool_id, "SUB-L1", "60000.00", "10000.00").await;

    // Execute sweep
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "run_type": "manual",
            "run_date": "2024-06-15",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Get lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/cash-pooling/sweeps/{}/lines", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let lines = body["data"].as_array().unwrap();
    assert!(!lines.is_empty());
    let line = &lines[0];
    assert_eq!(line["direction"], "debit");
    assert_eq!(line["status"], "completed");

    let sweep_amount: f64 = line["sweepAmount"].as_str().unwrap().parse().unwrap();
    assert!((sweep_amount - 50000.0).abs() < 1.0);
}

#[tokio::test]
async fn test_list_sweep_runs() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-SLR", "Sweep List Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate pool
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Execute two sweeps
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "run_type": "manual",
                "run_date": format!("2024-06-1{}", i),
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweeps", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_execute_sweep_with_target_balance_rule() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-TB", "Target Balance Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate pool
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Add participant
    let participant = add_participant(&app, pool_id, "SUB-TB1", "100000.00", "0.00").await;
    let participant_id: Uuid = participant["id"].as_str().unwrap().parse().unwrap();

    // Create target-balance rule
    let payload = json!({
        "rule_code": "RULE-TB",
        "rule_name": "Target Balance Rule",
        "sweep_type": "target_balance",
        "participant_id": participant_id.to_string(),
        "direction": "to_concentration",
        "target_balance": "25000.00",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Execute sweep
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "run_type": "manual",
            "run_date": "2024-06-15",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(run["status"], "completed");

    let total: f64 = run["totalSweptAmount"].as_str().unwrap().parse().unwrap();
    // 100000 - 25000 = 75000
    assert!((total - 75000.0).abs() < 1.0, "Expected 75000, got {}", total);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_cash_pooling_dashboard() {
    let (_state, app) = setup_test().await;
    create_pool(&app, "POOL-DASH", "Dashboard Pool", "physical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/cash-pooling/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalPools").is_some());
    assert!(body.get("activePools").is_some());
    assert!(body.get("totalParticipants").is_some());
    assert!(body.get("totalConcentratedBalance").is_some());
    assert!(body.get("totalSweptToday").is_some());
    assert!(body.get("pendingSweeps").is_some());
    assert!(body.get("byPoolType").is_some());
    assert!(body.get("byCurrency").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_pool_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "pool_code": "POOL-BAD",
        "pool_name": "Bad Pool",
        "pool_type": "hybrid",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/cash-pooling/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_pool_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "pool_code": "",
        "pool_name": "No Code Pool",
        "pool_type": "physical",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/cash-pooling/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_pool_invalid_frequency_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "pool_code": "POOL-BF",
        "pool_name": "Bad Freq Pool",
        "pool_type": "physical",
        "currency_code": "USD",
        "sweep_frequency": "hourly",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/cash-pooling/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_participant_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-BPT", "Bad Participant Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "participant_code": "SUB-BAD",
        "participant_type": "unknown_type",
        "sweep_direction": "to_concentration",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/participants", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_sweep_rule_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-BST", "Bad Sweep Type Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rule_code": "RULE-BAD",
        "rule_name": "Bad Rule",
        "sweep_type": "random",
        "direction": "to_concentration",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/rules", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_sweep_on_inactive_pool_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-ISP", "Inactive Sweep Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "run_type": "manual",
            "run_date": "2024-06-15",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_sweep_on_closed_pool_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-CSP", "Closed Sweep Pool", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate then close
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/close", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try sweep
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/sweep", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "run_type": "manual",
            "run_date": "2024-06-15",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_participant_to_closed_pool_fails() {
    let (_state, app) = setup_test().await;
    let pool = create_pool(&app, "POOL-CPA", "Closed Pool Add", "physical").await;
    let pool_id: Uuid = pool["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate then close
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/close", pool_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add participant
    let payload = json!({
        "participant_code": "SUB-CLOSED",
        "participant_type": "source",
        "sweep_direction": "to_concentration",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cash-pooling/pools/{}/participants", pool_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
