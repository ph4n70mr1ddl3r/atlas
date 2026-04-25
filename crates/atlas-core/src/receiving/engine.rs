//! Receiving Engine
//!
//! Manages goods receipt processing: receiving locations, receipt creation,
//! line-level receiving, quality inspections, delivery/putaway to subinventory,
//! and return-to-supplier (RTV) workflows.
//!
//! Oracle Fusion Cloud SCM equivalent: SCM > Receiving

use atlas_shared::{
    ReceivingLocation, ReceiptHeader, ReceiptLine, ReceiptInspection,
    InspectionDetail, ReceiptDelivery, ReceiptReturn, ReceivingDashboard,
    AtlasError, AtlasResult,
};
use super::ReceivingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid receipt types
#[allow(dead_code)]
const VALID_RECEIPT_TYPES: &[&str] = &["standard", "blind", "express", "return"];

/// Valid receipt sources
#[allow(dead_code)]
const VALID_RECEIPT_SOURCES: &[&str] = &["purchase_order", "internal", "customer_return"];

/// Valid receipt statuses
const VALID_RECEIPT_STATUSES: &[&str] = &[
    "draft", "received", "partially_inspected", "inspected",
    "partially_delivered", "delivered", "closed", "cancelled",
];

/// Valid inspection statuses
#[allow(dead_code)]
const VALID_INSPECTION_STATUSES: &[&str] = &["pending", "in_progress", "completed", "cancelled"];

/// Valid inspection dispositions
const VALID_DISPOSITIONS: &[&str] = &["accept", "reject", "conditional", "scrap", "rework"];

/// Valid delivery statuses
#[allow(dead_code)]
const VALID_DELIVERY_STATUSES: &[&str] = &["pending", "delivered", "cancelled"];

/// Valid delivery destination types
#[allow(dead_code)]
const VALID_DESTINATION_TYPES: &[&str] = &["inventory", "expense", "receiving", "job_shop"];

/// Valid return types
const VALID_RETURN_TYPES: &[&str] = &["reject", "excess", "wrong_item", "damaged", "other"];

/// Valid return statuses
const VALID_RETURN_STATUSES: &[&str] = &[
    "draft", "submitted", "shipped", "received_by_supplier", "credited", "cancelled",
];

/// Valid location types
#[allow(dead_code)]
const VALID_LOCATION_TYPES: &[&str] = &["warehouse", "dock", "staging_area", "inspection"];

/// Valid check types
#[allow(dead_code)]
const VALID_CHECK_TYPES: &[&str] = &["visual", "measurement", "functional", "documentation"];

/// Valid check results
#[allow(dead_code)]
const VALID_CHECK_RESULTS: &[&str] = &["pass", "fail", "conditional", "n_a"];

/// Receiving Management engine
pub struct ReceivingEngine {
    repository: Arc<dyn ReceivingRepository>,
}

impl ReceivingEngine {
    pub fn new(repository: Arc<dyn ReceivingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Receiving Locations
    // ========================================================================

    /// Create a receiving location
    pub async fn create_location(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        location_type: &str,
        address: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        country: Option<&str>,
        postal_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceivingLocation> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Location code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Location name is required".to_string()));
        }
        if !VALID_LOCATION_TYPES.contains(&location_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid location type '{}'. Must be one of: {}",
                location_type, VALID_LOCATION_TYPES.join(", ")
            )));
        }

        info!("Creating receiving location {} in org {}", code, org_id);
        self.repository.create_location(
            org_id, code, name, description, location_type,
            address, city, state, country, postal_code, created_by,
        ).await
    }

    /// Get a receiving location by code
    pub async fn get_location(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReceivingLocation>> {
        self.repository.get_location(org_id, code).await
    }

    /// List receiving locations
    pub async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<ReceivingLocation>> {
        self.repository.list_locations(org_id).await
    }

    /// Delete (deactivate) a receiving location
    pub async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_location(org_id, code).await
    }

    // ========================================================================
    // Receipt Headers
    // ========================================================================

    /// Create a new receipt header
    pub async fn create_receipt(
        &self,
        org_id: Uuid,
        receipt_number: &str,
        receipt_type: &str,
        receipt_source: &str,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        supplier_number: Option<&str>,
        purchase_order_id: Option<Uuid>,
        purchase_order_number: Option<&str>,
        receiving_location_id: Option<Uuid>,
        receiving_location_code: Option<&str>,
        receiving_date: chrono::NaiveDate,
        packing_slip_number: Option<&str>,
        bill_of_lading: Option<&str>,
        carrier: Option<&str>,
        tracking_number: Option<&str>,
        waybill_number: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptHeader> {
        if receipt_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Receipt number is required".to_string()));
        }
        if !VALID_RECEIPT_TYPES.contains(&receipt_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt type '{}'. Must be one of: {}",
                receipt_type, VALID_RECEIPT_TYPES.join(", ")
            )));
        }
        if !VALID_RECEIPT_SOURCES.contains(&receipt_source) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt source '{}'. Must be one of: {}",
                receipt_source, VALID_RECEIPT_SOURCES.join(", ")
            )));
        }

        info!("Creating receipt {} in org {}", receipt_number, org_id);
        self.repository.create_receipt(
            org_id, receipt_number, receipt_type, receipt_source,
            supplier_id, supplier_name, supplier_number,
            purchase_order_id, purchase_order_number,
            receiving_location_id, receiving_location_code,
            receiving_date, packing_slip_number, bill_of_lading,
            carrier, tracking_number, waybill_number, notes, created_by,
        ).await
    }

    /// Get a receipt by ID
    pub async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ReceiptHeader>> {
        self.repository.get_receipt(id).await
    }

    /// Get a receipt by number
    pub async fn get_receipt_by_number(&self, org_id: Uuid, receipt_number: &str) -> AtlasResult<Option<ReceiptHeader>> {
        self.repository.get_receipt_by_number(org_id, receipt_number).await
    }

    /// List receipts with optional filters
    pub async fn list_receipts(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ReceiptHeader>> {
        if let Some(s) = status {
            if !VALID_RECEIPT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RECEIPT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_receipts(org_id, status, supplier_id).await
    }

    /// Confirm receipt of goods (transition to 'received')
    pub async fn confirm_receipt(&self, receipt_id: Uuid, received_by: Uuid) -> AtlasResult<ReceiptHeader> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot confirm receipt in '{}' status. Must be 'draft'.", receipt.status)
            ));
        }

        info!("Confirming receipt {}", receipt.receipt_number);
        self.repository.update_receipt_status(
            receipt_id, "received", Some(received_by), None,
        ).await
    }

    /// Close a receipt
    pub async fn close_receipt(&self, receipt_id: Uuid) -> AtlasResult<ReceiptHeader> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status == "closed" || receipt.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Receipt is already '{}'", receipt.status)
            ));
        }

        info!("Closing receipt {}", receipt.receipt_number);
        self.repository.update_receipt_status(
            receipt_id, "closed", None, None,
        ).await
    }

    /// Cancel a receipt
    pub async fn cancel_receipt(&self, receipt_id: Uuid) -> AtlasResult<ReceiptHeader> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel receipt in '{}' status. Must be 'draft'.", receipt.status)
            ));
        }

        info!("Cancelling receipt {}", receipt.receipt_number);
        self.repository.update_receipt_status(
            receipt_id, "cancelled", None, None,
        ).await
    }

    // ========================================================================
    // Receipt Lines
    // ========================================================================

    /// Add a line to a receipt
    pub async fn add_receipt_line(
        &self,
        org_id: Uuid,
        receipt_id: Uuid,
        purchase_order_line_id: Option<Uuid>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        ordered_qty: &str,
        ordered_uom: Option<&str>,
        received_qty: &str,
        received_uom: Option<&str>,
        lot_number: Option<&str>,
        serial_numbers: serde_json::Value,
        expiration_date: Option<chrono::NaiveDate>,
        manufacture_date: Option<chrono::NaiveDate>,
        unit_price: Option<&str>,
        currency: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptLine> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "draft" && receipt.status != "received" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to receipt in '{}' status", receipt.status)
            ));
        }

        let received: f64 = received_qty.parse().map_err(|_| AtlasError::ValidationFailed(
            "Received quantity must be a valid number".to_string(),
        ))?;
        if received < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Received quantity must be non-negative".to_string(),
            ));
        }

        // Get next line number
        let existing_lines = self.repository.list_receipt_lines(receipt_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to receipt {}", line_number, receipt.receipt_number);

        self.repository.create_receipt_line(
            org_id, receipt_id, line_number,
            purchase_order_line_id, item_id, item_code, item_description,
            ordered_qty, ordered_uom, received_qty, received_uom,
            lot_number, serial_numbers, expiration_date, manufacture_date,
            unit_price, currency, notes, created_by,
        ).await
    }

    /// List lines for a receipt
    pub async fn list_receipt_lines(&self, receipt_id: Uuid) -> AtlasResult<Vec<ReceiptLine>> {
        self.repository.list_receipt_lines(receipt_id).await
    }

    /// Get a receipt line by ID
    pub async fn get_receipt_line(&self, id: Uuid) -> AtlasResult<Option<ReceiptLine>> {
        self.repository.get_receipt_line(id).await
    }

    // ========================================================================
    // Inspections
    // ========================================================================

    /// Create an inspection for a receipt line
    pub async fn create_inspection(
        &self,
        org_id: Uuid,
        receipt_id: Uuid,
        receipt_line_id: Uuid,
        inspection_template: Option<&str>,
        inspector_id: Option<Uuid>,
        inspector_name: Option<&str>,
        inspection_date: chrono::NaiveDate,
        sample_size: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptInspection> {
        let _line = self.repository.get_receipt_line(receipt_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt line {} not found", receipt_line_id)
            ))?;

        let inspection_number = format!("INS-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating inspection {} for line {}", inspection_number, receipt_line_id);

        self.repository.create_inspection(
            org_id, receipt_id, receipt_line_id, &inspection_number,
            inspection_template, inspector_id, inspector_name,
            inspection_date, sample_size, notes, created_by,
        ).await
    }

    /// Get an inspection by ID
    pub async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<ReceiptInspection>> {
        self.repository.get_inspection(id).await
    }

    /// List inspections for a receipt
    pub async fn list_inspections(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptInspection>> {
        self.repository.list_inspections(org_id, receipt_id).await
    }

    /// Complete an inspection with results
    pub async fn complete_inspection(
        &self,
        inspection_id: Uuid,
        quantity_inspected: &str,
        quantity_accepted: &str,
        quantity_rejected: &str,
        disposition: &str,
        quality_score: Option<&str>,
        rejection_reason: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<ReceiptInspection> {
        let inspection = self.repository.get_inspection(inspection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Inspection {} not found", inspection_id)
            ))?;

        if inspection.status != "pending" && inspection.status != "in_progress" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete inspection in '{}' status", inspection.status)
            ));
        }

        if !VALID_DISPOSITIONS.contains(&disposition) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid disposition '{}'. Must be one of: {}",
                disposition, VALID_DISPOSITIONS.join(", ")
            )));
        }

        let inspected: f64 = quantity_inspected.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity inspected must be a valid number".to_string(),
        ))?;
        let accepted: f64 = quantity_accepted.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity accepted must be a valid number".to_string(),
        ))?;
        let rejected: f64 = quantity_rejected.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity rejected must be a valid number".to_string(),
        ))?;

        if (accepted + rejected - inspected).abs() > 0.01 {
            return Err(AtlasError::ValidationFailed(
                "Quantity accepted + rejected must equal quantity inspected".to_string(),
            ));
        }

        if let Some(score) = quality_score {
            let s: f64 = score.parse().unwrap_or(-1.0);
            if s < 0.0 || s > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "Quality score must be between 0 and 100".to_string(),
                ));
            }
        }

        info!("Completing inspection {} with disposition {}", inspection.inspection_number, disposition);

        self.repository.complete_inspection(
            inspection_id,
            quantity_inspected, quantity_accepted, quantity_rejected,
            disposition, quality_score, rejection_reason, notes,
        ).await
    }

    // ========================================================================
    // Inspection Details (Quality Checks)
    // ========================================================================

    /// Add a quality check detail to an inspection
    pub async fn add_inspection_detail(
        &self,
        org_id: Uuid,
        inspection_id: Uuid,
        check_name: &str,
        check_type: &str,
        specification: Option<&str>,
        result: &str,
        measured_value: Option<&str>,
        expected_value: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<InspectionDetail> {
        if !VALID_CHECK_TYPES.contains(&check_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid check type '{}'. Must be one of: {}",
                check_type, VALID_CHECK_TYPES.join(", ")
            )));
        }
        if !VALID_CHECK_RESULTS.contains(&result) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid result '{}'. Must be one of: {}",
                result, VALID_CHECK_RESULTS.join(", ")
            )));
        }

        let details = self.repository.list_inspection_details(inspection_id).await?;
        let check_number = (details.len() as i32) + 1;

        self.repository.create_inspection_detail(
            org_id, inspection_id, check_number,
            check_name, check_type, specification, result,
            measured_value, expected_value, notes,
        ).await
    }

    /// List inspection details
    pub async fn list_inspection_details(&self, inspection_id: Uuid) -> AtlasResult<Vec<InspectionDetail>> {
        self.repository.list_inspection_details(inspection_id).await
    }

    // ========================================================================
    // Deliveries
    // ========================================================================

    /// Create a delivery (putaway) for a receipt line
    pub async fn create_delivery(
        &self,
        org_id: Uuid,
        receipt_id: Uuid,
        receipt_line_id: Uuid,
        subinventory: Option<&str>,
        locator: Option<&str>,
        quantity_delivered: &str,
        uom: Option<&str>,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        delivered_by: Option<Uuid>,
        delivered_by_name: Option<&str>,
        destination_type: &str,
        account_code: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptDelivery> {
        let _line = self.repository.get_receipt_line(receipt_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt line {} not found", receipt_line_id)
            ))?;

        if !VALID_DESTINATION_TYPES.contains(&destination_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid destination type '{}'. Must be one of: {}",
                destination_type, VALID_DESTINATION_TYPES.join(", ")
            )));
        }

        let qty: f64 = quantity_delivered.parse().map_err(|_| AtlasError::ValidationFailed(
            "Delivered quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Delivered quantity must be positive".to_string(),
            ));
        }

        let delivery_number = format!("DEL-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating delivery {} for receipt line {}", delivery_number, receipt_line_id);

        self.repository.create_delivery(
            org_id, receipt_id, receipt_line_id, &delivery_number,
            subinventory, locator, quantity_delivered, uom,
            lot_number, serial_number, delivered_by, delivered_by_name,
            destination_type, account_code, notes, created_by,
        ).await
    }

    /// List deliveries for a receipt
    pub async fn list_deliveries(&self, org_id: Uuid, receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptDelivery>> {
        self.repository.list_deliveries(org_id, receipt_id).await
    }

    /// Get a delivery by ID
    pub async fn get_delivery(&self, id: Uuid) -> AtlasResult<Option<ReceiptDelivery>> {
        self.repository.get_delivery(id).await
    }

    // ========================================================================
    // Returns to Supplier
    // ========================================================================

    /// Create a return-to-supplier request
    pub async fn create_return(
        &self,
        org_id: Uuid,
        receipt_id: Option<Uuid>,
        receipt_line_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        return_type: &str,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity_returned: &str,
        uom: Option<&str>,
        unit_price: Option<&str>,
        currency: Option<&str>,
        return_reason: Option<&str>,
        return_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptReturn> {
        if !VALID_RETURN_TYPES.contains(&return_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid return type '{}'. Must be one of: {}",
                return_type, VALID_RETURN_TYPES.join(", ")
            )));
        }
        let qty: f64 = quantity_returned.parse().map_err(|_| AtlasError::ValidationFailed(
            "Return quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Return quantity must be positive".to_string(),
            ));
        }

        let return_number = format!("RTV-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating return {} for supplier {:?}", return_number, supplier_id);

        self.repository.create_return(
            org_id, &return_number, receipt_id, receipt_line_id,
            supplier_id, supplier_name, return_type,
            item_id, item_code, item_description,
            quantity_returned, uom, unit_price, currency,
            return_reason, return_date, created_by,
        ).await
    }

    /// Get a return by ID
    pub async fn get_return(&self, id: Uuid) -> AtlasResult<Option<ReceiptReturn>> {
        self.repository.get_return(id).await
    }

    /// List returns
    pub async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptReturn>> {
        if let Some(s) = status {
            if !VALID_RETURN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RETURN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_returns(org_id, status).await
    }

    /// Submit a return for processing
    pub async fn submit_return(&self, return_id: Uuid) -> AtlasResult<ReceiptReturn> {
        let rtv = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Return {} not found", return_id)
            ))?;

        if rtv.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit return in '{}' status. Must be 'draft'.", rtv.status)
            ));
        }

        info!("Submitting return {}", rtv.return_number);
        self.repository.update_return_status(return_id, "submitted", None, None).await
    }

    /// Mark return as shipped
    pub async fn ship_return(&self, return_id: Uuid, carrier: Option<&str>, tracking_number: Option<&str>) -> AtlasResult<ReceiptReturn> {
        let rtv = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Return {} not found", return_id)
            ))?;

        if rtv.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot ship return in '{}' status. Must be 'submitted'.", rtv.status)
            ));
        }

        info!("Shipping return {}", rtv.return_number);
        self.repository.update_return_status(return_id, "shipped", carrier, tracking_number).await
    }

    /// Mark return as credited
    pub async fn credit_return(&self, return_id: Uuid) -> AtlasResult<ReceiptReturn> {
        let rtv = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Return {} not found", return_id)
            ))?;

        if rtv.status != "shipped" && rtv.status != "received_by_supplier" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot credit return in '{}' status.", rtv.status)
            ));
        }

        info!("Crediting return {}", rtv.return_number);
        self.repository.update_return_status(return_id, "credited", None, None).await
    }

    /// Cancel a return
    pub async fn cancel_return(&self, return_id: Uuid) -> AtlasResult<ReceiptReturn> {
        let rtv = self.repository.get_return(return_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Return {} not found", return_id)
            ))?;

        if rtv.status != "draft" && rtv.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel return in '{}' status.", rtv.status)
            ));
        }

        info!("Cancelling return {}", rtv.return_number);
        self.repository.update_return_status(return_id, "cancelled", None, None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get receiving dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ReceivingDashboard> {
        let receipts = self.repository.list_receipts(org_id, None, None).await?;
        let returns = self.repository.list_returns(org_id, None).await?;

        let total_receipts = receipts.len() as i32;
        let pending_receipts = receipts.iter().filter(|r| r.status == "draft").count() as i32;
        let received_today = receipts.iter().filter(|r| {
            r.status == "received" &&
            r.received_at.map_or(false, |d| d.date_naive() == chrono::Utc::now().date_naive())
        }).count() as i32;

        let pending_inspections = self.repository.list_inspections(org_id, None).await?
            .iter().filter(|i| i.status == "pending" || i.status == "in_progress").count() as i32;

        let pending_deliveries = self.repository.list_deliveries(org_id, None).await?
            .iter().filter(|d| d.status == "pending").count() as i32;

        let total_returns = returns.len() as i32;

        // Group receipts by status
        let mut status_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for r in &receipts {
            *status_counts.entry(r.status.clone()).or_insert(0) += 1;
        }
        let receipts_by_status: serde_json::Value = status_counts.into_iter()
            .map(|(k, v)| serde_json::json!({"status": k, "count": v}))
            .collect();

        // Top suppliers by receipt count
        let mut supplier_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for r in &receipts {
            if let Some(name) = &r.supplier_name {
                *supplier_counts.entry(name.clone()).or_insert(0) += 1;
            }
        }
        let mut supplier_vec: Vec<_> = supplier_counts.into_iter().collect();
        supplier_vec.sort_by(|a, b| b.1.cmp(&a.1));
        supplier_vec.truncate(5);
        let top_suppliers: serde_json::Value = supplier_vec.into_iter()
            .map(|(name, count)| serde_json::json!({"supplier_name": name, "receipt_count": count}))
            .collect();

        // Recent receipts
        let mut recent = receipts.clone();
        recent.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent.truncate(5);
        let recent_receipts: serde_json::Value = recent.iter().map(|r| serde_json::json!({
            "id": r.id,
            "receipt_number": r.receipt_number,
            "status": r.status,
            "supplier_name": r.supplier_name,
            "receiving_date": r.receiving_date,
        })).collect();

        Ok(ReceivingDashboard {
            total_receipts,
            pending_receipts,
            received_today,
            pending_inspections,
            pending_deliveries,
            total_returns,
            receipts_by_status,
            top_suppliers,
            recent_receipts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_receipt_types() {
        assert!(VALID_RECEIPT_TYPES.contains(&"standard"));
        assert!(VALID_RECEIPT_TYPES.contains(&"blind"));
        assert!(VALID_RECEIPT_TYPES.contains(&"express"));
        assert!(VALID_RECEIPT_TYPES.contains(&"return"));
    }

    #[test]
    fn test_valid_receipt_statuses() {
        assert!(VALID_RECEIPT_STATUSES.contains(&"draft"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"received"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"inspected"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"delivered"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"closed"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_dispositions() {
        assert!(VALID_DISPOSITIONS.contains(&"accept"));
        assert!(VALID_DISPOSITIONS.contains(&"reject"));
        assert!(VALID_DISPOSITIONS.contains(&"conditional"));
        assert!(VALID_DISPOSITIONS.contains(&"scrap"));
        assert!(VALID_DISPOSITIONS.contains(&"rework"));
    }

    #[test]
    fn test_valid_return_types() {
        assert!(VALID_RETURN_TYPES.contains(&"reject"));
        assert!(VALID_RETURN_TYPES.contains(&"excess"));
        assert!(VALID_RETURN_TYPES.contains(&"wrong_item"));
        assert!(VALID_RETURN_TYPES.contains(&"damaged"));
        assert!(VALID_RETURN_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_return_statuses() {
        assert!(VALID_RETURN_STATUSES.contains(&"draft"));
        assert!(VALID_RETURN_STATUSES.contains(&"submitted"));
        assert!(VALID_RETURN_STATUSES.contains(&"shipped"));
        assert!(VALID_RETURN_STATUSES.contains(&"received_by_supplier"));
        assert!(VALID_RETURN_STATUSES.contains(&"credited"));
        assert!(VALID_RETURN_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_location_types() {
        assert!(VALID_LOCATION_TYPES.contains(&"warehouse"));
        assert!(VALID_LOCATION_TYPES.contains(&"dock"));
        assert!(VALID_LOCATION_TYPES.contains(&"staging_area"));
        assert!(VALID_LOCATION_TYPES.contains(&"inspection"));
    }
}
