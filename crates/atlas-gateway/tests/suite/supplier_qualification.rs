//! Supplier Qualification Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Supplier Qualification:
//! - Qualification area CRUD
//! - Question management
//! - Initiative lifecycle (create, activate, complete, cancel)
//! - Supplier invitation & qualification lifecycle
//! - Response submission & scoring
//! - Supplier qualification (qualify / disqualify)
//! - Certification management (create, revoke, renew)
//! - Dashboard summary
//! - Validation edge cases

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
    sqlx::query(include_str!("../../../../migrations/039_supplier_qualification.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_area(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "area_code": code,
        "name": name,
        "description": "Test qualification area",
        "area_type": "questionnaire",
        "scoring_model": "weighted",
        "passing_score": "70",
        "is_mandatory": true,
        "renewal_period_days": 365,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/areas")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create area");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_initiative(
    app: &axum::Router,
    area_id: Uuid,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "name": name,
        "description": "Test initiative",
        "area_id": area_id.to_string(),
        "qualification_purpose": "new_supplier",
        "deadline": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/initiatives")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create initiative");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_question(
    app: &axum::Router,
    area_id: Uuid,
    number: i32,
    text: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "question_number": number,
        "question_text": text,
        "response_type": "text",
        "is_required": true,
        "weight": "1",
        "max_score": "10",
        "display_order": number,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/areas/{}/questions", area_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create question");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn invite_test_supplier(
    app: &axum::Router,
    initiative_id: Uuid,
    supplier_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplier_id": Uuid::new_v4().to_string(),
        "supplier_name": supplier_name,
        "supplier_contact_name": "John Doe",
        "supplier_contact_email": "john@example.com",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/invitations", initiative_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to invite supplier");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Qualification Area Tests
// ============================================================================

#[tokio::test]
async fn test_create_qualification_area() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-001", "Quality Assessment").await;

    assert_eq!(area["area_code"], "QA-001");
    assert_eq!(area["name"], "Quality Assessment");
    assert_eq!(area["area_type"], "questionnaire");
    assert_eq!(area["scoring_model"], "weighted");
    assert_eq!(area["is_mandatory"], true);
    assert_eq!(area["is_active"], true);
}

#[tokio::test]
async fn test_get_qualification_area() {
    let (_state, app) = setup_test().await;
    create_test_area(&app, "QA-002", "Financial Assessment").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/supplier-qualification/areas/QA-002")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["area_code"], "QA-002");
    assert_eq!(body["name"], "Financial Assessment");
}

#[tokio::test]
async fn test_list_qualification_areas() {
    let (_state, app) = setup_test().await;
    create_test_area(&app, "QA-LIST1", "Area 1").await;
    create_test_area(&app, "QA-LIST2", "Area 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/supplier-qualification/areas")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_qualification_area() {
    let (_state, app) = setup_test().await;
    create_test_area(&app, "QA-DEL", "Deletable Area").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/supplier-qualification/areas/QA-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Question Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_list_questions() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-Q1", "Question Test Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    let q1 = create_test_question(&app, area_id, 1, "Describe your QA process").await;
    let q2 = create_test_question(&app, area_id, 2, "Do you have ISO 9001 certification?").await;

    assert_eq!(q1["question_number"], 1);
    assert_eq!(q1["question_text"], "Describe your QA process");
    assert_eq!(q2["question_number"], 2);

    // List questions
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/supplier-qualification/areas/{}/questions", area_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Initiative Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_initiative_create_activate_complete_lifecycle() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-INIT1", "Initiative Test Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    // Create initiative
    let initiative = create_test_initiative(&app, area_id, "Q1 2025 Supplier Qualification").await;
    let init_id: Uuid = initiative["id"].as_str().unwrap().parse().unwrap();

    assert_eq!(initiative["name"], "Q1 2025 Supplier Qualification");
    assert_eq!(initiative["status"], "draft");
    assert_eq!(initiative["qualification_purpose"], "new_supplier");

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/activate", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // Complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/complete", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "completed");
    assert!(body["completed_at"].is_string());
}

#[tokio::test]
async fn test_initiative_cancel() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-CANCEL", "Cancel Test Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    let initiative = create_test_initiative(&app, area_id, "To Be Cancelled").await;
    let init_id: Uuid = initiative["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/cancel", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_list_initiatives_with_filter() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-LIST-INIT", "List Initiative Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    create_test_initiative(&app, area_id, "Initiative A").await;
    create_test_initiative(&app, area_id, "Initiative B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/supplier-qualification/initiatives?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Full Qualification Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_full_supplier_qualification_workflow() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-FULL", "Full Workflow Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    // Create questions
    let q1 = create_test_question(&app, area_id, 1, "Describe your quality process").await;
    let q2 = create_test_question(&app, area_id, 2, "Years in business?").await;
    let q1_id: Uuid = q1["id"].as_str().unwrap().parse().unwrap();
    let q2_id: Uuid = q2["id"].as_str().unwrap().parse().unwrap();

    // Create and activate initiative
    let initiative = create_test_initiative(&app, area_id, "Full Workflow Initiative").await;
    let init_id: Uuid = initiative["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/activate", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Invite supplier
    let invitation = invite_test_supplier(&app, init_id, "Acme Corp").await;
    let inv_id: Uuid = invitation["id"].as_str().unwrap().parse().unwrap();

    assert_eq!(invitation["supplier_name"], "Acme Corp");
    assert_eq!(invitation["status"], "initiated");

    // Submit responses
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/responses", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "question_id": q1_id.to_string(),
            "response_text": "We have a comprehensive QA process with ISO 9001 certification"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/responses", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "question_id": q2_id.to_string(),
            "response_text": "15 years"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Submit invitation response
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/submit", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "pending_response");

    // Start evaluation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/evaluate", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "under_evaluation");

    // Qualify the supplier
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/qualify", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "evaluation_notes": "Supplier meets all qualification criteria"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "qualified");
}

#[tokio::test]
async fn test_disqualify_supplier() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-DISQ", "Disqualification Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    let initiative = create_test_initiative(&app, area_id, "Disqualification Test").await;
    let init_id: Uuid = initiative["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/activate", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let invitation = invite_test_supplier(&app, init_id, "Bad Supplier Inc").await;
    let inv_id: Uuid = invitation["id"].as_str().unwrap().parse().unwrap();

    // Submit → evaluate → disqualify
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/submit", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/evaluate", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/disqualify", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Failed to meet minimum quality standards"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "disqualified");
}

// ============================================================================
// Certification Tests
// ============================================================================

#[tokio::test]
async fn test_create_certification() {
    let (_state, app) = setup_test().await;
    let supplier_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplier_id": supplier_id.to_string(),
        "supplier_name": "Certified Supplier Co",
        "certification_type": "ISO 9001",
        "certification_name": "ISO 9001:2015 Quality Management",
        "certifying_body": "Bureau Veritas",
        "certificate_number": "CERT-2025-001",
        "issued_date": "2024-01-15",
        "expiry_date": "2027-01-15",
        "notes": "Full scope certification"
    });

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/certifications")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["certification_type"], "ISO 9001");
    assert_eq!(body["certification_name"], "ISO 9001:2015 Quality Management");
    assert_eq!(body["status"], "active");
    assert_eq!(body["supplier_name"], "Certified Supplier Co");
}

#[tokio::test]
async fn test_list_certifications() {
    let (_state, app) = setup_test().await;
    let supplier_id = Uuid::new_v4();

    // Create two certifications
    for cert_type in &["ISO 9001", "ISO 14001"] {
        let (k, v) = auth_header(&admin_claims());
        let payload = json!({
            "supplier_id": supplier_id.to_string(),
            "supplier_name": "Multi-Cert Supplier",
            "certification_type": *cert_type,
            "certification_name": format!("{} Certificate", cert_type),
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/supplier-qualification/certifications")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // List all
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/supplier-qualification/certifications?supplier_id={}", supplier_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_revoke_certification() {
    let (_state, app) = setup_test().await;
    let supplier_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplier_id": supplier_id.to_string(),
        "supplier_name": "Revoke Test Supplier",
        "certification_type": "ISO 9001",
        "certification_name": "ISO 9001 Certificate",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/certifications")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let cert: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let cert_id = cert["id"].as_str().unwrap();

    // Revoke
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/certifications/{}/revoke", cert_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "revoked");
}

#[tokio::test]
async fn test_renew_certification() {
    let (_state, app) = setup_test().await;
    let supplier_id = Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "supplier_id": supplier_id.to_string(),
        "supplier_name": "Renew Test Supplier",
        "certification_type": "ISO 9001",
        "certification_name": "ISO 9001 Certificate",
        "expiry_date": "2025-06-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/certifications")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    let cert: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let cert_id = cert["id"].as_str().unwrap();

    // Renew
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/certifications/{}/renew", cert_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expiry_date": "2028-06-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_qualification_dashboard() {
    let (_state, app) = setup_test().await;

    // Create some data first
    create_test_area(&app, "QA-DASH", "Dashboard Area").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/supplier-qualification/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Verify all expected fields are present
    assert!(body.get("total_active_areas").is_some());
    assert!(body.get("total_active_initiatives").is_some());
    assert!(body.get("total_suppliers_qualified").is_some());
    assert!(body.get("total_suppliers_pending").is_some());
    assert!(body.get("total_certifications_active").is_some());
    assert!(body.get("qualification_rate_percent").is_some());
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_area_with_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "area_code": "",
        "name": "No Code Area",
        "area_type": "questionnaire",
        "scoring_model": "manual",
        "passing_score": "70",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/areas")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_area_with_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "area_code": "QA-INVALID",
        "name": "Invalid Type Area",
        "area_type": "nonexistent_type",
        "scoring_model": "manual",
        "passing_score": "70",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/supplier-qualification/areas")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_nonexistent_initiative_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/activate", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_disqualify_without_reason_fails() {
    let (_state, app) = setup_test().await;
    let area = create_test_area(&app, "QA-DISQ-REASON", "Disq Reason Area").await;
    let area_id: Uuid = area["id"].as_str().unwrap().parse().unwrap();

    let initiative = create_test_initiative(&app, area_id, "Disq Reason Test").await;
    let init_id: Uuid = initiative["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/initiatives/{}/activate", init_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let invitation = invite_test_supplier(&app, init_id, "Test Supplier").await;
    let inv_id: Uuid = invitation["id"].as_str().unwrap().parse().unwrap();

    // Submit and evaluate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/submit", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/evaluate", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to disqualify with empty reason
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/supplier-qualification/invitations/{}/disqualify", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": ""
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
