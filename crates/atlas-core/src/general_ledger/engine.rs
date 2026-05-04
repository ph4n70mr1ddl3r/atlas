//! General Ledger Engine
//!
//! Core GL operations:
//! - Chart of Accounts management (create/query accounts)
//! - Journal Entry management (create/validate/post/reverse)
//! - Trial Balance generation
//! - Automatic balancing validation
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger

use atlas_shared::{
    GlAccount, GlJournalEntry, GlJournalLine, GlTrialBalance, GlTrialBalanceLine,
    AtlasError, AtlasResult,
};
use super::GeneralLedgerRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid account types
const VALID_ACCOUNT_TYPES: &[&str] = &[
    "asset", "liability", "equity", "revenue", "expense",
];

/// Valid account subtypes
const VALID_SUBTYPES: &[&str] = &[
    "current_asset", "fixed_asset", "current_liability",
    "long_term_liability", "operating_revenue", "other_revenue",
    "cost_of_goods", "operating_expense", "other_expense",
];

/// Valid natural balance directions
const VALID_NATURAL_BALANCES: &[&str] = &[
    "debit", "credit",
];

/// Valid entry types
const VALID_ENTRY_TYPES: &[&str] = &[
    "standard", "adjusting", "closing", "reversing", "budget",
];

/// Valid journal statuses
const VALID_JOURNAL_STATUSES: &[&str] = &[
    "draft", "submitted", "posted", "reversed", "error",
];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &[
    "debit", "credit",
];

/// General Ledger Engine
pub struct GeneralLedgerEngine {
    repository: Arc<dyn GeneralLedgerRepository>,
}

impl GeneralLedgerEngine {
    pub fn new(repository: Arc<dyn GeneralLedgerRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Chart of Accounts
    // ========================================================================

    /// Create a new GL account
    pub async fn create_account(
        &self,
        org_id: Uuid,
        account_code: &str,
        account_name: &str,
        description: Option<&str>,
        account_type: &str,
        subtype: Option<&str>,
        parent_account_id: Option<Uuid>,
        natural_balance: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAccount> {
        if account_code.is_empty() || account_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account code and name are required".to_string(),
            ));
        }
        if !VALID_ACCOUNT_TYPES.contains(&account_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid account_type '{}'. Must be one of: {}",
                account_type, VALID_ACCOUNT_TYPES.join(", ")
            )));
        }
        if let Some(st) = subtype {
            if !VALID_SUBTYPES.contains(&st) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid subtype '{}'. Must be one of: {}",
                    st, VALID_SUBTYPES.join(", ")
                )));
            }
        }
        if !VALID_NATURAL_BALANCES.contains(&natural_balance) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid natural_balance '{}'. Must be one of: {}",
                natural_balance, VALID_NATURAL_BALANCES.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_account_by_code(org_id, account_code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Account code '{}' already exists", account_code)
            ));
        }

        info!("Creating GL account {} ({}) for org {}", account_code, account_name, org_id);

        self.repository.create_account(
            org_id, account_code, account_name, description,
            account_type, subtype, parent_account_id,
            natural_balance, created_by,
        ).await
    }

    /// Get account by ID
    pub async fn get_account(&self, id: Uuid) -> AtlasResult<Option<GlAccount>> {
        self.repository.get_account(id).await
    }

    /// Get account by code
    pub async fn get_account_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAccount>> {
        self.repository.get_account_by_code(org_id, code).await
    }

    /// List all accounts for an organization
    pub async fn list_accounts(&self, org_id: Uuid, account_type: Option<&str>) -> AtlasResult<Vec<GlAccount>> {
        if let Some(t) = account_type {
            if !VALID_ACCOUNT_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid account_type '{}'. Must be one of: {}", t, VALID_ACCOUNT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_accounts(org_id, account_type).await
    }

    // ========================================================================
    // Journal Entries
    // ========================================================================

    /// Create a new journal entry (draft)
    pub async fn create_journal_entry(
        &self,
        org_id: Uuid,
        entry_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        entry_type: &str,
        description: Option<&str>,
        currency_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalEntry> {
        if !VALID_ENTRY_TYPES.contains(&entry_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid entry_type '{}'. Must be one of: {}",
                entry_type, VALID_ENTRY_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let entry_number = format!("JE-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating journal entry {} for org {}", entry_number, org_id);

        self.repository.create_journal_entry(
            org_id, &entry_number, entry_date, gl_date, entry_type,
            description, currency_code, source_type, source_id, created_by,
        ).await
    }

    /// Get a journal entry by ID
    pub async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<GlJournalEntry>> {
        self.repository.get_journal_entry(id).await
    }

    /// Get a journal entry by number
    pub async fn get_journal_entry_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlJournalEntry>> {
        self.repository.get_journal_entry_by_number(org_id, number).await
    }

    /// List journal entries
    pub async fn list_journal_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        entry_type: Option<&str>,
    ) -> AtlasResult<Vec<GlJournalEntry>> {
        if let Some(s) = status {
            if !VALID_JOURNAL_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_JOURNAL_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = entry_type {
            if !VALID_ENTRY_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid entry_type '{}'. Must be one of: {}", t, VALID_ENTRY_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_journal_entries(org_id, status, entry_type).await
    }

    /// Add a line to a journal entry
    pub async fn add_journal_line(
        &self,
        org_id: Uuid,
        journal_entry_id: Uuid,
        line_type: &str,
        account_code: &str,
        description: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalLine> {
        let je = self.repository.get_journal_entry(journal_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", journal_entry_id)
            ))?;

        if je.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to journal entry in '{}' status. Must be 'draft'.", je.status)
            ));
        }

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        // Validate account exists
        let account = self.repository.get_account_by_code(org_id, account_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Account code '{}' not found", account_code)
            ))?;

        let dr: f64 = entered_dr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_dr must be a valid number".to_string(),
        ))?;
        let cr: f64 = entered_cr.parse().map_err(|_| AtlasError::ValidationFailed(
            "entered_cr must be a valid number".to_string(),
        ))?;

        if dr < 0.0 || cr < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Debit and credit amounts must be non-negative".to_string(),
            ));
        }

        let lines = self.repository.list_journal_lines(journal_entry_id).await?;
        let line_number = (lines.len() as i32) + 1;

        info!("Adding line {} to journal entry {} ({}: dr={}, cr={})",
            line_number, je.entry_number, account_code, dr, cr);

        let line = self.repository.create_journal_line(
            org_id, journal_entry_id, line_number, line_type,
            account_code, Some(&account.account_name),
            description, entered_dr, entered_cr,
            entered_dr, entered_cr, // accounted = entered for now (same currency)
            &je.currency_code, None, None, None, created_by,
        ).await?;

        // Recalculate totals
        self.recalculate_journal_totals(journal_entry_id).await?;

        Ok(line)
    }

    /// List lines for a journal entry
    pub async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<GlJournalLine>> {
        self.repository.list_journal_lines(journal_entry_id).await
    }

    /// Validate that a journal entry is balanced (total debits == total credits)
    pub async fn validate_balance(&self, journal_entry_id: Uuid) -> AtlasResult<bool> {
        let lines = self.repository.list_journal_lines(journal_entry_id).await?;
        let total_dr: f64 = lines.iter()
            .map(|l| l.entered_dr.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_cr: f64 = lines.iter()
            .map(|l| l.entered_cr.parse::<f64>().unwrap_or(0.0))
            .sum();

        let balanced = (total_dr - total_cr).abs() < 0.01;
        Ok(balanced)
    }

    /// Post a journal entry (validates balance then posts)
    pub async fn post_journal_entry(&self, journal_entry_id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<GlJournalEntry> {
        let je = self.repository.get_journal_entry(journal_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", journal_entry_id)
            ))?;

        if je.status != "draft" && je.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post journal entry in '{}' status. Must be 'draft' or 'submitted'.", je.status)
            ));
        }

        let balanced = self.validate_balance(journal_entry_id).await?;
        if !balanced {
            return Err(AtlasError::WorkflowError(
                "Journal entry is not balanced (debits != credits)".to_string()
            ));
        }

        let lines = self.repository.list_journal_lines(journal_entry_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot post a journal entry with no lines".to_string(),
            ));
        }

        info!("Posting journal entry {}", je.entry_number);

        self.repository.update_journal_status(
            journal_entry_id, "posted", posted_by, None,
        ).await
    }

    /// Reverse a posted journal entry (creates a reversal)
    pub async fn reverse_journal_entry(&self, journal_entry_id: Uuid) -> AtlasResult<GlJournalEntry> {
        let je = self.repository.get_journal_entry(journal_entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", journal_entry_id)
            ))?;

        if je.status != "posted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse journal entry in '{}' status. Must be 'posted'.", je.status)
            ));
        }

        info!("Reversing journal entry {}", je.entry_number);

        self.repository.update_journal_status(
            journal_entry_id, "reversed", None, None,
        ).await
    }

    // ========================================================================
    // Trial Balance
    // ========================================================================

    /// Generate a trial balance as of a given date
    pub async fn generate_trial_balance(
        &self,
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
    ) -> AtlasResult<GlTrialBalance> {
        info!("Generating trial balance for org {} as of {}", org_id, as_of_date);

        let accounts = self.repository.list_accounts(org_id, None).await?;
        let mut lines = Vec::new();
        let mut total_debit = 0.0;
        let mut total_credit = 0.0;

        for account in &accounts {
            if !account.is_active {
                continue;
            }

            let (period_debit, period_credit) = self.repository
                .get_account_period_activity(org_id, &account.account_code, as_of_date)
                .await
                .unwrap_or((0.0, 0.0));

            let beginning = self.repository
                .get_account_balance(org_id, &account.account_code, as_of_date)
                .await
                .unwrap_or(0.0);

            let ending = match account.natural_balance.as_str() {
                "debit" => beginning + period_debit - period_credit,
                "credit" => beginning - period_debit + period_credit,
                _ => beginning + period_debit - period_credit,
            };

            let net = period_debit - period_credit;

            if beginning.abs() > 0.001 || period_debit.abs() > 0.001 || period_credit.abs() > 0.001 {
                if ending > 0.0 {
                    total_debit += ending;
                } else {
                    total_credit += ending.abs();
                }

                lines.push(GlTrialBalanceLine {
                    account_code: account.account_code.clone(),
                    account_name: account.account_name.clone(),
                    account_type: account.account_type.clone(),
                    beginning_balance: format!("{:.2}", beginning),
                    period_debit: format!("{:.2}", period_debit),
                    period_credit: format!("{:.2}", period_credit),
                    ending_balance: format!("{:.2}", ending),
                    net_activity: format!("{:.2}", net),
                });
            }
        }

        let total_net = total_debit - total_credit;

        Ok(GlTrialBalance {
            organization_id: org_id,
            as_of_date,
            ledger_id: None,
            lines,
            total_debit: format!("{:.2}", total_debit),
            total_credit: format!("{:.2}", total_credit),
            total_net: format!("{:.2}", total_net),
        })
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn recalculate_journal_totals(&self, journal_entry_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_journal_lines(journal_entry_id).await?;
        let total_dr: f64 = lines.iter()
            .map(|l| l.entered_dr.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_cr: f64 = lines.iter()
            .map(|l| l.entered_cr.parse::<f64>().unwrap_or(0.0))
            .sum();
        let balanced = (total_dr - total_cr).abs() < 0.01;

        self.repository.update_journal_totals(
            journal_entry_id,
            &format!("{:.2}", total_dr),
            &format!("{:.2}", total_cr),
            balanced,
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_account_types() {
        assert!(VALID_ACCOUNT_TYPES.contains(&"asset"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"liability"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"equity"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"revenue"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"expense"));
        assert_eq!(VALID_ACCOUNT_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_subtypes() {
        assert!(VALID_SUBTYPES.contains(&"current_asset"));
        assert!(VALID_SUBTYPES.contains(&"fixed_asset"));
        assert!(VALID_SUBTYPES.contains(&"current_liability"));
        assert!(VALID_SUBTYPES.contains(&"long_term_liability"));
        assert!(VALID_SUBTYPES.contains(&"operating_revenue"));
        assert!(VALID_SUBTYPES.contains(&"other_revenue"));
        assert!(VALID_SUBTYPES.contains(&"cost_of_goods"));
        assert!(VALID_SUBTYPES.contains(&"operating_expense"));
        assert!(VALID_SUBTYPES.contains(&"other_expense"));
    }

    #[test]
    fn test_valid_natural_balances() {
        assert!(VALID_NATURAL_BALANCES.contains(&"debit"));
        assert!(VALID_NATURAL_BALANCES.contains(&"credit"));
        assert_eq!(VALID_NATURAL_BALANCES.len(), 2);
    }

    #[test]
    fn test_valid_entry_types() {
        assert!(VALID_ENTRY_TYPES.contains(&"standard"));
        assert!(VALID_ENTRY_TYPES.contains(&"adjusting"));
        assert!(VALID_ENTRY_TYPES.contains(&"closing"));
        assert!(VALID_ENTRY_TYPES.contains(&"reversing"));
        assert!(VALID_ENTRY_TYPES.contains(&"budget"));
        assert_eq!(VALID_ENTRY_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_journal_statuses() {
        assert!(VALID_JOURNAL_STATUSES.contains(&"draft"));
        assert!(VALID_JOURNAL_STATUSES.contains(&"submitted"));
        assert!(VALID_JOURNAL_STATUSES.contains(&"posted"));
        assert!(VALID_JOURNAL_STATUSES.contains(&"reversed"));
        assert!(VALID_JOURNAL_STATUSES.contains(&"error"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"debit"));
        assert!(VALID_LINE_TYPES.contains(&"credit"));
        assert_eq!(VALID_LINE_TYPES.len(), 2);
    }
}
