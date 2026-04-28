//! Transportation Management HTTP Handlers
//!
//! Oracle Fusion Cloud: SCM > Transportation Management endpoints
//! Provides: carrier management, carrier services, transport lanes, shipments,
//! shipment stops/lines, tracking events, freight rates, and dashboard analytics.

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
pub struct ListCarriersQuery {
    pub status: Option<String>,
    pub carrier_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListLanesQuery {
    pub status: Option<String>,
    pub lane_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListShipmentsQuery {
    pub status: Option<String>,
    pub shipment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFreightRatesQuery {
    pub carrier_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCarrierServicesQuery {
    pub active_only: Option<bool>,
}

// ============================================================================
// Carrier Handlers
// ============================================================================

pub async fn create_carrier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let carrier = state.transportation_engine.create_carrier(
        org_id,
        body["carrierCode"].as_str().unwrap_or(""),
        body["name"].as_str().unwrap_or(""),
        body["description"].as_str(),
        body["carrierType"].as_str().unwrap_or("parcel"),
        body["scacCode"].as_str(),
        body["dotNumber"].as_str(),
        body["mcNumber"].as_str(),
        body["taxId"].as_str(),
        body["contactName"].as_str(),
        body["contactEmail"].as_str(),
        body["contactPhone"].as_str(),
        body["addressLine1"].as_str(),
        body["addressLine2"].as_str(),
        body["city"].as_str(),
        body["state"].as_str(),
        body["postalCode"].as_str(),
        body["country"].as_str().unwrap_or("USA"),
        body["currencyCode"].as_str().unwrap_or("USD"),
        body["paymentTerms"].as_str().unwrap_or("net_30"),
        body["insurancePolicyNumber"].as_str(),
        body["insuranceExpiryDate"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["defaultServiceLevel"].as_str().unwrap_or("standard"),
        body.get("capabilities").cloned(),
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create carrier error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(carrier).unwrap())))
}

pub async fn list_carriers(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCarriersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let carriers = state.transportation_engine.list_carriers(
        org_id, params.status.as_deref(), params.carrier_type.as_deref(),
    ).await.map_err(|e| {
        tracing::error!("List carriers error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(serde_json::json!({ "data": carriers })))
}

pub async fn get_carrier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let carrier = state.transportation_engine.get_carrier(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match carrier {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn suspend_carrier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let carrier = state.transportation_engine.suspend_carrier(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(carrier).unwrap()))
}

pub async fn reactivate_carrier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let carrier = state.transportation_engine.reactivate_carrier(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(carrier).unwrap()))
}

pub async fn blacklist_carrier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let carrier = state.transportation_engine.blacklist_carrier(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(carrier).unwrap()))
}

pub async fn update_carrier_performance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rating = body["performanceRating"].as_f64().unwrap_or(0.0);
    let on_time_pct = body["onTimeDeliveryPct"].as_f64().unwrap_or(0.0);
    let claims = body["claimsRatio"].as_f64().unwrap_or(0.0);

    let carrier = state.transportation_engine.update_carrier_performance(
        id, rating, on_time_pct, claims,
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(carrier).unwrap()))
}

pub async fn delete_carrier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.transportation_engine.delete_carrier(org_id, &code).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Carrier Service Handlers
// ============================================================================

pub async fn create_carrier_service(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(carrier_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let service = state.transportation_engine.create_carrier_service(
        org_id,
        carrier_id,
        body["serviceCode"].as_str().unwrap_or(""),
        body["name"].as_str().unwrap_or(""),
        body["description"].as_str(),
        body["serviceLevel"].as_str().unwrap_or("standard"),
        body["transitDaysMin"].as_i64().unwrap_or(1) as i32,
        body["transitDaysMax"].as_i64().unwrap_or(5) as i32,
        body["maxWeightKg"].as_f64(),
        body.get("maxDimensions").cloned(),
        body["cutoffTime"].as_str().and_then(|t| chrono::NaiveTime::parse_from_str(t, "%H:%M").ok()),
        body["operatesOnWeekends"].as_bool().unwrap_or(false),
        body["isInternational"].as_bool().unwrap_or(false),
        body["ratePerKg"].as_f64().unwrap_or(0.0),
        body["minimumCharge"].as_f64().unwrap_or(0.0),
        body["fuelSurchargePct"].as_f64().unwrap_or(0.0),
    ).await.map_err(|e| {
        tracing::error!("Create carrier service error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(service).unwrap())))
}

pub async fn list_carrier_services(
    State(state): State<Arc<AppState>>,
    Path(carrier_id): Path<Uuid>,
    Query(params): Query<ListCarrierServicesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let active_only = params.active_only.unwrap_or(false);
    let services = state.transportation_engine.list_carrier_services(
        carrier_id, active_only,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": services })))
}

pub async fn toggle_carrier_service(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let is_active = body["isActive"].as_bool().unwrap_or(true);
    let service = state.transportation_engine.toggle_carrier_service(id, is_active).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(service).unwrap()))
}

pub async fn delete_carrier_service(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.transportation_engine.delete_carrier_service(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Transport Lane Handlers
// ============================================================================

pub async fn create_lane(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lane = state.transportation_engine.create_lane(
        org_id,
        body["laneCode"].as_str().unwrap_or(""),
        body["name"].as_str().unwrap_or(""),
        body["description"].as_str(),
        body["originLocationId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["originLocationName"].as_str(),
        body["originCity"].as_str(),
        body["originState"].as_str(),
        body["originCountry"].as_str().unwrap_or("USA"),
        body["originPostalCode"].as_str(),
        body["destinationLocationId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["destinationLocationName"].as_str(),
        body["destinationCity"].as_str(),
        body["destinationState"].as_str(),
        body["destinationCountry"].as_str().unwrap_or("USA"),
        body["destinationPostalCode"].as_str(),
        body["distanceKm"].as_f64(),
        body["estimatedTransitHours"].as_f64(),
        body["laneType"].as_str().unwrap_or("domestic"),
        body["preferredCarrierId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["preferredServiceId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["effectiveFrom"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["effectiveTo"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create lane error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(lane).unwrap())))
}

pub async fn list_lanes(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListLanesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lanes = state.transportation_engine.list_lanes(
        org_id, params.status.as_deref(), params.lane_type.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": lanes })))
}

pub async fn get_lane(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lane = state.transportation_engine.get_lane(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match lane {
        Some(l) => Ok(Json(serde_json::to_value(l).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn deactivate_lane(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lane = state.transportation_engine.deactivate_lane(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(lane).unwrap()))
}

pub async fn delete_lane(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.transportation_engine.delete_lane(org_id, &code).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Shipment Handlers
// ============================================================================

pub async fn create_shipment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let shipment = state.transportation_engine.create_shipment(
        org_id,
        body["shipmentNumber"].as_str().unwrap_or(""),
        body["name"].as_str(),
        body["description"].as_str(),
        body["shipmentType"].as_str().unwrap_or("outbound"),
        body["priority"].as_str().unwrap_or("normal"),
        body["carrierId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["carrierServiceId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["laneId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["originLocationId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["originLocationName"].as_str(),
        body.get("originAddress").cloned().unwrap_or(serde_json::json!({})),
        body["destinationLocationId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["destinationLocationName"].as_str(),
        body.get("destinationAddress").cloned().unwrap_or(serde_json::json!({})),
        body["plannedShipDate"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["plannedDeliveryDate"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["currencyCode"].as_str().unwrap_or("USD"),
        body["salesOrderId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["salesOrderNumber"].as_str(),
        body["purchaseOrderId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["purchaseOrderNumber"].as_str(),
        body["transferOrderId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["specialInstructions"].as_str(),
        body["declaredValue"].as_f64(),
        body["insuranceRequired"].as_bool().unwrap_or(false),
        body["signatureRequired"].as_bool().unwrap_or(false),
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create shipment error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(shipment).unwrap())))
}

pub async fn list_shipments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListShipmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let shipments = state.transportation_engine.list_shipments(
        org_id, params.status.as_deref(), params.shipment_type.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": shipments })))
}

pub async fn get_shipment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.get_shipment(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match shipment {
        Some(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn book_shipment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let shipment = state.transportation_engine.book_shipment(id, Some(user_id)).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn confirm_pickup(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.confirm_pickup(
        id,
        body["trackingNumber"].as_str(),
        body["proNumber"].as_str(),
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn start_transit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.start_transit(
        id,
        body["driverName"].as_str(),
        body["vehicleId"].as_str(),
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn arrive_at_destination(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.arrive_at_destination(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn confirm_delivery(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let shipment = state.transportation_engine.confirm_delivery(id, Some(user_id)).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn cancel_shipment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.cancel_shipment(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn mark_exception(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.mark_exception(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn assign_carrier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let carrier_id = body["carrierId"].as_str().and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let carrier_service_id = body["carrierServiceId"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    let shipment = state.transportation_engine.assign_carrier(
        id, carrier_id, carrier_service_id,
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn update_tracking(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.update_tracking(
        id,
        body["trackingNumber"].as_str(),
        body["trackingUrl"].as_str(),
        body["proNumber"].as_str(),
        body["billOfLading"].as_str(),
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

pub async fn delete_shipment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.transportation_engine.delete_shipment(org_id, &number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Shipment Stop Handlers
// ============================================================================

pub async fn add_stop(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(shipment_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stop = state.transportation_engine.add_stop(
        org_id,
        shipment_id,
        body["stopNumber"].as_i64().unwrap_or(1) as i32,
        body["stopType"].as_str().unwrap_or("pickup"),
        body["locationId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["locationName"].as_str(),
        body.get("address").cloned().unwrap_or(serde_json::json!({})),
        body["plannedArrival"].as_str().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.to_utc())),
        body["plannedDeparture"].as_str().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.to_utc())),
        body["contactName"].as_str(),
        body["contactPhone"].as_str(),
        body["specialInstructions"].as_str(),
    ).await.map_err(|e| {
        tracing::error!("Add stop error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(stop).unwrap())))
}

pub async fn update_stop_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stop = state.transportation_engine.update_stop_status(
        id,
        body["status"].as_str().unwrap_or("pending"),
        body["actualArrival"].as_str().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.to_utc())),
        body["actualDeparture"].as_str().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.to_utc())),
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(stop).unwrap()))
}

pub async fn list_stops(
    State(state): State<Arc<AppState>>,
    Path(shipment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stops = state.transportation_engine.list_stops(shipment_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": stops })))
}

// ============================================================================
// Shipment Line Handlers
// ============================================================================

pub async fn add_shipment_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(shipment_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let line = state.transportation_engine.add_shipment_line(
        org_id,
        shipment_id,
        body["lineNumber"].as_i64().unwrap_or(1) as i32,
        body["itemId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["itemNumber"].as_str(),
        body["itemDescription"].as_str(),
        body["quantity"].as_i64().unwrap_or(0) as i32,
        body["unitOfMeasure"].as_str().unwrap_or("EA"),
        body["weightKg"].as_f64().unwrap_or(0.0),
        body["volumeCbm"].as_f64().unwrap_or(0.0),
        body["sourceLineId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["sourceLineType"].as_str(),
        body["stopId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
    ).await.map_err(|e| {
        tracing::error!("Add shipment line error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap())))
}

pub async fn list_shipment_lines(
    State(state): State<Arc<AppState>>,
    Path(shipment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state.transportation_engine.list_shipment_lines(shipment_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": lines })))
}

pub async fn recalculate_shipment_totals(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let shipment = state.transportation_engine.recalculate_shipment_totals(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(shipment).unwrap()))
}

// ============================================================================
// Tracking Event Handlers
// ============================================================================

pub async fn add_tracking_event(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(shipment_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = state.transportation_engine.add_tracking_event(
        org_id,
        shipment_id,
        body["eventType"].as_str().unwrap_or("in_transit"),
        body["locationDescription"].as_str(),
        body["city"].as_str(),
        body["state"].as_str(),
        body["country"].as_str(),
        body["latitude"].as_f64(),
        body["longitude"].as_f64(),
        body["description"].as_str(),
        body["carrierEventCode"].as_str(),
        body["carrierEventDescription"].as_str(),
        body["updatedBy"].as_str(),
    ).await.map_err(|e| {
        tracing::error!("Add tracking event error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(event).unwrap())))
}

pub async fn list_tracking_events(
    State(state): State<Arc<AppState>>,
    Path(shipment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let events = state.transportation_engine.list_tracking_events(shipment_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": events })))
}

// ============================================================================
// Freight Rate Handlers
// ============================================================================

pub async fn create_freight_rate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let carrier_id = body["carrierId"].as_str().and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let rate = state.transportation_engine.create_freight_rate(
        org_id,
        body["rateCode"].as_str().unwrap_or(""),
        body["name"].as_str().unwrap_or(""),
        body["description"].as_str(),
        carrier_id,
        body["carrierServiceId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["laneId"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        body["rateType"].as_str().unwrap_or("per_kg"),
        body["rateAmount"].as_f64().unwrap_or(0.0),
        body["minimumCharge"].as_f64().unwrap_or(0.0),
        body["currencyCode"].as_str().unwrap_or("USD"),
        body["fuelSurchargePct"].as_f64().unwrap_or(0.0),
        body.get("accessorialRates").cloned(),
        body["effectiveFrom"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .ok_or(StatusCode::BAD_REQUEST)?,
        body["effectiveTo"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["isContractRate"].as_bool().unwrap_or(false),
        body["contractNumber"].as_str(),
        body["volumeThresholdMin"].as_f64(),
        body["volumeThresholdMax"].as_f64(),
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create freight rate error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rate).unwrap())))
}

pub async fn list_freight_rates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListFreightRatesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rates = state.transportation_engine.list_freight_rates(
        org_id, params.carrier_id.as_ref(), params.status.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": rates })))
}

pub async fn expire_freight_rate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rate = state.transportation_engine.expire_freight_rate(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(rate).unwrap()))
}

pub async fn delete_freight_rate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.transportation_engine.delete_freight_rate(org_id, &code).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_transportation_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.transportation_engine.get_dashboard(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
