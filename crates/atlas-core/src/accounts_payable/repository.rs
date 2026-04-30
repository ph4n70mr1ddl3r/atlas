//! Accounts Payable Repository
//!
//! PostgreSQL storage for AP invoices, lines, distributions, holds, and payments.

use atlas_shared::{
    ApInvoice, ApInvoiceLine, ApInvoiceDistribution, ApInvoiceHold, ApPayment,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for accounts payable data storage
#[async_trait]
pub trait AccountsPayableRepository: Send + Sync {
    // Invoices
    async fn create_invoice(
        &self,
        org_id: Uuid,
        invoice_number: &str,
        invoice_date: chrono::NaiveDate,
        invoice_type: &str,
        description: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        invoice_currency_code: &str,
        payment_currency_code: &str,
        exchange_rate: Option<&str>,
        exchange_rate_type: Option<&str>,
        exchange_date: Option<chrono::NaiveDate>,
        invoice_amount: &str,
        tax_amount: &str,
        total_amount: &str,
        payment_terms: Option<&str>,
        payment_method: Option<&str>,
        payment_due_date: Option<chrono::NaiveDate>,
        discount_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        po_number: Option<&str>,
        receipt_number: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoice>;

    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ApInvoice>>;
    async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ApInvoice>>;
    async fn list_invoices(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>, invoice_type: Option<&str>) -> AtlasResult<Vec<ApInvoice>>;
    async fn update_invoice_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        cancelled_reason: Option<&str>,
        cancelled_by: Option<Uuid>,
        cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ApInvoice>;
    async fn update_invoice_paid(&self, id: Uuid, amount_paid: &str) -> AtlasResult<ApInvoice>;
    async fn update_invoice_amounts(&self, id: Uuid, invoice_amount: &str, tax_amount: &str, total_amount: &str) -> AtlasResult<()>;

    // Invoice Lines
    async fn create_line(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        line_number: i32,
        line_type: &str,
        description: Option<&str>,
        amount: &str,
        unit_price: Option<&str>,
        quantity_invoiced: Option<&str>,
        unit_of_measure: Option<&str>,
        po_line_id: Option<Uuid>,
        po_line_number: Option<&str>,
        product_code: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceLine>;
    async fn list_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceLine>>;
    async fn delete_line(&self, id: Uuid) -> AtlasResult<()>;

    // Invoice Distributions
    async fn create_distribution(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_line_id: Option<Uuid>,
        distribution_line_number: i32,
        distribution_type: &str,
        account_combination: Option<&str>,
        description: Option<&str>,
        amount: &str,
        base_amount: Option<&str>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        gl_account: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        expenditure_type: Option<&str>,
        tax_code: Option<&str>,
        tax_recoverable: bool,
        tax_recoverable_amount: Option<&str>,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceDistribution>;
    async fn list_distributions(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceDistribution>>;

    // Holds
    async fn create_hold(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        hold_type: &str,
        hold_reason: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceHold>;
    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<ApInvoiceHold>>;
    async fn list_holds(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceHold>>;
    async fn list_active_holds(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceHold>>;
    async fn update_hold_status(
        &self,
        id: Uuid,
        status: &str,
        released_by: Option<Uuid>,
        release_reason: Option<&str>,
    ) -> AtlasResult<ApInvoiceHold>;

    // Payments
    async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        payment_currency_code: &str,
        payment_amount: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        invoice_ids: &serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApPayment>;
    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ApPayment>>;
    async fn list_payments(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ApPayment>>;
    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        confirmed_by: Option<Uuid>,
        cancelled_reason: Option<&str>,
    ) -> AtlasResult<ApPayment>;
}

/// PostgreSQL implementation
pub struct PostgresAccountsPayableRepository {
    pool: PgPool,
}

impl PostgresAccountsPayableRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_invoice(&self, row: &sqlx::postgres::PgRow) -> ApInvoice {
        fn num_to_str(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: Option<serde_json::Value> = row.try_get(col).ok().flatten();
            v.map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_else(|| "0".to_string())
        }
        ApInvoice {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            invoice_number: row.get("invoice_number"),
            invoice_date: row.get("invoice_date"),
            invoice_type: row.get("invoice_type"),
            description: row.get("description"),
            supplier_id: row.get("supplier_id"),
            supplier_number: row.get("supplier_number"),
            supplier_name: row.get("supplier_name"),
            supplier_site: row.get("supplier_site"),
            invoice_currency_code: row.get("invoice_currency_code"),
            payment_currency_code: row.get("payment_currency_code"),
            exchange_rate: row.get("exchange_rate"),
            exchange_rate_type: row.get("exchange_rate_type"),
            exchange_date: row.get("exchange_date"),
            invoice_amount: num_to_str(row, "invoice_amount"),
            tax_amount: num_to_str(row, "tax_amount"),
            total_amount: num_to_str(row, "total_amount"),
            amount_paid: num_to_str(row, "amount_paid"),
            amount_remaining: num_to_str(row, "amount_remaining"),
            discount_available: num_to_str(row, "discount_available"),
            discount_taken: num_to_str(row, "discount_taken"),
            payment_terms: row.get("payment_terms"),
            payment_method: row.get("payment_method"),
            payment_due_date: row.get("payment_due_date"),
            discount_date: row.get("discount_date"),
            gl_date: row.get("gl_date"),
            gl_posted_date: row.get("gl_posted_date"),
            status: row.get("status"),
            approval_status: row.get("approval_status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            cancelled_reason: row.get("cancelled_reason"),
            cancelled_by: row.get("cancelled_by"),
            cancelled_at: row.get("cancelled_at"),
            po_number: row.get("po_number"),
            receipt_number: row.get("receipt_number"),
            source: row.get("source"),
            batch_id: row.get("batch_id"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> ApInvoiceLine {
        fn num_to_str(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: Option<serde_json::Value> = row.try_get(col).ok().flatten();
            v.map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_else(|| "0".to_string())
        }
        ApInvoiceLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            invoice_id: row.get("invoice_id"),
            line_number: row.get("line_number"),
            line_type: row.get("line_type"),
            description: row.get("description"),
            amount: num_to_str(row, "amount"),
            unit_price: row.get("unit_price"),
            quantity_invoiced: row.get("quantity_invoiced"),
            unit_of_measure: row.get("unit_of_measure"),
            po_line_id: row.get("po_line_id"),
            po_line_number: row.get("po_line_number"),
            product_code: row.get("product_code"),
            tax_code: row.get("tax_code"),
            tax_amount: row.get("tax_amount"),
            asset_category_code: row.get("asset_category_code"),
            project_id: row.get("project_id"),
            task_id: row.get("task_id"),
            expenditure_type: row.get("expenditure_type"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_distribution(&self, row: &sqlx::postgres::PgRow) -> ApInvoiceDistribution {
        fn num_to_str(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: Option<serde_json::Value> = row.try_get(col).ok().flatten();
            v.map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_else(|| "0".to_string())
        }
        ApInvoiceDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            invoice_id: row.get("invoice_id"),
            invoice_line_id: row.get("invoice_line_id"),
            distribution_line_number: row.get("distribution_line_number"),
            distribution_type: row.get("distribution_type"),
            account_combination: row.get("account_combination"),
            description: row.get("description"),
            amount: num_to_str(row, "amount"),
            base_amount: row.get("base_amount"),
            currency_code: row.get("currency_code"),
            exchange_rate: row.get("exchange_rate"),
            gl_account: row.get("gl_account"),
            cost_center: row.get("cost_center"),
            department: row.get("department"),
            project_id: row.get("project_id"),
            task_id: row.get("task_id"),
            expenditure_type: row.get("expenditure_type"),
            tax_code: row.get("tax_code"),
            tax_recoverable: row.get("tax_recoverable"),
            tax_recoverable_amount: row.get("tax_recoverable_amount"),
            accounting_date: row.get("accounting_date"),
            posted_status: row.get("posted_status"),
            posted_at: row.get("posted_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_hold(&self, row: &sqlx::postgres::PgRow) -> ApInvoiceHold {
        ApInvoiceHold {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            invoice_id: row.get("invoice_id"),
            hold_type: row.get("hold_type"),
            hold_reason: row.get("hold_reason"),
            hold_status: row.get("hold_status"),
            released_by: row.get("released_by"),
            released_at: row.get("released_at"),
            release_reason: row.get("release_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_payment(&self, row: &sqlx::postgres::PgRow) -> ApPayment {
        fn num_to_str(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: Option<serde_json::Value> = row.try_get(col).ok().flatten();
            v.map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_else(|| "0".to_string())
        }
        ApPayment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            payment_number: row.get("payment_number"),
            payment_date: row.get("payment_date"),
            payment_method: row.get("payment_method"),
            payment_currency_code: row.get("payment_currency_code"),
            payment_amount: num_to_str(row, "payment_amount"),
            bank_account_id: row.get("bank_account_id"),
            bank_account_name: row.get("bank_account_name"),
            payment_document: row.get("payment_document"),
            status: row.get("status"),
            supplier_id: row.get("supplier_id"),
            supplier_number: row.get("supplier_number"),
            supplier_name: row.get("supplier_name"),
            invoice_ids: row.get("invoice_ids"),
            confirmed_by: row.get("confirmed_by"),
            confirmed_at: row.get("confirmed_at"),
            cancelled_reason: row.get("cancelled_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl AccountsPayableRepository for PostgresAccountsPayableRepository {
    // ========================================================================
    // Invoices
    // ========================================================================

    async fn create_invoice(
        &self,
        org_id: Uuid,
        invoice_number: &str,
        invoice_date: chrono::NaiveDate,
        invoice_type: &str,
        description: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        invoice_currency_code: &str,
        payment_currency_code: &str,
        exchange_rate: Option<&str>,
        exchange_rate_type: Option<&str>,
        exchange_date: Option<chrono::NaiveDate>,
        invoice_amount: &str,
        tax_amount: &str,
        total_amount: &str,
        payment_terms: Option<&str>,
        payment_method: Option<&str>,
        payment_due_date: Option<chrono::NaiveDate>,
        discount_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        po_number: Option<&str>,
        receipt_number: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoice> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ap_invoices
                (organization_id, invoice_number, invoice_date, invoice_type, description,
                 supplier_id, supplier_number, supplier_name, supplier_site,
                 invoice_currency_code, payment_currency_code,
                 exchange_rate, exchange_rate_type, exchange_date,
                 invoice_amount, tax_amount, total_amount,
                 amount_paid, amount_remaining,
                 discount_available, discount_taken,
                 payment_terms, payment_method,
                 payment_due_date, discount_date, gl_date,
                 po_number, receipt_number, source,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, $15::numeric, $16::numeric, $17::numeric,
                    0, $17::numeric, 0, 0, $18, $19, $20, $21, $22, $23, $24, $25, $26)
            ON CONFLICT (organization_id, invoice_number) DO UPDATE
                SET invoice_date = $3, invoice_type = $4, description = $5,
                    supplier_id = $6, supplier_number = $7, supplier_name = $8,
                    supplier_site = $9, invoice_currency_code = $10,
                    payment_currency_code = $11, exchange_rate = $12,
                    exchange_rate_type = $13, exchange_date = $14,
                    invoice_amount = $15::numeric, tax_amount = $16::numeric,
                    total_amount = $17::numeric, amount_remaining = $17::numeric,
                    payment_terms = $18, payment_method = $19,
                    payment_due_date = $20, discount_date = $21, gl_date = $22,
                    po_number = $23, receipt_number = $24, source = $25,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(invoice_number).bind(invoice_date).bind(invoice_type).bind(description)
        .bind(supplier_id).bind(supplier_number).bind(supplier_name).bind(supplier_site)
        .bind(invoice_currency_code).bind(payment_currency_code)
        .bind(exchange_rate).bind(exchange_rate_type).bind(exchange_date)
        .bind(invoice_amount).bind(tax_amount).bind(total_amount)
        .bind(payment_terms).bind(payment_method)
        .bind(payment_due_date).bind(discount_date).bind(gl_date)
        .bind(po_number).bind(receipt_number).bind(source)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_invoice(&row))
    }

    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ApInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.ap_invoices WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_invoice(&r)))
    }

    async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ApInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND invoice_number = $2"
        )
        .bind(org_id).bind(invoice_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_invoice(&r)))
    }

    async fn list_invoices(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>, invoice_type: Option<&str>) -> AtlasResult<Vec<ApInvoice>> {
        let rows = match (supplier_id, status, invoice_type) {
            (Some(sid), Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND supplier_id = $2 AND status = $3 AND invoice_type = $4 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).bind(s).bind(t).fetch_all(&self.pool).await,
            (Some(sid), Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND supplier_id = $2 AND status = $3 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).bind(s).fetch_all(&self.pool).await,
            (Some(sid), None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND supplier_id = $2 AND invoice_type = $3 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).bind(t).fetch_all(&self.pool).await,
            (Some(sid), None, None) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND supplier_id = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).fetch_all(&self.pool).await,
            (None, Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND status = $2 AND invoice_type = $3 ORDER BY created_at DESC"
            ).bind(org_id).bind(s).bind(t).fetch_all(&self.pool).await,
            (None, Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 AND invoice_type = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(t).fetch_all(&self.pool).await,
            (None, None, None) => sqlx::query(
                "SELECT * FROM _atlas.ap_invoices WHERE organization_id = $1 ORDER BY created_at DESC"
            ).bind(org_id).fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_invoice(r)).collect())
    }

    async fn update_invoice_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        cancelled_reason: Option<&str>,
        cancelled_by: Option<Uuid>,
        cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ApInvoice> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ap_invoices
            SET status = $2, approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE approved_at END,
                cancelled_reason = $4, cancelled_by = $5, cancelled_at = $6,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(cancelled_reason)
        .bind(cancelled_by).bind(cancelled_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_invoice(&row))
    }

    async fn update_invoice_paid(&self, id: Uuid, amount_paid: &str) -> AtlasResult<ApInvoice> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ap_invoices
            SET status = 'paid', amount_paid = $2::numeric,
                amount_remaining = total_amount - $2::numeric,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(amount_paid)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_invoice(&row))
    }

    async fn update_invoice_amounts(&self, id: Uuid, invoice_amount: &str, tax_amount: &str, total_amount: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.ap_invoices
            SET invoice_amount = $2::numeric, tax_amount = $3::numeric,
                total_amount = $4::numeric, amount_remaining = $4::numeric - amount_paid,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(invoice_amount).bind(tax_amount).bind(total_amount)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Invoice Lines
    // ========================================================================

    async fn create_line(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        line_number: i32,
        line_type: &str,
        description: Option<&str>,
        amount: &str,
        unit_price: Option<&str>,
        quantity_invoiced: Option<&str>,
        unit_of_measure: Option<&str>,
        po_line_id: Option<Uuid>,
        po_line_number: Option<&str>,
        product_code: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ap_invoice_lines
                (organization_id, invoice_id, line_number, line_type, description,
                 amount, unit_price, quantity_invoiced, unit_of_measure,
                 po_line_id, po_line_number, product_code,
                 tax_code, tax_amount, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(invoice_id).bind(line_number).bind(line_type).bind(description)
        .bind(amount).bind(unit_price).bind(quantity_invoiced).bind(unit_of_measure)
        .bind(po_line_id).bind(po_line_number).bind(product_code)
        .bind(tax_code).bind(tax_amount).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_line(&row))
    }

    async fn list_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.ap_invoice_lines WHERE invoice_id = $1 ORDER BY line_number"
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_line(r)).collect())
    }

    async fn delete_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.ap_invoice_lines WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Invoice Distributions
    // ========================================================================

    async fn create_distribution(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_line_id: Option<Uuid>,
        distribution_line_number: i32,
        distribution_type: &str,
        account_combination: Option<&str>,
        description: Option<&str>,
        amount: &str,
        base_amount: Option<&str>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        gl_account: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        expenditure_type: Option<&str>,
        tax_code: Option<&str>,
        tax_recoverable: bool,
        tax_recoverable_amount: Option<&str>,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceDistribution> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ap_invoice_distributions
                (organization_id, invoice_id, invoice_line_id,
                 distribution_line_number, distribution_type,
                 account_combination, description, amount,
                 base_amount, currency_code, exchange_rate,
                 gl_account, cost_center, department,
                 project_id, task_id, expenditure_type,
                 tax_code, tax_recoverable, tax_recoverable_amount,
                 accounting_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric,
                    $9::numeric, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20::numeric,
                    $21, $22)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(invoice_id).bind(invoice_line_id)
        .bind(distribution_line_number).bind(distribution_type)
        .bind(account_combination).bind(description).bind(amount)
        .bind(base_amount).bind(currency_code).bind(exchange_rate)
        .bind(gl_account).bind(cost_center).bind(department)
        .bind(project_id).bind(task_id).bind(expenditure_type)
        .bind(tax_code).bind(tax_recoverable).bind(tax_recoverable_amount)
        .bind(accounting_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_distribution(&row))
    }

    async fn list_distributions(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.ap_invoice_distributions WHERE invoice_id = $1 ORDER BY distribution_line_number"
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(r)).collect())
    }

    // ========================================================================
    // Holds
    // ========================================================================

    async fn create_hold(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        hold_type: &str,
        hold_reason: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceHold> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ap_invoice_holds
                (organization_id, invoice_id, hold_type, hold_reason, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(invoice_id).bind(hold_type).bind(hold_reason).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_hold(&row))
    }

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<ApInvoiceHold>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.ap_invoice_holds WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_hold(&r)))
    }

    async fn list_holds(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceHold>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.ap_invoice_holds WHERE invoice_id = $1 ORDER BY created_at DESC"
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_hold(r)).collect())
    }

    async fn list_active_holds(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceHold>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.ap_invoice_holds WHERE invoice_id = $1 AND hold_status = 'active' ORDER BY created_at DESC"
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_hold(r)).collect())
    }

    async fn update_hold_status(
        &self,
        id: Uuid,
        status: &str,
        released_by: Option<Uuid>,
        release_reason: Option<&str>,
    ) -> AtlasResult<ApInvoiceHold> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ap_invoice_holds
            SET hold_status = $2, released_by = $3, released_at = now(),
                release_reason = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(released_by).bind(release_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_hold(&row))
    }

    // ========================================================================
    // Payments
    // ========================================================================

    async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        payment_currency_code: &str,
        payment_amount: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        invoice_ids: &serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApPayment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.ap_payments
                (organization_id, payment_number, payment_date, payment_method,
                 payment_currency_code, payment_amount,
                 bank_account_id, bank_account_name, payment_document,
                 supplier_id, supplier_number, supplier_name,
                 invoice_ids, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9,
                    $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payment_number).bind(payment_date).bind(payment_method)
        .bind(payment_currency_code).bind(payment_amount)
        .bind(bank_account_id).bind(bank_account_name).bind(payment_document)
        .bind(supplier_id).bind(supplier_number).bind(supplier_name)
        .bind(invoice_ids).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payment(&row))
    }

    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ApPayment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.ap_payments WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_payment(&r)))
    }

    async fn list_payments(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ApPayment>> {
        let rows = match (supplier_id, status) {
            (Some(sid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.ap_payments WHERE organization_id = $1 AND supplier_id = $2 AND status = $3 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).bind(s).fetch_all(&self.pool).await,
            (Some(sid), None) => sqlx::query(
                "SELECT * FROM _atlas.ap_payments WHERE organization_id = $1 AND supplier_id = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(sid).fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.ap_payments WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.ap_payments WHERE organization_id = $1 ORDER BY created_at DESC"
            ).bind(org_id).fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_payment(r)).collect())
    }

    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        confirmed_by: Option<Uuid>,
        cancelled_reason: Option<&str>,
    ) -> AtlasResult<ApPayment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.ap_payments
            SET status = $2, confirmed_by = $3,
                confirmed_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE confirmed_at END,
                cancelled_reason = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(confirmed_by).bind(cancelled_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payment(&row))
    }
}
