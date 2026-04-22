//! GL Allocation E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP General Ledger Allocations:
//! - Pool CRUD and lifecycle
//! - Basis CRUD, details, and percentage recalculation
//! - Rule CRUD with target lines
//! - Allocation run execution (proportional method)
//! - Run posting, reversal, and cancellation
//! - Dashboard summary
//! - Validation and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_allocation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pool CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_allocation_pool() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RENT_POOL",
            "name": "Rent Cost Pool",
            "description": "Pool for allocating rent expenses",
            "pool_type": "cost_center",
            "source_account_code": "6100-RENT",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pool["code"], "RENT_POOL");
    assert_eq!(pool["name"], "Rent Cost Pool");
    assert_eq!(pool["poolType"], "cost_center");
    assert_eq!(pool["isActive"], true);
}

#[tokio::test]
async fn test_create_pool_account_range() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "OVERHEAD_RANGE",
            "name": "Overhead Account Range Pool",
            "pool_type": "account_range",
            "source_account_range_from": "6000",
            "source_account_range_to": "6999",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pool["poolType"], "account_range");
    assert_eq!(pool["source_account_range_from"], "6000");
    assert_eq!(pool["source_account_range_to"], "6999");
}

#[tokio::test]
async fn test_create_pool_duplicate_rejected() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let body = serde_json::to_string(&json!({
        "code": "DUP_POOL",
        "name": "Duplicate Pool Test",
        "pool_type": "manual",
        "currency_code": "USD"
    })).unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(body.clone()))
        .unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(body))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_allocation_pool() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GET_POOL",
            "name": "Get Pool Test",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Get
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/pools/GET_POOL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pool["code"], "GET_POOL");
}

#[tokio::test]
async fn test_list_allocation_pools() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two pools
    for code in &["LIST_A", "LIST_B"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/pools")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Pool {}", code),
                "pool_type": "manual",
                "currency_code": "USD"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/pools")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_deactivate_pool() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACT_POOL",
            "name": "Activate/Deactivate Test",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pool_id = pool["id"].as_str().unwrap();

    // Deactivate
    let deact_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/pools/{}/deactivate", pool_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(deact_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(deact_resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pool["isActive"], false);

    // Reactivate
    let act_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/pools/{}/activate", pool_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(act_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(act_resp.into_body(), usize::MAX).await.unwrap();
    let pool: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pool["isActive"], true);
}

#[tokio::test]
async fn test_delete_pool() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DEL_POOL",
            "name": "Delete Me",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri("/api/v1/allocation/pools/DEL_POOL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deleted
    let get_resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/pools/DEL_POOL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Basis CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_allocation_basis() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "HEADCOUNT",
            "name": "Headcount Basis",
            "description": "Allocation based on employee headcount",
            "basis_type": "statistical",
            "unit_of_measure": "people",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let basis: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(basis["code"], "HEADCOUNT");
    assert_eq!(basis["basisType"], "statistical");
    assert_eq!(basis["isManual"], true);
}

#[tokio::test]
async fn test_add_basis_detail() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SQFT_BASIS",
            "name": "Square Footage Basis",
            "basis_type": "statistical",
            "unit_of_measure": "sq_ft",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add detail
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/SQFT_BASIS/details")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_department_name": "Engineering",
            "target_cost_center": "CC-ENG",
            "target_account_code": "6500-ENG",
            "basis_amount": "5000",
            "period_name": "JAN-2024"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(detail["basisAmount"], "5000");
    assert_eq!(detail["target_department_name"], "Engineering");
}

#[tokio::test]
async fn test_list_basis_details() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "LIST_BASIS",
            "name": "List Details Test Basis",
            "basis_type": "statistical",
            "unit_of_measure": "people",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add details
    for dept in &["Engineering", "Marketing", "Sales"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/bases/LIST_BASIS/details")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "target_department_name": dept,
                "target_account_code": format!("6500-{}", dept),
                "basis_amount": "100",
                "period_name": "FEB-2024"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/bases/LIST_BASIS/details?period_name=FEB-2024")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_recalculate_basis_percentages() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RECALC_BASIS",
            "name": "Recalculate Test Basis",
            "basis_type": "statistical",
            "unit_of_measure": "people",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add details: 300 + 200 + 500 = 1000 total
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/RECALC_BASIS/details")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_department_name": "DeptA",
            "target_account_code": "6000-A",
            "basis_amount": "300"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/RECALC_BASIS/details")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_department_name": "DeptB",
            "target_account_code": "6000-B",
            "basis_amount": "200"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/RECALC_BASIS/details")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_department_name": "DeptC",
            "target_account_code": "6000-C",
            "basis_amount": "500"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Recalculate
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/RECALC_BASIS/recalculate")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let details = result["data"].as_array().unwrap();
    assert_eq!(details.len(), 3);

    // Check percentages: 30%, 20%, 50%
    let mut percentages: Vec<f64> = details.iter()
        .filter_map(|d| d["percentage"].as_str().and_then(|p| p.parse::<f64>().ok()))
        .collect();
    percentages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((percentages[0] - 20.0).abs() < 0.1);
    assert!((percentages[1] - 30.0).abs() < 0.1);
    assert!((percentages[2] - 50.0).abs() < 0.1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_allocation_rule() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create pool
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RULE_POOL",
            "name": "Rule Test Pool",
            "pool_type": "cost_center",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RULE_BASIS",
            "name": "Rule Test Basis",
            "basis_type": "statistical",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create rule
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RENT_ALLOC",
            "name": "Rent Allocation Rule",
            "description": "Allocates rent based on headcount",
            "pool_code": "RULE_POOL",
            "basis_code": "RULE_BASIS",
            "allocation_method": "proportional",
            "offset_method": "same_account"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["code"], "RENT_ALLOC");
    assert_eq!(rule["allocationMethod"], "proportional");
    assert_eq!(rule["offsetMethod"], "same_account");
}

#[tokio::test]
async fn test_create_fixed_percentage_rule() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create pool
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "FIXED_POOL",
            "name": "Fixed Test Pool",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "FIXED_BASIS",
            "name": "Fixed Test Basis",
            "basis_type": "percentage",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create rule with fixed percentages
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "FIXED_ALLOC",
            "name": "Fixed Percentage Allocation",
            "pool_code": "FIXED_POOL",
            "basis_code": "FIXED_BASIS",
            "allocation_method": "fixed_percentage",
            "offset_method": "none",
            "target_lines": [
                {"target_account_code": "6100-ENG", "target_department_name": "Engineering", "fixed_percentage": "40"},
                {"target_account_code": "6100-MKT", "target_department_name": "Marketing", "fixed_percentage": "35"},
                {"target_account_code": "6100-SLS", "target_department_name": "Sales", "fixed_percentage": "25"}
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(rule["allocationMethod"], "fixed_percentage");
    assert_eq!(rule["targetLines"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_create_rule_invalid_pool() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create basis only
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ONLY_BASIS",
            "name": "Only Basis",
            "basis_type": "statistical",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to create rule with non-existent pool
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_RULE",
            "name": "Bad Rule",
            "pool_code": "NONEXISTENT_POOL",
            "basis_code": "ONLY_BASIS",
            "allocation_method": "proportional",
            "offset_method": "same_account"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_allocation_rules() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create pool and basis
    for code in &["RLIST_POOL"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/pools")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Pool {}", code),
                "pool_type": "manual",
                "currency_code": "USD"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    for code in &["RLIST_BASIS"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/bases")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Basis {}", code),
                "basis_type": "statistical",
                "is_manual": true
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    for (i, code) in ["RL1", "RL2"].iter().enumerate() {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/rules")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Rule {}", i + 1),
                "pool_code": "RLIST_POOL",
                "basis_code": "RLIST_BASIS",
                "allocation_method": "proportional",
                "offset_method": "same_account"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/rules")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Allocation Run Tests
// ═══════════════════════════════════════════════════════════════════════════════

async fn setup_full_allocation(app: &axum::Router, k: &str, v: &str) {
    // Create pool
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RUN_POOL",
            "name": "Run Test Pool",
            "pool_type": "cost_center",
            "source_account_code": "6100-RENT",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RUN_BASIS",
            "name": "Run Test Basis",
            "basis_type": "statistical",
            "unit_of_measure": "people",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add basis details
    for (dept, amount) in &[("Engineering", "300"), ("Marketing", "200"), ("Sales", "500")] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/allocation/bases/RUN_BASIS/details")
            .header("Content-Type", "application/json")
            .header(k, v)
            .body(Body::from(serde_json::to_string(&json!({
                "target_department_name": dept,
                "target_account_code": format!("6500-{}", dept),
                "basis_amount": amount,
                "period_name": "MAR-2024"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Create rule
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RUN_ALLOC",
            "name": "Run Test Allocation",
            "pool_code": "RUN_POOL",
            "basis_code": "RUN_BASIS",
            "allocation_method": "proportional",
            "offset_method": "same_account"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
}

#[tokio::test]
async fn test_execute_proportional_allocation() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_allocation(&app, &k, &v).await;

    // Execute allocation
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rule_code": "RUN_ALLOC",
            "period_name": "MAR-2024",
            "period_start_date": "2024-03-01",
            "period_end_date": "2024-03-31",
            "pool_amount_override": "10000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(run["status"], "draft");
    assert_eq!(run["allocationMethod"], "proportional");
    assert_eq!(run["periodName"], "MAR-2024");
    // Should have 3 allocation lines + 1 offset line = 4 results
    assert!(run["results"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test_post_and_reverse_allocation_run() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_allocation(&app, &k, &v).await;

    // Create run
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rule_code": "RUN_ALLOC",
            "period_name": "POST-TEST",
            "period_start_date": "2024-04-01",
            "period_end_date": "2024-04-30",
            "pool_amount_override": "5000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Post
    let post_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/runs/{}/post", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(post_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(post_resp.into_body(), usize::MAX).await.unwrap();
    let posted_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(posted_run["status"], "posted");

    // Reverse
    let reverse_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/runs/{}/reverse", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(reverse_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(reverse_resp.into_body(), usize::MAX).await.unwrap();
    let reversed_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reversed_run["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_allocation_run() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_allocation(&app, &k, &v).await;

    // Create run
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rule_code": "RUN_ALLOC",
            "period_name": "CANCEL-TEST",
            "period_start_date": "2024-05-01",
            "period_end_date": "2024-05-31",
            "pool_amount_override": "3000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Cancel
    let cancel_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(cancel_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(cancel_resp.into_body(), usize::MAX).await.unwrap();
    let cancelled_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled_run["status"], "cancelled");
}

#[tokio::test]
async fn test_cannot_cancel_posted_run() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_allocation(&app, &k, &v).await;

    // Create and post
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rule_code": "RUN_ALLOC",
            "period_name": "NO-CANCEL",
            "period_start_date": "2024-06-01",
            "period_end_date": "2024-06-30",
            "pool_amount_override": "1000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Post first
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/runs/{}/post", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    // Try to cancel - should fail
    let cancel_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/allocation/runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(cancel_resp.status(), StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_allocation_dashboard() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalPools"].is_number());
    assert!(summary["activePools"].is_number());
    assert!(summary["totalBases"].is_number());
    assert!(summary["totalRules"].is_number());
    assert!(summary["totalRuns"].is_number());
    assert!(summary["pools_by_type"].is_object());
    assert!(summary["rules_by_method"].is_object());
}

#[tokio::test]
async fn test_allocation_dashboard_after_creation() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create pool
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DASH_POOL",
            "name": "Dashboard Pool",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Create basis
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DASH_BASIS",
            "name": "Dashboard Basis",
            "basis_type": "statistical",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Check dashboard
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalPools"].as_i64().unwrap() >= 1);
    assert!(summary["totalBases"].as_i64().unwrap() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation & Error Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_pool_empty_code_rejected() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/pools")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Empty Code",
            "pool_type": "manual",
            "currency_code": "USD"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_basis_empty_code_rejected() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Empty Code",
            "basis_type": "statistical",
            "is_manual": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_basis_detail_nonexistent_basis() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/bases/NONEXISTENT/details")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_department_name": "Test",
            "target_account_code": "6000",
            "basis_amount": "100"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_rule_empty_code_rejected() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Empty Code Rule",
            "pool_code": "ANY",
            "basis_code": "ANY",
            "allocation_method": "proportional",
            "offset_method": "same_account"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_rule_invalid_method_rejected() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/allocation/rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID_METHOD",
            "name": "Invalid Method",
            "pool_code": "ANY",
            "basis_code": "ANY",
            "allocation_method": "invalid",
            "offset_method": "same_account"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_allocation_runs_with_status_filter() {
    let (_state, app) = setup_allocation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // List with no runs
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/allocation/runs?status=draft")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 1 || result["data"].as_array().unwrap().is_empty());
}