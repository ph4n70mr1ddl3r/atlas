//! AutoInvoice Repository
//!
//! PostgreSQL storage for AutoInvoice batches, lines, rules, and results.

use atlas_shared::{
    AutoInvoiceBatch, AutoInvoiceLine, AutoInvoiceGroupingRule,
    AutoInvoiceValidationRule, AutoInvoiceResult, AutoInvoiceResultLine,
    AutoInvoiceValidationError,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for AutoInvoice data storage
#[async_trait]
pub trait AutoInvoiceRepository: Send + Sync {
    // Grouping Rules
    async fn create_grouping_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        transaction_types: serde_json::Value,
        group_by_fields: serde_json::Value,
        line_order_by: serde_json::Value,
        is_default: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceGroupingRule>;

    async fn get_grouping_rule(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceGroupingRule>>;
    async fn get_grouping_rule_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<AutoInvoiceGroupingRule>>;
    async fn get_default_grouping_rule(&self, org_id: Uuid) -> AtlasResult<Option<AutoInvoiceGroupingRule>>;
    async fn list_grouping_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceGroupingRule>>;
    async fn delete_grouping_rule(&self, id: Uuid) -> AtlasResult<()>;

    // Validation Rules
    async fn create_validation_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        field_name: &str,
        validation_type: &str,
        validation_expression: Option<&str>,
        error_message: &str,
        is_fatal: bool,
        transaction_types: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceValidationRule>;

    async fn get_validation_rules(&self, org_id: Uuid, transaction_type: Option<&str>) -> AtlasResult<Vec<AutoInvoiceValidationRule>>;
    async fn list_validation_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceValidationRule>>;
    async fn delete_validation_rule(&self, id: Uuid) -> AtlasResult<()>;

    // Batches
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        batch_source: &str,
        description: Option<&str>,
        grouping_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceBatch>;

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceBatch>>;
    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<AutoInvoiceBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AutoInvoiceBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<AutoInvoiceBatch>;
    async fn update_batch_counts(
        &self,
        id: Uuid,
        total_lines: i32,
        valid_lines: i32,
        invalid_lines: i32,
        invoices_created: i32,
        invoices_total_amount: &str,
        validation_errors: serde_json::Value,
    ) -> AtlasResult<()>;

    // Lines
    async fn create_line(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        line_number: i32,
        source_line_id: Option<&str>,
        transaction_type: &str,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        bill_to_customer_id: Option<Uuid>,
        bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>,
        ship_to_site_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_price: &str,
        line_amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>,
        receivable_account_code: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        sales_rep_id: Option<Uuid>,
        sales_rep_name: Option<&str>,
        memo_line: Option<&str>,
        reference_number: Option<&str>,
        sales_order_number: Option<&str>,
        sales_order_line: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceLine>;

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceLine>>;
    async fn list_lines_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceLine>>;
    async fn list_lines_by_status(&self, batch_id: Uuid, status: &str) -> AtlasResult<Vec<AutoInvoiceLine>>;
    async fn update_line_status(&self, id: Uuid, status: &str, validation_errors: serde_json::Value) -> AtlasResult<()>;
    async fn update_line_invoice(&self, id: Uuid, invoice_id: Uuid, invoice_line_number: i32) -> AtlasResult<()>;

    // Results
    async fn create_result(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        invoice_number: &str,
        transaction_type: &str,
        customer_id: Option<Uuid>,
        bill_to_customer_id: Option<Uuid>,
        bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>,
        ship_to_site_id: Option<Uuid>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        receivable_account_code: Option<&str>,
        sales_rep_id: Option<Uuid>,
        sales_order_number: Option<&str>,
        reference_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceResult>;

    async fn get_result(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceResult>>;
    async fn get_result_by_invoice_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<AutoInvoiceResult>>;
    async fn list_results_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResult>>;
    async fn update_result_totals(
        &self,
        id: Uuid,
        subtotal: &str,
        tax_amount: &str,
        total_amount: &str,
        line_count: i32,
    ) -> AtlasResult<()>;
    async fn update_result_status(&self, id: Uuid, status: &str) -> AtlasResult<AutoInvoiceResult>;

    // Result Lines
    async fn create_result_line(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        line_number: i32,
        source_line_id: Option<&str>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_price: &str,
        line_amount: &str,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        revenue_account_code: Option<&str>,
        sales_order_number: Option<&str>,
        sales_order_line: Option<&str>,
    ) -> AtlasResult<AutoInvoiceResultLine>;

    async fn list_result_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResultLine>>;

    // Summary
    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::AutoInvoiceSummary>;
}

/// PostgreSQL implementation
pub struct PostgresAutoInvoiceRepository {
    pool: PgPool,
}

impl PostgresAutoInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_grouping_rule(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceGroupingRule {
        AutoInvoiceGroupingRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            transaction_types: row.get("transaction_types"),
            group_by_fields: row.get("group_by_fields"),
            line_order_by: row.get("line_order_by"),
            is_default: row.get("is_default"),
            is_active: row.get("is_active"),
            priority: row.get("priority"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_validation_rule(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceValidationRule {
        AutoInvoiceValidationRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            field_name: row.get("field_name"),
            validation_type: row.get("validation_type"),
            validation_expression: row.get("validation_expression"),
            error_message: row.get("error_message"),
            is_fatal: row.get("is_fatal"),
            transaction_types: row.get("transaction_types"),
            is_active: row.get("is_active"),
            priority: row.get("priority"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_batch(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceBatch {
        AutoInvoiceBatch {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            batch_number: row.get("batch_number"),
            batch_source: row.get("batch_source"),
            description: row.get("description"),
            status: row.get("status"),
            total_lines: row.get("total_lines"),
            valid_lines: row.get("valid_lines"),
            invalid_lines: row.get("invalid_lines"),
            invoices_created: row.get("invoices_created"),
            invoices_total_amount: row.try_get("invoices_total_amount").unwrap_or(serde_json::json!("0")).to_string(),
            grouping_rule_id: row.get("grouping_rule_id"),
            validation_errors: row.get("validation_errors"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceLine {
        AutoInvoiceLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            batch_id: row.get("batch_id"),
            line_number: row.get("line_number"),
            source_line_id: row.get("source_line_id"),
            transaction_type: row.get("transaction_type"),
            customer_id: row.get("customer_id"),
            customer_number: row.get("customer_number"),
            customer_name: row.get("customer_name"),
            bill_to_customer_id: row.get("bill_to_customer_id"),
            bill_to_site_id: row.get("bill_to_site_id"),
            ship_to_customer_id: row.get("ship_to_customer_id"),
            ship_to_site_id: row.get("ship_to_site_id"),
            item_code: row.get("item_code"),
            item_description: row.get("item_description"),
            quantity: row.try_get("quantity").ok().map(|v: serde_json::Value| v.to_string()),
            unit_of_measure: row.get("unit_of_measure"),
            unit_price: row.try_get("unit_price").unwrap_or(serde_json::json!("0")).to_string(),
            line_amount: row.try_get("line_amount").unwrap_or(serde_json::json!("0")).to_string(),
            currency_code: row.get("currency_code"),
            exchange_rate: row.try_get("exchange_rate").ok().map(|v: serde_json::Value| v.to_string()),
            transaction_date: row.get("transaction_date"),
            gl_date: row.get("gl_date"),
            due_date: row.get("due_date"),
            revenue_account_code: row.get("revenue_account_code"),
            receivable_account_code: row.get("receivable_account_code"),
            tax_code: row.get("tax_code"),
            tax_amount: row.try_get("tax_amount").ok().map(|v: serde_json::Value| v.to_string()),
            sales_rep_id: row.get("sales_rep_id"),
            sales_rep_name: row.get("sales_rep_name"),
            memo_line: row.get("memo_line"),
            reference_number: row.get("reference_number"),
            sales_order_number: row.get("sales_order_number"),
            sales_order_line: row.get("sales_order_line"),
            status: row.get("status"),
            validation_errors: row.get("validation_errors"),
            invoice_id: row.get("invoice_id"),
            invoice_line_number: row.get("invoice_line_number"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_result(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceResult {
        AutoInvoiceResult {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            batch_id: row.get("batch_id"),
            invoice_number: row.get("invoice_number"),
            transaction_type: row.get("transaction_type"),
            customer_id: row.get("customer_id"),
            bill_to_customer_id: row.get("bill_to_customer_id"),
            bill_to_site_id: row.get("bill_to_site_id"),
            ship_to_customer_id: row.get("ship_to_customer_id"),
            ship_to_site_id: row.get("ship_to_site_id"),
            currency_code: row.get("currency_code"),
            exchange_rate: row.try_get("exchange_rate").ok().map(|v: serde_json::Value| v.to_string()),
            transaction_date: row.get("transaction_date"),
            gl_date: row.get("gl_date"),
            due_date: row.get("due_date"),
            subtotal: row.try_get("subtotal").unwrap_or(serde_json::json!("0")).to_string(),
            tax_amount: row.try_get("tax_amount").unwrap_or(serde_json::json!("0")).to_string(),
            total_amount: row.try_get("total_amount").unwrap_or(serde_json::json!("0")).to_string(),
            line_count: row.get("line_count"),
            receivable_account_code: row.get("receivable_account_code"),
            sales_rep_id: row.get("sales_rep_id"),
            sales_order_number: row.get("sales_order_number"),
            reference_number: row.get("reference_number"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_result_line(&self, row: &sqlx::postgres::PgRow) -> AutoInvoiceResultLine {
        AutoInvoiceResultLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            invoice_id: row.get("invoice_id"),
            line_number: row.get("line_number"),
            source_line_id: row.get("source_line_id"),
            item_code: row.get("item_code"),
            item_description: row.get("item_description"),
            quantity: row.try_get("quantity").ok().map(|v: serde_json::Value| v.to_string()),
            unit_of_measure: row.get("unit_of_measure"),
            unit_price: row.try_get("unit_price").unwrap_or(serde_json::json!("0")).to_string(),
            line_amount: row.try_get("line_amount").unwrap_or(serde_json::json!("0")).to_string(),
            tax_code: row.get("tax_code"),
            tax_amount: row.try_get("tax_amount").ok().map(|v: serde_json::Value| v.to_string()),
            revenue_account_code: row.get("revenue_account_code"),
            sales_order_number: row.get("sales_order_number"),
            sales_order_line: row.get("sales_order_line"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl AutoInvoiceRepository for PostgresAutoInvoiceRepository {
    // Grouping Rules
    async fn create_grouping_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        transaction_types: serde_json::Value,
        group_by_fields: serde_json::Value,
        line_order_by: serde_json::Value,
        is_default: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceGroupingRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_grouping_rules
                (organization_id, name, description, transaction_types,
                 group_by_fields, line_order_by, is_default, priority, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(org_id).bind(name).bind(description)
        .bind(&transaction_types).bind(&group_by_fields).bind(&line_order_by)
        .bind(is_default).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_grouping_rule(&row))
    }

    async fn get_grouping_rule(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceGroupingRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_grouping_rules WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_grouping_rule(&r)))
    }

    async fn get_grouping_rule_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<AutoInvoiceGroupingRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_grouping_rules WHERE organization_id = $1 AND name = $2 AND is_active = true"
        )
        .bind(org_id).bind(name)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_grouping_rule(&r)))
    }

    async fn get_default_grouping_rule(&self, org_id: Uuid) -> AtlasResult<Option<AutoInvoiceGroupingRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_grouping_rules WHERE organization_id = $1 AND is_default = true AND is_active = true ORDER BY priority LIMIT 1"
        )
        .bind(org_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_grouping_rule(&r)))
    }

    async fn list_grouping_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceGroupingRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_grouping_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_grouping_rule(r)).collect())
    }

    async fn delete_grouping_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.autoinvoice_grouping_rules WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Validation Rules
    async fn create_validation_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        field_name: &str,
        validation_type: &str,
        validation_expression: Option<&str>,
        error_message: &str,
        is_fatal: bool,
        transaction_types: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceValidationRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_validation_rules
                (organization_id, name, description, field_name, validation_type,
                 validation_expression, error_message, is_fatal, transaction_types,
                 priority, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(org_id).bind(name).bind(description)
        .bind(field_name).bind(validation_type).bind(validation_expression)
        .bind(error_message).bind(is_fatal).bind(&transaction_types)
        .bind(priority).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_validation_rule(&row))
    }

    async fn get_validation_rules(&self, org_id: Uuid, transaction_type: Option<&str>) -> AtlasResult<Vec<AutoInvoiceValidationRule>> {
        let rows = if let Some(tt) = transaction_type {
            sqlx::query(
                r#"
                SELECT * FROM _atlas.autoinvoice_validation_rules
                WHERE organization_id = $1 AND is_active = true
                  AND transaction_types ? $2
                ORDER BY priority
                "#
            )
            .bind(org_id).bind(tt)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.autoinvoice_validation_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };

        Ok(rows.iter().map(|r| self.row_to_validation_rule(r)).collect())
    }

    async fn list_validation_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceValidationRule>> {
        self.get_validation_rules(org_id, None).await
    }

    async fn delete_validation_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.autoinvoice_validation_rules WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Batches
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        batch_source: &str,
        description: Option<&str>,
        grouping_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_batches
                (organization_id, batch_number, batch_source, description, grouping_rule_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(org_id).bind(batch_number).bind(batch_source)
        .bind(description).bind(grouping_rule_id).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_batch(&row))
    }

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.autoinvoice_batches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_batch(&r)))
    }

    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<AutoInvoiceBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_batches WHERE organization_id = $1 AND batch_number = $2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_batch(&r)))
    }

    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AutoInvoiceBatch>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.autoinvoice_batches WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.autoinvoice_batches WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };

        Ok(rows.iter().map(|r| self.row_to_batch(r)).collect())
    }

    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<AutoInvoiceBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.autoinvoice_batches
            SET status = $2, updated_at = now(),
                started_at = CASE WHEN $2 = 'validating' THEN now() ELSE started_at END,
                completed_at = CASE WHEN $2 IN ('completed', 'failed') THEN now() ELSE completed_at END
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_batch(&row))
    }

    async fn update_batch_counts(
        &self,
        id: Uuid,
        total_lines: i32,
        valid_lines: i32,
        invalid_lines: i32,
        invoices_created: i32,
        invoices_total_amount: &str,
        validation_errors: serde_json::Value,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.autoinvoice_batches
            SET total_lines = $2, valid_lines = $3, invalid_lines = $4,
                invoices_created = $5, invoices_total_amount = $6,
                validation_errors = $7, updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id).bind(total_lines).bind(valid_lines).bind(invalid_lines)
        .bind(invoices_created)
        .bind(invoices_total_amount.parse::<f64>().unwrap_or(0.0))
        .bind(&validation_errors)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Lines
    async fn create_line(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        line_number: i32,
        source_line_id: Option<&str>,
        transaction_type: &str,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        bill_to_customer_id: Option<Uuid>,
        bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>,
        ship_to_site_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_price: &str,
        line_amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>,
        receivable_account_code: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        sales_rep_id: Option<Uuid>,
        sales_rep_name: Option<&str>,
        memo_line: Option<&str>,
        reference_number: Option<&str>,
        sales_order_number: Option<&str>,
        sales_order_line: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_lines
                (organization_id, batch_id, line_number, source_line_id, transaction_type,
                 customer_id, customer_number, customer_name,
                 bill_to_customer_id, bill_to_site_id,
                 ship_to_customer_id, ship_to_site_id,
                 item_code, item_description, quantity, unit_of_measure,
                 unit_price, line_amount, currency_code, exchange_rate,
                 transaction_date, gl_date, due_date,
                 revenue_account_code, receivable_account_code,
                 tax_code, tax_amount,
                 sales_rep_id, sales_rep_name,
                 memo_line, reference_number,
                 sales_order_number, sales_order_line,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                    $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23,
                    $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34)
            RETURNING *
            "#
        )
        .bind(org_id).bind(batch_id).bind(line_number).bind(source_line_id).bind(transaction_type)
        .bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(bill_to_customer_id).bind(bill_to_site_id)
        .bind(ship_to_customer_id).bind(ship_to_site_id)
        .bind(item_code).bind(item_description)
        .bind(quantity.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(unit_of_measure)
        .bind(unit_price.parse::<f64>().unwrap_or(0.0))
        .bind(line_amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code)
        .bind(exchange_rate.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(transaction_date).bind(gl_date).bind(due_date)
        .bind(revenue_account_code).bind(receivable_account_code)
        .bind(tax_code)
        .bind(tax_amount.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(sales_rep_id).bind(sales_rep_name)
        .bind(memo_line).bind(reference_number)
        .bind(sales_order_number).bind(sales_order_line)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_line(&row))
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.autoinvoice_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_line(&r)))
    }

    async fn list_lines_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_lines WHERE batch_id = $1 ORDER BY line_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_line(r)).collect())
    }

    async fn list_lines_by_status(&self, batch_id: Uuid, status: &str) -> AtlasResult<Vec<AutoInvoiceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_lines WHERE batch_id = $1 AND status = $2 ORDER BY line_number"
        )
        .bind(batch_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_line(r)).collect())
    }

    async fn update_line_status(&self, id: Uuid, status: &str, validation_errors: serde_json::Value) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.autoinvoice_lines SET status = $2, validation_errors = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(status).bind(&validation_errors)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_line_invoice(&self, id: Uuid, invoice_id: Uuid, invoice_line_number: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.autoinvoice_lines SET invoice_id = $2, invoice_line_number = $3, status = 'grouped', updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(invoice_id).bind(invoice_line_number)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Results
    async fn create_result(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        invoice_number: &str,
        transaction_type: &str,
        customer_id: Option<Uuid>,
        bill_to_customer_id: Option<Uuid>,
        bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>,
        ship_to_site_id: Option<Uuid>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        receivable_account_code: Option<&str>,
        sales_rep_id: Option<Uuid>,
        sales_order_number: Option<&str>,
        reference_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceResult> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_results
                (organization_id, batch_id, invoice_number, transaction_type,
                 customer_id, bill_to_customer_id, bill_to_site_id,
                 ship_to_customer_id, ship_to_site_id,
                 currency_code, exchange_rate,
                 transaction_date, gl_date, due_date,
                 receivable_account_code, sales_rep_id,
                 sales_order_number, reference_number, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19)
            RETURNING *
            "#
        )
        .bind(org_id).bind(batch_id).bind(invoice_number).bind(transaction_type)
        .bind(customer_id).bind(bill_to_customer_id).bind(bill_to_site_id)
        .bind(ship_to_customer_id).bind(ship_to_site_id)
        .bind(currency_code)
        .bind(exchange_rate.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(transaction_date).bind(gl_date).bind(due_date)
        .bind(receivable_account_code).bind(sales_rep_id)
        .bind(sales_order_number).bind(reference_number).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_result(&row))
    }

    async fn get_result(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceResult>> {
        let row = sqlx::query("SELECT * FROM _atlas.autoinvoice_results WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_result(&r)))
    }

    async fn get_result_by_invoice_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<AutoInvoiceResult>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_results WHERE organization_id = $1 AND invoice_number = $2"
        )
        .bind(org_id).bind(invoice_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_result(&r)))
    }

    async fn list_results_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResult>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_results WHERE batch_id = $1 ORDER BY invoice_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_result(r)).collect())
    }

    async fn update_result_totals(
        &self,
        id: Uuid,
        subtotal: &str,
        tax_amount: &str,
        total_amount: &str,
        line_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.autoinvoice_results
            SET subtotal = $2, tax_amount = $3, total_amount = $4,
                line_count = $5, updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(subtotal.parse::<f64>().unwrap_or(0.0))
        .bind(tax_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_amount.parse::<f64>().unwrap_or(0.0))
        .bind(line_count)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_result_status(&self, id: Uuid, status: &str) -> AtlasResult<AutoInvoiceResult> {
        let row = sqlx::query(
            "UPDATE _atlas.autoinvoice_results SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_result(&row))
    }

    // Result Lines
    async fn create_result_line(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        line_number: i32,
        source_line_id: Option<&str>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_price: &str,
        line_amount: &str,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        revenue_account_code: Option<&str>,
        sales_order_number: Option<&str>,
        sales_order_line: Option<&str>,
    ) -> AtlasResult<AutoInvoiceResultLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.autoinvoice_result_lines
                (organization_id, invoice_id, line_number, source_line_id,
                 item_code, item_description, quantity, unit_of_measure,
                 unit_price, line_amount, tax_code, tax_amount,
                 revenue_account_code, sales_order_number, sales_order_line)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#
        )
        .bind(org_id).bind(invoice_id).bind(line_number).bind(source_line_id)
        .bind(item_code).bind(item_description)
        .bind(quantity.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(unit_of_measure)
        .bind(unit_price.parse::<f64>().unwrap_or(0.0))
        .bind(line_amount.parse::<f64>().unwrap_or(0.0))
        .bind(tax_code)
        .bind(tax_amount.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(revenue_account_code).bind(sales_order_number).bind(sales_order_line)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_result_line(&row))
    }

    async fn list_result_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResultLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autoinvoice_result_lines WHERE invoice_id = $1 ORDER BY line_number"
        )
        .bind(invoice_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_result_line(r)).collect())
    }

    // Summary
    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::AutoInvoiceSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_batches,
                COUNT(*) FILTER (WHERE status = 'pending') as pending_batches,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_batches,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_batches,
                COALESCE(SUM(total_lines), 0) as total_lines_imported,
                COALESCE(SUM(invoices_created), 0) as total_invoices_created,
                COALESCE(SUM(invoices_total_amount), 0) as total_invoice_amount
            FROM _atlas.autoinvoice_batches
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(atlas_shared::AutoInvoiceSummary {
            total_batches: row.get::<i64, _>("total_batches") as i32,
            pending_batches: row.get::<i64, _>("pending_batches") as i32,
            completed_batches: row.get::<i64, _>("completed_batches") as i32,
            failed_batches: row.get::<i64, _>("failed_batches") as i32,
            total_lines_imported: row.get::<i64, _>("total_lines_imported") as i32,
            total_invoices_created: row.get::<i64, _>("total_invoices_created") as i32,
            total_invoice_amount: row.try_get("total_invoice_amount").unwrap_or(serde_json::json!("0")).to_string(),
        })
    }
}
