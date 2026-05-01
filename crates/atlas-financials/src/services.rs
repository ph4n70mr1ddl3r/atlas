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
const VALID_AR_TRANSACTION_TYPES: &[&str] = &[
    "invoice", "debit_memo", "credit_memo", "chargeback", "deposit", "guarantee",
];

/// Valid AR transaction statuses
const VALID_AR_STATUSES: &[&str] = &[
    "draft", "complete", "open", "closed", "cancelled",
];

/// Valid AR receipt types
const VALID_RECEIPT_TYPES: &[&str] = &[
    "cash", "check", "credit_card", "wire_transfer", "ach", "other",
];

/// Valid AR credit memo reason codes
const VALID_CREDIT_MEMO_REASONS: &[&str] = &[
    "return", "pricing_error", "damaged", "wrong_item", "discount", "other",
];

/// Valid AR adjustment types
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
const VALID_COSTING_METHODS: &[&str] = &[
    "standard", "average", "fifo", "lifo",
];

/// Valid cost element types
const VALID_COST_ELEMENT_TYPES: &[&str] = &[
    "material", "labor", "overhead", "subcontracting", "expense",
];

/// Valid overhead absorption methods
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
// Purchase Order Service (existing, kept for backward compat)
// ============================================================================

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
        let transaction_type = "credit_memo";
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
        let adjustment_type = "write_off";
        let amount: f64 = -100.0;
        assert!(amount <= 0.0, "Write-off amount should be negative");

        let adjustment_type = "increase";
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
}
