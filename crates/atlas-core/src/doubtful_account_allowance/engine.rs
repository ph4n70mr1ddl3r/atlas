//! Doubtful Account Allowance Engine
//!
//! Business logic for managing the Allowance for Doubtful Accounts (Bad Debt Provision).
//!
//! Features:
//! - Provision policy CRUD (aging-based, percentage-based, specific identification)
//! - Aging bucket management with configurable provision percentages
//! - Provision run execution with automatic calculation
//! - Provision posting and reversal
//! - Full audit trail
//! - Dashboard summary
//!
//! Oracle Fusion Cloud ERP equivalent: Receivables > Collections > Allowance for Doubtful Accounts

use atlas_shared::{
    DoubtfulAccountPolicy, AgingBucketDefinition, ProvisionRun,
    ProvisionRunDetail, ProvisionRunActivity, DoubtfulAccountDashboard,
    AtlasError, AtlasResult,
};
use super::DoubtfulAccountAllowanceRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid Constants
// ============================================================================

/// Valid calculation methods
pub const VALID_CALCULATION_METHODS: &[&str] = &[
    "aging_based", "percentage_based", "specific_identification",
];

/// Valid policy statuses
pub const VALID_POLICY_STATUSES: &[&str] = &[
    "active", "inactive", "archived",
];

/// Valid run statuses
pub const VALID_RUN_STATUSES: &[&str] = &[
    "draft", "calculated", "posted", "reversed", "cancelled",
];

/// Valid activity actions
pub const VALID_ACTIONS: &[&str] = &[
    "created", "calculated", "posted", "reversed", "cancelled",
    "bucket_added", "bucket_updated", "status_changed",
];

// ============================================================================
// Engine
// ============================================================================

/// Doubtful Account Allowance Engine
///
/// Manages the full lifecycle of doubtful account provision policies,
/// aging buckets, provision runs, and posting.
pub struct DoubtfulAccountAllowanceEngine {
    repository: Arc<dyn DoubtfulAccountAllowanceRepository>,
}

impl DoubtfulAccountAllowanceEngine {
    pub fn new(repository: Arc<dyn DoubtfulAccountAllowanceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Policy Operations
    // ========================================================================

    /// Create a new provision policy
    /// Oracle Fusion: Receivables > Collections > Provision Policies > Create
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        policy_name: &str,
        description: Option<&str>,
        calculation_method: &str,
        flat_percentage: &str,
        default_provision_account: Option<&str>,
        default_expense_account: Option<&str>,
        currency_code: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DoubtfulAccountPolicy> {
        // Validate required fields
        if policy_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Policy code is required".to_string(),
            ));
        }
        if policy_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Policy name is required".to_string(),
            ));
        }
        if !VALID_CALCULATION_METHODS.contains(&calculation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation_method '{}'. Must be one of: {}",
                calculation_method, VALID_CALCULATION_METHODS.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be 3 characters".to_string(),
            ));
        }

        // Validate date range
        if let Some(to) = effective_to {
            if to < effective_from {
                return Err(AtlasError::ValidationFailed(
                    "effective_to must be after effective_from".to_string(),
                ));
            }
        }

        // Validate flat_percentage for percentage_based method
        if calculation_method == "percentage_based" {
            let pct: f64 = flat_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
                "flat_percentage must be a valid number".to_string(),
            ))?;
            if pct < 0.0 || pct > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "flat_percentage must be between 0 and 100".to_string(),
                ));
            }
        }

        // Check for duplicate code
        if let Some(_existing) = self.repository.get_policy_by_code(org_id, policy_code).await? {
            return Err(AtlasError::ValidationFailed(format!(
                "Policy code '{}' already exists for this organization", policy_code
            )));
        }

        info!(
            "Doubtful Account: Creating policy '{}' ({}) with {} method",
            policy_code, policy_name, calculation_method
        );

        let policy = self.repository.create_policy(
            org_id, policy_code, policy_name, description,
            calculation_method, flat_percentage,
            default_provision_account, default_expense_account,
            currency_code, effective_from, effective_to,
            created_by,
        ).await?;

        // Log activity
        self.repository.create_activity(
            org_id, None, Some(policy.id),
            "created",
            Some(&format!("Policy '{}' created with {} method", policy_code, calculation_method)),
            created_by, None, Some("active"), None,
        ).await.ok();

        Ok(policy)
    }

    /// Get a policy by ID
    pub async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<DoubtfulAccountPolicy>> {
        self.repository.get_policy(id).await
    }

    /// Get a policy by code
    pub async fn get_policy_by_code(
        &self,
        org_id: Uuid,
        policy_code: &str,
    ) -> AtlasResult<Option<DoubtfulAccountPolicy>> {
        self.repository.get_policy_by_code(org_id, policy_code).await
    }

    /// List policies
    pub async fn list_policies(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DoubtfulAccountPolicy>> {
        self.repository.list_policies(org_id, status).await
    }

    /// Deactivate a policy
    pub async fn deactivate_policy(&self, policy_id: Uuid, deactivated_by: Option<Uuid>) -> AtlasResult<DoubtfulAccountPolicy> {
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))?;

        if policy.status == "inactive" {
            return Err(AtlasError::ValidationFailed(
                "Policy is already inactive".to_string(),
            ));
        }

        let updated = self.repository.update_policy_status(policy_id, "inactive").await?;

        self.repository.create_activity(
            policy.organization_id, None, Some(policy_id),
            "status_changed",
            Some(&format!("Policy '{}' deactivated", policy.policy_code)),
            deactivated_by, Some(&policy.status), Some("inactive"), None,
        ).await.ok();

        Ok(updated)
    }

    /// Reactivate a policy
    pub async fn reactivate_policy(&self, policy_id: Uuid, reactivated_by: Option<Uuid>) -> AtlasResult<DoubtfulAccountPolicy> {
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))?;

        if policy.status == "active" {
            return Err(AtlasError::ValidationFailed(
                "Policy is already active".to_string(),
            ));
        }

        let updated = self.repository.update_policy_status(policy_id, "active").await?;

        self.repository.create_activity(
            policy.organization_id, None, Some(policy_id),
            "status_changed",
            Some(&format!("Policy '{}' reactivated", policy.policy_code)),
            reactivated_by, Some(&policy.status), Some("active"), None,
        ).await.ok();

        Ok(updated)
    }

    // ========================================================================
    // Aging Bucket Operations
    // ========================================================================

    /// Add an aging bucket to a policy
    /// Oracle Fusion: Receivables > Collections > Aging Buckets
    pub async fn create_aging_bucket(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        bucket_name: &str,
        from_days: i32,
        to_days: Option<i32>,
        provision_percentage: &str,
        display_order: i32,
    ) -> AtlasResult<AgingBucketDefinition> {
        // Validate policy exists
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))?;

        if policy.calculation_method != "aging_based" {
            return Err(AtlasError::ValidationFailed(
                "Aging buckets can only be added to aging_based policies".to_string(),
            ));
        }

        if bucket_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Bucket name is required".to_string(),
            ));
        }

        if from_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "from_days must be non-negative".to_string(),
            ));
        }

        if let Some(to) = to_days {
            if to < from_days {
                return Err(AtlasError::ValidationFailed(
                    "to_days must be >= from_days".to_string(),
                ));
            }
        }

        let pct: f64 = provision_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "provision_percentage must be a valid number".to_string(),
        ))?;

        if pct < 0.0 || pct > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "provision_percentage must be between 0 and 100".to_string(),
            ));
        }

        info!(
            "Doubtful Account: Creating aging bucket '{}' ({}-{} days, {}%) for policy {}",
            bucket_name, from_days,
            to_days.map(|d| d.to_string()).unwrap_or_else(|| "∞".to_string()),
            pct, policy.policy_code
        );

        let bucket = self.repository.create_aging_bucket(
            org_id, policy_id, bucket_name,
            from_days, to_days, provision_percentage, display_order,
        ).await?;

        self.repository.create_activity(
            org_id, None, Some(policy_id),
            "bucket_added",
            Some(&format!("Bucket '{}' added ({:.1}%)", bucket_name, pct)),
            None, None, None, None,
        ).await.ok();

        Ok(bucket)
    }

    // ========================================================================
    // Provision Run Operations
    // ========================================================================

    /// Create a new provision run (draft)
    /// Oracle Fusion: Receivables > Collections > Run Provision
    pub async fn create_provision_run(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        as_of_date: chrono::NaiveDate,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun> {
        // Validate policy exists and is active
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))?;

        if policy.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot create provision run for inactive policy".to_string(),
            ));
        }

        // Generate run number
        let next_num = self.repository.get_next_run_number(org_id).await.unwrap_or(1);
        let run_number = format!("PROV-{:04}", next_num);

        let today = chrono::Utc::now().date_naive();

        info!(
            "Doubtful Account: Creating provision run '{}' for policy '{}' as of {}",
            run_number, policy.policy_code, as_of_date
        );

        let run = self.repository.create_provision_run(
            org_id, &run_number, policy_id, &policy.policy_code,
            today, as_of_date, &policy.calculation_method,
            &policy.currency_code, description, created_by,
        ).await?;

        // Log activity
        self.repository.create_activity(
            org_id, Some(run.id), Some(policy_id),
            "created",
            Some(&format!("Provision run '{}' created", run_number)),
            created_by, None, Some("draft"), None,
        ).await.ok();

        Ok(run)
    }

    /// Calculate provision amounts for a run
    /// Simulates aging analysis and applies provision percentages
    pub async fn calculate_provision(
        &self,
        run_id: Uuid,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun> {
        let run = self.repository.get_provision_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Provision run not found".to_string()))?;

        if run.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot calculate provision for run in '{}' status. Must be 'draft'.",
                run.status
            )));
        }

        // Get the policy and its aging buckets
        let policy = self.repository.get_policy(run.policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))?;

        let mut total_outstanding: f64 = 0.0;
        let mut total_provision: f64 = 0.0;
        let total_customer_count = 0;
        let total_transaction_count = 0;

        match policy.calculation_method.as_str() {
            "aging_based" => {
                // Create detail lines for each aging bucket
                // In production, this would query actual AR open balances grouped by aging
                // For now, we create the structure with the configured percentages
                for bucket in &policy.aging_buckets {
                    // Simulated: in production, query actual AR aging data
                    let simulated_outstanding = 0.0;
                    let pct: f64 = bucket.provision_percentage.parse().unwrap_or(0.0);
                    let bucket_provision = simulated_outstanding * pct / 100.0;

                    self.repository.create_provision_detail(
                        run.organization_id, run_id,
                        Some(bucket.id), &bucket.bucket_name,
                        bucket.from_days, bucket.to_days,
                        &bucket.provision_percentage,
                        &format!("{:.2}", simulated_outstanding),
                        0, 0,
                        &format!("{:.2}", bucket_provision),
                    ).await.ok();

                    total_outstanding += simulated_outstanding;
                    total_provision += bucket_provision;
                }
            }
            "percentage_based" => {
                let flat_pct: f64 = policy.flat_percentage.parse().unwrap_or(0.0);
                // In production, query total AR outstanding
                let simulated_outstanding = 0.0;
                let provision = simulated_outstanding * flat_pct / 100.0;

                self.repository.create_provision_detail(
                    run.organization_id, run_id,
                    None, "All Outstanding",
                    0, None,
                    &format!("{:.4}", flat_pct),
                    &format!("{:.2}", simulated_outstanding),
                    0, 0,
                    &format!("{:.2}", provision),
                ).await.ok();

                total_outstanding = simulated_outstanding;
                total_provision = provision;
            }
            "specific_identification" => {
                // In production, would load manually entered amounts per customer
                // Here we just create an empty detail
                self.repository.create_provision_detail(
                    run.organization_id, run_id,
                    None, "Specific Identification",
                    0, None,
                    "0.0000",
                    "0.00",
                    0, 0,
                    "0.00",
                ).await.ok();
            }
            _ => {}
        }

        // Get prior provision amount
        let prior_runs = self.repository.list_provision_runs(
            run.organization_id,
            Some(run.policy_id),
            Some("posted"),
        ).await.unwrap_or_default();

        let total_prior_provision: f64 = prior_runs
            .first()
            .map(|r| r.total_provision_amount.parse().unwrap_or(0.0))
            .unwrap_or(0.0);

        let incremental = total_provision - total_prior_provision;

        let updated_run = self.repository.update_provision_run_results(
            run_id,
            &format!("{:.2}", total_outstanding),
            &format!("{:.2}", total_provision),
            &format!("{:.2}", total_prior_provision),
            &format!("{:.2}", incremental),
            total_customer_count,
            total_transaction_count,
            "calculated",
        ).await?;

        self.repository.create_activity(
            run.organization_id, Some(run_id), Some(run.policy_id),
            "calculated",
            Some(&format!(
                "Provision calculated: outstanding={:.2}, provision={:.2}, incremental={:.2}",
                total_outstanding, total_provision, incremental
            )),
            calculated_by, Some("draft"), Some("calculated"), None,
        ).await.ok();

        info!(
            "Doubtful Account: Calculated provision run '{}': outstanding={:.2}, provision={:.2}, incremental={:.2}",
            run.run_number, total_outstanding, total_provision, incremental
        );

        Ok(updated_run)
    }

    /// Post a calculated provision run (creates journal entry)
    /// Oracle Fusion: Receivables > Collections > Post Provision
    pub async fn post_provision(
        &self,
        run_id: Uuid,
        posted_by: Option<Uuid>,
        journal_entry_number: Option<&str>,
    ) -> AtlasResult<ProvisionRun> {
        let run = self.repository.get_provision_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Provision run not found".to_string()))?;

        if run.status != "calculated" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot post provision in '{}' status. Must be 'calculated'.",
                run.status
            )));
        }

        let je_number = journal_entry_number.unwrap_or(&run.run_number);

        let updated = self.repository.update_provision_run_status(
            run_id,
            "posted",
            posted_by,
            Some(je_number),
        ).await?;

        self.repository.create_activity(
            run.organization_id, Some(run_id), Some(run.policy_id),
            "posted",
            Some(&format!(
                "Provision posted with journal entry '{}'. Amount: {}",
                je_number, run.total_provision_amount
            )),
            posted_by, Some("calculated"), Some("posted"), None,
        ).await.ok();

        info!(
            "Doubtful Account: Posted provision run '{}' with JE '{}'",
            run.run_number, je_number
        );

        Ok(updated)
    }

    /// Reverse a posted provision run
    /// Oracle Fusion: Receivables > Collections > Reverse Provision
    pub async fn reverse_provision(
        &self,
        run_id: Uuid,
        reversed_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun> {
        let run = self.repository.get_provision_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Provision run not found".to_string()))?;

        if run.status != "posted" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot reverse provision in '{}' status. Must be 'posted'.",
                run.status
            )));
        }

        let updated = self.repository.update_provision_run_status(
            run_id,
            "reversed",
            reversed_by,
            None,
        ).await?;

        self.repository.create_activity(
            run.organization_id, Some(run_id), Some(run.policy_id),
            "reversed",
            Some(&format!("Provision '{}' reversed", run.run_number)),
            reversed_by, Some("posted"), Some("reversed"), None,
        ).await.ok();

        info!("Doubtful Account: Reversed provision run '{}'", run.run_number);

        Ok(updated)
    }

    /// Cancel a draft provision run
    pub async fn cancel_provision(
        &self,
        run_id: Uuid,
        cancelled_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun> {
        let run = self.repository.get_provision_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Provision run not found".to_string()))?;

        if run.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot cancel provision in '{}' status. Must be 'draft'.",
                run.status
            )));
        }

        let updated = self.repository.update_provision_run_status(
            run_id,
            "cancelled",
            cancelled_by,
            None,
        ).await?;

        self.repository.create_activity(
            run.organization_id, Some(run_id), Some(run.policy_id),
            "cancelled",
            Some(&format!("Provision '{}' cancelled", run.run_number)),
            cancelled_by, Some("draft"), Some("cancelled"), None,
        ).await.ok();

        Ok(updated)
    }

    /// Get a provision run by ID
    pub async fn get_provision_run(&self, id: Uuid) -> AtlasResult<Option<ProvisionRun>> {
        self.repository.get_provision_run(id).await
    }

    /// Get a provision run by number
    pub async fn get_provision_run_by_number(
        &self,
        org_id: Uuid,
        run_number: &str,
    ) -> AtlasResult<Option<ProvisionRun>> {
        self.repository.get_provision_run_by_number(org_id, run_number).await
    }

    /// List provision runs
    pub async fn list_provision_runs(
        &self,
        org_id: Uuid,
        policy_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ProvisionRun>> {
        self.repository.list_provision_runs(org_id, policy_id, status).await
    }

    /// List provision run details
    pub async fn list_provision_details(
        &self,
        run_id: Uuid,
    ) -> AtlasResult<Vec<ProvisionRunDetail>> {
        self.repository.list_provision_details(run_id).await
    }

    /// List activities for a run
    pub async fn list_activities(
        &self,
        run_id: Uuid,
    ) -> AtlasResult<Vec<ProvisionRunActivity>> {
        self.repository.list_activities(run_id).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<DoubtfulAccountDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    /// List aging buckets for a policy (public accessor for handlers)
    pub async fn list_aging_buckets_for_policy(
        &self,
        policy_id: Uuid,
    ) -> AtlasResult<Vec<AgingBucketDefinition>> {
        self.repository.list_aging_buckets(policy_id).await
    }
}
