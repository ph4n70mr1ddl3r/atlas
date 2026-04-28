//! Product Configurator HTTP Handlers
//!
//! Oracle Fusion Cloud: SCM > Product Management > Configurator endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::auth::Claims;
use crate::AppState;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListModelsQuery {
    pub status: Option<String>,
    pub model_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListInstancesQuery {
    pub status: Option<String>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub rule_type: Option<String>,
}

// ============================================================================
// Model Handlers
// ============================================================================

pub async fn create_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let model_number = body["modelNumber"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let base_product_id = body["baseProductId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let base_product_number = body["baseProductNumber"].as_str();
    let base_product_name = body["baseProductName"].as_str();
    let model_type = body["modelType"].as_str().unwrap_or("standard");
    let effective_from = body["effectiveFrom"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effectiveTo"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let validation_mode = body["validationMode"].as_str().unwrap_or("strict");
    let default_config = body.get("defaultConfig").cloned();
    let ui_layout = body.get("uiLayout").cloned();

    let model = state.configurator_engine.create_model(
        org_id, model_number, name, description,
        base_product_id, base_product_number, base_product_name,
        model_type, effective_from, effective_to,
        default_config, validation_mode, ui_layout, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create config model error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(model).unwrap())))
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListModelsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let models = state.configurator_engine.list_models(
        org_id, params.status.as_deref(), params.model_type.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": models })))
}

pub async fn get_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model = state.configurator_engine.get_model(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match model {
        Some(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn activate_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model = state.configurator_engine.activate_model(id).await.map_err(|e| {
        tracing::error!("Activate model error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(model).unwrap()))
}

pub async fn deactivate_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model = state.configurator_engine.deactivate_model(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(model).unwrap()))
}

pub async fn delete_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(model_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.configurator_engine.delete_model(org_id, &model_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Feature Handlers
// ============================================================================

pub async fn create_feature(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(model_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let feature_code = body["featureCode"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let feature_type = body["featureType"].as_str().unwrap_or("single_select");
    let is_required = body["isRequired"].as_bool().unwrap_or(false);
    let display_order = body["displayOrder"].as_i64().unwrap_or(0) as i32;
    let ui_hints = body.get("uiHints").cloned();

    let feature = state.configurator_engine.create_feature(
        org_id, model_id, feature_code, name, description,
        feature_type, is_required, display_order, ui_hints,
    ).await.map_err(|e| {
        tracing::error!("Create feature error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(feature).unwrap())))
}

pub async fn list_features(
    State(state): State<Arc<AppState>>,
    Path(model_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let features = state.configurator_engine.list_features(model_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": features })))
}

pub async fn delete_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.configurator_engine.delete_feature(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Option Handlers
// ============================================================================

pub async fn create_option(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(feature_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let option_code = body["optionCode"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let option_type = body["optionType"].as_str().unwrap_or("standard");
    let price_adjustment = body["priceAdjustment"].as_f64().unwrap_or(0.0);
    let cost_adjustment = body["costAdjustment"].as_f64().unwrap_or(0.0);
    let lead_time_days = body["leadTimeDays"].as_i64().unwrap_or(0) as i32;
    let is_default = body["isDefault"].as_bool().unwrap_or(false);
    let is_available = body["isAvailable"].as_bool().unwrap_or(true);
    let display_order = body["displayOrder"].as_i64().unwrap_or(0) as i32;

    let option = state.configurator_engine.create_option(
        org_id, feature_id, option_code, name, description,
        option_type, price_adjustment, cost_adjustment,
        lead_time_days, is_default, is_available, display_order,
    ).await.map_err(|e| {
        tracing::error!("Create option error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(option).unwrap())))
}

pub async fn list_options(
    State(state): State<Arc<AppState>>,
    Path(feature_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let options = state.configurator_engine.list_options(feature_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": options })))
}

pub async fn delete_option(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.configurator_engine.delete_option(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Rule Handlers
// ============================================================================

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(model_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rule_code = body["ruleCode"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let rule_type = body["ruleType"].as_str().unwrap_or("compatibility");
    let source_feature_id = body["sourceFeatureId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let source_option_id = body["sourceOptionId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let target_feature_id = body["targetFeatureId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let target_option_id = body["targetOptionId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let condition_expression = body["conditionExpression"].as_str();
    let severity = body["severity"].as_str().unwrap_or("error");
    let is_active = body["isActive"].as_bool().unwrap_or(true);
    let priority = body["priority"].as_i64().unwrap_or(0) as i32;

    let rule = state.configurator_engine.create_rule(
        org_id, model_id, rule_code, name, description,
        rule_type, source_feature_id, source_option_id,
        target_feature_id, target_option_id,
        condition_expression, severity, is_active, priority, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create rule error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap())))
}

pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    Path(model_id): Path<Uuid>,
    Query(params): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rules = state.configurator_engine.list_rules(model_id, params.rule_type.as_deref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": rules })))
}

pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.configurator_engine.delete_rule(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Instance Handlers
// ============================================================================

pub async fn create_instance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let instance_number = body["instanceNumber"].as_str().unwrap_or("");
    let model_id = body["modelId"].as_str().and_then(|s| Uuid::parse_str(s).ok()).ok_or(StatusCode::BAD_REQUEST)?;
    let name = body["name"].as_str();
    let description = body["description"].as_str();
    let selections = body.get("selections").cloned().unwrap_or(serde_json::json!({}));
    let base_price = body["basePrice"].as_f64().unwrap_or(0.0);
    let currency_code = body["currencyCode"].as_str().unwrap_or("USD");
    let effective_date = body["effectiveDate"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let instance = state.configurator_engine.create_instance(
        org_id, instance_number, model_id, name, description,
        selections, base_price, currency_code, effective_date,
        Some(user_id), Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create config instance error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(instance).unwrap())))
}

pub async fn list_instances(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListInstancesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let instances = state.configurator_engine.list_instances(
        org_id, params.status.as_deref(), params.model_id.as_ref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": instances })))
}

pub async fn get_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance = state.configurator_engine.get_instance(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match instance {
        Some(i) => Ok(Json(serde_json::to_value(i).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn submit_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance = state.configurator_engine.submit_instance(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(instance).unwrap()))
}

pub async fn approve_instance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let instance = state.configurator_engine.approve_instance(id, Some(user_id)).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(instance).unwrap()))
}

pub async fn cancel_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let instance = state.configurator_engine.cancel_instance(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(instance).unwrap()))
}

pub async fn delete_instance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(instance_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.configurator_engine.delete_instance(org_id, &instance_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.configurator_engine.get_dashboard(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
