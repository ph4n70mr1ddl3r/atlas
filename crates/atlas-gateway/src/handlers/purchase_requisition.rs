//! Purchase Requisition API Handlers
//!
//! Oracle Fusion Cloud ERP: Self-Service Procurement > Requisitions
//!
//! Endpoints for managing purchase requisitions, lines, distributions,
//! approval workflow, and AutoCreate conversion to purchase orders.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListRequisitionsQuery {
    pub status: Option<String>,
    pub requester_id: Option<String>,
}

// ============================================================================
// Requisition CRUD
// ============================================================================

/// Create a new purchase requisition
pub async fn create_requisition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let lines = match body["lines"].as_array() {
        Some(arr) => arr.iter().filter_map(|l| {
            let dists = l["distributions"].as_array().map(|da| {
                da.iter().filter_map(|d| {
                    Some(atlas_shared::RequisitionDistributionRequest {
                        charge_account_code: d["charge_account_code"].as_str()?.to_string(),
                        allocation_percentage: d["allocation_percentage"].as_str().map(String::from),
                        amount: d["amount"].as_str().map(String::from),
                        project_code: d["project_code"].as_str().map(String::from),
                        cost_center: d["cost_center"].as_str().map(String::from),
                    })
                }).collect::<Vec<_>>()
            });
            Some(atlas_shared::RequisitionLineRequest {
                item_code: l["item_code"].as_str().map(String::from),
                item_description: l["item_description"].as_str().unwrap_or("").to_string(),
                category: l["category"].as_str().map(String::from),
                quantity: l["quantity"].as_str().map(String::from),
                unit_of_measure: l["unit_of_measure"].as_str().map(String::from),
                unit_price: l["unit_price"].as_str().map(String::from),
                currency_code: l["currency_code"].as_str().map(String::from),
                charge_account_code: l["charge_account_code"].as_str().map(String::from),
                requested_delivery_date: l["requested_delivery_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                supplier_id: l["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                supplier_name: l["supplier_name"].as_str().map(String::from),
                source_type: l["source_type"].as_str().map(String::from),
                source_reference: l["source_reference"].as_str().map(String::from),
                notes: l["notes"].as_str().map(String::from),
                distributions: dists,
            })
        }).collect(),
        None => Vec::new(),
    };

    let request = atlas_shared::PurchaseRequisitionRequest {
        description: body["description"].as_str().map(String::from),
        urgency_code: body["urgency_code"].as_str().map(String::from),
        requester_id: body["requester_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        requester_name: body["requester_name"].as_str().map(String::from),
        department: body["department"].as_str().map(String::from),
        justification: body["justification"].as_str().map(String::from),
        budget_code: body["budget_code"].as_str().map(String::from),
        amount_limit: body["amount_limit"].as_str().map(String::from),
        currency_code: body["currency_code"].as_str().map(String::from),
        charge_account_code: body["charge_account_code"].as_str().map(String::from),
        delivery_address: body["delivery_address"].as_str().map(String::from),
        requested_delivery_date: body["requested_delivery_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        notes: body["notes"].as_str().map(String::from),
        lines,
    };

    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.purchase_requisition_engine.create_requisition(org_id, &request, created_by).await {
        Ok(req) => Ok((StatusCode::CREATED, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a requisition by ID
pub async fn get_requisition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.get_requisition(id).await {
        Ok(Some(req)) => Ok(Json(serde_json::to_value(req).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Requisition not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List requisitions
pub async fn list_requisitions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRequisitionsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let requester_id = query.requester_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    match state.purchase_requisition_engine.list_requisitions(org_id, query.status.as_deref(), requester_id).await {
        Ok(requisitions) => Ok(Json(json!({"data": requisitions}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Update a requisition
pub async fn update_requisition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let lines = match body["lines"].as_array() {
        Some(arr) => arr.iter().filter_map(|l| {
            Some(atlas_shared::RequisitionLineRequest {
                item_code: l["item_code"].as_str().map(String::from),
                item_description: l["item_description"].as_str().unwrap_or("").to_string(),
                category: l["category"].as_str().map(String::from),
                quantity: l["quantity"].as_str().map(String::from),
                unit_of_measure: l["unit_of_measure"].as_str().map(String::from),
                unit_price: l["unit_price"].as_str().map(String::from),
                currency_code: l["currency_code"].as_str().map(String::from),
                charge_account_code: l["charge_account_code"].as_str().map(String::from),
                requested_delivery_date: l["requested_delivery_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                supplier_id: l["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                supplier_name: l["supplier_name"].as_str().map(String::from),
                source_type: l["source_type"].as_str().map(String::from),
                source_reference: l["source_reference"].as_str().map(String::from),
                notes: l["notes"].as_str().map(String::from),
                distributions: None,
            })
        }).collect(),
        None => Vec::new(),
    };

    let request = atlas_shared::PurchaseRequisitionRequest {
        description: body["description"].as_str().map(String::from),
        urgency_code: body["urgency_code"].as_str().map(String::from),
        requester_id: body["requester_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        requester_name: body["requester_name"].as_str().map(String::from),
        department: body["department"].as_str().map(String::from),
        justification: body["justification"].as_str().map(String::from),
        budget_code: body["budget_code"].as_str().map(String::from),
        amount_limit: body["amount_limit"].as_str().map(String::from),
        currency_code: body["currency_code"].as_str().map(String::from),
        charge_account_code: body["charge_account_code"].as_str().map(String::from),
        delivery_address: body["delivery_address"].as_str().map(String::from),
        requested_delivery_date: body["requested_delivery_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        notes: body["notes"].as_str().map(String::from),
        lines,
    };

    let updated_by = Uuid::parse_str(&claims.sub).ok();

    match state.purchase_requisition_engine.update_requisition(id, org_id, &request, updated_by).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Delete a requisition
pub async fn delete_requisition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.delete_requisition(id).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Deleted"})))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Requisition Lines
// ============================================================================

/// Add a line to a requisition
pub async fn add_requisition_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(requisition_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let distributions = body["distributions"].as_array().map(|da| {
        da.iter().filter_map(|d| {
            Some(atlas_shared::RequisitionDistributionRequest {
                charge_account_code: d["charge_account_code"].as_str()?.to_string(),
                allocation_percentage: d["allocation_percentage"].as_str().map(String::from),
                amount: d["amount"].as_str().map(String::from),
                project_code: d["project_code"].as_str().map(String::from),
                cost_center: d["cost_center"].as_str().map(String::from),
            })
        }).collect::<Vec<_>>()
    });

    let request = atlas_shared::RequisitionLineRequest {
        item_code: body["item_code"].as_str().map(String::from),
        item_description: body["item_description"].as_str().unwrap_or("").to_string(),
        category: body["category"].as_str().map(String::from),
        quantity: body["quantity"].as_str().map(String::from),
        unit_of_measure: body["unit_of_measure"].as_str().map(String::from),
        unit_price: body["unit_price"].as_str().map(String::from),
        currency_code: body["currency_code"].as_str().map(String::from),
        charge_account_code: body["charge_account_code"].as_str().map(String::from),
        requested_delivery_date: body["requested_delivery_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        supplier_id: body["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        supplier_name: body["supplier_name"].as_str().map(String::from),
        source_type: body["source_type"].as_str().map(String::from),
        source_reference: body["source_reference"].as_str().map(String::from),
        notes: body["notes"].as_str().map(String::from),
        distributions,
    };

    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.purchase_requisition_engine.add_line(org_id, requisition_id, &request, created_by).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List lines for a requisition
pub async fn list_requisition_lines(
    State(state): State<Arc<AppState>>,
    Path(requisition_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.list_lines(requisition_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Remove a line from a requisition
pub async fn remove_requisition_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.remove_line(line_id).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Line removed"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Distributions
// ============================================================================

/// Add a distribution to a line
pub async fn add_requisition_distribution(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((requisition_id, line_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let request = atlas_shared::RequisitionDistributionRequest {
        charge_account_code: body["charge_account_code"].as_str().unwrap_or("").to_string(),
        allocation_percentage: body["allocation_percentage"].as_str().map(String::from),
        amount: body["amount"].as_str().map(String::from),
        project_code: body["project_code"].as_str().map(String::from),
        cost_center: body["cost_center"].as_str().map(String::from),
    };

    match state.purchase_requisition_engine.add_distribution(org_id, requisition_id, line_id, &request).await {
        Ok(dist) => Ok((StatusCode::CREATED, Json(serde_json::to_value(dist).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List distributions for a line
pub async fn list_requisition_distributions(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.list_distributions(line_id).await {
        Ok(distributions) => Ok(Json(json!({"data": distributions}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Approval Workflow
// ============================================================================

/// Submit a requisition for approval
pub async fn submit_requisition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let submitted_by = Uuid::parse_str(&claims.sub).ok();
    match state.purchase_requisition_engine.submit_requisition(id, submitted_by).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Approve a requisition
pub async fn approve_requisition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approver_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    let approver_name = claims.email.clone();
    let comments = body["comments"].as_str().map(String::from);

    match state.purchase_requisition_engine.approve_requisition(
        id, approver_id, Some(&approver_name), comments.as_deref()
    ).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Reject a requisition
pub async fn reject_requisition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approver_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    let approver_name = claims.email.clone();
    let comments = body["comments"].as_str().map(String::from);

    match state.purchase_requisition_engine.reject_requisition(
        id, approver_id, Some(&approver_name), comments.as_deref()
    ).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Cancel a requisition
pub async fn cancel_requisition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.cancel_requisition(id).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Close a requisition
pub async fn close_requisition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.close_requisition(id).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Return a requisition to draft
pub async fn return_requisition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.return_requisition(id).await {
        Ok(req) => Ok((StatusCode::OK, Json(serde_json::to_value(req).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List approvals for a requisition
pub async fn list_requisition_approvals(
    State(state): State<Arc<AppState>>,
    Path(requisition_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.list_approvals(requisition_id).await {
        Ok(approvals) => Ok(Json(json!({"data": approvals}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// AutoCreate
// ============================================================================

/// Create purchase orders from approved requisition lines (AutoCreate)
pub async fn autocreate(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let line_ids = body["requisition_line_ids"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok())).collect())
        .unwrap_or_default();

    let request = atlas_shared::AutocreateRequest {
        requisition_line_ids: line_ids,
        purchase_order_number: body["purchase_order_number"].as_str().map(String::from),
        supplier_id: body["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        supplier_name: body["supplier_name"].as_str().map(String::from),
    };

    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.purchase_requisition_engine.autocreate(org_id, &request, created_by).await {
        Ok(links) => Ok((StatusCode::CREATED, Json(json!({"data": links})))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List AutoCreate links for a requisition
pub async fn list_autocreate_links(
    State(state): State<Arc<AppState>>,
    Path(requisition_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.list_autocreate_links(requisition_id).await {
        Ok(links) => Ok(Json(json!({"data": links}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel an AutoCreate link
pub async fn cancel_autocreate_link(
    State(state): State<Arc<AppState>>,
    Path(link_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.purchase_requisition_engine.cancel_autocreate_link(link_id).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "AutoCreate link cancelled"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get requisition dashboard summary
pub async fn get_requisition_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.purchase_requisition_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}