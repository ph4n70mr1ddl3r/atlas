//! Recurring Journal Engine
//!
//! Manages recurring journal schedule lifecycle (create, activate, deactivate),
//! template line management, journal generation with support for three types
//! (standard, skeleton, incremental), and generation history.
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Journals > Recurring Journals

use atlas_shared::{
    RecurringJournalSchedule, RecurringJournalScheduleLine,
    RecurringJournalGeneration, RecurringJournalGenerationLine,
    RecurringJournalDashboardSummary,
    AtlasError, AtlasResult,
};
use super::RecurringJournalRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Intermediate line data produced during journal generation.
///
/// Replaces a complex 11-tuple to keep clippy happy and improve readability.
pub(crate) struct GenerationLineData {
    pub schedule_line_id: Uuid,
    #[allow(dead_code)] // reserved for future use
    pub line_number: i32,
    pub line_type: String,
    pub account_code: String,
    pub account_name: String,
    pub amount: String,
    pub description: String,
    pub tax_code: String,
    pub cost_center: String,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

/// Valid recurrence types
const VALID_RECURRENCE_TYPES: &[&str] = &[
    "daily", "weekly", "monthly", "quarterly", "semi_annual", "annual",
];

/// Valid journal types
const VALID_JOURNAL_TYPES: &[&str] = &["standard", "skeleton", "incremental"];

/// Valid schedule statuses
const VALID_SCHEDULE_STATUSES: &[&str] = &["draft", "active", "inactive"];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &["debit", "credit"];

/// Number of months per recurrence period
#[allow(dead_code)]
fn months_per_recurrence(recurrence: &str) -> i32 {
    match recurrence {
        "daily" => 0,
        "weekly" => 0,
        "monthly" => 1,
        "quarterly" => 3,
        "semi_annual" => 6,
        "annual" => 12,
        _ => 1,
    }
}

/// Calculate the next generation date based on recurrence
fn calculate_next_date(current: chrono::NaiveDate, recurrence: &str) -> chrono::NaiveDate {
    match recurrence {
        "daily" => current + chrono::Duration::days(1),
        "weekly" => current + chrono::Duration::weeks(1),
        "monthly" => current.checked_add_months(chrono::Months::new(1)).unwrap_or(current),
        "quarterly" => current.checked_add_months(chrono::Months::new(3)).unwrap_or(current),
        "semi_annual" => current.checked_add_months(chrono::Months::new(6)).unwrap_or(current),
        "annual" => current.checked_add_months(chrono::Months::new(12)).unwrap_or(current),
        _ => current.checked_add_months(chrono::Months::new(1)).unwrap_or(current),
    }
}

/// Recurring Journal engine
pub struct RecurringJournalEngine {
    repository: Arc<dyn RecurringJournalRepository>,
}

impl RecurringJournalEngine {
    pub fn new(repository: Arc<dyn RecurringJournalRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Schedule Management
    // ========================================================================

    /// Create a new recurring journal schedule in draft status
    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        recurrence_type: &str,
        journal_type: &str,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        incremental_percent: Option<&str>,
        auto_post: bool,
        reversal_method: Option<&str>,
        ledger_id: Option<Uuid>,
        journal_category: Option<&str>,
        reference_template: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalSchedule> {
        if schedule_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule name is required".to_string()));
        }
        if !VALID_RECURRENCE_TYPES.contains(&recurrence_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recurrence type '{}'. Must be one of: {}",
                recurrence_type,
                VALID_RECURRENCE_TYPES.join(", ")
            )));
        }
        if !VALID_JOURNAL_TYPES.contains(&journal_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid journal type '{}'. Must be one of: {}",
                journal_type,
                VALID_JOURNAL_TYPES.join(", ")
            )));
        }
        if journal_type == "incremental" {
            if let Some(pct) = incremental_percent {
                let val: f64 = pct.parse().map_err(|_| AtlasError::ValidationFailed(
                    "Incremental percent must be a valid number".to_string(),
                ))?;
                if !(-100.0..=1000.0).contains(&val) {
                    return Err(AtlasError::ValidationFailed(
                        "Incremental percent must be between -100 and 1000".to_string(),
                    ));
                }
            } else {
                return Err(AtlasError::ValidationFailed(
                    "Incremental percent is required for incremental journal type".to_string(),
                ));
            }
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        // Calculate the first next generation date
        let next_gen = effective_from.or_else(|| Some(chrono::Utc::now().date_naive()));

        info!(
            "Creating recurring journal schedule {} ({}) for org {}",
            schedule_number, name, org_id
        );

        self.repository
            .create_schedule(
                org_id,
                schedule_number,
                name,
                description,
                recurrence_type,
                journal_type,
                currency_code,
                effective_from,
                effective_to,
                next_gen,
                incremental_percent,
                auto_post,
                reversal_method,
                ledger_id,
                journal_category,
                reference_template,
                created_by,
            )
            .await
    }

    /// Get a schedule by number
    pub async fn get_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<RecurringJournalSchedule>> {
        self.repository.get_schedule(org_id, schedule_number).await
    }

    /// Get a schedule by ID
    pub async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalSchedule>> {
        self.repository.get_schedule_by_id(id).await
    }

    /// List schedules with optional status filter
    pub async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RecurringJournalSchedule>> {
        if let Some(s) = status {
            if !VALID_SCHEDULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_SCHEDULE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_schedules(org_id, status).await
    }

    /// Activate a draft schedule
    pub async fn activate_schedule(&self, schedule_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<RecurringJournalSchedule> {
        let schedule = self
            .repository
            .get_schedule_by_id(schedule_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate schedule in '{}' status. Must be 'draft'.",
                schedule.status
            )));
        }

        // Validate schedule has lines
        let lines = self.repository.list_schedule_lines(schedule_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot activate schedule without template lines. Add at least one line.".to_string(),
            ));
        }

        // Validate balanced (total debits == total credits) for standard and incremental types
        if schedule.journal_type != "skeleton" {
            let total_debits: f64 = lines.iter()
                .filter(|l| l.line_type == "debit")
                .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
                .sum();
            let total_credits: f64 = lines.iter()
                .filter(|l| l.line_type == "credit")
                .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
                .sum();

            if (total_debits - total_credits).abs() > f64::EPSILON {
                return Err(AtlasError::ValidationFailed(format!(
                    "Schedule lines must be balanced. Total debits ({:.2}) != total credits ({:.2})",
                    total_debits, total_credits
                )));
            }
        }

        info!("Activated recurring journal schedule {}", schedule.schedule_number);
        self.repository
            .update_schedule_status(schedule_id, "active", approved_by)
            .await
    }

    /// Deactivate an active schedule
    pub async fn deactivate_schedule(&self, schedule_id: Uuid) -> AtlasResult<RecurringJournalSchedule> {
        let schedule = self
            .repository
            .get_schedule_by_id(schedule_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deactivate schedule in '{}' status. Must be 'active'.",
                schedule.status
            )));
        }

        info!("Deactivated recurring journal schedule {}", schedule.schedule_number);
        self.repository
            .update_schedule_status(schedule_id, "inactive", None)
            .await
    }

    /// Delete a draft schedule
    pub async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        let schedule = self
            .repository
            .get_schedule(org_id, schedule_number)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_number)))?;

        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete schedule that is not in 'draft' status".to_string(),
            ));
        }

        info!("Deleted recurring journal schedule {}", schedule_number);
        self.repository.delete_schedule(org_id, schedule_number).await
    }

    // ========================================================================
    // Schedule Lines
    // ========================================================================

    /// Add a template line to a schedule
    pub async fn add_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        line_type: &str,
        account_code: &str,
        account_name: Option<&str>,
        description: Option<&str>,
        amount: &str,
        currency_code: &str,
        tax_code: Option<&str>,
        cost_center: Option<&str>,
        department_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalScheduleLine> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}",
                line_type,
                VALID_LINE_TYPES.join(", ")
            )));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".to_string()));
        }

        let schedule = self
            .repository
            .get_schedule_by_id(schedule_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to a schedule that is not in 'draft' status".to_string(),
            ));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amt < 0.0 {
            return Err(AtlasError::ValidationFailed("Amount cannot be negative".to_string()));
        }

        // Get next line number
        let existing = self.repository.list_schedule_lines(schedule_id).await?;
        let line_number = existing.len() as i32 + 1;

        info!(
            "Adding line {} ({}) to schedule {}",
            line_number, account_code, schedule.schedule_number
        );

        self.repository
            .create_schedule_line(
                org_id,
                schedule_id,
                line_number,
                line_type,
                account_code,
                account_name,
                description,
                amount,
                currency_code,
                tax_code,
                cost_center,
                department_id,
                project_id,
            )
            .await
    }

    /// List template lines for a schedule
    pub async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalScheduleLine>> {
        self.repository.list_schedule_lines(schedule_id).await
    }

    /// Delete a template line
    pub async fn delete_schedule_line(&self, line_id: Uuid) -> AtlasResult<()> {
        self.repository.delete_schedule_line(line_id).await
    }

    // ========================================================================
    // Generation
    // ========================================================================

    /// Generate a journal from a recurring schedule
    pub async fn generate_journal(
        &self,
        schedule_id: Uuid,
        generation_date: chrono::NaiveDate,
        override_amounts: Option<Vec<(i32, String)>>, // For skeleton type: (line_number, amount)
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalGeneration> {
        let schedule = self
            .repository
            .get_schedule_by_id(schedule_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot generate from schedule in '{}' status. Must be 'active'.",
                schedule.status
            )));
        }

        // Check effective date range
        if let Some(from) = schedule.effective_from {
            if generation_date < from {
                return Err(AtlasError::ValidationFailed(format!(
                    "Generation date {} is before effective from date {}",
                    generation_date, from
                )));
            }
        }
        if let Some(to) = schedule.effective_to {
            if generation_date > to {
                return Err(AtlasError::ValidationFailed(format!(
                    "Generation date {} is after effective to date {}",
                    generation_date, to
                )));
            }
        }

        let lines = self.repository.list_schedule_lines(schedule_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot generate from schedule without template lines".to_string(),
            ));
        }

        // Calculate line amounts based on journal type
        let gen_lines = self.calculate_generation_lines(
            &schedule, &lines, override_amounts, generation_date,
        )?;

        // Get next generation number
        let gen_number = self.repository.get_latest_generation_number(schedule_id).await? + 1;

        // Calculate totals
        let total_debit: f64 = gen_lines.iter()
            .filter(|l| l.line_type == "debit")
            .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = gen_lines.iter()
            .filter(|l| l.line_type == "credit")
            .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let period_name = generation_date.format("%Y-%m").to_string();

        // Create generation record
        let generation = self.repository.create_generation(
            schedule.organization_id,
            schedule_id,
            gen_number,
            generation_date,
            Some(&period_name),
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            gen_lines.len() as i32,
            generated_by,
        ).await?;

        // Create generation lines
        for (idx, line) in gen_lines.iter().enumerate() {
            self.repository.create_generation_line(
                schedule.organization_id,
                generation.id,
                if line.schedule_line_id != Uuid::nil() { Some(line.schedule_line_id) } else { None },
                idx as i32 + 1,
                &line.line_type,
                &line.account_code,
                if line.account_name.is_empty() { None } else { Some(&line.account_name) },
                if line.description.is_empty() { None } else { Some(&line.description) },
                &line.amount,
                &schedule.currency_code,
                if line.tax_code.is_empty() { None } else { Some(&line.tax_code) },
                if line.cost_center.is_empty() { None } else { Some(&line.cost_center) },
                line.department_id,
                line.project_id,
            ).await?;
        }

        // Update schedule generation info
        let next_gen = calculate_next_date(generation_date, &schedule.recurrence_type);
        self.repository.update_schedule_generation_info(
            schedule_id,
            generation_date,
            Some(next_gen),
            schedule.total_generations + 1,
        ).await?;

        info!(
            "Generated recurring journal #{} for schedule {} (debit: {:.2}, credit: {:.2})",
            gen_number, schedule.schedule_number, total_debit, total_credit
        );

        Ok(generation)
    }

    /// Get a generation by ID
    pub async fn get_generation(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalGeneration>> {
        self.repository.get_generation(id).await
    }

    /// List generations for a schedule
    pub async fn list_generations(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalGeneration>> {
        self.repository.list_generations(schedule_id).await
    }

    /// Post a generated journal
    pub async fn post_generation(&self, generation_id: Uuid) -> AtlasResult<RecurringJournalGeneration> {
        let gen = self
            .repository
            .get_generation(generation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Generation {} not found", generation_id)))?;

        if gen.status != "generated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post generation in '{}' status. Must be 'generated'.",
                gen.status
            )));
        }

        info!("Posted recurring journal generation #{}", gen.generation_number);
        self.repository
            .update_generation_status(generation_id, "posted", Some(chrono::Utc::now()), None, None)
            .await
    }

    /// Reverse a posted generation
    pub async fn reverse_generation(&self, generation_id: Uuid) -> AtlasResult<RecurringJournalGeneration> {
        let gen = self
            .repository
            .get_generation(generation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Generation {} not found", generation_id)))?;

        if gen.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse generation in '{}' status. Must be 'posted'.",
                gen.status
            )));
        }

        info!("Reversed recurring journal generation #{}", gen.generation_number);
        self.repository
            .update_generation_status(generation_id, "reversed", None, Some(chrono::Utc::now()), Some(Uuid::new_v4()))
            .await
    }

    /// Cancel a generated (unposted) journal
    pub async fn cancel_generation(&self, generation_id: Uuid) -> AtlasResult<RecurringJournalGeneration> {
        let gen = self
            .repository
            .get_generation(generation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Generation {} not found", generation_id)))?;

        if gen.status != "generated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel generation in '{}' status. Must be 'generated'.",
                gen.status
            )));
        }

        info!("Cancelled recurring journal generation #{}", gen.generation_number);
        self.repository
            .update_generation_status(generation_id, "cancelled", None, None, None)
            .await
    }

    /// List generation lines
    pub async fn list_generation_lines(&self, generation_id: Uuid) -> AtlasResult<Vec<RecurringJournalGenerationLine>> {
        self.repository.list_generation_lines(generation_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get recurring journal dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<RecurringJournalDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Calculate generation lines based on journal type
    fn calculate_generation_lines(
        &self,
        schedule: &RecurringJournalSchedule,
        lines: &[RecurringJournalScheduleLine],
        override_amounts: Option<Vec<(i32, String)>>,
        _generation_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<GenerationLineData>> {
        let gen_number = schedule.total_generations + 1;

        let result: Vec<GenerationLineData> = lines
            .iter()
            .map(|line| {
                let amount = match schedule.journal_type.as_str() {
                    "standard" => line.amount.clone(),
                    "skeleton" => {
                        // For skeleton, look for override amount by line number
                        if let Some(ref overrides) = override_amounts {
                            overrides.iter()
                                .find(|(num, _)| *num == line.line_number)
                                .map(|(_, amt)| amt.clone())
                                .unwrap_or_else(|| "0.00".to_string())
                        } else {
                            "0.00".to_string()
                        }
                    }
                    "incremental" => {
                        // For incremental, increase amount by incremental_percent each generation
                        let base: f64 = line.amount.parse().unwrap_or(0.0);
                        let pct: f64 = schedule.incremental_percent
                            .as_ref()
                            .and_then(|p| p.parse().ok())
                            .unwrap_or(0.0);
                        let factor = 1.0 + (pct / 100.0);
                        let incremental_amount = base * factor.powi(gen_number - 1);
                        format!("{:.2}", incremental_amount)
                    }
                    _ => line.amount.clone(),
                };

                GenerationLineData {
                    schedule_line_id: line.id,
                    line_number: line.line_number,
                    line_type: line.line_type.clone(),
                    account_code: line.account_code.clone(),
                    account_name: line.account_name.clone().unwrap_or_default(),
                    amount,
                    description: line.description.clone().unwrap_or_default(),
                    tax_code: line.tax_code.clone().unwrap_or_default(),
                    cost_center: line.cost_center.clone().unwrap_or_default(),
                    department_id: line.department_id,
                    project_id: line.project_id,
                }
            })
            .collect();

        // Validate balanced for non-skeleton types
        if schedule.journal_type != "skeleton" {
            let total_debits: f64 = result.iter()
                .filter(|l| l.line_type == "debit")
                .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
                .sum();
            let total_credits: f64 = result.iter()
                .filter(|l| l.line_type == "credit")
                .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
                .sum();

            if (total_debits - total_credits).abs() > f64::EPSILON {
                return Err(AtlasError::ValidationFailed(format!(
                    "Generated lines are not balanced: debits={:.2}, credits={:.2}",
                    total_debits, total_credits
                )));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_recurrence_types() {
        assert!(VALID_RECURRENCE_TYPES.contains(&"daily"));
        assert!(VALID_RECURRENCE_TYPES.contains(&"weekly"));
        assert!(VALID_RECURRENCE_TYPES.contains(&"monthly"));
        assert!(VALID_RECURRENCE_TYPES.contains(&"quarterly"));
        assert!(VALID_RECURRENCE_TYPES.contains(&"semi_annual"));
        assert!(VALID_RECURRENCE_TYPES.contains(&"annual"));
    }

    #[test]
    fn test_valid_journal_types() {
        assert!(VALID_JOURNAL_TYPES.contains(&"standard"));
        assert!(VALID_JOURNAL_TYPES.contains(&"skeleton"));
        assert!(VALID_JOURNAL_TYPES.contains(&"incremental"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"debit"));
        assert!(VALID_LINE_TYPES.contains(&"credit"));
    }

    #[test]
    fn test_months_per_recurrence() {
        assert_eq!(months_per_recurrence("daily"), 0);
        assert_eq!(months_per_recurrence("weekly"), 0);
        assert_eq!(months_per_recurrence("monthly"), 1);
        assert_eq!(months_per_recurrence("quarterly"), 3);
        assert_eq!(months_per_recurrence("semi_annual"), 6);
        assert_eq!(months_per_recurrence("annual"), 12);
    }

    #[test]
    fn test_calculate_next_date_monthly() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = calculate_next_date(date, "monthly");
        assert_eq!(next, chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
    }

    #[test]
    fn test_calculate_next_date_quarterly() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let next = calculate_next_date(date, "quarterly");
        assert_eq!(next, chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
    }

    #[test]
    fn test_calculate_next_date_annual() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let next = calculate_next_date(date, "annual");
        assert_eq!(next, chrono::NaiveDate::from_ymd_opt(2025, 6, 1).unwrap());
    }

    #[test]
    fn test_calculate_next_date_weekly() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let next = calculate_next_date(date, "weekly");
        assert_eq!(next, chrono::NaiveDate::from_ymd_opt(2024, 1, 8).unwrap());
    }

    #[test]
    fn test_calculate_next_date_daily() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let next = calculate_next_date(date, "daily");
        assert_eq!(next, chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn test_incremental_calculation() {
        // Base amount 1000, 10% increase, generation #2 should be 1100
        let base: f64 = 1000.0;
        let pct: f64 = 10.0;
        let gen_number = 2;
        let factor = 1.0 + (pct / 100.0);
        let amount = base * factor.powi(gen_number - 1);
        assert!((amount - 1100.0).abs() < 0.01);

        // Generation #3 should be 1210
        let gen_number = 3;
        let amount = base * factor.powi(gen_number - 1);
        assert!((amount - 1210.0).abs() < 0.01);
    }

    #[test]
    fn test_valid_schedule_statuses() {
        assert!(VALID_SCHEDULE_STATUSES.contains(&"draft"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"active"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"inactive"));
    }
}
