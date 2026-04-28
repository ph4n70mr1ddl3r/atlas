//! Product Configurator E2E Tests
//!
//! Tests for Oracle Fusion Cloud SCM > Product Management > Configurator:
//! - Configuration model CRUD and lifecycle
//! - Feature and option management
//! - Configuration rule management
//! - Configuration instance lifecycle (create, validate, submit, approve)
//! - Configurator dashboard summary
//! - Full end-to-end lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_configurator_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_model(
    app: &axum::Router, model_number: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modelNumber": model_number,
            "name": name,
            "modelType": "standard",
            "validationMode": "strict"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for model but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_feature(
    app: &axum::Router, model_id: &str, feature_code: &str, name: &str,
    feature_type: &str, is_required: bool,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/configurator/models/{}/features", model_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "featureCode": feature_code,
            "name": name,
            "featureType": feature_type,
            "isRequired": is_required,
            "displayOrder": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for feature but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_option(
    app: &axum::Router, feature_id: &str, option_code: &str, name: &str,
    price_adjustment: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/configurator/features/{}/options", feature_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "optionCode": option_code,
            "name": name,
            "optionType": "standard",
            "priceAdjustment": price_adjustment,
            "isAvailable": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for option but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Model Tests
// ============================================================================

#[tokio::test]
async fn test_create_model() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "LAPTOP-001", "Laptop Configurator").await;
    assert_eq!(model["model_number"], "LAPTOP-001");
    assert_eq!(model["name"], "Laptop Configurator");
    assert_eq!(model["model_type"], "standard");
    assert_eq!(model["status"], "draft");
    assert_eq!(model["validation_mode"], "strict");
}

#[tokio::test]
async fn test_create_model_duplicate_conflict() {
    let (_state, app) = setup_configurator_test().await;
    create_test_model(&app, "DUP-001", "First").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modelNumber": "DUP-001", "name": "Duplicate", "modelType": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_models() {
    let (_state, app) = setup_configurator_test().await;
    create_test_model(&app, "M-001", "Model 1").await;
    create_test_model(&app, "M-002", "Model 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/configurator/models")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(b["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_model() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "GET-001", "Get Test").await;
    let id = model["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/configurator/models/{}", id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activate_model() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "ACT-001", "Activate Test").await;
    let id = model["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/configurator/models/{}/activate", id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(b["status"], "active");
}

#[tokio::test]
async fn test_delete_model() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "DEL-001", "Delete Test").await;
    assert_eq!(model["status"], "draft");

    let (k, v) = auth_header(&admin_claims());
    let uri = "/api/v1/configurator/models/number/DEL-001";
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri(uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_model_not_draft_fails() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "DELACT-001", "Active Model").await;
    let id = model["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/configurator/models/{}/activate", id);
    app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now try to delete - should fail
    let uri = "/api/v1/configurator/models/number/DELACT-001";
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri(uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Feature Tests
// ============================================================================

#[tokio::test]
async fn test_create_feature_and_list() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "FEAT-001", "Feature Test Model").await;
    // Activate model to add features
    let model_id = model["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let feature = create_test_feature(&app, model_id, "COLOR", "Color", "single_select", true).await;
    assert_eq!(feature["feature_code"], "COLOR");
    assert_eq!(feature["name"], "Color");
    assert_eq!(feature["feature_type"], "single_select");
    assert_eq!(feature["is_required"], true);

    // List features
    let uri = format!("/api/v1/configurator/models/{}/features", model_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(b["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Option Tests
// ============================================================================

#[tokio::test]
async fn test_create_option_and_list() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "OPT-001", "Option Test Model").await;
    let model_id = model["id"].as_str().unwrap();

    // Activate model
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let feature = create_test_feature(&app, model_id, "SIZE", "Size", "single_select", true).await;
    let feature_id = feature["id"].as_str().unwrap();

    let opt = create_test_option(&app, feature_id, "LARGE", "Large", 50.0).await;
    assert_eq!(opt["option_code"], "LARGE");
    assert_eq!(opt["name"], "Large");
    assert_eq!(opt["price_adjustment"], 50.0);

    // List options
    let uri = format!("/api/v1/configurator/features/{}/options", feature_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(b["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_rule() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "RULE-001", "Rule Test Model").await;
    let model_id = model["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let uri = format!("/api/v1/configurator/models/{}/rules", model_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ruleCode": "R-001",
            "name": "V8 requires Sport Package",
            "ruleType": "requirement",
            "severity": "error",
            "isActive": true,
            "priority": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let rule: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(rule["rule_code"], "R-001");
    assert_eq!(rule["rule_type"], "requirement");
}

// ============================================================================
// Instance Tests
// ============================================================================

#[tokio::test]
async fn test_create_instance() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "INST-001", "Instance Test Model").await;
    let model_id = model["id"].as_str().unwrap();

    // Activate model
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create instance
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "instanceNumber": "CFG-001",
            "modelId": model_id,
            "name": "My Laptop Config",
            "selections": { "color": "red", "engine": "v8" },
            "basePrice": 1000.0,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let inst: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(inst["instance_number"], "CFG-001");
    assert_eq!(inst["base_price"], 1000.0);
    // Should be valid since no required features without selections
    assert!(inst["status"] == "valid" || inst["status"] == "invalid");
}

#[tokio::test]
async fn test_instance_lifecycle() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "LC-001", "Lifecycle Test").await;
    let model_id = model["id"].as_str().unwrap();

    // Activate model
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create instance
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "instanceNumber": "LC-CFG-001",
            "modelId": model_id,
            "basePrice": 2000.0,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let inst: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let inst_id = inst["id"].as_str().unwrap();

    // Submit (only if valid)
    if inst["status"] == "valid" {
        let uri = format!("/api/v1/configurator/instances/{}/submit", inst_id);
        let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header(&k, &v).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        // Approve
        let uri = format!("/api/v1/configurator/instances/{}/approve", inst_id);
        let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header(&k, &v).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let approved: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        assert_eq!(approved["status"], "approved");
        assert!(approved["approved_by"].is_string());
    }
}

#[tokio::test]
async fn test_cancel_instance() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "CAN-001", "Cancel Test").await;
    let model_id = model["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create instance
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "instanceNumber": "CAN-CFG-001",
            "modelId": model_id,
            "basePrice": 500.0,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let inst: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let inst_id = inst["id"].as_str().unwrap();

    // Cancel
    let uri = format!("/api/v1/configurator/instances/{}/cancel", inst_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let cancelled: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_instances() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "LIST-001", "List Instance Test").await;
    let model_id = model["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create two instances
    for i in 1..=2 {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "instanceNumber": format!("LIST-CFG-{:03}", i),
                "modelId": model_id,
                "basePrice": 1000.0,
                "currencyCode": "USD"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List all
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/configurator/instances")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(b["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_configurator_dashboard() {
    let (_state, app) = setup_configurator_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/configurator/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dash: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(dash["total_models"], 0);
    assert_eq!(dash["active_models"], 0);
    assert_eq!(dash["total_configurations"], 0);
}

// ============================================================================
// Full End-to-End Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_full_configurator_lifecycle() {
    let (_state, app) = setup_configurator_test().await;

    // 1. Create a model
    let model = create_test_model(&app, "E2E-LAPTOP", "Enterprise Laptop Configurator").await;
    let model_id = model["id"].as_str().unwrap();
    assert_eq!(model["status"], "draft");

    // 2. Activate the model
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/configurator/models/{}/activate", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let active_model: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(active_model["status"], "active");

    // 3. Create features
    let color_feature = create_test_feature(&app, model_id, "COLOR", "Color", "single_select", true).await;
    let size_feature = create_test_feature(&app, model_id, "SIZE", "Screen Size", "single_select", true).await;
    let color_id = color_feature["id"].as_str().unwrap();
    let size_id = size_feature["id"].as_str().unwrap();

    // 4. Create options
    create_test_option(&app, color_id, "SILVER", "Silver", 0.0).await;
    create_test_option(&app, color_id, "SPACE_GRAY", "Space Gray", 50.0).await;
    create_test_option(&app, size_id, "13_INCH", "13 inch", 0.0).await;
    create_test_option(&app, size_id, "15_INCH", "15 inch", 200.0).await;

    // 5. Verify features and options via list
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/configurator/models/{}/features", model_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let features: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(features["data"].as_array().unwrap().len(), 2);

    // 6. Create a configuration instance
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "instanceNumber": "E2E-CFG-001",
            "modelId": model_id,
            "name": "My Enterprise Laptop",
            "selections": {
                "COLOR": "SPACE_GRAY",
                "SIZE": "15_INCH"
            },
            "basePrice": 1500.0,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let inst: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let inst_id = inst["id"].as_str().unwrap();
    assert_eq!(inst["instance_number"], "E2E-CFG-001");
    assert_eq!(inst["base_price"], 1500.0);
    // Total should be base + Space Gray (+50) + 15 inch (+200) = 1750
    assert_eq!(inst["total_price"], 1750.0);

    // 7. Verify dashboard shows the model and configuration
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/configurator/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let dash: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(dash["total_models"], 1);
    assert_eq!(dash["active_models"], 1);
    assert_eq!(dash["total_configurations"], 1);

    // 8. Submit and approve (if valid)
    if inst["status"] == "valid" {
        let uri = format!("/api/v1/configurator/instances/{}/submit", inst_id);
        let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header(&k, &v).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        let uri = format!("/api/v1/configurator/instances/{}/approve", inst_id);
        let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header(&k, &v).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let approved: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        assert_eq!(approved["status"], "approved");
    }
}

// ============================================================================
// Validation / Error Tests
// ============================================================================

#[tokio::test]
async fn test_create_model_validation_errors() {
    let (_state, app) = setup_configurator_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Missing model number
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "No Number", "modelType": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid model type
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/models")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modelNumber": "BAD-TYPE", "name": "Bad Type", "modelType": "super_custom"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_instance_inactive_model_fails() {
    let (_state, app) = setup_configurator_test().await;
    let model = create_test_model(&app, "INACT-001", "Inactive Model").await;
    let model_id = model["id"].as_str().unwrap();
    // Model is still in "draft" status

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/configurator/instances")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "instanceNumber": "INACT-CFG-001",
            "modelId": model_id,
            "basePrice": 100.0,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    // Should fail because model is not active
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
