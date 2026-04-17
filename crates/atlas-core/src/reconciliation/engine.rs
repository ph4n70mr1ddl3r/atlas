//! Bank Reconciliation Engine Implementation
//!
//! Manages bank accounts, bank statement import, auto-matching,
//! manual matching, and reconciliation summary generation.
//!
//! Oracle Fusion Cloud ERP equivalent: Cash Management > Bank Statements and Reconciliation

use atlas_shared::{
    BankAccount, BankStatement, BankStatementLine, SystemTransaction,
    ReconciliationMatch, ReconciliationSummary, ReconciliationMatchingRule,
    AutoMatchResult, AutoMatchPair,
    AtlasError, AtlasResult,
};
use super::ReconciliationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Bank Reconciliation engine
pub struct ReconciliationEngine {
    repository: Arc<dyn ReconciliationRepository>,
}

impl ReconciliationEngine {
    pub fn new(repository: Arc<dyn ReconciliationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Bank Account Management
    // ========================================================================

    /// Create a new bank account
    pub async fn create_bank_account(
        &self,
        org_id: Uuid,
        account_number: &str,
        account_name: &str,
        bank_name: &str,
        bank_code: Option<&str>,
        branch_name: Option<&str>,
        branch_code: Option<&str>,
        gl_account_code: Option<&str>,
        currency_code: &str,
        account_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccount> {
        info!("Creating bank account '{}' for org {}", account_number, org_id);

        if account_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "account_number is required".to_string(),
            ));
        }
        if account_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "account_name is required".to_string(),
            ));
        }

        self.repository
            .create_bank_account(
                org_id,
                account_number,
                account_name,
                bank_name,
                bank_code,
                branch_name,
                branch_code,
                gl_account_code,
                currency_code,
                account_type,
                created_by,
            )
            .await
    }

    /// Get a bank account by ID
    pub async fn get_bank_account(&self, id: Uuid) -> AtlasResult<Option<BankAccount>> {
        self.repository.get_bank_account(id).await
    }

    /// List bank accounts for an organization
    pub async fn list_bank_accounts(&self, org_id: Uuid) -> AtlasResult<Vec<BankAccount>> {
        self.repository.list_bank_accounts(org_id).await
    }

    /// Delete (soft-delete) a bank account
    pub async fn delete_bank_account(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting bank account {}", id);
        self.repository.delete_bank_account(id).await
    }

    // ========================================================================
    // Bank Statement Management
    // ========================================================================

    /// Create a bank statement with line items
    pub async fn create_bank_statement(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        statement_number: &str,
        statement_date: chrono::NaiveDate,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        opening_balance: &str,
        closing_balance: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<BankStatement> {
        info!("Creating bank statement '{}' for account {}", statement_number, bank_account_id);

        if start_date > end_date {
            return Err(AtlasError::ValidationFailed(
                "start_date must be before or equal to end_date".to_string(),
            ));
        }

        // Verify bank account exists and belongs to this org
        let account = self
            .repository
            .get_bank_account(bank_account_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Bank account {}", bank_account_id)))?;

        if account.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Bank account does not belong to this organization".to_string(),
            ));
        }

        self.repository
            .create_bank_statement(
                org_id,
                bank_account_id,
                statement_number,
                statement_date,
                start_date,
                end_date,
                opening_balance,
                closing_balance,
                imported_by,
            )
            .await
    }

    /// Get a bank statement by ID
    pub async fn get_bank_statement(&self, id: Uuid) -> AtlasResult<Option<BankStatement>> {
        self.repository.get_bank_statement(id).await
    }

    /// List bank statements for a bank account
    pub async fn list_bank_statements(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<BankStatement>> {
        self.repository
            .list_bank_statements(org_id, bank_account_id)
            .await
    }

    // ========================================================================
    // Bank Statement Lines
    // ========================================================================

    /// Add a line to a bank statement
    #[allow(clippy::too_many_arguments)]
    pub async fn add_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_number: i32,
        transaction_date: chrono::NaiveDate,
        transaction_type: &str,
        amount: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        counterparty_account: Option<&str>,
    ) -> AtlasResult<BankStatementLine> {
        // Validate transaction_type
        let valid_types = [
            "deposit", "withdrawal", "interest", "charge",
            "transfer_in", "transfer_out", "adjustment",
        ];
        if !valid_types.contains(&transaction_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction_type '{}'. Must be one of: {}",
                transaction_type,
                valid_types.join(", ")
            )));
        }

        self.repository
            .create_statement_line(
                org_id,
                statement_id,
                line_number,
                transaction_date,
                transaction_type,
                amount,
                description,
                reference_number,
                check_number,
                counterparty_name,
                counterparty_account,
            )
            .await
    }

    /// List statement lines for a bank statement
    pub async fn list_statement_lines(
        &self,
        statement_id: Uuid,
    ) -> AtlasResult<Vec<BankStatementLine>> {
        self.repository.list_statement_lines(statement_id).await
    }

    // ========================================================================
    // System Transactions
    // ========================================================================

    /// Create a system transaction (AP payment, AR receipt, GL entry)
    #[allow(clippy::too_many_arguments)]
    pub async fn create_system_transaction(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        source_type: &str,
        source_id: Uuid,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        amount: &str,
        transaction_type: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SystemTransaction> {
        let valid_source_types = ["ap_payment", "ar_receipt", "gl_journal", "cash_transfer"];
        if !valid_source_types.contains(&source_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source_type '{}'. Must be one of: {}",
                source_type,
                valid_source_types.join(", ")
            )));
        }

        self.repository
            .create_system_transaction(
                org_id,
                bank_account_id,
                source_type,
                source_id,
                source_number,
                transaction_date,
                amount,
                transaction_type,
                description,
                reference_number,
                check_number,
                counterparty_name,
                created_by,
            )
            .await
    }

    /// List unreconciled system transactions for a bank account
    pub async fn list_unreconciled_transactions(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<SystemTransaction>> {
        self.repository
            .list_unreconciled_transactions(org_id, bank_account_id)
            .await
    }

    /// Get a system transaction by ID
    pub async fn get_system_transaction(&self, id: Uuid) -> AtlasResult<Option<SystemTransaction>> {
        self.repository.get_system_transaction(id).await
    }

    // ========================================================================
    // Auto-Matching
    // ========================================================================

    /// Run auto-matching for a bank statement against system transactions.
    ///
    /// Matching strategies (in order):
    /// 1. Exact check number match
    /// 2. Exact reference number match
    /// 3. Amount + date match (within tolerance)
    pub async fn auto_match(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        matched_by: Option<Uuid>,
    ) -> AtlasResult<AutoMatchResult> {
        info!("Running auto-match for statement {}", statement_id);

        let statement = self
            .repository
            .get_bank_statement(statement_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Statement {}", statement_id)))?;

        if statement.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Statement does not belong to this organization".to_string(),
            ));
        }

        let lines = self.repository.list_statement_lines(statement_id).await?;
        let transactions = self
            .repository
            .list_unreconciled_transactions(org_id, statement.bank_account_id)
            .await?;

        let mut result = AutoMatchResult {
            total_lines: lines.len() as i32,
            matched: 0,
            unmatched: 0,
            already_matched: 0,
            matches: Vec::new(),
        };

        let mut used_transactions: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        for line in &lines {
            if line.match_status != "unmatched" {
                result.already_matched += 1;
                continue;
            }

            let line_amount: f64 = line.amount
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let _line_amount_abs = line_amount.abs();

            // Strategy 1: Check number exact match
            if let Some(ref check_num) = line.check_number {
                if let Some(txn) = transactions.iter().find(|t| {
                    t.status == "unreconciled"
                        && !used_transactions.contains(&t.id)
                        && t.check_number.as_deref() == Some(check_num.as_str())
                }) {
                    result.matches.push(AutoMatchPair {
                        statement_line_id: line.id,
                        system_transaction_id: txn.id,
                        match_method: "auto_check".to_string(),
                        confidence: 100.0,
                    });
                    used_transactions.insert(txn.id);
                    result.matched += 1;
                    continue;
                }
            }

            // Strategy 2: Reference number exact match
            if let Some(ref ref_num) = line.reference_number {
                if let Some(txn) = transactions.iter().find(|t| {
                    t.status == "unreconciled"
                        && !used_transactions.contains(&t.id)
                        && t.reference_number.as_deref() == Some(ref_num.as_str())
                }) {
                    result.matches.push(AutoMatchPair {
                        statement_line_id: line.id,
                        system_transaction_id: txn.id,
                        match_method: "auto_reference".to_string(),
                        confidence: 95.0,
                    });
                    used_transactions.insert(txn.id);
                    result.matched += 1;
                    continue;
                }
            }

            // Strategy 3: Amount + date match (within 3 days tolerance)
            if let Some(txn) = transactions.iter().find(|t| {
                if t.status != "unreconciled" || used_transactions.contains(&t.id) {
                    return false;
                }
                let txn_amount: f64 = t.amount
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                if (txn_amount - line_amount).abs() > 0.01 {
                    return false;
                }
                // Date within 3 days tolerance
                let date_diff = (t.transaction_date - line.transaction_date).num_days().abs();
                date_diff <= 3
            }) {
                result.matches.push(AutoMatchPair {
                    statement_line_id: line.id,
                    system_transaction_id: txn.id,
                    match_method: "auto_amount_date".to_string(),
                    confidence: 80.0,
                });
                used_transactions.insert(txn.id);
                result.matched += 1;
                continue;
            }

            result.unmatched += 1;
        }

        // Persist matches
        for pair in &result.matches {
            if let Err(e) = self
                .repository
                .create_match(
                    org_id,
                    statement_id,
                    pair.statement_line_id,
                    pair.system_transaction_id,
                    &pair.match_method,
                    Some(pair.confidence),
                    matched_by,
                )
                .await
            {
                tracing::warn!("Failed to create match: {}", e);
            }
        }

        // Update statement line counts
        let total_matched = self
            .repository
            .list_statement_lines(statement_id)
            .await?
            .iter()
            .filter(|l| l.match_status == "matched" || l.match_status == "manually_matched")
            .count() as i32;

        let total_lines = lines.len() as i32;
        let unmatched_lines = total_lines - total_matched;
        let recon_percent = if total_lines > 0 {
            (total_matched as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        self.repository
            .update_statement_counts(statement_id, total_lines, total_matched, unmatched_lines, recon_percent)
            .await?;

        // Auto-transition status
        if recon_percent >= 100.0 {
            self.repository
                .update_statement_status(statement_id, "reconciled", matched_by)
                .await?;
        } else if result.matched > 0 {
            self.repository
                .update_statement_status(statement_id, "in_review", None)
                .await?;
        }

        info!(
            "Auto-match complete: {} matched, {} unmatched, {} already matched",
            result.matched, result.unmatched, result.already_matched
        );

        Ok(result)
    }

    // ========================================================================
    // Manual Matching
    // ========================================================================

    /// Manually match a statement line to a system transaction
    pub async fn manual_match(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        statement_line_id: Uuid,
        system_transaction_id: Uuid,
        matched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch> {
        info!(
            "Manual match: line {} to transaction {}",
            statement_line_id, system_transaction_id
        );

        // Verify statement line exists and is unmatched
        let lines = self.repository.list_statement_lines(statement_id).await?;
        let line = lines
            .iter()
            .find(|l| l.id == statement_line_id)
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Statement line {}", statement_line_id)))?;

        if line.match_status != "unmatched" {
            return Err(AtlasError::Conflict(format!(
                "Statement line {} is already matched (status: {})",
                statement_line_id, line.match_status
            )));
        }

        // Verify system transaction exists and is unreconciled
        let txn = self
            .repository
            .get_system_transaction(system_transaction_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("System transaction {}", system_transaction_id)))?;

        if txn.status != "unreconciled" {
            return Err(AtlasError::Conflict(format!(
                "System transaction {} is already reconciled (status: {})",
                system_transaction_id, txn.status
            )));
        }

        self.repository
            .create_match(
                org_id,
                statement_id,
                statement_line_id,
                system_transaction_id,
                "manual",
                None,
                matched_by,
            )
            .await
    }

    /// Unmatch a previously matched statement line
    pub async fn unmatch(
        &self,
        match_id: Uuid,
        unmatched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch> {
        info!("Unmatching reconciliation match {}", match_id);

        let existing = self
            .repository
            .get_match(match_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Match {}", match_id)))?;

        if existing.status != "active" {
            return Err(AtlasError::Conflict(format!(
                "Match {} is already unmatched (status: {})",
                match_id, existing.status
            )));
        }

        self.repository
            .unmatch(match_id, unmatched_by)
            .await
    }

    // ========================================================================
    // Reconciliation Summary
    // ========================================================================

    /// Get or create reconciliation summary for an account + period
    pub async fn get_reconciliation_summary(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<ReconciliationSummary> {
        self.repository
            .get_or_create_summary(org_id, bank_account_id, period_start, period_end)
            .await
    }

    /// List reconciliation summaries for an organization
    pub async fn list_reconciliation_summaries(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<Vec<ReconciliationSummary>> {
        self.repository.list_summaries(org_id).await
    }

    // ========================================================================
    // Matching Rules
    // ========================================================================

    /// Create a matching rule
    pub async fn create_matching_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        bank_account_id: Option<Uuid>,
        priority: i32,
        criteria: serde_json::Value,
        stop_on_match: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatchingRule> {
        info!("Creating matching rule '{}' for org {}", name, org_id);

        // Validate criteria has a match_by field
        if criteria.get("match_by").is_none() {
            return Err(AtlasError::ValidationFailed(
                "criteria must include a 'match_by' field".to_string(),
            ));
        }

        self.repository
            .create_matching_rule(
                org_id,
                name,
                description,
                bank_account_id,
                priority,
                criteria,
                stop_on_match,
                created_by,
            )
            .await
    }

    /// List matching rules for an organization
    pub async fn list_matching_rules(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<Vec<ReconciliationMatchingRule>> {
        self.repository.list_matching_rules(org_id).await
    }

    /// Delete a matching rule
    pub async fn delete_matching_rule(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting matching rule {}", id);
        self.repository.delete_matching_rule(id).await
    }

    // ========================================================================
    // Reconciliation Matches listing
    // ========================================================================

    /// List matches for a statement
    pub async fn list_matches(
        &self,
        statement_id: Uuid,
    ) -> AtlasResult<Vec<ReconciliationMatch>> {
        self.repository.list_matches(statement_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transaction_types() {
        let valid_types = [
            "deposit", "withdrawal", "interest", "charge",
            "transfer_in", "transfer_out", "adjustment",
        ];
        assert!(valid_types.contains(&"deposit"));
        assert!(valid_types.contains(&"withdrawal"));
        assert!(!valid_types.contains(&"invalid"));
    }

    #[test]
    fn test_valid_source_types() {
        let valid_source_types = ["ap_payment", "ar_receipt", "gl_journal", "cash_transfer"];
        assert!(valid_source_types.contains(&"ap_payment"));
        assert!(valid_source_types.contains(&"ar_receipt"));
    }

    #[test]
    fn test_auto_match_pair_serialization() {
        let pair = AutoMatchPair {
            statement_line_id: Uuid::new_v4(),
            system_transaction_id: Uuid::new_v4(),
            match_method: "auto_check".to_string(),
            confidence: 100.0,
        };
        let json = serde_json::to_value(&pair).unwrap();
        assert_eq!(json["matchMethod"], "auto_check");
        assert_eq!(json["confidence"], 100.0);
    }

    #[test]
    fn test_auto_match_result_serialization() {
        let result = AutoMatchResult {
            total_lines: 10,
            matched: 7,
            unmatched: 2,
            already_matched: 1,
            matches: vec![],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["totalLines"], 10);
        assert_eq!(json["matched"], 7);
        assert_eq!(json["unmatched"], 2);
        assert_eq!(json["alreadyMatched"], 1);
    }
}
