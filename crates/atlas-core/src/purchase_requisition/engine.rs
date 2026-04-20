//! Purchase Requisition Engine Implementation
//!
//! Manages the complete lifecycle of purchase requisitions:
//! creation, line items, accounting distributions, approval workflow,
//! and AutoCreate conversion to purchase orders.
//!
//! Oracle Fusion Cloud ERP equivalent: Self-Service Procurement > Requisitions

use atlas_shared::{
    PurchaseRequisition, PurchaseRequisitionRequest,
    RequisitionLine, RequisitionLineRequest,
    RequisitionDistribution, RequisitionDistributionRequest,
    RequisitionApproval, AutocreateLink, AutocreateRequest,
    RequisitionDashboardSummary,
    AtlasError, AtlasResult,
};
use super::PurchaseRequisitionRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "urgent"];
const VALID_STATUSES: &[&str] = &["draft", "submitted", "approved", "rejected", "cancelled", "closed", "in_review"];
const VALID_LINE_STATUSES: &[&str] = &["draft", "submitted", "approved", "rejected", "cancelled", "partially_ordered", "ordered", "closed"];
const VALID_LINE_SOURCE_TYPES: &[&str] = &["manual", "catalog", "punchout"];
const VALID_APPROVAL_ACTIONS: &[&str] = &["approved", "rejected", "delegated", "returned"];
const VALID_AUTOCREATE_STATUSES: &[&str] = &["pending", "ordered", "partial", "completed", "cancelled"];

/// Purchase Requisition engine
pub struct PurchaseRequisitionEngine {
    repository: Arc<dyn PurchaseRequisitionRepository>,
}

impl PurchaseRequisitionEngine {
    pub fn new(repository: Arc<dyn PurchaseRequisitionRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Requisition CRUD
    // ========================================================================

    /// Create a new purchase requisition
    pub async fn create_requisition(
        &self,
        org_id: Uuid,
        request: &PurchaseRequisitionRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition> {
        // Validate priority
        let urgency_code = request.urgency_code.as_deref().unwrap_or("medium");
        if !VALID_PRIORITIES.contains(&urgency_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid urgency code '{}'. Must be one of: {}", urgency_code, VALID_PRIORITIES.join(", ")
            )));
        }

        // Validate lines
        if request.lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "At least one requisition line is required".to_string(),
            ));
        }

        // Generate requisition number
        let requisition_number = format!("REQ-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        info!("Creating purchase requisition {}", requisition_number);

        // Calculate total amount from lines
        let mut total_amount: f64 = 0.0;
        for line in &request.lines {
            let qty: f64 = line.quantity.as_deref().unwrap_or("1").parse()
                .map_err(|_| AtlasError::ValidationFailed("Invalid quantity".to_string()))?;
            let price: f64 = line.unit_price.as_deref().unwrap_or("0").parse()
                .map_err(|_| AtlasError::ValidationFailed("Invalid unit_price".to_string()))?;
            total_amount += qty * price;
        }

        // Check amount limit
        if let Some(ref limit_str) = request.amount_limit {
            let limit: f64 = limit_str.parse()
                .map_err(|_| AtlasError::ValidationFailed("Invalid amount_limit".to_string()))?;
            if total_amount > limit {
                return Err(AtlasError::ValidationFailed(format!(
                    "Requisition total {} exceeds amount limit {}", total_amount, limit
                )));
            }
        }

        let requisition = self.repository.create_requisition(
            org_id,
            &requisition_number,
            request.description.as_deref(),
            urgency_code,
            request.requester_id,
            request.requester_name.as_deref(),
            request.department.as_deref(),
            request.justification.as_deref(),
            request.budget_code.as_deref(),
            request.amount_limit.as_deref(),
            &format!("{:.2}", total_amount),
            request.currency_code.as_deref().unwrap_or("USD"),
            request.charge_account_code.as_deref(),
            request.delivery_address.as_deref(),
            request.requested_delivery_date,
            request.notes.as_deref(),
            created_by,
        ).await?;

        // Create lines
        let mut lines = Vec::new();
        for (idx, line_req) in request.lines.iter().enumerate() {
            let line_number = (idx + 1) as i32;
            let qty: f64 = line_req.quantity.as_deref().unwrap_or("1").parse().unwrap_or(1.0);
            let price: f64 = line_req.unit_price.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
            let line_amount = qty * price;

            let line = self.repository.create_line(
                org_id,
                requisition.id,
                line_number,
                line_req.item_code.as_deref(),
                &line_req.item_description,
                line_req.category.as_deref(),
                &format!("{:.4}", qty),
                line_req.unit_of_measure.as_deref().unwrap_or("EACH"),
                &format!("{:.4}", price),
                &format!("{:.4}", line_amount),
                line_req.currency_code.as_deref().unwrap_or("USD"),
                line_req.charge_account_code.as_deref(),
                line_req.requested_delivery_date,
                line_req.supplier_id,
                line_req.supplier_name.as_deref(),
                line_req.source_type.as_deref().unwrap_or("manual"),
                line_req.source_reference.as_deref(),
                line_req.notes.as_deref(),
                created_by,
            ).await?;

            // Create distributions if provided
            if let Some(distributions) = &line_req.distributions {
                for (d_idx, dist_req) in distributions.iter().enumerate() {
                    self.repository.create_distribution(
                        org_id,
                        requisition.id,
                        line.id,
                        (d_idx + 1) as i32,
                        &dist_req.charge_account_code,
                        dist_req.allocation_percentage.as_deref().unwrap_or("100.0000"),
                        dist_req.amount.as_deref().unwrap_or(&format!("{:.4}", line_amount)),
                        dist_req.project_code.as_deref(),
                        dist_req.cost_center.as_deref(),
                    ).await?;
                }
            }

            lines.push(line);
        }

        // Reload with lines and distributions
        self.repository.get_requisition_by_id(requisition.id).await?
            .ok_or_else(|| AtlasError::Internal("Created requisition not found".to_string()))
    }

    /// Get a requisition by ID
    pub async fn get_requisition(&self, id: Uuid) -> AtlasResult<Option<PurchaseRequisition>> {
        self.repository.get_requisition_by_id(id).await
    }

    /// List requisitions for an organization
    pub async fn list_requisitions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        requester_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PurchaseRequisition>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_requisitions(org_id, status, requester_id).await
    }

    /// Update a requisition (only in draft status)
    pub async fn update_requisition(
        &self,
        id: Uuid,
        org_id: Uuid,
        request: &PurchaseRequisitionRequest,
        updated_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot update requisition in '{}' status. Must be 'draft'.", requisition.status
            )));
        }

        let urgency_code = request.urgency_code.as_deref().unwrap_or(&requisition.urgency_code);
        if !VALID_PRIORITIES.contains(&urgency_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid urgency code '{}'. Must be one of: {}", urgency_code, VALID_PRIORITIES.join(", ")
            )));
        }

        // Calculate new total
        let mut total_amount: f64 = 0.0;
        for line in &request.lines {
            let qty: f64 = line.quantity.as_deref().unwrap_or("1").parse()
                .map_err(|_| AtlasError::ValidationFailed("Invalid quantity".to_string()))?;
            let price: f64 = line.unit_price.as_deref().unwrap_or("0").parse()
                .map_err(|_| AtlasError::ValidationFailed("Invalid unit_price".to_string()))?;
            total_amount += qty * price;
        }

        info!("Updating purchase requisition {}", requisition.requisition_number);

        self.repository.update_requisition(
            id,
            request.description.as_deref(),
            urgency_code,
            request.department.as_deref(),
            request.justification.as_deref(),
            request.budget_code.as_deref(),
            &format!("{:.2}", total_amount),
            request.charge_account_code.as_deref(),
            request.delivery_address.as_deref(),
            request.requested_delivery_date,
            request.notes.as_deref(),
            updated_by,
        ).await
    }

    /// Delete a requisition (only in draft status)
    pub async fn delete_requisition(&self, id: Uuid) -> AtlasResult<()> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot delete requisition in '{}' status. Must be 'draft'.", requisition.status
            )));
        }

        info!("Deleting purchase requisition {}", requisition.requisition_number);
        self.repository.delete_requisition(id).await
    }

    // ========================================================================
    // Requisition Lines
    // ========================================================================

    /// Add a line to an existing requisition
    pub async fn add_line(
        &self,
        org_id: Uuid,
        requisition_id: Uuid,
        request: &RequisitionLineRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RequisitionLine> {
        let requisition = self.repository.get_requisition_by_id(requisition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", requisition_id)))?;

        if requisition.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to requisition in '{}' status", requisition.status
            )));
        }

        let line_number = (requisition.lines.len() + 1) as i32;
        let qty: f64 = request.quantity.as_deref().unwrap_or("1").parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid quantity".to_string()))?;
        let price: f64 = request.unit_price.as_deref().unwrap_or("0").parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid unit_price".to_string()))?;
        let line_amount = qty * price;

        let line = self.repository.create_line(
            org_id,
            requisition_id,
            line_number,
            request.item_code.as_deref(),
            &request.item_description,
            request.category.as_deref(),
            &format!("{:.4}", qty),
            request.unit_of_measure.as_deref().unwrap_or("EACH"),
            &format!("{:.4}", price),
            &format!("{:.4}", line_amount),
            request.currency_code.as_deref().unwrap_or("USD"),
            request.charge_account_code.as_deref(),
            request.requested_delivery_date,
            request.supplier_id,
            request.supplier_name.as_deref(),
            request.source_type.as_deref().unwrap_or("manual"),
            request.source_reference.as_deref(),
            request.notes.as_deref(),
            created_by,
        ).await?;

        // Create distributions if provided
        if let Some(distributions) = &request.distributions {
            for (d_idx, dist_req) in distributions.iter().enumerate() {
                let pct: f64 = dist_req.allocation_percentage.as_deref().unwrap_or("100").parse().unwrap_or(100.0);
                let dist_amount = line_amount * pct / 100.0;
                self.repository.create_distribution(
                    org_id,
                    requisition_id,
                    line.id,
                    (d_idx + 1) as i32,
                    &dist_req.charge_account_code,
                    dist_req.allocation_percentage.as_deref().unwrap_or("100.0000"),
                    dist_req.amount.as_deref().unwrap_or(&format!("{:.4}", dist_amount)),
                    dist_req.project_code.as_deref(),
                    dist_req.cost_center.as_deref(),
                ).await?;
            }
        }

        // Recalculate total
        let total = requisition.lines.iter()
            .filter_map(|l| l.line_amount.parse::<f64>().ok())
            .sum::<f64>() + line_amount;
        self.repository.update_requisition_total(requisition_id, &format!("{:.2}", total)).await?;

        Ok(line)
    }

    /// List lines for a requisition
    pub async fn list_lines(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionLine>> {
        let requisition = self.repository.get_requisition_by_id(requisition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", requisition_id)))?;
        Ok(requisition.lines)
    }

    /// Remove a line from a requisition
    pub async fn remove_line(&self, line_id: Uuid) -> AtlasResult<()> {
        info!("Removing requisition line {}", line_id);
        self.repository.delete_line(line_id).await
    }

    // ========================================================================
    // Requisition Distributions
    // ========================================================================

    /// Add a distribution to a line
    pub async fn add_distribution(
        &self,
        org_id: Uuid,
        requisition_id: Uuid,
        line_id: Uuid,
        request: &RequisitionDistributionRequest,
    ) -> AtlasResult<RequisitionDistribution> {
        // Verify line belongs to requisition
        let requisition = self.repository.get_requisition_by_id(requisition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", requisition_id)))?;

        if requisition.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot modify distributions on requisition in '{}' status", requisition.status
            )));
        }

        let line = requisition.lines.iter()
            .find(|l| l.id == line_id)
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Line {} not found in requisition", line_id)))?;

        let next_dist_num = line.distributions.len() as i32 + 1;

        self.repository.create_distribution(
            org_id,
            requisition_id,
            line_id,
            next_dist_num,
            &request.charge_account_code,
            request.allocation_percentage.as_deref().unwrap_or("100.0000"),
            request.amount.as_deref().unwrap_or(&line.line_amount),
            request.project_code.as_deref(),
            request.cost_center.as_deref(),
        ).await
    }

    /// List distributions for a line
    pub async fn list_distributions(&self, line_id: Uuid) -> AtlasResult<Vec<RequisitionDistribution>> {
        self.repository.list_distributions_by_line(line_id).await
    }

    // ========================================================================
    // Approval Workflow
    // ========================================================================

    /// Submit a requisition for approval
    pub async fn submit_requisition(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit requisition in '{}' status. Must be 'draft'.", requisition.status
            )));
        }

        if requisition.lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit requisition with no lines".to_string(),
            ));
        }

        info!("Submitting purchase requisition {}", requisition.requisition_number);
        self.repository.update_requisition_status(id, "submitted", submitted_by, None).await
    }

    /// Approve a requisition
    pub async fn approve_requisition(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_name: Option<&str>,
        comments: Option<&str>,
    ) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve requisition in '{}' status. Must be 'submitted'.", requisition.status
            )));
        }

        info!("Approving purchase requisition {}", requisition.requisition_number);

        // Record the approval
        self.repository.create_approval(
            requisition.organization_id,
            id,
            approver_id,
            approver_name,
            "approved",
            comments,
        ).await?;

        // Update requisition status and line statuses
        self.repository.update_requisition_status(id, "approved", Some(approver_id), approver_name).await?;
        self.repository.update_line_statuses(id, "approved").await?;

        self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::Internal("Requisition not found after approval".to_string()))
    }

    /// Reject a requisition
    pub async fn reject_requisition(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_name: Option<&str>,
        comments: Option<&str>,
    ) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject requisition in '{}' status. Must be 'submitted'.", requisition.status
            )));
        }

        info!("Rejecting purchase requisition {}", requisition.requisition_number);

        self.repository.create_approval(
            requisition.organization_id,
            id,
            approver_id,
            approver_name,
            "rejected",
            comments,
        ).await?;

        self.repository.update_requisition_status(id, "rejected", None, None).await?;
        self.repository.update_line_statuses(id, "rejected").await?;

        self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::Internal("Requisition not found after rejection".to_string()))
    }

    /// Cancel a requisition
    pub async fn cancel_requisition(&self, id: Uuid) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status == "closed" || requisition.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel requisition in '{}' status.", requisition.status
            )));
        }

        info!("Cancelling purchase requisition {}", requisition.requisition_number);
        self.repository.update_requisition_status(id, "cancelled", None, None).await?;
        self.repository.update_line_statuses(id, "cancelled").await?;

        self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::Internal("Requisition not found after cancellation".to_string()))
    }

    /// Close a requisition
    pub async fn close_requisition(&self, id: Uuid) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close requisition in '{}' status. Must be 'approved'.", requisition.status
            )));
        }

        info!("Closing purchase requisition {}", requisition.requisition_number);
        self.repository.update_requisition_status(id, "closed", None, None).await?;
        self.repository.update_line_statuses(id, "closed").await?;

        self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::Internal("Requisition not found after close".to_string()))
    }

    /// Return a requisition to draft (for re-editing)
    pub async fn return_requisition(&self, id: Uuid) -> AtlasResult<PurchaseRequisition> {
        let requisition = self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;

        if requisition.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot return requisition in '{}' status. Must be 'submitted'.", requisition.status
            )));
        }

        info!("Returning purchase requisition {} to draft", requisition.requisition_number);
        self.repository.update_requisition_status(id, "draft", None, None).await?;
        self.repository.update_line_statuses(id, "draft").await?;

        self.repository.get_requisition_by_id(id).await?
            .ok_or_else(|| AtlasError::Internal("Requisition not found after return".to_string()))
    }

    /// List approval history for a requisition
    pub async fn list_approvals(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionApproval>> {
        self.repository.list_approvals(requisition_id).await
    }

    // ========================================================================
    // AutoCreate (Convert Requisitions to Purchase Orders)
    // ========================================================================

    /// Create purchase orders from approved requisition lines (AutoCreate)
    pub async fn autocreate(
        &self,
        org_id: Uuid,
        request: &AutocreateRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Vec<AutocreateLink>> {
        if request.requisition_line_ids.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "At least one requisition line must be selected for AutoCreate".to_string(),
            ));
        }

        // Validate all lines are approved
        let mut links = Vec::new();
        for line_id in &request.requisition_line_ids {
            let line = self.repository.get_line_by_id(*line_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition line {} not found", line_id)))?;

            if line.status != "approved" {
                return Err(AtlasError::WorkflowError(format!(
                    "Line {} is in '{}' status. Only 'approved' lines can be AutoCreated.", line.line_number, line.status
                )));
            }

            let default_po = format!("PO-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
            let po_number = request.purchase_order_number.as_deref()
                .unwrap_or(&default_po);

            let link = self.repository.create_autocreate_link(
                org_id,
                line.requisition_id,
                line.id,
                po_number,
                request.supplier_id,
                request.supplier_name.as_deref(),
                &line.quantity,
                "ordered",
                created_by,
            ).await?;

            // Update line status to ordered
            self.repository.update_line_status(line.id, "ordered").await?;

            links.push(link);
        }

        // Check if all lines of each requisition are now ordered
        for line_id in &request.requisition_line_ids {
            if let Some(line) = self.repository.get_line_by_id(*line_id).await? {
                let requisition = self.repository.get_requisition_by_id(line.requisition_id).await?;
                if let Some(req) = requisition {
                    let all_ordered = req.lines.iter().all(|l| {
                        l.status == "ordered" || l.status == "partially_ordered" || l.status == "closed"
                    });
                    if all_ordered {
                        self.repository.update_requisition_status(req.id, "closed", None, None).await?;
                    }
                }
            }
        }

        info!("AutoCreated {} purchase order lines", links.len());
        Ok(links)
    }

    /// List AutoCreate links for a requisition
    pub async fn list_autocreate_links(&self, requisition_id: Uuid) -> AtlasResult<Vec<AutocreateLink>> {
        self.repository.list_autocreate_links(requisition_id).await
    }

    /// Cancel an AutoCreate link
    pub async fn cancel_autocreate_link(&self, link_id: Uuid) -> AtlasResult<()> {
        info!("Cancelling AutoCreate link {}", link_id);
        self.repository.update_autocreate_link_status(link_id, "cancelled").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get requisition dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RequisitionDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"medium"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"urgent"));
        assert!(!VALID_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"submitted"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"rejected"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert!(VALID_STATUSES.contains(&"closed"));
        assert!(VALID_STATUSES.contains(&"in_review"));
    }

    #[test]
    fn test_valid_line_statuses() {
        assert!(VALID_LINE_STATUSES.contains(&"draft"));
        assert!(VALID_LINE_STATUSES.contains(&"approved"));
        assert!(VALID_LINE_STATUSES.contains(&"ordered"));
        assert!(VALID_LINE_STATUSES.contains(&"partially_ordered"));
        assert!(!VALID_LINE_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_approval_actions() {
        assert!(VALID_APPROVAL_ACTIONS.contains(&"approved"));
        assert!(VALID_APPROVAL_ACTIONS.contains(&"rejected"));
        assert!(VALID_APPROVAL_ACTIONS.contains(&"delegated"));
        assert!(VALID_APPROVAL_ACTIONS.contains(&"returned"));
    }

    #[test]
    fn test_valid_source_types() {
        assert!(VALID_LINE_SOURCE_TYPES.contains(&"manual"));
        assert!(VALID_LINE_SOURCE_TYPES.contains(&"catalog"));
        assert!(VALID_LINE_SOURCE_TYPES.contains(&"punchout"));
    }

    #[test]
    fn test_valid_autocreate_statuses() {
        assert!(VALID_AUTOCREATE_STATUSES.contains(&"pending"));
        assert!(VALID_AUTOCREATE_STATUSES.contains(&"ordered"));
        assert!(VALID_AUTOCREATE_STATUSES.contains(&"completed"));
        assert!(VALID_AUTOCREATE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_line_amount_calculation() {
        let quantity: f64 = 10.0;
        let unit_price: f64 = 25.50;
        let line_amount = quantity * unit_price;
        assert!((line_amount - 255.0).abs() < 0.01);
    }

    #[test]
    fn test_total_amount_with_multiple_lines() {
        let lines = vec![
            (5.0_f64, 100.0_f64),  // 500.0
            (3.0_f64, 50.0_f64),   // 150.0
            (10.0_f64, 25.0_f64),  // 250.0
        ];
        let total: f64 = lines.iter().map(|(q, p)| q * p).sum();
        assert!((total - 900.0).abs() < 0.01);
    }

    #[test]
    fn test_amount_limit_check() {
        let total_amount: f64 = 5000.0;
        let amount_limit: f64 = 10000.0;
        assert!(total_amount <= amount_limit);

        let exceed_limit: f64 = 15000.0;
        assert!(total_amount > exceed_limit == false);
        assert!(total_amount > amount_limit == false);
    }

    #[test]
    fn test_distribution_percentage() {
        let line_amount: f64 = 1000.0;
        let pct: f64 = 60.0;
        let dist_amount = line_amount * pct / 100.0;
        assert!((dist_amount - 600.0).abs() < 0.01);

        let pct2: f64 = 40.0;
        let dist_amount2 = line_amount * pct2 / 100.0;
        assert!((dist_amount2 - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_requisition_number_format() {
        let req_num = format!("REQ-{}", "20240101120000");
        assert!(req_num.starts_with("REQ-"));
    }
}