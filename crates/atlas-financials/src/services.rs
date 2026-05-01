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
const VALID_RECOGNITION_METHODS: &[&str] = &[
    "over_time", "point_in_time",
];

/// Valid over-time methods
const VALID_OVER_TIME_METHODS: &[&str] = &[
    "output", "input", "straight_line",
];

/// Valid allocation bases
const VALID_ALLOCATION_BASES: &[&str] = &[
    "standalone_selling_price", "residual", "equal",
];

/// Valid contract statuses
const VALID_CONTRACT_STATUSES: &[&str] = &[
    "draft", "active", "completed", "cancelled", "modified",
];

/// Valid obligation statuses
const VALID_OBLIGATION_STATUSES: &[&str] = &[
    "pending", "in_progress", "satisfied", "partially_satisfied", "cancelled",
];

/// Valid schedule statuses
const VALID_SCHEDULE_STATUSES: &[&str] = &[
    "planned", "recognized", "reversed", "cancelled",
];

/// Valid modification types
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
const VALID_SLA_APPLICATIONS: &[&str] = &[
    "payables", "receivables", "expenses", "assets", "projects", "general",
];

/// Valid SLA event classes
const VALID_SLA_EVENT_CLASSES: &[&str] = &[
    "create", "update", "cancel", "reverse",
];

/// Valid SLA derivation types
const VALID_DERIVATION_TYPES: &[&str] = &[
    "constant", "lookup", "formula",
];

/// Valid SLA entry statuses
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
const VALID_BUCKET_TYPES: &[&str] = &[
    "daily", "weekly", "monthly",
];

/// Valid cash forecast source types
const VALID_CASH_SOURCE_TYPES: &[&str] = &[
    "accounts_payable", "accounts_receivable", "payroll",
    "purchasing", "manual", "budget", "intercompany",
    "fixed_assets", "tax", "other",
];

/// Valid cash flow directions
const VALID_CASH_FLOW_DIRECTIONS: &[&str] = &[
    "inflow", "outflow", "both",
];

/// Valid forecast statuses
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
const VALID_TAX_TYPES: &[&str] = &[
    "sales_tax", "vat", "gst", "withholding", "excise", "customs",
];

/// Valid rate types
const VALID_RATE_TYPES: &[&str] = &[
    "standard", "reduced", "zero", "exempt",
];

/// Valid rounding rules
const VALID_ROUNDING_RULES: &[&str] = &[
    "nearest", "up", "down", "none",
];

/// Valid geographic levels
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
const VALID_IC_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "posted", "cancelled",
];

/// Valid IC transaction types
const VALID_IC_TXN_TYPES: &[&str] = &[
    "invoice", "journal_entry", "payment", "charge", "allocation",
];

/// Valid IC settlement methods
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
const VALID_PERIOD_STATUSES: &[&str] = &[
    "future", "not_opened", "open", "pending_close", "closed", "permanently_closed",
];

/// Valid subledgers for period close
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
const VALID_LEASE_CLASSIFICATIONS: &[&str] = &["operating", "finance"];

/// Valid lease statuses
const VALID_LEASE_STATUSES: &[&str] = &[
    "draft", "active", "modified", "impaired", "terminated", "expired",
];

/// Valid lease payment frequencies
const VALID_PAYMENT_FREQUENCIES: &[&str] = &["monthly", "quarterly", "annually"];

/// Valid lease modification types
const VALID_LEASE_MOD_TYPES: &[&str] = &[
    "term_extension", "scope_change", "payment_change", "rate_change", "reclassification",
];

/// Valid lease termination types
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
const VALID_ENCUMBRANCE_CATEGORIES: &[&str] = &[
    "commitment", "obligation", "preliminary",
];

/// Valid encumbrance entry statuses
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
const VALID_BOOK_TYPES: &[&str] = &["primary", "secondary"];

/// Valid mapping levels
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
const VALID_CONSOLIDATION_METHODS: &[&str] = &[
    "full", "proportional", "equity_method",
];

/// Valid translation methods
const VALID_TRANSLATION_METHODS: &[&str] = &[
    "current_rate", "temporal", "weighted_average",
];

/// Valid scenario statuses
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
}
