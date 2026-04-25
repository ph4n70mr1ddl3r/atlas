//! Shipping Execution Handlers
//!
//! Oracle Fusion SCM: Shipping Execution

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

fn ship_map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }
}

#[derive(Debug, Deserialize)]
pub struct CreateCarrierRequest {
    pub code: String, pub name: String, pub description: Option<String>,
    pub carrier_type: Option<String>, pub tracking_url_template: Option<String>,
    pub contact_name: Option<String>, pub contact_phone: Option<String>, pub contact_email: Option<String>,
}

pub async fn create_carrier(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Json(payload): Json<CreateCarrierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.create_carrier(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.carrier_type.as_deref().unwrap_or("external"),
        payload.tracking_url_template.as_deref(), payload.contact_name.as_deref(),
        payload.contact_phone.as_deref(), payload.contact_email.as_deref(), user_id,
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or_default()))),
        Err(e) => { error!("Failed to create carrier: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

pub async fn list_carriers(State(state): State<Arc<AppState>>, claims: Extension<Claims>) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_carriers(org_id).await {
        Ok(c) => Ok(Json(serde_json::json!({"data": c}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_carrier(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.get_carrier(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_carrier(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(code): Path<String>) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.delete_carrier(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateShippingMethodRequest {
    pub code: String, pub name: String, pub description: Option<String>,
    pub carrier_id: Option<Uuid>, pub transit_time_days: Option<i32>, pub is_express: Option<bool>,
}

pub async fn create_shipping_method(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Json(payload): Json<CreateShippingMethodRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.create_method(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.carrier_id, payload.transit_time_days.unwrap_or(1), payload.is_express.unwrap_or(false), user_id,
    ).await {
        Ok(m) => Ok((StatusCode::CREATED, Json(serde_json::to_value(m).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

pub async fn list_shipping_methods(State(state): State<Arc<AppState>>, claims: Extension<Claims>) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_methods(org_id).await {
        Ok(m) => Ok(Json(serde_json::json!({"data": m}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_shipping_method(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(code): Path<String>) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.delete_method(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    pub shipment_number: String, pub description: Option<String>,
    pub carrier_id: Option<Uuid>, pub carrier_name: Option<String>,
    pub shipping_method_id: Option<Uuid>, pub shipping_method_name: Option<String>,
    pub order_id: Option<Uuid>, pub order_number: Option<String>,
    pub customer_id: Option<Uuid>, pub customer_name: Option<String>,
    pub ship_from_warehouse: Option<String>,
    pub ship_to_name: Option<String>, pub ship_to_address: Option<String>,
    pub ship_to_city: Option<String>, pub ship_to_state: Option<String>,
    pub ship_to_postal_code: Option<String>, pub ship_to_country: Option<String>,
    pub estimated_delivery: Option<chrono::NaiveDate>, pub notes: Option<String>,
}

pub async fn create_shipment(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Json(payload): Json<CreateShipmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.create_shipment(
        org_id, &payload.shipment_number, payload.description.as_deref(),
        payload.carrier_id, payload.carrier_name.as_deref(),
        payload.shipping_method_id, payload.shipping_method_name.as_deref(),
        payload.order_id, payload.order_number.as_deref(),
        payload.customer_id, payload.customer_name.as_deref(),
        payload.ship_from_warehouse.as_deref(),
        payload.ship_to_name.as_deref(), payload.ship_to_address.as_deref(),
        payload.ship_to_city.as_deref(), payload.ship_to_state.as_deref(),
        payload.ship_to_postal_code.as_deref(), payload.ship_to_country.as_deref(),
        payload.estimated_delivery, payload.notes.as_deref(), user_id,
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListShipmentsQuery { pub status: Option<String> }

pub async fn list_shipments(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Query(query): Query<ListShipmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_shipments(org_id, query.status.as_deref()).await {
        Ok(s) => Ok(Json(serde_json::json!({"data": s}))),
        Err(e) => { error!("Error: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

pub async fn get_shipment(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.get_shipment(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn confirm_shipment(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.confirm_shipment(id, user_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShipConfirmRequest { pub tracking_number: Option<String> }

pub async fn ship_confirm(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<ShipConfirmRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let tracking = payload.tracking_number.as_deref();
    match state.shipping_engine.ship_confirm(id, tracking.as_deref(), user_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

pub async fn deliver_shipment(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.deliver(id, user_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, _=>StatusCode::INTERNAL_SERVER_ERROR }) }
    }
}

pub async fn cancel_shipment(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.shipping_engine.cancel_shipment(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(ship_map_err(e)) }
    }
}

pub async fn delete_shipment(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(num): Path<String>) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.delete_shipment(org_id, &num).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT), Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddShipmentLineRequest {
    pub item_code: String, pub item_name: Option<String>, pub item_description: Option<String>,
    pub requested_quantity: String, pub unit_of_measure: Option<String>,
    pub weight: Option<String>, pub weight_unit: Option<String>,
    pub lot_number: Option<String>, pub serial_number: Option<String>,
    pub is_fragile: Option<bool>, pub is_hazardous: Option<bool>,
    pub notes: Option<String>, pub order_line_id: Option<Uuid>,
}

pub async fn add_shipment_line(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(shipment_id): Path<Uuid>, Json(payload): Json<AddShipmentLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.add_shipment_line(
        org_id, shipment_id, &payload.item_code, payload.item_name.as_deref(),
        payload.item_description.as_deref(), &payload.requested_quantity,
        payload.unit_of_measure.as_deref(), payload.weight.as_deref(), payload.weight_unit.as_deref(),
        payload.lot_number.as_deref(), payload.serial_number.as_deref(),
        payload.is_fragile.unwrap_or(false), payload.is_hazardous.unwrap_or(false),
        payload.notes.as_deref(), payload.order_line_id,
    ).await {
        Ok(l) => Ok((StatusCode::CREATED, Json(serde_json::to_value(l).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(ship_map_err(e)) }
    }
}

pub async fn list_shipment_lines(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(shipment_id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_shipment_lines(shipment_id).await {
        Ok(l) => Ok(Json(serde_json::json!({"data": l}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_shipment_line(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    match state.shipping_engine.delete_shipment_line(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT), Err(e) => { error!("Error: {}", e); Err(ship_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateShippedQtyRequest { pub shipped_quantity: String }

pub async fn update_shipped_quantity(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<UpdateShippedQtyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.shipping_engine.update_line_shipped_quantity(id, &payload.shipped_quantity).await {
        Ok(l) => Ok(Json(serde_json::to_value(l).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(ship_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePackingSlipRequest {
    pub packing_slip_number: String, pub package_number: Option<i32>,
    pub package_type: Option<String>, pub weight: Option<String>, pub weight_unit: Option<String>,
    pub dimensions_length: Option<String>, pub dimensions_width: Option<String>,
    pub dimensions_height: Option<String>, pub dimensions_unit: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_packing_slip(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(shipment_id): Path<Uuid>, Json(payload): Json<CreatePackingSlipRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.shipping_engine.create_packing_slip(
        org_id, shipment_id, &payload.packing_slip_number, payload.package_number.unwrap_or(1),
        payload.package_type.as_deref(), payload.weight.as_deref(), payload.weight_unit.as_deref(),
        payload.dimensions_length.as_deref(), payload.dimensions_width.as_deref(),
        payload.dimensions_height.as_deref(), payload.dimensions_unit.as_deref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(ps) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ps).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(ship_map_err(e)) }
    }
}

pub async fn list_packing_slips(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(shipment_id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_packing_slips(shipment_id).await {
        Ok(ps) => Ok(Json(serde_json::json!({"data": ps}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_packing_slip(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    match state.shipping_engine.delete_packing_slip(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT), Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddPackingSlipLineRequest {
    pub shipment_line_id: Uuid, pub item_code: String, pub item_name: Option<String>,
    pub packed_quantity: String, pub notes: Option<String>,
}

pub async fn add_packing_slip_line(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(packing_slip_id): Path<Uuid>, Json(payload): Json<AddPackingSlipLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.add_packing_slip_line(
        org_id, packing_slip_id, payload.shipment_line_id,
        &payload.item_code, payload.item_name.as_deref(), &payload.packed_quantity, payload.notes.as_deref(),
    ).await {
        Ok(l) => Ok((StatusCode::CREATED, Json(serde_json::to_value(l).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(ship_map_err(e)) }
    }
}

pub async fn list_packing_slip_lines(State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(packing_slip_id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.list_packing_slip_lines(packing_slip_id).await {
        Ok(l) => Ok(Json(serde_json::json!({"data": l}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_packing_slip_line(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    match state.shipping_engine.delete_packing_slip_line(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT), Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_shipping_dashboard(State(state): State<Arc<AppState>>, claims: Extension<Claims>) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.shipping_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
