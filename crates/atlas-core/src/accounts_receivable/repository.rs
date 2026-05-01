//! Accounts Receivable Repository
//!
//! PostgreSQL storage for AR transactions, receipts, credit memos, and adjustments.

use atlas_shared::{
    ArTransaction, ArTransactionLine, ArReceipt, ArCreditMemo, ArAdjustment,
    ArAgingSummary, ArAgingByCustomer,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for AR data storage
#[async_trait]
pub trait AccountsReceivableRepository: Send + Sync {
    // Transactions
    async fn create_transaction(
        &self,
        org_id: Uuid,
        transaction_number: &str,
        transaction_type: &str,
        transaction_date: chrono::NaiveDate,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        currency_code: &str,
        entered_amount: &str,
        tax_amount: &str,
        total_amount: &str,
        payment_terms: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        purchase_order: Option<&str>,
        sales_rep: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArTransaction>;

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<ArTransaction>>;
    async fn get_transaction_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ArTransaction>>;
    async fn list_transactions(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>, transaction_type: Option<&str>) -> AtlasResult<Vec<ArTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, posted_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<ArTransaction>;
    async fn update_transaction_amounts(&self, id: Uuid, amount_due_remaining: &str, amount_applied: Option<&str>, status: &str) -> AtlasResult<ArTransaction>;
    async fn update_transaction_adjusted(&self, id: Uuid, amount_adjusted: &str) -> AtlasResult<ArTransaction>;
    async fn update_transaction_totals(&self, id: Uuid, entered_amount: &str, tax_amount: &str, total_amount: &str, amount_due_original: &str) -> AtlasResult<ArTransaction>;

    // Transaction Lines
    async fn create_transaction_line(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        line_number: i32,
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
    ) -> AtlasResult<ArTransactionLine>;
    async fn list_transaction_lines(&self, transaction_id: Uuid) -> AtlasResult<Vec<ArTransactionLine>>;

    // Receipts
    async fn create_receipt(
        &self,
        org_id: Uuid,
        receipt_number: &str,
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
    ) -> AtlasResult<ArReceipt>;
    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ArReceipt>>;
    async fn list_receipts(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArReceipt>>;
    async fn update_receipt_status(&self, id: Uuid, status: &str) -> AtlasResult<ArReceipt>;

    // Credit Memos
    async fn create_credit_memo(
        &self,
        org_id: Uuid,
        credit_memo_number: &str,
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
        total_amount: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArCreditMemo>;
    async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<ArCreditMemo>>;
    async fn list_credit_memos(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArCreditMemo>>;
    async fn update_credit_memo_status(&self, id: Uuid, status: &str) -> AtlasResult<ArCreditMemo>;

    // Adjustments
    async fn create_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
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
    ) -> AtlasResult<ArAdjustment>;
    async fn get_adjustment(&self, id: Uuid) -> AtlasResult<Option<ArAdjustment>>;
    async fn list_adjustments(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArAdjustment>>;
    async fn update_adjustment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ArAdjustment>;

    // Aging
    async fn get_aging_summary(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<ArAgingSummary>;
    async fn get_aging_by_customer(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<ArAgingByCustomer>>;
}

/// PostgreSQL implementation
pub struct PostgresAccountsReceivableRepository {
    pool: PgPool,
}

impl PostgresAccountsReceivableRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountsReceivableRepository for PostgresAccountsReceivableRepository {
    async fn create_transaction(
        &self,
        org_id: Uuid,
        transaction_number: &str,
        transaction_type: &str,
        transaction_date: chrono::NaiveDate,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        currency_code: &str,
        entered_amount: &str,
        tax_amount: &str,
        total_amount: &str,
        payment_terms: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        purchase_order: Option<&str>,
        sales_rep: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ar_transactions
                (organization_id, transaction_number, transaction_type, transaction_date,
                 customer_id, customer_number, customer_name,
                 currency_code, entered_amount, tax_amount, total_amount,
                 amount_due_original, amount_due_remaining,
                 payment_terms, due_date, gl_date,
                 reference_number, purchase_order, sales_rep, status, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::double precision, $10::double precision, $11::double precision,
                    $11::double precision, $11::double precision,
                    $12, $13, $14, $15, $16, $17, 'draft', $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transaction_number).bind(transaction_type).bind(transaction_date)
        .bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(currency_code).bind(entered_amount).bind(tax_amount).bind(total_amount)
        .bind(payment_terms).bind(due_date).bind(gl_date)
        .bind(reference_number).bind(purchase_order).bind(sales_rep)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<ArTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.ar_transactions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn get_transaction_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ArTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.ar_transactions WHERE organization_id = $1 AND transaction_number = $2")
            .bind(org_id).bind(number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn list_transactions(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>, transaction_type: Option<&str>) -> AtlasResult<Vec<ArTransaction>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.ar_transactions
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
              AND ($4::text IS NULL OR transaction_type = $4)
            ORDER BY transaction_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id).bind(transaction_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transaction).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, _posted_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<ArTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_transactions
            SET status = $2,
                notes = COALESCE($3, notes),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn update_transaction_amounts(&self, id: Uuid, amount_due_remaining: &str, amount_applied: Option<&str>, status: &str) -> AtlasResult<ArTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_transactions
            SET amount_due_remaining = $2::double precision,
                amount_applied = COALESCE($3::double precision, amount_applied),
                status = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(amount_due_remaining).bind(amount_applied).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn update_transaction_adjusted(&self, id: Uuid, amount_adjusted: &str) -> AtlasResult<ArTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_transactions
            SET amount_adjusted = $2::double precision,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(amount_adjusted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn update_transaction_totals(&self, id: Uuid, entered_amount: &str, tax_amount: &str, total_amount: &str, amount_due_original: &str) -> AtlasResult<ArTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_transactions
            SET entered_amount = $2::double precision,
                tax_amount = $3::double precision,
                total_amount = $4::double precision,
                amount_due_original = $5::double precision,
                amount_due_remaining = $5::double precision,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(entered_amount).bind(tax_amount).bind(total_amount).bind(amount_due_original)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn create_transaction_line(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        line_number: i32,
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ar_transaction_lines
                (organization_id, transaction_id, line_number, line_type,
                 description, item_code, item_description, unit_of_measure,
                 quantity, unit_price, line_amount, tax_amount,
                 tax_code, revenue_account, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::double precision, $12::double precision, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transaction_id).bind(line_number).bind(line_type)
        .bind(description).bind(item_code).bind(item_description).bind(unit_of_measure)
        .bind(quantity).bind(unit_price).bind(line_amount).bind(tax_amount)
        .bind(tax_code).bind(revenue_account).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transaction_line(&row))
    }

    async fn list_transaction_lines(&self, transaction_id: Uuid) -> AtlasResult<Vec<ArTransactionLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.ar_transaction_lines WHERE transaction_id = $1 ORDER BY line_number"
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transaction_line).collect())
    }

    async fn create_receipt(
        &self,
        org_id: Uuid,
        receipt_number: &str,
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ar_receipts
                (organization_id, receipt_number, receipt_date, receipt_type, receipt_method,
                 amount, currency_code, customer_id, customer_number, customer_name,
                 reference_number, bank_account_name, check_number, status, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::double precision, $7, $8, $9, $10,
                    $11, $12, $13, 'draft', $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(receipt_number).bind(receipt_date).bind(receipt_type).bind(receipt_method)
        .bind(amount).bind(currency_code).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(reference_number).bind(bank_account_name).bind(check_number).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_receipt(&row))
    }

    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<ArReceipt>> {
        let row = sqlx::query("SELECT * FROM _atlas.ar_receipts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_receipt(&r)))
    }

    async fn list_receipts(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArReceipt>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.ar_receipts
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY receipt_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_receipt).collect())
    }

    async fn update_receipt_status(&self, id: Uuid, status: &str) -> AtlasResult<ArReceipt> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_receipts
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_receipt(&row))
    }

    async fn create_credit_memo(
        &self,
        org_id: Uuid,
        credit_memo_number: &str,
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
        total_amount: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArCreditMemo> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ar_credit_memos
                (organization_id, credit_memo_number, customer_id, customer_number, customer_name,
                 transaction_id, transaction_number, credit_memo_date,
                 reason_code, reason_description, amount, tax_amount, total_amount,
                 status, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::double precision, $12::double precision, $13::double precision,
                    'draft', $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(credit_memo_number).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(transaction_id).bind(transaction_number).bind(credit_memo_date)
        .bind(reason_code).bind(reason_description).bind(amount).bind(tax_amount).bind(total_amount)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_credit_memo(&row))
    }

    async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<ArCreditMemo>> {
        let row = sqlx::query("SELECT * FROM _atlas.ar_credit_memos WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_credit_memo(&r)))
    }

    async fn list_credit_memos(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArCreditMemo>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.ar_credit_memos
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY credit_memo_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_credit_memo).collect())
    }

    async fn update_credit_memo_status(&self, id: Uuid, status: &str) -> AtlasResult<ArCreditMemo> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_credit_memos
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_credit_memo(&row))
    }

    async fn create_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ar_adjustments
                (organization_id, adjustment_number, transaction_id, transaction_number,
                 customer_id, customer_number, adjustment_date, gl_date,
                 adjustment_type, amount, receivable_account, adjustment_account,
                 reason_code, reason_description, status, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::double precision, $11, $12, $13, $14, 'draft', $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(adjustment_number).bind(transaction_id).bind(transaction_number)
        .bind(customer_id).bind(customer_number).bind(adjustment_date).bind(gl_date)
        .bind(adjustment_type).bind(amount).bind(receivable_account).bind(adjustment_account)
        .bind(reason_code).bind(reason_description).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_adjustment(&row))
    }

    async fn get_adjustment(&self, id: Uuid) -> AtlasResult<Option<ArAdjustment>> {
        let row = sqlx::query("SELECT * FROM _atlas.ar_adjustments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_adjustment(&r)))
    }

    async fn list_adjustments(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<ArAdjustment>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.ar_adjustments
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY adjustment_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_adjustment).collect())
    }

    async fn update_adjustment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ArAdjustment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ar_adjustments
            SET status = $2, approved_by = COALESCE($3, approved_by), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_adjustment(&row))
    }

    async fn get_aging_summary(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<ArAgingSummary> {
        let rows = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(amount_due_remaining), 0) as total_outstanding,
                COALESCE(SUM(CASE WHEN due_date < $2 THEN amount_due_remaining ELSE 0 END), 0) as total_overdue,
                COALESCE(SUM(CASE WHEN due_date >= $2 OR due_date IS NULL THEN amount_due_remaining ELSE 0 END), 0) as aging_current,
                COALESCE(SUM(CASE WHEN due_date >= $2 - INTERVAL '30 days' AND due_date < $2 THEN amount_due_remaining ELSE 0 END), 0) as aging_1_30,
                COALESCE(SUM(CASE WHEN due_date >= $2 - INTERVAL '60 days' AND due_date < $2 - INTERVAL '30 days' THEN amount_due_remaining ELSE 0 END), 0) as aging_31_60,
                COALESCE(SUM(CASE WHEN due_date >= $2 - INTERVAL '90 days' AND due_date < $2 - INTERVAL '60 days' THEN amount_due_remaining ELSE 0 END), 0) as aging_61_90,
                COALESCE(SUM(CASE WHEN due_date < $2 - INTERVAL '90 days' THEN amount_due_remaining ELSE 0 END), 0) as aging_91_plus,
                COUNT(DISTINCT customer_id) as customer_count,
                COUNT(DISTINCT CASE WHEN due_date < $2 THEN customer_id END) as overdue_customer_count
            FROM _atlas.ar_transactions
            WHERE organization_id = $1 AND status IN ('open', 'complete')
            "#,
        )
        .bind(org_id).bind(as_of_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(ArAgingSummary {
            organization_id: org_id,
            as_of_date,
            total_outstanding: format!("{:.2}", rows.try_get::<f64, _>("total_outstanding").unwrap_or(0.0)),
            total_overdue: format!("{:.2}", rows.try_get::<f64, _>("total_overdue").unwrap_or(0.0)),
            aging_current: format!("{:.2}", rows.try_get::<f64, _>("aging_current").unwrap_or(0.0)),
            aging_1_30: format!("{:.2}", rows.try_get::<f64, _>("aging_1_30").unwrap_or(0.0)),
            aging_31_60: format!("{:.2}", rows.try_get::<f64, _>("aging_31_60").unwrap_or(0.0)),
            aging_61_90: format!("{:.2}", rows.try_get::<f64, _>("aging_61_90").unwrap_or(0.0)),
            aging_91_plus: format!("{:.2}", rows.try_get::<f64, _>("aging_91_plus").unwrap_or(0.0)),
            customer_count: rows.try_get::<i64, _>("customer_count").unwrap_or(0) as i32,
            overdue_customer_count: rows.try_get::<i64, _>("overdue_customer_count").unwrap_or(0) as i32,
        })
    }

    async fn get_aging_by_customer(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<ArAgingByCustomer>> {
        let rows = sqlx::query(
            r#"
            SELECT customer_id,
                   COALESCE(customer_name, '') as customer_name,
                   COALESCE(customer_number, '') as customer_number,
                   SUM(amount_due_remaining) as total_outstanding,
                   SUM(CASE WHEN due_date >= $2 OR due_date IS NULL THEN amount_due_remaining ELSE 0 END) as current_amount,
                   SUM(CASE WHEN due_date >= $2 - INTERVAL '30 days' AND due_date < $2 THEN amount_due_remaining ELSE 0 END) as aging_1_30,
                   SUM(CASE WHEN due_date >= $2 - INTERVAL '60 days' AND due_date < $2 - INTERVAL '30 days' THEN amount_due_remaining ELSE 0 END) as aging_31_60,
                   SUM(CASE WHEN due_date >= $2 - INTERVAL '90 days' AND due_date < $2 - INTERVAL '60 days' THEN amount_due_remaining ELSE 0 END) as aging_61_90,
                   SUM(CASE WHEN due_date < $2 - INTERVAL '90 days' THEN amount_due_remaining ELSE 0 END) as aging_91_plus,
                   COUNT(*) as invoice_count
            FROM _atlas.ar_transactions
            WHERE organization_id = $1 AND status IN ('open', 'complete')
            GROUP BY customer_id, customer_name, customer_number
            ORDER BY total_outstanding DESC
            "#,
        )
        .bind(org_id).bind(as_of_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(rows.iter().map(|r| ArAgingByCustomer {
            customer_id: r.get("customer_id"),
            customer_name: r.try_get::<String, _>("customer_name").unwrap_or_default(),
            customer_number: r.try_get::<Option<String>, _>("customer_number").ok().flatten(),
            total_outstanding: format!("{:.2}", r.try_get::<f64, _>("total_outstanding").unwrap_or(0.0)),
            current_amount: format!("{:.2}", r.try_get::<f64, _>("current_amount").unwrap_or(0.0)),
            aging_1_30: format!("{:.2}", r.try_get::<f64, _>("aging_1_30").unwrap_or(0.0)),
            aging_31_60: format!("{:.2}", r.try_get::<f64, _>("aging_31_60").unwrap_or(0.0)),
            aging_61_90: format!("{:.2}", r.try_get::<f64, _>("aging_61_90").unwrap_or(0.0)),
            aging_91_plus: format!("{:.2}", r.try_get::<f64, _>("aging_91_plus").unwrap_or(0.0)),
            invoice_count: r.try_get::<i64, _>("invoice_count").unwrap_or(0) as i32,
        }).collect())
    }
}

// Row mapping helpers
use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> ArTransaction {
    ArTransaction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        transaction_number: row.get("transaction_number"),
        transaction_type: row.get("transaction_type"),
        transaction_date: row.get("transaction_date"),
        gl_date: row.get("gl_date"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        bill_to_site: row.get("bill_to_site"),
        currency_code: row.get("currency_code"),
        exchange_rate: row.get("exchange_rate"),
        exchange_rate_type: row.get("exchange_rate_type"),
        entered_amount: get_num(row, "entered_amount"),
        tax_amount: get_num(row, "tax_amount"),
        total_amount: get_num(row, "total_amount"),
        amount_due_original: get_num(row, "amount_due_original"),
        amount_due_remaining: get_num(row, "amount_due_remaining"),
        amount_applied: get_num(row, "amount_applied"),
        amount_adjusted: get_num(row, "amount_adjusted"),
        payment_terms: row.get("payment_terms"),
        due_date: row.get("due_date"),
        discount_due_date: row.get("discount_due_date"),
        reference_number: row.get("reference_number"),
        purchase_order: row.get("purchase_order"),
        sales_rep: row.get("sales_rep"),
        status: row.get("status"),
        receipt_method: row.get("receipt_method"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transaction_line(row: &sqlx::postgres::PgRow) -> ArTransactionLine {
    ArTransactionLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        transaction_id: row.get("transaction_id"),
        line_number: row.get("line_number"),
        description: row.get("description"),
        line_type: row.get("line_type"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        unit_of_measure: row.get("unit_of_measure"),
        quantity: row.get("quantity"),
        unit_price: row.get("unit_price"),
        line_amount: get_num(row, "line_amount"),
        tax_amount: get_num(row, "tax_amount"),
        tax_code: row.get("tax_code"),
        revenue_account: row.get("revenue_account"),
        tax_account: row.get("tax_account"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_receipt(row: &sqlx::postgres::PgRow) -> ArReceipt {
    ArReceipt {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        receipt_number: row.get("receipt_number"),
        receipt_date: row.get("receipt_date"),
        receipt_type: row.get("receipt_type"),
        receipt_method: row.get("receipt_method"),
        amount: get_num(row, "amount"),
        currency_code: row.get("currency_code"),
        exchange_rate: row.get("exchange_rate"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        reference_number: row.get("reference_number"),
        bank_account_name: row.get("bank_account_name"),
        check_number: row.get("check_number"),
        maturity_date: row.get("maturity_date"),
        status: row.get("status"),
        applied_transaction_number: row.get("applied_transaction_number"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_credit_memo(row: &sqlx::postgres::PgRow) -> ArCreditMemo {
    ArCreditMemo {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        credit_memo_number: row.get("credit_memo_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        transaction_id: row.get("transaction_id"),
        transaction_number: row.get("transaction_number"),
        credit_memo_date: row.get("credit_memo_date"),
        gl_date: row.get("gl_date"),
        reason_code: row.get("reason_code"),
        reason_description: row.get("reason_description"),
        amount: get_num(row, "amount"),
        tax_amount: get_num(row, "tax_amount"),
        total_amount: get_num(row, "total_amount"),
        status: row.get("status"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_adjustment(row: &sqlx::postgres::PgRow) -> ArAdjustment {
    ArAdjustment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        adjustment_number: row.get("adjustment_number"),
        transaction_id: row.get("transaction_id"),
        transaction_number: row.get("transaction_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        adjustment_date: row.get("adjustment_date"),
        gl_date: row.get("gl_date"),
        adjustment_type: row.get("adjustment_type"),
        amount: get_num(row, "amount"),
        receivable_account: row.get("receivable_account"),
        adjustment_account: row.get("adjustment_account"),
        reason_code: row.get("reason_code"),
        reason_description: row.get("reason_description"),
        status: row.get("status"),
        approved_by: row.get("approved_by"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
