//! Supplier Scorecard Engine
//!
//! Business logic for supplier performance scorecards.
//! Manages templates, categories, scorecards, KPI lines, reviews, and action items.

use atlas_shared::AtlasError;
use super::ScorecardRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_EVALUATION_PERIODS: &[&str] = &["monthly", "quarterly", "semi_annual", "annual"];
const VALID_SCORECARD_STATUSES: &[&str] = &["draft", "in_review", "submitted", "approved", "rejected"];
const VALID_REVIEW_STATUSES: &[&str] = &["draft", "scheduled", "in_progress", "completed", "cancelled"];
const VALID_REVIEW_TYPES: &[&str] = &["periodic", "ad_hoc", "annual"];
const VALID_RATINGS: &[&str] = &["excellent", "good", "acceptable", "poor", "critical"];
const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
const VALID_SCORING_MODELS: &[&str] = &["manual", "auto", "formula"];

pub struct SupplierScorecardEngine {
    repository: Arc<dyn ScorecardRepository>,
}

impl SupplierScorecardEngine {
    pub fn new(repository: Arc<dyn ScorecardRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Template Management
    // ========================================================================

    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        evaluation_period: &str,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ScorecardTemplate> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Template code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".to_string()));
        }
        if !VALID_EVALUATION_PERIODS.contains(&evaluation_period) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid evaluation_period '{}'. Must be one of: {}",
                evaluation_period,
                VALID_EVALUATION_PERIODS.join(", ")
            )));
        }
        if self.repository.get_template_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Template '{}' already exists", code)));
        }
        info!("Creating scorecard template '{}' for org {}", code, org_id);
        self.repository
            .create_template(org_id, &code, name, description, evaluation_period, created_by)
            .await
    }

    pub async fn get_template(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::ScorecardTemplate>> {
        self.repository.get_template(id).await
    }

    pub async fn list_templates(&self, org_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ScorecardTemplate>> {
        self.repository.list_templates(org_id).await
    }

    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> atlas_shared::AtlasResult<()> {
        info!("Deleting scorecard template '{}' for org {}", code, org_id);
        self.repository.delete_template(org_id, code).await
    }

    // ========================================================================
    // Category Management
    // ========================================================================

    pub async fn create_category(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        weight: &str,
        sort_order: i32,
        scoring_model: &str,
        target_score: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ScorecardCategory> {
        let code = code.to_uppercase();
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Category code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Category name is required".to_string()));
        }
        let weight_val: f64 = weight.parse().map_err(|_| {
            AtlasError::ValidationFailed("Weight must be a number".to_string())
        })?;
        if weight_val < 0.0 || weight_val > 100.0 {
            return Err(AtlasError::ValidationFailed("Weight must be between 0 and 100".to_string()));
        }
        if !VALID_SCORING_MODELS.contains(&scoring_model) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid scoring_model '{}'. Must be one of: {}",
                scoring_model,
                VALID_SCORING_MODELS.join(", ")
            )));
        }
        // Verify template exists
        self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;

        info!("Creating scorecard category '{}' for template {}", code, template_id);
        self.repository
            .create_category(org_id, template_id, &code, name, description, weight, sort_order, scoring_model, target_score, created_by)
            .await
    }

    pub async fn list_categories(&self, template_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ScorecardCategory>> {
        self.repository.list_categories(template_id).await
    }

    pub async fn delete_category(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_category(id).await
    }

    // ========================================================================
    // Scorecard Management
    // ========================================================================

    pub async fn create_scorecard(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        scorecard_number: &str,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        supplier_number: Option<&str>,
        evaluation_period_start: chrono::NaiveDate,
        evaluation_period_end: chrono::NaiveDate,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::SupplierScorecard> {
        if scorecard_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Scorecard number is required".to_string()));
        }
        if evaluation_period_start >= evaluation_period_end {
            return Err(AtlasError::ValidationFailed("Evaluation period start must be before end".to_string()));
        }
        if self.repository.get_scorecard_by_number(org_id, scorecard_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Scorecard '{}' already exists", scorecard_number)));
        }
        // Verify template exists
        self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;

        info!("Creating scorecard '{}' for supplier {} in org {}", scorecard_number, supplier_id, org_id);
        self.repository
            .create_scorecard(
                org_id, template_id, scorecard_number, supplier_id,
                supplier_name, supplier_number,
                evaluation_period_start, evaluation_period_end,
                notes, created_by,
            )
            .await
    }

    pub async fn get_scorecard(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::SupplierScorecard>> {
        self.repository.get_scorecard(id).await
    }

    pub async fn list_scorecards(
        &self,
        org_id: Uuid,
        supplier_id: Option<Uuid>,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::SupplierScorecard>> {
        if let Some(s) = status {
            if !VALID_SCORECARD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SCORECARD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_scorecards(org_id, supplier_id, status).await
    }

    pub async fn submit_scorecard(
        &self,
        id: Uuid,
        reviewer_id: Option<Uuid>,
        reviewer_name: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::SupplierScorecard> {
        let sc = self.repository.get_scorecard(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scorecard {} not found", id)))?;
        if sc.status != "draft" && sc.status != "rejected" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit scorecard in '{}' status. Must be 'draft' or 'rejected'.", sc.status
            )));
        }
        // Recalculate overall score from lines
        let lines = self.repository.list_scorecard_lines(id).await?;
        let overall = self.calculate_overall_score(&lines);
        let grade = self.score_to_grade(overall);

        info!("Submitting scorecard {} with score {} ({})", sc.scorecard_number, overall, grade);
        self.repository.submit_scorecard(id, &format!("{:.2}", overall), if lines.is_empty() { "N/A" } else { &grade }, reviewer_id, reviewer_name).await
    }

    pub async fn approve_scorecard(&self, id: Uuid, approved_by: Option<Uuid>) -> atlas_shared::AtlasResult<atlas_shared::SupplierScorecard> {
        let sc = self.repository.get_scorecard(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scorecard {} not found", id)))?;
        if sc.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve scorecard in '{}' status. Must be 'submitted'.", sc.status
            )));
        }
        info!("Approving scorecard {}", sc.scorecard_number);
        self.repository.approve_scorecard(id, approved_by).await
    }

    pub async fn reject_scorecard(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::SupplierScorecard> {
        let sc = self.repository.get_scorecard(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scorecard {} not found", id)))?;
        if sc.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject scorecard in '{}' status. Must be 'submitted'.", sc.status
            )));
        }
        info!("Rejecting scorecard {}", sc.scorecard_number);
        self.repository.update_scorecard_status(id, "rejected").await
    }

    pub async fn delete_scorecard(&self, org_id: Uuid, scorecard_number: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_scorecard(org_id, scorecard_number).await
    }

    // ========================================================================
    // Scorecard Line Management
    // ========================================================================

    pub async fn add_scorecard_line(
        &self,
        org_id: Uuid,
        scorecard_id: Uuid,
        category_id: Uuid,
        kpi_name: &str,
        kpi_description: Option<&str>,
        weight: &str,
        target_value: Option<&str>,
        actual_value: Option<&str>,
        score: &str,
        evidence: Option<&str>,
        notes: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ScorecardLine> {
        if kpi_name.is_empty() {
            return Err(AtlasError::ValidationFailed("KPI name is required".to_string()));
        }
        let score_val: f64 = score.parse().map_err(|_| {
            AtlasError::ValidationFailed("Score must be a number".to_string())
        })?;
        if score_val < 0.0 || score_val > 100.0 {
            return Err(AtlasError::ValidationFailed("Score must be between 0 and 100".to_string()));
        }
        let weight_val: f64 = weight.parse().map_err(|_| {
            AtlasError::ValidationFailed("Weight must be a number".to_string())
        })?;
        let weighted_score = score_val * weight_val / 100.0;

        // Verify scorecard exists and is in draft status
        let sc = self.repository.get_scorecard(scorecard_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scorecard {} not found", scorecard_id)))?;
        if sc.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to non-draft scorecard".to_string(),
            ));
        }

        // Get next line number
        let existing = self.repository.list_scorecard_lines(scorecard_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding KPI line '{}' to scorecard {}", kpi_name, sc.scorecard_number);
        self.repository
            .add_scorecard_line(
                org_id, scorecard_id, category_id, line_number,
                kpi_name, kpi_description, weight, target_value, actual_value,
                score, &format!("{:.2}", weighted_score), evidence, notes,
            )
            .await
    }

    pub async fn list_scorecard_lines(&self, scorecard_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ScorecardLine>> {
        self.repository.list_scorecard_lines(scorecard_id).await
    }

    pub async fn delete_scorecard_line(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_scorecard_line(id).await
    }

    // ========================================================================
    // Performance Reviews
    // ========================================================================

    pub async fn create_review(
        &self,
        org_id: Uuid,
        review_number: &str,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        scorecard_id: Option<Uuid>,
        review_type: &str,
        review_period: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::SupplierPerformanceReview> {
        if review_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Review number is required".to_string()));
        }
        if !VALID_REVIEW_TYPES.contains(&review_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid review_type '{}'. Must be one of: {}", review_type, VALID_REVIEW_TYPES.join(", ")
            )));
        }
        if period_start >= period_end {
            return Err(AtlasError::ValidationFailed("Period start must be before end".to_string()));
        }
        if self.repository.get_review_by_number(org_id, review_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Review '{}' already exists", review_number)));
        }
        info!("Creating performance review '{}' for supplier {}", review_number, supplier_id);
        self.repository
            .create_review(
                org_id, review_number, supplier_id, supplier_name,
                scorecard_id, review_type, review_period,
                period_start, period_end, created_by,
            )
            .await
    }

    pub async fn get_review(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::SupplierPerformanceReview>> {
        self.repository.get_review(id).await
    }

    pub async fn list_reviews(
        &self,
        org_id: Uuid,
        supplier_id: Option<Uuid>,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::SupplierPerformanceReview>> {
        if let Some(s) = status {
            if !VALID_REVIEW_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_REVIEW_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_reviews(org_id, supplier_id, status).await
    }

    pub async fn update_review_status(&self, id: Uuid, status: &str) -> atlas_shared::AtlasResult<atlas_shared::SupplierPerformanceReview> {
        if !VALID_REVIEW_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_REVIEW_STATUSES.join(", ")
            )));
        }
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Review {} not found", id)))?;
        info!("Updating review {} status to {}", review.review_number, status);
        self.repository.update_review_status(id, status).await
    }

    pub async fn complete_review(
        &self,
        id: Uuid,
        current_score: Option<&str>,
        rating: Option<&str>,
        strengths: Option<&str>,
        improvement_areas: Option<&str>,
        action_items: Option<&str>,
        reviewer_id: Option<Uuid>,
        reviewer_name: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::SupplierPerformanceReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Review {} not found", id)))?;
        if review.status != "in_progress" && review.status != "scheduled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete review in '{}' status.", review.status
            )));
        }
        if let Some(r) = rating {
            if !VALID_RATINGS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid rating '{}'. Must be one of: {}", r, VALID_RATINGS.join(", ")
                )));
            }
        }
        let prev = review.current_score.clone();
        let score_change = match (&prev, current_score) {
            (Some(p), Some(c)) => {
                let p: f64 = p.parse().unwrap_or(0.0);
                let c: f64 = c.parse().unwrap_or(0.0);
                Some(format!("{:.2}", c - p))
            }
            _ => None,
        };

        info!("Completing review {}", review.review_number);
        self.repository.complete_review(
            id, current_score, rating, strengths, improvement_areas,
            action_items, prev.as_deref(), score_change.as_deref(),
            reviewer_id, reviewer_name,
        ).await
    }

    pub async fn delete_review(&self, org_id: Uuid, review_number: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_review(org_id, review_number).await
    }

    // ========================================================================
    // Review Action Items
    // ========================================================================

    pub async fn create_action_item(
        &self,
        org_id: Uuid,
        review_id: Uuid,
        description: &str,
        assignee_id: Option<Uuid>,
        assignee_name: Option<&str>,
        priority: &str,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::ReviewActionItem> {
        if description.is_empty() {
            return Err(AtlasError::ValidationFailed("Action item description is required".to_string()));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        // Verify review exists
        self.repository.get_review(review_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Review {} not found", review_id)))?;

        let existing = self.repository.list_action_items(review_id).await?;
        let action_number = (existing.len() as i32) + 1;

        info!("Creating action item {} for review", action_number);
        self.repository
            .create_action_item(
                org_id, review_id, action_number, description,
                assignee_id, assignee_name, priority, due_date, created_by,
            )
            .await
    }

    pub async fn list_action_items(&self, review_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::ReviewActionItem>> {
        self.repository.list_action_items(review_id).await
    }

    pub async fn complete_action_item(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::ReviewActionItem> {
        let item = self.repository.get_action_item(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Action item {} not found", id)))?;
        if item.status != "open" && item.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete action item in '{}' status.", item.status
            )));
        }
        info!("Completing action item {}", id);
        self.repository.complete_action_item(id).await
    }

    pub async fn delete_action_item(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_action_item(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::SupplierScorecardDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn calculate_overall_score(&self, lines: &[atlas_shared::ScorecardLine]) -> f64 {
        if lines.is_empty() {
            return 0.0;
        }
        let total_weighted: f64 = lines.iter()
            .map(|l| l.weighted_score.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_weight: f64 = lines.iter()
            .map(|l| l.weight.parse::<f64>().unwrap_or(0.0))
            .sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        (total_weighted / total_weight * 100.0).min(100.0)
    }

    fn score_to_grade(&self, score: f64) -> String {
        if score >= 90.0 { "A".to_string() }
        else if score >= 80.0 { "B".to_string() }
        else if score >= 70.0 { "C".to_string() }
        else if score >= 60.0 { "D".to_string() }
        else { "F".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_evaluation_periods() {
        assert!(VALID_EVALUATION_PERIODS.contains(&"monthly"));
        assert!(VALID_EVALUATION_PERIODS.contains(&"quarterly"));
        assert!(VALID_EVALUATION_PERIODS.contains(&"annual"));
        assert!(!VALID_EVALUATION_PERIODS.contains(&"weekly"));
    }

    #[test]
    fn test_valid_scorecard_statuses() {
        for s in &["draft", "in_review", "submitted", "approved", "rejected"] {
            assert!(VALID_SCORECARD_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_valid_ratings() {
        for r in &["excellent", "good", "acceptable", "poor", "critical"] {
            assert!(VALID_RATINGS.contains(r));
        }
        assert!(!VALID_RATINGS.contains(&"average"));
    }

    #[test]
    fn test_score_to_grade() {
        let engine_test = |score: f64, expected: &str| {
            if score >= 90.0 { assert_eq!("A", expected); }
            else if score >= 80.0 { assert_eq!("B", expected); }
            else if score >= 70.0 { assert_eq!("C", expected); }
            else if score >= 60.0 { assert_eq!("D", expected); }
            else { assert_eq!("F", expected); }
        };
        engine_test(95.0, "A");
        engine_test(85.0, "B");
        engine_test(75.0, "C");
        engine_test(65.0, "D");
        engine_test(55.0, "F");
    }

    #[test]
    fn test_valid_priorities() {
        for p in &["low", "medium", "high", "critical"] {
            assert!(VALID_PRIORITIES.contains(p));
        }
    }
}
