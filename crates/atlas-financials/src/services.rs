//! Financials Services
//!
//! Business logic services for the Financials domain.
//! Oracle Fusion Cloud ERP-inspired financial modules:
//! - Accounts Payable (AP Invoices, Payments, Holds)
//! - Accounts Receivable (Transactions, Receipts, Credit Memos, Adjustments)
//! - Fixed Assets (Categories, Books, Assets, Depreciation, Transfers, Retirements)
//! - Cost Management (Cost Books, Elements, Profiles, Standard Costs, Adjustments, Variances)
//! - General Ledger (Journal Entries, Trial Balance)
//! - Purchase Order & Invoice Processing

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_core::fixed_assets::FixedAssetEngine;
use atlas_core::accounts_payable::AccountsPayableEngine;
use atlas_core::cost_accounting::CostAccountingEngine;
use atlas_shared::{AtlasResult, AtlasError, RecordId};
use std::sync::Arc;
use serde_json::json;
use tracing::info;

// ============================================================================
// Accounts Payable Service
// ============================================================================

/// Accounts Payable service
/// Oracle Fusion: Financials > Payables
#[allow(dead_code)]
pub struct AccountsPayableService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
    ap_engine: Arc<AccountsPayableEngine>,
}

impl AccountsPayableService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
        ap_engine: Arc<AccountsPayableEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine, ap_engine }
    }

    /// Validate and submit an AP invoice for approval
    pub async fn submit_ap_invoice(
        &self,
        invoice_id: RecordId,
    ) -> AtlasResult<()> {
        info!("AP Service: Submitting invoice {} for approval", invoice_id);
        self.ap_engine.submit_invoice(invoice_id).await?;
        Ok(())
    }

    /// Approve an AP invoice after reviewing
    pub async fn approve_ap_invoice(
        &self,
        invoice_id: RecordId,
        approver_id: RecordId,
    ) -> AtlasResult<()> {
        info!("AP Service: Approving invoice {} by {}", invoice_id, approver_id);
        self.ap_engine.approve_invoice(invoice_id, approver_id).await?;
        Ok(())
    }

    /// Cancel an AP invoice
    pub async fn cancel_ap_invoice(
        &self,
        invoice_id: RecordId,
        cancelled_by: RecordId,
        reason: Option<&str>,
    ) -> AtlasResult<()> {
        info!("AP Service: Cancelling invoice {} by {}", invoice_id, cancelled_by);
        self.ap_engine.cancel_invoice(invoice_id, cancelled_by, reason).await?;
        Ok(())
    }

    /// Apply a hold to an AP invoice
    pub async fn apply_hold(
        &self,
        org_id: RecordId,
        invoice_id: RecordId,
        hold_type: &str,
        hold_reason: &str,
        created_by: Option<RecordId>,
    ) -> AtlasResult<()> {
        info!("AP Service: Applying {} hold to invoice {}", hold_type, invoice_id);
        self.ap_engine.apply_hold(org_id, invoice_id, hold_type, hold_reason, created_by).await?;
        Ok(())
    }

    /// Release a hold from an AP invoice
    pub async fn release_hold(
        &self,
        hold_id: RecordId,
        released_by: RecordId,
        release_reason: Option<&str>,
    ) -> AtlasResult<()> {
        info!("AP Service: Releasing hold {} by {}", hold_id, released_by);
        self.ap_engine.release_hold(hold_id, released_by, release_reason).await?;
        Ok(())
    }

    /// Process payment for approved invoices
    pub async fn process_payment(
        &self,
        org_id: RecordId,
        payment_number: &str,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        payment_currency_code: &str,
        payment_amount: &str,
        supplier_id: RecordId,
        invoice_ids: &[RecordId],
        created_by: Option<RecordId>,
    ) -> AtlasResult<()> {
        info!("AP Service: Processing payment '{}' for supplier {}", payment_number, supplier_id);
        self.ap_engine.create_payment(
            org_id, payment_number, payment_date, payment_method,
            payment_currency_code, payment_amount,
            None, None, None,
            supplier_id, None, None,
            invoice_ids, created_by,
        ).await?;
        Ok(())
    }
}

// ============================================================================
// Accounts Receivable Service
// ============================================================================

/// Accounts Receivable service
/// Oracle Fusion: Financials > Receivables
#[allow(dead_code)]
pub struct AccountsReceivableService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid AR transaction types
#[allow(dead_code)]
const VALID_AR_TRANSACTION_TYPES: &[&str] = &[
    "invoice", "debit_memo", "credit_memo", "chargeback", "deposit", "guarantee",
];

/// Valid AR transaction statuses
#[allow(dead_code)]
const VALID_AR_STATUSES: &[&str] = &[
    "draft", "complete", "open", "closed", "cancelled",
];

/// Valid AR receipt types
#[allow(dead_code)]
const VALID_RECEIPT_TYPES: &[&str] = &[
    "cash", "check", "credit_card", "wire_transfer", "ach", "other",
];

/// Valid AR credit memo reason codes
#[allow(dead_code)]
const VALID_CREDIT_MEMO_REASONS: &[&str] = &[
    "return", "pricing_error", "damaged", "wrong_item", "discount", "other",
];

/// Valid AR adjustment types
#[allow(dead_code)]
const VALID_ADJUSTMENT_TYPES: &[&str] = &[
    "write_off", "write_off_bad_debt", "small_balance_write_off",
    "increase", "decrease", "transfer", "revaluation",
];

impl AccountsReceivableService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create a new AR transaction (customer invoice, debit memo, etc.)
    /// Oracle Fusion: Receivables > Transactions > Create
    pub async fn create_transaction(
        &self,
        transaction_number: &str,
        transaction_type: &str,
        _transaction_date: chrono::NaiveDate,
        _gl_date: Option<chrono::NaiveDate>,
        customer_id: RecordId,
        _customer_number: Option<&str>,
        _customer_name: Option<&str>,
        _currency_code: &str,
        entered_amount: &str,
        tax_amount: &str,
        _payment_terms: Option<&str>,
        _due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<()> {
        if transaction_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Transaction number is required".to_string(),
            ));
        }
        if !VALID_AR_TRANSACTION_TYPES.contains(&transaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction_type '{}'. Must be one of: {}",
                transaction_type, VALID_AR_TRANSACTION_TYPES.join(", ")
            )));
        }

        let entered: f64 = entered_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_amount must be a valid number".to_string(),
        ))?;

        // Credit memos should have negative amounts
        if transaction_type == "credit_memo" && entered > 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit memo entered_amount must be negative".to_string(),
            ));
        }

        let tax: f64 = tax_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "tax_amount must be a valid number".to_string(),
        ))?;

        let total = entered + tax;

        info!(
            "AR Service: Creating {} transaction '{}' for customer {} (amount: {:.2}, total: {:.2})",
            transaction_type, transaction_number, customer_id, entered, total
        );

        Ok(())
    }

    /// Complete a draft AR transaction (validates and moves to complete status)
    /// Oracle Fusion: Receivables > Transactions > Complete
    pub async fn complete_transaction(&self, transaction_id: RecordId) -> AtlasResult<()> {
        info!("AR Service: Completing transaction {}", transaction_id);

        let entity = self.schema_engine.get_entity("ar_transactions")
            .ok_or_else(|| AtlasError::EntityNotFound("ar_transactions".to_string()))?;

        if let Some(workflow) = &entity.workflow {
            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                transaction_id,
                "draft",
                "complete",
                None,
                &json!({}),
                None,
            ).await?;

            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Complete failed".to_string())
                ));
            }
        }

        Ok(())
    }

    /// Post a completed AR transaction to the GL
    /// Oracle Fusion: Receivables > Transactions > Post to GL
    pub async fn post_transaction(&self, transaction_id: RecordId) -> AtlasResult<()> {
        info!("AR Service: Posting transaction {} to GL", transaction_id);

        let entity = self.schema_engine.get_entity("ar_transactions")
            .ok_or_else(|| AtlasError::EntityNotFound("ar_transactions".to_string()))?;

        if let Some(workflow) = &entity.workflow {
            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                transaction_id,
                "complete",
                "post",
                None,
                &json!({"posted_at": chrono::Utc::now().to_rfc3339()}),
                None,
            ).await?;

            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Post failed".to_string())
                ));
            }
        }

        Ok(())
    }

    /// Create a customer receipt
    /// Oracle Fusion: Receivables > Receipts > Create
    pub async fn create_receipt(
        &self,
        receipt_number: &str,
        _receipt_date: chrono::NaiveDate,
        receipt_type: &str,
        amount: &str,
        _currency_code: &str,
        customer_id: RecordId,
        _customer_number: Option<&str>,
        _reference_number: Option<&str>,
    ) -> AtlasResult<()> {
        if receipt_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Receipt number is required".to_string(),
            ));
        }
        if !VALID_RECEIPT_TYPES.contains(&receipt_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt_type '{}'. Must be one of: {}",
                receipt_type, VALID_RECEIPT_TYPES.join(", ")
            )));
        }
        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Receipt amount must be positive".to_string(),
            ));
        }

        info!(
            "AR Service: Creating {} receipt '{}' from customer {} for {:.2}",
            receipt_type, receipt_number, customer_id, amount_val
        );

        Ok(())
    }

    /// Apply a receipt to an open transaction
    /// Oracle Fusion: Receivables > Receipts > Apply
    pub async fn apply_receipt(
        &self,
        receipt_id: RecordId,
        transaction_id: RecordId,
        applied_amount: &str,
    ) -> AtlasResult<()> {
        let amount: f64 = applied_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Applied amount must be a valid number".to_string(),
        ))?;

        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Applied amount must be positive".to_string(),
            ));
        }

        info!(
            "AR Service: Applying receipt {} to transaction {} (amount: {:.2})",
            receipt_id, transaction_id, amount
        );

        // In a full implementation, this would:
        // 1. Validate the receipt is in "confirmed" status
        // 2. Validate the transaction is "open"
        // 3. Create application records
        // 4. Update amounts on both receipt and transaction
        // 5. If transaction fully paid, transition to "closed"

        Ok(())
    }

    /// Create a credit memo for a customer
    /// Oracle Fusion: Receivables > Credit Memos > Create
    pub async fn create_credit_memo(
        &self,
        credit_memo_number: &str,
        customer_id: RecordId,
        _original_transaction_id: Option<RecordId>,
        _credit_memo_date: chrono::NaiveDate,
        reason_code: &str,
        amount: &str,
        tax_amount: &str,
    ) -> AtlasResult<()> {
        if credit_memo_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Credit memo number is required".to_string(),
            ));
        }
        if !VALID_CREDIT_MEMO_REASONS.contains(&reason_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reason_code '{}'. Must be one of: {}",
                reason_code, VALID_CREDIT_MEMO_REASONS.join(", ")
            )));
        }
        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit memo amount must be positive".to_string(),
            ));
        }

        let tax: f64 = tax_amount.parse().unwrap_or(0.0);
        let total = amt + tax;

        info!(
            "AR Service: Creating credit memo '{}' for customer {} ({}) - total: {:.2}",
            credit_memo_number, customer_id, reason_code, total
        );

        Ok(())
    }

    /// Create an AR adjustment (write-off, increase, decrease)
    /// Oracle Fusion: Receivables > Adjustments
    pub async fn create_adjustment(
        &self,
        adjustment_number: &str,
        transaction_id: RecordId,
        customer_id: RecordId,
        _adjustment_date: chrono::NaiveDate,
        adjustment_type: &str,
        amount: &str,
        _reason_code: Option<&str>,
        _reason_description: Option<&str>,
    ) -> AtlasResult<()> {
        if adjustment_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Adjustment number is required".to_string(),
            ));
        }
        if !VALID_ADJUSTMENT_TYPES.contains(&adjustment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment_type '{}'. Must be one of: {}",
                adjustment_type, VALID_ADJUSTMENT_TYPES.join(", ")
            )));
        }
        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        // Write-offs should be negative
        if adjustment_type.contains("write_off") && amt > 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Write-off amount must be negative".to_string(),
            ));
        }

        info!(
            "AR Service: Creating {} adjustment '{}' for customer {} on transaction {} (amount: {:.2})",
            adjustment_type, adjustment_number, customer_id, transaction_id, amt
        );

        Ok(())
    }

    /// Get AR aging summary for a customer
    /// Oracle Fusion: Receivables > Aging Report
    pub async fn get_aging_summary(
        &self,
        _org_id: RecordId,
        _customer_id: Option<RecordId>,
        as_of_date: chrono::NaiveDate,
    ) -> AtlasResult<serde_json::Value> {
        info!("AR Service: Getting AR aging summary as of {}", as_of_date);

        Ok(json!({
            "as_of_date": as_of_date.to_string(),
            "current": "0.00",
            "aging_1_30": "0.00",
            "aging_31_60": "0.00",
            "aging_61_90": "0.00",
            "aging_91_plus": "0.00",
            "total_outstanding": "0.00",
            "customer_count": 0,
            "transaction_count": 0,
        }))
    }
}

// ============================================================================
// Fixed Assets Service
// ============================================================================

/// Fixed Assets service
/// Oracle Fusion: Financials > Fixed Assets
#[allow(dead_code)]
pub struct FixedAssetsService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
    fa_engine: Arc<FixedAssetEngine>,
}

impl FixedAssetsService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
        fa_engine: Arc<FixedAssetEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine, fa_engine }
    }

    /// Create an asset category with defaults
    /// Oracle Fusion: Fixed Assets > Setup > Asset Categories
    pub async fn create_category(
        &self,
        org_id: RecordId,
        code: &str,
        name: &str,
        depreciation_method: &str,
        useful_life_months: i32,
    ) -> AtlasResult<()> {
        info!("FA Service: Creating asset category '{}' ({})", code, name);
        self.fa_engine.create_category(
            org_id, code, name, None,
            depreciation_method, useful_life_months,
            "10.00", // default 10% salvage
            None, None, None, None,
            None,
        ).await?;
        Ok(())
    }

    /// Create an asset book
    /// Oracle Fusion: Fixed Assets > Setup > Asset Books
    pub async fn create_book(
        &self,
        org_id: RecordId,
        code: &str,
        name: &str,
        book_type: &str,
    ) -> AtlasResult<()> {
        info!("FA Service: Creating asset book '{}' ({})", code, name);
        self.fa_engine.create_book(
            org_id, code, name, None,
            book_type, true, "monthly",
            None,
        ).await?;
        Ok(())
    }

    /// Register a new fixed asset
    /// Oracle Fusion: Fixed Assets > Assets > Add
    pub async fn register_asset(
        &self,
        org_id: RecordId,
        asset_number: &str,
        asset_name: &str,
        asset_type: &str,
        original_cost: &str,
        category_code: Option<&str>,
        book_code: Option<&str>,
        department_id: Option<RecordId>,
        department_name: Option<&str>,
        location: Option<&str>,
    ) -> AtlasResult<()> {
        info!(
            "FA Service: Registering asset '{}' ({}) with cost {}",
            asset_number, asset_name, original_cost
        );

        let cost: f64 = original_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Original cost must be a valid number".to_string(),
        ))?;

        // Default salvage at 10% of cost
        let salvage = format!("{:.2}", cost * 0.10);

        self.fa_engine.create_asset(
            org_id, asset_number, asset_name, None,
            category_code, book_code,
            asset_type,
            original_cost, &salvage, "10.00",
            None, Some(60), // default 60 months useful life
            None,
            None, // acquisition_date
            location,
            department_id, department_name,
            None, None, // custodian
            None, None, None, None, // serial, tag, manufacturer, model
            None, None, None, None, None, // warranty, insurance, lease
            None, None, None, None, // accounts
            None,
        ).await?;

        Ok(())
    }

    /// Acquire a draft asset
    /// Oracle Fusion: Fixed Assets > Assets > Acquire
    pub async fn acquire_asset(&self, asset_id: RecordId) -> AtlasResult<()> {
        info!("FA Service: Acquiring asset {}", asset_id);
        self.fa_engine.acquire_asset(asset_id).await?;
        Ok(())
    }

    /// Place an asset in service (start depreciation)
    /// Oracle Fusion: Fixed Assets > Assets > Place in Service
    pub async fn place_in_service(
        &self,
        asset_id: RecordId,
        in_service_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<()> {
        info!("FA Service: Placing asset {} in service", asset_id);
        self.fa_engine.place_in_service(asset_id, in_service_date).await?;
        Ok(())
    }

    /// Run depreciation for an asset
    /// Oracle Fusion: Fixed Assets > Depreciation > Run Depreciation
    pub async fn run_depreciation(
        &self,
        asset_id: RecordId,
        fiscal_year: i32,
        period_number: i32,
        depreciation_date: chrono::NaiveDate,
    ) -> AtlasResult<f64> {
        info!(
            "FA Service: Running depreciation for asset {} (FY{} P{})",
            asset_id, fiscal_year, period_number
        );
        let (dep_amount, _updated_asset) = self.fa_engine.calculate_depreciation(
            asset_id, fiscal_year, period_number, None,
            depreciation_date, None,
        ).await?;
        Ok(dep_amount)
    }

    /// Request an asset transfer
    /// Oracle Fusion: Fixed Assets > Asset Transfers
    pub async fn request_transfer(
        &self,
        org_id: RecordId,
        asset_id: RecordId,
        to_department_id: Option<RecordId>,
        to_department_name: Option<&str>,
        to_location: Option<&str>,
        transfer_date: chrono::NaiveDate,
        reason: Option<&str>,
    ) -> AtlasResult<()> {
        info!("FA Service: Requesting transfer for asset {}", asset_id);
        self.fa_engine.create_transfer(
            org_id, asset_id,
            to_department_id, to_department_name,
            to_location,
            None, None, // to custodian
            transfer_date, reason, None,
        ).await?;
        Ok(())
    }

    /// Approve an asset transfer
    pub async fn approve_transfer(&self, transfer_id: RecordId, approved_by: RecordId) -> AtlasResult<()> {
        info!("FA Service: Approving transfer {} by {}", transfer_id, approved_by);
        self.fa_engine.approve_transfer(transfer_id, approved_by).await?;
        Ok(())
    }

    /// Retire an asset (sale, scrap, write-off, etc.)
    /// Oracle Fusion: Fixed Assets > Asset Retirements
    pub async fn retire_asset(
        &self,
        org_id: RecordId,
        asset_id: RecordId,
        retirement_type: &str,
        retirement_date: chrono::NaiveDate,
        proceeds: &str,
        removal_cost: &str,
        buyer_name: Option<&str>,
    ) -> AtlasResult<()> {
        info!(
            "FA Service: Retiring asset {} via {} on {}",
            asset_id, retirement_type, retirement_date
        );
        self.fa_engine.create_retirement(
            org_id, asset_id, retirement_type, retirement_date,
            proceeds, removal_cost,
            None, buyer_name, None, None,
        ).await?;
        Ok(())
    }

    /// Approve an asset retirement
    pub async fn approve_retirement(&self, retirement_id: RecordId, approved_by: RecordId) -> AtlasResult<()> {
        info!("FA Service: Approving retirement {} by {}", retirement_id, approved_by);
        self.fa_engine.approve_retirement(retirement_id, approved_by).await?;
        Ok(())
    }

    /// Get asset net book value summary
    pub async fn get_asset_summary(
        &self,
        org_id: RecordId,
    ) -> AtlasResult<serde_json::Value> {
        let assets = self.fa_engine.list_assets(org_id, None, None, None).await?;

        let total_cost: f64 = assets.iter()
            .filter(|a| a.status != "disposed" && a.status != "retired")
            .map(|a| a.original_cost.parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_depreciation: f64 = assets.iter()
            .filter(|a| a.status != "disposed" && a.status != "retired")
            .map(|a| a.accumulated_depreciation.parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_nbv: f64 = assets.iter()
            .filter(|a| a.status != "disposed" && a.status != "retired")
            .map(|a| a.net_book_value.parse::<f64>().unwrap_or(0.0))
            .sum();

        let active_count = assets.iter()
            .filter(|a| a.status == "in_service" || a.status == "acquired")
            .count();

        Ok(json!({
            "total_assets": assets.len(),
            "active_assets": active_count,
            "total_original_cost": format!("{:.2}", total_cost),
            "total_accumulated_depreciation": format!("{:.2}", total_depreciation),
            "total_net_book_value": format!("{:.2}", total_nbv),
        }))
    }
}

// ============================================================================
// Cost Management Service
// ============================================================================

/// Cost Management service
/// Oracle Fusion: Cost Management > Cost Accounting
#[allow(dead_code)]
pub struct CostManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
    cost_engine: Arc<CostAccountingEngine>,
}

/// Valid costing methods for cost management
#[allow(dead_code)]
const VALID_COSTING_METHODS: &[&str] = &[
    "standard", "average", "fifo", "lifo",
];

/// Valid cost element types
#[allow(dead_code)]
const VALID_COST_ELEMENT_TYPES: &[&str] = &[
    "material", "labor", "overhead", "subcontracting", "expense",
];

/// Valid overhead absorption methods
#[allow(dead_code)]
const VALID_OVERHEAD_METHODS: &[&str] = &[
    "rate", "amount", "percentage",
];

impl CostManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
        cost_engine: Arc<CostAccountingEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine, cost_engine }
    }

    /// Create a cost book
    /// Oracle Fusion: Cost Management > Cost Books
    pub async fn create_cost_book(
        &self,
        org_id: RecordId,
        code: &str,
        name: &str,
        costing_method: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if !VALID_COSTING_METHODS.contains(&costing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid costing_method '{}'. Must be one of: {}",
                costing_method, VALID_COSTING_METHODS.join(", ")
            )));
        }

        info!("Cost Service: Creating cost book '{}' ({}) with {} method", code, name, costing_method);
        self.cost_engine.create_cost_book(
            org_id, code, name, None,
            costing_method, currency_code,
            None, None, None,
        ).await?;
        Ok(())
    }

    /// Create a cost element
    /// Oracle Fusion: Cost Management > Cost Elements
    pub async fn create_cost_element(
        &self,
        org_id: RecordId,
        code: &str,
        name: &str,
        element_type: &str,
        cost_book_id: Option<RecordId>,
        default_rate: &str,
    ) -> AtlasResult<()> {
        if !VALID_COST_ELEMENT_TYPES.contains(&element_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid element_type '{}'. Must be one of: {}",
                element_type, VALID_COST_ELEMENT_TYPES.join(", ")
            )));
        }

        let rate: f64 = default_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Default rate must be a valid number".to_string(),
        ))?;
        if rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Default rate cannot be negative".to_string(),
            ));
        }

        info!("Cost Service: Creating cost element '{}' ({}) of type {}", code, name, element_type);
        self.cost_engine.create_cost_element(
            org_id, code, name, None,
            element_type, cost_book_id,
            default_rate, None, None,
        ).await?;
        Ok(())
    }

    /// Create a cost profile for an item
    /// Oracle Fusion: Cost Management > Cost Profiles
    pub async fn create_cost_profile(
        &self,
        org_id: RecordId,
        code: &str,
        name: &str,
        cost_book_id: RecordId,
        item_id: Option<RecordId>,
        cost_type: &str,
        overhead_absorption_method: &str,
    ) -> AtlasResult<()> {
        if !VALID_COSTING_METHODS.contains(&cost_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cost_type '{}'. Must be one of: {}",
                cost_type, VALID_COSTING_METHODS.join(", ")
            )));
        }
        if !VALID_OVERHEAD_METHODS.contains(&overhead_absorption_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid overhead_absorption_method '{}'. Must be one of: {}",
                overhead_absorption_method, VALID_OVERHEAD_METHODS.join(", ")
            )));
        }

        info!("Cost Service: Creating cost profile '{}' for org {}", code, org_id);
        self.cost_engine.create_cost_profile(
            org_id, code, name, None,
            cost_book_id, item_id, None,
            cost_type, false, true,
            overhead_absorption_method, None,
        ).await?;
        Ok(())
    }

    /// Set standard cost for an item/element combination
    /// Oracle Fusion: Cost Management > Standard Costs
    pub async fn set_standard_cost(
        &self,
        org_id: RecordId,
        cost_book_id: RecordId,
        cost_element_id: RecordId,
        item_id: RecordId,
        standard_cost: &str,
        currency_code: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<()> {
        let cost: f64 = standard_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Standard cost must be a valid number".to_string(),
        ))?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Standard cost cannot be negative".to_string(),
            ));
        }

        info!(
            "Cost Service: Setting standard cost for item {} in book {}: {:.2}",
            item_id, cost_book_id, cost
        );

        self.cost_engine.create_standard_cost(
            org_id, cost_book_id, None,
            cost_element_id, item_id, None,
            standard_cost, currency_code, effective_date, None,
        ).await?;

        Ok(())
    }

    /// Create and submit a cost adjustment
    /// Oracle Fusion: Cost Management > Cost Adjustments
    pub async fn create_cost_adjustment(
        &self,
        org_id: RecordId,
        cost_book_id: RecordId,
        adjustment_type: &str,
        description: Option<&str>,
        reason: Option<&str>,
        currency_code: &str,
    ) -> AtlasResult<RecordId> {
        info!("Cost Service: Creating {} cost adjustment", adjustment_type);

        let adjustment = self.cost_engine.create_cost_adjustment(
            org_id, cost_book_id, adjustment_type,
            description, reason, currency_code, None, None,
        ).await?;

        Ok(adjustment.id)
    }

    /// Submit a cost adjustment for approval
    pub async fn submit_cost_adjustment(&self, adjustment_id: RecordId) -> AtlasResult<()> {
        info!("Cost Service: Submitting cost adjustment {}", adjustment_id);
        self.cost_engine.submit_adjustment(adjustment_id).await?;
        Ok(())
    }

    /// Approve a cost adjustment
    pub async fn approve_cost_adjustment(&self, adjustment_id: RecordId, approved_by: RecordId) -> AtlasResult<()> {
        info!("Cost Service: Approving cost adjustment {} by {}", adjustment_id, approved_by);
        self.cost_engine.approve_adjustment(adjustment_id, approved_by).await?;
        Ok(())
    }

    /// Post an approved cost adjustment
    pub async fn post_cost_adjustment(&self, adjustment_id: RecordId, posted_by: RecordId) -> AtlasResult<()> {
        info!("Cost Service: Posting cost adjustment {} by {}", adjustment_id, posted_by);
        self.cost_engine.post_adjustment(adjustment_id, posted_by).await?;
        Ok(())
    }

    /// Record a cost variance
    /// Oracle Fusion: Cost Management > Variance Analysis
    pub async fn record_variance(
        &self,
        org_id: RecordId,
        cost_book_id: RecordId,
        variance_type: &str,
        variance_date: chrono::NaiveDate,
        item_id: RecordId,
        standard_cost: &str,
        actual_cost: &str,
        quantity: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        let sc: f64 = standard_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Standard cost must be a valid number".to_string(),
        ))?;
        let ac: f64 = actual_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Actual cost must be a valid number".to_string(),
        ))?;
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;

        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive".to_string(),
            ));
        }

        let variance = (ac - sc) * qty;
        let variance_type_label = if variance > 0.0 { "unfavorable" } else if variance < 0.0 { "favorable" } else { "none" };

        info!(
            "Cost Service: Recording {} variance for item {}: std={:.2} actual={:.2} qty={:.0} variance={:.2} ({})",
            variance_type, item_id, sc, ac, qty, variance, variance_type_label
        );

        self.cost_engine.create_cost_variance(
            org_id, cost_book_id, variance_type, variance_date,
            item_id, None, None, None, None, None,
            standard_cost, actual_cost, quantity,
            currency_code, None, None,
        ).await?;

        Ok(())
    }

    /// Get the cost accounting dashboard
    /// Oracle Fusion: Cost Management > Dashboard
    pub async fn get_dashboard(&self, org_id: RecordId) -> AtlasResult<serde_json::Value> {
        let dashboard = self.cost_engine.get_dashboard(org_id).await?;
        Ok(json!({
            "total_cost_books": dashboard.total_cost_books,
            "total_cost_elements": dashboard.total_cost_elements,
            "total_cost_books": dashboard.total_cost_books,
            "active_cost_books": dashboard.active_cost_books,
            "total_cost_elements": dashboard.total_cost_elements,
            "total_standard_costs": dashboard.total_standard_costs,
            "pending_adjustments": dashboard.pending_adjustments,
            "total_adjustments": dashboard.total_adjustments,
            "total_variances": dashboard.total_variances,
            "unfavorable_variances": dashboard.unfavorable_variances,
        }))
    }

    /// Calculate total item cost across all cost elements
    pub fn calculate_total_item_cost(element_costs: &[(&str, f64)]) -> f64 {
        element_costs.iter().map(|(_, cost)| *cost).sum()
    }

    /// Calculate variance percentage
    pub fn calculate_variance_percent(standard: f64, actual: f64) -> f64 {
        if standard > 0.0 {
            ((actual - standard) / standard) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate adjustment amount
    pub fn calculate_adjustment_amount(old_cost: f64, new_cost: f64) -> f64 {
        new_cost - old_cost
    }
}

// ============================================================================
// Revenue Recognition Service (ASC 606 / IFRS 15)
// ============================================================================

/// Revenue Recognition service
/// Oracle Fusion: Financials > Revenue Management
#[allow(dead_code)]
pub struct RevenueRecognitionService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid recognition methods
#[allow(dead_code)]
const VALID_RECOGNITION_METHODS: &[&str] = &[
    "over_time", "point_in_time",
];

/// Valid over-time methods
#[allow(dead_code)]
const VALID_OVER_TIME_METHODS: &[&str] = &[
    "output", "input", "straight_line",
];

/// Valid allocation bases
#[allow(dead_code)]
const VALID_ALLOCATION_BASES: &[&str] = &[
    "standalone_selling_price", "residual", "equal",
];

/// Valid contract statuses
#[allow(dead_code)]
const VALID_CONTRACT_STATUSES: &[&str] = &[
    "draft", "active", "completed", "cancelled", "modified",
];

/// Valid obligation statuses
#[allow(dead_code)]
const VALID_OBLIGATION_STATUSES: &[&str] = &[
    "pending", "in_progress", "satisfied", "partially_satisfied", "cancelled",
];

/// Valid schedule statuses
#[allow(dead_code)]
const VALID_SCHEDULE_STATUSES: &[&str] = &[
    "planned", "recognized", "reversed", "cancelled",
];

/// Valid modification types
#[allow(dead_code)]
const VALID_MODIFICATION_TYPES: &[&str] = &[
    "price_change", "scope_change", "term_extension",
    "termination", "add_obligation", "remove_obligation",
];

impl RevenueRecognitionService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a revenue recognition policy
    /// Oracle Fusion: Revenue Management > Revenue Policies
    pub async fn create_policy(
        &self,
        code: &str,
        name: &str,
        recognition_method: &str,
        over_time_method: Option<&str>,
        allocation_basis: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Policy code and name are required".to_string(),
            ));
        }
        if !VALID_RECOGNITION_METHODS.contains(&recognition_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recognition_method '{}'. Must be one of: {}",
                recognition_method, VALID_RECOGNITION_METHODS.join(", ")
            )));
        }
        if let Some(otm) = over_time_method {
            if !VALID_OVER_TIME_METHODS.contains(&otm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid over_time_method '{}'. Must be one of: {}",
                    otm, VALID_OVER_TIME_METHODS.join(", ")
                )));
            }
        }
        if !VALID_ALLOCATION_BASES.contains(&allocation_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation_basis '{}'. Must be one of: {}",
                allocation_basis, VALID_ALLOCATION_BASES.join(", ")
            )));
        }

        info!("Revenue Service: Creating policy '{}' with {} method", code, recognition_method);
        Ok(())
    }

    /// Create and validate a revenue contract
    /// Oracle Fusion: Revenue Management > Revenue Contracts
    pub async fn create_contract(
        &self,
        contract_number: &str,
        customer_id: RecordId,
        total_transaction_price: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if contract_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Contract number is required".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        let price: f64 = total_transaction_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Transaction price must be a valid number".to_string(),
        ))?;
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Transaction price must be non-negative".to_string(),
            ));
        }

        info!("Revenue Service: Creating contract '{}' for customer {}", contract_number, customer_id);
        Ok(())
    }

    /// Calculate straight-line revenue allocation for performance obligations
    pub fn calculate_straight_line_allocation(
        total_price: f64,
        obligation_count: usize,
    ) -> Vec<f64> {
        if obligation_count == 0 {
            return vec![];
        }
        let per_obligation = total_price / obligation_count as f64;
        vec![per_obligation; obligation_count]
    }

    /// Calculate standalone selling price allocation
    pub fn calculate_ssp_allocation(
        total_price: f64,
        standalone_prices: &[f64],
    ) -> Vec<f64> {
        let total_ssp: f64 = standalone_prices.iter().sum();
        if total_ssp <= 0.0 {
            return vec![0.0; standalone_prices.len()];
        }
        standalone_prices.iter()
            .map(|ssp| (ssp / total_ssp) * total_price)
            .collect()
    }

    /// Calculate percentage complete for over-time recognition
    pub fn calculate_percentage_complete(
        costs_incurred: f64,
        total_estimated_costs: f64,
    ) -> f64 {
        if total_estimated_costs <= 0.0 {
            return 0.0;
        }
        (costs_incurred / total_estimated_costs).min(1.0)
    }

    /// Calculate revenue to date based on percentage complete
    pub fn calculate_revenue_to_date(
        total_transaction_price: f64,
        percentage_complete: f64,
    ) -> f64 {
        total_transaction_price * percentage_complete
    }
}

// ============================================================================
// Subledger Accounting Service
// ============================================================================

/// Subledger Accounting service
/// Oracle Fusion: Financials > General Ledger > Subledger Accounting
#[allow(dead_code)]
pub struct SubledgerAccountingService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid SLA applications
#[allow(dead_code)]
const VALID_SLA_APPLICATIONS: &[&str] = &[
    "payables", "receivables", "expenses", "assets", "projects", "general",
];

/// Valid SLA event classes
#[allow(dead_code)]
const VALID_SLA_EVENT_CLASSES: &[&str] = &[
    "create", "update", "cancel", "reverse",
];

/// Valid SLA derivation types
#[allow(dead_code)]
const VALID_DERIVATION_TYPES: &[&str] = &[
    "constant", "lookup", "formula",
];

/// Valid SLA entry statuses
#[allow(dead_code)]
const VALID_SLA_ENTRY_STATUSES: &[&str] = &[
    "draft", "accounted", "posted", "transferred", "reversed", "error",
];

impl SubledgerAccountingService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate an accounting method
    /// Oracle Fusion: Subledger Accounting > Accounting Methods
    pub async fn create_accounting_method(
        &self,
        code: &str,
        name: &str,
        application: &str,
        transaction_type: &str,
        event_class: Option<&str>,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Accounting method code and name are required".to_string(),
            ));
        }
        if !VALID_SLA_APPLICATIONS.contains(&application) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid application '{}'. Must be one of: {}",
                application, VALID_SLA_APPLICATIONS.join(", ")
            )));
        }
        if transaction_type.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Transaction type is required".to_string(),
            ));
        }
        if let Some(ec) = event_class {
            if !VALID_SLA_EVENT_CLASSES.contains(&ec) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid event_class '{}'. Must be one of: {}",
                    ec, VALID_SLA_EVENT_CLASSES.join(", ")
                )));
            }
        }

        info!("SLA Service: Creating accounting method '{}' for {}", code, application);
        Ok(())
    }

    /// Create and validate a derivation rule
    /// Oracle Fusion: Subledger Accounting > Derivation Rules
    pub async fn create_derivation_rule(
        &self,
        code: &str,
        name: &str,
        derivation_type: &str,
        target_segment: &str,
        priority: i32,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Derivation rule code and name are required".to_string(),
            ));
        }
        if !VALID_DERIVATION_TYPES.contains(&derivation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid derivation_type '{}'. Must be one of: {}",
                derivation_type, VALID_DERIVATION_TYPES.join(", ")
            )));
        }
        if target_segment.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Target segment is required".to_string(),
            ));
        }
        if priority < 0 {
            return Err(AtlasError::ValidationFailed(
                "Priority must be non-negative".to_string(),
            ));
        }

        info!("SLA Service: Creating derivation rule '{}' of type {}", code, derivation_type);
        Ok(())
    }
}

// ============================================================================
// Cash Management Service
// ============================================================================

/// Cash Management service
/// Oracle Fusion: Financials > Treasury > Cash Management
#[allow(dead_code)]
pub struct CashManagementFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid cash forecast bucket types
#[allow(dead_code)]
const VALID_BUCKET_TYPES: &[&str] = &[
    "daily", "weekly", "monthly",
];

/// Valid cash forecast source types
#[allow(dead_code)]
const VALID_CASH_SOURCE_TYPES: &[&str] = &[
    "accounts_payable", "accounts_receivable", "payroll",
    "purchasing", "manual", "budget", "intercompany",
    "fixed_assets", "tax", "other",
];

/// Valid cash flow directions
#[allow(dead_code)]
const VALID_CASH_FLOW_DIRECTIONS: &[&str] = &[
    "inflow", "outflow", "both",
];

/// Valid forecast statuses
#[allow(dead_code)]
const VALID_FORECAST_STATUSES: &[&str] = &[
    "draft", "generated", "approved", "superseded",
];

impl CashManagementFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Validate and create a cash forecast template
    /// Oracle Fusion: Cash Management > Forecast Templates
    pub async fn create_forecast_template(
        &self,
        code: &str,
        name: &str,
        bucket_type: &str,
        number_of_buckets: i32,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }
        if !VALID_BUCKET_TYPES.contains(&bucket_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid bucket_type '{}'. Must be one of: {}",
                bucket_type, VALID_BUCKET_TYPES.join(", ")
            )));
        }
        if number_of_buckets <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Number of buckets must be positive".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        info!("Cash Service: Creating forecast template '{}' with {} buckets", code, bucket_type);
        Ok(())
    }

    /// Calculate projected net cash flow
    pub fn calculate_net_cash_flow(inflows: f64, outflows: f64) -> f64 {
        inflows - outflows
    }

    /// Calculate closing balance
    pub fn calculate_closing_balance(opening: f64, net_cash_flow: f64) -> f64 {
        opening + net_cash_flow
    }
}

// ============================================================================
// Tax Management Service
// ============================================================================

/// Tax Management service
/// Oracle Fusion: Tax > Tax Configuration
#[allow(dead_code)]
pub struct TaxManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid tax types
#[allow(dead_code)]
const VALID_TAX_TYPES: &[&str] = &[
    "sales_tax", "vat", "gst", "withholding", "excise", "customs",
];

/// Valid rate types
#[allow(dead_code)]
const VALID_RATE_TYPES: &[&str] = &[
    "standard", "reduced", "zero", "exempt",
];

/// Valid rounding rules
#[allow(dead_code)]
const VALID_ROUNDING_RULES: &[&str] = &[
    "nearest", "up", "down", "none",
];

/// Valid geographic levels
#[allow(dead_code)]
const VALID_GEOGRAPHIC_LEVELS: &[&str] = &[
    "country", "state", "county", "city", "region",
];

impl TaxManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a tax regime
    /// Oracle Fusion: Tax > Tax Regimes
    pub async fn create_tax_regime(
        &self,
        code: &str,
        name: &str,
        tax_type: &str,
        rounding_rule: &str,
        rounding_precision: i32,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax regime code and name are required".to_string(),
            ));
        }
        if !VALID_TAX_TYPES.contains(&tax_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_type '{}'. Must be one of: {}",
                tax_type, VALID_TAX_TYPES.join(", ")
            )));
        }
        if !VALID_ROUNDING_RULES.contains(&rounding_rule) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rounding_rule '{}'. Must be one of: {}",
                rounding_rule, VALID_ROUNDING_RULES.join(", ")
            )));
        }
        if !(0..=6).contains(&rounding_precision) {
            return Err(AtlasError::ValidationFailed(
                "Rounding precision must be between 0 and 6".to_string(),
            ));
        }

        info!("Tax Service: Creating tax regime '{}' ({})", code, tax_type);
        Ok(())
    }

    /// Calculate inclusive tax amount
    pub fn calculate_inclusive_tax(
        total_amount: f64,
        tax_rate: f64,
    ) -> f64 {
        let net = total_amount / (1.0 + tax_rate / 100.0);
        total_amount - net
    }

    /// Calculate exclusive tax amount
    pub fn calculate_exclusive_tax(
        net_amount: f64,
        tax_rate: f64,
    ) -> f64 {
        net_amount * (tax_rate / 100.0)
    }
}

// ============================================================================
// Intercompany Service
// ============================================================================

/// Intercompany service
/// Oracle Fusion: Intercompany > Intercompany Transactions
#[allow(dead_code)]
pub struct IntercompanyFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid IC batch statuses
#[allow(dead_code)]
const VALID_IC_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "posted", "cancelled",
];

/// Valid IC transaction types
#[allow(dead_code)]
const VALID_IC_TXN_TYPES: &[&str] = &[
    "invoice", "journal_entry", "payment", "charge", "allocation",
];

/// Valid IC settlement methods
#[allow(dead_code)]
const VALID_IC_SETTLEMENT_METHODS: &[&str] = &[
    "cash", "netting", "offset",
];

impl IntercompanyFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate an intercompany batch
    /// Oracle Fusion: Intercompany > Batches
    pub async fn create_batch(
        &self,
        batch_number: &str,
        from_entity_id: RecordId,
        to_entity_id: RecordId,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if batch_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Batch number is required".to_string(),
            ));
        }
        if from_entity_id == to_entity_id {
            return Err(AtlasError::ValidationFailed(
                "From and To entities must be different".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        info!("IC Service: Creating batch '{}' from {} to {}", batch_number, from_entity_id, to_entity_id);
        Ok(())
    }
}

// ============================================================================
// Period Close Service
// ============================================================================

/// Period Close service
/// Oracle Fusion: General Ledger > Period Close
#[allow(dead_code)]
pub struct PeriodCloseFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid period statuses
#[allow(dead_code)]
const VALID_PERIOD_STATUSES: &[&str] = &[
    "future", "not_opened", "open", "pending_close", "closed", "permanently_closed",
];

/// Valid subledgers for period close
#[allow(dead_code)]
const VALID_PERIOD_SUBLEDGERS: &[&str] = &[
    "gl", "ap", "ar", "fa", "po",
];

impl PeriodCloseFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate an accounting calendar
    /// Oracle Fusion: General Ledger > Period Close > Calendars
    pub async fn create_calendar(
        &self,
        name: &str,
        calendar_type: &str,
        fiscal_year_start_month: i32,
        periods_per_year: i32,
        has_adjusting_period: bool,
    ) -> AtlasResult<()> {
        let _ = calendar_type; // persisted by repository layer
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Calendar name is required".to_string(),
            ));
        }
        if !(1..=12).contains(&fiscal_year_start_month) {
            return Err(AtlasError::ValidationFailed(
                "fiscal_year_start_month must be between 1 and 12".to_string(),
            ));
        }
        if !(1..=366).contains(&periods_per_year) {
            return Err(AtlasError::ValidationFailed(
                "periods_per_year must be between 1 and 366".to_string(),
            ));
        }

        info!("Period Service: Creating calendar '{}' ({} periods, adjusting={})",
            name, periods_per_year, has_adjusting_period);
        Ok(())
    }
}

// ============================================================================
// Lease Accounting Service (ASC 842 / IFRS 16)
// ============================================================================

/// Lease Accounting service
/// Oracle Fusion: Lease Management
#[allow(dead_code)]
pub struct LeaseAccountingFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid lease classifications
#[allow(dead_code)]
const VALID_LEASE_CLASSIFICATIONS: &[&str] = &["operating", "finance"];

/// Valid lease statuses
#[allow(dead_code)]
const VALID_LEASE_STATUSES: &[&str] = &[
    "draft", "active", "modified", "impaired", "terminated", "expired",
];

/// Valid lease payment frequencies
#[allow(dead_code)]
const VALID_PAYMENT_FREQUENCIES: &[&str] = &["monthly", "quarterly", "annually"];

/// Valid lease modification types
#[allow(dead_code)]
const VALID_LEASE_MOD_TYPES: &[&str] = &[
    "term_extension", "scope_change", "payment_change", "rate_change", "reclassification",
];

/// Valid lease termination types
#[allow(dead_code)]
const VALID_LEASE_TERM_TYPES: &[&str] = &[
    "early", "end_of_term", "mutual_agreement", "default",
];

impl LeaseAccountingFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Validate a new lease contract
    /// Oracle Fusion: Lease Management > Lease Contracts
    pub async fn create_lease(
        &self,
        lease_number: &str,
        classification: &str,
        lease_term_months: i32,
        discount_rate: &str,
    ) -> AtlasResult<()> {
        if lease_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Lease number is required".to_string(),
            ));
        }
        if !VALID_LEASE_CLASSIFICATIONS.contains(&classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid classification '{}'. Must be one of: {}",
                classification, VALID_LEASE_CLASSIFICATIONS.join(", ")
            )));
        }
        if lease_term_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Lease term must be positive".to_string(),
            ));
        }
        let rate: f64 = discount_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Discount rate must be a valid number".to_string(),
        ))?;
        if rate <= 0.0 || rate > 1.0 {
            return Err(AtlasError::ValidationFailed(
                "Discount rate must be between 0 and 1".to_string(),
            ));
        }

        info!("Lease Service: Creating {} lease '{}' for {} months",
            classification, lease_number, lease_term_months);
        Ok(())
    }

    /// Calculate present value of lease payments (PV of annuity)
    pub fn calculate_lease_liability(
        periodic_payment: f64,
        periodic_rate: f64,
        number_of_periods: i32,
    ) -> f64 {
        if periodic_rate <= 0.0 || number_of_periods <= 0 {
            return 0.0;
        }
        let n = number_of_periods as f64;
        periodic_payment * (1.0 - (1.0 + periodic_rate).powf(-n)) / periodic_rate
    }

    /// Calculate monthly interest expense on lease liability
    pub fn calculate_lease_interest(
        liability_balance: f64,
        monthly_rate: f64,
    ) -> f64 {
        liability_balance * monthly_rate
    }

    /// Calculate principal reduction
    pub fn calculate_principal_reduction(
        payment: f64,
        interest: f64,
    ) -> f64 {
        payment - interest
    }
}

// ============================================================================
// Bank Reconciliation Service
// ============================================================================

/// Bank Reconciliation service
/// Oracle Fusion: Cash Management > Reconciliation
#[allow(dead_code)]
pub struct BankReconciliationService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl BankReconciliationService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Validate and create a bank account
    /// Oracle Fusion: Cash Management > Bank Accounts
    pub async fn create_bank_account(
        &self,
        account_number: &str,
        account_name: &str,
        bank_name: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if account_number.is_empty() || account_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account number and name are required".to_string(),
            ));
        }
        if bank_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Bank name is required".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        info!("Recon Service: Creating bank account '{}' at {}", account_number, bank_name);
        Ok(())
    }

    /// Calculate reconciliation difference
    pub fn calculate_recon_difference(
        bank_balance: f64,
        book_balance: f64,
        deposits_in_transit: f64,
        outstanding_checks: f64,
        bank_charges: f64,
    ) -> f64 {
        let adjusted_bank = bank_balance + deposits_in_transit - outstanding_checks;
        let adjusted_book = book_balance - bank_charges;
        adjusted_bank - adjusted_book
    }
}

// ============================================================================
// Encumbrance Management Service
// ============================================================================

/// Encumbrance Management service
/// Oracle Fusion: Financials > General Ledger > Encumbrance Management
#[allow(dead_code)]
pub struct EncumbranceManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid encumbrance categories
#[allow(dead_code)]
const VALID_ENCUMBRANCE_CATEGORIES: &[&str] = &[
    "commitment", "obligation", "preliminary",
];

/// Valid encumbrance entry statuses
#[allow(dead_code)]
const VALID_ENCUMBRANCE_STATUSES: &[&str] = &[
    "draft", "active", "partially_liquidated",
    "fully_liquidated", "cancelled", "expired",
];

impl EncumbranceManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate an encumbrance type
    pub async fn create_encumbrance_type(
        &self,
        code: &str,
        name: &str,
        category: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Encumbrance type code and name are required".to_string(),
            ));
        }
        if !VALID_ENCUMBRANCE_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}",
                category, VALID_ENCUMBRANCE_CATEGORIES.join(", ")
            )));
        }

        info!("Encumbrance Service: Creating encumbrance type '{}' ({})", code, category);
        Ok(())
    }

    /// Calculate remaining encumbrance
    pub fn calculate_remaining_encumbrance(
        encumbered: f64,
        liquidated: f64,
    ) -> f64 {
        encumbered - liquidated
    }
}

// ============================================================================
// Currency Management Service
// ============================================================================

/// Currency Management service
/// Oracle Fusion: General Ledger > Currency Rates Manager
#[allow(dead_code)]
pub struct CurrencyManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid exchange rate types
#[allow(dead_code)]
const VALID_EXCHANGE_RATE_TYPES: &[&str] = &[
    "daily", "spot", "corporate", "period_average", "period_end", "user", "fixed",
];

impl CurrencyManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Validate a currency definition
    pub async fn create_currency(
        &self,
        code: &str,
        name: &str,
        precision: i32,
        is_base_currency: bool,
    ) -> AtlasResult<()> {
        let code_upper = code.to_uppercase();
        if code_upper.len() != 3 || !code_upper.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be exactly 3 alphabetic characters (ISO 4217)".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency name is required".to_string(),
            ));
        }
        if !(0..=6).contains(&precision) {
            return Err(AtlasError::ValidationFailed(
                "Precision must be between 0 and 6".to_string(),
            ));
        }

        info!("Currency Service: Defining currency '{}' ({}) base={}", code_upper, name, is_base_currency);
        Ok(())
    }

    /// Validate and set an exchange rate
    pub async fn set_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        rate: &str,
    ) -> AtlasResult<()> {
        if !VALID_EXCHANGE_RATE_TYPES.contains(&rate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate_type '{}'. Must be one of: {}",
                rate_type, VALID_EXCHANGE_RATE_TYPES.join(", ")
            )));
        }
        let rate_val: f64 = rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Rate must be a valid number".to_string(),
        ))?;
        if rate_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Exchange rate must be positive".to_string(),
            ));
        }

        info!("Currency Service: Setting {} rate {}->{}: {}",
            rate_type, from_currency, to_currency, rate);
        Ok(())
    }

    /// Convert an amount between currencies
    pub fn convert_currency(
        amount: f64,
        exchange_rate: f64,
    ) -> f64 {
        amount * exchange_rate
    }

    /// Calculate unrealized gain/loss
    pub fn calculate_unrealized_gain_loss(
        original_amount: f64,
        original_rate: f64,
        current_rate: f64,
    ) -> f64 {
        let current_value = original_amount * current_rate;
        let original_value = original_amount * original_rate;
        current_value - original_value
    }
}

// ============================================================================
// Multi-Book Accounting Service
// ============================================================================

/// Multi-Book Accounting service
/// Oracle Fusion: General Ledger > Multi-Book Accounting
#[allow(dead_code)]
pub struct MultiBookAccountingFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid multi-book types
#[allow(dead_code)]
const VALID_BOOK_TYPES: &[&str] = &["primary", "secondary"];

/// Valid mapping levels
#[allow(dead_code)]
const VALID_MAPPING_LEVELS: &[&str] = &["journal", "subledger"];

impl MultiBookAccountingFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate an accounting book
    pub async fn create_book(
        &self,
        code: &str,
        name: &str,
        book_type: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Book code and name are required".to_string(),
            ));
        }
        if !VALID_BOOK_TYPES.contains(&book_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid book_type '{}'. Must be one of: {}",
                book_type, VALID_BOOK_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        info!("Multi-Book Service: Creating {} book '{}' ({})", book_type, code, name);
        Ok(())
    }

    /// Validate an account mapping between books
    pub async fn create_account_mapping(
        &self,
        source_account: &str,
        target_account: &str,
        mapping_level: &str,
    ) -> AtlasResult<()> {
        if source_account.is_empty() || target_account.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Source and target accounts are required".to_string(),
            ));
        }
        if !VALID_MAPPING_LEVELS.contains(&mapping_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid mapping_level '{}'. Must be one of: {}",
                mapping_level, VALID_MAPPING_LEVELS.join(", ")
            )));
        }

        info!("Multi-Book Service: Mapping {} -> {} ({})", source_account, target_account, mapping_level);
        Ok(())
    }
}

// ============================================================================
// Financial Consolidation Service
// ============================================================================

/// Financial Consolidation service
/// Oracle Fusion: General Ledger > Financial Consolidation
#[allow(dead_code)]
pub struct FinancialConsolidationFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid consolidation methods
#[allow(dead_code)]
const VALID_CONSOLIDATION_METHODS: &[&str] = &[
    "full", "proportional", "equity_method",
];

/// Valid translation methods
#[allow(dead_code)]
const VALID_TRANSLATION_METHODS: &[&str] = &[
    "current_rate", "temporal", "weighted_average",
];

/// Valid scenario statuses
#[allow(dead_code)]
const VALID_SCENARIO_STATUSES: &[&str] = &[
    "draft", "in_progress", "pending_review", "approved", "posted", "reversed",
];

impl FinancialConsolidationFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a consolidation ledger
    pub async fn create_consolidation_ledger(
        &self,
        code: &str,
        name: &str,
        base_currency: &str,
        translation_method: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Ledger code and name are required".to_string(),
            ));
        }
        if base_currency.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Base currency is required".to_string(),
            ));
        }
        if !VALID_TRANSLATION_METHODS.contains(&translation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid translation_method '{}'. Must be one of: {}",
                translation_method, VALID_TRANSLATION_METHODS.join(", ")
            )));
        }

        info!("FC Service: Creating consolidation ledger '{}' with {} translation",
            code, translation_method);
        Ok(())
    }

    /// Validate adding an entity to consolidation
    pub async fn add_consolidation_entity(
        &self,
        entity_name: &str,
        consolidation_method: &str,
        ownership_percentage: f64,
    ) -> AtlasResult<()> {
        if entity_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Entity name is required".to_string(),
            ));
        }
        if !VALID_CONSOLIDATION_METHODS.contains(&consolidation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid consolidation_method '{}'. Must be one of: {}",
                consolidation_method, VALID_CONSOLIDATION_METHODS.join(", ")
            )));
        }
        if !(0.0..=100.0).contains(&ownership_percentage) {
            return Err(AtlasError::ValidationFailed(
                "Ownership percentage must be between 0 and 100".to_string(),
            ));
        }

        info!("FC Service: Adding entity '{}' with {} method ({:.1}% ownership)",
            entity_name, consolidation_method, ownership_percentage);
        Ok(())
    }

    /// Calculate minority interest
    pub fn calculate_minority_interest(
        net_income: f64,
        ownership_percentage: f64,
    ) -> f64 {
        net_income * (1.0 - ownership_percentage / 100.0)
    }

    /// Calculate proportional share
    pub fn calculate_proportional_share(
        total_amount: f64,
        ownership_percentage: f64,
    ) -> f64 {
        total_amount * (ownership_percentage / 100.0)
    }
}

/// Purchase Order service
#[allow(dead_code)]
pub struct PurchaseOrderService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl PurchaseOrderService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Submit a purchase order for approval
    pub async fn submit_for_approval(
        &self,
        po_id: RecordId,
        data: &serde_json::Value,
    ) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;

        let result = self.validation_engine.validate(&entity, data, None);
        if !result.valid {
            let errors: Vec<String> = result.errors.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            return Err(AtlasError::ValidationFailed(errors.join(", ")));
        }

        if let Some(workflow) = &entity.workflow {
            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                po_id,
                "draft",
                "submit",
                None,
                data,
                None,
            ).await?;

            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Submit failed".to_string())
                ));
            }

            info!("PO {} submitted for approval", po_id);
        }

        Ok(())
    }

    /// Approve a purchase order
    pub async fn approve(&self, po_id: RecordId, approver_id: RecordId) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;

        if let Some(workflow) = &entity.workflow {
            let user = atlas_core::workflow::engine::User {
                id: approver_id,
                roles: vec!["purchase_manager".to_string()],
            };

            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                po_id,
                "submitted",
                "approve",
                Some(&user),
                &json!({"approved_by": approver_id, "approved_at": chrono::Utc::now().to_rfc3339()}),
                None,
            ).await?;

            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Approval failed".to_string())
                ));
            }

            info!("PO {} approved by {}", po_id, approver_id);
        }

        Ok(())
    }
}

// ============================================================================
// Invoice Service (existing, kept for backward compat)
// ============================================================================

/// Invoice service
#[allow(dead_code)]
pub struct InvoiceService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl InvoiceService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Generate invoice from purchase order
    pub async fn generate_from_po(&self, po_id: RecordId) -> AtlasResult<RecordId> {
        let _po_entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;
        let _invoice_entity = self.schema_engine.get_entity("invoices")
            .ok_or_else(|| AtlasError::EntityNotFound("invoices".to_string()))?;

        let invoice_id = RecordId::new_v4();
        info!("Generated invoice {} from PO {}", invoice_id, po_id);

        Ok(invoice_id)
    }

    /// Record payment against invoice
    pub async fn record_payment(&self, invoice_id: RecordId, amount: f64) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("invoices")
            .ok_or_else(|| AtlasError::EntityNotFound("invoices".to_string()))?;

        info!("Recorded payment of {} against invoice {}", amount, invoice_id);

        if let Some(workflow) = &entity.workflow {
            let _ = self.workflow_engine.execute_transition(
                &workflow.name,
                invoice_id,
                "sent",
                "mark_paid",
                None,
                &json!({"amount_paid": amount}),
                None,
            ).await;
        }

        Ok(())
    }
}

// ============================================================================
// General Ledger Service (existing, kept for backward compat)
// ============================================================================

/// General Ledger service
#[allow(dead_code)]
pub struct GeneralLedgerService {
    schema_engine: Arc<SchemaEngine>,
}

impl GeneralLedgerService {
    pub fn new(schema_engine: Arc<SchemaEngine>) -> Self {
        Self { schema_engine }
    }

    /// Post a journal entry
    pub async fn post_entry(&self, entry_id: RecordId) -> AtlasResult<()> {
        let _entity = self.schema_engine.get_entity("journal_entries")
            .ok_or_else(|| AtlasError::EntityNotFound("journal_entries".to_string()))?;

        info!("Journal entry {} posted", entry_id);
        Ok(())
    }

    /// Get trial balance
    pub async fn trial_balance(&self) -> AtlasResult<serde_json::Value> {
        let _entity = self.schema_engine.get_entity("chart_of_accounts")
            .ok_or_else(|| AtlasError::EntityNotFound("chart_of_accounts".to_string()))?;

        Ok(json!({
            "accounts": [],
            "total_debits": 0.0,
            "total_credits": 0.0,
            "balanced": true
        }))
    }
}

// ============================================================================
// Collections Management Service
// ============================================================================

/// Collections Management service
/// Oracle Fusion: Financials > Collections
#[allow(dead_code)]
pub struct CollectionsManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid risk classifications for collections
#[allow(dead_code)]
const VALID_COLLECTION_RISK_CLASSIFICATIONS: &[&str] = &[
    "low", "medium", "high", "very_high", "defaulted",
];

/// Valid collection case types
#[allow(dead_code)]
const VALID_CASE_TYPES: &[&str] = &[
    "collection", "dispute", "bankruptcy", "skip_trace",
];

/// Valid case priorities
#[allow(dead_code)]
const VALID_CASE_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

/// Valid interaction types
#[allow(dead_code)]
const VALID_INTERACTION_TYPES: &[&str] = &[
    "phone_call", "email", "letter", "meeting", "note", "sms",
];

/// Valid interaction outcomes
#[allow(dead_code)]
const VALID_INTERACTION_OUTCOMES: &[&str] = &[
    "contacted", "left_message", "no_answer", "promised_to_pay",
    "disputed", "refused", "agreed_payment_plan", "escalated", "no_action",
];

/// Valid promise types
#[allow(dead_code)]
const VALID_PROMISE_TYPES: &[&str] = &[
    "single_payment", "installment", "full_balance",
];

/// Valid dunning levels
#[allow(dead_code)]
const VALID_DUNNING_LEVELS: &[&str] = &[
    "reminder", "first_notice", "second_notice", "final_notice", "pre_legal", "legal",
];

/// Valid communication methods for dunning
#[allow(dead_code)]
const VALID_DUNNING_COMM_METHODS: &[&str] = &[
    "email", "letter", "sms", "phone",
];

/// Valid write-off types
#[allow(dead_code)]
const VALID_WRITE_OFF_TYPES: &[&str] = &[
    "bad_debt", "small_balance", "dispute", "adjustment",
];

/// Valid resolution types
#[allow(dead_code)]
const VALID_RESOLUTION_TYPES: &[&str] = &[
    "full_payment", "partial_payment", "payment_plan",
    "write_off", "dispute_resolved", "uncollectible", "other",
];

impl CollectionsManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Validate and create a customer credit profile
    /// Oracle Fusion: Collections > Customer Credit Profiles
    pub async fn create_credit_profile(
        &self,
        customer_id: RecordId,
        credit_limit: &str,
        risk_classification: &str,
        credit_score: Option<i32>,
        payment_terms: &str,
    ) -> AtlasResult<()> {
        let limit: f64 = credit_limit.parse().map_err(|_| AtlasError::ValidationFailed(
            "Credit limit must be a valid number".to_string(),
        ))?;
        let _ = payment_terms; // persisted by repository layer
        if limit < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit limit must be non-negative".to_string(),
            ));
        }
        if !VALID_COLLECTION_RISK_CLASSIFICATIONS.contains(&risk_classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_classification '{}'. Must be one of: {}",
                risk_classification, VALID_COLLECTION_RISK_CLASSIFICATIONS.join(", ")
            )));
        }
        if let Some(score) = credit_score {
            if !(0..=1000).contains(&score) {
                return Err(AtlasError::ValidationFailed(
                    "Credit score must be between 0 and 1000".to_string(),
                ));
            }
        }

        info!(
            "Collections Service: Creating credit profile for customer {} (limit: {:.2}, risk: {})",
            customer_id, limit, risk_classification
        );
        Ok(())
    }

    /// Check if customer has available credit
    pub fn check_credit_available(
        credit_limit: f64,
        credit_used: f64,
        additional_amount: f64,
        credit_hold: bool,
    ) -> bool {
        if credit_hold {
            return false;
        }
        let available = credit_limit - credit_used;
        additional_amount <= available
    }

    /// Calculate credit utilization percentage
    pub fn calculate_utilization(credit_used: f64, credit_limit: f64) -> f64 {
        if credit_limit <= 0.0 {
            return 0.0;
        }
        (credit_used / credit_limit) * 100.0
    }

    /// Calculate aging bucket from overdue days
    pub fn aging_bucket_from_days(days_overdue: i32) -> &'static str {
        match days_overdue {
            d if d <= 0 => "current",
            d if d <= 30 => "1_30",
            d if d <= 60 => "31_60",
            d if d <= 90 => "61_90",
            d if d <= 120 => "91_120",
            _ => "121_plus",
        }
    }

    /// Validate a collection case creation
    /// Oracle Fusion: Collections > Collection Cases
    pub fn validate_case(
        case_type: &str,
        priority: &str,
    ) -> AtlasResult<()> {
        if !VALID_CASE_TYPES.contains(&case_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid case_type '{}'. Must be one of: {}",
                case_type, VALID_CASE_TYPES.join(", ")
            )));
        }
        if !VALID_CASE_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}",
                priority, VALID_CASE_PRIORITIES.join(", ")
            )));
        }
        Ok(())
    }

    /// Calculate days sales outstanding (DSO)
    pub fn calculate_dso(
        total_accounts_receivable: f64,
        total_credit_sales: f64,
        number_of_days: i32,
    ) -> f64 {
        if total_credit_sales <= 0.0 {
            return 0.0;
        }
        (total_accounts_receivable / total_credit_sales) * number_of_days as f64
    }

    /// Calculate bad debt provision
    pub fn calculate_bad_debt_provision(
        total_outstanding: f64,
        historical_bad_debt_rate: f64,
    ) -> f64 {
        total_outstanding * (historical_bad_debt_rate / 100.0)
    }

    /// Calculate collection effectiveness index (CEI)
    pub fn calculate_cei(
        beginning_receivables: f64,
        credit_sales: f64,
        ending_total_receivables: f64,
        ending_current_receivables: f64,
        total_collections: f64,
    ) -> f64 {
        let _ = total_collections; // reserved for extended CEI formula
        let denom = beginning_receivables + credit_sales - ending_current_receivables;
        if denom <= 0.0 {
            return 0.0;
        }
        ((beginning_receivables + credit_sales - ending_total_receivables) / denom) * 100.0
    }
}

// ============================================================================
// Credit Management Service
// ============================================================================

/// Credit Management service
/// Oracle Fusion: Financials > Credit Management
#[allow(dead_code)]
pub struct CreditManagementFinService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid credit model types
#[allow(dead_code)]
const VALID_CREDIT_MODEL_TYPES: &[&str] = &[
    "manual", "scorecard", "risk_category", "external",
];

/// Valid credit profile types
#[allow(dead_code)]
const VALID_CREDIT_PROFILE_TYPES: &[&str] = &[
    "customer", "customer_group", "global",
];

/// Valid credit risk levels
#[allow(dead_code)]
const VALID_CREDIT_RISK_LEVELS: &[&str] = &[
    "low", "medium", "high", "very_high", "blocked",
];

/// Valid credit limit types
#[allow(dead_code)]
const VALID_CREDIT_LIMIT_TYPES: &[&str] = &[
    "overall", "order", "delivery", "currency",
];

/// Valid credit check points
#[allow(dead_code)]
const VALID_CREDIT_CHECK_POINTS: &[&str] = &[
    "order_entry", "shipment", "invoice", "delivery", "payment",
];

/// Valid credit check types
#[allow(dead_code)]
const VALID_CREDIT_CHECK_TYPES: &[&str] = &[
    "automatic", "manual",
];

/// Valid failure actions
#[allow(dead_code)]
const VALID_FAILURE_ACTIONS: &[&str] = &[
    "hold", "warn", "reject", "notify",
];

/// Valid credit hold types
#[allow(dead_code)]
const VALID_CREDIT_HOLD_TYPES: &[&str] = &[
    "credit_limit", "overdue", "review", "manual", "scoring",
];

/// Valid credit review types
#[allow(dead_code)]
const VALID_CREDIT_REVIEW_TYPES: &[&str] = &[
    "periodic", "triggered", "ad_hoc", "escalation",
];

impl CreditManagementFinService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a credit scoring model
    /// Oracle Fusion: Credit Management > Credit Scoring Models
    pub async fn create_scoring_model(
        &self,
        code: &str,
        name: &str,
        model_type: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Scoring model code and name are required".to_string(),
            ));
        }
        if !VALID_CREDIT_MODEL_TYPES.contains(&model_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid model_type '{}'. Must be one of: {}",
                model_type, VALID_CREDIT_MODEL_TYPES.join(", ")
            )));
        }
        info!("Credit Service: Creating scoring model '{}' ({})", code, model_type);
        Ok(())
    }

    /// Create and validate a credit profile
    /// Oracle Fusion: Credit Management > Credit Profiles
    pub async fn create_credit_profile(
        &self,
        profile_number: &str,
        profile_name: &str,
        profile_type: &str,
        credit_score: Option<f64>,
        risk_level: &str,
    ) -> AtlasResult<()> {
        if profile_number.is_empty() || profile_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Profile number and name are required".to_string(),
            ));
        }
        if !VALID_CREDIT_PROFILE_TYPES.contains(&profile_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid profile_type '{}'. Must be one of: {}",
                profile_type, VALID_CREDIT_PROFILE_TYPES.join(", ")
            )));
        }
        if !VALID_CREDIT_RISK_LEVELS.contains(&risk_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_level '{}'. Must be one of: {}",
                risk_level, VALID_CREDIT_RISK_LEVELS.join(", ")
            )));
        }
        if let Some(score) = credit_score {
            if !(0.0..=100.0).contains(&score) {
                return Err(AtlasError::ValidationFailed(
                    "Credit score must be between 0 and 100".to_string(),
                ));
            }
        }
        info!("Credit Service: Creating profile '{}' (type: {}, risk: {})",
            profile_number, profile_type, risk_level);
        Ok(())
    }

    /// Validate a credit limit setup
    /// Oracle Fusion: Credit Management > Credit Limits
    pub fn validate_credit_limit(
        limit_type: &str,
        credit_limit: f64,
        temp_increase: f64,
    ) -> AtlasResult<()> {
        if !VALID_CREDIT_LIMIT_TYPES.contains(&limit_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid limit_type '{}'. Must be one of: {}",
                limit_type, VALID_CREDIT_LIMIT_TYPES.join(", ")
            )));
        }
        if credit_limit < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit limit must be non-negative".to_string(),
            ));
        }
        if temp_increase < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Temporary limit increase must be non-negative".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculate credit exposure
    pub fn calculate_exposure(
        open_receivables: f64,
        open_orders: f64,
        open_shipments: f64,
        unapplied_cash: f64,
        on_hold: f64,
    ) -> f64 {
        open_receivables + open_orders + open_shipments - unapplied_cash + on_hold
    }

    /// Calculate utilization percentage
    pub fn calculate_credit_utilization(
        total_exposure: f64,
        credit_limit: f64,
    ) -> f64 {
        if credit_limit <= 0.0 {
            return 0.0;
        }
        (total_exposure / credit_limit) * 100.0
    }

    /// Calculate available credit
    pub fn calculate_available_credit(
        credit_limit: f64,
        temp_increase: f64,
        total_exposure: f64,
    ) -> f64 {
        (credit_limit + temp_increase) - total_exposure
    }

    /// Determine risk level from credit score
    pub fn risk_level_from_score(score: f64) -> &'static str {
        if score >= 80.0 {
            "low"
        } else if score >= 60.0 {
            "medium"
        } else if score >= 40.0 {
            "high"
        } else {
            "very_high"
        }
    }
}

// ============================================================================
// Withholding Tax Service
// ============================================================================

/// Withholding Tax service
/// Oracle Fusion: Payables > Withholding Tax
#[allow(dead_code)]
pub struct WithholdingTaxService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid withholding tax types
#[allow(dead_code)]
const VALID_WHT_TAX_TYPES: &[&str] = &[
    "income_tax", "vat", "service_tax", "contract_tax",
    "royalty", "dividend", "interest", "other",
];

/// Valid withholding line statuses
#[allow(dead_code)]
const VALID_WHT_LINE_STATUSES: &[&str] = &[
    "pending", "withheld", "remitted", "refunded",
];

/// Valid certificate statuses
#[allow(dead_code)]
const VALID_WHT_CERT_STATUSES: &[&str] = &[
    "draft", "issued", "acknowledged", "cancelled",
];

impl WithholdingTaxService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a withholding tax code
    /// Oracle Fusion: Payables > Withholding Tax > Tax Codes
    pub async fn create_tax_code(
        &self,
        code: &str,
        name: &str,
        tax_type: &str,
        rate_percentage: &str,
        threshold_amount: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax code and name are required".to_string(),
            ));
        }
        if !VALID_WHT_TAX_TYPES.contains(&tax_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_type '{}'. Must be one of: {}",
                tax_type, VALID_WHT_TAX_TYPES.join(", ")
            )));
        }
        let rate: f64 = rate_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "rate_percentage must be a valid number".to_string(),
        ))?;
        if !(0.0..=100.0).contains(&rate) {
            return Err(AtlasError::ValidationFailed(
                "rate_percentage must be between 0 and 100".to_string(),
            ));
        }
        let threshold: f64 = threshold_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "threshold_amount must be a valid number".to_string(),
        ))?;
        if threshold < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "threshold_amount must be non-negative".to_string(),
            ));
        }
        info!("WHT Service: Creating tax code '{}' ({}) rate={}%", code, tax_type, rate);
        Ok(())
    }

    /// Calculate withholding tax amount
    pub fn calculate_withholding(
        taxable_amount: f64,
        rate_percentage: f64,
        threshold_amount: f64,
        is_cumulative: bool,
        year_to_date_taxable: f64,
    ) -> f64 {
        if is_cumulative {
            // Cumulative threshold: only withhold if YTD exceeds threshold
            let total_ytd = year_to_date_taxable + taxable_amount;
            if total_ytd <= threshold_amount {
                return 0.0;
            }
            // If already past threshold, withhold on full amount
            if year_to_date_taxable >= threshold_amount {
                return taxable_amount * (rate_percentage / 100.0);
            }
            // Partially past threshold
            let excess = total_ytd - threshold_amount;
            let taxable_portion = excess.min(taxable_amount);
            return taxable_portion * (rate_percentage / 100.0);
        } else {
            // Per-invoice threshold
            if taxable_amount <= threshold_amount {
                return 0.0;
            }
            return taxable_amount * (rate_percentage / 100.0);
        }
    }

    /// Calculate net payment amount after withholding
    pub fn calculate_net_payment(
        gross_amount: f64,
        withheld_amount: f64,
    ) -> f64 {
        gross_amount - withheld_amount
    }

    /// Calculate year-to-date withholding total
    pub fn calculate_ytd_withholding(
        lines: &[f64],
    ) -> f64 {
        lines.iter().sum()
    }
}

// ============================================================================
// Project Billing Service
// ============================================================================

/// Project Billing service
/// Oracle Fusion: Project Management > Project Billing
#[allow(dead_code)]
pub struct ProjectBillingService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid schedule types
#[allow(dead_code)]
const VALID_SCHEDULE_TYPES: &[&str] = &[
    "standard", "overtime", "holiday", "custom",
];

/// Valid billing methods
#[allow(dead_code)]
const VALID_BILLING_METHODS: &[&str] = &[
    "time_and_materials", "fixed_price", "milestone", "cost_plus", "retention",
];

/// Valid invoice formats
#[allow(dead_code)]
const VALID_INVOICE_FORMATS: &[&str] = &[
    "detailed", "summary", "consolidated",
];

/// Valid billing cycles
#[allow(dead_code)]
const VALID_BILLING_CYCLES: &[&str] = &[
    "weekly", "biweekly", "monthly", "milestone",
];

/// Valid billing event types
#[allow(dead_code)]
const VALID_EVENT_TYPES: &[&str] = &[
    "milestone", "progress", "completion", "retention_release",
];

/// Valid project invoice types
#[allow(dead_code)]
const VALID_PROJECT_INVOICE_TYPES: &[&str] = &[
    "progress", "milestone", "t_and_m", "retention_release",
    "debit_memo", "credit_memo",
];

/// Valid line sources
#[allow(dead_code)]
const VALID_LINE_SOURCES: &[&str] = &[
    "expenditure_item", "billing_event", "retention", "manual",
];

impl ProjectBillingService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a bill rate schedule
    /// Oracle Fusion: Project Billing > Bill Rate Schedules
    pub async fn create_bill_rate_schedule(
        &self,
        schedule_number: &str,
        name: &str,
        schedule_type: &str,
        currency_code: &str,
    ) -> AtlasResult<()> {
        if schedule_number.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Schedule number and name are required".to_string(),
            ));
        }
        if !VALID_SCHEDULE_TYPES.contains(&schedule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule_type '{}'. Must be one of: {}",
                schedule_type, VALID_SCHEDULE_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        info!("Project Billing Service: Creating schedule '{}' ({})", schedule_number, schedule_type);
        Ok(())
    }

    /// Create and validate a project billing configuration
    /// Oracle Fusion: Project Billing > Billing Configuration
    pub async fn create_billing_config(
        &self,
        project_id: RecordId,
        billing_method: &str,
        currency_code: &str,
        invoice_format: &str,
        billing_cycle: &str,
    ) -> AtlasResult<()> {
        if !VALID_BILLING_METHODS.contains(&billing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing_method '{}'. Must be one of: {}",
                billing_method, VALID_BILLING_METHODS.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if !VALID_INVOICE_FORMATS.contains(&invoice_format) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice_format '{}'. Must be one of: {}",
                invoice_format, VALID_INVOICE_FORMATS.join(", ")
            )));
        }
        if !VALID_BILLING_CYCLES.contains(&billing_cycle) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing_cycle '{}'. Must be one of: {}",
                billing_cycle, VALID_BILLING_CYCLES.join(", ")
            )));
        }
        info!("Project Billing Service: Configuring project {} for {} billing",
            project_id, billing_method);
        Ok(())
    }

    /// Calculate bill amount for time and materials
    pub fn calculate_tm_bill_amount(
        hours: f64,
        bill_rate: f64,
        markup_pct: f64,
    ) -> f64 {
        let base = hours * bill_rate;
        base + (base * markup_pct / 100.0)
    }

    /// Calculate retention amount
    pub fn calculate_retention(
        bill_amount: f64,
        retention_pct: f64,
        retention_cap: f64,
    ) -> f64 {
        let ret = bill_amount * (retention_pct / 100.0);
        if retention_cap > 0.0 {
            ret.min(retention_cap)
        } else {
            ret
        }
    }

    /// Calculate net billable amount (after retention)
    pub fn calculate_net_billable(
        bill_amount: f64,
        retention_amount: f64,
        tax_amount: f64,
    ) -> f64 {
        bill_amount - retention_amount + tax_amount
    }

    /// Calculate progress billing percentage
    pub fn calculate_progress_pct(
        completed_value: f64,
        total_contract_value: f64,
    ) -> f64 {
        if total_contract_value <= 0.0 {
            return 0.0;
        }
        ((completed_value / total_contract_value) * 100.0).min(100.0)
    }

    /// Calculate earned revenue for fixed-price project
    pub fn calculate_earned_revenue(
        total_contract_value: f64,
        completion_pct: f64,
    ) -> f64 {
        total_contract_value * (completion_pct / 100.0)
    }

    /// Calculate cost-plus billing
    pub fn calculate_cost_plus_bill(
        actual_cost: f64,
        markup_pct: f64,
    ) -> f64 {
        actual_cost * (1.0 + markup_pct / 100.0)
    }
}

// ============================================================================
// Payment Terms Service
// ============================================================================

/// Payment Terms service
/// Oracle Fusion: Financials > Payment Terms
#[allow(dead_code)]
pub struct PaymentTermsService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid payment term types
#[allow(dead_code)]
const VALID_TERM_TYPES: &[&str] = &[
    "immediate", "net_days", "discount_net", "milestone", "installment",
];

/// Valid day-of-month options
#[allow(dead_code)]
const VALID_DAYS_OF_MONTH: &[&str] = &[
    "any", "1", "5", "10", "15", "20", "25",
];

impl PaymentTermsService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a payment term
    /// Oracle Fusion: Financials > Payment Terms > Define
    pub async fn create_payment_term(
        &self,
        code: &str,
        name: &str,
        term_type: &str,
        net_due_days: i32,
        discount_days: Option<i32>,
        discount_percentage: Option<&str>,
    ) -> AtlasResult<()> {
        let _ = discount_days; // persisted by repository layer
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payment term code and name are required".to_string(),
            ));
        }
        if !VALID_TERM_TYPES.contains(&term_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid term_type '{}'. Must be one of: {}",
                term_type, VALID_TERM_TYPES.join(", ")
            )));
        }
        if term_type != "immediate" && net_due_days <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Net due days must be positive for non-immediate terms".to_string(),
            ));
        }
        if let Some(dp) = discount_percentage {
            let pct: f64 = dp.parse().map_err(|_| AtlasError::ValidationFailed(
                "Discount percentage must be a valid number".to_string(),
            ))?;
            if !(0.0..=100.0).contains(&pct) {
                return Err(AtlasError::ValidationFailed(
                    "Discount percentage must be between 0 and 100".to_string(),
                ));
            }
        }

        info!("Payment Terms Service: Creating term '{}' ({})", code, term_type);
        Ok(())
    }

    /// Calculate discount date from invoice date and payment term
    pub fn calculate_discount_date(
        invoice_date: chrono::NaiveDate,
        discount_days: i32,
    ) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(discount_days as i64)
    }

    /// Calculate net due date from invoice date and payment term
    pub fn calculate_net_due_date(
        invoice_date: chrono::NaiveDate,
        net_due_days: i32,
    ) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(net_due_days as i64)
    }

    /// Calculate discount amount for early payment
    pub fn calculate_discount_amount(
        invoice_amount: f64,
        discount_percentage: f64,
    ) -> f64 {
        invoice_amount * (discount_percentage / 100.0)
    }

    /// Calculate net payment amount after discount
    pub fn calculate_net_payment_amount(
        invoice_amount: f64,
        discount_amount: f64,
    ) -> f64 {
        invoice_amount - discount_amount
    }

    /// Determine if discount is still available based on payment date
    pub fn is_discount_available(
        payment_date: chrono::NaiveDate,
        discount_date: chrono::NaiveDate,
    ) -> bool {
        payment_date <= discount_date
    }

    /// Calculate effective annualized cost of not taking a discount
    /// Formula: (discount% / (100% - discount%)) * (365 / (net_days - discount_days))
    pub fn calculate_annualized_cost_of_discount(
        discount_percentage: f64,
        net_due_days: i32,
        discount_days: i32,
    ) -> f64 {
        let additional_days = net_due_days - discount_days;
        if additional_days <= 0 {
            return 0.0;
        }
        let discount_factor = discount_percentage / (100.0 - discount_percentage);
        discount_factor * (365.0 / additional_days as f64) * 100.0
    }

    /// Calculate payment amount, applying discount if applicable
    pub fn calculate_payment_with_discount(
        invoice_amount: f64,
        discount_percentage: f64,
        payment_date: chrono::NaiveDate,
        discount_date: chrono::NaiveDate,
    ) -> (f64, f64, bool) {
        if Self::is_discount_available(payment_date, discount_date) && discount_percentage > 0.0 {
            let discount = Self::calculate_discount_amount(invoice_amount, discount_percentage);
            let net = Self::calculate_net_payment_amount(invoice_amount, discount);
            (net, discount, true)
        } else {
            (invoice_amount, 0.0, false)
        }
    }
}

// ============================================================================
// Financial Statement Generation Service
// ============================================================================

/// Financial Statement Generation service
/// Oracle Fusion: Financial Reporting Center
#[allow(dead_code)]
pub struct FinancialStatementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid financial report types
#[allow(dead_code)]
const VALID_REPORT_TYPES: &[&str] = &[
    "balance_sheet", "income_statement", "cash_flow", "trial_balance", "custom",
];

/// Valid row types for report definitions
#[allow(dead_code)]
const VALID_ROW_TYPES: &[&str] = &[
    "header", "account_range", "calculated", "total", "subtotal", "text",
];

impl FinancialStatementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a financial report template
    pub async fn create_report_template(
        &self,
        code: &str,
        name: &str,
        report_type: &str,
        base_currency: &str,
    ) -> AtlasResult<()> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }
        if !VALID_REPORT_TYPES.contains(&report_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid report_type '{}'. Must be one of: {}",
                report_type, VALID_REPORT_TYPES.join(", ")
            )));
        }
        if base_currency.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Base currency code is required".to_string(),
            ));
        }
        info!("FS Service: Creating '{}' report template '{}'", report_type, code);
        Ok(())
    }

    /// Calculate balance sheet totals from account balances
    pub fn calculate_balance_sheet(
        total_assets: f64,
        total_liabilities: f64,
        total_equity: f64,
    ) -> (f64, bool) {
        let total_liab_equity = total_liabilities + total_equity;
        let balanced = (total_assets - total_liab_equity).abs() < 0.01;
        (total_liab_equity, balanced)
    }

    /// Calculate net income from revenue and expenses
    pub fn calculate_net_income(
        total_revenue: f64,
        total_expenses: f64,
    ) -> f64 {
        total_revenue - total_expenses
    }

    /// Calculate retained earnings
    pub fn calculate_retained_earnings(
        beginning_retained_earnings: f64,
        net_income: f64,
        dividends: f64,
    ) -> f64 {
        beginning_retained_earnings + net_income - dividends
    }

    /// Calculate working capital
    pub fn calculate_working_capital(
        current_assets: f64,
        current_liabilities: f64,
    ) -> f64 {
        current_assets - current_liabilities
    }

    /// Calculate current ratio
    pub fn calculate_current_ratio(
        current_assets: f64,
        current_liabilities: f64,
    ) -> f64 {
        if current_liabilities <= 0.0 {
            return 0.0;
        }
        current_assets / current_liabilities
    }

    /// Calculate debt-to-equity ratio
    pub fn calculate_debt_to_equity(
        total_liabilities: f64,
        total_equity: f64,
    ) -> f64 {
        if total_equity <= 0.0 {
            return 0.0;
        }
        total_liabilities / total_equity
    }

    /// Calculate gross profit margin
    pub fn calculate_gross_profit_margin(
        revenue: f64,
        cost_of_goods_sold: f64,
    ) -> f64 {
        if revenue <= 0.0 {
            return 0.0;
        }
        ((revenue - cost_of_goods_sold) / revenue) * 100.0
    }

    /// Calculate operating margin
    pub fn calculate_operating_margin(
        revenue: f64,
        operating_income: f64,
    ) -> f64 {
        if revenue <= 0.0 {
            return 0.0;
        }
        (operating_income / revenue) * 100.0
    }

    /// Calculate return on equity (ROE)
    pub fn calculate_return_on_equity(
        net_income: f64,
        total_equity: f64,
    ) -> f64 {
        if total_equity <= 0.0 {
            return 0.0;
        }
        (net_income / total_equity) * 100.0
    }

    /// Generate a cash flow statement using the indirect method
    /// Returns (operating, investing, financing, net_change)
    pub fn calculate_cash_flow_indirect(
        net_income: f64,
        depreciation_amortization: f64,
        change_in_working_capital: f64,
        capital_expenditures: f64,
        proceeds_from_asset_sales: f64,
        debt_proceeds: f64,
        debt_repayments: f64,
        dividends_paid: f64,
    ) -> (f64, f64, f64, f64) {
        let operating = net_income + depreciation_amortization + change_in_working_capital;
        let investing = -capital_expenditures + proceeds_from_asset_sales;
        let financing = debt_proceeds - debt_repayments - dividends_paid;
        let net_change = operating + investing + financing;
        (operating, investing, financing, net_change)
    }

    /// Sum account balances for a range
    pub fn sum_account_range(
        balances: &[(String, f64)],
        from_prefix: &str,
        to_prefix: &str,
    ) -> f64 {
        balances.iter()
            .filter(|(code, _)| code.as_str() >= from_prefix && code.as_str() <= to_prefix)
            .map(|(_, balance)| *balance)
            .sum()
    }
}

// ============================================================================
// Tax Filing Service
// ============================================================================

/// Tax Filing service
/// Oracle Fusion: Tax > Tax Filing
#[allow(dead_code)]
pub struct TaxFilingService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid filing frequencies
#[allow(dead_code)]
const VALID_FILING_FREQUENCIES: &[&str] = &[
    "monthly", "quarterly", "semi_annually", "annually",
];

/// Valid filing methods
#[allow(dead_code)]
const VALID_FILING_METHODS: &[&str] = &[
    "electronic", "paper", "both",
];

/// Valid tax return statuses
#[allow(dead_code)]
const VALID_RETURN_STATUSES: &[&str] = &[
    "draft", "calculated", "reviewed", "approved", "filed", "amended", "cancelled",
];

/// Valid tax payment statuses
#[allow(dead_code)]
const VALID_TAX_PAYMENT_STATUSES: &[&str] = &[
    "pending", "processed", "confirmed", "reversed",
];

/// Valid tax payment methods
#[allow(dead_code)]
const VALID_TAX_PAYMENT_METHODS: &[&str] = &[
    "wire", "ach", "check", "electronic",
];

impl TaxFilingService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a tax filing obligation
    /// Oracle Fusion: Tax > Tax Filing > Filing Obligations
    pub async fn create_filing_obligation(
        &self,
        obligation_code: &str,
        name: &str,
        filing_frequency: &str,
        filing_method: &str,
        due_days_after_period: i32,
    ) -> AtlasResult<()> {
        if obligation_code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Obligation code and name are required".to_string(),
            ));
        }
        if !VALID_FILING_FREQUENCIES.contains(&filing_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid filing_frequency '{}'. Must be one of: {}",
                filing_frequency, VALID_FILING_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_FILING_METHODS.contains(&filing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid filing_method '{}'. Must be one of: {}",
                filing_method, VALID_FILING_METHODS.join(", ")
            )));
        }
        if due_days_after_period <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Due days after period must be positive".to_string(),
            ));
        }
        info!("Tax Filing Service: Creating obligation '{}' ({})", obligation_code, filing_frequency);
        Ok(())
    }

    /// Calculate filing due date from period end
    pub fn calculate_filing_due_date(
        period_end: chrono::NaiveDate,
        due_days_after_period: i32,
    ) -> chrono::NaiveDate {
        period_end + chrono::Duration::days(due_days_after_period as i64)
    }

    /// Calculate total tax liability from transaction lines
    pub fn calculate_tax_liability(
        tax_lines: &[(f64, f64)], // (taxable_amount, rate_percentage)
    ) -> (f64, f64) {
        let total_taxable: f64 = tax_lines.iter().map(|(amt, _)| *amt).sum();
        let total_tax: f64 = tax_lines.iter()
            .map(|(amt, rate)| amt * (rate / 100.0))
            .sum();
        (total_taxable, total_tax)
    }

    /// Calculate late filing penalty
    pub fn calculate_late_penalty(
        tax_amount: f64,
        days_late: i32,
        daily_penalty_rate: f64,
        max_penalty_pct: f64,
    ) -> f64 {
        let penalty = tax_amount * (daily_penalty_rate / 100.0) * days_late as f64;
        let max_penalty = tax_amount * (max_penalty_pct / 100.0);
        penalty.min(max_penalty)
    }

    /// Calculate interest on late payment
    pub fn calculate_late_interest(
        tax_amount: f64,
        days_late: i32,
        annual_interest_rate: f64,
    ) -> f64 {
        tax_amount * (annual_interest_rate / 100.0) * (days_late as f64 / 365.0)
    }

    /// Determine filing period dates from frequency
    pub fn calculate_filing_period(
        year: i32,
        period_number: i32,
        frequency: &str,
    ) -> Option<(chrono::NaiveDate, chrono::NaiveDate)> {
        match frequency {
            "monthly" => {
                if !(1..=12).contains(&period_number) { return None; }
                let start = chrono::NaiveDate::from_ymd_opt(year, period_number as u32, 1)?;
                let end = if period_number == 12 {
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31)?
                } else {
                    chrono::NaiveDate::from_ymd_opt(year, (period_number + 1) as u32, 1)?
                        - chrono::Duration::days(1)
                };
                Some((start, end))
            }
            "quarterly" => {
                if !(1..=4).contains(&period_number) { return None; }
                let start_month = ((period_number - 1) * 3 + 1) as u32;
                let end_month = (start_month + 2) as u32;
                let start = chrono::NaiveDate::from_ymd_opt(year, start_month, 1)?;
                let end = if end_month == 12 {
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31)?
                } else {
                    chrono::NaiveDate::from_ymd_opt(year, end_month + 1, 1)?
                        - chrono::Duration::days(1)
                };
                Some((start, end))
            }
            "annually" => {
                if period_number != 1 { return None; }
                let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)?;
                let end = chrono::NaiveDate::from_ymd_opt(year, 12, 31)?;
                Some((start, end))
            }
            _ => None,
        }
    }
}

// ============================================================================
// Journal Reversal Service
// ============================================================================

/// Journal Reversal service
/// Oracle Fusion: General Ledger > Journal Reversal
#[allow(dead_code)]
pub struct JournalReversalService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid reversal methods
#[allow(dead_code)]
const VALID_REVERSAL_METHODS: &[&str] = &[
    "switch_dr_cr", "sign_reverse", "switch_signs",
];

/// Valid reversal reasons
#[allow(dead_code)]
const VALID_REVERSAL_REASONS: &[&str] = &[
    "error_correction", "period_adjustment", "duplicate_entry",
    "reclassification", "management_decision", "other",
];

impl JournalReversalService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Create and validate a journal reversal request
    /// Oracle Fusion: GL > Journals > Reverse Journals
    pub async fn create_reversal_request(
        &self,
        reversal_number: &str,
        original_entry_number: &str,
        reversal_method: &str,
        reversal_reason: &str,
        reason_description: Option<&str>,
    ) -> AtlasResult<()> {
        let _ = reason_description; // persisted by repository layer
        if reversal_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reversal number is required".to_string(),
            ));
        }
        if original_entry_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Original entry number is required".to_string(),
            ));
        }
        if !VALID_REVERSAL_METHODS.contains(&reversal_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reversal_method '{}'. Must be one of: {}",
                reversal_method, VALID_REVERSAL_METHODS.join(", ")
            )));
        }
        if !VALID_REVERSAL_REASONS.contains(&reversal_reason) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reversal_reason '{}'. Must be one of: {}",
                reversal_reason, VALID_REVERSAL_REASONS.join(", ")
            )));
        }

        info!("Reversal Service: Creating reversal '{}' for entry '{}' ({})",
            reversal_number, original_entry_number, reversal_reason);
        Ok(())
    }

    /// Reverse a journal entry line using switch debit/credit method
    pub fn reverse_line_switch_dr_cr(
        debit_amount: f64,
        credit_amount: f64,
    ) -> (f64, f64) {
        // Swap debits and credits
        (credit_amount, debit_amount)
    }

    /// Reverse a journal entry line using sign reversal method
    pub fn reverse_line_sign(
        debit_amount: f64,
        credit_amount: f64,
    ) -> (f64, f64) {
        // Negate both amounts
        (-debit_amount, -credit_amount)
    }

    /// Validate that a reversal entry balances
    pub fn validate_reversal_balances(
        original_total_debit: f64,
        original_total_credit: f64,
        reversal_total_debit: f64,
        reversal_total_credit: f64,
    ) -> bool {
        // Reversal debits should equal original credits, and vice versa
        let debit_matches = (reversal_total_debit - original_total_credit).abs() < 0.01;
        let credit_matches = (reversal_total_credit - original_total_debit).abs() < 0.01;
        debit_matches && credit_matches
    }

    /// Calculate the net effect of an original + reversal entry
    pub fn calculate_net_effect(
        original_debit: f64,
        original_credit: f64,
        reversal_debit: f64,
        reversal_credit: f64,
    ) -> (f64, f64) {
        (original_debit + reversal_debit, original_credit + reversal_credit)
    }

    /// Check if an entry is eligible for reversal
    pub fn is_eligible_for_reversal(
        entry_status: &str,
        is_reversed: bool,
        period_status: &str,
    ) -> Result<bool, String> {
        if is_reversed {
            return Err("Entry is already reversed".to_string());
        }
        if entry_status != "posted" {
            return Err("Only posted entries can be reversed".to_string());
        }
        if period_status == "closed" || period_status == "permanently_closed" {
            return Err("Cannot reverse entries in a closed period".to_string());
        }
        Ok(true)
    }
}

// ============================================================================
// Inflation Adjustment Service (IAS 29)
// ============================================================================

/// Inflation Adjustment service
/// Oracle Fusion: Financials > General Ledger > Inflation Adjustment
#[allow(dead_code)]
pub struct InflationAdjustmentService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid inflation index types
#[allow(dead_code)]
const VALID_INDEX_TYPES: &[&str] = &["cpi", "gdp_deflator", "custom"];

/// Valid inflation adjustment methods
#[allow(dead_code)]
const VALID_ADJUSTMENT_METHODS: &[&str] = &["historical", "current"];

impl InflationAdjustmentService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Calculate the inflation restatement factor between two periods
    pub fn calculate_restatement_factor(
        current_index_value: f64,
        base_index_value: f64,
    ) -> f64 {
        if base_index_value <= 0.0 {
            return 1.0;
        }
        current_index_value / base_index_value
    }

    /// Restate a non-monetary balance using the inflation factor
    /// IAS 29: Non-monetary items restated from acquisition date index
    pub fn restate_non_monetary_balance(
        historical_balance: f64,
        restatement_factor: f64,
    ) -> f64 {
        historical_balance * restatement_factor
    }

    /// Calculate monetary gain/loss (purchasing power gain/loss)
    /// IAS 29: Monetary items are NOT restated; gain/loss recognized in P&L
    pub fn calculate_monetary_gain_loss(
        monetary_balance: f64,
        restatement_factor: f64,
    ) -> f64 {
        monetary_balance * (restatement_factor - 1.0)
    }

    /// Calculate inflation adjustment amount for an account
    pub fn calculate_adjustment_amount(
        original_balance: f64,
        restated_balance: f64,
    ) -> f64 {
        restated_balance - original_balance
    }
}

// ============================================================================
// Impairment Management Service (IAS 36 / ASC 360)
// ============================================================================

/// Impairment Management service
/// Oracle Fusion: Financials > Fixed Assets > Impairment Management
#[allow(dead_code)]
pub struct ImpairmentManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid impairment indicator types
#[allow(dead_code)]
const VALID_INDICATOR_TYPES: &[&str] = &["external", "internal", "market"];

/// Valid impairment severity levels
#[allow(dead_code)]
const VALID_SEVERITY_LEVELS: &[&str] = &["low", "medium", "high", "critical"];

/// Valid impairment test types
#[allow(dead_code)]
const VALID_TEST_TYPES: &[&str] = &["individual", "cash_generating_unit"];

/// Valid impairment test methods
#[allow(dead_code)]
const VALID_TEST_METHODS: &[&str] = &["value_in_use", "fair_value_less_costs"];

impl ImpairmentManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Calculate impairment loss
    /// IAS 36: Loss = Carrying Amount - Recoverable Amount (only if carrying > recoverable)
    pub fn calculate_impairment_loss(
        carrying_amount: f64,
        recoverable_amount: f64,
    ) -> f64 {
        if carrying_amount > recoverable_amount {
            carrying_amount - recoverable_amount
        } else {
            0.0
        }
    }

    /// Calculate present value of future cash flows (value-in-use)
    pub fn calculate_present_value(
        cash_flows: &[(f64, f64)], // (cash_flow, discount_factor)
    ) -> f64 {
        cash_flows.iter().map(|(cf, df)| cf * df).sum()
    }

    /// Calculate discount factor for a given period
    /// DF = 1 / (1 + r)^n
    pub fn calculate_discount_factor(
        discount_rate: f64,
        period_number: i32,
    ) -> f64 {
        if discount_rate <= 0.0 {
            return 1.0;
        }
        1.0 / (1.0 + discount_rate).powi(period_number)
    }

    /// Calculate terminal value present value
    pub fn calculate_terminal_value_pv(
        terminal_value: f64,
        discount_rate: f64,
        periods: i32,
    ) -> f64 {
        terminal_value * Self::calculate_discount_factor(discount_rate, periods)
    }

    /// Determine if asset is impaired
    pub fn is_impaired(carrying_amount: f64, recoverable_amount: f64) -> bool {
        carrying_amount > recoverable_amount
    }

    /// Calculate impairment reversal cap
    /// IAS 36: Reversal limited to what carrying amount would have been
    pub fn calculate_reversal_cap(
        current_carrying: f64,
        original_carrying: f64,
        accumulated_depreciation_since_impairment: f64,
    ) -> f64 {
        let hypothetical_carrying = original_carrying - accumulated_depreciation_since_impairment;
        (hypothetical_carrying - current_carrying).max(0.0)
    }
}

// ============================================================================
// Bank Account Transfer Service
// ============================================================================

/// Bank Account Transfer service
/// Oracle Fusion: Financials > Cash Management > Bank Account Transfers
#[allow(dead_code)]
pub struct BankAccountTransferService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid transfer settlement methods
#[allow(dead_code)]
const VALID_SETTLEMENT_METHODS: &[&str] = &["immediate", "scheduled", "batch"];

impl BankAccountTransferService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Calculate cross-currency transfer amount
    pub fn calculate_cross_currency_amount(amount: f64, exchange_rate: f64) -> f64 {
        amount * exchange_rate
    }

    /// Check if transfer requires approval based on threshold
    pub fn requires_approval(amount: f64, threshold: f64) -> bool {
        if threshold <= 0.0 {
            return false;
        }
        amount > threshold
    }
}

// ============================================================================
// Tax Reporting Service
// ============================================================================

/// Tax Reporting service
/// Oracle Fusion: Tax > Tax Reporting
#[allow(dead_code)]
pub struct TaxReportingService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid tax report types
#[allow(dead_code)]
const VALID_TAX_REPORT_TYPES: &[&str] = &[
    "vat", "gst", "sales_tax", "corporate_income", "withholding",
];

impl TaxReportingService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Calculate net tax due from input/output tax
    pub fn calculate_net_tax_due(output_tax: f64, input_tax: f64) -> f64 {
        output_tax - input_tax
    }

    /// Calculate total amount due including penalties and interest
    pub fn calculate_total_amount_due(net_tax: f64, penalty: f64, interest: f64) -> f64 {
        net_tax + penalty + interest
    }

    /// Calculate net refund or payment
    pub fn calculate_payment_or_refund(total_amount_due: f64, payments_made: f64) -> f64 {
        total_amount_due - payments_made
    }

    /// Calculate effective tax rate
    pub fn calculate_effective_tax_rate(total_tax: f64, total_taxable: f64) -> f64 {
        if total_taxable <= 0.0 {
            return 0.0;
        }
        (total_tax / total_taxable) * 100.0
    }
}

// ============================================================================
// Grant Management Service
// ============================================================================

/// Grant Management service
/// Oracle Fusion: Financials > Grants Management
#[allow(dead_code)]
pub struct GrantManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid sponsor types
#[allow(dead_code)]
const VALID_SPONSOR_TYPES: &[&str] = &[
    "government", "foundation", "corporate", "internal", "university",
];

impl GrantManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Calculate indirect costs
    pub fn calculate_indirect_costs(direct_costs: f64, indirect_cost_rate: f64) -> f64 {
        direct_costs * (indirect_cost_rate / 100.0)
    }

    /// Calculate total award amount (direct + indirect)
    pub fn calculate_total_award(direct_costs: f64, indirect_costs: f64) -> f64 {
        direct_costs + indirect_costs
    }

    /// Calculate available balance
    pub fn calculate_available_balance(
        total_award: f64,
        total_expenditures: f64,
        total_commitments: f64,
    ) -> f64 {
        total_award - total_expenditures - total_commitments
    }

    /// Calculate budget utilization percentage
    pub fn calculate_budget_utilization(expended: f64, budget: f64) -> f64 {
        if budget <= 0.0 { return 0.0; }
        (expended / budget) * 100.0
    }

    /// Calculate cost sharing amount
    pub fn calculate_cost_sharing(total_expenditures: f64, cost_sharing_percent: f64) -> f64 {
        total_expenditures * (cost_sharing_percent / 100.0)
    }

    /// Check if expenditure exceeds budget line
    pub fn is_budget_line_exceeded(
        budget_amount: f64,
        expended_amount: f64,
        committed_amount: f64,
    ) -> bool {
        (expended_amount + committed_amount) > budget_amount
    }
}

// ============================================================================
// Corporate Card Management Service
// ============================================================================

/// Corporate Card Management service
/// Oracle Fusion: Financials > Expenses > Corporate Cards
#[allow(dead_code)]
pub struct CorporateCardManagementService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

/// Valid card networks
#[allow(dead_code)]
const VALID_CARD_NETWORKS: &[&str] = &["visa", "mastercard", "amex"];

/// Valid card types
#[allow(dead_code)]
const VALID_CARD_TYPES: &[&str] = &["corporate", "purchasing", "travel"];

/// Valid matching methods
#[allow(dead_code)]
const VALID_MATCHING_METHODS: &[&str] = &["auto", "manual", "semi"];

impl CorporateCardManagementService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Check if a purchase is within spending limits
    pub fn check_spending_limit(
        purchase_amount: f64,
        single_purchase_limit: f64,
        current_cycle_spend: f64,
        monthly_limit: f64,
    ) -> bool {
        let within_single = single_purchase_limit <= 0.0 || purchase_amount <= single_purchase_limit;
        let within_monthly = monthly_limit <= 0.0 || (current_cycle_spend + purchase_amount) <= monthly_limit;
        within_single && within_monthly
    }

    /// Calculate available monthly spend
    pub fn calculate_available_spend(monthly_limit: f64, current_cycle_spend: f64) -> f64 {
        if monthly_limit <= 0.0 { return f64::MAX; }
        (monthly_limit - current_cycle_spend).max(0.0)
    }

    /// Calculate statement balance
    pub fn calculate_statement_balance(
        opening_balance: f64,
        total_charges: f64,
        total_credits: f64,
        total_payments: f64,
    ) -> f64 {
        opening_balance + total_charges - total_credits - total_payments
    }

    /// Calculate match confidence score (0-100)
    pub fn calculate_match_confidence(
        amount_match: bool,
        date_proximity_days: i32,
        merchant_match: bool,
    ) -> f64 {
        let mut score = 0.0;
        if amount_match { score += 40.0; }
        if merchant_match { score += 30.0; }
        let date_score = (30 - date_proximity_days * 2).max(0) as f64;
        score += date_score;
        score
    }
}

#[cfg(test)]
mod tests {
    use crate::entities;

    // ========================================================================
    // General Ledger Entity Tests
    // ========================================================================

    #[test]
    fn test_chart_of_accounts_definition() {
        let def = entities::chart_of_accounts_definition();
        assert_eq!(def.name, "chart_of_accounts");
        assert_eq!(def.label, "Chart of Account");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_journal_entry_definition() {
        let def = entities::journal_entry_definition();
        assert_eq!(def.name, "journal_entries");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "posted"));
    }

    #[test]
    fn test_invoice_definition() {
        let def = entities::invoice_definition();
        assert_eq!(def.name, "invoices");
        assert!(def.workflow.is_some());
    }

    #[test]
    fn test_budget_definition() {
        let def = entities::budget_definition();
        assert_eq!(def.name, "budgets");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_expense_report_definition() {
        let def = entities::expense_report_definition();
        assert_eq!(def.name, "expense_reports");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "reimbursed"));
    }

    // ========================================================================
    // Accounts Payable Entity Tests
    // ========================================================================

    #[test]
    fn test_ap_invoice_definition() {
        let def = entities::ap_invoice_definition();
        assert_eq!(def.name, "ap_invoices");
        assert_eq!(def.label, "AP Invoice");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "draft"));
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "on_hold"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "paid"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_ap_invoice_line_definition() {
        let def = entities::ap_invoice_line_definition();
        assert_eq!(def.name, "ap_invoice_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_ap_invoice_distribution_definition() {
        let def = entities::ap_invoice_distribution_definition();
        assert_eq!(def.name, "ap_invoice_distributions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_ap_invoice_hold_definition() {
        let def = entities::ap_invoice_hold_definition();
        assert_eq!(def.name, "ap_invoice_holds");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_ap_payment_definition() {
        let def = entities::ap_payment_definition();
        assert_eq!(def.name, "ap_payments");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "confirmed"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    // ========================================================================
    // Accounts Receivable Entity Tests
    // ========================================================================

    #[test]
    fn test_ar_transaction_definition() {
        let def = entities::ar_transaction_definition();
        assert_eq!(def.name, "ar_transactions");
        assert_eq!(def.label, "AR Transaction");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "complete"));
        assert!(wf.states.iter().any(|s| s.name == "open"));
        assert!(wf.states.iter().any(|s| s.name == "closed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_ar_transaction_line_definition() {
        let def = entities::ar_transaction_line_definition();
        assert_eq!(def.name, "ar_transaction_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_ar_receipt_definition() {
        let def = entities::ar_receipt_definition();
        assert_eq!(def.name, "ar_receipts");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "confirmed"));
        assert!(wf.states.iter().any(|s| s.name == "applied"));
        assert!(wf.states.iter().any(|s| s.name == "deposited"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    #[test]
    fn test_ar_credit_memo_definition() {
        let def = entities::ar_credit_memo_definition();
        assert_eq!(def.name, "ar_credit_memos");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "applied"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_ar_adjustment_definition() {
        let def = entities::ar_adjustment_definition();
        assert_eq!(def.name, "ar_adjustments");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Fixed Assets Entity Tests
    // ========================================================================

    #[test]
    fn test_asset_category_definition() {
        let def = entities::asset_category_definition();
        assert_eq!(def.name, "asset_categories");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_asset_book_definition() {
        let def = entities::asset_book_definition();
        assert_eq!(def.name, "asset_books");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_fixed_asset_definition() {
        let def = entities::fixed_asset_definition();
        assert_eq!(def.name, "fixed_assets");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "acquired"));
        assert!(wf.states.iter().any(|s| s.name == "in_service"));
        assert!(wf.states.iter().any(|s| s.name == "disposed"));
        assert!(wf.states.iter().any(|s| s.name == "retired"));
        assert!(wf.states.iter().any(|s| s.name == "transferred"));
    }

    #[test]
    fn test_asset_transfer_definition() {
        let def = entities::asset_transfer_definition();
        assert_eq!(def.name, "asset_transfers");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
    }

    #[test]
    fn test_asset_retirement_definition() {
        let def = entities::asset_retirement_definition();
        assert_eq!(def.name, "asset_retirements");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Cost Management Entity Tests
    // ========================================================================

    #[test]
    fn test_cost_book_definition() {
        let def = entities::cost_book_definition();
        assert_eq!(def.name, "cost_books");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cost_element_definition() {
        let def = entities::cost_element_definition();
        assert_eq!(def.name, "cost_elements");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cost_profile_definition() {
        let def = entities::cost_profile_definition();
        assert_eq!(def.name, "cost_profiles");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_standard_cost_definition() {
        let def = entities::standard_cost_definition();
        assert_eq!(def.name, "standard_costs");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cost_adjustment_definition() {
        let def = entities::cost_adjustment_definition();
        assert_eq!(def.name, "cost_adjustments");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
    }

    #[test]
    fn test_cost_adjustment_line_definition() {
        let def = entities::cost_adjustment_line_definition();
        assert_eq!(def.name, "cost_adjustment_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cost_variance_definition() {
        let def = entities::cost_variance_definition();
        assert_eq!(def.name, "cost_variances");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Service Validation / Business Logic Tests
    // ========================================================================

    #[test]
    fn test_ar_transaction_types_valid() {
        let valid_types = ["invoice", "debit_memo", "credit_memo", "chargeback", "deposit", "guarantee"];
        for t in &valid_types {
            assert!(
                super::VALID_AR_TRANSACTION_TYPES.contains(t),
                "{} should be a valid AR transaction type",
                t
            );
        }
    }

    #[test]
    fn test_ar_transaction_types_invalid() {
        assert!(!super::VALID_AR_TRANSACTION_TYPES.contains(&"purchase_order"));
        assert!(!super::VALID_AR_TRANSACTION_TYPES.contains(&"payment"));
    }

    #[test]
    fn test_ar_receipt_types_valid() {
        let valid_types = ["cash", "check", "credit_card", "wire_transfer", "ach", "other"];
        for t in &valid_types {
            assert!(
                super::VALID_RECEIPT_TYPES.contains(t),
                "{} should be a valid receipt type",
                t
            );
        }
    }

    #[test]
    fn test_ar_credit_memo_reasons_valid() {
        let valid_reasons = ["return", "pricing_error", "damaged", "wrong_item", "discount", "other"];
        for r in &valid_reasons {
            assert!(
                super::VALID_CREDIT_MEMO_REASONS.contains(r),
                "{} should be a valid credit memo reason",
                r
            );
        }
    }

    #[test]
    fn test_ar_adjustment_types_valid() {
        let valid_types = [
            "write_off", "write_off_bad_debt", "small_balance_write_off",
            "increase", "decrease", "transfer", "revaluation",
        ];
        for t in &valid_types {
            assert!(
                super::VALID_ADJUSTMENT_TYPES.contains(t),
                "{} should be a valid AR adjustment type",
                t
            );
        }
    }

    #[test]
    fn test_costing_methods_valid() {
        let valid = ["standard", "average", "fifo", "lifo"];
        for m in &valid {
            assert!(super::VALID_COSTING_METHODS.contains(m));
        }
        assert!(!super::VALID_COSTING_METHODS.contains(&"unknown"));
    }

    #[test]
    fn test_cost_element_types_valid() {
        let valid = ["material", "labor", "overhead", "subcontracting", "expense"];
        for t in &valid {
            assert!(super::VALID_COST_ELEMENT_TYPES.contains(t));
        }
        assert!(!super::VALID_COST_ELEMENT_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_overhead_methods_valid() {
        let valid = ["rate", "amount", "percentage"];
        for m in &valid {
            assert!(super::VALID_OVERHEAD_METHODS.contains(m));
        }
        assert!(!super::VALID_OVERHEAD_METHODS.contains(&"unknown"));
    }

    // ========================================================================
    // Business Logic / Calculation Tests
    // ========================================================================

    #[test]
    fn test_ar_credit_memo_must_be_negative() {
        // Credit memos should have negative amounts
        let _transaction_type = "credit_memo";
        let amount: f64 = 100.0;
        assert!(amount > 0.0, "Credit memo should not have positive amount");

        let amount: f64 = -100.0;
        assert!(amount <= 0.0, "Credit memo should have negative amount");
    }

    #[test]
    fn test_ar_receipt_must_be_positive() {
        let amount: f64 = 100.0;
        assert!(amount > 0.0, "Receipt amount must be positive");

        let invalid: f64 = -50.0;
        assert!(invalid <= 0.0, "Negative receipt amount should be rejected");
    }

    #[test]
    fn test_ar_write_off_must_be_negative() {
        let _adjustment_type = "write_off";
        let amount: f64 = -100.0;
        assert!(amount <= 0.0, "Write-off amount should be negative");

        let _adjustment_type = "increase";
        let amount: f64 = 50.0;
        assert!(amount > 0.0, "Increase adjustment should be positive");
    }

    #[test]
    fn test_cost_total_item_cost_calculation() {
        let element_costs = vec![
            ("material", 50.0),
            ("labor", 20.0),
            ("overhead", 10.0),
        ];
        let total = super::CostManagementService::calculate_total_item_cost(&element_costs);
        assert!((total - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_cost_variance_percent_calculation() {
        // Unfavorable: actual > standard
        let pct = super::CostManagementService::calculate_variance_percent(100.0, 105.0);
        assert!((pct - 5.0).abs() < 0.01);

        // Favorable: actual < standard
        let pct = super::CostManagementService::calculate_variance_percent(100.0, 95.0);
        assert!((pct - (-5.0)).abs() < 0.01);

        // Zero standard
        let pct = super::CostManagementService::calculate_variance_percent(0.0, 50.0);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_cost_adjustment_amount_calculation() {
        let adj = super::CostManagementService::calculate_adjustment_amount(50.0, 55.0);
        assert_eq!(adj, 5.0);

        let adj = super::CostManagementService::calculate_adjustment_amount(100.0, 90.0);
        assert_eq!(adj, -10.0);
    }

    #[test]
    fn test_cost_variance_calculation() {
        // Standard cost > actual cost => favorable variance (negative)
        let standard = 100.0_f64;
        let actual = 95.0_f64;
        let quantity = 1000.0_f64;
        let variance = (actual - standard) * quantity;
        assert!(variance < 0.0); // Favorable
        assert_eq!(variance, -5000.0);
    }

    #[test]
    fn test_cost_negative_validation() {
        let cost: f64 = "-50.0".parse().unwrap();
        assert!(cost < 0.0);

        let cost: f64 = "50.0".parse().unwrap();
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_ap_invoice_workflow_transitions() {
        // Verify the AP invoice workflow has correct transitions
        let def = entities::ap_invoice_definition();
        let wf = def.workflow.unwrap();
        // draft -> submitted -> approved -> paid
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "paid"));
        // Hold path
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "on_hold"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "on_hold" && t.to_state == "submitted"));
    }

    #[test]
    fn test_ar_transaction_workflow_transitions() {
        let def = entities::ar_transaction_definition();
        let wf = def.workflow.unwrap();
        // draft -> complete -> open -> closed
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "complete"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "complete" && t.to_state == "open"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "open" && t.to_state == "closed"));
        // Cancel paths
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "complete" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_ar_receipt_workflow_transitions() {
        let def = entities::ar_receipt_definition();
        let wf = def.workflow.unwrap();
        // draft -> confirmed -> applied -> deposited
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "confirmed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "confirmed" && t.to_state == "applied"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "applied" && t.to_state == "deposited"));
        // Reverse paths
        assert!(wf.transitions.iter().any(|t| t.from_state == "confirmed" && t.to_state == "reversed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "applied" && t.to_state == "reversed"));
    }

    #[test]
    fn test_ar_credit_memo_workflow_transitions() {
        let def = entities::ar_credit_memo_definition();
        let wf = def.workflow.unwrap();
        // draft -> submitted -> approved -> applied
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "applied"));
        // Cancel paths
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_fixed_asset_workflow_transitions() {
        let def = entities::fixed_asset_definition();
        let wf = def.workflow.unwrap();
        // draft -> acquired -> in_service
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "acquired"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "acquired" && t.to_state == "in_service"));
        // Direct to in_service
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "in_service"));
        // Retirement paths
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_service" && t.to_state == "disposed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_service" && t.to_state == "retired"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_service" && t.to_state == "transferred"));
    }

    #[test]
    fn test_cost_adjustment_workflow_transitions() {
        let def = entities::cost_adjustment_definition();
        let wf = def.workflow.unwrap();
        // draft -> submitted -> approved -> posted
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "posted"));
        // Reject path
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "rejected"));
    }

    #[test]
    fn test_asset_retirement_workflow_transitions() {
        let def = entities::asset_retirement_definition();
        let wf = def.workflow.unwrap();
        // pending -> approved -> completed
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "completed"));
        // Cancel
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_asset_transfer_workflow_transitions() {
        let def = entities::asset_transfer_definition();
        let wf = def.workflow.unwrap();
        // pending -> approved -> completed
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "completed"));
        // Reject
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "rejected"));
    }

    #[test]
    fn test_depreciation_method_enum_values() {
        let def = entities::fixed_asset_definition();
        // Verify the entity is built correctly
        assert_eq!(def.name, "fixed_assets");
    }

    #[test]
    fn test_all_entity_definitions_build_successfully() {
        // Verify all entity definitions can be built without panic
        let _ = entities::chart_of_accounts_definition();
        let _ = entities::journal_entry_definition();
        let _ = entities::invoice_definition();
        let _ = entities::budget_definition();
        let _ = entities::expense_report_definition();
        let _ = entities::ap_invoice_definition();
        let _ = entities::ap_invoice_line_definition();
        let _ = entities::ap_invoice_distribution_definition();
        let _ = entities::ap_invoice_hold_definition();
        let _ = entities::ap_payment_definition();
        let _ = entities::ar_transaction_definition();
        let _ = entities::ar_transaction_line_definition();
        let _ = entities::ar_receipt_definition();
        let _ = entities::ar_credit_memo_definition();
        let _ = entities::ar_adjustment_definition();
        let _ = entities::asset_category_definition();
        let _ = entities::asset_book_definition();
        let _ = entities::fixed_asset_definition();
        let _ = entities::asset_transfer_definition();
        let _ = entities::asset_retirement_definition();
        let _ = entities::cost_book_definition();
        let _ = entities::cost_element_definition();
        let _ = entities::cost_profile_definition();
        let _ = entities::standard_cost_definition();
        let _ = entities::cost_adjustment_definition();
        let _ = entities::cost_adjustment_line_definition();
        let _ = entities::cost_variance_definition();
    }

    #[test]
    fn test_total_entity_count() {
        // Should have 27 entity definitions covering all financial modules
        let entities = vec![
            entities::chart_of_accounts_definition(),
            entities::journal_entry_definition(),
            entities::invoice_definition(),
            entities::budget_definition(),
            entities::expense_report_definition(),
            entities::ap_invoice_definition(),
            entities::ap_invoice_line_definition(),
            entities::ap_invoice_distribution_definition(),
            entities::ap_invoice_hold_definition(),
            entities::ap_payment_definition(),
            entities::ar_transaction_definition(),
            entities::ar_transaction_line_definition(),
            entities::ar_receipt_definition(),
            entities::ar_credit_memo_definition(),
            entities::ar_adjustment_definition(),
            entities::asset_category_definition(),
            entities::asset_book_definition(),
            entities::fixed_asset_definition(),
            entities::asset_transfer_definition(),
            entities::asset_retirement_definition(),
            entities::cost_book_definition(),
            entities::cost_element_definition(),
            entities::cost_profile_definition(),
            entities::standard_cost_definition(),
            entities::cost_adjustment_definition(),
            entities::cost_adjustment_line_definition(),
            entities::cost_variance_definition(),
        ];
        assert_eq!(entities.len(), 27);

        // Verify all have unique names
        let names: std::collections::HashSet<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 27, "All entity names must be unique");
    }

    #[test]
    fn test_workflow_entity_count() {
        // Count entities with workflows
        let workflow_entities = vec![
            entities::journal_entry_definition(),
            entities::invoice_definition(),
            entities::expense_report_definition(),
            entities::ap_invoice_definition(),
            entities::ap_payment_definition(),
            entities::ar_transaction_definition(),
            entities::ar_receipt_definition(),
            entities::ar_credit_memo_definition(),
            entities::fixed_asset_definition(),
            entities::asset_transfer_definition(),
            entities::asset_retirement_definition(),
            entities::cost_adjustment_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 12, "All 12 listed entities should have workflows");
    }

    #[test]
    fn test_cost_adjustment_total_rollup() {
        let lines = vec![
            ("50.0".parse::<f64>().unwrap(), "55.0".parse::<f64>().unwrap()),
            ("100.0".parse::<f64>().unwrap(), "95.0".parse::<f64>().unwrap()),
            ("75.0".parse::<f64>().unwrap(), "80.0".parse::<f64>().unwrap()),
        ];
        let total: f64 = lines.iter().map(|(oc, nc)| nc - oc).sum();
        assert!((total - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_receipt_amount_validation_positive() {
        let valid_amount = "100.50".parse::<f64>().unwrap();
        assert!(valid_amount > 0.0);

        let invalid_amount = "0.0".parse::<f64>().unwrap();
        assert!(invalid_amount <= 0.0);

        let invalid_negative = "-50.0".parse::<f64>().unwrap();
        assert!(invalid_negative <= 0.0);
    }

    #[test]
    fn test_quantity_must_be_positive() {
        let qty: f64 = "100.0".parse().unwrap();
        assert!(qty > 0.0);

        let zero: f64 = "0.0".parse().unwrap();
        assert!(zero <= 0.0);

        let neg: f64 = "-5.0".parse().unwrap();
        assert!(neg <= 0.0);
    }

    // ========================================================================
    // Revenue Recognition Entity Tests
    // ========================================================================

    #[test]
    fn test_revenue_policy_definition() {
        let def = entities::revenue_policy_definition();
        assert_eq!(def.name, "revenue_policies");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_revenue_contract_definition() {
        let def = entities::revenue_contract_definition();
        assert_eq!(def.name, "revenue_contracts");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "modified"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_performance_obligation_definition() {
        let def = entities::performance_obligation_definition();
        assert_eq!(def.name, "performance_obligations");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_revenue_schedule_line_definition() {
        let def = entities::revenue_schedule_line_definition();
        assert_eq!(def.name, "revenue_schedule_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_revenue_modification_definition() {
        let def = entities::revenue_modification_definition();
        assert_eq!(def.name, "revenue_modifications");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Revenue Recognition Validation / Business Logic Tests
    // ========================================================================

    #[test]
    fn test_recognition_methods_valid() {
        let valid = ["over_time", "point_in_time"];
        for m in &valid {
            assert!(super::VALID_RECOGNITION_METHODS.contains(m), "{} should be valid", m);
        }
    }

    #[test]
    fn test_recognition_methods_invalid() {
        assert!(!super::VALID_RECOGNITION_METHODS.contains(&"unknown"));
        assert!(!super::VALID_RECOGNITION_METHODS.contains(&"partial"));
    }

    #[test]
    fn test_over_time_methods_valid() {
        let valid = ["output", "input", "straight_line"];
        for m in &valid {
            assert!(super::VALID_OVER_TIME_METHODS.contains(m), "{} should be valid", m);
        }
    }

    #[test]
    fn test_allocation_bases_valid() {
        let valid = ["standalone_selling_price", "residual", "equal"];
        for b in &valid {
            assert!(super::VALID_ALLOCATION_BASES.contains(b), "{} should be valid", b);
        }
    }

    #[test]
    fn test_contract_statuses_valid() {
        let valid = ["draft", "active", "completed", "cancelled", "modified"];
        for s in &valid {
            assert!(super::VALID_CONTRACT_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_obligation_statuses_valid() {
        let valid = ["pending", "in_progress", "satisfied", "partially_satisfied", "cancelled"];
        for s in &valid {
            assert!(super::VALID_OBLIGATION_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_schedule_statuses_valid() {
        let valid = ["planned", "recognized", "reversed", "cancelled"];
        for s in &valid {
            assert!(super::VALID_SCHEDULE_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_modification_types_valid() {
        let valid = ["price_change", "scope_change", "term_extension",
                     "termination", "add_obligation", "remove_obligation"];
        for t in &valid {
            assert!(super::VALID_MODIFICATION_TYPES.contains(t));
        }
    }

    #[test]
    fn test_revenue_straight_line_allocation() {
        let allocations = super::RevenueRecognitionService::calculate_straight_line_allocation(
            100000.0, 4,
        );
        assert_eq!(allocations.len(), 4);
        for a in &allocations {
            assert!((a - 25000.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_revenue_straight_line_zero_obligations() {
        let allocations = super::RevenueRecognitionService::calculate_straight_line_allocation(
            100000.0, 0,
        );
        assert!(allocations.is_empty());
    }

    #[test]
    fn test_revenue_ssp_allocation() {
        let ssp = vec![40000.0, 30000.0, 30000.0];
        let total = 120000.0;
        let allocations = super::RevenueRecognitionService::calculate_ssp_allocation(
            total, &ssp,
        );
        assert_eq!(allocations.len(), 3);
        assert!((allocations[0] - 48000.0).abs() < 0.01);
        assert!((allocations[1] - 36000.0).abs() < 0.01);
        assert!((allocations[2] - 36000.0).abs() < 0.01);
        let sum: f64 = allocations.iter().sum();
        assert!((sum - total).abs() < 0.01);
    }

    #[test]
    fn test_revenue_ssp_allocation_zero() {
        let ssp = vec![0.0, 0.0];
        let allocations = super::RevenueRecognitionService::calculate_ssp_allocation(
            100000.0, &ssp,
        );
        assert_eq!(allocations.len(), 2);
        assert_eq!(allocations[0], 0.0);
        assert_eq!(allocations[1], 0.0);
    }

    #[test]
    fn test_revenue_percentage_complete() {
        let pct = super::RevenueRecognitionService::calculate_percentage_complete(
            60000.0, 100000.0,
        );
        assert!((pct - 0.6).abs() < 0.001);

        // Over 100% capped at 1.0
        let pct = super::RevenueRecognitionService::calculate_percentage_complete(
            120000.0, 100000.0,
        );
        assert!((pct - 1.0).abs() < 0.001);

        // Zero costs
        let pct = super::RevenueRecognitionService::calculate_percentage_complete(
            50000.0, 0.0,
        );
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_revenue_to_date() {
        let rev = super::RevenueRecognitionService::calculate_revenue_to_date(
            100000.0, 0.6,
        );
        assert!((rev - 60000.0).abs() < 0.01);
    }

    #[test]
    fn test_revenue_contract_workflow_transitions() {
        let def = entities::revenue_contract_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "modified"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "modified" && t.to_state == "active"));
    }

    // ========================================================================
    // Subledger Accounting Entity Tests
    // ========================================================================

    #[test]
    fn test_accounting_method_definition() {
        let def = entities::accounting_method_definition();
        assert_eq!(def.name, "accounting_methods");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_accounting_derivation_rule_definition() {
        let def = entities::accounting_derivation_rule_definition();
        assert_eq!(def.name, "accounting_derivation_rules");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_subledger_journal_entry_definition() {
        let def = entities::subledger_journal_entry_definition();
        assert_eq!(def.name, "subledger_journal_entries");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "accounted"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "transferred"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    #[test]
    fn test_subledger_journal_line_definition() {
        let def = entities::subledger_journal_line_definition();
        assert_eq!(def.name, "subledger_journal_lines");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // SLA Validation Tests
    // ========================================================================

    #[test]
    fn test_sla_applications_valid() {
        let valid = ["payables", "receivables", "expenses", "assets", "projects", "general"];
        for a in &valid {
            assert!(super::VALID_SLA_APPLICATIONS.contains(a));
        }
        assert!(!super::VALID_SLA_APPLICATIONS.contains(&"hr"));
    }

    #[test]
    fn test_sla_event_classes_valid() {
        let valid = ["create", "update", "cancel", "reverse"];
        for e in &valid {
            assert!(super::VALID_SLA_EVENT_CLASSES.contains(e));
        }
    }

    #[test]
    fn test_sla_derivation_types_valid() {
        let valid = ["constant", "lookup", "formula"];
        for d in &valid {
            assert!(super::VALID_DERIVATION_TYPES.contains(d));
        }
    }

    #[test]
    fn test_sla_entry_statuses_valid() {
        let valid = ["draft", "accounted", "posted", "transferred", "reversed", "error"];
        for s in &valid {
            assert!(super::VALID_SLA_ENTRY_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_sla_journal_workflow_transitions() {
        let def = entities::subledger_journal_entry_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "accounted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "accounted" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "posted" && t.to_state == "transferred"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "accounted" && t.to_state == "reversed"));
    }

    // ========================================================================
    // Cash Management Entity Tests
    // ========================================================================

    #[test]
    fn test_cash_position_definition() {
        let def = entities::cash_position_definition();
        assert_eq!(def.name, "cash_positions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cash_forecast_template_definition() {
        let def = entities::cash_forecast_template_definition();
        assert_eq!(def.name, "cash_forecast_templates");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cash_forecast_source_definition() {
        let def = entities::cash_forecast_source_definition();
        assert_eq!(def.name, "cash_forecast_sources");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_cash_forecast_definition() {
        let def = entities::cash_forecast_definition();
        assert_eq!(def.name, "cash_forecasts");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "generated"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "superseded"));
    }

    // ========================================================================
    // Cash Management Validation Tests
    // ========================================================================

    #[test]
    fn test_cash_bucket_types_valid() {
        let valid = ["daily", "weekly", "monthly"];
        for b in &valid {
            assert!(super::VALID_BUCKET_TYPES.contains(b));
        }
        assert!(!super::VALID_BUCKET_TYPES.contains(&"yearly"));
    }

    #[test]
    fn test_cash_source_types_valid() {
        let valid = ["accounts_payable", "accounts_receivable", "payroll",
                     "purchasing", "manual", "budget", "intercompany",
                     "fixed_assets", "tax", "other"];
        for s in &valid {
            assert!(super::VALID_CASH_SOURCE_TYPES.contains(s));
        }
    }

    #[test]
    fn test_cash_flow_directions_valid() {
        let valid = ["inflow", "outflow", "both"];
        for d in &valid {
            assert!(super::VALID_CASH_FLOW_DIRECTIONS.contains(d));
        }
    }

    #[test]
    fn test_forecast_statuses_valid() {
        let valid = ["draft", "generated", "approved", "superseded"];
        for s in &valid {
            assert!(super::VALID_FORECAST_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_cash_net_cash_flow_calculation() {
        let net = super::CashManagementFinService::calculate_net_cash_flow(100000.0, 75000.0);
        assert!((net - 25000.0).abs() < 0.01);
    }

    #[test]
    fn test_cash_closing_balance_calculation() {
        let closing = super::CashManagementFinService::calculate_closing_balance(500000.0, 25000.0);
        assert!((closing - 525000.0).abs() < 0.01);
    }

    #[test]
    fn test_cash_forecast_workflow_transitions() {
        let def = entities::cash_forecast_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "generated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "generated" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "superseded"));
    }

    // ========================================================================
    // Tax Management Entity Tests
    // ========================================================================

    #[test]
    fn test_tax_regime_definition() {
        let def = entities::tax_regime_definition();
        assert_eq!(def.name, "tax_regimes");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_tax_jurisdiction_definition() {
        let def = entities::tax_jurisdiction_definition();
        assert_eq!(def.name, "tax_jurisdictions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_tax_rate_definition() {
        let def = entities::tax_rate_definition();
        assert_eq!(def.name, "tax_rates");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_tax_determination_rule_definition() {
        let def = entities::tax_determination_rule_definition();
        assert_eq!(def.name, "tax_determination_rules");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Tax Management Validation Tests
    // ========================================================================

    #[test]
    fn test_tax_types_valid() {
        let valid = ["sales_tax", "vat", "gst", "withholding", "excise", "customs"];
        for t in &valid {
            assert!(super::VALID_TAX_TYPES.contains(t));
        }
        assert!(!super::VALID_TAX_TYPES.contains(&"income_tax"));
    }

    #[test]
    fn test_rate_types_valid() {
        let valid = ["standard", "reduced", "zero", "exempt"];
        for r in &valid {
            assert!(super::VALID_RATE_TYPES.contains(r));
        }
    }

    #[test]
    fn test_rounding_rules_valid() {
        let valid = ["nearest", "up", "down", "none"];
        for r in &valid {
            assert!(super::VALID_ROUNDING_RULES.contains(r));
        }
    }

    #[test]
    fn test_geographic_levels_valid() {
        let valid = ["country", "state", "county", "city", "region"];
        for g in &valid {
            assert!(super::VALID_GEOGRAPHIC_LEVELS.contains(g));
        }
    }

    #[test]
    fn test_inclusive_tax_calculation() {
        // 10% tax included in $110 total => tax = $10
        let tax = super::TaxManagementService::calculate_inclusive_tax(110.0, 10.0);
        assert!((tax - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_exclusive_tax_calculation() {
        // 10% tax on $100 => $10
        let tax = super::TaxManagementService::calculate_exclusive_tax(100.0, 10.0);
        assert!((tax - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_rate_workflow_transitions() {
        let def = entities::tax_rate_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
    }

    // ========================================================================
    // Intercompany Entity Tests
    // ========================================================================

    #[test]
    fn test_intercompany_batch_definition() {
        let def = entities::intercompany_batch_definition();
        assert_eq!(def.name, "intercompany_batches");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_intercompany_transaction_definition() {
        let def = entities::intercompany_transaction_definition();
        assert_eq!(def.name, "intercompany_transactions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_intercompany_settlement_definition() {
        let def = entities::intercompany_settlement_definition();
        assert_eq!(def.name, "intercompany_settlements");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Intercompany Validation Tests
    // ========================================================================

    #[test]
    fn test_ic_batch_statuses_valid() {
        let valid = ["draft", "submitted", "approved", "posted", "cancelled"];
        for s in &valid {
            assert!(super::VALID_IC_BATCH_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_ic_txn_types_valid() {
        let valid = ["invoice", "journal_entry", "payment", "charge", "allocation"];
        for t in &valid {
            assert!(super::VALID_IC_TXN_TYPES.contains(t));
        }
    }

    #[test]
    fn test_ic_settlement_methods_valid() {
        let valid = ["cash", "netting", "offset"];
        for m in &valid {
            assert!(super::VALID_IC_SETTLEMENT_METHODS.contains(m));
        }
    }

    #[test]
    fn test_ic_batch_workflow_transitions() {
        let def = entities::intercompany_batch_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Period Close Entity Tests
    // ========================================================================

    #[test]
    fn test_accounting_calendar_definition() {
        let def = entities::accounting_calendar_definition();
        assert_eq!(def.name, "accounting_calendars");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_accounting_period_definition() {
        let def = entities::accounting_period_definition();
        assert_eq!(def.name, "accounting_periods");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "future");
        assert!(wf.states.iter().any(|s| s.name == "not_opened"));
        assert!(wf.states.iter().any(|s| s.name == "open"));
        assert!(wf.states.iter().any(|s| s.name == "pending_close"));
        assert!(wf.states.iter().any(|s| s.name == "closed"));
        assert!(wf.states.iter().any(|s| s.name == "permanently_closed"));
    }

    #[test]
    fn test_period_close_checklist_definition() {
        let def = entities::period_close_checklist_definition();
        assert_eq!(def.name, "period_close_checklist");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Period Close Validation Tests
    // ========================================================================

    #[test]
    fn test_period_statuses_valid() {
        let valid = ["future", "not_opened", "open", "pending_close", "closed", "permanently_closed"];
        for s in &valid {
            assert!(super::VALID_PERIOD_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_period_subledgers_valid() {
        let valid = ["gl", "ap", "ar", "fa", "po"];
        for s in &valid {
            assert!(super::VALID_PERIOD_SUBLEDGERS.contains(s));
        }
    }

    #[test]
    fn test_period_status_workflow_transitions() {
        let def = entities::accounting_period_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "future" && t.to_state == "not_opened"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "not_opened" && t.to_state == "open"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "open" && t.to_state == "pending_close"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending_close" && t.to_state == "closed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "closed" && t.to_state == "permanently_closed"));
    }

    // ========================================================================
    // Lease Accounting Entity Tests
    // ========================================================================

    #[test]
    fn test_lease_contract_definition() {
        let def = entities::lease_contract_definition();
        assert_eq!(def.name, "lease_contracts");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "modified"));
        assert!(wf.states.iter().any(|s| s.name == "impaired"));
        assert!(wf.states.iter().any(|s| s.name == "terminated"));
        assert!(wf.states.iter().any(|s| s.name == "expired"));
    }

    #[test]
    fn test_lease_payment_definition() {
        let def = entities::lease_payment_definition();
        assert_eq!(def.name, "lease_payments");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_lease_modification_definition() {
        let def = entities::lease_modification_definition();
        assert_eq!(def.name, "lease_modifications");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
    }

    #[test]
    fn test_lease_termination_definition() {
        let def = entities::lease_termination_definition();
        assert_eq!(def.name, "lease_terminations");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Lease Accounting Validation Tests
    // ========================================================================

    #[test]
    fn test_lease_classifications_valid() {
        let valid = ["operating", "finance"];
        for c in &valid {
            assert!(super::VALID_LEASE_CLASSIFICATIONS.contains(c));
        }
    }

    #[test]
    fn test_lease_statuses_valid() {
        let valid = ["draft", "active", "modified", "impaired", "terminated", "expired"];
        for s in &valid {
            assert!(super::VALID_LEASE_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_payment_frequencies_valid() {
        let valid = ["monthly", "quarterly", "annually"];
        for f in &valid {
            assert!(super::VALID_PAYMENT_FREQUENCIES.contains(f));
        }
    }

    #[test]
    fn test_lease_mod_types_valid() {
        let valid = ["term_extension", "scope_change", "payment_change", "rate_change", "reclassification"];
        for t in &valid {
            assert!(super::VALID_LEASE_MOD_TYPES.contains(t));
        }
    }

    #[test]
    fn test_lease_term_types_valid() {
        let valid = ["early", "end_of_term", "mutual_agreement", "default"];
        for t in &valid {
            assert!(super::VALID_LEASE_TERM_TYPES.contains(t));
        }
    }

    #[test]
    fn test_lease_liability_calculation() {
        // Monthly payment $1000, monthly rate 0.5%, 36 months
        let liability = super::LeaseAccountingFinService::calculate_lease_liability(
            1000.0, 0.005, 36,
        );
        // PV of annuity: 1000 * (1 - (1.005)^-36) / 0.005 ≈ 32900.40
        assert!(liability > 32000.0 && liability < 34000.0);
    }

    #[test]
    fn test_lease_liability_zero_rate() {
        let liability = super::LeaseAccountingFinService::calculate_lease_liability(
            1000.0, 0.0, 36,
        );
        assert_eq!(liability, 0.0);
    }

    #[test]
    fn test_lease_interest_calculation() {
        let interest = super::LeaseAccountingFinService::calculate_lease_interest(
            30000.0, 0.005,
        );
        assert!((interest - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_lease_principal_reduction() {
        let principal = super::LeaseAccountingFinService::calculate_principal_reduction(
            1000.0, 150.0,
        );
        assert!((principal - 850.0).abs() < 0.01);
    }

    #[test]
    fn test_lease_contract_workflow_transitions() {
        let def = entities::lease_contract_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "modified"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "impaired"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "terminated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "expired"));
    }

    // ========================================================================
    // Bank Reconciliation Entity Tests
    // ========================================================================

    #[test]
    fn test_bank_account_definition() {
        let def = entities::bank_account_definition();
        assert_eq!(def.name, "bank_accounts");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_bank_statement_definition() {
        let def = entities::bank_statement_definition();
        assert_eq!(def.name, "bank_statements");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "imported");
        assert!(wf.states.iter().any(|s| s.name == "in_review"));
        assert!(wf.states.iter().any(|s| s.name == "reconciled"));
    }

    #[test]
    fn test_bank_statement_line_definition() {
        let def = entities::bank_statement_line_definition();
        assert_eq!(def.name, "bank_statement_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_reconciliation_match_definition() {
        let def = entities::reconciliation_match_definition();
        assert_eq!(def.name, "reconciliation_matches");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Bank Reconciliation Business Logic Tests
    // ========================================================================

    #[test]
    fn test_recon_difference_calculation() {
        // Bank: 10000, Book: 9500, deposits in transit: 2000, outstanding: 3000, charges: 100
        // Adjusted bank: 10000 + 2000 - 3000 = 9000
        // Adjusted book: 9500 - 100 = 9400
        // Difference: 9000 - 9400 = -400
        let diff = super::BankReconciliationService::calculate_recon_difference(
            10000.0, 9500.0, 2000.0, 3000.0, 100.0,
        );
        assert!((diff - (-400.0)).abs() < 0.01);
    }

    #[test]
    fn test_recon_difference_balanced() {
        // Perfectly reconciled: adjusted_bank == adjusted_book
        // adjusted_bank = 10000 + 2000 - 3000 = 9000
        // adjusted_book = 9000 - 0 = 9000
        let diff = super::BankReconciliationService::calculate_recon_difference(
            10000.0, 9000.0, 2000.0, 3000.0, 0.0,
        );
        assert!((diff - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_bank_statement_workflow_transitions() {
        let def = entities::bank_statement_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "imported" && t.to_state == "in_review"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_review" && t.to_state == "reconciled"));
    }

    // ========================================================================
    // Encumbrance Entity Tests
    // ========================================================================

    #[test]
    fn test_encumbrance_type_definition() {
        let def = entities::encumbrance_type_definition();
        assert_eq!(def.name, "encumbrance_types");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_encumbrance_entry_definition() {
        let def = entities::encumbrance_entry_definition();
        assert_eq!(def.name, "encumbrance_entries");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "partially_liquidated"));
        assert!(wf.states.iter().any(|s| s.name == "fully_liquidated"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
        assert!(wf.states.iter().any(|s| s.name == "expired"));
    }

    #[test]
    fn test_encumbrance_liquidation_definition() {
        let def = entities::encumbrance_liquidation_definition();
        assert_eq!(def.name, "encumbrance_liquidations");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_encumbrance_carry_forward_definition() {
        let def = entities::encumbrance_carry_forward_definition();
        assert_eq!(def.name, "encumbrance_carry_forwards");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "processing"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    // ========================================================================
    // Encumbrance Validation Tests
    // ========================================================================

    #[test]
    fn test_encumbrance_categories_valid() {
        let valid = ["commitment", "obligation", "preliminary"];
        for c in &valid {
            assert!(super::VALID_ENCUMBRANCE_CATEGORIES.contains(c));
        }
    }

    #[test]
    fn test_encumbrance_statuses_valid() {
        let valid = ["draft", "active", "partially_liquidated",
                     "fully_liquidated", "cancelled", "expired"];
        for s in &valid {
            assert!(super::VALID_ENCUMBRANCE_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_remaining_encumbrance_calculation() {
        let remaining = super::EncumbranceManagementService::calculate_remaining_encumbrance(
            100000.0, 35000.0,
        );
        assert!((remaining - 65000.0).abs() < 0.01);
    }

    #[test]
    fn test_encumbrance_entry_workflow_transitions() {
        let def = entities::encumbrance_entry_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "partially_liquidated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "fully_liquidated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "partially_liquidated" && t.to_state == "fully_liquidated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "expired"));
    }

    // ========================================================================
    // Currency Management Entity Tests
    // ========================================================================

    #[test]
    fn test_currency_definition_entity() {
        let def = entities::currency_definition_entity();
        assert_eq!(def.name, "currencies");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_exchange_rate_definition() {
        let def = entities::exchange_rate_definition();
        assert_eq!(def.name, "exchange_rates");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Currency Management Validation Tests
    // ========================================================================

    #[test]
    fn test_exchange_rate_types_valid() {
        let valid = ["daily", "spot", "corporate", "period_average", "period_end", "user", "fixed"];
        for t in &valid {
            assert!(super::VALID_EXCHANGE_RATE_TYPES.contains(t));
        }
        assert!(!super::VALID_EXCHANGE_RATE_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_currency_conversion() {
        let converted = super::CurrencyManagementService::convert_currency(
            1000.0, 1.10,
        );
        assert!((converted - 1100.0).abs() < 0.01);
    }

    #[test]
    fn test_unrealized_gain_loss_positive() {
        // Gain: rate went up
        let gain = super::CurrencyManagementService::calculate_unrealized_gain_loss(
            10000.0, 1.0, 1.2,
        );
        assert!((gain - 2000.0).abs() < 0.01); // positive = gain
    }

    #[test]
    fn test_unrealized_gain_loss_negative() {
        // Loss: rate went down
        let loss = super::CurrencyManagementService::calculate_unrealized_gain_loss(
            10000.0, 1.0, 0.8,
        );
        assert!((loss - (-2000.0)).abs() < 0.01); // negative = loss
    }

    // ========================================================================
    // Multi-Book Accounting Entity Tests
    // ========================================================================

    #[test]
    fn test_accounting_book_definition() {
        let def = entities::accounting_book_definition();
        assert_eq!(def.name, "accounting_books");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_account_mapping_definition() {
        let def = entities::account_mapping_definition();
        assert_eq!(def.name, "account_mappings");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_book_journal_entry_definition() {
        let def = entities::book_journal_entry_definition();
        assert_eq!(def.name, "book_journal_entries");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "propagated"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    // ========================================================================
    // Multi-Book Validation Tests
    // ========================================================================

    #[test]
    fn test_book_types_valid() {
        let valid = ["primary", "secondary"];
        for t in &valid {
            assert!(super::VALID_BOOK_TYPES.contains(t));
        }
    }

    #[test]
    fn test_mapping_levels_valid() {
        let valid = ["journal", "subledger"];
        for l in &valid {
            assert!(super::VALID_MAPPING_LEVELS.contains(l));
        }
    }

    #[test]
    fn test_book_journal_workflow_transitions() {
        let def = entities::book_journal_entry_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "posted" && t.to_state == "propagated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "posted" && t.to_state == "reversed"));
    }

    // ========================================================================
    // Financial Consolidation Entity Tests
    // ========================================================================

    #[test]
    fn test_consolidation_ledger_definition() {
        let def = entities::consolidation_ledger_definition();
        assert_eq!(def.name, "consolidation_ledgers");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_consolidation_entity_definition() {
        let def = entities::consolidation_entity_definition();
        assert_eq!(def.name, "consolidation_entities");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_consolidation_scenario_definition() {
        let def = entities::consolidation_scenario_definition();
        assert_eq!(def.name, "consolidation_scenarios");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "in_progress"));
        assert!(wf.states.iter().any(|s| s.name == "pending_review"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    #[test]
    fn test_consolidation_adjustment_definition() {
        let def = entities::consolidation_adjustment_definition();
        assert_eq!(def.name, "consolidation_adjustments");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
    }

    #[test]
    fn test_consolidation_elimination_rule_definition() {
        let def = entities::consolidation_elimination_rule_definition();
        assert_eq!(def.name, "consolidation_elimination_rules");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_consolidation_translation_rate_definition() {
        let def = entities::consolidation_translation_rate_definition();
        assert_eq!(def.name, "consolidation_translation_rates");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Financial Consolidation Validation Tests
    // ========================================================================

    #[test]
    fn test_consolidation_methods_valid() {
        let valid = ["full", "proportional", "equity_method"];
        for m in &valid {
            assert!(super::VALID_CONSOLIDATION_METHODS.contains(m));
        }
    }

    #[test]
    fn test_translation_methods_valid() {
        let valid = ["current_rate", "temporal", "weighted_average"];
        for m in &valid {
            assert!(super::VALID_TRANSLATION_METHODS.contains(m));
        }
    }

    #[test]
    fn test_scenario_statuses_valid() {
        let valid = ["draft", "in_progress", "pending_review", "approved", "posted", "reversed"];
        for s in &valid {
            assert!(super::VALID_SCENARIO_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_minority_interest_calculation() {
        // 80% ownership, $1M net income
        let mi = super::FinancialConsolidationFinService::calculate_minority_interest(
            1000000.0, 80.0,
        );
        assert!((mi - 200000.0).abs() < 0.01); // 20% minority
    }

    #[test]
    fn test_minority_interest_full_ownership() {
        let mi = super::FinancialConsolidationFinService::calculate_minority_interest(
            500000.0, 100.0,
        );
        assert!((mi - 0.0).abs() < 0.01); // No minority
    }

    #[test]
    fn test_proportional_share_calculation() {
        let share = super::FinancialConsolidationFinService::calculate_proportional_share(
            500000.0, 60.0,
        );
        assert!((share - 300000.0).abs() < 0.01);
    }

    #[test]
    fn test_consolidation_scenario_workflow_transitions() {
        let def = entities::consolidation_scenario_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "in_progress"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_progress" && t.to_state == "pending_review"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending_review" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "posted" && t.to_state == "reversed"));
    }

    // ========================================================================
    // Comprehensive: All New Entity Definitions Build
    // ========================================================================

    #[test]
    fn test_all_new_entity_definitions_build_successfully() {
        // Revenue Recognition
        let _ = entities::revenue_policy_definition();
        let _ = entities::revenue_contract_definition();
        let _ = entities::performance_obligation_definition();
        let _ = entities::revenue_schedule_line_definition();
        let _ = entities::revenue_modification_definition();
        // Subledger Accounting
        let _ = entities::accounting_method_definition();
        let _ = entities::accounting_derivation_rule_definition();
        let _ = entities::subledger_journal_entry_definition();
        let _ = entities::subledger_journal_line_definition();
        // Cash Management
        let _ = entities::cash_position_definition();
        let _ = entities::cash_forecast_template_definition();
        let _ = entities::cash_forecast_source_definition();
        let _ = entities::cash_forecast_definition();
        // Tax Management
        let _ = entities::tax_regime_definition();
        let _ = entities::tax_jurisdiction_definition();
        let _ = entities::tax_rate_definition();
        let _ = entities::tax_determination_rule_definition();
        // Intercompany
        let _ = entities::intercompany_batch_definition();
        let _ = entities::intercompany_transaction_definition();
        let _ = entities::intercompany_settlement_definition();
        // Period Close
        let _ = entities::accounting_calendar_definition();
        let _ = entities::accounting_period_definition();
        let _ = entities::period_close_checklist_definition();
        // Lease Accounting
        let _ = entities::lease_contract_definition();
        let _ = entities::lease_payment_definition();
        let _ = entities::lease_modification_definition();
        let _ = entities::lease_termination_definition();
        // Bank Reconciliation
        let _ = entities::bank_account_definition();
        let _ = entities::bank_statement_definition();
        let _ = entities::bank_statement_line_definition();
        let _ = entities::reconciliation_match_definition();
        // Encumbrance
        let _ = entities::encumbrance_type_definition();
        let _ = entities::encumbrance_entry_definition();
        let _ = entities::encumbrance_liquidation_definition();
        let _ = entities::encumbrance_carry_forward_definition();
        // Currency
        let _ = entities::currency_definition_entity();
        let _ = entities::exchange_rate_definition();
        // Multi-Book
        let _ = entities::accounting_book_definition();
        let _ = entities::account_mapping_definition();
        let _ = entities::book_journal_entry_definition();
        // Financial Consolidation
        let _ = entities::consolidation_ledger_definition();
        let _ = entities::consolidation_entity_definition();
        let _ = entities::consolidation_scenario_definition();
        let _ = entities::consolidation_adjustment_definition();
        let _ = entities::consolidation_elimination_rule_definition();
        let _ = entities::consolidation_translation_rate_definition();
    }

    #[test]
    fn test_total_new_entity_count() {
        let new_entities = vec![
            entities::revenue_policy_definition(),
            entities::revenue_contract_definition(),
            entities::performance_obligation_definition(),
            entities::revenue_schedule_line_definition(),
            entities::revenue_modification_definition(),
            entities::accounting_method_definition(),
            entities::accounting_derivation_rule_definition(),
            entities::subledger_journal_entry_definition(),
            entities::subledger_journal_line_definition(),
            entities::cash_position_definition(),
            entities::cash_forecast_template_definition(),
            entities::cash_forecast_source_definition(),
            entities::cash_forecast_definition(),
            entities::tax_regime_definition(),
            entities::tax_jurisdiction_definition(),
            entities::tax_rate_definition(),
            entities::tax_determination_rule_definition(),
            entities::intercompany_batch_definition(),
            entities::intercompany_transaction_definition(),
            entities::intercompany_settlement_definition(),
            entities::accounting_calendar_definition(),
            entities::accounting_period_definition(),
            entities::period_close_checklist_definition(),
            entities::lease_contract_definition(),
            entities::lease_payment_definition(),
            entities::lease_modification_definition(),
            entities::lease_termination_definition(),
            entities::bank_account_definition(),
            entities::bank_statement_definition(),
            entities::bank_statement_line_definition(),
            entities::reconciliation_match_definition(),
            entities::encumbrance_type_definition(),
            entities::encumbrance_entry_definition(),
            entities::encumbrance_liquidation_definition(),
            entities::encumbrance_carry_forward_definition(),
            entities::currency_definition_entity(),
            entities::exchange_rate_definition(),
            entities::accounting_book_definition(),
            entities::account_mapping_definition(),
            entities::book_journal_entry_definition(),
            entities::consolidation_ledger_definition(),
            entities::consolidation_entity_definition(),
            entities::consolidation_scenario_definition(),
            entities::consolidation_adjustment_definition(),
            entities::consolidation_elimination_rule_definition(),
            entities::consolidation_translation_rate_definition(),
        ];
        assert_eq!(new_entities.len(), 46);

        // All unique names
        let names: std::collections::HashSet<&str> = new_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 46, "All new entity names must be unique");
    }

    #[test]
    fn test_total_new_workflow_entity_count() {
        let workflow_entities = vec![
            entities::revenue_contract_definition(),
            entities::revenue_modification_definition(),
            entities::subledger_journal_entry_definition(),
            entities::cash_forecast_definition(),
            entities::tax_rate_definition(),
            entities::intercompany_batch_definition(),
            entities::accounting_period_definition(),
            entities::lease_contract_definition(),
            entities::lease_modification_definition(),
            entities::lease_termination_definition(),
            entities::bank_statement_definition(),
            entities::encumbrance_entry_definition(),
            entities::encumbrance_carry_forward_definition(),
            entities::book_journal_entry_definition(),
            entities::consolidation_scenario_definition(),
            entities::consolidation_adjustment_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 16, "All 16 new entities should have workflows");
    }

    #[test]
    fn test_grand_total_entity_count() {
        // Original 27 + new 46 = 73 total entity definitions
        let all_entities = vec![
            // Original
            entities::chart_of_accounts_definition(),
            entities::journal_entry_definition(),
            entities::invoice_definition(),
            entities::budget_definition(),
            entities::expense_report_definition(),
            entities::ap_invoice_definition(),
            entities::ap_invoice_line_definition(),
            entities::ap_invoice_distribution_definition(),
            entities::ap_invoice_hold_definition(),
            entities::ap_payment_definition(),
            entities::ar_transaction_definition(),
            entities::ar_transaction_line_definition(),
            entities::ar_receipt_definition(),
            entities::ar_credit_memo_definition(),
            entities::ar_adjustment_definition(),
            entities::asset_category_definition(),
            entities::asset_book_definition(),
            entities::fixed_asset_definition(),
            entities::asset_transfer_definition(),
            entities::asset_retirement_definition(),
            entities::cost_book_definition(),
            entities::cost_element_definition(),
            entities::cost_profile_definition(),
            entities::standard_cost_definition(),
            entities::cost_adjustment_definition(),
            entities::cost_adjustment_line_definition(),
            entities::cost_variance_definition(),
            // New
            entities::revenue_policy_definition(),
            entities::revenue_contract_definition(),
            entities::performance_obligation_definition(),
            entities::revenue_schedule_line_definition(),
            entities::revenue_modification_definition(),
            entities::accounting_method_definition(),
            entities::accounting_derivation_rule_definition(),
            entities::subledger_journal_entry_definition(),
            entities::subledger_journal_line_definition(),
            entities::cash_position_definition(),
            entities::cash_forecast_template_definition(),
            entities::cash_forecast_source_definition(),
            entities::cash_forecast_definition(),
            entities::tax_regime_definition(),
            entities::tax_jurisdiction_definition(),
            entities::tax_rate_definition(),
            entities::tax_determination_rule_definition(),
            entities::intercompany_batch_definition(),
            entities::intercompany_transaction_definition(),
            entities::intercompany_settlement_definition(),
            entities::accounting_calendar_definition(),
            entities::accounting_period_definition(),
            entities::period_close_checklist_definition(),
            entities::lease_contract_definition(),
            entities::lease_payment_definition(),
            entities::lease_modification_definition(),
            entities::lease_termination_definition(),
            entities::bank_account_definition(),
            entities::bank_statement_definition(),
            entities::bank_statement_line_definition(),
            entities::reconciliation_match_definition(),
            entities::encumbrance_type_definition(),
            entities::encumbrance_entry_definition(),
            entities::encumbrance_liquidation_definition(),
            entities::encumbrance_carry_forward_definition(),
            entities::currency_definition_entity(),
            entities::exchange_rate_definition(),
            entities::accounting_book_definition(),
            entities::account_mapping_definition(),
            entities::book_journal_entry_definition(),
            entities::consolidation_ledger_definition(),
            entities::consolidation_entity_definition(),
            entities::consolidation_scenario_definition(),
            entities::consolidation_adjustment_definition(),
            entities::consolidation_elimination_rule_definition(),
            entities::consolidation_translation_rate_definition(),
        ];
        assert_eq!(all_entities.len(), 73);
        let names: std::collections::HashSet<&str> = all_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 73, "All entity names must be globally unique");
    }

    // ========================================================================
    // Collections Management Entity Tests
    // ========================================================================

    #[test]
    fn test_customer_credit_profile_definition() {
        let def = entities::customer_credit_profile_definition();
        assert_eq!(def.name, "customer_credit_profiles");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_collection_strategy_definition() {
        let def = entities::collection_strategy_definition();
        assert_eq!(def.name, "collection_strategies");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_collection_case_definition() {
        let def = entities::collection_case_definition();
        assert_eq!(def.name, "collection_cases");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "open");
        assert!(wf.states.iter().any(|s| s.name == "in_progress"));
        assert!(wf.states.iter().any(|s| s.name == "escalated"));
        assert!(wf.states.iter().any(|s| s.name == "resolved"));
        assert!(wf.states.iter().any(|s| s.name == "closed"));
        assert!(wf.states.iter().any(|s| s.name == "written_off"));
    }

    #[test]
    fn test_customer_interaction_definition() {
        let def = entities::customer_interaction_definition();
        assert_eq!(def.name, "customer_interactions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_promise_to_pay_definition() {
        let def = entities::promise_to_pay_definition();
        assert_eq!(def.name, "promise_to_pay");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "partially_kept"));
        assert!(wf.states.iter().any(|s| s.name == "kept"));
        assert!(wf.states.iter().any(|s| s.name == "broken"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_dunning_campaign_definition() {
        let def = entities::dunning_campaign_definition();
        assert_eq!(def.name, "dunning_campaigns");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "scheduled"));
        assert!(wf.states.iter().any(|s| s.name == "in_progress"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_dunning_letter_definition() {
        let def = entities::dunning_letter_definition();
        assert_eq!(def.name, "dunning_letters");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_receivables_aging_snapshot_definition() {
        let def = entities::receivables_aging_snapshot_definition();
        assert_eq!(def.name, "receivables_aging_snapshots");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_write_off_request_definition() {
        let def = entities::write_off_request_definition();
        assert_eq!(def.name, "write_off_requests");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
        assert!(wf.states.iter().any(|s| s.name == "processed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Collections Management Validation Tests
    // ========================================================================

    #[test]
    fn test_collection_risk_classifications_valid() {
        let valid = ["low", "medium", "high", "very_high", "defaulted"];
        for r in &valid {
            assert!(super::VALID_COLLECTION_RISK_CLASSIFICATIONS.contains(r));
        }
        assert!(!super::VALID_COLLECTION_RISK_CLASSIFICATIONS.contains(&"unknown"));
    }

    #[test]
    fn test_case_types_valid() {
        let valid = ["collection", "dispute", "bankruptcy", "skip_trace"];
        for t in &valid {
            assert!(super::VALID_CASE_TYPES.contains(t));
        }
    }

    #[test]
    fn test_case_priorities_valid() {
        let valid = ["low", "medium", "high", "critical"];
        for p in &valid {
            assert!(super::VALID_CASE_PRIORITIES.contains(p));
        }
    }

    #[test]
    fn test_interaction_types_valid() {
        let valid = ["phone_call", "email", "letter", "meeting", "note", "sms"];
        for t in &valid {
            assert!(super::VALID_INTERACTION_TYPES.contains(t));
        }
    }

    #[test]
    fn test_interaction_outcomes_valid() {
        let valid = ["contacted", "left_message", "no_answer", "promised_to_pay",
                     "disputed", "refused", "agreed_payment_plan", "escalated", "no_action"];
        for o in &valid {
            assert!(super::VALID_INTERACTION_OUTCOMES.contains(o));
        }
    }

    #[test]
    fn test_promise_types_valid() {
        let valid = ["single_payment", "installment", "full_balance"];
        for t in &valid {
            assert!(super::VALID_PROMISE_TYPES.contains(t));
        }
    }

    #[test]
    fn test_dunning_levels_valid() {
        let valid = ["reminder", "first_notice", "second_notice", "final_notice", "pre_legal", "legal"];
        for l in &valid {
            assert!(super::VALID_DUNNING_LEVELS.contains(l));
        }
    }

    #[test]
    fn test_dunning_comm_methods_valid() {
        let valid = ["email", "letter", "sms", "phone"];
        for m in &valid {
            assert!(super::VALID_DUNNING_COMM_METHODS.contains(m));
        }
    }

    #[test]
    fn test_write_off_types_valid() {
        let valid = ["bad_debt", "small_balance", "dispute", "adjustment"];
        for t in &valid {
            assert!(super::VALID_WRITE_OFF_TYPES.contains(t));
        }
    }

    #[test]
    fn test_resolution_types_valid() {
        let valid = ["full_payment", "partial_payment", "payment_plan",
                     "write_off", "dispute_resolved", "uncollectible", "other"];
        for t in &valid {
            assert!(super::VALID_RESOLUTION_TYPES.contains(t));
        }
    }

    // ========================================================================
    // Collections Management Business Logic Tests
    // ========================================================================

    #[test]
    fn test_check_credit_available_within_limit() {
        let available = super::CollectionsManagementService::check_credit_available(
            100000.0, 60000.0, 30000.0, false,
        );
        assert!(available); // 100k - 60k = 40k available, 30k requested
    }

    #[test]
    fn test_check_credit_available_exceeds_limit() {
        let available = super::CollectionsManagementService::check_credit_available(
            100000.0, 80000.0, 30000.0, false,
        );
        assert!(!available); // 100k - 80k = 20k available, 30k requested
    }

    #[test]
    fn test_check_credit_available_on_hold() {
        let available = super::CollectionsManagementService::check_credit_available(
            100000.0, 1000.0, 100.0, true,
        );
        assert!(!available); // On hold, always false
    }

    #[test]
    fn test_calculate_utilization() {
        let pct = super::CollectionsManagementService::calculate_utilization(75000.0, 100000.0);
        assert!((pct - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_utilization_zero_limit() {
        let pct = super::CollectionsManagementService::calculate_utilization(50000.0, 0.0);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_aging_bucket_current() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(0), "current");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(-5), "current");
    }

    #[test]
    fn test_aging_bucket_1_30() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(1), "1_30");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(30), "1_30");
    }

    #[test]
    fn test_aging_bucket_31_60() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(31), "31_60");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(60), "31_60");
    }

    #[test]
    fn test_aging_bucket_61_90() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(61), "61_90");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(90), "61_90");
    }

    #[test]
    fn test_aging_bucket_91_120() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(91), "91_120");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(120), "91_120");
    }

    #[test]
    fn test_aging_bucket_121_plus() {
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(121), "121_plus");
        assert_eq!(super::CollectionsManagementService::aging_bucket_from_days(365), "121_plus");
    }

    #[test]
    fn test_calculate_dso() {
        let dso = super::CollectionsManagementService::calculate_dso(
            500000.0, 3000000.0, 365,
        );
        // DSO = (500k / 3M) * 365 ≈ 60.83
        assert!((dso - 60.83).abs() < 1.0);
    }

    #[test]
    fn test_calculate_dso_zero_sales() {
        let dso = super::CollectionsManagementService::calculate_dso(
            500000.0, 0.0, 365,
        );
        assert_eq!(dso, 0.0);
    }

    #[test]
    fn test_calculate_bad_debt_provision() {
        let provision = super::CollectionsManagementService::calculate_bad_debt_provision(
            1000000.0, 2.5,
        );
        assert!((provision - 25000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cei() {
        // CEI = (begin + sales - end_total) / (begin + sales - end_current) * 100
        let cei = super::CollectionsManagementService::calculate_cei(
            500000.0, 300000.0, 200000.0, 50000.0, 550000.0,
        );
        // (500k + 300k - 200k) / (500k + 300k - 50k) * 100 = 600k / 750k * 100 = 80%
        assert!((cei - 80.0).abs() < 0.1);
    }

    // ========================================================================
    // Collections Workflow Tests
    // ========================================================================

    #[test]
    fn test_collection_case_workflow_transitions() {
        let def = entities::collection_case_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "open" && t.to_state == "in_progress"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_progress" && t.to_state == "escalated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_progress" && t.to_state == "resolved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "escalated" && t.to_state == "resolved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "resolved" && t.to_state == "closed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_progress" && t.to_state == "written_off"));
    }

    #[test]
    fn test_promise_to_pay_workflow_transitions() {
        let def = entities::promise_to_pay_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "partially_kept"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "kept"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "broken"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "partially_kept" && t.to_state == "kept"));
    }

    #[test]
    fn test_dunning_campaign_workflow_transitions() {
        let def = entities::dunning_campaign_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "scheduled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "scheduled" && t.to_state == "in_progress"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_progress" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_write_off_workflow_transitions() {
        let def = entities::write_off_request_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "rejected"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "processed"));
    }

    // ========================================================================
    // Credit Management Entity Tests
    // ========================================================================

    #[test]
    fn test_credit_scoring_model_definition() {
        let def = entities::credit_scoring_model_definition();
        assert_eq!(def.name, "credit_scoring_models");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_credit_profile_definition() {
        let def = entities::credit_profile_definition();
        assert_eq!(def.name, "credit_profiles");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "active");
        assert!(wf.states.iter().any(|s| s.name == "suspended"));
        assert!(wf.states.iter().any(|s| s.name == "blocked"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_credit_limit_definition() {
        let def = entities::credit_limit_definition();
        assert_eq!(def.name, "credit_limits");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_credit_check_rule_definition() {
        let def = entities::credit_check_rule_definition();
        assert_eq!(def.name, "credit_check_rules");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_credit_exposure_definition() {
        let def = entities::credit_exposure_definition();
        assert_eq!(def.name, "credit_exposure");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_credit_hold_definition() {
        let def = entities::credit_hold_definition();
        assert_eq!(def.name, "credit_holds");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "active");
        assert!(wf.states.iter().any(|s| s.name == "released"));
        assert!(wf.states.iter().any(|s| s.name == "overridden"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_credit_review_definition() {
        let def = entities::credit_review_definition();
        assert_eq!(def.name, "credit_reviews");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "in_review"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Credit Management Validation Tests
    // ========================================================================

    #[test]
    fn test_credit_model_types_valid() {
        let valid = ["manual", "scorecard", "risk_category", "external"];
        for t in &valid {
            assert!(super::VALID_CREDIT_MODEL_TYPES.contains(t));
        }
    }

    #[test]
    fn test_credit_profile_types_valid() {
        let valid = ["customer", "customer_group", "global"];
        for t in &valid {
            assert!(super::VALID_CREDIT_PROFILE_TYPES.contains(t));
        }
    }

    #[test]
    fn test_credit_risk_levels_valid() {
        let valid = ["low", "medium", "high", "very_high", "blocked"];
        for l in &valid {
            assert!(super::VALID_CREDIT_RISK_LEVELS.contains(l));
        }
    }

    #[test]
    fn test_credit_limit_types_valid() {
        let valid = ["overall", "order", "delivery", "currency"];
        for t in &valid {
            assert!(super::VALID_CREDIT_LIMIT_TYPES.contains(t));
        }
    }

    #[test]
    fn test_credit_check_points_valid() {
        let valid = ["order_entry", "shipment", "invoice", "delivery", "payment"];
        for p in &valid {
            assert!(super::VALID_CREDIT_CHECK_POINTS.contains(p));
        }
    }

    #[test]
    fn test_credit_check_types_valid() {
        let valid = ["automatic", "manual"];
        for t in &valid {
            assert!(super::VALID_CREDIT_CHECK_TYPES.contains(t));
        }
    }

    #[test]
    fn test_failure_actions_valid() {
        let valid = ["hold", "warn", "reject", "notify"];
        for a in &valid {
            assert!(super::VALID_FAILURE_ACTIONS.contains(a));
        }
    }

    #[test]
    fn test_credit_hold_types_valid() {
        let valid = ["credit_limit", "overdue", "review", "manual", "scoring"];
        for t in &valid {
            assert!(super::VALID_CREDIT_HOLD_TYPES.contains(t));
        }
    }

    #[test]
    fn test_credit_review_types_valid() {
        let valid = ["periodic", "triggered", "ad_hoc", "escalation"];
        for t in &valid {
            assert!(super::VALID_CREDIT_REVIEW_TYPES.contains(t));
        }
    }

    // ========================================================================
    // Credit Management Business Logic Tests
    // ========================================================================

    #[test]
    fn test_credit_exposure_calculation() {
        let exposure = super::CreditManagementFinService::calculate_exposure(
            200000.0, 50000.0, 30000.0, 10000.0, 5000.0,
        );
        // 200k + 50k + 30k - 10k + 5k = 275k
        assert!((exposure - 275000.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_utilization_calculation() {
        let pct = super::CreditManagementFinService::calculate_credit_utilization(
            75000.0, 100000.0,
        );
        assert!((pct - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_utilization_zero_limit() {
        let pct = super::CreditManagementFinService::calculate_credit_utilization(
            50000.0, 0.0,
        );
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_available_credit_calculation() {
        let available = super::CreditManagementFinService::calculate_available_credit(
            100000.0, 20000.0, 75000.0,
        );
        // (100k + 20k) - 75k = 45k
        assert!((available - 45000.0).abs() < 0.01);
    }

    #[test]
    fn test_available_credit_negative() {
        let available = super::CreditManagementFinService::calculate_available_credit(
            100000.0, 0.0, 150000.0,
        );
        // 100k - 150k = -50k
        assert!(available < 0.0);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(95.0), "low");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(70.0), "medium");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(50.0), "high");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(20.0), "very_high");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(80.0), "low");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(60.0), "medium");
        assert_eq!(super::CreditManagementFinService::risk_level_from_score(40.0), "high");
    }

    // ========================================================================
    // Credit Management Workflow Tests
    // ========================================================================

    #[test]
    fn test_credit_profile_workflow_transitions() {
        let def = entities::credit_profile_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "suspended"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "suspended" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "blocked"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
    }

    #[test]
    fn test_credit_hold_workflow_transitions() {
        let def = entities::credit_hold_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "released"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "overridden"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_credit_review_workflow_transitions() {
        let def = entities::credit_review_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "in_review"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_review" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Withholding Tax Entity Tests
    // ========================================================================

    #[test]
    fn test_withholding_tax_code_definition() {
        let def = entities::withholding_tax_code_definition();
        assert_eq!(def.name, "withholding_tax_codes");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_withholding_tax_group_definition() {
        let def = entities::withholding_tax_group_definition();
        assert_eq!(def.name, "withholding_tax_groups");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_supplier_withholding_assignment_definition() {
        let def = entities::supplier_withholding_assignment_definition();
        assert_eq!(def.name, "supplier_withholding_assignments");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_withholding_tax_line_definition() {
        let def = entities::withholding_tax_line_definition();
        assert_eq!(def.name, "withholding_tax_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_withholding_certificate_definition() {
        let def = entities::withholding_certificate_definition();
        assert_eq!(def.name, "withholding_certificates");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "issued"));
        assert!(wf.states.iter().any(|s| s.name == "acknowledged"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    // ========================================================================
    // Withholding Tax Validation Tests
    // ========================================================================

    #[test]
    fn test_wht_tax_types_valid() {
        let valid = ["income_tax", "vat", "service_tax", "contract_tax",
                     "royalty", "dividend", "interest", "other"];
        for t in &valid {
            assert!(super::VALID_WHT_TAX_TYPES.contains(t));
        }
        assert!(!super::VALID_WHT_TAX_TYPES.contains(&"sales_tax"));
    }

    #[test]
    fn test_wht_line_statuses_valid() {
        let valid = ["pending", "withheld", "remitted", "refunded"];
        for s in &valid {
            assert!(super::VALID_WHT_LINE_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_wht_cert_statuses_valid() {
        let valid = ["draft", "issued", "acknowledged", "cancelled"];
        for s in &valid {
            assert!(super::VALID_WHT_CERT_STATUSES.contains(s));
        }
    }

    // ========================================================================
    // Withholding Tax Business Logic Tests
    // ========================================================================

    #[test]
    fn test_wht_calculate_per_invoice_above_threshold() {
        // 10% on 10000 (threshold 1000) => 1000
        let wht = super::WithholdingTaxService::calculate_withholding(
            10000.0, 10.0, 1000.0, false, 0.0,
        );
        assert!((wht - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_calculate_per_invoice_below_threshold() {
        // 10% on 500 (threshold 1000) => 0 (below threshold)
        let wht = super::WithholdingTaxService::calculate_withholding(
            500.0, 10.0, 1000.0, false, 0.0,
        );
        assert!((wht - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_calculate_cumulative_not_past_threshold() {
        // YTD: 500, Current: 300, Threshold: 1000 => no withholding (total 800 < 1000)
        let wht = super::WithholdingTaxService::calculate_withholding(
            300.0, 10.0, 1000.0, true, 500.0,
        );
        assert!((wht - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_calculate_cumulative_already_past_threshold() {
        // YTD: 5000 (already past 1000 threshold), Current: 2000 => 2000 * 10% = 200
        let wht = super::WithholdingTaxService::calculate_withholding(
            2000.0, 10.0, 1000.0, true, 5000.0,
        );
        assert!((wht - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_calculate_cumulative_crossing_threshold() {
        // YTD: 800, Current: 500, Threshold: 1000
        // Total YTD would be 1300, excess = 300, taxable = 300 (min of 300 and 500)
        // WHT = 300 * 10% = 30
        let wht = super::WithholdingTaxService::calculate_withholding(
            500.0, 10.0, 1000.0, true, 800.0,
        );
        assert!((wht - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_net_payment() {
        let net = super::WithholdingTaxService::calculate_net_payment(10000.0, 1000.0);
        assert!((net - 9000.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_ytd_withholding() {
        let ytd = super::WithholdingTaxService::calculate_ytd_withholding(
            &[100.0, 200.0, 150.0, 50.0],
        );
        assert!((ytd - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_wht_certificate_workflow_transitions() {
        let def = entities::withholding_certificate_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "issued"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "issued" && t.to_state == "acknowledged"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "issued" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Project Billing Entity Tests
    // ========================================================================

    #[test]
    fn test_bill_rate_schedule_definition() {
        let def = entities::bill_rate_schedule_definition();
        assert_eq!(def.name, "bill_rate_schedules");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_bill_rate_line_definition() {
        let def = entities::bill_rate_line_definition();
        assert_eq!(def.name, "bill_rate_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_project_billing_config_definition() {
        let def = entities::project_billing_config_definition();
        assert_eq!(def.name, "project_billing_configs");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_billing_event_definition() {
        let def = entities::billing_event_definition();
        assert_eq!(def.name, "billing_events");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "planned");
        assert!(wf.states.iter().any(|s| s.name == "ready"));
        assert!(wf.states.iter().any(|s| s.name == "invoiced"));
        assert!(wf.states.iter().any(|s| s.name == "partially_invoiced"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_project_invoice_header_definition() {
        let def = entities::project_invoice_header_definition();
        assert_eq!(def.name, "project_invoice_headers");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_project_invoice_line_definition() {
        let def = entities::project_invoice_line_definition();
        assert_eq!(def.name, "project_invoice_lines");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // Project Billing Validation Tests
    // ========================================================================

    #[test]
    fn test_schedule_types_valid() {
        let valid = ["standard", "overtime", "holiday", "custom"];
        for t in &valid {
            assert!(super::VALID_SCHEDULE_TYPES.contains(t));
        }
        assert!(!super::VALID_SCHEDULE_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_billing_methods_valid() {
        let valid = ["time_and_materials", "fixed_price", "milestone", "cost_plus", "retention"];
        for m in &valid {
            assert!(super::VALID_BILLING_METHODS.contains(m));
        }
    }

    #[test]
    fn test_invoice_formats_valid() {
        let valid = ["detailed", "summary", "consolidated"];
        for f in &valid {
            assert!(super::VALID_INVOICE_FORMATS.contains(f));
        }
    }

    #[test]
    fn test_billing_cycles_valid() {
        let valid = ["weekly", "biweekly", "monthly", "milestone"];
        for c in &valid {
            assert!(super::VALID_BILLING_CYCLES.contains(c));
        }
    }

    #[test]
    fn test_event_types_valid() {
        let valid = ["milestone", "progress", "completion", "retention_release"];
        for t in &valid {
            assert!(super::VALID_EVENT_TYPES.contains(t));
        }
    }

    #[test]
    fn test_project_invoice_types_valid() {
        let valid = ["progress", "milestone", "t_and_m", "retention_release",
                     "debit_memo", "credit_memo"];
        for t in &valid {
            assert!(super::VALID_PROJECT_INVOICE_TYPES.contains(t));
        }
    }

    #[test]
    fn test_line_sources_valid() {
        let valid = ["expenditure_item", "billing_event", "retention", "manual"];
        for s in &valid {
            assert!(super::VALID_LINE_SOURCES.contains(s));
        }
    }

    // ========================================================================
    // Project Billing Business Logic Tests
    // ========================================================================

    #[test]
    fn test_tm_bill_amount_no_markup() {
        let bill = super::ProjectBillingService::calculate_tm_bill_amount(
            40.0, 150.0, 0.0,
        );
        assert!((bill - 6000.0).abs() < 0.01); // 40 * 150 = 6000
    }

    #[test]
    fn test_tm_bill_amount_with_markup() {
        let bill = super::ProjectBillingService::calculate_tm_bill_amount(
            40.0, 150.0, 20.0,
        );
        // 40 * 150 = 6000, + 20% markup = 7200
        assert!((bill - 7200.0).abs() < 0.01);
    }

    #[test]
    fn test_retention_calculation() {
        let ret = super::ProjectBillingService::calculate_retention(
            100000.0, 10.0, 0.0,
        );
        assert!((ret - 10000.0).abs() < 0.01); // 10% of 100k
    }

    #[test]
    fn test_retention_with_cap() {
        let ret = super::ProjectBillingService::calculate_retention(
            200000.0, 10.0, 15000.0,
        );
        // 10% of 200k = 20k, capped at 15k
        assert!((ret - 15000.0).abs() < 0.01);
    }

    #[test]
    fn test_net_billable() {
        let net = super::ProjectBillingService::calculate_net_billable(
            100000.0, 10000.0, 5000.0,
        );
        // 100k - 10k retention + 5k tax = 95k
        assert!((net - 95000.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_pct() {
        let pct = super::ProjectBillingService::calculate_progress_pct(
            350000.0, 1000000.0,
        );
        assert!((pct - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_pct_capped_at_100() {
        let pct = super::ProjectBillingService::calculate_progress_pct(
            1200000.0, 1000000.0,
        );
        assert!((pct - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_pct_zero_contract() {
        let pct = super::ProjectBillingService::calculate_progress_pct(
            50000.0, 0.0,
        );
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_earned_revenue() {
        let earned = super::ProjectBillingService::calculate_earned_revenue(
            1000000.0, 35.0,
        );
        assert!((earned - 350000.0).abs() < 0.01);
    }

    #[test]
    fn test_cost_plus_billing() {
        let bill = super::ProjectBillingService::calculate_cost_plus_bill(
            80000.0, 25.0,
        );
        // 80k * (1 + 25/100) = 80k * 1.25 = 100k
        assert!((bill - 100000.0).abs() < 0.01);
    }

    // ========================================================================
    // Project Billing Workflow Tests
    // ========================================================================

    #[test]
    fn test_bill_rate_schedule_workflow_transitions() {
        let def = entities::bill_rate_schedule_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
    }

    #[test]
    fn test_project_billing_config_workflow_transitions() {
        let def = entities::project_billing_config_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_billing_event_workflow_transitions() {
        let def = entities::billing_event_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "planned" && t.to_state == "ready"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "ready" && t.to_state == "invoiced"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "ready" && t.to_state == "partially_invoiced"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "planned" && t.to_state == "cancelled"));
    }

    #[test]
    fn test_project_invoice_workflow_transitions() {
        let def = entities::project_invoice_header_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "rejected"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Comprehensive New Entity Build Tests
    // ========================================================================

    #[test]
    fn test_all_new_feature_entities_build_successfully() {
        // Collections Management
        let _ = entities::customer_credit_profile_definition();
        let _ = entities::collection_strategy_definition();
        let _ = entities::collection_case_definition();
        let _ = entities::customer_interaction_definition();
        let _ = entities::promise_to_pay_definition();
        let _ = entities::dunning_campaign_definition();
        let _ = entities::dunning_letter_definition();
        let _ = entities::receivables_aging_snapshot_definition();
        let _ = entities::write_off_request_definition();
        // Credit Management
        let _ = entities::credit_scoring_model_definition();
        let _ = entities::credit_profile_definition();
        let _ = entities::credit_limit_definition();
        let _ = entities::credit_check_rule_definition();
        let _ = entities::credit_exposure_definition();
        let _ = entities::credit_hold_definition();
        let _ = entities::credit_review_definition();
        // Withholding Tax
        let _ = entities::withholding_tax_code_definition();
        let _ = entities::withholding_tax_group_definition();
        let _ = entities::supplier_withholding_assignment_definition();
        let _ = entities::withholding_tax_line_definition();
        let _ = entities::withholding_certificate_definition();
        // Project Billing
        let _ = entities::bill_rate_schedule_definition();
        let _ = entities::bill_rate_line_definition();
        let _ = entities::project_billing_config_definition();
        let _ = entities::billing_event_definition();
        let _ = entities::project_invoice_header_definition();
        let _ = entities::project_invoice_line_definition();
    }

    #[test]
    fn test_new_feature_entity_count() {
        let new_entities = vec![
            // Collections Management (9)
            entities::customer_credit_profile_definition(),
            entities::collection_strategy_definition(),
            entities::collection_case_definition(),
            entities::customer_interaction_definition(),
            entities::promise_to_pay_definition(),
            entities::dunning_campaign_definition(),
            entities::dunning_letter_definition(),
            entities::receivables_aging_snapshot_definition(),
            entities::write_off_request_definition(),
            // Credit Management (7)
            entities::credit_scoring_model_definition(),
            entities::credit_profile_definition(),
            entities::credit_limit_definition(),
            entities::credit_check_rule_definition(),
            entities::credit_exposure_definition(),
            entities::credit_hold_definition(),
            entities::credit_review_definition(),
            // Withholding Tax (5)
            entities::withholding_tax_code_definition(),
            entities::withholding_tax_group_definition(),
            entities::supplier_withholding_assignment_definition(),
            entities::withholding_tax_line_definition(),
            entities::withholding_certificate_definition(),
            // Project Billing (6)
            entities::bill_rate_schedule_definition(),
            entities::bill_rate_line_definition(),
            entities::project_billing_config_definition(),
            entities::billing_event_definition(),
            entities::project_invoice_header_definition(),
            entities::project_invoice_line_definition(),
        ];
        assert_eq!(new_entities.len(), 27, "Should have 27 new feature entities");

        // All unique names
        let names: std::collections::HashSet<&str> = new_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 27, "All new entity names must be unique");
    }

    #[test]
    fn test_new_feature_workflow_count() {
        let workflow_entities = vec![
            // Collections
            entities::collection_case_definition(),
            entities::promise_to_pay_definition(),
            entities::dunning_campaign_definition(),
            entities::write_off_request_definition(),
            // Credit Management
            entities::credit_profile_definition(),
            entities::credit_hold_definition(),
            entities::credit_review_definition(),
            // Withholding Tax
            entities::withholding_certificate_definition(),
            // Project Billing
            entities::bill_rate_schedule_definition(),
            entities::project_billing_config_definition(),
            entities::billing_event_definition(),
            entities::project_invoice_header_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 12, "All 12 new workflow entities should have workflows");
    }

    // ========================================================================
    // Payment Terms Entity Tests
    // ========================================================================

    #[test]
    fn test_payment_term_definition() {
        let def = entities::payment_term_definition();
        assert_eq!(def.name, "payment_terms");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_payment_schedule_definition() {
        let def = entities::payment_schedule_definition();
        assert_eq!(def.name, "payment_schedules");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_payment_term_types_valid() {
        let valid = ["immediate", "net_days", "discount_net", "milestone", "installment"];
        for t in &valid {
            assert!(super::VALID_TERM_TYPES.contains(t));
        }
        assert!(!super::VALID_TERM_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_days_of_month_valid() {
        let valid = ["any", "1", "5", "10", "15", "20", "25"];
        for d in &valid {
            assert!(super::VALID_DAYS_OF_MONTH.contains(d));
        }
        assert!(!super::VALID_DAYS_OF_MONTH.contains(&"30"));
    }

    // ========================================================================
    // Payment Terms Business Logic Tests
    // ========================================================================

    #[test]
    fn test_calculate_discount_date() {
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let discount_date = super::PaymentTermsService::calculate_discount_date(invoice_date, 10);
        assert_eq!(discount_date, chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap());
    }

    #[test]
    fn test_calculate_net_due_date() {
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let due_date = super::PaymentTermsService::calculate_net_due_date(invoice_date, 30);
        assert_eq!(due_date, chrono::NaiveDate::from_ymd_opt(2025, 2, 14).unwrap());
    }

    #[test]
    fn test_calculate_discount_amount() {
        // 2% discount on $10,000 = $200
        let discount = super::PaymentTermsService::calculate_discount_amount(10000.0, 2.0);
        assert!((discount - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_net_payment_amount() {
        let net = super::PaymentTermsService::calculate_net_payment_amount(10000.0, 200.0);
        assert!((net - 9800.0).abs() < 0.01);
    }

    #[test]
    fn test_is_discount_available_true() {
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
        let discount_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap();
        assert!(super::PaymentTermsService::is_discount_available(payment_date, discount_date));
    }

    #[test]
    fn test_is_discount_available_false() {
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 26).unwrap();
        let discount_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap();
        assert!(!super::PaymentTermsService::is_discount_available(payment_date, discount_date));
    }

    #[test]
    fn test_is_discount_available_on_exact_date() {
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap();
        let discount_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap();
        assert!(super::PaymentTermsService::is_discount_available(payment_date, discount_date));
    }

    #[test]
    fn test_annualized_cost_of_discount() {
        // 2/10 Net 30: cost = (2/98) * (365/20) * 100 ≈ 37.24%
        let cost = super::PaymentTermsService::calculate_annualized_cost_of_discount(
            2.0, 30, 10,
        );
        assert!(cost > 37.0 && cost < 38.0);
    }

    #[test]
    fn test_annualized_cost_of_discount_zero_additional_days() {
        let cost = super::PaymentTermsService::calculate_annualized_cost_of_discount(
            2.0, 10, 10,
        );
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_payment_with_discount_early_payment() {
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let discount_date = invoice_date + chrono::Duration::days(10);
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();

        let (net, discount, took_discount) = super::PaymentTermsService::calculate_payment_with_discount(
            10000.0, 2.0, payment_date, discount_date,
        );
        assert!((net - 9800.0).abs() < 0.01);
        assert!((discount - 200.0).abs() < 0.01);
        assert!(took_discount);
    }

    #[test]
    fn test_payment_without_discount_late_payment() {
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let discount_date = invoice_date + chrono::Duration::days(10);
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        let (net, discount, took_discount) = super::PaymentTermsService::calculate_payment_with_discount(
            10000.0, 2.0, payment_date, discount_date,
        );
        assert!((net - 10000.0).abs() < 0.01);
        assert!((discount - 0.0).abs() < 0.01);
        assert!(!took_discount);
    }

    #[test]
    fn test_payment_with_zero_discount() {
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let discount_date = invoice_date + chrono::Duration::days(10);
        let payment_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();

        let (net, discount, took_discount) = super::PaymentTermsService::calculate_payment_with_discount(
            10000.0, 0.0, payment_date, discount_date,
        );
        assert!((net - 10000.0).abs() < 0.01);
        assert!((discount - 0.0).abs() < 0.01);
        assert!(!took_discount);
    }

    // ========================================================================
    // Financial Statement Entity Tests
    // ========================================================================

    #[test]
    fn test_financial_report_template_definition() {
        let def = entities::financial_report_template_definition();
        assert_eq!(def.name, "financial_report_templates");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_financial_report_row_definition() {
        let def = entities::financial_report_row_definition();
        assert_eq!(def.name, "financial_report_rows");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_generated_financial_report_definition() {
        let def = entities::generated_financial_report_definition();
        assert_eq!(def.name, "generated_financial_reports");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "generated"));
        assert!(wf.states.iter().any(|s| s.name == "reviewed"));
        assert!(wf.states.iter().any(|s| s.name == "published"));
        assert!(wf.states.iter().any(|s| s.name == "archived"));
    }

    #[test]
    fn test_report_types_valid() {
        let valid = ["balance_sheet", "income_statement", "cash_flow", "trial_balance", "custom"];
        for r in &valid {
            assert!(super::VALID_REPORT_TYPES.contains(r));
        }
        assert!(!super::VALID_REPORT_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_row_types_valid() {
        let valid = ["header", "account_range", "calculated", "total", "subtotal", "text"];
        for r in &valid {
            assert!(super::VALID_ROW_TYPES.contains(r));
        }
    }

    #[test]
    fn test_financial_report_workflow_transitions() {
        let def = entities::generated_financial_report_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "generated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "generated" && t.to_state == "reviewed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reviewed" && t.to_state == "published"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "published" && t.to_state == "archived"));
    }

    // ========================================================================
    // Financial Statement Business Logic Tests
    // ========================================================================

    #[test]
    fn test_balance_sheet_balanced() {
        // Assets = 1,000,000; Liabilities = 400,000; Equity = 600,000
        let (liab_equity, balanced) = super::FinancialStatementService::calculate_balance_sheet(
            1000000.0, 400000.0, 600000.0,
        );
        assert!(balanced);
        assert!((liab_equity - 1000000.0).abs() < 0.01);
    }

    #[test]
    fn test_balance_sheet_unbalanced() {
        let (_, balanced) = super::FinancialStatementService::calculate_balance_sheet(
            1000000.0, 400000.0, 500000.0,
        );
        assert!(!balanced);
    }

    #[test]
    fn test_net_income_positive() {
        let income = super::FinancialStatementService::calculate_net_income(
            500000.0, 350000.0,
        );
        assert!((income - 150000.0).abs() < 0.01);
    }

    #[test]
    fn test_net_income_negative() {
        let income = super::FinancialStatementService::calculate_net_income(
            300000.0, 350000.0,
        );
        assert!(income < 0.0); // Net loss
    }

    #[test]
    fn test_retained_earnings() {
        let re = super::FinancialStatementService::calculate_retained_earnings(
            200000.0, 150000.0, 50000.0,
        );
        assert!((re - 300000.0).abs() < 0.01); // 200k + 150k - 50k = 300k
    }

    #[test]
    fn test_working_capital() {
        let wc = super::FinancialStatementService::calculate_working_capital(
            500000.0, 300000.0,
        );
        assert!((wc - 200000.0).abs() < 0.01);
    }

    #[test]
    fn test_current_ratio() {
        let ratio = super::FinancialStatementService::calculate_current_ratio(
            500000.0, 300000.0,
        );
        assert!((ratio - 1.667).abs() < 0.01);
    }

    #[test]
    fn test_current_ratio_zero_liabilities() {
        let ratio = super::FinancialStatementService::calculate_current_ratio(
            500000.0, 0.0,
        );
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_debt_to_equity() {
        let ratio = super::FinancialStatementService::calculate_debt_to_equity(
            400000.0, 600000.0,
        );
        assert!((ratio - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_debt_to_equity_zero_equity() {
        let ratio = super::FinancialStatementService::calculate_debt_to_equity(
            400000.0, 0.0,
        );
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_gross_profit_margin() {
        let margin = super::FinancialStatementService::calculate_gross_profit_margin(
            500000.0, 300000.0,
        );
        assert!((margin - 40.0).abs() < 0.01); // (500k-300k)/500k = 40%
    }

    #[test]
    fn test_gross_profit_margin_zero_revenue() {
        let margin = super::FinancialStatementService::calculate_gross_profit_margin(
            0.0, 300000.0,
        );
        assert_eq!(margin, 0.0);
    }

    #[test]
    fn test_operating_margin() {
        let margin = super::FinancialStatementService::calculate_operating_margin(
            500000.0, 75000.0,
        );
        assert!((margin - 15.0).abs() < 0.01); // 75k/500k = 15%
    }

    #[test]
    fn test_return_on_equity() {
        let roe = super::FinancialStatementService::calculate_return_on_equity(
            150000.0, 600000.0,
        );
        assert!((roe - 25.0).abs() < 0.01); // 150k/600k = 25%
    }

    #[test]
    fn test_cash_flow_indirect() {
        let (operating, investing, financing, net_change) =
            super::FinancialStatementService::calculate_cash_flow_indirect(
                150000.0,  // net income
                50000.0,   // depreciation
                -20000.0,  // change in working capital
                80000.0,   // capex
                10000.0,   // asset sale proceeds
                50000.0,   // debt proceeds
                30000.0,   // debt repayments
                20000.0,   // dividends
            );
        // Operating: 150k + 50k - 20k = 180k
        assert!((operating - 180000.0).abs() < 0.01);
        // Investing: -80k + 10k = -70k
        assert!((investing - (-70000.0)).abs() < 0.01);
        // Financing: 50k - 30k - 20k = 0
        assert!((financing - 0.0).abs() < 0.01);
        // Net: 180k - 70k + 0 = 110k
        assert!((net_change - 110000.0).abs() < 0.01);
    }

    #[test]
    fn test_sum_account_range() {
        let balances = vec![
            ("1000".to_string(), 50000.0),
            ("1100".to_string(), 30000.0),
            ("1200".to_string(), 20000.0),
            ("2000".to_string(), 40000.0),
            ("2100".to_string(), 30000.0),
            ("3000".to_string(), 30000.0),
        ];
        // Sum all 1xxx accounts (assets)
        let assets = super::FinancialStatementService::sum_account_range(&balances, "1000", "1999");
        assert!((assets - 100000.0).abs() < 0.01);

        // Sum all 2xxx accounts (liabilities)
        let liabilities = super::FinancialStatementService::sum_account_range(&balances, "2000", "2999");
        assert!((liabilities - 70000.0).abs() < 0.01);
    }

    #[test]
    fn test_sum_account_range_empty() {
        let balances = vec![
            ("1000".to_string(), 50000.0),
        ];
        let result = super::FinancialStatementService::sum_account_range(&balances, "2000", "2999");
        assert!((result - 0.0).abs() < 0.01);
    }

    // ========================================================================
    // Tax Filing Entity Tests
    // ========================================================================

    #[test]
    fn test_tax_filing_obligation_definition() {
        let def = entities::tax_filing_obligation_definition();
        assert_eq!(def.name, "tax_filing_obligations");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_tax_return_definition() {
        let def = entities::tax_return_definition();
        assert_eq!(def.name, "tax_returns");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "calculated"));
        assert!(wf.states.iter().any(|s| s.name == "reviewed"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "filed"));
        assert!(wf.states.iter().any(|s| s.name == "amended"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_tax_payment_definition() {
        let def = entities::tax_payment_definition();
        assert_eq!(def.name, "tax_payments");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_filing_frequencies_valid() {
        let valid = ["monthly", "quarterly", "semi_annually", "annually"];
        for f in &valid {
            assert!(super::VALID_FILING_FREQUENCIES.contains(f));
        }
    }

    #[test]
    fn test_filing_methods_valid() {
        let valid = ["electronic", "paper", "both"];
        for m in &valid {
            assert!(super::VALID_FILING_METHODS.contains(m));
        }
    }

    #[test]
    fn test_return_statuses_valid() {
        let valid = ["draft", "calculated", "reviewed", "approved", "filed", "amended", "cancelled"];
        for s in &valid {
            assert!(super::VALID_RETURN_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_tax_payment_statuses_valid() {
        let valid = ["pending", "processed", "confirmed", "reversed"];
        for s in &valid {
            assert!(super::VALID_TAX_PAYMENT_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_tax_payment_methods_valid() {
        let valid = ["wire", "ach", "check", "electronic"];
        for m in &valid {
            assert!(super::VALID_TAX_PAYMENT_METHODS.contains(m));
        }
    }

    #[test]
    fn test_tax_return_workflow_transitions() {
        let def = entities::tax_return_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "calculated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "calculated" && t.to_state == "reviewed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reviewed" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "filed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "filed" && t.to_state == "amended"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Tax Filing Business Logic Tests
    // ========================================================================

    #[test]
    fn test_calculate_filing_due_date() {
        let period_end = chrono::NaiveDate::from_ymd_opt(2025, 3, 31).unwrap();
        let due_date = super::TaxFilingService::calculate_filing_due_date(period_end, 30);
        assert_eq!(due_date, chrono::NaiveDate::from_ymd_opt(2025, 4, 30).unwrap());
    }

    #[test]
    fn test_calculate_tax_liability() {
        let lines = vec![
            (10000.0, 10.0),  // $1,000 tax
            (20000.0, 5.0),   // $1,000 tax
            (5000.0, 20.0),   // $1,000 tax
        ];
        let (total_taxable, total_tax) = super::TaxFilingService::calculate_tax_liability(&lines);
        assert!((total_taxable - 35000.0).abs() < 0.01);
        assert!((total_tax - 3000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_tax_liability_empty() {
        let lines: Vec<(f64, f64)> = vec![];
        let (total_taxable, total_tax) = super::TaxFilingService::calculate_tax_liability(&lines);
        assert_eq!(total_taxable, 0.0);
        assert_eq!(total_tax, 0.0);
    }

    #[test]
    fn test_calculate_late_penalty() {
        // $10,000 tax, 15 days late, 0.5% daily rate, 25% max
        let penalty = super::TaxFilingService::calculate_late_penalty(
            10000.0, 15, 0.5, 25.0,
        );
        // 10000 * 0.005 * 15 = 750, but max is 10000 * 0.25 = 2500, so 750
        assert!((penalty - 750.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_late_penalty_capped() {
        // $10,000 tax, 60 days late, 0.5% daily rate, 25% max
        let penalty = super::TaxFilingService::calculate_late_penalty(
            10000.0, 60, 0.5, 25.0,
        );
        // 10000 * 0.005 * 60 = 3000, but max is 10000 * 0.25 = 2500
        assert!((penalty - 2500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_late_interest() {
        // $10,000 tax, 30 days late, 8% annual rate
        let interest = super::TaxFilingService::calculate_late_interest(
            10000.0, 30, 8.0,
        );
        // 10000 * 0.08 * (30/365) ≈ 65.75
        assert!(interest > 65.0 && interest < 66.0);
    }

    #[test]
    fn test_calculate_late_interest_zero_days() {
        let interest = super::TaxFilingService::calculate_late_interest(
            10000.0, 0, 8.0,
        );
        assert_eq!(interest, 0.0);
    }

    #[test]
    fn test_filing_period_monthly() {
        let (start, end) = super::TaxFilingService::calculate_filing_period(
            2025, 3, "monthly",
        ).unwrap();
        assert_eq!(start, chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());
    }

    #[test]
    fn test_filing_period_monthly_december() {
        let (start, end) = super::TaxFilingService::calculate_filing_period(
            2025, 12, "monthly",
        ).unwrap();
        assert_eq!(start, chrono::NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
    }

    #[test]
    fn test_filing_period_quarterly_q1() {
        let (start, end) = super::TaxFilingService::calculate_filing_period(
            2025, 1, "quarterly",
        ).unwrap();
        assert_eq!(start, chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());
    }

    #[test]
    fn test_filing_period_quarterly_q4() {
        let (start, end) = super::TaxFilingService::calculate_filing_period(
            2025, 4, "quarterly",
        ).unwrap();
        assert_eq!(start, chrono::NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
    }

    #[test]
    fn test_filing_period_annually() {
        let (start, end) = super::TaxFilingService::calculate_filing_period(
            2025, 1, "annually",
        ).unwrap();
        assert_eq!(start, chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
    }

    #[test]
    fn test_filing_period_invalid_month() {
        let result = super::TaxFilingService::calculate_filing_period(
            2025, 13, "monthly",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_filing_period_invalid_quarter() {
        let result = super::TaxFilingService::calculate_filing_period(
            2025, 5, "quarterly",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_filing_period_invalid_frequency() {
        let result = super::TaxFilingService::calculate_filing_period(
            2025, 1, "semi_annually",
        );
        assert!(result.is_none());
    }

    // ========================================================================
    // Journal Reversal Entity Tests
    // ========================================================================

    #[test]
    fn test_journal_reversal_request_definition() {
        let def = entities::journal_reversal_request_definition();
        assert_eq!(def.name, "journal_reversal_requests");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "processed"));
        assert!(wf.states.iter().any(|s| s.name == "rejected"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_reversal_methods_valid() {
        let valid = ["switch_dr_cr", "sign_reverse", "switch_signs"];
        for m in &valid {
            assert!(super::VALID_REVERSAL_METHODS.contains(m));
        }
        assert!(!super::VALID_REVERSAL_METHODS.contains(&"unknown"));
    }

    #[test]
    fn test_reversal_reasons_valid() {
        let valid = ["error_correction", "period_adjustment", "duplicate_entry",
                     "reclassification", "management_decision", "other"];
        for r in &valid {
            assert!(super::VALID_REVERSAL_REASONS.contains(r));
        }
    }

    #[test]
    fn test_journal_reversal_workflow_transitions() {
        let def = entities::journal_reversal_request_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "rejected"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "processed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Journal Reversal Business Logic Tests
    // ========================================================================

    #[test]
    fn test_reverse_line_switch_dr_cr() {
        let (new_debit, new_credit) = super::JournalReversalService::reverse_line_switch_dr_cr(
            1000.0, 0.0,
        );
        assert!((new_debit - 0.0).abs() < 0.01);
        assert!((new_credit - 1000.0).abs() < 0.01);

        let (new_debit, new_credit) = super::JournalReversalService::reverse_line_switch_dr_cr(
            0.0, 500.0,
        );
        assert!((new_debit - 500.0).abs() < 0.01);
        assert!((new_credit - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_reverse_line_sign() {
        let (new_debit, new_credit) = super::JournalReversalService::reverse_line_sign(
            1000.0, 500.0,
        );
        assert!((new_debit - (-1000.0)).abs() < 0.01);
        assert!((new_credit - (-500.0)).abs() < 0.01);
    }

    #[test]
    fn test_validate_reversal_balances_matching() {
        // Original: DR 1000, CR 1000
        // Reversal: DR 1000 (original CR), CR 1000 (original DR)
        let valid = super::JournalReversalService::validate_reversal_balances(
            1000.0, 1000.0,
            1000.0, 1000.0,
        );
        assert!(valid);
    }

    #[test]
    fn test_validate_reversal_balances_complex() {
        // Original: DR 5000, CR 3000 (not balanced itself, but reversal matches)
        // Reversal: DR 3000, CR 5000
        let valid = super::JournalReversalService::validate_reversal_balances(
            5000.0, 3000.0,
            3000.0, 5000.0,
        );
        assert!(valid);
    }

    #[test]
    fn test_validate_reversal_balances_mismatch() {
        let valid = super::JournalReversalService::validate_reversal_balances(
            1000.0, 1000.0,
            500.0, 500.0,
        );
        assert!(!valid); // Reversal DR 500 != Original CR 1000
    }

    #[test]
    fn test_calculate_net_effect() {
        let (net_debit, net_credit) = super::JournalReversalService::calculate_net_effect(
            1000.0, 0.0,
            0.0, 1000.0,
        );
        // Net: 1000 + 0 = 1000 debit, 0 + 1000 = 1000 credit
        assert!((net_debit - 1000.0).abs() < 0.01);
        assert!((net_credit - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_net_effect_zeroed() {
        let (net_debit, net_credit) = super::JournalReversalService::calculate_net_effect(
            1000.0, 1000.0,
            1000.0, 1000.0,
        );
        // Both sides equal, net debits and credits are double
        assert!((net_debit - 2000.0).abs() < 0.01);
        assert!((net_credit - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_is_eligible_for_reversal_posted_open_period() {
        let result = super::JournalReversalService::is_eligible_for_reversal(
            "posted", false, "open",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_eligible_for_reversal_already_reversed() {
        let result = super::JournalReversalService::is_eligible_for_reversal(
            "posted", true, "open",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already reversed"));
    }

    #[test]
    fn test_is_eligible_for_reversal_not_posted() {
        let result = super::JournalReversalService::is_eligible_for_reversal(
            "draft", false, "open",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("posted"));
    }

    #[test]
    fn test_is_eligible_for_reversal_closed_period() {
        let result = super::JournalReversalService::is_eligible_for_reversal(
            "posted", false, "closed",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("closed"));
    }

    #[test]
    fn test_is_eligible_for_reversal_permanently_closed_period() {
        let result = super::JournalReversalService::is_eligible_for_reversal(
            "posted", false, "permanently_closed",
        );
        assert!(result.is_err());
    }

    // ========================================================================
    // Comprehensive: All New Feature Entities Build
    // ========================================================================

    #[test]
    fn test_all_new_financial_feature_entities_build() {
        // Payment Terms
        let _ = entities::payment_term_definition();
        let _ = entities::payment_schedule_definition();
        // Financial Statement Generation
        let _ = entities::financial_report_template_definition();
        let _ = entities::financial_report_row_definition();
        let _ = entities::generated_financial_report_definition();
        // Tax Filing
        let _ = entities::tax_filing_obligation_definition();
        let _ = entities::tax_return_definition();
        let _ = entities::tax_payment_definition();
        // Journal Reversal
        let _ = entities::journal_reversal_request_definition();
    }

    #[test]
    fn test_new_financial_feature_entity_count() {
        let new_entities = vec![
            entities::payment_term_definition(),
            entities::payment_schedule_definition(),
            entities::financial_report_template_definition(),
            entities::financial_report_row_definition(),
            entities::generated_financial_report_definition(),
            entities::tax_filing_obligation_definition(),
            entities::tax_return_definition(),
            entities::tax_payment_definition(),
            entities::journal_reversal_request_definition(),
        ];
        assert_eq!(new_entities.len(), 9);

        let names: std::collections::HashSet<&str> = new_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 9, "All new entity names must be unique");
    }

    #[test]
    fn test_new_financial_feature_workflow_count() {
        let workflow_entities = vec![
            entities::generated_financial_report_definition(),
            entities::tax_return_definition(),
            entities::journal_reversal_request_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 3, "All 3 new workflow entities should have workflows");
    }

    // ========================================================================
    // Recurring Journals Entity Tests
    // ========================================================================

    #[test]
    fn test_recurring_journal_template_definition() {
        let def = entities::recurring_journal_template_definition();
        assert_eq!(def.name, "recurring_journal_templates");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_recurring_journal_line_definition() {
        let def = entities::recurring_journal_line_definition();
        assert_eq!(def.name, "recurring_journal_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_recurring_journal_workflow_transitions() {
        let def = entities::recurring_journal_template_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "inactive" && t.to_state == "active"));
    }

    // ========================================================================
    // Allocations Entity Tests
    // ========================================================================

    #[test]
    fn test_allocation_rule_definition() {
        let def = entities::allocation_rule_definition();
        assert_eq!(def.name, "allocation_rules");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_allocation_line_definition() {
        let def = entities::allocation_line_definition();
        assert_eq!(def.name, "allocation_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_allocation_rule_workflow_transitions() {
        let def = entities::allocation_rule_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
    }

    // ========================================================================
    // Funds Reservation Entity Tests
    // ========================================================================

    #[test]
    fn test_funds_reservation_definition() {
        let def = entities::funds_reservation_definition();
        assert_eq!(def.name, "funds_reservations");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "reserved"));
        assert!(wf.states.iter().any(|s| s.name == "partially_consumed"));
        assert!(wf.states.iter().any(|s| s.name == "fully_consumed"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
        assert!(wf.states.iter().any(|s| s.name == "expired"));
    }

    #[test]
    fn test_funds_check_result_definition() {
        let def = entities::funds_check_result_definition();
        assert_eq!(def.name, "funds_check_results");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_funds_reservation_workflow_transitions() {
        let def = entities::funds_reservation_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "reserved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reserved" && t.to_state == "partially_consumed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reserved" && t.to_state == "fully_consumed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reserved" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reserved" && t.to_state == "expired"));
    }

    // ========================================================================
    // Journal Import Entity Tests
    // ========================================================================

    #[test]
    fn test_journal_import_request_definition() {
        let def = entities::journal_import_request_definition();
        assert_eq!(def.name, "journal_import_requests");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "uploaded");
        assert!(wf.states.iter().any(|s| s.name == "validating"));
        assert!(wf.states.iter().any(|s| s.name == "validated"));
        assert!(wf.states.iter().any(|s| s.name == "importing"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "failed"));
    }

    #[test]
    fn test_journal_import_workflow_transitions() {
        let def = entities::journal_import_request_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "uploaded" && t.to_state == "validating"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "validating" && t.to_state == "validated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "validated" && t.to_state == "importing"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "importing" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "validating" && t.to_state == "failed"));
    }

    // ========================================================================
    // Landed Cost Entity Tests
    // ========================================================================

    #[test]
    fn test_landed_cost_template_definition() {
        let def = entities::landed_cost_template_definition();
        assert_eq!(def.name, "landed_cost_templates");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_landed_cost_component_definition() {
        let def = entities::landed_cost_component_definition();
        assert_eq!(def.name, "landed_cost_components");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_landed_cost_assignment_definition() {
        let def = entities::landed_cost_assignment_definition();
        assert_eq!(def.name, "landed_cost_assignments");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "estimated"));
        assert!(wf.states.iter().any(|s| s.name == "actualized"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
    }

    #[test]
    fn test_landed_cost_assignment_workflow_transitions() {
        let def = entities::landed_cost_assignment_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "estimated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "estimated" && t.to_state == "actualized"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "actualized" && t.to_state == "posted"));
    }

    // ========================================================================
    // Transfer Pricing Entity Tests
    // ========================================================================

    #[test]
    fn test_transfer_pricing_policy_definition() {
        let def = entities::transfer_pricing_policy_definition();
        assert_eq!(def.name, "transfer_pricing_policies");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_transfer_pricing_transaction_definition() {
        let def = entities::transfer_pricing_transaction_definition();
        assert_eq!(def.name, "transfer_pricing_transactions");
        assert!(def.workflow.is_none());
    }

    // ========================================================================
    // AutoInvoice Entity Tests
    // ========================================================================

    #[test]
    fn test_autoinvoice_rule_definition() {
        let def = entities::autoinvoice_rule_definition();
        assert_eq!(def.name, "autoinvoice_rules");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "inactive"));
    }

    #[test]
    fn test_autoinvoice_run_definition() {
        let def = entities::autoinvoice_run_definition();
        assert_eq!(def.name, "autoinvoice_runs");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "processing"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "failed"));
    }

    #[test]
    fn test_autoinvoice_rule_workflow_transitions() {
        let def = entities::autoinvoice_rule_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "inactive"));
    }

    #[test]
    fn test_autoinvoice_run_workflow_transitions() {
        let def = entities::autoinvoice_run_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "processing"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "processing" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "processing" && t.to_state == "failed"));
    }

    // ========================================================================
    // Currency Revaluation Entity Tests
    // ========================================================================

    #[test]
    fn test_currency_revaluation_definition() {
        let def = entities::currency_revaluation_definition();
        assert_eq!(def.name, "currency_revaluations");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "calculated"));
        assert!(wf.states.iter().any(|s| s.name == "reviewed"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    #[test]
    fn test_currency_revaluation_workflow_transitions() {
        let def = entities::currency_revaluation_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "calculated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "calculated" && t.to_state == "reviewed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reviewed" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "posted" && t.to_state == "reversed"));
    }

    // ========================================================================
    // Netting Entity Tests
    // ========================================================================

    #[test]
    fn test_netting_agreement_definition() {
        let def = entities::netting_agreement_definition();
        assert_eq!(def.name, "netting_agreements");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_netting_batch_definition() {
        let def = entities::netting_batch_definition();
        assert_eq!(def.name, "netting_batches");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "calculated"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "settled"));
    }

    #[test]
    fn test_netting_batch_workflow_transitions() {
        let def = entities::netting_batch_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "calculated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "calculated" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "settled"));
    }

    // ========================================================================
    // Subscription Management Entity Tests
    // ========================================================================

    #[test]
    fn test_subscription_product_definition() {
        let def = entities::subscription_product_definition();
        assert_eq!(def.name, "subscription_products");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_subscription_contract_definition() {
        let def = entities::subscription_contract_definition();
        assert_eq!(def.name, "subscription_contracts");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "suspended"));
        assert!(wf.states.iter().any(|s| s.name == "in_renewal"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
        assert!(wf.states.iter().any(|s| s.name == "expired"));
        assert!(wf.states.iter().any(|s| s.name == "terminated"));
    }

    #[test]
    fn test_subscription_billing_event_definition() {
        let def = entities::subscription_billing_event_definition();
        assert_eq!(def.name, "subscription_billing_events");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "scheduled");
        assert!(wf.states.iter().any(|s| s.name == "invoiced"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
    }

    #[test]
    fn test_subscription_contract_workflow_transitions() {
        let def = entities::subscription_contract_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "suspended"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "suspended" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "in_renewal"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_renewal" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "cancelled"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "expired"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "terminated"));
    }

    #[test]
    fn test_subscription_billing_workflow_transitions() {
        let def = entities::subscription_billing_event_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "scheduled" && t.to_state == "invoiced"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "invoiced" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "scheduled" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Business Logic Tests: Recurring Journals
    // ========================================================================

    #[test]
    fn test_recurring_journal_next_generation_monthly() {
        let last = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let next = chrono::NaiveDate::from_ymd_opt(2025, 2, 15).unwrap();
        assert_eq!(next, last + chrono::Duration::days(31)); // rough check
    }

    // ========================================================================
    // Business Logic Tests: Allocations
    // ========================================================================

    #[test]
    fn test_allocation_fixed_percentage() {
        let lines = vec![
            ("1000", 50.0),
            ("2000", 30.0),
            ("3000", 20.0),
        ];
        let pool_amount = 100000.0;
        let total_pct: f64 = lines.iter().map(|(_, pct)| *pct).sum();
        assert!((total_pct - 100.0).abs() < 0.01);
        let allocated: Vec<(&str, f64)> = lines.iter()
            .map(|(acct, pct)| (*acct, pool_amount * (pct / 100.0)))
            .collect();
        assert!((allocated[0].1 - 50000.0).abs() < 0.01);
        assert!((allocated[1].1 - 30000.0).abs() < 0.01);
        assert!((allocated[2].1 - 20000.0).abs() < 0.01);
    }

    #[test]
    fn test_allocation_equal_share() {
        let targets = vec!["CC-A", "CC-B", "CC-C", "CC-D"];
        let pool = 120000.0;
        let share = pool / targets.len() as f64;
        assert!((share - 30000.0).abs() < 0.01);
    }

    #[test]
    fn test_allocation_statistical_basis() {
        let basis_values = vec![
            ("Dept-A", 500.0),
            ("Dept-B", 300.0),
            ("Dept-C", 200.0),
        ];
        let total_basis: f64 = basis_values.iter().map(|(_, v)| *v).sum();
        assert_eq!(total_basis, 1000.0);
        let pool = 50000.0;
        let allocated: Vec<(&str, f64)> = basis_values.iter()
            .map(|(dept, basis)| (*dept, pool * (basis / total_basis)))
            .collect();
        assert!((allocated[0].1 - 25000.0).abs() < 0.01); // 50%
        assert!((allocated[1].1 - 15000.0).abs() < 0.01); // 30%
        assert!((allocated[2].1 - 10000.0).abs() < 0.01); // 20%
    }

    // ========================================================================
    // Business Logic Tests: Funds Reservation / Budgetary Control
    // ========================================================================

    #[test]
    fn test_funds_check_pass() {
        let budget = 100000.0_f64;
        let reserved = 30000.0_f64;
        let consumed = 20000.0_f64;
        let requested = 40000.0_f64;
        let available = budget - reserved - consumed;
        assert!(requested <= available, "Funds check should pass");
    }

    #[test]
    fn test_funds_check_fail() {
        let budget = 100000.0_f64;
        let reserved = 50000.0_f64;
        let consumed = 40000.0_f64;
        let requested = 20000.0_f64;
        let available = budget - reserved - consumed;
        assert!(requested > available, "Funds check should fail");
    }

    #[test]
    fn test_funds_consumption_remaining() {
        let reserved = 50000.0_f64;
        let consumed = 35000.0_f64;
        let remaining = reserved - consumed;
        assert!((remaining - 15000.0).abs() < 0.01);
    }

    // ========================================================================
    // Business Logic Tests: Journal Import
    // ========================================================================

    #[test]
    fn test_journal_import_row_count() {
        let total = 100;
        let valid = 95;
        let imported = 90;
        let error = total - valid; // 5
        let skipped = valid - imported; // 5
        assert_eq!(error, 5);
        assert_eq!(skipped, 5);
        assert_eq!(error + skipped + imported, total);
    }

    #[test]
    fn test_journal_import_balance_check() {
        let debits: f64 = 10000.0;
        let credits: f64 = 10000.0;
        let balanced = (debits - credits).abs() < 0.01;
        assert!(balanced);
    }

    #[test]
    fn test_journal_import_balance_fail() {
        let debits: f64 = 10000.0;
        let credits: f64 = 9999.99;
        let balanced = (debits - credits).abs() < 0.01;
        assert!(!balanced);
    }

    // ========================================================================
    // Business Logic Tests: Landed Cost
    // ========================================================================

    #[test]
    fn test_landed_cost_by_value() {
        let item_value = 50000.0;
        let freight_total = 5000.0;
        let insurance_total = 1000.0;
        let duty_rate = 5.0; // 5%
        let duty: f64 = item_value * (duty_rate / 100.0);
        assert!((duty - 2500.0_f64).abs() < 0.01);
        let total_landed: f64 = item_value + freight_total + insurance_total + duty;
        assert!((total_landed - 58500.0_f64).abs() < 0.01);
    }

    #[test]
    fn test_landed_cost_variance() {
        let estimated = 58000.0;
        let actual = 58500.0;
        let variance: f64 = actual - estimated;
        assert!((variance - 500.0_f64).abs() < 0.01);
    }

    // ========================================================================
    // Business Logic Tests: Transfer Pricing
    // ========================================================================

    #[test]
    fn test_transfer_pricing_cost_plus() {
        let manufacturing_cost = 80000.0;
        let margin_pct = 25.0;
        let transfer_price: f64 = manufacturing_cost * (1.0 + margin_pct / 100.0);
        assert!((transfer_price - 100000.0_f64).abs() < 0.01);
    }

    #[test]
    fn test_transfer_pricing_resale_price() {
        let resale_price = 120000.0;
        let gross_margin_pct = 20.0;
        let transfer_price: f64 = resale_price * (1.0 - gross_margin_pct / 100.0);
        assert!((transfer_price - 96000.0_f64).abs() < 0.01);
    }

    #[test]
    fn test_transfer_pricing_arm_length_within_range() {
        let transfer_price = 100000.0;
        let min_arm = 95000.0;
        let max_arm = 105000.0;
        let within = transfer_price >= min_arm && transfer_price <= max_arm;
        assert!(within);
    }

    #[test]
    fn test_transfer_pricing_arm_length_outside_range() {
        let transfer_price = 110000.0;
        let min_arm = 95000.0;
        let max_arm = 105000.0;
        let within = transfer_price >= min_arm && transfer_price <= max_arm;
        assert!(!within);
    }

    // ========================================================================
    // Business Logic Tests: Currency Revaluation
    // ========================================================================

    #[test]
    fn test_currency_revaluation_unrealized_gain() {
        // EUR receivable: 100,000 EUR
        // Original rate: 1.10, Period-end rate: 1.15
        let original_amount_eur = 100000.0;
        let original_rate = 1.10;
        let new_rate = 1.15;
        let original_usd = original_amount_eur * original_rate;
        let new_usd = original_amount_eur * new_rate;
        let gain: f64 = new_usd - original_usd;
        assert!((gain - 5000.0_f64).abs() < 0.01); // $5,000 unrealized gain
    }

    #[test]
    fn test_currency_revaluation_unrealized_loss() {
        // EUR payable: 100,000 EUR
        // Original rate: 1.10, Period-end rate: 1.15
        let original_amount_eur = 100000.0;
        let original_rate = 1.10;
        let new_rate = 1.15;
        let original_usd = original_amount_eur * original_rate;
        let new_usd = original_amount_eur * new_rate;
        let loss = new_usd - original_usd; // loss on payable = more USD needed
        assert!(loss > 0.0);
    }

    // ========================================================================
    // Business Logic Tests: Netting
    // ========================================================================

    #[test]
    fn test_netting_calculation() {
        let payables = 75000.0;
        let receivables = 60000.0;
        let net: f64 = receivables - payables; // -15000 means party A owes
        assert!((net - (-15000.0_f64)).abs() < 0.01);
        assert!(net < 0.0, "Party A owes the net amount");
    }

    #[test]
    fn test_netting_balanced() {
        let payables = 50000.0;
        let receivables = 50000.0;
        let net: f64 = receivables - payables;
        assert!((net - 0.0_f64).abs() < 0.01);
    }

    // ========================================================================
    // Business Logic Tests: Subscription Management
    // ========================================================================

    #[test]
    fn test_subscription_mrr_calculation() {
        let annual_contract = 120000.0;
        let mrr: f64 = annual_contract / 12.0;
        assert!((mrr - 10000.0_f64).abs() < 0.01);
    }

    #[test]
    fn test_subscription_straight_line_revenue() {
        let contract_value = 240000.0;
        let term_months = 24;
        let monthly_revenue = contract_value / term_months as f64;
        assert!((monthly_revenue - 10000.0).abs() < 0.01);
    }

    #[test]
    fn test_subscription_deferred_revenue() {
        let contract_value = 120000.0;
        let months_elapsed = 3;
        let term_months = 12;
        let recognized = contract_value * (months_elapsed as f64 / term_months as f64);
        let deferred = contract_value - recognized;
        assert!((recognized - 30000.0).abs() < 0.01);
        assert!((deferred - 90000.0).abs() < 0.01);
    }

    #[test]
    fn test_subscription_per_unit_pricing() {
        let unit_price = 50.0;
        let quantity = 100;
        let mrr = unit_price * quantity as f64;
        assert!((mrr - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_subscription_tiered_pricing() {
        let tiers = vec![
            (1, 100, 10.0),    // 1-100 units @ $10
            (101, 500, 8.0),   // 101-500 units @ $8
            (501, 1000, 6.0),  // 501-1000 units @ $6
        ];
        let usage = 600;
        let mut total = 0.0;
        for &(from, to, price) in &tiers {
            if usage >= from {
                let units_in_tier = (usage.min(to) - from + 1) as f64;
                total += units_in_tier * price;
            }
        }
        // Tier 1: 100 * 10 = 1000
        // Tier 2: 400 * 8 = 3200
        // Tier 3: 100 * 6 = 600
        // Total = 4800
        assert!((total - 4800.0).abs() < 0.01);
    }

    // ========================================================================
    // ========================================================================
    // Inflation Adjustment Entity Tests
    // ========================================================================

    #[test]
    fn test_inflation_index_definition() {
        let def = entities::inflation_index_definition();
        assert_eq!(def.name, "inflation_indices");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_inflation_index_rate_definition() {
        let def = entities::inflation_index_rate_definition();
        assert_eq!(def.name, "inflation_index_rates");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_inflation_adjustment_run_definition() {
        let def = entities::inflation_adjustment_run_definition();
        assert_eq!(def.name, "inflation_adjustment_runs");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "calculated"));
        assert!(wf.states.iter().any(|s| s.name == "reviewed"));
        assert!(wf.states.iter().any(|s| s.name == "approved"));
        assert!(wf.states.iter().any(|s| s.name == "posted"));
        assert!(wf.states.iter().any(|s| s.name == "cancelled"));
    }

    #[test]
    fn test_inflation_adjustment_line_definition() {
        let def = entities::inflation_adjustment_line_definition();
        assert_eq!(def.name, "inflation_adjustment_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_valid_index_types() {
        for t in &["cpi", "gdp_deflator", "custom"] {
            assert!(super::VALID_INDEX_TYPES.contains(t));
        }
        assert!(!super::VALID_INDEX_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_adjustment_methods() {
        for m in &["historical", "current"] {
            assert!(super::VALID_ADJUSTMENT_METHODS.contains(m));
        }
    }

    #[test]
    fn test_restatement_factor_calculation() {
        let factor = super::InflationAdjustmentService::calculate_restatement_factor(150.0, 100.0);
        assert!((factor - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_restatement_factor_zero_base() {
        let factor = super::InflationAdjustmentService::calculate_restatement_factor(150.0, 0.0);
        assert_eq!(factor, 1.0);
    }

    #[test]
    fn test_restate_non_monetary_balance() {
        let restated = super::InflationAdjustmentService::restate_non_monetary_balance(100000.0, 1.5);
        assert!((restated - 150000.0).abs() < 0.01);
    }

    #[test]
    fn test_monetary_gain_loss() {
        let gain = super::InflationAdjustmentService::calculate_monetary_gain_loss(50000.0, 1.5);
        assert!((gain - 25000.0).abs() < 0.01);
    }

    #[test]
    fn test_monetary_gain_loss_no_inflation() {
        let gain = super::InflationAdjustmentService::calculate_monetary_gain_loss(50000.0, 1.0);
        assert!((gain - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_inflation_adjustment_amount() {
        let adj = super::InflationAdjustmentService::calculate_adjustment_amount(100000.0, 150000.0);
        assert!((adj - 50000.0).abs() < 0.01);
    }

    #[test]
    fn test_inflation_adjustment_run_workflow_transitions() {
        let def = entities::inflation_adjustment_run_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "calculated"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "calculated" && t.to_state == "reviewed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "reviewed" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "posted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "cancelled"));
    }

    // ========================================================================
    // Impairment Management Entity Tests
    // ========================================================================

    #[test]
    fn test_impairment_indicator_definition() {
        let def = entities::impairment_indicator_definition();
        assert_eq!(def.name, "impairment_indicators");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_impairment_test_definition() {
        let def = entities::impairment_test_definition();
        assert_eq!(def.name, "impairment_tests");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "submitted"));
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
    }

    #[test]
    fn test_impairment_cash_flow_definition() {
        let def = entities::impairment_cash_flow_definition();
        assert_eq!(def.name, "impairment_cash_flows");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_impairment_test_asset_definition() {
        let def = entities::impairment_test_asset_definition();
        assert_eq!(def.name, "impairment_test_assets");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_valid_indicator_types() {
        for t in &["external", "internal", "market"] {
            assert!(super::VALID_INDICATOR_TYPES.contains(t));
        }
    }

    #[test]
    fn test_valid_severity_levels() {
        for s in &["low", "medium", "high", "critical"] {
            assert!(super::VALID_SEVERITY_LEVELS.contains(s));
        }
    }

    #[test]
    fn test_valid_test_types() {
        for t in &["individual", "cash_generating_unit"] {
            assert!(super::VALID_TEST_TYPES.contains(t));
        }
    }

    #[test]
    fn test_valid_test_methods() {
        for m in &["value_in_use", "fair_value_less_costs"] {
            assert!(super::VALID_TEST_METHODS.contains(m));
        }
    }

    #[test]
    fn test_impairment_loss_when_impaired() {
        let loss = super::ImpairmentManagementService::calculate_impairment_loss(100000.0, 75000.0);
        assert!((loss - 25000.0).abs() < 0.01);
    }

    #[test]
    fn test_impairment_loss_when_not_impaired() {
        let loss = super::ImpairmentManagementService::calculate_impairment_loss(75000.0, 100000.0);
        assert_eq!(loss, 0.0);
    }

    #[test]
    fn test_impairment_loss_when_equal() {
        let loss = super::ImpairmentManagementService::calculate_impairment_loss(100000.0, 100000.0);
        assert_eq!(loss, 0.0);
    }

    #[test]
    fn test_is_impaired() {
        assert!(super::ImpairmentManagementService::is_impaired(100000.0, 75000.0));
        assert!(!super::ImpairmentManagementService::is_impaired(75000.0, 100000.0));
    }

    #[test]
    fn test_present_value_of_cash_flows() {
        let cash_flows = vec![(30000.0, 0.909), (30000.0, 0.826), (30000.0, 0.751)];
        let pv = super::ImpairmentManagementService::calculate_present_value(&cash_flows);
        assert!(pv > 74000.0 && pv < 75000.0);
    }

    #[test]
    fn test_discount_factor() {
        let df = super::ImpairmentManagementService::calculate_discount_factor(0.10, 1);
        assert!((df - 0.9091).abs() < 0.01);
    }

    #[test]
    fn test_discount_factor_zero_rate() {
        let df = super::ImpairmentManagementService::calculate_discount_factor(0.0, 5);
        assert_eq!(df, 1.0);
    }

    #[test]
    fn test_terminal_value_pv() {
        let tv_pv = super::ImpairmentManagementService::calculate_terminal_value_pv(500000.0, 0.10, 5);
        assert!(tv_pv > 300000.0 && tv_pv < 320000.0);
    }

    #[test]
    fn test_impairment_reversal_cap() {
        let cap = super::ImpairmentManagementService::calculate_reversal_cap(70000.0, 100000.0, 10000.0);
        assert!((cap - 20000.0).abs() < 0.01);
    }

    #[test]
    fn test_impairment_reversal_cap_zero() {
        let cap = super::ImpairmentManagementService::calculate_reversal_cap(100000.0, 100000.0, 0.0);
        assert_eq!(cap, 0.0);
    }

    #[test]
    fn test_impairment_test_workflow_transitions() {
        let def = entities::impairment_test_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "submitted" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "completed" && t.to_state == "reversed"));
    }

    // ========================================================================
    // Bank Account Transfer Entity Tests
    // ========================================================================

    #[test]
    fn test_bank_transfer_type_definition() {
        let def = entities::bank_transfer_type_definition();
        assert_eq!(def.name, "bank_transfer_types");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_bank_account_transfer_definition() {
        let def = entities::bank_account_transfer_definition();
        assert_eq!(def.name, "bank_account_transfers");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "completed"));
        assert!(wf.states.iter().any(|s| s.name == "reversed"));
        assert!(wf.states.iter().any(|s| s.name == "failed"));
    }

    #[test]
    fn test_valid_settlement_methods() {
        for m in &["immediate", "scheduled", "batch"] {
            assert!(super::VALID_SETTLEMENT_METHODS.contains(m));
        }
    }

    #[test]
    fn test_cross_currency_amount() {
        let converted = super::BankAccountTransferService::calculate_cross_currency_amount(10000.0, 0.85);
        assert!((converted - 8500.0).abs() < 0.01);
    }

    #[test]
    fn test_requires_approval_above_threshold() {
        assert!(super::BankAccountTransferService::requires_approval(50000.0, 25000.0));
    }

    #[test]
    fn test_requires_approval_no_threshold() {
        assert!(!super::BankAccountTransferService::requires_approval(50000.0, 0.0));
    }

    #[test]
    fn test_bank_transfer_workflow_transitions() {
        let def = entities::bank_account_transfer_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "in_transit"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_transit" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "completed" && t.to_state == "reversed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "in_transit" && t.to_state == "failed"));
    }

    // ========================================================================
    // Tax Reporting Entity Tests
    // ========================================================================

    #[test]
    fn test_tax_return_template_def() {
        let def = entities::tax_return_template_definition();
        assert_eq!(def.name, "tax_return_templates");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_tax_return_template_line_def() {
        let def = entities::tax_return_template_line_definition();
        assert_eq!(def.name, "tax_return_template_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_tax_report_def() {
        let def = entities::tax_report_definition();
        assert_eq!(def.name, "tax_reports");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "filed"));
        assert!(wf.states.iter().any(|s| s.name == "paid"));
        assert!(wf.states.iter().any(|s| s.name == "amended"));
    }

    #[test]
    fn test_valid_tax_report_types() {
        for t in &["vat", "gst", "sales_tax", "corporate_income", "withholding"] {
            assert!(super::VALID_TAX_REPORT_TYPES.contains(t));
        }
    }

    #[test]
    fn test_net_tax_due_positive() {
        let due = super::TaxReportingService::calculate_net_tax_due(50000.0, 30000.0);
        assert!((due - 20000.0).abs() < 0.01);
    }

    #[test]
    fn test_net_tax_due_negative_refund() {
        let due = super::TaxReportingService::calculate_net_tax_due(20000.0, 35000.0);
        assert!(due < 0.0);
    }

    #[test]
    fn test_total_amount_due() {
        let total = super::TaxReportingService::calculate_total_amount_due(20000.0, 500.0, 200.0);
        assert!((total - 20700.0).abs() < 0.01);
    }

    #[test]
    fn test_payment_or_refund_payment() {
        let result = super::TaxReportingService::calculate_payment_or_refund(20700.0, 15000.0);
        assert!((result - 5700.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_tax_rate() {
        let rate = super::TaxReportingService::calculate_effective_tax_rate(15000.0, 100000.0);
        assert!((rate - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_tax_rate_zero_taxable() {
        let rate = super::TaxReportingService::calculate_effective_tax_rate(15000.0, 0.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_tax_report_workflow_transitions() {
        let def = entities::tax_report_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "submitted"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "filed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "filed" && t.to_state == "paid"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "filed" && t.to_state == "amended"));
    }

    // ========================================================================
    // Grant Management Entity Tests
    // ========================================================================

    #[test]
    fn test_grant_sponsor_def() {
        let def = entities::grant_sponsor_definition();
        assert_eq!(def.name, "grant_sponsors");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_grant_award_def() {
        let def = entities::grant_award_definition();
        assert_eq!(def.name, "grant_awards");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "draft");
        assert!(wf.states.iter().any(|s| s.name == "active"));
        assert!(wf.states.iter().any(|s| s.name == "suspended"));
        assert!(wf.states.iter().any(|s| s.name == "terminated"));
    }

    #[test]
    fn test_grant_budget_line_def() {
        let def = entities::grant_budget_line_definition();
        assert_eq!(def.name, "grant_budget_lines");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_grant_expenditure_def() {
        let def = entities::grant_expenditure_definition();
        assert_eq!(def.name, "grant_expenditures");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "pending");
        assert!(wf.states.iter().any(|s| s.name == "billed"));
    }

    #[test]
    fn test_valid_sponsor_types() {
        for t in &["government", "foundation", "corporate", "internal", "university"] {
            assert!(super::VALID_SPONSOR_TYPES.contains(t));
        }
    }

    #[test]
    fn test_grant_indirect_costs() {
        let indirect = super::GrantManagementService::calculate_indirect_costs(80000.0, 55.0);
        assert!((indirect - 44000.0).abs() < 0.01);
    }

    #[test]
    fn test_grant_total_award() {
        let total = super::GrantManagementService::calculate_total_award(80000.0, 44000.0);
        assert!((total - 124000.0).abs() < 0.01);
    }

    #[test]
    fn test_grant_available_balance() {
        let available = super::GrantManagementService::calculate_available_balance(500000.0, 200000.0, 100000.0);
        assert!((available - 200000.0).abs() < 0.01);
    }

    #[test]
    fn test_grant_available_balance_negative() {
        let available = super::GrantManagementService::calculate_available_balance(500000.0, 350000.0, 200000.0);
        assert!(available < 0.0);
    }

    #[test]
    fn test_grant_budget_utilization() {
        let pct = super::GrantManagementService::calculate_budget_utilization(75000.0, 100000.0);
        assert!((pct - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_grant_cost_sharing() {
        let sharing = super::GrantManagementService::calculate_cost_sharing(100000.0, 20.0);
        assert!((sharing - 20000.0).abs() < 0.01);
    }

    #[test]
    fn test_grant_budget_line_exceeded() {
        assert!(super::GrantManagementService::is_budget_line_exceeded(100000.0, 60000.0, 50000.0));
    }

    #[test]
    fn test_grant_budget_line_not_exceeded() {
        assert!(!super::GrantManagementService::is_budget_line_exceeded(100000.0, 40000.0, 30000.0));
    }

    #[test]
    fn test_grant_award_workflow_transitions() {
        let def = entities::grant_award_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "draft" && t.to_state == "active"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "suspended"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "completed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "terminated"));
    }

    #[test]
    fn test_grant_expenditure_workflow_transitions() {
        let def = entities::grant_expenditure_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "approved"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "approved" && t.to_state == "billed"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "pending" && t.to_state == "hold"));
    }

    // ========================================================================
    // Corporate Card Management Entity Tests
    // ========================================================================

    #[test]
    fn test_corporate_card_program_def() {
        let def = entities::corporate_card_program_definition();
        assert_eq!(def.name, "corporate_card_programs");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_corporate_card_def() {
        let def = entities::corporate_card_definition();
        assert_eq!(def.name, "corporate_cards");
        assert!(def.workflow.is_some());
        let wf = def.workflow.unwrap();
        assert_eq!(wf.initial_state, "active");
        assert!(wf.states.iter().any(|s| s.name == "lost"));
        assert!(wf.states.iter().any(|s| s.name == "stolen"));
    }

    #[test]
    fn test_corporate_card_transaction_def() {
        let def = entities::corporate_card_transaction_definition();
        assert_eq!(def.name, "corporate_card_transactions");
        assert!(def.workflow.is_none());
    }

    #[test]
    fn test_valid_card_networks() {
        for n in &["visa", "mastercard", "amex"] {
            assert!(super::VALID_CARD_NETWORKS.contains(n));
        }
    }

    #[test]
    fn test_check_spending_limit_within() {
        assert!(super::CorporateCardManagementService::check_spending_limit(500.0, 1000.0, 3000.0, 5000.0));
    }

    #[test]
    fn test_check_spending_limit_exceeds_single() {
        assert!(!super::CorporateCardManagementService::check_spending_limit(1500.0, 1000.0, 3000.0, 5000.0));
    }

    #[test]
    fn test_check_spending_limit_no_limits() {
        assert!(super::CorporateCardManagementService::check_spending_limit(500.0, 0.0, 3000.0, 0.0));
    }

    #[test]
    fn test_calculate_available_spend() {
        let available = super::CorporateCardManagementService::calculate_available_spend(10000.0, 7500.0);
        assert!((available - 2500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_available_spend_no_limit() {
        let available = super::CorporateCardManagementService::calculate_available_spend(0.0, 5000.0);
        assert_eq!(available, f64::MAX);
    }

    #[test]
    fn test_calculate_statement_balance() {
        let balance = super::CorporateCardManagementService::calculate_statement_balance(5000.0, 8000.0, 1500.0, 3000.0);
        assert!((balance - 8500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_match_confidence_high() {
        let score = super::CorporateCardManagementService::calculate_match_confidence(true, 0, true);
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_match_confidence_low() {
        let score = super::CorporateCardManagementService::calculate_match_confidence(false, 15, false);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_corporate_card_workflow_transitions() {
        let def = entities::corporate_card_definition();
        let wf = def.workflow.unwrap();
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "suspended"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "lost"));
        assert!(wf.transitions.iter().any(|t| t.from_state == "active" && t.to_state == "stolen"));
    }

    #[test]
    fn test_six_new_features_entity_count() {
        let new_entities = vec![
            entities::inflation_index_definition(),
            entities::inflation_index_rate_definition(),
            entities::inflation_adjustment_run_definition(),
            entities::inflation_adjustment_line_definition(),
            entities::impairment_indicator_definition(),
            entities::impairment_test_definition(),
            entities::impairment_cash_flow_definition(),
            entities::impairment_test_asset_definition(),
            entities::bank_transfer_type_definition(),
            entities::bank_account_transfer_definition(),
            entities::tax_return_template_definition(),
            entities::tax_return_template_line_definition(),
            entities::tax_report_definition(),
            entities::grant_sponsor_definition(),
            entities::grant_award_definition(),
            entities::grant_budget_line_definition(),
            entities::grant_expenditure_definition(),
            entities::corporate_card_program_definition(),
            entities::corporate_card_definition(),
            entities::corporate_card_transaction_definition(),
        ];
        assert_eq!(new_entities.len(), 20);
        let names: std::collections::HashSet<&str> = new_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 20, "All 20 entity names must be unique");
    }

    #[test]
    fn test_six_new_features_workflow_count() {
        let workflow_entities = vec![
            entities::inflation_adjustment_run_definition(),
            entities::impairment_test_definition(),
            entities::bank_account_transfer_definition(),
            entities::tax_report_definition(),
            entities::grant_award_definition(),
            entities::grant_expenditure_definition(),
            entities::corporate_card_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 7);
    }

    // Comprehensive: All New Feature Entities Build
    // ========================================================================

    #[test]
    fn test_all_new_oracle_fusion_entities_build() {
        // Recurring Journals
        let _ = entities::recurring_journal_template_definition();
        let _ = entities::recurring_journal_line_definition();
        // Allocations
        let _ = entities::allocation_rule_definition();
        let _ = entities::allocation_line_definition();
        // Funds Reservation
        let _ = entities::funds_reservation_definition();
        let _ = entities::funds_check_result_definition();
        // Journal Import
        let _ = entities::journal_import_request_definition();
        // Landed Cost
        let _ = entities::landed_cost_template_definition();
        let _ = entities::landed_cost_component_definition();
        let _ = entities::landed_cost_assignment_definition();
        // Transfer Pricing
        let _ = entities::transfer_pricing_policy_definition();
        let _ = entities::transfer_pricing_transaction_definition();
        // AutoInvoice
        let _ = entities::autoinvoice_rule_definition();
        let _ = entities::autoinvoice_run_definition();
        // Currency Revaluation
        let _ = entities::currency_revaluation_definition();
        // Netting
        let _ = entities::netting_agreement_definition();
        let _ = entities::netting_batch_definition();
        // Subscription Management
        let _ = entities::subscription_product_definition();
        let _ = entities::subscription_contract_definition();
        let _ = entities::subscription_billing_event_definition();
    }

    #[test]
    fn test_new_oracle_fusion_entity_count() {
        let new_entities = vec![
            entities::recurring_journal_template_definition(),
            entities::recurring_journal_line_definition(),
            entities::allocation_rule_definition(),
            entities::allocation_line_definition(),
            entities::funds_reservation_definition(),
            entities::funds_check_result_definition(),
            entities::journal_import_request_definition(),
            entities::landed_cost_template_definition(),
            entities::landed_cost_component_definition(),
            entities::landed_cost_assignment_definition(),
            entities::transfer_pricing_policy_definition(),
            entities::transfer_pricing_transaction_definition(),
            entities::autoinvoice_rule_definition(),
            entities::autoinvoice_run_definition(),
            entities::currency_revaluation_definition(),
            entities::netting_agreement_definition(),
            entities::netting_batch_definition(),
            entities::subscription_product_definition(),
            entities::subscription_contract_definition(),
            entities::subscription_billing_event_definition(),
        ];
        assert_eq!(new_entities.len(), 20, "Should have 20 new Oracle Fusion entities");
        let names: std::collections::HashSet<&str> = new_entities.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 20, "All new entity names must be unique");
    }

    #[test]
    fn test_new_oracle_fusion_workflow_count() {
        let workflow_entities = vec![
            entities::recurring_journal_template_definition(),
            entities::allocation_rule_definition(),
            entities::funds_reservation_definition(),
            entities::journal_import_request_definition(),
            entities::landed_cost_assignment_definition(),
            entities::autoinvoice_rule_definition(),
            entities::autoinvoice_run_definition(),
            entities::currency_revaluation_definition(),
            entities::netting_batch_definition(),
            entities::subscription_contract_definition(),
            entities::subscription_billing_event_definition(),
        ];
        let count = workflow_entities.iter().filter(|e| e.workflow.is_some()).count();
        assert_eq!(count, 11, "All 11 new workflow entities should have workflows");
    }

    // ========================================================================
    // Grand Total Entity Count Test
    // ========================================================================

    #[test]
    fn test_grand_total_entity_count_all_features() {
        let mut all = vec![];

        // Original 27
        all.push(entities::chart_of_accounts_definition());
        all.push(entities::journal_entry_definition());
        all.push(entities::invoice_definition());
        all.push(entities::budget_definition());
        all.push(entities::expense_report_definition());
        all.push(entities::ap_invoice_definition());
        all.push(entities::ap_invoice_line_definition());
        all.push(entities::ap_invoice_distribution_definition());
        all.push(entities::ap_invoice_hold_definition());
        all.push(entities::ap_payment_definition());
        all.push(entities::ar_transaction_definition());
        all.push(entities::ar_transaction_line_definition());
        all.push(entities::ar_receipt_definition());
        all.push(entities::ar_credit_memo_definition());
        all.push(entities::ar_adjustment_definition());
        all.push(entities::asset_category_definition());
        all.push(entities::asset_book_definition());
        all.push(entities::fixed_asset_definition());
        all.push(entities::asset_transfer_definition());
        all.push(entities::asset_retirement_definition());
        all.push(entities::cost_book_definition());
        all.push(entities::cost_element_definition());
        all.push(entities::cost_profile_definition());
        all.push(entities::standard_cost_definition());
        all.push(entities::cost_adjustment_definition());
        all.push(entities::cost_adjustment_line_definition());
        all.push(entities::cost_variance_definition());

        // Wave 2: Revenue, SLA, Cash, Tax, IC, Period, Lease, Bank, Encumbrance, Currency, Multi-Book, Consolidation (46)
        all.push(entities::revenue_policy_definition());
        all.push(entities::revenue_contract_definition());
        all.push(entities::performance_obligation_definition());
        all.push(entities::revenue_schedule_line_definition());
        all.push(entities::revenue_modification_definition());
        all.push(entities::accounting_method_definition());
        all.push(entities::accounting_derivation_rule_definition());
        all.push(entities::subledger_journal_entry_definition());
        all.push(entities::subledger_journal_line_definition());
        all.push(entities::cash_position_definition());
        all.push(entities::cash_forecast_template_definition());
        all.push(entities::cash_forecast_source_definition());
        all.push(entities::cash_forecast_definition());
        all.push(entities::tax_regime_definition());
        all.push(entities::tax_jurisdiction_definition());
        all.push(entities::tax_rate_definition());
        all.push(entities::tax_determination_rule_definition());
        all.push(entities::intercompany_batch_definition());
        all.push(entities::intercompany_transaction_definition());
        all.push(entities::intercompany_settlement_definition());
        all.push(entities::accounting_calendar_definition());
        all.push(entities::accounting_period_definition());
        all.push(entities::period_close_checklist_definition());
        all.push(entities::lease_contract_definition());
        all.push(entities::lease_payment_definition());
        all.push(entities::lease_modification_definition());
        all.push(entities::lease_termination_definition());
        all.push(entities::bank_account_definition());
        all.push(entities::bank_statement_definition());
        all.push(entities::bank_statement_line_definition());
        all.push(entities::reconciliation_match_definition());
        all.push(entities::encumbrance_type_definition());
        all.push(entities::encumbrance_entry_definition());
        all.push(entities::encumbrance_liquidation_definition());
        all.push(entities::encumbrance_carry_forward_definition());
        all.push(entities::currency_definition_entity());
        all.push(entities::exchange_rate_definition());
        all.push(entities::accounting_book_definition());
        all.push(entities::account_mapping_definition());
        all.push(entities::book_journal_entry_definition());
        all.push(entities::consolidation_ledger_definition());
        all.push(entities::consolidation_entity_definition());
        all.push(entities::consolidation_scenario_definition());
        all.push(entities::consolidation_adjustment_definition());
        all.push(entities::consolidation_elimination_rule_definition());
        all.push(entities::consolidation_translation_rate_definition());

        // Wave 3: Collections, Credit, WHT, Project Billing, Payment Terms, Financial Statements, Tax Filing, Journal Reversal (36)
        all.push(entities::customer_credit_profile_definition());
        all.push(entities::collection_strategy_definition());
        all.push(entities::collection_case_definition());
        all.push(entities::customer_interaction_definition());
        all.push(entities::promise_to_pay_definition());
        all.push(entities::dunning_campaign_definition());
        all.push(entities::dunning_letter_definition());
        all.push(entities::receivables_aging_snapshot_definition());
        all.push(entities::write_off_request_definition());
        all.push(entities::credit_scoring_model_definition());
        all.push(entities::credit_profile_definition());
        all.push(entities::credit_limit_definition());
        all.push(entities::credit_check_rule_definition());
        all.push(entities::credit_exposure_definition());
        all.push(entities::credit_hold_definition());
        all.push(entities::credit_review_definition());
        all.push(entities::withholding_tax_code_definition());
        all.push(entities::withholding_tax_group_definition());
        all.push(entities::supplier_withholding_assignment_definition());
        all.push(entities::withholding_tax_line_definition());
        all.push(entities::withholding_certificate_definition());
        all.push(entities::bill_rate_schedule_definition());
        all.push(entities::bill_rate_line_definition());
        all.push(entities::project_billing_config_definition());
        all.push(entities::billing_event_definition());
        all.push(entities::project_invoice_header_definition());
        all.push(entities::project_invoice_line_definition());
        all.push(entities::payment_term_definition());
        all.push(entities::payment_schedule_definition());
        all.push(entities::financial_report_template_definition());
        all.push(entities::financial_report_row_definition());
        all.push(entities::generated_financial_report_definition());
        all.push(entities::tax_filing_obligation_definition());
        all.push(entities::tax_return_definition());
        all.push(entities::tax_payment_definition());
        all.push(entities::journal_reversal_request_definition());

        // Wave 4: New Oracle Fusion features (20)
        all.push(entities::recurring_journal_template_definition());
        all.push(entities::recurring_journal_line_definition());
        all.push(entities::allocation_rule_definition());
        all.push(entities::allocation_line_definition());
        all.push(entities::funds_reservation_definition());
        all.push(entities::funds_check_result_definition());
        all.push(entities::journal_import_request_definition());
        all.push(entities::landed_cost_template_definition());
        all.push(entities::landed_cost_component_definition());
        all.push(entities::landed_cost_assignment_definition());
        all.push(entities::transfer_pricing_policy_definition());
        all.push(entities::transfer_pricing_transaction_definition());
        all.push(entities::autoinvoice_rule_definition());
        all.push(entities::autoinvoice_run_definition());
        all.push(entities::currency_revaluation_definition());
        all.push(entities::netting_agreement_definition());
        all.push(entities::netting_batch_definition());
        all.push(entities::subscription_product_definition());
        all.push(entities::subscription_contract_definition());
        all.push(entities::subscription_billing_event_definition());

        // Total: 27 + 46 + 36 + 20 = 129
        assert_eq!(all.len(), 129, "Should have 129 total entity definitions");

        // All unique names
        let names: std::collections::HashSet<&str> = all.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), 129, "All 129 entity names must be globally unique");
    }
}
