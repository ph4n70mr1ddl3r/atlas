//! Risk Management Engine
//!
//! Manages risk register, control registry, risk-control mappings,
//! control testing, issue/remediation tracking, and risk dashboard.
//!
//! Oracle Fusion Cloud equivalent: GRC > Risk Manager, Advanced Controls

use atlas_shared::{
    RiskCategory, RiskEntry, ControlEntry, RiskControlMapping,
    ControlTest, RiskIssue, RiskDashboard,
    AtlasError, AtlasResult,
};
use super::RiskManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_RISK_SOURCES: &[&str] = &[
    "operational", "financial", "compliance", "strategic", "reputational", "technology",
];

const VALID_RISK_STATUSES: &[&str] = &[
    "identified", "assessed", "mitigated", "accepted", "closed",
];

const VALID_RESPONSE_STRATEGIES: &[&str] = &[
    "avoid", "mitigate", "transfer", "accept",
];

const VALID_CONTROL_TYPES: &[&str] = &[
    "preventive", "detective", "corrective",
];

const VALID_CONTROL_NATURES: &[&str] = &[
    "automated", "manual", "it_dependent",
];

const VALID_CONTROL_FREQUENCIES: &[&str] = &[
    "daily", "weekly", "monthly", "quarterly", "annually", "per_transaction",
];

const VALID_CONTROL_STATUSES: &[&str] = &[
    "draft", "active", "inactive", "deprecated",
];

const VALID_EFFECTIVENESS: &[&str] = &[
    "effective", "ineffective", "not_tested",
];

const VALID_MITIGATION_EFFECTIVENESS: &[&str] = &[
    "full", "partial", "minimal",
];

const VALID_TEST_RESULTS: &[&str] = &[
    "pass", "fail", "not_tested", "in_progress",
];

const VALID_TEST_STATUSES: &[&str] = &[
    "planned", "in_progress", "completed", "cancelled",
];

const VALID_REVIEW_STATUSES: &[&str] = &[
    "pending", "approved", "rejected",
];

const VALID_DEFICIENCY_SEVERITIES: &[&str] = &[
    "minor", "significant", "material",
];

const VALID_ISSUE_SOURCES: &[&str] = &[
    "control_test", "risk_event", "audit_finding", "regulatory", "self_identified",
];

const VALID_SEVERITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

const VALID_PRIORITIES: &[&str] = &[
    "low", "normal", "high", "urgent",
];

const VALID_ISSUE_STATUSES: &[&str] = &[
    "open", "investigating", "remediation_in_progress", "resolved", "closed", "accepted",
];

/// Compute risk level from a risk score (likelihood * impact)
fn compute_risk_level(score: i32) -> String {
    match score {
        1..=4 => "low".to_string(),
        5..=9 => "medium".to_string(),
        10..=15 => "high".to_string(),
        16..=25 => "critical".to_string(),
        _ => "medium".to_string(),
    }
}

/// Risk Management Engine
pub struct RiskManagementEngine {
    repository: Arc<dyn RiskManagementRepository>,
}

impl RiskManagementEngine {
    pub fn new(repository: Arc<dyn RiskManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Risk Categories
    // ========================================================================

    /// Create a risk category
    pub async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        sort_order: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskCategory> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Category code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Category name is required".to_string(),
            ));
        }
        if self.repository.get_category_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Risk category '{}' already exists", code_upper
            )));
        }

        info!("Creating risk category '{}' ({}) for org {}", code_upper, name, org_id);
        self.repository.create_category(
            org_id, &code_upper, name, description,
            parent_category_id, sort_order.unwrap_or(0), created_by,
        ).await
    }

    /// Get a risk category by ID
    pub async fn get_category(&self, id: Uuid) -> AtlasResult<Option<RiskCategory>> {
        self.repository.get_category(id).await
    }

    /// Get a risk category by code
    pub async fn get_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskCategory>> {
        self.repository.get_category_by_code(org_id, code).await
    }

    /// List risk categories for an organization
    pub async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<RiskCategory>> {
        self.repository.list_categories(org_id).await
    }

    /// Delete a risk category by code
    pub async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting risk category '{}' for org {}", code, org_id);
        self.repository.delete_category(org_id, code).await
    }

    // ========================================================================
    // Risk Register
    // ========================================================================

    /// Create a risk register entry
    #[allow(clippy::too_many_arguments)]
    pub async fn create_risk(
        &self,
        org_id: Uuid,
        risk_number: &str,
        title: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        risk_source: &str,
        likelihood: i32,
        impact: i32,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        response_strategy: Option<&str>,
        business_units: Option<serde_json::Value>,
        related_entity_type: Option<&str>,
        related_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskEntry> {
        if risk_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Risk number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Risk title is required".to_string()));
        }
        if !(1..=5).contains(&likelihood) {
            return Err(AtlasError::ValidationFailed(
                "Likelihood must be between 1 and 5".to_string(),
            ));
        }
        if !(1..=5).contains(&impact) {
            return Err(AtlasError::ValidationFailed(
                "Impact must be between 1 and 5".to_string(),
            ));
        }
        if !VALID_RISK_SOURCES.contains(&risk_source) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_source '{}'. Must be one of: {}", risk_source, VALID_RISK_SOURCES.join(", ")
            )));
        }
        if let Some(rs) = response_strategy {
            if !VALID_RESPONSE_STRATEGIES.contains(&rs) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid response_strategy '{}'. Must be one of: {}", rs, VALID_RESPONSE_STRATEGIES.join(", ")
                )));
            }
        }

        // Check for duplicate risk_number
        if self.repository.get_risk_by_number(org_id, risk_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Risk '{}' already exists", risk_number
            )));
        }

        let risk_level = compute_risk_level(likelihood * impact);
        let bu = business_units.unwrap_or(serde_json::json!([]));

        info!("Creating risk '{}' ({}) for org {} [score={}, level={}]",
              risk_number, title, org_id, likelihood * impact, risk_level);

        self.repository.create_risk(
            org_id, risk_number, title, description, category_id,
            risk_source, likelihood, impact, &risk_level,
            owner_id, owner_name, response_strategy,
            bu.as_object().map(|o| serde_json::Value::Object(o.clone())).unwrap_or(serde_json::json!([])),
            related_entity_type, related_entity_id, created_by,
        ).await
    }

    /// Get a risk by ID
    pub async fn get_risk(&self, id: Uuid) -> AtlasResult<Option<RiskEntry>> {
        self.repository.get_risk(id).await
    }

    /// Get a risk by number
    pub async fn get_risk_by_number(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<Option<RiskEntry>> {
        self.repository.get_risk_by_number(org_id, risk_number).await
    }

    /// List risks for an organization with optional filters
    pub async fn list_risks(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_level: Option<&str>,
        risk_source: Option<&str>,
    ) -> AtlasResult<Vec<RiskEntry>> {
        self.repository.list_risks(org_id, status, risk_level, risk_source).await
    }

    /// Update risk status
    pub async fn update_risk_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskEntry> {
        if !VALID_RISK_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk status '{}'. Must be one of: {}", status, VALID_RISK_STATUSES.join(", ")
            )));
        }
        info!("Updating risk {} status to {}", id, status);
        self.repository.update_risk_status(id, status).await
    }

    /// Assess (re-score) a risk
    pub async fn assess_risk(
        &self,
        id: Uuid,
        likelihood: i32,
        impact: i32,
        residual_likelihood: Option<i32>,
        residual_impact: Option<i32>,
    ) -> AtlasResult<RiskEntry> {
        if !(1..=5).contains(&likelihood) {
            return Err(AtlasError::ValidationFailed("Likelihood must be 1-5".to_string()));
        }
        if !(1..=5).contains(&impact) {
            return Err(AtlasError::ValidationFailed("Impact must be 1-5".to_string()));
        }
        if let Some(rl) = residual_likelihood {
            if !(1..=5).contains(&rl) {
                return Err(AtlasError::ValidationFailed("Residual likelihood must be 1-5".to_string()));
            }
        }
        if let Some(ri) = residual_impact {
            if !(1..=5).contains(&ri) {
                return Err(AtlasError::ValidationFailed("Residual impact must be 1-5".to_string()));
            }
        }
        let risk_level = compute_risk_level(likelihood * impact);
        info!("Assessing risk {} [L={}, I={}, score={}, level={}]",
              id, likelihood, impact, likelihood * impact, risk_level);
        self.repository.assess_risk(
            id, likelihood, impact, &risk_level,
            residual_likelihood, residual_impact,
        ).await
    }

    /// Delete a risk by number
    pub async fn delete_risk(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<()> {
        info!("Deleting risk '{}' for org {}", risk_number, org_id);
        self.repository.delete_risk(org_id, risk_number).await
    }

    // ========================================================================
    // Control Registry
    // ========================================================================

    /// Create a control
    #[allow(clippy::too_many_arguments)]
    pub async fn create_control(
        &self,
        org_id: Uuid,
        control_number: &str,
        title: &str,
        description: Option<&str>,
        control_type: &str,
        control_nature: &str,
        frequency: &str,
        objective: Option<&str>,
        test_procedures: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        is_key_control: bool,
        business_processes: Option<serde_json::Value>,
        regulatory_frameworks: Option<serde_json::Value>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlEntry> {
        if control_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Control number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Control title is required".to_string()));
        }
        if !VALID_CONTROL_TYPES.contains(&control_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid control_type '{}'. Must be one of: {}", control_type, VALID_CONTROL_TYPES.join(", ")
            )));
        }
        if !VALID_CONTROL_NATURES.contains(&control_nature) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid control_nature '{}'. Must be one of: {}", control_nature, VALID_CONTROL_NATURES.join(", ")
            )));
        }
        if !VALID_CONTROL_FREQUENCIES.contains(&frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid frequency '{}'. Must be one of: {}", frequency, VALID_CONTROL_FREQUENCIES.join(", ")
            )));
        }

        if self.repository.get_control_by_number(org_id, control_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Control '{}' already exists", control_number
            )));
        }

        info!("Creating control '{}' ({}) for org {}", control_number, title, org_id);

        self.repository.create_control(
            org_id, control_number, title, description,
            control_type, control_nature, frequency, objective, test_procedures,
            owner_id, owner_name, is_key_control,
            business_processes.unwrap_or(serde_json::json!([])),
            regulatory_frameworks.unwrap_or(serde_json::json!([])),
            created_by,
        ).await
    }

    /// Get a control by ID
    pub async fn get_control(&self, id: Uuid) -> AtlasResult<Option<ControlEntry>> {
        self.repository.get_control(id).await
    }

    /// Get a control by number
    pub async fn get_control_by_number(&self, org_id: Uuid, control_number: &str) -> AtlasResult<Option<ControlEntry>> {
        self.repository.get_control_by_number(org_id, control_number).await
    }

    /// List controls with optional filters
    pub async fn list_controls(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        control_type: Option<&str>,
    ) -> AtlasResult<Vec<ControlEntry>> {
        self.repository.list_controls(org_id, status, control_type).await
    }

    /// Update control status
    pub async fn update_control_status(&self, id: Uuid, status: &str) -> AtlasResult<ControlEntry> {
        if !VALID_CONTROL_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid control status '{}'. Must be one of: {}", status, VALID_CONTROL_STATUSES.join(", ")
            )));
        }
        info!("Updating control {} status to {}", id, status);
        self.repository.update_control_status(id, status).await
    }

    /// Update control effectiveness
    pub async fn update_control_effectiveness(&self, id: Uuid, effectiveness: &str) -> AtlasResult<ControlEntry> {
        if !VALID_EFFECTIVENESS.contains(&effectiveness) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid effectiveness '{}'. Must be one of: {}", effectiveness, VALID_EFFECTIVENESS.join(", ")
            )));
        }
        info!("Updating control {} effectiveness to {}", id, effectiveness);
        self.repository.update_control_effectiveness(id, effectiveness).await
    }

    /// Delete a control by number
    pub async fn delete_control(&self, org_id: Uuid, control_number: &str) -> AtlasResult<()> {
        info!("Deleting control '{}' for org {}", control_number, org_id);
        self.repository.delete_control(org_id, control_number).await
    }

    // ========================================================================
    // Risk-Control Mappings
    // ========================================================================

    /// Create a risk-control mapping
    pub async fn create_risk_control_mapping(
        &self,
        org_id: Uuid,
        risk_id: Uuid,
        control_id: Uuid,
        mitigation_effectiveness: &str,
        description: Option<&str>,
        mapped_by: Option<Uuid>,
    ) -> AtlasResult<RiskControlMapping> {
        if !VALID_MITIGATION_EFFECTIVENESS.contains(&mitigation_effectiveness) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid mitigation_effectiveness '{}'. Must be one of: {}",
                mitigation_effectiveness, VALID_MITIGATION_EFFECTIVENESS.join(", ")
            )));
        }
        // Verify risk exists
        self.repository.get_risk(risk_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Risk {} not found", risk_id)))?;
        // Verify control exists
        self.repository.get_control(control_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Control {} not found", control_id)))?;

        info!("Mapping risk {} to control {} [{}]", risk_id, control_id, mitigation_effectiveness);
        self.repository.create_risk_control_mapping(
            org_id, risk_id, control_id, mitigation_effectiveness,
            description, mapped_by,
        ).await
    }

    /// List mappings for a risk
    pub async fn list_risk_mappings(&self, risk_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>> {
        self.repository.list_risk_mappings(risk_id).await
    }

    /// List mappings for a control
    pub async fn list_control_mappings(&self, control_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>> {
        self.repository.list_control_mappings(control_id).await
    }

    /// Delete a mapping
    pub async fn delete_mapping(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_mapping(id).await
    }

    // ========================================================================
    // Control Tests
    // ========================================================================

    /// Create a control test
    #[allow(clippy::too_many_arguments)]
    pub async fn create_control_test(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        test_number: &str,
        test_plan: &str,
        test_period_start: chrono::NaiveDate,
        test_period_end: chrono::NaiveDate,
        tester_id: Option<Uuid>,
        tester_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlTest> {
        if test_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Test number is required".to_string()));
        }
        if test_plan.is_empty() {
            return Err(AtlasError::ValidationFailed("Test plan is required".to_string()));
        }
        if test_period_start > test_period_end {
            return Err(AtlasError::ValidationFailed(
                "Test period start must be before end".to_string(),
            ));
        }
        // Verify control exists
        self.repository.get_control(control_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Control {} not found", control_id)))?;

        info!("Creating control test '{}' for control {}", test_number, control_id);
        self.repository.create_control_test(
            org_id, control_id, test_number, test_plan,
            test_period_start, test_period_end,
            tester_id, tester_name, created_by,
        ).await
    }

    /// Get a control test by ID
    pub async fn get_control_test(&self, id: Uuid) -> AtlasResult<Option<ControlTest>> {
        self.repository.get_control_test(id).await
    }

    /// List tests for a control
    pub async fn list_control_tests(&self, control_id: Uuid) -> AtlasResult<Vec<ControlTest>> {
        self.repository.list_control_tests(control_id).await
    }

    /// Start a control test (change status to in_progress)
    pub async fn start_control_test(&self, id: Uuid) -> AtlasResult<ControlTest> {
        info!("Starting control test {}", id);
        self.repository.update_control_test_status(id, "in_progress").await
    }

    /// Complete a control test with results
    #[allow(clippy::too_many_arguments)]
    pub async fn complete_control_test(
        &self,
        id: Uuid,
        result: &str,
        findings: Option<&str>,
        deficiency_severity: Option<&str>,
        sample_size: Option<i32>,
        sample_exceptions: Option<i32>,
    ) -> AtlasResult<ControlTest> {
        if !VALID_TEST_RESULTS.contains(&result) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid test result '{}'. Must be one of: {}", result, VALID_TEST_RESULTS.join(", ")
            )));
        }
        if let Some(ds) = deficiency_severity {
            if !VALID_DEFICIENCY_SEVERITIES.contains(&ds) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid deficiency_severity '{}'. Must be one of: {}",
                    ds, VALID_DEFICIENCY_SEVERITIES.join(", ")
                )));
            }
        }
        info!("Completing control test {} with result {}", id, result);
        self.repository.complete_control_test(
            id, result, findings, deficiency_severity,
            sample_size, sample_exceptions,
        ).await
    }

    /// Delete a control test by number
    pub async fn delete_control_test(&self, org_id: Uuid, test_number: &str) -> AtlasResult<()> {
        info!("Deleting control test '{}' for org {}", test_number, org_id);
        self.repository.delete_control_test(org_id, test_number).await
    }

    // ========================================================================
    // Issues & Remediations
    // ========================================================================

    /// Create a risk issue
    #[allow(clippy::too_many_arguments)]
    pub async fn create_issue(
        &self,
        org_id: Uuid,
        issue_number: &str,
        title: &str,
        description: &str,
        source: &str,
        risk_id: Option<Uuid>,
        control_id: Option<Uuid>,
        control_test_id: Option<Uuid>,
        severity: &str,
        priority: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        remediation_plan: Option<&str>,
        remediation_due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskIssue> {
        if issue_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Issue number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Issue title is required".to_string()));
        }
        if description.is_empty() {
            return Err(AtlasError::ValidationFailed("Issue description is required".to_string()));
        }
        if !VALID_ISSUE_SOURCES.contains(&source) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source '{}'. Must be one of: {}", source, VALID_ISSUE_SOURCES.join(", ")
            )));
        }
        if !VALID_SEVERITIES.contains(&severity) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid severity '{}'. Must be one of: {}", severity, VALID_SEVERITIES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        if self.repository.get_issue_by_number(org_id, issue_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Issue '{}' already exists", issue_number
            )));
        }

        info!("Creating issue '{}' ({}) for org {} [severity={}, priority={}]",
              issue_number, title, org_id, severity, priority);

        self.repository.create_issue(
            org_id, issue_number, title, description, source,
            risk_id, control_id, control_test_id, severity, priority,
            owner_id, owner_name, remediation_plan, remediation_due_date, created_by,
        ).await
    }

    /// Get an issue by ID
    pub async fn get_issue(&self, id: Uuid) -> AtlasResult<Option<RiskIssue>> {
        self.repository.get_issue(id).await
    }

    /// Get an issue by number
    pub async fn get_issue_by_number(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<Option<RiskIssue>> {
        self.repository.get_issue_by_number(org_id, issue_number).await
    }

    /// List issues with optional filters
    pub async fn list_issues(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        severity: Option<&str>,
    ) -> AtlasResult<Vec<RiskIssue>> {
        self.repository.list_issues(org_id, status, severity).await
    }

    /// Update issue status
    pub async fn update_issue_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskIssue> {
        if !VALID_ISSUE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid issue status '{}'. Must be one of: {}", status, VALID_ISSUE_STATUSES.join(", ")
            )));
        }
        info!("Updating issue {} status to {}", id, status);
        self.repository.update_issue_status(id, status).await
    }

    /// Resolve an issue with corrective actions
    pub async fn resolve_issue(
        &self,
        id: Uuid,
        root_cause: Option<&str>,
        corrective_actions: Option<&str>,
    ) -> AtlasResult<RiskIssue> {
        info!("Resolving issue {}", id);
        self.repository.resolve_issue(id, root_cause, corrective_actions).await
    }

    /// Delete an issue by number
    pub async fn delete_issue(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<()> {
        info!("Deleting issue '{}' for org {}", issue_number, org_id);
        self.repository.delete_issue(org_id, issue_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the risk management dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RiskDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}
