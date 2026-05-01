//! Workplace Health & Safety Engine
//!
//! Manages safety incidents, hazard identification, safety inspections,
//! corrective/preventive actions (CAPA), and compliance tracking.
//!
//! Oracle Fusion Cloud equivalent: Environment, Health, and Safety (EHS)

use atlas_shared::{
    SafetyIncident, Hazard, SafetyInspection, SafetyCorrectiveAction,
    HealthSafetyDashboard,
    AtlasError, AtlasResult,
};
use super::HealthSafetyRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_INCIDENT_TYPES: &[&str] = &[
    "injury", "illness", "near_miss", "property_damage",
    "environmental_release", "fire", "vehicle_incident", "other",
];

const VALID_SEVERITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

const VALID_INCIDENT_STATUSES: &[&str] = &[
    "reported", "under_investigation", "corrective_action",
    "resolved", "closed",
];

const VALID_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "urgent",
];

const VALID_OSHA_CLASSIFICATIONS: &[&str] = &[
    "death", "days_away_from_work", "job_transfer_restriction",
    "other_recordable", "first_aid_only",
];

const VALID_HAZARD_CATEGORIES: &[&str] = &[
    "physical", "chemical", "biological", "ergonomic",
    "psychosocial", "electrical", "mechanical", "thermal",
    "radiation", "noise", "vibration", "other",
];

const VALID_RISK_LEVELS: &[&str] = &[
    "negligible", "low", "medium", "high", "very_high", "extreme",
];

const VALID_LIKELIHOODS: &[&str] = &[
    "rare", "unlikely", "possible", "likely", "almost_certain",
];

const VALID_CONSEQUENCES: &[&str] = &[
    "insignificant", "minor", "moderate", "major", "catastrophic",
];

const VALID_HAZARD_STATUSES: &[&str] = &[
    "identified", "assessed", "mitigated", "closed", "transferred",
];

const VALID_INSPECTION_TYPES: &[&str] = &[
    "routine", "periodic", "pre_use", "post_incident",
    "regulatory", "internal_audit", "external_audit",
];

const VALID_INSPECTION_STATUSES: &[&str] = &[
    "scheduled", "in_progress", "completed", "cancelled",
];

const VALID_ACTION_TYPES: &[&str] = &[
    "corrective", "preventive", "corrective_and_preventive",
];

const VALID_CAPA_STATUSES: &[&str] = &[
    "open", "in_progress", "pending_verification",
    "completed", "closed", "cancelled",
];

const VALID_EFFECTIVENESS: &[&str] = &[
    "not_effective", "partially_effective", "effective", "highly_effective",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Compute risk score from likelihood and consequence using a 5x5 matrix
fn compute_risk_score(likelihood: &str, consequence: &str) -> i32 {
    let l = match likelihood {
        "rare" => 1,
        "unlikely" => 2,
        "possible" => 3,
        "likely" => 4,
        "almost_certain" => 5,
        _ => 1,
    };
    let c = match consequence {
        "insignificant" => 1,
        "minor" => 2,
        "moderate" => 3,
        "major" => 4,
        "catastrophic" => 5,
        _ => 1,
    };
    l * c
}

/// Determine risk level from risk score
fn risk_level_from_score(score: i32) -> &'static str {
    match score {
        1..=3 => "low",
        4..=6 => "medium",
        7..=12 => "high",
        13..=25 => "extreme",
        _ => "negligible",
    }
}

/// Workplace Health & Safety Engine
pub struct HealthSafetyEngine {
    repository: Arc<dyn HealthSafetyRepository>,
}

impl HealthSafetyEngine {
    pub fn new(repository: Arc<dyn HealthSafetyRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Safety Incidents
    // ========================================================================

    /// Create a new safety incident
    #[allow(clippy::too_many_arguments)]
    pub async fn create_incident(
        &self,
        org_id: Uuid,
        incident_number: &str,
        title: &str,
        description: Option<&str>,
        incident_type: &str,
        severity: &str,
        priority: &str,
        incident_date: chrono::NaiveDate,
        incident_time: Option<&str>,
        location: Option<&str>,
        facility_id: Option<Uuid>,
        department_id: Option<Uuid>,
        reported_by_id: Option<Uuid>,
        reported_by_name: Option<&str>,
        assigned_to_id: Option<Uuid>,
        assigned_to_name: Option<&str>,
        osha_recordable: bool,
        osha_classification: Option<&str>,
        body_part: Option<&str>,
        injury_source: Option<&str>,
        event_type: Option<&str>,
        environment_factor: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyIncident> {
        if incident_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Incident number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Incident title is required".to_string()));
        }
        validate_enum("incident_type", incident_type, VALID_INCIDENT_TYPES)?;
        validate_enum("severity", severity, VALID_SEVERITIES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;
        if let Some(osha) = osha_classification {
            validate_enum("osha_classification", osha, VALID_OSHA_CLASSIFICATIONS)?;
        }

        if self.repository.get_incident_by_number(org_id, incident_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Incident '{}' already exists", incident_number
            )));
        }

        info!("Creating safety incident '{}' ({}) for org {} [type={}, severity={}]",
              incident_number, title, org_id, incident_type, severity);

        self.repository.create_incident(
            org_id, incident_number, title, description,
            incident_type, severity, "reported", priority,
            incident_date, incident_time, location,
            facility_id, department_id,
            reported_by_id, reported_by_name,
            assigned_to_id, assigned_to_name,
            None, None, // root_cause, immediate_action
            osha_recordable, osha_classification,
            0, 0, // days_away, days_restricted
            body_part, injury_source, event_type, environment_factor,
            serde_json::json!([]), serde_json::json!([]), serde_json::json!([]),
            None, None, None, // resolution_date, closed_date, closed_by
            serde_json::json!({}),
            created_by,
        ).await
    }

    /// Get an incident by ID
    pub async fn get_incident(&self, id: Uuid) -> AtlasResult<Option<SafetyIncident>> {
        self.repository.get_incident(id).await
    }

    /// Get an incident by number
    pub async fn get_incident_by_number(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<Option<SafetyIncident>> {
        self.repository.get_incident_by_number(org_id, incident_number).await
    }

    /// List incidents with optional filters
    pub async fn list_incidents(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        severity: Option<&str>,
        incident_type: Option<&str>,
        facility_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<SafetyIncident>> {
        self.repository.list_incidents(org_id, status, severity, incident_type, facility_id).await
    }

    /// Update incident status
    pub async fn update_incident_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<SafetyIncident> {
        validate_enum("incident status", status, VALID_INCIDENT_STATUSES)?;
        info!("Updating incident {} status to {}", id, status);
        self.repository.update_incident_status(id, status).await
    }

    /// Update incident investigation details
    #[allow(clippy::too_many_arguments)]
    pub async fn update_incident_investigation(
        &self,
        id: Uuid,
        root_cause: Option<&str>,
        immediate_action: Option<&str>,
        assigned_to_id: Option<Uuid>,
        assigned_to_name: Option<&str>,
        days_away_from_work: Option<i32>,
        days_restricted: Option<i32>,
    ) -> AtlasResult<SafetyIncident> {
        info!("Updating investigation for incident {}", id);
        self.repository.update_incident_investigation(
            id, root_cause, immediate_action,
            assigned_to_id, assigned_to_name,
            days_away_from_work, days_restricted,
        ).await
    }

    /// Close an incident
    pub async fn close_incident(
        &self,
        id: Uuid,
        closed_by: Option<Uuid>,
    ) -> AtlasResult<SafetyIncident> {
        info!("Closing incident {} by {:?}", id, closed_by);
        self.repository.close_incident(id, closed_by).await
    }

    /// Delete an incident by number
    pub async fn delete_incident(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<()> {
        info!("Deleting incident '{}' for org {}", incident_number, org_id);
        self.repository.delete_incident(org_id, incident_number).await
    }

    // ========================================================================
    // Hazard Identification
    // ========================================================================

    /// Create a hazard identification record
    #[allow(clippy::too_many_arguments)]
    pub async fn create_hazard(
        &self,
        org_id: Uuid,
        hazard_code: &str,
        title: &str,
        description: Option<&str>,
        hazard_category: &str,
        likelihood: &str,
        consequence: &str,
        location: Option<&str>,
        facility_id: Option<Uuid>,
        department_id: Option<Uuid>,
        identified_by_id: Option<Uuid>,
        identified_by_name: Option<&str>,
        identified_date: chrono::NaiveDate,
        mitigation_measures: Option<serde_json::Value>,
        review_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Hazard> {
        if hazard_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Hazard code is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Hazard title is required".to_string()));
        }
        validate_enum("hazard_category", hazard_category, VALID_HAZARD_CATEGORIES)?;
        validate_enum("likelihood", likelihood, VALID_LIKELIHOODS)?;
        validate_enum("consequence", consequence, VALID_CONSEQUENCES)?;

        let risk_score = compute_risk_score(likelihood, consequence);
        let risk_level = risk_level_from_score(risk_score).to_string();

        if self.repository.get_hazard_by_code(org_id, hazard_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Hazard '{}' already exists", hazard_code
            )));
        }

        info!("Creating hazard '{}' ({}) for org {} [category={}, risk={}, score={}]",
              hazard_code, title, org_id, hazard_category, risk_level, risk_score);

        self.repository.create_hazard(
            org_id, hazard_code, title, description,
            hazard_category, &risk_level, likelihood, consequence,
            risk_score, "identified",
            location, facility_id, department_id,
            identified_by_id, identified_by_name, identified_date,
            mitigation_measures.unwrap_or(serde_json::json!([])),
            None, None, // residual_risk_level, residual_risk_score
            review_date,
            owner_id, owner_name,
            serde_json::json!({}),
            created_by,
        ).await
    }

    /// Get a hazard by ID
    pub async fn get_hazard(&self, id: Uuid) -> AtlasResult<Option<Hazard>> {
        self.repository.get_hazard(id).await
    }

    /// Get a hazard by code
    pub async fn get_hazard_by_code(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<Option<Hazard>> {
        self.repository.get_hazard_by_code(org_id, hazard_code).await
    }

    /// List hazards with optional filters
    pub async fn list_hazards(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_level: Option<&str>,
        hazard_category: Option<&str>,
        facility_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<Hazard>> {
        self.repository.list_hazards(org_id, status, risk_level, hazard_category, facility_id).await
    }

    /// Update hazard status
    pub async fn update_hazard_status(&self, id: Uuid, status: &str) -> AtlasResult<Hazard> {
        validate_enum("hazard status", status, VALID_HAZARD_STATUSES)?;
        info!("Updating hazard {} status to {}", id, status);
        self.repository.update_hazard_status(id, status).await
    }

    /// Assess residual risk after mitigation
    pub async fn assess_residual_risk(
        &self,
        id: Uuid,
        residual_likelihood: &str,
        residual_consequence: &str,
    ) -> AtlasResult<Hazard> {
        validate_enum("residual_likelihood", residual_likelihood, VALID_LIKELIHOODS)?;
        validate_enum("residual_consequence", residual_consequence, VALID_CONSEQUENCES)?;

        let score = compute_risk_score(residual_likelihood, residual_consequence);
        let level = risk_level_from_score(score).to_string();

        info!("Assessing residual risk for hazard {} [level={}, score={}]", id, level, score);
        self.repository.update_residual_risk(id, &level, score).await
    }

    /// Delete a hazard by code
    pub async fn delete_hazard(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<()> {
        info!("Deleting hazard '{}' for org {}", hazard_code, org_id);
        self.repository.delete_hazard(org_id, hazard_code).await
    }

    // ========================================================================
    // Safety Inspections
    // ========================================================================

    /// Create a safety inspection
    #[allow(clippy::too_many_arguments)]
    pub async fn create_inspection(
        &self,
        org_id: Uuid,
        inspection_number: &str,
        title: &str,
        description: Option<&str>,
        inspection_type: &str,
        priority: &str,
        scheduled_date: chrono::NaiveDate,
        location: Option<&str>,
        facility_id: Option<Uuid>,
        department_id: Option<Uuid>,
        inspector_id: Option<Uuid>,
        inspector_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyInspection> {
        if inspection_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Inspection number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Inspection title is required".to_string()));
        }
        validate_enum("inspection_type", inspection_type, VALID_INSPECTION_TYPES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;

        if self.repository.get_inspection_by_number(org_id, inspection_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Inspection '{}' already exists", inspection_number
            )));
        }

        info!("Creating safety inspection '{}' ({}) for org {} [type={}, scheduled={}]",
              inspection_number, title, org_id, inspection_type, scheduled_date);

        self.repository.create_inspection(
            org_id, inspection_number, title, description,
            inspection_type, "scheduled", priority,
            scheduled_date, None, // completed_date
            location, facility_id, department_id,
            inspector_id, inspector_name,
            None, 0, 0, 0, 0, // findings_summary, counts
            None, None, None, // score, max_score, score_pct
            serde_json::json!([]), serde_json::json!([]),
            serde_json::json!({}),
            created_by,
        ).await
    }

    /// Get an inspection by ID
    pub async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<SafetyInspection>> {
        self.repository.get_inspection(id).await
    }

    /// Get an inspection by number
    pub async fn get_inspection_by_number(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<Option<SafetyInspection>> {
        self.repository.get_inspection_by_number(org_id, inspection_number).await
    }

    /// List inspections with optional filters
    pub async fn list_inspections(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        inspection_type: Option<&str>,
        facility_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<SafetyInspection>> {
        self.repository.list_inspections(org_id, status, inspection_type, facility_id).await
    }

    /// Complete an inspection with findings
    pub async fn complete_inspection(
        &self,
        id: Uuid,
        findings_summary: Option<&str>,
        findings: serde_json::Value,
        critical_findings: i32,
        non_conformities: i32,
        observations: i32,
        score: Option<f64>,
        max_score: Option<f64>,
    ) -> AtlasResult<SafetyInspection> {
        let total_findings = critical_findings + non_conformities + observations;
        let score_pct = match (score, max_score) {
            (Some(s), Some(m)) if m > 0.0 => Some((s / m * 100.0).clamp(0.0, 100.0)),
            _ => None,
        };

        info!("Completing inspection {} [findings={}, critical={}, score={:?}]",
              id, total_findings, critical_findings, score);

        self.repository.complete_inspection(
            id, findings_summary,
            total_findings, critical_findings, non_conformities, observations,
            score, max_score, score_pct,
            findings,
        ).await
    }

    /// Update inspection status
    pub async fn update_inspection_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyInspection> {
        validate_enum("inspection status", status, VALID_INSPECTION_STATUSES)?;
        info!("Updating inspection {} status to {}", id, status);
        self.repository.update_inspection_status(id, status).await
    }

    /// Delete an inspection by number
    pub async fn delete_inspection(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<()> {
        info!("Deleting inspection '{}' for org {}", inspection_number, org_id);
        self.repository.delete_inspection(org_id, inspection_number).await
    }

    // ========================================================================
    // Corrective & Preventive Actions (CAPA)
    // ========================================================================

    /// Create a corrective/preventive action
    #[allow(clippy::too_many_arguments)]
    pub async fn create_corrective_action(
        &self,
        org_id: Uuid,
        action_number: &str,
        title: &str,
        description: Option<&str>,
        action_type: &str,
        priority: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        root_cause: Option<&str>,
        corrective_action_plan: Option<&str>,
        preventive_action_plan: Option<&str>,
        assigned_to_id: Option<Uuid>,
        assigned_to_name: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        facility_id: Option<Uuid>,
        department_id: Option<Uuid>,
        estimated_cost: Option<f64>,
        currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyCorrectiveAction> {
        if action_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Action number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Action title is required".to_string()));
        }
        validate_enum("action_type", action_type, VALID_ACTION_TYPES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;

        if let Some(est) = estimated_cost {
            if est < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Estimated cost cannot be negative".to_string(),
                ));
            }
        }

        if self.repository.get_corrective_action_by_number(org_id, action_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Corrective action '{}' already exists", action_number
            )));
        }

        info!("Creating corrective action '{}' ({}) for org {} [type={}, priority={}]",
              action_number, title, org_id, action_type, priority);

        self.repository.create_corrective_action(
            org_id, action_number, title, description,
            action_type, "open", priority,
            source_type, source_id, source_number,
            root_cause, corrective_action_plan, preventive_action_plan,
            assigned_to_id, assigned_to_name,
            due_date, None, // completed_date
            None, None, None, // verified_by, verified_date, effectiveness
            facility_id, department_id,
            estimated_cost, None, currency_code, // actual_cost
            None, // notes
            serde_json::json!([]), serde_json::json!({}),
            created_by,
        ).await
    }

    /// Get a corrective action by ID
    pub async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<SafetyCorrectiveAction>> {
        self.repository.get_corrective_action(id).await
    }

    /// Get a corrective action by number
    pub async fn get_corrective_action_by_number(&self, org_id: Uuid, action_number: &str) -> AtlasResult<Option<SafetyCorrectiveAction>> {
        self.repository.get_corrective_action_by_number(org_id, action_number).await
    }

    /// List corrective actions with optional filters
    pub async fn list_corrective_actions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        action_type: Option<&str>,
        source_type: Option<&str>,
    ) -> AtlasResult<Vec<SafetyCorrectiveAction>> {
        self.repository.list_corrective_actions(org_id, status, action_type, source_type).await
    }

    /// Update corrective action status
    pub async fn update_corrective_action_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyCorrectiveAction> {
        validate_enum("CAPA status", status, VALID_CAPA_STATUSES)?;
        info!("Updating corrective action {} status to {}", id, status);
        self.repository.update_corrective_action_status(id, status).await
    }

    /// Complete a corrective action with verification
    pub async fn complete_corrective_action(
        &self,
        id: Uuid,
        effectiveness: &str,
        actual_cost: Option<f64>,
        verified_by: Option<Uuid>,
    ) -> AtlasResult<SafetyCorrectiveAction> {
        validate_enum("effectiveness", effectiveness, VALID_EFFECTIVENESS)?;
        if let Some(cost) = actual_cost {
            if cost < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Actual cost cannot be negative".to_string(),
                ));
            }
        }
        info!("Completing corrective action {} [effectiveness={}]", id, effectiveness);
        self.repository.complete_corrective_action(id, effectiveness, actual_cost, verified_by).await
    }

    /// Delete a corrective action by number
    pub async fn delete_corrective_action(&self, org_id: Uuid, action_number: &str) -> AtlasResult<()> {
        info!("Deleting corrective action '{}' for org {}", action_number, org_id);
        self.repository.delete_corrective_action(org_id, action_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the Health & Safety dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HealthSafetyDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_incident_types() {
        assert!(VALID_INCIDENT_TYPES.contains(&"injury"));
        assert!(VALID_INCIDENT_TYPES.contains(&"near_miss"));
        assert!(VALID_INCIDENT_TYPES.contains(&"property_damage"));
        assert!(VALID_INCIDENT_TYPES.contains(&"environmental_release"));
        assert!(VALID_INCIDENT_TYPES.contains(&"fire"));
        assert!(!VALID_INCIDENT_TYPES.contains(&"flood"));
    }

    #[test]
    fn test_valid_severities() {
        assert!(VALID_SEVERITIES.contains(&"low"));
        assert!(VALID_SEVERITIES.contains(&"medium"));
        assert!(VALID_SEVERITIES.contains(&"high"));
        assert!(VALID_SEVERITIES.contains(&"critical"));
        assert!(!VALID_SEVERITIES.contains(&"extreme"));
    }

    #[test]
    fn test_valid_incident_statuses() {
        assert!(VALID_INCIDENT_STATUSES.contains(&"reported"));
        assert!(VALID_INCIDENT_STATUSES.contains(&"under_investigation"));
        assert!(VALID_INCIDENT_STATUSES.contains(&"corrective_action"));
        assert!(VALID_INCIDENT_STATUSES.contains(&"resolved"));
        assert!(VALID_INCIDENT_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_hazard_categories() {
        assert!(VALID_HAZARD_CATEGORIES.contains(&"physical"));
        assert!(VALID_HAZARD_CATEGORIES.contains(&"chemical"));
        assert!(VALID_HAZARD_CATEGORIES.contains(&"biological"));
        assert!(VALID_HAZARD_CATEGORIES.contains(&"ergonomic"));
        assert!(VALID_HAZARD_CATEGORIES.contains(&"electrical"));
    }

    #[test]
    fn test_valid_risk_levels() {
        assert!(VALID_RISK_LEVELS.contains(&"negligible"));
        assert!(VALID_RISK_LEVELS.contains(&"low"));
        assert!(VALID_RISK_LEVELS.contains(&"medium"));
        assert!(VALID_RISK_LEVELS.contains(&"high"));
        assert!(VALID_RISK_LEVELS.contains(&"extreme"));
    }

    #[test]
    fn test_valid_inspection_types() {
        assert!(VALID_INSPECTION_TYPES.contains(&"routine"));
        assert!(VALID_INSPECTION_TYPES.contains(&"periodic"));
        assert!(VALID_INSPECTION_TYPES.contains(&"post_incident"));
        assert!(VALID_INSPECTION_TYPES.contains(&"regulatory"));
        assert!(VALID_INSPECTION_TYPES.contains(&"internal_audit"));
    }

    #[test]
    fn test_valid_capa_statuses() {
        assert!(VALID_CAPA_STATUSES.contains(&"open"));
        assert!(VALID_CAPA_STATUSES.contains(&"in_progress"));
        assert!(VALID_CAPA_STATUSES.contains(&"pending_verification"));
        assert!(VALID_CAPA_STATUSES.contains(&"completed"));
        assert!(VALID_CAPA_STATUSES.contains(&"closed"));
        assert!(VALID_CAPA_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_effectiveness() {
        assert!(VALID_EFFECTIVENESS.contains(&"not_effective"));
        assert!(VALID_EFFECTIVENESS.contains(&"partially_effective"));
        assert!(VALID_EFFECTIVENESS.contains(&"effective"));
        assert!(VALID_EFFECTIVENESS.contains(&"highly_effective"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test", "injury", VALID_INCIDENT_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("incident_type", "flood", VALID_INCIDENT_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("incident_type"));
                assert!(msg.contains("flood"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("severity", "", VALID_SEVERITIES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_compute_risk_score() {
        assert_eq!(compute_risk_score("rare", "insignificant"), 1);
        assert_eq!(compute_risk_score("almost_certain", "catastrophic"), 25);
        assert_eq!(compute_risk_score("possible", "moderate"), 9);
        assert_eq!(compute_risk_score("likely", "minor"), 8);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(risk_level_from_score(1), "low");
        assert_eq!(risk_level_from_score(3), "low");
        assert_eq!(risk_level_from_score(4), "medium");
        assert_eq!(risk_level_from_score(6), "medium");
        assert_eq!(risk_level_from_score(9), "high");
        assert_eq!(risk_level_from_score(12), "high");
        assert_eq!(risk_level_from_score(16), "extreme");
        assert_eq!(risk_level_from_score(25), "extreme");
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockHealthSafetyRepository;
    use chrono::NaiveDate;

    fn create_engine() -> HealthSafetyEngine {
        HealthSafetyEngine::new(Arc::new(MockHealthSafetyRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    // --- Incident Tests ---

    #[tokio::test]
    async fn test_create_incident_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "", "Slip and Fall", None,
            "injury", "medium", "high",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), None,
            Some("Building A"), None, None, None, None,
            None, None,
            false, None, Some("back"), Some("wet_floor"), None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_incident_validation_empty_title() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "INC-001", "", None,
            "injury", "medium", "high",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), None,
            None, None, None, None, None,
            None, None,
            false, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("title")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_incident_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "INC-001", "Test", None,
            "explosion", "medium", "high",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), None,
            None, None, None, None, None,
            None, None,
            false, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("incident_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_incident_validation_bad_severity() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "INC-001", "Test", None,
            "injury", "extreme", "high",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), None,
            None, None, None, None, None,
            None, None,
            false, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("severity")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_incident_validation_bad_priority() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "INC-001", "Test", None,
            "injury", "medium", "critical",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), None,
            None, None, None, None, None,
            None, None,
            false, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("priority")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_incident_success() {
        let engine = create_engine();
        let result = engine.create_incident(
            test_org_id(), "INC-001", "Slip and Fall in Building A", Some("Employee slipped on wet floor"),
            "injury", "medium", "high",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), Some("09:30"),
            Some("Building A, Floor 2"), None, None,
            Some(test_user_id()), Some("John Smith"),
            None, None,
            true, Some("other_recordable"),
            Some("back"), Some("wet_floor"), Some("slip_trip_fall"), None,
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let inc = result.unwrap();
        assert_eq!(inc.incident_number, "INC-001");
        assert_eq!(inc.incident_type, "injury");
        assert_eq!(inc.severity, "medium");
        assert_eq!(inc.status, "reported");
        assert!(inc.osha_recordable);
    }

    #[tokio::test]
    async fn test_update_incident_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_incident_status(Uuid::new_v4(), "deleted").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_incident_status_valid() {
        let engine = create_engine();
        let result = engine.update_incident_status(Uuid::new_v4(), "under_investigation").await;
        // Mock returns error for non-existent, but validation passes
        assert!(result.is_ok());
    }

    // --- Hazard Tests ---

    #[tokio::test]
    async fn test_create_hazard_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_hazard(
            test_org_id(), "", "Chemical Spill Risk", None,
            "chemical", "possible", "major",
            None, None, None, None, None,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_hazard_validation_bad_category() {
        let engine = create_engine();
        let result = engine.create_hazard(
            test_org_id(), "HAZ-001", "Test", None,
            "nuclear", "possible", "major",
            None, None, None, None, None,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("hazard_category")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_hazard_validation_bad_likelihood() {
        let engine = create_engine();
        let result = engine.create_hazard(
            test_org_id(), "HAZ-001", "Test", None,
            "chemical", "frequent", "major",
            None, None, None, None, None,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("likelihood")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_hazard_success() {
        let engine = create_engine();
        let result = engine.create_hazard(
            test_org_id(), "HAZ-001", "Chemical Storage Area", Some("Improper chemical storage"),
            "chemical", "likely", "major",
            Some("Chemical Storage Room"), None, None,
            Some(test_user_id()), Some("Jane Safety"),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            Some(serde_json::json!([{"measure": "Install ventilation", "status": "planned"}])),
            Some(NaiveDate::from_ymd_opt(2024, 9, 1).unwrap()),
            Some(test_user_id()), Some("Safety Manager"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let haz = result.unwrap();
        assert_eq!(haz.hazard_code, "HAZ-001");
        assert_eq!(haz.hazard_category, "chemical");
        assert_eq!(haz.risk_score, 16); // likely(4) * major(4) = 16
        assert_eq!(haz.risk_level, "extreme");
        assert_eq!(haz.status, "identified");
    }

    #[tokio::test]
    async fn test_update_hazard_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_hazard_status(Uuid::new_v4(), "deleted").await;
        assert!(result.is_err());
    }

    // --- Inspection Tests ---

    #[tokio::test]
    async fn test_create_inspection_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_inspection(
            test_org_id(), "", "Monthly Fire Safety", None,
            "routine", "medium",
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_inspection_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_inspection(
            test_org_id(), "INS-001", "Test", None,
            "surprise", "medium",
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("inspection_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_inspection_success() {
        let engine = create_engine();
        let result = engine.create_inspection(
            test_org_id(), "INS-001", "Q2 Fire Safety Audit", Some("Quarterly fire safety inspection"),
            "periodic", "high",
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            Some("All Buildings"), None, None,
            Some(test_user_id()), Some("Inspector Bob"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let ins = result.unwrap();
        assert_eq!(ins.inspection_number, "INS-001");
        assert_eq!(ins.inspection_type, "periodic");
        assert_eq!(ins.status, "scheduled");
    }

    #[tokio::test]
    async fn test_update_inspection_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_inspection_status(Uuid::new_v4(), "approved").await;
        assert!(result.is_err());
    }

    // --- CAPA Tests ---

    #[tokio::test]
    async fn test_create_capa_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_corrective_action(
            test_org_id(), "", "Fix Wet Floor", None,
            "corrective", "high", Some("incident"), None, None,
            None, None, None, None, None,
            Some(NaiveDate::from_ymd_opt(2024, 7, 15).unwrap()),
            None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_capa_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_corrective_action(
            test_org_id(), "CAPA-001", "Test", None,
            "emergency", "high", None, None, None,
            None, None, None, None, None,
            None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("action_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_capa_validation_negative_cost() {
        let engine = create_engine();
        let result = engine.create_corrective_action(
            test_org_id(), "CAPA-001", "Test", None,
            "corrective", "high", None, None, None,
            None, None, None, None, None,
            None, None, None, Some(-100.0), Some("USD"),
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("cost")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_capa_success() {
        let engine = create_engine();
        let result = engine.create_corrective_action(
            test_org_id(), "CAPA-001", "Install Non-Slip Mats", Some("Install non-slip mats in all wet areas"),
            "corrective", "high", Some("incident"), None, Some("INC-001"),
            Some("Wet floor without warning signs"), Some("Install mats and warning signs"), None,
            Some(test_user_id()), Some("Facilities Manager"),
            Some(NaiveDate::from_ymd_opt(2024, 7, 15).unwrap()),
            None, None, Some(5000.0), Some("USD"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let capa = result.unwrap();
        assert_eq!(capa.action_number, "CAPA-001");
        assert_eq!(capa.action_type, "corrective");
        assert_eq!(capa.status, "open");
    }

    #[tokio::test]
    async fn test_update_capa_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_corrective_action_status(Uuid::new_v4(), "rejected").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_capa_validation_bad_effectiveness() {
        let engine = create_engine();
        let result = engine.complete_corrective_action(
            Uuid::new_v4(), "perfect", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("effectiveness")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_complete_capa_validation_negative_cost() {
        let engine = create_engine();
        let result = engine.complete_corrective_action(
            Uuid::new_v4(), "effective", Some(-50.0), None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("cost")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Dashboard ---

    #[tokio::test]
    async fn test_get_dashboard() {
        let engine = create_engine();
        let result = engine.get_dashboard(test_org_id()).await;
        assert!(result.is_ok());
        let dash = result.unwrap();
        assert_eq!(dash.organization_id, test_org_id());
    }
}
