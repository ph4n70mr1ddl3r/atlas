//! Customer Returns Engine Implementation
//!
//! Manages return reason codes, Return Material Authorizations (RMAs),
//! return receipt and inspection, credit memo generation, and returns analytics.
//!
//! Oracle Fusion Cloud ERP equivalent: Order Management > Returns

use atlas_shared::{
    ReturnReason, ReturnAuthorization, ReturnLine, CreditMemo,
    ReturnsDashboardSummary,
    AtlasError, AtlasResult,
};
use super::CustomerReturnsRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid return types for RMAs
#[allow(dead_code)]
const VALID_RETURN_TYPES: &[&str] = &[
    "standard_return", "exchange", "repair", "warranty",
];

/// Valid RMA statuses
#[allow(dead_code)]
const VALID_RMA_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected",
    "partially_received", "received", "closed", "cancelled",
];

/// Valid disposition options
#[allow(dead_code)]
const VALID_DISPOSITIONS: &[&str] = &[
    "return_to_stock", "scrap", "inspect", "repair", "exchange",
];

/// Valid item conditions
#[allow(dead_code)]
const VALID_CONDITIONS: &[&str] = &[
    "good", "damaged", "defective", "wrong_item",
];

/// Valid inspection statuses
#[allow(dead_code)]
const VALID_INSPECTION_STATUSES: &[&str] = &[
    "pending", "passed", "failed", "pending_review",
];

/// Valid credit statuses for return lines
#[allow(dead_code)]
const VALID_CREDIT_STATUSES: &[&str] = &[
    "pending", "issued", "reversed",
];

/// Valid credit memo statuses
#[allow(dead_code)]
const VALID_CREDIT_MEMO_STATUSES: &[&str] = &[
    "draft", "issued", "applied", "partially_applied", "reversed", "cancelled",
];

/// Customer Returns engine for managing RMAs and credit memos
pub struct CustomerReturnsEngine {
    repository: Arc<dyn CustomerReturnsRepository>,
}

impl CustomerReturnsEngine {
    pub fn new(repository: Arc<dyn CustomerReturnsRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Return Reason Code Management
    // ========================================================================

    /// Create a new return reason code
    pub async fn create_return_reason(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        return_type: &str,
        default_disposition: Option<&str>,
        requires_approval: bool,
        credit_issued_automatically: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnReason> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Return reason code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Return reason name is required".to_string(),
            ));
        }
        if !VALID_RETURN_TYPES.contains(&return_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid return_type '{}'. Must be one of: {}", return_type, VALID_RETURN_TYPES.join(", ")
            )));
        }
        if let Some(disp) = default_disposition {
            if !VALID_DISPOSITIONS.contains(&disp) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid default_disposition '{}'. Must be one of: {}", disp, VALID_DISPOSITIONS.join(", ")
                )));
            }
        }

        info!("Creating return reason '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_return_reason(
            org_id, &code_upper, name, description, return_type,
            default_disposition, requires_approval, credit_issued_automatically,
            created_by,
        ).await
    }

    /// Get a return reason by code
    pub async fn get_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReturnReason>> {
        self.repository.get_return_reason(org_id, &code.to_uppercase()).await
    }

    /// List return reasons, optionally filtered by return type
    pub async fn list_return_reasons(&self, org_id: Uuid, return_type: Option<&str>) -> AtlasResult<Vec<ReturnReason>> {
        self.repository.list_return_reasons(org_id, return_type).await
    }

    /// Deactivate a return reason
    pub async fn delete_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating return reason '{}' for org {}", code, org_id);
        self.repository.delete_return_reason(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // RMA Management
    // ========================================================================

    /// Create a new Return Material Authorization
    pub async fn create_rma(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        return_type: &str,
        reason_code: Option<&str>,
        original_order_number: Option<&str>,
        original_order_id: Option<Uuid>,
        customer_contact: Option<&str>,
        customer_email: Option<&str>,
        customer_phone: Option<&str>,
        return_date: chrono::NaiveDate,
        expected_receipt_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnAuthorization> {
        if !VALID_RETURN_TYPES.contains(&return_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid return_type '{}'. Must be one of: {}", return_type, VALID_RETURN_TYPES.join(", ")
            )));
        }

        // Look up reason name if code is provided
        let reason_name = if let Some(rc) = reason_code {
            self.repository.get_return_reason(org_id, rc).await?
                .map(|r| r.name.clone())
        } else {
            None
        };

        // Generate RMA number
        let rma_number = format!("RMA-{}", chrono::Utc::now().format("%Y%m%d%-H%M%S"));

        info!("Creating RMA {} for customer {} in org {}", rma_number, customer_id, org_id);

        self.repository.create_rma(
            org_id, &rma_number, customer_id, customer_number, customer_name,
            return_type, reason_code, reason_name.as_deref(),
            original_order_number, original_order_id,
            customer_contact, customer_email, customer_phone,
            return_date, expected_receipt_date,
            currency_code, notes, created_by,
        ).await
    }

    /// Get an RMA by ID
    pub async fn get_rma(&self, id: Uuid) -> AtlasResult<Option<ReturnAuthorization>> {
        self.repository.get_rma(id).await
    }

    /// Get an RMA by number
    pub async fn get_rma_by_number(&self, org_id: Uuid, rma_number: &str) -> AtlasResult<Option<ReturnAuthorization>> {
        self.repository.get_rma_by_number(org_id, rma_number).await
    }

    /// List RMAs with optional filters
    pub async fn list_rmas(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        return_type: Option<&str>,
    ) -> AtlasResult<Vec<ReturnAuthorization>> {
        if let Some(s) = status {
            if !VALID_RMA_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RMA_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_rmas(org_id, status, customer_id, return_type).await
    }

    /// Submit an RMA for approval (draft -> submitted)
    pub async fn submit_rma(&self, id: Uuid) -> AtlasResult<ReturnAuthorization> {
        let rma = self.repository.get_rma(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", id)))?;

        if rma.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit RMA in '{}' status. Must be 'draft'.", rma.status)
            ));
        }

        // Check RMA has at least one line
        let lines = self.repository.list_return_lines_by_rma(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit RMA with no lines".to_string(),
            ));
        }

        info!("Submitting RMA {}", rma.rma_number);
        self.repository.update_rma_status(id, "submitted", None, None).await
    }

    /// Approve an RMA (submitted -> approved)
    pub async fn approve_rma(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<ReturnAuthorization> {
        let rma = self.repository.get_rma(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", id)))?;

        if rma.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve RMA in '{}' status. Must be 'submitted'.", rma.status)
            ));
        }

        info!("Approving RMA {} by {}", rma.rma_number, approved_by);
        self.repository.update_rma_status(id, "approved", Some(approved_by), None).await
    }

    /// Reject an RMA (submitted -> rejected)
    pub async fn reject_rma(&self, id: Uuid, reason: &str) -> AtlasResult<ReturnAuthorization> {
        let rma = self.repository.get_rma(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", id)))?;

        if rma.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject RMA in '{}' status. Must be 'submitted'.", rma.status)
            ));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rejection reason is required".to_string(),
            ));
        }

        info!("Rejecting RMA {}: {}", rma.rma_number, reason);
        self.repository.update_rma_status(id, "rejected", None, Some(reason)).await
    }

    /// Cancel an RMA
    pub async fn cancel_rma(&self, id: Uuid) -> AtlasResult<ReturnAuthorization> {
        let rma = self.repository.get_rma(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", id)))?;

        if rma.status == "received" || rma.status == "closed" || rma.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel RMA in '{}' status", rma.status)
            ));
        }

        info!("Cancelling RMA {}", rma.rma_number);
        self.repository.update_rma_status(id, "cancelled", None, None).await
    }

    // ========================================================================
    // Return Line Management
    // ========================================================================

    /// Add a line to an RMA
    pub async fn add_return_line(
        &self,
        org_id: Uuid,
        rma_id: Uuid,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        original_line_id: Option<Uuid>,
        original_quantity: &str,
        return_quantity: &str,
        unit_price: &str,
        reason_code: Option<&str>,
        disposition: Option<&str>,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        condition: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnLine> {
        let rma = self.repository.get_rma(rma_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", rma_id)))?;

        if rma.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add lines to a draft RMA".to_string(),
            ));
        }

        // Validate quantities
        let ret_qty: f64 = return_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "return_quantity must be a valid number".to_string(),
        ))?;
        if ret_qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "return_quantity must be positive".to_string(),
            ));
        }

        let orig_qty: f64 = original_quantity.parse().unwrap_or(0.0);
        if ret_qty > orig_qty {
            return Err(AtlasError::ValidationFailed(
                "return_quantity cannot exceed original_quantity".to_string(),
            ));
        }

        if let Some(cond) = condition {
            if !VALID_CONDITIONS.contains(&cond) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid condition '{}'. Must be one of: {}", cond, VALID_CONDITIONS.join(", ")
                )));
            }
        }
        if let Some(disp) = disposition {
            if !VALID_DISPOSITIONS.contains(&disp) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid disposition '{}'. Must be one of: {}", disp, VALID_DISPOSITIONS.join(", ")
                )));
            }
        }

        let price: f64 = unit_price.parse().unwrap_or(0.0);
        let return_amount = format!("{:.2}", ret_qty * price);
        let credit_amount = return_amount.clone(); // Default: full credit

        // Get next line number
        let existing_lines = self.repository.list_return_lines_by_rma(rma_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        info!("Adding return line {} to RMA {}", line_number, rma.rma_number);

        let line = self.repository.create_return_line(
            org_id, rma_id, line_number,
            item_id, item_code, item_description,
            original_line_id, original_quantity, return_quantity,
            unit_price, &return_amount, &credit_amount,
            reason_code, disposition,
            lot_number, serial_number, condition,
            notes, created_by,
        ).await?;

        // Recalculate RMA totals
        let all_lines = self.repository.list_return_lines_by_rma(rma_id).await?;
        let total_qty: f64 = all_lines.iter()
            .map(|l| l.return_quantity.parse().unwrap_or(0.0))
            .sum();
        let total_amt: f64 = all_lines.iter()
            .map(|l| l.return_amount.parse().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = all_lines.iter()
            .map(|l| l.credit_amount.parse().unwrap_or(0.0))
            .sum();

        self.repository.update_rma_totals(
            rma_id,
            &format!("{:.2}", total_qty),
            &format!("{:.2}", total_amt),
            &format!("{:.2}", total_credit),
        ).await?;

        Ok(line)
    }

    /// Get return lines for an RMA
    pub async fn list_return_lines(&self, rma_id: Uuid) -> AtlasResult<Vec<ReturnLine>> {
        self.repository.list_return_lines_by_rma(rma_id).await
    }

    /// Receive a returned item (record quantity received)
    pub async fn receive_return_line(
        &self,
        line_id: Uuid,
        received_quantity: &str,
    ) -> AtlasResult<ReturnLine> {
        let line = self.repository.get_return_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return line {} not found", line_id)))?;

        let recv_qty: f64 = received_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "received_quantity must be a valid number".to_string(),
        ))?;
        if recv_qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "received_quantity must be positive".to_string(),
            ));
        }

        let ret_qty: f64 = line.return_quantity.parse().unwrap_or(0.0);
        let already_recv: f64 = line.received_quantity.parse().unwrap_or(0.0);
        if already_recv + recv_qty > ret_qty {
            return Err(AtlasError::ValidationFailed(
                "Received quantity cannot exceed return quantity".to_string(),
            ));
        }

        let today = chrono::Utc::now().date_naive();
        info!("Receiving {} units for return line {}", received_quantity, line_id);

        let updated = self.repository.update_return_line_receipt(
            line_id, received_quantity, Some(today),
        ).await?;

        // Update RMA status based on all lines
        let all_lines = self.repository.list_return_lines_by_rma(line.rma_id).await?;
        let total_return_qty: f64 = all_lines.iter().map(|l| l.return_quantity.parse().unwrap_or(0.0)).sum();
        let total_recv_qty: f64 = all_lines.iter().map(|l| l.received_quantity.parse().unwrap_or(0.0)).sum();

        if total_recv_qty >= total_return_qty {
            self.repository.update_rma_status(line.rma_id, "received", None, None).await?;
        } else if total_recv_qty > 0.0 {
            self.repository.update_rma_status(line.rma_id, "partially_received", None, None).await?;
        }

        Ok(updated)
    }

    /// Inspect a returned item
    pub async fn inspect_return_line(
        &self,
        line_id: Uuid,
        inspection_status: &str,
        inspection_notes: Option<&str>,
        disposition: Option<&str>,
    ) -> AtlasResult<ReturnLine> {
        let line = self.repository.get_return_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Return line {} not found", line_id)))?;

        let recv_qty: f64 = line.received_quantity.parse().unwrap_or(0.0);
        if recv_qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Cannot inspect a line that has not been received".to_string(),
            ));
        }

        if !VALID_INSPECTION_STATUSES.contains(&inspection_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid inspection_status '{}'. Must be one of: {}",
                inspection_status, VALID_INSPECTION_STATUSES.join(", ")
            )));
        }

        if let Some(disp) = disposition {
            if !VALID_DISPOSITIONS.contains(&disp) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid disposition '{}'. Must be one of: {}", disp, VALID_DISPOSITIONS.join(", ")
                )));
            }
        }

        info!("Inspecting return line {} - status: {}", line_id, inspection_status);
        self.repository.update_return_line_inspection(
            line_id, inspection_status, inspection_notes, disposition,
        ).await
    }

    // ========================================================================
    // Credit Memo Management
    // ========================================================================

    /// Generate a credit memo from an RMA
    pub async fn generate_credit_memo(
        &self,
        rma_id: Uuid,
        gl_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditMemo> {
        let rma = self.repository.get_rma(rma_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("RMA {} not found", rma_id)))?;

        // RMA must be approved or received to generate credit memo
        if rma.status != "approved" && rma.status != "received" && rma.status != "partially_received" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot generate credit memo for RMA in '{}' status", rma.status)
            ));
        }

        if rma.credit_memo_id.is_some() {
            return Err(AtlasError::ValidationFailed(
                "Credit memo already generated for this RMA".to_string(),
            ));
        }

        let lines = self.repository.list_return_lines_by_rma(rma_id).await?;
        let total_credit: f64 = lines.iter()
            .map(|l| l.credit_amount.parse().unwrap_or(0.0))
            .sum();

        if total_credit <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "No credit amount to generate memo for".to_string(),
            ));
        }

        // Generate credit memo number
        let cm_number = format!("CM-{}", chrono::Utc::now().format("%Y%m%d%-H%M%S"));

        info!("Generating credit memo {} for RMA {} amount {}", cm_number, rma.rma_number, total_credit);

        let memo = self.repository.create_credit_memo(
            rma.organization_id, &cm_number,
            Some(rma_id), Some(&rma.rma_number),
            rma.customer_id, rma.customer_number.as_deref(), rma.customer_name.as_deref(),
            &format!("{:.2}", total_credit), &rma.currency_code,
            gl_account_code, None, created_by,
        ).await?;

        // Update RMA with credit memo reference
        self.repository.update_rma_credit_memo(rma_id, memo.id, &cm_number).await?;

        // Mark all lines as credit issued
        for line in &lines {
            self.repository.update_return_line_credit_status(line.id, "issued").await?;
        }

        Ok(memo)
    }

    /// Get a credit memo by ID
    pub async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<CreditMemo>> {
        self.repository.get_credit_memo(id).await
    }

    /// Get a credit memo by number
    pub async fn get_credit_memo_by_number(&self, org_id: Uuid, credit_memo_number: &str) -> AtlasResult<Option<CreditMemo>> {
        self.repository.get_credit_memo_by_number(org_id, credit_memo_number).await
    }

    /// List credit memos with optional filters
    pub async fn list_credit_memos(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CreditMemo>> {
        self.repository.list_credit_memos(org_id, customer_id, status).await
    }

    /// Issue a credit memo (draft -> issued)
    pub async fn issue_credit_memo(&self, id: Uuid) -> AtlasResult<CreditMemo> {
        let memo = self.repository.get_credit_memo(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Credit memo {} not found", id)))?;

        if memo.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot issue credit memo in '{}' status. Must be 'draft'.", memo.status)
            ));
        }

        info!("Issuing credit memo {}", memo.credit_memo_number);
        self.repository.update_credit_memo_status(id, "issued").await
    }

    /// Cancel a credit memo
    pub async fn cancel_credit_memo(&self, id: Uuid) -> AtlasResult<CreditMemo> {
        let memo = self.repository.get_credit_memo(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Credit memo {} not found", id)))?;

        if memo.status == "applied" || memo.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel credit memo in '{}' status", memo.status)
            ));
        }

        info!("Cancelling credit memo {}", memo.credit_memo_number);
        self.repository.update_credit_memo_status(id, "cancelled").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get a returns dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ReturnsDashboardSummary> {
        let rmas = self.repository.list_rmas(org_id, None, None, None).await?;
        let credit_memos = self.repository.list_credit_memos(org_id, None, None).await?;

        let total_rmas = rmas.len() as i32;
        let open_rmas = rmas.iter().filter(|r| r.status == "draft" || r.status == "submitted" || r.status == "approved").count() as i32;
        let pending_approval = rmas.iter().filter(|r| r.status == "submitted").count() as i32;
        let pending_receipt = rmas.iter().filter(|r| r.status == "approved").count() as i32;
        let pending_inspection = rmas.iter().filter(|r| r.status == "partially_received" || r.status == "received").count() as i32;

        let total_credit_issued: f64 = credit_memos.iter()
            .filter(|cm| cm.status == "issued" || cm.status == "applied" || cm.status == "partially_applied")
            .map(|cm| cm.amount.parse().unwrap_or(0.0))
            .sum();

        let total_credit_pending: f64 = credit_memos.iter()
            .filter(|cm| cm.status == "draft")
            .map(|cm| cm.amount.parse().unwrap_or(0.0))
            .sum();

        // Group by status
        let mut by_status = serde_json::Map::new();
        for rma in &rmas {
            let count = by_status.entry(rma.status.clone())
                .or_insert(serde_json::Value::Number(0.into()));
            *count = serde_json::Value::Number(
                (count.as_u64().unwrap_or(0) + 1).into()
            );
        }

        // Group by reason
        let mut by_reason = serde_json::Map::new();
        for rma in &rmas {
            if let Some(rc) = &rma.reason_code {
                let count = by_reason.entry(rc.clone())
                    .or_insert(serde_json::Value::Number(0.into()));
                *count = serde_json::Value::Number(
                    (count.as_u64().unwrap_or(0) + 1).into()
                );
            }
        }

        Ok(ReturnsDashboardSummary {
            total_rmas,
            open_rmas,
            pending_approval,
            pending_receipt,
            pending_inspection,
            total_credit_issued_amount: format!("{:.2}", total_credit_issued),
            total_credit_pending_amount: format!("{:.2}", total_credit_pending),
            rmas_by_status: serde_json::Value::Object(by_status),
            rmas_by_reason: serde_json::Value::Object(by_reason),
            rmas_by_disposition: serde_json::json!({}),
            top_returned_items: serde_json::json!({}),
            average_processing_days: "0.0".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_return_types() {
        assert!(VALID_RETURN_TYPES.contains(&"standard_return"));
        assert!(VALID_RETURN_TYPES.contains(&"exchange"));
        assert!(VALID_RETURN_TYPES.contains(&"repair"));
        assert!(VALID_RETURN_TYPES.contains(&"warranty"));
    }

    #[test]
    fn test_valid_rma_statuses() {
        assert!(VALID_RMA_STATUSES.contains(&"draft"));
        assert!(VALID_RMA_STATUSES.contains(&"submitted"));
        assert!(VALID_RMA_STATUSES.contains(&"approved"));
        assert!(VALID_RMA_STATUSES.contains(&"rejected"));
        assert!(VALID_RMA_STATUSES.contains(&"received"));
        assert!(VALID_RMA_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_dispositions() {
        assert!(VALID_DISPOSITIONS.contains(&"return_to_stock"));
        assert!(VALID_DISPOSITIONS.contains(&"scrap"));
        assert!(VALID_DISPOSITIONS.contains(&"inspect"));
        assert!(VALID_DISPOSITIONS.contains(&"repair"));
    }

    #[test]
    fn test_valid_conditions() {
        assert!(VALID_CONDITIONS.contains(&"good"));
        assert!(VALID_CONDITIONS.contains(&"damaged"));
        assert!(VALID_CONDITIONS.contains(&"defective"));
        assert!(VALID_CONDITIONS.contains(&"wrong_item"));
    }

    #[test]
    fn test_valid_inspection_statuses() {
        assert!(VALID_INSPECTION_STATUSES.contains(&"pending"));
        assert!(VALID_INSPECTION_STATUSES.contains(&"passed"));
        assert!(VALID_INSPECTION_STATUSES.contains(&"failed"));
        assert!(VALID_INSPECTION_STATUSES.contains(&"pending_review"));
    }

    #[test]
    fn test_valid_credit_memo_statuses() {
        assert!(VALID_CREDIT_MEMO_STATUSES.contains(&"draft"));
        assert!(VALID_CREDIT_MEMO_STATUSES.contains(&"issued"));
        assert!(VALID_CREDIT_MEMO_STATUSES.contains(&"applied"));
        assert!(VALID_CREDIT_MEMO_STATUSES.contains(&"reversed"));
    }
}
