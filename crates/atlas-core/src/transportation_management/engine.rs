//! Transportation Management Engine
//!
//! Manages carriers, services, transport lanes, shipments, stops, lines,
//! tracking events, freight rates, cost calculation, and delivery analytics.
//!
//! Oracle Fusion Cloud equivalent: SCM > Transportation Management

use atlas_shared::{
    Carrier, CarrierService, TransportLane, TransportShipment, TransportShipmentStop,
    TransportShipmentLine, TransportShipmentTrackingEvent, FreightRate, TransportationDashboard,
    AtlasError, AtlasResult,
};
use super::TransportationManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_CARRIER_TYPES: &[&str] = &[
    "parcel", "ltl", "ftl", "air", "ocean", "rail", "multimodal",
];

const VALID_CARRIER_STATUSES: &[&str] = &[
    "active", "inactive", "suspended", "blacklisted",
];

const VALID_SERVICE_LEVELS: &[&str] = &[
    "standard", "express", "economy", "premium", "same_day",
];

const VALID_LANE_TYPES: &[&str] = &[
    "domestic", "international", "intercompany",
];

const VALID_LANE_STATUSES: &[&str] = &[
    "active", "inactive", "seasonal",
];

const VALID_SHIPMENT_STATUSES: &[&str] = &[
    "draft", "booked", "picked_up", "in_transit",
    "at_destination", "delivered", "cancelled", "exception",
];

const VALID_SHIPMENT_TYPES: &[&str] = &[
    "outbound", "inbound", "transfer", "returns",
];

const VALID_PRIORITIES: &[&str] = &[
    "low", "normal", "high", "critical",
];

const VALID_STOP_TYPES: &[&str] = &[
    "pickup", "delivery", "transfer",
];

const VALID_STOP_STATUSES: &[&str] = &[
    "pending", "arrived", "departed", "skipped", "failed",
];

const VALID_TRACKING_EVENT_TYPES: &[&str] = &[
    "picked_up", "in_transit", "out_for_delivery", "delivered",
    "exception", "delayed", "customs_clearance", "at_hub",
];

const VALID_RATE_TYPES: &[&str] = &[
    "per_kg", "per_unit", "flat", "per_mile", "per_pallet", "zone_based",
];

const VALID_RATE_STATUSES: &[&str] = &[
    "active", "expired", "pending", "superseded",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Transportation Management Engine
pub struct TransportationManagementEngine {
    repository: Arc<dyn TransportationManagementRepository>,
}

impl TransportationManagementEngine {
    pub fn new(repository: Arc<dyn TransportationManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Carriers
    // ========================================================================

    /// Create a new carrier
    #[allow(clippy::too_many_arguments)]
    pub async fn create_carrier(
        &self,
        org_id: Uuid,
        carrier_code: &str,
        name: &str,
        description: Option<&str>,
        carrier_type: &str,
        scac_code: Option<&str>,
        dot_number: Option<&str>,
        mc_number: Option<&str>,
        tax_id: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        address_line1: Option<&str>,
        address_line2: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        postal_code: Option<&str>,
        country: &str,
        currency_code: &str,
        payment_terms: &str,
        insurance_policy_number: Option<&str>,
        insurance_expiry_date: Option<chrono::NaiveDate>,
        default_service_level: &str,
        capabilities: Option<serde_json::Value>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Carrier> {
        if carrier_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Carrier code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Carrier name is required".to_string()));
        }
        validate_enum("carrier_type", carrier_type, VALID_CARRIER_TYPES)?;
        validate_enum("default_service_level", default_service_level, VALID_SERVICE_LEVELS)?;

        if self.repository.get_carrier_by_code(org_id, carrier_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Carrier '{}' already exists", carrier_code
            )));
        }

        info!("Creating carrier '{}' ({}) for org {} [type={}]",
              carrier_code, name, org_id, carrier_type);

        self.repository.create_carrier(
            org_id, carrier_code, name, description,
            carrier_type, "active",
            scac_code, dot_number, mc_number, tax_id,
            contact_name, contact_email, contact_phone,
            address_line1, address_line2, city, state, postal_code, country,
            currency_code, payment_terms,
            insurance_policy_number, insurance_expiry_date,
            default_service_level,
            capabilities.unwrap_or(serde_json::json!([])),
            serde_json::json!({}),
            created_by,
        ).await
    }

    /// Get a carrier by ID
    pub async fn get_carrier(&self, id: Uuid) -> AtlasResult<Option<Carrier>> {
        self.repository.get_carrier(id).await
    }

    /// Get a carrier by code
    pub async fn get_carrier_by_code(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<Option<Carrier>> {
        self.repository.get_carrier_by_code(org_id, carrier_code).await
    }

    /// List carriers with optional filters
    pub async fn list_carriers(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        carrier_type: Option<&str>,
    ) -> AtlasResult<Vec<Carrier>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_CARRIER_STATUSES)?;
        }
        if let Some(t) = carrier_type {
            validate_enum("carrier_type", t, VALID_CARRIER_TYPES)?;
        }
        self.repository.list_carriers(org_id, status, carrier_type).await
    }

    /// Suspend a carrier
    pub async fn suspend_carrier(&self, id: Uuid) -> AtlasResult<Carrier> {
        let carrier = self.repository.get_carrier(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Carrier {} not found", id)))?;

        if carrier.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot suspend carrier in '{}' status. Must be 'active'.", carrier.status)
            ));
        }

        info!("Suspending carrier {} ({})", carrier.carrier_code, carrier.name);
        self.repository.update_carrier_status(id, "suspended").await
    }

    /// Reactivate a carrier
    pub async fn reactivate_carrier(&self, id: Uuid) -> AtlasResult<Carrier> {
        let carrier = self.repository.get_carrier(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Carrier {} not found", id)))?;

        if carrier.status != "inactive" && carrier.status != "suspended" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot reactivate carrier in '{}' status. Must be 'inactive' or 'suspended'.", carrier.status)
            ));
        }

        info!("Reactivating carrier {} ({})", carrier.carrier_code, carrier.name);
        self.repository.update_carrier_status(id, "active").await
    }

    /// Blacklist a carrier
    pub async fn blacklist_carrier(&self, id: Uuid) -> AtlasResult<Carrier> {
        let carrier = self.repository.get_carrier(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Carrier {} not found", id)))?;

        if carrier.status == "blacklisted" {
            return Err(AtlasError::ValidationFailed(
                "Carrier is already blacklisted".to_string()
            ));
        }

        info!("Blacklisting carrier {} ({})", carrier.carrier_code, carrier.name);
        self.repository.update_carrier_status(id, "blacklisted").await
    }

    /// Update carrier performance metrics
    pub async fn update_carrier_performance(
        &self,
        id: Uuid,
        rating: f64,
        on_time_pct: f64,
        claims: f64,
    ) -> AtlasResult<Carrier> {
        if !(0.0..=5.0).contains(&rating) {
            return Err(AtlasError::ValidationFailed(
                "Performance rating must be between 0.0 and 5.0".to_string()
            ));
        }
        if !(0.0..=100.0).contains(&on_time_pct) {
            return Err(AtlasError::ValidationFailed(
                "On-time delivery percentage must be between 0.0 and 100.0".to_string()
            ));
        }
        if !(0.0..=1.0).contains(&claims) {
            return Err(AtlasError::ValidationFailed(
                "Claims ratio must be between 0.0 and 1.0".to_string()
            ));
        }

        info!("Updating carrier {} performance: rating={:.2}, on_time={:.1}%, claims={:.4}",
              id, rating, on_time_pct, claims);
        self.repository.update_carrier_performance(id, rating, on_time_pct, claims).await
    }

    /// Delete a carrier (only if inactive/blacklisted)
    pub async fn delete_carrier(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<()> {
        if let Some(carrier) = self.repository.get_carrier_by_code(org_id, carrier_code).await? {
            if carrier.status == "active" {
                return Err(AtlasError::ValidationFailed(
                    "Cannot delete an active carrier. Suspend or blacklist first.".to_string()
                ));
            }
        }
        info!("Deleting carrier '{}' for org {}", carrier_code, org_id);
        self.repository.delete_carrier(org_id, carrier_code).await
    }

    // ========================================================================
    // Carrier Services
    // ========================================================================

    /// Create a carrier service
    #[allow(clippy::too_many_arguments)]
    pub async fn create_carrier_service(
        &self,
        org_id: Uuid,
        carrier_id: Uuid,
        service_code: &str,
        name: &str,
        description: Option<&str>,
        service_level: &str,
        transit_days_min: i32,
        transit_days_max: i32,
        max_weight_kg: Option<f64>,
        max_dimensions: Option<serde_json::Value>,
        cutoff_time: Option<chrono::NaiveTime>,
        operates_on_weekends: bool,
        is_international: bool,
        rate_per_kg: f64,
        minimum_charge: f64,
        fuel_surcharge_pct: f64,
    ) -> AtlasResult<CarrierService> {
        if service_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Service code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Service name is required".to_string()));
        }
        validate_enum("service_level", service_level, VALID_SERVICE_LEVELS)?;
        if transit_days_min < 0 || transit_days_max < 0 {
            return Err(AtlasError::ValidationFailed(
                "Transit days cannot be negative".to_string()
            ));
        }
        if transit_days_min > transit_days_max {
            return Err(AtlasError::ValidationFailed(
                "Minimum transit days cannot exceed maximum".to_string()
            ));
        }
        if rate_per_kg < 0.0 {
            return Err(AtlasError::ValidationFailed("Rate per kg cannot be negative".to_string()));
        }
        if minimum_charge < 0.0 {
            return Err(AtlasError::ValidationFailed("Minimum charge cannot be negative".to_string()));
        }
        if !(0.0..=100.0).contains(&fuel_surcharge_pct) {
            return Err(AtlasError::ValidationFailed(
                "Fuel surcharge must be between 0.0 and 100.0".to_string()
            ));
        }

        // Verify carrier exists
        let carrier = self.repository.get_carrier(carrier_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Carrier {} not found", carrier_id
            )))?;

        if carrier.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add services to carrier in '{}' status", carrier.status)
            ));
        }

        info!("Creating service '{}' ({}) for carrier {} [level={}]",
              service_code, name, carrier.carrier_code, service_level);

        self.repository.create_carrier_service(
            org_id, carrier_id, service_code, name, description,
            service_level, transit_days_min, transit_days_max,
            max_weight_kg, max_dimensions, cutoff_time,
            operates_on_weekends, is_international,
            rate_per_kg, minimum_charge, fuel_surcharge_pct,
            true, serde_json::json!({}),
        ).await
    }

    /// Get a carrier service by ID
    pub async fn get_carrier_service(&self, id: Uuid) -> AtlasResult<Option<CarrierService>> {
        self.repository.get_carrier_service(id).await
    }

    /// List services for a carrier
    pub async fn list_carrier_services(&self, carrier_id: Uuid, active_only: bool) -> AtlasResult<Vec<CarrierService>> {
        self.repository.list_carrier_services(carrier_id, active_only).await
    }

    /// Toggle carrier service active/inactive
    pub async fn toggle_carrier_service(&self, id: Uuid, is_active: bool) -> AtlasResult<CarrierService> {
        info!("Toggling carrier service {} to active={}", id, is_active);
        self.repository.update_carrier_service_active(id, is_active).await
    }

    /// Delete a carrier service
    pub async fn delete_carrier_service(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting carrier service {}", id);
        self.repository.delete_carrier_service(id).await
    }

    // ========================================================================
    // Transport Lanes
    // ========================================================================

    /// Create a transport lane
    #[allow(clippy::too_many_arguments)]
    pub async fn create_lane(
        &self,
        org_id: Uuid,
        lane_code: &str,
        name: &str,
        description: Option<&str>,
        origin_location_id: Option<Uuid>,
        origin_location_name: Option<&str>,
        origin_city: Option<&str>,
        origin_state: Option<&str>,
        origin_country: &str,
        origin_postal_code: Option<&str>,
        destination_location_id: Option<Uuid>,
        destination_location_name: Option<&str>,
        destination_city: Option<&str>,
        destination_state: Option<&str>,
        destination_country: &str,
        destination_postal_code: Option<&str>,
        distance_km: Option<f64>,
        estimated_transit_hours: Option<f64>,
        lane_type: &str,
        preferred_carrier_id: Option<Uuid>,
        preferred_service_id: Option<Uuid>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransportLane> {
        if lane_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Lane code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Lane name is required".to_string()));
        }
        validate_enum("lane_type", lane_type, VALID_LANE_TYPES)?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from must be before effective to".to_string()
                ));
            }
        }
        if let Some(dist) = distance_km {
            if dist < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Distance cannot be negative".to_string()
                ));
            }
        }
        if let Some(hours) = estimated_transit_hours {
            if hours < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Estimated transit hours cannot be negative".to_string()
                ));
            }
        }

        // Verify preferred carrier exists if specified
        if let Some(pc_id) = preferred_carrier_id {
            self.repository.get_carrier(pc_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Preferred carrier {} not found", pc_id
                )))?;
        }

        if self.repository.get_lane_by_code(org_id, lane_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Lane '{}' already exists", lane_code
            )));
        }

        info!("Creating lane '{}' ({}) [{} → {}, type={}]",
              lane_code, name, origin_country, destination_country, lane_type);

        self.repository.create_lane(
            org_id, lane_code, name, description,
            origin_location_id, origin_location_name,
            origin_city, origin_state, origin_country, origin_postal_code,
            destination_location_id, destination_location_name,
            destination_city, destination_state, destination_country, destination_postal_code,
            distance_km, estimated_transit_hours,
            lane_type, preferred_carrier_id, preferred_service_id,
            "active", effective_from, effective_to,
            serde_json::json!([]), serde_json::json!({}), created_by,
        ).await
    }

    /// Get a lane by ID
    pub async fn get_lane(&self, id: Uuid) -> AtlasResult<Option<TransportLane>> {
        self.repository.get_lane(id).await
    }

    /// Get a lane by code
    pub async fn get_lane_by_code(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<Option<TransportLane>> {
        self.repository.get_lane_by_code(org_id, lane_code).await
    }

    /// List lanes with optional filters
    pub async fn list_lanes(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        lane_type: Option<&str>,
    ) -> AtlasResult<Vec<TransportLane>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_LANE_STATUSES)?;
        }
        if let Some(t) = lane_type {
            validate_enum("lane_type", t, VALID_LANE_TYPES)?;
        }
        self.repository.list_lanes(org_id, status, lane_type).await
    }

    /// Deactivate a lane
    pub async fn deactivate_lane(&self, id: Uuid) -> AtlasResult<TransportLane> {
        let lane = self.repository.get_lane(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Lane {} not found", id)))?;

        if lane.status != "active" && lane.status != "seasonal" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot deactivate lane in '{}' status", lane.status)
            ));
        }

        info!("Deactivating lane {} ({})", lane.lane_code, lane.name);
        self.repository.update_lane_status(id, "inactive").await
    }

    /// Delete a lane
    pub async fn delete_lane(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<()> {
        if let Some(lane) = self.repository.get_lane_by_code(org_id, lane_code).await? {
            if lane.status == "active" {
                return Err(AtlasError::ValidationFailed(
                    "Cannot delete an active lane. Deactivate first.".to_string()
                ));
            }
        }
        info!("Deleting lane '{}' for org {}", lane_code, org_id);
        self.repository.delete_lane(org_id, lane_code).await
    }

    // ========================================================================
    // Shipments
    // ========================================================================

    /// Create a new shipment
    #[allow(clippy::too_many_arguments)]
    pub async fn create_shipment(
        &self,
        org_id: Uuid,
        shipment_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        shipment_type: &str,
        priority: &str,
        carrier_id: Option<Uuid>,
        carrier_service_id: Option<Uuid>,
        lane_id: Option<Uuid>,
        origin_location_id: Option<Uuid>,
        origin_location_name: Option<&str>,
        origin_address: serde_json::Value,
        destination_location_id: Option<Uuid>,
        destination_location_name: Option<&str>,
        destination_address: serde_json::Value,
        planned_ship_date: Option<chrono::NaiveDate>,
        planned_delivery_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        sales_order_id: Option<Uuid>,
        sales_order_number: Option<&str>,
        purchase_order_id: Option<Uuid>,
        purchase_order_number: Option<&str>,
        transfer_order_id: Option<Uuid>,
        special_instructions: Option<&str>,
        declared_value: Option<f64>,
        insurance_required: bool,
        signature_required: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransportShipment> {
        if shipment_number.is_empty() {
            return Err(AtlasError::ValidationFailed("TransportShipment number is required".to_string()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }
        validate_enum("shipment_type", shipment_type, VALID_SHIPMENT_TYPES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;

        if let (Some(ship), Some(delivery)) = (planned_ship_date, planned_delivery_date) {
            if ship > delivery {
                return Err(AtlasError::ValidationFailed(
                    "Planned ship date cannot be after planned delivery date".to_string()
                ));
            }
        }
        if let Some(dv) = declared_value {
            if dv < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Declared value cannot be negative".to_string()
                ));
            }
        }

        // Verify carrier if specified
        let mut carrier_code: Option<String> = None;
        let mut carrier_name: Option<String> = None;
        let mut carrier_service_code: Option<String> = None;
        if let Some(cid) = carrier_id {
            let carrier = self.repository.get_carrier(cid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Carrier {} not found", cid
                )))?;
            if carrier.status != "active" {
                return Err(AtlasError::ValidationFailed(
                    format!("Carrier '{}' is not active (status: {})", carrier.carrier_code, carrier.status)
                ));
            }
            carrier_code = Some(carrier.carrier_code.clone());
            carrier_name = Some(carrier.name.clone());

            if let Some(sid) = carrier_service_id {
                let svc = self.repository.get_carrier_service(sid).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(format!(
                        "Carrier service {} not found", sid
                    )))?;
                if !svc.is_active {
                    return Err(AtlasError::ValidationFailed(
                        format!("Carrier service '{}' is not active", svc.service_code)
                    ));
                }
                carrier_service_code = Some(svc.service_code.clone());
            }
        }

        // Verify lane if specified
        let mut lane_code: Option<String> = None;
        if let Some(lid) = lane_id {
            let lane = self.repository.get_lane(lid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Lane {} not found", lid
                )))?;
            if lane.status != "active" {
                return Err(AtlasError::ValidationFailed(
                    format!("Lane '{}' is not active", lane.lane_code)
                ));
            }
            lane_code = Some(lane.lane_code.clone());
        }

        if self.repository.get_shipment_by_number(org_id, shipment_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "TransportShipment '{}' already exists", shipment_number
            )));
        }

        info!("Creating shipment '{}' [type={}, priority={}, carrier={:?}]",
              shipment_number, shipment_type, priority, carrier_code);

        self.repository.create_shipment(
            org_id, shipment_number, name, description,
            "draft", shipment_type, priority,
            carrier_id,
            carrier_code.as_deref(),
            carrier_name.as_deref(),
            carrier_service_id,
            carrier_service_code.as_deref(),
            lane_id,
            lane_code.as_deref(),
            origin_location_id, origin_location_name, origin_address,
            destination_location_id, destination_location_name, destination_address,
            planned_ship_date, planned_delivery_date,
            None, None, None, None,
            currency_code,
            None, None, None,
            sales_order_id, sales_order_number,
            purchase_order_id, purchase_order_number,
            transfer_order_id,
            special_instructions,
            declared_value, insurance_required, signature_required,
            None, None,
            serde_json::json!({}), created_by,
        ).await
    }

    /// Get a shipment by ID
    pub async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<TransportShipment>> {
        self.repository.get_shipment(id).await
    }

    /// Get a shipment by number
    pub async fn get_shipment_by_number(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<Option<TransportShipment>> {
        self.repository.get_shipment_by_number(org_id, shipment_number).await
    }

    /// List shipments with optional filters
    pub async fn list_shipments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        shipment_type: Option<&str>,
    ) -> AtlasResult<Vec<TransportShipment>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_SHIPMENT_STATUSES)?;
        }
        if let Some(t) = shipment_type {
            validate_enum("shipment_type", t, VALID_SHIPMENT_TYPES)?;
        }
        self.repository.list_shipments(org_id, status, shipment_type).await
    }

    /// Book a shipment (draft → booked)
    pub async fn book_shipment(&self, id: Uuid, booked_by: Option<Uuid>) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot book shipment in '{}' status. Must be 'draft'.", shipment.status)
            ));
        }

        if shipment.carrier_id.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Cannot book shipment without a carrier assigned".to_string()
            ));
        }

        info!("Booking shipment {} by {:?}", shipment.shipment_number, booked_by);
        self.repository.update_shipment_status(id, "booked").await
    }

    /// Confirm pickup (booked → picked_up)
    pub async fn confirm_pickup(&self, id: Uuid, tracking_number: Option<&str>, pro_number: Option<&str>) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "booked" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot confirm pickup for shipment in '{}' status. Must be 'booked'.", shipment.status)
            ));
        }

        info!("Confirming pickup for shipment {}", shipment.shipment_number);

        let mut s = self.repository.update_shipment_status(id, "picked_up").await?;
        if tracking_number.is_some() || pro_number.is_some() {
            s = self.repository.update_shipment_tracking(id, tracking_number, None, pro_number, None).await?;
        }
        self.repository.update_shipment_dates(id, Some(chrono::Utc::now().date_naive()), None).await?;
        Ok(s)
    }

    /// Start transit (picked_up → in_transit)
    pub async fn start_transit(&self, id: Uuid, _driver_name: Option<&str>, _vehicle_id: Option<&str>) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "picked_up" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot start transit for shipment in '{}' status. Must be 'picked_up'.", shipment.status)
            ));
        }

        info!("Starting transit for shipment {}", shipment.shipment_number);
        self.repository.update_shipment_status(id, "in_transit").await
    }

    /// Mark as at destination
    pub async fn arrive_at_destination(&self, id: Uuid) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "in_transit" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot mark arrival for shipment in '{}' status. Must be 'in_transit'.", shipment.status)
            ));
        }

        info!("TransportShipment {} arrived at destination", shipment.shipment_number);
        self.repository.update_shipment_status(id, "at_destination").await
    }

    /// Confirm delivery (at_destination → delivered)
    pub async fn confirm_delivery(&self, id: Uuid, received_by: Option<Uuid>) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "at_destination" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot confirm delivery for shipment in '{}' status. Must be 'at_destination'.", shipment.status)
            ));
        }

        info!("Confirming delivery for shipment {} by {:?}", shipment.shipment_number, received_by);
        let _s = self.repository.update_shipment_status(id, "delivered").await?;
        let s = self.repository.update_shipment_delivery(id, received_by).await?;
        Ok(s)
    }

    /// Cancel a shipment
    pub async fn cancel_shipment(&self, id: Uuid) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status == "delivered" || shipment.status == "cancelled" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot cancel shipment in '{}' status", shipment.status)
            ));
        }

        info!("Cancelling shipment {}", shipment.shipment_number);
        self.repository.update_shipment_status(id, "cancelled").await
    }

    /// Mark shipment as exception
    pub async fn mark_exception(&self, id: Uuid) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status == "delivered" || shipment.status == "cancelled" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot mark exception for shipment in '{}' status", shipment.status)
            ));
        }

        info!("Marking exception for shipment {}", shipment.shipment_number);
        self.repository.update_shipment_status(id, "exception").await
    }

    /// Assign a carrier to a shipment
    pub async fn assign_carrier(
        &self,
        id: Uuid,
        carrier_id: Uuid,
        carrier_service_id: Option<Uuid>,
    ) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;

        if shipment.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only assign carrier to a draft shipment".to_string()
            ));
        }

        let carrier = self.repository.get_carrier(carrier_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Carrier {} not found", carrier_id)))?;

        if carrier.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Carrier '{}' is not active", carrier.carrier_code)
            ));
        }

        let mut svc_code: Option<String> = None;
        if let Some(sid) = carrier_service_id {
            let svc = self.repository.get_carrier_service(sid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Service {} not found", sid)))?;
            svc_code = Some(svc.service_code.clone());
        }

        info!("Assigning carrier {} to shipment {}", carrier.carrier_code, shipment.shipment_number);
        self.repository.update_shipment_carrier(
            id,
            Some(carrier_id),
            Some(&carrier.carrier_code),
            Some(&carrier.name),
            carrier_service_id,
            svc_code.as_deref(),
        ).await
    }

    /// Update shipment tracking info
    pub async fn update_tracking(
        &self,
        id: Uuid,
        tracking_number: Option<&str>,
        tracking_url: Option<&str>,
        pro_number: Option<&str>,
        bill_of_lading: Option<&str>,
    ) -> AtlasResult<TransportShipment> {
        info!("Updating tracking for shipment {}", id);
        self.repository.update_shipment_tracking(id, tracking_number, tracking_url, pro_number, bill_of_lading).await
    }

    /// Delete a shipment (only draft or cancelled)
    pub async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<()> {
        if let Some(shipment) = self.repository.get_shipment_by_number(org_id, shipment_number).await? {
            if shipment.status != "draft" && shipment.status != "cancelled" {
                return Err(AtlasError::ValidationFailed(
                    "Only draft or cancelled shipments can be deleted".to_string()
                ));
            }
        }
        info!("Deleting shipment '{}' for org {}", shipment_number, org_id);
        self.repository.delete_shipment(org_id, shipment_number).await
    }

    // ========================================================================
    // TransportShipment Stops
    // ========================================================================

    /// Add a stop to a shipment
    #[allow(clippy::too_many_arguments)]
    pub async fn add_stop(
        &self,
        org_id: Uuid,
        shipment_id: Uuid,
        stop_number: i32,
        stop_type: &str,
        location_id: Option<Uuid>,
        location_name: Option<&str>,
        address: serde_json::Value,
        planned_arrival: Option<chrono::DateTime<chrono::Utc>>,
        planned_departure: Option<chrono::DateTime<chrono::Utc>>,
        contact_name: Option<&str>,
        contact_phone: Option<&str>,
        special_instructions: Option<&str>,
    ) -> AtlasResult<TransportShipmentStop> {
        validate_enum("stop_type", stop_type, VALID_STOP_TYPES)?;
        if stop_number < 1 {
            return Err(AtlasError::ValidationFailed(
                "Stop number must be at least 1".to_string()
            ));
        }

        // Verify shipment exists and is in editable state
        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "TransportShipment {} not found", shipment_id
            )))?;

        if shipment.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add stops to shipment in '{}' status", shipment.status)
            ));
        }

        info!("Adding stop #{} ({}) to shipment {}", stop_number, stop_type, shipment.shipment_number);

        self.repository.create_stop(
            org_id, shipment_id, stop_number, stop_type,
            location_id, location_name, address,
            planned_arrival, planned_departure,
            contact_name, contact_phone, special_instructions,
            serde_json::json!({}),
        ).await
    }

    /// Update stop status
    pub async fn update_stop_status(
        &self,
        id: Uuid,
        status: &str,
        actual_arrival: Option<chrono::DateTime<chrono::Utc>>,
        actual_departure: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<TransportShipmentStop> {
        validate_enum("stop_status", status, VALID_STOP_STATUSES)?;
        info!("Updating stop {} to status {}", id, status);
        self.repository.update_stop_status(id, status, actual_arrival, actual_departure).await
    }

    /// List stops for a shipment
    pub async fn list_stops(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentStop>> {
        self.repository.list_stops(shipment_id).await
    }

    // ========================================================================
    // TransportShipment Lines
    // ========================================================================

    /// Add a line to a shipment
    #[allow(clippy::too_many_arguments)]
    pub async fn add_shipment_line(
        &self,
        org_id: Uuid,
        shipment_id: Uuid,
        line_number: i32,
        item_id: Option<Uuid>,
        item_number: Option<&str>,
        item_description: Option<&str>,
        quantity: i32,
        unit_of_measure: &str,
        weight_kg: f64,
        volume_cbm: f64,
        source_line_id: Option<Uuid>,
        source_line_type: Option<&str>,
        stop_id: Option<Uuid>,
    ) -> AtlasResult<TransportShipmentLine> {
        if line_number < 1 {
            return Err(AtlasError::ValidationFailed(
                "Line number must be at least 1".to_string()
            ));
        }
        if quantity < 0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity cannot be negative".to_string()
            ));
        }
        if weight_kg < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Weight cannot be negative".to_string()
            ));
        }
        if volume_cbm < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Volume cannot be negative".to_string()
            ));
        }

        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "TransportShipment {} not found", shipment_id
            )))?;

        if shipment.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add lines to shipment in '{}' status", shipment.status)
            ));
        }

        info!("Adding line #{} to shipment {}", line_number, shipment.shipment_number);

        self.repository.create_shipment_line(
            org_id, shipment_id, line_number,
            item_id, item_number, item_description,
            quantity, unit_of_measure,
            weight_kg, volume_cbm,
            None, serde_json::json!([]),
            source_line_id, source_line_type,
            stop_id,
            None, None, None,
            serde_json::json!({}),
        ).await
    }

    /// List lines for a shipment
    pub async fn list_shipment_lines(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentLine>> {
        self.repository.list_shipment_lines(shipment_id).await
    }

    /// Recalculate shipment totals from lines
    pub async fn recalculate_shipment_totals(&self, shipment_id: Uuid) -> AtlasResult<TransportShipment> {
        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("TransportShipment {} not found", shipment_id)))?;

        let lines = self.repository.list_shipment_lines(shipment_id).await?;

        let total_weight: f64 = lines.iter().map(|l| l.weight_kg).sum();
        let total_volume: f64 = lines.iter().map(|l| l.volume_cbm).sum();
        let total_pieces: i32 = lines.iter().map(|l| l.quantity).sum();

        // Calculate freight cost
        let freight_cost = self.calculate_freight_cost(
            shipment.carrier_id,
            shipment.lane_id,
            total_weight,
            total_pieces,
        ).await?;

        let fuel_surcharge = freight_cost * 0.15; // simplified
        let accessorial_charges = 0.0;
        let total_cost = freight_cost + fuel_surcharge + accessorial_charges;

        info!("Recalculating shipment {} totals: weight={:.2}, pieces={}, cost={:.2}",
              shipment.shipment_number, total_weight, total_pieces, total_cost);

        self.repository.update_shipment_totals(
            shipment_id,
            total_weight, total_volume, total_pieces,
            freight_cost, fuel_surcharge, accessorial_charges, total_cost,
        ).await
    }

    // ========================================================================
    // Tracking Events
    // ========================================================================

    /// Add a tracking event
    pub async fn add_tracking_event(
        &self,
        org_id: Uuid,
        shipment_id: Uuid,
        event_type: &str,
        location_description: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        country: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        description: Option<&str>,
        carrier_event_code: Option<&str>,
        carrier_event_description: Option<&str>,
        updated_by: Option<&str>,
    ) -> AtlasResult<TransportShipmentTrackingEvent> {
        validate_enum("event_type", event_type, VALID_TRACKING_EVENT_TYPES)?;

        // Verify shipment exists
        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "TransportShipment {} not found", shipment_id
            )))?;

        info!("Adding tracking event '{}' for shipment {}", event_type, shipment.shipment_number);

        // Auto-transition shipment status based on tracking event
        let _ = self.auto_transition_from_event(shipment_id, event_type, &shipment.status).await;

        self.repository.create_tracking_event(
            org_id, shipment_id, event_type,
            chrono::Utc::now(),
            location_description, city, state, country,
            latitude, longitude, description,
            carrier_event_code, carrier_event_description,
            updated_by,
            serde_json::json!({}),
        ).await
    }

    /// List tracking events for a shipment
    pub async fn list_tracking_events(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentTrackingEvent>> {
        self.repository.list_tracking_events(shipment_id).await
    }

    // ========================================================================
    // Freight Rates
    // ========================================================================

    /// Create a freight rate
    #[allow(clippy::too_many_arguments)]
    pub async fn create_freight_rate(
        &self,
        org_id: Uuid,
        rate_code: &str,
        name: &str,
        description: Option<&str>,
        carrier_id: Uuid,
        carrier_service_id: Option<Uuid>,
        lane_id: Option<Uuid>,
        rate_type: &str,
        rate_amount: f64,
        minimum_charge: f64,
        currency_code: &str,
        fuel_surcharge_pct: f64,
        accessorial_rates: Option<serde_json::Value>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_contract_rate: bool,
        contract_number: Option<&str>,
        volume_threshold_min: Option<f64>,
        volume_threshold_max: Option<f64>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FreightRate> {
        if rate_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rate code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rate name is required".to_string()));
        }
        validate_enum("rate_type", rate_type, VALID_RATE_TYPES)?;
        if rate_amount < 0.0 {
            return Err(AtlasError::ValidationFailed("Rate amount cannot be negative".to_string()));
        }
        if minimum_charge < 0.0 {
            return Err(AtlasError::ValidationFailed("Minimum charge cannot be negative".to_string()));
        }
        if !(0.0..=100.0).contains(&fuel_surcharge_pct) {
            return Err(AtlasError::ValidationFailed(
                "Fuel surcharge must be between 0.0 and 100.0".to_string()
            ));
        }

        if let Some(to) = effective_to {
            if effective_from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from must be before effective to".to_string()
                ));
            }
        }

        // Verify carrier exists
        self.repository.get_carrier(carrier_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Carrier {} not found", carrier_id
            )))?;

        if self.repository.get_freight_rate_by_code(org_id, rate_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Freight rate '{}' already exists", rate_code
            )));
        }

        info!("Creating freight rate '{}' ({}) [type={}, amount={:.4}]",
              rate_code, name, rate_type, rate_amount);

        self.repository.create_freight_rate(
            org_id, rate_code, name, description,
            carrier_id, carrier_service_id, lane_id,
            rate_type, rate_amount, minimum_charge, currency_code,
            fuel_surcharge_pct,
            accessorial_rates.unwrap_or(serde_json::json!({})),
            effective_from, effective_to,
            "active", is_contract_rate, contract_number,
            volume_threshold_min, volume_threshold_max,
            serde_json::json!({}), created_by,
        ).await
    }

    /// List freight rates
    pub async fn list_freight_rates(
        &self,
        org_id: Uuid,
        carrier_id: Option<&Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<FreightRate>> {
        if let Some(s) = status {
            validate_enum("rate_status", s, VALID_RATE_STATUSES)?;
        }
        self.repository.list_freight_rates(org_id, carrier_id, status).await
    }

    /// Expire a freight rate
    pub async fn expire_freight_rate(&self, id: Uuid) -> AtlasResult<FreightRate> {
        let rate = self.repository.get_freight_rate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Freight rate {} not found", id)))?;

        if rate.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot expire rate in '{}' status", rate.status)
            ));
        }

        info!("Expiring freight rate {} ({})", rate.rate_code, rate.name);
        self.repository.update_freight_rate_status(id, "expired").await
    }

    /// Delete a freight rate
    pub async fn delete_freight_rate(&self, org_id: Uuid, rate_code: &str) -> AtlasResult<()> {
        info!("Deleting freight rate '{}' for org {}", rate_code, org_id);
        self.repository.delete_freight_rate(org_id, rate_code).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the transportation management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransportationDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Calculate freight cost for a shipment
    async fn calculate_freight_cost(
        &self,
        _carrier_id: Option<Uuid>,
        _lane_id: Option<Uuid>,
        weight_kg: f64,
        _pieces: i32,
    ) -> AtlasResult<f64> {
        // In production this would query freight_rates and apply
        // the best matching rate. For now, a simple calculation.
        let base_rate_per_kg = 2.50; // default
        let cost = weight_kg * base_rate_per_kg;
        Ok(f64::max(cost, 25.0)) // minimum charge of 25.0
    }

    /// Auto-transition shipment status based on tracking event
    async fn auto_transition_from_event(
        &self,
        shipment_id: Uuid,
        event_type: &str,
        current_status: &str,
    ) -> AtlasResult<()> {
        match event_type {
            "picked_up" if current_status == "booked" => {
                let _ = self.repository.update_shipment_status(shipment_id, "picked_up").await;
            }
            "in_transit" if current_status == "picked_up" => {
                let _ = self.repository.update_shipment_status(shipment_id, "in_transit").await;
            }
            "delivered" if current_status == "in_transit" || current_status == "at_destination" || current_status == "out_for_delivery" => {
                let _ = self.repository.update_shipment_status(shipment_id, "delivered").await;
            }
            "exception" => {
                if current_status != "delivered" && current_status != "cancelled" {
                    let _ = self.repository.update_shipment_status(shipment_id, "exception").await;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_carrier_types() {
        assert!(VALID_CARRIER_TYPES.contains(&"parcel"));
        assert!(VALID_CARRIER_TYPES.contains(&"ltl"));
        assert!(VALID_CARRIER_TYPES.contains(&"ftl"));
        assert!(VALID_CARRIER_TYPES.contains(&"air"));
        assert!(VALID_CARRIER_TYPES.contains(&"ocean"));
        assert!(VALID_CARRIER_TYPES.contains(&"rail"));
        assert!(VALID_CARRIER_TYPES.contains(&"multimodal"));
        assert!(!VALID_CARRIER_TYPES.contains(&"drone"));
    }

    #[test]
    fn test_valid_carrier_statuses() {
        assert!(VALID_CARRIER_STATUSES.contains(&"active"));
        assert!(VALID_CARRIER_STATUSES.contains(&"inactive"));
        assert!(VALID_CARRIER_STATUSES.contains(&"suspended"));
        assert!(VALID_CARRIER_STATUSES.contains(&"blacklisted"));
    }

    #[test]
    fn test_valid_service_levels() {
        assert!(VALID_SERVICE_LEVELS.contains(&"standard"));
        assert!(VALID_SERVICE_LEVELS.contains(&"express"));
        assert!(VALID_SERVICE_LEVELS.contains(&"economy"));
        assert!(VALID_SERVICE_LEVELS.contains(&"premium"));
        assert!(VALID_SERVICE_LEVELS.contains(&"same_day"));
    }

    #[test]
    fn test_valid_lane_types() {
        assert!(VALID_LANE_TYPES.contains(&"domestic"));
        assert!(VALID_LANE_TYPES.contains(&"international"));
        assert!(VALID_LANE_TYPES.contains(&"intercompany"));
    }

    #[test]
    fn test_valid_shipment_statuses() {
        assert!(VALID_SHIPMENT_STATUSES.contains(&"draft"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"booked"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"picked_up"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"in_transit"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"at_destination"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"delivered"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"cancelled"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"exception"));
    }

    #[test]
    fn test_valid_shipment_types() {
        assert!(VALID_SHIPMENT_TYPES.contains(&"outbound"));
        assert!(VALID_SHIPMENT_TYPES.contains(&"inbound"));
        assert!(VALID_SHIPMENT_TYPES.contains(&"transfer"));
        assert!(VALID_SHIPMENT_TYPES.contains(&"returns"));
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"normal"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_tracking_event_types() {
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"picked_up"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"in_transit"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"out_for_delivery"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"delivered"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"exception"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"delayed"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"customs_clearance"));
        assert!(VALID_TRACKING_EVENT_TYPES.contains(&"at_hub"));
    }

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"per_kg"));
        assert!(VALID_RATE_TYPES.contains(&"per_unit"));
        assert!(VALID_RATE_TYPES.contains(&"flat"));
        assert!(VALID_RATE_TYPES.contains(&"per_mile"));
        assert!(VALID_RATE_TYPES.contains(&"per_pallet"));
        assert!(VALID_RATE_TYPES.contains(&"zone_based"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("carrier_type", "parcel", VALID_CARRIER_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("carrier_type", "spaceship", VALID_CARRIER_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("carrier_type"));
                assert!(msg.contains("spaceship"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("carrier_type", "", VALID_CARRIER_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockTransportationManagementRepository;

    fn create_engine() -> TransportationManagementEngine {
        TransportationManagementEngine::new(Arc::new(MockTransportationManagementRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    // --- Carrier Tests ---

    #[tokio::test]
    async fn test_create_carrier_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_carrier(
            test_org_id(), "", "FedEx", None, "parcel",
            None, None, None, None,
            None, None, None, None, None, None, None, None, "USA",
            "USD", "net_30", None, None, "standard", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_validation_empty_name() {
        let engine = create_engine();
        let result = engine.create_carrier(
            test_org_id(), "FEDEX", "", None, "parcel",
            None, None, None, None,
            None, None, None, None, None, None, None, None, "USA",
            "USD", "net_30", None, None, "standard", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("name")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_carrier(
            test_org_id(), "FEDEX", "Federal Express", None, "spaceship",
            None, None, None, None,
            None, None, None, None, None, None, None, None, "USA",
            "USD", "net_30", None, None, "standard", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("carrier_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_validation_bad_service_level() {
        let engine = create_engine();
        let result = engine.create_carrier(
            test_org_id(), "FEDEX", "Federal Express", None, "parcel",
            None, None, None, None,
            None, None, None, None, None, None, None, None, "USA",
            "USD", "net_30", None, None, "ultra_fast", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("service_level")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_success() {
        let engine = create_engine();
        let result = engine.create_carrier(
            test_org_id(), "FEDEX", "Federal Express",
            Some("Global parcel delivery"),
            "parcel",
            Some("FDCC"), None, None, None,
            Some("John Doe"), Some("john@fedex.com"), Some("555-0100"),
            Some("123 Main St"), None, Some("Memphis"), Some("TN"), Some("38118"), "USA",
            "USD", "net_30", None, None, "express",
            Some(serde_json::json!(["hazardous", "temperature_controlled"])),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let carrier = result.unwrap();
        assert_eq!(carrier.carrier_code, "FEDEX");
        assert_eq!(carrier.name, "Federal Express");
        assert_eq!(carrier.carrier_type, "parcel");
        assert_eq!(carrier.status, "active");
    }

    #[tokio::test]
    async fn test_update_carrier_performance_validation_bad_rating() {
        let engine = create_engine();
        let result = engine.update_carrier_performance(Uuid::new_v4(), 6.0, 90.0, 0.01).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("rating")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_update_carrier_performance_validation_bad_on_time() {
        let engine = create_engine();
        let result = engine.update_carrier_performance(Uuid::new_v4(), 4.0, 150.0, 0.01).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("On-time")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_update_carrier_performance_validation_bad_claims() {
        let engine = create_engine();
        let result = engine.update_carrier_performance(Uuid::new_v4(), 4.0, 90.0, 2.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Claims")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_suspend_carrier_not_found() {
        let engine = create_engine();
        let result = engine.suspend_carrier(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    // --- Carrier Service Tests ---

    #[tokio::test]
    async fn test_create_carrier_service_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_carrier_service(
            test_org_id(), Uuid::new_v4(), "", "Ground", None,
            "standard", 1, 5, None, None, None, false, false,
            2.5, 15.0, 12.5,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_service_validation_inverted_transit() {
        let engine = create_engine();
        let result = engine.create_carrier_service(
            test_org_id(), Uuid::new_v4(), "GND", "Ground", None,
            "standard", 5, 1, None, None, None, false, false,
            2.5, 15.0, 12.5,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("transit")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_service_validation_negative_rate() {
        let engine = create_engine();
        let result = engine.create_carrier_service(
            test_org_id(), Uuid::new_v4(), "GND", "Ground", None,
            "standard", 1, 5, None, None, None, false, false,
            -1.0, 15.0, 12.5,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Rate")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_carrier_service_carrier_not_found() {
        let engine = create_engine();
        let result = engine.create_carrier_service(
            test_org_id(), Uuid::new_v4(), "GND", "Ground", None,
            "standard", 1, 5, None, None, None, false, false,
            2.5, 15.0, 12.5,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("Carrier")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    // --- Lane Tests ---

    #[tokio::test]
    async fn test_create_lane_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_lane(
            test_org_id(), "", "NYC-LAX", None,
            None, None, None, None, "USA", None,
            None, None, None, None, "USA", None,
            None, None, "domestic", None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_lane_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_lane(
            test_org_id(), "NYC-LAX", "NYC to LAX", None,
            None, None, None, None, "USA", None,
            None, None, None, None, "USA", None,
            None, None, "intergalactic", None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("lane_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_lane_validation_negative_distance() {
        let engine = create_engine();
        let result = engine.create_lane(
            test_org_id(), "NYC-LAX", "NYC to LAX", None,
            None, None, None, None, "USA", None,
            None, None, None, None, "USA", None,
            Some(-100.0), None, "domestic", None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Distance")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_lane_validation_inverted_dates() {
        let engine = create_engine();
        let result = engine.create_lane(
            test_org_id(), "NYC-LAX", "NYC to LAX", None,
            None, None, None, None, "USA", None,
            None, None, None, None, "USA", None,
            None, None, "domestic", None, None,
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Effective from")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_lane_success() {
        let engine = create_engine();
        let result = engine.create_lane(
            test_org_id(), "NYC-LAX", "NYC to Los Angeles",
            Some("East Coast to West Coast route"),
            None, Some("New York Warehouse"), Some("New York"), Some("NY"), "USA", Some("10001"),
            None, Some("LA Distribution Center"), Some("Los Angeles"), Some("CA"), "USA", Some("90001"),
            Some(3944.0), Some(48.0), "domestic", None, None,
            Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let lane = result.unwrap();
        assert_eq!(lane.lane_code, "NYC-LAX");
        assert_eq!(lane.lane_type, "domestic");
        assert_eq!(lane.status, "active");
        assert_eq!(lane.distance_km, Some(3944.0));
    }

    // --- TransportShipment Tests ---

    #[tokio::test]
    async fn test_create_shipment_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "", None, None,
            "outbound", "normal", None, None, None,
            None, None, serde_json::json!({}),
            None, None, serde_json::json!({}),
            None, None, "USD",
            None, None, None, None, None,
            None, None, false, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_shipment_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "SHP-001", None, None,
            "interstellar", "normal", None, None, None,
            None, None, serde_json::json!({}),
            None, None, serde_json::json!({}),
            None, None, "USD",
            None, None, None, None, None,
            None, None, false, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("shipment_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_shipment_validation_bad_priority() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "SHP-001", None, None,
            "outbound", "urgent", None, None, None,
            None, None, serde_json::json!({}),
            None, None, serde_json::json!({}),
            None, None, "USD",
            None, None, None, None, None,
            None, None, false, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("priority")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_shipment_validation_inverted_dates() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "SHP-001", None, None,
            "outbound", "normal", None, None, None,
            None, Some("Warehouse A"), serde_json::json!({}),
            None, Some("Customer B"), serde_json::json!({}),
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15),
            chrono::NaiveDate::from_ymd_opt(2025, 6, 10),
            "USD",
            None, None, None, None, None,
            None, None, false, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("ship date")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_shipment_validation_negative_declared_value() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "SHP-001", None, None,
            "outbound", "normal", None, None, None,
            None, None, serde_json::json!({}),
            None, None, serde_json::json!({}),
            None, None, "USD",
            None, None, None, None, None,
            None, Some(-500.0), false, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Declared")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_shipment_success() {
        let engine = create_engine();
        let result = engine.create_shipment(
            test_org_id(), "SHP-001", Some("Customer Order #12345"), None,
            "outbound", "high", None, None, None,
            None, Some("NYC Warehouse"), serde_json::json!({"city": "New York", "state": "NY"}),
            None, Some("LA Customer"), serde_json::json!({"city": "Los Angeles", "state": "CA"}),
            chrono::NaiveDate::from_ymd_opt(2025, 6, 1),
            chrono::NaiveDate::from_ymd_opt(2025, 6, 5),
            "USD",
            Some(Uuid::new_v4()), Some("SO-12345"), None, None, None,
            Some("Fragile - handle with care"), Some(5000.0), true, true,
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let shipment = result.unwrap();
        assert_eq!(shipment.shipment_number, "SHP-001");
        assert_eq!(shipment.status, "draft");
        assert_eq!(shipment.shipment_type, "outbound");
        assert_eq!(shipment.priority, "high");
    }

    #[tokio::test]
    async fn test_book_shipment_not_found() {
        let engine = create_engine();
        let result = engine.book_shipment(Uuid::new_v4(), Some(test_user_id())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_shipment_not_found() {
        let engine = create_engine();
        let result = engine.cancel_shipment(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    // --- Tracking Event Tests ---

    #[tokio::test]
    async fn test_add_tracking_event_validation_bad_type() {
        let engine = create_engine();
        let result = engine.add_tracking_event(
            test_org_id(), Uuid::new_v4(), "beamed_up",
            None, None, None, None, None, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("event_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Freight Rate Tests ---

    #[tokio::test]
    async fn test_create_freight_rate_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_freight_rate(
            test_org_id(), "", "Standard Rate", None,
            Uuid::new_v4(), None, None,
            "per_kg", 2.50, 25.0, "USD", 12.5, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            false, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_freight_rate_validation_bad_rate_type() {
        let engine = create_engine();
        let result = engine.create_freight_rate(
            test_org_id(), "RATE-001", "Standard Rate", None,
            Uuid::new_v4(), None, None,
            "dynamic", 2.50, 25.0, "USD", 12.5, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            false, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("rate_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_freight_rate_validation_negative_amount() {
        let engine = create_engine();
        let result = engine.create_freight_rate(
            test_org_id(), "RATE-001", "Standard Rate", None,
            Uuid::new_v4(), None, None,
            "per_kg", -1.0, 25.0, "USD", 12.5, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            false, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("amount")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_freight_rate_validation_bad_fuel_surcharge() {
        let engine = create_engine();
        let result = engine.create_freight_rate(
            test_org_id(), "RATE-001", "Standard Rate", None,
            Uuid::new_v4(), None, None,
            "per_kg", 2.50, 25.0, "USD", 150.0, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            false, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Fuel")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_freight_rate_carrier_not_found() {
        let engine = create_engine();
        let result = engine.create_freight_rate(
            test_org_id(), "RATE-001", "Standard Rate", None,
            Uuid::new_v4(), None, None,
            "per_kg", 2.50, 25.0, "USD", 12.5, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            false, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("Carrier")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    // --- Stop Tests ---

    #[tokio::test]
    async fn test_add_stop_validation_bad_type() {
        let engine = create_engine();
        let result = engine.add_stop(
            test_org_id(), Uuid::new_v4(), 1, "layover",
            None, None, serde_json::json!({}),
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("stop_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_add_stop_validation_bad_stop_number() {
        let engine = create_engine();
        let result = engine.add_stop(
            test_org_id(), Uuid::new_v4(), 0, "pickup",
            None, None, serde_json::json!({}),
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Stop number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- TransportShipment Line Tests ---

    #[tokio::test]
    async fn test_add_shipment_line_validation_negative_qty() {
        let engine = create_engine();
        let result = engine.add_shipment_line(
            test_org_id(), Uuid::new_v4(), 1,
            None, None, None,
            -5, "EA", 10.0, 0.5, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Quantity")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_add_shipment_line_validation_negative_weight() {
        let engine = create_engine();
        let result = engine.add_shipment_line(
            test_org_id(), Uuid::new_v4(), 1,
            None, None, None,
            10, "EA", -5.0, 0.5, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Weight")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Dashboard Test ---

    #[tokio::test]
    async fn test_get_dashboard() {
        let engine = create_engine();
        let result = engine.get_dashboard(test_org_id()).await;
        assert!(result.is_ok());
        let dashboard = result.unwrap();
        assert_eq!(dashboard.total_shipments, 0);
        assert_eq!(dashboard.active_shipments, 0);
        assert_eq!(dashboard.total_carriers, 0);
        assert_eq!(dashboard.active_carriers, 0);
        assert_eq!(dashboard.total_lanes, 0);
    }
}
