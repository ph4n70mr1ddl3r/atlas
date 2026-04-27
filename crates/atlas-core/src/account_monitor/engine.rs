//! Account Monitor & Balance Inquiry Engine
//!
//! Manages account groups, balance snapshots, threshold alerting,
//! and saved balance inquiries.
//!
//! Oracle Fusion equivalent: General Ledger > Journals > Account Monitor

use atlas_shared::{
    AccountGroup, AccountGroupMember, BalanceSnapshot, SavedBalanceInquiry,
    AccountMonitorSummary,
    AtlasError, AtlasResult,
};
use super::AccountMonitorRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid comparison types
const VALID_COMPARISON_TYPES: &[&str] = &[
    "prior_period", "prior_year", "budget",
];

/// Valid amount types for saved inquiries
const VALID_AMOUNT_TYPES: &[&str] = &[
    "beginning_balance", "ending_balance", "net_activity",
    "debits", "credits",
];

/// Account Monitor Engine
pub struct AccountMonitorEngine {
    repository: Arc<dyn AccountMonitorRepository>,
}

impl AccountMonitorEngine {
    pub fn new(repository: Arc<dyn AccountMonitorRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Account Group Management
    // ========================================================================

    /// Create a new account group
    #[allow(clippy::too_many_arguments)]
    pub async fn create_account_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        owner_id: Option<Uuid>,
        is_shared: bool,
        threshold_warning_pct: Option<&str>,
        threshold_critical_pct: Option<&str>,
        comparison_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountGroup> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Account group code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account group name is required".to_string(),
            ));
        }
        if !VALID_COMPARISON_TYPES.contains(&comparison_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid comparison_type '{}'. Must be one of: {}",
                comparison_type,
                VALID_COMPARISON_TYPES.join(", ")
            )));
        }
        if let Some(wp) = threshold_warning_pct {
            if wp.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Warning threshold must be a valid number".to_string(),
                ));
            }
        }
        if let Some(cp) = threshold_critical_pct {
            if cp.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Critical threshold must be a valid number".to_string(),
                ));
            }
        }

        // Check for duplicate code
        if self.repository.get_account_group_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Account group with code '{}' already exists", code_upper
            )));
        }

        info!("Creating account group '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_account_group(
            org_id, &code_upper, name, description, owner_id, is_shared,
            threshold_warning_pct, threshold_critical_pct, comparison_type, created_by,
        ).await
    }

    /// Get an account group by ID
    pub async fn get_account_group(&self, id: Uuid) -> AtlasResult<Option<AccountGroup>> {
        self.repository.get_account_group(id).await
    }

    /// Get an account group by code
    pub async fn get_account_group_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountGroup>> {
        self.repository.get_account_group_by_code(org_id, code).await
    }

    /// List account groups for an organization
    pub async fn list_account_groups(
        &self,
        org_id: Uuid,
        owner_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AccountGroup>> {
        self.repository.list_account_groups(org_id, owner_id).await
    }

    /// Delete an account group by code
    pub async fn delete_account_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting account group '{}' for org {}", code, org_id);
        self.repository.delete_account_group(org_id, code).await
    }

    // ========================================================================
    // Account Group Members
    // ========================================================================

    /// Add a member to an account group
    #[allow(clippy::too_many_arguments)]
    pub async fn add_group_member(
        &self,
        group_id: Uuid,
        account_segment: &str,
        account_label: Option<&str>,
        display_order: i32,
        include_children: bool,
    ) -> AtlasResult<AccountGroupMember> {
        if account_segment.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account segment is required".to_string(),
            ));
        }

        // Verify group exists
        let _group = self.repository.get_account_group(group_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Account group {} not found", group_id
            )))?;

        info!("Adding member '{}' to account group {}", account_segment, group_id);

        self.repository.add_group_member(
            group_id, account_segment, account_label, display_order, include_children,
        ).await
    }

    /// Remove a member from an account group
    pub async fn remove_group_member(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_group_member(id).await
    }

    /// List members of an account group
    pub async fn list_group_members(&self, group_id: Uuid) -> AtlasResult<Vec<AccountGroupMember>> {
        self.repository.list_group_members(group_id).await
    }

    // ========================================================================
    // Balance Snapshots
    // ========================================================================

    /// Capture a balance snapshot for all members of an account group
    pub async fn capture_snapshot(
        &self,
        org_id: Uuid,
        group_id: Uuid,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        fiscal_year: i32,
        period_number: i32,
    ) -> AtlasResult<Vec<BalanceSnapshot>> {
        let group = self.repository.get_account_group(group_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Account group {} not found", group_id
            )))?;

        if group.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot capture snapshot for inactive account group".to_string(),
            ));
        }

        let members = self.repository.list_group_members(group_id).await?;
        if members.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account group has no members".to_string(),
            ));
        }

        info!(
            "Capturing balance snapshot for group '{}' ({} members) in period {}",
            group.code, members.len(), period_name
        );

        let mut snapshots = Vec::new();
        let warning_pct = group.threshold_warning_pct.as_deref()
            .and_then(|v| v.parse::<f64>().ok());
        let critical_pct = group.threshold_critical_pct.as_deref()
            .and_then(|v| v.parse::<f64>().ok());

        for member in &members {
            // Generate a deterministic balance based on account segment hash
            // In production, this would query actual GL balances
            let (beginning, debits, credits, je_count) = compute_mock_balance(
                &member.account_segment, fiscal_year, period_number,
            );
            let net = debits - credits;
            let ending = beginning + net;

            // Determine comparison balance based on comparison type
            let (comp_balance, comp_period) = match group.comparison_type.as_str() {
                "prior_period" => {
                    let prev = if period_number > 1 { period_number - 1 } else { 12 };
                    let (b, d, c, _) = compute_mock_balance(&member.account_segment, fiscal_year, prev);
                    let comp_end = b + (d - c);
                    (Some(comp_end), Some(format!("P{}", prev)))
                }
                "prior_year" => {
                    let (b, d, c, _) = compute_mock_balance(&member.account_segment, fiscal_year - 1, period_number);
                    let comp_end = b + (d - c);
                    (Some(comp_end), Some(format!("FY{}-P{}", fiscal_year - 1, period_number)))
                }
                _ => (None, None),
            };

            // Compute variance
            let (variance_amt, variance_pct, alert) = compute_variance(
                ending, comp_balance, warning_pct, critical_pct,
            );

            let snapshot = self.repository.create_balance_snapshot(
                org_id, group_id, Some(member.id), &member.account_segment,
                period_name, period_start, period_end, fiscal_year, period_number,
                &format!("{:.4}", beginning),
                &format!("{:.4}", debits),
                &format!("{:.4}", credits),
                &format!("{:.4}", net),
                &format!("{:.4}", ending),
                je_count,
                comp_balance.map(|v| format!("{:.4}", v)).as_deref(),
                comp_period.as_deref(),
                variance_amt.map(|v| format!("{:.4}", v)).as_deref(),
                variance_pct.map(|v| format!("{:.4}", v)).as_deref(),
                &alert,
            ).await?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }

    /// Get balance snapshots for a group
    pub async fn get_group_snapshots(
        &self,
        group_id: Uuid,
        snapshot_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<BalanceSnapshot>> {
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        self.repository.get_group_snapshots(group_id, snapshot_date, limit, offset).await
    }

    /// Get balance snapshots that have alerts
    pub async fn get_alert_snapshots(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<Vec<BalanceSnapshot>> {
        self.repository.get_alert_snapshots(org_id).await
    }

    /// Delete a balance snapshot
    pub async fn delete_snapshot(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_snapshot(id).await
    }

    // ========================================================================
    // Saved Balance Inquiries
    // ========================================================================

    /// Create a saved balance inquiry
    #[allow(clippy::too_many_arguments)]
    pub async fn create_saved_inquiry(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        account_segments: serde_json::Value,
        period_from: &str,
        period_to: &str,
        currency_code: &str,
        amount_type: &str,
        include_zero_balances: bool,
        comparison_enabled: bool,
        comparison_type: Option<&str>,
        sort_by: &str,
        sort_direction: &str,
        is_shared: bool,
    ) -> AtlasResult<SavedBalanceInquiry> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Inquiry name is required".to_string(),
            ));
        }
        if period_from.is_empty() || period_to.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Period from and to are required".to_string(),
            ));
        }
        if !VALID_AMOUNT_TYPES.contains(&amount_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid amount_type '{}'. Must be one of: {}",
                amount_type,
                VALID_AMOUNT_TYPES.join(", ")
            )));
        }
        if sort_direction != "asc" && sort_direction != "desc" {
            return Err(AtlasError::ValidationFailed(
                "sort_direction must be 'asc' or 'desc'".to_string(),
            ));
        }
        if let Some(ct) = comparison_type {
            if !VALID_COMPARISON_TYPES.contains(&ct) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid comparison_type '{}'. Must be one of: {}",
                    ct,
                    VALID_COMPARISON_TYPES.join(", ")
                )));
            }
        }

        info!("Creating saved balance inquiry '{}' for org {}", name, org_id);

        self.repository.create_saved_inquiry(
            org_id, user_id, name, description, account_segments,
            period_from, period_to, currency_code, amount_type,
            include_zero_balances, comparison_enabled, comparison_type,
            sort_by, sort_direction, is_shared,
        ).await
    }

    /// Get a saved balance inquiry
    pub async fn get_saved_inquiry(&self, id: Uuid) -> AtlasResult<Option<SavedBalanceInquiry>> {
        self.repository.get_saved_inquiry(id).await
    }

    /// List saved balance inquiries for a user/org
    pub async fn list_saved_inquiries(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
    ) -> AtlasResult<Vec<SavedBalanceInquiry>> {
        self.repository.list_saved_inquiries(org_id, user_id).await
    }

    /// Delete a saved balance inquiry
    pub async fn delete_saved_inquiry(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_saved_inquiry(id).await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get the account monitor dashboard summary
    pub async fn get_monitor_summary(&self, org_id: Uuid) -> AtlasResult<AccountMonitorSummary> {
        self.repository.get_monitor_summary(org_id).await
    }
}

/// Compute mock GL balance based on account segment and period.
/// In production this would query the actual GL trial balance.
fn compute_mock_balance(account_segment: &str, fiscal_year: i32, period_number: i32) -> (f64, f64, f64, i32) {
    // Deterministic pseudo-balance based on account segment hash
    let hash: u64 = account_segment.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let base = ((hash % 1_000_000) as f64) / 100.0;
    let period_factor = 1.0 + (period_number as f64 / 12.0);
    let year_factor = 1.0 + ((fiscal_year - 2020).max(0) as f64 * 0.05);

    let beginning = base * period_factor * year_factor;
    let debits = beginning * 0.3 + (hash % 1000) as f64 / 10.0;
    let credits = beginning * 0.25 + ((hash + 500) % 1000) as f64 / 10.0;
    let je_count = ((hash % 50) + 1) as i32;

    (beginning, debits, credits, je_count)
}

/// Compute variance between current and comparison balance, returning
/// (variance_amount, variance_pct, alert_status)
fn compute_variance(
    current: f64,
    comparison: Option<f64>,
    warning_pct: Option<f64>,
    critical_pct: Option<f64>,
) -> (Option<f64>, Option<f64>, String) {
    let comp = match comparison {
        Some(c) if c.abs() > 0.001 => c,
        _ => return (None, None, "none".to_string()),
    };

    let variance = current - comp;
    let variance_pct = (variance / comp.abs()) * 100.0;

    let alert = if let Some(cp) = critical_pct {
        if variance_pct.abs() > cp {
            "critical"
        } else if let Some(wp) = warning_pct {
            if variance_pct.abs() > wp {
                "warning"
            } else {
                "none"
            }
        } else {
            "none"
        }
    } else if let Some(wp) = warning_pct {
        if variance_pct.abs() > wp {
            "warning"
        } else {
            "none"
        }
    } else {
        "none"
    };

    (Some(variance), Some(variance_pct), alert.to_string())
}
