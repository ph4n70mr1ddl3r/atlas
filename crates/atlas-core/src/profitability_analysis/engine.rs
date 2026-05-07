//! Profitability Analysis Engine
//!
//! Orchestrates profitability segment management, analysis runs,
//! margin calculations, period-over-period comparisons, templates,
//! and dashboard summaries.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Profitability Analysis

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    ProfitabilityAnalysisRepository,
    ProfitabilitySegment, ProfitabilityRun, ProfitabilityRunLine, ProfitabilityTemplate,
    ProfitabilityDashboard,
    SegmentCreateParams, RunCreateParams, RunLineCreateParams, TemplateCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid segment types
const VALID_SEGMENT_TYPES: &[&str] = &[
    "product", "customer", "channel", "geography",
    "business_unit", "cost_center", "project", "other",
];

// Valid analysis types
const VALID_ANALYSIS_TYPES: &[&str] = &[
    "standard", "detailed", "comparison", "trend", "contribution",
];

// Valid run statuses
const VALID_RUN_STATUSES: &[&str] = &[
    "draft", "calculated", "reviewed", "completed", "cancelled",
];

/// Profitability Analysis Engine
pub struct ProfitabilityAnalysisEngine {
    repo: Arc<dyn ProfitabilityAnalysisRepository>,
}

impl ProfitabilityAnalysisEngine {
    pub fn new(repo: Arc<dyn ProfitabilityAnalysisRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    fn validate_segment_type(segment_type: &str) -> AtlasResult<()> {
        if !VALID_SEGMENT_TYPES.contains(&segment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid segment_type '{}'. Must be one of: {}", segment_type, VALID_SEGMENT_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_analysis_type(analysis_type: &str) -> AtlasResult<()> {
        if !VALID_ANALYSIS_TYPES.contains(&analysis_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid analysis_type '{}'. Must be one of: {}", analysis_type, VALID_ANALYSIS_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_run_status(status: &str) -> AtlasResult<()> {
        if !VALID_RUN_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_RUN_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validate that a status transition is allowed for analysis runs.
    /// draft -> calculated -> reviewed -> completed
    /// draft -> cancelled
    /// calculated -> cancelled
    /// reviewed -> cancelled
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "calculated") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("calculated", "reviewed") => Ok(()),
            ("calculated", "cancelled") => Ok(()),
            ("calculated", "draft") => Ok(()),
            ("reviewed", "completed") => Ok(()),
            ("reviewed", "cancelled") => Ok(()),
            ("reviewed", "calculated") => Ok(()),
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid status transition from '{}' to '{}'. \
                 Valid transitions: draft→calculated, draft→cancelled, \
                 calculated→reviewed, calculated→cancelled, calculated→draft, \
                 reviewed→completed, reviewed→cancelled, reviewed→calculated",
                current, target
            ))),
        }
    }

    // ========================================================================
    // Margin Calculation
    // ========================================================================

    /// Calculate gross margin = revenue - COGS
    pub fn calculate_gross_margin(revenue: f64, cogs: f64) -> f64 {
        revenue - cogs
    }

    /// Calculate gross margin percentage
    pub fn calculate_gross_margin_pct(revenue: f64, gross_margin: f64) -> f64 {
        if revenue.abs() > 0.0 {
            (gross_margin / revenue) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate operating margin = gross margin - operating expenses
    pub fn calculate_operating_margin(gross_margin: f64, operating_expenses: f64) -> f64 {
        gross_margin - operating_expenses
    }

    /// Calculate operating margin percentage
    pub fn calculate_operating_margin_pct(revenue: f64, operating_margin: f64) -> f64 {
        if revenue.abs() > 0.0 {
            (operating_margin / revenue) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate net margin = operating margin + other income - other expense
    pub fn calculate_net_margin(operating_margin: f64, other_income: f64, other_expense: f64) -> f64 {
        operating_margin + other_income - other_expense
    }

    /// Calculate net margin percentage
    pub fn calculate_net_margin_pct(revenue: f64, net_margin: f64) -> f64 {
        if revenue.abs() > 0.0 {
            (net_margin / revenue) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate percentage change between two values
    pub fn calculate_pct_change(prior: f64, current: f64) -> f64 {
        if prior.abs() > 0.0 {
            ((current - prior) / prior.abs()) * 100.0
        } else if current.abs() > 0.0 {
            100.0
        } else {
            0.0
        }
    }

    // ========================================================================
    // Segment CRUD
    // ========================================================================

    /// Create a new profitability segment
    pub async fn create_segment(
        &self,
        org_id: Uuid,
        segment_code: &str,
        segment_name: &str,
        segment_type: &str,
        description: Option<&str>,
        parent_segment_id: Option<Uuid>,
        sort_order: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProfitabilitySegment> {
        info!("Creating profitability segment '{}' ({}) for org {}", segment_code, segment_name, org_id);
        Self::validate_segment_type(segment_type)?;

        if segment_code.is_empty() || segment_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Segment code and name are required".to_string(),
            ));
        }

        let params = SegmentCreateParams {
            org_id,
            segment_code: segment_code.to_string(),
            segment_name: segment_name.to_string(),
            segment_type: segment_type.to_string(),
            description: description.map(|s| s.to_string()),
            parent_segment_id,
            sort_order,
            metadata: None,
            created_by,
        };

        self.repo.create_segment(&params).await
    }

    /// Get a segment by ID
    pub async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<ProfitabilitySegment>> {
        self.repo.get_segment(id).await
    }

    /// Get a segment by code
    pub async fn get_segment_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilitySegment>> {
        self.repo.get_segment_by_code(org_id, code).await
    }

    /// List segments with optional filters
    pub async fn list_segments(
        &self,
        org_id: Uuid,
        segment_type: Option<&str>,
        is_active: Option<bool>,
    ) -> AtlasResult<Vec<ProfitabilitySegment>> {
        self.repo.list_segments(org_id, segment_type, is_active).await
    }

    /// Delete a segment
    pub async fn delete_segment(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting profitability segment '{}' for org {}", code, org_id);
        self.repo.delete_segment(org_id, code).await
    }

    // ========================================================================
    // Analysis Run CRUD
    // ========================================================================

    /// Create a new analysis run
    pub async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        run_name: &str,
        analysis_type: &str,
        period_from: chrono::NaiveDate,
        period_to: chrono::NaiveDate,
        currency_code: &str,
        comparison_run_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProfitabilityRun> {
        info!("Creating profitability run '{}' for org {}", run_number, org_id);
        Self::validate_analysis_type(analysis_type)?;

        if run_number.is_empty() || run_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Run number and name are required".to_string(),
            ));
        }
        if period_from >= period_to {
            return Err(AtlasError::ValidationFailed(
                "Period from must be before period to".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let params = RunCreateParams {
            org_id,
            run_number: run_number.to_string(),
            run_name: run_name.to_string(),
            analysis_type: analysis_type.to_string(),
            period_from,
            period_to,
            currency_code: currency_code.to_string(),
            comparison_run_id,
            notes: notes.map(|s| s.to_string()),
            created_by,
        };

        self.repo.create_run(&params).await
    }

    /// Get a run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityRun>> {
        self.repo.get_run(id).await
    }

    /// Get a run by number
    pub async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<ProfitabilityRun>> {
        self.repo.get_run_by_number(org_id, run_number).await
    }

    /// List runs with optional status filter
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProfitabilityRun>> {
        self.repo.list_runs(org_id, status).await
    }

    /// Transition run status
    pub async fn transition_run(&self, id: Uuid, new_status: &str) -> AtlasResult<ProfitabilityRun> {
        info!("Transitioning profitability run {} to '{}'", id, new_status);
        Self::validate_run_status(new_status)?;

        let current = self.repo.get_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))?;

        Self::validate_status_transition(&current.status, new_status)?;

        self.repo.update_run_status(id, new_status).await
    }

    /// Delete a draft run
    pub async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()> {
        info!("Deleting profitability run '{}' for org {}", run_number, org_id);
        let run = self.repo.get_run_by_number(org_id, run_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))?;

        if run.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Only runs in 'draft' status can be deleted".to_string(),
            ));
        }

        self.repo.delete_run(org_id, run_number).await
    }

    // ========================================================================
    // Run Lines
    // ========================================================================

    /// Add a line to a run with automatic margin calculation
    pub async fn add_run_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        segment_id: Option<Uuid>,
        segment_code: Option<&str>,
        segment_name: Option<&str>,
        segment_type: Option<&str>,
        line_number: i32,
        revenue: f64,
        cost_of_goods_sold: f64,
        operating_expenses: f64,
        other_income: f64,
        other_expense: f64,
    ) -> AtlasResult<ProfitabilityRunLine> {
        info!("Adding line {} to profitability run {}", line_number, run_id);

        // Verify run exists and is in editable status
        let run = self.repo.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))?;

        if run.status != "draft" && run.status != "calculated" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be added to runs in 'draft' or 'calculated' status".to_string(),
            ));
        }

        let params = RunLineCreateParams {
            org_id,
            run_id,
            segment_id,
            segment_code: segment_code.map(|s| s.to_string()),
            segment_name: segment_name.map(|s| s.to_string()),
            segment_type: segment_type.map(|s| s.to_string()),
            line_number,
            revenue,
            cost_of_goods_sold,
            operating_expenses,
            other_income,
            other_expense,
        };

        let line = self.repo.create_run_line(&params).await?;

        // Calculate margins
        let gross_margin = Self::calculate_gross_margin(revenue, cost_of_goods_sold);
        let gross_margin_pct = Self::calculate_gross_margin_pct(revenue, gross_margin);
        let operating_margin = Self::calculate_operating_margin(gross_margin, operating_expenses);
        let operating_margin_pct = Self::calculate_operating_margin_pct(revenue, operating_margin);
        let net_margin = Self::calculate_net_margin(operating_margin, other_income, other_expense);
        let net_margin_pct = Self::calculate_net_margin_pct(revenue, net_margin);

        self.repo.update_run_line_margins(
            line.id,
            gross_margin, gross_margin_pct,
            operating_margin, operating_margin_pct,
            net_margin, net_margin_pct,
            0.0, // will be recalculated
            0.0, // will be recalculated
        ).await?;

        // Recalculate run totals
        self.recalculate_run_totals(run_id).await?;

        // Re-fetch with updated margins
        self.repo.get_run_line(line.id).await.map(|l| l.unwrap())
    }

    /// List lines for a run
    pub async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<ProfitabilityRunLine>> {
        self.repo.list_run_lines(run_id).await
    }

    /// Remove a line from a run
    pub async fn remove_run_line(&self, run_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        info!("Removing line {} from profitability run {}", line_id, run_id);

        let run = self.repo.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))?;

        if run.status != "draft" && run.status != "calculated" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be removed from runs in 'draft' or 'calculated' status".to_string(),
            ));
        }

        self.repo.delete_run_line(run_id, line_id).await?;
        self.recalculate_run_totals(run_id).await?;
        Ok(())
    }

    /// Recalculate run totals from lines
    async fn recalculate_run_totals(&self, run_id: Uuid) -> AtlasResult<()> {
        let lines = self.repo.list_run_lines(run_id).await?;

        let total_revenue: f64 = lines.iter().map(|l| l.revenue).sum();
        let total_cogs: f64 = lines.iter().map(|l| l.cost_of_goods_sold).sum();
        let total_gross_margin = Self::calculate_gross_margin(total_revenue, total_cogs);
        let total_opex: f64 = lines.iter().map(|l| l.operating_expenses).sum();
        let total_operating_margin = Self::calculate_operating_margin(total_gross_margin, total_opex);
        let total_other_income: f64 = lines.iter().map(|l| l.other_income).sum();
        let total_other_expense: f64 = lines.iter().map(|l| l.other_expense).sum();
        let total_net_margin = Self::calculate_net_margin(total_operating_margin, total_other_income, total_other_expense);

        let gross_margin_pct = Self::calculate_gross_margin_pct(total_revenue, total_gross_margin);
        let operating_margin_pct = Self::calculate_operating_margin_pct(total_revenue, total_operating_margin);
        let net_margin_pct = Self::calculate_net_margin_pct(total_revenue, total_net_margin);

        self.repo.update_run_totals(
            run_id,
            total_revenue, total_cogs, total_gross_margin,
            total_opex, total_operating_margin, total_net_margin,
            gross_margin_pct, operating_margin_pct, net_margin_pct,
            lines.len() as i32,
        ).await?;

        // Update contribution percentages for each line
        for line in &lines {
            let rev_contrib = if total_revenue.abs() > 0.0 {
                (line.revenue / total_revenue) * 100.0
            } else {
                0.0
            };
            let margin_contrib = if total_net_margin.abs() > 0.0 {
                (line.net_margin / total_net_margin) * 100.0
            } else {
                0.0
            };

            self.repo.update_run_line_margins(
                line.id,
                line.gross_margin, line.gross_margin_pct,
                line.operating_margin, line.operating_margin_pct,
                line.net_margin, line.net_margin_pct,
                rev_contrib, margin_contrib,
            ).await?;
        }

        Ok(())
    }

    // ========================================================================
    // Templates
    // ========================================================================

    /// Create a profitability analysis template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        template_code: &str,
        template_name: &str,
        description: Option<&str>,
        segment_type: &str,
        includes_cogs: Option<bool>,
        includes_operating: Option<bool>,
        includes_other: Option<bool>,
        auto_calculate: Option<bool>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProfitabilityTemplate> {
        info!("Creating profitability template '{}' for org {}", template_code, org_id);
        Self::validate_segment_type(segment_type)?;

        if template_code.is_empty() || template_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }

        let params = TemplateCreateParams {
            org_id,
            template_code: template_code.to_string(),
            template_name: template_name.to_string(),
            description: description.map(|s| s.to_string()),
            segment_type: segment_type.to_string(),
            includes_cogs,
            includes_operating,
            includes_other,
            auto_calculate,
            created_by,
        };

        self.repo.create_template(&params).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityTemplate>> {
        self.repo.get_template(id).await
    }

    /// Get a template by code
    pub async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilityTemplate>> {
        self.repo.get_template_by_code(org_id, code).await
    }

    /// List templates
    pub async fn list_templates(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<ProfitabilityTemplate>> {
        self.repo.list_templates(org_id, is_active).await
    }

    /// Delete a template
    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting profitability template '{}' for org {}", code, org_id);
        self.repo.delete_template(org_id, code).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get profitability dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProfitabilityDashboard> {
        self.repo.get_dashboard(org_id).await
    }

    // ========================================================================
    // Exported validation for handler use
    // ========================================================================

    pub fn valid_segment_types() -> &'static [&'static str] { VALID_SEGMENT_TYPES }
    pub fn valid_analysis_types() -> &'static [&'static str] { VALID_ANALYSIS_TYPES }
    pub fn valid_run_statuses() -> &'static [&'static str] { VALID_RUN_STATUSES }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gross_margin() {
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin(1000.0, 600.0) - 400.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin(500.0, 500.0)).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin(200.0, 300.0) - (-100.0)).abs() < 0.01);
    }

    #[test]
    fn test_gross_margin_pct() {
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin_pct(1000.0, 400.0) - 40.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin_pct(1000.0, -100.0) - (-10.0)).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_gross_margin_pct(0.0, 100.0)).abs() < 0.01);
    }

    #[test]
    fn test_operating_margin() {
        assert!((ProfitabilityAnalysisEngine::calculate_operating_margin(400.0, 150.0) - 250.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_operating_margin(100.0, 200.0) - (-100.0)).abs() < 0.01);
    }

    #[test]
    fn test_operating_margin_pct() {
        assert!((ProfitabilityAnalysisEngine::calculate_operating_margin_pct(1000.0, 250.0) - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_net_margin() {
        assert!((ProfitabilityAnalysisEngine::calculate_net_margin(250.0, 50.0, 25.0) - 275.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_net_margin(100.0, 0.0, 50.0) - 50.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_net_margin(200.0, 30.0, 250.0) - (-20.0)).abs() < 0.01);
    }

    #[test]
    fn test_net_margin_pct() {
        assert!((ProfitabilityAnalysisEngine::calculate_net_margin_pct(1000.0, 275.0) - 27.5).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_net_margin_pct(0.0, 100.0)).abs() < 0.01);
    }

    #[test]
    fn test_pct_change() {
        assert!((ProfitabilityAnalysisEngine::calculate_pct_change(100.0, 120.0) - 20.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_pct_change(100.0, 80.0) - (-20.0)).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_pct_change(0.0, 50.0) - 100.0).abs() < 0.01);
        assert!((ProfitabilityAnalysisEngine::calculate_pct_change(0.0, 0.0)).abs() < 0.01);
    }

    #[test]
    fn test_valid_segment_types() {
        assert!(ProfitabilityAnalysisEngine::validate_segment_type("product").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_segment_type("customer").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_segment_type("channel").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_segment_type("geography").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_segment_type("invalid").is_err());
    }

    #[test]
    fn test_valid_analysis_types() {
        assert!(ProfitabilityAnalysisEngine::validate_analysis_type("standard").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_analysis_type("comparison").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_analysis_type("invalid").is_err());
    }

    #[test]
    fn test_valid_status_transitions() {
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("draft", "calculated").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("draft", "cancelled").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("calculated", "reviewed").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("calculated", "cancelled").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("calculated", "draft").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("reviewed", "completed").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("reviewed", "cancelled").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("reviewed", "calculated").is_ok());
    }

    #[test]
    fn test_invalid_status_transitions() {
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("draft", "completed").is_err());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("draft", "reviewed").is_err());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("completed", "draft").is_err());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("completed", "calculated").is_err());
        assert!(ProfitabilityAnalysisEngine::validate_status_transition("cancelled", "draft").is_err());
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(ProfitabilityAnalysisEngine::validate_run_status("draft").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_run_status("completed").is_ok());
        assert!(ProfitabilityAnalysisEngine::validate_run_status("invalid").is_err());
    }

    #[test]
    fn test_full_margin_chain() {
        // Revenue: 10000, COGS: 6000, OpEx: 1500, Other Income: 200, Other Expense: 100
        let revenue = 10000.0;
        let cogs = 6000.0;
        let opex = 1500.0;
        let other_income = 200.0;
        let other_expense = 100.0;

        let gross = ProfitabilityAnalysisEngine::calculate_gross_margin(revenue, cogs);
        assert!((gross - 4000.0).abs() < 0.01);

        let gross_pct = ProfitabilityAnalysisEngine::calculate_gross_margin_pct(revenue, gross);
        assert!((gross_pct - 40.0).abs() < 0.01);

        let operating = ProfitabilityAnalysisEngine::calculate_operating_margin(gross, opex);
        assert!((operating - 2500.0).abs() < 0.01);

        let operating_pct = ProfitabilityAnalysisEngine::calculate_operating_margin_pct(revenue, operating);
        assert!((operating_pct - 25.0).abs() < 0.01);

        let net = ProfitabilityAnalysisEngine::calculate_net_margin(operating, other_income, other_expense);
        assert!((net - 2600.0).abs() < 0.01);

        let net_pct = ProfitabilityAnalysisEngine::calculate_net_margin_pct(revenue, net);
        assert!((net_pct - 26.0).abs() < 0.01);
    }
}
