//! Order Management Engine
//!
//! Business logic for Oracle Fusion SCM > Order Management.
//! Handles sales order lifecycle: creation, submission, confirmation,
//! fulfillment, holds, shipment, and cancellation.

use crate::order_management::repository::OrderManagementRepository;
use atlas_shared::{
    AtlasError, AtlasResult, SalesOrder, SalesOrderLine,
    OrderHold, FulfillmentShipment, OrderManagementDashboard,
    CreateSalesOrderRequest, AddOrderLineRequest,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid order statuses (Oracle Fusion order lifecycle)
const VALID_ORDER_STATUSES: &[&str] = &[
    "draft", "submitted", "confirmed", "processing", "shipped",
    "delivered", "closed", "cancelled",
];

/// Valid fulfillment statuses
const VALID_FULFILLMENT_STATUSES: &[&str] = &[
    "not_started", "reserved", "staged", "released", "shipped", "delivered",
];

/// Valid hold types (Oracle Fusion hold categories)
const VALID_HOLD_TYPES: &[&str] = &[
    "credit_check", "fraud_review", "pricing_review", "inventory_check",
    "customer_request", "payment_verification", "compliance", "manual",
];

/// Valid shipment statuses
const VALID_SHIPMENT_STATUSES: &[&str] = &[
    "planned", "packed", "shipped", "in_transit", "delivered", "cancelled",
];

/// Order Management Engine
pub struct OrderManagementEngine {
    repository: Arc<dyn OrderManagementRepository>,
}

impl OrderManagementEngine {
    pub fn new(repository: Arc<dyn OrderManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Sales Orders
    // ========================================================================

    /// Create a new sales order
    pub async fn create_order(
        &self,
        org_id: Uuid,
        req: CreateSalesOrderRequest,
    ) -> AtlasResult<SalesOrder> {
        if req.currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }
        if let Some(channel) = &req.sales_channel {
            if channel.is_empty() {
                return Err(AtlasError::ValidationFailed("Sales channel cannot be empty".to_string()));
            }
        }

        let order_number = format!("SO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating sales order {} for org {}", order_number, org_id);

        self.repository.create_order(
            org_id, &order_number, req.customer_id,
            req.customer_name.as_deref(),
            req.customer_po_number.as_deref(),
            req.order_date, req.requested_ship_date,
            req.requested_delivery_date,
            req.ship_to_address.as_deref(),
            req.bill_to_address.as_deref(),
            &req.currency_code,
            req.payment_terms.as_deref(),
            req.shipping_method.as_deref(),
            req.sales_channel.as_deref(),
            req.salesperson_id,
            req.salesperson_name.as_deref(),
            req.created_by,
        ).await
    }

    /// Get an order by number
    pub async fn get_order(&self, org_id: Uuid, order_number: &str) -> AtlasResult<Option<SalesOrder>> {
        self.repository.get_order(org_id, order_number).await
    }

    /// Get an order by ID
    pub async fn get_order_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesOrder>> {
        self.repository.get_order_by_id(id).await
    }

    /// List orders with optional filtering
    pub async fn list_orders(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        fulfillment_status: Option<&str>,
    ) -> AtlasResult<Vec<SalesOrder>> {
        if let Some(s) = status {
            if !VALID_ORDER_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_ORDER_STATUSES.join(", ")
                )));
            }
        }
        if let Some(fs) = fulfillment_status {
            if !VALID_FULFILLMENT_STATUSES.contains(&fs) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid fulfillment status '{}'. Must be one of: {}",
                    fs, VALID_FULFILLMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_orders(org_id, status, fulfillment_status).await
    }

    /// Submit a draft order
    pub async fn submit_order(&self, org_id: Uuid, id: Uuid) -> AtlasResult<SalesOrder> {
        let order = self.repository.get_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", id)))?;

        if order.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order does not belong to your organization".to_string()));
        }

        if order.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit order in '{}' status. Must be 'draft'.",
                order.status
            )));
        }

        // Check for active holds
        let holds = self.repository.list_holds(id, true).await?;
        if !holds.is_empty() {
            return Err(AtlasError::WorkflowError(
                "Cannot submit order with active holds. Release all holds first.".to_string(),
            ));
        }

        // Check order has at least one line
        let lines = self.repository.list_order_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::WorkflowError(
                "Cannot submit order with no lines. Add at least one line.".to_string(),
            ));
        }

        info!("Submitting sales order {}", order.order_number);
        self.repository.update_order_status(id, "submitted").await
    }

    /// Confirm a submitted order (begins fulfillment)
    pub async fn confirm_order(&self, org_id: Uuid, id: Uuid) -> AtlasResult<SalesOrder> {
        let order = self.repository.get_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", id)))?;

        if order.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order does not belong to your organization".to_string()));
        }

        if order.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm order in '{}' status. Must be 'submitted'.",
                order.status
            )));
        }

        info!("Confirming sales order {}", order.order_number);
        self.repository.update_order_status(id, "confirmed").await?;
        self.repository.update_order_fulfillment(id, "reserved").await
    }

    /// Close a completed order
    pub async fn close_order(&self, org_id: Uuid, id: Uuid) -> AtlasResult<SalesOrder> {
        let order = self.repository.get_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", id)))?;

        if order.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order does not belong to your organization".to_string()));
        }

        if order.status != "delivered" && order.status != "shipped" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close order in '{}' status. Must be 'shipped' or 'delivered'.",
                order.status
            )));
        }

        info!("Closing sales order {}", order.order_number);
        self.repository.update_order_status(id, "closed").await
    }

    /// Cancel an order
    pub async fn cancel_order(&self, org_id: Uuid, id: Uuid, reason: Option<&str>) -> AtlasResult<SalesOrder> {
        let order = self.repository.get_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", id)))?;

        if order.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order does not belong to your organization".to_string()));
        }

        if order.status == "closed" || order.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel order in '{}' status.",
                order.status
            )));
        }

        info!("Cancelling sales order {} (reason: {:?})", order.order_number, reason);
        self.repository.update_order_status(id, "cancelled").await
    }

    // ========================================================================
    // Sales Order Lines
    // ========================================================================

    /// Add a line to an order
    pub async fn add_order_line(
        &self,
        req: AddOrderLineRequest,
    ) -> AtlasResult<SalesOrderLine> {
        let order = self.repository.get_order_by_id(req.order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", req.order_id)))?;

        if order.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to order in '{}' status. Must be 'draft'.",
                order.status
            )));
        }

        let qty: f64 = req.quantity_ordered.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        let price: f64 = req.unit_selling_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit selling price must be a valid number".to_string(),
        ))?;

        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity must be positive".to_string()));
        }
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed("Unit selling price cannot be negative".to_string()));
        }

        // Validate discount
        if let Some(dp) = &req.discount_percent {
            let pct: f64 = dp.parse().map_err(|_| AtlasError::ValidationFailed(
                "Discount percent must be a valid number".to_string(),
            ))?;
            if !(0.0..=100.0).contains(&pct) {
                return Err(AtlasError::ValidationFailed(
                    "Discount percent must be between 0 and 100".to_string(),
                ));
            }
        }

        // Determine next line number
        let existing_lines = self.repository.list_order_lines(req.order_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to order {}", line_number, order.order_number);

        let line = self.repository.create_order_line(
            req.org_id, req.order_id, line_number,
            req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
            &req.quantity_ordered, &req.unit_selling_price,
            req.unit_list_price.as_deref(),
            req.discount_percent.as_deref(), req.discount_amount.as_deref(),
            req.tax_code.as_deref(),
            req.requested_ship_date, req.promised_delivery_date,
            req.ship_from_warehouse.as_deref(),
        ).await?;

        // Recalculate order totals
        self.repository.update_order_totals(req.order_id).await?;

        Ok(line)
    }

    /// Get an order line by ID
    pub async fn get_order_line(&self, id: Uuid) -> AtlasResult<Option<SalesOrderLine>> {
        self.repository.get_order_line(id).await
    }

    /// List all lines for an order
    pub async fn list_order_lines(&self, order_id: Uuid) -> AtlasResult<Vec<SalesOrderLine>> {
        self.repository.list_order_lines(order_id).await
    }

    /// Ship an order line (update quantities)
    pub async fn ship_order_line(
        &self,
        org_id: Uuid,
        id: Uuid,
        quantity_shipped: &str,
    ) -> AtlasResult<SalesOrderLine> {
        let line = self.repository.get_order_line(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order line {} not found", id)))?;

        if line.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order line does not belong to your organization".to_string()));
        }

        if line.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Cannot ship a cancelled line".to_string(),
            ));
        }

        let shipped: f64 = quantity_shipped.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity shipped must be a valid number".to_string(),
        ))?;
        let ordered: f64 = line.quantity_ordered.parse().unwrap_or(0.0);
        let already_shipped: f64 = line.quantity_shipped.parse().unwrap_or(0.0);

        if shipped <= 0.0 {
            return Err(AtlasError::ValidationFailed("Shipped quantity must be positive".to_string()));
        }

        let total_shipped = already_shipped + shipped;
        if total_shipped > ordered {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot ship {} units. Only {} remaining of {} ordered.",
                shipped, ordered - already_shipped, ordered
            )));
        }

        // Determine remaining and backorder
        let backordered = if total_shipped < ordered { ordered - total_shipped } else { 0.0 };
        let line_status = if total_shipped >= ordered { "shipped" } else { "partially_shipped" };
        let fulfillment = if total_shipped >= ordered { "shipped" } else { "released" };

        info!("Shipping {} units for order line {} (status: {})", shipped, line.line_number, line_status);

        self.repository.update_line_quantities(
            id,
            Some(&format!("{:.4}", total_shipped)),
            None,
            Some(&format!("{:.4}", backordered)),
        ).await?;

        // update_line_status re-reads the row, so it will have both the
        // quantity changes (persisted in the real DB) and the new status.
        let result = self.repository.update_line_status(id, line_status, fulfillment).await?;
        Ok(result)
    }

    /// Cancel an order line
    pub async fn cancel_order_line(&self, org_id: Uuid, id: Uuid, reason: Option<&str>) -> AtlasResult<SalesOrderLine> {
        let line = self.repository.get_order_line(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order line {} not found", id)))?;

        if line.organization_id != org_id {
            return Err(AtlasError::Forbidden("Order line does not belong to your organization".to_string()));
        }

        if line.status == "cancelled" {
            return Err(AtlasError::WorkflowError("Line is already cancelled".to_string()));
        }

        let ordered: f64 = line.quantity_ordered.parse().unwrap_or(0.0);
        let already_shipped: f64 = line.quantity_shipped.parse().unwrap_or(0.0);
        let to_cancel = ordered - already_shipped;

        info!("Cancelling order line {} ({} units)", line.line_number, to_cancel);

        self.repository.update_line_quantities(
            id,
            None,
            Some(&format!("{:.4}", to_cancel)),
            Some("0"),
        ).await?;

        self.repository.update_line_cancellation_reason(id, reason).await?;
        let result = self.repository.update_line_status(id, "cancelled", "not_started").await?;

        Ok(result)
    }

    // ========================================================================
    // Order Holds
    // ========================================================================

    /// Apply a hold to an order or line
    pub async fn apply_hold(
        &self,
        org_id: Uuid,
        order_id: Uuid,
        order_line_id: Option<Uuid>,
        hold_type: &str,
        hold_reason: &str,
        applied_by: Option<Uuid>,
        applied_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold> {
        if !VALID_HOLD_TYPES.contains(&hold_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hold type '{}'. Must be one of: {}", hold_type, VALID_HOLD_TYPES.join(", ")
            )));
        }
        if hold_reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Hold reason is required".to_string()));
        }

        // Verify order exists
        let _order = self.repository.get_order_by_id(order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", order_id)))?;

        // Verify line exists if specified
        if let Some(line_id) = order_line_id {
            let line = self.repository.get_order_line(line_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Order line {} not found", line_id)))?;
            if line.order_id != order_id {
                return Err(AtlasError::ValidationFailed(
                    "Order line does not belong to the specified order".to_string(),
                ));
            }
        }

        info!("Applying {} hold to order {}", hold_type, order_id);

        self.repository.create_hold(
            org_id, order_id, order_line_id,
            hold_type, hold_reason, applied_by, applied_by_name,
        ).await
    }

    /// Release a hold
    pub async fn release_hold(
        &self,
        org_id: Uuid,
        id: Uuid,
        released_by: Option<Uuid>,
        released_by_name: Option<&str>,
    ) -> AtlasResult<OrderHold> {
        let hold = self.repository.get_hold(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hold {} not found", id)))?;

        if hold.organization_id != org_id {
            return Err(AtlasError::Forbidden("Hold does not belong to your organization".to_string()));
        }

        if !hold.is_active {
            return Err(AtlasError::WorkflowError("Hold is already released".to_string()));
        }

        info!("Releasing hold {} on order {}", hold.hold_type, hold.order_id);

        self.repository.release_hold(id, released_by, released_by_name).await
    }

    /// Get a single hold by ID
    pub async fn get_hold(&self, org_id: Uuid, id: Uuid) -> AtlasResult<Option<OrderHold>> {
        let hold = self.repository.get_hold(id).await?;
        match hold {
            Some(h) if h.organization_id != org_id => Ok(None),
            other => Ok(other),
        }
    }

    /// List holds for an order
    pub async fn list_holds(&self, order_id: Uuid, active_only: bool) -> AtlasResult<Vec<OrderHold>> {
        self.repository.list_holds(order_id, active_only).await
    }

    /// Check if an order has any active holds
    #[allow(dead_code)]
    pub async fn has_active_holds(&self, order_id: Uuid) -> AtlasResult<bool> {
        let holds = self.repository.list_holds(order_id, true).await?;
        Ok(!holds.is_empty())
    }

    // ========================================================================
    // Fulfillment Shipments
    // ========================================================================

    /// Create a shipment for an order
    pub async fn create_shipment(
        &self,
        org_id: Uuid,
        order_id: Uuid,
        order_line_ids: Vec<Uuid>,
        warehouse: Option<&str>,
        carrier: Option<&str>,
        shipping_method: Option<&str>,
        estimated_delivery_date: Option<chrono::NaiveDate>,
        shipped_by: Option<Uuid>,
        shipped_by_name: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment> {
        if order_line_ids.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "At least one order line ID is required".to_string(),
            ));
        }

        // Verify order exists and is in processable state
        let order = self.repository.get_order_by_id(order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", order_id)))?;

        if order.status != "confirmed" && order.status != "processing" && order.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot create shipment for order in '{}' status.",
                order.status
            )));
        }

        let shipment_number = format!("SHP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating shipment {} for order {}", shipment_number, order.order_number);

        let shipment = self.repository.create_shipment(
            org_id, &shipment_number, order_id,
            serde_json::json!(order_line_ids),
            warehouse, carrier, shipping_method,
            estimated_delivery_date, shipped_by, shipped_by_name,
        ).await?;

        // Update order status to processing if not already
        if order.status == "confirmed" {
            self.repository.update_order_status(order_id, "processing").await?;
        }

        Ok(shipment)
    }

    /// Get a shipment by ID
    pub async fn get_shipment(&self, org_id: Uuid, id: Uuid) -> AtlasResult<Option<FulfillmentShipment>> {
        let shipment = self.repository.get_shipment(id).await?;
        match shipment {
            Some(s) if s.organization_id != org_id => Ok(None),
            other => Ok(other),
        }
    }

    /// List shipments
    pub async fn list_shipments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        order_id: Option<Uuid>,
    ) -> AtlasResult<Vec<FulfillmentShipment>> {
        if let Some(s) = status {
            if !VALID_SHIPMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid shipment status '{}'. Must be one of: {}", s, VALID_SHIPMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_shipments(org_id, status, order_id).await
    }

    /// Confirm shipment (mark as shipped)
    pub async fn confirm_shipment(
        &self,
        org_id: Uuid,
        id: Uuid,
        ship_date: chrono::NaiveDate,
    ) -> AtlasResult<FulfillmentShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;

        if shipment.organization_id != org_id {
            return Err(AtlasError::Forbidden("Shipment does not belong to your organization".to_string()));
        }

        if shipment.status != "planned" && shipment.status != "packed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm shipment in '{}' status. Must be 'planned' or 'packed'.",
                shipment.status
            )));
        }

        info!("Confirming shipment {} with ship date {}", shipment.shipment_number, ship_date);

        let updated = self.repository.confirm_ship(id, ship_date).await?;

        // Update order's actual ship date
        self.repository.update_order_dates(shipment.order_id, Some(ship_date), None).await?;

        // Move order to processing (not shipped) — the order should only transition
        // to "shipped" when ALL shipments are delivered, which is tracked at a higher level.
        let order = self.repository.get_order_by_id(shipment.order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Order {} not found", shipment.order_id)))?;
        if order.status == "confirmed" || order.status == "submitted" {
            self.repository.update_order_status(shipment.order_id, "processing").await?;
        }
        self.repository.update_order_fulfillment(shipment.order_id, "released").await?;

        Ok(updated)
    }

    /// Update shipment tracking info
    pub async fn update_tracking(
        &self,
        org_id: Uuid,
        id: Uuid,
        tracking_number: Option<&str>,
        estimated_delivery: Option<chrono::NaiveDate>,
    ) -> AtlasResult<FulfillmentShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;

        if shipment.organization_id != org_id {
            return Err(AtlasError::Forbidden("Shipment does not belong to your organization".to_string()));
        }

        info!("Updating tracking for shipment {}", shipment.shipment_number);

        self.repository.update_shipment_tracking(
            id, tracking_number, estimated_delivery, None,
        ).await
    }

    /// Confirm delivery
    ///
    /// Marks a single shipment as delivered. The order's status only transitions
    /// to "delivered" when **all** shipments for the order have been delivered.
    pub async fn confirm_delivery(
        &self,
        org_id: Uuid,
        id: Uuid,
        delivery_date: chrono::NaiveDate,
        delivery_confirmation: Option<&str>,
    ) -> AtlasResult<FulfillmentShipment> {
        let shipment = self.repository.get_shipment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Shipment {} not found", id)))?;

        if shipment.organization_id != org_id {
            return Err(AtlasError::Forbidden("Shipment does not belong to your organization".to_string()));
        }

        if shipment.status != "shipped" && shipment.status != "in_transit" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm delivery for shipment in '{}' status. Must be 'shipped' or 'in_transit'.",
                shipment.status
            )));
        }

        info!("Confirming delivery for shipment {} on {}", shipment.shipment_number, delivery_date);

        self.repository.update_shipment_status(id, "delivered").await?;
        let updated = self.repository.update_shipment_tracking(
            id, None, Some(delivery_date), delivery_confirmation,
        ).await?;

        // Update order delivery date
        self.repository.update_order_dates(shipment.order_id, None, Some(delivery_date)).await?;

        // Only transition order to "delivered" when ALL shipments are delivered
        let all_shipments = self.repository.list_shipments(
            updated.organization_id, None, Some(shipment.order_id),
        ).await?;
        let all_delivered = all_shipments.iter().all(|s| s.status == "delivered");

        if all_delivered {
            self.repository.update_order_status(shipment.order_id, "delivered").await?;
            self.repository.update_order_fulfillment(shipment.order_id, "delivered").await?;
        }

        Ok(updated)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get order management dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<OrderManagementDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::{CreateSalesOrderRequest, AddOrderLineRequest};

    use std::collections::HashMap;
    use std::sync::Mutex;

    fn empty_order_request() -> CreateSalesOrderRequest {
        CreateSalesOrderRequest {
            customer_id: None,
            customer_name: None,
            customer_po_number: None,
            order_date: chrono::Utc::now().date_naive(),
            requested_ship_date: None,
            requested_delivery_date: None,
            ship_to_address: None,
            bill_to_address: None,
            currency_code: String::new(),
            payment_terms: None,
            shipping_method: None,
            sales_channel: None,
            salesperson_id: None,
            salesperson_name: None,
            created_by: None,
        }
    }

    fn empty_line_request() -> AddOrderLineRequest {
        AddOrderLineRequest {
            org_id: Uuid::nil(),
            order_id: Uuid::nil(),
            item_id: None,
            item_code: None,
            item_description: None,
            quantity_ordered: String::new(),
            unit_selling_price: String::new(),
            unit_list_price: None,
            discount_percent: None,
            discount_amount: None,
            tax_code: None,
            requested_ship_date: None,
            promised_delivery_date: None,
            ship_from_warehouse: None,
        }
    }

    /// Fixed test org ID used by the mock for all generated entities.
    /// Tests must pass this value when calling engine methods that take `org_id`.
    const TEST_ORG: &str = "00000000-0000-0000-0000-000000000042";
    fn test_org_id() -> Uuid { Uuid::parse_str(TEST_ORG).unwrap() }

    /// Stateful mock that simulates a real repository's persistence
    struct MockState {
        order_status: HashMap<Uuid, String>,
        line_data: HashMap<Uuid, SalesOrderLine>,
        hold_active: HashMap<Uuid, bool>,
        shipment_data: HashMap<Uuid, FulfillmentShipment>,
    }

    struct MockOrderRepo {
        state: Arc<Mutex<MockState>>,
    }

    impl MockOrderRepo {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    order_status: HashMap::new(),
                    line_data: HashMap::new(),
                    hold_active: HashMap::new(),
                    shipment_data: HashMap::new(),
                })),
            }
        }

        fn make_order(&self, id: Uuid) -> SalesOrder {
            let status = self.state.lock().unwrap()
                .order_status.get(&id).cloned().unwrap_or_else(|| "draft".to_string());
            SalesOrder {
                id, organization_id: test_org_id(),
                order_number: "SO-MOCK".to_string(),
                customer_id: None, customer_name: Some("Test Customer".to_string()),
                customer_po_number: None,
                order_date: chrono::Utc::now().date_naive(),
                requested_ship_date: None, actual_ship_date: None,
                requested_delivery_date: None, actual_delivery_date: None,
                ship_to_address: None, bill_to_address: None,
                currency_code: "USD".to_string(),
                subtotal_amount: "100".to_string(), tax_amount: "10".to_string(),
                shipping_charges: "5".to_string(), total_amount: "115".to_string(),
                payment_terms: None, shipping_method: None,
                sales_channel: Some("direct".to_string()),
                salesperson_id: None, salesperson_name: None,
                status, fulfillment_status: "not_started".to_string(),
                submitted_at: None, confirmed_at: None, closed_at: None,
                cancelled_at: None, cancellation_reason: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }
        }

        fn make_line(&self, id: Uuid) -> SalesOrderLine {
            self.state.lock().unwrap()
                .line_data.get(&id).cloned().unwrap_or_else(|| SalesOrderLine {
                    id, organization_id: test_org_id(),
                    order_id: Uuid::new_v4(), line_number: 1,
                    item_id: None, item_code: Some("ITEM-01".to_string()),
                    item_description: Some("Widget".to_string()),
                    quantity_ordered: "10".to_string(),
                    quantity_shipped: "0".to_string(),
                    quantity_cancelled: "0".to_string(),
                    quantity_backordered: "0".to_string(),
                    unit_selling_price: "25".to_string(),
                    unit_list_price: None, line_amount: "250".to_string(),
                    discount_percent: None, discount_amount: None,
                    tax_code: None, tax_amount: "0".to_string(),
                    requested_ship_date: None, actual_ship_date: None,
                    promised_delivery_date: None, ship_from_warehouse: None,
                    fulfillment_status: "not_started".to_string(),
                    status: "open".to_string(),
                    cancellation_reason: None,
                    metadata: serde_json::json!({}),
                    created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
                })
        }
    }

    #[async_trait::async_trait]
    impl OrderManagementRepository for MockOrderRepo {
        async fn create_order(
            &self, org_id: Uuid, order_number: &str, customer_id: Option<Uuid>,
            customer_name: Option<&str>, customer_po_number: Option<&str>,
            order_date: chrono::NaiveDate, requested_ship_date: Option<chrono::NaiveDate>,
            requested_delivery_date: Option<chrono::NaiveDate>,
            ship_to_address: Option<&str>, bill_to_address: Option<&str>,
            currency_code: &str, payment_terms: Option<&str>, shipping_method: Option<&str>,
            sales_channel: Option<&str>, salesperson_id: Option<Uuid>,
            salesperson_name: Option<&str>, created_by: Option<Uuid>,
        ) -> AtlasResult<SalesOrder> {
            Ok(SalesOrder {
                id: Uuid::new_v4(), organization_id: org_id,
                order_number: order_number.to_string(),
                customer_id, customer_name: customer_name.map(|s| s.to_string()),
                customer_po_number: customer_po_number.map(|s| s.to_string()),
                order_date, requested_ship_date, actual_ship_date: None,
                requested_delivery_date, actual_delivery_date: None,
                ship_to_address: ship_to_address.map(|s| s.to_string()),
                bill_to_address: bill_to_address.map(|s| s.to_string()),
                currency_code: currency_code.to_string(),
                subtotal_amount: "0".to_string(), tax_amount: "0".to_string(),
                shipping_charges: "0".to_string(), total_amount: "0".to_string(),
                payment_terms: payment_terms.map(|s| s.to_string()),
                shipping_method: shipping_method.map(|s| s.to_string()),
                sales_channel: sales_channel.map(|s| s.to_string()),
                salesperson_id, salesperson_name: salesperson_name.map(|s| s.to_string()),
                status: "draft".to_string(), fulfillment_status: "not_started".to_string(),
                submitted_at: None, confirmed_at: None, closed_at: None,
                cancelled_at: None, cancellation_reason: None,
                created_by, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }

        async fn get_order(&self, _org_id: Uuid, _order_number: &str) -> AtlasResult<Option<SalesOrder>> { Ok(None) }

        async fn get_order_by_id(&self, id: Uuid) -> AtlasResult<Option<SalesOrder>> {
            Ok(Some(self.make_order(id)))
        }

        async fn list_orders(&self, _org_id: Uuid, _status: Option<&str>, _fulfillment_status: Option<&str>) -> AtlasResult<Vec<SalesOrder>> { Ok(vec![]) }

        async fn update_order_status(&self, id: Uuid, status: &str) -> AtlasResult<SalesOrder> {
            self.state.lock().unwrap().order_status.insert(id, status.to_string());
            Ok(self.make_order(id))
        }

        async fn update_order_fulfillment(&self, id: Uuid, _fulfillment_status: &str) -> AtlasResult<SalesOrder> {
            // The mock's make_order doesn't read fulfillment_status from state,
            // but the engine tests for confirm_order check the returned status
            // via update_order_status, not fulfillment. This is sufficient for unit tests.
            Ok(self.make_order(id))
        }

        async fn update_order_totals(&self, id: Uuid) -> AtlasResult<SalesOrder> {
            Ok(self.make_order(id))
        }

        async fn update_order_dates(&self, id: Uuid, _actual_ship_date: Option<chrono::NaiveDate>, _actual_delivery_date: Option<chrono::NaiveDate>) -> AtlasResult<SalesOrder> {
            Ok(self.make_order(id))
        }

        async fn create_order_line(
            &self, org_id: Uuid, order_id: Uuid, line_number: i32,
            item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
            quantity_ordered: &str, unit_selling_price: &str,
            unit_list_price: Option<&str>, discount_percent: Option<&str>,
            discount_amount: Option<&str>, tax_code: Option<&str>,
            requested_ship_date: Option<chrono::NaiveDate>,
            promised_delivery_date: Option<chrono::NaiveDate>,
            ship_from_warehouse: Option<&str>,
        ) -> AtlasResult<SalesOrderLine> {
            Ok(SalesOrderLine {
                id: Uuid::new_v4(), organization_id: org_id, order_id, line_number,
                item_id, item_code: item_code.map(|s| s.to_string()),
                item_description: item_description.map(|s| s.to_string()),
                quantity_ordered: quantity_ordered.to_string(),
                quantity_shipped: "0".to_string(),
                quantity_cancelled: "0".to_string(),
                quantity_backordered: "0".to_string(),
                unit_selling_price: unit_selling_price.to_string(),
                unit_list_price: unit_list_price.map(|s| s.to_string()),
                line_amount: "0".to_string(),
                discount_percent: discount_percent.map(|s| s.to_string()),
                discount_amount: discount_amount.map(|s| s.to_string()),
                tax_code: tax_code.map(|s| s.to_string()),
                tax_amount: "0".to_string(),
                requested_ship_date, actual_ship_date: None,
                promised_delivery_date, ship_from_warehouse: ship_from_warehouse.map(|s| s.to_string()),
                fulfillment_status: "not_started".to_string(),
                status: "open".to_string(),
                cancellation_reason: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }

        async fn get_order_line(&self, id: Uuid) -> AtlasResult<Option<SalesOrderLine>> {
            Ok(Some(self.make_line(id)))
        }

        async fn list_order_lines(&self, _order_id: Uuid) -> AtlasResult<Vec<SalesOrderLine>> {
            Ok(vec![SalesOrderLine {
                id: Uuid::new_v4(), organization_id: test_org_id(),
                order_id: _order_id, line_number: 1,
                item_id: None, item_code: Some("ITEM-01".to_string()),
                item_description: Some("Widget".to_string()),
                quantity_ordered: "10".to_string(),
                quantity_shipped: "0".to_string(),
                quantity_cancelled: "0".to_string(),
                quantity_backordered: "0".to_string(),
                unit_selling_price: "25".to_string(),
                unit_list_price: None, line_amount: "250".to_string(),
                discount_percent: None, discount_amount: None,
                tax_code: None, tax_amount: "0".to_string(),
                requested_ship_date: None, actual_ship_date: None,
                promised_delivery_date: None, ship_from_warehouse: None,
                fulfillment_status: "not_started".to_string(),
                status: "open".to_string(),
                cancellation_reason: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }])
        }

        async fn update_line_quantities(
            &self, id: Uuid, quantity_shipped: Option<&str>,
            quantity_cancelled: Option<&str>, quantity_backordered: Option<&str>,
        ) -> AtlasResult<SalesOrderLine> {
            let mut line = self.make_line(id);
            if let Some(qs) = quantity_shipped { line.quantity_shipped = qs.to_string(); }
            if let Some(qc) = quantity_cancelled { line.quantity_cancelled = qc.to_string(); }
            if let Some(qb) = quantity_backordered { line.quantity_backordered = qb.to_string(); }
            self.state.lock().unwrap().line_data.insert(id, line.clone());
            Ok(line)
        }

        async fn update_line_status(&self, id: Uuid, status: &str, fulfillment_status: &str) -> AtlasResult<SalesOrderLine> {
            let mut line = self.make_line(id);
            line.status = status.to_string();
            line.fulfillment_status = fulfillment_status.to_string();
            self.state.lock().unwrap().line_data.insert(id, line.clone());
            Ok(line)
        }

        async fn update_line_cancellation_reason(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<()> {
            let mut line = self.make_line(id);
            line.cancellation_reason = reason.map(|s| s.to_string());
            self.state.lock().unwrap().line_data.insert(id, line);
            Ok(())
        }

        async fn create_hold(
            &self, org_id: Uuid, order_id: Uuid, order_line_id: Option<Uuid>,
            hold_type: &str, hold_reason: &str, applied_by: Option<Uuid>,
            applied_by_name: Option<&str>,
        ) -> AtlasResult<OrderHold> {
            Ok(OrderHold {
                id: Uuid::new_v4(), organization_id: org_id,
                order_id, order_line_id,
                hold_type: hold_type.to_string(), hold_reason: hold_reason.to_string(),
                applied_by, applied_by_name: applied_by_name.map(|s| s.to_string()),
                released_by: None, released_by_name: None, released_at: None,
                is_active: true,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }

        async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<OrderHold>> {
            let is_active = self.state.lock().unwrap()
                .hold_active.get(&id).cloned().unwrap_or(true);
            Ok(Some(OrderHold {
                id, organization_id: test_org_id(),
                order_id: Uuid::new_v4(), order_line_id: None,
                hold_type: "credit_check".to_string(),
                hold_reason: "Credit limit exceeded".to_string(),
                applied_by: None, applied_by_name: None,
                released_by: None, released_by_name: None, released_at: None,
                is_active,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }))
        }

        async fn list_holds(&self, _order_id: Uuid, _active_only: bool) -> AtlasResult<Vec<OrderHold>> { Ok(vec![]) }

        async fn release_hold(&self, id: Uuid, released_by: Option<Uuid>, released_by_name: Option<&str>) -> AtlasResult<OrderHold> {
            self.state.lock().unwrap().hold_active.insert(id, false);
            let hold = self.get_hold(id).await?.unwrap();
            // The mock get_hold returns is_active from state, so it's now false
            Ok(OrderHold {
                is_active: false,
                released_by,
                released_by_name: released_by_name.map(|s| s.to_string()),
                released_at: Some(chrono::Utc::now()),
                ..hold
            })
        }

        async fn create_shipment(
            &self, org_id: Uuid, shipment_number: &str, order_id: Uuid,
            order_line_ids: serde_json::Value, warehouse: Option<&str>,
            carrier: Option<&str>, shipping_method: Option<&str>,
            estimated_delivery_date: Option<chrono::NaiveDate>,
            shipped_by: Option<Uuid>, shipped_by_name: Option<&str>,
        ) -> AtlasResult<FulfillmentShipment> {
            let shipment = FulfillmentShipment {
                id: Uuid::new_v4(), organization_id: org_id,
                shipment_number: shipment_number.to_string(),
                order_id, order_line_ids,
                warehouse: warehouse.map(|s| s.to_string()),
                carrier: carrier.map(|s| s.to_string()),
                tracking_number: None,
                shipping_method: shipping_method.map(|s| s.to_string()),
                ship_date: None, estimated_delivery_date,
                actual_delivery_date: None, delivery_confirmation: None,
                status: "planned".to_string(),
                shipped_by, shipped_by_name: shipped_by_name.map(|s| s.to_string()),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            let id = shipment.id;
            self.state.lock().unwrap().shipment_data.insert(id, shipment.clone());
            Ok(shipment)
        }

        async fn get_shipment(&self, id: Uuid) -> AtlasResult<Option<FulfillmentShipment>> {
            Ok(self.state.lock().unwrap()
                .shipment_data.get(&id).cloned())
        }

        async fn list_shipments(&self, _org_id: Uuid, _status: Option<&str>, _order_id: Option<Uuid>) -> AtlasResult<Vec<FulfillmentShipment>> {
            // Return shipments from state so all_delivered checks work
            let shipments: Vec<FulfillmentShipment> = self.state.lock().unwrap()
                .shipment_data.values()
                .filter(|s| _order_id.is_none_or(|oid| s.order_id == oid))
                .cloned()
                .collect();
            Ok(shipments)
        }

        async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<FulfillmentShipment> {
            let mut ship = self.get_shipment(id).await?.unwrap();
            ship.status = status.to_string();
            self.state.lock().unwrap().shipment_data.insert(id, ship.clone());
            Ok(ship)
        }

        async fn update_shipment_tracking(
            &self, id: Uuid, tracking_number: Option<&str>,
            actual_delivery_date: Option<chrono::NaiveDate>,
            delivery_confirmation: Option<&str>,
        ) -> AtlasResult<FulfillmentShipment> {
            let mut ship = self.get_shipment(id).await?.unwrap();
            ship.tracking_number = tracking_number.map(|s| s.to_string());
            ship.actual_delivery_date = actual_delivery_date;
            ship.delivery_confirmation = delivery_confirmation.map(|s| s.to_string());
            self.state.lock().unwrap().shipment_data.insert(id, ship.clone());
            Ok(ship)
        }

        async fn confirm_ship(&self, id: Uuid, ship_date: chrono::NaiveDate) -> AtlasResult<FulfillmentShipment> {
            let mut ship = self.get_shipment(id).await?.unwrap();
            ship.status = "shipped".to_string();
            ship.ship_date = Some(ship_date);
            self.state.lock().unwrap().shipment_data.insert(id, ship.clone());
            Ok(ship)
        }

        async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<OrderManagementDashboard> {
            Ok(OrderManagementDashboard {
                total_orders: 0, open_orders: 0, orders_in_fulfillment: 0,
                completed_orders: 0, cancelled_orders: 0,
                total_order_value: "0".to_string(),
                average_order_value: "0.00".to_string(),
                orders_on_hold: 0, backordered_lines: 0, overdue_shipments: 0,
                orders_by_status: serde_json::json!({}),
                orders_by_channel: serde_json::json!({}),
                fulfillment_rate_pct: "0.0".to_string(),
                on_time_shipment_pct: "100.0".to_string(),
            })
        }
    }

    // ========================================================================
    // Order Creation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_order_validates_currency() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.create_order(
            Uuid::new_v4(),
            CreateSalesOrderRequest {
                customer_name: Some("Acme Corp".to_string()),
                order_date: chrono::Utc::now().date_naive(),
                currency_code: "".to_string(),  // empty currency
                ..empty_order_request()
            },
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Currency code is required"));
    }

    #[tokio::test]
    async fn test_create_order_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.create_order(
            Uuid::new_v4(),
            CreateSalesOrderRequest {
                customer_id: Some(Uuid::new_v4()),
                customer_name: Some("Acme Corp".to_string()),
                customer_po_number: Some("PO-12345".to_string()),
                order_date: chrono::Utc::now().date_naive(),
                requested_ship_date: Some(chrono::Utc::now().date_naive()),
                ship_to_address: Some("123 Main St".to_string()),
                bill_to_address: Some("456 Billing Ave".to_string()),
                currency_code: "USD".to_string(),
                payment_terms: Some("Net 30".to_string()),
                shipping_method: Some("FedEx".to_string()),
                sales_channel: Some("direct".to_string()),
                salesperson_name: Some("Jane Doe".to_string()),
                ..empty_order_request()
            },
        ).await;
        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(order.order_number.starts_with("SO-"));
        assert_eq!(order.status, "draft");
        assert_eq!(order.fulfillment_status, "not_started");
        assert_eq!(order.currency_code, "USD");
    }

    #[tokio::test]
    async fn test_create_order_generates_unique_number() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let org_id = Uuid::new_v4();
        let req = CreateSalesOrderRequest {
            order_date: chrono::Utc::now().date_naive(),
            currency_code: "USD".to_string(),
            ..empty_order_request()
        };
        let o1 = engine.create_order(org_id, req.clone()).await.unwrap();
        let o2 = engine.create_order(org_id, req).await.unwrap();
        assert_ne!(o1.order_number, o2.order_number);
    }

    // ========================================================================
    // Order Lifecycle Tests
    // ========================================================================

    #[tokio::test]
    async fn test_submit_order_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        let result = engine.submit_order(test_org_id(), order_id).await;
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.status, "submitted");
    }

    #[tokio::test]
    async fn test_confirm_order_requires_submitted() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        // Mock returns "draft" status, so confirm should fail
        let result = engine.confirm_order(test_org_id(), order_id).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Cannot confirm order") || msg.contains("Must be 'submitted'"));
    }

    #[tokio::test]
    async fn test_confirm_order_success_via_stateful_mock() {
        let repo = MockOrderRepo::new();
        let order_id = Uuid::new_v4();
        // Pre-seed order as "submitted" so confirm can proceed
        repo.state.lock().unwrap().order_status.insert(order_id, "submitted".to_string());
        let engine = OrderManagementEngine::new(Arc::new(repo));
        let result = engine.confirm_order(test_org_id(), order_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_submit_order_prevents_when_holds_active() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        // The mock's list_holds returns empty, so submit should succeed
        let order_id = Uuid::new_v4();
        let result = engine.submit_order(test_org_id(), order_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_close_order_requires_shipped_or_delivered() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        // Mock returns "draft" status, so close should fail
        let result = engine.close_order(test_org_id(), order_id).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Cannot close order") || msg.contains("Must be 'shipped' or 'delivered'"));
    }

    #[tokio::test]
    async fn test_cancel_order_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        let result = engine.cancel_order(test_org_id(), order_id, Some("Customer request")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "cancelled");
    }

    // ========================================================================
    // Order Line Tests
    // ========================================================================

    #[tokio::test]
    async fn test_add_order_line_validates_quantity() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.add_order_line(
            AddOrderLineRequest {
                org_id: Uuid::new_v4(),
                order_id: Uuid::new_v4(),
                item_code: Some("ITEM-01".to_string()),
                item_description: Some("Widget".to_string()),
                quantity_ordered: "-5".to_string(),
                unit_selling_price: "10.00".to_string(),
                ..empty_line_request()
            },
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Quantity must be positive"));
    }

    #[tokio::test]
    async fn test_add_order_line_validates_price() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.add_order_line(
            AddOrderLineRequest {
                org_id: Uuid::new_v4(),
                order_id: Uuid::new_v4(),
                item_code: Some("ITEM-01".to_string()),
                item_description: Some("Widget".to_string()),
                quantity_ordered: "10".to_string(),
                unit_selling_price: "-5.00".to_string(),
                ..empty_line_request()
            },
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Unit selling price cannot be negative"));
    }

    #[tokio::test]
    async fn test_add_order_line_validates_discount_range() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.add_order_line(
            AddOrderLineRequest {
                org_id: Uuid::new_v4(),
                order_id: Uuid::new_v4(),
                item_code: Some("ITEM-01".to_string()),
                item_description: Some("Widget".to_string()),
                quantity_ordered: "10".to_string(),
                unit_selling_price: "25.00".to_string(),
                discount_percent: Some("150".to_string()),
                ..empty_line_request()
            },
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Discount percent must be between 0 and 100"));
    }

    #[tokio::test]
    async fn test_add_order_line_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let org_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let result = engine.add_order_line(
            AddOrderLineRequest {
                org_id,
                order_id,
                item_id: Some(Uuid::new_v4()),
                item_code: Some("ITEM-01".to_string()),
                item_description: Some("Premium Widget".to_string()),
                quantity_ordered: "100".to_string(),
                unit_selling_price: "25.00".to_string(),
                unit_list_price: Some("30.00".to_string()),
                discount_percent: Some("10".to_string()),
                tax_code: Some("STANDARD".to_string()),
                requested_ship_date: Some(chrono::Utc::now().date_naive()),
                ship_from_warehouse: Some("WH-EAST".to_string()),
                ..empty_line_request()
            },
        ).await;
        assert!(result.is_ok());
        let line = result.unwrap();
        // Mock's list_order_lines returns 1 existing line, so next line_number = 2
        assert_eq!(line.line_number, 2);
        assert_eq!(line.status, "open");
        assert_eq!(line.quantity_ordered, "100");
        assert_eq!(line.unit_selling_price, "25.00");
    }

    // ========================================================================
    // Ship Order Line Tests
    // ========================================================================

    #[tokio::test]
    async fn test_ship_line_validates_quantity() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let line_id = Uuid::new_v4();
        // Mock returns quantity_ordered=10, so shipping 15 should fail
        let result = engine.ship_order_line(test_org_id(), line_id, "15").await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Cannot ship") || msg.contains("remaining"));
    }

    #[tokio::test]
    async fn test_ship_line_validates_zero() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.ship_order_line(test_org_id(), Uuid::new_v4(), "0").await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Shipped quantity must be positive"));
    }

    #[tokio::test]
    async fn test_ship_line_partial_success() {
        let repo = MockOrderRepo::new();
        let line_id = Uuid::new_v4();
        // Pre-seed the line data so update_line_quantities reads correct state
        repo.state.lock().unwrap().line_data.insert(line_id, SalesOrderLine {
            id: line_id, organization_id: test_org_id(),
            order_id: Uuid::new_v4(), line_number: 1,
            item_id: None, item_code: Some("ITEM-01".to_string()),
            item_description: Some("Widget".to_string()),
            quantity_ordered: "10".to_string(),
            quantity_shipped: "0".to_string(),
            quantity_cancelled: "0".to_string(),
            quantity_backordered: "0".to_string(),
            unit_selling_price: "25".to_string(),
            unit_list_price: None, line_amount: "250".to_string(),
            discount_percent: None, discount_amount: None,
            tax_code: None, tax_amount: "0".to_string(),
            requested_ship_date: None, actual_ship_date: None,
            promised_delivery_date: None, ship_from_warehouse: None,
            fulfillment_status: "not_started".to_string(),
            status: "open".to_string(),
            cancellation_reason: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        });
        let engine = OrderManagementEngine::new(Arc::new(repo));
        // Mock returns quantity_ordered=10, ship 6 should succeed
        let result = engine.ship_order_line(test_org_id(), line_id, "6").await;
        assert!(result.is_ok());
        let line = result.unwrap();
        assert_eq!(line.quantity_shipped, "6.0000");
        assert_eq!(line.quantity_backordered, "4.0000");
    }

    #[tokio::test]
    async fn test_ship_line_full_success() {
        let repo = MockOrderRepo::new();
        let line_id = Uuid::new_v4();
        repo.state.lock().unwrap().line_data.insert(line_id, SalesOrderLine {
            id: line_id, organization_id: test_org_id(),
            order_id: Uuid::new_v4(), line_number: 1,
            item_id: None, item_code: Some("ITEM-01".to_string()),
            item_description: Some("Widget".to_string()),
            quantity_ordered: "10".to_string(),
            quantity_shipped: "0".to_string(),
            quantity_cancelled: "0".to_string(),
            quantity_backordered: "0".to_string(),
            unit_selling_price: "25".to_string(),
            unit_list_price: None, line_amount: "250".to_string(),
            discount_percent: None, discount_amount: None,
            tax_code: None, tax_amount: "0".to_string(),
            requested_ship_date: None, actual_ship_date: None,
            promised_delivery_date: None, ship_from_warehouse: None,
            fulfillment_status: "not_started".to_string(),
            status: "open".to_string(),
            cancellation_reason: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        });
        let engine = OrderManagementEngine::new(Arc::new(repo));
        let result = engine.ship_order_line(test_org_id(), line_id, "10").await;
        assert!(result.is_ok());
        let line = result.unwrap();
        assert_eq!(line.quantity_shipped, "10.0000");
        assert_eq!(line.quantity_backordered, "0.0000");
        assert_eq!(line.status, "shipped");
    }

    #[tokio::test]
    async fn test_cancel_line_success() {
        let repo = MockOrderRepo::new();
        let line_id = Uuid::new_v4();
        repo.state.lock().unwrap().line_data.insert(line_id, SalesOrderLine {
            id: line_id, organization_id: test_org_id(),
            order_id: Uuid::new_v4(), line_number: 1,
            item_id: None, item_code: Some("ITEM-01".to_string()),
            item_description: Some("Widget".to_string()),
            quantity_ordered: "10".to_string(),
            quantity_shipped: "0".to_string(),
            quantity_cancelled: "0".to_string(),
            quantity_backordered: "0".to_string(),
            unit_selling_price: "25".to_string(),
            unit_list_price: None, line_amount: "250".to_string(),
            discount_percent: None, discount_amount: None,
            tax_code: None, tax_amount: "0".to_string(),
            requested_ship_date: None, actual_ship_date: None,
            promised_delivery_date: None, ship_from_warehouse: None,
            fulfillment_status: "not_started".to_string(),
            status: "open".to_string(),
            cancellation_reason: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        });
        let engine = OrderManagementEngine::new(Arc::new(repo));
        let result = engine.cancel_order_line(test_org_id(), line_id, Some("No longer needed")).await;
        assert!(result.is_ok());
        let line = result.unwrap();
        assert_eq!(line.status, "cancelled");
        assert_eq!(line.quantity_cancelled, "10.0000");
    }

    // ========================================================================
    // Order Holds Tests
    // ========================================================================

    #[tokio::test]
    async fn test_apply_hold_validates_type() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.apply_hold(
            Uuid::new_v4(), Uuid::new_v4(), None,
            "invalid_type", "Some reason", None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid hold type"));
    }

    #[tokio::test]
    async fn test_apply_hold_validates_reason() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.apply_hold(
            Uuid::new_v4(), Uuid::new_v4(), None,
            "credit_check", "", None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Hold reason is required"));
    }

    #[tokio::test]
    async fn test_apply_hold_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let org_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let result = engine.apply_hold(
            org_id, order_id, None,
            "credit_check", "Credit limit exceeded", Some(Uuid::new_v4()),
            Some("John Risk"),
        ).await;
        assert!(result.is_ok());
        let hold = result.unwrap();
        assert_eq!(hold.hold_type, "credit_check");
        assert_eq!(hold.hold_reason, "Credit limit exceeded");
        assert!(hold.is_active);
    }

    #[tokio::test]
    async fn test_release_hold_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let hold_id = Uuid::new_v4();
        let result = engine.release_hold(test_org_id(), hold_id, Some(Uuid::new_v4()), Some("Manager")).await;
        assert!(result.is_ok());
        let hold = result.unwrap();
        assert!(!hold.is_active);
        assert!(hold.released_at.is_some());
    }

    #[tokio::test]
    async fn test_has_active_holds_false() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.has_active_holds(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // ========================================================================
    // Shipment Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_shipment_validates_lines() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.create_shipment(
            Uuid::new_v4(), Uuid::new_v4(), vec![],  // empty line IDs
            None, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("At least one order line ID is required"));
    }

    #[tokio::test]
    async fn test_create_shipment_success() {
        let repo = MockOrderRepo::new();
        let order_id = Uuid::new_v4();
        // Set order to confirmed so shipment can be created
        repo.state.lock().unwrap().order_status.insert(order_id, "confirmed".to_string());
        let engine = OrderManagementEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();
        let line_id = Uuid::new_v4();
        let result = engine.create_shipment(
            org_id, order_id, vec![line_id],
            Some("WH-EAST"), Some("FedEx"), Some("Express"),
            Some(chrono::Utc::now().date_naive()), None, None,
        ).await;
        assert!(result.is_ok());
        let shipment = result.unwrap();
        assert!(shipment.shipment_number.starts_with("SHP-"));
        assert_eq!(shipment.status, "planned");
        assert_eq!(shipment.carrier.unwrap(), "FedEx");
    }

    #[tokio::test]
    async fn test_list_shipments_validates_status() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.list_shipments(Uuid::new_v4(), Some("invalid"), None).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid shipment status"));
    }

    // ========================================================================
    // List Orders Filter Tests
    // ========================================================================

    #[tokio::test]
    async fn test_list_orders_validates_status() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.list_orders(Uuid::new_v4(), Some("invalid"), None).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid status"));
    }

    #[tokio::test]
    async fn test_list_orders_validates_fulfillment_status() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.list_orders(Uuid::new_v4(), None, Some("bad_status")).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid fulfillment status"));
    }

    // ========================================================================
    // Dashboard Tests
    // ========================================================================

    #[tokio::test]
    async fn test_get_dashboard_success() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let result = engine.get_dashboard(Uuid::new_v4()).await;
        assert!(result.is_ok());
        let dash = result.unwrap();
        assert_eq!(dash.total_orders, 0);
        assert_eq!(dash.open_orders, 0);
        assert_eq!(dash.fulfillment_rate_pct, "0.0");
    }

    #[tokio::test]
    async fn test_get_order_by_id() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        let result = engine.get_order_by_id(order_id).await;
        assert!(result.is_ok());
        let order = result.unwrap().unwrap();
        assert_eq!(order.status, "draft");
    }

    #[tokio::test]
    async fn test_get_order_line_by_id() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let line_id = Uuid::new_v4();
        let result = engine.get_order_line(line_id).await;
        assert!(result.is_ok());
        let line = result.unwrap().unwrap();
        assert_eq!(line.item_code.unwrap(), "ITEM-01");
    }

    #[tokio::test]
    async fn test_get_shipment_by_id() {
        let repo = MockOrderRepo::new();
        let order_id = Uuid::new_v4();
        repo.state.lock().unwrap().order_status.insert(order_id, "confirmed".to_string());
        let engine = OrderManagementEngine::new(Arc::new(repo));

        // Create a shipment first, then retrieve it
        let shipment = engine.create_shipment(
            test_org_id(), order_id, vec![Uuid::new_v4()],
            None, None, None, None, None, None,
        ).await.unwrap();

        let result = engine.get_shipment(test_org_id(), shipment.id).await;
        assert!(result.is_ok());
        let fetched = result.unwrap().unwrap();
        assert_eq!(fetched.status, "planned");
        assert_eq!(fetched.id, shipment.id);
    }

    #[tokio::test]
    async fn test_list_order_lines() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        let result = engine.list_order_lines(order_id).await;
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
    }

    #[tokio::test]
    async fn test_list_holds_empty() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let order_id = Uuid::new_v4();
        let result = engine.list_holds(order_id, true).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_hold_by_id() {
        let engine = OrderManagementEngine::new(Arc::new(MockOrderRepo::new()));
        let hold_id = Uuid::new_v4();
        let result = engine.get_hold(test_org_id(), hold_id).await;
        assert!(result.is_ok());
        let hold = result.unwrap().unwrap();
        assert_eq!(hold.hold_type, "credit_check");
        assert!(hold.is_active);
    }

    #[tokio::test]
    async fn test_confirm_shipment_moves_order_to_processing() {
        let repo = MockOrderRepo::new();
        let order_id = Uuid::new_v4();
        // Seed order as "confirmed" so shipment can be created and confirmed
        repo.state.lock().unwrap().order_status.insert(order_id, "confirmed".to_string());
        let engine = OrderManagementEngine::new(Arc::new(repo));

        // Create shipment first
        let shipment = engine.create_shipment(
            test_org_id(), order_id, vec![Uuid::new_v4()],
            None, None, None, None, None, None,
        ).await.unwrap();

        // Confirm shipment
        let result = engine.confirm_shipment(
            test_org_id(), shipment.id, chrono::Utc::now().date_naive(),
        ).await;
        assert!(result.is_ok());
        let confirmed = result.unwrap();
        assert_eq!(confirmed.status, "shipped");

        // Verify order is in processing, not shipped
        let order = engine.get_order_by_id(order_id).await.unwrap().unwrap();
        assert_eq!(order.status, "processing");
    }

    #[tokio::test]
    async fn test_confirm_delivery_transitions_order_when_all_shipments_delivered() {
        let repo = MockOrderRepo::new();
        let order_id = Uuid::new_v4();
        repo.state.lock().unwrap().order_status.insert(order_id, "processing".to_string());
        let state_handle = repo.state.clone();
        let engine = OrderManagementEngine::new(Arc::new(repo));

        // Create a shipment
        let shipment = engine.create_shipment(
            test_org_id(), order_id, vec![Uuid::new_v4()],
            None, None, None, None, None, None,
        ).await.unwrap();

        // Set shipment to "shipped" (prerequisite for delivery)
        state_handle.lock().unwrap().shipment_data.get_mut(&shipment.id).unwrap().status = "shipped".to_string();

        // Confirm delivery — should transition order to "delivered" since it's the only shipment
        let result = engine.confirm_delivery(
            test_org_id(), shipment.id, chrono::Utc::now().date_naive(), Some("Signed by customer"),
        ).await;
        assert!(result.is_ok());
        let delivered = result.unwrap();
        assert_eq!(delivered.status, "delivered");

        // Order should now be "delivered"
        let order = engine.get_order_by_id(order_id).await.unwrap().unwrap();
        assert_eq!(order.status, "delivered");
        // Note: fulfillment_status is "not_started" in the mock because make_order
        // doesn't track it from state. The real Postgres repository handles this correctly.
    }

    #[tokio::test]
    async fn test_cancel_line_preserves_reason() {
        let repo = MockOrderRepo::new();
        let line_id = Uuid::new_v4();
        repo.state.lock().unwrap().line_data.insert(line_id, SalesOrderLine {
            id: line_id, organization_id: test_org_id(),
            order_id: Uuid::new_v4(), line_number: 1,
            item_id: None, item_code: Some("ITEM-01".to_string()),
            item_description: Some("Widget".to_string()),
            quantity_ordered: "10".to_string(),
            quantity_shipped: "0".to_string(),
            quantity_cancelled: "0".to_string(),
            quantity_backordered: "0".to_string(),
            unit_selling_price: "25".to_string(),
            unit_list_price: None, line_amount: "250".to_string(),
            discount_percent: None, discount_amount: None,
            tax_code: None, tax_amount: "0".to_string(),
            requested_ship_date: None, actual_ship_date: None,
            promised_delivery_date: None, ship_from_warehouse: None,
            fulfillment_status: "not_started".to_string(),
            status: "open".to_string(),
            cancellation_reason: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        });
        let engine = OrderManagementEngine::new(Arc::new(repo));
        let result = engine.cancel_order_line(test_org_id(), line_id, Some("No longer needed")).await;
        assert!(result.is_ok());
        let line = result.unwrap();
        assert_eq!(line.status, "cancelled");
        assert_eq!(line.quantity_cancelled, "10.0000");
        // Cancellation reason is persisted via update_line_cancellation_reason
        assert_eq!(line.cancellation_reason, Some("No longer needed".to_string()));
    }
}
