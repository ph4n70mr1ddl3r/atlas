//! Cost Allocation E2E Tests (Oracle Fusion GL > Allocations / Mass Allocations)
//!
//! Tests for Oracle Fusion Cloud ERP Cost Allocation:
//! - Allocation pool CRUD
//! - Allocation base CRUD and base value entry
//! - Allocation rule CRUD with targets
//! - Rule activation workflow
//! - Proportional allocation execution
//! - Fixed percent allocation execution
//! - Run posting and reversal
//! - Allocation summary dashboard
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_cost_allocation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_pool(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test cost pool",
        "pool_type": "cost_center",
        "source_account_codes": ["6100", "6110", "6120"],
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/pools")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create pool: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_base(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test allocation base",
        "base_type": "statistical",
        "unit_of_measure": "persons",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/bases")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create base: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_rule(
    app: &axum::Router,
    name: &str,
    pool_code: &str,
    base_code: &str,
    method: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": name,
        "description": "Test allocation rule",
        "pool_code": pool_code,
        "base_code": base_code,
        "allocation_method": method,
        "journal_description": "Monthly cost allocation",
        "offset_account_code": "9999",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create rule: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Allocation Pool Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_allocation_pool() {
    let (_state, app) = setup_cost_allocation_test().await;

    let pool = create_test_pool(&app, "RENT_POOL", "Rent Cost Pool").await;

    assert_eq!(pool["code"], "RENT_POOL");
    assert_eq!(pool["name"], "Rent Cost Pool");
    assert_eq!(pool["pool_type"], "cost_center");
    assert!(pool["is_active"].as_bool().unwrap());
}

#[tokio::test]
#[ignore]
async fn test_list_pools() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "POOL_A", "Pool A").await;
    create_test_pool(&app, "POOL_B", "Pool B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/cost-allocation/pools")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_get_pool() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "IT_POOL", "IT Overhead Pool").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cost-allocation/pools/IT_POOL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(pool["code"], "IT_POOL");
}

#[tokio::test]
#[ignore]
async fn test_delete_pool() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "DEL_POOL", "Pool to Delete").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/cost-allocation/pools/DEL_POOL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cost-allocation/pools/DEL_POOL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Allocation Base Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_allocation_base() {
    let (_state, app) = setup_cost_allocation_test().await;

    let base = create_test_base(&app, "HEADCOUNT", "Employee Headcount").await;

    assert_eq!(base["code"], "HEADCOUNT");
    assert_eq!(base["name"], "Employee Headcount");
    assert_eq!(base["base_type"], "statistical");
    assert_eq!(base["unit_of_measure"], "persons");
}

#[tokio::test]
#[ignore]
async fn test_set_base_values() {
    let (_state, app) = setup_cost_allocation_test().await;

    let base = create_test_base(&app, "HEADCOUNT", "Employee Headcount").await;
    let base_id = base["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Set values for departments
    let departments = vec![
        ("Engineering", "50"),
        ("Marketing", "30"),
        ("Sales", "20"),
    ];

    for (dept, val) in &departments {
        let payload = json!({
            "base_code": "HEADCOUNT",
            "department_name": dept,
            "value": val,
            "effective_date": "2024-01-01",
        });
        let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/base-values")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // List base values
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/cost-allocation/base-values?base_id={}", base_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Allocation Rule Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_allocation_rule() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let rule = create_test_rule(&app, "Rent by Headcount", "RENT_POOL", "HEADCOUNT", "proportional").await;

    assert_eq!(rule["name"], "Rent by Headcount");
    assert_eq!(rule["allocation_method"], "proportional");
    assert_eq!(rule["status"], "draft");
    assert_eq!(rule["pool_code"], "RENT_POOL");
    assert_eq!(rule["base_code"], "HEADCOUNT");
    assert!(rule["rule_number"].as_str().unwrap().starts_with("ALLOC-"));
}

#[tokio::test]
#[ignore]
async fn test_cannot_activate_rule_without_targets() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let rule = create_test_rule(&app, "No Targets Rule", "RENT_POOL", "HEADCOUNT", "proportional").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_activate_rule_with_targets() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let rule = create_test_rule(&app, "Rent Rule", "RENT_POOL", "HEADCOUNT", "proportional").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add targets
    let targets = vec![
        ("Engineering", "6200"),
        ("Marketing", "6300"),
        ("Sales", "6400"),
    ];
    for (dept, account) in &targets {
        let payload = json!({
            "department_name": dept,
            "target_account_code": account,
        });
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cost-allocation/rules/{}/targets", rule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED, "Failed to add target");
    }

    // Now activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

// ============================================================================
// Proportional Allocation Execution Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_execute_proportional_allocation() {
    let (_state, app) = setup_cost_allocation_test().await;

    // Setup pool and base
    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    let _base = create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let (k, v) = auth_header(&admin_claims());

    // Set base values: Eng=50, Marketing=30, Sales=20 (total=100)
    for (dept, val) in &[("Engineering", "50"), ("Marketing", "30"), ("Sales", "20")] {
        let payload = json!({
            "base_code": "HEADCOUNT",
            "department_name": dept,
            "value": val,
            "effective_date": "2024-01-01",
        });
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/base-values")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Create rule
    let rule = create_test_rule(&app, "Rent Allocation", "RENT_POOL", "HEADCOUNT", "proportional").await;
    let rule_id = rule["id"].as_str().unwrap();

    // Add targets
    for (dept, account) in &[("Engineering", "6200"), ("Marketing", "6300"), ("Sales", "6400")] {
        let payload = json!({
            "department_name": dept,
            "target_account_code": account,
        });
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cost-allocation/rules/{}/targets", rule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Activate
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Execute with $100,000 source amount
    let payload = json!({
        "source_amount": "100000",
        "period_start": "2024-01-01",
        "period_end": "2024-01-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/execute", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(run["status"], "draft");
    assert_eq!(run["line_count"], 4); // 3 debit + 1 credit
    assert_eq!(run["total_source_amount"].as_str().unwrap().parse::<f64>().unwrap(), 100000.0);

    let run_id = run["id"].as_str().unwrap();

    // Verify run lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/cost-allocation/runs/{}/lines", run_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_list = lines["data"].as_array().unwrap();

    assert_eq!(line_list.len(), 4);

    // Check debit lines
    let debit_lines: Vec<_> = line_list.iter()
        .filter(|l| l["line_type"] == "debit").collect();
    assert_eq!(debit_lines.len(), 3);

    // Engineering should get 50% = $50,000
    let eng = debit_lines.iter().find(|l| l["department_name"] == "Engineering").unwrap();
    let eng_amount: f64 = eng["amount"].as_str().unwrap().parse().unwrap();
    assert!((eng_amount - 50000.0).abs() < 1.0, "Expected 50000, got {}", eng_amount);

    // Marketing should get 30% = $30,000
    let mkt = debit_lines.iter().find(|l| l["department_name"] == "Marketing").unwrap();
    let mkt_amount: f64 = mkt["amount"].as_str().unwrap().parse().unwrap();
    assert!((mkt_amount - 30000.0).abs() < 1.0, "Expected 30000, got {}", mkt_amount);

    // Sales should get 20% = $20,000
    let sales = debit_lines.iter().find(|l| l["department_name"] == "Sales").unwrap();
    let sales_amount: f64 = sales["amount"].as_str().unwrap().parse().unwrap();
    assert!((sales_amount - 20000.0).abs() < 1.0, "Expected 20000, got {}", sales_amount);

    // Check offset credit line
    let credit_lines: Vec<_> = line_list.iter()
        .filter(|l| l["line_type"] == "credit").collect();
    assert_eq!(credit_lines.len(), 1);
    assert_eq!(credit_lines[0]["account_code"], "9999");
}

// ============================================================================
// Fixed Percent Allocation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_execute_fixed_percent_allocation() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "IT_POOL", "IT Pool").await;
    create_test_base(&app, "STATIC", "Static Base").await;

    let (k, v) = auth_header(&admin_claims());

    let rule = create_test_rule(&app, "IT Allocation", "IT_POOL", "STATIC", "fixed_percent").await;
    let rule_id = rule["id"].as_str().unwrap();

    // Add targets with fixed percentages
    let targets = vec![
        ("Engineering", "6200", "60"),
        ("Marketing", "6300", "25"),
        ("Sales", "6400", "15"),
    ];
    for (dept, account, pct) in &targets {
        let payload = json!({
            "department_name": dept,
            "target_account_code": account,
            "fixed_percent": pct,
        });
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cost-allocation/rules/{}/targets", rule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Activate
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Execute
    let payload = json!({
        "source_amount": "50000",
        "period_start": "2024-01-01",
        "period_end": "2024-01-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/execute", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Check lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/cost-allocation/runs/{}/lines", run_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_list = lines["data"].as_array().unwrap();

    // Engineering 60% = $30,000
    let eng = line_list.iter().find(|l| l["department_name"] == "Engineering" && l["line_type"] == "debit").unwrap();
    let eng_amount: f64 = eng["amount"].as_str().unwrap().parse().unwrap();
    assert!((eng_amount - 30000.0).abs() < 1.0, "Expected 30000, got {}", eng_amount);

    // Marketing 25% = $12,500
    let mkt = line_list.iter().find(|l| l["department_name"] == "Marketing" && l["line_type"] == "debit").unwrap();
    let mkt_amount: f64 = mkt["amount"].as_str().unwrap().parse().unwrap();
    assert!((mkt_amount - 12500.0).abs() < 1.0, "Expected 12500, got {}", mkt_amount);
}

// ============================================================================
// Run Posting & Reversal Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_post_and_reverse_allocation_run() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let (k, v) = auth_header(&admin_claims());

    // Set base values
    for (dept, val) in &[("Engineering", "50"), ("Marketing", "30")] {
        let payload = json!({
            "base_code": "HEADCOUNT",
            "department_name": dept,
            "value": val,
            "effective_date": "2024-01-01",
        });
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/base-values")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Create and activate rule
    let rule = create_test_rule(&app, "Test Rule", "RENT_POOL", "HEADCOUNT", "proportional").await;
    let rule_id = rule["id"].as_str().unwrap();

    for (dept, account) in &[("Engineering", "6200"), ("Marketing", "6300")] {
        let payload = json!({ "department_name": dept, "target_account_code": account });
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cost-allocation/rules/{}/targets", rule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Execute
    let payload = json!({ "source_amount": "10000", "period_start": "2024-01-01", "period_end": "2024-01-31" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/execute", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let run_id = run["id"].as_str().unwrap();
    assert_eq!(run["status"], "draft");

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/runs/{}/post", run_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let posted: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(posted["status"], "posted");

    // Reverse
    let payload = json!({ "reason": "Correction needed - incorrect source amount" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/runs/{}/reverse", run_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let reversed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(reversed["status"], "reversed");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_create_pool_with_empty_code() {
    let (_state, app) = setup_cost_allocation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "",
        "name": "Bad Pool",
        "pool_type": "cost_center",
        "source_account_codes": [],
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/pools")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_rule_with_invalid_pool() {
    let (_state, app) = setup_cost_allocation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": "Bad Rule",
        "pool_code": "NONEXISTENT",
        "base_code": "ALSO_NONEXISTENT",
        "allocation_method": "proportional",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_cannot_execute_draft_rule() {
    let (_state, app) = setup_cost_allocation_test().await;

    create_test_pool(&app, "RENT_POOL", "Rent Pool").await;
    create_test_base(&app, "HEADCOUNT", "Headcount").await;

    let rule = create_test_rule(&app, "Draft Rule", "RENT_POOL", "HEADCOUNT", "proportional").await;
    let rule_id = rule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "source_amount": "1000", "period_start": "2024-01-01", "period_end": "2024-01-31" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/execute", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cost_allocation_full_lifecycle() {
    let (_state, app) = setup_cost_allocation_test().await;

    let (k, v) = auth_header(&admin_claims());

    // 1. Create pool
    let pool = create_test_pool(&app, "OVERHEAD", "Overhead Cost Pool").await;
    assert_eq!(pool["code"], "OVERHEAD");

    // 2. Create base
    let base = create_test_base(&app, "SQFT", "Square Footage").await;
    assert_eq!(base["code"], "SQFT");

    // 3. Set base values
    for (dept, sqft) in &[("Operations", "5000"), ("R&D", "3000"), ("Admin", "2000")] {
        let payload = json!({
            "base_code": "SQFT",
            "department_name": dept,
            "value": sqft,
            "effective_date": "2024-01-01",
        });
        let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cost-allocation/base-values")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // 4. Create rule
    let rule = create_test_rule(&app, "Overhead by SQFT", "OVERHEAD", "SQFT", "proportional").await;
    assert_eq!(rule["status"], "draft");
    let rule_id = rule["id"].as_str().unwrap();

    // 5. Add targets
    for (dept, account) in &[("Operations", "7100"), ("R&D", "7200"), ("Admin", "7300")] {
        let payload = json!({ "department_name": dept, "target_account_code": account });
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/cost-allocation/rules/{}/targets", rule_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // 6. Activate rule
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/activate", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 7. Execute allocation ($200,000 overhead)
    let payload = json!({ "source_amount": "200000", "period_start": "2024-01-01", "period_end": "2024-01-31" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/rules/{}/execute", rule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let run: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // 8. Post the run
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/cost-allocation/runs/{}/post", run_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 9. Check dashboard
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cost-allocation/summary")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["active_rule_count"], 1);
    assert_eq!(summary["pool_count"], 1);
    assert!(summary["total_allocated_amount"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);

    // 10. List runs
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cost-allocation/runs")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let runs: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(runs["data"].as_array().unwrap().len(), 1);
}
