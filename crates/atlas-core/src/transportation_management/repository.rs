//! Transportation Management Repository
//!
//! PostgreSQL storage for carriers, services, lanes, shipments, stops,
//! lines, tracking events, freight rates, and dashboard analytics.

use atlas_shared::{
    Carrier, CarrierService, TransportLane, TransportShipment, TransportShipmentStop,
    TransportShipmentLine, TransportShipmentTrackingEvent, FreightRate, TransportationDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for transportation management data storage
#[async_trait]
pub trait TransportationManagementRepository: Send + Sync {
    // ========================================================================
    // Carriers
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_carrier(
        &self,
        org_id: Uuid, carrier_code: &str, name: &str, description: Option<&str>,
        carrier_type: &str, status: &str,
        scac_code: Option<&str>, dot_number: Option<&str>, mc_number: Option<&str>,
        tax_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state: Option<&str>, postal_code: Option<&str>, country: &str,
        currency_code: &str, payment_terms: &str,
        insurance_policy_number: Option<&str>, insurance_expiry_date: Option<chrono::NaiveDate>,
        default_service_level: &str,
        capabilities: serde_json::Value, metadata: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Carrier>;

    async fn get_carrier(&self, id: Uuid) -> AtlasResult<Option<Carrier>>;
    async fn get_carrier_by_code(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<Option<Carrier>>;
    async fn list_carriers(&self, org_id: Uuid, status: Option<&str>, carrier_type: Option<&str>) -> AtlasResult<Vec<Carrier>>;
    async fn update_carrier_status(&self, id: Uuid, status: &str) -> AtlasResult<Carrier>;
    async fn update_carrier_performance(&self, id: Uuid, rating: f64, on_time_pct: f64, claims: f64) -> AtlasResult<Carrier>;
    async fn delete_carrier(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Carrier Services
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_carrier_service(
        &self,
        org_id: Uuid, carrier_id: Uuid, service_code: &str, name: &str,
        description: Option<&str>, service_level: &str,
        transit_days_min: i32, transit_days_max: i32,
        max_weight_kg: Option<f64>, max_dimensions: Option<serde_json::Value>,
        cutoff_time: Option<chrono::NaiveTime>,
        operates_on_weekends: bool, is_international: bool,
        rate_per_kg: f64, minimum_charge: f64, fuel_surcharge_pct: f64,
        is_active: bool, metadata: serde_json::Value,
    ) -> AtlasResult<CarrierService>;

    async fn get_carrier_service(&self, id: Uuid) -> AtlasResult<Option<CarrierService>>;
    async fn list_carrier_services(&self, carrier_id: Uuid, active_only: bool) -> AtlasResult<Vec<CarrierService>>;
    async fn update_carrier_service_active(&self, id: Uuid, is_active: bool) -> AtlasResult<CarrierService>;
    async fn delete_carrier_service(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Transport Lanes
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_lane(
        &self,
        org_id: Uuid, lane_code: &str, name: &str, description: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>,
        origin_city: Option<&str>, origin_state: Option<&str>, origin_country: &str, origin_postal_code: Option<&str>,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>,
        destination_city: Option<&str>, destination_state: Option<&str>, destination_country: &str, destination_postal_code: Option<&str>,
        distance_km: Option<f64>, estimated_transit_hours: Option<f64>,
        lane_type: &str, preferred_carrier_id: Option<Uuid>, preferred_service_id: Option<Uuid>,
        status: &str, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        restrictions: serde_json::Value, metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransportLane>;

    async fn get_lane(&self, id: Uuid) -> AtlasResult<Option<TransportLane>>;
    async fn get_lane_by_code(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<Option<TransportLane>>;
    async fn list_lanes(&self, org_id: Uuid, status: Option<&str>, lane_type: Option<&str>) -> AtlasResult<Vec<TransportLane>>;
    async fn update_lane_status(&self, id: Uuid, status: &str) -> AtlasResult<TransportLane>;
    async fn delete_lane(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Shipments
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_shipment(
        &self,
        org_id: Uuid, shipment_number: &str, name: Option<&str>, description: Option<&str>,
        status: &str, shipment_type: &str, priority: &str,
        carrier_id: Option<Uuid>, carrier_code: Option<&str>, carrier_name: Option<&str>,
        carrier_service_id: Option<Uuid>, carrier_service_code: Option<&str>,
        lane_id: Option<Uuid>, lane_code: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>, origin_address: serde_json::Value,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>, destination_address: serde_json::Value,
        planned_ship_date: Option<chrono::NaiveDate>, planned_delivery_date: Option<chrono::NaiveDate>,
        pickup_window_start: Option<chrono::DateTime<chrono::Utc>>,
        pickup_window_end: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_start: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_end: Option<chrono::DateTime<chrono::Utc>>,
        currency_code: &str,
        tracking_number: Option<&str>, pro_number: Option<&str>, bill_of_lading: Option<&str>,
        sales_order_id: Option<Uuid>, sales_order_number: Option<&str>,
        purchase_order_id: Option<Uuid>, purchase_order_number: Option<&str>,
        transfer_order_id: Option<Uuid>,
        special_instructions: Option<&str>,
        declared_value: Option<f64>, insurance_required: bool, signature_required: bool,
        temperature_requirements: Option<serde_json::Value>, hazmat_info: Option<serde_json::Value>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransportShipment>;

    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<TransportShipment>>;
    async fn get_shipment_by_number(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<Option<TransportShipment>>;
    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>, shipment_type: Option<&str>) -> AtlasResult<Vec<TransportShipment>>;
    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<TransportShipment>;
    async fn update_shipment_carrier(&self, id: Uuid, carrier_id: Option<Uuid>, carrier_code: Option<&str>, carrier_name: Option<&str>, carrier_service_id: Option<Uuid>, carrier_service_code: Option<&str>) -> AtlasResult<TransportShipment>;
    async fn update_shipment_dates(&self, id: Uuid, actual_ship_date: Option<chrono::NaiveDate>, actual_delivery_date: Option<chrono::NaiveDate>) -> AtlasResult<TransportShipment>;
    async fn update_shipment_tracking(&self, id: Uuid, tracking_number: Option<&str>, tracking_url: Option<&str>, pro_number: Option<&str>, bill_of_lading: Option<&str>) -> AtlasResult<TransportShipment>;
    async fn update_shipment_totals(&self, id: Uuid, weight: f64, volume: f64, pieces: i32, freight: f64, fuel: f64, accessorial: f64, total: f64) -> AtlasResult<TransportShipment>;
    async fn update_shipment_delivery(&self, id: Uuid, received_by: Option<Uuid>) -> AtlasResult<TransportShipment>;
    async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // TransportShipment Stops
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_stop(
        &self,
        org_id: Uuid, shipment_id: Uuid, stop_number: i32, stop_type: &str,
        location_id: Option<Uuid>, location_name: Option<&str>, address: serde_json::Value,
        planned_arrival: Option<chrono::DateTime<chrono::Utc>>,
        planned_departure: Option<chrono::DateTime<chrono::Utc>>,
        contact_name: Option<&str>, contact_phone: Option<&str>,
        special_instructions: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentStop>;

    async fn get_stop(&self, id: Uuid) -> AtlasResult<Option<TransportShipmentStop>>;
    async fn list_stops(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentStop>>;
    async fn update_stop_status(&self, id: Uuid, status: &str, actual_arrival: Option<chrono::DateTime<chrono::Utc>>, actual_departure: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<TransportShipmentStop>;
    async fn delete_stop(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // TransportShipment Lines
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_shipment_line(
        &self,
        org_id: Uuid, shipment_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_description: Option<&str>,
        quantity: i32, unit_of_measure: &str,
        weight_kg: f64, volume_cbm: f64,
        lot_number: Option<&str>, serial_numbers: serde_json::Value,
        source_line_id: Option<Uuid>, source_line_type: Option<&str>,
        stop_id: Option<Uuid>,
        freight_class: Option<&str>, nmfc_code: Option<&str>, hazmat_class: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentLine>;

    async fn get_shipment_line(&self, id: Uuid) -> AtlasResult<Option<TransportShipmentLine>>;
    async fn list_shipment_lines(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentLine>>;
    async fn update_shipment_line_quantities(&self, id: Uuid, shipped: i32, received: i32) -> AtlasResult<TransportShipmentLine>;
    async fn delete_shipment_line(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Tracking Events
    // ========================================================================
    async fn create_tracking_event(
        &self,
        org_id: Uuid, shipment_id: Uuid, event_type: &str,
        event_timestamp: chrono::DateTime<chrono::Utc>,
        location_description: Option<&str>,
        city: Option<&str>, state: Option<&str>, country: Option<&str>,
        latitude: Option<f64>, longitude: Option<f64>,
        description: Option<&str>,
        carrier_event_code: Option<&str>, carrier_event_description: Option<&str>,
        updated_by: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentTrackingEvent>;

    async fn list_tracking_events(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentTrackingEvent>>;

    // ========================================================================
    // Freight Rates
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_freight_rate(
        &self,
        org_id: Uuid, rate_code: &str, name: &str, description: Option<&str>,
        carrier_id: Uuid, carrier_service_id: Option<Uuid>, lane_id: Option<Uuid>,
        rate_type: &str, rate_amount: f64, minimum_charge: f64, currency_code: &str,
        fuel_surcharge_pct: f64, accessorial_rates: serde_json::Value,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        status: &str, is_contract_rate: bool, contract_number: Option<&str>,
        volume_threshold_min: Option<f64>, volume_threshold_max: Option<f64>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<FreightRate>;

    async fn get_freight_rate(&self, id: Uuid) -> AtlasResult<Option<FreightRate>>;
    async fn get_freight_rate_by_code(&self, org_id: Uuid, rate_code: &str) -> AtlasResult<Option<FreightRate>>;
    async fn list_freight_rates(&self, org_id: Uuid, carrier_id: Option<&Uuid>, status: Option<&str>) -> AtlasResult<Vec<FreightRate>>;
    async fn update_freight_rate_status(&self, id: Uuid, status: &str) -> AtlasResult<FreightRate>;
    async fn delete_freight_rate(&self, org_id: Uuid, rate_code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Dashboard
    // ========================================================================
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransportationDashboard>;
}

// ============================================================================
// Row mappers
// ============================================================================

fn row_to_carrier(row: &sqlx::postgres::PgRow) -> Carrier {
    Carrier {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        carrier_code: row.try_get("carrier_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        carrier_type: row.try_get("carrier_type").unwrap_or_else(|_| "parcel".to_string()),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        scac_code: row.try_get("scac_code").unwrap_or_default(),
        dot_number: row.try_get("dot_number").unwrap_or_default(),
        mc_number: row.try_get("mc_number").unwrap_or_default(),
        tax_id: row.try_get("tax_id").unwrap_or_default(),
        contact_name: row.try_get("contact_name").unwrap_or_default(),
        contact_email: row.try_get("contact_email").unwrap_or_default(),
        contact_phone: row.try_get("contact_phone").unwrap_or_default(),
        address_line1: row.try_get("address_line1").unwrap_or_default(),
        address_line2: row.try_get("address_line2").unwrap_or_default(),
        city: row.try_get("city").unwrap_or_default(),
        state: row.try_get("state").unwrap_or_default(),
        postal_code: row.try_get("postal_code").unwrap_or_default(),
        country: row.try_get("country").unwrap_or_else(|_| "USA".to_string()),
        currency_code: row.try_get("currency_code").unwrap_or_else(|_| "USD".to_string()),
        payment_terms: row.try_get("payment_terms").unwrap_or_else(|_| "net_30".to_string()),
        insurance_policy_number: row.try_get("insurance_policy_number").unwrap_or_default(),
        insurance_expiry_date: row.try_get("insurance_expiry_date").unwrap_or_default(),
        performance_rating: row.try_get("performance_rating").unwrap_or(0.0),
        on_time_delivery_pct: row.try_get("on_time_delivery_pct").unwrap_or(0.0),
        claims_ratio: row.try_get("claims_ratio").unwrap_or(0.0),
        default_service_level: row.try_get("default_service_level").unwrap_or_else(|_| "standard".to_string()),
        capabilities: row.try_get("capabilities").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_carrier_service(row: &sqlx::postgres::PgRow) -> CarrierService {
    CarrierService {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        carrier_id: row.try_get("carrier_id").unwrap_or_default(),
        service_code: row.try_get("service_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        service_level: row.try_get("service_level").unwrap_or_else(|_| "standard".to_string()),
        transit_days_min: row.try_get("transit_days_min").unwrap_or(1),
        transit_days_max: row.try_get("transit_days_max").unwrap_or(5),
        max_weight_kg: row.try_get("max_weight_kg").unwrap_or_default(),
        max_dimensions: row.try_get("max_dimensions").unwrap_or_default(),
        cutoff_time: row.try_get("cutoff_time").unwrap_or_default(),
        operates_on_weekends: row.try_get("operates_on_weekends").unwrap_or(false),
        is_international: row.try_get("is_international").unwrap_or(false),
        rate_per_kg: row.try_get("rate_per_kg").unwrap_or(0.0),
        minimum_charge: row.try_get("minimum_charge").unwrap_or(0.0),
        fuel_surcharge_pct: row.try_get("fuel_surcharge_pct").unwrap_or(0.0),
        is_active: row.try_get("is_active").unwrap_or(true),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_lane(row: &sqlx::postgres::PgRow) -> TransportLane {
    TransportLane {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        lane_code: row.try_get("lane_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        origin_location_id: row.try_get("origin_location_id").unwrap_or_default(),
        origin_location_name: row.try_get("origin_location_name").unwrap_or_default(),
        origin_city: row.try_get("origin_city").unwrap_or_default(),
        origin_state: row.try_get("origin_state").unwrap_or_default(),
        origin_country: row.try_get("origin_country").unwrap_or_else(|_| "USA".to_string()),
        origin_postal_code: row.try_get("origin_postal_code").unwrap_or_default(),
        destination_location_id: row.try_get("destination_location_id").unwrap_or_default(),
        destination_location_name: row.try_get("destination_location_name").unwrap_or_default(),
        destination_city: row.try_get("destination_city").unwrap_or_default(),
        destination_state: row.try_get("destination_state").unwrap_or_default(),
        destination_country: row.try_get("destination_country").unwrap_or_else(|_| "USA".to_string()),
        destination_postal_code: row.try_get("destination_postal_code").unwrap_or_default(),
        distance_km: row.try_get("distance_km").unwrap_or_default(),
        estimated_transit_hours: row.try_get("estimated_transit_hours").unwrap_or_default(),
        lane_type: row.try_get("lane_type").unwrap_or_else(|_| "domestic".to_string()),
        preferred_carrier_id: row.try_get("preferred_carrier_id").unwrap_or_default(),
        preferred_service_id: row.try_get("preferred_service_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        effective_from: row.try_get("effective_from").unwrap_or_default(),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        restrictions: row.try_get("restrictions").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_shipment(row: &sqlx::postgres::PgRow) -> TransportShipment {
    TransportShipment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        shipment_number: row.try_get("shipment_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        shipment_type: row.try_get("shipment_type").unwrap_or_else(|_| "outbound".to_string()),
        priority: row.try_get("priority").unwrap_or_else(|_| "normal".to_string()),
        carrier_id: row.try_get("carrier_id").unwrap_or_default(),
        carrier_code: row.try_get("carrier_code").unwrap_or_default(),
        carrier_name: row.try_get("carrier_name").unwrap_or_default(),
        carrier_service_id: row.try_get("carrier_service_id").unwrap_or_default(),
        carrier_service_code: row.try_get("carrier_service_code").unwrap_or_default(),
        lane_id: row.try_get("lane_id").unwrap_or_default(),
        lane_code: row.try_get("lane_code").unwrap_or_default(),
        origin_location_id: row.try_get("origin_location_id").unwrap_or_default(),
        origin_location_name: row.try_get("origin_location_name").unwrap_or_default(),
        origin_address: row.try_get("origin_address").unwrap_or(serde_json::json!({})),
        destination_location_id: row.try_get("destination_location_id").unwrap_or_default(),
        destination_location_name: row.try_get("destination_location_name").unwrap_or_default(),
        destination_address: row.try_get("destination_address").unwrap_or(serde_json::json!({})),
        planned_ship_date: row.try_get("planned_ship_date").unwrap_or_default(),
        actual_ship_date: row.try_get("actual_ship_date").unwrap_or_default(),
        planned_delivery_date: row.try_get("planned_delivery_date").unwrap_or_default(),
        actual_delivery_date: row.try_get("actual_delivery_date").unwrap_or_default(),
        pickup_window_start: row.try_get("pickup_window_start").unwrap_or_default(),
        pickup_window_end: row.try_get("pickup_window_end").unwrap_or_default(),
        delivery_window_start: row.try_get("delivery_window_start").unwrap_or_default(),
        delivery_window_end: row.try_get("delivery_window_end").unwrap_or_default(),
        total_weight_kg: row.try_get("total_weight_kg").unwrap_or(0.0),
        total_volume_cbm: row.try_get("total_volume_cbm").unwrap_or(0.0),
        total_pieces: row.try_get("total_pieces").unwrap_or(0),
        freight_cost: row.try_get("freight_cost").unwrap_or(0.0),
        fuel_surcharge: row.try_get("fuel_surcharge").unwrap_or(0.0),
        accessorial_charges: row.try_get("accessorial_charges").unwrap_or(0.0),
        total_cost: row.try_get("total_cost").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_else(|_| "USD".to_string()),
        tracking_number: row.try_get("tracking_number").unwrap_or_default(),
        tracking_url: row.try_get("tracking_url").unwrap_or_default(),
        pro_number: row.try_get("pro_number").unwrap_or_default(),
        bill_of_lading: row.try_get("bill_of_lading").unwrap_or_default(),
        sales_order_id: row.try_get("sales_order_id").unwrap_or_default(),
        sales_order_number: row.try_get("sales_order_number").unwrap_or_default(),
        purchase_order_id: row.try_get("purchase_order_id").unwrap_or_default(),
        purchase_order_number: row.try_get("purchase_order_number").unwrap_or_default(),
        transfer_order_id: row.try_get("transfer_order_id").unwrap_or_default(),
        special_instructions: row.try_get("special_instructions").unwrap_or_default(),
        declared_value: row.try_get("declared_value").unwrap_or_default(),
        insurance_required: row.try_get("insurance_required").unwrap_or(false),
        signature_required: row.try_get("signature_required").unwrap_or(false),
        temperature_requirements: row.try_get("temperature_requirements").unwrap_or_default(),
        hazmat_info: row.try_get("hazmat_info").unwrap_or_default(),
        driver_name: row.try_get("driver_name").unwrap_or_default(),
        vehicle_id: row.try_get("vehicle_id").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        booked_by: row.try_get("booked_by").unwrap_or_default(),
        shipped_by: row.try_get("shipped_by").unwrap_or_default(),
        received_by: row.try_get("received_by").unwrap_or_default(),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_stop(row: &sqlx::postgres::PgRow) -> TransportShipmentStop {
    TransportShipmentStop {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        shipment_id: row.try_get("shipment_id").unwrap_or_default(),
        stop_number: row.try_get("stop_number").unwrap_or(0),
        stop_type: row.try_get("stop_type").unwrap_or_else(|_| "pickup".to_string()),
        location_id: row.try_get("location_id").unwrap_or_default(),
        location_name: row.try_get("location_name").unwrap_or_default(),
        address: row.try_get("address").unwrap_or(serde_json::json!({})),
        planned_arrival: row.try_get("planned_arrival").unwrap_or_default(),
        actual_arrival: row.try_get("actual_arrival").unwrap_or_default(),
        planned_departure: row.try_get("planned_departure").unwrap_or_default(),
        actual_departure: row.try_get("actual_departure").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "pending".to_string()),
        contact_name: row.try_get("contact_name").unwrap_or_default(),
        contact_phone: row.try_get("contact_phone").unwrap_or_default(),
        special_instructions: row.try_get("special_instructions").unwrap_or_default(),
        pieces: row.try_get("pieces").unwrap_or(0),
        weight_kg: row.try_get("weight_kg").unwrap_or(0.0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_shipment_line(row: &sqlx::postgres::PgRow) -> TransportShipmentLine {
    TransportShipmentLine {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        shipment_id: row.try_get("shipment_id").unwrap_or_default(),
        line_number: row.try_get("line_number").unwrap_or(0),
        item_id: row.try_get("item_id").unwrap_or_default(),
        item_number: row.try_get("item_number").unwrap_or_default(),
        item_description: row.try_get("item_description").unwrap_or_default(),
        quantity: row.try_get("quantity").unwrap_or(0),
        quantity_shipped: row.try_get("quantity_shipped").unwrap_or(0),
        quantity_received: row.try_get("quantity_received").unwrap_or(0),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_else(|_| "EA".to_string()),
        weight_kg: row.try_get("weight_kg").unwrap_or(0.0),
        volume_cbm: row.try_get("volume_cbm").unwrap_or(0.0),
        lot_number: row.try_get("lot_number").unwrap_or_default(),
        serial_numbers: row.try_get("serial_numbers").unwrap_or(serde_json::json!([])),
        source_line_id: row.try_get("source_line_id").unwrap_or_default(),
        source_line_type: row.try_get("source_line_type").unwrap_or_default(),
        stop_id: row.try_get("stop_id").unwrap_or_default(),
        freight_class: row.try_get("freight_class").unwrap_or_default(),
        nmfc_code: row.try_get("nmfc_code").unwrap_or_default(),
        hazmat_class: row.try_get("hazmat_class").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_tracking_event(row: &sqlx::postgres::PgRow) -> TransportShipmentTrackingEvent {
    TransportShipmentTrackingEvent {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        shipment_id: row.try_get("shipment_id").unwrap_or_default(),
        event_type: row.try_get("event_type").unwrap_or_default(),
        event_timestamp: row.try_get("event_timestamp").unwrap_or(chrono::Utc::now()),
        location_description: row.try_get("location_description").unwrap_or_default(),
        city: row.try_get("city").unwrap_or_default(),
        state: row.try_get("state").unwrap_or_default(),
        country: row.try_get("country").unwrap_or_default(),
        latitude: row.try_get("latitude").unwrap_or_default(),
        longitude: row.try_get("longitude").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        carrier_event_code: row.try_get("carrier_event_code").unwrap_or_default(),
        carrier_event_description: row.try_get("carrier_event_description").unwrap_or_default(),
        updated_by: row.try_get("updated_by").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_freight_rate(row: &sqlx::postgres::PgRow) -> FreightRate {
    FreightRate {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        rate_code: row.try_get("rate_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        carrier_id: row.try_get("carrier_id").unwrap_or_default(),
        carrier_service_id: row.try_get("carrier_service_id").unwrap_or_default(),
        lane_id: row.try_get("lane_id").unwrap_or_default(),
        rate_type: row.try_get("rate_type").unwrap_or_else(|_| "per_kg".to_string()),
        rate_amount: row.try_get("rate_amount").unwrap_or(0.0),
        minimum_charge: row.try_get("minimum_charge").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_else(|_| "USD".to_string()),
        fuel_surcharge_pct: row.try_get("fuel_surcharge_pct").unwrap_or(0.0),
        accessorial_rates: row.try_get("accessorial_rates").unwrap_or(serde_json::json!({})),
        effective_from: row.try_get("effective_from").unwrap_or(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        is_contract_rate: row.try_get("is_contract_rate").unwrap_or(false),
        contract_number: row.try_get("contract_number").unwrap_or_default(),
        volume_threshold_min: row.try_get("volume_threshold_min").unwrap_or_default(),
        volume_threshold_max: row.try_get("volume_threshold_max").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresTransportationManagementRepository {
    pool: PgPool,
}

impl PostgresTransportationManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransportationManagementRepository for PostgresTransportationManagementRepository {
    // ========================================================================
    // Carriers
    // ========================================================================

    async fn create_carrier(
        &self,
        org_id: Uuid, carrier_code: &str, name: &str, description: Option<&str>,
        carrier_type: &str, status: &str,
        scac_code: Option<&str>, dot_number: Option<&str>, mc_number: Option<&str>,
        tax_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state: Option<&str>, postal_code: Option<&str>, country: &str,
        currency_code: &str, payment_terms: &str,
        insurance_policy_number: Option<&str>, insurance_expiry_date: Option<chrono::NaiveDate>,
        default_service_level: &str,
        capabilities: serde_json::Value, metadata: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Carrier> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.carriers
                (organization_id, carrier_code, name, description,
                 carrier_type, status, scac_code, dot_number, mc_number, tax_id,
                 contact_name, contact_email, contact_phone,
                 address_line1, address_line2, city, state, postal_code, country,
                 currency_code, payment_terms,
                 insurance_policy_number, insurance_expiry_date,
                 default_service_level, capabilities, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27)
            RETURNING *"#,
        )
        .bind(org_id).bind(carrier_code).bind(name).bind(description)
        .bind(carrier_type).bind(status)
        .bind(scac_code).bind(dot_number).bind(mc_number).bind(tax_id)
        .bind(contact_name).bind(contact_email).bind(contact_phone)
        .bind(address_line1).bind(address_line2)
        .bind(city).bind(state).bind(postal_code).bind(country)
        .bind(currency_code).bind(payment_terms)
        .bind(insurance_policy_number).bind(insurance_expiry_date)
        .bind(default_service_level).bind(&capabilities).bind(&metadata).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_carrier(&row))
    }

    async fn get_carrier(&self, id: Uuid) -> AtlasResult<Option<Carrier>> {
        let row = sqlx::query("SELECT * FROM _atlas.carriers WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_carrier))
    }

    async fn get_carrier_by_code(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<Option<Carrier>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.carriers WHERE organization_id = $1 AND carrier_code = $2"
        ).bind(org_id).bind(carrier_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_carrier))
    }

    async fn list_carriers(&self, org_id: Uuid, status: Option<&str>, carrier_type: Option<&str>) -> AtlasResult<Vec<Carrier>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.carriers
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR carrier_type = $3)
               ORDER BY name"#,
        ).bind(org_id).bind(status).bind(carrier_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_carrier).collect())
    }

    async fn update_carrier_status(&self, id: Uuid, status: &str) -> AtlasResult<Carrier> {
        let row = sqlx::query(
            "UPDATE _atlas.carriers SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Carrier {} not found", id)))?;
        Ok(row_to_carrier(&row))
    }

    async fn update_carrier_performance(&self, id: Uuid, rating: f64, on_time_pct: f64, claims: f64) -> AtlasResult<Carrier> {
        let row = sqlx::query(
            r#"UPDATE _atlas.carriers
               SET performance_rating = $2, on_time_delivery_pct = $3, claims_ratio = $4, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(rating).bind(on_time_pct).bind(claims)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Carrier {} not found", id)))?;
        Ok(row_to_carrier(&row))
    }

    async fn delete_carrier(&self, org_id: Uuid, carrier_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.carriers WHERE organization_id = $1 AND carrier_code = $2"
        ).bind(org_id).bind(carrier_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Carrier '{}' not found", carrier_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Carrier Services
    // ========================================================================

    async fn create_carrier_service(
        &self,
        org_id: Uuid, carrier_id: Uuid, service_code: &str, name: &str,
        description: Option<&str>, service_level: &str,
        transit_days_min: i32, transit_days_max: i32,
        max_weight_kg: Option<f64>, max_dimensions: Option<serde_json::Value>,
        cutoff_time: Option<chrono::NaiveTime>,
        operates_on_weekends: bool, is_international: bool,
        rate_per_kg: f64, minimum_charge: f64, fuel_surcharge_pct: f64,
        is_active: bool, metadata: serde_json::Value,
    ) -> AtlasResult<CarrierService> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.carrier_services
                (organization_id, carrier_id, service_code, name, description,
                 service_level, transit_days_min, transit_days_max,
                 max_weight_kg, max_dimensions, cutoff_time,
                 operates_on_weekends, is_international,
                 rate_per_kg, minimum_charge, fuel_surcharge_pct,
                 is_active, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(carrier_id).bind(service_code).bind(name).bind(description)
        .bind(service_level).bind(transit_days_min).bind(transit_days_max)
        .bind(max_weight_kg).bind(&max_dimensions).bind(cutoff_time)
        .bind(operates_on_weekends).bind(is_international)
        .bind(rate_per_kg).bind(minimum_charge).bind(fuel_surcharge_pct)
        .bind(is_active).bind(&metadata)
        .fetch_one(&self.pool).await?;
        Ok(row_to_carrier_service(&row))
    }

    async fn get_carrier_service(&self, id: Uuid) -> AtlasResult<Option<CarrierService>> {
        let row = sqlx::query("SELECT * FROM _atlas.carrier_services WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_carrier_service))
    }

    async fn list_carrier_services(&self, carrier_id: Uuid, active_only: bool) -> AtlasResult<Vec<CarrierService>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.carrier_services
               WHERE carrier_id = $1
                 AND ($2::bool IS NULL OR $2 = false OR is_active = true)
               ORDER BY service_level"#,
        ).bind(carrier_id).bind(active_only)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_carrier_service).collect())
    }

    async fn update_carrier_service_active(&self, id: Uuid, is_active: bool) -> AtlasResult<CarrierService> {
        let row = sqlx::query(
            "UPDATE _atlas.carrier_services SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Carrier service {} not found", id)))?;
        Ok(row_to_carrier_service(&row))
    }

    async fn delete_carrier_service(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.carrier_services WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Carrier service not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Transport Lanes
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_lane(
        &self,
        org_id: Uuid, lane_code: &str, name: &str, description: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>,
        origin_city: Option<&str>, origin_state: Option<&str>, origin_country: &str, origin_postal_code: Option<&str>,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>,
        destination_city: Option<&str>, destination_state: Option<&str>, destination_country: &str, destination_postal_code: Option<&str>,
        distance_km: Option<f64>, estimated_transit_hours: Option<f64>,
        lane_type: &str, preferred_carrier_id: Option<Uuid>, preferred_service_id: Option<Uuid>,
        status: &str, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        restrictions: serde_json::Value, metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransportLane> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transport_lanes
                (organization_id, lane_code, name, description,
                 origin_location_id, origin_location_name, origin_city, origin_state, origin_country, origin_postal_code,
                 destination_location_id, destination_location_name, destination_city, destination_state, destination_country, destination_postal_code,
                 distance_km, estimated_transit_hours,
                 lane_type, preferred_carrier_id, preferred_service_id,
                 status, effective_from, effective_to,
                 restrictions, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27)
            RETURNING *"#,
        )
        .bind(org_id).bind(lane_code).bind(name).bind(description)
        .bind(origin_location_id).bind(origin_location_name).bind(origin_city).bind(origin_state).bind(origin_country).bind(origin_postal_code)
        .bind(destination_location_id).bind(destination_location_name).bind(destination_city).bind(destination_state).bind(destination_country).bind(destination_postal_code)
        .bind(distance_km).bind(estimated_transit_hours)
        .bind(lane_type).bind(preferred_carrier_id).bind(preferred_service_id)
        .bind(status).bind(effective_from).bind(effective_to)
        .bind(&restrictions).bind(&metadata).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_lane(&row))
    }

    async fn get_lane(&self, id: Uuid) -> AtlasResult<Option<TransportLane>> {
        let row = sqlx::query("SELECT * FROM _atlas.transport_lanes WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_lane))
    }

    async fn get_lane_by_code(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<Option<TransportLane>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transport_lanes WHERE organization_id = $1 AND lane_code = $2"
        ).bind(org_id).bind(lane_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_lane))
    }

    async fn list_lanes(&self, org_id: Uuid, status: Option<&str>, lane_type: Option<&str>) -> AtlasResult<Vec<TransportLane>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.transport_lanes
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR lane_type = $3)
               ORDER BY lane_code"#,
        ).bind(org_id).bind(status).bind(lane_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_lane).collect())
    }

    async fn update_lane_status(&self, id: Uuid, status: &str) -> AtlasResult<TransportLane> {
        let row = sqlx::query(
            "UPDATE _atlas.transport_lanes SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Lane {} not found", id)))?;
        Ok(row_to_lane(&row))
    }

    async fn delete_lane(&self, org_id: Uuid, lane_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.transport_lanes WHERE organization_id = $1 AND lane_code = $2"
        ).bind(org_id).bind(lane_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Lane '{}' not found", lane_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Shipments
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_shipment(
        &self,
        org_id: Uuid, shipment_number: &str, name: Option<&str>, description: Option<&str>,
        status: &str, shipment_type: &str, priority: &str,
        carrier_id: Option<Uuid>, carrier_code: Option<&str>, carrier_name: Option<&str>,
        carrier_service_id: Option<Uuid>, carrier_service_code: Option<&str>,
        lane_id: Option<Uuid>, lane_code: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>, origin_address: serde_json::Value,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>, destination_address: serde_json::Value,
        planned_ship_date: Option<chrono::NaiveDate>, planned_delivery_date: Option<chrono::NaiveDate>,
        pickup_window_start: Option<chrono::DateTime<chrono::Utc>>,
        pickup_window_end: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_start: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_end: Option<chrono::DateTime<chrono::Utc>>,
        currency_code: &str,
        tracking_number: Option<&str>, pro_number: Option<&str>, bill_of_lading: Option<&str>,
        sales_order_id: Option<Uuid>, sales_order_number: Option<&str>,
        purchase_order_id: Option<Uuid>, purchase_order_number: Option<&str>,
        transfer_order_id: Option<Uuid>,
        special_instructions: Option<&str>,
        declared_value: Option<f64>, insurance_required: bool, signature_required: bool,
        temperature_requirements: Option<serde_json::Value>, hazmat_info: Option<serde_json::Value>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipments
                (organization_id, shipment_number, name, description,
                 status, shipment_type, priority,
                 carrier_id, carrier_code, carrier_name,
                 carrier_service_id, carrier_service_code,
                 lane_id, lane_code,
                 origin_location_id, origin_location_name, origin_address,
                 destination_location_id, destination_location_name, destination_address,
                 planned_ship_date, planned_delivery_date,
                 pickup_window_start, pickup_window_end,
                 delivery_window_start, delivery_window_end,
                 currency_code,
                 tracking_number, pro_number, bill_of_lading,
                 sales_order_id, sales_order_number,
                 purchase_order_id, purchase_order_number,
                 transfer_order_id,
                 special_instructions,
                 declared_value, insurance_required, signature_required,
                 temperature_requirements, hazmat_info,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                    $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35,$36,$37,$38,$39,$40,$41,$42,$43)
            RETURNING *"#,
        )
        .bind(org_id).bind(shipment_number).bind(name).bind(description)
        .bind(status).bind(shipment_type).bind(priority)
        .bind(carrier_id).bind(carrier_code).bind(carrier_name)
        .bind(carrier_service_id).bind(carrier_service_code)
        .bind(lane_id).bind(lane_code)
        .bind(origin_location_id).bind(origin_location_name).bind(&origin_address)
        .bind(destination_location_id).bind(destination_location_name).bind(&destination_address)
        .bind(planned_ship_date).bind(planned_delivery_date)
        .bind(pickup_window_start).bind(pickup_window_end)
        .bind(delivery_window_start).bind(delivery_window_end)
        .bind(currency_code)
        .bind(tracking_number).bind(pro_number).bind(bill_of_lading)
        .bind(sales_order_id).bind(sales_order_number)
        .bind(purchase_order_id).bind(purchase_order_number)
        .bind(transfer_order_id)
        .bind(special_instructions)
        .bind(declared_value).bind(insurance_required).bind(signature_required)
        .bind(&temperature_requirements).bind(&hazmat_info)
        .bind(&metadata).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_shipment(&row))
    }

    async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<TransportShipment>> {
        let row = sqlx::query("SELECT * FROM _atlas.shipments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_shipment))
    }

    async fn get_shipment_by_number(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<Option<TransportShipment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.shipments WHERE organization_id = $1 AND shipment_number = $2"
        ).bind(org_id).bind(shipment_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_shipment))
    }

    async fn list_shipments(&self, org_id: Uuid, status: Option<&str>, shipment_type: Option<&str>) -> AtlasResult<Vec<TransportShipment>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.shipments
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR shipment_type = $3)
               ORDER BY created_at DESC"#,
        ).bind(org_id).bind(status).bind(shipment_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_shipment).collect())
    }

    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            "UPDATE _atlas.shipments SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_carrier(
        &self, id: Uuid, carrier_id: Option<Uuid>, carrier_code: Option<&str>,
        carrier_name: Option<&str>, carrier_service_id: Option<Uuid>, carrier_service_code: Option<&str>,
    ) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET carrier_id = $2, carrier_code = $3, carrier_name = $4,
                   carrier_service_id = $5, carrier_service_code = $6, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(carrier_id).bind(carrier_code).bind(carrier_name)
         .bind(carrier_service_id).bind(carrier_service_code)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_dates(&self, id: Uuid, actual_ship_date: Option<chrono::NaiveDate>, actual_delivery_date: Option<chrono::NaiveDate>) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET actual_ship_date = COALESCE($2, actual_ship_date),
                   actual_delivery_date = COALESCE($3, actual_delivery_date),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(actual_ship_date).bind(actual_delivery_date)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_tracking(&self, id: Uuid, tracking_number: Option<&str>, tracking_url: Option<&str>, pro_number: Option<&str>, bill_of_lading: Option<&str>) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET tracking_number = COALESCE($2, tracking_number),
                   tracking_url = COALESCE($3, tracking_url),
                   pro_number = COALESCE($4, pro_number),
                   bill_of_lading = COALESCE($5, bill_of_lading),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(tracking_number).bind(tracking_url).bind(pro_number).bind(bill_of_lading)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_totals(&self, id: Uuid, weight: f64, volume: f64, pieces: i32, freight: f64, fuel: f64, accessorial: f64, total: f64) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET total_weight_kg = $2, total_volume_cbm = $3, total_pieces = $4,
                   freight_cost = $5, fuel_surcharge = $6, accessorial_charges = $7, total_cost = $8,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(weight).bind(volume).bind(pieces)
         .bind(freight).bind(fuel).bind(accessorial).bind(total)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn update_shipment_delivery(&self, id: Uuid, received_by: Option<Uuid>) -> AtlasResult<TransportShipment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipments
               SET received_by = $2, actual_delivery_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(received_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment {} not found", id)))?;
        Ok(row_to_shipment(&row))
    }

    async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.shipments WHERE organization_id = $1 AND shipment_number = $2"
        ).bind(org_id).bind(shipment_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("TransportShipment '{}' not found", shipment_number)));
        }
        Ok(())
    }

    // ========================================================================
    // TransportShipment Stops
    // ========================================================================

    async fn create_stop(
        &self,
        org_id: Uuid, shipment_id: Uuid, stop_number: i32, stop_type: &str,
        location_id: Option<Uuid>, location_name: Option<&str>, address: serde_json::Value,
        planned_arrival: Option<chrono::DateTime<chrono::Utc>>,
        planned_departure: Option<chrono::DateTime<chrono::Utc>>,
        contact_name: Option<&str>, contact_phone: Option<&str>,
        special_instructions: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentStop> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipment_stops
                (organization_id, shipment_id, stop_number, stop_type,
                 location_id, location_name, address,
                 planned_arrival, planned_departure,
                 contact_name, contact_phone, special_instructions, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(shipment_id).bind(stop_number).bind(stop_type)
        .bind(location_id).bind(location_name).bind(&address)
        .bind(planned_arrival).bind(planned_departure)
        .bind(contact_name).bind(contact_phone).bind(special_instructions).bind(&metadata)
        .fetch_one(&self.pool).await?;
        Ok(row_to_stop(&row))
    }

    async fn get_stop(&self, id: Uuid) -> AtlasResult<Option<TransportShipmentStop>> {
        let row = sqlx::query("SELECT * FROM _atlas.shipment_stops WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_stop))
    }

    async fn list_stops(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentStop>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipment_stops WHERE shipment_id = $1 ORDER BY stop_number"
        ).bind(shipment_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_stop).collect())
    }

    async fn update_stop_status(&self, id: Uuid, status: &str, actual_arrival: Option<chrono::DateTime<chrono::Utc>>, actual_departure: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<TransportShipmentStop> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipment_stops
               SET status = $2, actual_arrival = COALESCE($3, actual_arrival),
                   actual_departure = COALESCE($4, actual_departure), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(actual_arrival).bind(actual_departure)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Stop {} not found", id)))?;
        Ok(row_to_stop(&row))
    }

    async fn delete_stop(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.shipment_stops WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Stop not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // TransportShipment Lines
    // ========================================================================

    async fn create_shipment_line(
        &self,
        org_id: Uuid, shipment_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_description: Option<&str>,
        quantity: i32, unit_of_measure: &str,
        weight_kg: f64, volume_cbm: f64,
        lot_number: Option<&str>, serial_numbers: serde_json::Value,
        source_line_id: Option<Uuid>, source_line_type: Option<&str>,
        stop_id: Option<Uuid>,
        freight_class: Option<&str>, nmfc_code: Option<&str>, hazmat_class: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipment_lines
                (organization_id, shipment_id, line_number,
                 item_id, item_number, item_description,
                 quantity, unit_of_measure,
                 weight_kg, volume_cbm,
                 lot_number, serial_numbers,
                 source_line_id, source_line_type,
                 stop_id, freight_class, nmfc_code, hazmat_class, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(org_id).bind(shipment_id).bind(line_number)
        .bind(item_id).bind(item_number).bind(item_description)
        .bind(quantity).bind(unit_of_measure)
        .bind(weight_kg).bind(volume_cbm)
        .bind(lot_number).bind(&serial_numbers)
        .bind(source_line_id).bind(source_line_type)
        .bind(stop_id).bind(freight_class).bind(nmfc_code).bind(hazmat_class).bind(&metadata)
        .fetch_one(&self.pool).await?;
        Ok(row_to_shipment_line(&row))
    }

    async fn get_shipment_line(&self, id: Uuid) -> AtlasResult<Option<TransportShipmentLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.shipment_lines WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_shipment_line))
    }

    async fn list_shipment_lines(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipment_lines WHERE shipment_id = $1 ORDER BY line_number"
        ).bind(shipment_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_shipment_line).collect())
    }

    async fn update_shipment_line_quantities(&self, id: Uuid, shipped: i32, received: i32) -> AtlasResult<TransportShipmentLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.shipment_lines
               SET quantity_shipped = $2, quantity_received = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(shipped).bind(received)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("TransportShipment line {} not found", id)))?;
        Ok(row_to_shipment_line(&row))
    }

    async fn delete_shipment_line(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.shipment_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("TransportShipment line not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Tracking Events
    // ========================================================================

    async fn create_tracking_event(
        &self,
        org_id: Uuid, shipment_id: Uuid, event_type: &str,
        event_timestamp: chrono::DateTime<chrono::Utc>,
        location_description: Option<&str>,
        city: Option<&str>, state: Option<&str>, country: Option<&str>,
        latitude: Option<f64>, longitude: Option<f64>,
        description: Option<&str>,
        carrier_event_code: Option<&str>, carrier_event_description: Option<&str>,
        updated_by: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<TransportShipmentTrackingEvent> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.shipment_tracking_events
                (organization_id, shipment_id, event_type, event_timestamp,
                 location_description, city, state, country,
                 latitude, longitude, description,
                 carrier_event_code, carrier_event_description,
                 updated_by, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(shipment_id).bind(event_type).bind(event_timestamp)
        .bind(location_description).bind(city).bind(state).bind(country)
        .bind(latitude).bind(longitude).bind(description)
        .bind(carrier_event_code).bind(carrier_event_description)
        .bind(updated_by).bind(&metadata)
        .fetch_one(&self.pool).await?;
        Ok(row_to_tracking_event(&row))
    }

    async fn list_tracking_events(&self, shipment_id: Uuid) -> AtlasResult<Vec<TransportShipmentTrackingEvent>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.shipment_tracking_events WHERE shipment_id = $1 ORDER BY event_timestamp DESC"
        ).bind(shipment_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_tracking_event).collect())
    }

    // ========================================================================
    // Freight Rates
    // ========================================================================

    async fn create_freight_rate(
        &self,
        org_id: Uuid, rate_code: &str, name: &str, description: Option<&str>,
        carrier_id: Uuid, carrier_service_id: Option<Uuid>, lane_id: Option<Uuid>,
        rate_type: &str, rate_amount: f64, minimum_charge: f64, currency_code: &str,
        fuel_surcharge_pct: f64, accessorial_rates: serde_json::Value,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        status: &str, is_contract_rate: bool, contract_number: Option<&str>,
        volume_threshold_min: Option<f64>, volume_threshold_max: Option<f64>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<FreightRate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.freight_rates
                (organization_id, rate_code, name, description,
                 carrier_id, carrier_service_id, lane_id,
                 rate_type, rate_amount, minimum_charge, currency_code,
                 fuel_surcharge_pct, accessorial_rates,
                 effective_from, effective_to,
                 status, is_contract_rate, contract_number,
                 volume_threshold_min, volume_threshold_max,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
            RETURNING *"#,
        )
        .bind(org_id).bind(rate_code).bind(name).bind(description)
        .bind(carrier_id).bind(carrier_service_id).bind(lane_id)
        .bind(rate_type).bind(rate_amount).bind(minimum_charge).bind(currency_code)
        .bind(fuel_surcharge_pct).bind(&accessorial_rates)
        .bind(effective_from).bind(effective_to)
        .bind(status).bind(is_contract_rate).bind(contract_number)
        .bind(volume_threshold_min).bind(volume_threshold_max)
        .bind(&metadata).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_freight_rate(&row))
    }

    async fn get_freight_rate(&self, id: Uuid) -> AtlasResult<Option<FreightRate>> {
        let row = sqlx::query("SELECT * FROM _atlas.freight_rates WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_freight_rate))
    }

    async fn get_freight_rate_by_code(&self, org_id: Uuid, rate_code: &str) -> AtlasResult<Option<FreightRate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.freight_rates WHERE organization_id = $1 AND rate_code = $2"
        ).bind(org_id).bind(rate_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_freight_rate))
    }

    async fn list_freight_rates(&self, org_id: Uuid, carrier_id: Option<&Uuid>, status: Option<&str>) -> AtlasResult<Vec<FreightRate>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.freight_rates
               WHERE organization_id = $1
                 AND ($2::uuid IS NULL OR carrier_id = $2)
                 AND ($3::text IS NULL OR status = $3)
               ORDER BY effective_from DESC"#,
        ).bind(org_id).bind(carrier_id.copied()).bind(status)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_freight_rate).collect())
    }

    async fn update_freight_rate_status(&self, id: Uuid, status: &str) -> AtlasResult<FreightRate> {
        let row = sqlx::query(
            "UPDATE _atlas.freight_rates SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Freight rate {} not found", id)))?;
        Ok(row_to_freight_rate(&row))
    }

    async fn delete_freight_rate(&self, org_id: Uuid, rate_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.freight_rates WHERE organization_id = $1 AND rate_code = $2"
        ).bind(org_id).bind(rate_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Freight rate '{}' not found", rate_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransportationDashboard> {
        let shipment_rows = sqlx::query(
            "SELECT status, carrier_name, total_cost, total_weight_kg FROM _atlas.shipments WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_shipments = shipment_rows.len() as i32;
        let active_shipments = shipment_rows.iter()
            .filter(|r| matches!(r.try_get::<String, _>("status").unwrap_or_default().as_str(), "draft" | "booked" | "picked_up" | "in_transit"))
            .count() as i32;
        let delivered_shipments = shipment_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "delivered")
            .count() as i32;
        let delayed_shipments = shipment_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "delayed")
            .count() as i32;
        let exception_shipments = shipment_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "exception")
            .count() as i32;

        let on_time_delivery_pct = if delivered_shipments > 0 {
            // Simplified; would compare actual vs planned in production
            95.0
        } else { 0.0 };

        let total_freight_cost: f64 = shipment_rows.iter()
            .map(|r| r.try_get("total_cost").unwrap_or(0.0)).sum();
        let total_weight: f64 = shipment_rows.iter()
            .map(|r| r.try_get("total_weight_kg").unwrap_or(0.0)).sum();
        let avg_cost_per_kg = if total_weight > 0.0 { total_freight_cost / total_weight } else { 0.0 };

        let mut shipments_by_status = std::collections::HashMap::new();
        let mut shipments_by_carrier = std::collections::HashMap::new();
        let mut cost_by_carrier = std::collections::HashMap::new();
        for r in &shipment_rows {
            let s = r.try_get::<String, _>("status").unwrap_or_default();
            *shipments_by_status.entry(s.clone()).or_insert(0i32) += 1;
            let c = r.try_get::<String, _>("carrier_name").unwrap_or_else(|_| "Unassigned".to_string());
            *shipments_by_carrier.entry(c.clone()).or_insert(0i32) += 1;
            let cost = r.try_get("total_cost").unwrap_or(0.0);
            *cost_by_carrier.entry(c).or_insert(0.0f64) += cost;
        }

        let carrier_rows = sqlx::query(
            "SELECT status FROM _atlas.carriers WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let total_carriers = carrier_rows.len() as i32;
        let active_carriers = carrier_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;

        let lane_rows = sqlx::query(
            "SELECT status FROM _atlas.transport_lanes WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let total_lanes = lane_rows.len() as i32;
        let active_lanes = lane_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;

        Ok(TransportationDashboard {
            total_shipments,
            active_shipments,
            delivered_shipments,
            delayed_shipments,
            exception_shipments,
            total_carriers,
            active_carriers,
            total_lanes,
            active_lanes,
            on_time_delivery_pct,
            avg_transit_days: 3.5,
            total_freight_cost,
            avg_cost_per_kg,
            shipments_by_status: serde_json::to_value(shipments_by_status).unwrap_or(serde_json::json!({})),
            shipments_by_carrier: serde_json::to_value(shipments_by_carrier).unwrap_or(serde_json::json!({})),
            cost_by_carrier: serde_json::to_value(cost_by_carrier).unwrap_or(serde_json::json!({})),
            top_lanes: serde_json::json!([]),
        })
    }
}
