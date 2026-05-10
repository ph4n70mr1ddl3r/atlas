//! Cash Receipt Management Engine
//!
//! Manages the full lifecycle of cash receipts:
//! - Create and manage receipt batches
//! - Record individual cash receipts
//! - Apply receipts to open invoices
//! - Unapply receipt applications
//! - Reverse receipts
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Receivables > Receipts

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_BATCH_STATUSES: &[&str] = &["draft", "confirmed", "closed", "cancelled"];
const VALID_RECEIPT_METHODS: &[&str] = &["bank", "cash", "credit_card", "wire_transfer", "other"];
const VALID_PAYMENT_METHODS: &[&str] = &["cash", "check", "credit_card", "wire_transfer", "bank_draft", "other"];
const VALID_RECEIPT_STATUSES: &[&str] = &[
    "unidentified", "identified", "applied", "partially_applied",
    "unapplied", "reversed",
];
const VALID_APPLICATION_STATUSES: &[&str] = &["applied", "unapplied", "reversed"];

pub struct CashReceiptEngine {
    repository: Arc<dyn CashReceiptRepository>,
}

impl CashReceiptEngine {
    pub fn new(repository: Arc<dyn CashReceiptRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Batches
    // ========================================================================

    /// Create a new receipt batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        batch_name: &str,
        description: Option<&str>,
        receipt_method: &str,
        bank_account_id: Option<Uuid>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptBatch> {
        if batch_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch number is required".into()));
        }
        if batch_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch name is required".into()));
        }
        if !VALID_RECEIPT_METHODS.contains(&receipt_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid receipt method '{}'. Must be one of: {}", receipt_method, VALID_RECEIPT_METHODS.join(", ")
            )));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        if self.repository.get_batch_by_number(org_id, batch_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Batch number '{}' already exists", batch_number)));
        }

        info!("Creating receipt batch '{}'", batch_number);
        self.repository.create_batch(
            org_id, batch_number, batch_name, description,
            receipt_method, bank_account_id, currency_code, created_by,
        ).await
    }

    /// Get a batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<ReceiptBatch>> {
        self.repository.get_batch(id).await
    }

    /// List batches
    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, status).await
    }

    /// Confirm a batch (transition from draft to confirmed)
    pub async fn confirm_batch(&self, batch_id: Uuid) -> AtlasResult<ReceiptBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot confirm batch in '{}' status. Must be 'draft'.", batch.status)
            ));
        }

        info!("Confirming receipt batch '{}'", batch.batch_number);
        self.repository.update_batch_status(batch_id, "confirmed").await
    }

    /// Close a batch
    pub async fn close_batch(&self, batch_id: Uuid) -> AtlasResult<ReceiptBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "confirmed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot close batch in '{}' status. Must be 'confirmed'.", batch.status)
            ));
        }

        info!("Closing receipt batch '{}'", batch.batch_number);
        self.repository.update_batch_status(batch_id, "closed").await
    }

    /// Cancel a batch (only draft batches can be cancelled)
    pub async fn cancel_batch(&self, batch_id: Uuid) -> AtlasResult<ReceiptBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Only draft batches can be cancelled".into()
            ));
        }

        info!("Cancelling receipt batch '{}'", batch.batch_number);
        self.repository.update_batch_status(batch_id, "cancelled").await
    }

    /// Delete a batch (only cancelled batches)
    pub async fn delete_batch(&self, batch_id: Uuid) -> AtlasResult<()> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "cancelled" && batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Only draft or cancelled batches can be deleted".into()
            ));
        }

        info!("Deleting receipt batch '{}'", batch.batch_number);
        self.repository.delete_batch(batch_id).await
    }

    // ========================================================================
    // Receipts
    // ========================================================================

    /// Create a new cash receipt
    pub async fn create_receipt(
        &self,
        org_id: Uuid,
        batch_id: Option<Uuid>,
        receipt_number: &str,
        customer_id: Uuid,
        customer_name: Option<&str>,
        customer_account_number: Option<&str>,
        payment_method: &str,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        receipt_date: chrono::NaiveDate,
        maturity_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        bank_name: Option<&str>,
        bank_branch: Option<&str>,
        deposit_date: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashReceipt> {
        if receipt_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Receipt number is required".into()));
        }
        if !VALID_PAYMENT_METHODS.contains(&payment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment method '{}'. Must be one of: {}", payment_method, VALID_PAYMENT_METHODS.join(", ")
            )));
        }

        let amount_val: f64 = amount.parse().unwrap_or(f64::NAN);
        if amount_val.is_nan() || amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("Amount must be a positive number".into()));
        }

        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }

        // Validate batch exists and is in a valid state for adding receipts
        if let Some(bid) = batch_id {
            let batch = self.repository.get_batch(bid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", bid)))?;
            if batch.status != "draft" && batch.status != "confirmed" {
                return Err(AtlasError::WorkflowError(
                    format!("Cannot add receipts to a batch in '{}' status", batch.status)
                ));
            }
        }

        // Check for duplicate receipt number
        if self.repository.get_receipt_by_number(org_id, receipt_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Receipt number '{}' already exists", receipt_number)));
        }

        info!("Creating cash receipt '{}' for customer {}", receipt_number, customer_id);

        let receipt = self.repository.create_receipt(
            org_id, batch_id, receipt_number, customer_id,
            customer_name, customer_account_number, payment_method,
            amount, currency_code, exchange_rate, receipt_date,
            maturity_date, reference_number, bank_name, bank_branch,
            deposit_date, notes, created_by,
        ).await?;

        // Update batch totals if batch is specified
        if let Some(bid) = batch_id {
            let _ = self.recalculate_batch_totals(bid).await;
        }

        Ok(receipt)
    }

    /// Get a receipt by ID
    pub async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<CashReceipt>> {
        self.repository.get_receipt(id).await
    }

    /// List receipts with optional filters
    pub async fn list_receipts(
        &self,
        org_id: Uuid,
        batch_id: Option<Uuid>,
        customer_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CashReceipt>> {
        if let Some(s) = status {
            if !VALID_RECEIPT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RECEIPT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_receipts(org_id, batch_id, customer_id, status).await
    }

    /// Identify a receipt (transition from unidentified to identified/unapplied)
    pub async fn identify_receipt(&self, receipt_id: Uuid) -> AtlasResult<CashReceipt> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", receipt_id)))?;

        if receipt.status != "unidentified" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot identify receipt in '{}' status. Must be 'unidentified'.", receipt.status)
            ));
        }

        info!("Identifying receipt '{}'", receipt.receipt_number);
        self.repository.update_receipt_status(receipt_id, "unapplied").await
    }

    // ========================================================================
    // Applications
    // ========================================================================

    /// Apply a receipt to an invoice
    pub async fn apply_receipt(
        &self,
        org_id: Uuid,
        receipt_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        applied_amount: &str,
        discount_taken: &str,
        application_date: chrono::NaiveDate,
        applied_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptApplication> {
        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", receipt_id)))?;

        if receipt.status == "reversed" {
            return Err(AtlasError::WorkflowError("Cannot apply a reversed receipt".into()));
        }
        if receipt.status == "unidentified" {
            return Err(AtlasError::WorkflowError("Cannot apply an unidentified receipt. Identify it first.".into()));
        }

        let apply_amt: f64 = applied_amount.parse().unwrap_or(f64::NAN);
        if apply_amt.is_nan() || apply_amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Applied amount must be a positive number".into()));
        }

        let discount_amt: f64 = discount_taken.parse().unwrap_or(0.0);

        let receipt_amount: f64 = receipt.amount.parse().unwrap_or(0.0);
        let already_applied: f64 = receipt.applied_amount.parse().unwrap_or(0.0);
        let available = receipt_amount - already_applied;

        if (apply_amt + discount_amt) > available + 0.01 {
            return Err(AtlasError::ValidationFailed(format!(
                "Applied amount ({}) + discount ({}) exceeds available amount ({:.2})",
                applied_amount, discount_taken, available
            )));
        }

        info!("Applying receipt '{}' to invoice {} (amount: {})", receipt.receipt_number, invoice_id, applied_amount);

        let application = self.repository.create_application(
            org_id, receipt_id, invoice_id, invoice_number,
            applied_amount, discount_taken, application_date,
            Some(application_date), applied_by,
        ).await?;

        // Recalculate receipt amounts
        let total_applied = self.repository.sum_applied_for_receipt(receipt_id).await?;
        let total_applied_f: f64 = total_applied.parse().unwrap_or(0.0);
        // Avoid -0.00 from floating point
        let total_applied_f = if total_applied_f.abs() < 0.005 { 0.0 } else { total_applied_f };
        let unapplied = receipt_amount - total_applied_f;
        // Avoid -0.00 from floating point
        let unapplied = if unapplied.abs() < 0.005 { 0.0 } else { unapplied };

        let new_status = if unapplied.abs() < 0.01 {
            "applied"
        } else {
            "partially_applied"
        };

        self.repository.update_receipt_amounts(
            receipt_id,
            &format!("{:.2}", total_applied_f),
            &format!("{:.2}", unapplied),
        ).await?;

        self.repository.update_receipt_status(receipt_id, new_status).await?;

        // Update batch totals if receipt has a batch
        if let Some(bid) = receipt.batch_id {
            let _ = self.recalculate_batch_totals(bid).await;
        }

        Ok(application)
    }

    /// Unapply a receipt application
    pub async fn unapply_receipt(&self, application_id: Uuid, reversed_by: Option<Uuid>) -> AtlasResult<ReceiptApplication> {
        let application = self.repository.get_application(application_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", application_id)))?;

        if application.status != "applied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot unapply application in '{}' status. Must be 'applied'.", application.status)
            ));
        }

        let receipt = self.repository.get_receipt(application.receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", application.receipt_id)))?;

        info!("Unapplying receipt '{}' from invoice {}", receipt.receipt_number, application.invoice_id);

        let app = self.repository.update_application_status(application_id, "reversed", reversed_by).await?;

        // Recalculate receipt amounts
        let total_applied = self.repository.sum_applied_for_receipt(receipt.id).await?;
        let total_applied_f: f64 = total_applied.parse().unwrap_or(0.0);
        // Avoid -0.00 from floating point
        let total_applied_f = if total_applied_f.abs() < 0.005 { 0.0 } else { total_applied_f };
        let receipt_amount: f64 = receipt.amount.parse().unwrap_or(0.0);
        let unapplied = receipt_amount - total_applied_f;
        // Avoid -0.00 from floating point
        let unapplied = if unapplied.abs() < 0.005 { 0.0 } else { unapplied };

        let new_status = if total_applied_f.abs() < 0.01 {
            "unapplied"
        } else {
            "partially_applied"
        };

        self.repository.update_receipt_amounts(
            receipt.id,
            &format!("{:.2}", total_applied_f),
            &format!("{:.2}", unapplied),
        ).await?;

        self.repository.update_receipt_status(receipt.id, new_status).await?;

        if let Some(bid) = receipt.batch_id {
            let _ = self.recalculate_batch_totals(bid).await;
        }

        Ok(app)
    }

    /// List applications for a receipt
    pub async fn list_applications(&self, receipt_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptApplication>> {
        if let Some(s) = status {
            if !VALID_APPLICATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_APPLICATION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_applications(receipt_id, status).await
    }

    // ========================================================================
    // Reversal
    // ========================================================================

    /// Reverse a receipt
    pub async fn reverse_receipt(&self, receipt_id: Uuid, reason: &str) -> AtlasResult<CashReceipt> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Reversal reason is required".into()));
        }

        let receipt = self.repository.get_receipt(receipt_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", receipt_id)))?;

        if receipt.status == "reversed" {
            return Err(AtlasError::WorkflowError("Receipt is already reversed".into()));
        }

        info!("Reversing receipt '{}' (reason: {})", receipt.receipt_number, reason);

        // Reverse all active applications
        let applications = self.repository.list_applications(receipt_id, Some("applied")).await?;
        for app in &applications {
            self.repository.update_application_status(app.id, "reversed", None).await?;
        }

        self.repository.update_receipt_reversal(receipt_id, reason, None).await?;
        let receipt = self.repository.update_receipt_status(receipt_id, "reversed").await?;

        if let Some(bid) = receipt.batch_id {
            let _ = self.recalculate_batch_totals(bid).await;
        }

        Ok(receipt)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashReceiptDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn recalculate_batch_totals(&self, batch_id: Uuid) -> AtlasResult<()> {
        let receipts = self.repository.list_receipts(Uuid::nil(), Some(batch_id), None, None).await?;
        let total: f64 = receipts.iter()
            .filter(|r| r.status != "reversed")
            .map(|r| r.amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let count = receipts.iter().filter(|r| r.status != "reversed").count() as i32;
        self.repository.update_batch_totals(batch_id, &format!("{:.2}", total), count).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        batches: std::sync::Mutex<Vec<ReceiptBatch>>,
        receipts: std::sync::Mutex<Vec<CashReceipt>>,
        applications: std::sync::Mutex<Vec<ReceiptApplication>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                batches: std::sync::Mutex::new(vec![]),
                receipts: std::sync::Mutex::new(vec![]),
                applications: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl CashReceiptRepository for MockRepo {
        async fn create_batch(
            &self, org_id: Uuid, num: &str, name: &str, desc: Option<&str>,
            method: &str, bank: Option<Uuid>, currency: &str, cb: Option<Uuid>,
        ) -> AtlasResult<ReceiptBatch> {
            let b = ReceiptBatch {
                id: Uuid::new_v4(), organization_id: org_id,
                batch_number: num.into(), batch_name: name.into(),
                description: desc.map(Into::into), receipt_method: method.into(),
                bank_account_id: bank, status: "draft".into(),
                currency_code: currency.into(), total_amount: "0.00".into(),
                receipt_count: 0, gl_posting_date: None, posted_by: None,
                metadata: serde_json::json!({}), created_by: cb,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.batches.lock().unwrap().push(b.clone());
            Ok(b)
        }

        async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<ReceiptBatch>> {
            Ok(self.batches.lock().unwrap().iter().find(|b| b.id == id).cloned())
        }

        async fn get_batch_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<ReceiptBatch>> {
            Ok(self.batches.lock().unwrap().iter()
                .find(|b| b.organization_id == org_id && b.batch_number == num).cloned())
        }

        async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptBatch>> {
            Ok(self.batches.lock().unwrap().iter()
                .filter(|b| b.organization_id == org_id)
                .filter(|b| status.map_or(true, |s| b.status == s))
                .cloned().collect())
        }

        async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<ReceiptBatch> {
            let mut bs = self.batches.lock().unwrap();
            let b = bs.iter_mut().find(|b| b.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", id)))?;
            b.status = status.into();
            b.updated_at = chrono::Utc::now();
            Ok(b.clone())
        }

        async fn update_batch_totals(&self, id: Uuid, total: &str, count: i32) -> AtlasResult<()> {
            let mut bs = self.batches.lock().unwrap();
            if let Some(b) = bs.iter_mut().find(|b| b.id == id) {
                b.total_amount = total.into();
                b.receipt_count = count;
            }
            Ok(())
        }

        async fn delete_batch(&self, id: Uuid) -> AtlasResult<()> {
            self.batches.lock().unwrap().retain(|b| b.id != id);
            Ok(())
        }

        async fn create_receipt(
            &self, org_id: Uuid, batch_id: Option<Uuid>, num: &str, customer_id: Uuid,
            customer_name: Option<&str>, customer_account_number: Option<&str>,
            payment_method: &str, amount: &str, currency: &str,
            exchange_rate: Option<&str>, receipt_date: chrono::NaiveDate,
            maturity_date: Option<chrono::NaiveDate>, reference_number: Option<&str>,
            bank_name: Option<&str>, bank_branch: Option<&str>,
            deposit_date: Option<chrono::NaiveDate>, notes: Option<&str>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<CashReceipt> {
            let r = CashReceipt {
                id: Uuid::new_v4(), organization_id: org_id, batch_id,
                receipt_number: num.into(), customer_id,
                customer_name: customer_name.map(Into::into),
                customer_account_number: customer_account_number.map(Into::into),
                payment_method: payment_method.into(),
                status: "unapplied".into(), amount: amount.into(),
                applied_amount: "0.00".into(), unapplied_amount: amount.into(),
                currency_code: currency.into(),
                exchange_rate: exchange_rate.map(Into::into),
                receipt_date, maturity_date,
                reference_number: reference_number.map(Into::into),
                bank_name: bank_name.map(Into::into),
                bank_branch: bank_branch.map(Into::into),
                deposit_date, clearance_status: Some("available".into()),
                notes: notes.map(Into::into), reversal_reason: None,
                reversed_from: None, metadata: serde_json::json!({}),
                created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.receipts.lock().unwrap().push(r.clone());
            Ok(r)
        }

        async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<CashReceipt>> {
            Ok(self.receipts.lock().unwrap().iter().find(|r| r.id == id).cloned())
        }

        async fn get_receipt_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<CashReceipt>> {
            Ok(self.receipts.lock().unwrap().iter()
                .find(|r| r.organization_id == org_id && r.receipt_number == num).cloned())
        }

        async fn list_receipts(&self, org_id: Uuid, batch_id: Option<Uuid>, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<CashReceipt>> {
            Ok(self.receipts.lock().unwrap().iter()
                .filter(|r| r.organization_id == org_id || org_id == Uuid::nil())
                .filter(|r| batch_id.map_or(true, |b| r.batch_id == Some(b)))
                .filter(|r| customer_id.map_or(true, |c| r.customer_id == c))
                .filter(|r| status.map_or(true, |s| r.status == s))
                .cloned().collect())
        }

        async fn update_receipt_status(&self, id: Uuid, status: &str) -> AtlasResult<CashReceipt> {
            let mut rs = self.receipts.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", id)))?;
            r.status = status.into();
            r.updated_at = chrono::Utc::now();
            Ok(r.clone())
        }

        async fn update_receipt_amounts(&self, id: Uuid, applied: &str, unapplied: &str) -> AtlasResult<CashReceipt> {
            let mut rs = self.receipts.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", id)))?;
            r.applied_amount = applied.into();
            r.unapplied_amount = unapplied.into();
            r.updated_at = chrono::Utc::now();
            Ok(r.clone())
        }

        async fn update_receipt_reversal(&self, id: Uuid, reason: &str, _: Option<Uuid>) -> AtlasResult<CashReceipt> {
            let mut rs = self.receipts.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Receipt {} not found", id)))?;
            r.reversal_reason = Some(reason.into());
            r.updated_at = chrono::Utc::now();
            Ok(r.clone())
        }

        async fn create_application(
            &self, org_id: Uuid, receipt_id: Uuid, invoice_id: Uuid,
            invoice_number: Option<&str>, applied: &str, discount: &str,
            app_date: chrono::NaiveDate, gl_date: Option<chrono::NaiveDate>,
            applied_by: Option<Uuid>,
        ) -> AtlasResult<ReceiptApplication> {
            let a = ReceiptApplication {
                id: Uuid::new_v4(), organization_id: org_id,
                receipt_id, invoice_id,
                invoice_number: invoice_number.map(Into::into),
                applied_amount: applied.into(), discount_taken: discount.into(),
                status: "applied".into(), application_date: app_date,
                gl_date, applied_by, reversed_by: None, reversal_date: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.applications.lock().unwrap().push(a.clone());
            Ok(a)
        }

        async fn get_application(&self, id: Uuid) -> AtlasResult<Option<ReceiptApplication>> {
            Ok(self.applications.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }

        async fn list_applications(&self, receipt_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptApplication>> {
            Ok(self.applications.lock().unwrap().iter()
                .filter(|a| a.receipt_id == receipt_id)
                .filter(|a| status.map_or(true, |s| a.status == s))
                .cloned().collect())
        }

        async fn update_application_status(&self, id: Uuid, status: &str, reversed_by: Option<Uuid>) -> AtlasResult<ReceiptApplication> {
            let mut as_ = self.applications.lock().unwrap();
            let a = as_.iter_mut().find(|a| a.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Application {} not found", id)))?;
            a.status = status.into();
            if reversed_by.is_some() {
                a.reversed_by = reversed_by;
                a.reversal_date = Some(chrono::Utc::now());
            }
            a.updated_at = chrono::Utc::now();
            Ok(a.clone())
        }

        async fn sum_applied_for_receipt(&self, receipt_id: Uuid) -> AtlasResult<String> {
            let apps = self.applications.lock().unwrap();
            let sum: f64 = apps.iter()
                .filter(|a| a.receipt_id == receipt_id && a.status == "applied")
                .map(|a| a.applied_amount.parse::<f64>().unwrap_or(0.0))
                .sum();
            Ok(format!("{:.2}", sum))
        }

        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CashReceiptDashboard> {
            Ok(CashReceiptDashboard {
                total_batches: 0, draft_batches: 0, confirmed_batches: 0,
                total_receipts: 0, applied_receipts: 0, unapplied_receipts: 0,
                partially_applied_receipts: 0, reversed_receipts: 0,
                total_receipt_amount: "0.00".into(), total_applied_amount: "0.00".into(),
                total_unapplied_amount: "0.00".into(),
            })
        }
    }

    fn eng() -> CashReceiptEngine {
        CashReceiptEngine::new(Arc::new(MockRepo::new()))
    }

    // ========================================================================
    // Batch Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_batch() {
        let b = eng().create_batch(
            Uuid::new_v4(), "BATCH-001", "Daily Receipts",
            Some("Today's receipts"), "bank", None, "USD", None,
        ).await.unwrap();
        assert_eq!(b.batch_number, "BATCH-001");
        assert_eq!(b.batch_name, "Daily Receipts");
        assert_eq!(b.status, "draft");
    }

    #[tokio::test]
    async fn test_create_batch_empty_number_fails() {
        let r = eng().create_batch(Uuid::new_v4(), "", "Batch", None, "bank", None, "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_batch_invalid_method_fails() {
        let r = eng().create_batch(Uuid::new_v4(), "B-1", "Batch", None, "crypto", None, "USD", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_batch_duplicate_number_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_batch(org, "DUP-001", "Batch 1", None, "bank", None, "USD", None).await;
        let r = e.create_batch(org, "DUP-001", "Batch 2", None, "cash", None, "USD", None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_confirm_and_close_batch() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "B-CONF", "Batch", None, "bank", None, "USD", None).await.unwrap();
        assert_eq!(b.status, "draft");

        let b = e.confirm_batch(b.id).await.unwrap();
        assert_eq!(b.status, "confirmed");

        let b = e.close_batch(b.id).await.unwrap();
        assert_eq!(b.status, "closed");
    }

    #[tokio::test]
    async fn test_confirm_non_draft_fails() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "B-CONF2", "Batch", None, "bank", None, "USD", None).await.unwrap();
        e.confirm_batch(b.id).await.unwrap();
        let r = e.confirm_batch(b.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft_batch() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "B-CANC", "Batch", None, "bank", None, "USD", None).await.unwrap();
        let b = e.cancel_batch(b.id).await.unwrap();
        assert_eq!(b.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_confirmed_batch_fails() {
        let e = eng();
        let b = e.create_batch(Uuid::new_v4(), "B-CANC2", "Batch", None, "bank", None, "USD", None).await.unwrap();
        e.confirm_batch(b.id).await.unwrap();
        let r = e.cancel_batch(b.id).await;
        assert!(r.is_err());
    }

    // ========================================================================
    // Receipt Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_receipt() {
        let r = eng().create_receipt(
            Uuid::new_v4(), None, "RCP-001", Uuid::new_v4(),
            Some("Acme Corp"), Some("CUST-001"),
            "check", "5000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, Some("CHK-12345"), Some("First National"), None,
            None, None, None,
        ).await.unwrap();
        assert_eq!(r.receipt_number, "RCP-001");
        assert_eq!(r.status, "unapplied");
        assert_eq!(r.amount, "5000.00");
        assert_eq!(r.unapplied_amount, "5000.00");
        assert_eq!(r.applied_amount, "0.00");
    }

    #[tokio::test]
    async fn test_create_receipt_empty_number_fails() {
        let r = eng().create_receipt(
            Uuid::new_v4(), None, "", Uuid::new_v4(), None, None,
            "cash", "100.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_negative_amount_fails() {
        let r = eng().create_receipt(
            Uuid::new_v4(), None, "RCP-NEG", Uuid::new_v4(), None, None,
            "cash", "-100.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_invalid_payment_method_fails() {
        let r = eng().create_receipt(
            Uuid::new_v4(), None, "RCP-PM", Uuid::new_v4(), None, None,
            "bitcoin", "100.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_receipt_duplicate_number_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_receipt(
            org, None, "RCP-DUP", Uuid::new_v4(), None, None,
            "cash", "100.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await;
        let r = e.create_receipt(
            org, None, "RCP-DUP", Uuid::new_v4(), None, None,
            "check", "200.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_identify_receipt() {
        let e = eng();
        let r = e.create_receipt(
            Uuid::new_v4(), None, "RCP-ID", Uuid::new_v4(), None, None,
            "cash", "1000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();
        // The mock creates receipts with "unapplied" status.
        // To test identification, we manually set status to "unidentified".
        {
            let repo = &e.repository;
            // Use the repository to update status directly
            repo.update_receipt_status(r.id, "unidentified").await.unwrap();
        }
        let r = e.identify_receipt(r.id).await.unwrap();
        assert_eq!(r.status, "unapplied");
    }

    // ========================================================================
    // Application Tests
    // ========================================================================

    #[tokio::test]
    async fn test_apply_receipt_full() {
        let e = eng();
        let org = Uuid::new_v4();
        let inv_id = Uuid::new_v4();

        let r = e.create_receipt(
            org, None, "RCP-APP", Uuid::new_v4(), Some("Acme"), None,
            "check", "5000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let app = e.apply_receipt(
            org, r.id, inv_id, Some("INV-001"), "5000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), None,
        ).await.unwrap();

        assert_eq!(app.status, "applied");
        assert_eq!(app.applied_amount, "5000.00");

        let updated = e.get_receipt(r.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "applied");
        assert_eq!(updated.applied_amount, "5000.00");
        assert_eq!(updated.unapplied_amount, "0.00");
    }

    #[tokio::test]
    async fn test_apply_receipt_partial() {
        let e = eng();
        let org = Uuid::new_v4();
        let inv_id = Uuid::new_v4();

        let r = e.create_receipt(
            org, None, "RCP-PART", Uuid::new_v4(), None, None,
            "check", "10000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let _ = e.apply_receipt(
            org, r.id, inv_id, Some("INV-002"), "3000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), None,
        ).await.unwrap();

        let updated = e.get_receipt(r.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "partially_applied");
        assert_eq!(updated.applied_amount, "3000.00");
        assert_eq!(updated.unapplied_amount, "7000.00");
    }

    #[tokio::test]
    async fn test_apply_receipt_exceeds_amount_fails() {
        let e = eng();
        let org = Uuid::new_v4();

        let r = e.create_receipt(
            org, None, "RCP-EXC", Uuid::new_v4(), None, None,
            "cash", "500.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let r = e.apply_receipt(
            org, r.id, Uuid::new_v4(), None, "600.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_unapply_receipt() {
        let e = eng();
        let org = Uuid::new_v4();
        let inv_id = Uuid::new_v4();

        let r = e.create_receipt(
            org, None, "RCP-UNAPP", Uuid::new_v4(), None, None,
            "check", "5000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let app = e.apply_receipt(
            org, r.id, inv_id, Some("INV-003"), "5000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), None,
        ).await.unwrap();

        let app = e.unapply_receipt(app.id, None).await.unwrap();
        assert_eq!(app.status, "reversed");

        let updated = e.get_receipt(r.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "unapplied");
        assert_eq!(updated.applied_amount, "0.00");
        assert_eq!(updated.unapplied_amount, "5000.00");
    }

    // ========================================================================
    // Reversal Tests
    // ========================================================================

    #[tokio::test]
    async fn test_reverse_receipt() {
        let e = eng();
        let org = Uuid::new_v4();
        let inv_id = Uuid::new_v4();

        let r = e.create_receipt(
            org, None, "RCP-REV", Uuid::new_v4(), None, None,
            "cash", "1000.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let _ = e.apply_receipt(
            org, r.id, inv_id, None, "1000.00", "0.00",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), None,
        ).await.unwrap();

        let r = e.reverse_receipt(r.id, "Payment returned").await.unwrap();
        assert_eq!(r.status, "reversed");
        assert_eq!(r.reversal_reason, Some("Payment returned".into()));
    }

    #[tokio::test]
    async fn test_reverse_receipt_no_reason_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let r = e.create_receipt(
            org, None, "RCP-REV2", Uuid::new_v4(), None, None,
            "cash", "500.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();

        let r = e.reverse_receipt(r.id, "").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_reverse_already_reversed_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let r = e.create_receipt(
            org, None, "RCP-REV3", Uuid::new_v4(), None, None,
            "cash", "500.00", "USD", None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, None, None, None, None, None,
        ).await.unwrap();
        e.reverse_receipt(r.id, "Duplicate").await.unwrap();
        let r = e.reverse_receipt(r.id, "Double reversed").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_receipts, 0);
        assert_eq!(d.total_receipt_amount, "0.00");
    }

    #[tokio::test]
    async fn test_list_batches_invalid_status() {
        let r = eng().list_batches(Uuid::new_v4(), Some("unknown")).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_list_receipts_invalid_status() {
        let r = eng().list_receipts(Uuid::new_v4(), None, None, Some("unknown")).await;
        assert!(r.is_err());
    }
}
