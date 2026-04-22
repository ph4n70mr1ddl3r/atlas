//! Performance Management Engine
//!
//! Manages performance review cycles, employee goals, competency assessments,
//! feedback, and the performance document lifecycle.
//!
//! Oracle Fusion Cloud HCM equivalent: My Client Groups > Performance

use atlas_shared::{
    PerformanceRatingModel, PerformanceReviewCycle, PerformanceCompetency,
    PerformanceDocument, PerformanceGoal, CompetencyAssessment,
    PerformanceFeedback, PerformanceDashboard,
    AtlasError, AtlasResult,
};
use super::PerformanceRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid cycle types
#[allow(dead_code)]
const VALID_CYCLE_TYPES: &[&str] = &[
    "annual", "mid_year", "quarterly", "project_end", "probation",
];

/// Valid cycle statuses
#[allow(dead_code)]
const VALID_CYCLE_STATUSES: &[&str] = &[
    "draft", "planning", "goal_setting", "self_evaluation",
    "manager_evaluation", "calibration", "completed", "cancelled",
];

/// Valid document statuses
#[allow(dead_code)]
const VALID_DOCUMENT_STATUSES: &[&str] = &[
    "not_started", "goal_setting", "self_evaluation",
    "manager_evaluation", "calibration", "completed", "cancelled",
];

/// Valid goal statuses
#[allow(dead_code)]
const VALID_GOAL_STATUSES: &[&str] = &[
    "draft", "active", "completed", "cancelled",
];

/// Valid goal categories
#[allow(dead_code)]
const VALID_GOAL_CATEGORIES: &[&str] = &[
    "performance", "development", "project", "behavioral",
];

/// Valid competency categories
#[allow(dead_code)]
const VALID_COMPETENCY_CATEGORIES: &[&str] = &[
    "core", "leadership", "technical", "functional",
];

/// Valid feedback types
#[allow(dead_code)]
const VALID_FEEDBACK_TYPES: &[&str] = &[
    "peer", "manager", "direct_report", "external", "self",
];

/// Valid feedback visibilities
#[allow(dead_code)]
const VALID_FEEDBACK_VISIBILITIES: &[&str] = &[
    "private", "manager_only", "manager_and_employee", "everyone",
];

/// Valid feedback statuses
#[allow(dead_code)]
const VALID_FEEDBACK_STATUSES: &[&str] = &[
    "draft", "submitted", "acknowledged", "withdrawn",
];

/// Performance Management engine
pub struct PerformanceEngine {
    repository: Arc<dyn PerformanceRepository>,
}

impl PerformanceEngine {
    pub fn new(repository: Arc<dyn PerformanceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Rating Models
    // ========================================================================

    /// Create or update a rating model
    pub async fn create_rating_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        rating_scale: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceRatingModel> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Rating model code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rating model name is required".to_string(),
            ));
        }
        if !rating_scale.is_array() || rating_scale.as_array().is_none_or(|a| a.is_empty()) {
            return Err(AtlasError::ValidationFailed(
                "Rating scale must be a non-empty array".to_string(),
            ));
        }

        info!("Creating rating model '{}' for org {}", code, org_id);
        self.repository.create_rating_model(
            org_id, &code, name, description, rating_scale, created_by,
        ).await
    }

    /// Get a rating model by code
    pub async fn get_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceRatingModel>> {
        self.repository.get_rating_model(org_id, &code.to_uppercase()).await
    }

    /// List rating models
    pub async fn list_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<PerformanceRatingModel>> {
        self.repository.list_rating_models(org_id).await
    }

    /// Deactivate a rating model
    pub async fn delete_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating rating model '{}' for org {}", code, org_id);
        self.repository.delete_rating_model(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Review Cycles
    // ========================================================================

    /// Create a review cycle
    pub async fn create_review_cycle(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        cycle_type: &str,
        rating_model_code: Option<&str>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        goal_setting_start: Option<chrono::NaiveDate>,
        goal_setting_end: Option<chrono::NaiveDate>,
        self_evaluation_start: Option<chrono::NaiveDate>,
        self_evaluation_end: Option<chrono::NaiveDate>,
        manager_evaluation_start: Option<chrono::NaiveDate>,
        manager_evaluation_end: Option<chrono::NaiveDate>,
        calibration_date: Option<chrono::NaiveDate>,
        require_goals: bool,
        require_competencies: bool,
        min_goals: i32,
        max_goals: i32,
        goal_weight_total: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceReviewCycle> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Cycle name is required".to_string()));
        }
        if !VALID_CYCLE_TYPES.contains(&cycle_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cycle_type '{}'. Must be one of: {}", cycle_type, VALID_CYCLE_TYPES.join(", ")
            )));
        }
        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }
        if min_goals < 0 || max_goals < min_goals {
            return Err(AtlasError::ValidationFailed(
                "Invalid goal count range".to_string(),
            ));
        }
        let weight: f64 = goal_weight_total.parse().map_err(|_| AtlasError::ValidationFailed(
            "Goal weight total must be a valid number".to_string(),
        ))?;
        if weight <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Goal weight total must be positive".to_string(),
            ));
        }

        // Resolve rating model
        let rating_model_id = if let Some(code) = rating_model_code {
            let model = self.get_rating_model(org_id, code).await?;
            model.map(|m| m.id)
        } else {
            None
        };

        info!("Creating review cycle '{}' ({}) for org {}", name, cycle_type, org_id);
        self.repository.create_review_cycle(
            org_id, name, description, cycle_type,
            rating_model_id, start_date, end_date,
            goal_setting_start, goal_setting_end,
            self_evaluation_start, self_evaluation_end,
            manager_evaluation_start, manager_evaluation_end,
            calibration_date,
            require_goals, require_competencies, min_goals, max_goals,
            goal_weight_total, created_by,
        ).await
    }

    /// Get a review cycle
    pub async fn get_review_cycle(&self, id: Uuid) -> AtlasResult<Option<PerformanceReviewCycle>> {
        self.repository.get_review_cycle(id).await
    }

    /// List review cycles
    pub async fn list_review_cycles(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PerformanceReviewCycle>> {
        if let Some(s) = status {
            if !VALID_CYCLE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CYCLE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_review_cycles(org_id, status).await
    }

    /// Transition cycle status
    pub async fn transition_cycle(
        &self,
        cycle_id: Uuid,
        new_status: &str,
    ) -> AtlasResult<PerformanceReviewCycle> {
        let cycle = self.repository.get_review_cycle(cycle_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Review cycle {} not found", cycle_id)
            ))?;

        // Validate transition
        let valid_next = match cycle.status.as_str() {
            "draft" => vec!["planning"],
            "planning" => vec!["goal_setting"],
            "goal_setting" => vec!["self_evaluation"],
            "self_evaluation" => vec!["manager_evaluation"],
            "manager_evaluation" => vec!["calibration", "completed"],
            "calibration" => vec!["completed"],
            _ => vec![],
        };
        if cycle.status == "cancelled" || !valid_next.contains(&new_status) {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot transition cycle from '{}' to '{}'", cycle.status, new_status
            )));
        }

        info!("Transitioning review cycle {} from {} to {}", cycle_id, cycle.status, new_status);
        self.repository.update_cycle_status(cycle_id, new_status).await
    }

    // ========================================================================
    // Competencies
    // ========================================================================

    /// Create a competency
    pub async fn create_competency(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: Option<&str>,
        rating_model_code: Option<&str>,
        behavioral_indicators: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceCompetency> {
        let code = code.to_uppercase();
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Competency code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Competency name is required".to_string()));
        }
        if let Some(cat) = category {
            if !VALID_COMPETENCY_CATEGORIES.contains(&cat) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid category '{}'. Must be one of: {}", cat, VALID_COMPETENCY_CATEGORIES.join(", ")
                )));
            }
        }

        let rating_model_id = if let Some(rm_code) = rating_model_code {
            self.get_rating_model(org_id, rm_code).await?.map(|m| m.id)
        } else {
            None
        };

        info!("Creating competency '{}' for org {}", code, org_id);
        self.repository.create_competency(
            org_id, &code, name, description, category,
            rating_model_id, behavioral_indicators, created_by,
        ).await
    }

    /// Get a competency by code
    pub async fn get_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceCompetency>> {
        self.repository.get_competency(org_id, &code.to_uppercase()).await
    }

    /// List competencies
    pub async fn list_competencies(
        &self,
        org_id: Uuid,
        category: Option<&str>,
    ) -> AtlasResult<Vec<PerformanceCompetency>> {
        self.repository.list_competencies(org_id, category).await
    }

    /// Deactivate a competency
    pub async fn delete_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating competency '{}' for org {}", code, org_id);
        self.repository.delete_competency(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Performance Documents
    // ========================================================================

    /// Create a performance document for an employee in a review cycle
    pub async fn create_document(
        &self,
        org_id: Uuid,
        review_cycle_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceDocument> {
        // Check cycle exists
        let cycle = self.repository.get_review_cycle(review_cycle_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Review cycle {} not found", review_cycle_id)
            ))?;

        if cycle.status == "draft" || cycle.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot create documents in '{}' cycle status", cycle.status)
            ));
        }

        // Check for duplicate
        let existing = self.repository.get_document_by_cycle_employee(org_id, review_cycle_id, employee_id).await?;
        if existing.is_some() {
            return Err(AtlasError::Conflict(
                format!("Employee {} already has a document in this cycle", employee_id)
            ));
        }

        let doc_number = format!("PD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating performance document {} for employee {} in cycle {}", doc_number, employee_id, review_cycle_id);
        self.repository.create_document(
            org_id, review_cycle_id, employee_id, employee_name,
            manager_id, manager_name, &doc_number, created_by,
        ).await
    }

    /// Get a performance document
    pub async fn get_document(&self, id: Uuid) -> AtlasResult<Option<PerformanceDocument>> {
        self.repository.get_document(id).await
    }

    /// List documents for a review cycle
    pub async fn list_documents(
        &self,
        org_id: Uuid,
        review_cycle_id: Option<Uuid>,
        employee_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PerformanceDocument>> {
        if let Some(s) = status {
            if !VALID_DOCUMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid document status '{}'. Must be one of: {}", s, VALID_DOCUMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_documents(org_id, review_cycle_id, employee_id, status).await
    }

    /// Transition a document to the next status
    pub async fn transition_document(
        &self,
        document_id: Uuid,
        new_status: &str,
    ) -> AtlasResult<PerformanceDocument> {
        let doc = self.repository.get_document(document_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Document {} not found", document_id)
            ))?;

        let valid_next = match doc.status.as_str() {
            "not_started" => vec!["goal_setting"],
            "goal_setting" => vec!["self_evaluation"],
            "self_evaluation" => vec!["manager_evaluation"],
            "manager_evaluation" => vec!["calibration", "completed"],
            "calibration" => vec!["completed"],
            _ => vec![],
        };
        if doc.status == "cancelled" || !valid_next.contains(&new_status) {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot transition document from '{}' to '{}'", doc.status, new_status
            )));
        }

        info!("Transitioning document {} from {} to {}", document_id, doc.status, new_status);
        self.repository.update_document_status(document_id, new_status).await
    }

    /// Submit self-evaluation for a document
    pub async fn submit_self_evaluation(
        &self,
        document_id: Uuid,
        overall_rating: Option<&str>,
        comments: Option<&str>,
    ) -> AtlasResult<PerformanceDocument> {
        let doc = self.repository.get_document(document_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Document {} not found", document_id)
            ))?;

        if doc.status != "self_evaluation" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit self-evaluation in '{}' status", doc.status)
            ));
        }

        self.repository.update_self_evaluation(document_id, overall_rating, comments).await
    }

    /// Submit manager evaluation for a document
    pub async fn submit_manager_evaluation(
        &self,
        document_id: Uuid,
        overall_rating: Option<&str>,
        comments: Option<&str>,
    ) -> AtlasResult<PerformanceDocument> {
        let doc = self.repository.get_document(document_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Document {} not found", document_id)
            ))?;

        if doc.status != "manager_evaluation" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit manager evaluation in '{}' status", doc.status)
            ));
        }

        self.repository.update_manager_evaluation(document_id, overall_rating, comments).await
    }

    /// Finalize a document with a final rating
    pub async fn finalize_document(
        &self,
        document_id: Uuid,
        final_rating: Option<&str>,
        final_comments: Option<&str>,
    ) -> AtlasResult<PerformanceDocument> {
        let doc = self.repository.get_document(document_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Document {} not found", document_id)
            ))?;

        if doc.status != "calibration" && doc.status != "manager_evaluation" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot finalize document in '{}' status", doc.status)
            ));
        }

        info!("Finalizing document {} with rating {:?}", document_id, final_rating);
        self.repository.finalize_document(document_id, final_rating, final_comments).await
    }

    // ========================================================================
    // Goals
    // ========================================================================

    /// Add a goal to a performance document
    pub async fn create_goal(
        &self,
        org_id: Uuid,
        document_id: Uuid,
        employee_id: Uuid,
        goal_name: &str,
        description: Option<&str>,
        goal_category: Option<&str>,
        weight: &str,
        target_metric: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceGoal> {
        // Validate document exists and is in the right state
        let doc = self.repository.get_document(document_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Document {} not found", document_id)
            ))?;

        if doc.status != "goal_setting" && doc.status != "not_started" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add goals to document in '{}' status", doc.status)
            ));
        }

        if goal_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Goal name is required".to_string()));
        }
        if let Some(cat) = goal_category {
            if !VALID_GOAL_CATEGORIES.contains(&cat) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid goal category '{}'. Must be one of: {}", cat, VALID_GOAL_CATEGORIES.join(", ")
                )));
            }
        }
        let weight_val: f64 = weight.parse().map_err(|_| AtlasError::ValidationFailed(
            "Weight must be a valid number".to_string(),
        ))?;
        if weight_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Goal weight must be positive".to_string(),
            ));
        }

        info!("Creating goal '{}' for document {}", goal_name, document_id);
        self.repository.create_goal(
            org_id, document_id, employee_id, goal_name, description,
            goal_category, weight, target_metric, start_date, due_date, created_by,
        ).await
    }

    /// Get a goal
    pub async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<PerformanceGoal>> {
        self.repository.get_goal(id).await
    }

    /// List goals for a document
    pub async fn list_goals(&self, document_id: Uuid) -> AtlasResult<Vec<PerformanceGoal>> {
        self.repository.list_goals(document_id).await
    }

    /// Complete a goal
    pub async fn complete_goal(
        &self,
        goal_id: Uuid,
        actual_result: Option<&str>,
    ) -> AtlasResult<PerformanceGoal> {
        let goal = self.repository.get_goal(goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Goal {} not found", goal_id)
            ))?;

        if goal.status != "active" && goal.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete goal in '{}' status", goal.status)
            ));
        }

        info!("Completing goal {}", goal_id);
        self.repository.update_goal_status(goal_id, "completed", actual_result, Some(chrono::Utc::now().date_naive())).await
    }

    /// Rate a goal (self or manager)
    pub async fn rate_goal(
        &self,
        goal_id: Uuid,
        rating_type: &str, // "self" or "manager"
        rating: &str,
        comments: Option<&str>,
    ) -> AtlasResult<PerformanceGoal> {
        let _goal = self.repository.get_goal(goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Goal {} not found", goal_id)
            ))?;

        let rating_val: f64 = rating.parse().map_err(|_| AtlasError::ValidationFailed(
            "Rating must be a valid number".to_string(),
        ))?;
        if rating_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Rating must be non-negative".to_string()));
        }

        info!("Rating goal {} ({}) with {}", goal_id, rating_type, rating);
        match rating_type {
            "self" => self.repository.update_goal_self_rating(goal_id, rating, comments).await,
            "manager" => self.repository.update_goal_manager_rating(goal_id, rating, comments).await,
            _ => Err(AtlasError::ValidationFailed(
                "Rating type must be 'self' or 'manager'".to_string(),
            )),
        }
    }

    /// Delete a goal
    pub async fn delete_goal(&self, goal_id: Uuid) -> AtlasResult<()> {
        let goal = self.repository.get_goal(goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Goal {} not found", goal_id)
            ))?;
        if goal.status == "completed" {
            return Err(AtlasError::WorkflowError("Cannot delete completed goal".to_string()));
        }
        self.repository.delete_goal(goal_id).await
    }

    // ========================================================================
    // Competency Assessments
    // ========================================================================

    /// Create or update a competency assessment
    pub async fn upsert_competency_assessment(
        &self,
        org_id: Uuid,
        document_id: Uuid,
        employee_id: Uuid,
        competency_id: Uuid,
        rating_type: &str, // "self", "manager", "calibration"
        rating: &str,
        comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompetencyAssessment> {
        let rating_val: f64 = rating.parse().map_err(|_| AtlasError::ValidationFailed(
            "Rating must be a valid number".to_string(),
        ))?;
        if rating_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Rating must be non-negative".to_string()));
        }
        if !["self", "manager", "calibration"].contains(&rating_type) {
            return Err(AtlasError::ValidationFailed(
                "Rating type must be 'self', 'manager', or 'calibration'".to_string(),
            ));
        }

        info!("Upserting competency assessment for doc {} comp {} ({})", document_id, competency_id, rating_type);
        self.repository.upsert_competency_assessment(
            org_id, document_id, employee_id, competency_id,
            rating_type, rating, comments, created_by,
        ).await
    }

    /// List competency assessments for a document
    pub async fn list_competency_assessments(
        &self,
        document_id: Uuid,
    ) -> AtlasResult<Vec<CompetencyAssessment>> {
        self.repository.list_competency_assessments(document_id).await
    }

    // ========================================================================
    // Feedback
    // ========================================================================

    /// Create feedback
    pub async fn create_feedback(
        &self,
        org_id: Uuid,
        document_id: Option<Uuid>,
        employee_id: Uuid,
        from_user_id: Uuid,
        from_user_name: Option<&str>,
        feedback_type: &str,
        subject: Option<&str>,
        content: &str,
        visibility: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceFeedback> {
        if !VALID_FEEDBACK_TYPES.contains(&feedback_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid feedback_type '{}'. Must be one of: {}", feedback_type, VALID_FEEDBACK_TYPES.join(", ")
            )));
        }
        if !VALID_FEEDBACK_VISIBILITIES.contains(&visibility) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid visibility '{}'. Must be one of: {}", visibility, VALID_FEEDBACK_VISIBILITIES.join(", ")
            )));
        }
        if content.is_empty() {
            return Err(AtlasError::ValidationFailed("Feedback content is required".to_string()));
        }

        info!("Creating {} feedback for employee {}", feedback_type, employee_id);
        self.repository.create_feedback(
            org_id, document_id, employee_id, from_user_id, from_user_name,
            feedback_type, subject, content, visibility, created_by,
        ).await
    }

    /// List feedback for an employee or document
    pub async fn list_feedback(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        document_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PerformanceFeedback>> {
        self.repository.list_feedback(org_id, employee_id, document_id).await
    }

    /// Submit feedback (change from draft to submitted)
    pub async fn submit_feedback(&self, feedback_id: Uuid) -> AtlasResult<PerformanceFeedback> {
        let fb = self.repository.get_feedback(feedback_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Feedback {} not found", feedback_id)
            ))?;

        if fb.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit feedback in '{}' status", fb.status)
            ));
        }

        info!("Submitting feedback {}", feedback_id);
        self.repository.update_feedback_status(feedback_id, "submitted").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get performance dashboard for a review cycle
    pub async fn get_dashboard(
        &self,
        org_id: Uuid,
        review_cycle_id: Uuid,
    ) -> AtlasResult<PerformanceDashboard> {
        let docs = self.repository.list_documents(org_id, Some(review_cycle_id), None, None).await?;
        let goals = self.repository.list_goals_by_cycle(org_id, review_cycle_id).await?;
        let feedback = self.repository.list_feedback(org_id, None, None).await?;

        let total = docs.len() as i32;
        let not_started = docs.iter().filter(|d| d.status == "not_started").count() as i32;
        let goal_setting = docs.iter().filter(|d| d.status == "goal_setting").count() as i32;
        let self_eval = docs.iter().filter(|d| d.status == "self_evaluation").count() as i32;
        let mgr_eval = docs.iter().filter(|d| d.status == "manager_evaluation").count() as i32;
        let calibration = docs.iter().filter(|d| d.status == "calibration").count() as i32;
        let completed = docs.iter().filter(|d| d.status == "completed").count() as i32;
        let cancelled = docs.iter().filter(|d| d.status == "cancelled").count() as i32;

        let avg_rating = if completed > 0 {
            let sum: f64 = docs.iter()
                .filter(|d| d.status == "completed")
                .filter_map(|d| d.final_rating.as_ref().and_then(|r| r.parse::<f64>().ok()))
                .sum();
            Some(format!("{:.2}", sum / completed as f64))
        } else {
            None
        };

        let goals_total = goals.len() as i32;
        let goals_completed = goals.iter().filter(|g| g.status == "completed").count() as i32;

        Ok(PerformanceDashboard {
            review_cycle_id,
            total_documents: total,
            not_started_count: not_started,
            goal_setting_count: goal_setting,
            self_evaluation_count: self_eval,
            manager_evaluation_count: mgr_eval,
            calibration_count: calibration,
            completed_count: completed,
            cancelled_count: cancelled,
            average_rating: avg_rating,
            goals_total,
            goals_completed,
            feedback_count: feedback.len() as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cycle_types() {
        assert!(VALID_CYCLE_TYPES.contains(&"annual"));
        assert!(VALID_CYCLE_TYPES.contains(&"mid_year"));
        assert!(VALID_CYCLE_TYPES.contains(&"quarterly"));
        assert!(VALID_CYCLE_TYPES.contains(&"project_end"));
        assert!(VALID_CYCLE_TYPES.contains(&"probation"));
    }

    #[test]
    fn test_valid_cycle_statuses() {
        for s in VALID_CYCLE_STATUSES {
            assert!(!s.is_empty());
        }
        assert!(VALID_CYCLE_STATUSES.contains(&"draft"));
        assert!(VALID_CYCLE_STATUSES.contains(&"completed"));
    }

    #[test]
    fn test_valid_document_statuses() {
        assert!(VALID_DOCUMENT_STATUSES.contains(&"not_started"));
        assert!(VALID_DOCUMENT_STATUSES.contains(&"goal_setting"));
        assert!(VALID_DOCUMENT_STATUSES.contains(&"self_evaluation"));
        assert!(VALID_DOCUMENT_STATUSES.contains(&"manager_evaluation"));
        assert!(VALID_DOCUMENT_STATUSES.contains(&"calibration"));
        assert!(VALID_DOCUMENT_STATUSES.contains(&"completed"));
    }

    #[test]
    fn test_valid_goal_categories() {
        assert!(VALID_GOAL_CATEGORIES.contains(&"performance"));
        assert!(VALID_GOAL_CATEGORIES.contains(&"development"));
        assert!(VALID_GOAL_CATEGORIES.contains(&"project"));
        assert!(VALID_GOAL_CATEGORIES.contains(&"behavioral"));
    }

    #[test]
    fn test_valid_feedback_types() {
        assert!(VALID_FEEDBACK_TYPES.contains(&"peer"));
        assert!(VALID_FEEDBACK_TYPES.contains(&"manager"));
        assert!(VALID_FEEDBACK_TYPES.contains(&"direct_report"));
        assert!(VALID_FEEDBACK_TYPES.contains(&"external"));
        assert!(VALID_FEEDBACK_TYPES.contains(&"self"));
    }

    #[test]
    fn test_valid_feedback_visibilities() {
        assert!(VALID_FEEDBACK_VISIBILITIES.contains(&"private"));
        assert!(VALID_FEEDBACK_VISIBILITIES.contains(&"manager_only"));
        assert!(VALID_FEEDBACK_VISIBILITIES.contains(&"manager_and_employee"));
        assert!(VALID_FEEDBACK_VISIBILITIES.contains(&"everyone"));
    }

    #[test]
    fn test_valid_competency_categories() {
        assert!(VALID_COMPETENCY_CATEGORIES.contains(&"core"));
        assert!(VALID_COMPETENCY_CATEGORIES.contains(&"leadership"));
        assert!(VALID_COMPETENCY_CATEGORIES.contains(&"technical"));
        assert!(VALID_COMPETENCY_CATEGORIES.contains(&"functional"));
    }
}
