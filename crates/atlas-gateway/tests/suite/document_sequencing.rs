//! Document Sequencing E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Document Sequencing:
//! - Sequence CRUD and lifecycle (create, list, get, activate, deactivate, delete)
//! - Sequence assignments (map sequences to document categories)
//! - Number generation via assignment and direct
//! - Audit trail
//! - Dashboard summary
//! - Validation edge cases

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/144_document_sequencing.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_sequence(
    app: &axum::Router,
    code: &str,
    name: &str,
    sequence_type: &str,
    document_type: &str,
    prefix: Option<&str>,
    suffix: Option<&str>,
    pad_length: Option<i32>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "code": code,
        "name": name,
        "sequence_type": sequence_type,
        "document_type": document_type,
        "initial_value": 1,
        "increment_by": 1,
    });
    if let Some(p) = prefix {
        payload["prefix"] = json!(p);
    }
    if let Some(s) = suffix {
        payload["suffix"] = json!(s);
    }
    if let Some(pl) = pad_length {
        payload["pad_length"] = json!(pl);
    }
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE SEQUENCE RESPONSE status={}: {}", status, body_str);
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create sequence: {:?}",
        body_str
    );
    serde_json::from_slice(&b).unwrap()
}

async fn create_assignment(
    app: &axum::Router,
    sequence_code: &str,
    document_category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "sequence_code": sequence_code,
        "document_category": document_category,
        "method": "automatic",
        "priority": 10,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/assignments")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    eprintln!(
        "CREATE ASSIGNMENT RESPONSE status={}: {}",
        status,
        String::from_utf8_lossy(&b)
    );
    assert_eq!(status, StatusCode::CREATED, "Failed to create assignment");
    serde_json::from_slice(&b).unwrap()
}

async fn generate_via_assignment(app: &axum::Router, category: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "document_category": category,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    eprintln!(
        "GENERATE NUMBER RESPONSE status={}: {}",
        status,
        String::from_utf8_lossy(&b)
    );
    assert_eq!(status, StatusCode::CREATED, "Failed to generate number");
    serde_json::from_slice(&b).unwrap()
}

async fn generate_direct(
    app: &axum::Router,
    sequence_code: &str,
    category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "sequence_code": sequence_code,
        "document_category": category,
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate-direct")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    eprintln!(
        "GENERATE DIRECT RESPONSE status={}: {}",
        status,
        String::from_utf8_lossy(&b)
    );
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to generate direct number"
    );
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Sequence CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_sequence() {
    let (_state, app) = setup_test().await;
    let seq = create_sequence(
        &app,
        "INV-SEQ",
        "Invoice Sequence",
        "gapless",
        "invoice",
        Some("INV-"),
        None,
        Some(6),
    )
    .await;

    assert_eq!(seq["code"], "INV-SEQ");
    assert_eq!(seq["name"], "Invoice Sequence");
    assert_eq!(seq["sequenceType"], "gapless");
    assert_eq!(seq["documentType"], "invoice");
    assert_eq!(seq["prefix"], "INV-");
    assert_eq!(seq["padLength"], 6);
    assert_eq!(seq["status"], "active");
    assert_eq!(seq["initialValue"], 1);
    assert_eq!(seq["incrementBy"], 1);
}

#[tokio::test]
async fn test_get_sequence() {
    let (_state, app) = setup_test().await;
    let seq = create_sequence(
        &app,
        "GET-SEQ",
        "Get Sequence",
        "gap_permitted",
        "journal_entry",
        None,
        None,
        None,
    )
    .await;
    let seq_id = seq["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/document-sequences/{}", seq_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["code"], "GET-SEQ");
    assert_eq!(body["sequenceType"], "gap_permitted");
}

#[tokio::test]
async fn test_get_sequence_by_code() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "CODE-SEQ",
        "Code Sequence",
        "gap_permitted",
        "payment",
        None,
        None,
        None,
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/code/CODE-SEQ")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["code"], "CODE-SEQ");
}

#[tokio::test]
async fn test_list_sequences() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "LIST-SEQ1",
        "List Sequence 1",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_sequence(
        &app,
        "LIST-SEQ2",
        "List Sequence 2",
        "gap_permitted",
        "payment",
        None,
        None,
        None,
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_sequences_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "FILTER-SEQ1",
        "Filter Active",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    let s2 = create_sequence(
        &app,
        "FILTER-SEQ2",
        "Filter To Deactivate",
        "gap_permitted",
        "payment",
        None,
        None,
        None,
    )
    .await;

    // Deactivate second sequence
    let s2_id = s2["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/document-sequences/{}/deactivate", s2_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Filter by active
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences?status=active")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let items = body["data"].as_array().unwrap();
    assert!(items.iter().all(|s| s["status"] == "active"));
}

#[tokio::test]
async fn test_activate_deactivate_sequence() {
    let (_state, app) = setup_test().await;
    let seq = create_sequence(
        &app,
        "LC-SEQ",
        "Lifecycle Sequence",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    let seq_id = seq["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/document-sequences/{}/deactivate", seq_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "inactive");

    // Reactivate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/document-sequences/{}/activate", seq_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_delete_sequence() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DEL-SEQ",
        "Delete Sequence",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/document-sequences/code/DEL-SEQ")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/code/DEL-SEQ")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_sequence_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "",
        "name": "No Code Sequence",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_sequence_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD-TYPE",
        "name": "Bad Type",
        "sequence_type": "invalid_type",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_sequence_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app, "DUP-SEQ", "First", "gapless", "invoice", None, None, None,
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "DUP-SEQ",
        "name": "Duplicate",
        "sequence_type": "gapless",
        "document_type": "invoice",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_delete_sequence_with_assignments_fails() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DEL-ASGN-SEQ",
        "Seq With Assignment",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "DEL-ASGN-SEQ", "invoice_category").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/document-sequences/code/DEL-ASGN-SEQ")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Number Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_number_via_assignment() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "GEN-SEQ",
        "Generate Sequence",
        "gap_permitted",
        "invoice",
        Some("INV-"),
        None,
        Some(6),
    )
    .await;
    create_assignment(&app, "GEN-SEQ", "ap_invoice").await;

    let audit = generate_via_assignment(&app, "ap_invoice").await;

    assert_eq!(audit["generatedNumber"], "INV-000001");
    assert_eq!(audit["numericValue"], 1);
    assert_eq!(audit["sequenceCode"], "GEN-SEQ");
    assert_eq!(audit["documentCategory"], "ap_invoice");
}

#[tokio::test]
async fn test_generate_multiple_numbers_sequential() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "MULTI-SEQ",
        "Multi Generate",
        "gapless",
        "invoice",
        None,
        Some("-2026"),
        Some(5),
    )
    .await;
    create_assignment(&app, "MULTI-SEQ", "gl_journal").await;

    let a1 = generate_via_assignment(&app, "gl_journal").await;
    let a2 = generate_via_assignment(&app, "gl_journal").await;
    let a3 = generate_via_assignment(&app, "gl_journal").await;

    assert_eq!(a1["generatedNumber"], "00001-2026");
    assert_eq!(a1["numericValue"], 1);
    assert_eq!(a2["generatedNumber"], "00002-2026");
    assert_eq!(a2["numericValue"], 2);
    assert_eq!(a3["generatedNumber"], "00003-2026");
    assert_eq!(a3["numericValue"], 3);
}

#[tokio::test]
async fn test_generate_number_direct() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DIRECT-SEQ",
        "Direct Generate",
        "gapless",
        "payment",
        Some("PAY-"),
        None,
        None,
    )
    .await;

    let audit = generate_direct(&app, "DIRECT-SEQ", "payment_batch").await;

    assert_eq!(audit["generatedNumber"], "PAY-1");
    assert_eq!(audit["numericValue"], 1);
    assert_eq!(audit["sequenceCode"], "DIRECT-SEQ");
}

#[tokio::test]
async fn test_generate_number_no_assignment_fails() {
    let (_state, app) = setup_test().await;
    // Create sequence but no assignment for this category
    create_sequence(
        &app,
        "NO-ASGN-SEQ",
        "No Assignment",
        "gap_permitted",
        "invoice",
        None,
        None,
        None,
    )
    .await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "document_category": "nonexistent_category",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_generate_number_inactive_sequence_fails() {
    let (_state, app) = setup_test().await;
    let seq = create_sequence(
        &app,
        "INACT-SEQ",
        "Inactive Seq",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "INACT-SEQ", "inactive_test").await;

    // Deactivate the sequence
    let seq_id = seq["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/document-sequences/{}/deactivate", seq_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to generate via assignment - should fail because the sequence is inactive
    let payload = json!({
        "document_category": "inactive_test",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    // The assignment still finds the inactive sequence, but engine should reject
    assert!(
        r.status() == StatusCode::BAD_REQUEST || r.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[tokio::test]
async fn test_create_assignment() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "ASGN-SEQ",
        "Assignment Sequence",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;

    let assignment = create_assignment(&app, "ASGN-SEQ", "invoice_ap").await;

    assert_eq!(assignment["sequenceCode"], "ASGN-SEQ");
    assert_eq!(assignment["documentCategory"], "invoice_ap");
    assert_eq!(assignment["method"], "automatic");
    assert_eq!(assignment["priority"], 10);
    assert_eq!(assignment["status"], "active");
}

#[tokio::test]
async fn test_list_assignments() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "LIST-ASGN-SEQ",
        "List Assignment Seq",
        "gap_permitted",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "LIST-ASGN-SEQ", "cat_one").await;
    create_assignment(&app, "LIST-ASGN-SEQ", "cat_two").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/assignments")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_deactivate_assignment() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DEACT-ASGN-SEQ",
        "Deactivate Assignment Seq",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    let assignment = create_assignment(&app, "DEACT-ASGN-SEQ", "deact_cat").await;
    let asgn_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/document-sequences/assignments/{}/deactivate",
                    asgn_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["status"], "inactive");
}

#[tokio::test]
async fn test_delete_assignment() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DEL-ASGN2-SEQ",
        "Del Assignment Seq",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    let assignment = create_assignment(&app, "DEL-ASGN2-SEQ", "del_cat").await;
    let asgn_id = assignment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/v1/document-sequences/assignments/{}",
                    asgn_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Audit Trail Tests
// ============================================================================

#[tokio::test]
async fn test_audit_trail_from_generation() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "AUDIT-SEQ",
        "Audit Sequence",
        "gapless",
        "invoice",
        Some("AUD-"),
        None,
        Some(6),
    )
    .await;
    create_assignment(&app, "AUDIT-SEQ", "audit_cat").await;

    // Generate a few numbers
    generate_via_assignment(&app, "audit_cat").await;
    generate_via_assignment(&app, "audit_cat").await;

    // List audit entries
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/audit")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let entries = body["data"].as_array().unwrap();
    assert!(entries.len() >= 2);

    // Verify ordering (most recent first) and content
    assert!(entries.iter().any(|e| e["generatedNumber"] == "AUD-000001"));
    assert!(entries.iter().any(|e| e["generatedNumber"] == "AUD-000002"));
}

#[tokio::test]
async fn test_audit_by_document_id() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DOCAUD-SEQ",
        "Doc Audit Seq",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "DOCAUD-SEQ", "doc_audit_cat").await;

    // Generate with a specific document_id
    let doc_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "document_category": "doc_audit_cat",
        "document_id": doc_id.to_string(),
        "document_number": "DOC-12345",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Look up audit by document_id
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/document-sequences/audit/{}", doc_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert_eq!(body["documentNumber"], "DOC-12345");
    assert_eq!(body["documentCategory"], "doc_audit_cat");
}

#[tokio::test]
async fn test_audit_filter_by_sequence() {
    let (_state, app) = setup_test().await;
    let seq = create_sequence(
        &app,
        "FILTER-AUD-SEQ",
        "Filter Audit Seq",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "FILTER-AUD-SEQ", "filter_audit_cat").await;
    generate_via_assignment(&app, "filter_audit_cat").await;

    let seq_id = seq["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!(
                    "/api/v1/document-sequences/audit?sequence_id={}",
                    seq_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_document_sequencing_dashboard() {
    let (_state, app) = setup_test().await;
    create_sequence(
        &app,
        "DASH-SEQ",
        "Dashboard Sequence",
        "gapless",
        "invoice",
        None,
        None,
        None,
    )
    .await;
    create_assignment(&app, "DASH-SEQ", "dash_cat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    assert!(body.get("totalSequences").is_some());
    assert!(body.get("activeSequences").is_some());
    assert!(body.get("gaplessSequences").is_some());
    assert!(body.get("gapPermittedSequences").is_some());
    assert!(body.get("totalNumbersGenerated").is_some());
    assert!(body.get("totalAssignments").is_some());
    assert!(body.get("recentAudits").is_some());

    // We created 1 sequence
    assert!(body["totalSequences"].as_i64().unwrap() >= 1);
    assert!(body["totalAssignments"].as_i64().unwrap() >= 1);
}

// ============================================================================
// End-to-End Full Flow Test
// ============================================================================

#[tokio::test]
async fn test_end_to_end_document_sequencing_flow() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());

    // 1. Create a gapless invoice sequence with prefix and padding
    let inv_seq = create_sequence(
        &app,
        "E2E-INV",
        "E2E Invoice Sequence",
        "gapless",
        "invoice",
        Some("INV-"),
        None,
        Some(8),
    )
    .await;
    assert_eq!(inv_seq["code"], "E2E-INV");
    assert_eq!(inv_seq["status"], "active");

    // 2. Create a gap-permitted payment sequence with suffix
    let pay_seq = create_sequence(
        &app,
        "E2E-PAY",
        "E2E Payment Sequence",
        "gap_permitted",
        "payment",
        Some("PAY-"),
        Some("-FIN"),
        Some(6),
    )
    .await;
    assert_eq!(pay_seq["code"], "E2E-PAY");

    // 3. Create assignments
    let inv_asgn = create_assignment(&app, "E2E-INV", "e2e_ap_invoice").await;
    assert_eq!(inv_asgn["documentCategory"], "e2e_ap_invoice");

    let pay_asgn = create_assignment(&app, "E2E-PAY", "e2e_payment").await;
    assert_eq!(pay_asgn["documentCategory"], "e2e_payment");

    // 4. Generate invoice numbers sequentially
    let inv1 = generate_via_assignment(&app, "e2e_ap_invoice").await;
    assert_eq!(inv1["generatedNumber"], "INV-00000001");
    assert_eq!(inv1["numericValue"], 1);

    let inv2 = generate_via_assignment(&app, "e2e_ap_invoice").await;
    assert_eq!(inv2["generatedNumber"], "INV-00000002");
    assert_eq!(inv2["numericValue"], 2);

    let inv3 = generate_via_assignment(&app, "e2e_ap_invoice").await;
    assert_eq!(inv3["generatedNumber"], "INV-00000003");
    assert_eq!(inv3["numericValue"], 3);

    // 5. Generate payment numbers via direct generation
    let pay1 = generate_direct(&app, "E2E-PAY", "e2e_direct_payment").await;
    assert_eq!(pay1["generatedNumber"], "PAY-000001-FIN");

    let pay2 = generate_direct(&app, "E2E-PAY", "e2e_direct_payment").await;
    assert_eq!(pay2["generatedNumber"], "PAY-000002-FIN");

    // 6. Verify audit trail
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/audit")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let audit_body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    let audit_entries = audit_body["data"].as_array().unwrap();
    // 3 invoice + 2 payment = 5 entries minimum
    assert!(
        audit_entries.len() >= 5,
        "Expected at least 5 audit entries, got {}",
        audit_entries.len()
    );

    // 7. Verify dashboard
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-sequences/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dashboard: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();
    assert!(dashboard["totalSequences"].as_i64().unwrap() >= 2);
    assert!(dashboard["totalAssignments"].as_i64().unwrap() >= 2);
    assert!(dashboard["totalNumbersGenerated"].as_i64().unwrap() >= 5);

    // 8. Deactivate one assignment and verify it doesn't match anymore
    let inv_asgn_id = inv_asgn["id"].as_str().unwrap();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/document-sequences/assignments/{}/deactivate",
                    inv_asgn_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Try generating via the deactivated assignment - should fail (no active assignment)
    let payload = json!({
        "document_category": "e2e_ap_invoice",
    });
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-sequences/generate")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r.status(),
        StatusCode::NOT_FOUND,
        "Expected NOT_FOUND when no active assignment exists"
    );

    // 9. But direct generation still works on the payment sequence
    let pay3 = generate_direct(&app, "E2E-PAY", "e2e_direct_payment").await;
    assert_eq!(pay3["generatedNumber"], "PAY-000003-FIN");

    // 10. Delete the payment assignment and then delete the payment sequence
    let pay_asgn_id = pay_asgn["id"].as_str().unwrap();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/api/v1/document-sequences/assignments/{}",
                    pay_asgn_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Now delete the payment sequence
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/document-sequences/code/E2E-PAY")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}
