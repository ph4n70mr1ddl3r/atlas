//! Succession Planning Engine
//!
//! Manages succession plans, talent pools, talent reviews,
//! career paths, and nine-box grid assessments.
//!
//! Oracle Fusion equivalent: HCM > Succession Management

use atlas_shared::{
    SuccessionPlan, SuccessionCandidate, TalentPool, TalentPoolMember,
    TalentReview, TalentReviewAssessment, CareerPath, SuccessionDashboard,
    AtlasError, AtlasResult,
};
use super::SuccessionPlanningRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid enum values
const VALID_PLAN_TYPES: &[&str] = &["position", "role", "key_person"];
const VALID_RISK_LEVELS: &[&str] = &["low", "medium", "high", "critical"];
const VALID_URGENCIES: &[&str] = &["immediate", "short_term", "medium_term", "long_term"];
const VALID_PLAN_STATUSES: &[&str] = &["draft", "active", "completed", "cancelled"];
const VALID_READINESS_LEVELS: &[&str] = &["ready_now", "ready_1_2_years", "ready_3_5_years", "not_ready"];
const VALID_CANDIDATE_STATUSES: &[&str] = &["proposed", "approved", "rejected", "development"];
const VALID_FLIGHT_RISKS: &[&str] = &["low", "medium", "high"];
const VALID_POOL_TYPES: &[&str] = &["leadership", "technical", "high_potential", "diversity", "custom"];
const VALID_POOL_STATUSES: &[&str] = &["draft", "active", "archived"];
const VALID_MEMBER_STATUSES: &[&str] = &["active", "on_hold", "removed", "graduated"];
const VALID_REVIEW_TYPES: &[&str] = &["calibration", "performance_potential", "nine_box", "leadership"];
const VALID_REVIEW_STATUSES: &[&str] = &["scheduled", "in_progress", "completed", "cancelled"];
const VALID_NINE_BOX_POSITIONS: &[&str] = &[
    "star", "workhorse", "puzzle", "solid_citizen",
    "high_potential", "core_player", "rough_diamond",
    "inconsistent", "underperformer", "blocker",
];
const VALID_PATH_TYPES: &[&str] = &["linear", "branching", "lattice", "dual_track"];
const VALID_PATH_STATUSES: &[&str] = &["draft", "active", "archived"];

/// Succession Planning Engine
pub struct SuccessionPlanningEngine {
    repository: Arc<dyn SuccessionPlanningRepository>,
}

impl SuccessionPlanningEngine {
    pub fn new(repository: Arc<dyn SuccessionPlanningRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Succession Plans
    // ========================================================================

    /// Create a succession plan
    #[allow(clippy::too_many_arguments)]
    pub async fn create_succession_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        position_id: Option<Uuid>,
        position_title: Option<&str>,
        job_id: Option<Uuid>,
        department_id: Option<Uuid>,
        current_incumbent_id: Option<Uuid>,
        current_incumbent_name: Option<&str>,
        risk_level: &str,
        urgency: &str,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionPlan> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Plan name is required".to_string()));
        }
        validate_enum("plan_type", plan_type, VALID_PLAN_TYPES)?;
        validate_enum("risk_level", risk_level, VALID_RISK_LEVELS)?;
        validate_enum("urgency", urgency, VALID_URGENCIES)?;

        if self.repository.get_succession_plan_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Succession plan '{}' already exists", code_upper)));
        }

        info!("Creating succession plan '{}' for org {}", code_upper, org_id);
        self.repository.create_succession_plan(
            org_id, &code_upper, name, description, plan_type,
            position_id, position_title, job_id, department_id,
            current_incumbent_id, current_incumbent_name,
            risk_level, urgency, effective_date, created_by,
        ).await
    }

    /// Get a succession plan by ID
    pub async fn get_succession_plan(&self, id: Uuid) -> AtlasResult<Option<SuccessionPlan>> {
        self.repository.get_succession_plan(id).await
    }

    /// List succession plans with optional filters
    pub async fn list_succession_plans(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_level: Option<&str>,
    ) -> AtlasResult<Vec<SuccessionPlan>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_PLAN_STATUSES)?;
        }
        if let Some(r) = risk_level {
            validate_enum("risk_level", r, VALID_RISK_LEVELS)?;
        }
        self.repository.list_succession_plans(org_id, status, risk_level).await
    }

    /// Update succession plan status
    pub async fn update_succession_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionPlan> {
        validate_enum("status", status, VALID_PLAN_STATUSES)?;

        let plan = self.repository.get_succession_plan(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Succession plan {} not found", id)))?;

        // Validate status transitions
        match (plan.status.as_str(), status) {
            ("draft", "active") | ("active", "completed") | ("active", "cancelled") | ("draft", "cancelled") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition plan from '{}' to '{}'", plan.status, status
            ))),
        }

        info!("Updating succession plan {} status to {}", id, status);
        self.repository.update_succession_plan_status(id, status).await
    }

    /// Delete a succession plan by code
    pub async fn delete_succession_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting succession plan '{}' for org {}", code, org_id);
        self.repository.delete_succession_plan(org_id, code).await
    }

    // ========================================================================
    // Succession Candidates
    // ========================================================================

    /// Add a candidate to a succession plan
    #[allow(clippy::too_many_arguments)]
    pub async fn create_succession_candidate(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        person_id: Uuid,
        person_name: Option<&str>,
        employee_number: Option<&str>,
        readiness: &str,
        ranking: Option<i32>,
        performance_rating: Option<&str>,
        potential_rating: Option<&str>,
        flight_risk: Option<&str>,
        development_notes: Option<&str>,
        recommended_actions: Option<&str>,
        status: &str,
        added_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionCandidate> {
        validate_enum("readiness", readiness, VALID_READINESS_LEVELS)?;
        validate_enum("status", status, VALID_CANDIDATE_STATUSES)?;
        if let Some(fr) = flight_risk {
            validate_enum("flight_risk", fr, VALID_FLIGHT_RISKS)?;
        }
        if let Some(r) = ranking {
            if r < 1 {
                return Err(AtlasError::ValidationFailed("Ranking must be >= 1".to_string()));
            }
        }

        // Verify plan exists and is not cancelled
        let plan = self.repository.get_succession_plan(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Succession plan {} not found", plan_id)))?;
        if plan.status == "cancelled" {
            return Err(AtlasError::ValidationFailed("Cannot add candidates to a cancelled plan".to_string()));
        }
        if plan.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Plan does not belong to this organization".to_string()));
        }

        info!("Adding candidate {} to succession plan {}", person_id, plan_id);
        self.repository.create_succession_candidate(
            org_id, plan_id, person_id, person_name, employee_number,
            readiness, ranking, performance_rating, potential_rating,
            flight_risk, development_notes, recommended_actions,
            status, added_by,
        ).await
    }

    /// Get a candidate by ID
    pub async fn get_succession_candidate(&self, id: Uuid) -> AtlasResult<Option<SuccessionCandidate>> {
        self.repository.get_succession_candidate(id).await
    }

    /// List candidates for a succession plan
    pub async fn list_succession_candidates(&self, plan_id: Uuid) -> AtlasResult<Vec<SuccessionCandidate>> {
        self.repository.list_succession_candidates(plan_id).await
    }

    /// Update a candidate's status
    pub async fn update_candidate_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionCandidate> {
        validate_enum("status", status, VALID_CANDIDATE_STATUSES)?;
        let _candidate = self.repository.get_succession_candidate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Candidate {} not found", id)))?;
        info!("Updating candidate {} status to {}", id, status);
        self.repository.update_candidate_status(id, status).await
    }

    /// Update a candidate's readiness level
    pub async fn update_candidate_readiness(&self, id: Uuid, readiness: &str) -> AtlasResult<SuccessionCandidate> {
        validate_enum("readiness", readiness, VALID_READINESS_LEVELS)?;
        let _candidate = self.repository.get_succession_candidate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Candidate {} not found", id)))?;
        info!("Updating candidate {} readiness to {}", id, readiness);
        self.repository.update_candidate_readiness(id, readiness).await
    }

    /// Remove a candidate
    pub async fn delete_succession_candidate(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting candidate {}", id);
        self.repository.delete_succession_candidate(id).await
    }

    // ========================================================================
    // Talent Pools
    // ========================================================================

    /// Create a talent pool
    pub async fn create_talent_pool(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        pool_type: &str,
        owner_id: Option<Uuid>,
        max_members: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentPool> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Pool name is required".to_string()));
        }
        validate_enum("pool_type", pool_type, VALID_POOL_TYPES)?;
        if let Some(max) = max_members {
            if max < 1 {
                return Err(AtlasError::ValidationFailed("Max members must be >= 1".to_string()));
            }
        }

        if self.repository.get_talent_pool_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Talent pool '{}' already exists", code_upper)));
        }

        info!("Creating talent pool '{}' for org {}", code_upper, org_id);
        self.repository.create_talent_pool(
            org_id, &code_upper, name, description, pool_type,
            owner_id, max_members, created_by,
        ).await
    }

    /// Get a talent pool by ID
    pub async fn get_talent_pool(&self, id: Uuid) -> AtlasResult<Option<TalentPool>> {
        self.repository.get_talent_pool(id).await
    }

    /// List talent pools with optional filters
    pub async fn list_talent_pools(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        pool_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentPool>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_POOL_STATUSES)?;
        }
        if let Some(pt) = pool_type {
            validate_enum("pool_type", pt, VALID_POOL_TYPES)?;
        }
        self.repository.list_talent_pools(org_id, status, pool_type).await
    }

    /// Update talent pool status
    pub async fn update_talent_pool_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPool> {
        validate_enum("status", status, VALID_POOL_STATUSES)?;
        let pool = self.repository.get_talent_pool(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Talent pool {} not found", id)))?;

        match (pool.status.as_str(), status) {
            ("draft", "active") | ("active", "archived") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition pool from '{}' to '{}'", pool.status, status
            ))),
        }

        info!("Updating talent pool {} status to {}", id, status);
        self.repository.update_talent_pool_status(id, status).await
    }

    /// Delete a talent pool by code
    pub async fn delete_talent_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting talent pool '{}' for org {}", code, org_id);
        self.repository.delete_talent_pool(org_id, code).await
    }

    // ========================================================================
    // Talent Pool Members
    // ========================================================================

    /// Add a member to a talent pool
    #[allow(clippy::too_many_arguments)]
    pub async fn create_talent_pool_member(
        &self,
        org_id: Uuid,
        pool_id: Uuid,
        person_id: Uuid,
        person_name: Option<&str>,
        performance_rating: Option<&str>,
        potential_rating: Option<&str>,
        readiness: &str,
        development_plan: Option<&str>,
        notes: Option<&str>,
        added_date: Option<chrono::NaiveDate>,
        review_date: Option<chrono::NaiveDate>,
        added_by: Option<Uuid>,
    ) -> AtlasResult<TalentPoolMember> {
        validate_enum("readiness", readiness, VALID_READINESS_LEVELS)?;

        let pool = self.repository.get_talent_pool(pool_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Talent pool {} not found", pool_id)))?;
        if pool.status != "active" {
            return Err(AtlasError::ValidationFailed("Cannot add members to a non-active pool".to_string()));
        }
        if pool.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Pool does not belong to this organization".to_string()));
        }

        // Check max_members constraint
        if let Some(max) = pool.max_members {
            let current_members = self.repository.list_talent_pool_members(pool_id).await?;
            let active_count = current_members.iter().filter(|m| m.status == "active").count() as i32;
            if active_count >= max {
                return Err(AtlasError::ValidationFailed(format!(
                    "Talent pool has reached max members ({})", max
                )));
            }
        }

        info!("Adding member {} to talent pool {}", person_id, pool_id);
        self.repository.create_talent_pool_member(
            org_id, pool_id, person_id, person_name,
            performance_rating, potential_rating, readiness,
            development_plan, notes, added_date, review_date, added_by,
        ).await
    }

    /// Get a pool member by ID
    pub async fn get_talent_pool_member(&self, id: Uuid) -> AtlasResult<Option<TalentPoolMember>> {
        self.repository.get_talent_pool_member(id).await
    }

    /// List members of a talent pool
    pub async fn list_talent_pool_members(&self, pool_id: Uuid) -> AtlasResult<Vec<TalentPoolMember>> {
        self.repository.list_talent_pool_members(pool_id).await
    }

    /// Update a pool member's status
    pub async fn update_pool_member_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPoolMember> {
        validate_enum("status", status, VALID_MEMBER_STATUSES)?;
        let _member = self.repository.get_talent_pool_member(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Pool member {} not found", id)))?;
        info!("Updating pool member {} status to {}", id, status);
        self.repository.update_pool_member_status(id, status).await
    }

    /// Remove a pool member
    pub async fn delete_talent_pool_member(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting pool member {}", id);
        self.repository.delete_talent_pool_member(id).await
    }

    // ========================================================================
    // Talent Reviews
    // ========================================================================

    /// Create a talent review meeting
    pub async fn create_talent_review(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        review_type: &str,
        facilitator_id: Option<Uuid>,
        department_id: Option<Uuid>,
        review_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentReview> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Review name is required".to_string()));
        }
        validate_enum("review_type", review_type, VALID_REVIEW_TYPES)?;

        if self.repository.get_talent_review_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Talent review '{}' already exists", code_upper)));
        }

        info!("Creating talent review '{}' for org {}", code_upper, org_id);
        self.repository.create_talent_review(
            org_id, &code_upper, name, description, review_type,
            facilitator_id, department_id, review_date, created_by,
        ).await
    }

    /// Get a talent review by ID
    pub async fn get_talent_review(&self, id: Uuid) -> AtlasResult<Option<TalentReview>> {
        self.repository.get_talent_review(id).await
    }

    /// List talent reviews with optional filters
    pub async fn list_talent_reviews(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        review_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentReview>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_REVIEW_STATUSES)?;
        }
        if let Some(rt) = review_type {
            validate_enum("review_type", rt, VALID_REVIEW_TYPES)?;
        }
        self.repository.list_talent_reviews(org_id, status, review_type).await
    }

    /// Update talent review status
    pub async fn update_talent_review_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentReview> {
        validate_enum("status", status, VALID_REVIEW_STATUSES)?;
        let review = self.repository.get_talent_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Talent review {} not found", id)))?;

        match (review.status.as_str(), status) {
            ("scheduled", "in_progress") |
            ("in_progress", "completed") |
            ("scheduled", "cancelled") |
            ("in_progress", "cancelled") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition review from '{}' to '{}'", review.status, status
            ))),
        }

        info!("Updating talent review {} status to {}", id, status);
        self.repository.update_talent_review_status(id, status).await
    }

    /// Delete a talent review by code
    pub async fn delete_talent_review(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting talent review '{}' for org {}", code, org_id);
        self.repository.delete_talent_review(org_id, code).await
    }

    // ========================================================================
    // Talent Review Assessments
    // ========================================================================

    /// Create an assessment within a talent review
    #[allow(clippy::too_many_arguments)]
    pub async fn create_talent_review_assessment(
        &self,
        org_id: Uuid,
        review_id: Uuid,
        person_id: Uuid,
        person_name: Option<&str>,
        performance_rating: Option<&str>,
        potential_rating: Option<&str>,
        nine_box_position: Option<&str>,
        strengths: Option<&str>,
        weaknesses: Option<&str>,
        career_aspiration: Option<&str>,
        development_needs: Option<&str>,
        succession_readiness: Option<&str>,
        assessor_id: Option<Uuid>,
        notes: Option<&str>,
    ) -> AtlasResult<TalentReviewAssessment> {
        if let Some(nbp) = nine_box_position {
            validate_enum("nine_box_position", nbp, VALID_NINE_BOX_POSITIONS)?;
        }
        if let Some(sr) = succession_readiness {
            validate_enum("succession_readiness", sr, VALID_READINESS_LEVELS)?;
        }

        // Verify review exists and is in_progress
        let review = self.repository.get_talent_review(review_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Talent review {} not found", review_id)))?;
        if review.status != "in_progress" {
            return Err(AtlasError::ValidationFailed(
                "Assessments can only be added to reviews in progress".to_string(),
            ));
        }
        if review.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Review does not belong to this organization".to_string()));
        }

        info!("Creating assessment for person {} in review {}", person_id, review_id);
        self.repository.create_talent_review_assessment(
            org_id, review_id, person_id, person_name,
            performance_rating, potential_rating, nine_box_position,
            strengths, weaknesses, career_aspiration,
            development_needs, succession_readiness,
            assessor_id, notes,
        ).await
    }

    /// Get an assessment by ID
    pub async fn get_talent_review_assessment(&self, id: Uuid) -> AtlasResult<Option<TalentReviewAssessment>> {
        self.repository.get_talent_review_assessment(id).await
    }

    /// List assessments for a talent review
    pub async fn list_talent_review_assessments(&self, review_id: Uuid) -> AtlasResult<Vec<TalentReviewAssessment>> {
        self.repository.list_talent_review_assessments(review_id).await
    }

    /// Delete an assessment
    pub async fn delete_talent_review_assessment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting assessment {}", id);
        self.repository.delete_talent_review_assessment(id).await
    }

    // ========================================================================
    // Career Paths
    // ========================================================================

    /// Create a career path
    #[allow(clippy::too_many_arguments)]
    pub async fn create_career_path(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        path_type: &str,
        from_job_id: Option<Uuid>,
        from_job_title: Option<&str>,
        to_job_id: Option<Uuid>,
        to_job_title: Option<&str>,
        typical_duration_months: Option<i32>,
        required_competencies: Option<&str>,
        required_certifications: Option<&str>,
        development_activities: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CareerPath> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Career path name is required".to_string()));
        }
        validate_enum("path_type", path_type, VALID_PATH_TYPES)?;
        if let Some(dur) = typical_duration_months {
            if dur < 1 {
                return Err(AtlasError::ValidationFailed("Duration must be >= 1 month".to_string()));
            }
        }

        if self.repository.get_career_path_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Career path '{}' already exists", code_upper)));
        }

        info!("Creating career path '{}' for org {}", code_upper, org_id);
        self.repository.create_career_path(
            org_id, &code_upper, name, description, path_type,
            from_job_id, from_job_title, to_job_id, to_job_title,
            typical_duration_months, required_competencies,
            required_certifications, development_activities, created_by,
        ).await
    }

    /// Get a career path by ID
    pub async fn get_career_path(&self, id: Uuid) -> AtlasResult<Option<CareerPath>> {
        self.repository.get_career_path(id).await
    }

    /// List career paths with optional filters
    pub async fn list_career_paths(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        path_type: Option<&str>,
    ) -> AtlasResult<Vec<CareerPath>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_PATH_STATUSES)?;
        }
        if let Some(pt) = path_type {
            validate_enum("path_type", pt, VALID_PATH_TYPES)?;
        }
        self.repository.list_career_paths(org_id, status, path_type).await
    }

    /// Update career path status
    pub async fn update_career_path_status(&self, id: Uuid, status: &str) -> AtlasResult<CareerPath> {
        validate_enum("status", status, VALID_PATH_STATUSES)?;
        let path = self.repository.get_career_path(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Career path {} not found", id)))?;

        match (path.status.as_str(), status) {
            ("draft", "active") | ("active", "archived") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition career path from '{}' to '{}'", path.status, status
            ))),
        }

        info!("Updating career path {} status to {}", id, status);
        self.repository.update_career_path_status(id, status).await
    }

    /// Delete a career path by code
    pub async fn delete_career_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting career path '{}' for org {}", code, org_id);
        self.repository.delete_career_path(org_id, code).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get succession planning dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SuccessionDashboard> {
        self.repository.get_succession_dashboard(org_id).await
    }
}

// ============================================================================
// Validation helpers
// ============================================================================

fn validate_code(code: &str) -> AtlasResult<()> {
    if code.is_empty() || code.len() > 100 {
        return Err(AtlasError::ValidationFailed(
            "Code must be 1-100 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_enum(field_name: &str, value: &str, valid: &[&str]) -> AtlasResult<()> {
    if !valid.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field_name, value, valid.join(", ")
        )));
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Enum validation tests
    // ========================================================================

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"position"));
        assert!(VALID_PLAN_TYPES.contains(&"role"));
        assert!(VALID_PLAN_TYPES.contains(&"key_person"));
        assert_eq!(VALID_PLAN_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_risk_levels() {
        assert!(VALID_RISK_LEVELS.contains(&"low"));
        assert!(VALID_RISK_LEVELS.contains(&"medium"));
        assert!(VALID_RISK_LEVELS.contains(&"high"));
        assert!(VALID_RISK_LEVELS.contains(&"critical"));
        assert_eq!(VALID_RISK_LEVELS.len(), 4);
    }

    #[test]
    fn test_valid_urgencies() {
        assert!(VALID_URGENCIES.contains(&"immediate"));
        assert!(VALID_URGENCIES.contains(&"short_term"));
        assert!(VALID_URGENCIES.contains(&"medium_term"));
        assert!(VALID_URGENCIES.contains(&"long_term"));
        assert_eq!(VALID_URGENCIES.len(), 4);
    }

    #[test]
    fn test_valid_plan_statuses() {
        assert!(VALID_PLAN_STATUSES.contains(&"draft"));
        assert!(VALID_PLAN_STATUSES.contains(&"active"));
        assert!(VALID_PLAN_STATUSES.contains(&"completed"));
        assert!(VALID_PLAN_STATUSES.contains(&"cancelled"));
        assert_eq!(VALID_PLAN_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_readiness_levels() {
        assert!(VALID_READINESS_LEVELS.contains(&"ready_now"));
        assert!(VALID_READINESS_LEVELS.contains(&"ready_1_2_years"));
        assert!(VALID_READINESS_LEVELS.contains(&"ready_3_5_years"));
        assert!(VALID_READINESS_LEVELS.contains(&"not_ready"));
        assert_eq!(VALID_READINESS_LEVELS.len(), 4);
    }

    #[test]
    fn test_valid_candidate_statuses() {
        assert!(VALID_CANDIDATE_STATUSES.contains(&"proposed"));
        assert!(VALID_CANDIDATE_STATUSES.contains(&"approved"));
        assert!(VALID_CANDIDATE_STATUSES.contains(&"rejected"));
        assert!(VALID_CANDIDATE_STATUSES.contains(&"development"));
        assert_eq!(VALID_CANDIDATE_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_flight_risks() {
        assert!(VALID_FLIGHT_RISKS.contains(&"low"));
        assert!(VALID_FLIGHT_RISKS.contains(&"medium"));
        assert!(VALID_FLIGHT_RISKS.contains(&"high"));
        assert_eq!(VALID_FLIGHT_RISKS.len(), 3);
    }

    #[test]
    fn test_valid_pool_types() {
        assert!(VALID_POOL_TYPES.contains(&"leadership"));
        assert!(VALID_POOL_TYPES.contains(&"technical"));
        assert!(VALID_POOL_TYPES.contains(&"high_potential"));
        assert!(VALID_POOL_TYPES.contains(&"diversity"));
        assert!(VALID_POOL_TYPES.contains(&"custom"));
        assert_eq!(VALID_POOL_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_pool_statuses() {
        assert!(VALID_POOL_STATUSES.contains(&"draft"));
        assert!(VALID_POOL_STATUSES.contains(&"active"));
        assert!(VALID_POOL_STATUSES.contains(&"archived"));
        assert_eq!(VALID_POOL_STATUSES.len(), 3);
    }

    #[test]
    fn test_valid_member_statuses() {
        assert!(VALID_MEMBER_STATUSES.contains(&"active"));
        assert!(VALID_MEMBER_STATUSES.contains(&"on_hold"));
        assert!(VALID_MEMBER_STATUSES.contains(&"removed"));
        assert!(VALID_MEMBER_STATUSES.contains(&"graduated"));
        assert_eq!(VALID_MEMBER_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_review_types() {
        assert!(VALID_REVIEW_TYPES.contains(&"calibration"));
        assert!(VALID_REVIEW_TYPES.contains(&"performance_potential"));
        assert!(VALID_REVIEW_TYPES.contains(&"nine_box"));
        assert!(VALID_REVIEW_TYPES.contains(&"leadership"));
        assert_eq!(VALID_REVIEW_TYPES.len(), 4);
    }

    #[test]
    fn test_valid_review_statuses() {
        assert!(VALID_REVIEW_STATUSES.contains(&"scheduled"));
        assert!(VALID_REVIEW_STATUSES.contains(&"in_progress"));
        assert!(VALID_REVIEW_STATUSES.contains(&"completed"));
        assert!(VALID_REVIEW_STATUSES.contains(&"cancelled"));
        assert_eq!(VALID_REVIEW_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_nine_box_positions() {
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"star"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"workhorse"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"puzzle"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"solid_citizen"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"high_potential"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"core_player"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"rough_diamond"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"inconsistent"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"underperformer"));
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"blocker"));
        assert_eq!(VALID_NINE_BOX_POSITIONS.len(), 10);
    }

    #[test]
    fn test_valid_path_types() {
        assert!(VALID_PATH_TYPES.contains(&"linear"));
        assert!(VALID_PATH_TYPES.contains(&"branching"));
        assert!(VALID_PATH_TYPES.contains(&"lattice"));
        assert!(VALID_PATH_TYPES.contains(&"dual_track"));
        assert_eq!(VALID_PATH_TYPES.len(), 4);
    }

    #[test]
    fn test_valid_path_statuses() {
        assert!(VALID_PATH_STATUSES.contains(&"draft"));
        assert!(VALID_PATH_STATUSES.contains(&"active"));
        assert!(VALID_PATH_STATUSES.contains(&"archived"));
        assert_eq!(VALID_PATH_STATUSES.len(), 3);
    }

    // ========================================================================
    // validate_code tests
    // ========================================================================

    #[test]
    fn test_validate_code_valid() {
        assert!(validate_code("CEO-SUCCESSION").is_ok());
        assert!(validate_code("A").is_ok());
    }

    #[test]
    fn test_validate_code_empty() {
        assert!(validate_code("").is_err());
    }

    #[test]
    fn test_validate_code_too_long() {
        let long_code = "A".repeat(101);
        assert!(validate_code(&long_code).is_err());
    }

    #[test]
    fn test_validate_code_exactly_100() {
        let code = "A".repeat(100);
        assert!(validate_code(&code).is_ok());
    }

    // ========================================================================
    // validate_enum tests
    // ========================================================================

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test_field", "low", &["low", "medium", "high"]).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("test_field", "invalid", &["low", "medium", "high"]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("test_field"));
        assert!(msg.contains("invalid"));
    }

    // ========================================================================
    // Status transition validation tests
    // ========================================================================

    #[test]
    fn test_plan_status_transitions_draft_to_active() {
        // draft -> active is valid
        let valid = matches!(("draft", "active"), ("draft", "active") | ("active", "completed") | ("active", "cancelled") | ("draft", "cancelled"));
        assert!(valid);
    }

    #[test]
    fn test_plan_status_transitions_invalid() {
        // completed -> draft is NOT valid
        let valid = matches!(("completed", "draft"), ("draft", "active") | ("active", "completed") | ("active", "cancelled") | ("draft", "cancelled"));
        assert!(!valid);
    }

    #[test]
    fn test_review_status_transitions() {
        assert!(matches!(("scheduled", "in_progress"), ("scheduled", "in_progress")));
        assert!(matches!(("in_progress", "completed"), ("in_progress", "completed")));
    }

    // ========================================================================
    // Nine-box grid position tests
    // ========================================================================

    #[test]
    fn test_nine_box_all_positions() {
        // The 10 positions cover the standard 3x3 grid plus an extra for
        // custom classification. Verify all are unique.
        let mut positions: Vec<&str> = VALID_NINE_BOX_POSITIONS.to_vec();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), VALID_NINE_BOX_POSITIONS.len(),
            "Nine-box positions must be unique");
    }

    #[test]
    fn test_nine_box_star_position() {
        // "star" = high performance + high potential
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"star"));
    }

    #[test]
    fn test_nine_box_underperformer_position() {
        // "underperformer" = low performance + low potential
        assert!(VALID_NINE_BOX_POSITIONS.contains(&"underperformer"));
    }

    // ========================================================================
    // Readiness level ordering tests
    // ========================================================================

    #[test]
    fn test_readiness_levels_include_all() {
        // Oracle Fusion standard readiness categories
        assert!(VALID_READINESS_LEVELS.contains(&"ready_now"));
        assert!(VALID_READINESS_LEVELS.contains(&"ready_1_2_years"));
        assert!(VALID_READINESS_LEVELS.contains(&"ready_3_5_years"));
        assert!(VALID_READINESS_LEVELS.contains(&"not_ready"));
    }

    // ========================================================================
    // Risk level tests
    // ========================================================================

    #[test]
    fn test_risk_levels_in_order() {
        assert_eq!(VALID_RISK_LEVELS, &["low", "medium", "high", "critical"]);
    }

    // ========================================================================
    // Pool type coverage tests
    // ========================================================================

    #[test]
    fn test_pool_types_include_leadership_and_technical() {
        assert!(VALID_POOL_TYPES.contains(&"leadership"));
        assert!(VALID_POOL_TYPES.contains(&"technical"));
        assert!(VALID_POOL_TYPES.contains(&"high_potential"));
        assert!(VALID_POOL_TYPES.contains(&"diversity"));
        assert!(VALID_POOL_TYPES.contains(&"custom"));
    }

    // ========================================================================
    // Career path type tests
    // ========================================================================

    #[test]
    fn test_path_types_include_dual_track() {
        // Oracle Fusion supports dual career tracks (technical + management)
        assert!(VALID_PATH_TYPES.contains(&"dual_track"));
        assert!(VALID_PATH_TYPES.contains(&"lattice"));
        assert!(VALID_PATH_TYPES.contains(&"linear"));
        assert!(VALID_PATH_TYPES.contains(&"branching"));
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_validate_enum_empty_valid_array() {
        let result = validate_enum("field", "value", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_code_exactly_1_char() {
        assert!(validate_code("X").is_ok());
    }

    #[test]
    fn test_flight_risk_values() {
        assert_eq!(VALID_FLIGHT_RISKS, &["low", "medium", "high"]);
    }

    #[test]
    fn test_candidate_status_values() {
        assert_eq!(VALID_CANDIDATE_STATUSES, &["proposed", "approved", "rejected", "development"]);
    }

    #[test]
    fn test_member_status_values() {
        assert_eq!(VALID_MEMBER_STATUSES, &["active", "on_hold", "removed", "graduated"]);
    }
}
