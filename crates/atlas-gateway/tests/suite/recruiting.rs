//! Recruiting Management E2E Tests
//!
//! Tests for Oracle Fusion HCM Recruiting:
//! - Job Requisition CRUD + lifecycle (draft -> open -> filled/closed)
//! - Candidate CRUD + status updates
//! - Job Applications (apply, screen, advance pipeline)
//! - Interview scheduling & feedback
//! - Job Offers (create -> approve -> extend -> accept/decline)
//! - Dashboard analytics

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_recruiting_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean recruiting data for isolation
    sqlx::query("DELETE FROM _atlas.job_offers").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.interviews").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.job_applications").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.candidates").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.job_requisitions").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_requisition(app: &axum::Router, number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/recruiting/requisitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "requisition_number": number,
            "title": format!("Senior Engineer - {}", number),
            "department": "Engineering",
            "location": "San Francisco, CA",
            "employment_type": "full_time",
            "position_type": "new",
            "vacancies": 2,
            "priority": "high",
            "salary_min": "120000",
            "salary_max": "180000",
            "experience_years_min": 5,
            "experience_years_max": 10,
            "education_level": "bachelors",
            "required_skills": ["Rust", "PostgreSQL", "Distributed Systems"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_candidate(app: &axum::Router, first: &str, last: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/recruiting/candidates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "first_name": first,
            "last_name": last,
            "email": format!("{}{}@example.com", first.to_lowercase(), last.to_lowercase()),
            "phone": "+1-555-0100",
            "source": "referral",
            "years_of_experience": 7,
            "education_level": "masters",
            "skills": ["Rust", "Go", "PostgreSQL"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn open_requisition(app: &axum::Router, id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/open", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_application(app: &axum::Router, req_id: &str, cand_id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/recruiting/applications")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "requisition_id": req_id,
            "candidate_id": cand_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Requisition Tests
// ============================================================================

#[tokio::test]
async fn test_create_requisition() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-001").await;
    assert_eq!(req["requisitionNumber"], "REQ-001");
    assert_eq!(req["title"], "Senior Engineer - REQ-001");
    assert_eq!(req["status"], "draft");
    assert_eq!(req["department"], "Engineering");
    assert_eq!(req["employmentType"], "full_time");
    assert_eq!(req["vacancies"], 2);
    assert_eq!(req["priority"], "high");
    assert!(req["id"].is_string());
}

#[tokio::test]
async fn test_create_requisition_duplicate() {
    let (_state, app) = setup_recruiting_test().await;
    create_test_requisition(&app, "REQ-DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/recruiting/requisitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "requisition_number": "REQ-DUP", "title": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_requisitions() {
    let (_state, app) = setup_recruiting_test().await;
    create_test_requisition(&app, "REQ-LA").await;
    create_test_requisition(&app, "REQ-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/recruiting/requisitions?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_requisition() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-GET").await;
    let id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/recruiting/requisitions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["requisitionNumber"], "REQ-GET");
}

#[tokio::test]
async fn test_requisition_lifecycle() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-LC").await;
    let id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Open
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/open", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let opened: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(opened["status"], "open");
    assert!(opened["postedDate"].is_string());

    // Hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/hold", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let held: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(held["status"], "on_hold");

    // Re-open from hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/open", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/close", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let closed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(closed["status"], "closed");
    assert!(closed["closedDate"].is_string());
}

#[tokio::test]
async fn test_cancel_requisition() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-CNL").await;
    let id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_delete_requisition() {
    let (_state, app) = setup_recruiting_test().await;
    create_test_requisition(&app, "REQ-DEL").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/recruiting/requisitions-by-number/REQ-DEL").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_open_non_draft_rejected() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-ND").await;
    let id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Open once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/open", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to open again (already open)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/open", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Candidate Tests
// ============================================================================

#[tokio::test]
async fn test_create_candidate() {
    let (_state, app) = setup_recruiting_test().await;
    let cand = create_test_candidate(&app, "Jane", "Smith").await;
    assert_eq!(cand["firstName"], "Jane");
    assert_eq!(cand["lastName"], "Smith");
    assert_eq!(cand["email"], "janesmith@example.com");
    assert_eq!(cand["source"], "referral");
    assert_eq!(cand["yearsOfExperience"], 7);
    assert_eq!(cand["status"], "active");
    assert!(cand["id"].is_string());
}

#[tokio::test]
async fn test_list_candidates() {
    let (_state, app) = setup_recruiting_test().await;
    create_test_candidate(&app, "Alice", "Foo").await;
    create_test_candidate(&app, "Bob", "Bar").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/recruiting/candidates").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_candidate() {
    let (_state, app) = setup_recruiting_test().await;
    let cand = create_test_candidate(&app, "John", "Doe").await;
    let id = cand["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/recruiting/candidates/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["firstName"], "John");
}

#[tokio::test]
async fn test_update_candidate_status() {
    let (_state, app) = setup_recruiting_test().await;
    let cand = create_test_candidate(&app, "Status", "Test").await;
    let id = cand["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/candidates/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "inactive"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "inactive");
}

#[tokio::test]
async fn test_delete_candidate() {
    let (_state, app) = setup_recruiting_test().await;
    let cand = create_test_candidate(&app, "Del", "Candidate").await;
    let id = cand["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/recruiting/candidates/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Application Tests
// ============================================================================

#[tokio::test]
async fn test_create_application() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-APP").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "App", "Test").await;
    let cand_id = cand["id"].as_str().unwrap();

    let app_resp = create_test_application(&app, req_id, cand_id).await;
    assert_eq!(app_resp["status"], "applied");
    assert_eq!(app_resp["requisitionId"], req_id);
    assert_eq!(app_resp["candidateId"], cand_id);
    assert!(app_resp["id"].is_string());
}

#[tokio::test]
async fn test_application_to_closed_requisition_rejected() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-CLO").await;
    let req_id = req["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Close without opening (still draft, close works)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/requisitions/{}/close", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let cand = create_test_candidate(&app, "Closed", "App").await;
    let cand_id = cand["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/recruiting/applications")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "requisition_id": req_id, "candidate_id": cand_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_application_pipeline() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-PIPE").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Pipe", "Line").await;
    let cand_id = cand["id"].as_str().unwrap();
    let application = create_test_application(&app, req_id, cand_id).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Advance to screening
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/status", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "screening"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Advance to interview
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/status", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "interview"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Advance to assessment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/status", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "assessment"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let assessed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(assessed["status"], "assessment");
}

#[tokio::test]
async fn test_withdraw_application() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-WD").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "With", "Draw").await;
    let cand_id = cand["id"].as_str().unwrap();
    let application = create_test_application(&app, req_id, cand_id).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/withdraw", app_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let withdrawn: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(withdrawn["status"], "withdrawn");
}

#[tokio::test]
async fn test_list_applications() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-LIST").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let c1 = create_test_candidate(&app, "List1", "A").await;
    let c2 = create_test_candidate(&app, "List2", "B").await;
    create_test_application(&app, req_id, c1["id"].as_str().unwrap()).await;
    create_test_application(&app, req_id, c2["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/recruiting/applications?requisition_id={}", req_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Interview Tests
// ============================================================================

#[tokio::test]
async fn test_create_interview() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-INT").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Inter", "View").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/interviews", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "interview_type": "technical",
            "round": 1,
            "duration_minutes": 90,
            "location": "Conference Room A",
            "interviewer_names": ["Alice Manager", "Bob Tech"],
            "notes": "Focus on systems design"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let interview: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(interview["interviewType"], "technical");
    assert_eq!(interview["round"], 1);
    assert_eq!(interview["durationMinutes"], 90);
    assert_eq!(interview["status"], "scheduled");
}

#[tokio::test]
async fn test_complete_interview() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-COMP").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Comp", "Int").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create interview
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/interviews", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "interview_type": "phone", "round": 1, "duration_minutes": 30
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let interview: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let int_id = interview["id"].as_str().unwrap();

    // Complete with feedback
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/interviews/{}/complete", int_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "feedback": "Strong technical skills, excellent communication",
            "rating": 4,
            "recommendation": "hire"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["rating"], 4);
    assert_eq!(completed["recommendation"], "hire");
    assert!(completed["completedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_interview() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-CINT").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Cancel", "Int").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/interviews", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "interview_type": "video", "round": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let interview: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let int_id = interview["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/interviews/{}/cancel", int_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_interviews() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-LINT").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "List", "Ints").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    for round in 1..=2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/recruiting/applications/{}/interviews", app_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "interview_type": "technical", "round": round
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/recruiting/applications/{}/interviews", app_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Offer Tests
// ============================================================================

#[tokio::test]
async fn test_offer_lifecycle() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-OFR").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Offer", "Test").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create offer
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/offers", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "offer_number": "OFR-001",
            "job_title": "Senior Software Engineer",
            "department": "Engineering",
            "employment_type": "full_time",
            "salary_offered": "150000",
            "salary_currency": "USD",
            "start_date": "2026-06-01",
            "signing_bonus": "10000",
            "benefits_summary": "Full medical, dental, 401k match"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(offer["status"], "draft");
    assert_eq!(offer["jobTitle"], "Senior Software Engineer");
    let offer_id = offer["id"].as_str().unwrap();

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/approve", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedAt"].is_string());

    // Extend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/extend", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let extended: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(extended["status"], "extended");
    assert!(extended["offerDate"].is_string());

    // Accept
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/accept", offer_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"notes": "Excited to join!"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let accepted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(accepted["status"], "accepted");
    assert!(accepted["respondedAt"].is_string());
}

#[tokio::test]
async fn test_decline_offer() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-DEC").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Dec", "Liner").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create, approve, extend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/offers", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "job_title": "Engineer", "salary_offered": "100000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let offer_id = offer["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/approve", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/extend", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Decline
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/decline", offer_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"notes": "Accepted another offer"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let declined: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(declined["status"], "declined");
}

#[tokio::test]
async fn test_withdraw_offer() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-WOF").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "With", "Offer").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/offers", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "job_title": "Engineer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let offer_id = offer["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/withdraw", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let withdrawn: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(withdrawn["status"], "withdrawn");
}

#[tokio::test]
async fn test_list_offers() {
    let (_state, app) = setup_recruiting_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/recruiting/offers").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].is_array());
}

#[tokio::test]
async fn test_extend_non_approved_rejected() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-EXT").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Ext", "Test").await;
    let application = create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let app_id = application["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/applications/{}/offers", app_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "job_title": "Engineer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let offer_id = offer["id"].as_str().unwrap();

    // Try to extend without approving first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recruiting/offers/{}/extend", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_recruiting_dashboard() {
    let (_state, app) = setup_recruiting_test().await;
    let req = create_test_requisition(&app, "REQ-DB").await;
    let req_id = req["id"].as_str().unwrap();
    open_requisition(&app, req_id).await;
    let cand = create_test_candidate(&app, "Dash", "Board").await;
    create_test_application(&app, req_id, cand["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/recruiting/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalRequisitions"].as_i64().unwrap() >= 1);
    assert!(dashboard["openRequisitions"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalCandidates"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalApplications"].as_i64().unwrap() >= 1);
    assert!(dashboard["requisitionsByStatus"].is_array());
    assert!(dashboard["applicationsByStatus"].is_array());
}
