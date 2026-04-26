//! KPI & Embedded Analytics E2E Tests
//!
//! Tests for Oracle Fusion OTBI-inspired analytics:
//! - KPI definition CRUD
//! - KPI data point recording and status computation
//! - Dashboard management
//! - Dashboard widget management
//! - KPI analytics dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_kpi_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_kpi(
    app: &axum::Router, code: &str, name: &str, category: &str,
    direction: &str, target: &str, warning: Option<&str>, critical: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "code": code,
        "name": name,
        "category": category,
        "direction": direction,
        "targetValue": target,
        "unitOfMeasure": "number",
        "evaluationFrequency": "daily"
    });
    if let Some(w) = warning {
        body["warningThreshold"] = json!(w);
    }
    if let Some(c) = critical {
        body["criticalThreshold"] = json!(c);
    }
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for KPI but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_dashboard(
    app: &axum::Router, code: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/dashboards")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "isShared": true,
            "layoutConfig": {"columns": 3}
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for dashboard but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// KPI Definition Tests
// ============================================================================

#[tokio::test]
async fn test_create_kpi() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "REV_GROWTH", "Revenue Growth", "financial", "higher_is_better", "10.0", Some("5.0"), Some("2.0")).await;
    assert_eq!(kpi["code"], "REV_GROWTH");
    assert_eq!(kpi["name"], "Revenue Growth");
    assert_eq!(kpi["category"], "financial");
    assert_eq!(kpi["direction"], "higher_is_better");
    assert_eq!(kpi["targetValue"], "10.0");
    assert_eq!(kpi["isActive"], true);
}

#[tokio::test]
async fn test_create_kpi_duplicate_code_conflict() {
    let (_state, app) = setup_kpi_test().await;
    create_test_kpi(&app, "DUP_TEST", "First KPI", "general", "higher_is_better", "100", None, None).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP_TEST", "name": "Duplicate", "category": "general",
            "direction": "higher_is_better", "targetValue": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_kpi_invalid_direction() {
    let (_state, app) = setup_kpi_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_DIR", "name": "Bad", "category": "general",
            "direction": "sideways", "targetValue": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_kpi_invalid_target_value() {
    let (_state, app) = setup_kpi_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD_VAL", "name": "Bad", "category": "general",
            "direction": "higher_is_better", "targetValue": "not_a_number"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_kpi() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "GET_TEST", "Get Test", "sales", "higher_is_better", "100", None, None).await;
    let id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/definitions/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["code"], "GET_TEST");
    assert_eq!(fetched["name"], "Get Test");
}

#[tokio::test]
async fn test_list_kpis() {
    let (_state, app) = setup_kpi_test().await;
    create_test_kpi(&app, "LIST_1", "KPI One", "financial", "higher_is_better", "10", None, None).await;
    create_test_kpi(&app, "LIST_2", "KPI Two", "sales", "lower_is_better", "5", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/kpi/definitions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_kpis_by_category() {
    let (_state, app) = setup_kpi_test().await;
    create_test_kpi(&app, "CAT_FIN", "Finance", "financial", "higher_is_better", "10", None, None).await;
    create_test_kpi(&app, "CAT_SAL", "Sales", "sales", "higher_is_better", "5", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/kpi/definitions?category=financial")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let kpis = list["data"].as_array().unwrap();
    assert!(kpis.iter().all(|k| k["category"] == "financial"));
}

#[tokio::test]
async fn test_delete_kpi() {
    let (_state, app) = setup_kpi_test().await;
    create_test_kpi(&app, "DEL_TEST", "Delete Me", "general", "higher_is_better", "10", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/kpi/definitions/code/DEL_TEST")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// KPI Data Point Tests
// ============================================================================

#[tokio::test]
async fn test_record_data_point_on_track() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "ON_TRACK", "Revenue", "financial", "higher_is_better", "100", Some("10"), Some("20")).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    // Record value = 105 (> target - 10% warning threshold = 90), should be on_track
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "105", "notes": "Good progress"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dp["value"], "105");
    assert_eq!(dp["status"], "on_track");
}

#[tokio::test]
async fn test_record_data_point_warning_status() {
    let (_state, app) = setup_kpi_test().await;
    // target=100, direction=higher_is_better, warning=10%, critical=20%
    // value=85 => deviation = (85-100)/100 = -0.15 => within critical threshold? No (15% > 10% warning, < 20% critical) => warning
    let kpi = create_test_kpi(&app, "WARN_KPI", "Warning KPI", "financial", "higher_is_better", "100", Some("10"), Some("20")).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "85"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dp["status"], "warning");
}

#[tokio::test]
async fn test_record_data_point_critical_status() {
    let (_state, app) = setup_kpi_test().await;
    // target=100, direction=higher_is_better, warning=10%, critical=20%
    // value=70 => deviation = (70-100)/100 = -0.30 => critical (30% > 20%)
    let kpi = create_test_kpi(&app, "CRIT_KPI", "Critical KPI", "financial", "higher_is_better", "100", Some("10"), Some("20")).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "70"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dp["status"], "critical");
}

#[tokio::test]
async fn test_record_data_point_lower_is_better() {
    let (_state, app) = setup_kpi_test().await;
    // target=5 (defect rate), direction=lower_is_better, warning=20%, critical=50%
    // value=3 => deviation = (5-3)/5 = 0.4 => on_track (positive deviation = good for lower_is_better)
    let kpi = create_test_kpi(&app, "DEF_RATE", "Defect Rate", "quality", "lower_is_better", "5", Some("20"), Some("50")).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "3"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dp["status"], "on_track");
}

#[tokio::test]
async fn test_record_data_point_invalid_value() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "INV_VAL", "Invalid", "general", "higher_is_better", "100", None, None).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "not_a_number"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_latest_data_point() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "LATEST_DP", "Latest", "general", "higher_is_better", "100", None, None).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    // Record two data points
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"value": "50"})).unwrap())).unwrap()
    ).await.unwrap();

    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"value": "75"})).unwrap())).unwrap()
    ).await.unwrap();

    // Get latest
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/definitions/{}/data-points/latest", kpi_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dp["value"], "75");
}

#[tokio::test]
async fn test_list_data_points() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "LIST_DP", "List DP", "general", "higher_is_better", "100", None, None).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    // Record 3 data points
    let (k, v) = auth_header(&admin_claims());
    for val in ["10", "20", "30"] {
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"value": val})).unwrap())).unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/definitions/{}/data-points?limit=10", kpi_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_delete_data_point() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "DEL_DP", "Delete DP", "general", "higher_is_better", "100", None, None).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"value": "99"})).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let dp_id = dp["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/kpi/data-points/{}", dp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_create_dashboard() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "EXEC_DASH", "Executive Dashboard").await;
    assert_eq!(db["code"], "EXEC_DASH");
    assert_eq!(db["name"], "Executive Dashboard");
    assert_eq!(db["isShared"], true);
}

#[tokio::test]
async fn test_create_dashboard_duplicate_code() {
    let (_state, app) = setup_kpi_test().await;
    create_test_dashboard(&app, "DUP_DASH", "First").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/kpi/dashboards")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP_DASH", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "GET_DASH", "Get Dashboard").await;
    let id = db["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/dashboards/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["code"], "GET_DASH");
}

#[tokio::test]
async fn test_list_dashboards() {
    let (_state, app) = setup_kpi_test().await;
    create_test_dashboard(&app, "LIST_D1", "Dashboard 1").await;
    create_test_dashboard(&app, "LIST_D2", "Dashboard 2").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/kpi/dashboards")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_dashboard() {
    let (_state, app) = setup_kpi_test().await;
    create_test_dashboard(&app, "DEL_DASH", "Delete Me").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/kpi/dashboards/code/DEL_DASH")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Widget Tests
// ============================================================================

#[tokio::test]
async fn test_add_widget() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "WIDGET_DASH", "Widget Dashboard").await;
    let db_id = db["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "widgetType": "kpi_card",
            "title": "Revenue KPI",
            "positionRow": 0,
            "positionCol": 0,
            "width": 2,
            "height": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let widget: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(widget["widgetType"], "kpi_card");
    assert_eq!(widget["title"], "Revenue KPI");
    assert_eq!(widget["width"], 2);
}

#[tokio::test]
async fn test_add_widget_with_kpi_link() {
    let (_state, app) = setup_kpi_test().await;
    let kpi = create_test_kpi(&app, "LINK_KPI", "Linked KPI", "financial", "higher_is_better", "100", None, None).await;
    let db = create_test_dashboard(&app, "LINK_DASH", "Linked Dashboard").await;
    let db_id = db["id"].as_str().unwrap();
    let kpi_id = kpi["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "kpiId": kpi_id,
            "widgetType": "gauge",
            "title": "Revenue Gauge",
            "positionRow": 0,
            "positionCol": 0,
            "width": 1,
            "height": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let widget: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(widget["kpiId"], kpi_id);
}

#[tokio::test]
async fn test_add_widget_invalid_type() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "INVTYPE_DASH", "Invalid Type").await;
    let db_id = db["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "widgetType": "hologram",
            "title": "Bad Widget"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_widgets() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "WLIST_DASH", "Widget List").await;
    let db_id = db["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add two widgets
    for (i, (wt, title)) in [("kpi_card", "Card 1"), ("trend", "Trend 1")].iter().enumerate() {
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "widgetType": wt, "title": title,
                "positionRow": i as i32, "positionCol": 0
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_widget() {
    let (_state, app) = setup_kpi_test().await;
    let db = create_test_dashboard(&app, "WDEL_DASH", "Widget Delete").await;
    let db_id = db["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "widgetType": "kpi_card", "title": "To Delete"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let widget: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let w_id = widget["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/kpi/widgets/{}", w_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Summary Test
// ============================================================================

#[tokio::test]
async fn test_kpi_dashboard_summary() {
    let (_state, app) = setup_kpi_test().await;

    // Create KPIs and record data
    let kpi1 = create_test_kpi(&app, "SUM_1", "On Track KPI", "financial", "higher_is_better", "100", None, None).await;
    let _kpi2 = create_test_kpi(&app, "SUM_2", "Another KPI", "sales", "higher_is_better", "50", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi1["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"value": "110"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create a dashboard
    create_test_dashboard(&app, "SUM_DASH", "Summary Dashboard").await;

    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/kpi/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalKpis"].as_i64().unwrap() >= 2);
    assert!(summary["totalDashboards"].as_i64().unwrap() >= 1);
    assert!(summary["onTrack"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_kpi_full_lifecycle() {
    let (_state, app) = setup_kpi_test().await;

    // 1. Create KPI
    let kpi = create_test_kpi(&app, "LIFE", "Lifecycle KPI", "operations", "higher_is_better", "100", Some("10"), Some("25")).await;
    let kpi_id = kpi["id"].as_str().unwrap();

    // 2. Create Dashboard
    let db = create_test_dashboard(&app, "LIFE_DASH", "Lifecycle Dashboard").await;
    let db_id = db["id"].as_str().unwrap();

    // 3. Add Widget linking KPI to Dashboard
    let (k, v) = auth_header(&admin_claims());
    let widget_resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "kpiId": kpi_id,
            "widgetType": "chart",
            "title": "Lifecycle Chart",
            "positionRow": 0,
            "positionCol": 0,
            "width": 2,
            "height": 2
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(widget_resp.status(), StatusCode::CREATED);

    // 4. Record multiple data points over time
    for val in ["80", "90", "95", "105", "110"] {
        let resp = app.clone().oneshot(Request::builder().method("POST")
            .uri(format!("/api/v1/kpi/definitions/{}/data-points", kpi_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"value": val})).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // 5. Verify latest data point
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/definitions/{}/data-points/latest", kpi_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let latest: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(latest["value"], "110");
    assert_eq!(latest["status"], "on_track");

    // 6. Verify dashboard summary
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/kpi/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalKpis"].as_i64().unwrap() >= 1);
    assert!(summary["totalDashboards"].as_i64().unwrap() >= 1);

    // 7. Delete widget, dashboard, and KPI
    // List widgets to get ID
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/kpi/dashboards/{}/widgets", db_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let widgets: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let widget_id = widgets["data"][0]["id"].as_str().unwrap();

    // Delete widget
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/kpi/widgets/{}", widget_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Delete dashboard
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/kpi/dashboards/code/LIFE_DASH")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Delete KPI
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/kpi/definitions/code/LIFE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
