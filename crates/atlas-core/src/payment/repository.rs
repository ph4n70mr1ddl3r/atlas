//! Payment Repository
//!
//! PostgreSQL storage for payment terms, payment batches, payments,
//! payment lines, scheduled payments, payment formats, and remittance advice.

use atlas_shared::{
    PaymentTerm, PaymentBatch, Payment, PaymentLine, ScheduledPayment,
    PaymentFormat, RemittanceAdvice,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for payment management data storage
#[async_trait]
pub trait PaymentRepository: Send + Sync {
    // Payment Terms
    async fn create_payment_term(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        due_days: i32,
        discount_days: Option<i32>,
        discount_percentage: Option<&str>,
        is_installment: bool,
        installment_count: Option<i32>,
        installment_frequency: Option<&str>,
        default_payment_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentTerm>;

    async fn get_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentTerm>>;
    async fn get_payment_term_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentTerm>>;
    async fn list_payment_terms(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentTerm>>;
    async fn delete_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Payment Batches
    async fn create_payment_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        bank_account_id: Option<Uuid>,
        payment_method: &str,
        currency_code: &str,
        selection_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentBatch>;

    async fn get_payment_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<PaymentBatch>>;
    async fn get_payment_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentBatch>>;
    async fn list_payment_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentBatch>>;
    async fn update_payment_batch_status(
        &self,
        id: Uuid,
        status: &str,
        action_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<PaymentBatch>;
    async fn update_payment_batch_totals(
        &self,
        id: Uuid,
        invoice_count: i32,
        payment_count: i32,
        payment_amount: &str,
        discount_taken: &str,
    ) -> AtlasResult<()>;

    // Payments
    async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        batch_id: Option<Uuid>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        payment_amount: &str,
        discount_taken: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        cash_account_code: Option<&str>,
        ap_account_code: Option<&str>,
        discount_account_code: Option<&str>,
        check_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Payment>;

    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<Payment>>;
    async fn get_payment_by_number(&self, org_id: Uuid, payment_number: &str) -> AtlasResult<Option<Payment>>;
    async fn list_payments(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>, batch_id: Option<Uuid>) -> AtlasResult<Vec<Payment>>;
    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        cleared_date: Option<chrono::NaiveDate>,
        cleared_by: Option<Uuid>,
        void_reason: Option<&str>,
        voided_by: Option<Uuid>,
    ) -> AtlasResult<Payment>;

    // Payment Lines
    async fn create_payment_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        line_number: i32,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>,
        invoice_amount: Option<&str>,
        amount_paid: &str,
        discount_taken: &str,
        withholding_amount: &str,
    ) -> AtlasResult<PaymentLine>;

    async fn list_payment_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<PaymentLine>>;

    // Scheduled Payments
    async fn create_scheduled_payment(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        scheduled_payment_date: chrono::NaiveDate,
        scheduled_amount: &str,
        installment_number: i32,
        payment_method: Option<&str>,
        bank_account_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledPayment>;

    async fn list_scheduled_payments(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ScheduledPayment>>;
    async fn update_scheduled_payment_status(
        &self,
        id: Uuid,
        status: &str,
        selected_batch_id: Option<Uuid>,
        payment_id: Option<Uuid>,
    ) -> AtlasResult<ScheduledPayment>;

    // Payment Formats
    async fn create_payment_format(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        format_type: &str,
        template_reference: Option<&str>,
        applicable_methods: serde_json::Value,
        is_system: bool,
    ) -> AtlasResult<PaymentFormat>;

    async fn list_payment_formats(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentFormat>>;
    async fn delete_payment_format(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Remittance Advice
    async fn create_remittance_advice(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        delivery_method: &str,
        delivery_address: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        payment_summary: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceAdvice>;

    async fn list_remittance_advices(&self, org_id: Uuid, payment_id: Option<Uuid>) -> AtlasResult<Vec<RemittanceAdvice>>;
    async fn update_remittance_advice_status(&self, id: Uuid, status: &str, failure_reason: Option<&str>) -> AtlasResult<RemittanceAdvice>;
}

/// PostgreSQL implementation
pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_payment_term(row: &sqlx::postgres::PgRow) -> PaymentTerm {
    PaymentTerm {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        due_days: row.get("due_days"),
        discount_days: row.get("discount_days"),
        discount_percentage: row.try_get("discount_percentage").ok().map(|v: serde_json::Value| v.to_string()),
        is_installment: row.get("is_installment"),
        installment_count: row.get("installment_count"),
        installment_frequency: row.get("installment_frequency"),
        default_payment_method: row.get("default_payment_method"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_payment_batch(row: &sqlx::postgres::PgRow) -> PaymentBatch {
    PaymentBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        name: row.get("name"),
        description: row.get("description"),
        payment_date: row.get("payment_date"),
        bank_account_id: row.get("bank_account_id"),
        payment_method: row.get("payment_method"),
        currency_code: row.get("currency_code"),
        selection_criteria: row.try_get("selection_criteria").unwrap_or(serde_json::json!({})),
        total_invoice_count: row.get("total_invoice_count"),
        total_payment_count: row.get("total_payment_count"),
        total_payment_amount: get_num(row, "total_payment_amount"),
        total_discount_taken: get_num(row, "total_discount_taken"),
        status: row.get("status"),
        selected_by: row.get("selected_by"),
        selected_at: row.get("selected_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        formatted_by: row.get("formatted_by"),
        formatted_at: row.get("formatted_at"),
        confirmed_by: row.get("confirmed_by"),
        confirmed_at: row.get("confirmed_at"),
        cancelled_by: row.get("cancelled_by"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        payment_file_name: row.get("payment_file_name"),
        payment_file_reference: row.get("payment_file_reference"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_payment(row: &sqlx::postgres::PgRow) -> Payment {
    Payment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        payment_number: row.get("payment_number"),
        batch_id: row.get("batch_id"),
        supplier_id: row.get("supplier_id"),
        supplier_number: row.get("supplier_number"),
        supplier_name: row.get("supplier_name"),
        supplier_site: row.get("supplier_site"),
        payment_date: row.get("payment_date"),
        payment_method: row.get("payment_method"),
        currency_code: row.get("currency_code"),
        payment_amount: get_num(row, "payment_amount"),
        discount_taken: get_num(row, "discount_taken"),
        bank_charges: get_num(row, "bank_charges"),
        bank_account_id: row.get("bank_account_id"),
        bank_account_name: row.get("bank_account_name"),
        cash_account_code: row.get("cash_account_code"),
        ap_account_code: row.get("ap_account_code"),
        discount_account_code: row.get("discount_account_code"),
        status: row.get("status"),
        check_number: row.get("check_number"),
        reference_number: row.get("reference_number"),
        voided_by: row.get("voided_by"),
        voided_at: row.get("voided_at"),
        void_reason: row.get("void_reason"),
        reissued_from_payment_id: row.get("reissued_from_payment_id"),
        reissued_payment_id: row.get("reissued_payment_id"),
        cleared_date: row.get("cleared_date"),
        cleared_by: row.get("cleared_by"),
        cleared_at: row.get("cleared_at"),
        journal_entry_id: row.get("journal_entry_id"),
        posted_at: row.get("posted_at"),
        remittance_sent: row.get("remittance_sent"),
        remittance_sent_at: row.get("remittance_sent_at"),
        remittance_method: row.get("remittance_method"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_payment_line(row: &sqlx::postgres::PgRow) -> PaymentLine {
    PaymentLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        payment_id: row.get("payment_id"),
        line_number: row.get("line_number"),
        invoice_id: row.get("invoice_id"),
        invoice_number: row.get("invoice_number"),
        invoice_date: row.get("invoice_date"),
        invoice_due_date: row.get("invoice_due_date"),
        invoice_amount: row.try_get("invoice_amount").ok().map(|v: serde_json::Value| v.to_string()),
        amount_paid: get_num(row, "amount_paid"),
        discount_taken: get_num(row, "discount_taken"),
        withholding_amount: get_num(row, "withholding_amount"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_scheduled_payment(row: &sqlx::postgres::PgRow) -> ScheduledPayment {
    ScheduledPayment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        invoice_id: row.get("invoice_id"),
        invoice_number: row.get("invoice_number"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        scheduled_payment_date: row.get("scheduled_payment_date"),
        scheduled_amount: get_num(row, "scheduled_amount"),
        installment_number: row.get("installment_number"),
        payment_method: row.get("payment_method"),
        bank_account_id: row.get("bank_account_id"),
        is_selected: row.get("is_selected"),
        selected_batch_id: row.get("selected_batch_id"),
        payment_id: row.get("payment_id"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_payment_format(row: &sqlx::postgres::PgRow) -> PaymentFormat {
    PaymentFormat {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        format_type: row.get("format_type"),
        template_reference: row.get("template_reference"),
        applicable_methods: row.try_get("applicable_methods").unwrap_or(serde_json::json!([])),
        is_system: row.get("is_system"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_remittance_advice(row: &sqlx::postgres::PgRow) -> RemittanceAdvice {
    RemittanceAdvice {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        payment_id: row.get("payment_id"),
        delivery_method: row.get("delivery_method"),
        delivery_address: row.get("delivery_address"),
        contact_name: row.get("contact_name"),
        contact_email: row.get("contact_email"),
        subject: row.get("subject"),
        body: row.get("body"),
        status: row.get("status"),
        sent_at: row.get("sent_at"),
        delivered_at: row.get("delivered_at"),
        failure_reason: row.get("failure_reason"),
        payment_summary: row.try_get("payment_summary").unwrap_or(serde_json::json!({})),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    // ========================================================================
    // Payment Terms
    // ========================================================================

    async fn create_payment_term(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        due_days: i32,
        discount_days: Option<i32>,
        discount_percentage: Option<&str>,
        is_installment: bool,
        installment_count: Option<i32>,
        installment_frequency: Option<&str>,
        default_payment_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentTerm> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payment_terms
                (organization_id, code, name, description, due_days,
                 discount_days, discount_percentage, is_installment,
                 installment_count, installment_frequency, default_payment_method,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, due_days = $5,
                    discount_days = $6, discount_percentage = $7::numeric,
                    is_installment = $8, installment_count = $9,
                    installment_frequency = $10, default_payment_method = $11,
                    effective_from = $12, effective_to = $13, is_active = true,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(due_days)
        .bind(discount_days).bind(discount_percentage).bind(is_installment)
        .bind(installment_count).bind(installment_frequency).bind(default_payment_method)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_payment_term(&row))
    }

    async fn get_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentTerm>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payment_terms WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment_term(&r)))
    }

    async fn get_payment_term_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentTerm>> {
        let row = sqlx::query("SELECT * FROM _atlas.payment_terms WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment_term(&r)))
    }

    async fn list_payment_terms(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentTerm>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.payment_terms WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_payment_term).collect())
    }

    async fn delete_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.payment_terms SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Payment Batches
    // ========================================================================

    async fn create_payment_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        bank_account_id: Option<Uuid>,
        payment_method: &str,
        currency_code: &str,
        selection_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payment_batches
                (organization_id, batch_number, name, description,
                 payment_date, bank_account_id, payment_method, currency_code,
                 selection_criteria, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_number).bind(name).bind(description)
        .bind(payment_date).bind(bank_account_id).bind(payment_method)
        .bind(currency_code).bind(selection_criteria).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_payment_batch(&row))
    }

    async fn get_payment_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<PaymentBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payment_batches WHERE organization_id = $1 AND batch_number = $2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment_batch(&r)))
    }

    async fn get_payment_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.payment_batches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment_batch(&r)))
    }

    async fn list_payment_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentBatch>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.payment_batches
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY payment_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_payment_batch).collect())
    }

    async fn update_payment_batch_status(
        &self,
        id: Uuid,
        status: &str,
        action_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<PaymentBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.payment_batches
            SET status = $2,
                selected_by = CASE WHEN $2 = 'selected' THEN $3 ELSE selected_by END,
                selected_at = CASE WHEN $2 = 'selected' THEN now() ELSE selected_at END,
                approved_by = CASE WHEN $2 = 'approved' THEN $3 ELSE approved_by END,
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                formatted_by = CASE WHEN $2 = 'formatted' THEN $3 ELSE formatted_by END,
                formatted_at = CASE WHEN $2 = 'formatted' THEN now() ELSE formatted_at END,
                confirmed_by = CASE WHEN $2 = 'confirmed' THEN $3 ELSE confirmed_by END,
                confirmed_at = CASE WHEN $2 = 'confirmed' THEN now() ELSE confirmed_at END,
                cancelled_by = CASE WHEN $2 = 'cancelled' THEN $3 ELSE cancelled_by END,
                cancelled_at = CASE WHEN $2 = 'cancelled' THEN now() ELSE cancelled_at END,
                cancellation_reason = CASE WHEN $2 = 'cancelled' THEN $4 ELSE cancellation_reason END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(action_by).bind(cancellation_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_payment_batch(&row))
    }

    async fn update_payment_batch_totals(
        &self,
        id: Uuid,
        invoice_count: i32,
        payment_count: i32,
        payment_amount: &str,
        discount_taken: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.payment_batches
            SET total_invoice_count = $2, total_payment_count = $3,
                total_payment_amount = $4::numeric, total_discount_taken = $5::numeric,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(invoice_count).bind(payment_count)
        .bind(payment_amount).bind(discount_taken)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Payments
    // ========================================================================

    async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        batch_id: Option<Uuid>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        payment_amount: &str,
        discount_taken: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        cash_account_code: Option<&str>,
        ap_account_code: Option<&str>,
        discount_account_code: Option<&str>,
        check_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Payment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payments
                (organization_id, payment_number, batch_id,
                 supplier_id, supplier_number, supplier_name, supplier_site,
                 payment_date, payment_method, currency_code,
                 payment_amount, discount_taken,
                 bank_account_id, bank_account_name,
                 cash_account_code, ap_account_code, discount_account_code,
                 check_number, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payment_number).bind(batch_id)
        .bind(supplier_id).bind(supplier_number).bind(supplier_name).bind(supplier_site)
        .bind(payment_date).bind(payment_method).bind(currency_code)
        .bind(payment_amount).bind(discount_taken)
        .bind(bank_account_id).bind(bank_account_name)
        .bind(cash_account_code).bind(ap_account_code).bind(discount_account_code)
        .bind(check_number).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_payment(&row))
    }

    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<Payment>> {
        let row = sqlx::query("SELECT * FROM _atlas.payments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment(&r)))
    }

    async fn get_payment_by_number(&self, org_id: Uuid, payment_number: &str) -> AtlasResult<Option<Payment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payments WHERE organization_id = $1 AND payment_number = $2"
        )
        .bind(org_id).bind(payment_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_payment(&r)))
    }

    async fn list_payments(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>, batch_id: Option<Uuid>) -> AtlasResult<Vec<Payment>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.payments
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR supplier_id = $3)
              AND ($4::uuid IS NULL OR batch_id = $4)
            ORDER BY payment_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(supplier_id).bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_payment).collect())
    }

    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        cleared_date: Option<chrono::NaiveDate>,
        cleared_by: Option<Uuid>,
        void_reason: Option<&str>,
        voided_by: Option<Uuid>,
    ) -> AtlasResult<Payment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.payments
            SET status = $2,
                cleared_date = CASE WHEN $2 = 'cleared' THEN $3 ELSE cleared_date END,
                cleared_by = CASE WHEN $2 = 'cleared' THEN $4 ELSE cleared_by END,
                cleared_at = CASE WHEN $2 = 'cleared' THEN now() ELSE cleared_at END,
                void_reason = CASE WHEN $2 = 'voided' THEN $5 ELSE void_reason END,
                voided_by = CASE WHEN $2 = 'voided' THEN $6 ELSE voided_by END,
                voided_at = CASE WHEN $2 = 'voided' THEN now() ELSE voided_at END,
                remittance_sent = CASE WHEN $2 = 'issued' THEN false ELSE remittance_sent END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(cleared_date).bind(cleared_by)
        .bind(void_reason).bind(voided_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_payment(&row))
    }

    // ========================================================================
    // Payment Lines
    // ========================================================================

    async fn create_payment_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        line_number: i32,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>,
        invoice_amount: Option<&str>,
        amount_paid: &str,
        discount_taken: &str,
        withholding_amount: &str,
    ) -> AtlasResult<PaymentLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payment_lines
                (organization_id, payment_id, line_number,
                 invoice_id, invoice_number, invoice_date, invoice_due_date,
                 invoice_amount, amount_paid, discount_taken, withholding_amount)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9::numeric, $10::numeric, $11::numeric)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payment_id).bind(line_number)
        .bind(invoice_id).bind(invoice_number).bind(invoice_date).bind(invoice_due_date)
        .bind(invoice_amount).bind(amount_paid).bind(discount_taken).bind(withholding_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_payment_line(&row))
    }

    async fn list_payment_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<PaymentLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.payment_lines WHERE payment_id = $1 ORDER BY line_number"
        )
        .bind(payment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_payment_line).collect())
    }

    // ========================================================================
    // Scheduled Payments
    // ========================================================================

    async fn create_scheduled_payment(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        scheduled_payment_date: chrono::NaiveDate,
        scheduled_amount: &str,
        installment_number: i32,
        payment_method: Option<&str>,
        bank_account_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledPayment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.scheduled_payments
                (organization_id, invoice_id, invoice_number,
                 supplier_id, supplier_name,
                 scheduled_payment_date, scheduled_amount,
                 installment_number, payment_method, bank_account_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(invoice_id).bind(invoice_number)
        .bind(supplier_id).bind(supplier_name)
        .bind(scheduled_payment_date).bind(scheduled_amount)
        .bind(installment_number).bind(payment_method).bind(bank_account_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_scheduled_payment(&row))
    }

    async fn list_scheduled_payments(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ScheduledPayment>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.scheduled_payments
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR supplier_id = $3)
            ORDER BY scheduled_payment_date, supplier_name
            "#,
        )
        .bind(org_id).bind(status).bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_scheduled_payment).collect())
    }

    async fn update_scheduled_payment_status(
        &self,
        id: Uuid,
        status: &str,
        selected_batch_id: Option<Uuid>,
        payment_id: Option<Uuid>,
    ) -> AtlasResult<ScheduledPayment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.scheduled_payments
            SET status = $2,
                is_selected = CASE WHEN $2 = 'selected' THEN true ELSE is_selected END,
                selected_batch_id = COALESCE($3, selected_batch_id),
                payment_id = COALESCE($4, payment_id),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(selected_batch_id).bind(payment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scheduled_payment(&row))
    }

    // ========================================================================
    // Payment Formats
    // ========================================================================

    async fn create_payment_format(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        format_type: &str,
        template_reference: Option<&str>,
        applicable_methods: serde_json::Value,
        is_system: bool,
    ) -> AtlasResult<PaymentFormat> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payment_formats
                (organization_id, code, name, description,
                 format_type, template_reference, applicable_methods, is_system)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, format_type = $5,
                    template_reference = $6, applicable_methods = $7,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(format_type).bind(template_reference).bind(applicable_methods).bind(is_system)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_payment_format(&row))
    }

    async fn list_payment_formats(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentFormat>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.payment_formats WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_payment_format).collect())
    }

    async fn delete_payment_format(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.payment_formats SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2 AND is_system = false"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Remittance Advice
    // ========================================================================

    async fn create_remittance_advice(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        delivery_method: &str,
        delivery_address: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        payment_summary: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceAdvice> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.remittance_advices
                (organization_id, payment_id, delivery_method, delivery_address,
                 contact_name, contact_email, subject, body,
                 payment_summary, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payment_id).bind(delivery_method).bind(delivery_address)
        .bind(contact_name).bind(contact_email).bind(subject).bind(body)
        .bind(payment_summary).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_remittance_advice(&row))
    }

    async fn list_remittance_advices(&self, org_id: Uuid, payment_id: Option<Uuid>) -> AtlasResult<Vec<RemittanceAdvice>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.remittance_advices
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR payment_id = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(payment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_remittance_advice).collect())
    }

    async fn update_remittance_advice_status(&self, id: Uuid, status: &str, failure_reason: Option<&str>) -> AtlasResult<RemittanceAdvice> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.remittance_advices
            SET status = $2,
                sent_at = CASE WHEN $2 = 'sent' THEN now() ELSE sent_at END,
                delivered_at = CASE WHEN $2 = 'delivered' THEN now() ELSE delivered_at END,
                failure_reason = $3,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(failure_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_remittance_advice(&row))
    }
}
