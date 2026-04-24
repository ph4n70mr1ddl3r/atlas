//! Absence Engine Implementation
//!
//! Manages absence types, absence plans with accrual rules, employee absence
//! entries, balance tracking, and approval workflows.
//!
//! Oracle Fusion Cloud HCM equivalent: HCM > Absence Management

use atlas_shared::{
    AbsenceType, AbsencePlan, AbsenceBalance, AbsenceEntry, AbsenceEntryHistory,
    AbsenceDashboard, AtlasError, AtlasResult,
};
use super::AbsenceRepository;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use chrono::Datelike;

/// Valid absence categories
const VALID_CATEGORIES: &[&str] = &[
    "general", "sick", "vacation", "parental",
    "bereavement", "jury_duty", "personal", "sabbatical",
];

/// Valid plan types
const VALID_PLAN_TYPES: &[&str] = &[
    "accrual", "qualification", "no_entitlement",
];

/// Valid accrual frequencies
const VALID_ACCRUAL_FREQUENCIES: &[&str] = &[
    "yearly", "monthly", "semi_monthly", "weekly",
];

/// Valid accrual units
const VALID_ACCRUAL_UNITS: &[&str] = &["days", "hours"];

/// Valid entry statuses
const VALID_ENTRY_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "cancelled",
];

/// Valid half-day periods
const VALID_HALF_DAY_PERIODS: &[&str] = &["first_half", "second_half"];

/// Absence engine for managing types, plans, entries, and balances
pub struct AbsenceEngine {
    repository: Arc<dyn AbsenceRepository>,
}

impl AbsenceEngine {
    pub fn new(repository: Arc<dyn AbsenceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Absence Type Management
    // ========================================================================

    /// Create a new absence type
    pub async fn create_absence_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        plan_type: &str,
        requires_approval: bool,
        requires_documentation: bool,
        auto_approve_below_days: f64,
        allow_negative_balance: bool,
        allow_half_day: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceType> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Absence type code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Absence type name is required".to_string(),
            ));
        }
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}", category, VALID_CATEGORIES.join(", ")
            )));
        }
        if !VALID_PLAN_TYPES.contains(&plan_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan_type '{}'. Must be one of: {}", plan_type, VALID_PLAN_TYPES.join(", ")
            )));
        }
        if auto_approve_below_days < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "auto_approve_below_days must be non-negative".to_string(),
            ));
        }

        info!("Creating absence type '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_absence_type(
            org_id, &code_upper, name, description, category, plan_type,
            requires_approval, requires_documentation,
            &format!("{:.2}", auto_approve_below_days),
            allow_negative_balance, allow_half_day, created_by,
        ).await
    }

    /// Get an absence type by code
    pub async fn get_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsenceType>> {
        self.repository.get_absence_type(org_id, &code.to_uppercase()).await
    }

    /// List all absence types for an organization
    pub async fn list_absence_types(
        &self,
        org_id: Uuid,
        category: Option<&str>,
    ) -> AtlasResult<Vec<AbsenceType>> {
        if let Some(c) = category {
            if !VALID_CATEGORIES.contains(&c) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid category filter '{}'. Must be one of: {}", c, VALID_CATEGORIES.join(", ")
                )));
            }
        }
        self.repository.list_absence_types(org_id, category).await
    }

    /// Deactivate an absence type
    pub async fn delete_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating absence type '{}' for org {}", code, org_id);
        self.repository.delete_absence_type(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Absence Plan Management
    // ========================================================================

    /// Create a new absence plan
    pub async fn create_absence_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        absence_type_code: &str,
        accrual_frequency: &str,
        accrual_rate: f64,
        accrual_unit: &str,
        carry_over_max: Option<f64>,
        carry_over_expiry_months: Option<i32>,
        max_balance: Option<f64>,
        probation_period_days: i32,
        prorate_first_year: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsencePlan> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Plan code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Plan name is required".to_string(),
            ));
        }
        if !VALID_ACCRUAL_FREQUENCIES.contains(&accrual_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid accrual_frequency '{}'. Must be one of: {}",
                accrual_frequency, VALID_ACCRUAL_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_ACCRUAL_UNITS.contains(&accrual_unit) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid accrual_unit '{}'. Must be one of: {}",
                accrual_unit, VALID_ACCRUAL_UNITS.join(", ")
            )));
        }
        if accrual_rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Accrual rate must be non-negative".to_string(),
            ));
        }
        if probation_period_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Probation period days must be non-negative".to_string(),
            ));
        }

        // Look up the absence type
        let absence_type = self.get_absence_type(org_id, absence_type_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence type '{}' not found", absence_type_code)
            ))?;

        if !absence_type.is_active {
            return Err(AtlasError::ValidationFailed(
                format!("Absence type '{}' is not active", absence_type_code)
            ));
        }

        info!("Creating absence plan '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_absence_plan(
            org_id, &code_upper, name, description,
            absence_type.id, accrual_frequency,
            &format!("{:.4}", accrual_rate), accrual_unit,
            carry_over_max.map(|v| format!("{:.2}", v)),
            carry_over_expiry_months,
            max_balance.map(|v| format!("{:.2}", v)),
            probation_period_days, prorate_first_year,
            created_by,
        ).await
    }

    /// Get an absence plan by code
    pub async fn get_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsencePlan>> {
        self.repository.get_absence_plan(org_id, &code.to_uppercase()).await
    }

    /// List all plans for an organization
    pub async fn list_absence_plans(
        &self,
        org_id: Uuid,
        absence_type_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AbsencePlan>> {
        self.repository.list_absence_plans(org_id, absence_type_id).await
    }

    /// Deactivate an absence plan
    pub async fn delete_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating absence plan '{}' for org {}", code, org_id);
        self.repository.delete_absence_plan(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Absence Entry Management
    // ========================================================================

    /// Create a new absence entry
    pub async fn create_entry(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        absence_type_code: &str,
        plan_code: Option<&str>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        duration_days: f64,
        duration_hours: Option<f64>,
        is_half_day: bool,
        half_day_period: Option<&str>,
        reason: Option<&str>,
        comments: Option<&str>,
        documentation_provided: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceEntry> {
        // Validate dates
        if start_date > end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be on or before end date".to_string(),
            ));
        }
        if duration_days <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Duration days must be positive".to_string(),
            ));
        }
        if is_half_day && half_day_period.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Half day period is required when is_half_day is true".to_string(),
            ));
        }
        if let Some(hdp) = half_day_period {
            if !VALID_HALF_DAY_PERIODS.contains(&hdp) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid half_day_period '{}'. Must be one of: {}", hdp, VALID_HALF_DAY_PERIODS.join(", ")
                )));
            }
        }

        // Look up absence type
        let absence_type = self.get_absence_type(org_id, absence_type_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence type '{}' not found", absence_type_code)
            ))?;

        if !absence_type.is_active {
            return Err(AtlasError::ValidationFailed(
                format!("Absence type '{}' is not active", absence_type_code)
            ));
        }

        // Look up plan if provided
        let plan_id = if let Some(pc) = plan_code {
            let plan = self.get_absence_plan(org_id, pc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Absence plan '{}' not found", pc)
                ))?;
            Some(plan.id)
        } else {
            None
        };

        // Check for overlapping entries
        let overlapping = self.repository.find_overlapping_entries(
            org_id, employee_id, start_date, end_date,
        ).await?;
        if !overlapping.is_empty() {
            return Err(AtlasError::Conflict(
                format!("Employee has {} overlapping absence entries", overlapping.len())
            ));
        }

        // Auto-approve if below threshold and type allows
        let auto_approve_threshold: f64 = absence_type.auto_approve_below_days.parse().unwrap_or(0.0);
        let status = if !absence_type.requires_approval
            || (auto_approve_threshold > 0.0 && duration_days <= auto_approve_threshold)
        {
            "approved"
        } else {
            "draft"
        };

        // Generate entry number
        let entry_number = format!("ABS-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating absence entry {} for employee {} type {} status {}", 
            entry_number, employee_id, absence_type_code, status);

        let entry = self.repository.create_entry(
            org_id, employee_id, employee_name,
            absence_type.id, plan_id,
            &entry_number, status,
            start_date, end_date,
            &format!("{:.4}", duration_days),
            duration_hours.map(|h| format!("{:.4}", h)),
            is_half_day, half_day_period,
            reason, comments,
            documentation_provided,
            if status == "approved" { Some("system") } else { None },
            created_by,
        ).await?;

        // Record history
        self.repository.add_history(
            entry.id,
            "create",
            None,
            Some(status),
            created_by,
            None,
        ).await?;

        Ok(entry)
    }

    /// Get an absence entry by ID (with org-scoping check)
    pub async fn get_entry(&self, org_id: Uuid, id: Uuid) -> AtlasResult<Option<AbsenceEntry>> {
        let entry = self.repository.get_entry(id).await?;
        match entry {
            Some(e) if e.organization_id != org_id => Ok(None),
            other => Ok(other),
        }
    }

    /// List entries with optional filters
    pub async fn list_entries(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        absence_type_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<AbsenceEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_ENTRY_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_entries(org_id, employee_id, absence_type_id, status).await
    }

    /// Submit a draft entry for approval
    pub async fn submit_entry(&self, org_id: Uuid, entry_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<AbsenceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence entry {} not found", entry_id)
            ))?;

        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)));
        }

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit entry in '{}' status. Must be 'draft'.", entry.status)
            ));
        }

        info!("Submitting absence entry {} by {:?}", entry.entry_number, submitted_by);
        let updated = self.repository.update_entry_status(entry_id, "submitted", None, None, None).await?;

        self.repository.add_history(
            entry_id, "submit", Some("draft"), Some("submitted"), submitted_by, None,
        ).await?;

        Ok(updated)
    }

    /// Approve a submitted entry
    pub async fn approve_entry(&self, org_id: Uuid, entry_id: Uuid, approved_by: Uuid) -> AtlasResult<AbsenceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence entry {} not found", entry_id)
            ))?;

        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)));
        }

        if entry.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve entry in '{}' status. Must be 'submitted'.", entry.status)
            ));
        }

        info!("Approving absence entry {} by {}", entry.entry_number, approved_by);
        let updated = self.repository.update_entry_status(
            entry_id, "approved", Some(approved_by), None, None,
        ).await?;

        // Update balance if plan exists
        if let Some(plan_id) = entry.plan_id {
            if let Err(e) = self.update_balance_on_approval(
                entry.organization_id, entry.employee_id, plan_id,
                entry.duration_days.parse::<f64>().unwrap_or(0.0),
            ).await {
                warn!("Failed to update balance for entry {}: {}", entry_id, e);
            }
        }

        self.repository.add_history(
            entry_id, "approve", Some("submitted"), Some("approved"),
            Some(approved_by), None,
        ).await?;

        Ok(updated)
    }

    /// Reject a submitted entry
    pub async fn reject_entry(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        rejected_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<AbsenceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence entry {} not found", entry_id)
            ))?;

        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)));
        }

        if entry.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject entry in '{}' status. Must be 'submitted'.", entry.status)
            ));
        }

        info!("Rejecting absence entry {} by {}", entry.entry_number, rejected_by);
        let updated = self.repository.update_entry_status(
            entry_id, "rejected", Some(rejected_by), reason, None,
        ).await?;

        self.repository.add_history(
            entry_id, "reject", Some("submitted"), Some("rejected"),
            Some(rejected_by), reason,
        ).await?;

        Ok(updated)
    }

    /// Cancel an entry (draft or submitted)
    pub async fn cancel_entry(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<AbsenceEntry> {
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence entry {} not found", entry_id)
            ))?;

        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)));
        }

        if entry.status != "draft" && entry.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel entry in '{}' status. Must be 'draft' or 'submitted'.", entry.status)
            ));
        }

        let old_status = entry.status.clone();
        info!("Cancelling absence entry {} reason: {:?}", entry.entry_number, reason);
        let updated = self.repository.update_entry_status(
            entry_id, "cancelled", None, None, reason,
        ).await?;

        self.repository.add_history(
            entry_id, "cancel", Some(&old_status), Some("cancelled"), None, reason,
        ).await?;

        Ok(updated)
    }

    // ========================================================================
    // Balance Management
    // ========================================================================

    /// Get or create an absence balance for an employee/plan/period
    pub async fn get_or_create_balance(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<AbsenceBalance> {
        if period_start > period_end {
            return Err(AtlasError::ValidationFailed(
                "Period start must be on or before period end".to_string(),
            ));
        }

        // Try to get existing balance
        if let Some(balance) = self.repository.get_balance(
            employee_id, plan_id, period_start, period_end,
        ).await? {
            return Ok(balance);
        }

        // Get plan for accrual info
        let plan = self.repository.get_plan_by_id(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Absence plan {} not found", plan_id)
            ))?;

        let accrual_rate: f64 = plan.accrual_rate.parse().unwrap_or(0.0);
        let carry_over_max: f64 = plan.carry_over_max
            .as_ref()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        // Calculate carry-over from previous period
        let prev_balance = self.repository.get_balance_for_previous_period(
            employee_id, plan_id, period_start,
        ).await?;

        let carried_over = if let Some(prev) = &prev_balance {
            let prev_remaining: f64 = prev.remaining.parse().unwrap_or(0.0);
            if carry_over_max > 0.0 {
                prev_remaining.min(carry_over_max)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let accrued = accrual_rate;
        let taken = 0.0_f64;
        let adjusted = 0.0_f64;
        let remaining = carried_over + accrued - taken - adjusted;

        info!("Creating balance for employee {} plan {} period {} to {}", 
            employee_id, plan_id, period_start, period_end);

        self.repository.create_balance(
            org_id, employee_id, plan_id,
            period_start, period_end,
            &format!("{:.4}", accrued),
            &format!("{:.4}", taken),
            &format!("{:.4}", adjusted),
            &format!("{:.4}", carried_over),
            &format!("{:.4}", remaining),
        ).await
    }

    /// List balances for an employee
    pub async fn list_balances(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
    ) -> AtlasResult<Vec<AbsenceBalance>> {
        self.repository.list_balances(org_id, employee_id).await
    }

    // ========================================================================
    // Entry History
    // ========================================================================

    /// Get history for an entry (org-scoped)
    pub async fn get_entry_history(&self, org_id: Uuid, entry_id: Uuid) -> AtlasResult<Vec<AbsenceEntryHistory>> {
        // Verify the entry belongs to the org before returning history
        let entry = self.repository.get_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)))?;
        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Absence entry {} not found", entry_id)));
        }
        self.repository.get_entry_history(entry_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get absence management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AbsenceDashboard> {
        let types = self.repository.list_absence_types(org_id, None).await?;
        let plans = self.repository.list_absence_plans(org_id, None).await?;
        let all_entries = self.repository.list_entries(org_id, None, None, None).await?;

        let total_types = types.len() as i64;
        let active_types = types.iter().filter(|t| t.is_active).count() as i64;
        let total_plans = plans.len() as i64;
        let active_plans = plans.iter().filter(|p| p.is_active).count() as i64;
        let pending_entries = all_entries.iter().filter(|e| e.status == "submitted").count() as i64;

        let today = chrono::Utc::now().date_naive();
        let approved_entries_today = all_entries.iter()
            .filter(|e| {
                e.status == "approved"
                    && e.approved_at.is_some_and(|at| at.date_naive() == today)
            })
            .count() as i64;

        // Group entries by status
        let mut by_status = serde_json::Map::new();
        for entry in &all_entries {
            let count = by_status.get(&entry.status)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) + 1;
            by_status.insert(entry.status.clone(), serde_json::json!(count));
        }

        // Group entries by type
        let mut by_type = serde_json::Map::new();
        for entry in &all_entries {
            let type_id = entry.absence_type_id.to_string();
            let count = by_type.get(&type_id)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) + 1;
            by_type.insert(type_id, serde_json::json!(count));
        }

        // Recent entries (last 10)
        let mut recent = all_entries.clone();
        recent.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent.truncate(10);

        Ok(AbsenceDashboard {
            total_types,
            active_types,
            total_plans,
            active_plans,
            pending_entries,
            approved_entries_today,
            entries_by_status: serde_json::Value::Object(by_status),
            entries_by_type: serde_json::Value::Object(by_type),
            recent_entries: recent,
        })
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Update the absence balance when an entry is approved
    async fn update_balance_on_approval(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        duration_days: f64,
    ) -> AtlasResult<()> {
        let plan = self.repository.get_plan_by_id(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Plan {} not found", plan_id)
            ))?;

        // Determine current period based on accrual frequency
        let (period_start, period_end) = self.calculate_current_period(&plan.accrual_frequency);

        // Get or create the balance
        let balance = self.get_or_create_balance(
            org_id, employee_id, plan_id, period_start, period_end,
        ).await?;

        // Update taken and remaining
        let taken: f64 = balance.taken.parse().unwrap_or(0.0);
        let accrued: f64 = balance.accrued.parse().unwrap_or(0.0);
        let carried_over: f64 = balance.carried_over.parse().unwrap_or(0.0);
        let adjusted: f64 = balance.adjusted.parse().unwrap_or(0.0);

        let new_taken = taken + duration_days;
        let new_remaining = carried_over + accrued - new_taken - adjusted;

        self.repository.update_balance(
            balance.id,
            &format!("{:.4}", new_taken),
            &format!("{:.4}", adjusted),
            &format!("{:.4}", new_remaining),
        ).await?;

        info!("Updated balance for employee {} plan {}: taken += {}", 
            employee_id, plan_id, duration_days);

        Ok(())
    }

    /// Calculate current period dates based on accrual frequency
    pub fn calculate_current_period(&self, frequency: &str) -> (chrono::NaiveDate, chrono::NaiveDate) {
        let today = chrono::Utc::now().date_naive();
        let year = today.year();

        match frequency {
            "yearly" => {
                let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(today);
                let end = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(today);
                (start, end)
            }
            "monthly" => {
                let month = today.month();
                let start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today);
                let end = {
                    let next_month = if month == 12 { 1 } else { month + 1 };
                    let next_year = if month == 12 { year + 1 } else { year };
                    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
                        .unwrap_or(today)
                        .pred_opt()
                        .unwrap_or(today)
                };
                (start, end)
            }
            "semi_monthly" => {
                let day = today.day();
                let month = today.month();
                if day <= 15 {
                    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today);
                    let end = chrono::NaiveDate::from_ymd_opt(year, month, 15).unwrap_or(today);
                    (start, end)
                } else {
                    let start = chrono::NaiveDate::from_ymd_opt(year, month, 16).unwrap_or(today);
                    let end = {
                        let next_month = if month == 12 { 1 } else { month + 1 };
                        let next_year = if month == 12 { year + 1 } else { year };
                        chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
                            .unwrap_or(today)
                            .pred_opt()
                            .unwrap_or(today)
                    };
                    (start, end)
                }
            }
            "weekly" => {
                let weekday = today.weekday().num_days_from_monday();
                let start = today - chrono::Duration::days(weekday as i64);
                let end = start + chrono::Duration::days(6);
                (start, end)
            }
            _ => {
                // Default to yearly
                let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(today);
                let end = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(today);
                (start, end)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_categories() {
        assert!(VALID_CATEGORIES.contains(&"general"));
        assert!(VALID_CATEGORIES.contains(&"sick"));
        assert!(VALID_CATEGORIES.contains(&"vacation"));
        assert!(VALID_CATEGORIES.contains(&"parental"));
        assert!(VALID_CATEGORIES.contains(&"bereavement"));
        assert!(VALID_CATEGORIES.contains(&"jury_duty"));
        assert!(VALID_CATEGORIES.contains(&"personal"));
        assert!(VALID_CATEGORIES.contains(&"sabbatical"));
    }

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"accrual"));
        assert!(VALID_PLAN_TYPES.contains(&"qualification"));
        assert!(VALID_PLAN_TYPES.contains(&"no_entitlement"));
    }

    #[test]
    fn test_valid_accrual_frequencies() {
        assert!(VALID_ACCRUAL_FREQUENCIES.contains(&"yearly"));
        assert!(VALID_ACCRUAL_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_ACCRUAL_FREQUENCIES.contains(&"semi_monthly"));
        assert!(VALID_ACCRUAL_FREQUENCIES.contains(&"weekly"));
    }

    #[test]
    fn test_valid_accrual_units() {
        assert!(VALID_ACCRUAL_UNITS.contains(&"days"));
        assert!(VALID_ACCRUAL_UNITS.contains(&"hours"));
    }

    #[test]
    fn test_valid_entry_statuses() {
        assert!(VALID_ENTRY_STATUSES.contains(&"draft"));
        assert!(VALID_ENTRY_STATUSES.contains(&"submitted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"approved"));
        assert!(VALID_ENTRY_STATUSES.contains(&"rejected"));
        assert!(VALID_ENTRY_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_calculate_yearly_period() {
        let engine = AbsenceEngine::new(Arc::new(crate::mock_repos::MockAbsenceRepository));
        let (start, end) = engine.calculate_current_period("yearly");
        assert_eq!(start.month(), 1);
        assert_eq!(start.day(), 1);
        assert_eq!(end.month(), 12);
        assert_eq!(end.day(), 31);
    }
}
