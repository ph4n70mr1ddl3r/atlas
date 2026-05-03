//! Suspense Account Processing E2E Tests
//!
//! Oracle Fusion: General Ledger > Suspense Accounts

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/118_suspense_account_processing.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ── Definition Tests ─────────────────────────────────────────

#[tokio::test]
async fn test_create_suspense_definition() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-E2E-01",
            "name": "Primary Suspense",
            "description": "Main suspense account for company segment",
            "balancing_segment": "company",
            "suspense_account": "9999-000-0001",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(def["code"], "SUSP-E2E-01");
    assert_eq!(def["status"], "active");
    assert!(def["enabled"].as_bool().unwrap());
}

#[tokio::test]
async fn test_create_definition_empty_code() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Name",
            "balancing_segment": "company",
            "suspense_account": "9999",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_definition_duplicate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "code": "SUSP-DUP",
        "name": "Name",
        "balancing_segment": "company",
        "suspense_account": "9999",
    })).unwrap();
    let r1 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body.clone())).unwrap()
    ).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let r2 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_definitions() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/suspense/definitions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deactivate_and_delete_definition() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-DEL",
            "name": "To Delete",
            "balancing_segment": "company",
            "suspense_account": "9999",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let id = def["id"].as_str().unwrap();
    // Try delete while active - should fail
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/suspense/definitions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/definitions/{}/deactivate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // Now delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/suspense/definitions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ── Entry Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_create_suspense_entry() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create definition first
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-ENT",
            "name": "Entry Test",
            "balancing_segment": "company",
            "suspense_account": "9999-000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let def_id = def["id"].as_str().unwrap();
    // Create entry
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_id": def_id,
            "balancing_segment_value": "US01",
            "suspense_amount": "1500.00",
            "original_amount": "10000.00",
            "entry_type": "auto",
            "entry_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(entry["status"], "open");
    assert_eq!(entry["entry_type"], "auto");
}

#[tokio::test]
async fn test_create_entry_zero_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_id": "00000000-0000-0000-0000-000000000099",
            "balancing_segment_value": "US01",
            "suspense_amount": "0",
            "entry_type": "auto",
            "entry_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_entries() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/suspense/entries")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_entries_invalid_status() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/suspense/entries?status=invalid")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_entry() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create definition
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-REV",
            "name": "Reverse Test",
            "balancing_segment": "company",
            "suspense_account": "9999-000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let def_id = def["id"].as_str().unwrap();
    // Create entry
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_id": def_id,
            "balancing_segment_value": "US01",
            "suspense_amount": "500.00",
            "entry_type": "auto",
            "entry_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let entry_id = entry["id"].as_str().unwrap();
    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/entries/{}/reverse", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution_notes": "Reversed in error",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(entry["status"], "reversed");
}

#[tokio::test]
async fn test_write_off_entry_without_notes() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create definition + entry
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-WO",
            "name": "Write-off Test",
            "balancing_segment": "company",
            "suspense_account": "9999-000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let def_id = def["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_id": def_id,
            "balancing_segment_value": "US01",
            "suspense_amount": "100.00",
            "entry_type": "manual",
            "entry_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let entry_id = entry["id"].as_str().unwrap();
    // Write-off without notes - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/entries/{}/write-off", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ── Clearing Batch Tests ─────────────────────────────────────

#[tokio::test]
async fn test_full_clearing_workflow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create definition
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SUSP-CLR",
            "name": "Clearing Test",
            "balancing_segment": "company",
            "suspense_account": "9999-000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let def_id = def["id"].as_str().unwrap();
    // Create entry
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_id": def_id,
            "balancing_segment_value": "US01",
            "suspense_amount": "2500.00",
            "entry_type": "auto",
            "entry_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let entry_id = entry["id"].as_str().unwrap();
    // Create clearing batch
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/clearing-batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "CLB-E2E-01",
            "description": "Monthly clearing",
            "clearing_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let batch: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let batch_id = batch["id"].as_str().unwrap();
    assert_eq!(batch["status"], "draft");
    // Add clearing line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/clearing-batches/{}/lines", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entry_id": entry_id,
            "clearing_account": "4100-000",
            "cleared_amount": "2500.00",
            "resolution_notes": "Fully cleared",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/clearing-batches/{}/submit", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let batch: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(batch["status"], "submitted");
    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/clearing-batches/{}/approve", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/clearing-batches/{}/post", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let batch: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(batch["status"], "posted");
    // Verify entry is cleared
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/suspense/entries/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let entry: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(entry["status"], "cleared");
}

#[tokio::test]
async fn test_create_clearing_batch_empty_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/clearing-batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "",
            "clearing_date": "2026-05-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_empty_clearing_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/clearing-batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "CLB-EMPTY",
            "clearing_date": "2026-05-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let batch_id = batch["id"].as_str().unwrap();
    // Submit empty batch - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/suspense/clearing-batches/{}/submit", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ── Dashboard & Aging Tests ──────────────────────────────────

#[tokio::test]
async fn test_get_suspense_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/suspense/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let d: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(d["total_definitions"].is_number());
    assert!(d["total_open_entries"].is_number());
}

#[tokio::test]
async fn test_create_aging_snapshot() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/suspense/aging-snapshot")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "snapshot_date": "2026-05-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_clearing_batches() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/suspense/clearing-batches")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
