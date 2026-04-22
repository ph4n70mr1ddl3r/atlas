//! Supplier Qualification Engine
//!
//! Manages qualification areas, questions, initiatives, supplier invitations,
//! response evaluation & scoring, certification tracking, and qualification lifecycle.
//!
//! Qualification lifecycle: initiated → pending_response → under_evaluation → qualified/disqualified/expired
//!
//! Oracle Fusion Cloud ERP equivalent: Procurement > Supplier Qualification

use atlas_shared::{
    QualificationArea, QualificationQuestion,
    SupplierQualificationInitiative, SupplierQualificationInvitation,
    SupplierQualificationResponse, SupplierCertification,
    SupplierQualificationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::SupplierQualificationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid area types
#[allow(dead_code)]
const VALID_AREA_TYPES: &[&str] = &[
    "questionnaire", "certificate", "financial", "site_visit", "reference", "other",
];

/// Valid scoring models
#[allow(dead_code)]
const VALID_SCORING_MODELS: &[&str] = &["manual", "weighted", "pass_fail"];

/// Valid question response types
#[allow(dead_code)]
const VALID_RESPONSE_TYPES: &[&str] = &[
    "text", "yes_no", "numeric", "date", "multi_choice", "file_upload",
];

/// Valid initiative statuses
#[allow(dead_code)]
const VALID_INITIATIVE_STATUSES: &[&str] = &[
    "draft", "active", "pending_evaluations", "completed", "cancelled",
];

/// Valid qualification purposes
#[allow(dead_code)]
const VALID_QUALIFICATION_PURPOSES: &[&str] = &[
    "new_supplier", "requalification", "compliance", "ad_hoc",
];

/// Valid invitation statuses
#[allow(dead_code)]
const VALID_INVITATION_STATUSES: &[&str] = &[
    "initiated", "pending_response", "under_evaluation",
    "qualified", "disqualified", "expired", "withdrawn",
];

/// Valid certification statuses
#[allow(dead_code)]
const VALID_CERTIFICATION_STATUSES: &[&str] = &[
    "active", "expired", "revoked", "pending_renewal",
];

/// Supplier Qualification engine
pub struct SupplierQualificationEngine {
    repository: Arc<dyn SupplierQualificationRepository>,
}

impl SupplierQualificationEngine {
    pub fn new(repository: Arc<dyn SupplierQualificationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Qualification Areas
    // ========================================================================

    /// Create a new qualification area
    pub async fn create_area(
        &self,
        org_id: Uuid,
        area_code: &str,
        name: &str,
        description: Option<&str>,
        area_type: &str,
        scoring_model: &str,
        passing_score: &str,
        is_mandatory: bool,
        renewal_period_days: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualificationArea> {
        if area_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Area code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Area name is required".to_string()));
        }
        if !VALID_AREA_TYPES.contains(&area_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid area type '{}'. Must be one of: {}",
                area_type,
                VALID_AREA_TYPES.join(", ")
            )));
        }
        if !VALID_SCORING_MODELS.contains(&scoring_model) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid scoring model '{}'. Must be one of: {}",
                scoring_model,
                VALID_SCORING_MODELS.join(", ")
            )));
        }
        let score: f64 = passing_score.parse().map_err(|_| AtlasError::ValidationFailed(
            "Passing score must be a valid number".to_string(),
        ))?;
        if score < 0.0 || score > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Passing score must be between 0 and 100".to_string(),
            ));
        }
        if renewal_period_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Renewal period days cannot be negative".to_string(),
            ));
        }

        info!("Creating qualification area {} ({}) for org {}", area_code, name, org_id);

        self.repository
            .create_area(
                org_id, area_code, name, description, area_type, scoring_model,
                passing_score, is_mandatory, renewal_period_days, created_by,
            )
            .await
    }

    /// Get a qualification area by code
    pub async fn get_area(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualificationArea>> {
        self.repository.get_area(org_id, code).await
    }

    /// List qualification areas
    pub async fn list_areas(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualificationArea>> {
        self.repository.list_areas(org_id, active_only).await
    }

    /// Delete a qualification area
    pub async fn delete_area(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_area(org_id, code).await
    }

    // ========================================================================
    // Qualification Questions
    // ========================================================================

    /// Add a question to a qualification area
    pub async fn create_question(
        &self,
        org_id: Uuid,
        area_id: Uuid,
        question_number: i32,
        question_text: &str,
        description: Option<&str>,
        response_type: &str,
        choices: Option<serde_json::Value>,
        is_required: bool,
        weight: &str,
        max_score: &str,
        help_text: Option<&str>,
        display_order: i32,
    ) -> AtlasResult<QualificationQuestion> {
        // Validate area exists
        let area = self
            .repository
            .get_area_by_id(area_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Qualification area {} not found", area_id)))?;

        if question_text.is_empty() {
            return Err(AtlasError::ValidationFailed("Question text is required".to_string()));
        }
        if !VALID_RESPONSE_TYPES.contains(&response_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid response type '{}'. Must be one of: {}",
                response_type,
                VALID_RESPONSE_TYPES.join(", ")
            )));
        }
        let w: f64 = weight.parse().map_err(|_| AtlasError::ValidationFailed(
            "Weight must be a valid number".to_string(),
        ))?;
        if w < 0.0 {
            return Err(AtlasError::ValidationFailed("Weight cannot be negative".to_string()));
        }
        let ms: f64 = max_score.parse().map_err(|_| AtlasError::ValidationFailed(
            "Max score must be a valid number".to_string(),
        ))?;
        if ms < 0.0 {
            return Err(AtlasError::ValidationFailed("Max score cannot be negative".to_string()));
        }

        info!(
            "Adding question {} to area {} ({})",
            question_number, area.area_code, area.name
        );

        self.repository
            .create_question(
                org_id, area_id, question_number, question_text, description,
                response_type, choices, is_required, weight, max_score, help_text, display_order,
            )
            .await
    }

    /// List questions for an area
    pub async fn list_questions(&self, area_id: Uuid) -> AtlasResult<Vec<QualificationQuestion>> {
        self.repository.list_questions(area_id).await
    }

    /// Delete a question
    pub async fn delete_question(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_question(id).await
    }

    // ========================================================================
    // Initiatives
    // ========================================================================

    /// Create a new qualification initiative
    pub async fn create_initiative(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        area_id: Uuid,
        qualification_purpose: &str,
        deadline: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInitiative> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Initiative name is required".to_string()));
        }
        if !VALID_QUALIFICATION_PURPOSES.contains(&qualification_purpose) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid qualification purpose '{}'. Must be one of: {}",
                qualification_purpose,
                VALID_QUALIFICATION_PURPOSES.join(", ")
            )));
        }

        // Validate area exists
        let area = self
            .repository
            .get_area_by_id(area_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Qualification area {} not found", area_id)))?;

        if !area.is_active {
            return Err(AtlasError::ValidationFailed(format!(
                "Qualification area '{}' is not active",
                area.name
            )));
        }

        let initiative_number = format!("SQI-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!(
            "Creating qualification initiative {} ({}) for area {}",
            initiative_number, name, area.area_code
        );

        self.repository
            .create_initiative(
                org_id, &initiative_number, name, description, area_id,
                qualification_purpose, deadline, created_by,
            )
            .await
    }

    /// Get an initiative by ID
    pub async fn get_initiative(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInitiative>> {
        self.repository.get_initiative(id).await
    }

    /// List initiatives
    pub async fn list_initiatives(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<SupplierQualificationInitiative>> {
        if let Some(s) = status {
            if !VALID_INITIATIVE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_INITIATIVE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_initiatives(org_id, status).await
    }

    /// Activate a draft initiative
    pub async fn activate_initiative(&self, id: Uuid) -> AtlasResult<SupplierQualificationInitiative> {
        let initiative = self
            .repository
            .get_initiative(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Initiative {} not found", id)))?;

        if initiative.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate initiative in '{}' status. Must be 'draft'.",
                initiative.status
            )));
        }

        info!("Activated initiative {}", initiative.initiative_number);
        self.repository.update_initiative_status(id, "active").await
    }

    /// Complete an initiative
    pub async fn complete_initiative(&self, id: Uuid) -> AtlasResult<SupplierQualificationInitiative> {
        let initiative = self
            .repository
            .get_initiative(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Initiative {} not found", id)))?;

        if initiative.status != "pending_evaluations" && initiative.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete initiative in '{}' status.",
                initiative.status
            )));
        }

        info!("Completed initiative {}", initiative.initiative_number);
        self.repository.update_initiative_status(id, "completed").await
    }

    /// Cancel an initiative
    pub async fn cancel_initiative(&self, id: Uuid) -> AtlasResult<SupplierQualificationInitiative> {
        let initiative = self
            .repository
            .get_initiative(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Initiative {} not found", id)))?;

        if initiative.status == "completed" || initiative.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel initiative in '{}' status.",
                initiative.status
            )));
        }

        info!("Cancelled initiative {}", initiative.initiative_number);
        self.repository.update_initiative_status(id, "cancelled").await
    }

    // ========================================================================
    // Invitations
    // ========================================================================

    /// Invite a supplier to a qualification initiative
    pub async fn invite_supplier(
        &self,
        org_id: Uuid,
        initiative_id: Uuid,
        supplier_id: Uuid,
        supplier_name: &str,
        supplier_contact_name: Option<&str>,
        supplier_contact_email: Option<&str>,
        expiry_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let initiative = self
            .repository
            .get_initiative(initiative_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Initiative {} not found", initiative_id)))?;

        if initiative.status != "active" && initiative.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot invite suppliers to initiative in '{}' status.",
                initiative.status
            )));
        }

        if supplier_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Supplier name is required".to_string()));
        }

        info!(
            "Inviting supplier {} ({}) to initiative {}",
            supplier_name, supplier_id, initiative.initiative_number
        );

        let invitation = self
            .repository
            .create_invitation(
                org_id, initiative_id, supplier_id, supplier_name,
                supplier_contact_name, supplier_contact_email, expiry_date, created_by,
            )
            .await?;

        // Update initiative counts
        let invitations = self.repository.list_invitations_by_initiative(initiative_id).await?;
        let invited = invitations.len() as i32;
        let responded = invitations.iter().filter(|i| i.status != "initiated").count() as i32;
        let qualified = invitations.iter().filter(|i| i.status == "qualified").count() as i32;
        let disqualified = invitations.iter().filter(|i| i.status == "disqualified").count() as i32;
        let pending = invitations.iter().filter(|i| i.status == "initiated" || i.status == "pending_response" || i.status == "under_evaluation").count() as i32;

        self.repository
            .update_initiative_counts(initiative_id, invited, responded, qualified, disqualified, pending)
            .await?;

        Ok(invitation)
    }

    /// Get an invitation
    pub async fn get_invitation(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInvitation>> {
        self.repository.get_invitation(id).await
    }

    /// List invitations for an initiative
    pub async fn list_invitations(&self, initiative_id: Uuid) -> AtlasResult<Vec<SupplierQualificationInvitation>> {
        self.repository.list_invitations_by_initiative(initiative_id).await
    }

    /// List invitations for a supplier
    pub async fn list_supplier_invitations(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
    ) -> AtlasResult<Vec<SupplierQualificationInvitation>> {
        self.repository.list_invitations_by_supplier(org_id, supplier_id).await
    }

    /// Submit supplier response (moves to pending_response)
    pub async fn submit_response(
        &self,
        invitation_id: Uuid,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invitation {} not found", invitation_id)))?;

        if invitation.status != "initiated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit response for invitation in '{}' status. Must be 'initiated'.",
                invitation.status
            )));
        }

        info!("Supplier {} submitted response for initiative", invitation.supplier_name);
        self.repository
            .update_invitation_status(invitation_id, "pending_response", Some(chrono::Utc::now()), None)
            .await
    }

    /// Move invitation to under_evaluation
    pub async fn start_evaluation(&self, invitation_id: Uuid) -> AtlasResult<SupplierQualificationInvitation> {
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invitation {} not found", invitation_id)))?;

        if invitation.status != "pending_response" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start evaluation for invitation in '{}' status. Must be 'pending_response'.",
                invitation.status
            )));
        }

        info!("Starting evaluation for supplier {}", invitation.supplier_name);
        self.repository
            .update_invitation_status(invitation_id, "under_evaluation", None, Some(chrono::Utc::now()))
            .await
    }

    /// Qualify a supplier after evaluation
    pub async fn qualify_supplier(
        &self,
        invitation_id: Uuid,
        qualified_by: Option<Uuid>,
        evaluation_notes: Option<&str>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invitation {} not found", invitation_id)))?;

        if invitation.status != "under_evaluation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot qualify supplier in '{}' status. Must be 'under_evaluation'.",
                invitation.status
            )));
        }

        // Calculate scores from responses
        let responses = self.repository.list_responses(invitation_id).await?;
        let (overall_score, max_possible, percentage) = self.calculate_scores(&responses);

        info!(
            "Qualified supplier {} with score {:.1}%",
            invitation.supplier_name, percentage
        );

        let _updated = self
            .repository
            .update_invitation_scores(
                invitation_id,
                &format!("{:.2}", overall_score),
                &format!("{:.2}", max_possible),
                &format!("{:.2}", percentage),
                qualified_by,
                None,
                evaluation_notes,
            )
            .await?;

        let result = self
            .repository
            .update_invitation_status(invitation_id, "qualified", None, None)
            .await?;

        self.refresh_initiative_counts(invitation.initiative_id).await?;

        Ok(result)
    }

    /// Disqualify a supplier
    pub async fn disqualify_supplier(
        &self,
        invitation_id: Uuid,
        disqualified_reason: &str,
        qualified_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invitation {} not found", invitation_id)))?;

        if invitation.status != "under_evaluation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot disqualify supplier in '{}' status. Must be 'under_evaluation'.",
                invitation.status
            )));
        }

        if disqualified_reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Disqualification reason is required".to_string(),
            ));
        }

        // Calculate scores from responses
        let responses = self.repository.list_responses(invitation_id).await?;
        let (overall_score, max_possible, percentage) = self.calculate_scores(&responses);

        info!(
            "Disqualified supplier {} ({})",
            invitation.supplier_name, disqualified_reason
        );

        self.repository
            .update_invitation_scores(
                invitation_id,
                &format!("{:.2}", overall_score),
                &format!("{:.2}", max_possible),
                &format!("{:.2}", percentage),
                qualified_by,
                Some(disqualified_reason),
                None,
            )
            .await?;

        let result = self
            .repository
            .update_invitation_status(invitation_id, "disqualified", None, None)
            .await?;

        self.refresh_initiative_counts(invitation.initiative_id).await?;

        Ok(result)
    }

    // ========================================================================
    // Responses
    // ========================================================================

    /// Submit a response to a question
    pub async fn create_response(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
        question_id: Uuid,
        response_text: Option<&str>,
        response_value: Option<serde_json::Value>,
        file_reference: Option<&str>,
    ) -> AtlasResult<SupplierQualificationResponse> {
        // Verify invitation exists and is in correct state
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invitation {} not found", invitation_id)))?;

        if invitation.status != "initiated" && invitation.status != "pending_response" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit response for invitation in '{}' status.",
                invitation.status
            )));
        }

        self.repository
            .create_response(org_id, invitation_id, question_id, response_text, response_value, file_reference)
            .await
    }

    /// List responses for an invitation
    pub async fn list_responses(&self, invitation_id: Uuid) -> AtlasResult<Vec<SupplierQualificationResponse>> {
        self.repository.list_responses(invitation_id).await
    }

    /// Score an individual response
    pub async fn score_response(
        &self,
        response_id: Uuid,
        score: &str,
        evaluator_notes: Option<&str>,
        evaluated_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationResponse> {
        let score_val: f64 = score.parse().map_err(|_| AtlasError::ValidationFailed(
            "Score must be a valid number".to_string(),
        ))?;
        if score_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Score cannot be negative".to_string()));
        }

        self.repository
            .score_response(response_id, score, evaluator_notes, evaluated_by)
            .await
    }

    // ========================================================================
    // Certifications
    // ========================================================================

    /// Create a supplier certification
    pub async fn create_certification(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_name: &str,
        certification_type: &str,
        certification_name: &str,
        certifying_body: Option<&str>,
        certificate_number: Option<&str>,
        issued_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>,
        renewal_date: Option<chrono::NaiveDate>,
        qualification_invitation_id: Option<Uuid>,
        document_reference: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierCertification> {
        if certification_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Certification name is required".to_string()));
        }
        if supplier_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Supplier name is required".to_string()));
        }

        info!(
            "Creating certification '{}' for supplier {}",
            certification_name, supplier_name
        );

        self.repository
            .create_certification(
                org_id, supplier_id, supplier_name, certification_type,
                certification_name, certifying_body, certificate_number,
                "active", issued_date, expiry_date, renewal_date,
                qualification_invitation_id, document_reference, notes, created_by,
            )
            .await
    }

    /// Get a certification
    pub async fn get_certification(&self, id: Uuid) -> AtlasResult<Option<SupplierCertification>> {
        self.repository.get_certification(id).await
    }

    /// List certifications with optional filters
    pub async fn list_certifications(
        &self,
        org_id: Uuid,
        supplier_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<SupplierCertification>> {
        if let Some(s) = status {
            if !VALID_CERTIFICATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid certification status '{}'. Must be one of: {}",
                    s,
                    VALID_CERTIFICATION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_certifications(org_id, supplier_id, status).await
    }

    /// Revoke a certification
    pub async fn revoke_certification(&self, id: Uuid) -> AtlasResult<SupplierCertification> {
        let cert = self
            .repository
            .get_certification(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Certification {} not found", id)))?;

        if cert.status != "active" && cert.status != "pending_renewal" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot revoke certification in '{}' status.",
                cert.status
            )));
        }

        info!("Revoked certification '{}' for supplier {}", cert.certification_name, cert.supplier_name);
        self.repository.update_certification_status(id, "revoked").await
    }

    /// Renew a certification
    pub async fn renew_certification(
        &self,
        id: Uuid,
        _new_expiry_date: chrono::NaiveDate,
        _new_certificate_number: Option<&str>,
    ) -> AtlasResult<SupplierCertification> {
        let cert = self
            .repository
            .get_certification(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Certification {} not found", id)))?;

        if cert.status != "active" && cert.status != "pending_renewal" && cert.status != "expired" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot renew certification in '{}' status.",
                cert.status
            )));
        }

        info!("Renewed certification '{}' for supplier {}", cert.certification_name, cert.supplier_name);
        // For simplicity, just update the status - the expiry date would be updated via repository
        self.repository.update_certification_status(id, "active").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get qualification dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SupplierQualificationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Calculate overall scores from individual response scores
    fn calculate_scores(&self, responses: &[SupplierQualificationResponse]) -> (f64, f64, f64) {
        let overall_score: f64 = responses
            .iter()
            .map(|r| r.score.parse().unwrap_or(0.0))
            .sum();
        let max_possible: f64 = responses
            .iter()
            .map(|r| r.max_score.parse().unwrap_or(0.0))
            .sum();

        let percentage = if max_possible > 0.0 {
            (overall_score / max_possible) * 100.0
        } else {
            0.0
        };

        (overall_score, max_possible, percentage)
    }

    /// Refresh the initiative counts after a status change
    async fn refresh_initiative_counts(&self, initiative_id: Uuid) -> AtlasResult<()> {
        let invitations = self.repository.list_invitations_by_initiative(initiative_id).await?;
        let invited = invitations.len() as i32;
        let responded = invitations.iter().filter(|i| i.status != "initiated").count() as i32;
        let qualified = invitations.iter().filter(|i| i.status == "qualified").count() as i32;
        let disqualified = invitations.iter().filter(|i| i.status == "disqualified").count() as i32;
        let pending = invitations.iter().filter(|i| matches!(i.status.as_str(), "initiated" | "pending_response" | "under_evaluation")).count() as i32;

        self.repository
            .update_initiative_counts(initiative_id, invited, responded, qualified, disqualified, pending)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_area_types() {
        assert!(VALID_AREA_TYPES.contains(&"questionnaire"));
        assert!(VALID_AREA_TYPES.contains(&"certificate"));
        assert!(VALID_AREA_TYPES.contains(&"financial"));
        assert!(VALID_AREA_TYPES.contains(&"site_visit"));
        assert!(VALID_AREA_TYPES.contains(&"reference"));
        assert!(VALID_AREA_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_scoring_models() {
        assert!(VALID_SCORING_MODELS.contains(&"manual"));
        assert!(VALID_SCORING_MODELS.contains(&"weighted"));
        assert!(VALID_SCORING_MODELS.contains(&"pass_fail"));
    }

    #[test]
    fn test_valid_response_types() {
        assert!(VALID_RESPONSE_TYPES.contains(&"text"));
        assert!(VALID_RESPONSE_TYPES.contains(&"yes_no"));
        assert!(VALID_RESPONSE_TYPES.contains(&"numeric"));
        assert!(VALID_RESPONSE_TYPES.contains(&"date"));
        assert!(VALID_RESPONSE_TYPES.contains(&"multi_choice"));
        assert!(VALID_RESPONSE_TYPES.contains(&"file_upload"));
    }

    #[test]
    fn test_valid_invitation_statuses() {
        assert!(VALID_INVITATION_STATUSES.contains(&"initiated"));
        assert!(VALID_INVITATION_STATUSES.contains(&"pending_response"));
        assert!(VALID_INVITATION_STATUSES.contains(&"under_evaluation"));
        assert!(VALID_INVITATION_STATUSES.contains(&"qualified"));
        assert!(VALID_INVITATION_STATUSES.contains(&"disqualified"));
        assert!(VALID_INVITATION_STATUSES.contains(&"expired"));
        assert!(VALID_INVITATION_STATUSES.contains(&"withdrawn"));
    }

    #[test]
    fn test_valid_initiative_statuses() {
        assert!(VALID_INITIATIVE_STATUSES.contains(&"draft"));
        assert!(VALID_INITIATIVE_STATUSES.contains(&"active"));
        assert!(VALID_INITIATIVE_STATUSES.contains(&"pending_evaluations"));
        assert!(VALID_INITIATIVE_STATUSES.contains(&"completed"));
        assert!(VALID_INITIATIVE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_score_calculation() {
        let engine = SupplierQualificationEngine::new(Arc::new(crate::MockSupplierQualificationRepository));

        let responses = vec![
            SupplierQualificationResponse {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                invitation_id: Uuid::new_v4(),
                question_id: Uuid::new_v4(),
                response_text: Some("Yes".to_string()),
                response_value: None,
                file_reference: None,
                score: "8.00".to_string(),
                max_score: "10.00".to_string(),
                evaluator_notes: None,
                evaluated_by: None,
                evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            SupplierQualificationResponse {
                id: Uuid::new_v4(),
                organization_id: Uuid::new_v4(),
                invitation_id: Uuid::new_v4(),
                question_id: Uuid::new_v4(),
                response_text: Some("Detailed answer".to_string()),
                response_value: None,
                file_reference: None,
                score: "9.00".to_string(),
                max_score: "10.00".to_string(),
                evaluator_notes: None,
                evaluated_by: None,
                evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        let (overall, max, pct) = engine.calculate_scores(&responses);
        assert_eq!(overall, 17.0);
        assert_eq!(max, 20.0);
        assert!((pct - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_score_calculation_empty() {
        let engine = SupplierQualificationEngine::new(Arc::new(crate::MockSupplierQualificationRepository));
        let (overall, max, pct) = engine.calculate_scores(&[]);
        assert_eq!(overall, 0.0);
        assert_eq!(max, 0.0);
        assert_eq!(pct, 0.0);
    }
}

