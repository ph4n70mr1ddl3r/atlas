//! Shipping Execution Engine
//!
//! Oracle Fusion SCM: Shipping Execution.
//! Manages carriers, shipping methods, shipments, shipment lines,
//! packing slips, and shipping analytics.
//!
//! The process follows Oracle Fusion Shipping Execution workflow:
//! 1. Define carriers (shipping partners)
//! 2. Define shipping methods (ground, express, overnight, etc.)
//! 3. Create shipments from sales orders
//! 4. Add shipment lines (items to ship)
//! 5. Confirm shipment (pick/pack)
//! 6. Ship confirm with tracking number
//! 7. Record delivery
//! 8. Create packing slips for packages
//! 9. Analyze via dashboard

use atlas_shared::AtlasError;
use super::ShippingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid carrier types
const VALID_CARRIER_TYPES: &[&str] = &["external", "internal", "third_party"];

/// Valid shipment statuses
const VALID_SHIPMENT_STATUSES: &[&str] = &[
    "draft", "confirmed", "picked", "packed", "shipped", "delivered", "cancelled",
];

/// Valid package types
const VALID_PACKAGE_TYPES: &[&str] = &["box", "pallet", "envelope", "crate"];

/// Shipping Execution engine
pub struct ShippingEngine {
    repository: Arc<dyn ShippingRepository>,
}

impl ShippingEngine {
    pub fn new(repository: Arc<dyn ShippingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Carrier Management
    // ========================================================================

    /// Create a carrier
    pub async fn create_carrier(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        carrier_type: &str,
        tracking_url_template: Option<&str>,
        contact_name: Option<&str>,
        contact_phone: Option<&str>,
        contact_email: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ShippingCarrier> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Carrier code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Carrier name is required".to_string()));
        }
        if !VALID_CARRIER_TYPES.contains(&carrier_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid carrier_type '{}'. Must be one of: {}", carrier_type, VALID_CARRIER_TYPES.join(", ")
            )));
        }
        if self.repository.get_carrier_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Carrier '{}' already exists", code)));
        }
        info!("Creating carrier '{}' for org {}", code, org_id);
        self.repository.create_carrier(
            org_id, &code, name, description, carrier_type,
            tracking_url_template, contact_name, contact_phone, contact_email,
            created_by,
        ).await
    }

    /// Get a carrier by ID
    pub async fn get_carrier(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::ShippingCarrier>> {
        self.repository.get_carrier(id).await
    }

    /// List carriers
    pub async fn list_carriers(&self, org_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ShippingCarrier>> {
        self.repository.list_carriers(org_id).await
    }

    /// Delete a carrier
    pub async fn delete_carrier(&self, org_id: Uuid, code: &str) -> atlas_shared::AtlasResult<()> {
        info!("Deleting carrier '{}' for org {}", code, org_id);
        self.repository.delete_carrier(org_id, code).await
    }

    // ========================================================================
    // Shipping Method Management
    // ========================================================================

    /// Create a shipping method
    pub async fn create_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        carrier_id: Option<Uuid>,
        transit_time_days: i32,
        is_express: bool,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ShippingMethod> {
        let code = code.to_uppercase();
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Method code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Method name is required".to_string()));
        }
        if transit_time_days < 0 {
            return Err(AtlasError::ValidationFailed("Transit time cannot be negative".to_string()));
        }
        info!("Creating shipping method '{}' for org {}", code, org_id);
        self.repository.create_method(
            org_id, &code, name, description, carrier_id,
            transit_time_days, is_express, created_by,
        ).await
    }

    /// Get a method by ID
    pub async fn get_method(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::ShippingMethod>> {
        self.repository.get_method(id).await
    }

    /// List methods
    pub async fn list_methods(&self, org_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ShippingMethod>> {
        self.repository.list_methods(org_id).await
    }

    /// Delete a method
    pub async fn delete_method(&self, org_id: Uuid, code: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_method(org_id, code).await
    }

    // ========================================================================
    // Shipment Management
    // ========================================================================

    /// Create a shipment
    pub async fn create_shipment(
        &self,
        org_id: Uuid,
        shipment_number: &str,
        description: Option<&str>,
        carrier_id: Option<Uuid>,
        carrier_name: Option<&str>,
        shipping_method_id: Option<Uuid>,
        shipping_method_name: Option<&str>,
        order_id: Option<Uuid>,
        order_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        ship_from_warehouse: Option<&str>,
        ship_to_name: Option<&str>,
        ship_to_address: Option<&str>,
        ship_to_city: Option<&str>,
        ship_to_state: Option<&str>,
        ship_to_postal_code: Option<&str>,
        ship_to_country: Option<&str>,
        estimated_delivery: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Shipment> {
        if shipment_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Shipment number is required".to_string()));
        }
        if self.repository.get_shipment_by_number(org_id, shipment_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Shipment '{}' already exists", shipment_number)));
        }

        // Resolve carrier name from carrier_id if not provided
        let carrier_name = if carrier_name.is_none() {
            if let Some(cid) = carrier_id {
                self.repository.get_carrier(cid).await?.map(|c| c.name.clone())
            } else {
                None
            }
        } else {
            carrier_name.map(|s| s.to_string())
        };

        // Resolve shipping method name from method_id if not provided
        let shipping_method_name = if shipping_method_name.is_none() {
            if let Some(mid) = shipping_method_id {
                self.repository.get_method(mid).await?.map(|m| m.name.clone())
            } else {
                None
            }
        } else {
            shipping_method_name.map(|s| s.to_string())
        };

        info!("Creating shipment '{}' for org {}", shipment_number, org_id);
        self.repository.create_shipment(
            org_id, shipment_number, description,
            carrier_id, carrier_name.as_deref(),
            shipping_method_id, shipping_method_name.as_deref(),
            order_id, order_number,
            customer_id, customer_name,
            ship_from_warehouse,
            ship_to_name, ship_to_address,
            ship_to_city, ship_to_state,
            ship_to_postal_code, ship_to_country,
            estimated_delivery, notes, created_by,
        ).await
    }

    /// Get a shipment by ID
    pub async fn get_shipment(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::Shipment>> {
        self.repository.get_shipment(id).await
    }

    /// List shipments with optional status filter
    pub async fn list_shipments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::Shipment>> {
        if let Some(s) = status {
            if !VALID_SHIPMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SHIPMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_shipments(org_id, status).await
    }

    /// Confirm a shipment (pick/pack stage)
    pub async fn confirm_shipment(
        &self,
        id: Uuid,
        confirmed_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Shipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;
        if shipment.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm shipment in '{}' status. Must be 'draft'.", shipment.status
            )));
        }
        info!("Confirming shipment {}", shipment.shipment_number);
        self.repository.confirm_shipment(id, confirmed_by).await
    }

    /// Ship confirm with tracking number
    pub async fn ship_confirm(
        &self,
        id: Uuid,
        tracking_number: Option<&str>,
        shipped_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Shipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;
        if shipment.status != "confirmed" && shipment.status != "picked" && shipment.status != "packed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot ship confirm shipment in '{}' status. Must be 'confirmed', 'picked', or 'packed'.", shipment.status
            )));
        }
        info!("Ship confirming shipment {} with tracking {:?}", shipment.shipment_number, tracking_number);
        self.repository.ship_confirm(id, tracking_number, shipped_by).await
    }

    /// Record delivery
    pub async fn deliver(
        &self,
        id: Uuid,
        delivered_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Shipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;
        if shipment.status != "shipped" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deliver shipment in '{}' status. Must be 'shipped'.", shipment.status
            )));
        }
        info!("Recording delivery for shipment {}", shipment.shipment_number);
        self.repository.deliver(id, delivered_by).await
    }

    /// Cancel a shipment
    pub async fn cancel_shipment(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::Shipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;
        if shipment.status == "shipped" || shipment.status == "delivered" || shipment.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel shipment in '{}' status.", shipment.status
            )));
        }
        info!("Cancelling shipment {}", shipment.shipment_number);
        self.repository.update_shipment_status(id, "cancelled").await
    }

    /// Delete a shipment
    pub async fn delete_shipment(&self, org_id: Uuid, shipment_number: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_shipment(org_id, shipment_number).await
    }

    // ========================================================================
    // Shipment Line Management
    // ========================================================================

    /// Add a shipment line
    pub async fn add_shipment_line(
        &self,
        org_id: Uuid,
        shipment_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        item_description: Option<&str>,
        requested_quantity: &str,
        unit_of_measure: Option<&str>,
        weight: Option<&str>,
        weight_unit: Option<&str>,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        is_fragile: bool,
        is_hazardous: bool,
        notes: Option<&str>,
        order_line_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ShipmentLine> {
        if item_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Item code is required".to_string()));
        }

        let qty: f64 = requested_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Requested quantity must be a number".to_string())
        })?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Requested quantity must be positive".to_string()));
        }

        // Verify shipment exists and is in editable state
        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", shipment_id)))?;

        if shipment.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to shipment in '{}' status. Must be 'draft'.", shipment.status
            )));
        }

        // Get next line number
        let existing = self.repository.list_shipment_lines(shipment_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding line {} (item: {}) to shipment {}", line_number, item_code, shipment.shipment_number);

        self.repository.add_shipment_line(
            org_id, shipment_id, line_number, order_line_id,
            item_code, item_name, item_description,
            requested_quantity, unit_of_measure,
            weight, weight_unit, lot_number, serial_number,
            is_fragile, is_hazardous, notes,
        ).await
    }

    /// List shipment lines
    pub async fn list_shipment_lines(&self, shipment_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ShipmentLine>> {
        self.repository.list_shipment_lines(shipment_id).await
    }

    /// Delete a shipment line
    pub async fn delete_shipment_line(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let line = self.repository.get_shipment_line(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment line {} not found", id)))?;

        // Check shipment status
        let shipment = self.repository.get_shipment(line.shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Shipment not found".to_string()))?;
        if shipment.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete lines from non-draft shipment".to_string(),
            ));
        }

        self.repository.delete_shipment_line(id).await
    }

    /// Update shipped quantity on a line (during ship confirm)
    pub async fn update_line_shipped_quantity(
        &self,
        id: Uuid,
        shipped_quantity: &str,
    ) -> atlas_shared::AtlasResult<atlas_shared::ShipmentLine> {
        let line = self.repository.get_shipment_line(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment line {} not found", id)))?;

        let shipped: f64 = shipped_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Shipped quantity must be a number".to_string())
        })?;
        if shipped < 0.0 {
            return Err(AtlasError::ValidationFailed("Shipped quantity cannot be negative".to_string()));
        }
        let requested: f64 = line.requested_quantity.parse().unwrap_or(0.0);
        if shipped > requested {
            return Err(AtlasError::ValidationFailed(format!(
                "Shipped quantity ({}) cannot exceed requested quantity ({})", shipped, requested
            )));
        }

        info!("Updating shipped qty for line {} item {} to {}", line.line_number, line.item_code, shipped);
        self.repository.update_line_shipped_quantity(id, shipped_quantity).await
    }

    // ========================================================================
    // Packing Slip Management
    // ========================================================================

    /// Create a packing slip
    pub async fn create_packing_slip(
        &self,
        org_id: Uuid,
        shipment_id: Uuid,
        packing_slip_number: &str,
        package_number: i32,
        package_type: Option<&str>,
        weight: Option<&str>,
        weight_unit: Option<&str>,
        dimensions_length: Option<&str>,
        dimensions_width: Option<&str>,
        dimensions_height: Option<&str>,
        dimensions_unit: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::PackingSlip> {
        if packing_slip_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Packing slip number is required".to_string()));
        }
        if package_number < 1 {
            return Err(AtlasError::ValidationFailed("Package number must be >= 1".to_string()));
        }
        if let Some(pt) = package_type {
            if !VALID_PACKAGE_TYPES.contains(&pt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid package_type '{}'. Must be one of: {}", pt, VALID_PACKAGE_TYPES.join(", ")
                )));
            }
        }

        // Verify shipment exists
        let shipment = self.repository.get_shipment(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", shipment_id)))?;

        info!("Creating packing slip {} for shipment {}", packing_slip_number, shipment.shipment_number);
        self.repository.create_packing_slip(
            org_id, shipment_id, packing_slip_number,
            package_number, package_type,
            weight, weight_unit,
            dimensions_length, dimensions_width, dimensions_height, dimensions_unit,
            notes, created_by,
        ).await
    }

    /// List packing slips for a shipment
    pub async fn list_packing_slips(&self, shipment_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::PackingSlip>> {
        self.repository.list_packing_slips(shipment_id).await
    }

    /// Get a packing slip
    pub async fn get_packing_slip(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::PackingSlip>> {
        self.repository.get_packing_slip(id).await
    }

    /// Delete a packing slip
    pub async fn delete_packing_slip(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_packing_slip(id).await
    }

    // ========================================================================
    // Packing Slip Line Management
    // ========================================================================

    /// Add a packing slip line
    pub async fn add_packing_slip_line(
        &self,
        org_id: Uuid,
        packing_slip_id: Uuid,
        shipment_line_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        packed_quantity: &str,
        notes: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::PackingSlipLine> {
        if item_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Item code is required".to_string()));
        }
        let qty: f64 = packed_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Packed quantity must be a number".to_string())
        })?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Packed quantity must be positive".to_string()));
        }

        // Get next line number
        let existing = self.repository.list_packing_slip_lines(packing_slip_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding packing slip line {} (item: {})", line_number, item_code);
        self.repository.add_packing_slip_line(
            org_id, packing_slip_id, shipment_line_id,
            line_number, item_code, item_name, packed_quantity, notes,
        ).await
    }

    /// List packing slip lines
    pub async fn list_packing_slip_lines(&self, packing_slip_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::PackingSlipLine>> {
        self.repository.list_packing_slip_lines(packing_slip_id).await
    }

    /// Delete a packing slip line
    pub async fn delete_packing_slip_line(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_packing_slip_line(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get shipping dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::ShippingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_carrier_types() {
        assert!(VALID_CARRIER_TYPES.contains(&"external"));
        assert!(VALID_CARRIER_TYPES.contains(&"internal"));
        assert!(VALID_CARRIER_TYPES.contains(&"third_party"));
        assert!(!VALID_CARRIER_TYPES.contains(&"courier"));
    }

    #[test]
    fn test_valid_shipment_statuses() {
        assert!(VALID_SHIPMENT_STATUSES.contains(&"draft"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"confirmed"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"picked"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"packed"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"shipped"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"delivered"));
        assert!(VALID_SHIPMENT_STATUSES.contains(&"cancelled"));
        assert!(!VALID_SHIPMENT_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_package_types() {
        assert!(VALID_PACKAGE_TYPES.contains(&"box"));
        assert!(VALID_PACKAGE_TYPES.contains(&"pallet"));
        assert!(VALID_PACKAGE_TYPES.contains(&"envelope"));
        assert!(VALID_PACKAGE_TYPES.contains(&"crate"));
        assert!(!VALID_PACKAGE_TYPES.contains(&"tube"));
    }

    #[test]
    fn test_shipment_lifecycle_transitions() {
        // draft -> confirmed -> shipped -> delivered
        let statuses = vec!["draft", "confirmed", "shipped", "delivered"];
        for s in &statuses {
            assert!(VALID_SHIPMENT_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_quantity_validation_positive() {
        let qty: f64 = "100".parse().unwrap();
        assert!(qty > 0.0);
        let qty: f64 = "0".parse().unwrap();
        assert!(qty <= 0.0);
        let qty: f64 = "-5".parse().unwrap();
        assert!(qty <= 0.0);
    }
}
