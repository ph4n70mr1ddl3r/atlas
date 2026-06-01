//! Average Balance Processing E2E Tests
//!
//! Tests for average balance processing - daily average balance calculations,
//! weighted averages, peak/trough tracking, and regulatory reporting support.
//! Oracle Fusion: Financials > General Ledger > Average Balances

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/141_average_balance_processing.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Book CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_book() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-E2E-01",
                        "book_name": "Primary Average Balance Book",
                        "description": "Main averaging book for regulatory reporting",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "is_primary": true,
                        "currency_code": "USD",
                        "effective_from": "2025-01-01",
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
    assert_eq!(d["book_code"], "AVG-E2E-01");
    assert_eq!(d["book_name"], "Primary Average Balance Book");
    assert_eq!(d["period_type"], "daily");
    assert_eq!(d["averaging_window_days"], 30);
    assert_eq!(d["is_primary"], true);
    assert_eq!(d["status"], "active");
}

#[tokio::test]
async fn test_create_monthly_book() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-E2E-MON",
                        "book_name": "Monthly Average Book",
                        "period_type": "monthly",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    assert_eq!(d["period_type"], "monthly");
}

#[tokio::test]
async fn test_create_book_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
async fn test_create_book_invalid_period_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-BAD",
                        "book_name": "Book",
                        "period_type": "hourly",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
async fn test_create_book_zero_window_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-ZW",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 0,
                        "currency_code": "USD",
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
async fn test_duplicate_book_code() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "book_code": "AVG-DUP",
        "book_name": "Book 1",
        "period_type": "daily",
        "averaging_window_days": 30,
        "currency_code": "USD",
    }))
    .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
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
                .uri("/api/v1/avg-balance/books")
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
async fn test_list_books() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/avg-balance/books")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_books_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/avg-balance/books?status=active")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_book_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/avg-balance/books/{}",
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

// ========================================================================
// Book Activate/Deactivate/Delete Tests
// ========================================================================

#[tokio::test]
async fn test_activate_deactivate_book() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-ACT-DEACT",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    assert_eq!(d["status"], "active");

    // Deactivate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/deactivate", id))
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
    assert_eq!(d["status"], "inactive");

    // Reactivate
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/activate", id))
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
}

#[tokio::test]
async fn test_delete_active_book_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-DEL-ACT",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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

    // Can't delete active book
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/avg-balance/books/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Account Tests
// ========================================================================

#[tokio::test]
async fn test_add_account_to_book() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create book
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-ACC-BOOK",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_id = d["id"].as_str().unwrap();

    // Add account
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "gl_account": "1000-100",
                        "gl_account_name": "Cash - Operating",
                        "account_type": "asset",
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
    assert_eq!(d["gl_account"], "1000-100");
    assert_eq!(d["gl_account_name"], "Cash - Operating");
    assert_eq!(d["account_type"], "asset");
}

#[tokio::test]
async fn test_list_accounts() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-LIST-ACC",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_add_duplicate_account_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-ACC-DUP",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_id = d["id"].as_str().unwrap();

    let body = serde_json::to_string(&json!({
        "gl_account": "1000",
        "account_type": "asset",
    }))
    .unwrap();

    // First should succeed
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Second should conflict
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ========================================================================
// Daily Balance Tests
// ========================================================================

#[tokio::test]
async fn test_upsert_daily_balance() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Setup book + account
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-DB-BOOK",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let book_id = book_d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "gl_account": "1000",
                        "account_type": "asset",
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
    let acc_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let acc_id = acc_d["id"].as_str().unwrap();

    // Record daily balance
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/avg-balance/books/{}/daily-balances",
                    book_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "account_id": acc_id,
                        "balance_date": "2025-01-15",
                        "closing_balance": "10500.00",
                        "opening_balance": "10000.00",
                        "total_debits": "1000.00",
                        "total_credits": "500.00",
                        "transaction_count": 5,
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
    assert_eq!(d["closing_balance"], "10500.00");
    assert_eq!(d["opening_balance"], "10000.00");
}

#[tokio::test]
async fn test_daily_balance_mismatch_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-DB-MIS",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let book_id = book_d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "gl_account": "2000",
                        "account_type": "liability",
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
    let acc_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let acc_id = acc_d["id"].as_str().unwrap();

    // closing(9000) != opening(10000) + debits(1000) - credits(500) = 10500
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/avg-balance/books/{}/daily-balances",
                    book_id
                ))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "account_id": acc_id,
                        "balance_date": "2025-01-15",
                        "closing_balance": "9000.00",
                        "opening_balance": "10000.00",
                        "total_debits": "1000.00",
                        "total_credits": "500.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Full Workflow: Create → Record → Calculate → Approve → Post
// ========================================================================

#[tokio::test]
async fn test_full_workflow_calculate_approve_post() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create book
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-WF",
                        "book_name": "Workflow Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let book_id = book_d["id"].as_str().unwrap();

    // 2. Add account
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "gl_account": "1000",
                        "gl_account_name": "Cash",
                        "account_type": "asset",
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
    let acc_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let acc_id = acc_d["id"].as_str().unwrap();

    // 3. Record daily balances (3 days)
    for (date, closing, opening, debits, credits) in [
        ("2025-01-01", "10000.00", "9000.00", "2000.00", "1000.00"),
        ("2025-01-02", "12000.00", "10000.00", "3000.00", "1000.00"),
        ("2025-01-03", "11000.00", "12000.00", "1000.00", "2000.00"),
    ] {
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!(
                        "/api/v1/avg-balance/books/{}/daily-balances",
                        book_id
                    ))
                    .header("Content-Type", "application/json")
                    .header(&k, &v)
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "account_id": acc_id,
                            "balance_date": date,
                            "closing_balance": closing,
                            "opening_balance": opening,
                            "total_debits": debits,
                            "total_credits": credits,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

    // 4. Calculate average balance
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/calculate", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "account_id": acc_id,
                        "period_start_date": "2025-01-01",
                        "period_end_date": "2025-01-03",
                        "calculation_type": "daily",
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
    let calc_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(calc_d["status"], "calculated");
    assert_eq!(calc_d["days_in_period"], 3);
    assert_eq!(calc_d["average_balance"], "11000.00");
    assert_eq!(calc_d["peak_balance"], "12000.00");
    assert_eq!(calc_d["trough_balance"], "10000.00");
    let calc_id = calc_d["id"].as_str().unwrap();

    // 5. Approve calculation
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/avg-balance/calculations/{}/approve",
                    calc_id
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
    assert_eq!(d["status"], "approved");

    // 6. Post calculation
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/avg-balance/calculations/{}/post",
                    calc_id
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
    assert_eq!(d["status"], "posted");
}

#[tokio::test]
async fn test_approve_non_calculated_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/avg-balance/calculations/{}/approve",
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
async fn test_calculate_empty_period_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create book + account
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/avg-balance/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "book_code": "AVG-CALC-EMPTY",
                        "book_name": "Book",
                        "period_type": "daily",
                        "averaging_window_days": 30,
                        "currency_code": "USD",
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
    let book_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let book_id = book_d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/accounts", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "gl_account": "1000",
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
    let acc_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let acc_id = acc_d["id"].as_str().unwrap();

    // Calculate with no daily balances recorded
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/avg-balance/books/{}/calculate", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "account_id": acc_id,
                        "period_start_date": "2025-02-01",
                        "period_end_date": "2025-02-28",
                        "calculation_type": "monthly",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
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
                .uri("/api/v1/avg-balance/dashboard")
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
    assert_eq!(d["total_books"], 0);
    assert_eq!(d["active_books"], 0);
}

// ========================================================================
// List Calculations Test
// ========================================================================

#[tokio::test]
async fn test_list_calculations() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/avg-balance/books/{}/calculations",
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
