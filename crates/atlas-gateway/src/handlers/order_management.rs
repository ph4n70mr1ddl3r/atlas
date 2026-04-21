//! Order Management API Handlers
//!
//! Oracle Fusion Cloud SCM: Order Management
//!
//! Endpoints for managing sales orders, order lines, fulfillment,
//! order holds, shipments, and the order management dashboard.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};

use crate::AppState;
use crate::handlers::auth::Claims;
use atlas_shared::{CreateSalesOrderRequest, AddOrderLineRequest};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub status: Option<String>,
    pub fulfillment_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListShipmentsQuery {
    pub status: Option<String>,
    pub order_id: Option<String>,
}

// ============================================================================
// Sales Order Handlers
// ============================================================================

pub async fn create_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSalesOrderRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.order_management_engine.create_order(org_id, payload).await {
        Ok(order) => Ok((StatusCode::CREATED, Json(serde_json::to_value(order).unwrap()))),
        Err(e) => {
            error!("Failed to create sales order: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(order_number): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.order_management_engine.get_order(org_id, &order_number).await {
        Ok(Some(order)) => Ok(Json(serde_json::to_value(order).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get order {}: {}", order_number, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_order_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.get_order_by_id(id).await {
        Ok(Some(order)) => Ok(Json(serde_json::to_value(order).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get order {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListOrdersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.order_management_engine.list_orders(
        org_id,
        params.status.as_deref(),
        params.fulfillment_status.as_deref(),
    ).await {
        Ok(orders) => Ok(Json(serde_json::to_value(orders).unwrap())),
        Err(e) => {
            error!("Failed to list orders: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn submit_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.submit_order(id).await {
        Ok(order) => Ok(Json(serde_json::to_value(order).unwrap())),
        Err(e) => {
            error!("Failed to submit order {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn confirm_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.confirm_order(id).await {
        Ok(order) => Ok(Json(serde_json::to_value(order).unwrap())),
        Err(e) => {
            error!("Failed to confirm order {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn close_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.close_order(id).await {
        Ok(order) => Ok(Json(serde_json::to_value(order).unwrap())),
        Err(e) => {
            error!("Failed to close order {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub reason: Option<String>,
}

pub async fn cancel_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CancelOrderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.cancel_order(id, payload.reason.as_deref()).await {
        Ok(order) => Ok(Json(serde_json::to_value(order).unwrap())),
        Err(e) => {
            error!("Failed to cancel order {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Order Line Handlers
// ============================================================================

pub async fn add_order_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(mut payload): Json<AddOrderLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    payload.org_id = org_id;

    match state.order_management_engine.add_order_line(payload).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add order line: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_order_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.get_order_line(id).await {
        Ok(Some(line)) => Ok(Json(serde_json::to_value(line).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get order line {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_order_lines(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let order_id = Uuid::parse_str(&order_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.list_order_lines(order_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap())),
        Err(e) => {
            error!("Failed to list order lines for order {}: {}", order_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShipLineRequest {
    pub quantity_shipped: String,
}

pub async fn ship_order_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ShipLineRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.ship_order_line(id, &payload.quantity_shipped).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap())),
        Err(e) => {
            error!("Failed to ship order line {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelLineRequest {
    pub reason: Option<String>,
}

pub async fn cancel_order_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CancelLineRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.cancel_order_line(id, payload.reason.as_deref()).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap())),
        Err(e) => {
            error!("Failed to cancel order line {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Order Hold Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyHoldRequest {
    pub order_id: String,
    pub order_line_id: Option<String>,
    pub hold_type: String,
    pub hold_reason: String,
    pub applied_by_name: Option<String>,
}

pub async fn apply_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ApplyHoldRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let order_id = Uuid::parse_str(&payload.order_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let order_line_id = payload.order_line_id.as_deref()
        .map(Uuid::parse_str).transpose().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.apply_hold(
        org_id, order_id, order_line_id,
        &payload.hold_type, &payload.hold_reason,
        user_id, payload.applied_by_name.as_deref(),
    ).await {
        Ok(hold) => Ok((StatusCode::CREATED, Json(serde_json::to_value(hold).unwrap()))),
        Err(e) => {
            error!("Failed to apply hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_hold(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.list_holds(id, false).await {
        Ok(holds) => Ok(Json(serde_json::to_value(holds).unwrap())),
        Err(e) => {
            error!("Failed to get holds: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListHoldsQuery {
    pub active_only: Option<bool>,
}

pub async fn list_holds(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
    Query(params): Query<ListHoldsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let order_id = Uuid::parse_str(&order_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let active_only = params.active_only.unwrap_or(true);

    match state.order_management_engine.list_holds(order_id, active_only).await {
        Ok(holds) => Ok(Json(serde_json::to_value(holds).unwrap())),
        Err(e) => {
            error!("Failed to list holds for order {}: {}", order_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReleaseHoldRequest {
    pub released_by_name: Option<String>,
}

pub async fn release_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ReleaseHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.order_management_engine.release_hold(id, user_id, payload.released_by_name.as_deref()).await {
        Ok(hold) => Ok(Json(serde_json::to_value(hold).unwrap())),
        Err(e) => {
            error!("Failed to release hold {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Shipment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    pub order_id: String,
    pub order_line_ids: Vec<String>,
    pub warehouse: Option<String>,
    pub carrier: Option<String>,
    pub shipping_method: Option<String>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub shipped_by_name: Option<String>,
}

pub async fn create_shipment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateShipmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let order_id = Uuid::parse_str(&payload.order_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let order_line_ids: Result<Vec<Uuid>, _> = payload.order_line_ids.iter()
        .map(|s| Uuid::parse_str(s)).collect();
    let order_line_ids = order_line_ids.map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.create_shipment(
        org_id, order_id, order_line_ids,
        payload.warehouse.as_deref(),
        payload.carrier.as_deref(),
        payload.shipping_method.as_deref(),
        payload.estimated_delivery_date,
        user_id,
        payload.shipped_by_name.as_deref(),
    ).await {
        Ok(shipment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(shipment).unwrap()))),
        Err(e) => {
            error!("Failed to create shipment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_shipment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.get_shipment(id).await {
        Ok(Some(shipment)) => Ok(Json(serde_json::to_value(shipment).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get shipment {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_shipments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListShipmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let order_id = params.order_id.as_deref()
        .map(Uuid::parse_str).transpose().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.list_shipments(
        org_id, params.status.as_deref(), order_id,
    ).await {
        Ok(shipments) => Ok(Json(serde_json::to_value(shipments).unwrap())),
        Err(e) => {
            error!("Failed to list shipments: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmShipmentRequest {
    pub ship_date: chrono::NaiveDate,
}

pub async fn confirm_shipment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ConfirmShipmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.confirm_shipment(id, payload.ship_date).await {
        Ok(shipment) => Ok(Json(serde_json::to_value(shipment).unwrap())),
        Err(e) => {
            error!("Failed to confirm shipment {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackingRequest {
    pub tracking_number: Option<String>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
}

pub async fn update_tracking(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTrackingRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.update_tracking(
        id, payload.tracking_number.as_deref(), payload.estimated_delivery,
    ).await {
        Ok(shipment) => Ok(Json(serde_json::to_value(shipment).unwrap())),
        Err(e) => {
            error!("Failed to update tracking for shipment {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmDeliveryRequest {
    pub delivery_date: chrono::NaiveDate,
    pub delivery_confirmation: Option<String>,
}

pub async fn confirm_delivery(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ConfirmDeliveryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.order_management_engine.confirm_delivery(
        id, payload.delivery_date, payload.delivery_confirmation.as_deref(),
    ).await {
        Ok(shipment) => Ok(Json(serde_json::to_value(shipment).unwrap())),
        Err(e) => {
            error!("Failed to confirm delivery for shipment {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_order_management_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.order_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => {
            error!("Failed to get order management dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
