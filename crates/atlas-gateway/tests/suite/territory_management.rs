//! Territory Management E2E Tests
//!
//! Tests for Oracle Fusion CX Sales > Territory Management:
//! - Territory CRUD (create, get, list, update, delete)
//! - Territory lifecycle (activate, deactivate)
//! - Territory hierarchy (parent/child)
//! - Member management (add, list, remove)
//! - Routing rules (add, list, remove)
//! - Entity routing (route leads/opportunities to matching territories)
//! - Territory quotas (set, list, update attainment, delete)
//! - Dashboard
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

/// Helper: create a territory
async fn create_territory(
    app: &axum::Router,
    code: &str,
    name: &str,
    territory_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "territoryType": territory_type,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating territory");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Helper: create a child territory
async fn create_child_territory(
    app: &axum::Router,
    code: &str,
    name: &str,
    territory_type: &str,
    parent_id: &Uuid,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "territoryType": territory_type,
            "parentId": parent_id.to_string(),
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating child territory");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Territory CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_territory() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "WEST", "West Region", "geography").await;
    assert_eq!(t["code"], "WEST");
    assert_eq!(t["name"], "West Region");
    assert_eq!(t["territoryType"], "geography");
    assert_eq!(t["isActive"], true);
    assert!(t["id"].is_string());
}

#[tokio::test]
async fn test_create_territory_all_types() {
    let (_state, app) = setup_test().await;
    for tt in &["geography", "product", "industry", "customer", "hybrid"] {
        let t = create_territory(&app, &format!("T-{}", tt), &format!("{} Territory", tt), tt).await;
        assert_eq!(t["territoryType"], *tt);
    }
}

#[tokio::test]
async fn test_create_territory_with_dates() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DATED",
            "name": "Dated Territory",
            "territoryType": "geography",
            "effectiveFrom": "2026-01-01",
            "effectiveTo": "2026-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let t: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(t["effectiveFrom"], "2026-01-01");
    assert_eq!(t["effectiveTo"], "2026-12-31");
}

#[tokio::test]
async fn test_get_territory() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "GET-T", "Get Test", "geography").await;
    let id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "GET-T");
    assert_eq!(fetched["name"], "Get Test");
}

#[tokio::test]
async fn test_list_territories() {
    let (_state, app) = setup_test().await;
    create_territory(&app, "LIST-A", "Territory A", "geography").await;
    create_territory(&app, "LIST-B", "Territory B", "product").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/territories")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = list.as_array().unwrap();
    assert!(arr.len() >= 2, "Expected at least 2 territories");
}

#[tokio::test]
async fn test_list_territories_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_territory(&app, "TYPE-GEO", "Geo Territory", "geography").await;
    create_territory(&app, "TYPE-PROD", "Prod Territory", "product").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/territories?territory_type=geography")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for t in list.as_array().unwrap() {
        assert_eq!(t["territoryType"], "geography");
    }
}

#[tokio::test]
async fn test_update_territory() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "UPD-T", "Original", "geography").await;
    let id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/territories/{}", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Updated Name",
            "description": "Updated description"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["name"], "Updated Name");
    assert_eq!(updated["description"], "Updated description");
    assert_eq!(updated["code"], "UPD-T"); // unchanged
}

#[tokio::test]
async fn test_delete_territory() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "DEL-T", "Delete Me", "geography").await;
    let id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/territories/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_empty_code_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Bad Code",
            "territoryType": "geography"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_type_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-TYPE",
            "name": "Bad Type",
            "territoryType": "unknown"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_duplicate_code_rejected() {
    let (_state, app) = setup_test().await;
    create_territory(&app, "DUP", "First", "geography").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP",
            "name": "Second",
            "territoryType": "geography"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_invalid_dates_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/territories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-DATES",
            "name": "Bad Dates",
            "territoryType": "geography",
            "effectiveFrom": "2027-01-01",
            "effectiveTo": "2026-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_activate_deactivate_territory() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "LC-T", "Lifecycle", "geography").await;
    let id = t["id"].as_str().unwrap();

    // Deactivate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/deactivate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let deactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(deactivated["isActive"], false);

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["isActive"], true);
}

// ============================================================================
// Hierarchy Tests
// ============================================================================

#[tokio::test]
async fn test_create_child_territory() {
    let (_state, app) = setup_test().await;
    let parent = create_territory(&app, "PARENT", "Parent Region", "geography").await;
    let parent_id: Uuid = parent["id"].as_str().unwrap().parse().unwrap();

    let child = create_child_territory(&app, "CHILD", "Child Area", "geography", &parent_id).await;
    assert_eq!(child["parentId"], parent_id.to_string());
}

#[tokio::test]
async fn test_list_child_territories() {
    let (_state, app) = setup_test().await;
    let parent = create_territory(&app, "ROOT", "Root", "geography").await;
    let parent_id: Uuid = parent["id"].as_str().unwrap().parse().unwrap();
    create_child_territory(&app, "CH-1", "Child 1", "geography", &parent_id).await;
    create_child_territory(&app, "CH-2", "Child 2", "geography", &parent_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories?parent_id={}", parent_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let children: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(children.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_cannot_delete_parent_with_children() {
    let (_state, app) = setup_test().await;
    let parent = create_territory(&app, "PARENT-DEL", "Parent", "geography").await;
    let parent_id: Uuid = parent["id"].as_str().unwrap().parse().unwrap();
    create_child_territory(&app, "CHILD-DEL", "Child", "geography", &parent_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/territories/{}", parent_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Member Tests
// ============================================================================

#[tokio::test]
async fn test_add_member() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "MEM-T", "Member Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();
    let user_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/members", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "userId": user_id.to_string(),
            "userName": "John Smith",
            "role": "owner"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let member: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(member["userId"], user_id.to_string());
    assert_eq!(member["userName"], "John Smith");
    assert_eq!(member["role"], "owner");
}

#[tokio::test]
async fn test_list_members() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "LM-T", "List Members", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add two members
    for (name, role) in &[("Alice", "owner"), ("Bob", "member")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/territories/{}/members", territory_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "userId": Uuid::new_v4().to_string(),
                "userName": name,
                "role": role
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List all
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/members", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let members: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(members.as_array().unwrap().len(), 2);

    // List by role
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/members?role=owner", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let owners: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(owners.as_array().unwrap().len(), 1);
    assert_eq!(owners[0]["userName"], "Alice");
}

#[tokio::test]
async fn test_remove_member() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "RM-T", "Remove Member", "geography").await;
    let territory_id = t["id"].as_str().unwrap();
    let user_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/members", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "userId": user_id.to_string(),
            "userName": "Remove Me",
            "role": "member"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let member: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let member_id = member["id"].as_str().unwrap();

    // Remove
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/territories/members/{}", member_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify member is gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/members", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let members: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(members.as_array().unwrap().len(), 0);
}

// ============================================================================
// Routing Rule Tests
// ============================================================================

#[tokio::test]
async fn test_add_routing_rule() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "RULE-T", "Rule Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "fieldName": "state",
            "matchOperator": "equals",
            "matchValue": "California",
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["entityType"], "lead");
    assert_eq!(rule["fieldName"], "state");
    assert_eq!(rule["matchOperator"], "equals");
    assert_eq!(rule["matchValue"], "California");
    assert_eq!(rule["priority"], 1);
}

#[tokio::test]
async fn test_list_routing_rules() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "LR-T", "List Rules", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add two rules
    for (field, op, val) in &[("state", "equals", "CA"), ("industry", "equals", "Tech")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/territories/{}/rules", territory_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "entityType": "lead",
                "fieldName": field,
                "matchOperator": op,
                "matchValue": val,
                "priority": 1
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List all rules
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rules: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rules.as_array().unwrap().len(), 2);

    // Filter by entity_type
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/rules?entity_type=opportunity", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let filtered: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(filtered.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_remove_routing_rule() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "RR-T", "Remove Rule", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "fieldName": "country",
            "matchOperator": "equals",
            "matchValue": "US",
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let rule_id = rule["id"].as_str().unwrap();

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/territories/rules/{}", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Entity Routing Tests
// ============================================================================

#[tokio::test]
async fn test_route_entity_match() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "ROUTE-MATCH", "CA Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add rule: state = California
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "fieldName": "state",
            "matchOperator": "equals",
            "matchValue": "California",
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Route a lead with state=California
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/territories/route")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "entityData": { "state": "California", "company": "Acme Corp" }
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["matched"], true);
    assert_eq!(result["bestMatch"]["territoryCode"].as_str().unwrap_or(""), "ROUTE-MATCH");
}

#[tokio::test]
async fn test_route_entity_no_match() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "ROUTE-NOMATCH", "TX Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add rule: state = Texas
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "fieldName": "state",
            "matchOperator": "equals",
            "matchValue": "Texas",
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Route a lead with state=New York - no match
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/territories/route")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "lead",
            "entityData": { "state": "New York" }
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["matched"], false);
}

#[tokio::test]
async fn test_route_entity_contains_operator() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "ROUTE-CONTAINS", "West Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/rules", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "opportunity",
            "fieldName": "city",
            "matchOperator": "contains",
            "matchValue": "San",
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/territories/route")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entityType": "opportunity",
            "entityData": { "city": "San Francisco" }
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["matched"], true);
}

// ============================================================================
// Quota Tests
// ============================================================================

#[tokio::test]
async fn test_set_quota() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "QUOTA-T", "Quota Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/quotas", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Q1-2026",
            "periodStart": "2026-01-01",
            "periodEnd": "2026-03-31",
            "revenueQuota": "500000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let quota: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(quota["periodName"].as_str().unwrap_or(""), "Q1-2026");
    assert_eq!(quota["revenueQuota"], "500000.00");
    assert_eq!(quota["actualRevenue"], "0.00");
    assert_eq!(quota["currencyCode"], "USD");
}

#[tokio::test]
async fn test_update_attainment() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "ATT-T", "Attainment Territory", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Set quota
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/quotas", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Q2-2026",
            "periodStart": "2026-04-01",
            "periodEnd": "2026-06-30",
            "revenueQuota": "100000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let quota: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let quota_id = quota["id"].as_str().unwrap();

    // Update attainment
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/territories/quotas/{}/attainment", quota_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualRevenue": "75000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["actualRevenue"], "75000.00");
    // attainment percent should be 75%
    let pct: f64 = updated["attainmentPercent"].as_str().unwrap().parse().unwrap();
    assert!((pct - 75.0).abs() < 1.0);
}

#[tokio::test]
async fn test_list_quotas() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "LQ-T", "List Quotas", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create two quotas
    for (name, start, end, amount) in &[("Q1", "2026-01-01", "2026-03-31", "100000"), ("Q2", "2026-04-01", "2026-06-30", "200000")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/territories/{}/quotas", territory_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "periodName": name,
                "periodStart": start,
                "periodEnd": end,
                "revenueQuota": amount,
                "currencyCode": "USD"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}/quotas", territory_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let quotas: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(quotas.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_quota() {
    let (_state, app) = setup_test().await;
    let t = create_territory(&app, "DQ-T", "Delete Quota", "geography").await;
    let territory_id = t["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/territories/{}/quotas", territory_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "periodName": "Q3-2026",
            "periodStart": "2026-07-01",
            "periodEnd": "2026-09-30",
            "revenueQuota": "300000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let quota: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let quota_id = quota["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/territories/quotas/{}", quota_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_territory_dashboard() {
    let (_state, app) = setup_test().await;
    create_territory(&app, "DASH-1", "Dashboard Geo", "geography").await;
    create_territory(&app, "DASH-2", "Dashboard Prod", "product").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/territories/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalTerritories"].as_i64().unwrap() >= 2);
    assert!(dashboard["activeTerritories"].as_i64().unwrap() >= 2);
    assert!(dashboard["topLevelTerrritories"].as_i64().unwrap_or(0) >= 0);
    assert!(dashboard["byType"].is_array());
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_territory() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/territories/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
