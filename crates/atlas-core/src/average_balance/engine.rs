//! Average Balance Processing Engine
//!
//! Manages the full lifecycle of average balance processing:
//! - Create and manage average balance books
//! - Register GL accounts for tracking
//! - Record and manage daily balance entries
//! - Calculate average balances over configurable windows
//! - Approve and post calculated averages
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Average Balances

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_PERIOD_TYPES: &[&str] = &["daily", "monthly", "quarterly", "yearly"];
const VALID_BOOK_STATUSES: &[&str] = &["active", "inactive"];
const VALID_ACCOUNT_TYPES: &[&str] = &["asset", "liability", "equity", "revenue", "expense"];
const VALID_CALC_TYPES: &[&str] = &["daily", "monthly", "quarterly", "yearly"];
const VALID_CALC_STATUSES: &[&str] = &["pending", "calculated", "approved", "posted"];

pub struct AverageBalanceEngine {
    repository: Arc<dyn AverageBalanceRepository>,
}

impl AverageBalanceEngine {
    pub fn new(repository: Arc<dyn AverageBalanceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Books
    // ========================================================================

    /// Create a new average balance book
    pub async fn create_book(
        &self,
        org_id: Uuid,
        book_code: &str,
        book_name: &str,
        description: Option<&str>,
        period_type: &str,
        averaging_window_days: i32,
        is_primary: bool,
        currency_code: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AverageBalanceBook> {
        if book_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Book code is required".into()));
        }
        if book_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Book name is required".into()));
        }
        if !VALID_PERIOD_TYPES.contains(&period_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid period type '{}'. Must be one of: {}", period_type, VALID_PERIOD_TYPES.join(", ")
            )));
        }
        if averaging_window_days < 1 {
            return Err(AtlasError::ValidationFailed("Averaging window must be at least 1 day".into()));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if let Some(to) = effective_to {
            if to < effective_from {
                return Err(AtlasError::ValidationFailed("Effective to date must be after effective from date".into()));
            }
        }

        // Check for duplicate book code
        if self.repository.get_book_by_code(org_id, book_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Book code '{}' already exists", book_code)));
        }

        // If setting as primary, validate no other primary exists
        if is_primary {
            let existing = self.repository.list_books(org_id, Some("active")).await?;
            for book in &existing {
                if book.is_primary && book.id != Uuid::nil() {
                    return Err(AtlasError::Conflict(
                        "Another primary book already exists for this organization".into()
                    ));
                }
            }
        }

        info!("Creating average balance book '{}' (period: {}, window: {} days)", book_code, period_type, averaging_window_days);

        self.repository.create_book(
            org_id, book_code, book_name, description, period_type,
            averaging_window_days, is_primary, currency_code, effective_from,
            effective_to, created_by,
        ).await
    }

    /// Get a book by ID
    pub async fn get_book(&self, id: Uuid) -> AtlasResult<Option<AverageBalanceBook>> {
        self.repository.get_book(id).await
    }

    /// Get a book by code
    pub async fn get_book_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AverageBalanceBook>> {
        self.repository.get_book_by_code(org_id, code).await
    }

    /// List books for an organization
    pub async fn list_books(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AverageBalanceBook>> {
        if let Some(s) = status {
            if !VALID_BOOK_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BOOK_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_books(org_id, status).await
    }

    /// Activate a book
    pub async fn activate_book(&self, book_id: Uuid) -> AtlasResult<AverageBalanceBook> {
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        if book.status == "active" {
            return Err(AtlasError::WorkflowError("Book is already active".into()));
        }

        info!("Activating average balance book '{}'", book.book_code);
        self.repository.update_book_status(book_id, "active").await
    }

    /// Deactivate a book
    pub async fn deactivate_book(&self, book_id: Uuid) -> AtlasResult<AverageBalanceBook> {
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        if book.status == "inactive" {
            return Err(AtlasError::WorkflowError("Book is already inactive".into()));
        }

        info!("Deactivating average balance book '{}'", book.book_code);
        self.repository.update_book_status(book_id, "inactive").await
    }

    /// Delete a book (only if inactive)
    pub async fn delete_book(&self, book_id: Uuid) -> AtlasResult<()> {
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        if book.status == "active" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete an active book. Deactivate it first.".into()
            ));
        }

        info!("Deleting average balance book '{}'", book.book_code);
        self.repository.delete_book(book_id).await
    }

    // ========================================================================
    // Book Accounts
    // ========================================================================

    /// Add a GL account to a book for tracking
    pub async fn add_account(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        gl_account: &str,
        gl_account_name: Option<&str>,
        account_type: Option<&str>,
        track_negative: bool,
    ) -> AtlasResult<BookAccount> {
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        if book.status != "active" {
            return Err(AtlasError::WorkflowError(
                "Cannot add accounts to an inactive book".into()
            ));
        }

        if gl_account.is_empty() {
            return Err(AtlasError::ValidationFailed("GL account is required".into()));
        }

        if let Some(at) = account_type {
            if !VALID_ACCOUNT_TYPES.contains(&at) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid account type '{}'. Must be one of: {}", at, VALID_ACCOUNT_TYPES.join(", ")
                )));
            }
        }

        // Check for duplicate account in the book
        let existing = self.repository.list_accounts(book_id).await?;
        if existing.iter().any(|a| a.gl_account == gl_account) {
            return Err(AtlasError::Conflict(format!(
                "GL account '{}' already exists in book", gl_account
            )));
        }

        info!("Adding GL account '{}' to book '{}'", gl_account, book.book_code);
        self.repository.add_account(
            org_id, book_id, gl_account, gl_account_name,
            account_type, track_negative,
        ).await
    }

    /// List accounts in a book
    pub async fn list_accounts(&self, book_id: Uuid) -> AtlasResult<Vec<BookAccount>> {
        self.repository.list_accounts(book_id).await
    }

    /// Remove an account from a book
    pub async fn remove_account(&self, account_id: Uuid) -> AtlasResult<()> {
        self.repository.remove_account(account_id).await
    }

    // ========================================================================
    // Daily Balances
    // ========================================================================

    /// Record or update a daily balance entry
    pub async fn upsert_daily_balance(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        account_id: Uuid,
        balance_date: chrono::NaiveDate,
        closing_balance: &str,
        opening_balance: &str,
        total_debits: &str,
        total_credits: &str,
        transaction_count: i32,
        negative_balance: &str,
    ) -> AtlasResult<DailyBalance> {
        // Validate the book exists and is active
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        if book.status != "active" {
            return Err(AtlasError::WorkflowError("Cannot record balances for an inactive book".into()));
        }

        // Validate the account exists in this book
        let account = self.repository.get_account(account_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Account {} not found", account_id)))?;

        if account.book_id != book_id {
            return Err(AtlasError::ValidationFailed("Account does not belong to this book".into()));
        }

        let cb: f64 = closing_balance.parse().unwrap_or(f64::NAN);
        if cb.is_nan() {
            return Err(AtlasError::ValidationFailed("Closing balance must be a valid number".into()));
        }

        let ob: f64 = opening_balance.parse().unwrap_or(f64::NAN);
        if ob.is_nan() {
            return Err(AtlasError::ValidationFailed("Opening balance must be a valid number".into()));
        }

        if transaction_count < 0 {
            return Err(AtlasError::ValidationFailed("Transaction count cannot be negative".into()));
        }

        // Verify that closing = opening + debits - credits
        let td: f64 = total_debits.parse().unwrap_or(0.0);
        let tc: f64 = total_credits.parse().unwrap_or(0.0);
        let expected_closing = ob + td - tc;
        let diff = (cb - expected_closing).abs();
        if diff > 0.01 {
            return Err(AtlasError::ValidationFailed(format!(
                "Closing balance ({}) does not equal opening ({}) + debits ({}) - credits ({})",
                closing_balance, opening_balance, total_debits, total_credits
            )));
        }

        info!("Recording daily balance for account {} on {} in book '{}'", account.gl_account, balance_date, book.book_code);

        self.repository.upsert_daily_balance(
            org_id, book_id, account_id, balance_date,
            closing_balance, opening_balance, total_debits, total_credits,
            transaction_count, negative_balance,
        ).await
    }

    /// List daily balances for an account
    pub async fn list_daily_balances(
        &self,
        book_id: Uuid,
        account_id: Uuid,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<DailyBalance>> {
        self.repository.list_daily_balances(book_id, account_id, from_date, to_date).await
    }

    // ========================================================================
    // Calculations
    // ========================================================================

    /// Calculate average balances for an account over a period
    pub async fn calculate_average_balance(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        account_id: Uuid,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        calculation_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AverageBalanceCalculation> {
        if !VALID_CALC_TYPES.contains(&calculation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation type '{}'. Must be one of: {}", calculation_type, VALID_CALC_TYPES.join(", ")
            )));
        }

        if period_end_date < period_start_date {
            return Err(AtlasError::ValidationFailed("Period end date must be after period start date".into()));
        }

        // Validate book and account
        let book = self.repository.get_book(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", book_id)))?;

        let account = self.repository.get_account(account_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Account {} not found", account_id)))?;

        if account.book_id != book_id {
            return Err(AtlasError::ValidationFailed("Account does not belong to this book".into()));
        }

        // Fetch daily balances for the period
        let daily_balances = self.repository.list_daily_balances(
            book_id, account_id,
            Some(period_start_date), Some(period_end_date),
        ).await?;

        if daily_balances.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No daily balance entries found for the specified period".into()
            ));
        }

        // Calculate metrics
        let days_in_period = (period_end_date - period_start_date).num_days() as i32 + 1;
        let actual_days = daily_balances.len() as i32;

        // Simple average
        let sum_balances: f64 = daily_balances.iter()
            .map(|d| d.closing_balance.parse::<f64>().unwrap_or(0.0))
            .sum();
        let average_balance = sum_balances / actual_days as f64;

        // Weighted average (balance * days weighted by recency)
        let total_weight: f64 = (1..=actual_days).map(|d| d as f64).sum::<f64>();
        let weighted_sum: f64 = daily_balances.iter().enumerate()
            .map(|(i, d)| d.closing_balance.parse::<f64>().unwrap_or(0.0) * (i + 1) as f64)
            .sum();
        let weighted_average = weighted_sum / total_weight;

        // Period totals
        let period_total_debits: f64 = daily_balances.iter()
            .map(|d| d.total_debits.parse::<f64>().unwrap_or(0.0))
            .sum();
        let period_total_credits: f64 = daily_balances.iter()
            .map(|d| d.total_credits.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Period end balance (last entry)
        let period_end_balance: f64 = daily_balances.last()
            .map(|d| d.closing_balance.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        // Peak and trough
        let balances: Vec<f64> = daily_balances.iter()
            .map(|d| d.closing_balance.parse::<f64>().unwrap_or(0.0))
            .collect();
        let peak_balance = balances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let trough_balance = balances.iter().cloned().fold(f64::INFINITY, f64::min);

        // Average-to-date (simple average from start)
        let average_to_date = average_balance; // Simplified; real impl would go back to fiscal year start

        let calc = AverageBalanceCalculation {
            id: Uuid::new_v4(),
            organization_id: org_id,
            book_id,
            account_id,
            period_start_date,
            period_end_date,
            days_in_period,
            average_balance: format!("{:.2}", average_balance),
            weighted_average_balance: format!("{:.2}", weighted_average),
            period_total_debits: format!("{:.2}", period_total_debits),
            period_total_credits: format!("{:.2}", period_total_credits),
            average_to_date: format!("{:.2}", average_to_date),
            period_end_balance: format!("{:.2}", period_end_balance),
            peak_balance: format!("{:.2}", peak_balance),
            trough_balance: format!("{:.2}", trough_balance),
            calculation_type: calculation_type.to_string(),
            status: "calculated".to_string(),
            calculated_at: Some(chrono::Utc::now()),
            approved_by: None,
            approved_at: None,
            metadata: serde_json::json!({}),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        info!(
            "Calculated {} average balance for account '{}' in book '{}': avg={}, weighted={}, peak={}, trough={}",
            calculation_type, account.gl_account, book.book_code,
            calc.average_balance, calc.weighted_average_balance,
            calc.peak_balance, calc.trough_balance
        );

        self.repository.save_calculation(calc).await
    }

    /// Get a calculation by ID
    pub async fn get_calculation(&self, id: Uuid) -> AtlasResult<Option<AverageBalanceCalculation>> {
        self.repository.get_calculation(id).await
    }

    /// List calculations with filters
    pub async fn list_calculations(
        &self,
        book_id: Uuid,
        account_id: Option<Uuid>,
        calculation_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<AverageBalanceCalculation>> {
        if let Some(ct) = calculation_type {
            if !VALID_CALC_TYPES.contains(&ct) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid calculation type '{}'. Must be one of: {}", ct, VALID_CALC_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_CALC_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CALC_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_calculations(book_id, account_id, calculation_type, status).await
    }

    /// Approve a calculated average balance
    pub async fn approve_calculation(&self, calc_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<AverageBalanceCalculation> {
        let calc = self.repository.get_calculation(calc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Calculation {} not found", calc_id)))?;

        if calc.status != "calculated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve calculation in '{}' status. Must be 'calculated'.", calc.status)
            ));
        }

        info!("Approving average balance calculation {}", calc_id);
        self.repository.update_calculation_status(calc_id, "approved", approved_by).await
    }

    /// Post an approved calculation
    pub async fn post_calculation(&self, calc_id: Uuid) -> AtlasResult<AverageBalanceCalculation> {
        let calc = self.repository.get_calculation(calc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Calculation {} not found", calc_id)))?;

        if calc.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post calculation in '{}' status. Must be 'approved'.", calc.status)
            ));
        }

        info!("Posting average balance calculation {}", calc_id);
        self.repository.update_calculation_status(calc_id, "posted", None).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AverageBalanceDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        books: std::sync::Mutex<Vec<AverageBalanceBook>>,
        accounts: std::sync::Mutex<Vec<BookAccount>>,
        daily_balances: std::sync::Mutex<Vec<DailyBalance>>,
        calculations: std::sync::Mutex<Vec<AverageBalanceCalculation>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                books: std::sync::Mutex::new(vec![]),
                accounts: std::sync::Mutex::new(vec![]),
                daily_balances: std::sync::Mutex::new(vec![]),
                calculations: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl AverageBalanceRepository for MockRepo {
        async fn create_book(
            &self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>,
            period_type: &str, window: i32, primary: bool, currency: &str,
            eff_from: chrono::NaiveDate, eff_to: Option<chrono::NaiveDate>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<AverageBalanceBook> {
            let b = AverageBalanceBook {
                id: Uuid::new_v4(), organization_id: org_id,
                book_code: code.into(), book_name: name.into(),
                description: desc.map(Into::into), period_type: period_type.into(),
                averaging_window_days: window, is_primary: primary,
                status: "active".into(), currency_code: currency.into(),
                effective_from: eff_from, effective_to: eff_to,
                metadata: serde_json::json!({}),
                created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.books.lock().unwrap().push(b.clone());
            Ok(b)
        }

        async fn get_book(&self, id: Uuid) -> AtlasResult<Option<AverageBalanceBook>> {
            Ok(self.books.lock().unwrap().iter().find(|b| b.id == id).cloned())
        }

        async fn get_book_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AverageBalanceBook>> {
            Ok(self.books.lock().unwrap().iter()
                .find(|b| b.organization_id == org_id && b.book_code == code).cloned())
        }

        async fn list_books(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AverageBalanceBook>> {
            Ok(self.books.lock().unwrap().iter()
                .filter(|b| b.organization_id == org_id)
                .filter(|b| status.map_or(true, |s| b.status == s))
                .cloned().collect())
        }

        async fn update_book_status(&self, id: Uuid, status: &str) -> AtlasResult<AverageBalanceBook> {
            let mut bs = self.books.lock().unwrap();
            let b = bs.iter_mut().find(|b| b.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Book {} not found", id)))?;
            b.status = status.into();
            b.updated_at = chrono::Utc::now();
            Ok(b.clone())
        }

        async fn delete_book(&self, id: Uuid) -> AtlasResult<()> {
            let mut bs = self.books.lock().unwrap();
            bs.retain(|b| b.id != id);
            Ok(())
        }

        async fn add_account(
            &self, org_id: Uuid, book_id: Uuid, gl: &str, name: Option<&str>,
            at: Option<&str>, tn: bool,
        ) -> AtlasResult<BookAccount> {
            let a = BookAccount {
                id: Uuid::new_v4(), organization_id: org_id, book_id,
                gl_account: gl.into(), gl_account_name: name.map(Into::into),
                account_type: at.map(Into::into), track_negative: tn,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.accounts.lock().unwrap().push(a.clone());
            Ok(a)
        }

        async fn list_accounts(&self, book_id: Uuid) -> AtlasResult<Vec<BookAccount>> {
            Ok(self.accounts.lock().unwrap().iter().filter(|a| a.book_id == book_id).cloned().collect())
        }

        async fn get_account(&self, id: Uuid) -> AtlasResult<Option<BookAccount>> {
            Ok(self.accounts.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }

        async fn remove_account(&self, id: Uuid) -> AtlasResult<()> {
            let mut as_ = self.accounts.lock().unwrap();
            as_.retain(|a| a.id != id);
            Ok(())
        }

        async fn upsert_daily_balance(
            &self, org_id: Uuid, book_id: Uuid, account_id: Uuid,
            date: chrono::NaiveDate, cb: &str, ob: &str, td: &str, tc: &str,
            txn_count: i32, neg: &str,
        ) -> AtlasResult<DailyBalance> {
            let mut dbs = self.daily_balances.lock().unwrap();
            if let Some(existing) = dbs.iter_mut().find(|d| d.book_id == book_id && d.account_id == account_id && d.balance_date == date) {
                existing.closing_balance = cb.into();
                existing.opening_balance = ob.into();
                existing.total_debits = td.into();
                existing.total_credits = tc.into();
                existing.transaction_count = txn_count;
                existing.negative_balance = neg.into();
                existing.updated_at = chrono::Utc::now();
                return Ok(existing.clone());
            }
            let db = DailyBalance {
                id: Uuid::new_v4(), organization_id: org_id, book_id, account_id,
                balance_date: date, closing_balance: cb.into(), opening_balance: ob.into(),
                total_debits: td.into(), total_credits: tc.into(),
                transaction_count: txn_count, negative_balance: neg.into(),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            dbs.push(db.clone());
            Ok(db)
        }

        async fn list_daily_balances(
            &self, book_id: Uuid, account_id: Uuid,
            from: Option<chrono::NaiveDate>, to: Option<chrono::NaiveDate>,
        ) -> AtlasResult<Vec<DailyBalance>> {
            Ok(self.daily_balances.lock().unwrap().iter()
                .filter(|d| d.book_id == book_id && d.account_id == account_id)
                .filter(|d| from.map_or(true, |f| d.balance_date >= f))
                .filter(|d| to.map_or(true, |t| d.balance_date <= t))
                .cloned().collect())
        }

        async fn get_daily_balance(&self, book_id: Uuid, account_id: Uuid, date: chrono::NaiveDate) -> AtlasResult<Option<DailyBalance>> {
            Ok(self.daily_balances.lock().unwrap().iter()
                .find(|d| d.book_id == book_id && d.account_id == account_id && d.balance_date == date).cloned())
        }

        async fn save_calculation(&self, calc: AverageBalanceCalculation) -> AtlasResult<AverageBalanceCalculation> {
            self.calculations.lock().unwrap().push(calc.clone());
            Ok(calc)
        }

        async fn get_calculation(&self, id: Uuid) -> AtlasResult<Option<AverageBalanceCalculation>> {
            Ok(self.calculations.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn list_calculations(&self, book_id: Uuid, account_id: Option<Uuid>, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<AverageBalanceCalculation>> {
            Ok(self.calculations.lock().unwrap().iter()
                .filter(|c| c.book_id == book_id)
                .filter(|c| account_id.map_or(true, |id| c.account_id == id))
                .cloned().collect())
        }

        async fn update_calculation_status(&self, id: Uuid, status: &str, ab: Option<Uuid>) -> AtlasResult<AverageBalanceCalculation> {
            let mut cs = self.calculations.lock().unwrap();
            let c = cs.iter_mut().find(|c| c.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Calc {} not found", id)))?;
            c.status = status.into();
            if status == "approved" {
                c.approved_by = ab;
                c.approved_at = Some(chrono::Utc::now());
            }
            c.updated_at = chrono::Utc::now();
            Ok(c.clone())
        }

        async fn get_dashboard(&self, _: Uuid) -> AtlasResult<AverageBalanceDashboard> {
            Ok(AverageBalanceDashboard {
                total_books: 0, active_books: 0, tracked_accounts: 0,
                daily_balance_entries: 0, total_average_balance: "0.00".into(),
                total_peak_balance: "0.00".into(), total_period_debits: "0.00".into(),
                total_period_credits: "0.00".into(), books_summary: serde_json::json!([]),
            })
        }
    }

    fn eng() -> AverageBalanceEngine {
        AverageBalanceEngine::new(Arc::new(MockRepo::new()))
    }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_PERIOD_TYPES.len(), 4);
        assert_eq!(VALID_BOOK_STATUSES.len(), 2);
        assert_eq!(VALID_ACCOUNT_TYPES.len(), 5);
        assert_eq!(VALID_CALC_TYPES.len(), 4);
        assert_eq!(VALID_CALC_STATUSES.len(), 4);
    }

    // ========================================================================
    // Book Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_book() {
        let b = eng().create_book(
            Uuid::new_v4(), "AVG-01", "Primary Average Balance Book",
            Some("Main averaging book"), "daily", 30, true, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        assert_eq!(b.book_code, "AVG-01");
        assert_eq!(b.book_name, "Primary Average Balance Book");
        assert_eq!(b.period_type, "daily");
        assert_eq!(b.averaging_window_days, 30);
        assert!(b.is_primary);
        assert_eq!(b.status, "active");
    }

    #[tokio::test]
    async fn test_create_book_empty_code_fails() {
        let r = eng().create_book(
            Uuid::new_v4(), "", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_book_empty_name_fails() {
        let r = eng().create_book(
            Uuid::new_v4(), "AVG-X", "", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_book_invalid_period_type_fails() {
        let r = eng().create_book(
            Uuid::new_v4(), "AVG-X", "Book", None, "hourly", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_book_zero_window_fails() {
        let r = eng().create_book(
            Uuid::new_v4(), "AVG-X", "Book", None, "daily", 0, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_book_invalid_currency_fails() {
        let r = eng().create_book(
            Uuid::new_v4(), "AVG-X", "Book", None, "daily", 30, false, "US",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_book_duplicate_code_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_book(
            org, "AVG-DUP", "Book 1", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        let r = e.create_book(
            org, "AVG-DUP", "Book 2", None, "monthly", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_activate_deactivate_book() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACT", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        assert_eq!(b.status, "active");

        let b = e.deactivate_book(b.id).await.unwrap();
        assert_eq!(b.status, "inactive");

        let b = e.activate_book(b.id).await.unwrap();
        assert_eq!(b.status, "active");
    }

    #[tokio::test]
    async fn test_activate_already_active_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-AA", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let r = e.activate_book(b.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_delete_active_book_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DEL", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let r = e.delete_book(b.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_delete_inactive_book() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DEL2", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        e.deactivate_book(b.id).await.unwrap();
        let r = e.delete_book(b.id).await;
        assert!(r.is_ok());
    }

    // ========================================================================
    // Account Tests
    // ========================================================================

    #[tokio::test]
    async fn test_add_account() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACC", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(
            b.organization_id, b.id, "1000-100", Some("Cash - Operating"), Some("asset"), false,
        ).await.unwrap();
        assert_eq!(a.gl_account, "1000-100");
        assert_eq!(a.gl_account_name, Some("Cash - Operating".into()));
        assert_eq!(a.account_type, Some("asset".into()));
    }

    #[tokio::test]
    async fn test_add_account_empty_gl_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACC-E", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let r = e.add_account(b.organization_id, b.id, "", None, None, false).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_account_invalid_type_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACC-T", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let r = e.add_account(b.organization_id, b.id, "1000", None, Some("bogus"), false).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_add_duplicate_account_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACC-D", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let _ = e.add_account(b.organization_id, b.id, "1000", None, None, false).await;
        let r = e.add_account(b.organization_id, b.id, "1000", None, None, false).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_add_account_to_inactive_book_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-ACC-I", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        e.deactivate_book(b.id).await.unwrap();
        let r = e.add_account(b.organization_id, b.id, "1000", None, None, false).await;
        assert!(r.is_err());
    }

    // ========================================================================
    // Daily Balance Tests
    // ========================================================================

    #[tokio::test]
    async fn test_upsert_daily_balance() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DB", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(b.organization_id, b.id, "1000", None, None, false).await.unwrap();
        let db = e.upsert_daily_balance(
            b.organization_id, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            "10500.00", "10000.00", "1000.00", "500.00", 5, "0.00",
        ).await.unwrap();
        assert_eq!(db.closing_balance, "10500.00");
        assert_eq!(db.opening_balance, "10000.00");
    }

    #[tokio::test]
    async fn test_daily_balance_invalid_closing_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DB-I", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(b.organization_id, b.id, "1000", None, None, false).await.unwrap();
        let r = e.upsert_daily_balance(
            b.organization_id, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            "abc", "10000.00", "1000.00", "500.00", 5, "0.00",
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_daily_balance_mismatch_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DB-M", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(b.organization_id, b.id, "1000", None, None, false).await.unwrap();
        // closing(9000) != opening(10000) + debits(1000) - credits(500) = 10500
        let r = e.upsert_daily_balance(
            b.organization_id, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            "9000.00", "10000.00", "1000.00", "500.00", 5, "0.00",
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_daily_balance_inactive_book_fails() {
        let e = eng();
        let b = e.create_book(
            Uuid::new_v4(), "AVG-DB-IA", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(b.organization_id, b.id, "1000", None, None, false).await.unwrap();
        e.deactivate_book(b.id).await.unwrap();
        let r = e.upsert_daily_balance(
            b.organization_id, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            "10500.00", "10000.00", "1000.00", "500.00", 5, "0.00",
        ).await;
        assert!(r.is_err());
    }

    // ========================================================================
    // Calculation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_calculate_average_balance() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_book(
            org, "AVG-CALC", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(org, b.id, "1000", None, None, false).await.unwrap();

        // Record 3 days of balances: 10000, 12000, 11000
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "10000.00", "9000.00", "2000.00", "1000.00", 3, "0.00").await.unwrap();
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            "12000.00", "10000.00", "3000.00", "1000.00", 4, "0.00").await.unwrap();
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
            "11000.00", "12000.00", "1000.00", "2000.00", 3, "0.00").await.unwrap();

        let calc = e.calculate_average_balance(
            org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
            "daily", None,
        ).await.unwrap();

        assert_eq!(calc.status, "calculated");
        assert_eq!(calc.days_in_period, 3);
        // Average of 10000, 12000, 11000 = 11000
        assert_eq!(calc.average_balance, "11000.00");
        assert_eq!(calc.peak_balance, "12000.00");
        assert_eq!(calc.trough_balance, "10000.00");
        assert_eq!(calc.period_end_balance, "11000.00");
    }

    #[tokio::test]
    async fn test_calculate_empty_period_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_book(
            org, "AVG-CALC-E", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(org, b.id, "1000", None, None, false).await.unwrap();

        let r = e.calculate_average_balance(
            org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
            "monthly", None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_approve_and_post_calculation() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_book(
            org, "AVG-AP", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(org, b.id, "1000", None, None, false).await.unwrap();
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "5000.00", "5000.00", "0.00", "0.00", 0, "0.00").await.unwrap();

        let calc = e.calculate_average_balance(
            org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "daily", None,
        ).await.unwrap();
        assert_eq!(calc.status, "calculated");

        let calc = e.approve_calculation(calc.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(calc.status, "approved");
        assert!(calc.approved_by.is_some());

        let calc = e.post_calculation(calc.id).await.unwrap();
        assert_eq!(calc.status, "posted");
    }

    #[tokio::test]
    async fn test_approve_non_calculated_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_book(
            org, "AVG-AP-F", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(org, b.id, "1000", None, None, false).await.unwrap();
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "5000.00", "5000.00", "0.00", "0.00", 0, "0.00").await.unwrap();

        let calc = e.calculate_average_balance(
            org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "daily", None,
        ).await.unwrap();
        e.approve_calculation(calc.id, None).await.unwrap();
        // Try to approve again (already approved)
        let r = e.approve_calculation(calc.id, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_post_non_approved_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let b = e.create_book(
            org, "AVG-POST-F", "Book", None, "daily", 30, false, "USD",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await.unwrap();
        let a = e.add_account(org, b.id, "1000", None, None, false).await.unwrap();
        e.upsert_daily_balance(org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "5000.00", "5000.00", "0.00", "0.00", 0, "0.00").await.unwrap();

        let calc = e.calculate_average_balance(
            org, b.id, a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "daily", None,
        ).await.unwrap();
        // Try to post without approving
        let r = e.post_calculation(calc.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_get_book_not_found() {
        let r = eng().get_book(Uuid::new_v4()).await.unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_books, 0);
        assert_eq!(d.total_average_balance, "0.00");
    }

    #[tokio::test]
    async fn test_list_books_invalid_status() {
        let r = eng().list_books(Uuid::new_v4(), Some("unknown")).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_calculate_invalid_type_fails() {
        let r = eng().calculate_average_balance(
            Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            "hourly", None,
        ).await;
        assert!(r.is_err());
    }
}
