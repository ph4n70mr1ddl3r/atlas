//! Automatic Offsets (Intercompany Balancing) E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP General Ledger > Automatic Offsets:
//! - Template CRUD (create, get, list, activate/deactivate, delete)
//! - Template line management
//! - Offset generation from multi-entity journal lines
//! - Full lifecycle (generate → post → reverse)
//! - Cancel workflow
//! - Dashboard
//! - Validation edge cases
//! - Activity audit trail

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
    // Clean auto offset test data
    sqlx::query("DELETE FROM _atlas.auto_offset_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.auto_offset_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.auto_offset_generations").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.auto_offset_template_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.auto_offset_templates").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/140_automatic_offsets.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run auto offset migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_template(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": code,
        "templateName": name,
        "description": "Test offset template",
        "balancingSegment": "entity",
        "generationMethod": "single_entry",
        "defaultOffsetAccount": "1990",
        "defaultOffsetAccountDescription": "Intercompany Due-To/Due-From",
        "enableIntraEntity": false,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE TEMPLATE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create template: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_template_line(
    app: &axum::Router,
    template_id: Uuid,
    segment_value: &str,
    due_to_acct: &str,
    due_from_acct: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "balancingSegmentValue": segment_value,
        "dueToAccount": due_to_acct,
        "dueToAccountDescription": format!("Due-To {}", segment_value),
        "dueFromAccount": due_from_acct,
        "dueFromAccountDescription": format!("Due-From {}", segment_value),
        "priority": 100,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/templates/{}/lines", template_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD LINE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add template line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn generate_offsets(
    app: &axum::Router,
    template_id: Uuid,
    journal_lines: Vec<serde_json::Value>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateId": template_id,
        "sourceType": "journal_entry",
        "sourceNumber": "JE-TEST-001",
        "fiscalYear": 2024,
        "periodName": "JUL-24",
        "generationDate": "2024-07-15",
        "currencyCode": "USD",
        "journalLines": journal_lines,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/generate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("GENERATE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to generate offsets: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Template CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-STD", "Standard Offsets").await;

    assert_eq!(tmpl["templateCode"], "OFS-STD");
    assert_eq!(tmpl["templateName"], "Standard Offsets");
    assert_eq!(tmpl["balancingSegment"], "entity");
    assert_eq!(tmpl["generationMethod"], "single_entry");
    assert_eq!(tmpl["defaultOffsetAccount"], "1990");
    assert!(tmpl["isActive"].as_bool().unwrap());
}

#[tokio::test]
async fn test_create_template_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_template(&app, "OFS-DUP", "First").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "OFS-DUP",
        "templateName": "Second",
        "balancingSegment": "entity",
        "generationMethod": "single_entry",
        "defaultOffsetAccount": "1990",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_test().await;
    create_template(&app, "OFS-A", "Template A").await;
    create_template(&app, "OFS-B", "Template B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/auto-offsets/templates")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-GET", "Get Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/auto-offsets/templates/{}", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["templateCode"], "OFS-GET");
}

#[tokio::test]
async fn test_activate_deactivate_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-AD", "Activate Deactivate").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/templates/{}/deactivate", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/templates/{}/activate", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-DEL", "Delete Me").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/auto-offsets/templates/{}", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/auto-offsets/templates/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Template Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_template_lines() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-LINE", "Lines Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-01", "1910", "2910").await;
    add_template_line(&app, id, "ENT-02", "1920", "2920").await;

    // List lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/auto-offsets/templates/{}/lines", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_template_line() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-DL", "Delete Line").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let line = add_template_line(&app, id, "ENT-03", "1930", "2930").await;
    let line_id: Uuid = line["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/auto-offsets/template-lines/{}", line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Offset Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_offsets_two_entities() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-GEN1", "Generate Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-01", "1910", "2910").await;
    add_template_line(&app, id, "ENT-02", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-01", "accountCode": "1000", "debit": 5000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-02", "accountCode": "2000", "debit": 0.0, "credit": 5000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    assert!(gen["generationNumber"].as_str().unwrap().starts_with("AO-"));
    assert_eq!(gen["status"], "generated");
    assert_eq!(gen["sourceType"], "journal_entry");
    assert_eq!(gen["fiscalYear"], 2024);
    assert_eq!(gen["balancingSegmentsAffected"], 2);
}

#[tokio::test]
async fn test_generate_offsets_three_entities() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-GEN3", "Three Entity Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-A", "1910", "2910").await;
    add_template_line(&app, id, "ENT-B", "1920", "2920").await;
    add_template_line(&app, id, "ENT-C", "1930", "2930").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-A", "accountCode": "1000", "debit": 8000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-B", "accountCode": "2000", "debit": 0.0, "credit": 3000.0}),
        json!({"balancingSegmentValue": "ENT-C", "accountCode": "3000", "debit": 0.0, "credit": 5000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    assert_eq!(gen["balancingSegmentsAffected"], 3);
    assert!(gen["totalOffsetLines"].as_i64().unwrap() >= 2);
}

#[tokio::test]
async fn test_generate_balanced_lines_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-BAL", "Balanced Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateId": id,
        "sourceType": "journal_entry",
        "fiscalYear": 2024,
        "periodName": "JUL-24",
        "generationDate": "2024-07-15",
        "currencyCode": "USD",
        "journalLines": [
            {"balancingSegmentValue": "ENT-01", "accountCode": "1000", "debit": 5000.0, "credit": 0.0},
            {"balancingSegmentValue": "ENT-01", "accountCode": "2000", "debit": 0.0, "credit": 5000.0},
        ],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/generate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_post_reverse() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-LC1", "Lifecycle Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-X", "1910", "2910").await;
    add_template_line(&app, id, "ENT-Y", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-X", "accountCode": "1000", "debit": 10000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-Y", "accountCode": "2000", "debit": 0.0, "credit": 10000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(gen["status"], "generated");

    let (k, v) = auth_header(&admin_claims());

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/post", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/reverse", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_generated() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-CNL", "Cancel Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-P", "1910", "2910").await;
    add_template_line(&app, id, "ENT-Q", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-P", "accountCode": "1000", "debit": 7000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-Q", "accountCode": "2000", "debit": 0.0, "credit": 7000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/cancel", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_cannot_reverse_generated_only() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-REV", "Reverse Validation").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-M", "1910", "2910").await;
    add_template_line(&app, id, "ENT-N", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-M", "accountCode": "1000", "debit": 3000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-N", "accountCode": "2000", "debit": 0.0, "credit": 3000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Try to reverse without posting first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/reverse", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_cancel_posted() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-CNP", "Cancel Posted Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ENT-U", "1910", "2910").await;
    add_template_line(&app, id, "ENT-V", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ENT-U", "accountCode": "1000", "debit": 4000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ENT-V", "accountCode": "2000", "debit": 0.0, "credit": 4000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Post first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/post", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel a posted generation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/cancel", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Offset Lines & Activities
// ============================================================================

#[tokio::test]
async fn test_list_offset_lines() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-LNS", "Lines List Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "SEG-A", "1910", "2910").await;
    add_template_line(&app, id, "SEG-B", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "SEG-A", "accountCode": "1000", "debit": 6000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "SEG-B", "accountCode": "2000", "debit": 0.0, "credit": 6000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/auto-offsets/generations/{}/lines", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let offset_lines = body["data"].as_array().unwrap();
    assert!(!offset_lines.is_empty(), "Should have offset lines");

    // Each line should have from/to segments and offset type
    for line in offset_lines {
        assert!(line["fromSegmentValue"].is_string());
        assert!(line["toSegmentValue"].is_string());
        assert!(line["offsetType"].is_string());
        assert!(line["amount"].as_f64().unwrap() > 0.0);
    }
}

#[tokio::test]
async fn test_list_activities() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-ACT", "Activity Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "ACT-1", "1910", "2910").await;
    add_template_line(&app, id, "ACT-2", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "ACT-1", "accountCode": "1000", "debit": 2000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "ACT-2", "accountCode": "2000", "debit": 0.0, "credit": 2000.0}),
    ];

    let gen = generate_offsets(&app, id, lines).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Post it
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/post", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check activities
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/auto-offsets/generations/{}/activities", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let activities = body["data"].as_array().unwrap();
    assert!(activities.len() >= 2, "Should have at least 2 activities (generated + posted)");

    // Check activity types
    let types: Vec<&str> = activities.iter()
        .map(|a| a["activityType"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"generated"));
    assert!(types.contains(&"posted"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_template(&app, "OFS-D1", "Dash 1").await;
    create_template(&app, "OFS-D2", "Dash 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/auto-offsets/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalTemplates").is_some());
    assert!(body.get("activeTemplates").is_some());
    assert!(body.get("totalGenerations").is_some());
    assert!(body.get("generatedCount").is_some());
    assert!(body.get("postedCount").is_some());
    assert!(body.get("reversedCount").is_some());
    assert!(body.get("totalOffsetLines").is_some());
    assert!(body.get("totalOffsetAmount").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_template_invalid_balancing_segment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "OFS-INV1",
        "templateName": "Invalid Segment",
        "balancingSegment": "invalid_segment",
        "generationMethod": "single_entry",
        "defaultOffsetAccount": "1990",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_generation_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "OFS-INV2",
        "templateName": "Invalid Method",
        "balancingSegment": "entity",
        "generationMethod": "auto_magic",
        "defaultOffsetAccount": "1990",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateCode": "",
        "templateName": "No Code",
        "balancingSegment": "entity",
        "generationMethod": "single_entry",
        "defaultOffsetAccount": "1990",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/templates")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_with_empty_lines_fails() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-EMPTY", "Empty Lines").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateId": id,
        "sourceType": "journal_entry",
        "fiscalYear": 2024,
        "periodName": "JUL-24",
        "generationDate": "2024-07-15",
        "currencyCode": "USD",
        "journalLines": [],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/generate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_with_nonexistent_template_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "templateId": Uuid::new_v4(),
        "sourceType": "journal_entry",
        "fiscalYear": 2024,
        "periodName": "JUL-24",
        "generationDate": "2024-07-15",
        "currencyCode": "USD",
        "journalLines": [
            {"balancingSegmentValue": "ENT-01", "accountCode": "1000", "debit": 5000.0, "credit": 0.0},
        ],
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/auto-offsets/generate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_generations_filter_by_status() {
    let (_state, app) = setup_test().await;
    let tmpl = create_template(&app, "OFS-FILT", "Filter Test").await;
    let id: Uuid = tmpl["id"].as_str().unwrap().parse().unwrap();

    add_template_line(&app, id, "F1", "1910", "2910").await;
    add_template_line(&app, id, "F2", "1920", "2920").await;

    let lines = vec![
        json!({"balancingSegmentValue": "F1", "accountCode": "1000", "debit": 1000.0, "credit": 0.0}),
        json!({"balancingSegmentValue": "F2", "accountCode": "2000", "debit": 0.0, "credit": 1000.0}),
    ];

    let gen = generate_offsets(&app, id, lines.clone()).await;
    let gen_id: Uuid = gen["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Post this one
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/auto-offsets/generations/{}/post", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create another one that stays in generated status
    generate_offsets(&app, id, lines).await;

    // Filter by generated
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/auto-offsets/generations?status=generated")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|g| g["status"] == "generated"));

    // Filter by posted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/auto-offsets/generations?status=posted")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|g| g["status"] == "posted"));
}
