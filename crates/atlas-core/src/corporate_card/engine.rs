//! Corporate Card Management Engine
//!
//! Manages corporate credit card programmes, card issuance, transaction
//! processing, statement import, expense matching, spending limits,
//! and dispute handling.
//!
//! Card programme lifecycle: created → active / inactive
//! Card lifecycle: active → suspended → cancelled / expired / lost / stolen
//! Transaction lifecycle: unmatched → matched / disputed → approved / rejected
//! Statement lifecycle: imported → processing → matched → reconciled → paid
//! Limit override lifecycle: pending → approved / rejected / expired
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Expenses > Corporate Cards

use atlas_shared::{
    CorporateCardProgram, CorporateCard, CorporateCardTransaction,
    CorporateCardStatement, CorporateCardLimitOverride, CorporateCardDashboardSummary,
    AtlasError, AtlasResult,
};
use super::CorporateCardRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ── Valid values ────────────────────────────────────────────────────────

const VALID_CARD_TYPES: &[&str] = &["corporate", "purchasing", "travel"];
const VALID_CARD_NETWORKS: &[&str] = &["Visa", "Mastercard", "Amex", "Discover", "JCB"];
const VALID_MATCHING_METHODS: &[&str] = &["auto", "manual", "semi"];
const VALID_CARD_STATUSES: &[&str] = &["active", "suspended", "cancelled", "expired", "lost", "stolen"];
const VALID_TXN_TYPES: &[&str] = &["charge", "credit", "payment", "cash_withdrawal", "fee", "interest"];
const VALID_TXN_STATUSES: &[&str] = &["unmatched", "matched", "disputed", "approved", "rejected"];
const VALID_STMT_STATUSES: &[&str] = &["imported", "processing", "matched", "reconciled", "paid"];
const VALID_OVERRIDE_TYPES: &[&str] = &["single_purchase", "monthly", "cash", "atm"];
const VALID_OVERRIDE_STATUSES: &[&str] = &["pending", "approved", "rejected", "expired"];

/// Corporate Card Management Engine
pub struct CorporateCardEngine {
    repository: Arc<dyn CorporateCardRepository>,
}

impl CorporateCardEngine {
    pub fn new(repository: Arc<dyn CorporateCardRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Card Program Management
    // ========================================================================

    /// Create a new corporate card programme
    pub async fn create_program(
        &self,
        org_id: Uuid,
        program_code: &str,
        name: &str,
        description: Option<&str>,
        issuer_bank: &str,
        card_network: &str,
        card_type: &str,
        currency_code: &str,
        default_single_purchase_limit: &str,
        default_monthly_limit: &str,
        default_cash_limit: &str,
        default_atm_limit: &str,
        allow_cash_withdrawal: bool,
        allow_international: bool,
        auto_deactivate_on_termination: bool,
        expense_matching_method: &str,
        billing_cycle_day: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardProgram> {
        // Validate
        if program_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Program code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Program name is required".to_string()));
        }
        if issuer_bank.is_empty() {
            return Err(AtlasError::ValidationFailed("Issuer bank is required".to_string()));
        }
        if !VALID_CARD_NETWORKS.contains(&card_network) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid card network '{}'. Must be one of: {}",
                card_network,
                VALID_CARD_NETWORKS.join(", ")
            )));
        }
        if !VALID_CARD_TYPES.contains(&card_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid card type '{}'. Must be one of: {}",
                card_type,
                VALID_CARD_TYPES.join(", ")
            )));
        }
        if !VALID_MATCHING_METHODS.contains(&expense_matching_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid matching method '{}'. Must be one of: {}",
                expense_matching_method,
                VALID_MATCHING_METHODS.join(", ")
            )));
        }
        if !(1..=28).contains(&billing_cycle_day) {
            return Err(AtlasError::ValidationFailed(
                "Billing cycle day must be between 1 and 28".to_string(),
            ));
        }
        Self::validate_positive_amount(default_single_purchase_limit, "Default single purchase limit")?;
        Self::validate_positive_amount(default_monthly_limit, "Default monthly limit")?;
        Self::validate_positive_amount(default_cash_limit, "Default cash limit")?;
        Self::validate_positive_amount(default_atm_limit, "Default ATM limit")?;

        info!(
            "Creating corporate card program {} ({}) for org {}",
            program_code, name, org_id
        );

        self.repository
            .create_program(
                org_id, program_code, name, description, issuer_bank, card_network,
                card_type, currency_code, default_single_purchase_limit,
                default_monthly_limit, default_cash_limit, default_atm_limit,
                allow_cash_withdrawal, allow_international,
                auto_deactivate_on_termination, expense_matching_method,
                billing_cycle_day, created_by,
            )
            .await
    }

    /// Get a programme by code
    pub async fn get_program(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CorporateCardProgram>> {
        self.repository.get_program(org_id, code).await
    }

    /// List programmes
    pub async fn list_programs(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CorporateCardProgram>> {
        self.repository.list_programs(org_id, active_only).await
    }

    // ========================================================================
    // Card Management
    // ========================================================================

    /// Issue a new corporate card to an employee
    pub async fn issue_card(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        card_number_masked: &str,
        cardholder_name: &str,
        cardholder_id: Uuid,
        cardholder_email: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        issue_date: chrono::NaiveDate,
        expiry_date: chrono::NaiveDate,
        gl_liability_account: Option<&str>,
        gl_expense_account: Option<&str>,
        cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCard> {
        // Validate programme exists
        let program = self
            .repository
            .get_program_by_id(program_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Program {} not found", program_id)))?;

        if !program.is_active {
            return Err(AtlasError::ValidationFailed(format!(
                "Program '{}' is not active",
                program.name
            )));
        }

        if card_number_masked.is_empty() {
            return Err(AtlasError::ValidationFailed("Card number (masked) is required".to_string()));
        }
        if cardholder_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Cardholder name is required".to_string()));
        }
        if expiry_date <= issue_date {
            return Err(AtlasError::ValidationFailed(
                "Expiry date must be after issue date".to_string(),
            ));
        }

        info!(
            "Issuing card {} to {} (program: {})",
            card_number_masked, cardholder_name, program.program_code
        );

        self.repository
            .create_card(
                org_id,
                program_id,
                card_number_masked,
                cardholder_name,
                cardholder_id,
                cardholder_email,
                department_id,
                department_name,
                "active",
                issue_date,
                expiry_date,
                &program.default_single_purchase_limit,
                &program.default_monthly_limit,
                &program.default_cash_limit,
                &program.default_atm_limit,
                gl_liability_account,
                gl_expense_account,
                cost_center,
                created_by,
            )
            .await
    }

    /// Get card by ID
    pub async fn get_card(&self, id: Uuid) -> AtlasResult<Option<CorporateCard>> {
        self.repository.get_card(id).await
    }

    /// List cards with optional filters
    pub async fn list_cards(
        &self,
        org_id: Uuid,
        program_id: Option<Uuid>,
        cardholder_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCard>> {
        if let Some(s) = status {
            if !VALID_CARD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid card status '{}'. Must be one of: {}",
                    s,
                    VALID_CARD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_cards(org_id, program_id, cardholder_id, status).await
    }

    /// Suspend a card
    pub async fn suspend_card(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        let card = self.get_card_or_error(card_id).await?;

        if card.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot suspend card in '{}' status. Must be 'active'.",
                card.status
            )));
        }

        info!("Suspended card {} ({})", card.card_number_masked, card.cardholder_name);
        self.repository.update_card_status(card_id, "suspended").await
    }

    /// Reactivate a suspended card
    pub async fn reactivate_card(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reactivate card in '{}' status. Must be 'suspended'.",
                card.status
            )));
        }

        info!("Reactivated card {} ({})", card.card_number_masked, card.cardholder_name);
        self.repository.update_card_status(card_id, "active").await
    }

    /// Cancel a card
    pub async fn cancel_card(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "active" && card.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel card in '{}' status. Must be 'active' or 'suspended'.",
                card.status
            )));
        }

        info!("Cancelled card {} ({})", card.card_number_masked, card.cardholder_name);
        self.repository.update_card_status(card_id, "cancelled").await
    }

    /// Report card lost
    pub async fn report_lost(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot report card lost in '{}' status. Must be 'active'.",
                card.status
            )));
        }

        info!("Reported card {} as lost ({})", card.card_number_masked, card.cardholder_name);
        self.repository.update_card_status(card_id, "lost").await
    }

    /// Report card stolen
    pub async fn report_stolen(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot report card stolen in '{}' status. Must be 'active'.",
                card.status
            )));
        }

        info!("Reported card {} as stolen ({})", card.card_number_masked, card.cardholder_name);
        self.repository.update_card_status(card_id, "stolen").await
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    /// Import a card transaction (typically from issuer feed)
    pub async fn import_transaction(
        &self,
        org_id: Uuid,
        card_id: Uuid,
        transaction_reference: &str,
        posting_date: chrono::NaiveDate,
        transaction_date: chrono::NaiveDate,
        merchant_name: &str,
        merchant_category: Option<&str>,
        merchant_category_code: Option<&str>,
        amount: &str,
        currency_code: &str,
        original_amount: Option<&str>,
        original_currency: Option<&str>,
        exchange_rate: Option<&str>,
        transaction_type: &str,
    ) -> AtlasResult<CorporateCardTransaction> {
        if !VALID_TXN_TYPES.contains(&transaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction type '{}'. Must be one of: {}",
                transaction_type,
                VALID_TXN_TYPES.join(", ")
            )));
        }

        // Validate card exists and is usable
        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot import transaction for card in '{}' status.",
                card.status
            )));
        }

        // Validate spending limits for charge transactions
        if transaction_type == "charge" || transaction_type == "cash_withdrawal" {
            self.validate_spending_limit(&card, amount, transaction_type)?;
        }

        Self::validate_positive_amount(amount, "Amount")?;

        info!(
            "Importing transaction {} on card {} ({})",
            transaction_reference, card.card_number_masked, merchant_name
        );

        let txn = self
            .repository
            .create_transaction(
                org_id,
                card_id,
                card.program_id,
                transaction_reference,
                posting_date,
                transaction_date,
                merchant_name,
                merchant_category,
                merchant_category_code,
                amount,
                currency_code,
                original_amount,
                original_currency,
                exchange_rate,
                transaction_type,
            )
            .await?;

        // Update card spend tracking
        let amount_val: f64 = amount.parse().unwrap_or(0.0);
        let current_spend: f64 = card.total_spend_current_cycle.parse().unwrap_or(0.0);
        let current_balance: f64 = card.current_balance.parse().unwrap_or(0.0);

        if transaction_type == "charge" || transaction_type == "cash_withdrawal" || transaction_type == "fee" || transaction_type == "interest" {
            let new_spend = current_spend + amount_val;
            let new_balance = current_balance + amount_val;
            self.repository
                .update_card_spend(card_id, &format!("{:.2}", new_spend), &format!("{:.2}", new_balance))
                .await?;
        } else if transaction_type == "credit" || transaction_type == "payment" {
            let new_balance = (current_balance - amount_val).max(0.0);
            self.repository
                .update_card_spend(card_id, &format!("{:.2}", current_spend), &format!("{:.2}", new_balance))
                .await?;
        }

        Ok(txn)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CorporateCardTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// List transactions with optional filters
    pub async fn list_transactions(
        &self,
        org_id: Uuid,
        card_id: Option<Uuid>,
        status: Option<&str>,
        date_from: Option<chrono::NaiveDate>,
        date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<CorporateCardTransaction>> {
        if let Some(s) = status {
            if !VALID_TXN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid transaction status '{}'. Must be one of: {}",
                    s,
                    VALID_TXN_STATUSES.join(", ")
                )));
            }
        }
        self.repository
            .list_transactions(org_id, card_id, status, date_from, date_to)
            .await
    }

    /// Match a card transaction to an expense report line
    pub async fn match_transaction(
        &self,
        transaction_id: Uuid,
        expense_report_id: Uuid,
        expense_line_id: Option<Uuid>,
        matched_by: Option<Uuid>,
        match_confidence: Option<&str>,
    ) -> AtlasResult<CorporateCardTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Transaction {} not found", transaction_id
            )))?;

        if txn.status != "unmatched" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot match transaction in '{}' status. Must be 'unmatched'.",
                txn.status
            )));
        }

        info!(
            "Matched transaction {} to expense report {}",
            txn.transaction_reference, expense_report_id
        );

        self.repository
            .update_transaction_match(
                transaction_id,
                Some(expense_report_id),
                expense_line_id,
                "matched",
                matched_by,
                match_confidence,
            )
            .await
    }

    /// Unmatch a previously matched transaction
    pub async fn unmatch_transaction(
        &self,
        transaction_id: Uuid,
    ) -> AtlasResult<CorporateCardTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Transaction {} not found", transaction_id
            )))?;

        if txn.status != "matched" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot unmatch transaction in '{}' status. Must be 'matched'.",
                txn.status
            )));
        }

        info!("Unmatched transaction {}", txn.transaction_reference);

        self.repository
            .update_transaction_match(transaction_id, None, None, "unmatched", None, None)
            .await
    }

    /// Dispute a transaction
    pub async fn dispute_transaction(
        &self,
        transaction_id: Uuid,
        reason: &str,
    ) -> AtlasResult<CorporateCardTransaction> {
        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Transaction {} not found", transaction_id
            )))?;

        if txn.status != "unmatched" && txn.status != "matched" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot dispute transaction in '{}' status.",
                txn.status
            )));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Dispute reason is required".to_string(),
            ));
        }

        info!(
            "Disputed transaction {} ({}): {}",
            txn.transaction_reference, txn.merchant_name, reason
        );

        self.repository
            .update_transaction_dispute(
                transaction_id,
                Some(reason),
                Some(chrono::Utc::now().date_naive()),
                None,
                "disputed",
            )
            .await
    }

    /// Resolve a disputed transaction
    pub async fn resolve_dispute(
        &self,
        transaction_id: Uuid,
        resolution: &str,
        resolved_status: &str,
    ) -> AtlasResult<CorporateCardTransaction> {
        if !resolved_status.eq("approved") && !resolved_status.eq("rejected") {
            return Err(AtlasError::ValidationFailed(
                "Resolution status must be 'approved' or 'rejected'".to_string(),
            ));
        }

        let txn = self.repository.get_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Transaction {} not found", transaction_id
            )))?;

        if txn.status != "disputed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot resolve dispute for transaction in '{}' status. Must be 'disputed'.",
                txn.status
            )));
        }

        info!(
            "Resolved dispute for transaction {} as {}",
            txn.transaction_reference, resolved_status
        );

        self.repository
            .update_transaction_dispute(
                transaction_id,
                None,
                None,
                Some(resolution),
                resolved_status,
            )
            .await
    }

    // ========================================================================
    // Statements
    // ========================================================================

    /// Import a card statement
    pub async fn import_statement(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        statement_number: &str,
        statement_date: chrono::NaiveDate,
        billing_period_start: chrono::NaiveDate,
        billing_period_end: chrono::NaiveDate,
        opening_balance: &str,
        closing_balance: &str,
        total_charges: &str,
        total_credits: &str,
        total_payments: &str,
        total_fees: &str,
        total_interest: &str,
        payment_due_date: Option<chrono::NaiveDate>,
        minimum_payment: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardStatement> {
        if statement_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Statement number is required".to_string(),
            ));
        }
        if billing_period_end <= billing_period_start {
            return Err(AtlasError::ValidationFailed(
                "Billing period end must be after start".to_string(),
            ));
        }

        // Validate programme exists
        let program = self.repository.get_program_by_id(program_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Program {} not found", program_id)))?;

        info!(
            "Importing statement {} for program {} (period: {} to {})",
            statement_number, program.program_code, billing_period_start, billing_period_end
        );

        let stmt = self
            .repository
            .create_statement(
                org_id,
                program_id,
                statement_number,
                statement_date,
                billing_period_start,
                billing_period_end,
                opening_balance,
                closing_balance,
                total_charges,
                total_credits,
                total_payments,
                total_fees,
                total_interest,
                payment_due_date,
                minimum_payment,
                imported_by,
            )
            .await?;

        // Auto-match transactions within the billing period
        let transactions = self
            .repository
            .list_transactions(
                org_id,
                None,
                Some("unmatched"),
                Some(billing_period_start),
                Some(billing_period_end),
            )
            .await?;

        let matched_count = transactions.len() as i32;
        // In a real implementation, we'd do fuzzy matching here.
        // For now, just update the counts.
        self.repository
            .update_statement_counts(
                stmt.id,
                transactions.len() as i32,
                matched_count,
                0,
            )
            .await?;

        Ok(stmt)
    }

    /// Get a statement
    pub async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CorporateCardStatement>> {
        self.repository.get_statement(id).await
    }

    /// List statements
    pub async fn list_statements(
        &self,
        org_id: Uuid,
        program_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardStatement>> {
        if let Some(s) = status {
            if !VALID_STMT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid statement status '{}'. Must be one of: {}",
                    s,
                    VALID_STMT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_statements(org_id, program_id, status).await
    }

    /// Mark a statement as reconciled
    pub async fn reconcile_statement(&self, statement_id: Uuid) -> AtlasResult<CorporateCardStatement> {
        let stmt = self.repository.get_statement(statement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Statement {} not found", statement_id
            )))?;

        if stmt.status != "matched" && stmt.status != "processing" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reconcile statement in '{}' status.",
                stmt.status
            )));
        }

        info!("Reconciled statement {}", stmt.statement_number);
        self.repository.update_statement_status(statement_id, "reconciled", None).await
    }

    /// Record payment for a statement
    pub async fn pay_statement(
        &self,
        statement_id: Uuid,
        payment_reference: &str,
    ) -> AtlasResult<CorporateCardStatement> {
        let stmt = self.repository.get_statement(statement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Statement {} not found", statement_id
            )))?;

        if stmt.status != "reconciled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot pay statement in '{}' status. Must be 'reconciled'.",
                stmt.status
            )));
        }

        if payment_reference.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payment reference is required".to_string(),
            ));
        }

        info!(
            "Paid statement {} with reference {}",
            stmt.statement_number, payment_reference
        );

        self.repository
            .update_statement_status(statement_id, "paid", Some(payment_reference))
            .await
    }

    // ========================================================================
    // Spending Limit Overrides
    // ========================================================================

    /// Request a spending limit override
    pub async fn request_limit_override(
        &self,
        org_id: Uuid,
        card_id: Uuid,
        override_type: &str,
        new_value: &str,
        reason: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardLimitOverride> {
        if !VALID_OVERRIDE_TYPES.contains(&override_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid override type '{}'. Must be one of: {}",
                override_type,
                VALID_OVERRIDE_TYPES.join(", ")
            )));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reason is required for limit override".to_string(),
            ));
        }

        let card = self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))?;

        if card.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot request limit override for card in '{}' status.",
                card.status
            )));
        }

        Self::validate_positive_amount(new_value, "New limit value")?;

        let original_value = match override_type {
            "single_purchase" => card.single_purchase_limit.clone(),
            "monthly" => card.monthly_limit.clone(),
            "cash" => card.cash_limit.clone(),
            "atm" => card.atm_limit.clone(),
            _ => "0".to_string(),
        };

        info!(
            "Requesting {} limit override for card {} ({} → {})",
            override_type, card.card_number_masked, original_value, new_value
        );

        self.repository
            .create_limit_override(
                org_id,
                card_id,
                override_type,
                &original_value,
                new_value,
                reason,
                effective_from,
                effective_to,
                created_by,
            )
            .await
    }

    /// Approve a spending limit override
    pub async fn approve_limit_override(
        &self,
        override_id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<CorporateCardLimitOverride> {
        let limit = self.repository.get_limit_override(override_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Limit override {} not found", override_id
            )))?;

        if limit.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve limit override in '{}' status. Must be 'pending'.",
                limit.status
            )));
        }

        // Apply the limit change to the card
        let card = self.repository.get_card(limit.card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Card {} not found", limit.card_id
            )))?;

        let (single, monthly, cash, atm) = match limit.override_type.as_str() {
            "single_purchase" => (
                limit.new_value.clone(),
                card.monthly_limit.clone(),
                card.cash_limit.clone(),
                card.atm_limit.clone(),
            ),
            "monthly" => (
                card.single_purchase_limit.clone(),
                limit.new_value.clone(),
                card.cash_limit.clone(),
                card.atm_limit.clone(),
            ),
            "cash" => (
                card.single_purchase_limit.clone(),
                card.monthly_limit.clone(),
                limit.new_value.clone(),
                card.atm_limit.clone(),
            ),
            "atm" => (
                card.single_purchase_limit.clone(),
                card.monthly_limit.clone(),
                card.cash_limit.clone(),
                limit.new_value.clone(),
            ),
            _ => return Err(AtlasError::ValidationFailed("Invalid override type".to_string())),
        };

        self.repository
            .update_card_limits(limit.card_id, &single, &monthly, &cash, &atm)
            .await?;

        info!(
            "Approved {} limit override for card {} (new: {})",
            limit.override_type, card.card_number_masked, limit.new_value
        );

        self.repository
            .update_limit_override_status(override_id, "approved", Some(approved_by))
            .await
    }

    /// Reject a spending limit override
    pub async fn reject_limit_override(
        &self,
        override_id: Uuid,
        rejected_by: Uuid,
    ) -> AtlasResult<CorporateCardLimitOverride> {
        let limit = self.repository.get_limit_override(override_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Limit override {} not found", override_id
            )))?;

        if limit.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject limit override in '{}' status. Must be 'pending'.",
                limit.status
            )));
        }

        info!("Rejected limit override {} for card", limit.card_id);

        self.repository
            .update_limit_override_status(override_id, "rejected", Some(rejected_by))
            .await
    }

    /// List limit overrides
    pub async fn list_limit_overrides(
        &self,
        card_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardLimitOverride>> {
        if let Some(s) = status {
            if !VALID_OVERRIDE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid override status '{}'. Must be one of: {}",
                    s,
                    VALID_OVERRIDE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_limit_overrides(card_id, status).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get corporate card dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CorporateCardDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Helper: fetch card or return EntityNotFound
    async fn get_card_or_error(&self, card_id: Uuid) -> AtlasResult<CorporateCard> {
        self.repository.get_card(card_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Card {} not found", card_id)))
    }

    /// Validate a spending limit against card limits
    fn validate_spending_limit(&self, card: &CorporateCard, amount: &str, txn_type: &str) -> AtlasResult<()> {
        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        // Check single purchase limit
        let single_limit: f64 = card.single_purchase_limit.parse().unwrap_or(0.0);
        if single_limit > 0.0 && amount_val > single_limit {
            return Err(AtlasError::ValidationFailed(format!(
                "Amount {:.2} exceeds single purchase limit {:.2}",
                amount_val, single_limit
            )));
        }

        // Check monthly limit
        if txn_type == "charge" {
            let monthly_limit: f64 = card.monthly_limit.parse().unwrap_or(0.0);
            let current_spend: f64 = card.total_spend_current_cycle.parse().unwrap_or(0.0);
            if monthly_limit > 0.0 && (current_spend + amount_val) > monthly_limit {
                return Err(AtlasError::ValidationFailed(format!(
                    "Transaction would exceed monthly limit ({:.2} + {:.2} > {:.2})",
                    current_spend, amount_val, monthly_limit
                )));
            }
        }

        // Check cash limit for cash withdrawals
        if txn_type == "cash_withdrawal" {
            let cash_limit: f64 = card.cash_limit.parse().unwrap_or(0.0);
            if cash_limit > 0.0 && amount_val > cash_limit {
                return Err(AtlasError::ValidationFailed(format!(
                    "Cash withdrawal {:.2} exceeds cash limit {:.2}",
                    amount_val, cash_limit
                )));
            }
        }

        Ok(())
    }

    /// Validate that a string parses to a non-negative amount
    fn validate_positive_amount(value: &str, field: &str) -> AtlasResult<()> {
        let v: f64 = value.parse().map_err(|_| AtlasError::ValidationFailed(format!(
            "{} must be a valid number", field
        )))?;
        if v < 0.0 {
            return Err(AtlasError::ValidationFailed(format!(
                "{} cannot be negative", field
            )));
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_card_types() {
        assert!(VALID_CARD_TYPES.contains(&"corporate"));
        assert!(VALID_CARD_TYPES.contains(&"purchasing"));
        assert!(VALID_CARD_TYPES.contains(&"travel"));
    }

    #[test]
    fn test_valid_card_networks() {
        assert!(VALID_CARD_NETWORKS.contains(&"Visa"));
        assert!(VALID_CARD_NETWORKS.contains(&"Mastercard"));
        assert!(VALID_CARD_NETWORKS.contains(&"Amex"));
    }

    #[test]
    fn test_valid_card_statuses() {
        assert!(VALID_CARD_STATUSES.contains(&"active"));
        assert!(VALID_CARD_STATUSES.contains(&"suspended"));
        assert!(VALID_CARD_STATUSES.contains(&"cancelled"));
        assert!(VALID_CARD_STATUSES.contains(&"expired"));
        assert!(VALID_CARD_STATUSES.contains(&"lost"));
        assert!(VALID_CARD_STATUSES.contains(&"stolen"));
    }

    #[test]
    fn test_valid_transaction_types() {
        assert!(VALID_TXN_TYPES.contains(&"charge"));
        assert!(VALID_TXN_TYPES.contains(&"credit"));
        assert!(VALID_TXN_TYPES.contains(&"payment"));
        assert!(VALID_TXN_TYPES.contains(&"cash_withdrawal"));
        assert!(VALID_TXN_TYPES.contains(&"fee"));
        assert!(VALID_TXN_TYPES.contains(&"interest"));
    }

    #[test]
    fn test_valid_transaction_statuses() {
        assert!(VALID_TXN_STATUSES.contains(&"unmatched"));
        assert!(VALID_TXN_STATUSES.contains(&"matched"));
        assert!(VALID_TXN_STATUSES.contains(&"disputed"));
        assert!(VALID_TXN_STATUSES.contains(&"approved"));
        assert!(VALID_TXN_STATUSES.contains(&"rejected"));
    }

    #[test]
    fn test_valid_statement_statuses() {
        assert!(VALID_STMT_STATUSES.contains(&"imported"));
        assert!(VALID_STMT_STATUSES.contains(&"processing"));
        assert!(VALID_STMT_STATUSES.contains(&"matched"));
        assert!(VALID_STMT_STATUSES.contains(&"reconciled"));
        assert!(VALID_STMT_STATUSES.contains(&"paid"));
    }

    #[test]
    fn test_valid_override_types() {
        assert!(VALID_OVERRIDE_TYPES.contains(&"single_purchase"));
        assert!(VALID_OVERRIDE_TYPES.contains(&"monthly"));
        assert!(VALID_OVERRIDE_TYPES.contains(&"cash"));
        assert!(VALID_OVERRIDE_TYPES.contains(&"atm"));
    }

    #[test]
    fn test_valid_override_statuses() {
        assert!(VALID_OVERRIDE_STATUSES.contains(&"pending"));
        assert!(VALID_OVERRIDE_STATUSES.contains(&"approved"));
        assert!(VALID_OVERRIDE_STATUSES.contains(&"rejected"));
        assert!(VALID_OVERRIDE_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_validate_positive_amount_valid() {
        assert!(CorporateCardEngine::validate_positive_amount("100.00", "test").is_ok());
        assert!(CorporateCardEngine::validate_positive_amount("0", "test").is_ok());
        assert!(CorporateCardEngine::validate_positive_amount("999999.99", "test").is_ok());
    }

    #[test]
    fn test_validate_positive_amount_negative() {
        assert!(CorporateCardEngine::validate_positive_amount("-1.00", "test").is_err());
    }

    #[test]
    fn test_validate_positive_amount_invalid() {
        assert!(CorporateCardEngine::validate_positive_amount("abc", "test").is_err());
        assert!(CorporateCardEngine::validate_positive_amount("", "test").is_err());
    }

    #[test]
    fn test_validate_spending_limit_within_limits() {
        let engine = CorporateCardEngine::new(Arc::new(crate::MockCorporateCardRepository));
        let card = CorporateCard {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            card_number_masked: "****-1234".to_string(),
            cardholder_name: "John Doe".to_string(),
            cardholder_id: Uuid::new_v4(),
            cardholder_email: None,
            department_id: None,
            department_name: None,
            status: "active".to_string(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            single_purchase_limit: "5000.00".to_string(),
            monthly_limit: "20000.00".to_string(),
            cash_limit: "1000.00".to_string(),
            atm_limit: "500.00".to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "3000.00".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: None,
            gl_expense_account: None,
            cost_center: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Should pass: well within all limits
        assert!(engine.validate_spending_limit(&card, "100.00", "charge").is_ok());
        // Should pass: exactly at single purchase limit
        assert!(engine.validate_spending_limit(&card, "5000.00", "charge").is_ok());
    }

    #[test]
    fn test_validate_spending_limit_exceeds_single_purchase() {
        let engine = CorporateCardEngine::new(Arc::new(crate::MockCorporateCardRepository));
        let card = CorporateCard {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            card_number_masked: "****-5678".to_string(),
            cardholder_name: "Jane Smith".to_string(),
            cardholder_id: Uuid::new_v4(),
            cardholder_email: None,
            department_id: None,
            department_name: None,
            status: "active".to_string(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            single_purchase_limit: "5000.00".to_string(),
            monthly_limit: "20000.00".to_string(),
            cash_limit: "1000.00".to_string(),
            atm_limit: "500.00".to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "0".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: None,
            gl_expense_account: None,
            cost_center: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.validate_spending_limit(&card, "6000.00", "charge");
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("single purchase limit"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_validate_spending_limit_exceeds_monthly() {
        let engine = CorporateCardEngine::new(Arc::new(crate::MockCorporateCardRepository));
        let card = CorporateCard {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            card_number_masked: "****-9999".to_string(),
            cardholder_name: "Bob Jones".to_string(),
            cardholder_id: Uuid::new_v4(),
            cardholder_email: None,
            department_id: None,
            department_name: None,
            status: "active".to_string(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            single_purchase_limit: "50000.00".to_string(),
            monthly_limit: "10000.00".to_string(),
            cash_limit: "1000.00".to_string(),
            atm_limit: "500.00".to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "9500.00".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: None,
            gl_expense_account: None,
            cost_center: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.validate_spending_limit(&card, "1000.00", "charge");
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("monthly limit"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_validate_spending_limit_cash_exceeds() {
        let engine = CorporateCardEngine::new(Arc::new(crate::MockCorporateCardRepository));
        let card = CorporateCard {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            card_number_masked: "****-0000".to_string(),
            cardholder_name: "Alice Brown".to_string(),
            cardholder_id: Uuid::new_v4(),
            cardholder_email: None,
            department_id: None,
            department_name: None,
            status: "active".to_string(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            single_purchase_limit: "5000.00".to_string(),
            monthly_limit: "20000.00".to_string(),
            cash_limit: "500.00".to_string(),
            atm_limit: "500.00".to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "0".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: None,
            gl_expense_account: None,
            cost_center: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.validate_spending_limit(&card, "600.00", "cash_withdrawal");
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("cash limit"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_validate_spending_limit_zero_means_unlimited() {
        let engine = CorporateCardEngine::new(Arc::new(crate::MockCorporateCardRepository));
        let card = CorporateCard {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            program_id: Uuid::new_v4(),
            card_number_masked: "****-0001".to_string(),
            cardholder_name: "No Limit".to_string(),
            cardholder_id: Uuid::new_v4(),
            cardholder_email: None,
            department_id: None,
            department_name: None,
            status: "active".to_string(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            single_purchase_limit: "0".to_string(),
            monthly_limit: "0".to_string(),
            cash_limit: "0".to_string(),
            atm_limit: "0".to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "0".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: None,
            gl_expense_account: None,
            cost_center: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Zero limits mean unlimited - any amount should pass
        assert!(engine.validate_spending_limit(&card, "999999.99", "charge").is_ok());
    }

    #[test]
    fn test_valid_matching_methods() {
        assert!(VALID_MATCHING_METHODS.contains(&"auto"));
        assert!(VALID_MATCHING_METHODS.contains(&"manual"));
        assert!(VALID_MATCHING_METHODS.contains(&"semi"));
    }
}
