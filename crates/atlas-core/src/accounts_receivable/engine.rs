//! Accounts Receivable Engine
//!
//! Manages the full AR lifecycle:
//! - Customer transactions (invoices, debit memos, credit memos, chargebacks)
//! - Receipt processing (apply customer payments)
//! - Credit memo management (issue and apply credits)
//! - Balance adjustments (write-offs, increases, decreases)
//! - AR aging analysis
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Receivables

use atlas_shared::{
    ArTransaction, ArTransactionLine, ArReceipt, ArCreditMemo, ArAdjustment,
    ArAgingSummary, ArAgingByCustomer,
    AtlasError, AtlasResult,
};
use super::AccountsReceivableRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid transaction types
const VALID_TRANSACTION_TYPES: &[&str] = &[
    "invoice", "debit_memo", "credit_memo", "chargeback", "deposit", "guarantee",
];

/// Valid transaction statuses
const VALID_TRANSACTION_STATUSES: &[&str] = &[
    "draft", "complete", "open", "closed", "cancelled",
];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &[
    "line", "tax", "freight", "charges",
];

/// Valid receipt types
const VALID_RECEIPT_TYPES: &[&str] = &[
    "cash", "check", "credit_card", "wire_transfer", "ach", "other",
];

/// Valid receipt methods
const VALID_RECEIPT_METHODS: &[&str] = &[
    "automatic_receipt", "manual_receipt", "quick_cash", "miscellaneous",
];

/// Valid receipt statuses
const VALID_RECEIPT_STATUSES: &[&str] = &[
    "draft", "confirmed", "applied", "deposited", "reversed",
];

/// Valid credit memo reason codes
const VALID_CREDIT_MEMO_REASONS: &[&str] = &[
    "return", "pricing_error", "damaged", "wrong_item", "discount", "other",
];

/// Valid credit memo statuses
const VALID_CREDIT_MEMO_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "applied", "cancelled",
];

/// Valid adjustment types
const VALID_ADJUSTMENT_TYPES: &[&str] = &[
    "write_off", "write_off_bad_debt", "small_balance_write_off",
    "increase", "decrease", "transfer", "revaluation",
];

/// Valid adjustment statuses
const VALID_ADJUSTMENT_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "posted",
];

/// Accounts Receivable Engine
pub struct AccountsReceivableEngine {
    repository: Arc<dyn AccountsReceivableRepository>,
}

impl AccountsReceivableEngine {
    pub fn new(repository: Arc<dyn AccountsReceivableRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Transaction Management
    // ========================================================================

    /// Create a new AR transaction (customer invoice, debit memo, etc.)
    pub async fn create_transaction(
        &self,
        org_id: Uuid,
        transaction_type: &str,
        transaction_date: chrono::NaiveDate,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        currency_code: &str,
        entered_amount: &str,
        tax_amount: &str,
        payment_terms: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        purchase_order: Option<&str>,
        sales_rep: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArTransaction> {
        if !VALID_TRANSACTION_TYPES.contains(&transaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction_type '{}'. Must be one of: {}",
                transaction_type, VALID_TRANSACTION_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let entered: f64 = entered_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_amount must be a valid number".to_string(),
        ))?;
        let tax: f64 = tax_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "tax_amount must be a valid number".to_string(),
        ))?;
        if entered < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "entered_amount must be non-negative".to_string(),
            ));
        }

        let total = entered + tax;
        let transaction_number = format!("AR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating AR transaction {} type {} for customer {} amount {}",
            transaction_number, transaction_type, customer_id, total);

        self.repository.create_transaction(
            org_id, &transaction_number, transaction_type, transaction_date,
            customer_id, customer_number, customer_name,
            currency_code, entered_amount, tax_amount, &format!("{:.2}", total),
            payment_terms, due_date, gl_date,
            reference_number, purchase_order, sales_rep, notes,
            created_by,
        ).await
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<ArTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// Get a transaction by number
    pub async fn get_transaction_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ArTransaction>> {
        self.repository.get_transaction_by_number(org_id, number).await
    }

    /// List transactions with optional filters
    pub async fn list_transactions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        transaction_type: Option<&str>,
    ) -> AtlasResult<Vec<ArTransaction>> {
        if let Some(s) = status {
            if !VALID_TRANSACTION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TRANSACTION_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = transaction_type {
            if !VALID_TRANSACTION_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid transaction_type '{}'. Must be one of: {}", t, VALID_TRANSACTION_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_transactions(org_id, status, customer_id, transaction_type).await
    }

    /// Complete a draft transaction (marks it ready for posting)
    pub async fn complete_transaction(&self, transaction_id: Uuid) -> AtlasResult<ArTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete transaction in '{}' status. Must be 'draft'.", txn.status)
            ));
        }

        info!("Completing AR transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(transaction_id, "complete", None, None).await
    }

    /// Post a completed transaction (opens it for receipt application)
    pub async fn post_transaction(&self, transaction_id: Uuid) -> AtlasResult<ArTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status != "complete" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post transaction in '{}' status. Must be 'complete'.", txn.status)
            ));
        }

        info!("Posting AR transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(transaction_id, "open", None, None).await
    }

    /// Cancel a transaction
    pub async fn cancel_transaction(&self, transaction_id: Uuid, reason: Option<&str>) -> AtlasResult<ArTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status == "closed" || txn.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel transaction in '{}' status", txn.status)
            ));
        }

        info!("Cancelling AR transaction {}", txn.transaction_number);
        self.repository.update_transaction_status(transaction_id, "cancelled", None, reason).await
    }

    // ========================================================================
    // Transaction Lines
    // ========================================================================

    /// Add a line to a transaction
    pub async fn create_transaction_line(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        line_type: &str,
        description: Option<&str>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        unit_of_measure: Option<&str>,
        quantity: Option<&str>,
        unit_price: Option<&str>,
        line_amount: &str,
        tax_amount: &str,
        tax_code: Option<&str>,
        revenue_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArTransactionLine> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status == "cancelled" || txn.status == "closed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to transaction in '{}' status", txn.status)
            ));
        }

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let amount: f64 = line_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "line_amount must be a valid number".to_string(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "line_amount must be non-negative".to_string(),
            ));
        }

        let lines = self.repository.list_transaction_lines(transaction_id).await?;
        let line_number = (lines.len() as i32) + 1;

        info!("Adding line {} to AR transaction {}", line_number, txn.transaction_number);

        let line = self.repository.create_transaction_line(
            org_id, transaction_id, line_number,
            line_type, description, item_code, item_description,
            unit_of_measure, quantity, unit_price,
            line_amount, tax_amount, tax_code, revenue_account,
            created_by,
        ).await?;

        // Update transaction totals
        let all_lines = self.repository.list_transaction_lines(transaction_id).await?;
        let total_line_amount: f64 = all_lines.iter()
            .map(|l| l.line_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_tax: f64 = all_lines.iter()
            .map(|l| l.tax_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total = total_line_amount + total_tax;

        self.repository.update_transaction_totals(
            transaction_id,
            &format!("{:.2}", total_line_amount),
            &format!("{:.2}", total_tax),
            &format!("{:.2}", total),
            &format!("{:.2}", total),
        ).await?;

        Ok(line)
    }

    /// List transaction lines
    pub async fn list_transaction_lines(&self, transaction_id: Uuid) -> AtlasResult<Vec<ArTransactionLine>> {
        self.repository.list_transaction_lines(transaction_id).await
    }

    // ========================================================================
    // Receipt Management
    // ========================================================================

    /// Create a new receipt
    pub async fn create_receipt(
        &self,
        org_id: Uuid,
        receipt_date: chrono::NaiveDate,
        receipt_type: &str,
        receipt_method: &str,
        amount: &str,
        currency_code: &str,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        reference_number: Option<&str>,
        bank_account_name: Option<&str>,
        check_number: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArReceipt> {
        if !VALID_RECEIPT_TYPES.contains(&receipt_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt_type '{}'. Must be one of: {}",
                receipt_type, VALID_RECEIPT_TYPES.join(", ")
            )));
        }
        if !VALID_RECEIPT_METHODS.contains(&receipt_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt_method '{}'. Must be one of: {}",
                receipt_method, VALID_RECEIPT_METHODS.join(", ")
            )));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "amount must be positive".to_string(),
            ));
        }

        let receipt_number = format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating receipt {} for {}", receipt_number, amt);

        self.repository.create_receipt(
            org_id, &receipt_number, receipt_date, receipt_type, receipt_method,
            amount, currency_code, customer_id, customer_number, customer_name,
            reference_number, bank_account_name, check_number, notes, created_by,
        ).await
    }

    /// Get a receipt by ID
    pub async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ArReceipt>> {
        self.repository.get_receipt(id).await
    }

    /// List receipts
    pub async fn list_receipts(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ArReceipt>> {
        if let Some(s) = status {
            if !VALID_RECEIPT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid receipt status '{}'. Must be one of: {}", s, VALID_RECEIPT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_receipts(org_id, status, customer_id).await
    }

    /// Confirm a draft receipt
    pub async fn confirm_receipt(&self, receipt_id: Uuid) -> AtlasResult<ArReceipt> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot confirm receipt in '{}' status. Must be 'draft'.", receipt.status)
            ));
        }

        info!("Confirming receipt {}", receipt.receipt_number);
        self.repository.update_receipt_status(receipt_id, "confirmed").await
    }

    /// Apply a confirmed receipt to a transaction
    pub async fn apply_receipt(
        &self,
        receipt_id: Uuid,
        transaction_id: Uuid,
    ) -> AtlasResult<ArReceipt> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "confirmed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply receipt in '{}' status. Must be 'confirmed'.", receipt.status)
            ));
        }

        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status != "open" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply receipt to transaction in '{}' status. Must be 'open'.", txn.status)
            ));
        }

        let receipt_amount: f64 = receipt.amount.parse().unwrap_or(0.0);
        let remaining: f64 = txn.amount_due_remaining.parse().unwrap_or(0.0);
        let applied_amount = receipt_amount.min(remaining);
        let new_remaining = remaining - applied_amount;
        let new_applied: f64 = txn.amount_applied.parse().unwrap_or(0.0) + applied_amount;
        let new_status = if new_remaining < 0.01 { "closed" } else { "open" };

        info!("Applying receipt {} ({}) to transaction {} (remaining: {})",
            receipt.receipt_number, receipt_amount, txn.transaction_number, remaining);

        self.repository.update_transaction_amounts(
            transaction_id,
            &format!("{:.2}", new_remaining),
            Some(&format!("{:.2}", new_applied)),
            new_status,
        ).await?;

        self.repository.update_receipt_status(receipt_id, "applied").await
    }

    /// Reverse a receipt
    pub async fn reverse_receipt(&self, receipt_id: Uuid) -> AtlasResult<ArReceipt> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Receipt {} not found", receipt_id)
            ))?;

        if receipt.status != "confirmed" && receipt.status != "applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse receipt in '{}' status", receipt.status)
            ));
        }

        info!("Reversing receipt {}", receipt.receipt_number);
        self.repository.update_receipt_status(receipt_id, "reversed").await
    }

    // ========================================================================
    // Credit Memo Management
    // ========================================================================

    /// Create a credit memo
    pub async fn create_credit_memo(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        credit_memo_date: chrono::NaiveDate,
        reason_code: &str,
        reason_description: Option<&str>,
        amount: &str,
        tax_amount: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArCreditMemo> {
        if !VALID_CREDIT_MEMO_REASONS.contains(&reason_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid reason_code '{}'. Must be one of: {}",
                reason_code, VALID_CREDIT_MEMO_REASONS.join(", ")
            )));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "credit memo amount must be positive".to_string(),
            ));
        }

        let credit_memo_number = format!("CM-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let tax: f64 = tax_amount.parse().unwrap_or(0.0);
        let total = amt + tax;

        info!("Creating credit memo {} for customer {} amount {}",
            credit_memo_number, customer_id, total);

        self.repository.create_credit_memo(
            org_id, &credit_memo_number, customer_id, customer_number, customer_name,
            transaction_id, transaction_number, credit_memo_date,
            reason_code, reason_description, amount, tax_amount,
            &format!("{:.2}", total), notes, created_by,
        ).await
    }

    /// Get a credit memo by ID
    pub async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<ArCreditMemo>> {
        self.repository.get_credit_memo(id).await
    }

    /// List credit memos
    pub async fn list_credit_memos(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ArCreditMemo>> {
        if let Some(s) = status {
            if !VALID_CREDIT_MEMO_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid credit memo status '{}'. Must be one of: {}",
                    s, VALID_CREDIT_MEMO_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_credit_memos(org_id, status, customer_id).await
    }

    /// Approve a credit memo (moves from submitted to approved)
    pub async fn approve_credit_memo(&self, memo_id: Uuid) -> AtlasResult<ArCreditMemo> {
        let memo = self.repository.get_credit_memo(memo_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit memo {} not found", memo_id)
            ))?;

        if memo.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve credit memo in '{}' status. Must be 'submitted'.", memo.status)
            ));
        }

        info!("Approving credit memo {}", memo.credit_memo_number);
        self.repository.update_credit_memo_status(memo_id, "approved").await
    }

    /// Apply a credit memo to a transaction
    pub async fn apply_credit_memo(
        &self,
        memo_id: Uuid,
        transaction_id: Uuid,
    ) -> AtlasResult<ArCreditMemo> {
        let memo = self.repository.get_credit_memo(memo_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit memo {} not found", memo_id)
            ))?;

        if memo.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply credit memo in '{}' status. Must be 'approved'.", memo.status)
            ));
        }

        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AR transaction {} not found", transaction_id)
            ))?;

        if txn.status != "open" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot apply credit memo to transaction in '{}' status.", txn.status)
            ));
        }

        let memo_amount: f64 = memo.total_amount.parse().unwrap_or(0.0);
        let remaining: f64 = txn.amount_due_remaining.parse().unwrap_or(0.0);
        let new_remaining = (remaining - memo_amount).max(0.0);
        let new_adjusted: f64 = txn.amount_adjusted.parse().unwrap_or(0.0) + memo_amount.min(remaining);
        let new_status = if new_remaining < 0.01 { "closed" } else { "open" };

        info!("Applying credit memo {} ({}) to transaction {}",
            memo.credit_memo_number, memo_amount, txn.transaction_number);

        self.repository.update_transaction_amounts(
            transaction_id,
            &format!("{:.2}", new_remaining),
            None,
            new_status,
        ).await?;

        // Also update the amount_adjusted
        self.repository.update_transaction_adjusted(transaction_id, &format!("{:.2}", new_adjusted)).await?;

        self.repository.update_credit_memo_status(memo_id, "applied").await
    }

    // ========================================================================
    // Adjustments
    // ========================================================================

    /// Create an AR adjustment
    pub async fn create_adjustment(
        &self,
        org_id: Uuid,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        adjustment_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        adjustment_type: &str,
        amount: &str,
        receivable_account: Option<&str>,
        adjustment_account: Option<&str>,
        reason_code: Option<&str>,
        reason_description: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArAdjustment> {
        if !VALID_ADJUSTMENT_TYPES.contains(&adjustment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment_type '{}'. Must be one of: {}",
                adjustment_type, VALID_ADJUSTMENT_TYPES.join(", ")
            )));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "adjustment amount must be positive".to_string(),
            ));
        }

        let adjustment_number = format!("ADJ-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating AR adjustment {} type {} amount {}", adjustment_number, adjustment_type, amt);

        self.repository.create_adjustment(
            org_id, &adjustment_number, transaction_id, transaction_number,
            customer_id, customer_number, adjustment_date, gl_date,
            adjustment_type, amount, receivable_account, adjustment_account,
            reason_code, reason_description, notes, created_by,
        ).await
    }

    /// List adjustments
    pub async fn list_adjustments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ArAdjustment>> {
        if let Some(s) = status {
            if !VALID_ADJUSTMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid adjustment status '{}'. Must be one of: {}",
                    s, VALID_ADJUSTMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_adjustments(org_id, status, customer_id).await
    }

    /// Approve an adjustment
    pub async fn approve_adjustment(&self, adjustment_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ArAdjustment> {
        let adj = self.repository.get_adjustment(adjustment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Adjustment {} not found", adjustment_id)
            ))?;

        if adj.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve adjustment in '{}' status. Must be 'submitted'.", adj.status)
            ));
        }

        info!("Approving AR adjustment {}", adj.adjustment_number);
        self.repository.update_adjustment_status(adjustment_id, "approved", approved_by).await
    }

    // ========================================================================
    // AR Aging
    // ========================================================================

    /// Get AR aging summary
    pub async fn get_aging_summary(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<ArAgingSummary> {
        info!("Generating AR aging summary for org {} as of {}", org_id, as_of_date);
        self.repository.get_aging_summary(org_id, as_of_date).await
    }

    /// Get AR aging by customer
    pub async fn get_aging_by_customer(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<ArAgingByCustomer>> {
        self.repository.get_aging_by_customer(org_id, as_of_date).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transaction_types() {
        assert!(VALID_TRANSACTION_TYPES.contains(&"invoice"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"debit_memo"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"credit_memo"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"chargeback"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"deposit"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"guarantee"));
        assert_eq!(VALID_TRANSACTION_TYPES.len(), 6);
    }

    #[test]
    fn test_valid_transaction_statuses() {
        assert!(VALID_TRANSACTION_STATUSES.contains(&"draft"));
        assert!(VALID_TRANSACTION_STATUSES.contains(&"complete"));
        assert!(VALID_TRANSACTION_STATUSES.contains(&"open"));
        assert!(VALID_TRANSACTION_STATUSES.contains(&"closed"));
        assert!(VALID_TRANSACTION_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"line"));
        assert!(VALID_LINE_TYPES.contains(&"tax"));
        assert!(VALID_LINE_TYPES.contains(&"freight"));
        assert!(VALID_LINE_TYPES.contains(&"charges"));
    }

    #[test]
    fn test_valid_receipt_types() {
        assert!(VALID_RECEIPT_TYPES.contains(&"cash"));
        assert!(VALID_RECEIPT_TYPES.contains(&"check"));
        assert!(VALID_RECEIPT_TYPES.contains(&"credit_card"));
        assert!(VALID_RECEIPT_TYPES.contains(&"wire_transfer"));
        assert!(VALID_RECEIPT_TYPES.contains(&"ach"));
        assert!(VALID_RECEIPT_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_receipt_methods() {
        assert!(VALID_RECEIPT_METHODS.contains(&"automatic_receipt"));
        assert!(VALID_RECEIPT_METHODS.contains(&"manual_receipt"));
        assert!(VALID_RECEIPT_METHODS.contains(&"quick_cash"));
        assert!(VALID_RECEIPT_METHODS.contains(&"miscellaneous"));
    }

    #[test]
    fn test_valid_receipt_statuses() {
        assert!(VALID_RECEIPT_STATUSES.contains(&"draft"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"confirmed"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"applied"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"deposited"));
        assert!(VALID_RECEIPT_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_credit_memo_reasons() {
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"return"));
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"pricing_error"));
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"damaged"));
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"wrong_item"));
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"discount"));
        assert!(VALID_CREDIT_MEMO_REASONS.contains(&"other"));
    }

    #[test]
    fn test_valid_adjustment_types() {
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"write_off"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"write_off_bad_debt"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"small_balance_write_off"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"increase"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"decrease"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"transfer"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"revaluation"));
    }

    #[test]
    fn test_valid_adjustment_statuses() {
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"draft"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"submitted"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"approved"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"rejected"));
        assert!(VALID_ADJUSTMENT_STATUSES.contains(&"posted"));
    }
}
