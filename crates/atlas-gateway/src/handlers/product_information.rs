//! Product Information Management (PIM) API Handlers
//!
//! Oracle Fusion Cloud ERP: Product Hub > Manage Items
//!
//! Endpoints for managing product items, categories, cross-references,
//! new item requests, and item templates.

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
pub struct ListItemsQuery {
    pub status: Option<String>,
    pub item_type: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCrossReferencesQuery {
    pub xref_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListNirsQuery {
    pub status: Option<String>,
}

// ============================================================================
// Product Item CRUD
// ============================================================================

/// Create a new product item
pub async fn create_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let item_number = body["item_number"].as_str().unwrap_or("");
    let item_name = body["item_name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let long_description = body["long_description"].as_str();
    let item_type = body["item_type"].as_str().unwrap_or("finished_good");
    let primary_uom = body["primary_uom_code"].as_str().unwrap_or("EA");
    let secondary_uom = body["secondary_uom_code"].as_str();
    let weight = body["weight"].as_str();
    let weight_uom = body["weight_uom"].as_str();
    let volume = body["volume"].as_str();
    let volume_uom = body["volume_uom"].as_str();
    let hazmat = body["hazmat_flag"].as_bool().unwrap_or(false);
    let lot_control = body["lot_control_flag"].as_bool().unwrap_or(false);
    let serial_control = body["serial_control_flag"].as_bool().unwrap_or(false);
    let shelf_life = body["shelf_life_days"].as_i64().map(|v| v as i32);
    let min_order = body["min_order_quantity"].as_str();
    let max_order = body["max_order_quantity"].as_str();
    let lead_time = body["lead_time_days"].as_i64().map(|v| v as i32);
    let list_price = body["list_price"].as_str();
    let cost_price = body["cost_price"].as_str();
    let currency = body["currency_code"].as_str().unwrap_or("USD");
    let inventory_flag = body["inventory_item_flag"].as_bool().unwrap_or(true);
    let purchasable = body["purchasable_flag"].as_bool().unwrap_or(true);
    let sellable = body["sellable_flag"].as_bool().unwrap_or(true);
    let stock_enabled = body["stock_enabled_flag"].as_bool().unwrap_or(true);
    let invoice_enabled = body["invoice_enabled_flag"].as_bool().unwrap_or(true);
    let default_buyer = body["default_buyer_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let default_supplier = body["default_supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let template_id = body["template_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    match state.product_information_engine.create_item(
        org_id, item_number, item_name, description, long_description,
        item_type, primary_uom, secondary_uom,
        weight, weight_uom, volume, volume_uom,
        hazmat, lot_control, serial_control, shelf_life,
        min_order, max_order, lead_time,
        list_price, cost_price, currency,
        inventory_flag, purchasable, sellable, stock_enabled, invoice_enabled,
        default_buyer, default_supplier, template_id,
        created_by,
    ).await {
        Ok(item) => Ok((StatusCode::CREATED, Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get an item by ID
pub async fn get_item(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.get_item(id).await {
        Ok(Some(item)) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Item not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Get an item by item number
pub async fn get_item_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(item_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.product_information_engine.get_item_by_number(org_id, &item_number).await {
        Ok(Some(item)) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Item not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List items with optional filtering
pub async fn list_items(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let category_id = query.category_id.as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.product_information_engine.list_items(
        org_id,
        query.status.as_deref(),
        query.item_type.as_deref(),
        category_id,
    ).await {
        Ok(items) => Ok(Json(json!({"data": items}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Update item status
pub async fn update_item_status(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = body["status"].as_str().unwrap_or("");

    match state.product_information_engine.update_item_status(id, status).await {
        Ok(item) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status_code = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Update item lifecycle phase
pub async fn update_item_lifecycle(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let phase = body["lifecycle_phase"].as_str().unwrap_or("");

    match state.product_information_engine.update_lifecycle_phase(id, phase).await {
        Ok(item) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status_code = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status_code, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Delete an item
pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.product_information_engine.delete_item(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Item Categories
// ============================================================================

/// Create a category
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let code = body["code"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let parent_id = body["parent_category_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    match state.product_information_engine.create_category(
        org_id, code, name, description, parent_id, created_by,
    ).await {
        Ok(cat) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cat).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a category by ID
pub async fn get_category(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.get_category(id).await {
        Ok(Some(cat)) => Ok(Json(serde_json::to_value(cat).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Category not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List categories
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let parent_id = query.parent_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match state.product_information_engine.list_categories(org_id, parent_id).await {
        Ok(cats) => Ok(Json(json!({"data": cats}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a category
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.product_information_engine.delete_category(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Item Category Assignments
// ============================================================================

/// Assign an item to a category
pub async fn assign_item_category(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(item_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let category_id = body["category_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "category_id is required"}))))?;
    let is_primary = body["is_primary"].as_bool().unwrap_or(false);

    match state.product_information_engine.assign_item_category(
        org_id, item_id, category_id, is_primary, created_by,
    ).await {
        Ok(assignment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(assignment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List categories for an item
pub async fn list_item_categories(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.list_item_categories(item_id).await {
        Ok(assignments) => Ok(Json(json!({"data": assignments}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Remove item from category
pub async fn remove_item_category(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(assignment_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.product_information_engine.remove_item_category(assignment_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Item Cross-References
// ============================================================================

/// Create a cross-reference
pub async fn create_cross_reference(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(item_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let xref_type = body["cross_reference_type"].as_str().unwrap_or("");
    let xref_value = body["cross_reference_value"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let source_system = body["source_system"].as_str();
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.product_information_engine.create_cross_reference(
        org_id, item_id, xref_type, xref_value,
        description, source_system, effective_from, effective_to,
        created_by,
    ).await {
        Ok(xref) => Ok((StatusCode::CREATED, Json(serde_json::to_value(xref).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List cross-references for an item
pub async fn list_cross_references(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.list_cross_references(item_id).await {
        Ok(xrefs) => Ok(Json(json!({"data": xrefs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all cross-references
pub async fn list_all_cross_references(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCrossReferencesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.product_information_engine.list_all_cross_references(
        org_id, query.xref_type.as_deref(),
    ).await {
        Ok(xrefs) => Ok(Json(json!({"data": xrefs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a cross-reference
pub async fn delete_cross_reference(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.product_information_engine.delete_cross_reference(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Item Templates
// ============================================================================

/// Create a template
pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let code = body["code"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let item_type = body["item_type"].as_str().unwrap_or("finished_good");
    let default_uom = body["default_uom_code"].as_str();
    let default_category_id = body["default_category_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let inventory_flag = body["default_inventory_flag"].as_bool().unwrap_or(true);
    let purchasable = body["default_purchasable_flag"].as_bool().unwrap_or(true);
    let sellable = body["default_sellable_flag"].as_bool().unwrap_or(true);
    let stock_enabled = body["default_stock_enabled_flag"].as_bool().unwrap_or(true);
    let attribute_defaults = body.get("attribute_defaults").cloned().unwrap_or(json!({}));

    match state.product_information_engine.create_template(
        org_id, code, name, description, item_type,
        default_uom, default_category_id,
        inventory_flag, purchasable, sellable, stock_enabled,
        attribute_defaults, created_by,
    ).await {
        Ok(tmpl) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tmpl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List templates
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.product_information_engine.list_templates(org_id).await {
        Ok(templates) => Ok(Json(json!({"data": templates}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a template
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.product_information_engine.delete_template(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// New Item Requests (NIR)
// ============================================================================

/// Create a new item request
pub async fn create_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let title = body["title"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let item_type = body["item_type"].as_str().unwrap_or("finished_good");
    let priority = body["priority"].as_str().unwrap_or("medium");
    let requested_item_number = body["requested_item_number"].as_str();
    let requested_item_name = body["requested_item_name"].as_str();
    let requested_category_id = body["requested_category_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let justification = body["justification"].as_str();
    let target_launch_date = body["target_launch_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let estimated_cost = body["estimated_cost"].as_str();
    let currency = body["currency_code"].as_str().unwrap_or("USD");

    match state.product_information_engine.create_new_item_request(
        org_id, title, description, item_type, priority,
        requested_item_number, requested_item_name,
        requested_category_id, justification,
        target_launch_date, estimated_cost, currency,
        created_by,
    ).await {
        Ok(nir) => Ok((StatusCode::CREATED, Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a NIR by ID
pub async fn get_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.get_new_item_request(id).await {
        Ok(Some(nir)) => Ok(Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "New item request not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List NIRs
pub async fn list_new_item_requests(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListNirsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.product_information_engine.list_new_item_requests(
        org_id, query.status.as_deref(),
    ).await {
        Ok(nirs) => Ok(Json(json!({"data": nirs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Submit a NIR for approval
pub async fn submit_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.submit_new_item_request(id).await {
        Ok(nir) => Ok(Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Approve a NIR
pub async fn approve_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).ok();

    match state.product_information_engine.approve_new_item_request(id, approved_by).await {
        Ok(nir) => Ok(Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Reject a NIR
pub async fn reject_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reason = body["rejection_reason"].as_str();

    match state.product_information_engine.reject_new_item_request(id, reason).await {
        Ok(nir) => Ok(Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Implement an approved NIR
pub async fn implement_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.implement_new_item_request(id).await {
        Ok(item) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Cancel a NIR
pub async fn cancel_new_item_request(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.product_information_engine.cancel_new_item_request(id).await {
        Ok(nir) => Ok(Json(serde_json::to_value(nir).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get PIM dashboard summary
pub async fn get_pim_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.product_information_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}
