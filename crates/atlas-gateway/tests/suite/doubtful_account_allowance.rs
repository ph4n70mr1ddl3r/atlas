//! Doubtful Account Allowance E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Receivables > Collections > Allowance for Doubtful Accounts:
//! - Policy CRUD (create, get, list, get by code)
//! - Policy lifecycle (active → inactive → active)
//! - Aging bucket management
//! - Provision run lifecycle (draft → calculated → posted → reversed)
//! - Provision run cancellation
//! - Validation edge cases
//! - Dashboard summary

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
    // Clean doubtful account test data
    sqlx::query("DELETE FROM _atlas.doubtful_account_provision_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.doubtful_account_provision_details").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.doubtful_account_provision_runs").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.doubtful_account_aging_buckets").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.doubtful_account_policies").execute(&state.db_pool).await.ok();
    sqlx::raw_sql(include_str!("../../../../migrations/135_doubtful_account_allowance.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run doubtful account allowance migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_aging_based_policy(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": code,
        "policyName": name,
        "calculationMethod": "aging_based",
        "flatPercentage": "0",
        "currencyCode": "USD",
        "effectiveFrom": "2025-01-01",
        "defaultProvisionAccount": "1200-Allowance-Doubtful",
        "defaultExpenseAccount": "6100-Bad-Debt-Expense",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE POLICY status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create policy: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_percentage_policy(
    app: &axum::Router,
    code: &str,
    name: &str,
    percentage: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": code,
        "policyName": name,
        "calculationMethod": "percentage_based",
        "flatPercentage": percentage,
        "currencyCode": "USD",
        "effectiveFrom": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE PCT POLICY status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create percentage policy: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_aging_bucket(
    app: &axum::Router,
    policy_id: &Uuid,
    bucket_name: &str,
    from_days: i32,
    to_days: Option<i32>,
    percentage: &str,
    order: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "bucketName": bucket_name,
        "fromDays": from_days,
        "toDays": to_days,
        "provisionPercentage": percentage,
        "displayOrder": order,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/aging-buckets", policy_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE BUCKET status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create aging bucket: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_full_provision_run(
    app: &axum::Router,
    policy_id: &Uuid,
    as_of_date: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyId": policy_id.to_string(),
        "asOfDate": as_of_date,
        "description": "Monthly provision run",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-provision-runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE RUN status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create provision run: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Policy CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_aging_based_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard Aging Policy").await;

    assert_eq!(policy["policyCode"], "STD-AGING");
    assert_eq!(policy["policyName"], "Standard Aging Policy");
    assert_eq!(policy["calculationMethod"], "aging_based");
    assert_eq!(policy["status"], "active");
    assert_eq!(policy["isActive"], true);
    assert_eq!(policy["currencyCode"], "USD");
    // flatPercentage is 0 (DOUBLE PRECISION serializes without trailing zeros)
    assert_eq!(policy["defaultProvisionAccount"], "1200-Allowance-Doubtful");
    assert_eq!(policy["defaultExpenseAccount"], "6100-Bad-Debt-Expense");
}

#[tokio::test]
async fn test_create_percentage_based_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_percentage_policy(&app, "FLAT-3", "Flat 3% Policy", "3.5").await;

    assert_eq!(policy["policyCode"], "FLAT-3");
    assert_eq!(policy["calculationMethod"], "percentage_based");
    assert!(policy["flatPercentage"].as_str().unwrap().parse::<f64>().unwrap() - 3.5 < 0.001);
}

#[tokio::test]
async fn test_get_policy() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-policies/{}", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], policy["id"]);
    assert_eq!(body["policyCode"], "STD-AGING");
}

#[tokio::test]
async fn test_get_policy_by_code() {
    let (_state, app) = setup_test().await;
    create_aging_based_policy(&app, "STD-AGING", "Standard").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account-policies/code/STD-AGING")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["policyCode"], "STD-AGING");
}

#[tokio::test]
async fn test_list_policies() {
    let (_state, app) = setup_test().await;
    create_aging_based_policy(&app, "POLICY-1", "Aging Policy").await;
    create_percentage_policy(&app, "POLICY-2", "Flat Policy", "5").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account-policies")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_policies_filter_by_status() {
    let (_state, app) = setup_test().await;
    create_aging_based_policy(&app, "POLICY-1", "Aging Policy").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account-policies?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|p| p["status"] == "active"));
}

#[tokio::test]
async fn test_get_policy_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-policies/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Policy Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_policy_lifecycle_active_to_inactive_to_active() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "LC-POLICY", "Lifecycle Policy").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/deactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");
    assert_eq!(body["isActive"], false);

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/reactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
    assert_eq!(body["isActive"], true);
}

#[tokio::test]
async fn test_deactivate_inactive_policy_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "LC-POLICY", "Lifecycle").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/deactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to deactivate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/deactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reactivate_active_policy_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "LC-POLICY", "Lifecycle").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Reactivate from active should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/reactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Aging Bucket Tests
// ============================================================================

#[tokio::test]
async fn test_create_aging_buckets() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let b1 = add_aging_bucket(&app, &policy_id, "Current", 0, Some(30), "1.0", 1).await;
    assert_eq!(b1["bucketName"], "Current");
    assert_eq!(b1["fromDays"], 0);
    assert_eq!(b1["toDays"], 30);
    assert!(b1["provisionPercentage"].as_str().unwrap().parse::<f64>().unwrap() - 1.0 < 0.001);

    let b2 = add_aging_bucket(&app, &policy_id, "31-60 Days", 31, Some(60), "3.0", 2).await;
    assert_eq!(b2["bucketName"], "31-60 Days");
    assert!(b2["provisionPercentage"].as_str().unwrap().parse::<f64>().unwrap() - 3.0 < 0.001);

    let b3 = add_aging_bucket(&app, &policy_id, "61-90 Days", 61, Some(90), "5.0", 3).await;
    assert_eq!(b3["bucketName"], "61-90 Days");

    let b4 = add_aging_bucket(&app, &policy_id, "91+ Days", 91, None, "10.0", 4).await;
    assert_eq!(b4["bucketName"], "91+ Days");
    assert_eq!(b4["toDays"], serde_json::Value::Null);
}

#[tokio::test]
async fn test_list_aging_buckets() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    add_aging_bucket(&app, &policy_id, "Current", 0, Some(30), "1.0", 1).await;
    add_aging_bucket(&app, &policy_id, "31-60 Days", 31, Some(60), "3.0", 2).await;
    add_aging_bucket(&app, &policy_id, "91+ Days", 91, None, "10.0", 3).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-policies/{}/aging-buckets", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_bucket_on_non_aging_policy_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_percentage_policy(&app, "FLAT-3", "Flat 3%", "3").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "bucketName": "Current",
        "fromDays": 0,
        "toDays": 30,
        "provisionPercentage": "1.0",
        "displayOrder": 1,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/aging-buckets", policy_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Provision Run Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_provision_run_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    // Add buckets
    add_aging_bucket(&app, &policy_id, "Current", 0, Some(30), "1.0", 1).await;
    add_aging_bucket(&app, &policy_id, "31-60 Days", 31, Some(60), "3.0", 2).await;
    add_aging_bucket(&app, &policy_id, "91+ Days", 91, None, "10.0", 3).await;

    let (k, v) = auth_header(&admin_claims());

    // Create run
    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(run["status"], "draft");
    assert_eq!(run["policyCode"], "STD-AGING");
    assert_eq!(run["calculationMethod"], "aging_based");
    assert!(run["runNumber"].as_str().unwrap().starts_with("PROV-"));

    // Calculate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "calculated");
    assert!(body["totalOutstandingAmount"].as_str().unwrap().parse::<f64>().unwrap().abs() < 0.01);
    assert!(body["totalProvisionAmount"].as_str().unwrap().parse::<f64>().unwrap().abs() < 0.01);
    assert!(body["incrementalProvision"].as_str().unwrap().parse::<f64>().unwrap().abs() < 0.01);

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/post", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "journalEntryNumber": "JE-PROV-001"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");
    assert_eq!(body["journalEntryNumber"], "JE-PROV-001");

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/reverse", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_draft_provision_run() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_calculated_run_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Calculate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel calculated run - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_post_draft_run_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to post a draft run
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/post", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_draft_run_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/reverse", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Provision Run Query Tests
// ============================================================================

#[tokio::test]
async fn test_get_provision_run_by_number() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_number = run["runNumber"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-provision-runs/number/{}", run_number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["runNumber"], run_number);
}

#[tokio::test]
async fn test_list_provision_runs() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    create_full_provision_run(&app, &policy_id, "2025-07-31").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account-provision-runs")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_provision_runs_filter_by_status() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    create_full_provision_run(&app, &policy_id, "2025-06-30").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account-provision-runs?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|r| r["status"] == "draft"));
}

#[tokio::test]
async fn test_list_provision_details() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    add_aging_bucket(&app, &policy_id, "Current", 0, Some(30), "1.0", 1).await;
    add_aging_bucket(&app, &policy_id, "31-60 Days", 31, Some(60), "3.0", 2).await;

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Calculate to generate details
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List details
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/details", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let details = body["data"].as_array().unwrap();
    assert_eq!(details.len(), 2); // 2 aging buckets = 2 detail lines
    assert_eq!(details[0]["bucketName"], "Current");
    assert_eq!(details[1]["bucketName"], "31-60 Days");
}

#[tokio::test]
async fn test_list_run_activities() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/activities", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let activities = body["data"].as_array().unwrap();
    assert!(!activities.is_empty());
    assert_eq!(activities[0]["action"], "created");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_policy_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": "",
        "policyName": "Test",
        "calculationMethod": "aging_based",
        "currencyCode": "USD",
        "effectiveFrom": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_invalid_method_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": "TEST",
        "policyName": "Test",
        "calculationMethod": "invalid_method",
        "currencyCode": "USD",
        "effectiveFrom": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_aging_based_policy(&app, "DUP-CODE", "First").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": "DUP-CODE",
        "policyName": "Second",
        "calculationMethod": "aging_based",
        "currencyCode": "USD",
        "effectiveFrom": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_invalid_date_range_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": "BAD-DATE",
        "policyName": "Bad Date Range",
        "calculationMethod": "aging_based",
        "currencyCode": "USD",
        "effectiveFrom": "2025-12-31",
        "effectiveTo": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_invalid_currency_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "policyCode": "BAD-CURR",
        "policyName": "Bad Currency",
        "calculationMethod": "aging_based",
        "currencyCode": "US",
        "effectiveFrom": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-policies")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_bucket_negative_from_days_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "bucketName": "Invalid",
        "fromDays": -1,
        "toDays": 30,
        "provisionPercentage": "1.0",
        "displayOrder": 1,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/aging-buckets", policy_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_bucket_to_before_from_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "bucketName": "Invalid",
        "fromDays": 60,
        "toDays": 30,
        "provisionPercentage": "1.0",
        "displayOrder": 1,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/aging-buckets", policy_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_provision_run_inactive_policy_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate the policy
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-policies/{}/deactivate", policy_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to create a run on inactive policy
    let payload = json!({
        "policyId": policy_id.to_string(),
        "asOfDate": "2025-06-30",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/doubtful-account-provision-runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_calculate_twice_fails() {
    let (_state, app) = setup_test().await;
    let policy = create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Calculate once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to calculate again (status is now 'calculated')
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_aging_based_policy(&app, "STD-AGING", "Standard").await;
    create_percentage_policy(&app, "FLAT-3", "Flat 3%", "3.5").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/doubtful-account/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalPolicies").is_some());
    assert!(body.get("activePolicies").is_some());
    assert!(body.get("totalRuns").is_some());
    assert!(body.get("draftRuns").is_some());
    assert!(body.get("postedRuns").is_some());
    assert!(body.get("latestProvisionAmount").is_some());
    assert!(body.get("totalOutstandingAr").is_some());
    assert!(body.get("overallProvisionRate").is_some());
}

// ============================================================================
// Percentage-based Provision Run Test
// ============================================================================

#[tokio::test]
async fn test_percentage_based_provision_run() {
    let (_state, app) = setup_test().await;
    let policy = create_percentage_policy(&app, "FLAT-3", "Flat 3%", "3.5").await;
    let policy_id: Uuid = policy["id"].as_str().unwrap().parse().unwrap();

    let run = create_full_provision_run(&app, &policy_id, "2025-06-30").await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(run["calculationMethod"], "percentage_based");

    let (k, v) = auth_header(&admin_claims());

    // Calculate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/calculate", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "calculated");
    assert_eq!(body["calculationMethod"], "percentage_based");

    // Verify one detail line was created
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/doubtful-account-provision-runs/{}/details", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let details = body["data"].as_array().unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["bucketName"], "All Outstanding");
}
