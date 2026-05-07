//! Chargeback Management Engine
//!
//! Orchestrates chargeback creation, lifecycle transitions, line management,
//! activity tracking, validation, and dashboard summary.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Receivables > Chargebacks

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    ChargebackManagementRepository,
    Chargeback, ChargebackLine, ChargebackActivity, ChargebackSummary,
    ChargebackCreateParams, ChargebackLineCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid reason codes for chargebacks
const VALID_REASON_CODES: &[&str] = &[
    "damaged_goods", "pricing_dispute", "promotional_allowance",
    "short_shipment", "quality_issue", "return_not_credited",
    "duplicate_charge", "late_delivery", "contract_discount",
    "volume_rebate", "advertising_allowance", "freight_dispute",
    "other",
];

// Valid categories
const VALID_CATEGORIES: &[&str] = &[
    "pricing", "quality", "delivery", "promotion",
    "returns", "freight", "other",
];

// Valid statuses
const VALID_STATUSES: &[&str] = &[
    "open", "under_review", "accepted", "rejected", "written_off",
];

// Valid priorities
const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];

// Valid line types
const VALID_LINE_TYPES: &[&str] = &[
    "chargeback", "tax", "freight", "discount", "adjustment",
];

/// Chargeback Management Engine
pub struct ChargebackManagementEngine {
    repo: Arc<dyn ChargebackManagementRepository>,
}

impl ChargebackManagementEngine {
    pub fn new(repo: Arc<dyn ChargebackManagementRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    fn validate_reason_code(reason_code: &str) -> AtlasResult<()> {
        if !VALID_REASON_CODES.contains(&reason_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reason_code '{}'. Must be one of: {}", reason_code, VALID_REASON_CODES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_category(category: &str) -> AtlasResult<()> {
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}", category, VALID_CATEGORIES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_status(status: &str) -> AtlasResult<()> {
        if !VALID_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_priority(priority: &str) -> AtlasResult<()> {
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_line_type(line_type: &str) -> AtlasResult<()> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validate that a status transition is allowed for chargebacks.
    /// open -> under_review -> accepted | rejected
    /// open -> written_off
    /// under_review -> written_off
    /// accepted -> written_off
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("open", "under_review") => Ok(()),
            ("open", "written_off") => Ok(()),
            ("under_review", "accepted") => Ok(()),
            ("under_review", "rejected") => Ok(()),
            ("under_review", "written_off") => Ok(()),
            ("accepted", "written_off") => Ok(()),
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid status transition from '{}' to '{}'. \
                 Valid transitions: open→under_review, open→written_off, \
                 under_review→accepted, under_review→rejected, under_review→written_off, \
                 accepted→written_off",
                current, target
            ))),
        }
    }

    // ========================================================================
    // Chargeback CRUD
    // ========================================================================

    /// Create a new chargeback
    pub async fn create_chargeback(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        receipt_id: Option<Uuid>,
        receipt_number: Option<&str>,
        invoice_id: Option<Uuid>,
        invoice_number: Option<&str>,
        chargeback_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        amount: f64,
        tax_amount: f64,
        reason_code: &str,
        reason_description: Option<&str>,
        category: Option<&str>,
        priority: Option<&str>,
        assigned_to: Option<&str>,
        assigned_team: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        reference: Option<&str>,
        customer_reference: Option<&str>,
        sales_rep: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Chargeback> {
        info!("Creating chargeback for org {} customer {:?}", org_id, customer_name);

        Self::validate_reason_code(reason_code)?;
        if let Some(cat) = category {
            Self::validate_category(cat)?;
        }
        if let Some(pri) = priority {
            Self::validate_priority(pri)?;
        }

        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Chargeback amount must be greater than zero".to_string()
            ));
        }

        let total_amount = amount + tax_amount;
        let params = ChargebackCreateParams {
            org_id,
            customer_id,
            customer_number: customer_number.map(|s| s.to_string()),
            customer_name: customer_name.map(|s| s.to_string()),
            receipt_id,
            receipt_number: receipt_number.map(|s| s.to_string()),
            invoice_id,
            invoice_number: invoice_number.map(|s| s.to_string()),
            chargeback_date,
            gl_date: gl_date.unwrap_or(chargeback_date),
            currency_code: currency_code.to_string(),
            exchange_rate_type: exchange_rate_type.map(|s| s.to_string()),
            exchange_rate,
            amount,
            tax_amount,
            total_amount,
            open_amount: total_amount,
            reason_code: reason_code.to_string(),
            reason_description: reason_description.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
            priority: priority.unwrap_or("medium").to_string(),
            assigned_to: assigned_to.map(|s| s.to_string()),
            assigned_team: assigned_team.map(|s| s.to_string()),
            due_date,
            reference: reference.map(|s| s.to_string()),
            customer_reference: customer_reference.map(|s| s.to_string()),
            sales_rep: sales_rep.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by,
        };

        let chargeback = self.repo.create_chargeback(&params).await?;

        // Log activity
        let _ = self.repo.create_activity(
            org_id, chargeback.id,
            "created",
            Some("Chargeback created"),
            None,
            Some("open"),
            created_by,
            None,
            None,
        ).await;

        Ok(chargeback)
    }

    /// Get a chargeback by ID
    pub async fn get_chargeback(&self, id: Uuid) -> AtlasResult<Option<Chargeback>> {
        self.repo.get_chargeback(id).await
    }

    /// Get a chargeback by number
    pub async fn get_chargeback_by_number(&self, org_id: Uuid, chargeback_number: &str) -> AtlasResult<Option<Chargeback>> {
        self.repo.get_chargeback_by_number(org_id, chargeback_number).await
    }

    /// List chargebacks with optional filters
    pub async fn list_chargebacks(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        reason_code: Option<&str>,
        category: Option<&str>,
        priority: Option<&str>,
    ) -> AtlasResult<Vec<Chargeback>> {
        self.repo.list_chargebacks(org_id, status, customer_id, reason_code, category, priority).await
    }

    /// Delete a chargeback (only allowed in open status)
    pub async fn delete_chargeback(&self, org_id: Uuid, chargeback_number: &str) -> AtlasResult<()> {
        info!("Deleting chargeback '{}' for org {}", chargeback_number, org_id);

        let cb = self.repo.get_chargeback_by_number(org_id, chargeback_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Chargeback not found".to_string()))?;

        if cb.status != "open" {
            return Err(AtlasError::ValidationFailed(
                "Only chargebacks in 'open' status can be deleted".to_string()
            ));
        }

        self.repo.delete_chargeback(org_id, chargeback_number).await
    }

    // ========================================================================
    // Status Transitions
    // ========================================================================

    /// Transition a chargeback to a new status
    pub async fn transition_chargeback(
        &self,
        id: Uuid,
        new_status: &str,
        resolution_notes: Option<&str>,
        resolved_by: Option<Uuid>,
        resolved_by_name: Option<&str>,
    ) -> AtlasResult<Chargeback> {
        info!("Transitioning chargeback {} to status '{}'", id, new_status);
        Self::validate_status(new_status)?;

        let current = self.repo.get_chargeback(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Chargeback not found".to_string()))?;

        Self::validate_status_transition(&current.status, new_status)?;

        let chargeback = self.repo.update_chargeback_status(
            id,
            new_status,
            if new_status == "accepted" || new_status == "rejected" || new_status == "written_off" {
                Some(chrono::Utc::now().naive_utc().date())
            } else {
                None
            },
            resolution_notes,
            resolved_by,
        ).await?;

        // Log activity
        let _ = self.repo.create_activity(
            current.organization_id, id,
            &format!("status_change_{}", new_status),
            Some(&format!("Status changed from '{}' to '{}'", current.status, new_status)),
            Some(&current.status),
            Some(new_status),
            resolved_by,
            resolved_by_name,
            resolution_notes,
        ).await;

        Ok(chargeback)
    }

    /// Assign a chargeback to a user/team
    pub async fn assign_chargeback(
        &self,
        id: Uuid,
        assigned_to: Option<&str>,
        assigned_team: Option<&str>,
    ) -> AtlasResult<Chargeback> {
        info!("Assigning chargeback {} to {:?}/{:?}", id, assigned_to, assigned_team);
        let cb = self.repo.assign_chargeback(id, assigned_to, assigned_team).await?;

        let _ = self.repo.create_activity(
            cb.organization_id, id,
            "assigned",
            Some(&format!("Assigned to {} / {}", assigned_to.unwrap_or("N/A"), assigned_team.unwrap_or("N/A"))),
            None, None, None, None,
            None,
        ).await;

        Ok(cb)
    }

    /// Update chargeback notes
    pub async fn update_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<Chargeback> {
        self.repo.update_notes(id, notes).await
    }

    // ========================================================================
    // Chargeback Lines
    // ========================================================================

    /// Add a line to a chargeback
    pub async fn add_chargeback_line(
        &self,
        org_id: Uuid,
        chargeback_id: Uuid,
        line_type: &str,
        description: Option<&str>,
        quantity: Option<i32>,
        unit_price: Option<f64>,
        amount: f64,
        tax_amount: Option<f64>,
        reason_code: Option<&str>,
        reason_description: Option<&str>,
        item_number: Option<&str>,
        item_description: Option<&str>,
        gl_account_code: Option<&str>,
        gl_account_name: Option<&str>,
        reference: Option<&str>,
    ) -> AtlasResult<ChargebackLine> {
        info!("Adding line to chargeback {}", chargeback_id);
        Self::validate_line_type(line_type)?;

        // Verify chargeback exists and is in editable status
        let cb = self.repo.get_chargeback(chargeback_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Chargeback not found".to_string()))?;

        if cb.status != "open" && cb.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be added to chargebacks in 'open' or 'under_review' status".to_string()
            ));
        }

        let tax = tax_amount.unwrap_or(0.0);
        let params = ChargebackLineCreateParams {
            org_id,
            chargeback_id,
            line_type: line_type.to_string(),
            description: description.map(|s| s.to_string()),
            quantity: quantity.unwrap_or(1),
            unit_price: unit_price.unwrap_or(amount),
            amount,
            tax_amount: tax,
            total_amount: amount + tax,
            reason_code: reason_code.map(|s| s.to_string()),
            reason_description: reason_description.map(|s| s.to_string()),
            item_number: item_number.map(|s| s.to_string()),
            item_description: item_description.map(|s| s.to_string()),
            gl_account_code: gl_account_code.map(|s| s.to_string()),
            gl_account_name: gl_account_name.map(|s| s.to_string()),
            reference: reference.map(|s| s.to_string()),
        };

        let line = self.repo.create_chargeback_line(&params).await?;

        // Recalculate chargeback totals
        self.recalculate_totals(chargeback_id).await?;

        // Log activity
        let _ = self.repo.create_activity(
            org_id, chargeback_id,
            "line_added",
            Some(&format!("Line added: {} ({})", line_type, amount)),
            None, None, None, None, None,
        ).await;

        Ok(line)
    }

    /// List lines for a chargeback
    pub async fn list_chargeback_lines(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackLine>> {
        self.repo.list_chargeback_lines(chargeback_id).await
    }

    /// Remove a line from a chargeback
    pub async fn remove_chargeback_line(&self, chargeback_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        info!("Removing line {} from chargeback {}", line_id, chargeback_id);

        // Verify chargeback is in editable status
        let cb = self.repo.get_chargeback(chargeback_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Chargeback not found".to_string()))?;

        if cb.status != "open" && cb.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be removed from chargebacks in 'open' or 'under_review' status".to_string()
            ));
        }

        self.repo.delete_chargeback_line(chargeback_id, line_id).await?;

        // Recalculate totals
        self.recalculate_totals(chargeback_id).await?;

        Ok(())
    }

    /// Recalculate chargeback totals from lines
    async fn recalculate_totals(&self, chargeback_id: Uuid) -> AtlasResult<()> {
        let lines = self.repo.list_chargeback_lines(chargeback_id).await?;
        let total_amount: f64 = lines.iter().map(|l| l.total_amount).sum();
        let amount: f64 = lines.iter().map(|l| l.amount).sum();
        let tax_amount: f64 = lines.iter().map(|l| l.tax_amount).sum();
        self.repo.update_chargeback_totals(chargeback_id, amount, tax_amount, total_amount, total_amount).await?;
        Ok(())
    }

    // ========================================================================
    // Activity Log
    // ========================================================================

    /// List activities for a chargeback
    pub async fn list_activities(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackActivity>> {
        self.repo.list_activities(chargeback_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get chargeback summary dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ChargebackSummary> {
        self.repo.get_dashboard(org_id).await
    }

    // ========================================================================
    // Exported validation functions for handler use
    // ========================================================================

    pub fn valid_reason_codes() -> &'static [&'static str] { VALID_REASON_CODES }
    pub fn valid_categories() -> &'static [&'static str] { VALID_CATEGORIES }
    pub fn valid_statuses() -> &'static [&'static str] { VALID_STATUSES }
    pub fn valid_priorities() -> &'static [&'static str] { VALID_PRIORITIES }
    pub fn valid_line_types() -> &'static [&'static str] { VALID_LINE_TYPES }
}
