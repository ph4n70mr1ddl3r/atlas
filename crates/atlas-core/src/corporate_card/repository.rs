//! Corporate Card Management Repository
//!
//! PostgreSQL storage for corporate card programmes, cards, transactions,
//! statements, and spending limit overrides.

use atlas_shared::{
    CorporateCardProgram, CorporateCard, CorporateCardTransaction,
    CorporateCardStatement, CorporateCardLimitOverride, CorporateCardDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for corporate card management data storage
#[async_trait]
pub trait CorporateCardRepository: Send + Sync {
    // ── Card Programmes ─────────────────────────────────────────────────
    async fn create_program(
        &self, org_id: Uuid, program_code: &str, name: &str, description: Option<&str>,
        issuer_bank: &str, card_network: &str, card_type: &str, currency_code: &str,
        default_single_purchase_limit: &str, default_monthly_limit: &str,
        default_cash_limit: &str, default_atm_limit: &str,
        allow_cash_withdrawal: bool, allow_international: bool,
        auto_deactivate_on_termination: bool, expense_matching_method: &str,
        billing_cycle_day: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardProgram>;

    async fn get_program(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CorporateCardProgram>>;
    async fn get_program_by_id(&self, id: Uuid) -> AtlasResult<Option<CorporateCardProgram>>;
    async fn list_programs(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CorporateCardProgram>>;

    // ── Cards ───────────────────────────────────────────────────────────
    async fn create_card(
        &self, org_id: Uuid, program_id: Uuid, card_number_masked: &str,
        cardholder_name: &str, cardholder_id: Uuid, cardholder_email: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        status: &str, issue_date: chrono::NaiveDate, expiry_date: chrono::NaiveDate,
        single_purchase_limit: &str, monthly_limit: &str,
        cash_limit: &str, atm_limit: &str,
        gl_liability_account: Option<&str>, gl_expense_account: Option<&str>,
        cost_center: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCard>;

    async fn get_card(&self, id: Uuid) -> AtlasResult<Option<CorporateCard>>;
    async fn get_card_by_masked_number(&self, org_id: Uuid, masked: &str) -> AtlasResult<Option<CorporateCard>>;
    async fn list_cards(
        &self, org_id: Uuid, program_id: Option<Uuid>,
        cardholder_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCard>>;
    async fn update_card_status(&self, id: Uuid, status: &str) -> AtlasResult<CorporateCard>;
    async fn update_card_limits(
        &self, id: Uuid, single_purchase: &str, monthly: &str,
        cash: &str, atm: &str,
    ) -> AtlasResult<()>;
    async fn update_card_spend(&self, id: Uuid, amount: &str, balance: &str) -> AtlasResult<()>;

    // ── Transactions ────────────────────────────────────────────────────
    async fn create_transaction(
        &self, org_id: Uuid, card_id: Uuid, program_id: Uuid,
        transaction_reference: &str, posting_date: chrono::NaiveDate,
        transaction_date: chrono::NaiveDate, merchant_name: &str,
        merchant_category: Option<&str>, merchant_category_code: Option<&str>,
        amount: &str, currency_code: &str,
        original_amount: Option<&str>, original_currency: Option<&str>,
        exchange_rate: Option<&str>, transaction_type: &str,
    ) -> AtlasResult<CorporateCardTransaction>;

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CorporateCardTransaction>>;
    async fn get_transaction_by_ref(
        &self, org_id: Uuid, reference: &str,
    ) -> AtlasResult<Option<CorporateCardTransaction>>;
    async fn list_transactions(
        &self, org_id: Uuid, card_id: Option<Uuid>, status: Option<&str>,
        date_from: Option<chrono::NaiveDate>, date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<CorporateCardTransaction>>;
    async fn update_transaction_match(
        &self, id: Uuid, expense_report_id: Option<Uuid>,
        expense_line_id: Option<Uuid>, status: &str,
        matched_by: Option<Uuid>, match_confidence: Option<&str>,
    ) -> AtlasResult<CorporateCardTransaction>;
    async fn update_transaction_dispute(
        &self, id: Uuid, reason: Option<&str>,
        dispute_date: Option<chrono::NaiveDate>,
        resolution: Option<&str>, status: &str,
    ) -> AtlasResult<CorporateCardTransaction>;

    // ── Statements ──────────────────────────────────────────────────────
    async fn create_statement(
        &self, org_id: Uuid, program_id: Uuid, statement_number: &str,
        statement_date: chrono::NaiveDate,
        billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        opening_balance: &str, closing_balance: &str,
        total_charges: &str, total_credits: &str, total_payments: &str,
        total_fees: &str, total_interest: &str,
        payment_due_date: Option<chrono::NaiveDate>, minimum_payment: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardStatement>;

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CorporateCardStatement>>;
    async fn list_statements(
        &self, org_id: Uuid, program_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardStatement>>;
    async fn update_statement_counts(
        &self, id: Uuid, total: i32, matched: i32, unmatched: i32,
    ) -> AtlasResult<()>;
    async fn update_statement_status(
        &self, id: Uuid, status: &str, payment_reference: Option<&str>,
    ) -> AtlasResult<CorporateCardStatement>;

    // ── Limit Overrides ─────────────────────────────────────────────────
    async fn create_limit_override(
        &self, org_id: Uuid, card_id: Uuid, override_type: &str,
        original_value: &str, new_value: &str, reason: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardLimitOverride>;

    async fn get_limit_override(&self, id: Uuid) -> AtlasResult<Option<CorporateCardLimitOverride>>;
    async fn list_limit_overrides(
        &self, card_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardLimitOverride>>;
    async fn update_limit_override_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardLimitOverride>;

    // ── Dashboard ───────────────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CorporateCardDashboardSummary>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresCorporateCardRepository {
    pool: PgPool,
}

impl PostgresCorporateCardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<String, _>(col)
        .or_else(|_| row.try_get::<&str, _>(col).map(|s| s.to_string()))
        .unwrap_or_else(|_| "0.00".to_string())
}

fn row_to_program(row: &sqlx::postgres::PgRow) -> CorporateCardProgram {
    CorporateCardProgram {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        program_code: row.get("program_code"),
        name: row.get("name"),
        description: row.get("description"),
        issuer_bank: row.get("issuer_bank"),
        card_network: row.get("card_network"),
        card_type: row.get("card_type"),
        currency_code: row.get("currency_code"),
        default_single_purchase_limit: get_numeric(row, "default_single_purchase_limit"),
        default_monthly_limit: get_numeric(row, "default_monthly_limit"),
        default_cash_limit: get_numeric(row, "default_cash_limit"),
        default_atm_limit: get_numeric(row, "default_atm_limit"),
        allow_cash_withdrawal: row.get("allow_cash_withdrawal"),
        allow_international: row.get("allow_international"),
        auto_deactivate_on_termination: row.get("auto_deactivate_on_termination"),
        expense_matching_method: row.get("expense_matching_method"),
        billing_cycle_day: row.get("billing_cycle_day"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_card(row: &sqlx::postgres::PgRow) -> CorporateCard {
    CorporateCard {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        program_id: row.get("program_id"),
        card_number_masked: row.get("card_number_masked"),
        cardholder_name: row.get("cardholder_name"),
        cardholder_id: row.get("cardholder_id"),
        cardholder_email: row.get("cardholder_email"),
        department_id: row.get("department_id"),
        department_name: row.get("department_name"),
        status: row.get("status"),
        issue_date: row.get("issue_date"),
        expiry_date: row.get("expiry_date"),
        single_purchase_limit: get_numeric(row, "single_purchase_limit"),
        monthly_limit: get_numeric(row, "monthly_limit"),
        cash_limit: get_numeric(row, "cash_limit"),
        atm_limit: get_numeric(row, "atm_limit"),
        current_balance: get_numeric(row, "current_balance"),
        total_spend_current_cycle: get_numeric(row, "total_spend_current_cycle"),
        last_statement_balance: get_numeric(row, "last_statement_balance"),
        last_statement_date: row.get("last_statement_date"),
        gl_liability_account: row.get("gl_liability_account"),
        gl_expense_account: row.get("gl_expense_account"),
        cost_center: row.get("cost_center"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> CorporateCardTransaction {
    CorporateCardTransaction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        card_id: row.get("card_id"),
        program_id: row.get("program_id"),
        transaction_reference: row.get("transaction_reference"),
        posting_date: row.get("posting_date"),
        transaction_date: row.get("transaction_date"),
        merchant_name: row.get("merchant_name"),
        merchant_category: row.get("merchant_category"),
        merchant_category_code: row.get("merchant_category_code"),
        amount: get_numeric(row, "amount"),
        currency_code: row.get("currency_code"),
        original_amount: row.try_get("original_amount").unwrap_or(None),
        original_currency: row.try_get("original_currency").unwrap_or(None),
        exchange_rate: row.try_get("exchange_rate").unwrap_or(None),
        transaction_type: row.get("transaction_type"),
        status: row.get("status"),
        expense_report_id: row.get("expense_report_id"),
        expense_line_id: row.get("expense_line_id"),
        matched_at: row.get("matched_at"),
        matched_by: row.get("matched_by"),
        match_confidence: row.try_get("match_confidence").unwrap_or(None),
        dispute_reason: row.get("dispute_reason"),
        dispute_date: row.get("dispute_date"),
        dispute_resolution: row.get("dispute_resolution"),
        gl_posted: row.get("gl_posted"),
        gl_journal_id: row.get("gl_journal_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_statement(row: &sqlx::postgres::PgRow) -> CorporateCardStatement {
    CorporateCardStatement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        program_id: row.get("program_id"),
        statement_number: row.get("statement_number"),
        statement_date: row.get("statement_date"),
        billing_period_start: row.get("billing_period_start"),
        billing_period_end: row.get("billing_period_end"),
        opening_balance: get_numeric(row, "opening_balance"),
        closing_balance: get_numeric(row, "closing_balance"),
        total_charges: get_numeric(row, "total_charges"),
        total_credits: get_numeric(row, "total_credits"),
        total_payments: get_numeric(row, "total_payments"),
        total_fees: get_numeric(row, "total_fees"),
        total_interest: get_numeric(row, "total_interest"),
        payment_due_date: row.get("payment_due_date"),
        minimum_payment: get_numeric(row, "minimum_payment"),
        total_transaction_count: row.get("total_transaction_count"),
        matched_transaction_count: row.get("matched_transaction_count"),
        unmatched_transaction_count: row.get("unmatched_transaction_count"),
        status: row.get("status"),
        payment_reference: row.get("payment_reference"),
        paid_at: row.get("paid_at"),
        gl_payment_journal_id: row.get("gl_payment_journal_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        imported_by: row.get("imported_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_limit_override(row: &sqlx::postgres::PgRow) -> CorporateCardLimitOverride {
    CorporateCardLimitOverride {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        card_id: row.get("card_id"),
        override_type: row.get("override_type"),
        original_value: get_numeric(row, "original_value"),
        new_value: get_numeric(row, "new_value"),
        reason: row.get("reason"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        status: row.get("status"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl CorporateCardRepository for PostgresCorporateCardRepository {
    // ── Card Programmes ─────────────────────────────────────────────────

    async fn create_program(
        &self, org_id: Uuid, program_code: &str, name: &str, description: Option<&str>,
        issuer_bank: &str, card_network: &str, card_type: &str, currency_code: &str,
        default_single_purchase_limit: &str, default_monthly_limit: &str,
        default_cash_limit: &str, default_atm_limit: &str,
        allow_cash_withdrawal: bool, allow_international: bool,
        auto_deactivate_on_termination: bool, expense_matching_method: &str,
        billing_cycle_day: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardProgram> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corporate_card_programs
                (organization_id, program_code, name, description, issuer_bank,
                 card_network, card_type, currency_code,
                 default_single_purchase_limit, default_monthly_limit,
                 default_cash_limit, default_atm_limit,
                 allow_cash_withdrawal, allow_international,
                 auto_deactivate_on_termination, expense_matching_method,
                 billing_cycle_day, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,
                    $9,$10,$11,$12,
                    $13,$14,$15,$16,$17,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_code).bind(name).bind(description)
        .bind(issuer_bank).bind(card_network).bind(card_type).bind(currency_code)
        .bind(default_single_purchase_limit).bind(default_monthly_limit)
        .bind(default_cash_limit).bind(default_atm_limit)
        .bind(allow_cash_withdrawal).bind(allow_international)
        .bind(auto_deactivate_on_termination).bind(expense_matching_method)
        .bind(billing_cycle_day).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_program(&row))
    }

    async fn get_program(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CorporateCardProgram>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_programs WHERE organization_id=$1 AND program_code=$2",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_program(&r)))
    }

    async fn get_program_by_id(&self, id: Uuid) -> AtlasResult<Option<CorporateCardProgram>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_programs WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_program(&r)))
    }

    async fn list_programs(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CorporateCardProgram>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.corporate_card_programs WHERE organization_id=$1 AND is_active=true ORDER BY program_code",
            )
            .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.corporate_card_programs WHERE organization_id=$1 ORDER BY program_code",
            )
            .bind(org_id).fetch_all(&self.pool).await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_program(&r)).collect())
    }

    // ── Cards ───────────────────────────────────────────────────────────

    async fn create_card(
        &self, org_id: Uuid, program_id: Uuid, card_number_masked: &str,
        cardholder_name: &str, cardholder_id: Uuid, cardholder_email: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        status: &str, issue_date: chrono::NaiveDate, expiry_date: chrono::NaiveDate,
        single_purchase_limit: &str, monthly_limit: &str,
        cash_limit: &str, atm_limit: &str,
        gl_liability_account: Option<&str>, gl_expense_account: Option<&str>,
        cost_center: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCard> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corporate_cards
                (organization_id, program_id, card_number_masked,
                 cardholder_name, cardholder_id, cardholder_email,
                 department_id, department_name, status,
                 issue_date, expiry_date,
                 single_purchase_limit, monthly_limit, cash_limit, atm_limit,
                 gl_liability_account, gl_expense_account, cost_center,
                 created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,
                    $12,$13,$14,$15,
                    $16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(card_number_masked)
        .bind(cardholder_name).bind(cardholder_id).bind(cardholder_email)
        .bind(department_id).bind(department_name).bind(status)
        .bind(issue_date).bind(expiry_date)
        .bind(single_purchase_limit).bind(monthly_limit)
        .bind(cash_limit).bind(atm_limit)
        .bind(gl_liability_account).bind(gl_expense_account).bind(cost_center)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_card(&row))
    }

    async fn get_card(&self, id: Uuid) -> AtlasResult<Option<CorporateCard>> {
        let row = sqlx::query("SELECT * FROM _atlas.corporate_cards WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_card(&r)))
    }

    async fn get_card_by_masked_number(&self, org_id: Uuid, masked: &str) -> AtlasResult<Option<CorporateCard>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_cards WHERE organization_id=$1 AND card_number_masked=$2",
        )
        .bind(org_id).bind(masked)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_card(&r)))
    }

    async fn list_cards(
        &self, org_id: Uuid, program_id: Option<Uuid>,
        cardholder_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCard>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.corporate_cards
            WHERE organization_id=$1
              AND ($2::uuid IS NULL OR program_id=$2)
              AND ($3::uuid IS NULL OR cardholder_id=$3)
              AND ($4::text IS NULL OR status=$4)
            ORDER BY cardholder_name"#,
        )
        .bind(org_id).bind(program_id).bind(cardholder_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_card(&r)).collect())
    }

    async fn update_card_status(&self, id: Uuid, status: &str) -> AtlasResult<CorporateCard> {
        let row = sqlx::query(
            "UPDATE _atlas.corporate_cards SET status=$2, updated_at=now() WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_card(&row))
    }

    async fn update_card_limits(
        &self, id: Uuid, single_purchase: &str, monthly: &str,
        cash: &str, atm: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.corporate_cards
            SET single_purchase_limit=$2, monthly_limit=$3,
                cash_limit=$4, atm_limit=$5, updated_at=now()
            WHERE id=$1"#,
        )
        .bind(id).bind(single_purchase).bind(monthly).bind(cash).bind(atm)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_card_spend(&self, id: Uuid, amount: &str, balance: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.corporate_cards
            SET total_spend_current_cycle=$2,
                current_balance=$3, updated_at=now()
            WHERE id=$1"#,
        )
        .bind(id).bind(amount).bind(balance)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Transactions ────────────────────────────────────────────────────

    async fn create_transaction(
        &self, org_id: Uuid, card_id: Uuid, program_id: Uuid,
        transaction_reference: &str, posting_date: chrono::NaiveDate,
        transaction_date: chrono::NaiveDate, merchant_name: &str,
        merchant_category: Option<&str>, merchant_category_code: Option<&str>,
        amount: &str, currency_code: &str,
        original_amount: Option<&str>, original_currency: Option<&str>,
        exchange_rate: Option<&str>, transaction_type: &str,
    ) -> AtlasResult<CorporateCardTransaction> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corporate_card_transactions
                (organization_id, card_id, program_id, transaction_reference,
                 posting_date, transaction_date, merchant_name,
                 merchant_category, merchant_category_code,
                 amount, currency_code,
                 original_amount, original_currency, exchange_rate,
                 transaction_type)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,
                    $10,$11,
                    $12,$13,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(card_id).bind(program_id).bind(transaction_reference)
        .bind(posting_date).bind(transaction_date).bind(merchant_name)
        .bind(merchant_category).bind(merchant_category_code)
        .bind(amount).bind(currency_code)
        .bind(original_amount).bind(original_currency).bind(exchange_rate)
        .bind(transaction_type)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CorporateCardTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_transactions WHERE id=$1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn get_transaction_by_ref(
        &self, org_id: Uuid, reference: &str,
    ) -> AtlasResult<Option<CorporateCardTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_transactions WHERE organization_id=$1 AND transaction_reference=$2",
        )
        .bind(org_id).bind(reference)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn list_transactions(
        &self, org_id: Uuid, card_id: Option<Uuid>, status: Option<&str>,
        date_from: Option<chrono::NaiveDate>, date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<CorporateCardTransaction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.corporate_card_transactions
            WHERE organization_id=$1
              AND ($2::uuid IS NULL OR card_id=$2)
              AND ($3::text IS NULL OR status=$3)
              AND ($4::date IS NULL OR transaction_date >= $4)
              AND ($5::date IS NULL OR transaction_date <= $5)
            ORDER BY transaction_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(card_id).bind(status).bind(date_from).bind(date_to)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_transaction(&r)).collect())
    }

    async fn update_transaction_match(
        &self, id: Uuid, expense_report_id: Option<Uuid>,
        expense_line_id: Option<Uuid>, status: &str,
        matched_by: Option<Uuid>, match_confidence: Option<&str>,
    ) -> AtlasResult<CorporateCardTransaction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.corporate_card_transactions
            SET expense_report_id=$2, expense_line_id=$3, status=$4,
                matched_by=$5, match_confidence=$6,
                matched_at=CASE WHEN $4='matched' AND matched_at IS NULL THEN now() ELSE matched_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(expense_report_id).bind(expense_line_id)
        .bind(status).bind(matched_by).bind(match_confidence)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn update_transaction_dispute(
        &self, id: Uuid, reason: Option<&str>,
        dispute_date: Option<chrono::NaiveDate>,
        resolution: Option<&str>, status: &str,
    ) -> AtlasResult<CorporateCardTransaction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.corporate_card_transactions
            SET dispute_reason=$2, dispute_date=$3, dispute_resolution=$4,
                status=$5, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(reason).bind(dispute_date).bind(resolution).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    // ── Statements ──────────────────────────────────────────────────────

    async fn create_statement(
        &self, org_id: Uuid, program_id: Uuid, statement_number: &str,
        statement_date: chrono::NaiveDate,
        billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        opening_balance: &str, closing_balance: &str,
        total_charges: &str, total_credits: &str, total_payments: &str,
        total_fees: &str, total_interest: &str,
        payment_due_date: Option<chrono::NaiveDate>, minimum_payment: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardStatement> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corporate_card_statements
                (organization_id, program_id, statement_number, statement_date,
                 billing_period_start, billing_period_end,
                 opening_balance, closing_balance,
                 total_charges, total_credits, total_payments,
                 total_fees, total_interest,
                 payment_due_date, minimum_payment, imported_by)
            VALUES ($1,$2,$3,$4,$5,$6,
                    $7,$8,
                    $9,$10,$11,
                    $12,$13,
                    $14,$15,$16)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(statement_number).bind(statement_date)
        .bind(billing_period_start).bind(billing_period_end)
        .bind(opening_balance).bind(closing_balance)
        .bind(total_charges).bind(total_credits).bind(total_payments)
        .bind(total_fees).bind(total_interest)
        .bind(payment_due_date).bind(minimum_payment).bind(imported_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_statement(&row))
    }

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CorporateCardStatement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_statements WHERE id=$1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_statement(&r)))
    }

    async fn list_statements(
        &self, org_id: Uuid, program_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardStatement>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.corporate_card_statements
            WHERE organization_id=$1
              AND ($2::uuid IS NULL OR program_id=$2)
              AND ($3::text IS NULL OR status=$3)
            ORDER BY statement_date DESC"#,
        )
        .bind(org_id).bind(program_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_statement(&r)).collect())
    }

    async fn update_statement_counts(
        &self, id: Uuid, total: i32, matched: i32, unmatched: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.corporate_card_statements
            SET total_transaction_count=$2, matched_transaction_count=$3,
                unmatched_transaction_count=$4, updated_at=now()
            WHERE id=$1"#,
        )
        .bind(id).bind(total).bind(matched).bind(unmatched)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_statement_status(
        &self, id: Uuid, status: &str, payment_reference: Option<&str>,
    ) -> AtlasResult<CorporateCardStatement> {
        let row = sqlx::query(
            r#"UPDATE _atlas.corporate_card_statements
            SET status=$2, payment_reference=COALESCE($3, payment_reference),
                paid_at=CASE WHEN $2='paid' AND paid_at IS NULL THEN now() ELSE paid_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(payment_reference)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_statement(&row))
    }

    // ── Limit Overrides ─────────────────────────────────────────────────

    async fn create_limit_override(
        &self, org_id: Uuid, card_id: Uuid, override_type: &str,
        original_value: &str, new_value: &str, reason: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardLimitOverride> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corporate_card_limit_overrides
                (organization_id, card_id, override_type,
                 original_value, new_value, reason,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(card_id).bind(override_type)
        .bind(original_value).bind(new_value).bind(reason)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_limit_override(&row))
    }

    async fn get_limit_override(&self, id: Uuid) -> AtlasResult<Option<CorporateCardLimitOverride>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corporate_card_limit_overrides WHERE id=$1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_limit_override(&r)))
    }

    async fn list_limit_overrides(
        &self, card_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<CorporateCardLimitOverride>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.corporate_card_limit_overrides
            WHERE ($1::uuid IS NULL OR card_id=$1)
              AND ($2::text IS NULL OR status=$2)
            ORDER BY created_at DESC"#,
        )
        .bind(card_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_limit_override(&r)).collect())
    }

    async fn update_limit_override_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<CorporateCardLimitOverride> {
        let row = sqlx::query(
            r#"UPDATE _atlas.corporate_card_limit_overrides
            SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_limit_override(&row))
    }

    // ── Dashboard ───────────────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CorporateCardDashboardSummary> {
        let card_stats = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_cards,
                COUNT(*) as total_cards,
                COALESCE(SUM(total_spend_current_cycle::numeric) FILTER (WHERE status = 'active'), 0) as current_spend
            FROM _atlas.corporate_cards WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_cards: i64 = card_stats.try_get("active_cards").unwrap_or(0);
        let current_spend: serde_json::Value = card_stats.try_get("current_spend").unwrap_or(serde_json::json!(0));

        let txn_stats = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'unmatched') as unmatched,
                COUNT(*) FILTER (WHERE status = 'disputed') as disputed
            FROM _atlas.corporate_card_transactions WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let unmatched: i64 = txn_stats.try_get("unmatched").unwrap_or(0);
        let disputed: i64 = txn_stats.try_get("disputed").unwrap_or(0);

        let program_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.corporate_card_programs WHERE organization_id=$1 AND is_active=true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let unreconciled = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.corporate_card_statements WHERE organization_id=$1 AND status NOT IN ('reconciled','paid')",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let overrides_pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.corporate_card_limit_overrides WHERE organization_id=$1 AND status='pending'",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(CorporateCardDashboardSummary {
            total_active_cards: active_cards as i32,
            total_programs: program_count as i32,
            total_cards_by_status: serde_json::json!({}),
            total_spend_current_month: current_spend.to_string(),
            total_spend_previous_month: "0".to_string(),
            spend_change_percent: "0".to_string(),
            total_unmatched_transactions: unmatched as i32,
            total_unreconciled_statements: unreconciled as i32,
            total_disputed_transactions: disputed as i32,
            top_spenders: serde_json::json!([]),
            spend_by_category: serde_json::json!({}),
            limit_overrides_pending: overrides_pending as i32,
        })
    }
}
