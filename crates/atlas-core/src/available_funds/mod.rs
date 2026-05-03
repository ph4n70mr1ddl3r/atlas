//! Available Funds Check Module
//!
//! Oracle Fusion Cloud ERP-inspired Budgetary Control / Available Funds Check.
//! Validates that sufficient budget/funds are available before a transaction
//! can be committed. Supports different tolerance levels and override workflows.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Budgetary Control > Available Funds

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Constants
// ============================================================================

const VALID_CHECK_RESULTS: &[&str] = &["pass", "warning", "fail", "advisory"];
const VALID_OVERRIDE_STATUSES: &[&str] = &["pending", "approved", "rejected", "expired"];
const VALID_TOLERANCE_LEVELS: &[&str] = &["none", "absolute", "percent"];
const VALID_FUND_TYPES: &[&str] = &["budget", "encumbrance", "actual", "available"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsCheckResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub check_number: String,
    pub budget_code: String,
    pub account_code: String,
    pub fund_type: String,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub budget_amount: String,
    pub encumbered_amount: String,
    pub actual_amount: String,
    pub available_amount: String,
    pub requested_amount: String,
    pub result_after: String,
    pub check_result: String,
    pub tolerance_type: Option<String>,
    pub tolerance_value: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub reference_number: Option<String>,
    pub metadata: serde_json::Value,
    pub checked_by: Option<Uuid>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsOverride {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub check_id: Uuid,
    pub override_number: String,
    pub reason: String,
    pub approval_level: String,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approval_notes: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub requested_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsCheckDashboard {
    pub organization_id: Uuid,
    pub total_checks: i32,
    pub passed_checks: i32,
    pub failed_checks: i32,
    pub warning_checks: i32,
    pub pending_overrides: i32,
    pub override_rate: String,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait AvailableFundsRepository: Send + Sync {
    async fn create_check(&self,
        org_id: Uuid, check_number: &str, budget_code: &str, account_code: &str,
        fund_type: &str, fiscal_year: i32, period_number: i32,
        budget_amount: &str, encumbered_amount: &str, actual_amount: &str,
        available_amount: &str, requested_amount: &str, result_after: &str,
        check_result: &str, tolerance_type: Option<&str>, tolerance_value: Option<&str>,
        reference_type: Option<&str>, reference_id: Option<Uuid>, reference_number: Option<&str>,
        checked_by: Option<Uuid>,
    ) -> AtlasResult<FundsCheckResult>;

    async fn get_check(&self, id: Uuid) -> AtlasResult<Option<FundsCheckResult>>;
    async fn list_checks(&self, org_id: Uuid, budget_code: Option<&str>, result: Option<&str>) -> AtlasResult<Vec<FundsCheckResult>>;
    async fn get_latest_check(&self, org_id: Uuid, budget_code: &str, account_code: &str) -> AtlasResult<Option<FundsCheckResult>>;

    async fn create_override(&self,
        org_id: Uuid, check_id: Uuid, override_number: &str,
        reason: &str, approval_level: &str, requested_by: Option<Uuid>,
    ) -> AtlasResult<FundsOverride>;
    async fn get_override(&self, id: Uuid) -> AtlasResult<Option<FundsOverride>>;
    async fn list_overrides(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FundsOverride>>;
    async fn approve_override(&self, id: Uuid, approved_by: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride>;
    async fn reject_override(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FundsCheckDashboard>;
}

/// PostgreSQL stub implementation
pub struct PostgresAvailableFundsRepository { pool: PgPool }
impl PostgresAvailableFundsRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl AvailableFundsRepository for PostgresAvailableFundsRepository {
    async fn create_check(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: i32, _: i32, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<FundsCheckResult> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_check(&self, _: Uuid) -> AtlasResult<Option<FundsCheckResult>> { Ok(None) }
    async fn list_checks(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<FundsCheckResult>> { Ok(vec![]) }
    async fn get_latest_check(&self, _: Uuid, _: &str, _: &str) -> AtlasResult<Option<FundsCheckResult>> { Ok(None) }
    async fn create_override(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: &str, _: Option<Uuid>) -> AtlasResult<FundsOverride> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_override(&self, _: Uuid) -> AtlasResult<Option<FundsOverride>> { Ok(None) }
    async fn list_overrides(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<FundsOverride>> { Ok(vec![]) }
    async fn approve_override(&self, _: Uuid, _: Uuid, _: Option<&str>) -> AtlasResult<FundsOverride> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn reject_override(&self, _: Uuid, _: Option<&str>) -> AtlasResult<FundsOverride> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FundsCheckDashboard> {
        Ok(FundsCheckDashboard { organization_id: org_id, total_checks: 0, passed_checks: 0, failed_checks: 0, warning_checks: 0, pending_overrides: 0, override_rate: "0".into() })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct AvailableFundsEngine {
    repository: Arc<dyn AvailableFundsRepository>,
}

impl AvailableFundsEngine {
    pub fn new(repository: Arc<dyn AvailableFundsRepository>) -> Self {
        Self { repository }
    }

    /// Check available funds for a transaction
    pub async fn check_funds(
        &self,
        org_id: Uuid,
        budget_code: &str,
        account_code: &str,
        fiscal_year: i32,
        period_number: i32,
        budget_amount: &str,
        encumbered_amount: &str,
        actual_amount: &str,
        requested_amount: &str,
        tolerance_type: Option<&str>,
        tolerance_value: Option<&str>,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        reference_number: Option<&str>,
        checked_by: Option<Uuid>,
    ) -> AtlasResult<FundsCheckResult> {
        if budget_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Budget code is required".into()));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".into()));
        }
        if fiscal_year <= 0 {
            return Err(AtlasError::ValidationFailed("Fiscal year must be positive".into()));
        }
        if period_number <= 0 || period_number > 13 {
            return Err(AtlasError::ValidationFailed("Period number must be 1-13".into()));
        }
        if let Some(tt) = tolerance_type {
            if !VALID_TOLERANCE_LEVELS.contains(&tt) {
                return Err(AtlasError::ValidationFailed(format!("Invalid tolerance type '{}'", tt)));
            }
        }

        let budget: f64 = budget_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid budget amount".into()))?;
        let encumbered: f64 = encumbered_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid encumbered amount".into()))?;
        let actual: f64 = actual_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid actual amount".into()))?;
        let requested: f64 = requested_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid requested amount".into()))?;

        if requested <= 0.0 {
            return Err(AtlasError::ValidationFailed("Requested amount must be positive".into()));
        }

        let available = budget - encumbered - actual;
        let result_after = available - requested;

        // Determine check result
        let check_result = if result_after >= 0.0 {
            "pass"
        } else if let Some(tt) = tolerance_type {
            let tol_val: f64 = tolerance_value.unwrap_or("0").parse().unwrap_or(0.0);
            let tolerance = match tt {
                "absolute" => tol_val,
                "percent" => budget * (tol_val / 100.0),
                _ => 0.0,
            };
            if result_after.abs() <= tolerance {
                "warning"
            } else {
                "fail"
            }
        } else {
            "fail"
        };

        let check_number = format!("FC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Funds check {} for budget {} account {}: {} (available={}, requested={}, after={})",
            check_number, budget_code, account_code, check_result, available, requested, result_after);

        self.repository.create_check(
            org_id, &check_number, budget_code, account_code,
            "budget", fiscal_year, period_number,
            budget_amount, encumbered_amount, actual_amount,
            &available.to_string(), requested_amount, &result_after.to_string(),
            check_result, tolerance_type, tolerance_value,
            reference_type, reference_id, reference_number,
            checked_by,
        ).await
    }

    /// Get check by ID
    pub async fn get_check(&self, id: Uuid) -> AtlasResult<Option<FundsCheckResult>> {
        self.repository.get_check(id).await
    }

    /// List checks
    pub async fn list_checks(&self, org_id: Uuid, budget_code: Option<&str>, result: Option<&str>) -> AtlasResult<Vec<FundsCheckResult>> {
        if let Some(r) = result {
            if !VALID_CHECK_RESULTS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!("Invalid check result '{}'", r)));
            }
        }
        self.repository.list_checks(org_id, budget_code, result).await
    }

    /// Get latest check for an account/budget
    pub async fn get_latest_check(&self, org_id: Uuid, budget_code: &str, account_code: &str) -> AtlasResult<Option<FundsCheckResult>> {
        self.repository.get_latest_check(org_id, budget_code, account_code).await
    }

    /// Request an override for a failed check
    pub async fn request_override(
        &self,
        org_id: Uuid,
        check_id: Uuid,
        reason: &str,
        approval_level: &str,
        requested_by: Option<Uuid>,
    ) -> AtlasResult<FundsOverride> {
        let check = self.repository.get_check(check_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Check {} not found", check_id)))?;
        if check.check_result != "fail" && check.check_result != "warning" {
            return Err(AtlasError::ValidationFailed("Override only allowed for failed or warning checks".into()));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Override reason is required".into()));
        }
        if approval_level.is_empty() {
            return Err(AtlasError::ValidationFailed("Approval level is required".into()));
        }

        let override_number = format!("FOV-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating funds override {} for check {}", override_number, check.check_number);
        self.repository.create_override(org_id, check_id, &override_number, reason, approval_level, requested_by).await
    }

    /// Get override
    pub async fn get_override(&self, id: Uuid) -> AtlasResult<Option<FundsOverride>> {
        self.repository.get_override(id).await
    }

    /// List overrides
    pub async fn list_overrides(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FundsOverride>> {
        if let Some(s) = status {
            if !VALID_OVERRIDE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid override status '{}'", s)));
            }
        }
        self.repository.list_overrides(org_id, status).await
    }

    /// Approve an override
    pub async fn approve_override(&self, id: Uuid, approved_by: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride> {
        let ov = self.repository.get_override(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Override {} not found", id)))?;
        if ov.status != "pending" {
            return Err(AtlasError::WorkflowError(format!("Cannot approve override in '{}' status", ov.status)));
        }
        info!("Approving funds override {}", ov.override_number);
        self.repository.approve_override(id, approved_by, notes).await
    }

    /// Reject an override
    pub async fn reject_override(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride> {
        let ov = self.repository.get_override(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Override {} not found", id)))?;
        if ov.status != "pending" {
            return Err(AtlasError::WorkflowError(format!("Cannot reject override in '{}' status", ov.status)));
        }
        info!("Rejecting funds override {}", ov.override_number);
        self.repository.reject_override(id, notes).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FundsCheckDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        checks: std::sync::Mutex<Vec<FundsCheckResult>>,
        overrides: std::sync::Mutex<Vec<FundsOverride>>,
    }
    impl MockRepo { fn new() -> Self { Self { checks: std::sync::Mutex::new(vec![]), overrides: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl AvailableFundsRepository for MockRepo {
        async fn create_check(&self, org_id: Uuid, check_number: &str, budget_code: &str, account_code: &str, fund_type: &str, fiscal_year: i32, period_number: i32, budget_amount: &str, encumbered_amount: &str, actual_amount: &str, available_amount: &str, requested_amount: &str, result_after: &str, check_result: &str, tolerance_type: Option<&str>, tolerance_value: Option<&str>, reference_type: Option<&str>, reference_id: Option<Uuid>, reference_number: Option<&str>, checked_by: Option<Uuid>) -> AtlasResult<FundsCheckResult> {
            let c = FundsCheckResult {
                id: Uuid::new_v4(), organization_id: org_id, check_number: check_number.into(),
                budget_code: budget_code.into(), account_code: account_code.into(),
                fund_type: fund_type.into(), fiscal_year, period_number,
                budget_amount: budget_amount.into(), encumbered_amount: encumbered_amount.into(),
                actual_amount: actual_amount.into(), available_amount: available_amount.into(),
                requested_amount: requested_amount.into(), result_after: result_after.into(),
                check_result: check_result.into(), tolerance_type: tolerance_type.map(Into::into),
                tolerance_value: tolerance_value.map(Into::into), reference_type: reference_type.map(Into::into),
                reference_id, reference_number: reference_number.map(Into::into),
                metadata: serde_json::json!({}), checked_by, checked_at: Utc::now(),
            };
            self.checks.lock().unwrap().push(c.clone());
            Ok(c)
        }
        async fn get_check(&self, id: Uuid) -> AtlasResult<Option<FundsCheckResult>> {
            Ok(self.checks.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }
        async fn list_checks(&self, org_id: Uuid, budget_code: Option<&str>, result: Option<&str>) -> AtlasResult<Vec<FundsCheckResult>> {
            Ok(self.checks.lock().unwrap().iter()
                .filter(|c| c.organization_id == org_id
                    && (budget_code.is_none() || c.budget_code == budget_code.unwrap())
                    && (result.is_none() || c.check_result == result.unwrap()))
                .cloned().collect())
        }
        async fn get_latest_check(&self, org_id: Uuid, budget_code: &str, account_code: &str) -> AtlasResult<Option<FundsCheckResult>> {
            Ok(self.checks.lock().unwrap().iter()
                .find(|c| c.organization_id == org_id && c.budget_code == budget_code && c.account_code == account_code)
                .cloned())
        }
        async fn create_override(&self, org_id: Uuid, check_id: Uuid, override_number: &str, reason: &str, approval_level: &str, requested_by: Option<Uuid>) -> AtlasResult<FundsOverride> {
            let o = FundsOverride {
                id: Uuid::new_v4(), organization_id: org_id, check_id, override_number: override_number.into(),
                reason: reason.into(), approval_level: approval_level.into(), status: "pending".into(),
                approved_by: None, approval_notes: None, expires_at: None, metadata: serde_json::json!({}),
                requested_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.overrides.lock().unwrap().push(o.clone());
            Ok(o)
        }
        async fn get_override(&self, id: Uuid) -> AtlasResult<Option<FundsOverride>> {
            Ok(self.overrides.lock().unwrap().iter().find(|o| o.id == id).cloned())
        }
        async fn list_overrides(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FundsOverride>> {
            Ok(self.overrides.lock().unwrap().iter()
                .filter(|o| o.organization_id == org_id && (status.is_none() || o.status == status.unwrap()))
                .cloned().collect())
        }
        async fn approve_override(&self, id: Uuid, approved_by: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride> {
            let mut all = self.overrides.lock().unwrap();
            let o = all.iter_mut().find(|o| o.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            o.status = "approved".into();
            o.approved_by = Some(approved_by);
            o.approval_notes = notes.map(Into::into);
            o.updated_at = Utc::now();
            Ok(o.clone())
        }
        async fn reject_override(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<FundsOverride> {
            let mut all = self.overrides.lock().unwrap();
            let o = all.iter_mut().find(|o| o.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            o.status = "rejected".into();
            o.approval_notes = notes.map(Into::into);
            o.updated_at = Utc::now();
            Ok(o.clone())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FundsCheckDashboard> {
            let checks = self.checks.lock().unwrap();
            let overrides = self.overrides.lock().unwrap();
            let org_checks: Vec<_> = checks.iter().filter(|c| c.organization_id == org_id).collect();
            let org_overrides: Vec<_> = overrides.iter().filter(|o| o.organization_id == org_id).collect();
            let total = org_checks.len() as i32;
            let passed = org_checks.iter().filter(|c| c.check_result == "pass").count() as i32;
            let failed = org_checks.iter().filter(|c| c.check_result == "fail").count() as i32;
            let pending = org_overrides.iter().filter(|o| o.status == "pending").count() as i32;
            let rate = if total > 0 { format!("{:.1}", (failed as f64 / total as f64) * 100.0) } else { "0".into() };
            Ok(FundsCheckDashboard {
                organization_id: org_id, total_checks: total, passed_checks: passed,
                failed_checks: failed, warning_checks: org_checks.iter().filter(|c| c.check_result == "warning").count() as i32,
                pending_overrides: pending, override_rate: rate,
            })
        }
    }

    fn eng() -> AvailableFundsEngine { AvailableFundsEngine::new(Arc::new(MockRepo::new())) }

    #[test]
    fn test_valid_check_results() {
        assert_eq!(VALID_CHECK_RESULTS.len(), 4);
        assert!(VALID_CHECK_RESULTS.contains(&"pass"));
        assert!(VALID_CHECK_RESULTS.contains(&"fail"));
    }

    #[tokio::test]
    async fn test_check_funds_pass() {
        let c = eng().check_funds(
            Uuid::new_v4(), "BUDGET-2026", "1000-100-1000", 2026, 5,
            "100000", "30000", "20000", "40000",
            None, None, Some("purchase_order"), Some(Uuid::new_v4()), Some("PO-001"), None,
        ).await.unwrap();
        assert_eq!(c.check_result, "pass");
        assert_eq!(c.available_amount, "50000"); // 100000 - 30000 - 20000
        assert_eq!(c.result_after, "10000"); // 50000 - 40000
    }

    #[tokio::test]
    async fn test_check_funds_fail() {
        let c = eng().check_funds(
            Uuid::new_v4(), "BUDGET-2026", "1000-100-1000", 2026, 5,
            "50000", "30000", "15000", "20000",
            None, None, Some("purchase_order"), Some(Uuid::new_v4()), Some("PO-002"), None,
        ).await.unwrap();
        assert_eq!(c.check_result, "fail");
        assert_eq!(c.available_amount, "5000"); // 50000 - 30000 - 15000
    }

    #[tokio::test]
    async fn test_check_funds_with_tolerance_warning() {
        let c = eng().check_funds(
            Uuid::new_v4(), "BUDGET-2026", "1000-100-1000", 2026, 5,
            "50000", "30000", "15000", "6000",
            Some("absolute"), Some("1000"), None, None, None, None,
        ).await.unwrap();
        // available = 5000, after = 5000 - 6000 = -1000, tolerance = 1000
        // |-1000| <= 1000 => warning
        assert_eq!(c.check_result, "warning");
    }

    #[tokio::test]
    async fn test_check_funds_with_percent_tolerance() {
        let c = eng().check_funds(
            Uuid::new_v4(), "BUDGET-2026", "1000-100-1000", 2026, 5,
            "100000", "80000", "15000", "10000",
            Some("percent"), Some("10"), None, None, None, None,
        ).await.unwrap();
        // available = 5000, after = 5000 - 10000 = -5000, tolerance = 100000 * 10% = 10000
        // |-5000| <= 10000 => warning
        assert_eq!(c.check_result, "warning");
    }

    #[tokio::test]
    async fn test_check_funds_empty_budget_code() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "", "1000", 2026, 5, "10000", "0", "0", "1000",
            None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_check_funds_empty_account_code() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "BUD", "", 2026, 5, "10000", "0", "0", "1000",
            None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_check_funds_invalid_year() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "BUD", "1000", 0, 5, "10000", "0", "0", "1000",
            None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_check_funds_invalid_period() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "BUD", "1000", 2026, 14, "10000", "0", "0", "1000",
            None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_check_funds_zero_requested() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "BUD", "1000", 2026, 5, "10000", "0", "0", "0",
            None, None, None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_check_funds_invalid_tolerance() {
        assert!(eng().check_funds(
            Uuid::new_v4(), "BUD", "1000", 2026, 5, "10000", "0", "0", "1000",
            Some("invalid"), Some("100"), None, None, None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_request_override() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "5000", "3000", "1000", "5000", None, None, None, None, None, None).await.unwrap();
        let ov = e.request_override(org, c.id, "Emergency procurement", "director", None).await.unwrap();
        assert_eq!(ov.status, "pending");
        assert_eq!(ov.reason, "Emergency procurement");
    }

    #[tokio::test]
    async fn test_request_override_pass_check() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "100000", "0", "0", "1000", None, None, None, None, None, None).await.unwrap();
        assert!(e.request_override(org, c.id, "reason", "level", None).await.is_err());
    }

    #[tokio::test]
    async fn test_request_override_empty_reason() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "5000", "3000", "1000", "5000", None, None, None, None, None, None).await.unwrap();
        assert!(e.request_override(org, c.id, "", "director", None).await.is_err());
    }

    #[tokio::test]
    async fn test_approve_override() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "5000", "3000", "1000", "5000", None, None, None, None, None, None).await.unwrap();
        let ov = e.request_override(org, c.id, "Emergency", "director", None).await.unwrap();
        let approved = e.approve_override(ov.id, Uuid::new_v4(), Some("Approved by CFO")).await.unwrap();
        assert_eq!(approved.status, "approved");
    }

    #[tokio::test]
    async fn test_reject_override() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "5000", "3000", "1000", "5000", None, None, None, None, None, None).await.unwrap();
        let ov = e.request_override(org, c.id, "Emergency", "director", None).await.unwrap();
        let rejected = e.reject_override(ov.id, Some("Insufficient justification")).await.unwrap();
        assert_eq!(rejected.status, "rejected");
    }

    #[tokio::test]
    async fn test_approve_override_not_pending() {
        let e = eng();
        let org = Uuid::new_v4();
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "5000", "3000", "1000", "5000", None, None, None, None, None, None).await.unwrap();
        let ov = e.request_override(org, c.id, "Emergency", "director", None).await.unwrap();
        let _ = e.approve_override(ov.id, Uuid::new_v4(), None).await.unwrap();
        assert!(e.approve_override(ov.id, Uuid::new_v4(), None).await.is_err());
    }

    #[tokio::test]
    async fn test_list_checks_invalid_result() {
        assert!(eng().list_checks(Uuid::new_v4(), None, Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_list_overrides_invalid_status() {
        assert!(eng().list_overrides(Uuid::new_v4(), Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_full_workflow() {
        let e = eng();
        let org = Uuid::new_v4();
        // Check fails
        let c = e.check_funds(org, "BUD", "1000", 2026, 5, "10000", "6000", "3000", "5000", None, None, Some("purchase_order"), None, None, None).await.unwrap();
        assert_eq!(c.check_result, "fail");

        // Request override
        let ov = e.request_override(org, c.id, "Critical need", "vp_finance", None).await.unwrap();
        assert_eq!(ov.status, "pending");

        // Approve override
        let approved = e.approve_override(ov.id, Uuid::new_v4(), Some("VP approved")).await.unwrap();
        assert_eq!(approved.status, "approved");

        // Dashboard
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_checks, 1);
        assert_eq!(dash.failed_checks, 1);
        assert_eq!(dash.pending_overrides, 0);
    }
}
