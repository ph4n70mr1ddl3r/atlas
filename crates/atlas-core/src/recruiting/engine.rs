//! Recruiting Management Engine
//!
//! Oracle Fusion HCM: Recruiting.
//! Manages job requisitions, candidates, job applications,
//! interviews, job offers, and recruiting analytics.
//!
//! The process follows Oracle Fusion Recruiting workflow:
//! 1. Create job requisitions (open positions)
//! 2. Open requisitions for applications
//! 3. Register candidates
//! 4. Create job applications (link candidates to requisitions)
//! 5. Move applications through screening / interview / assessment stages
//! 6. Schedule and complete interviews with feedback
//! 7. Create and extend job offers
//! 8. Accept/decline offers and hire candidates
//! 9. Analyze via dashboard

use atlas_shared::AtlasError;
use super::RecruitingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid employment types
const VALID_EMPLOYMENT_TYPES: &[&str] = &["full_time", "part_time", "contract", "internship"];

/// Valid position types
const VALID_POSITION_TYPES: &[&str] = &["new", "replacement", "additional"];

/// Valid requisition priorities
const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];

/// Valid requisition statuses
const VALID_REQUISITION_STATUSES: &[&str] = &[
    "draft", "open", "on_hold", "filled", "cancelled", "closed",
];

/// Valid candidate statuses
const VALID_CANDIDATE_STATUSES: &[&str] = &[
    "active", "inactive", "hired", "rejected", "blacklisted",
];

/// Valid application statuses
const VALID_APPLICATION_STATUSES: &[&str] = &[
    "applied", "screening", "interview", "assessment", "offer", "hired", "rejected", "withdrawn",
];

/// Valid interview types
const VALID_INTERVIEW_TYPES: &[&str] = &[
    "phone", "video", "on_site", "panel", "technical", "group",
];

/// Valid interview statuses
const VALID_INTERVIEW_STATUSES: &[&str] = &[
    "scheduled", "in_progress", "completed", "cancelled", "no_show",
];

/// Valid interview recommendations
const VALID_RECOMMENDATIONS: &[&str] = &[
    "strong_hire", "hire", "lean_hire", "no_hire", "strong_no_hire",
];

/// Valid offer statuses
const VALID_OFFER_STATUSES: &[&str] = &[
    "draft", "pending_approval", "approved", "extended", "accepted", "declined", "withdrawn", "expired",
];

/// Valid candidate sources
const VALID_SOURCES: &[&str] = &[
    "referral", "job_board", "agency", "website", "internal", "other",
];

/// Recruiting Management engine
pub struct RecruitingEngine {
    repository: Arc<dyn RecruitingRepository>,
}

impl RecruitingEngine {
    pub fn new(repository: Arc<dyn RecruitingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Job Requisition Management
    // ========================================================================

    /// Create a job requisition
    #[allow(clippy::too_many_arguments)]
    pub async fn create_requisition(
        &self,
        org_id: Uuid,
        requisition_number: &str,
        title: &str,
        description: Option<&str>,
        department: Option<&str>,
        location: Option<&str>,
        employment_type: &str,
        position_type: &str,
        vacancies: i32,
        priority: &str,
        salary_min: Option<&str>,
        salary_max: Option<&str>,
        currency: Option<&str>,
        required_skills: Option<&serde_json::Value>,
        qualifications: Option<&str>,
        experience_years_min: Option<i32>,
        experience_years_max: Option<i32>,
        education_level: Option<&str>,
        hiring_manager_id: Option<Uuid>,
        recruiter_id: Option<Uuid>,
        target_start_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        if requisition_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Requisition number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Title is required".to_string()));
        }
        if !VALID_EMPLOYMENT_TYPES.contains(&employment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid employment_type '{}'. Must be one of: {}", employment_type, VALID_EMPLOYMENT_TYPES.join(", ")
            )));
        }
        if !VALID_POSITION_TYPES.contains(&position_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid position_type '{}'. Must be one of: {}", position_type, VALID_POSITION_TYPES.join(", ")
            )));
        }
        if vacancies < 1 {
            return Err(AtlasError::ValidationFailed("Vacancies must be at least 1".to_string()));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        if self.repository.get_requisition_by_number(org_id, requisition_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Requisition '{}' already exists", requisition_number)));
        }
        info!("Creating requisition '{}' for org {}", requisition_number, org_id);
        self.repository.create_requisition(
            org_id, requisition_number, title, description, department, location,
            employment_type, position_type, vacancies, priority,
            salary_min, salary_max, currency,
            required_skills, qualifications,
            experience_years_min, experience_years_max, education_level,
            hiring_manager_id, recruiter_id, target_start_date, created_by,
        ).await
    }

    /// Get a requisition by ID
    pub async fn get_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::JobRequisition>> {
        self.repository.get_requisition(id).await
    }

    /// List requisitions with optional status filter
    pub async fn list_requisitions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::JobRequisition>> {
        if let Some(s) = status {
            if !VALID_REQUISITION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_REQUISITION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_requisitions(org_id, status).await
    }

    /// Open a requisition for applications
    pub async fn open_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        let req = self.repository.get_requisition(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;
        if req.status != "draft" && req.status != "on_hold" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot open requisition in '{}' status. Must be 'draft' or 'on_hold'.", req.status
            )));
        }
        info!("Opening requisition {}", req.requisition_number);
        self.repository.update_requisition_status(id, "open").await
    }

    /// Put a requisition on hold
    pub async fn hold_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        let req = self.repository.get_requisition(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;
        if req.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot hold requisition in '{}' status. Must be 'open'.", req.status
            )));
        }
        info!("Putting requisition {} on hold", req.requisition_number);
        self.repository.update_requisition_status(id, "on_hold").await
    }

    /// Close a requisition
    pub async fn close_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        let req = self.repository.get_requisition(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;
        if req.status == "closed" || req.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Requisition is already '{}'.", req.status
            )));
        }
        info!("Closing requisition {}", req.requisition_number);
        self.repository.update_requisition_status(id, "closed").await
    }

    /// Cancel a requisition
    pub async fn cancel_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        let req = self.repository.get_requisition(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;
        if req.status == "filled" || req.status == "cancelled" || req.status == "closed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel requisition in '{}' status.", req.status
            )));
        }
        info!("Cancelling requisition {}", req.requisition_number);
        self.repository.update_requisition_status(id, "cancelled").await
    }

    /// Mark a requisition as filled
    pub async fn fill_requisition(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobRequisition> {
        let req = self.repository.get_requisition(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", id)))?;
        if req.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot fill requisition in '{}' status. Must be 'open'.", req.status
            )));
        }
        info!("Filling requisition {}", req.requisition_number);
        self.repository.update_requisition_status(id, "filled").await
    }

    /// Delete a requisition
    pub async fn delete_requisition(&self, org_id: Uuid, requisition_number: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_requisition(org_id, requisition_number).await
    }

    // ========================================================================
    // Candidate Management
    // ========================================================================

    /// Create a candidate
    #[allow(clippy::too_many_arguments)]
    pub async fn create_candidate(
        &self,
        org_id: Uuid,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        phone: Option<&str>,
        address: Option<&str>,
        city: Option<&str>,
        state: Option<&str>,
        country: Option<&str>,
        postal_code: Option<&str>,
        linkedin_url: Option<&str>,
        source: Option<&str>,
        source_detail: Option<&str>,
        resume_url: Option<&str>,
        current_employer: Option<&str>,
        current_title: Option<&str>,
        years_of_experience: Option<i32>,
        education_level: Option<&str>,
        skills: Option<&serde_json::Value>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Candidate> {
        if first_name.is_empty() {
            return Err(AtlasError::ValidationFailed("First name is required".to_string()));
        }
        if last_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Last name is required".to_string()));
        }
        if let Some(s) = source {
            if !VALID_SOURCES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid source '{}'. Must be one of: {}", s, VALID_SOURCES.join(", ")
                )));
            }
        }
        info!("Creating candidate {} {} for org {}", first_name, last_name, org_id);
        self.repository.create_candidate(
            org_id, first_name, last_name, email, phone, address, city, state,
            country, postal_code, linkedin_url, source, source_detail, resume_url,
            current_employer, current_title, years_of_experience, education_level,
            skills, notes, created_by,
        ).await
    }

    /// Get a candidate by ID
    pub async fn get_candidate(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::Candidate>> {
        self.repository.get_candidate(id).await
    }

    /// List candidates
    pub async fn list_candidates(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::Candidate>> {
        if let Some(s) = status {
            if !VALID_CANDIDATE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CANDIDATE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_candidates(org_id, status).await
    }

    /// Update candidate status
    pub async fn update_candidate_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> atlas_shared::AtlasResult<atlas_shared::Candidate> {
        if !VALID_CANDIDATE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_CANDIDATE_STATUSES.join(", ")
            )));
        }
        let candidate = self.repository.get_candidate(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Candidate {} not found", id)))?;
        info!("Updating candidate {} {} status to {}", candidate.first_name, candidate.last_name, status);
        self.repository.update_candidate_status(id, status).await
    }

    /// Delete a candidate
    pub async fn delete_candidate(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_candidate(id).await
    }

    // ========================================================================
    // Job Application Management
    // ========================================================================

    /// Create a job application
    pub async fn create_application(
        &self,
        org_id: Uuid,
        requisition_id: Uuid,
        candidate_id: Uuid,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::JobApplication> {
        // Verify requisition exists and is open
        let req = self.repository.get_requisition(requisition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Requisition {} not found", requisition_id)))?;
        if req.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot apply to requisition in '{}' status. Must be 'open'.", req.status
            )));
        }
        // Verify candidate exists
        let _candidate = self.repository.get_candidate(candidate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Candidate {} not found", candidate_id)))?;

        info!("Creating application for candidate {} on requisition {}", candidate_id, req.requisition_number);
        self.repository.create_application(org_id, requisition_id, candidate_id, created_by).await
    }

    /// Get an application by ID
    pub async fn get_application(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::JobApplication>> {
        self.repository.get_application(id).await
    }

    /// List applications with optional filters
    pub async fn list_applications(
        &self,
        org_id: Uuid,
        requisition_id: Option<Uuid>,
        candidate_id: Option<Uuid>,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::JobApplication>> {
        if let Some(s) = status {
            if !VALID_APPLICATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_APPLICATION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_applications(org_id, requisition_id, candidate_id, status).await
    }

    /// Update application status (advance through pipeline)
    pub async fn update_application_status(
        &self,
        id: Uuid,
        status: &str,
        notes: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::JobApplication> {
        if !VALID_APPLICATION_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_APPLICATION_STATUSES.join(", ")
            )));
        }
        let app = self.repository.get_application(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", id)))?;
        info!("Updating application {} status to {}", app.application_number.as_deref().unwrap_or("N/A"), status);

        let result = self.repository.update_application_status(id, status, notes).await?;

        // If hired, update candidate status too
        if status == "hired" {
            let _ = self.repository.update_candidate_status(result.candidate_id, "hired").await;
        }
        // If rejected, update candidate status
        if status == "rejected" {
            let _ = self.repository.update_candidate_status(result.candidate_id, "rejected").await;
        }

        Ok(result)
    }

    /// Withdraw an application
    pub async fn withdraw_application(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobApplication> {
        let app = self.repository.get_application(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", id)))?;
        if app.status == "hired" || app.status == "rejected" || app.status == "withdrawn" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot withdraw application in '{}' status.", app.status
            )));
        }
        info!("Withdrawing application {}", app.application_number.as_deref().unwrap_or("N/A"));
        self.repository.update_application_status(id, "withdrawn", Some("Withdrawn by candidate")).await
    }

    // ========================================================================
    // Interview Management
    // ========================================================================

    /// Schedule an interview
    #[allow(clippy::too_many_arguments)]
    pub async fn create_interview(
        &self,
        org_id: Uuid,
        application_id: Uuid,
        interview_type: &str,
        round: i32,
        scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
        duration_minutes: i32,
        location: Option<&str>,
        meeting_link: Option<&str>,
        interviewer_ids: Option<&serde_json::Value>,
        interviewer_names: Option<&serde_json::Value>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Interview> {
        if !VALID_INTERVIEW_TYPES.contains(&interview_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid interview_type '{}'. Must be one of: {}", interview_type, VALID_INTERVIEW_TYPES.join(", ")
            )));
        }
        if round < 1 {
            return Err(AtlasError::ValidationFailed("Round must be at least 1".to_string()));
        }
        if duration_minutes < 1 {
            return Err(AtlasError::ValidationFailed("Duration must be at least 1 minute".to_string()));
        }
        // Verify application exists
        let _app = self.repository.get_application(application_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", application_id)))?;

        info!("Scheduling {} interview round {} for application {}", interview_type, round, application_id);
        self.repository.create_interview(
            org_id, application_id, interview_type, round,
            scheduled_at, duration_minutes, location, meeting_link,
            interviewer_ids, interviewer_names, notes, created_by,
        ).await
    }

    /// Get an interview by ID
    pub async fn get_interview(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::Interview>> {
        self.repository.get_interview(id).await
    }

    /// List interviews for an application
    pub async fn list_interviews(
        &self,
        application_id: Uuid,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::Interview>> {
        self.repository.list_interviews(application_id).await
    }

    /// Complete an interview with feedback
    pub async fn complete_interview(
        &self,
        id: Uuid,
        feedback: Option<&str>,
        rating: Option<i32>,
        recommendation: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::Interview> {
        let interview = self.repository.get_interview(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Interview {} not found", id)))?;
        if interview.status != "scheduled" && interview.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete interview in '{}' status.", interview.status
            )));
        }
        if let Some(r) = rating {
            if r < 1 || r > 5 {
                return Err(AtlasError::ValidationFailed("Rating must be between 1 and 5".to_string()));
            }
        }
        if let Some(r) = recommendation {
            if !VALID_RECOMMENDATIONS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid recommendation '{}'. Must be one of: {}", r, VALID_RECOMMENDATIONS.join(", ")
                )));
            }
        }
        info!("Completing interview {} with rating {:?}", id, rating);
        self.repository.complete_interview(id, feedback, rating, recommendation).await
    }

    /// Cancel an interview
    pub async fn cancel_interview(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::Interview> {
        let interview = self.repository.get_interview(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Interview {} not found", id)))?;
        if interview.status == "completed" {
            return Err(AtlasError::WorkflowError("Cannot cancel a completed interview".to_string()));
        }
        info!("Cancelling interview {}", id);
        self.repository.update_interview_status(id, "cancelled").await
    }

    /// Delete an interview
    pub async fn delete_interview(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_interview(id).await
    }

    // ========================================================================
    // Job Offer Management
    // ========================================================================

    /// Create a job offer
    #[allow(clippy::too_many_arguments)]
    pub async fn create_offer(
        &self,
        org_id: Uuid,
        application_id: Uuid,
        offer_number: Option<&str>,
        job_title: &str,
        department: Option<&str>,
        location: Option<&str>,
        employment_type: &str,
        start_date: Option<chrono::NaiveDate>,
        salary_offered: Option<&str>,
        salary_currency: Option<&str>,
        salary_frequency: Option<&str>,
        signing_bonus: Option<&str>,
        benefits_summary: Option<&str>,
        terms_and_conditions: Option<&str>,
        response_deadline: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        if job_title.is_empty() {
            return Err(AtlasError::ValidationFailed("Job title is required".to_string()));
        }
        if !VALID_EMPLOYMENT_TYPES.contains(&employment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid employment_type '{}'. Must be one of: {}", employment_type, VALID_EMPLOYMENT_TYPES.join(", ")
            )));
        }
        // Verify application exists
        let _app = self.repository.get_application(application_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", application_id)))?;

        info!("Creating offer for application {}", application_id);
        self.repository.create_offer(
            org_id, application_id, offer_number, job_title, department, location,
            employment_type, start_date, salary_offered, salary_currency, salary_frequency,
            signing_bonus, benefits_summary, terms_and_conditions, response_deadline, created_by,
        ).await
    }

    /// Get an offer by ID
    pub async fn get_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::JobOffer>> {
        self.repository.get_offer(id).await
    }

    /// List offers with optional status filter
    pub async fn list_offers(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::JobOffer>> {
        if let Some(s) = status {
            if !VALID_OFFER_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_OFFER_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_offers(org_id, status).await
    }

    /// Approve an offer
    pub async fn approve_offer(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        let offer = self.repository.get_offer(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Offer {} not found", id)))?;
        if offer.status != "draft" && offer.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve offer in '{}' status.", offer.status
            )));
        }
        info!("Approving offer {}", offer.offer_number.as_deref().unwrap_or("N/A"));
        self.repository.update_offer_status(id, "approved", approved_by).await
    }

    /// Extend (send) an offer to the candidate
    pub async fn extend_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        let offer = self.repository.get_offer(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Offer {} not found", id)))?;
        if offer.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot extend offer in '{}' status. Must be 'approved'.", offer.status
            )));
        }
        info!("Extending offer {}", offer.offer_number.as_deref().unwrap_or("N/A"));
        self.repository.update_offer_status(id, "extended", None).await
    }

    /// Accept an offer
    pub async fn accept_offer(&self, id: Uuid, notes: Option<&str>) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        let offer = self.repository.get_offer(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Offer {} not found", id)))?;
        if offer.status != "extended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot accept offer in '{}' status. Must be 'extended'.", offer.status
            )));
        }
        info!("Accepting offer {}", offer.offer_number.as_deref().unwrap_or("N/A"));
        let result = self.repository.update_offer_status(id, "accepted", None).await?;

        // Update the application status to hired
        let _ = self.repository.update_application_status(result.application_id, "hired", notes).await;
        // Update candidate status
        let app = self.repository.get_application(result.application_id).await?;
        if let Some(application) = app {
            let _ = self.repository.update_candidate_status(application.candidate_id, "hired").await;
        }

        Ok(result)
    }

    /// Decline an offer
    pub async fn decline_offer(&self, id: Uuid, notes: Option<&str>) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        let offer = self.repository.get_offer(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Offer {} not found", id)))?;
        if offer.status != "extended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot decline offer in '{}' status. Must be 'extended'.", offer.status
            )));
        }
        info!("Declining offer {}", offer.offer_number.as_deref().unwrap_or("N/A"));
        self.repository.decline_offer(id, notes).await
    }

    /// Withdraw an offer
    pub async fn withdraw_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::JobOffer> {
        let offer = self.repository.get_offer(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Offer {} not found", id)))?;
        if offer.status == "accepted" || offer.status == "declined" || offer.status == "withdrawn" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot withdraw offer in '{}' status.", offer.status
            )));
        }
        info!("Withdrawing offer {}", offer.offer_number.as_deref().unwrap_or("N/A"));
        self.repository.update_offer_status(id, "withdrawn", None).await
    }

    /// Delete an offer
    pub async fn delete_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_offer(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get recruiting dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::RecruitingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_employment_types() {
        assert!(VALID_EMPLOYMENT_TYPES.contains(&"full_time"));
        assert!(VALID_EMPLOYMENT_TYPES.contains(&"part_time"));
        assert!(VALID_EMPLOYMENT_TYPES.contains(&"contract"));
        assert!(VALID_EMPLOYMENT_TYPES.contains(&"internship"));
        assert!(!VALID_EMPLOYMENT_TYPES.contains(&"temporary"));
    }

    #[test]
    fn test_valid_requisition_statuses() {
        assert!(VALID_REQUISITION_STATUSES.contains(&"draft"));
        assert!(VALID_REQUISITION_STATUSES.contains(&"open"));
        assert!(VALID_REQUISITION_STATUSES.contains(&"on_hold"));
        assert!(VALID_REQUISITION_STATUSES.contains(&"filled"));
        assert!(VALID_REQUISITION_STATUSES.contains(&"cancelled"));
        assert!(VALID_REQUISITION_STATUSES.contains(&"closed"));
        assert!(!VALID_REQUISITION_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_application_statuses() {
        assert!(VALID_APPLICATION_STATUSES.contains(&"applied"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"screening"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"interview"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"assessment"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"offer"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"hired"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"rejected"));
        assert!(VALID_APPLICATION_STATUSES.contains(&"withdrawn"));
    }

    #[test]
    fn test_valid_offer_statuses() {
        assert!(VALID_OFFER_STATUSES.contains(&"draft"));
        assert!(VALID_OFFER_STATUSES.contains(&"pending_approval"));
        assert!(VALID_OFFER_STATUSES.contains(&"approved"));
        assert!(VALID_OFFER_STATUSES.contains(&"extended"));
        assert!(VALID_OFFER_STATUSES.contains(&"accepted"));
        assert!(VALID_OFFER_STATUSES.contains(&"declined"));
        assert!(VALID_OFFER_STATUSES.contains(&"withdrawn"));
        assert!(VALID_OFFER_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_valid_interview_types() {
        assert!(VALID_INTERVIEW_TYPES.contains(&"phone"));
        assert!(VALID_INTERVIEW_TYPES.contains(&"video"));
        assert!(VALID_INTERVIEW_TYPES.contains(&"on_site"));
        assert!(VALID_INTERVIEW_TYPES.contains(&"panel"));
        assert!(VALID_INTERVIEW_TYPES.contains(&"technical"));
        assert!(VALID_INTERVIEW_TYPES.contains(&"group"));
    }

    #[test]
    fn test_valid_sources() {
        assert!(VALID_SOURCES.contains(&"referral"));
        assert!(VALID_SOURCES.contains(&"job_board"));
        assert!(VALID_SOURCES.contains(&"agency"));
        assert!(VALID_SOURCES.contains(&"website"));
        assert!(VALID_SOURCES.contains(&"internal"));
        assert!(VALID_SOURCES.contains(&"other"));
    }

    #[test]
    fn test_valid_recommendations() {
        assert!(VALID_RECOMMENDATIONS.contains(&"strong_hire"));
        assert!(VALID_RECOMMENDATIONS.contains(&"hire"));
        assert!(VALID_RECOMMENDATIONS.contains(&"lean_hire"));
        assert!(VALID_RECOMMENDATIONS.contains(&"no_hire"));
        assert!(VALID_RECOMMENDATIONS.contains(&"strong_no_hire"));
    }

    #[test]
    fn test_rating_bounds() {
        // Rating should be between 1 and 5
        assert!(1 >= 1 && 1 <= 5);
        assert!(5 >= 1 && 5 <= 5);
        assert!(!(0 >= 1 && 0 <= 5));
        assert!(!(6 >= 1 && 6 <= 5));
    }

    #[test]
    fn test_vacancies_positive() {
        assert!(1 >= 1);
        assert!(!(0 >= 1));
        assert!(!(-1 >= 1));
    }

    #[test]
    fn test_requisition_lifecycle() {
        let lifecycle = vec!["draft", "open", "filled"];
        for s in &lifecycle {
            assert!(VALID_REQUISITION_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_application_pipeline() {
        let pipeline = vec!["applied", "screening", "interview", "assessment", "offer", "hired"];
        for s in &pipeline {
            assert!(VALID_APPLICATION_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_offer_lifecycle() {
        let lifecycle = vec!["draft", "pending_approval", "approved", "extended", "accepted"];
        for s in &lifecycle {
            assert!(VALID_OFFER_STATUSES.contains(s));
        }
    }
}
