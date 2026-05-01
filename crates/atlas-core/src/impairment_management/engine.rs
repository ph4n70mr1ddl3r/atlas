//! Impairment Management Engine
//!
//! Core impairment operations:
//! - Impairment indicator management
//! - Impairment test creation (value-in-use, fair value less costs to sell)
//! - Cash flow projections for value-in-use calculation
//! - Discounted cash flow (DCF) calculations
//! - Test asset management and impairment recognition
//! - Workflow: draft -> submitted -> approved -> completed
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Fixed Assets > Impairment Management

use atlas_shared::{
    ImpairmentIndicator, ImpairmentTest, ImpairmentCashFlow, ImpairmentTestAsset,
    ImpairmentDashboardSummary,
    AtlasError, AtlasResult,
};
use super::ImpairmentManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid indicator types
#[allow(dead_code)]
const VALID_INDICATOR_TYPES: &[&str] = &["external", "internal", "market"];

/// Valid severity levels
#[allow(dead_code)]
const VALID_SEVERITIES: &[&str] = &["low", "medium", "high", "critical"];

/// Valid test types
#[allow(dead_code)]
const VALID_TEST_TYPES: &[&str] = &["individual", "cash_generating_unit"];

/// Valid test methods
#[allow(dead_code)]
const VALID_TEST_METHODS: &[&str] = &["value_in_use", "fair_value_less_costs"];

/// Valid test statuses
#[allow(dead_code)]
const VALID_TEST_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "completed", "reversed",
];

/// Valid asset statuses
#[allow(dead_code)]
const VALID_ASSET_STATUSES: &[&str] = &[
    "pending", "impaired", "not_impaired", "reversed",
];

/// Impairment Management Engine
pub struct ImpairmentManagementEngine {
    repository: Arc<dyn ImpairmentManagementRepository>,
}

impl ImpairmentManagementEngine {
    pub fn new(repository: Arc<dyn ImpairmentManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Impairment Indicators
    // ========================================================================

    /// Create an impairment indicator
    pub async fn create_indicator(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        indicator_type: &str,
        severity: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentIndicator> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Indicator code and name are required".to_string(),
            ));
        }
        if !VALID_INDICATOR_TYPES.contains(&indicator_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid indicator_type '{}'. Must be one of: {}", indicator_type, VALID_INDICATOR_TYPES.join(", ")
            )));
        }
        if !VALID_SEVERITIES.contains(&severity) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid severity '{}'. Must be one of: {}", severity, VALID_SEVERITIES.join(", ")
            )));
        }

        if self.repository.get_indicator_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Indicator code '{}' already exists", code)
            ));
        }

        info!("Creating impairment indicator '{}'", code);

        self.repository.create_indicator(
            org_id, code, name, description, indicator_type, severity, created_by,
        ).await
    }

    /// Get indicator by ID
    pub async fn get_indicator(&self, id: Uuid) -> AtlasResult<Option<ImpairmentIndicator>> {
        self.repository.get_indicator(id).await
    }

    /// List indicators
    pub async fn list_indicators(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ImpairmentIndicator>> {
        self.repository.list_indicators(org_id, active_only).await
    }

    // ========================================================================
    // Impairment Tests
    // ========================================================================

    /// Create an impairment test
    pub async fn create_test(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        test_type: &str,
        test_method: &str,
        test_date: chrono::NaiveDate,
        reporting_period: Option<&str>,
        indicator_id: Option<Uuid>,
        carrying_amount: &str,
        impairment_account: Option<&str>,
        reversal_account: Option<&str>,
        asset_id: Option<Uuid>,
        cgu_id: Option<Uuid>,
        discount_rate: Option<&str>,
        growth_rate: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentTest> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Test name is required".to_string(),
            ));
        }
        if !VALID_TEST_TYPES.contains(&test_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid test_type '{}'. Must be one of: {}", test_type, VALID_TEST_TYPES.join(", ")
            )));
        }
        if !VALID_TEST_METHODS.contains(&test_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid test_method '{}'. Must be one of: {}", test_method, VALID_TEST_METHODS.join(", ")
            )));
        }

        let carrying: f64 = carrying_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Carrying amount must be a valid number".to_string(),
        ))?;
        if carrying < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Carrying amount must be non-negative".to_string(),
            ));
        }

        let test_number = format!("IMP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating impairment test {} for asset/cgu", test_number);

        self.repository.create_test(
            org_id, &test_number, name, description, test_type, test_method,
            test_date, reporting_period, indicator_id, carrying_amount,
            "0", "0", // recoverable_amount, impairment_loss - will be calculated
            impairment_account, reversal_account, asset_id, cgu_id,
            discount_rate, growth_rate, None, // terminal_value
            created_by,
        ).await
    }

    /// Get test by ID
    pub async fn get_test(&self, id: Uuid) -> AtlasResult<Option<ImpairmentTest>> {
        self.repository.get_test(id).await
    }

    /// List tests
    pub async fn list_tests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ImpairmentTest>> {
        if let Some(s) = status {
            if !VALID_TEST_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TEST_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_tests(org_id, status).await
    }

    // ========================================================================
    // Cash Flow Projections
    // ========================================================================

    /// Add a cash flow projection for value-in-use calculation
    pub async fn add_cash_flow(
        &self,
        org_id: Uuid,
        test_id: Uuid,
        period_year: i32,
        period_number: i32,
        description: Option<&str>,
        cash_inflow: &str,
        cash_outflow: &str,
        discount_factor: Option<&str>,
    ) -> AtlasResult<ImpairmentCashFlow> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add cash flows to test in '{}' status. Must be 'draft'.", test.status)
            ));
        }

        if test.test_method != "value_in_use" {
            return Err(AtlasError::ValidationFailed(
                "Cash flow projections are only applicable for value-in-use method".to_string(),
            ));
        }

        let inflow: f64 = cash_inflow.parse().map_err(|_| AtlasError::ValidationFailed(
            "Cash inflow must be a valid number".to_string(),
        ))?;
        let outflow: f64 = cash_outflow.parse().map_err(|_| AtlasError::ValidationFailed(
            "Cash outflow must be a valid number".to_string(),
        ))?;
        let factor: f64 = discount_factor
            .map(|f| f.parse::<f64>().map_err(|_| AtlasError::ValidationFailed(
                "Discount factor must be a valid number".to_string(),
            )))
            .transpose()?
            .unwrap_or(1.0);

        let net = inflow - outflow;
        let pv = net * factor;

        info!("Adding cash flow projection for test {} year {} (net: {:.2}, PV: {:.2})",
            test.test_number, period_year, net, pv);

        self.repository.create_cash_flow(
            org_id, test_id, period_year, period_number, description,
            cash_inflow, cash_outflow, &format!("{:.2}", net),
            &format!("{:.6}", factor), &format!("{:.2}", pv),
        ).await
    }

    /// List cash flows for a test
    pub async fn list_cash_flows(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentCashFlow>> {
        self.repository.list_cash_flows(test_id).await
    }

    // ========================================================================
    // Test Asset Management
    // ========================================================================

    /// Add an asset to an impairment test
    pub async fn add_test_asset(
        &self,
        org_id: Uuid,
        test_id: Uuid,
        asset_id: Uuid,
        asset_number: Option<&str>,
        asset_name: Option<&str>,
        asset_category: Option<&str>,
        carrying_amount: &str,
    ) -> AtlasResult<ImpairmentTestAsset> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add assets to test in '{}' status. Must be 'draft'.", test.status)
            ));
        }

        let carrying: f64 = carrying_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Carrying amount must be a valid number".to_string(),
        ))?;
        if carrying < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Carrying amount must be non-negative".to_string(),
            ));
        }

        info!("Adding asset {} to impairment test {}", asset_id, test.test_number);

        self.repository.create_test_asset(
            org_id, test_id, asset_id, asset_number, asset_name, asset_category,
            carrying_amount, "0", "0", "pending", None,
        ).await
    }

    /// List test assets
    pub async fn list_test_assets(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentTestAsset>> {
        self.repository.list_test_assets(test_id).await
    }

    // ========================================================================
    // Calculations
    // ========================================================================

    /// Calculate value in use from cash flows (sum of present values)
    pub fn calculate_value_in_use(cash_flows: &[(f64, f64, f64)]) -> f64 {
        // Each tuple is (inflow, outflow, discount_factor)
        cash_flows.iter()
            .map(|(inflow, outflow, factor)| (inflow - outflow) * factor)
            .sum()
    }

    /// Calculate impairment loss
    /// Returns (recoverable_amount, impairment_loss)
    pub fn calculate_impairment(carrying_amount: f64, recoverable_amount: f64) -> (f64, f64) {
        if carrying_amount > recoverable_amount {
            (recoverable_amount, carrying_amount - recoverable_amount)
        } else {
            (recoverable_amount, 0.0)
        }
    }

    /// Calculate discount factor for a period
    /// factor = 1 / (1 + rate)^period
    pub fn calculate_discount_factor(discount_rate: f64, period: i32) -> f64 {
        if discount_rate <= -1.0 {
            return 0.0; // invalid rate
        }
        1.0 / (1.0 + discount_rate).powi(period)
    }

    /// Calculate terminal value (for value-in-use beyond projection period)
    /// TV = terminal_cash_flow / (discount_rate - growth_rate)
    pub fn calculate_terminal_value(
        terminal_cash_flow: f64,
        discount_rate: f64,
        growth_rate: f64,
    ) -> f64 {
        let spread = discount_rate - growth_rate;
        if spread <= 0.0 {
            return 0.0; // invalid or growth exceeds discount
        }
        terminal_cash_flow / spread
    }

    // ========================================================================
    // Test Workflow
    // ========================================================================

    /// Execute impairment test calculations
    pub async fn execute_test(&self, test_id: Uuid) -> AtlasResult<ImpairmentTest> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot execute test in '{}' status. Must be 'draft'.", test.status)
            ));
        }

        let mut recoverable_amount = 0.0;

        // For value-in-use: sum PV of cash flows
        if test.test_method == "value_in_use" {
            let cash_flows = self.repository.list_cash_flows(test_id).await?;
            let cf_data: Vec<(f64, f64, f64)> = cash_flows.iter()
                .map(|cf| {
                    let inflow: f64 = cf.cash_inflow.parse().unwrap_or(0.0);
                    let outflow: f64 = cf.cash_outflow.parse().unwrap_or(0.0);
                    let factor: f64 = cf.discount_factor.parse().unwrap_or(1.0);
                    (inflow, outflow, factor)
                })
                .collect();
            recoverable_amount = Self::calculate_value_in_use(&cf_data);

            // Add terminal value if available
            if let Some(tv_str) = &test.terminal_value {
                let tv: f64 = tv_str.parse().unwrap_or(0.0);
                recoverable_amount += tv;
            }
        }

        // Update recoverable amount
        self.repository.update_test_recoverable(
            test_id,
            &format!("{:.2}", recoverable_amount),
        ).await?;

        // Update test assets
        let assets = self.repository.list_test_assets(test_id).await?;
        for asset in &assets {
            let carrying: f64 = asset.carrying_amount.parse().unwrap_or(0.0);
            let (_, loss) = Self::calculate_impairment(carrying, recoverable_amount);
            let status = if loss > 0.0 { "impaired" } else { "not_impaired" };
            self.repository.update_test_asset(asset.id, &format!("{:.2}", recoverable_amount), &format!("{:.2}", loss), status).await?;
        }

        // Calculate total impairment
        let total_impairment: f64 = assets.iter()
            .map(|a| a.carrying_amount.parse::<f64>().unwrap_or(0.0))
            .sum::<f64>() - (recoverable_amount * assets.len() as f64).max(0.0);
        let total_impairment = total_impairment.max(0.0);

        info!("Executing impairment test {}: recoverable={:.2}, loss={:.2}",
            test.test_number, recoverable_amount, total_impairment);

        self.repository.update_test_results(
            test_id,
            &format!("{:.2}", recoverable_amount),
            &format!("{:.2}", total_impairment),
        ).await
    }

    /// Submit test for approval
    pub async fn submit_test(&self, test_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<ImpairmentTest> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit test in '{}' status. Must be 'draft'.", test.status)
            ));
        }

        info!("Submitting impairment test {}", test.test_number);
        self.repository.update_test_status(test_id, "submitted", submitted_by, None).await
    }

    /// Approve test
    pub async fn approve_test(&self, test_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ImpairmentTest> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve test in '{}' status. Must be 'submitted'.", test.status)
            ));
        }

        info!("Approving impairment test {}", test.test_number);
        self.repository.update_test_status(test_id, "approved", None, approved_by).await
    }

    /// Complete test
    pub async fn complete_test(&self, test_id: Uuid) -> AtlasResult<ImpairmentTest> {
        let test = self.repository.get_test(test_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Test {} not found", test_id)))?;

        if test.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete test in '{}' status. Must be 'approved'.", test.status)
            ));
        }

        info!("Completing impairment test {}", test.test_number);
        self.repository.update_test_status(test_id, "completed", None, None).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ImpairmentDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_indicator_types() {
        assert!(VALID_INDICATOR_TYPES.contains(&"external"));
        assert!(VALID_INDICATOR_TYPES.contains(&"internal"));
        assert!(VALID_INDICATOR_TYPES.contains(&"market"));
        assert_eq!(VALID_INDICATOR_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_severities() {
        assert!(VALID_SEVERITIES.contains(&"low"));
        assert!(VALID_SEVERITIES.contains(&"medium"));
        assert!(VALID_SEVERITIES.contains(&"high"));
        assert!(VALID_SEVERITIES.contains(&"critical"));
        assert_eq!(VALID_SEVERITIES.len(), 4);
    }

    #[test]
    fn test_valid_test_types() {
        assert!(VALID_TEST_TYPES.contains(&"individual"));
        assert!(VALID_TEST_TYPES.contains(&"cash_generating_unit"));
        assert_eq!(VALID_TEST_TYPES.len(), 2);
    }

    #[test]
    fn test_valid_test_methods() {
        assert!(VALID_TEST_METHODS.contains(&"value_in_use"));
        assert!(VALID_TEST_METHODS.contains(&"fair_value_less_costs"));
        assert_eq!(VALID_TEST_METHODS.len(), 2);
    }

    #[test]
    fn test_valid_test_statuses() {
        assert!(VALID_TEST_STATUSES.contains(&"draft"));
        assert!(VALID_TEST_STATUSES.contains(&"submitted"));
        assert!(VALID_TEST_STATUSES.contains(&"approved"));
        assert!(VALID_TEST_STATUSES.contains(&"completed"));
        assert!(VALID_TEST_STATUSES.contains(&"reversed"));
        assert_eq!(VALID_TEST_STATUSES.len(), 5);
    }

    #[test]
    fn test_valid_asset_statuses() {
        assert!(VALID_ASSET_STATUSES.contains(&"pending"));
        assert!(VALID_ASSET_STATUSES.contains(&"impaired"));
        assert!(VALID_ASSET_STATUSES.contains(&"not_impaired"));
        assert!(VALID_ASSET_STATUSES.contains(&"reversed"));
        assert_eq!(VALID_ASSET_STATUSES.len(), 4);
    }

    #[test]
    fn test_calculate_value_in_use() {
        // 3 years of cash flows with discount factors
        let cash_flows = vec![
            (100000.0, 30000.0, 0.9091),  // year 1: net 70000, PV 63637
            (110000.0, 35000.0, 0.8264),  // year 2: net 75000, PV 61980
            (120000.0, 40000.0, 0.7513),  // year 3: net 80000, PV 60104
        ];
        let viu = ImpairmentManagementEngine::calculate_value_in_use(&cash_flows);
        assert!(viu > 0.0);
        // Approximate: 63637 + 61980 + 60104 ≈ 185721
        assert!((viu - 185721.0).abs() < 100.0);
    }

    #[test]
    fn test_calculate_impairment_with_loss() {
        // Carrying > recoverable -> impairment
        let (recoverable, loss) = ImpairmentManagementEngine::calculate_impairment(100000.0, 75000.0);
        assert!((recoverable - 75000.0).abs() < 0.01);
        assert!((loss - 25000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_impairment_no_loss() {
        // Carrying <= recoverable -> no impairment
        let (recoverable, loss) = ImpairmentManagementEngine::calculate_impairment(50000.0, 75000.0);
        assert!((recoverable - 75000.0).abs() < 0.01);
        assert!((loss - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_impairment_equal() {
        let (_, loss) = ImpairmentManagementEngine::calculate_impairment(100000.0, 100000.0);
        assert!((loss - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_discount_factor() {
        // 10% rate, 1 period: 1/(1.1)^1 ≈ 0.9091
        let factor = ImpairmentManagementEngine::calculate_discount_factor(0.10, 1);
        assert!((factor - 0.9091).abs() < 0.001);

        // 10% rate, 5 periods: 1/(1.1)^5 ≈ 0.6209
        let factor = ImpairmentManagementEngine::calculate_discount_factor(0.10, 5);
        assert!((factor - 0.6209).abs() < 0.001);
    }

    #[test]
    fn test_calculate_discount_factor_zero_rate() {
        let factor = ImpairmentManagementEngine::calculate_discount_factor(0.0, 5);
        assert!((factor - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_terminal_value() {
        // TV = 10000 / (0.10 - 0.02) = 10000 / 0.08 = 125000
        let tv = ImpairmentManagementEngine::calculate_terminal_value(10000.0, 0.10, 0.02);
        assert!((tv - 125000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_terminal_value_invalid() {
        // Growth > discount -> returns 0
        let tv = ImpairmentManagementEngine::calculate_terminal_value(10000.0, 0.02, 0.10);
        assert!((tv - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_terminal_value_equal() {
        // Growth == discount -> returns 0
        let tv = ImpairmentManagementEngine::calculate_terminal_value(10000.0, 0.05, 0.05);
        assert!((tv - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_value_in_use_empty() {
        let viu = ImpairmentManagementEngine::calculate_value_in_use(&[]);
        assert!((viu - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_discount_factor_negative_rate() {
        // -2.0 is invalid (<= -1.0), should return 0
        let factor = ImpairmentManagementEngine::calculate_discount_factor(-2.0, 1);
        assert!((factor - 0.0).abs() < 0.001);
    }
}
