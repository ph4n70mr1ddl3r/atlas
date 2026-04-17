//! Encumbrance Management Engine
//!
//! Manages financial commitments before actual expenditure.
//! Tracks encumbrance types, entries, lines, liquidations,
//! and year-end carry-forward processing.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Encumbrance Management

use atlas_shared::{
    EncumbranceType, EncumbranceEntry, EncumbranceLine,
    EncumbranceLiquidation, EncumbranceCarryForward, EncumbranceSummary,
    AtlasError, AtlasResult,
};
use super::EncumbranceRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid encumbrance categories
const VALID_CATEGORIES: &[&str] = &[
    "commitment", "obligation", "preliminary",
];

/// Valid entry statuses
const VALID_ENTRY_STATUSES: &[&str] = &[
    "draft", "active", "partially_liquidated",
    "fully_liquidated", "cancelled", "expired",
];

/// Valid liquidation types
const VALID_LIQUIDATION_TYPES: &[&str] = &[
    "full", "partial", "final",
];

/// Valid liquidation statuses
const VALID_LIQUIDATION_STATUSES: &[&str] = &[
    "draft", "processed", "reversed",
];

/// Valid carry-forward statuses
const VALID_CARRY_FORWARD_STATUSES: &[&str] = &[
    "draft", "processing", "completed", "reversed",
];

/// Encumbrance Management Engine
pub struct EncumbranceEngine {
    repository: Arc<dyn EncumbranceRepository>,
}

impl EncumbranceEngine {
    pub fn new(repository: Arc<dyn EncumbranceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Encumbrance Types
    // ========================================================================

    /// Create or update an encumbrance type
    pub async fn create_encumbrance_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        allow_manual_entry: bool,
        default_encumbrance_account_code: Option<&str>,
        allow_carry_forward: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceType> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Encumbrance type code and name are required".to_string(),
            ));
        }
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}",
                category, VALID_CATEGORIES.join(", ")
            )));
        }
        if priority < 0 {
            return Err(AtlasError::ValidationFailed(
                "Priority must be non-negative".to_string(),
            ));
        }

        info!("Creating encumbrance type '{}' for org {}", code, org_id);

        self.repository.create_encumbrance_type(
            org_id, code, name, description, category,
            allow_manual_entry, default_encumbrance_account_code,
            allow_carry_forward, priority, created_by,
        ).await
    }

    /// Get an encumbrance type by code
    pub async fn get_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<EncumbranceType>> {
        self.repository.get_encumbrance_type(org_id, code).await
    }

    /// List all encumbrance types for an organization
    pub async fn list_encumbrance_types(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceType>> {
        self.repository.list_encumbrance_types(org_id).await
    }

    /// Delete (soft-delete) an encumbrance type
    pub async fn delete_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_encumbrance_type(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance type '{}' not found", code)
            ))?;

        info!("Deleting encumbrance type {} in org {}", code, org_id);
        self.repository.delete_encumbrance_type(org_id, code).await
    }

    // ========================================================================
    // Encumbrance Entries
    // ========================================================================

    /// Create a new encumbrance entry
    pub async fn create_entry(
        &self,
        org_id: Uuid,
        encumbrance_type_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        encumbrance_date: chrono::NaiveDate,
        amount: &str,
        currency_code: &str,
        fiscal_year: Option<i32>,
        period_name: Option<&str>,
        expiry_date: Option<chrono::NaiveDate>,
        budget_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceEntry> {
        // Validate encumbrance type exists
        let enc_type = self.repository.get_encumbrance_type(org_id, encumbrance_type_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance type '{}' not found", encumbrance_type_code)
            ))?;

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be positive".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let entry_number = format!("ENC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating encumbrance entry {} for org {}", entry_number, org_id);

        self.repository.create_entry(
            org_id, &entry_number, enc_type.id, encumbrance_type_code,
            source_type, source_id, source_number, description,
            encumbrance_date, amount, amount,
            currency_code, "draft", fiscal_year, period_name,
            expiry_date, budget_line_id, created_by,
        ).await
    }

    /// Get an encumbrance entry by ID
    pub async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<EncumbranceEntry>> {
        self.repository.get_entry(id).await
    }

    /// Get an encumbrance entry by number
    pub async fn get_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<EncumbranceEntry>> {
        self.repository.get_entry_by_number(org_id, entry_number).await
    }

    /// List encumbrance entries with optional filters
    pub async fn list_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        encumbrance_type_code: Option<&str>,
        source_type: Option<&str>,
        fiscal_year: Option<i32>,
    ) -> AtlasResult<Vec<EncumbranceEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_ENTRY_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_entries(org_id, status, encumbrance_type_code, source_type, fiscal_year).await
    }

    /// Activate a draft encumbrance entry (approves the commitment)
    pub async fn activate_entry(&self, entry_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<EncumbranceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance entry {} not found", entry_id)
            ))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate entry in '{}' status. Must be 'draft'.",
                entry.status
            )));
        }

        info!("Activating encumbrance entry {}", entry.entry_number);
        self.repository.update_entry_status(entry_id, "active", approved_by, None, None).await
    }

    /// Cancel an encumbrance entry
    pub async fn cancel_entry(
        &self,
        entry_id: Uuid,
        cancelled_by: Uuid,
        reason: &str,
    ) -> AtlasResult<EncumbranceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance entry {} not found", entry_id)
            ))?;

        if entry.status == "fully_liquidated" || entry.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel entry in '{}' status", entry.status
            )));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cancellation reason is required".to_string(),
            ));
        }

        info!("Cancelling encumbrance entry {} - reason: {}", entry.entry_number, reason);
        self.repository.update_entry_status(entry_id, "cancelled", None, Some(cancelled_by), Some(reason)).await
    }

    // ========================================================================
    // Encumbrance Lines
    // ========================================================================

    /// Add a line to an encumbrance entry
    pub async fn add_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        account_code: &str,
        account_description: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        amount: &str,
        encumbrance_account_code: Option<&str>,
        source_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLine> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance entry {} not found", entry_id)
            ))?;

        if entry.status == "cancelled" || entry.status == "fully_liquidated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to entry in '{}' status", entry.status
            )));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be positive".to_string(),
            ));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account code is required".to_string(),
            ));
        }

        // Get next line number
        let existing_lines = self.repository.list_lines_by_entry(entry_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to encumbrance entry {}", line_number, entry.entry_number);

        self.repository.create_line(
            org_id, entry_id, line_number, account_code, account_description,
            department_id, department_name, project_id, project_name, cost_center,
            amount, amount, encumbrance_account_code, source_line_id, created_by,
        ).await
    }

    /// Get an encumbrance line by ID
    pub async fn get_line(&self, id: Uuid) -> AtlasResult<Option<EncumbranceLine>> {
        self.repository.get_line(id).await
    }

    /// List lines for an encumbrance entry
    pub async fn list_lines(&self, entry_id: Uuid) -> AtlasResult<Vec<EncumbranceLine>> {
        self.repository.list_lines_by_entry(entry_id).await
    }

    /// Delete an encumbrance line (only from draft entries)
    pub async fn delete_line(&self, line_id: Uuid) -> AtlasResult<()> {
        let line = self.repository.get_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance line {} not found", line_id)
            ))?;

        let entry = self.repository.get_entry(line.entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance entry {} not found", line.entry_id)
            ))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only delete lines from draft entries".to_string(),
            ));
        }

        info!("Deleting encumbrance line {} from entry {}", line_id, entry.entry_number);
        self.repository.delete_line(line_id).await
    }

    // ========================================================================
    // Liquidations
    // ========================================================================

    /// Liquidate (reduce) an encumbrance when actual expenditure occurs
    pub async fn liquidate(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_id: Option<Uuid>,
        liquidation_type: &str,
        liquidation_amount: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        liquidation_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLiquidation> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Encumbrance entry {} not found", entry_id)
            ))?;

        if entry.status != "active" && entry.status != "partially_liquidated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot liquidate entry in '{}' status. Must be 'active' or 'partially_liquidated'.",
                entry.status
            )));
        }

        if !VALID_LIQUIDATION_TYPES.contains(&liquidation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid liquidation type '{}'. Must be one of: {}",
                liquidation_type, VALID_LIQUIDATION_TYPES.join(", ")
            )));
        }

        let amount: f64 = liquidation_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Liquidation amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Liquidation amount must be positive".to_string(),
            ));
        }

        let current: f64 = entry.current_amount.parse().unwrap_or(0.0);
        if amount > current + 0.01 {
            return Err(AtlasError::ValidationFailed(format!(
                "Liquidation amount {} exceeds current encumbrance amount {}",
                amount, current
            )));
        }

        let liquidation_number = format!("LIQ-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Liquidating {} from encumbrance entry {} ({})", amount, entry.entry_number, liquidation_type);

        let liquidation = self.repository.create_liquidation(
            org_id, &liquidation_number, entry_id, line_id,
            liquidation_type, liquidation_amount,
            source_type, source_id, source_number, description,
            liquidation_date, created_by,
        ).await?;

        // Update entry amounts
        let new_current = (current - amount).max(0.0);
        let prev_liquidated: f64 = entry.liquidated_amount.parse().unwrap_or(0.0);
        let new_liquidated = prev_liquidated + amount;
        let new_status = if new_current < 0.01 {
            "fully_liquidated"
        } else {
            "partially_liquidated"
        };

        self.repository.update_entry_amounts(
            entry_id,
            &format!("{:.2}", new_current),
            &format!("{:.2}", new_liquidated),
            &entry.adjusted_amount,
            new_status,
        ).await?;

        // Update line amounts if a specific line was targeted
        if let Some(lid) = line_id {
            let line = self.repository.get_line(lid).await?.unwrap();
            let line_current: f64 = line.current_amount.parse().unwrap_or(0.0);
            let line_prev_liq: f64 = line.liquidated_amount.parse().unwrap_or(0.0);
            let new_line_current = (line_current - amount).max(0.0);
            let new_line_liq = line_prev_liq + amount;

            self.repository.update_line_amounts(
                lid,
                &format!("{:.2}", new_line_current),
                &format!("{:.2}", new_line_liq),
            ).await?;
        }

        // Mark liquidation as processed
        self.repository.update_liquidation_status(liquidation.id, "processed", None, None).await?;

        Ok(liquidation)
    }

    /// Reverse a liquidation
    pub async fn reverse_liquidation(
        &self,
        liquidation_id: Uuid,
        reason: &str,
    ) -> AtlasResult<EncumbranceLiquidation> {
        let liquidation = self.repository.get_liquidation(liquidation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Liquidation {} not found", liquidation_id)
            ))?;

        if liquidation.status != "processed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse liquidation in '{}' status. Must be 'processed'.",
                liquidation.status
            )));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reversal reason is required".to_string(),
            ));
        }

        let amount: f64 = liquidation.liquidation_amount.parse().unwrap_or(0.0);

        info!("Reversing liquidation {} ({})", liquidation.liquidation_number, amount);

        // Restore entry amounts
        let entry = self.repository.get_entry(liquidation.encumbrance_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Entry not found")
            ))?;
        let current: f64 = entry.current_amount.parse().unwrap_or(0.0);
        let prev_liquidated: f64 = entry.liquidated_amount.parse().unwrap_or(0.0);

        let new_current = current + amount;
        let new_liquidated = (prev_liquidated - amount).max(0.0);
        let new_status = if new_liquidated < 0.01 {
            "active"
        } else {
            "partially_liquidated"
        };

        self.repository.update_entry_amounts(
            entry.id,
            &format!("{:.2}", new_current),
            &format!("{:.2}", new_liquidated),
            &entry.adjusted_amount,
            new_status,
        ).await?;

        // Restore line amounts if applicable
        if let Some(lid) = liquidation.encumbrance_line_id {
            let line = self.repository.get_line(lid).await?.unwrap();
            let line_current: f64 = line.current_amount.parse().unwrap_or(0.0);
            let line_liq: f64 = line.liquidated_amount.parse().unwrap_or(0.0);

            self.repository.update_line_amounts(
                lid,
                &format!("{:.2}", line_current + amount),
                &format!("{:.2}", (line_liq - amount).max(0.0)),
            ).await?;
        }

        self.repository.update_liquidation_status(
            liquidation_id, "reversed", Some(liquidation_id), Some(reason),
        ).await
    }

    /// List liquidations with optional filters
    pub async fn list_liquidations(
        &self,
        org_id: Uuid,
        entry_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<EncumbranceLiquidation>> {
        if let Some(s) = status {
            if !VALID_LIQUIDATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_LIQUIDATION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_liquidations(org_id, entry_id, status).await
    }

    // ========================================================================
    // Year-End Carry-Forward
    // ========================================================================

    /// Process year-end carry-forward for open encumbrances
    pub async fn process_carry_forward(
        &self,
        org_id: Uuid,
        from_fiscal_year: i32,
        to_fiscal_year: i32,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceCarryForward> {
        if from_fiscal_year >= to_fiscal_year {
            return Err(AtlasError::ValidationFailed(
                "Target fiscal year must be after source fiscal year".to_string(),
            ));
        }

        let batch_number = format!("CF-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Processing carry-forward {} from FY{} to FY{}",
            batch_number, from_fiscal_year, to_fiscal_year);

        let carry_forward = self.repository.create_carry_forward(
            org_id, &batch_number, from_fiscal_year, to_fiscal_year,
            description, created_by,
        ).await?;

        // Find all active/partially liquidated entries for the source fiscal year
        let entries = self.repository.list_entries(
            org_id, None, None, None, Some(from_fiscal_year),
        ).await?;

        let mut carried_count = 0i32;
        let mut carried_amount = 0.0f64;

        for entry in &entries {
            if entry.status != "active" && entry.status != "partially_liquidated" {
                continue;
            }

            // Check if the encumbrance type allows carry-forward
            let enc_type = self.repository.get_encumbrance_type_by_id(entry.encumbrance_type_id).await?;
            if let Some(et) = enc_type {
                if !et.allow_carry_forward {
                    continue;
                }
            }

            let current: f64 = entry.current_amount.parse().unwrap_or(0.0);
            if current <= 0.0 {
                continue;
            }

            // Create a new entry in the target fiscal year
            let new_entry_number = format!("ENC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
            let _new_entry = self.repository.create_entry(
                org_id, &new_entry_number, entry.encumbrance_type_id,
                &entry.encumbrance_type_code,
                entry.source_type.as_deref(), entry.source_id,
                entry.source_number.as_deref(),
                Some(&format!("Carry-forward from FY{} - {}", from_fiscal_year, entry.entry_number)),
                entry.encumbrance_date,
                &format!("{:.2}", current),
                &format!("{:.2}", current),
                &entry.currency_code,
                "active", Some(to_fiscal_year), None,
                entry.expiry_date, entry.budget_line_id,
                created_by,
            ).await?;

            carried_count += 1;
            carried_amount += current;
        }

        // Update carry-forward batch
        let result = self.repository.update_carry_forward_status(
            carry_forward.id,
            "completed",
            carried_count,
            &format!("{:.2}", carried_amount),
            created_by,
        ).await?;

        info!("Carry-forward complete: {} entries, {} amount", carried_count, carried_amount);

        Ok(result)
    }

    /// List carry-forward batches
    pub async fn list_carry_forwards(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceCarryForward>> {
        self.repository.list_carry_forwards(org_id).await
    }

    /// Get a carry-forward batch
    pub async fn get_carry_forward(&self, id: Uuid) -> AtlasResult<Option<EncumbranceCarryForward>> {
        self.repository.get_carry_forward(id).await
    }

    // ========================================================================
    // Dashboard / Reporting
    // ========================================================================

    /// Generate an encumbrance summary for budgetary control
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<EncumbranceSummary> {
        let entries = self.repository.list_entries(org_id, None, None, None, None).await?;

        let mut total_active = 0.0f64;
        let mut total_liquidated = 0.0f64;
        let mut total_adjusted = 0.0f64;
        let mut active_count = 0i32;
        let mut expiring_soon_count = 0i32;
        let mut expiring_soon_amount = 0.0f64;

        let mut by_status: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        let mut by_type: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        let mut by_account: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        let mut by_department: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();

        let today = chrono::Utc::now().date_naive();
        let thirty_days = today + chrono::Duration::days(30);

        for entry in &entries {
            let current: f64 = entry.current_amount.parse().unwrap_or(0.0);
            let liquidated: f64 = entry.liquidated_amount.parse().unwrap_or(0.0);
            let adjusted: f64 = entry.adjusted_amount.parse().unwrap_or(0.0);

            let status_entry = by_status.entry(entry.status.clone()).or_insert((0, 0.0));
            status_entry.0 += 1;
            status_entry.1 += current;

            let type_entry = by_type.entry(entry.encumbrance_type_code.clone()).or_insert((0, 0.0));
            type_entry.0 += 1;
            type_entry.1 += current;

            if entry.status == "active" || entry.status == "partially_liquidated" {
                total_active += current;
                active_count += 1;

                if let Some(expiry) = entry.expiry_date {
                    if expiry <= thirty_days {
                        expiring_soon_count += 1;
                        expiring_soon_amount += current;
                    }
                }
            }

            total_liquidated += liquidated;
            total_adjusted += adjusted;
        }

        // Gather account and department breakdowns from lines
        for entry in &entries {
            if entry.status == "active" || entry.status == "partially_liquidated" {
                let lines = self.repository.list_lines_by_entry(entry.id).await?;
                for line in &lines {
                    let current: f64 = line.current_amount.parse().unwrap_or(0.0);

                    let acct = by_account.entry(line.account_code.clone()).or_insert((0, 0.0));
                    acct.0 += 1;
                    acct.1 += current;

                    if let Some(dept) = &line.department_name {
                        let dept_entry = by_department.entry(dept.clone()).or_insert((0, 0.0));
                        dept_entry.0 += 1;
                        dept_entry.1 += current;
                    }
                }
            }
        }

        let by_status_json: serde_json::Value = by_status.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "status": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        let by_type_json: serde_json::Value = by_type.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "type": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        let by_account_json: serde_json::Value = by_account.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "account": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        let by_department_json: serde_json::Value = by_department.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "department": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        Ok(EncumbranceSummary {
            total_active_amount: format!("{:.2}", total_active),
            total_liquidated_amount: format!("{:.2}", total_liquidated),
            total_adjusted_amount: format!("{:.2}", total_adjusted),
            active_entry_count: active_count,
            entries_by_status: by_status_json,
            entries_by_type: by_type_json,
            by_account: by_account_json,
            by_department: by_department_json,
            expiring_soon_count,
            expiring_soon_amount: format!("{:.2}", expiring_soon_amount),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_categories() {
        assert!(VALID_CATEGORIES.contains(&"commitment"));
        assert!(VALID_CATEGORIES.contains(&"obligation"));
        assert!(VALID_CATEGORIES.contains(&"preliminary"));
    }

    #[test]
    fn test_valid_entry_statuses() {
        assert!(VALID_ENTRY_STATUSES.contains(&"draft"));
        assert!(VALID_ENTRY_STATUSES.contains(&"active"));
        assert!(VALID_ENTRY_STATUSES.contains(&"partially_liquidated"));
        assert!(VALID_ENTRY_STATUSES.contains(&"fully_liquidated"));
        assert!(VALID_ENTRY_STATUSES.contains(&"cancelled"));
        assert!(VALID_ENTRY_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_valid_liquidation_types() {
        assert!(VALID_LIQUIDATION_TYPES.contains(&"full"));
        assert!(VALID_LIQUIDATION_TYPES.contains(&"partial"));
        assert!(VALID_LIQUIDATION_TYPES.contains(&"final"));
    }

    #[test]
    fn test_valid_liquidation_statuses() {
        assert!(VALID_LIQUIDATION_STATUSES.contains(&"draft"));
        assert!(VALID_LIQUIDATION_STATUSES.contains(&"processed"));
        assert!(VALID_LIQUIDATION_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_carry_forward_statuses() {
        assert!(VALID_CARRY_FORWARD_STATUSES.contains(&"draft"));
        assert!(VALID_CARRY_FORWARD_STATUSES.contains(&"processing"));
        assert!(VALID_CARRY_FORWARD_STATUSES.contains(&"completed"));
        assert!(VALID_CARRY_FORWARD_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_dashboard_summary_empty() {
        let engine = EncumbranceEngine::new(Arc::new(crate::MockEncumbranceRepository));

        let rt = tokio::runtime::Runtime::new().unwrap();
        let summary = rt.block_on(async {
            engine.get_summary(Uuid::new_v4()).await.unwrap()
        });

        assert_eq!(summary.active_entry_count, 0);
        assert_eq!(summary.expiring_soon_count, 0);
        assert_eq!(summary.total_active_amount, "0.00");
        assert_eq!(summary.total_liquidated_amount, "0.00");
    }
}
