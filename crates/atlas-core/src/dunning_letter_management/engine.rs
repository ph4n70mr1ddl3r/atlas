//! Dunning Letter Management Engine
//!
//! Business logic for managing dunning letter sets, customer profiles,
//! dunning runs, and run results.
//!
//! Oracle Fusion: Financials > Receivables > Dunning Letters

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_SET_STATUSES: &[&str] = &["draft", "active", "inactive"];
const VALID_PROFILE_STATUSES: &[&str] = &["enabled", "disabled", "hold"];
const VALID_RUN_STATUSES: &[&str] = &["draft", "submitted", "completed", "cancelled"];
const VALID_RESULT_STATUSES: &[&str] = &["pending", "generated", "sent", "failed", "skipped"];
const VALID_DELIVERY_METHODS: &[&str] = &["print", "email", "both"];
const VALID_AGING_BASIS: &[&str] = &["days_overdue", "invoice_date", "due_date"];

pub struct DunningLetterManagementEngine {
    repository: Arc<dyn DunningLetterManagementRepository>,
}

impl DunningLetterManagementEngine {
    pub fn new(repository: Arc<dyn DunningLetterManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Letter Sets
    // ========================================================================

    pub async fn create_letter_set(
        &self,
        org_id: Uuid,
        set_name: &str,
        description: Option<&str>,
        number_of_levels: i32,
        minimum_overdue_days: i32,
        currency_code: &str,
        include_finance_charges: bool,
        include_unapplied_receipts: bool,
        aging_basis: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterSet> {
        if set_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Letter set name is required".into()));
        }
        if number_of_levels < 1 {
            return Err(AtlasError::ValidationFailed("Number of levels must be at least 1".into()));
        }
        if minimum_overdue_days < 0 {
            return Err(AtlasError::ValidationFailed("Minimum overdue days cannot be negative".into()));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if !VALID_AGING_BASIS.contains(&aging_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid aging basis '{}'. Must be one of: {}", aging_basis, VALID_AGING_BASIS.join(", ")
            )));
        }

        if self.repository.get_letter_set_by_name(org_id, set_name).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Letter set '{}' already exists", set_name)));
        }

        info!("Creating dunning letter set '{}'", set_name);
        self.repository.create_letter_set(
            org_id, set_name, description, number_of_levels,
            minimum_overdue_days, currency_code, include_finance_charges,
            include_unapplied_receipts, aging_basis, created_by,
        ).await
    }

    pub async fn get_letter_set(&self, id: Uuid) -> AtlasResult<Option<DunningLetterSet>> {
        self.repository.get_letter_set(id).await
    }

    pub async fn list_letter_sets(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DunningLetterSet>> {
        if let Some(s) = status {
            if !VALID_SET_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SET_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_letter_sets(org_id, status).await
    }

    pub async fn activate_letter_set(&self, id: Uuid) -> AtlasResult<DunningLetterSet> {
        let set = self.repository.get_letter_set(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Letter set {} not found", id)))?;

        if set.status != "draft" && set.status != "inactive" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate letter set in '{}' status. Must be 'draft' or 'inactive'.", set.status)
            ));
        }

        // Verify the set has at least one line
        let lines = self.repository.list_letter_set_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot activate letter set without at least one level line".into()
            ));
        }

        info!("Activating dunning letter set '{}'", set.set_name);
        self.repository.update_letter_set_status(id, "active").await
    }

    pub async fn deactivate_letter_set(&self, id: Uuid) -> AtlasResult<DunningLetterSet> {
        let set = self.repository.get_letter_set(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Letter set {} not found", id)))?;

        if set.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot deactivate letter set in '{}' status. Must be 'active'.", set.status)
            ));
        }

        info!("Deactivating dunning letter set '{}'", set.set_name);
        self.repository.update_letter_set_status(id, "inactive").await
    }

    // ========================================================================
    // Letter Set Lines
    // ========================================================================

    pub async fn add_letter_set_line(
        &self,
        set_id: Uuid,
        level_number: i32,
        level_name: &str,
        min_days_overdue: i32,
        max_days_overdue: Option<i32>,
        minimum_amount: &str,
        letter_template: Option<&str>,
        delivery_method: &str,
        apply_credit_hold: bool,
        assess_finance_charges: bool,
        letter_text: Option<&str>,
        escalation_days: Option<i32>,
    ) -> AtlasResult<DunningLetterSetLine> {
        let set = self.repository.get_letter_set(set_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Letter set {} not found", set_id)))?;

        if set.status == "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot add lines to an active letter set. Deactivate first.".into()
            ));
        }

        if level_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Level name is required".into()));
        }
        if level_number < 1 {
            return Err(AtlasError::ValidationFailed("Level number must be at least 1".into()));
        }
        if !VALID_DELIVERY_METHODS.contains(&delivery_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid delivery method '{}'. Must be one of: {}", delivery_method, VALID_DELIVERY_METHODS.join(", ")
            )));
        }
        if let Some(max) = max_days_overdue {
            if max <= min_days_overdue {
                return Err(AtlasError::ValidationFailed(
                    "Max days overdue must be greater than min days overdue".into()
                ));
            }
        }
        if let Ok(amt) = minimum_amount.parse::<f64>() {
            if amt < 0.0 {
                return Err(AtlasError::ValidationFailed("Minimum amount cannot be negative".into()));
            }
        }

        info!("Adding level {} to dunning letter set '{}'", level_number, set.set_name);
        self.repository.add_letter_set_line(
            set_id, level_number, level_name, min_days_overdue,
            max_days_overdue, minimum_amount, letter_template,
            delivery_method, apply_credit_hold, assess_finance_charges,
            letter_text, escalation_days,
        ).await
    }

    pub async fn list_letter_set_lines(&self, set_id: Uuid) -> AtlasResult<Vec<DunningLetterSetLine>> {
        self.repository.list_letter_set_lines(set_id).await
    }

    // ========================================================================
    // Dunning Profiles
    // ========================================================================

    pub async fn create_profile(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        letter_set_id: Option<Uuid>,
        minimum_overdue_amount: &str,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        preferred_delivery_method: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningProfile> {
        if let Some(method) = preferred_delivery_method {
            if !VALID_DELIVERY_METHODS.contains(&method) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid delivery method '{}'. Must be one of: {}", method, VALID_DELIVERY_METHODS.join(", ")
                )));
            }
        }
        if let Ok(amt) = minimum_overdue_amount.parse::<f64>() {
            if amt < 0.0 {
                return Err(AtlasError::ValidationFailed("Minimum overdue amount cannot be negative".into()));
            }
        }

        // Check for duplicate customer
        if self.repository.get_profile_by_customer(org_id, customer_id).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Dunning profile already exists for customer {}", customer_id
            )));
        }

        // Verify letter set exists if provided
        if let Some(ls_id) = letter_set_id {
            let ls = self.repository.get_letter_set(ls_id).await?;
            if ls.is_none() {
                return Err(AtlasError::ValidationFailed(format!(
                    "Letter set {} not found", ls_id
                )));
            }
        }

        info!("Creating dunning profile for customer {}", customer_id);
        self.repository.create_profile(
            org_id, customer_id, customer_name, letter_set_id,
            minimum_overdue_amount, contact_name, contact_email,
            preferred_delivery_method, notes, created_by,
        ).await
    }

    pub async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<DunningProfile>> {
        self.repository.get_profile(id).await
    }

    pub async fn list_profiles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DunningProfile>> {
        if let Some(s) = status {
            if !VALID_PROFILE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_PROFILE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_profiles(org_id, status).await
    }

    pub async fn enable_profile(&self, id: Uuid) -> AtlasResult<DunningProfile> {
        let profile = self.repository.get_profile(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Profile {} not found", id)))?;
        if profile.dunning_status == "enabled" {
            return Err(AtlasError::WorkflowError("Profile is already enabled".into()));
        }
        self.repository.update_profile_status(id, "enabled", None).await
    }

    pub async fn disable_profile(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<DunningProfile> {
        let _profile = self.repository.get_profile(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Profile {} not found", id)))?;
        info!("Disabling dunning profile {}", id);
        self.repository.update_profile_status(id, "disabled", reason).await
    }

    pub async fn hold_profile(&self, id: Uuid, reason: &str) -> AtlasResult<DunningProfile> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Hold reason is required".into()));
        }
        let _profile = self.repository.get_profile(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Profile {} not found", id)))?;
        info!("Putting dunning profile {} on hold", id);
        self.repository.update_profile_status(id, "hold", Some(reason)).await
    }

    // ========================================================================
    // Dunning Runs
    // ========================================================================

    pub async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        description: Option<&str>,
        letter_set_id: Option<Uuid>,
        run_date: chrono::NaiveDate,
        aging_as_of_date: chrono::NaiveDate,
        currency_code: &str,
        minimum_amount_filter: Option<&str>,
        specific_level: Option<i32>,
        customer_id_filter: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterRun> {
        if run_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Run number is required".into()));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if let Some(level) = specific_level {
            if level < 1 {
                return Err(AtlasError::ValidationFailed("Specific level must be at least 1".into()));
            }
        }

        if self.repository.get_run_by_number(org_id, run_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Run number '{}' already exists", run_number)));
        }

        // Verify letter set exists and is active if provided
        if let Some(ls_id) = letter_set_id {
            let ls = self.repository.get_letter_set(ls_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Letter set {} not found", ls_id)))?;
            if ls.status != "active" {
                return Err(AtlasError::ValidationFailed(
                    "Letter set must be active to create a run".into()
                ));
            }
        }

        info!("Creating dunning letter run '{}'", run_number);
        self.repository.create_run(
            org_id, run_number, description, letter_set_id,
            run_date, aging_as_of_date, currency_code,
            minimum_amount_filter, specific_level, customer_id_filter,
            notes, created_by,
        ).await
    }

    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRun>> {
        self.repository.get_run(id).await
    }

    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DunningLetterRun>> {
        if let Some(s) = status {
            if !VALID_RUN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RUN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_runs(org_id, status).await
    }

    pub async fn submit_run(&self, id: Uuid) -> AtlasResult<DunningLetterRun> {
        let run = self.repository.get_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit run in '{}' status. Must be 'draft'.", run.status)
            ));
        }

        info!("Submitting dunning letter run '{}'", run.run_number);
        self.repository.update_run_status(id, "submitted").await
    }

    pub async fn complete_run(&self, id: Uuid) -> AtlasResult<DunningLetterRun> {
        let run = self.repository.get_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", id)))?;

        if run.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete run in '{}' status. Must be 'submitted'.", run.status)
            ));
        }

        info!("Completing dunning letter run '{}'", run.run_number);
        self.repository.update_run_status(id, "completed").await
    }

    pub async fn cancel_run(&self, id: Uuid) -> AtlasResult<DunningLetterRun> {
        let run = self.repository.get_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", id)))?;

        if run.status == "completed" || run.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel run in '{}' status.", run.status)
            ));
        }

        info!("Cancelling dunning letter run '{}'", run.run_number);
        self.repository.update_run_status(id, "cancelled").await
    }

    // ========================================================================
    // Run Results
    // ========================================================================

    pub async fn add_run_result(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        customer_number: Option<&str>,
        profile_id: Option<Uuid>,
        dunning_level: i32,
        level_name: Option<&str>,
        number_of_overdue_items: i32,
        total_overdue_amount: &str,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        days_overdue: i32,
        finance_charge_amount: &str,
        letter_template: Option<&str>,
        delivery_method: Option<&str>,
        status: &str,
        reason: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult> {
        // Verify run exists and is in draft/submitted
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "draft" && run.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add results to run in '{}' status", run.status)
            ));
        }

        if !VALID_RESULT_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid result status '{}'. Must be one of: {}", status, VALID_RESULT_STATUSES.join(", ")
            )));
        }

        let result = self.repository.create_run_result(
            org_id, run_id, customer_id, customer_name, customer_number,
            profile_id, dunning_level, level_name, number_of_overdue_items,
            total_overdue_amount, oldest_overdue_date, days_overdue,
            finance_charge_amount, letter_template, delivery_method,
            status, reason,
        ).await?;

        // Update run stats
        let all_results = self.repository.list_run_results(run_id).await?;
        let total_customers = all_results.len() as i32;
        let total_letters = all_results.iter().filter(|r| r.status != "skipped").count() as i32;
        let total_amount: f64 = all_results.iter()
            .filter(|r| r.status != "skipped")
            .map(|r| r.total_overdue_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_run_stats(
            run_id,
            total_customers,
            total_letters,
            &format!("{:.2}", total_amount),
        ).await?;

        Ok(result)
    }

    pub async fn list_run_results(&self, run_id: Uuid) -> AtlasResult<Vec<DunningLetterRunResult>> {
        self.repository.list_run_results(run_id).await
    }

    pub async fn get_run_result(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRunResult>> {
        self.repository.get_run_result(id).await
    }

    pub async fn mark_result_sent(
        &self,
        id: Uuid,
        delivery_confirmation: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult> {
        let result = self.repository.get_run_result(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Result {} not found", id)))?;

        if result.status != "generated" && result.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot mark result as sent from '{}' status", result.status)
            ));
        }

        let today = chrono::Utc::now().date_naive();

        // Update the result
        let updated = self.repository.update_run_result_status(
            id, "sent", Some(today), delivery_confirmation, None,
        ).await?;

        // Update the customer's profile dunning tracking
        if let Some(profile_id) = result.profile_id {
            let _ = self.repository.update_profile_dunning_sent(
                profile_id, result.dunning_level, today,
            ).await;
        }

        Ok(updated)
    }

    pub async fn mark_result_failed(
        &self,
        id: Uuid,
        reason: &str,
    ) -> AtlasResult<DunningLetterRunResult> {
        let result = self.repository.get_run_result(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Result {} not found", id)))?;

        if result.status != "pending" && result.status != "generated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot mark result as failed from '{}' status", result.status)
            ));
        }

        self.repository.update_run_result_status(
            id, "failed", None, None, Some(reason),
        ).await
    }
}
