//! Recurring Invoice Repository
//!
//! Database access layer for recurring invoice templates, lines,
//! and generation history.

use atlas_shared::AtlasResult;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Recurring invoice template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecurringInvoiceTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_number: String,
    pub template_name: String,
    pub description: Option<String>,

    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,

    pub invoice_type: String,
    pub invoice_currency_code: String,
    pub payment_currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,

    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_due_days: i32,
    pub liability_account_code: Option<String>,
    pub expense_account_code: Option<String>,

    pub amount_type: String,

    pub recurrence_type: String,
    pub recurrence_interval: i32,
    pub generation_day: Option<i32>,
    pub days_in_advance: i32,

    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub maximum_generations: Option<i32>,

    pub auto_submit: bool,
    pub auto_approve: bool,
    pub hold_for_review: bool,

    pub po_number: Option<String>,

    pub gl_date_basis: String,

    pub status: String,
    pub last_generation_date: Option<chrono::NaiveDate>,
    pub next_generation_date: Option<chrono::NaiveDate>,
    pub generation_count: i32,
    pub total_generated_amount: f64,

    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Recurring invoice template line
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecurringInvoiceTemplateLine {
    pub id: Uuid,
    pub template_id: Uuid,
    pub organization_id: Uuid,
    pub line_number: i32,

    pub line_type: String,
    pub description: Option<String>,
    pub item_code: Option<String>,
    pub unit_of_measure: Option<String>,

    pub amount: f64,
    pub quantity: f64,
    pub unit_price: Option<f64>,

    pub gl_account_code: String,
    pub cost_center: Option<String>,
    pub department: Option<String>,

    pub tax_code: Option<String>,
    pub tax_amount: f64,

    pub project_id: Option<Uuid>,
    pub expenditure_type: Option<String>,

    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

/// Recurring invoice generation record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecurringInvoiceGeneration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub generation_number: i32,

    pub generated_invoice_id: Option<Uuid>,
    pub generated_invoice_number: Option<String>,
    pub invoice_date: chrono::NaiveDate,
    pub invoice_due_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,

    pub invoice_amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,

    pub generation_status: String,
    pub error_message: Option<String>,

    pub period_name: Option<String>,
    pub fiscal_year: Option<i32>,
    pub period_number: Option<i32>,

    pub generated_at: Option<chrono::DateTime<Utc>>,
    pub generated_by: Option<Uuid>,
}

/// Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringInvoiceDashboard {
    pub total_templates: i64,
    pub active_templates: i64,
    pub suspended_templates: i64,
    pub total_generations: i64,
    pub total_generated_amount: f64,
    pub upcoming_this_month: i64,
    pub upcoming_next_30_days: Vec<UpcomingInvoice>,
    pub by_recurrence_type: serde_json::Value,
    pub by_supplier: serde_json::Value,
}

/// Upcoming invoice in dashboard
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingInvoice {
    pub template_id: Uuid,
    pub template_number: String,
    pub template_name: String,
    pub supplier_name: Option<String>,
    pub next_generation_date: chrono::NaiveDate,
    pub estimated_amount: f64,
    pub status: String,
}

/// Parameters for creating a template
pub struct TemplateCreateParams {
    pub template_number: String,
    pub template_name: String,
    pub description: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub invoice_type: String,
    pub invoice_currency_code: String,
    pub payment_currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_due_days: i32,
    pub liability_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub amount_type: String,
    pub recurrence_type: String,
    pub recurrence_interval: i32,
    pub generation_day: Option<i32>,
    pub days_in_advance: i32,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub maximum_generations: Option<i32>,
    pub auto_submit: bool,
    pub auto_approve: bool,
    pub hold_for_review: bool,
    pub po_number: Option<String>,
    pub gl_date_basis: String,
}

/// Parameters for creating a template line
pub struct TemplateLineCreateParams {
    pub line_type: String,
    pub description: Option<String>,
    pub item_code: Option<String>,
    pub unit_of_measure: Option<String>,
    pub amount: f64,
    pub quantity: f64,
    pub unit_price: Option<f64>,
    pub gl_account_code: String,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: f64,
    pub project_id: Option<Uuid>,
    pub expenditure_type: Option<String>,
}

// ============================================================================
// Repository Trait
// ============================================================================

#[async_trait]
pub trait RecurringInvoiceRepository: Send + Sync {
    async fn create_template(
        &self,
        org_id: Uuid,
        params: &TemplateCreateParams,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceTemplate>;

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<RecurringInvoiceTemplate>>;

    async fn get_template_by_number(
        &self,
        org_id: Uuid,
        template_number: &str,
    ) -> AtlasResult<Option<RecurringInvoiceTemplate>>;

    async fn list_templates(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplate>>;

    async fn update_template_status(
        &self,
        id: Uuid,
        status: &str,
        next_generation_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<RecurringInvoiceTemplate>;

    async fn update_template_generation(
        &self,
        id: Uuid,
        generation_date: chrono::NaiveDate,
        next_generation_date: Option<chrono::NaiveDate>,
        total_amount: f64,
    ) -> AtlasResult<RecurringInvoiceTemplate>;

    async fn delete_template(&self, org_id: Uuid, template_number: &str) -> AtlasResult<()>;

    async fn create_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        line_number: i32,
        params: &TemplateLineCreateParams,
    ) -> AtlasResult<RecurringInvoiceTemplateLine>;

    async fn list_template_lines(
        &self,
        template_id: Uuid,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplateLine>>;

    async fn remove_template_line(&self, template_id: Uuid, line_id: Uuid) -> AtlasResult<()>;

    async fn create_generation(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        generation_number: i32,
        invoice_date: chrono::NaiveDate,
        invoice_due_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        invoice_amount: f64,
        tax_amount: f64,
        total_amount: f64,
        period_name: Option<&str>,
        fiscal_year: Option<i32>,
        period_number: Option<i32>,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceGeneration>;

    async fn list_generations(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        generation_status: Option<&str>,
    ) -> AtlasResult<Vec<RecurringInvoiceGeneration>>;

    async fn update_generation_status(
        &self,
        id: Uuid,
        generation_status: &str,
        error_message: Option<&str>,
    ) -> AtlasResult<RecurringInvoiceGeneration>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RecurringInvoiceDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

use sqlx::PgPool;

pub struct PostgresRecurringInvoiceRepository {
    pool: PgPool,
}

impl PostgresRecurringInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecurringInvoiceRepository for PostgresRecurringInvoiceRepository {
    async fn create_template(
        &self,
        org_id: Uuid,
        params: &TemplateCreateParams,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceTemplate> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            r#"
            INSERT INTO _atlas.recurring_invoice_templates (
                organization_id, template_number, template_name, description,
                supplier_id, supplier_number, supplier_name, supplier_site,
                invoice_type, invoice_currency_code, payment_currency_code, exchange_rate_type,
                payment_terms, payment_method, payment_due_days,
                liability_account_code, expense_account_code,
                amount_type,
                recurrence_type, recurrence_interval, generation_day, days_in_advance,
                effective_from, effective_to, maximum_generations,
                auto_submit, auto_approve, hold_for_review,
                po_number, gl_date_basis,
                status, created_by
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, $11, $12,
                $13, $14, $15,
                $16, $17,
                $18,
                $19, $20, $21, $22,
                $23, $24, $25,
                $26, $27, $28,
                $29, $30,
                'draft', $31
            ) RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(&params.template_number)
        .bind(&params.template_name)
        .bind(&params.description)
        .bind(params.supplier_id)
        .bind(&params.supplier_number)
        .bind(&params.supplier_name)
        .bind(&params.supplier_site)
        .bind(&params.invoice_type)
        .bind(&params.invoice_currency_code)
        .bind(&params.payment_currency_code)
        .bind(&params.exchange_rate_type)
        .bind(&params.payment_terms)
        .bind(&params.payment_method)
        .bind(params.payment_due_days)
        .bind(&params.liability_account_code)
        .bind(&params.expense_account_code)
        .bind(&params.amount_type)
        .bind(&params.recurrence_type)
        .bind(params.recurrence_interval)
        .bind(params.generation_day)
        .bind(params.days_in_advance)
        .bind(params.effective_from)
        .bind(params.effective_to)
        .bind(params.maximum_generations)
        .bind(params.auto_submit)
        .bind(params.auto_approve)
        .bind(params.hold_for_review)
        .bind(&params.po_number)
        .bind(&params.gl_date_basis)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<RecurringInvoiceTemplate>> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            "SELECT * FROM _atlas.recurring_invoice_templates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn get_template_by_number(
        &self,
        org_id: Uuid,
        template_number: &str,
    ) -> AtlasResult<Option<RecurringInvoiceTemplate>> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            "SELECT * FROM _atlas.recurring_invoice_templates WHERE organization_id = $1 AND template_number = $2"
        )
        .bind(org_id)
        .bind(template_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn list_templates(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplate>> {
        let rows = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            r#"SELECT * FROM _atlas.recurring_invoice_templates
            WHERE organization_id = $1
                AND ($2::text IS NULL OR status = $2)
                AND ($3::uuid IS NULL OR supplier_id = $3)
            ORDER BY template_number"#
        )
        .bind(org_id)
        .bind(status)
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    async fn update_template_status(
        &self,
        id: Uuid,
        status: &str,
        next_generation_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<RecurringInvoiceTemplate> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            r#"UPDATE _atlas.recurring_invoice_templates
            SET status = $2, next_generation_date = $3, updated_at = now()
            WHERE id = $1
            RETURNING *"#
        )
        .bind(id)
        .bind(status)
        .bind(next_generation_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn update_template_generation(
        &self,
        id: Uuid,
        generation_date: chrono::NaiveDate,
        next_generation_date: Option<chrono::NaiveDate>,
        total_amount: f64,
    ) -> AtlasResult<RecurringInvoiceTemplate> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplate>(
            r#"UPDATE _atlas.recurring_invoice_templates
            SET last_generation_date = $2,
                next_generation_date = $3,
                generation_count = generation_count + 1,
                total_generated_amount = total_generated_amount + $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *"#
        )
        .bind(id)
        .bind(generation_date)
        .bind(next_generation_date)
        .bind(total_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn delete_template(&self, org_id: Uuid, template_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.recurring_invoice_templates WHERE organization_id = $1 AND template_number = $2 AND status = 'draft'"
        )
        .bind(org_id)
        .bind(template_number)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(atlas_shared::AtlasError::EntityNotFound(
                "Template not found or not in draft status".to_string(),
            ));
        }

        Ok(())
    }

    // ========================================================================
    // Template Lines
    // ========================================================================

    async fn create_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        line_number: i32,
        params: &TemplateLineCreateParams,
    ) -> AtlasResult<RecurringInvoiceTemplateLine> {
        let row = sqlx::query_as::<_, RecurringInvoiceTemplateLine>(
            r#"
            INSERT INTO _atlas.recurring_invoice_template_lines (
                template_id, organization_id, line_number,
                line_type, description, item_code, unit_of_measure,
                amount, quantity, unit_price,
                gl_account_code, cost_center, department,
                tax_code, tax_amount,
                project_id, expenditure_type
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7,
                $8, $9, $10,
                $11, $12, $13,
                $14, $15,
                $16, $17
            ) RETURNING *
            "#,
        )
        .bind(template_id)
        .bind(org_id)
        .bind(line_number)
        .bind(&params.line_type)
        .bind(&params.description)
        .bind(&params.item_code)
        .bind(&params.unit_of_measure)
        .bind(params.amount)
        .bind(params.quantity)
        .bind(params.unit_price)
        .bind(&params.gl_account_code)
        .bind(&params.cost_center)
        .bind(&params.department)
        .bind(&params.tax_code)
        .bind(params.tax_amount)
        .bind(params.project_id)
        .bind(&params.expenditure_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn list_template_lines(
        &self,
        template_id: Uuid,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplateLine>> {
        let rows = sqlx::query_as::<_, RecurringInvoiceTemplateLine>(
            "SELECT * FROM _atlas.recurring_invoice_template_lines WHERE template_id = $1 ORDER BY line_number"
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    async fn remove_template_line(&self, template_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.recurring_invoice_template_lines WHERE template_id = $1 AND id = $2"
        )
        .bind(template_id)
        .bind(line_id)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Generations
    // ========================================================================

    async fn create_generation(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        generation_number: i32,
        invoice_date: chrono::NaiveDate,
        invoice_due_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        invoice_amount: f64,
        tax_amount: f64,
        total_amount: f64,
        period_name: Option<&str>,
        fiscal_year: Option<i32>,
        period_number: Option<i32>,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceGeneration> {
        let row = sqlx::query_as::<_, RecurringInvoiceGeneration>(
            r#"
            INSERT INTO _atlas.recurring_invoice_generations (
                organization_id, template_id, generation_number,
                invoice_date, invoice_due_date, gl_date,
                invoice_amount, tax_amount, total_amount,
                period_name, fiscal_year, period_number,
                generated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(template_id)
        .bind(generation_number)
        .bind(invoice_date)
        .bind(invoice_due_date)
        .bind(gl_date)
        .bind(invoice_amount)
        .bind(tax_amount)
        .bind(total_amount)
        .bind(period_name)
        .bind(fiscal_year)
        .bind(period_number)
        .bind(generated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn list_generations(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        generation_status: Option<&str>,
    ) -> AtlasResult<Vec<RecurringInvoiceGeneration>> {
        let rows = sqlx::query_as::<_, RecurringInvoiceGeneration>(
            r#"SELECT * FROM _atlas.recurring_invoice_generations
            WHERE organization_id = $1
                AND ($2::uuid IS NULL OR template_id = $2)
                AND ($3::text IS NULL OR generation_status = $3)
            ORDER BY invoice_date DESC, generation_number DESC"#
        )
        .bind(org_id)
        .bind(template_id)
        .bind(generation_status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    async fn update_generation_status(
        &self,
        id: Uuid,
        generation_status: &str,
        error_message: Option<&str>,
    ) -> AtlasResult<RecurringInvoiceGeneration> {
        let row = sqlx::query_as::<_, RecurringInvoiceGeneration>(
            r#"UPDATE _atlas.recurring_invoice_generations
            SET generation_status = $2, error_message = $3
            WHERE id = $1
            RETURNING *"#
        )
        .bind(id)
        .bind(generation_status)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RecurringInvoiceDashboard> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            total: Option<i64>,
            active: Option<i64>,
            suspended: Option<i64>,
            total_gen_amount: Option<f64>,
        }

        let stats = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE status = 'suspended') as suspended,
                COALESCE(SUM(total_generated_amount), 0) as total_gen_amount
            FROM _atlas.recurring_invoice_templates
            WHERE organization_id = $1"#
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        #[derive(sqlx::FromRow)]
        struct GenCountRow { gen_count: Option<i64> }

        let gen_stats = sqlx::query_as::<_, GenCountRow>(
            "SELECT COUNT(*) as gen_count FROM _atlas.recurring_invoice_generations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let upcoming = sqlx::query_as::<_, UpcomingInvoice>(
            r#"SELECT
                t.id as template_id,
                t.template_number,
                t.template_name,
                t.supplier_name,
                t.next_generation_date,
                COALESCE(
                    (SELECT SUM(l.amount) FROM _atlas.recurring_invoice_template_lines l WHERE l.template_id = t.id),
                    0
                ) as estimated_amount,
                t.status
            FROM _atlas.recurring_invoice_templates t
            WHERE t.organization_id = $1
                AND t.status = 'active'
                AND t.next_generation_date IS NOT NULL
                AND t.next_generation_date <= CURRENT_DATE + INTERVAL '30 days'
            ORDER BY t.next_generation_date
            LIMIT 10"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let upcoming_count = upcoming.len() as i64;

        #[derive(sqlx::FromRow)]
        struct RecurrenceRow { recurrence_type: String, count: Option<i64> }

        let by_recurrence = sqlx::query_as::<_, RecurrenceRow>(
            r#"SELECT recurrence_type, COUNT(*) as count
            FROM _atlas.recurring_invoice_templates
            WHERE organization_id = $1
            GROUP BY recurrence_type"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let by_recurrence_type = serde_json::Value::Object(
            by_recurrence.into_iter().map(|r| {
                (r.recurrence_type, serde_json::Value::Number(r.count.unwrap_or(0).into()))
            }).collect()
        );

        #[derive(sqlx::FromRow)]
        struct SupplierRow { name: Option<String>, count: Option<i64> }

        let by_supplier_rows = sqlx::query_as::<_, SupplierRow>(
            r#"SELECT COALESCE(supplier_name, 'Unknown') as name, COUNT(*) as count
            FROM _atlas.recurring_invoice_templates
            WHERE organization_id = $1
            GROUP BY supplier_name
            ORDER BY count DESC"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let by_supplier = serde_json::Value::Object(
            by_supplier_rows.into_iter().map(|r| {
                (r.name.unwrap_or_else(|| "Unknown".to_string()), serde_json::Value::Number(r.count.unwrap_or(0).into()))
            }).collect()
        );

        Ok(RecurringInvoiceDashboard {
            total_templates: stats.total.unwrap_or(0),
            active_templates: stats.active.unwrap_or(0),
            suspended_templates: stats.suspended.unwrap_or(0),
            total_generations: gen_stats.gen_count.unwrap_or(0),
            total_generated_amount: stats.total_gen_amount.unwrap_or(0.0),
            upcoming_this_month: upcoming_count,
            upcoming_next_30_days: upcoming,
            by_recurrence_type,
            by_supplier,
        })
    }
}
