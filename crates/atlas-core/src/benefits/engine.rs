//! Benefits Engine Implementation
//!
//! Manages benefits plans, employee enrollments, coverage tiers,
//! qualifying life events, and payroll deduction calculations.
//!
//! Oracle Fusion Cloud HCM equivalent: Benefits > Plans, Enrollments, Coverage

use atlas_shared::{
    BenefitsPlan, BenefitsEnrollment, BenefitsDeduction, BenefitsSummary,
    AtlasError, AtlasResult,
};
use super::BenefitsRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid plan types
const VALID_PLAN_TYPES: &[&str] = &[
    "medical", "dental", "vision", "life_insurance",
    "disability", "retirement", "hsa", "fsa",
];

/// Valid enrollment types
const VALID_ENROLLMENT_TYPES: &[&str] = &[
    "open_enrollment", "new_hire", "life_event", "manual",
];

/// Valid enrollment statuses
const VALID_ENROLLMENT_STATUSES: &[&str] = &[
    "pending", "active", "waived", "cancelled", "suspended",
];

/// Valid deduction frequencies
const VALID_DEDUCTION_FREQUENCIES: &[&str] = &[
    "per_pay_period", "monthly", "semi_monthly",
];

/// Benefits engine for managing plans, enrollments, and deductions
pub struct BenefitsEngine {
    repository: Arc<dyn BenefitsRepository>,
}

impl BenefitsEngine {
    pub fn new(repository: Arc<dyn BenefitsRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Benefits Plan Management
    // ========================================================================

    /// Create a new benefits plan
    pub async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        coverage_tiers: serde_json::Value,
        provider_name: Option<&str>,
        provider_plan_id: Option<&str>,
        plan_year_start: Option<chrono::NaiveDate>,
        plan_year_end: Option<chrono::NaiveDate>,
        open_enrollment_start: Option<chrono::NaiveDate>,
        open_enrollment_end: Option<chrono::NaiveDate>,
        allow_life_event_changes: bool,
        requires_eoi: bool,
        waiting_period_days: i32,
        max_dependents: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsPlan> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Plan code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Plan name is required".to_string(),
            ));
        }
        if !VALID_PLAN_TYPES.contains(&plan_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan_type '{}'. Must be one of: {}", plan_type, VALID_PLAN_TYPES.join(", ")
            )));
        }

        // Validate coverage tiers structure
        if !coverage_tiers.is_array() || coverage_tiers.as_array().map_or(true, |a| a.is_empty()) {
            return Err(AtlasError::ValidationFailed(
                "Coverage tiers must be a non-empty array".to_string(),
            ));
        }

        // Validate date ranges
        if let (Some(start), Some(end)) = (plan_year_start, plan_year_end) {
            if start > end {
                return Err(AtlasError::ValidationFailed(
                    "Plan year start must be before plan year end".to_string(),
                ));
            }
        }
        if let (Some(start), Some(end)) = (open_enrollment_start, open_enrollment_end) {
            if start > end {
                return Err(AtlasError::ValidationFailed(
                    "Open enrollment start must be before open enrollment end".to_string(),
                ));
            }
        }

        if waiting_period_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Waiting period days must be non-negative".to_string(),
            ));
        }

        info!("Creating benefits plan '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_plan(
            org_id, &code_upper, name, description, plan_type,
            coverage_tiers, provider_name, provider_plan_id,
            plan_year_start, plan_year_end,
            open_enrollment_start, open_enrollment_end,
            allow_life_event_changes, requires_eoi,
            waiting_period_days, max_dependents, created_by,
        ).await
    }

    /// Get a benefits plan by code
    pub async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BenefitsPlan>> {
        self.repository.get_plan(org_id, &code.to_uppercase()).await
    }

    /// List all plans for an organization
    pub async fn list_plans(&self, org_id: Uuid, plan_type: Option<&str>) -> AtlasResult<Vec<BenefitsPlan>> {
        if let Some(pt) = plan_type {
            if !VALID_PLAN_TYPES.contains(&pt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid plan_type filter '{}'. Must be one of: {}", pt, VALID_PLAN_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_plans(org_id, plan_type).await
    }

    /// Deactivate a plan
    pub async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating benefits plan '{}' for org {}", code, org_id);
        self.repository.delete_plan(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Benefits Enrollment Management
    // ========================================================================

    /// Enroll an employee in a benefits plan
    pub async fn enroll(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        plan_code: &str,
        coverage_tier: &str,
        enrollment_type: &str,
        effective_start_date: chrono::NaiveDate,
        effective_end_date: Option<chrono::NaiveDate>,
        deduction_frequency: &str,
        deduction_account_code: Option<&str>,
        employer_contribution_account_code: Option<&str>,
        dependents: Option<serde_json::Value>,
        life_event_reason: Option<&str>,
        life_event_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsEnrollment> {
        if !VALID_ENROLLMENT_TYPES.contains(&enrollment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid enrollment_type '{}'. Must be one of: {}", enrollment_type, VALID_ENROLLMENT_TYPES.join(", ")
            )));
        }
        if !VALID_DEDUCTION_FREQUENCIES.contains(&deduction_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid deduction_frequency '{}'. Must be one of: {}", deduction_frequency, VALID_DEDUCTION_FREQUENCIES.join(", ")
            )));
        }

        // Validate life event fields consistency
        if enrollment_type == "life_event" && life_event_reason.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Life event reason is required for life_event enrollment type".to_string(),
            ));
        }

        // Look up the plan
        let plan = self.get_plan(org_id, plan_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Benefits plan '{}' not found", plan_code)
            ))?;

        if !plan.is_active {
            return Err(AtlasError::ValidationFailed(
                format!("Benefits plan '{}' is not active", plan_code)
            ));
        }

        // Check if enrollment is within open enrollment period (for open_enrollment type)
        if enrollment_type == "open_enrollment" {
            if let (Some(oe_start), Some(oe_end)) = (plan.open_enrollment_start, plan.open_enrollment_end) {
                let today = chrono::Utc::now().date_naive();
                if today < oe_start || today > oe_end {
                    return Err(AtlasError::ValidationFailed(
                        "Current date is outside the open enrollment period".to_string(),
                    ));
                }
            }
        }

        // Check for life event changes permission
        if enrollment_type == "life_event" && !plan.allow_life_event_changes {
            return Err(AtlasError::ValidationFailed(
                format!("Plan '{}' does not allow mid-year life event changes", plan_code)
            ));
        }

        // Validate coverage tier exists in the plan
        let tier_found = plan.coverage_tiers.as_array()
            .map(|tiers| tiers.iter().any(|t| {
                t.get("tierCode").and_then(|v| v.as_str()).map_or(false, |s| s == coverage_tier)
            }))
            .unwrap_or(false);

        if !tier_found {
            return Err(AtlasError::ValidationFailed(format!(
                "Coverage tier '{}' is not available for plan '{}'", coverage_tier, plan_code
            )));
        }

        // Extract costs for the selected tier
        let (employee_cost, employer_cost, total_cost) = self.extract_tier_costs(&plan, coverage_tier)?;

        // Validate effective dates
        if let Some(end) = effective_end_date {
            if effective_start_date > end {
                return Err(AtlasError::ValidationFailed(
                    "Effective start date must be before effective end date".to_string(),
                ));
            }
        }

        // Check for duplicate active enrollment
        let existing = self.repository.get_active_enrollment(org_id, employee_id, plan.id).await?;
        if existing.is_some() {
            return Err(AtlasError::Conflict(
                format!("Employee {} already has an active enrollment in plan '{}'", employee_id, plan_code)
            ));
        }

        // Validate dependents
        if let Some(ref deps) = dependents {
            if let Some(arr) = deps.as_array() {
                if let Some(max_dep) = plan.max_dependents {
                    if arr.len() > max_dep as usize {
                        return Err(AtlasError::ValidationFailed(format!(
                            "Plan '{}' allows a maximum of {} dependents", plan_code, max_dep
                        )));
                    }
                }
            }
        }

        info!("Enrolling employee {} in plan '{}' with tier '{}'", employee_id, plan_code, coverage_tier);

        self.repository.create_enrollment(
            org_id, employee_id, employee_name,
            plan.id, Some(&plan.code), Some(&plan.name), Some(&plan.plan_type),
            coverage_tier, enrollment_type,
            "pending", effective_start_date, effective_end_date,
            &employee_cost, &employer_cost, &total_cost,
            deduction_frequency, deduction_account_code,
            employer_contribution_account_code,
            dependents.unwrap_or(serde_json::json!([])),
            life_event_reason, life_event_date,
            created_by,
        ).await
    }

    /// Get an enrollment by ID
    pub async fn get_enrollment(&self, id: Uuid) -> AtlasResult<Option<BenefitsEnrollment>> {
        self.repository.get_enrollment(id).await
    }

    /// List enrollments with optional filters
    pub async fn list_enrollments(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        plan_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BenefitsEnrollment>> {
        if let Some(s) = status {
            if !VALID_ENROLLMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_ENROLLMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_enrollments(org_id, employee_id, plan_id, status).await
    }

    /// Activate a pending enrollment (post-approval)
    pub async fn activate_enrollment(&self, enrollment_id: Uuid, processed_by: Uuid) -> AtlasResult<BenefitsEnrollment> {
        let enrollment = self.repository.get_enrollment(enrollment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Enrollment {} not found", enrollment_id)
            ))?;

        if enrollment.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate enrollment in '{}' status. Must be 'pending'.", enrollment.status)
            ));
        }

        info!("Activating enrollment {} by {}", enrollment_id, processed_by);
        self.repository.update_enrollment_status(enrollment_id, "active", Some(processed_by), None).await
    }

    /// Waive a pending enrollment
    pub async fn waive_enrollment(&self, enrollment_id: Uuid) -> AtlasResult<BenefitsEnrollment> {
        let enrollment = self.repository.get_enrollment(enrollment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Enrollment {} not found", enrollment_id)
            ))?;

        if enrollment.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot waive enrollment in '{}' status. Must be 'pending'.", enrollment.status)
            ));
        }

        info!("Waiving enrollment {}", enrollment_id);
        self.repository.update_enrollment_status(enrollment_id, "waived", None, None).await
    }

    /// Cancel an active enrollment
    pub async fn cancel_enrollment(
        &self,
        enrollment_id: Uuid,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BenefitsEnrollment> {
        let enrollment = self.repository.get_enrollment(enrollment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Enrollment {} not found", enrollment_id)
            ))?;

        if enrollment.status != "active" && enrollment.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel enrollment in '{}' status. Must be 'active' or 'pending'.", enrollment.status)
            ));
        }

        info!("Cancelling enrollment {} reason: {:?}", enrollment_id, cancellation_reason);
        self.repository.cancel_enrollment(enrollment_id, cancellation_reason).await
    }

    /// Suspend an active enrollment
    pub async fn suspend_enrollment(&self, enrollment_id: Uuid) -> AtlasResult<BenefitsEnrollment> {
        let enrollment = self.repository.get_enrollment(enrollment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Enrollment {} not found", enrollment_id)
            ))?;

        if enrollment.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot suspend enrollment in '{}' status. Must be 'active'.", enrollment.status)
            ));
        }

        info!("Suspending enrollment {}", enrollment_id);
        self.repository.update_enrollment_status(enrollment_id, "suspended", None, None).await
    }

    /// Reactivate a suspended enrollment
    pub async fn reactivate_enrollment(&self, enrollment_id: Uuid) -> AtlasResult<BenefitsEnrollment> {
        let enrollment = self.repository.get_enrollment(enrollment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Enrollment {} not found", enrollment_id)
            ))?;

        if enrollment.status != "suspended" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reactivate enrollment in '{}' status. Must be 'suspended'.", enrollment.status)
            ));
        }

        info!("Reactivating enrollment {}", enrollment_id);
        self.repository.update_enrollment_status(enrollment_id, "active", None, None).await
    }

    // ========================================================================
    // Benefits Deductions
    // ========================================================================

    /// Generate deductions for a pay period
    pub async fn generate_deductions(
        &self,
        org_id: Uuid,
        pay_period_start: chrono::NaiveDate,
        pay_period_end: chrono::NaiveDate,
    ) -> AtlasResult<Vec<BenefitsDeduction>> {
        if pay_period_start > pay_period_end {
            return Err(AtlasError::ValidationFailed(
                "Pay period start must be before pay period end".to_string(),
            ));
        }

        info!("Generating benefits deductions for period {} to {} org {}", pay_period_start, pay_period_end, org_id);

        // Get all active enrollments for this organization
        let enrollments = self.repository.list_enrollments(org_id, None, None, Some("active")).await?;

        let mut deductions = Vec::new();
        for enrollment in &enrollments {
            // Check if enrollment is effective during this pay period
            let effective = enrollment.effective_start_date <= pay_period_end
                && enrollment.effective_end_date.map_or(true, |end| end >= pay_period_start);

            if !effective {
                continue;
            }

            let deduction = self.repository.create_deduction(
                org_id, enrollment.id, enrollment.employee_id,
                enrollment.plan_id, enrollment.plan_code.as_deref(), enrollment.plan_name.as_deref(),
                &enrollment.employee_cost, &enrollment.employer_cost, &enrollment.total_cost,
                pay_period_start, pay_period_end,
                enrollment.deduction_account_code.as_deref(),
                enrollment.created_by,
            ).await?;

            deductions.push(deduction);
        }

        info!("Generated {} benefits deductions for org {}", deductions.len(), org_id);
        Ok(deductions)
    }

    /// List deductions with optional filters
    pub async fn list_deductions(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        enrollment_id: Option<Uuid>,
    ) -> AtlasResult<Vec<BenefitsDeduction>> {
        self.repository.list_deductions(org_id, employee_id, enrollment_id).await
    }

    /// Mark deductions as processed
    pub async fn mark_deductions_processed(&self, deduction_ids: &[Uuid]) -> AtlasResult<()> {
        for id in deduction_ids {
            self.repository.mark_deduction_processed(*id).await?;
        }
        info!("Marked {} deductions as processed", deduction_ids.len());
        Ok(())
    }

    // ========================================================================
    // Benefits Summary / Dashboard
    // ========================================================================

    /// Get a benefits summary dashboard for the organization
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<BenefitsSummary> {
        let plans = self.repository.list_plans(org_id, None).await?;
        let enrollments = self.repository.list_enrollments(org_id, None, None, None).await?;

        let total_plans = plans.len() as i32;
        let active_plans = plans.iter().filter(|p| p.is_active).count() as i32;
        let total_enrollments = enrollments.len() as i32;
        let active_enrollments = enrollments.iter().filter(|e| e.status == "active").count() as i32;
        let pending_enrollments = enrollments.iter().filter(|e| e.status == "pending").count() as i32;
        let waived_enrollments = enrollments.iter().filter(|e| e.status == "waived").count() as i32;

        // Calculate total costs from active enrollments
        let total_employee_cost: f64 = enrollments.iter()
            .filter(|e| e.status == "active")
            .map(|e| e.employee_cost.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_employer_cost: f64 = enrollments.iter()
            .filter(|e| e.status == "active")
            .map(|e| e.employer_cost.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Group enrollments by plan type
        let mut by_type = serde_json::Map::new();
        for enrollment in &enrollments {
            let plan_type = enrollment.plan_type.as_deref().unwrap_or("unknown");
            let count = by_type.get(plan_type)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) + 1;
            by_type.insert(plan_type.to_string(), serde_json::json!(count));
        }

        Ok(BenefitsSummary {
            total_plans,
            active_plans,
            total_enrollments,
            active_enrollments,
            pending_enrollments,
            waived_enrollments,
            total_employee_cost: format!("{:.2}", total_employee_cost),
            total_employer_cost: format!("{:.2}", total_employer_cost),
            enrollments_by_plan_type: serde_json::Value::Object(by_type),
        })
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Extract costs for a given coverage tier from the plan's tier definitions
    fn extract_tier_costs(&self, plan: &BenefitsPlan, tier_code: &str) -> AtlasResult<(String, String, String)> {
        let tiers = plan.coverage_tiers.as_array()
            .ok_or_else(|| AtlasError::Internal("Coverage tiers is not an array".to_string()))?;

        let tier = tiers.iter()
            .find(|t| t.get("tierCode").and_then(|v| v.as_str()) == Some(tier_code))
            .ok_or_else(|| AtlasError::ValidationFailed(
                format!("Coverage tier '{}' not found in plan", tier_code)
            ))?;

        let employee_cost = tier.get("employeeCost")
            .and_then(|v| v.as_str())
            .unwrap_or("0.00")
            .to_string();
        let employer_cost = tier.get("employerCost")
            .and_then(|v| v.as_str())
            .unwrap_or("0.00")
            .to_string();
        let total_cost = tier.get("totalCost")
            .and_then(|v| v.as_str())
            .unwrap_or("0.00")
            .to_string();

        Ok((employee_cost, employer_cost, total_cost))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"medical"));
        assert!(VALID_PLAN_TYPES.contains(&"dental"));
        assert!(VALID_PLAN_TYPES.contains(&"vision"));
        assert!(VALID_PLAN_TYPES.contains(&"life_insurance"));
        assert!(VALID_PLAN_TYPES.contains(&"disability"));
        assert!(VALID_PLAN_TYPES.contains(&"retirement"));
        assert!(VALID_PLAN_TYPES.contains(&"hsa"));
        assert!(VALID_PLAN_TYPES.contains(&"fsa"));
    }

    #[test]
    fn test_valid_enrollment_types() {
        assert!(VALID_ENROLLMENT_TYPES.contains(&"open_enrollment"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"new_hire"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"life_event"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"manual"));
    }

    #[test]
    fn test_valid_enrollment_statuses() {
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"pending"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"active"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"waived"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"cancelled"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"suspended"));
    }

    #[test]
    fn test_valid_deduction_frequencies() {
        assert!(VALID_DEDUCTION_FREQUENCIES.contains(&"per_pay_period"));
        assert!(VALID_DEDUCTION_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_DEDUCTION_FREQUENCIES.contains(&"semi_monthly"));
    }
}
