//! Payment Risk & Fraud Detection Engine
//!
//! Orchestrates duplicate payment detection, risk scoring, velocity checks,
//! sanctions screening, supplier risk assessment, behavioral analysis,
//! and fraud alert lifecycle management.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Payment Risk

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    PaymentRiskRepository,
    RiskProfileCreateParams, FraudAlertCreateParams,
    SanctionsScreeningCreateParams, SupplierRiskAssessmentCreateParams,
    RiskProfile, FraudAlert, SanctionsScreeningResult, SupplierRiskAssessment,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid constants for validation
const VALID_ALERT_TYPES: &[&str] = &[
    "duplicate_payment", "amount_anomaly", "velocity_breach",
    "sanctions_match", "behavioral_anomaly", "pattern_match",
    "supplier_risk", "manual_referral",
];

const VALID_SEVERITIES: &[&str] = &["low", "medium", "high", "critical"];

const VALID_RISK_LEVELS: &[&str] = &["low", "medium", "high", "critical"];

const VALID_PROFILE_TYPES: &[&str] = &[
    "supplier_risk", "payment_risk", "invoice_risk", "global",
];

const VALID_ALERT_STATUSES: &[&str] = &[
    "open", "investigating", "escalated", "confirmed_fraud", "false_positive", "closed",
];

const VALID_ASSESSMENT_STATUSES: &[&str] = &["pending", "in_review", "approved", "rejected"];

const VALID_SCREENING_TYPES: &[&str] = &[
    "supplier_onboarding", "payment_processing", "periodic_review", "ad_hoc",
];

const VALID_SCREENED_LISTS: &[&str] = &[
    "ofac_sdn", "eu_consolidated", "un_security_council",
    "uk_hmt", "bis_entity", "dpl", "local_sanctions",
];

const VALID_MATCH_TYPES: &[&str] = &["exact", "partial", "fuzzy", "alias", "none"];

const VALID_MATCH_STATUSES: &[&str] = &[
    "potential_match", "confirmed_match", "false_positive", "no_match",
];

const VALID_ACTIONS: &[&str] = &["none", "blocked", "flagged_for_review", "reported", "whitelisted"];

/// Payment Risk & Fraud Detection engine
pub struct PaymentRiskEngine {
    repo: Arc<dyn PaymentRiskRepository>,
}

impl PaymentRiskEngine {
    pub fn new(repo: Arc<dyn PaymentRiskRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    fn validate_alert_type(alert_type: &str) -> AtlasResult<()> {
        if !VALID_ALERT_TYPES.contains(&alert_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid alert_type '{}'. Must be one of: {}", alert_type, VALID_ALERT_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_severity(severity: &str) -> AtlasResult<()> {
        if !VALID_SEVERITIES.contains(&severity) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid severity '{}'. Must be one of: {}", severity, VALID_SEVERITIES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_profile_type(profile_type: &str) -> AtlasResult<()> {
        if !VALID_PROFILE_TYPES.contains(&profile_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid profile_type '{}'. Must be one of: {}", profile_type, VALID_PROFILE_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_risk_level(risk_level: &str) -> AtlasResult<()> {
        if !VALID_RISK_LEVELS.contains(&risk_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_level '{}'. Must be one of: {}", risk_level, VALID_RISK_LEVELS.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_alert_status(status: &str) -> AtlasResult<()> {
        if !VALID_ALERT_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid alert status '{}'. Must be one of: {}", status, VALID_ALERT_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_screening_type(screening_type: &str) -> AtlasResult<()> {
        if !VALID_SCREENING_TYPES.contains(&screening_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid screening_type '{}'. Must be one of: {}", screening_type, VALID_SCREENING_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_screened_list(screened_list: &str) -> AtlasResult<()> {
        if !VALID_SCREENED_LISTS.contains(&screened_list) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid screened_list '{}'. Must be one of: {}", screened_list, VALID_SCREENED_LISTS.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_match_type(match_type: &str) -> AtlasResult<()> {
        if !VALID_MATCH_TYPES.contains(&match_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid match_type '{}'. Must be one of: {}", match_type, VALID_MATCH_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_match_status(match_status: &str) -> AtlasResult<()> {
        if !VALID_MATCH_STATUSES.contains(&match_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid match_status '{}'. Must be one of: {}", match_status, VALID_MATCH_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_action(action: &str) -> AtlasResult<()> {
        if !VALID_ACTIONS.contains(&action) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid action '{}'. Must be one of: {}", action, VALID_ACTIONS.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_assessment_status(status: &str) -> AtlasResult<()> {
        if !VALID_ASSESSMENT_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid assessment status '{}'. Must be one of: {}", status, VALID_ASSESSMENT_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    // ========================================================================
    // Risk Profiles
    // ========================================================================

    /// Create a new risk profile
    pub async fn create_risk_profile(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        profile_type: &str,
        default_risk_level: &str,
        duplicate_amount_tolerance_pct: Option<&str>,
        duplicate_date_tolerance_days: Option<&str>,
        velocity_daily_limit: Option<&str>,
        velocity_weekly_limit: Option<&str>,
        amount_anomaly_std_dev: Option<&str>,
        enable_sanctions_screening: bool,
        enable_duplicate_detection: bool,
        enable_velocity_checks: bool,
        enable_amount_anomaly: bool,
        enable_behavioral_analysis: bool,
        auto_block_critical: bool,
        auto_block_high: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskProfile> {
        info!("Creating risk profile '{}' for org {}", code, org_id);
        Self::validate_profile_type(profile_type)?;
        Self::validate_risk_level(default_risk_level)?;

        let params = RiskProfileCreateParams {
            org_id,
            code: code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            profile_type: profile_type.to_string(),
            default_risk_level: default_risk_level.to_string(),
            duplicate_amount_tolerance_pct: duplicate_amount_tolerance_pct.map(|s| s.to_string()),
            duplicate_date_tolerance_days: duplicate_date_tolerance_days.map(|s| s.to_string()),
            velocity_daily_limit: velocity_daily_limit.map(|s| s.to_string()),
            velocity_weekly_limit: velocity_weekly_limit.map(|s| s.to_string()),
            amount_anomaly_std_dev: amount_anomaly_std_dev.map(|s| s.to_string()),
            enable_sanctions_screening,
            enable_duplicate_detection,
            enable_velocity_checks,
            enable_amount_anomaly,
            enable_behavioral_analysis,
            auto_block_critical,
            auto_block_high,
            effective_from,
            effective_to,
            created_by,
        };
        self.repo.create_risk_profile(&params).await
    }

    /// Get a risk profile by code
    pub async fn get_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskProfile>> {
        self.repo.get_risk_profile(org_id, code).await
    }

    /// List risk profiles with optional filters
    pub async fn list_risk_profiles(&self, org_id: Uuid, profile_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<RiskProfile>> {
        self.repo.list_risk_profiles(org_id, profile_type, is_active).await
    }

    /// Activate or deactivate a risk profile
    pub async fn set_risk_profile_active(&self, id: Uuid, is_active: bool) -> AtlasResult<RiskProfile> {
        self.repo.update_risk_profile_status(id, is_active).await
    }

    /// Delete a risk profile
    pub async fn delete_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting risk profile '{}' for org {}", code, org_id);
        self.repo.delete_risk_profile(org_id, code).await
    }

    // ========================================================================
    // Fraud Alerts
    // ========================================================================

    /// Create a new fraud alert
    pub async fn create_fraud_alert(
        &self,
        org_id: Uuid,
        alert_type: &str,
        severity: &str,
        payment_id: Option<Uuid>,
        invoice_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        amount: Option<&str>,
        currency_code: Option<&str>,
        risk_score: Option<&str>,
        detection_rule: Option<&str>,
        description: Option<&str>,
        evidence: Option<&str>,
        assigned_to: Option<&str>,
        assigned_team: Option<&str>,
        related_alert_ids: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FraudAlert> {
        info!("Creating fraud alert type '{}' severity '{}' for org {}", alert_type, severity, org_id);
        Self::validate_alert_type(alert_type)?;
        Self::validate_severity(severity)?;

        // Auto-generate alert number
        let seq = self.repo.get_next_alert_sequence(org_id).await.unwrap_or(1);
        let alert_number = format!("FA-{:06}", seq);

        let params = FraudAlertCreateParams {
            org_id,
            alert_type: alert_type.to_string(),
            severity: severity.to_string(),
            payment_id,
            invoice_id,
            supplier_id,
            supplier_number: supplier_number.map(|s| s.to_string()),
            supplier_name: supplier_name.map(|s| s.to_string()),
            amount: amount.map(|s| s.to_string()),
            currency_code: currency_code.map(|s| s.to_string()),
            risk_score: risk_score.map(|s| s.to_string()),
            detection_rule: detection_rule.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            evidence: evidence.map(|s| s.to_string()),
            assigned_to: assigned_to.map(|s| s.to_string()),
            assigned_team: assigned_team.map(|s| s.to_string()),
            related_alert_ids: related_alert_ids.map(|s| s.to_string()),
            created_by,
        };

        // We need to override the alert number - use a direct approach
        // The repo create_fraud_alert generates the number from the sequence
        let mut alert = self.repo.create_fraud_alert(&params).await?;
        // The alert_number is set by the database trigger or we set it here
        // For now, we rely on the sequence in the repo
        Ok(alert)
    }

    /// Get a fraud alert by number
    pub async fn get_fraud_alert(&self, org_id: Uuid, alert_number: &str) -> AtlasResult<Option<FraudAlert>> {
        self.repo.get_fraud_alert(org_id, alert_number).await
    }

    /// List fraud alerts with optional filters
    pub async fn list_fraud_alerts(&self, org_id: Uuid, status: Option<&str>, alert_type: Option<&str>, severity: Option<&str>) -> AtlasResult<Vec<FraudAlert>> {
        self.repo.list_fraud_alerts(org_id, status, alert_type, severity).await
    }

    /// Transition a fraud alert to a new status (workflow action)
    pub async fn transition_fraud_alert(
        &self,
        id: Uuid,
        new_status: &str,
        resolution_notes: Option<&str>,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<FraudAlert> {
        info!("Transitioning fraud alert {} to status '{}'", id, new_status);
        Self::validate_alert_status(new_status)?;

        // Verify current status allows this transition
        let current = self.repo.get_fraud_alert_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Fraud alert not found".to_string()))?;

        Self::validate_fraud_alert_transition(&current.status, new_status)?;

        self.repo.update_fraud_alert_status(id, new_status, resolution_notes, resolved_by).await
    }

    /// Assign a fraud alert to a user/team
    pub async fn assign_fraud_alert(&self, id: Uuid, assigned_to: Option<&str>, assigned_team: Option<&str>) -> AtlasResult<FraudAlert> {
        self.repo.assign_fraud_alert(id, assigned_to, assigned_team).await
    }

    /// Validate workflow transitions for fraud alerts
    fn validate_fraud_alert_transition(from: &str, to: &str) -> AtlasResult<()> {
        let allowed = match from {
            "open" => matches!(to, "investigating" | "closed"),
            "investigating" => matches!(to, "escalated" | "confirmed_fraud" | "false_positive"),
            "escalated" => matches!(to, "confirmed_fraud" | "false_positive"),
            _ => false,
        };
        if !allowed {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transition from '{}' to '{}'", from, to
            )));
        }
        Ok(())
    }

    // ========================================================================
    // Sanctions Screening
    // ========================================================================

    /// Create a sanctions screening result
    pub async fn create_screening_result(
        &self,
        org_id: Uuid,
        screening_type: &str,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        payment_id: Option<Uuid>,
        screened_list: &str,
        match_name: Option<&str>,
        match_type: &str,
        match_score: Option<&str>,
        match_status: &str,
        sanctions_list_entry: Option<&str>,
        sanctions_list_program: Option<&str>,
        match_details: Option<&str>,
        action_taken: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SanctionsScreeningResult> {
        info!("Creating sanctions screening result for org {}", org_id);
        Self::validate_screening_type(screening_type)?;
        Self::validate_screened_list(screened_list)?;
        Self::validate_match_type(match_type)?;
        Self::validate_match_status(match_status)?;
        if let Some(action) = action_taken {
            Self::validate_action(action)?;
        }

        let seq = self.repo.get_next_screening_sequence(org_id).await.unwrap_or(1);
        let screening_id = format!("SC-{:06}", seq);

        let params = SanctionsScreeningCreateParams {
            org_id,
            screening_type: screening_type.to_string(),
            supplier_id,
            supplier_name: supplier_name.map(|s| s.to_string()),
            payment_id,
            screened_list: screened_list.to_string(),
            match_name: match_name.map(|s| s.to_string()),
            match_type: match_type.to_string(),
            match_score: match_score.map(|s| s.to_string()),
            match_status: match_status.to_string(),
            sanctions_list_entry: sanctions_list_entry.map(|s| s.to_string()),
            sanctions_list_program: sanctions_list_program.map(|s| s.to_string()),
            match_details: match_details.map(|s| s.to_string()),
            action_taken: action_taken.map(|s| s.to_string()),
            created_by,
        };
        self.repo.create_screening_result(&params).await
    }

    /// Get a screening result by ID
    pub async fn get_screening_result(&self, org_id: Uuid, screening_id: &str) -> AtlasResult<Option<SanctionsScreeningResult>> {
        self.repo.get_screening_result(org_id, screening_id).await
    }

    /// List screening results
    pub async fn list_screening_results(&self, org_id: Uuid, supplier_id: Option<Uuid>, match_status: Option<&str>) -> AtlasResult<Vec<SanctionsScreeningResult>> {
        self.repo.list_screening_results(org_id, supplier_id, match_status).await
    }

    /// Review a screening result
    pub async fn review_screening_result(&self, id: Uuid, reviewed_by: &str, review_notes: Option<&str>, action_taken: &str) -> AtlasResult<SanctionsScreeningResult> {
        info!("Reviewing screening result {} by {}", id, reviewed_by);
        Self::validate_action(action_taken)?;
        self.repo.review_screening_result(id, reviewed_by, review_notes, action_taken).await
    }

    // ========================================================================
    // Supplier Risk Assessments
    // ========================================================================

    /// Create a supplier risk assessment
    pub async fn create_assessment(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_name: &str,
        assessment_type: &str,
        financial_risk_score: Option<&str>,
        operational_risk_score: Option<&str>,
        compliance_risk_score: Option<&str>,
        payment_history_score: Option<&str>,
        years_in_business: Option<i32>,
        has_financial_statements: bool,
        has_audit_reports: bool,
        has_insurance: bool,
        is_sanctions_clear: bool,
        is_aml_clear: bool,
        is_pep_clear: bool,
        total_historical_payments: Option<i32>,
        total_historical_amount: Option<&str>,
        fraud_alerts_count: Option<i32>,
        duplicate_payments_count: Option<i32>,
        assessed_by: Option<&str>,
        findings: Option<&str>,
        recommendations: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierRiskAssessment> {
        info!("Creating risk assessment for supplier '{}' org {}", supplier_name, org_id);

        let seq = self.repo.get_next_assessment_sequence(org_id).await.unwrap_or(1);
        let assessment_number = format!("RA-{:06}", seq);

        let params = SupplierRiskAssessmentCreateParams {
            org_id,
            supplier_id,
            supplier_name: supplier_name.to_string(),
            assessment_type: assessment_type.to_string(),
            financial_risk_score: financial_risk_score.map(|s| s.to_string()),
            operational_risk_score: operational_risk_score.map(|s| s.to_string()),
            compliance_risk_score: compliance_risk_score.map(|s| s.to_string()),
            payment_history_score: payment_history_score.map(|s| s.to_string()),
            years_in_business,
            has_financial_statements,
            has_audit_reports,
            has_insurance,
            is_sanctions_clear,
            is_aml_clear,
            is_pep_clear,
            total_historical_payments,
            total_historical_amount: total_historical_amount.map(|s| s.to_string()),
            fraud_alerts_count,
            duplicate_payments_count,
            assessed_by: assessed_by.map(|s| s.to_string()),
            findings: findings.map(|s| s.to_string()),
            recommendations: recommendations.map(|s| s.to_string()),
            created_by,
        };
        self.repo.create_assessment(&params).await
    }

    /// Get an assessment by number
    pub async fn get_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<Option<SupplierRiskAssessment>> {
        self.repo.get_assessment(org_id, assessment_number).await
    }

    /// List assessments with optional filters
    pub async fn list_assessments(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierRiskAssessment>> {
        self.repo.list_assessments(org_id, supplier_id, status).await
    }

    /// Transition an assessment to a new status
    pub async fn transition_assessment(&self, id: Uuid, new_status: &str) -> AtlasResult<SupplierRiskAssessment> {
        info!("Transitioning assessment {} to '{}'", id, new_status);
        Self::validate_assessment_status(new_status)?;

        let current = self.repo.get_assessment_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Assessment not found".to_string()))?;

        Self::validate_assessment_transition(&current.status, new_status)?;

        self.repo.update_assessment_status(id, new_status).await
    }

    /// Delete an assessment
    pub async fn delete_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<()> {
        self.repo.delete_assessment(org_id, assessment_number).await
    }

    /// Validate workflow transitions for supplier risk assessments
    fn validate_assessment_transition(from: &str, to: &str) -> AtlasResult<()> {
        let allowed = match from {
            "pending" => matches!(to, "in_review"),
            "in_review" => matches!(to, "approved" | "rejected"),
            _ => false,
        };
        if !allowed {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transition from '{}' to '{}'", from, to
            )));
        }
        Ok(())
    }
}
