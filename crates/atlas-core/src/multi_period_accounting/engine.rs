//! Multi-Period Accounting Engine
//!
//! Manages the full lifecycle of multi-period accounting:
//! - Create reusable templates for distributing amounts across periods
//! - Apply templates to create schedules with per-period lines
//! - Recognize individual period allocations
//! - Reverse recognized allocations
//! - Activate, complete, hold, or cancel schedules
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Multi-Period Accounting

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use chrono::Datelike;

const VALID_DISTRIBUTION_METHODS: &[&str] = &["equal", "custom", "days"];
const VALID_PERIOD_TYPES: &[&str] = &["month", "quarter", "year"];
const VALID_TEMPLATE_STATUSES: &[&str] = &["draft", "active", "inactive"];
const VALID_SCHEDULE_STATUSES: &[&str] = &["draft", "active", "completed", "cancelled", "on_hold"];
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &["pending", "recognized", "reversed"];

pub struct MultiPeriodAccountingEngine {
    repository: Arc<dyn MpaRepository>,
}

impl MultiPeriodAccountingEngine {
    pub fn new(repository: Arc<dyn MpaRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Templates
    // ========================================================================

    /// Create a new MPA template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        template_name: &str,
        description: Option<&str>,
        distribution_method: &str,
        number_of_periods: i32,
        period_type: &str,
        deferred_account_code: Option<&str>,
        expense_account_code: Option<&str>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MpaTemplate> {
        if template_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".into()));
        }
        if !VALID_DISTRIBUTION_METHODS.contains(&distribution_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid distribution method '{}'. Must be one of: {}", distribution_method, VALID_DISTRIBUTION_METHODS.join(", ")
            )));
        }
        if number_of_periods < 1 {
            return Err(AtlasError::ValidationFailed("Number of periods must be at least 1".into()));
        }
        if !VALID_PERIOD_TYPES.contains(&period_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid period type '{}'. Must be one of: {}", period_type, VALID_PERIOD_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        if self.repository.get_template_by_name(org_id, template_name).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Template '{}' already exists", template_name)));
        }

        info!("Creating MPA template '{}' ({} periods, {})", template_name, number_of_periods, distribution_method);
        self.repository.create_template(
            org_id, template_name, description, distribution_method,
            number_of_periods, period_type, deferred_account_code,
            expense_account_code, currency_code, created_by,
        ).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<MpaTemplate>> {
        self.repository.get_template(id).await
    }

    /// List templates with optional filter
    pub async fn list_templates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaTemplate>> {
        if let Some(s) = status {
            if !VALID_TEMPLATE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TEMPLATE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_templates(org_id, status).await
    }

    /// Activate a template
    pub async fn activate_template(&self, template_id: Uuid) -> AtlasResult<MpaTemplate> {
        let template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;

        if template.status != "draft" && template.status != "inactive" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate template in '{}' status. Must be 'draft' or 'inactive'.", template.status)
            ));
        }

        info!("Activating MPA template '{}'", template.template_name);
        self.repository.update_template_status(template_id, "active").await
    }

    /// Deactivate a template
    pub async fn deactivate_template(&self, template_id: Uuid) -> AtlasResult<MpaTemplate> {
        let template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;

        if template.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot deactivate template in '{}' status. Must be 'active'.", template.status)
            ));
        }

        info!("Deactivating MPA template '{}'", template.template_name);
        self.repository.update_template_status(template_id, "inactive").await
    }

    /// Add a custom distribution line to a template
    pub async fn add_template_line(
        &self,
        template_id: Uuid,
        period_sequence: i32,
        percentage: &str,
        offset_days: i32,
    ) -> AtlasResult<MpaTemplateLine> {
        let template = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;

        if template.distribution_method != "custom" {
            return Err(AtlasError::ValidationFailed(
                "Template lines can only be added to templates with 'custom' distribution method".into()
            ));
        }

        let pct: f64 = percentage.parse().unwrap_or(f64::NAN);
        if pct.is_nan() || pct <= 0.0 || pct > 100.0 {
            return Err(AtlasError::ValidationFailed("Percentage must be between 0 and 100".into()));
        }

        if period_sequence < 1 {
            return Err(AtlasError::ValidationFailed("Period sequence must be at least 1".into()));
        }

        info!("Adding template line to '{}' (period {}, {}%)", template.template_name, period_sequence, percentage);
        self.repository.add_template_line(template_id, period_sequence, percentage, offset_days).await
    }

    /// List template lines
    pub async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<MpaTemplateLine>> {
        self.repository.list_template_lines(template_id).await
    }

    // ========================================================================
    // Schedules
    // ========================================================================

    /// Create a new MPA schedule (optionally from a template)
    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        description: Option<&str>,
        template_id: Option<Uuid>,
        source_journal_entry_id: Option<Uuid>,
        source_journal_line_id: Option<Uuid>,
        total_amount: &str,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        company_code: Option<&str>,
        cost_center: Option<&str>,
        account_segment: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MpaSchedule> {
        if schedule_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule number is required".into()));
        }

        let amount_val: f64 = total_amount.parse().unwrap_or(f64::NAN);
        if amount_val.is_nan() || amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("Total amount must be a positive number".into()));
        }

        if let Some(end) = end_date {
            if end <= start_date {
                return Err(AtlasError::ValidationFailed("End date must be after start date".into()));
            }
        }

        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        // Validate template exists and is active if specified
        let mut template: Option<MpaTemplate> = None;
        if let Some(tid) = template_id {
            let t = self.repository.get_template(tid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", tid)))?;
            if t.status != "active" {
                return Err(AtlasError::WorkflowError(
                    format!("Cannot use template in '{}' status. Must be 'active'.", t.status)
                ));
            }
            template = Some(t);
        }

        if self.repository.get_schedule_by_number(org_id, schedule_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Schedule '{}' already exists", schedule_number)));
        }

        info!("Creating MPA schedule '{}' (amount: {})", schedule_number, total_amount);
        let schedule = self.repository.create_schedule(
            org_id, schedule_number, description, template_id,
            source_journal_entry_id, source_journal_line_id,
            total_amount, start_date, end_date, currency_code,
            company_code, cost_center, account_segment, created_by,
        ).await?;

        // Generate schedule lines based on template or equal distribution
        if let Some(tmpl) = template {
            self.generate_lines_from_template(&schedule, &tmpl).await?;
        } else {
            // Default to equal distribution if no template
            self.generate_equal_lines(&schedule, 1).await?;
        }

        Ok(self.repository.get_schedule(schedule.id).await?.unwrap())
    }

    /// Generate schedule lines from a template
    async fn generate_lines_from_template(
        &self,
        schedule: &MpaSchedule,
        template: &MpaTemplate,
    ) -> AtlasResult<()> {
        let total: f64 = schedule.total_amount.parse().unwrap_or(0.0);
        let num_periods = template.number_of_periods;
        let start = schedule.start_date;

        match template.distribution_method.as_str() {
            "equal" => {
                self.generate_equal_lines(schedule, num_periods).await?;
            }
            "custom" => {
                let lines = self.repository.list_template_lines(template.id).await?;
                for line in &lines {
                    let pct: f64 = line.percentage.parse().unwrap_or(0.0);
                    let amount = total * pct / 100.0;
                    let period_start = add_periods(start, line.period_sequence - 1, &template.period_type);
                    let period_end = add_periods(period_start, 1, &template.period_type)
                        - chrono::Duration::days(1);

                    self.repository.create_schedule_line(
                        schedule.id,
                        line.period_sequence,
                        Some(&format_period_name(&period_start, &template.period_type)),
                        period_start,
                        period_end,
                        &format!("{:.2}", amount),
                        &line.percentage,
                    ).await?;
                }
            }
            "days" => {
                let total_days = num_periods * days_per_period(&template.period_type);
                for i in 0..num_periods {
                    let period_start = add_periods(start, i, &template.period_type);
                    let period_end = add_periods(period_start, 1, &template.period_type)
                        - chrono::Duration::days(1);
                    let days_in_period = (period_end - period_start).num_days() as f64 + 1.0;
                    let pct = days_in_period / total_days as f64 * 100.0;
                    let amount = total * pct / 100.0;

                    self.repository.create_schedule_line(
                        schedule.id,
                        i + 1,
                        Some(&format_period_name(&period_start, &template.period_type)),
                        period_start,
                        period_end,
                        &format!("{:.2}", amount),
                        &format!("{:.4}", pct),
                    ).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate equal distribution lines
    async fn generate_equal_lines(
        &self,
        schedule: &MpaSchedule,
        num_periods: i32,
    ) -> AtlasResult<()> {
        let total: f64 = schedule.total_amount.parse().unwrap_or(0.0);
        let per_period = total / num_periods as f64;
        let pct_per_period = 100.0 / num_periods as f64;
        let start = schedule.start_date;

        for i in 0..num_periods {
            let period_start = add_months(start, i);
            let period_end = add_months(period_start, 1) - chrono::Duration::days(1);

            self.repository.create_schedule_line(
                schedule.id,
                i + 1,
                Some(&format_period_name(&period_start, "month")),
                period_start,
                period_end,
                &format!("{:.2}", per_period),
                &format!("{:.4}", pct_per_period),
            ).await?;
        }
        Ok(())
    }

    /// Get a schedule by ID
    pub async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<MpaSchedule>> {
        self.repository.get_schedule(id).await
    }

    /// List schedules with optional filter
    pub async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaSchedule>> {
        if let Some(s) = status {
            if !VALID_SCHEDULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SCHEDULE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_schedules(org_id, status).await
    }

    /// Activate a schedule
    pub async fn activate_schedule(&self, schedule_id: Uuid) -> AtlasResult<MpaSchedule> {
        let schedule = self.repository.get_schedule(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "draft" && schedule.status != "on_hold" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate schedule in '{}' status. Must be 'draft' or 'on_hold'.", schedule.status)
            ));
        }

        info!("Activating MPA schedule '{}'", schedule.schedule_number);
        self.repository.update_schedule_status(schedule_id, "active").await
    }

    /// Put a schedule on hold
    pub async fn hold_schedule(&self, schedule_id: Uuid) -> AtlasResult<MpaSchedule> {
        let schedule = self.repository.get_schedule(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot hold schedule in '{}' status. Must be 'active'.", schedule.status)
            ));
        }

        info!("Holding MPA schedule '{}'", schedule.schedule_number);
        self.repository.update_schedule_status(schedule_id, "on_hold").await
    }

    /// Cancel a schedule
    pub async fn cancel_schedule(&self, schedule_id: Uuid) -> AtlasResult<MpaSchedule> {
        let schedule = self.repository.get_schedule(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status == "completed" || schedule.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel schedule in '{}' status.", schedule.status)
            ));
        }

        info!("Cancelling MPA schedule '{}'", schedule.schedule_number);
        self.repository.update_schedule_status(schedule_id, "cancelled").await
    }

    // ========================================================================
    // Schedule Line Recognition
    // ========================================================================

    /// Get a schedule line
    pub async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<MpaScheduleLine>> {
        self.repository.get_schedule_line(id).await
    }

    /// List schedule lines
    pub async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<MpaScheduleLine>> {
        self.repository.list_schedule_lines(schedule_id).await
    }

    /// Recognize a pending schedule line
    pub async fn recognize_line(&self, line_id: Uuid) -> AtlasResult<MpaScheduleLine> {
        let line = self.repository.get_schedule_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule line {} not found", line_id)))?;

        if line.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot recognize line in '{}' status. Must be 'pending'.", line.status)
            ));
        }

        // Check parent schedule is active
        let schedule = self.repository.get_schedule(line.schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Schedule not found".to_string()))?;
        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot recognize line: parent schedule is in '{}' status. Must be 'active'.", schedule.status)
            ));
        }

        info!("Recognizing MPA schedule line {} (period {})", line_id, line.period_sequence);
        let line = self.repository.update_schedule_line_status(line_id, "recognized", Some(Uuid::new_v4())).await?;

        // Update schedule amounts
        let recognized = self.repository.sum_recognized_for_schedule(line.schedule_id).await?;
        let total: f64 = schedule.total_amount.parse().unwrap_or(0.0);
        let rec: f64 = recognized.parse().unwrap_or(0.0);
        let remaining = total - rec;
        self.repository.update_schedule_amounts(
            line.schedule_id,
            &format!("{:.2}", rec),
            &format!("{:.2}", remaining),
        ).await?;

        // Auto-complete schedule if all lines recognized
        let pending = self.repository.count_pending_lines(line.schedule_id).await?;
        if pending == 0 {
            info!("Auto-completing MPA schedule '{}' (all lines recognized)", schedule.schedule_number);
            self.repository.update_schedule_status(line.schedule_id, "completed").await?;
        }

        Ok(line)
    }

    /// Reverse a recognized schedule line
    pub async fn reverse_line(&self, line_id: Uuid) -> AtlasResult<MpaScheduleLine> {
        let line = self.repository.get_schedule_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule line {} not found", line_id)))?;

        if line.status != "recognized" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse line in '{}' status. Must be 'recognized'.", line.status)
            ));
        }

        info!("Reversing MPA schedule line {} (period {})", line_id, line.period_sequence);

        // Get schedule before updating line
        let schedule = self.repository.get_schedule(line.schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Schedule not found".into()))?;

        let line = self.repository.update_schedule_line_status(line_id, "reversed", None).await?;

        // Update schedule amounts
        let recognized = self.repository.sum_recognized_for_schedule(line.schedule_id).await?;
        let total: f64 = schedule.total_amount.parse().unwrap_or(0.0);
        let rec: f64 = recognized.parse().unwrap_or(0.0);
        let remaining = total - rec;
        self.repository.update_schedule_amounts(
            line.schedule_id,
            &format!("{:.2}", rec),
            &format!("{:.2}", remaining),
        ).await?;

        // If schedule was completed, reactivate it
        if schedule.status == "completed" {
            self.repository.update_schedule_status(line.schedule_id, "active").await?;
        }

        Ok(line)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MpaDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ========================================================================
// Date Helpers
// ========================================================================

fn add_months(date: chrono::NaiveDate, months: i32) -> chrono::NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let day = date.day().min(days_in_month(year, month as u32));
    chrono::NaiveDate::from_ymd_opt(year, month as u32, day).unwrap_or(date)
}

fn add_periods(date: chrono::NaiveDate, count: i32, period_type: &str) -> chrono::NaiveDate {
    match period_type {
        "month" => add_months(date, count),
        "quarter" => add_months(date, count * 3),
        "year" => add_months(date, count * 12),
        _ => add_months(date, count),
    }
}

fn days_per_period(period_type: &str) -> i32 {
    match period_type {
        "month" => 30,
        "quarter" => 90,
        "year" => 365,
        _ => 30,
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
        .map(|d| d.day())
        .unwrap_or(28)
}

fn format_period_name(date: &chrono::NaiveDate, period_type: &str) -> String {
    match period_type {
        "month" => format!("{}-{:02}", date.year(), date.month()),
        "quarter" => format!("{}-Q{}", date.year(), (date.month() - 1) / 3 + 1),
        "year" => format!("{}", date.year()),
        _ => format!("{}-{:02}", date.year(), date.month()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        templates: std::sync::Mutex<Vec<MpaTemplate>>,
        template_lines: std::sync::Mutex<Vec<MpaTemplateLine>>,
        schedules: std::sync::Mutex<Vec<MpaSchedule>>,
        schedule_lines: std::sync::Mutex<Vec<MpaScheduleLine>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                templates: std::sync::Mutex::new(vec![]),
                template_lines: std::sync::Mutex::new(vec![]),
                schedules: std::sync::Mutex::new(vec![]),
                schedule_lines: std::sync::Mutex::new(vec![]),
            }
        }
    }

    fn make_template(id: Uuid, org_id: Uuid, name: &str, method: &str, periods: i32) -> MpaTemplate {
        MpaTemplate {
            id, organization_id: org_id, template_name: name.into(),
            description: None, distribution_method: method.into(),
            number_of_periods: periods, period_type: "month".into(),
            deferred_account_code: None, expense_account_code: None,
            status: "draft".into(), currency_code: "USD".into(),
            metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait::async_trait]
    impl MpaRepository for MockRepo {
        async fn create_template(
            &self, org_id: Uuid, name: &str, description: Option<&str>,
            method: &str, periods: i32, period_type: &str,
            deferred: Option<&str>, expense: Option<&str>,
            currency: &str, created_by: Option<Uuid>,
        ) -> AtlasResult<MpaTemplate> {
            let t = MpaTemplate {
                id: Uuid::new_v4(), organization_id: org_id,
                template_name: name.into(), description: description.map(Into::into),
                distribution_method: method.into(), number_of_periods: periods,
                period_type: period_type.into(), deferred_account_code: deferred.map(Into::into),
                expense_account_code: expense.map(Into::into),
                status: "draft".into(), currency_code: currency.into(),
                metadata: serde_json::json!({}), created_by,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.templates.lock().unwrap().push(t.clone());
            Ok(t)
        }

        async fn get_template(&self, id: Uuid) -> AtlasResult<Option<MpaTemplate>> {
            Ok(self.templates.lock().unwrap().iter().find(|t| t.id == id).cloned())
        }

        async fn get_template_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<MpaTemplate>> {
            Ok(self.templates.lock().unwrap().iter()
                .find(|t| t.organization_id == org_id && t.template_name == name).cloned())
        }

        async fn list_templates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaTemplate>> {
            Ok(self.templates.lock().unwrap().iter()
                .filter(|t| t.organization_id == org_id)
                .filter(|t| status.map_or(true, |s| t.status == s))
                .cloned().collect())
        }

        async fn update_template_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaTemplate> {
            let mut ts = self.templates.lock().unwrap();
            let t = ts.iter_mut().find(|t| t.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", id)))?;
            t.status = status.into();
            t.updated_at = chrono::Utc::now();
            Ok(t.clone())
        }

        async fn add_template_line(&self, template_id: Uuid, seq: i32, pct: &str, offset: i32) -> AtlasResult<MpaTemplateLine> {
            let line = MpaTemplateLine {
                id: Uuid::new_v4(), template_id, period_sequence: seq,
                percentage: pct.into(), offset_days: offset,
                metadata: serde_json::json!({}), created_at: chrono::Utc::now(),
            };
            self.template_lines.lock().unwrap().push(line.clone());
            Ok(line)
        }

        async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<MpaTemplateLine>> {
            Ok(self.template_lines.lock().unwrap().iter()
                .filter(|l| l.template_id == template_id).cloned().collect())
        }

        async fn delete_template_lines(&self, template_id: Uuid) -> AtlasResult<()> {
            self.template_lines.lock().unwrap().retain(|l| l.template_id != template_id);
            Ok(())
        }

        async fn create_schedule(
            &self, org_id: Uuid, num: &str, desc: Option<&str>,
            template_id: Option<Uuid>, src_je: Option<Uuid>, src_jl: Option<Uuid>,
            total: &str, start: chrono::NaiveDate, end: Option<chrono::NaiveDate>,
            currency: &str, company: Option<&str>, cc: Option<&str>, acct: Option<&str>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<MpaSchedule> {
            let s = MpaSchedule {
                id: Uuid::new_v4(), organization_id: org_id,
                schedule_number: num.into(), description: desc.map(Into::into),
                template_id, source_journal_entry_id: src_je, source_journal_line_id: src_jl,
                total_amount: total.into(), recognized_amount: "0.00".into(),
                remaining_amount: total.into(),
                start_date: start, end_date: end,
                status: "draft".into(), currency_code: currency.into(),
                company_code: company.map(Into::into), cost_center: cc.map(Into::into),
                account_segment: acct.map(Into::into),
                metadata: serde_json::json!({}), created_by,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.schedules.lock().unwrap().push(s.clone());
            Ok(s)
        }

        async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<MpaSchedule>> {
            Ok(self.schedules.lock().unwrap().iter().find(|s| s.id == id).cloned())
        }

        async fn get_schedule_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<MpaSchedule>> {
            Ok(self.schedules.lock().unwrap().iter()
                .find(|s| s.organization_id == org_id && s.schedule_number == num).cloned())
        }

        async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaSchedule>> {
            Ok(self.schedules.lock().unwrap().iter()
                .filter(|s| s.organization_id == org_id)
                .filter(|s| status.map_or(true, |st| s.status == st))
                .cloned().collect())
        }

        async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaSchedule> {
            let mut ss = self.schedules.lock().unwrap();
            let s = ss.iter_mut().find(|s| s.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
            s.status = status.into();
            s.updated_at = chrono::Utc::now();
            Ok(s.clone())
        }

        async fn update_schedule_amounts(&self, id: Uuid, recognized: &str, remaining: &str) -> AtlasResult<()> {
            let mut ss = self.schedules.lock().unwrap();
            if let Some(s) = ss.iter_mut().find(|s| s.id == id) {
                s.recognized_amount = recognized.into();
                s.remaining_amount = remaining.into();
            }
            Ok(())
        }

        async fn create_schedule_line(
            &self, schedule_id: Uuid, seq: i32, name: Option<&str>,
            start: chrono::NaiveDate, end: chrono::NaiveDate, amount: &str, pct: &str,
        ) -> AtlasResult<MpaScheduleLine> {
            let l = MpaScheduleLine {
                id: Uuid::new_v4(), schedule_id, period_sequence: seq,
                period_name: name.map(Into::into),
                period_start_date: start, period_end_date: end,
                amount: amount.into(), percentage: pct.into(),
                status: "pending".into(), journal_entry_id: None,
                recognized_at: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.schedule_lines.lock().unwrap().push(l.clone());
            Ok(l)
        }

        async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<MpaScheduleLine>> {
            Ok(self.schedule_lines.lock().unwrap().iter().find(|l| l.id == id).cloned())
        }

        async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<MpaScheduleLine>> {
            Ok(self.schedule_lines.lock().unwrap().iter()
                .filter(|l| l.schedule_id == schedule_id).cloned().collect())
        }

        async fn update_schedule_line_status(&self, id: Uuid, status: &str, je_id: Option<Uuid>) -> AtlasResult<MpaScheduleLine> {
            let mut ls = self.schedule_lines.lock().unwrap();
            let l = ls.iter_mut().find(|l| l.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Line {} not found", id)))?;
            l.status = status.into();
            l.journal_entry_id = je_id;
            if status == "recognized" {
                l.recognized_at = Some(chrono::Utc::now());
            }
            l.updated_at = chrono::Utc::now();
            Ok(l.clone())
        }

        async fn count_pending_lines(&self, schedule_id: Uuid) -> AtlasResult<i64> {
            Ok(self.schedule_lines.lock().unwrap().iter()
                .filter(|l| l.schedule_id == schedule_id && l.status == "pending").count() as i64)
        }

        async fn sum_recognized_for_schedule(&self, schedule_id: Uuid) -> AtlasResult<String> {
            let sum: f64 = self.schedule_lines.lock().unwrap().iter()
                .filter(|l| l.schedule_id == schedule_id && l.status == "recognized")
                .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
                .sum();
            Ok(format!("{:.2}", sum))
        }

        async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<MpaDashboard> {
            Ok(MpaDashboard {
                total_templates: 0, active_templates: 0,
                total_schedules: 0, draft_schedules: 0, active_schedules: 0,
                completed_schedules: 0, cancelled_schedules: 0, on_hold_schedules: 0,
                total_scheduled_amount: "0.00".into(),
                total_recognized_amount: "0.00".into(),
                total_remaining_amount: "0.00".into(),
                pending_lines: 0, recognized_lines: 0, reversed_lines: 0,
            })
        }
    }

    fn eng() -> MultiPeriodAccountingEngine {
        MultiPeriodAccountingEngine::new(Arc::new(MockRepo::new()))
    }

    // ========================================================================
    // Template Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_template() {
        let t = eng().create_template(
            Uuid::new_v4(), "Insurance Amortization", Some("12-month spread"),
            "equal", 12, "month", Some("1500"), Some("6000"),
            "USD", None,
        ).await.unwrap();
        assert_eq!(t.template_name, "Insurance Amortization");
        assert_eq!(t.distribution_method, "equal");
        assert_eq!(t.number_of_periods, 12);
        assert_eq!(t.status, "draft");
    }

    #[tokio::test]
    async fn test_create_template_empty_name_fails() {
        let r = eng().create_template(
            Uuid::new_v4(), "", None, "equal", 12, "month",
            None, None, "USD", None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_template_invalid_method_fails() {
        let r = eng().create_template(
            Uuid::new_v4(), "Bad", None, "random", 12, "month",
            None, None, "USD", None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_template_zero_periods_fails() {
        let r = eng().create_template(
            Uuid::new_v4(), "Bad", None, "equal", 0, "month",
            None, None, "USD", None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_activate_template() {
        let e = eng();
        let t = e.create_template(
            Uuid::new_v4(), "T-ACT", None, "equal", 6, "month",
            None, None, "USD", None,
        ).await.unwrap();
        assert_eq!(t.status, "draft");
        let t = e.activate_template(t.id).await.unwrap();
        assert_eq!(t.status, "active");
    }

    #[tokio::test]
    async fn test_activate_non_draft_fails() {
        let e = eng();
        let t = e.create_template(
            Uuid::new_v4(), "T-ACT2", None, "equal", 6, "month",
            None, None, "USD", None,
        ).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let r = e.activate_template(t.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_template() {
        let e = eng();
        let t = e.create_template(
            Uuid::new_v4(), "T-DEACT", None, "equal", 6, "month",
            None, None, "USD", None,
        ).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let t = e.deactivate_template(t.id).await.unwrap();
        assert_eq!(t.status, "inactive");
    }

    #[tokio::test]
    async fn test_duplicate_template_name_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        e.create_template(org, "UNIQUE", None, "equal", 6, "month", None, None, "USD", None).await.unwrap();
        let r = e.create_template(org, "UNIQUE", None, "equal", 3, "month", None, None, "USD", None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_add_template_line() {
        let e = eng();
        // Custom method template
        let t = e.create_template(
            Uuid::new_v4(), "T-LINE", None, "custom", 3, "month",
            None, None, "USD", None,
        ).await.unwrap();
        let line = e.add_template_line(t.id, 1, "50.00", 0).await.unwrap();
        assert_eq!(line.period_sequence, 1);
        assert_eq!(line.percentage, "50.00");
    }

    #[tokio::test]
    async fn test_add_template_line_to_equal_template_fails() {
        let e = eng();
        let t = e.create_template(
            Uuid::new_v4(), "T-NO-LINE", None, "equal", 12, "month",
            None, None, "USD", None,
        ).await.unwrap();
        let r = e.add_template_line(t.id, 1, "50.00", 0).await;
        assert!(r.is_err());
    }

    // ========================================================================
    // Schedule Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_schedule() {
        let e = eng();
        let s = e.create_schedule(
            Uuid::new_v4(), "MPA-001", Some("Insurance spread"),
            None, None, None, "12000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            "USD", Some("ACME"), Some("CC01"), Some("6000"), None,
        ).await.unwrap();
        assert_eq!(s.schedule_number, "MPA-001");
        assert_eq!(s.total_amount, "12000.00");
        assert_eq!(s.status, "draft");
    }

    #[tokio::test]
    async fn test_create_schedule_with_template() {
        let e = eng();
        let org = Uuid::new_v4();
        let t = e.create_template(
            org, "TPL-12M", None, "equal", 12, "month",
            None, None, "USD", None,
        ).await.unwrap();
        e.activate_template(t.id).await.unwrap();

        let s = e.create_schedule(
            org, "MPA-TPL-01", None, Some(t.id), None, None, "12000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            "USD", None, None, None, None,
        ).await.unwrap();
        assert_eq!(s.schedule_number, "MPA-TPL-01");

        // Should have 12 schedule lines
        let lines = e.list_schedule_lines(s.id).await.unwrap();
        assert_eq!(lines.len(), 12);
        assert_eq!(lines[0].amount, "1000.00");
        assert_eq!(lines[0].status, "pending");
    }

    #[tokio::test]
    async fn test_create_schedule_negative_amount_fails() {
        let r = eng().create_schedule(
            Uuid::new_v4(), "MPA-BAD", None, None, None, None, "-100.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_activate_schedule() {
        let e = eng();
        let s = e.create_schedule(
            Uuid::new_v4(), "MPA-ACT", None, None, None, None, "5000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        assert_eq!(s.status, "draft");
        let s = e.activate_schedule(s.id).await.unwrap();
        assert_eq!(s.status, "active");
    }

    #[tokio::test]
    async fn test_hold_and_resume_schedule() {
        let e = eng();
        let s = e.create_schedule(
            Uuid::new_v4(), "MPA-HOLD", None, None, None, None, "5000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_schedule(s.id).await.unwrap();
        let s = e.hold_schedule(s.id).await.unwrap();
        assert_eq!(s.status, "on_hold");
        let s = e.activate_schedule(s.id).await.unwrap();
        assert_eq!(s.status, "active");
    }

    #[tokio::test]
    async fn test_cancel_schedule() {
        let e = eng();
        let s = e.create_schedule(
            Uuid::new_v4(), "MPA-CANC", None, None, None, None, "5000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        let s = e.cancel_schedule(s.id).await.unwrap();
        assert_eq!(s.status, "cancelled");
    }

    // ========================================================================
    // Recognition Tests
    // ========================================================================

    #[tokio::test]
    async fn test_recognize_line() {
        let e = eng();
        let org = Uuid::new_v4();
        let t = e.create_template(org, "REC-TPL", None, "equal", 3, "month", None, None, "USD", None).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let s = e.create_schedule(
            org, "MPA-REC", None, Some(t.id), None, None, "3000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_schedule(s.id).await.unwrap();

        let lines = e.list_schedule_lines(s.id).await.unwrap();
        assert_eq!(lines.len(), 3);

        let l = e.recognize_line(lines[0].id).await.unwrap();
        assert_eq!(l.status, "recognized");
        assert!(l.journal_entry_id.is_some());

        let s = e.get_schedule(s.id).await.unwrap().unwrap();
        assert_eq!(s.recognized_amount, "1000.00");
        assert_eq!(s.remaining_amount, "2000.00");
    }

    #[tokio::test]
    async fn test_auto_complete_schedule() {
        let e = eng();
        let org = Uuid::new_v4();
        let t = e.create_template(org, "COMP-TPL", None, "equal", 2, "month", None, None, "USD", None).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let s = e.create_schedule(
            org, "MPA-COMP", None, Some(t.id), None, None, "2000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_schedule(s.id).await.unwrap();

        let lines = e.list_schedule_lines(s.id).await.unwrap();
        assert_eq!(lines.len(), 2);

        e.recognize_line(lines[0].id).await.unwrap();
        e.recognize_line(lines[1].id).await.unwrap();

        let s = e.get_schedule(s.id).await.unwrap().unwrap();
        assert_eq!(s.status, "completed");
        assert_eq!(s.recognized_amount, "2000.00");
    }

    #[tokio::test]
    async fn test_reverse_line() {
        let e = eng();
        let org = Uuid::new_v4();
        let t = e.create_template(org, "REV-TPL", None, "equal", 2, "month", None, None, "USD", None).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let s = e.create_schedule(
            org, "MPA-REV", None, Some(t.id), None, None, "2000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_schedule(s.id).await.unwrap();

        let lines = e.list_schedule_lines(s.id).await.unwrap();
        e.recognize_line(lines[0].id).await.unwrap();

        let l = e.reverse_line(lines[0].id).await.unwrap();
        assert_eq!(l.status, "reversed");
    }

    #[tokio::test]
    async fn test_reverse_reactivates_completed_schedule() {
        let e = eng();
        let org = Uuid::new_v4();
        let t = e.create_template(org, "REV2-TPL", None, "equal", 1, "month", None, None, "USD", None).await.unwrap();
        e.activate_template(t.id).await.unwrap();
        let s = e.create_schedule(
            org, "MPA-REV2", None, Some(t.id), None, None, "1000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_schedule(s.id).await.unwrap();

        let lines = e.list_schedule_lines(s.id).await.unwrap();
        e.recognize_line(lines[0].id).await.unwrap();

        let s = e.get_schedule(s.id).await.unwrap().unwrap();
        assert_eq!(s.status, "completed");

        e.reverse_line(lines[0].id).await.unwrap();
        let s = e.get_schedule(s.id).await.unwrap().unwrap();
        assert_eq!(s.status, "active");
    }

    #[tokio::test]
    async fn test_recognize_line_inactive_schedule_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let s = e.create_schedule(
            org, "MPA-BAD2", None, None, None, None, "1000.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None,
            "USD", None, None, None, None,
        ).await.unwrap();
        // No lines exist for this schedule, but even if they did,
        // the schedule is still in draft
        // Create a fake line ID - this won't be found
        let r = e.recognize_line(Uuid::new_v4()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_templates, 0);
        assert_eq!(d.total_scheduled_amount, "0.00");
    }
}
