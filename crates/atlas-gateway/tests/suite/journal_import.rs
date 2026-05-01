//! Journal Import E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Journal Import:
//! - Import format CRUD
//! - Column mapping management
//! - Import batch lifecycle
//! - Row data management
//! - Validation and import processing
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_journal_import_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_format(
    app: &axum::Router,
    code: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/formats")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("Import Format {}", code),
            "source_type": "file",
            "file_format": "csv",
            "delimiter": ",",
            "header_row": true,
            "currency_code": "USD",
            "validation_enabled": true,
            "auto_post": false,
            "max_errors_allowed": 50,
            "column_mappings": []
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create import format");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_create_import_format() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-GL").await;

    assert_eq!(format["code"], "CSV-GL");
    assert_eq!(format["currency_code"], "USD");
    assert_eq!(format["status"], "active");
    assert_eq!(format["source_type"], "file");
    assert_eq!(format["file_format"], "csv");
}

#[tokio::test]
#[ignore]
async fn test_list_import_formats() {
    let (_state, app) = setup_journal_import_test().await;

    create_test_format(&app, "CSV-001").await;
    create_test_format(&app, "CSV-002").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/journal-import/formats")
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
async fn test_get_import_format() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-010").await;
    let format_id = format["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/journal-import/formats/{}", format_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_duplicate_format() {
    let (_state, app) = setup_journal_import_test().await;

    create_test_format(&app, "CSV-DUP").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/formats")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "CSV-DUP",
            "name": "Duplicate",
            "source_type": "file",
            "file_format": "csv",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore]
async fn test_delete_import_format() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-DEL").await;
    let code = format["code"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/journal-import/formats/{}", code))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_add_column_mapping() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-MAP").await;
    let format_id = format["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/formats/{}/mappings", format_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "column_position": 1,
            "source_column": "Account",
            "target_field": "account_code",
            "data_type": "string",
            "is_required": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let mapping: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(mapping["target_field"], "account_code");
    assert_eq!(mapping["source_column"], "Account");
}

#[tokio::test]
#[ignore]
async fn test_list_column_mappings() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-LMAP").await;
    let format_id = format["id"].as_str().unwrap();

    // Add two mappings
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/formats/{}/mappings", format_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "column_position": 1,
            "source_column": "Account",
            "target_field": "account_code",
            "data_type": "string",
            "is_required": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/formats/{}/mappings", format_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "column_position": 2,
            "source_column": "Debit",
            "target_field": "entered_dr",
            "data_type": "number",
            "is_required": false
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List mappings
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/journal-import/formats/{}/mappings", format_id))
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
async fn test_create_import_batch() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-BAT").await;
    let format_id = format["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_id": format_id,
            "name": "Q4 Journal Import",
            "description": "Quarter-end journal entries",
            "source": "file",
            "source_file_name": "q4_journals.csv"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(batch["status"], "uploaded");
    assert_eq!(batch["source"], "file");
}

#[tokio::test]
#[ignore]
async fn test_add_import_rows_and_validate() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-VAL").await;
    let format_id = format["id"].as_str().unwrap();

    // Create batch
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_id": format_id,
            "name": "Validation Test",
            "source": "api"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let batch_id = batch["id"].as_str().unwrap();

    // Add debit row
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1000",
            "description": "Cash deposit",
            "entered_dr": "1000.00",
            "entered_cr": "0.00",
            "raw_data": {"line": 1}
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add credit row
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "4000",
            "description": "Revenue",
            "entered_dr": "0.00",
            "entered_cr": "1000.00",
            "raw_data": {"line": 2}
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Validate batch
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/validate", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let validated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(validated["status"], "validated");
    assert_eq!(validated["valid_rows"], 2);
    assert_eq!(validated["error_rows"], 0);
    assert_eq!(validated["is_balanced"], true);
}

#[tokio::test]
#[ignore]
async fn test_validation_catches_errors() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-ERR").await;
    let format_id = format["id"].as_str().unwrap();

    // Create batch
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_id": format_id,
            "source": "api"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let batch_id = batch["id"].as_str().unwrap();

    // Add row with no account code (should fail validation)
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "",
            "entered_dr": "100.00",
            "entered_cr": "0.00"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add row with zero amounts (should fail)
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "2000",
            "entered_dr": "0.00",
            "entered_cr": "0.00"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Validate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/validate", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let validated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(validated["error_rows"], 2);
}

#[tokio::test]
#[ignore]
async fn test_import_batch_lifecycle() {
    let (_state, app) = setup_journal_import_test().await;

    let format = create_test_format(&app, "CSV-LIFE").await;
    let format_id = format["id"].as_str().unwrap();

    // Create batch
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/journal-import/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_id": format_id,
            "source": "api"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let batch_id = batch["id"].as_str().unwrap();

    // Add balanced rows
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1000",
            "entered_dr": "5000.00",
            "entered_cr": "0.00"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/rows", batch_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "4000",
            "entered_dr": "0.00",
            "entered_cr": "5000.00"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Validate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/validate", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Import
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journal-import/batches/{}/import", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let imported: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(imported["status"], "completed");
    assert_eq!(imported["imported_rows"], 2);
}

#[tokio::test]
#[ignore]
async fn test_journal_import_dashboard() {
    let (_state, app) = setup_journal_import_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/journal-import/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["total_formats"].is_number());
    assert!(dashboard["total_batches"].is_number());
}
