//! Direct Debit Mandate Management E2E Tests
//!
//! Tests for direct debit mandate lifecycle: mandate creation, activation,
//! collection processing (submit, complete, fail, return), revocation,
//! cancellation, and dashboard.
//! Oracle Fusion: Financials > Receivables > Direct Debit Mandates

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/149_direct_debit_mandate.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Mandate CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_mandate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-E2E-01",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "customer_name": "Acme Corporation",
                        "mandate_type": "core",
                        "bank_account_holder": "Acme Corporation",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_account_number_type": "iban",
                        "bank_code": "COBADEFFXXX",
                        "bank_code_type": "bic",
                        "bank_name": "Commerzbank",
                        "creditor_scheme_id": "CRED-001",
                        "mandate_reference": "MANDATE-REF-001",
                        "currency_code": "EUR",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["mandate_number"], "MANDATE-E2E-01");
    assert_eq!(d["status"], "draft");
    assert_eq!(d["mandate_type"], "core");
}

#[tokio::test]
async fn test_create_mandate_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_mandate_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-BAD",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "mandate_type": "crypto",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_mandate_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "mandate_number": "MANDATE-DUP",
        "customer_id": "00000000-0000-0000-0000-000000000100",
        "mandate_type": "core",
        "bank_account_number": "DE89370400440532013000",
        "bank_code": "COBADEFFXXX",
    }))
    .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_mandates() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/direct-debit-mandates")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_mandates_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/direct-debit-mandates?status=draft")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_mandate_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/direct-debit-mandates/{}",
                    uuid::Uuid::new_v4()
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_activate_mandate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-ACT",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "mandate_type": "core",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // Activate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/direct-debit-mandates/{}/activate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");
    assert!(d["activation_date"].is_string());
}

#[tokio::test]
async fn test_activate_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-ACT2",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    // Activate once
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/direct-debit-mandates/{}/activate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try again
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/direct-debit-mandates/{}/activate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_mandate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-CANC",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/direct-debit-mandates/{}/cancel", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Customer request",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

#[tokio::test]
async fn test_revoke_mandate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-REVOKE",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/direct-debit-mandates/{}/revoke", id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Customer revoked authorization",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "revoked");
}

// ========================================================================
// Full Workflow Test: Create → Activate → Collect → Submit → Complete → Return
// ========================================================================

#[tokio::test]
async fn test_full_workflow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-WF",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "customer_name": "Acme Corporation",
                        "mandate_type": "core",
                        "bank_account_holder": "Acme Corporation",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_account_number_type": "iban",
                        "bank_code": "COBADEFFXXX",
                        "bank_code_type": "bic",
                        "bank_name": "Commerzbank",
                        "currency_code": "EUR",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let mandate_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // 2. Activate mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-mandates/{}/activate",
                    mandate_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");

    // 3. Create first collection
    let inv_id = uuid::Uuid::new_v4().to_string();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-collections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_id": mandate_id,
                        "collection_number": "COLL-WF-001",
                        "collection_type": "first",
                        "amount": "5000.00",
                        "currency_code": "EUR",
                        "invoice_id": inv_id,
                        "invoice_number": "INV-001",
                        "scheduled_date": "2025-02-01",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let coll_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "pending");
    assert_eq!(d["collection_type"], "first");

    // 4. Submit collection
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/submit",
                    coll_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "submitted");

    // 5. Complete collection
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/complete",
                    coll_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "bank_reference": "BANK-REF-001",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "completed");
    assert_eq!(d["bank_reference"], "BANK-REF-001");

    // 6. Verify mandate is now 'used'
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/direct-debit-mandates/{}", mandate_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "used");

    // 7. Create a second (recurring) collection
    let inv2_id = uuid::Uuid::new_v4().to_string();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-collections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_id": mandate_id,
                        "collection_number": "COLL-WF-002",
                        "collection_type": "recurring",
                        "amount": "3000.00",
                        "currency_code": "EUR",
                        "invoice_id": inv2_id,
                        "invoice_number": "INV-002",
                        "scheduled_date": "2025-03-01",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let coll2_id = d["id"].as_str().unwrap();

    // 8. Submit and complete
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/submit",
                    coll2_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/complete",
                    coll2_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "bank_reference": "BANK-REF-002",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 9. Return the second collection
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/return",
                    coll2_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason_code": "R01",
                        "reason_text": "Insufficient funds",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "returned");
    assert_eq!(d["return_reason_code"], "R01");

    // 10. Revoke mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-mandates/{}/revoke",
                    mandate_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Customer revoked direct debit authorization",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "revoked");
}

#[tokio::test]
async fn test_collection_on_draft_mandate_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create mandate (stays draft)
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-DRAFT-COLL",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let mandate_id = d["id"].as_str().unwrap();

    // Try to create collection on draft mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-collections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_id": mandate_id,
                        "collection_number": "COLL-BAD",
                        "collection_type": "first",
                        "amount": "1000.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_fail_collection() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create and activate mandate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-mandates")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_number": "MANDATE-FAIL",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "bank_account_number": "DE89370400440532013000",
                        "bank_code": "COBADEFFXXX",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let mandate_id = d["id"].as_str().unwrap();
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-mandates/{}/activate",
                    mandate_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Create and submit collection
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/direct-debit-collections")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "mandate_id": mandate_id,
                        "collection_number": "COLL-FAIL",
                        "collection_type": "first",
                        "amount": "1000.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let coll_id = d["id"].as_str().unwrap();
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/submit",
                    coll_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Fail collection
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/direct-debit-collections/{}/fail",
                    coll_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason_code": "R04",
                        "reason_text": "Account closed",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "failed");
    assert_eq!(d["return_reason_code"], "R04");
}

// ========================================================================
// List Collections Test
// ========================================================================

#[tokio::test]
async fn test_list_collections() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/direct-debit-mandates/{}/collections",
                    uuid::Uuid::new_v4()
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ========================================================================
// Dashboard Test
// ========================================================================

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/direct-debit-mandates/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_mandates"], 0);
    assert_eq!(d["total_collected_amount"], "0.00");
}
