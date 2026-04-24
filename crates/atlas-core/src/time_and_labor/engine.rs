//! Time and Labor Engine Implementation
//!
//! Manages work schedules, overtime rules, time cards, time entries,
//! labor distribution, and approval workflows.
//!
//! Oracle Fusion Cloud HCM equivalent: HCM > Time and Labor

use atlas_shared::{
    WorkSchedule, OvertimeRule, TimeCard, TimeEntry, TimeCardHistory,
    LaborDistribution, TimeAndLaborDashboard,
    AtlasError, AtlasResult,
};
use super::TimeAndLaborRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid schedule types
const VALID_SCHEDULE_TYPES: &[&str] = &[
    "fixed", "flexible", "rotating", "shift",
];

/// Valid threshold types
const VALID_THRESHOLD_TYPES: &[&str] = &[
    "daily", "weekly", "both",
];

/// Valid time card statuses
const VALID_CARD_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "cancelled",
];

/// Valid entry types
const VALID_ENTRY_TYPES: &[&str] = &[
    "regular", "overtime", "double_time", "holiday", "sick", "vacation", "break",
];

/// Time and Labor engine
pub struct TimeAndLaborEngine {
    repository: Arc<dyn TimeAndLaborRepository>,
}

impl TimeAndLaborEngine {
    pub fn new(repository: Arc<dyn TimeAndLaborRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Work Schedule Management
    // ========================================================================

    /// Create or update a work schedule
    pub async fn create_work_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        schedule_type: &str,
        standard_hours_per_day: f64,
        standard_hours_per_week: f64,
        work_days_per_week: i32,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        break_duration_minutes: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WorkSchedule> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Schedule code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Schedule name is required".to_string(),
            ));
        }
        if !VALID_SCHEDULE_TYPES.contains(&schedule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule_type '{}'. Must be one of: {}", schedule_type, VALID_SCHEDULE_TYPES.join(", ")
            )));
        }
        if standard_hours_per_day <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Standard hours per day must be positive".to_string(),
            ));
        }
        if standard_hours_per_week <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Standard hours per week must be positive".to_string(),
            ));
        }
        if !(1..=7).contains(&work_days_per_week) {
            return Err(AtlasError::ValidationFailed(
                "Work days per week must be 1-7".to_string(),
            ));
        }

        info!("Creating work schedule '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_work_schedule(
            org_id, &code_upper, name, description, schedule_type,
            &format!("{:.2}", standard_hours_per_day),
            &format!("{:.2}", standard_hours_per_week),
            work_days_per_week,
            start_time, end_time, break_duration_minutes,
            created_by,
        ).await
    }

    /// Get a work schedule by code
    pub async fn get_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WorkSchedule>> {
        self.repository.get_work_schedule(org_id, &code.to_uppercase()).await
    }

    /// List all work schedules for an organization
    pub async fn list_work_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<WorkSchedule>> {
        self.repository.list_work_schedules(org_id).await
    }

    /// Deactivate a work schedule
    pub async fn delete_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating work schedule '{}' for org {}", code, org_id);
        self.repository.delete_work_schedule(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Overtime Rule Management
    // ========================================================================

    /// Create or update an overtime rule
    pub async fn create_overtime_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        threshold_type: &str,
        daily_threshold_hours: f64,
        weekly_threshold_hours: f64,
        overtime_multiplier: f64,
        double_time_threshold_hours: Option<f64>,
        double_time_multiplier: f64,
        include_holidays: bool,
        include_weekends: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<OvertimeRule> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Overtime rule code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Overtime rule name is required".to_string(),
            ));
        }
        if !VALID_THRESHOLD_TYPES.contains(&threshold_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid threshold_type '{}'. Must be one of: {}", threshold_type, VALID_THRESHOLD_TYPES.join(", ")
            )));
        }
        if overtime_multiplier < 1.0 {
            return Err(AtlasError::ValidationFailed(
                "Overtime multiplier must be >= 1.0".to_string(),
            ));
        }
        if let Some(dt) = double_time_threshold_hours {
            if dt < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Double time threshold must be non-negative".to_string(),
                ));
            }
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from must be before effective to".to_string(),
                ));
            }
        }

        info!("Creating overtime rule '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_overtime_rule(
            org_id, &code_upper, name, description, threshold_type,
            &format!("{:.2}", daily_threshold_hours),
            &format!("{:.2}", weekly_threshold_hours),
            &format!("{:.4}", overtime_multiplier),
            double_time_threshold_hours.map(|v| format!("{:.2}", v)).as_deref(),
            &format!("{:.4}", double_time_multiplier),
            include_holidays, include_weekends,
            effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get an overtime rule by code
    pub async fn get_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<OvertimeRule>> {
        self.repository.get_overtime_rule(org_id, &code.to_uppercase()).await
    }

    /// List all overtime rules for an organization
    pub async fn list_overtime_rules(&self, org_id: Uuid) -> AtlasResult<Vec<OvertimeRule>> {
        self.repository.list_overtime_rules(org_id).await
    }

    /// Deactivate an overtime rule
    pub async fn delete_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating overtime rule '{}' for org {}", code, org_id);
        self.repository.delete_overtime_rule(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Time Card Management
    // ========================================================================

    /// Create or update a time card for an employee period
    pub async fn create_time_card(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        schedule_code: Option<&str>,
        overtime_rule_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeCard> {
        if period_start > period_end {
            return Err(AtlasError::ValidationFailed(
                "Period start must be on or before period end".to_string(),
            ));
        }

        let schedule_id = if let Some(sc) = schedule_code {
            let schedule = self.get_work_schedule(org_id, sc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Work schedule '{}' not found", sc)
                ))?;
            Some(schedule.id)
        } else {
            None
        };

        let overtime_rule_id = if let Some(oc) = overtime_rule_code {
            let rule = self.get_overtime_rule(org_id, oc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Overtime rule '{}' not found", oc)
                ))?;
            Some(rule.id)
        } else {
            None
        };

        let card_number = format!("TC-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating time card {} for employee {} period {} to {}",
            card_number, employee_id, period_start, period_end);

        let card = self.repository.create_time_card(
            org_id, employee_id, employee_name, &card_number,
            period_start, period_end, schedule_id, overtime_rule_id, created_by,
        ).await?;

        self.repository.add_history(
            card.id, "create", None, Some("draft"), created_by, None,
        ).await?;

        Ok(card)
    }

    /// Get a time card by ID (scoped to org)
    pub async fn get_time_card(&self, org_id: Uuid, id: Uuid) -> AtlasResult<Option<TimeCard>> {
        let card = self.repository.get_time_card(id).await?;
        if let Some(ref c) = card {
            if c.organization_id != org_id {
                return Ok(None);
            }
        }
        Ok(card)
    }

    /// Get a time card by number
    pub async fn get_time_card_by_number(&self, org_id: Uuid, card_number: &str) -> AtlasResult<Option<TimeCard>> {
        self.repository.get_time_card_by_number(org_id, card_number).await
    }

    /// List time cards with optional filters
    pub async fn list_time_cards(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TimeCard>> {
        if let Some(s) = status {
            if !VALID_CARD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CARD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_time_cards(org_id, employee_id, status).await
    }

    /// Submit a time card for approval (scoped to org)
    pub async fn submit_time_card(&self, org_id: Uuid, card_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<TimeCard> {
        let card = self.repository.get_time_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ))?;

        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ));
        }

        if card.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit card in '{}' status. Must be 'draft'.", card.status)
            ));
        }

        info!("Submitting time card {} by {:?}", card.card_number, submitted_by);
        let updated = self.repository.update_time_card_status(card_id, "submitted", None, None).await?;

        self.repository.add_history(
            card_id, "submit", Some("draft"), Some("submitted"), submitted_by, None,
        ).await?;

        Ok(updated)
    }

    /// Approve a submitted time card (scoped to org)
    pub async fn approve_time_card(&self, org_id: Uuid, card_id: Uuid, approved_by: Uuid) -> AtlasResult<TimeCard> {
        let card = self.repository.get_time_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ))?;

        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ));
        }

        if card.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve card in '{}' status. Must be 'submitted'.", card.status)
            ));
        }

        info!("Approving time card {} by {}", card.card_number, approved_by);
        let updated = self.repository.update_time_card_status(card_id, "approved", Some(approved_by), None).await?;

        self.repository.add_history(
            card_id, "approve", Some("submitted"), Some("approved"), Some(approved_by), None,
        ).await?;

        Ok(updated)
    }

    /// Reject a submitted time card (scoped to org)
    pub async fn reject_time_card(&self, org_id: Uuid, card_id: Uuid, rejected_by: Uuid, reason: Option<&str>) -> AtlasResult<TimeCard> {
        let card = self.repository.get_time_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ))?;

        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ));
        }

        if card.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject card in '{}' status. Must be 'submitted'.", card.status)
            ));
        }

        info!("Rejecting time card {} by {}", card.card_number, rejected_by);
        let updated = self.repository.update_time_card_status(card_id, "rejected", Some(rejected_by), reason).await?;

        self.repository.add_history(
            card_id, "reject", Some("submitted"), Some("rejected"), Some(rejected_by), reason,
        ).await?;

        Ok(updated)
    }

    /// Cancel a time card (draft or submitted, scoped to org)
    pub async fn cancel_time_card(&self, org_id: Uuid, card_id: Uuid, reason: Option<&str>) -> AtlasResult<TimeCard> {
        let card = self.repository.get_time_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ))?;

        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", card_id)
            ));
        }

        if card.status != "draft" && card.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel card in '{}' status. Must be 'draft' or 'submitted'.", card.status)
            ));
        }

        let old_status = card.status.clone();
        info!("Cancelling time card {} reason: {:?}", card.card_number, reason);
        let updated = self.repository.update_time_card_status(card_id, "cancelled", None, reason).await?;

        self.repository.add_history(
            card_id, "cancel", Some(&old_status), Some("cancelled"), None, reason,
        ).await?;

        Ok(updated)
    }

    // ========================================================================
    // Time Entry Management
    // ========================================================================

    /// Add a time entry to a time card
    pub async fn create_time_entry(
        &self,
        org_id: Uuid,
        time_card_id: Uuid,
        entry_date: chrono::NaiveDate,
        entry_type: &str,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        duration_hours: f64,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        task_name: Option<&str>,
        location: Option<&str>,
        cost_center: Option<&str>,
        labor_category: Option<&str>,
        comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeEntry> {
        let card = self.repository.get_time_card(time_card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", time_card_id)
            ))?;

        if card.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add entries to a draft time card".to_string(),
            ));
        }

        if entry_date < card.period_start || entry_date > card.period_end {
            return Err(AtlasError::ValidationFailed(
                format!("Entry date {} is outside the time card period {} to {}",
                    entry_date, card.period_start, card.period_end)
            ));
        }

        if !VALID_ENTRY_TYPES.contains(&entry_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid entry_type '{}'. Must be one of: {}", entry_type, VALID_ENTRY_TYPES.join(", ")
            )));
        }

        if duration_hours <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Duration hours must be positive".to_string(),
            ));
        }

        if let (Some(st), Some(et)) = (start_time, end_time) {
            if st >= et {
                return Err(AtlasError::ValidationFailed(
                    "Start time must be before end time".to_string(),
                ));
            }
        }

        info!("Creating time entry for card {} date {} type {} hours {}",
            card.card_number, entry_date, entry_type, duration_hours);

        let entry = self.repository.create_time_entry(
            org_id, time_card_id, entry_date, entry_type,
            start_time, end_time,
            &format!("{:.4}", duration_hours),
            project_id, project_name, department_id, department_name,
            task_name, location, cost_center, labor_category,
            comments, created_by,
        ).await?;

        // Recalculate time card totals
        self.recalculate_totals(time_card_id).await?;

        Ok(entry)
    }

    /// Get a time entry by ID (scoped to org)
    pub async fn get_time_entry(&self, org_id: Uuid, id: Uuid) -> AtlasResult<Option<TimeEntry>> {
        let entry = self.repository.get_time_entry(id).await?;
        if let Some(ref e) = entry {
            if e.organization_id != org_id {
                return Ok(None);
            }
        }
        Ok(entry)
    }

    /// List time entries for a time card (scoped to org)
    pub async fn list_time_entries(&self, org_id: Uuid, time_card_id: Uuid) -> AtlasResult<Vec<TimeEntry>> {
        // Verify the time card belongs to org
        let card = self.repository.get_time_card(time_card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", time_card_id)
            ))?;
        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", time_card_id)
            ));
        }
        self.repository.list_time_entries_by_card(time_card_id).await
    }

    /// Delete a time entry (scoped to org)
    pub async fn delete_time_entry(&self, org_id: Uuid, id: Uuid) -> AtlasResult<()> {
        let entry = self.repository.get_time_entry(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time entry {} not found", id)
            ))?;

        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time entry {} not found", id)
            ));
        }

        let card = self.repository.get_time_card(entry.time_card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", entry.time_card_id)
            ))?;

        if card.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only delete entries from a draft time card".to_string(),
            ));
        }

        self.repository.delete_time_entry(id).await?;
        self.recalculate_totals(entry.time_card_id).await?;
        Ok(())
    }

    // ========================================================================
    // Time Card History
    // ========================================================================

    /// Get history for a time card (scoped to org)
    pub async fn get_time_card_history(&self, org_id: Uuid, time_card_id: Uuid) -> AtlasResult<Vec<TimeCardHistory>> {
        // Verify the time card belongs to org
        let card = self.repository.get_time_card(time_card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time card {} not found", time_card_id)
            ))?;
        if card.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time card {} not found", time_card_id)
            ));
        }
        self.repository.get_time_card_history(time_card_id).await
    }

    // ========================================================================
    // Labor Distribution
    // ========================================================================

    /// Create a labor distribution for a time entry
    pub async fn create_labor_distribution(
        &self,
        org_id: Uuid,
        time_entry_id: Uuid,
        distribution_percent: f64,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        gl_account_code: Option<&str>,
    ) -> AtlasResult<LaborDistribution> {
        let entry = self.repository.get_time_entry(time_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time entry {} not found", time_entry_id)
            ))?;

        if distribution_percent <= 0.0 || distribution_percent > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Distribution percent must be between 0 and 100".to_string(),
            ));
        }

        let duration: f64 = entry.duration_hours.parse().unwrap_or(0.0);
        let allocated = duration * (distribution_percent / 100.0);

        info!("Creating labor distribution for entry {} percent {}%", time_entry_id, distribution_percent);

        self.repository.create_labor_distribution(
            org_id, time_entry_id,
            &format!("{:.2}", distribution_percent),
            cost_center, project_id, project_name,
            department_id, department_name, gl_account_code,
            &format!("{:.4}", allocated),
        ).await
    }

    /// List labor distributions for a time entry (scoped to org)
    pub async fn list_labor_distributions(&self, org_id: Uuid, time_entry_id: Uuid) -> AtlasResult<Vec<LaborDistribution>> {
        // Verify the entry belongs to org
        let entry = self.repository.get_time_entry(time_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Time entry {} not found", time_entry_id)
            ))?;
        if entry.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(
                format!("Time entry {} not found", time_entry_id)
            ));
        }
        self.repository.list_labor_distributions_by_entry(time_entry_id).await
    }

    /// Delete a labor distribution (scoped to org)
    pub async fn delete_labor_distribution(&self, org_id: Uuid, id: Uuid) -> AtlasResult<()> {
        // Verify the distribution belongs to org via the org-scoped repository delete
        self.repository.delete_labor_distribution_org_scoped(org_id, id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get Time and Labor dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TimeAndLaborDashboard> {
        let schedules = self.repository.list_work_schedules(org_id).await?;
        let overtime_rules = self.repository.list_overtime_rules(org_id).await?;
        let all_cards = self.repository.list_time_cards(org_id, None, None).await?;

        let total_schedules = schedules.len() as i64;
        let active_schedules = schedules.iter().filter(|s| s.is_active).count() as i64;
        let total_overtime_rules = overtime_rules.len() as i64;
        let total_time_cards = all_cards.len() as i64;
        let pending_approval_count = all_cards.iter().filter(|c| c.status == "submitted").count() as i64;

        let today = chrono::Utc::now().date_naive();
        let submitted_today_count = all_cards.iter()
            .filter(|c| {
                c.submitted_at.is_some_and(|at| at.date_naive() == today)
            })
            .count() as i64;

        // Group by status
        let mut by_status = serde_json::Map::new();
        for card in &all_cards {
            let count = by_status.get(&card.status)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) + 1;
            by_status.insert(card.status.clone(), serde_json::json!(count));
        }

        // Group entries by type
        let mut hours_by_type = serde_json::Map::new();
        for card in &all_cards {
            let entries = self.repository.list_time_entries_by_card(card.id).await?;
            for entry in &entries {
                let hours: f64 = entry.duration_hours.parse().unwrap_or(0.0);
                let current = hours_by_type.get(&entry.entry_type)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                hours_by_type.insert(entry.entry_type.clone(), serde_json::json!(((current + hours) * 100.0).round() / 100.0));
            }
        }

        // Recent cards (last 10)
        let mut recent = all_cards.clone();
        recent.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent.truncate(10);

        Ok(TimeAndLaborDashboard {
            total_schedules,
            active_schedules,
            total_overtime_rules,
            total_time_cards,
            pending_approval_count,
            submitted_today_count,
            cards_by_status: serde_json::Value::Object(by_status),
            hours_by_type: serde_json::Value::Object(hours_by_type),
            recent_time_cards: recent,
        })
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate time card totals from entries
    async fn recalculate_totals(&self, time_card_id: Uuid) -> AtlasResult<()> {
        let entries = self.repository.list_time_entries_by_card(time_card_id).await?;

        let mut regular = 0.0_f64;
        let mut overtime = 0.0_f64;
        let mut double_time = 0.0_f64;

        for entry in &entries {
            let hours: f64 = entry.duration_hours.parse().unwrap_or(0.0);
            match entry.entry_type.as_str() {
                "overtime" => overtime += hours,
                "double_time" => double_time += hours,
                _ => regular += hours,
            }
        }

        let total = regular + overtime + double_time;

        self.repository.update_time_card_totals(
            time_card_id,
            &format!("{:.4}", regular),
            &format!("{:.4}", overtime),
            &format!("{:.4}", double_time),
            &format!("{:.4}", total),
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_schedule_types() {
        assert!(VALID_SCHEDULE_TYPES.contains(&"fixed"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"flexible"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"rotating"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"shift"));
    }

    #[test]
    fn test_valid_threshold_types() {
        assert!(VALID_THRESHOLD_TYPES.contains(&"daily"));
        assert!(VALID_THRESHOLD_TYPES.contains(&"weekly"));
        assert!(VALID_THRESHOLD_TYPES.contains(&"both"));
    }

    #[test]
    fn test_valid_card_statuses() {
        assert!(VALID_CARD_STATUSES.contains(&"draft"));
        assert!(VALID_CARD_STATUSES.contains(&"submitted"));
        assert!(VALID_CARD_STATUSES.contains(&"approved"));
        assert!(VALID_CARD_STATUSES.contains(&"rejected"));
        assert!(VALID_CARD_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_entry_types() {
        assert!(VALID_ENTRY_TYPES.contains(&"regular"));
        assert!(VALID_ENTRY_TYPES.contains(&"overtime"));
        assert!(VALID_ENTRY_TYPES.contains(&"double_time"));
        assert!(VALID_ENTRY_TYPES.contains(&"holiday"));
        assert!(VALID_ENTRY_TYPES.contains(&"sick"));
        assert!(VALID_ENTRY_TYPES.contains(&"vacation"));
        assert!(VALID_ENTRY_TYPES.contains(&"break"));
    }
}
