//! Transfer Pricing Engine
//!
//! Oracle Fusion Cloud Financials > Transfer Pricing.
//! Manages intercompany pricing policies, transfer price transactions,
//! benchmarking studies (arm's-length analyses), comparables, and
//! BEPS documentation packages.
//!
//! Policy lifecycle: draft → active → inactive/expired
//! Transaction lifecycle: draft → submitted → approved/rejected → completed
//! Benchmark lifecycle: draft → in_review → approved/rejected/superseded
//! Documentation lifecycle: draft → in_review → approved → filed/superseded

use atlas_shared::{
    TransferPricingPolicy, TransferPriceTransaction,
    BenchmarkStudy, BenchmarkComparable,
    TransferPricingDocumentation, TransferPricingDashboard,
    AtlasError, AtlasResult,
};
use super::TransferPricingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ========================================================================
// Valid constants
// ========================================================================

const VALID_PRICING_METHODS: &[&str] = &[
    "CUP", "resale_price", "cost_plus", "profit_split", "tnmm", "other",
];

const VALID_COST_BASES: &[&str] = &[
    "full_cost", "variable_cost", "total_cost", "custom",
];

const VALID_POLICY_STATUSES: &[&str] = &[
    "draft", "active", "inactive", "expired",
];

const VALID_TXN_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "completed",
];

const VALID_SOURCE_TYPES: &[&str] = &[
    "intercompany", "sales_order", "purchase_order", "manual",
];

const VALID_ANALYSIS_METHODS: &[&str] = &[
    "cup", "resale_price", "cost_plus", "profit_split", "tnmm", "berry_ratio",
];

const VALID_BENCHMARK_STATUSES: &[&str] = &[
    "draft", "in_review", "approved", "rejected", "superseded",
];

const VALID_DOC_TYPES: &[&str] = &[
    "master_file", "local_file", "cbcr", "country_by_country", "other",
];

const VALID_DOC_STATUSES: &[&str] = &[
    "draft", "in_review", "approved", "filed", "superseded",
];

/// Transfer Pricing Engine
pub struct TransferPricingEngine {
    repository: Arc<dyn TransferPricingRepository>,
}

impl TransferPricingEngine {
    pub fn new(repository: Arc<dyn TransferPricingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Policies
    // ========================================================================

    /// Create a new transfer pricing policy
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        name: &str,
        description: Option<&str>,
        pricing_method: &str,
        from_entity_id: Option<Uuid>,
        from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>,
        to_entity_name: Option<&str>,
        product_category: Option<&str>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        geography: Option<&str>,
        tax_jurisdiction: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        arm_length_range_low: Option<&str>,
        arm_length_range_mid: Option<&str>,
        arm_length_range_high: Option<&str>,
        margin_pct: Option<&str>,
        cost_base: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingPolicy> {
        if policy_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Policy code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Policy name is required".to_string()));
        }
        if !VALID_PRICING_METHODS.contains(&pricing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pricing method '{}'. Must be one of: {}",
                pricing_method, VALID_PRICING_METHODS.join(", ")
            )));
        }
        if let Some(cb) = cost_base {
            if !VALID_COST_BASES.contains(&cb) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid cost base '{}'. Must be one of: {}", cb, VALID_COST_BASES.join(", ")
                )));
            }
        }
        if let Some(pct_str) = margin_pct {
            let pct: f64 = pct_str.parse().map_err(|_| AtlasError::ValidationFailed(
                "Margin percent must be a valid number".to_string(),
            ))?;
            if pct < 0.0 || pct > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "Margin percent must be between 0 and 100".to_string(),
                ));
            }
        }
        // Validate arm's-length range consistency
        if let (Some(low), Some(high)) = (arm_length_range_low, arm_length_range_high) {
            let low_val: f64 = low.parse().unwrap_or(0.0);
            let high_val: f64 = high.parse().unwrap_or(0.0);
            if low_val > high_val {
                return Err(AtlasError::ValidationFailed(
                    "Arm's-length range low cannot exceed high".to_string(),
                ));
            }
        }
        // Validate date range
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from date cannot be after effective to date".to_string(),
                ));
            }
        }

        info!("Creating transfer pricing policy {} ({}) for org {}", policy_code, name, org_id);

        self.repository.create_policy(
            org_id, policy_code, name, description, pricing_method,
            from_entity_id, from_entity_name, to_entity_id, to_entity_name,
            product_category, item_id, item_code, geography, tax_jurisdiction,
            effective_from, effective_to,
            arm_length_range_low, arm_length_range_mid, arm_length_range_high,
            margin_pct, cost_base, created_by,
        ).await
    }

    /// Get a policy by code
    pub async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransferPricingPolicy>> {
        self.repository.get_policy(org_id, code).await
    }

    /// Get a policy by ID
    pub async fn get_policy_by_id(&self, id: Uuid) -> AtlasResult<Option<TransferPricingPolicy>> {
        self.repository.get_policy_by_id(id).await
    }

    /// List policies
    pub async fn list_policies(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TransferPricingPolicy>> {
        if let Some(s) = status {
            if !VALID_POLICY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_POLICY_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_policies(org_id, status).await
    }

    /// Activate a policy
    pub async fn activate_policy(&self, id: Uuid) -> AtlasResult<TransferPricingPolicy> {
        let policy = self.repository.get_policy_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy {} not found", id)))?;

        if policy.status != "draft" && policy.status != "inactive" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate policy in '{}' status. Must be 'draft' or 'inactive'.",
                policy.status
            )));
        }

        info!("Activating transfer pricing policy {}", policy.policy_code);
        self.repository.update_policy_status(id, "active", None).await
    }

    /// Deactivate a policy
    pub async fn deactivate_policy(&self, id: Uuid) -> AtlasResult<TransferPricingPolicy> {
        let policy = self.repository.get_policy_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy {} not found", id)))?;

        if policy.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deactivate policy in '{}' status. Must be 'active'.",
                policy.status
            )));
        }

        info!("Deactivating transfer pricing policy {}", policy.policy_code);
        self.repository.update_policy_status(id, "inactive", None).await
    }

    /// Delete a policy (only in draft status)
    pub async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let policy = self.repository.get_policy(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy '{}' not found", code)))?;

        if policy.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete policy that is not in 'draft' status".to_string(),
            ));
        }

        self.repository.delete_policy(org_id, code).await
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    /// Create a new transfer price transaction
    pub async fn create_transaction(
        &self,
        org_id: Uuid,
        policy_id: Option<Uuid>,
        from_entity_id: Option<Uuid>,
        from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>,
        to_entity_name: Option<&str>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity: &str,
        unit_cost: &str,
        transfer_price: &str,
        currency_code: &str,
        transaction_date: chrono::NaiveDate,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPriceTransaction> {
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        let cost: f64 = unit_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit cost must be a valid number".to_string(),
        ))?;
        let tp: f64 = transfer_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Transfer price must be a valid number".to_string(),
        ))?;

        if qty < 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity cannot be negative".to_string()));
        }
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed("Unit cost cannot be negative".to_string()));
        }
        if tp < 0.0 {
            return Err(AtlasError::ValidationFailed("Transfer price cannot be negative".to_string()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }
        if let Some(st) = source_type {
            if !VALID_SOURCE_TYPES.contains(&st) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid source type '{}'. Must be one of: {}", st, VALID_SOURCE_TYPES.join(", ")
                )));
            }
        }

        // Calculate margin
        let total_amount = qty * tp;
        let margin_applied = if cost > 0.0 { Some(((tp - cost) / cost) * 100.0) } else { None };
        let margin_amount = if margin_applied.is_some() { Some(tp - cost) } else { None };

        // Check arm's-length compliance if policy is set
        let (is_compliant, compliance_notes) = if let Some(pid) = policy_id {
            self.check_arm_length_compliance(pid, tp).await?
        } else {
            (None, None)
        };

        let txn_number = format!("TPT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating transfer price transaction {} for org {}", txn_number, org_id);

        self.repository.create_transaction(
            org_id, &txn_number, policy_id,
            from_entity_id, from_entity_name, to_entity_id, to_entity_name,
            item_id, item_code, item_description,
            quantity, unit_cost, transfer_price,
            &format!("{:.4}", total_amount),
            currency_code, transaction_date,
            source_type, source_id, source_number,
            margin_applied.as_ref().map(|m| format!("{:.4}", m)).as_deref(),
            margin_amount.as_ref().map(|m| format!("{:.4}", m)).as_deref(),
            is_compliant,
            compliance_notes.as_deref(),
            created_by,
        ).await
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<TransferPriceTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// List transactions
    pub async fn list_transactions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        policy_id: Option<Uuid>,
    ) -> AtlasResult<Vec<TransferPriceTransaction>> {
        if let Some(s) = status {
            if !VALID_TXN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TXN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_transactions(org_id, status, policy_id).await
    }

    /// Submit a transaction for approval
    pub async fn submit_transaction(&self, id: Uuid) -> AtlasResult<TransferPriceTransaction> {
        let txn = self.repository.get_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transaction {} not found", id)))?;

        if txn.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit transaction in '{}' status. Must be 'draft'.",
                txn.status
            )));
        }

        info!("Submitting transfer price transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(id, "submitted", Some(chrono::Utc::now()), None).await
    }

    /// Approve a transaction
    pub async fn approve_transaction(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<TransferPriceTransaction> {
        let txn = self.repository.get_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transaction {} not found", id)))?;

        if txn.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve transaction in '{}' status. Must be 'submitted'.",
                txn.status
            )));
        }

        info!("Approving transfer price transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(id, "approved", None, approved_by).await
    }

    /// Reject a transaction
    pub async fn reject_transaction(&self, id: Uuid) -> AtlasResult<TransferPriceTransaction> {
        let txn = self.repository.get_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transaction {} not found", id)))?;

        if txn.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject transaction in '{}' status. Must be 'submitted'.",
                txn.status
            )));
        }

        info!("Rejecting transfer price transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(id, "rejected", None, None).await
    }

    // ========================================================================
    // Benchmark Studies
    // ========================================================================

    /// Create a new benchmark study
    pub async fn create_benchmark(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        policy_id: Option<Uuid>,
        analysis_method: &str,
        fiscal_year: Option<i32>,
        from_entity_id: Option<Uuid>,
        from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>,
        to_entity_name: Option<&str>,
        product_category: Option<&str>,
        tested_party: Option<&str>,
        prepared_by: Option<Uuid>,
        prepared_by_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenchmarkStudy> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Benchmark title is required".to_string()));
        }
        if !VALID_ANALYSIS_METHODS.contains(&analysis_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid analysis method '{}'. Must be one of: {}",
                analysis_method, VALID_ANALYSIS_METHODS.join(", ")
            )));
        }
        if let Some(fy) = fiscal_year {
            if fy < 1900 || fy > 2100 {
                return Err(AtlasError::ValidationFailed("Invalid fiscal year".to_string()));
            }
        }

        let study_number = format!("BMS-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating benchmark study {} ({})", study_number, title);

        self.repository.create_benchmark(
            org_id, &study_number, title, description, policy_id,
            analysis_method, fiscal_year,
            from_entity_id, from_entity_name, to_entity_id, to_entity_name,
            product_category, tested_party,
            prepared_by, prepared_by_name, created_by,
        ).await
    }

    /// Get a benchmark by ID
    pub async fn get_benchmark(&self, id: Uuid) -> AtlasResult<Option<BenchmarkStudy>> {
        self.repository.get_benchmark(id).await
    }

    /// List benchmarks
    pub async fn list_benchmarks(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BenchmarkStudy>> {
        if let Some(s) = status {
            if !VALID_BENCHMARK_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BENCHMARK_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_benchmarks(org_id, status).await
    }

    /// Submit a benchmark for review
    pub async fn submit_benchmark_for_review(&self, id: Uuid) -> AtlasResult<BenchmarkStudy> {
        let bm = self.repository.get_benchmark(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Benchmark {} not found", id)))?;

        if bm.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit benchmark in '{}' status. Must be 'draft'.",
                bm.status
            )));
        }

        info!("Submitting benchmark {} for review", bm.study_number);
        self.repository.update_benchmark_status(id, "in_review", None, None).await
    }

    /// Approve a benchmark
    pub async fn approve_benchmark(
        &self,
        id: Uuid,
        reviewed_by: Option<Uuid>,
        reviewed_by_name: Option<&str>,
    ) -> AtlasResult<BenchmarkStudy> {
        let bm = self.repository.get_benchmark(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Benchmark {} not found", id)))?;

        if bm.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve benchmark in '{}' status. Must be 'in_review'.",
                bm.status
            )));
        }

        info!("Approving benchmark {}", bm.study_number);
        self.repository.update_benchmark_status(id, "approved", reviewed_by, reviewed_by_name).await
    }

    /// Reject a benchmark
    pub async fn reject_benchmark(&self, id: Uuid) -> AtlasResult<BenchmarkStudy> {
        let bm = self.repository.get_benchmark(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Benchmark {} not found", id)))?;

        if bm.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject benchmark in '{}' status. Must be 'in_review'.",
                bm.status
            )));
        }

        info!("Rejecting benchmark {}", bm.study_number);
        self.repository.update_benchmark_status(id, "rejected", None, None).await
    }

    /// Delete a benchmark (only in draft status)
    pub async fn delete_benchmark(&self, id: Uuid) -> AtlasResult<()> {
        let bm = self.repository.get_benchmark(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Benchmark {} not found", id)))?;

        if bm.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete benchmark that is not in 'draft' status".to_string(),
            ));
        }
        self.repository.delete_benchmark(id).await
    }

    // ========================================================================
    // Comparables
    // ========================================================================

    /// Add a comparable to a benchmark study
    pub async fn add_comparable(
        &self,
        org_id: Uuid,
        benchmark_id: Uuid,
        comparable_number: i32,
        company_name: &str,
        country: Option<&str>,
        industry_code: Option<&str>,
        industry_description: Option<&str>,
        fiscal_year: Option<i32>,
        revenue: Option<&str>,
        operating_income: Option<&str>,
        operating_margin_pct: Option<&str>,
        net_income: Option<&str>,
        total_assets: Option<&str>,
        employees: Option<i32>,
        data_source: Option<&str>,
    ) -> AtlasResult<BenchmarkComparable> {
        // Validate benchmark exists
        let _bm = self.repository.get_benchmark(benchmark_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Benchmark {} not found", benchmark_id)))?;

        if company_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Company name is required".to_string()));
        }
        if comparable_number < 1 {
            return Err(AtlasError::ValidationFailed("Comparable number must be positive".to_string()));
        }

        info!("Adding comparable {} ({}) to benchmark {}", comparable_number, company_name, benchmark_id);

        self.repository.add_comparable(
            org_id, benchmark_id, comparable_number, company_name,
            country, industry_code, industry_description, fiscal_year,
            revenue, operating_income, operating_margin_pct,
            net_income, total_assets, employees, data_source,
        ).await
    }

    /// List comparables for a benchmark
    pub async fn list_comparables(&self, benchmark_id: Uuid) -> AtlasResult<Vec<BenchmarkComparable>> {
        self.repository.list_comparables(benchmark_id).await
    }

    /// Exclude a comparable from analysis
    pub async fn exclude_comparable(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<BenchmarkComparable> {
        self.repository.update_comparable_inclusion(id, false, reason).await
    }

    /// Include a comparable in analysis
    pub async fn include_comparable(&self, id: Uuid) -> AtlasResult<BenchmarkComparable> {
        self.repository.update_comparable_inclusion(id, true, None).await
    }

    // ========================================================================
    // Documentation Packages
    // ========================================================================

    /// Create a documentation package
    pub async fn create_documentation(
        &self,
        org_id: Uuid,
        title: &str,
        doc_type: &str,
        fiscal_year: i32,
        country: Option<&str>,
        reporting_entity_id: Option<Uuid>,
        reporting_entity_name: Option<&str>,
        description: Option<&str>,
        content_summary: Option<&str>,
        filing_deadline: Option<chrono::NaiveDate>,
        responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingDocumentation> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Title is required".to_string()));
        }
        if !VALID_DOC_TYPES.contains(&doc_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid doc type '{}'. Must be one of: {}", doc_type, VALID_DOC_TYPES.join(", ")
            )));
        }
        if fiscal_year < 1900 || fiscal_year > 2100 {
            return Err(AtlasError::ValidationFailed("Invalid fiscal year".to_string()));
        }

        let doc_number = format!("TPD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating TP documentation {} ({})", doc_number, title);

        self.repository.create_documentation(
            org_id, &doc_number, title, doc_type, fiscal_year,
            country, reporting_entity_id, reporting_entity_name,
            description, content_summary, filing_deadline,
            responsible_party, created_by,
        ).await
    }

    /// Get documentation by ID
    pub async fn get_documentation(&self, id: Uuid) -> AtlasResult<Option<TransferPricingDocumentation>> {
        self.repository.get_documentation(id).await
    }

    /// List documentation packages
    pub async fn list_documentation(
        &self,
        org_id: Uuid,
        doc_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TransferPricingDocumentation>> {
        if let Some(dt) = doc_type {
            if !VALID_DOC_TYPES.contains(&dt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid doc type '{}'. Must be one of: {}", dt, VALID_DOC_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_DOC_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_DOC_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_documentation(org_id, doc_type, status).await
    }

    /// Submit documentation for review
    pub async fn submit_documentation_for_review(&self, id: Uuid) -> AtlasResult<TransferPricingDocumentation> {
        let doc = self.repository.get_documentation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Documentation {} not found", id)))?;

        if doc.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit documentation in '{}' status. Must be 'draft'.",
                doc.status
            )));
        }

        info!("Submitting documentation {} for review", doc.doc_number);
        self.repository.update_documentation_status(id, "in_review", None, None).await
    }

    /// Approve documentation
    pub async fn approve_documentation(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<TransferPricingDocumentation> {
        let doc = self.repository.get_documentation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Documentation {} not found", id)))?;

        if doc.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve documentation in '{}' status. Must be 'in_review'.",
                doc.status
            )));
        }

        info!("Approving documentation {}", doc.doc_number);
        self.repository.update_documentation_status(id, "approved", approved_by, None).await
    }

    /// File documentation (mark as filed)
    pub async fn file_documentation(&self, id: Uuid) -> AtlasResult<TransferPricingDocumentation> {
        let doc = self.repository.get_documentation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Documentation {} not found", id)))?;

        if doc.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot file documentation in '{}' status. Must be 'approved'.",
                doc.status
            )));
        }

        info!("Filing documentation {}", doc.doc_number);
        self.repository.update_documentation_status(id, "filed", None, Some(chrono::Utc::now())).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get transfer pricing dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransferPricingDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Check if a transfer price falls within the arm's-length range
    async fn check_arm_length_compliance(
        &self,
        policy_id: Uuid,
        transfer_price: f64,
    ) -> AtlasResult<(Option<bool>, Option<String>)> {
        let policy = self.repository.get_policy_by_id(policy_id).await?;

        if let Some(policy) = policy {
            let low: f64 = policy.arm_length_range_low.parse().unwrap_or(0.0);
            let high: f64 = policy.arm_length_range_high.parse().unwrap_or(0.0);

            if low > 0.0 || high > 0.0 {
                let compliant = transfer_price >= low && transfer_price <= high;
                let notes = if compliant {
                    Some(format!("Transfer price {:.4} within arm's-length range [{:.4}, {:.4}]",
                        transfer_price, low, high))
                } else {
                    Some(format!("Transfer price {:.4} OUTSIDE arm's-length range [{:.4}, {:.4}]",
                        transfer_price, low, high))
                };
                Ok((Some(compliant), notes))
            } else {
                Ok((None, Some("No arm's-length range defined on policy".to_string())))
            }
        } else {
            Ok((None, Some("Policy not found".to_string())))
        }
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    struct MockTPRepository;

    #[async_trait::async_trait]
    impl TransferPricingRepository for MockTPRepository {
        async fn create_policy(
            &self, _org_id: Uuid, policy_code: &str, name: &str, _description: Option<&str>,
            pricing_method: &str, _from_entity_id: Option<Uuid>, _from_entity_name: Option<&str>,
            _to_entity_id: Option<Uuid>, _to_entity_name: Option<&str>,
            _product_category: Option<&str>, _item_id: Option<Uuid>, _item_code: Option<&str>,
            _geography: Option<&str>, _tax_jurisdiction: Option<&str>,
            _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
            _arm_length_range_low: Option<&str>, _arm_length_range_mid: Option<&str>,
            _arm_length_range_high: Option<&str>, _margin_pct: Option<&str>,
            _cost_base: Option<&str>, _created_by: Option<Uuid>,
        ) -> AtlasResult<TransferPricingPolicy> {
            Ok(TransferPricingPolicy {
                id: Uuid::new_v4(), organization_id: _org_id,
                policy_code: policy_code.to_string(), name: name.to_string(),
                description: None, pricing_method: pricing_method.to_string(),
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                product_category: None, item_id: None, item_code: None,
                geography: None, tax_jurisdiction: None,
                effective_from: None, effective_to: None,
                arm_length_range_low: "0".to_string(), arm_length_range_mid: "0".to_string(),
                arm_length_range_high: "0".to_string(), margin_pct: "0".to_string(),
                cost_base: None, status: "draft".to_string(), version: 1,
                approved_by: None, approved_at: None, created_by: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_policy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<TransferPricingPolicy>> { Ok(None) }
        async fn get_policy_by_id(&self, _id: Uuid) -> AtlasResult<Option<TransferPricingPolicy>> {
            Ok(Some(TransferPricingPolicy {
                id: _id, organization_id: Uuid::new_v4(),
                policy_code: "MOCK".to_string(), name: "Mock".to_string(),
                description: None, pricing_method: "cost_plus".to_string(),
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                product_category: None, item_id: None, item_code: None,
                geography: None, tax_jurisdiction: None,
                effective_from: None, effective_to: None,
                arm_length_range_low: "10".to_string(), arm_length_range_mid: "15".to_string(),
                arm_length_range_high: "20".to_string(), margin_pct: "10".to_string(),
                cost_base: None, status: "active".to_string(), version: 1,
                approved_by: None, approved_at: None, created_by: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }))
        }
        async fn list_policies(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<TransferPricingPolicy>> { Ok(vec![]) }
        async fn update_policy_status(&self, _id: Uuid, status: &str, _approved_by: Option<Uuid>) -> AtlasResult<TransferPricingPolicy> {
            Ok(TransferPricingPolicy {
                id: _id, organization_id: Uuid::new_v4(),
                policy_code: "MOCK".to_string(), name: "Mock".to_string(),
                description: None, pricing_method: "cost_plus".to_string(),
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                product_category: None, item_id: None, item_code: None,
                geography: None, tax_jurisdiction: None,
                effective_from: None, effective_to: None,
                arm_length_range_low: "10".to_string(), arm_length_range_mid: "15".to_string(),
                arm_length_range_high: "20".to_string(), margin_pct: "10".to_string(),
                cost_base: None, status: status.to_string(), version: 1,
                approved_by: None, approved_at: None, created_by: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn delete_policy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

        async fn create_transaction(
            &self, _org_id: Uuid, transaction_number: &str, _policy_id: Option<Uuid>,
            _from_entity_id: Option<Uuid>, _from_entity_name: Option<&str>,
            _to_entity_id: Option<Uuid>, _to_entity_name: Option<&str>,
            _item_id: Option<Uuid>, _item_code: Option<&str>, _item_description: Option<&str>,
            quantity: &str, unit_cost: &str, transfer_price: &str,
            total_amount: &str, currency_code: &str, transaction_date: chrono::NaiveDate,
            _source_type: Option<&str>, _source_id: Option<Uuid>, _source_number: Option<&str>,
            margin_applied: Option<&str>, margin_amount: Option<&str>,
            is_arm_length_compliant: Option<bool>, compliance_notes: Option<&str>,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<TransferPriceTransaction> {
            Ok(TransferPriceTransaction {
                id: Uuid::new_v4(), organization_id: _org_id,
                transaction_number: transaction_number.to_string(),
                policy_id: None,
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                item_id: None, item_code: None, item_description: None,
                quantity: quantity.to_string(), unit_cost: unit_cost.to_string(),
                transfer_price: transfer_price.to_string(),
                total_amount: total_amount.to_string(),
                currency_code: currency_code.to_string(),
                transaction_date, gl_date: None,
                source_type: None, source_id: None, source_number: None,
                margin_applied: margin_applied.map(|s| s.to_string()),
                margin_amount: margin_amount.map(|s| s.to_string()),
                is_arm_length_compliant, compliance_notes: compliance_notes.map(|s| s.to_string()),
                status: "draft".to_string(),
                submitted_at: None, approved_by: None, approved_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_transaction(&self, _id: Uuid) -> AtlasResult<Option<TransferPriceTransaction>> { Ok(None) }
        async fn list_transactions(&self, _org_id: Uuid, _status: Option<&str>, _policy_id: Option<Uuid>) -> AtlasResult<Vec<TransferPriceTransaction>> { Ok(vec![]) }
        async fn update_transaction_status(&self, _id: Uuid, status: &str, _submitted_at: Option<DateTime<Utc>>, _approved_by: Option<Uuid>) -> AtlasResult<TransferPriceTransaction> {
            Ok(TransferPriceTransaction {
                id: _id, organization_id: Uuid::new_v4(),
                transaction_number: "MOCK".to_string(),
                policy_id: None,
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                item_id: None, item_code: None, item_description: None,
                quantity: "100".to_string(), unit_cost: "10".to_string(),
                transfer_price: "15".to_string(), total_amount: "1500".to_string(),
                currency_code: "USD".to_string(),
                transaction_date: chrono::Utc::now().date_naive(),
                gl_date: None, source_type: None, source_id: None, source_number: None,
                margin_applied: None, margin_amount: None,
                is_arm_length_compliant: None, compliance_notes: None,
                status: status.to_string(),
                submitted_at: None, approved_by: None, approved_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }

        async fn create_benchmark(
            &self, _org_id: Uuid, study_number: &str, title: &str, _description: Option<&str>,
            _policy_id: Option<Uuid>, analysis_method: &str, _fiscal_year: Option<i32>,
            _from_entity_id: Option<Uuid>, _from_entity_name: Option<&str>,
            _to_entity_id: Option<Uuid>, _to_entity_name: Option<&str>,
            _product_category: Option<&str>, _tested_party: Option<&str>,
            _prepared_by: Option<Uuid>, _prepared_by_name: Option<&str>,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<BenchmarkStudy> {
            Ok(BenchmarkStudy {
                id: Uuid::new_v4(), organization_id: _org_id,
                study_number: study_number.to_string(), title: title.to_string(),
                description: None, policy_id: None,
                analysis_method: analysis_method.to_string(), fiscal_year: None,
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                product_category: None, tested_party: None,
                interquartile_range_low: "0".to_string(), interquartile_range_mid: "0".to_string(),
                interquartile_range_high: "0".to_string(), tested_result: "0".to_string(),
                is_within_range: None, conclusion: None,
                prepared_by: None, prepared_by_name: None,
                reviewed_by: None, reviewed_by_name: None,
                status: "draft".to_string(), approved_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_benchmark(&self, _id: Uuid) -> AtlasResult<Option<BenchmarkStudy>> {
            Ok(Some(BenchmarkStudy {
                id: _id, organization_id: Uuid::new_v4(),
                study_number: "BMS-MOCK".to_string(), title: "Mock".to_string(),
                description: None, policy_id: None,
                analysis_method: "cost_plus".to_string(), fiscal_year: Some(2024),
                from_entity_id: None, from_entity_name: None,
                to_entity_id: None, to_entity_name: None,
                product_category: None, tested_party: None,
                interquartile_range_low: "5".to_string(), interquartile_range_mid: "10".to_string(),
                interquartile_range_high: "15".to_string(), tested_result: "8".to_string(),
                is_within_range: Some(true), conclusion: None,
                prepared_by: None, prepared_by_name: None,
                reviewed_by: None, reviewed_by_name: None,
                status: "draft".to_string(), approved_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }))
        }
        async fn list_benchmarks(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<BenchmarkStudy>> { Ok(vec![]) }
        async fn update_benchmark_status(&self, _id: Uuid, status: &str, _reviewed_by: Option<Uuid>, _reviewed_by_name: Option<&str>) -> AtlasResult<BenchmarkStudy> {
            let mut bm = self.get_benchmark(_id).await?.unwrap();
            bm.status = status.to_string();
            Ok(bm)
        }
        async fn delete_benchmark(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

        async fn add_comparable(
            &self, _org_id: Uuid, _benchmark_id: Uuid, comparable_number: i32,
            company_name: &str, _country: Option<&str>,
            _industry_code: Option<&str>, _industry_description: Option<&str>,
            _fiscal_year: Option<i32>, revenue: Option<&str>,
            operating_income: Option<&str>, operating_margin_pct: Option<&str>,
            net_income: Option<&str>, total_assets: Option<&str>,
            _employees: Option<i32>, _data_source: Option<&str>,
        ) -> AtlasResult<BenchmarkComparable> {
            Ok(BenchmarkComparable {
                id: Uuid::new_v4(), organization_id: _org_id, benchmark_id: _benchmark_id,
                comparable_number, company_name: company_name.to_string(),
                country: None, industry_code: None, industry_description: None,
                fiscal_year: None,
                revenue: revenue.unwrap_or("0").to_string(),
                operating_income: operating_income.unwrap_or("0").to_string(),
                operating_margin_pct: operating_margin_pct.unwrap_or("0").to_string(),
                net_income: net_income.unwrap_or("0").to_string(),
                total_assets: total_assets.unwrap_or("0").to_string(),
                employees: None, data_source: None,
                is_included: true, exclusion_reason: None,
                relevance_score: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn list_comparables(&self, _benchmark_id: Uuid) -> AtlasResult<Vec<BenchmarkComparable>> { Ok(vec![]) }
        async fn update_comparable_inclusion(&self, _id: Uuid, _included: bool, _reason: Option<&str>) -> AtlasResult<BenchmarkComparable> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }

        async fn create_documentation(
            &self, _org_id: Uuid, doc_number: &str, title: &str, doc_type: &str,
            fiscal_year: i32, _country: Option<&str>,
            _reporting_entity_id: Option<Uuid>, _reporting_entity_name: Option<&str>,
            _description: Option<&str>, _content_summary: Option<&str>,
            _filing_deadline: Option<chrono::NaiveDate>, _responsible_party: Option<&str>,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<TransferPricingDocumentation> {
            Ok(TransferPricingDocumentation {
                id: Uuid::new_v4(), organization_id: _org_id,
                doc_number: doc_number.to_string(), title: title.to_string(),
                doc_type: doc_type.to_string(), fiscal_year,
                country: None, reporting_entity_id: None, reporting_entity_name: None,
                description: None, content_summary: None,
                policy_ids: None, benchmark_ids: None,
                filing_date: None, filing_deadline: None,
                responsible_party: None,
                status: "draft".to_string(),
                reviewed_by: None, approved_by: None, approved_at: None, filed_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_documentation(&self, _id: Uuid) -> AtlasResult<Option<TransferPricingDocumentation>> { Ok(None) }
        async fn list_documentation(&self, _org_id: Uuid, _doc_type: Option<&str>, _status: Option<&str>) -> AtlasResult<Vec<TransferPricingDocumentation>> { Ok(vec![]) }
        async fn update_documentation_status(&self, _id: Uuid, status: &str, _approved_by: Option<Uuid>, _filed_at: Option<DateTime<Utc>>) -> AtlasResult<TransferPricingDocumentation> {
            Ok(TransferPricingDocumentation {
                id: _id, organization_id: Uuid::new_v4(),
                doc_number: "TPD-MOCK".to_string(), title: "Mock".to_string(),
                doc_type: "local_file".to_string(), fiscal_year: 2024,
                country: None, reporting_entity_id: None, reporting_entity_name: None,
                description: None, content_summary: None,
                policy_ids: None, benchmark_ids: None,
                filing_date: None, filing_deadline: None,
                responsible_party: None,
                status: status.to_string(),
                reviewed_by: None, approved_by: None, approved_at: None, filed_at: None,
                created_by: None, metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }

        async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<TransferPricingDashboard> {
            Ok(TransferPricingDashboard {
                total_policies: 0, active_policies: 0,
                total_transactions: 0, total_transaction_value: "0".to_string(),
                pending_transactions: 0, non_compliant_transactions: 0,
                compliance_rate_pct: "0".to_string(),
                total_benchmarks: 0, active_benchmarks: 0,
                benchmarks_within_range: 0,
                total_documentation: 0, pending_filings: 0, overdue_filings: 0,
                transactions_by_method: serde_json::json!({}),
                transactions_by_status: serde_json::json!({}),
            })
        }
    }

    #[tokio::test]
    async fn test_create_policy_validates_empty_code() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_policy(
            Uuid::new_v4(), "", "Name", None, "cost_plus",
            None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Policy code is required"));
    }

    #[tokio::test]
    async fn test_create_policy_validates_method() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_policy(
            Uuid::new_v4(), "POL-01", "Name", None, "invalid_method",
            None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid pricing method"));
    }

    #[tokio::test]
    async fn test_create_policy_validates_margin_range() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_policy(
            Uuid::new_v4(), "POL-01", "Name", None, "cost_plus",
            None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, Some("150"), None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Margin percent must be between 0 and 100"));
    }

    #[tokio::test]
    async fn test_create_policy_validates_arm_length_range() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_policy(
            Uuid::new_v4(), "POL-01", "Name", None, "cost_plus",
            None, None, None, None, None, None, None, None, None,
            None, None, Some("20"), None, Some("10"), None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Arm's-length range low cannot exceed high"));
    }

    #[tokio::test]
    async fn test_create_policy_success() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_policy(
            Uuid::new_v4(), "POL-001", "US-DE Transfer Policy", Some("Intercompany pricing"),
            "cost_plus", None, Some("US Corp"), None, Some("DE GmbH"),
            Some("Electronics"), None, None, Some("US-DE"), None,
            None, None, Some("10"), Some("15"), Some("20"), Some("12.5"), Some("full_cost"), None,
        ).await;
        assert!(result.is_ok());
        let policy = result.unwrap();
        assert_eq!(policy.policy_code, "POL-001");
        assert_eq!(policy.status, "draft");
    }

    #[tokio::test]
    async fn test_create_transaction_validates_negative_qty() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_transaction(
            Uuid::new_v4(), None, None, None, None, None, None, None, None,
            "-10", "10", "15", "USD",
            chrono::Utc::now().date_naive(), None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Quantity cannot be negative"));
    }

    #[tokio::test]
    async fn test_create_transaction_success() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_transaction(
            Uuid::new_v4(), None, None, Some("US Corp"), None, Some("DE GmbH"),
            None, Some("ITEM-01"), Some("Widget"), "100", "10", "15", "USD",
            chrono::Utc::now().date_naive(), None, None, None, None,
        ).await;
        assert!(result.is_ok());
        let txn = result.unwrap();
        assert!(txn.transaction_number.starts_with("TPT-"));
        assert_eq!(txn.status, "draft");
        assert_eq!(txn.total_amount, "1500.0000");
    }

    #[tokio::test]
    async fn test_create_benchmark_validates_method() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_benchmark(
            Uuid::new_v4(), "Title", None, None, "bad_method",
            None, None, None, None, None, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid analysis method"));
    }

    #[tokio::test]
    async fn test_create_benchmark_success() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_benchmark(
            Uuid::new_v4(), "FY2024 Cost Plus Analysis", Some("Annual benchmarking"),
            None, "cost_plus", Some(2024), None, Some("US Corp"),
            None, Some("DE GmbH"), Some("Electronics"), Some("DE GmbH"),
            None, None, None,
        ).await;
        assert!(result.is_ok());
        let bm = result.unwrap();
        assert!(bm.study_number.starts_with("BMS-"));
        assert_eq!(bm.status, "draft");
    }

    #[tokio::test]
    async fn test_create_documentation_validates_type() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_documentation(
            Uuid::new_v4(), "Title", "bad_type", 2024, None, None, None,
            None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid doc type"));
    }

    #[tokio::test]
    async fn test_create_documentation_success() {
        let engine = TransferPricingEngine::new(Arc::new(MockTPRepository));
        let result = engine.create_documentation(
            Uuid::new_v4(), "BEPS Local File FY2024", "local_file", 2024,
            Some("DE"), None, Some("DE GmbH"), Some("Annual TP documentation"),
            Some("Full analysis attached"), None, Some("Tax Dept"), None,
        ).await;
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(doc.doc_number.starts_with("TPD-"));
        assert_eq!(doc.doc_type, "local_file");
        assert_eq!(doc.status, "draft");
    }
}
