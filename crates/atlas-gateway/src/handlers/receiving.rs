//! Receiving Management Handlers
//!
//! Oracle Fusion SCM: Receiving

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

fn rcv_map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }
}

// ============================================================================
// Location Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_type: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

pub async fn create_location(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateLocationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.create_location(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.location_type.as_deref().unwrap_or("warehouse"),
        payload.address.as_deref(), payload.city.as_deref(), payload.state.as_deref(),
        payload.country.as_deref(), payload.postal_code.as_deref(), user_id,
    ).await {
        Ok(l) => Ok((StatusCode::CREATED, Json(serde_json::to_value(l).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn list_locations(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_locations(org_id).await {
        Ok(l) => Ok(Json(serde_json::json!({"data": l}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_location(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.delete_location(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Receipt Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub receipt_number: String,
    pub receipt_type: Option<String>,
    pub receipt_source: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub supplier_number: Option<String>,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub receiving_location_id: Option<Uuid>,
    pub receiving_location_code: Option<String>,
    pub receiving_date: chrono::NaiveDate,
    pub packing_slip_number: Option<String>,
    pub bill_of_lading: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub waybill_number: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_receipt(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.create_receipt(
        org_id, &payload.receipt_number,
        payload.receipt_type.as_deref().unwrap_or("standard"),
        payload.receipt_source.as_deref().unwrap_or("purchase_order"),
        payload.supplier_id, payload.supplier_name.as_deref(), payload.supplier_number.as_deref(),
        payload.purchase_order_id, payload.purchase_order_number.as_deref(),
        payload.receiving_location_id, payload.receiving_location_code.as_deref(),
        payload.receiving_date, payload.packing_slip_number.as_deref(),
        payload.bill_of_lading.as_deref(), payload.carrier.as_deref(),
        payload.tracking_number.as_deref(), payload.waybill_number.as_deref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn get_receipt(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.get_receipt(id).await {
        Ok(Some(r)) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReceiptsParams {
    status: Option<String>,
    supplier_id: Option<Uuid>,
}

pub async fn list_receipts(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListReceiptsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_receipts(org_id, params.status.as_deref(), params.supplier_id).await {
        Ok(r) => Ok(Json(serde_json::json!({"data": r}))),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn confirm_receipt(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.confirm_receipt(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn close_receipt(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.close_receipt(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn cancel_receipt(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.cancel_receipt(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

// ============================================================================
// Receipt Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddReceiptLineRequest {
    pub purchase_order_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub ordered_qty: Option<String>,
    pub ordered_uom: Option<String>,
    pub received_qty: Option<String>,
    pub received_uom: Option<String>,
    pub lot_number: Option<String>,
    pub serial_numbers: Option<serde_json::Value>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub manufacture_date: Option<chrono::NaiveDate>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

pub async fn add_receipt_line(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
    Json(payload): Json<AddReceiptLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.add_receipt_line(
        org_id, receipt_id,
        payload.purchase_order_line_id, payload.item_id,
        payload.item_code.as_deref(), payload.item_description.as_deref(),
        payload.ordered_qty.as_deref().unwrap_or("0"),
        payload.ordered_uom.as_deref(),
        payload.received_qty.as_deref().unwrap_or("0"),
        payload.received_uom.as_deref(),
        payload.lot_number.as_deref(),
        payload.serial_numbers.as_ref().cloned().unwrap_or(serde_json::json!([])),
        payload.expiration_date, payload.manufacture_date,
        payload.unit_price.as_deref(), payload.currency.as_deref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(l) => Ok((StatusCode::CREATED, Json(serde_json::to_value(l).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn list_receipt_lines(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_receipt_lines(receipt_id).await {
        Ok(l) => Ok(Json(serde_json::json!({"data": l}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Inspection Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateInspectionRequest {
    pub receipt_line_id: Uuid,
    pub inspection_template: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: chrono::NaiveDate,
    pub sample_size: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_inspection(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
    Json(payload): Json<CreateInspectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.create_inspection(
        org_id, receipt_id, payload.receipt_line_id,
        payload.inspection_template.as_deref(), payload.inspector_id,
        payload.inspector_name.as_deref(), payload.inspection_date,
        payload.sample_size.as_deref(), payload.notes.as_deref(), user_id,
    ).await {
        Ok(i) => Ok((StatusCode::CREATED, Json(serde_json::to_value(i).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn list_inspections(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_inspections(org_id, Some(receipt_id)).await {
        Ok(i) => Ok(Json(serde_json::json!({"data": i}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteInspectionRequest {
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub disposition: String,
    pub quality_score: Option<String>,
    pub rejection_reason: Option<String>,
    pub notes: Option<String>,
}

pub async fn complete_inspection(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<CompleteInspectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.complete_inspection(
        id, &payload.quantity_inspected, &payload.quantity_accepted,
        &payload.quantity_rejected, &payload.disposition,
        payload.quality_score.as_deref(), payload.rejection_reason.as_deref(),
        payload.notes.as_deref(),
    ).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

// ============================================================================
// Inspection Detail Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddInspectionDetailRequest {
    pub check_name: String,
    pub check_type: Option<String>,
    pub specification: Option<String>,
    pub result: Option<String>,
    pub measured_value: Option<String>,
    pub expected_value: Option<String>,
    pub notes: Option<String>,
}

pub async fn add_inspection_detail(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(inspection_id): Path<Uuid>,
    Json(payload): Json<AddInspectionDetailRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.add_inspection_detail(
        org_id, inspection_id, &payload.check_name,
        payload.check_type.as_deref().unwrap_or("visual"),
        payload.specification.as_deref(),
        payload.result.as_deref().unwrap_or("pass"),
        payload.measured_value.as_deref(), payload.expected_value.as_deref(),
        payload.notes.as_deref(),
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn list_inspection_details(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(inspection_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_inspection_details(inspection_id).await {
        Ok(d) => Ok(Json(serde_json::json!({"data": d}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Delivery Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDeliveryRequest {
    pub receipt_line_id: Uuid,
    pub subinventory: Option<String>,
    pub locator: Option<String>,
    pub quantity_delivered: String,
    pub uom: Option<String>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub delivered_by_name: Option<String>,
    pub destination_type: Option<String>,
    pub account_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_delivery(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
    Json(payload): Json<CreateDeliveryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.create_delivery(
        org_id, receipt_id, payload.receipt_line_id,
        payload.subinventory.as_deref(), payload.locator.as_deref(),
        &payload.quantity_delivered, payload.uom.as_deref(),
        payload.lot_number.as_deref(), payload.serial_number.as_deref(),
        user_id, payload.delivered_by_name.as_deref(),
        payload.destination_type.as_deref().unwrap_or("inventory"),
        payload.account_code.as_deref(), payload.notes.as_deref(), user_id,
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn list_deliveries(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_deliveries(org_id, Some(receipt_id)).await {
        Ok(d) => Ok(Json(serde_json::json!({"data": d}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Return Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReturnRequest {
    pub receipt_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub return_type: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_returned: String,
    pub uom: Option<String>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub return_reason: Option<String>,
    pub return_date: chrono::NaiveDate,
}

pub async fn create_return(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateReturnRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.receiving_engine.create_return(
        org_id, payload.receipt_id, payload.receipt_line_id,
        payload.supplier_id, payload.supplier_name.as_deref(),
        payload.return_type.as_deref().unwrap_or("reject"),
        payload.item_id, payload.item_code.as_deref(), payload.item_description.as_deref(),
        &payload.quantity_returned, payload.uom.as_deref(),
        payload.unit_price.as_deref(), payload.currency.as_deref(),
        payload.return_reason.as_deref(), payload.return_date, user_id,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rcv_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReturnsParams {
    status: Option<String>,
}

pub async fn list_returns(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListReturnsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.list_returns(org_id, params.status.as_deref()).await {
        Ok(r) => Ok(Json(serde_json::json!({"data": r}))),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShipReturnRequest {
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
}

pub async fn submit_return(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.submit_return(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn ship_return(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<ShipReturnRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.ship_return(id, payload.carrier.as_deref(), payload.tracking_number.as_deref()).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn credit_return(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.credit_return(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

pub async fn cancel_return(
    State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receiving_engine.cancel_return(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rcv_map_err(e)) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_receiving_dashboard(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receiving_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
