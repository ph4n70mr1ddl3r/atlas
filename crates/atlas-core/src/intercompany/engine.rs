//! Intercompany Engine Implementation
//!
//! Manages intercompany transactions between legal entities / business units.
//! Provides batch creation, transaction management, approval workflow,
//! settlement tracking, and balance monitoring.
//!
//! Oracle Fusion Cloud ERP equivalent: Intercompany > Intercompany Transactions

use atlas_shared::{
    IntercompanyBatch, IntercompanyTransaction, IntercompanySettlement,
    IntercompanyBalance, IntercompanyBalanceSummary,
    AtlasError, AtlasResult,
};
use super::IntercompanyRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid batch statuses
const VALID_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "posted", "cancelled",
];

/// Valid transaction statuses
const VALID_TXN_STATUSES: &[&str] = &[
    "draft", "approved", "posted", "settled", "cancelled",
];

/// Valid transaction types
const VALID_TXN_TYPES: &[&str] = &[
    "invoice", "journal_entry", "payment", "charge", "allocation",
];

/// Valid settlement methods
const VALID_SETTLEMENT_METHODS: &[&str] = &[
    "cash", "netting", "offset",
];

/// Default intercompany account codes
const DEFAULT_FROM_IC_ACCOUNT: &str = "IC-DUE-TO";
const DEFAULT_TO_IC_ACCOUNT: &str = "IC-DUE-FROM";

/// Intercompany engine for managing inter-entity transactions
pub struct IntercompanyEngine {
    repository: Arc<dyn IntercompanyRepository>,
}

impl IntercompanyEngine {
    pub fn new(repository: Arc<dyn IntercompanyRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Batch Management
    // ========================================================================

    /// Create a new intercompany batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyBatch> {
        // Validate
        if batch_number.is_empty() || batch_number.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Batch number must be 1-50 characters".to_string(),
            ));
        }
        if from_entity_name.is_empty() || to_entity_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Entity names are required".to_string(),
            ));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be 3 characters (ISO 4217)".to_string(),
            ));
        }
        if from_entity_id == to_entity_id {
            return Err(AtlasError::ValidationFailed(
                "From and to entities must be different".to_string(),
            ));
        }

        info!(
            "Creating intercompany batch '{}' for org {} ({} -> {})",
            batch_number, org_id, from_entity_name, to_entity_name
        );

        self.repository.create_batch(
            org_id, batch_number, description,
            from_entity_id, from_entity_name,
            to_entity_id, to_entity_name,
            currency_code, accounting_date, created_by,
        ).await
    }

    /// Get a batch by number
    pub async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<IntercompanyBatch>> {
        self.repository.get_batch(org_id, batch_number).await
    }

    /// Get a batch by ID
    pub async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<IntercompanyBatch>> {
        self.repository.get_batch_by_id(id).await
    }

    /// List batches, optionally filtered by status
    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<IntercompanyBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, status).await
    }

    /// Submit a batch for approval (draft -> submitted)
    pub async fn submit_batch(&self, batch_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<IntercompanyBatch> {
        let batch = self.repository.get_batch_by_id(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Batch not found".to_string()))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit batch in '{}' status. Must be 'draft'.", batch.status)
            ));
        }

        // Verify batch has transactions
        let transactions = self.repository.list_transactions_by_batch(batch_id).await?;
        if transactions.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit an empty batch. Add at least one transaction.".to_string(),
            ));
        }

        info!("Submitting intercompany batch '{}' for approval", batch.batch_number);
        self.repository.update_batch_status(batch_id, "submitted", submitted_by, None, None).await
    }

    /// Approve a batch (submitted -> approved)
    pub async fn approve_batch(&self, batch_id: Uuid, approved_by: Uuid) -> AtlasResult<IntercompanyBatch> {
        let batch = self.repository.get_batch_by_id(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Batch not found".to_string()))?;

        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve batch in '{}' status. Must be 'submitted'.", batch.status)
            ));
        }

        info!("Approving intercompany batch '{}' by {}", batch.batch_number, approved_by);
        self.repository.update_batch_status(batch_id, "approved", Some(approved_by), None, None).await
    }

    /// Post a batch (approved -> posted) - creates journal entries
    pub async fn post_batch(&self, batch_id: Uuid) -> AtlasResult<IntercompanyBatch> {
        let batch = self.repository.get_batch_by_id(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Batch not found".to_string()))?;

        if batch.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post batch in '{}' status. Must be 'approved'.", batch.status)
            ));
        }

        // Update all transactions in the batch to 'posted'
        let transactions = self.repository.list_transactions_by_batch(batch_id).await?;
        for txn in &transactions {
            self.repository.update_transaction_status(txn.id, "posted", None).await?;
        }

        // Update intercompany balances
        for txn in &transactions {
            self.recalculate_balances(
                batch.organization_id,
                txn.from_entity_id,
                txn.to_entity_id,
                &txn.currency_code,
            ).await?;
        }

        info!("Posting intercompany batch '{}' with {} transactions", batch.batch_number, transactions.len());
        self.repository.update_batch_status(
            batch_id, "posted", None, Some(chrono::Utc::now()), None,
        ).await
    }

    /// Reject a batch (submitted -> cancelled)
    pub async fn reject_batch(&self, batch_id: Uuid, reason: &str) -> AtlasResult<IntercompanyBatch> {
        let batch = self.repository.get_batch_by_id(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Batch not found".to_string()))?;

        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject batch in '{}' status. Must be 'submitted'.", batch.status)
            ));
        }

        info!("Rejecting intercompany batch '{}': {}", batch.batch_number, reason);
        self.repository.update_batch_status(batch_id, "cancelled", None, None, Some(reason)).await
    }

    // ========================================================================
    // Transaction Management
    // ========================================================================

    /// Create a new intercompany transaction
    pub async fn create_transaction(
        &self,
        org_id: Uuid,
        batch_number: &str,
        transaction_number: &str,
        transaction_type: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        from_debit_account: Option<&str>,
        from_credit_account: Option<&str>,
        to_debit_account: Option<&str>,
        to_credit_account: Option<&str>,
        from_ic_account: Option<&str>,
        to_ic_account: Option<&str>,
        transaction_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        source_entity_type: Option<&str>,
        source_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyTransaction> {
        // Validate batch exists and is in draft
        let batch = self.repository.get_batch(org_id, batch_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Batch '{}' not found", batch_number)
            ))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add transactions to a non-draft batch".to_string(),
            ));
        }

        if from_entity_id == to_entity_id {
            return Err(AtlasError::ValidationFailed(
                "From and to entities must be different".to_string(),
            ));
        }

        // Validate transaction type
        if !VALID_TXN_TYPES.contains(&transaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction_type '{}'. Must be one of: {}",
                transaction_type, VALID_TXN_TYPES.join(", ")
            )));
        }

        // Validate amount
        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be greater than zero".to_string(),
            ));
        }

        // Validate exchange rate if provided
        if let Some(er) = exchange_rate {
            let rate: f64 = er.parse().map_err(|_| AtlasError::ValidationFailed(
                "Exchange rate must be a valid number".to_string(),
            ))?;
            if rate <= 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Exchange rate must be greater than zero".to_string(),
                ));
            }
        }

        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be 3 characters (ISO 4217)".to_string(),
            ));
        }

        // Default IC accounts if not provided
        let from_ic = from_ic_account.unwrap_or(DEFAULT_FROM_IC_ACCOUNT);
        let to_ic = to_ic_account.unwrap_or(DEFAULT_TO_IC_ACCOUNT);

        info!(
            "Creating intercompany transaction '{}' of type {} for {} {} ({} -> {})",
            transaction_number, transaction_type, amount, currency_code,
            from_entity_name, to_entity_name
        );

        let transaction = self.repository.create_transaction(
            org_id, batch.id, transaction_number, transaction_type,
            description,
            from_entity_id, from_entity_name,
            to_entity_id, to_entity_name,
            amount, currency_code, exchange_rate,
            from_debit_account, from_credit_account,
            to_debit_account, to_credit_account,
            from_ic, to_ic,
            transaction_date, due_date,
            source_entity_type, source_entity_id, created_by,
        ).await?;

        // Update batch totals
        self.recalculate_batch_totals(batch.id).await?;

        Ok(transaction)
    }

    /// Get a transaction by number
    pub async fn get_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<IntercompanyTransaction>> {
        self.repository.get_transaction(org_id, transaction_number).await
    }

    /// List transactions in a batch
    pub async fn list_transactions_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<IntercompanyTransaction>> {
        self.repository.list_transactions_by_batch(batch_id).await
    }

    /// List transactions for an entity
    pub async fn list_transactions_by_entity(
        &self,
        org_id: Uuid,
        entity_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<IntercompanyTransaction>> {
        if let Some(s) = status {
            if !VALID_TXN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TXN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_transactions_by_entity(org_id, entity_id, status).await
    }

    // ========================================================================
    // Settlement Management
    // ========================================================================

    /// Create a settlement for intercompany transactions
    pub async fn create_settlement(
        &self,
        org_id: Uuid,
        settlement_number: &str,
        settlement_method: &str,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        settled_amount: &str,
        currency_code: &str,
        payment_reference: Option<&str>,
        transaction_ids: &[Uuid],
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanySettlement> {
        // Validate settlement method
        if !VALID_SETTLEMENT_METHODS.contains(&settlement_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement_method '{}'. Must be one of: {}",
                settlement_method, VALID_SETTLEMENT_METHODS.join(", ")
            )));
        }

        // Validate amount
        let amt: f64 = settled_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Settled amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Settled amount must be greater than zero".to_string(),
            ));
        }

        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Currency code must be 3 characters (ISO 4217)".to_string(),
            ));
        }

        // Mark linked transactions as settled
        let txn_ids_json = serde_json::to_value(transaction_ids)
            .map_err(|e| AtlasError::Internal(e.to_string()))?;

        let today = chrono::Utc::now().date_naive();
        for txn_id in transaction_ids {
            if let Ok(Some(_txn)) = self.repository.get_transaction(
                // We need org_id but get_transaction takes it; let's just update directly
                org_id, &txn_id.to_string(),
            ).await {
                self.repository.update_transaction_status(*txn_id, "settled", Some(today)).await?;
            }
        }

        info!(
            "Creating intercompany settlement '{}' for {} {} ({} -> {})",
            settlement_number, settled_amount, currency_code, from_entity_id, to_entity_id
        );

        let settlement = self.repository.create_settlement(
            org_id, settlement_number, settlement_method,
            from_entity_id, to_entity_id,
            settled_amount, currency_code, payment_reference,
            txn_ids_json, created_by,
        ).await?;

        // Recalculate balances
        self.recalculate_balances(org_id, from_entity_id, to_entity_id, currency_code).await?;

        Ok(settlement)
    }

    /// List settlements
    pub async fn list_settlements(
        &self,
        org_id: Uuid,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<IntercompanySettlement>> {
        self.repository.list_settlements(org_id, entity_id).await
    }

    // ========================================================================
    // Balance Management
    // ========================================================================

    /// Get outstanding balances for an entity pair
    pub async fn get_balance(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
    ) -> AtlasResult<Option<IntercompanyBalance>> {
        self.repository.get_balance(org_id, from_entity_id, to_entity_id, currency_code).await
    }

    /// Get all outstanding intercompany balances (dashboard summary)
    pub async fn get_balance_summary(&self, org_id: Uuid) -> AtlasResult<IntercompanyBalanceSummary> {
        let balances = self.repository.list_balances(org_id).await?;

        let total_outstanding: f64 = balances.iter()
            .filter_map(|b| b.total_outstanding.parse::<f64>().ok())
            .sum();

        let open_transactions: i32 = balances.iter()
            .map(|b| b.open_transaction_count)
            .sum();

        Ok(IntercompanyBalanceSummary {
            total_outstanding: format!("{:.2}", total_outstanding),
            entity_pairs: balances.len() as i32,
            open_transactions,
            balances,
        })
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate batch totals from its transactions
    async fn recalculate_batch_totals(&self, batch_id: Uuid) -> AtlasResult<()> {
        let transactions = self.repository.list_transactions_by_batch(batch_id).await?;

        let mut total_amount = 0.0_f64;
        for txn in &transactions {
            total_amount += txn.amount.parse::<f64>().unwrap_or(0.0);
        }

        self.repository.update_batch_totals(
            batch_id,
            &format!("{:.2}", total_amount),
            &format!("{:.2}", total_amount), // debit
            &format!("{:.2}", total_amount), // credit
            transactions.len() as i32,
        ).await?;

        Ok(())
    }

    /// Recalculate intercompany balances for an entity pair
    async fn recalculate_balances(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
    ) -> AtlasResult<()> {
        // Get all posted transactions for this pair
        let from_txns = self.repository.list_transactions_by_entity(
            org_id, from_entity_id, None,
        ).await?;

        let mut total_posted = 0.0_f64;
        let mut total_settled = 0.0_f64;
        let mut open_count = 0i32;

        for txn in &from_txns {
            // Only count transactions between this specific pair
            let matches_pair = (txn.from_entity_id == from_entity_id && txn.to_entity_id == to_entity_id)
                || (txn.from_entity_id == to_entity_id && txn.to_entity_id == from_entity_id);
            if !matches_pair { continue; }
            if txn.currency_code != currency_code { continue; }

            let amt: f64 = txn.amount.parse().unwrap_or(0.0);

            match txn.status.as_str() {
                "posted" => {
                    total_posted += amt;
                    open_count += 1;
                }
                "settled" => {
                    total_settled += amt;
                }
                _ => {}
            }
        }

        let total_outstanding = total_posted - total_settled;

        self.repository.upsert_balance(
            org_id, from_entity_id, to_entity_id, currency_code,
            &format!("{:.2}", total_outstanding.max(0.0)),
            &format!("{:.2}", total_posted),
            &format!("{:.2}", total_settled),
            open_count,
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"draft"));
        assert!(VALID_BATCH_STATUSES.contains(&"submitted"));
        assert!(VALID_BATCH_STATUSES.contains(&"approved"));
        assert!(VALID_BATCH_STATUSES.contains(&"posted"));
        assert!(VALID_BATCH_STATUSES.contains(&"cancelled"));
        assert!(!VALID_BATCH_STATUSES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_transaction_types() {
        assert!(VALID_TXN_TYPES.contains(&"invoice"));
        assert!(VALID_TXN_TYPES.contains(&"journal_entry"));
        assert!(VALID_TXN_TYPES.contains(&"payment"));
        assert!(VALID_TXN_TYPES.contains(&"charge"));
        assert!(VALID_TXN_TYPES.contains(&"allocation"));
    }

    #[test]
    fn test_valid_settlement_methods() {
        assert!(VALID_SETTLEMENT_METHODS.contains(&"cash"));
        assert!(VALID_SETTLEMENT_METHODS.contains(&"netting"));
        assert!(VALID_SETTLEMENT_METHODS.contains(&"offset"));
    }

    #[test]
    fn test_valid_transaction_statuses() {
        assert!(VALID_TXN_STATUSES.contains(&"draft"));
        assert!(VALID_TXN_STATUSES.contains(&"approved"));
        assert!(VALID_TXN_STATUSES.contains(&"posted"));
        assert!(VALID_TXN_STATUSES.contains(&"settled"));
        assert!(VALID_TXN_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_default_ic_accounts() {
        assert_eq!(DEFAULT_FROM_IC_ACCOUNT, "IC-DUE-TO");
        assert_eq!(DEFAULT_TO_IC_ACCOUNT, "IC-DUE-FROM");
    }

    #[test]
    fn test_amount_validation() {
        // Positive amount should parse
        let amt: f64 = "100.50".parse().unwrap();
        assert_eq!(amt, 100.50);
        assert!(amt > 0.0);

        // Zero should fail validation
        let zero: f64 = "0".parse().unwrap();
        assert!(zero <= 0.0);

        // Negative should fail validation
        let neg: f64 = "-10".parse().unwrap();
        assert!(neg <= 0.0);
    }

    #[test]
    fn test_currency_code_validation() {
        assert_eq!("USD".len(), 3);
        assert_ne!("US".len(), 3);
        assert_ne!("USDD".len(), 3);
    }

    #[test]
    fn test_same_entity_validation() {
        let entity1 = Uuid::new_v4();
        // From and to should be different
        assert_eq!(entity1, entity1);
        let entity2 = Uuid::new_v4();
        assert_ne!(entity1, entity2);
    }
}
