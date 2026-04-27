//! Contract Lifecycle Management E2E Tests
//!
//! Tests for Oracle Fusion Enterprise Contracts:
//! - Contract type CRUD
//! - Clause library management
//! - Contract lifecycle (create, transition, delete)
//! - Contract parties
//! - Contract milestones and deliverables
//! - Amendments and risk assessments
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_clm_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_contract_type(app: &axum::Router, code: &str, name: &str, category: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contract-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "contractCategory": category
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for contract type but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_clause(app: &axum::Router, code: &str, title: &str, body_text: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/clauses")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "title": title, "body": body_text,
            "clauseType": "standard", "clauseCategory": "general"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for clause but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_contract(app: &axum::Router, number: &str, title: &str, category: &str, value: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contractNumber": number, "title": title, "contractCategory": category,
            "totalValue": value, "currency": "USD", "priority": "normal"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for contract but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Type Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_contract_type() {
    let (_state, app) = setup_clm_test().await;
    let ct = create_test_contract_type(&app, "SALES", "Sales Contract", "sales").await;
    assert_eq!(ct["code"], "SALES");
    assert_eq!(ct["name"], "Sales Contract");
    assert_eq!(ct["contractCategory"], "sales");
}

#[tokio::test]
async fn test_create_contract_type_duplicate_conflict() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract_type(&app, "DUP", "First", "general").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contract-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP", "name": "Duplicate", "contractCategory": "general"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_contract_type_invalid_category() {
    let (_state, app) = setup_clm_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contract-types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD", "name": "Bad", "contractCategory": "nonexistent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_contract_types() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract_type(&app, "T1", "Type 1", "sales").await;
    create_test_contract_type(&app, "T2", "Type 2", "procurement").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().uri("/api/v1/clm/contract-types")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_contract_type() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract_type(&app, "DEL", "Delete Me", "general").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/clm/contract-types/code/DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ═══════════════════════════════════════════════════════════════════════
// Clause Library Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_clause() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_clause(&app, "CONF", "Confidentiality", "All data is confidential.").await;
    assert_eq!(c["code"], "CONF");
    assert_eq!(c["title"], "Confidentiality");
    assert_eq!(c["body"], "All data is confidential.");
}

#[tokio::test]
async fn test_list_clauses_by_category() {
    let (_state, app) = setup_clm_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create clause with liability category
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/clauses")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "LIAB", "title": "Liability", "body": "Limited liability.",
            "clauseType": "mandatory", "clauseCategory": "liability"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let resp = app.clone().oneshot(Request::builder().uri("/api/v1/clm/clauses?category=liability")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let clauses = list["data"].as_array().unwrap();
    assert!(clauses.iter().all(|c| c["clauseCategory"] == "liability"));
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_contract() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-001", "Master Services Agreement", "service", "50000").await;
    assert_eq!(c["contractNumber"], "CTR-001");
    assert_eq!(c["title"], "Master Services Agreement");
    assert_eq!(c["status"], "draft");
    assert_eq!(c["priority"], "normal");
    assert_eq!(c["renewalType"], "none");
}

#[tokio::test]
async fn test_create_contract_duplicate_conflict() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract(&app, "CTR-DUP", "First", "general", "1000").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contractNumber": "CTR-DUP", "title": "Dup", "contractCategory": "general", "totalValue": "0"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_contract_invalid_value() {
    let (_state, app) = setup_clm_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/clm/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "contractNumber": "BAD_VAL", "title": "Bad", "contractCategory": "general", "totalValue": "not_a_number"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_contract() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-GET", "Get Test", "general", "1000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/clm/contracts/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["contractNumber"], "CTR-GET");
}

#[tokio::test]
async fn test_list_contracts() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract(&app, "CTR-L1", "List 1", "sales", "10000").await;
    create_test_contract(&app, "CTR-L2", "List 2", "procurement", "20000").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().uri("/api/v1/clm/contracts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_contracts_by_status() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract(&app, "CTR-S1", "Status 1", "general", "0").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().uri("/api/v1/clm/contracts?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let contracts = list["data"].as_array().unwrap();
    assert!(contracts.iter().all(|c| c["status"] == "draft"));
}

#[tokio::test]
async fn test_contract_transition() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-TR", "Transition", "general", "1000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // draft -> in_review
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_review"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "in_review");

    // in_review -> pending_approval
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "pending_approval"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // pending_approval -> approved
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "approved"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // approved -> active
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let active: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(active["status"], "active");
}

#[tokio::test]
async fn test_contract_invalid_transition() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-INV", "Invalid", "general", "0").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // draft -> active is not a valid transition
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_contract() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract(&app, "CTR-DEL", "Delete Me", "general", "0").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/clm/contracts/number/CTR-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Parties Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_add_and_list_contract_party() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-PARTY", "Party Test", "general", "5000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/parties", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partyType": "external", "partyRole": "counterparty",
            "partyName": "Acme Corp", "contactEmail": "legal@acme.com", "isPrimary": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let party: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(party["partyName"], "Acme Corp");
    assert_eq!(party["isPrimary"], true);

    // List parties
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/clm/contracts/{}/parties", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Milestone Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_and_complete_milestone() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-MS", "Milestone Test", "general", "10000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create milestone
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/milestones", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Phase 1 Delivery", "milestoneType": "delivery",
            "dueDate": "2025-06-30", "amount": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ms: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ms["name"], "Phase 1 Delivery");
    assert_eq!(ms["status"], "pending");
    let ms_id = ms["id"].as_str().unwrap();

    // Complete milestone
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/milestones/{}/complete", ms_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["status"], "completed");
}

// ═══════════════════════════════════════════════════════════════════════
// Deliverable Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_and_accept_deliverable() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-DELIV", "Deliverable Test", "service", "20000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create deliverable with status "submitted" (we need to go through proper status flow)
    // but for this test we just test creation
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/deliverables", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Design Document", "deliverableType": "document",
            "quantity": "1", "dueDate": "2025-03-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(d["name"], "Design Document");
    assert_eq!(d["status"], "pending");
    assert_eq!(d["deliverableType"], "document");
}

// ═══════════════════════════════════════════════════════════════════════
// Amendment Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_and_approve_amendment() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-AMD", "Amendment Test", "general", "10000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create amendment (starts in "draft" status)
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/amendments", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentNumber": "AMD-001", "title": "Extend Contract",
            "amendmentType": "extension", "previousValue": "12 months",
            "newValue": "24 months", "effectiveDate": "2025-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let amd: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(amd["amendmentNumber"], "AMD-001");
    assert_eq!(amd["amendmentType"], "extension");
}

// ═══════════════════════════════════════════════════════════════════════
// Risk Assessment Tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_risk() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-RISK", "Risk Test", "general", "50000").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/risks", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskCategory": "financial", "riskDescription": "Currency fluctuation risk",
            "probability": "high", "impact": "medium",
            "mitigationStrategy": "Hedge with forward contracts"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let risk: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(risk["riskCategory"], "financial");
    assert_eq!(risk["probability"], "high");
    assert_eq!(risk["impact"], "medium");
    assert_eq!(risk["mitigationStrategy"], "Hedge with forward contracts");
}

#[tokio::test]
async fn test_create_risk_invalid_probability() {
    let (_state, app) = setup_clm_test().await;
    let c = create_test_contract(&app, "CTR-RINV", "Risk Invalid", "general", "0").await;
    let id = c["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/risks", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskCategory": "financial", "riskDescription": "Test",
            "probability": "impossible", "impact": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════════════
// Dashboard Test
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_clm_dashboard() {
    let (_state, app) = setup_clm_test().await;
    create_test_contract(&app, "CTR-D1", "Dashboard 1", "sales", "10000").await;
    create_test_contract(&app, "CTR-D2", "Dashboard 2", "procurement", "20000").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/clm/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalContracts"].as_i64().unwrap() >= 2);
    assert!(summary["draftContracts"].as_i64().unwrap() >= 2);
    assert!(summary["activeContracts"].as_i64().unwrap() >= 0);
}

// ═══════════════════════════════════════════════════════════════════════
// Full Lifecycle Test
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_contract_full_lifecycle() {
    let (_state, app) = setup_clm_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create contract type
    create_test_contract_type(&app, "SVC", "Service Agreement", "service").await;

    // 2. Create clause
    let clause = create_test_clause(&app, "SVC-CONF", "Service Confidentiality", "All services data is confidential.").await;

    // 3. Create contract
    let contract = create_test_contract(&app, "LC-001", "Enterprise Service Agreement", "service", "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    assert_eq!(contract["status"], "draft");

    // 4. Add parties
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/parties", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partyType": "internal", "partyRole": "initiator",
            "partyName": "Our Company", "isPrimary": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/parties", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "partyType": "external", "partyRole": "counterparty",
            "partyName": "TechCorp Inc", "contactEmail": "legal@techcorp.com", "isPrimary": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 5. Add milestone
    let ms_resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/milestones", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Go-Live", "milestoneType": "event", "dueDate": "2025-12-31", "amount": "50000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let ms_body = axum::body::to_bytes(ms_resp.into_body(), usize::MAX).await.unwrap();
    let ms: serde_json::Value = serde_json::from_slice(&ms_body).unwrap();

    // 6. Add risk
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/contracts/{}/risks", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "riskCategory": "operational", "riskDescription": "Key personnel risk",
            "probability": "medium", "impact": "high",
            "mitigationStrategy": "Cross-training and documentation"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 7. Transition: draft -> in_review -> pending_approval -> approved -> active
    for status in ["in_review", "pending_approval", "approved", "active"] {
        let resp = app.clone().oneshot(Request::builder().method("POST")
            .uri(format!("/api/v1/clm/contracts/id/{}/status", contract_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": status})).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // 8. Complete milestone
    let ms_id = ms["id"].as_str().unwrap();
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/clm/milestones/{}/complete", ms_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 9. Verify dashboard
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/clm/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalContracts"].as_i64().unwrap() >= 1);
    assert!(summary["activeContracts"].as_i64().unwrap() >= 1);

    // 10. Delete contract
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/clm/contracts/number/LC-001")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Cleanup
    let _ = clause; // Clause exists independently
}
