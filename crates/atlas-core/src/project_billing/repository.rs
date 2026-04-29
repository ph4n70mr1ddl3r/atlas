//! Project Billing Repository
//!
//! PostgreSQL storage for bill rate schedules, billing configs,
//! billing events, project invoices, and billing dashboard.

use atlas_shared::{
    BillRateSchedule, BillRateLine, ProjectBillingConfig,
    BillingEvent, ProjectInvoiceHeader, ProjectInvoiceLine,
    ProjectBillingDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for project billing data storage
#[async_trait]
pub trait ProjectBillingRepository: Send + Sync {
    // ------------------------------------------------------------------
    // Bill Rate Schedules
    // ------------------------------------------------------------------
    async fn create_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str,
        description: Option<&str>, schedule_type: &str, currency_code: &str,
        effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
        default_markup_pct: f64, created_by: Option<Uuid>,
    ) -> AtlasResult<BillRateSchedule>;
    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<BillRateSchedule>>;
    async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<BillRateSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BillRateSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<BillRateSchedule>;
    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()>;

    // ------------------------------------------------------------------
    // Bill Rate Lines
    // ------------------------------------------------------------------
    async fn create_rate_line(
        &self, org_id: Uuid, schedule_id: Uuid, role_name: &str,
        project_id: Option<Uuid>, bill_rate: f64, unit_of_measure: &str,
        effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
        markup_pct: Option<f64>,
    ) -> AtlasResult<BillRateLine>;
    async fn list_rate_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BillRateLine>>;
    async fn find_rate_for_role(
        &self, schedule_id: Uuid, role_name: &str, date: chrono::NaiveDate,
    ) -> AtlasResult<Option<BillRateLine>>;
    async fn delete_rate_line(&self, id: Uuid) -> AtlasResult<()>;

    // ------------------------------------------------------------------
    // Project Billing Config
    // ------------------------------------------------------------------
    async fn create_billing_config(
        &self, org_id: Uuid, project_id: Uuid, billing_method: &str,
        bill_rate_schedule_id: Option<Uuid>, contract_amount: f64,
        currency_code: &str, invoice_format: &str, billing_cycle: &str,
        payment_terms_days: i32, retention_pct: f64, retention_amount_cap: f64,
        customer_id: Option<Uuid>, customer_name: Option<&str>,
        customer_po_number: Option<&str>, contract_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectBillingConfig>;
    async fn get_billing_config(&self, id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>>;
    async fn get_billing_config_by_project(&self, org_id: Uuid, project_id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>>;
    async fn update_billing_config_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectBillingConfig>;
    async fn list_billing_configs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectBillingConfig>>;

    // ------------------------------------------------------------------
    // Billing Events
    // ------------------------------------------------------------------
    async fn create_billing_event(
        &self, org_id: Uuid, project_id: Uuid, event_number: &str,
        event_name: &str, description: Option<&str>, event_type: &str,
        billing_amount: f64, currency_code: &str, completion_pct: f64,
        planned_date: Option<chrono::NaiveDate>,
        task_id: Option<Uuid>, task_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BillingEvent>;
    async fn get_billing_event(&self, id: Uuid) -> AtlasResult<Option<BillingEvent>>;
    async fn get_billing_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<BillingEvent>>;
    async fn list_billing_events(
        &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<BillingEvent>>;
    async fn update_billing_event_status(&self, id: Uuid, status: &str) -> AtlasResult<BillingEvent>;
    async fn complete_billing_event(
        &self, id: Uuid, actual_date: chrono::NaiveDate, completion_pct: f64,
    ) -> AtlasResult<BillingEvent>;
    async fn delete_billing_event(&self, org_id: Uuid, event_number: &str) -> AtlasResult<()>;

    // ------------------------------------------------------------------
    // Project Invoices
    // ------------------------------------------------------------------
    async fn create_invoice(
        &self, org_id: Uuid, invoice_number: &str, project_id: Uuid,
        project_number: Option<&str>, project_name: Option<&str>,
        invoice_type: &str, customer_id: Option<Uuid>, customer_name: Option<&str>,
        invoice_amount: f64, tax_amount: f64, retention_held: f64,
        total_amount: f64, currency_code: &str,
        billing_period_start: Option<chrono::NaiveDate>,
        billing_period_end: Option<chrono::NaiveDate>,
        invoice_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        billing_event_id: Option<Uuid>,
        customer_po_number: Option<&str>, contract_number: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectInvoiceHeader>;
    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ProjectInvoiceHeader>>;
    async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ProjectInvoiceHeader>>;
    async fn list_invoices(
        &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<ProjectInvoiceHeader>>;
    async fn update_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectInvoiceHeader>;
    async fn reject_invoice(&self, id: Uuid, reason: &str) -> AtlasResult<ProjectInvoiceHeader>;
    async fn mark_invoice_posted(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader>;
    async fn delete_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<()>;

    // ------------------------------------------------------------------
    // Invoice Lines
    // ------------------------------------------------------------------
    async fn create_invoice_line(
        &self, org_id: Uuid, invoice_header_id: Uuid, line_number: i32,
        line_source: &str, expenditure_item_id: Option<Uuid>,
        billing_event_id: Option<Uuid>,
        task_id: Option<Uuid>, task_number: Option<&str>, task_name: Option<&str>,
        description: Option<&str>,
        employee_id: Option<Uuid>, employee_name: Option<&str>,
        role_name: Option<&str>, expenditure_type: Option<&str>,
        quantity: f64, unit_of_measure: &str, bill_rate: f64,
        raw_cost_amount: f64, bill_amount: f64, markup_amount: f64,
        retention_amount: f64, tax_amount: f64,
        transaction_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ProjectInvoiceLine>;
    async fn list_invoice_lines(&self, invoice_header_id: Uuid) -> AtlasResult<Vec<ProjectInvoiceLine>>;

    // ------------------------------------------------------------------
    // Dashboard
    // ------------------------------------------------------------------
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProjectBillingDashboard>;
}

// ============================================================================
// Row mappers
// ============================================================================

fn row_to_schedule(row: &sqlx::postgres::PgRow) -> BillRateSchedule {
    BillRateSchedule {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        schedule_number: row.try_get("schedule_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        schedule_type: row.try_get("schedule_type").unwrap_or_default(),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        effective_start: row.try_get("effective_start").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        effective_end: row.try_get("effective_end").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        default_markup_pct: row.try_get("default_markup_pct").unwrap_or(0.0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

fn row_to_rate_line(row: &sqlx::postgres::PgRow) -> BillRateLine {
    BillRateLine {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        schedule_id: row.try_get("schedule_id").unwrap_or_default(),
        role_name: row.try_get("role_name").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        bill_rate: row.try_get("bill_rate").unwrap_or(0.0),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        effective_start: row.try_get("effective_start").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        effective_end: row.try_get("effective_end").unwrap_or_default(),
        markup_pct: row.try_get("markup_pct").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

fn row_to_billing_config(row: &sqlx::postgres::PgRow) -> ProjectBillingConfig {
    ProjectBillingConfig {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        billing_method: row.try_get("billing_method").unwrap_or_default(),
        bill_rate_schedule_id: row.try_get("bill_rate_schedule_id").unwrap_or_default(),
        contract_amount: row.try_get("contract_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        invoice_format: row.try_get("invoice_format").unwrap_or_default(),
        billing_cycle: row.try_get("billing_cycle").unwrap_or_default(),
        payment_terms_days: row.try_get("payment_terms_days").unwrap_or(30),
        retention_pct: row.try_get("retention_pct").unwrap_or(0.0),
        retention_amount_cap: row.try_get("retention_amount_cap").unwrap_or(0.0),
        customer_id: row.try_get("customer_id").unwrap_or_default(),
        customer_name: row.try_get("customer_name").unwrap_or_default(),
        customer_po_number: row.try_get("customer_po_number").unwrap_or_default(),
        contract_number: row.try_get("contract_number").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

fn row_to_billing_event(row: &sqlx::postgres::PgRow) -> BillingEvent {
    BillingEvent {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        event_number: row.try_get("event_number").unwrap_or_default(),
        event_name: row.try_get("event_name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        event_type: row.try_get("event_type").unwrap_or_default(),
        billing_amount: row.try_get("billing_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        completion_pct: row.try_get("completion_pct").unwrap_or(0.0),
        status: row.try_get("status").unwrap_or_default(),
        planned_date: row.try_get("planned_date").unwrap_or_default(),
        actual_date: row.try_get("actual_date").unwrap_or_default(),
        task_id: row.try_get("task_id").unwrap_or_default(),
        task_name: row.try_get("task_name").unwrap_or_default(),
        invoice_header_id: row.try_get("invoice_header_id").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

fn row_to_invoice(row: &sqlx::postgres::PgRow) -> ProjectInvoiceHeader {
    ProjectInvoiceHeader {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        invoice_number: row.try_get("invoice_number").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        project_number: row.try_get("project_number").unwrap_or_default(),
        project_name: row.try_get("project_name").unwrap_or_default(),
        invoice_type: row.try_get("invoice_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        customer_id: row.try_get("customer_id").unwrap_or_default(),
        customer_name: row.try_get("customer_name").unwrap_or_default(),
        invoice_amount: row.try_get("invoice_amount").unwrap_or(0.0),
        tax_amount: row.try_get("tax_amount").unwrap_or(0.0),
        retention_held: row.try_get("retention_held").unwrap_or(0.0),
        total_amount: row.try_get("total_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        exchange_rate: row.try_get("exchange_rate").unwrap_or(1.0),
        billing_period_start: row.try_get("billing_period_start").unwrap_or_default(),
        billing_period_end: row.try_get("billing_period_end").unwrap_or_default(),
        invoice_date: row.try_get("invoice_date").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        due_date: row.try_get("due_date").unwrap_or_default(),
        billing_event_id: row.try_get("billing_event_id").unwrap_or_default(),
        customer_po_number: row.try_get("customer_po_number").unwrap_or_default(),
        contract_number: row.try_get("contract_number").unwrap_or_default(),
        gl_posted_flag: row.try_get("gl_posted_flag").unwrap_or(false),
        gl_posted_date: row.try_get("gl_posted_date").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        rejected_reason: row.try_get("rejected_reason").unwrap_or_default(),
        payment_status: row.try_get("payment_status").unwrap_or_default(),
        payment_date: row.try_get("payment_date").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

fn row_to_invoice_line(row: &sqlx::postgres::PgRow) -> ProjectInvoiceLine {
    ProjectInvoiceLine {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        invoice_header_id: row.try_get("invoice_header_id").unwrap_or_default(),
        line_number: row.try_get("line_number").unwrap_or(0),
        line_source: row.try_get("line_source").unwrap_or_default(),
        expenditure_item_id: row.try_get("expenditure_item_id").unwrap_or_default(),
        billing_event_id: row.try_get("billing_event_id").unwrap_or_default(),
        task_id: row.try_get("task_id").unwrap_or_default(),
        task_number: row.try_get("task_number").unwrap_or_default(),
        task_name: row.try_get("task_name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        employee_id: row.try_get("employee_id").unwrap_or_default(),
        employee_name: row.try_get("employee_name").unwrap_or_default(),
        role_name: row.try_get("role_name").unwrap_or_default(),
        expenditure_type: row.try_get("expenditure_type").unwrap_or_default(),
        quantity: row.try_get("quantity").unwrap_or(0.0),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        bill_rate: row.try_get("bill_rate").unwrap_or(0.0),
        raw_cost_amount: row.try_get("raw_cost_amount").unwrap_or(0.0),
        bill_amount: row.try_get("bill_amount").unwrap_or(0.0),
        markup_amount: row.try_get("markup_amount").unwrap_or(0.0),
        retention_amount: row.try_get("retention_amount").unwrap_or(0.0),
        tax_amount: row.try_get("tax_amount").unwrap_or(0.0),
        transaction_date: row.try_get("transaction_date").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresProjectBillingRepository {
    pool: PgPool,
}

impl PostgresProjectBillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectBillingRepository for PostgresProjectBillingRepository {
    // ========================================================================
    // Bill Rate Schedules
    // ========================================================================

    async fn create_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str,
        description: Option<&str>, schedule_type: &str, currency_code: &str,
        effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
        default_markup_pct: f64, created_by: Option<Uuid>,
    ) -> AtlasResult<BillRateSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.bill_rate_schedules
                (organization_id, schedule_number, name, description,
                 schedule_type, currency_code, effective_start, effective_end,
                 default_markup_pct, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '{}'::jsonb, $10)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_number).bind(name).bind(description)
        .bind(schedule_type).bind(currency_code)
        .bind(effective_start).bind(effective_end)
        .bind(default_markup_pct).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_schedule(&row))
    }

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<BillRateSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.bill_rate_schedules WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_schedule))
    }

    async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<BillRateSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.bill_rate_schedules WHERE organization_id = $1 AND schedule_number = $2"
        ).bind(org_id).bind(schedule_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_schedule))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BillRateSchedule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.bill_rate_schedules
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY created_at DESC"#,
        ).bind(org_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_schedule).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<BillRateSchedule> {
        let row = sqlx::query(
            "UPDATE _atlas.bill_rate_schedules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        Ok(row_to_schedule(&row))
    }

    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.bill_rate_schedules WHERE organization_id = $1 AND schedule_number = $2"
        ).bind(org_id).bind(schedule_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Schedule '{}' not found", schedule_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Bill Rate Lines
    // ========================================================================

    async fn create_rate_line(
        &self, org_id: Uuid, schedule_id: Uuid, role_name: &str,
        project_id: Option<Uuid>, bill_rate: f64, unit_of_measure: &str,
        effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
        markup_pct: Option<f64>,
    ) -> AtlasResult<BillRateLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.bill_rate_lines
                (organization_id, schedule_id, role_name, project_id,
                 bill_rate, unit_of_measure, effective_start, effective_end,
                 markup_pct, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(role_name).bind(project_id)
        .bind(bill_rate).bind(unit_of_measure)
        .bind(effective_start).bind(effective_end)
        .bind(markup_pct)
        .fetch_one(&self.pool).await?;
        Ok(row_to_rate_line(&row))
    }

    async fn list_rate_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BillRateLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.bill_rate_lines WHERE schedule_id = $1 ORDER BY role_name, effective_start"
        ).bind(schedule_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_rate_line).collect())
    }

    async fn find_rate_for_role(
        &self, schedule_id: Uuid, role_name: &str, date: chrono::NaiveDate,
    ) -> AtlasResult<Option<BillRateLine>> {
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.bill_rate_lines
               WHERE schedule_id = $1 AND role_name = $2
                 AND effective_start <= $3
                 AND (effective_end IS NULL OR effective_end >= $3)
               ORDER BY project_id NULLS LAST, effective_start DESC
               LIMIT 1"#,
        ).bind(schedule_id).bind(role_name).bind(date)
        .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_rate_line))
    }

    async fn delete_rate_line(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.bill_rate_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Rate line not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Project Billing Config
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_billing_config(
        &self, org_id: Uuid, project_id: Uuid, billing_method: &str,
        bill_rate_schedule_id: Option<Uuid>, contract_amount: f64,
        currency_code: &str, invoice_format: &str, billing_cycle: &str,
        payment_terms_days: i32, retention_pct: f64, retention_amount_cap: f64,
        customer_id: Option<Uuid>, customer_name: Option<&str>,
        customer_po_number: Option<&str>, contract_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectBillingConfig> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.project_billing_configs
                (organization_id, project_id, billing_method,
                 bill_rate_schedule_id, contract_amount,
                 currency_code, invoice_format, billing_cycle,
                 payment_terms_days, retention_pct, retention_amount_cap,
                 customer_id, customer_name, customer_po_number, contract_number,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, '{}'::jsonb, $16)
            RETURNING *"#,
        )
        .bind(org_id).bind(project_id).bind(billing_method)
        .bind(bill_rate_schedule_id).bind(contract_amount)
        .bind(currency_code).bind(invoice_format).bind(billing_cycle)
        .bind(payment_terms_days).bind(retention_pct).bind(retention_amount_cap)
        .bind(customer_id).bind(customer_name).bind(customer_po_number).bind(contract_number)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_billing_config(&row))
    }

    async fn get_billing_config(&self, id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
        let row = sqlx::query("SELECT * FROM _atlas.project_billing_configs WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_billing_config))
    }

    async fn get_billing_config_by_project(&self, org_id: Uuid, project_id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.project_billing_configs WHERE organization_id = $1 AND project_id = $2"
        ).bind(org_id).bind(project_id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_billing_config))
    }

    async fn update_billing_config_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectBillingConfig> {
        let row = sqlx::query(
            "UPDATE _atlas.project_billing_configs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Billing config {} not found", id)))?;
        Ok(row_to_billing_config(&row))
    }

    async fn list_billing_configs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectBillingConfig>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.project_billing_configs
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY created_at DESC"#,
        ).bind(org_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_billing_config).collect())
    }

    // ========================================================================
    // Billing Events
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_billing_event(
        &self, org_id: Uuid, project_id: Uuid, event_number: &str,
        event_name: &str, description: Option<&str>, event_type: &str,
        billing_amount: f64, currency_code: &str, completion_pct: f64,
        planned_date: Option<chrono::NaiveDate>,
        task_id: Option<Uuid>, task_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BillingEvent> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.billing_events
                (organization_id, project_id, event_number, event_name, description,
                 event_type, billing_amount, currency_code, completion_pct,
                 planned_date, task_id, task_name, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}'::jsonb, $13)
            RETURNING *"#,
        )
        .bind(org_id).bind(project_id).bind(event_number).bind(event_name).bind(description)
        .bind(event_type).bind(billing_amount).bind(currency_code).bind(completion_pct)
        .bind(planned_date).bind(task_id).bind(task_name).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_billing_event(&row))
    }

    async fn get_billing_event(&self, id: Uuid) -> AtlasResult<Option<BillingEvent>> {
        let row = sqlx::query("SELECT * FROM _atlas.billing_events WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_billing_event))
    }

    async fn get_billing_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<BillingEvent>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.billing_events WHERE organization_id = $1 AND event_number = $2"
        ).bind(org_id).bind(event_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_billing_event))
    }

    async fn list_billing_events(
        &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<BillingEvent>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.billing_events
               WHERE organization_id = $1
                 AND ($2::uuid IS NULL OR project_id = $2)
                 AND ($3::text IS NULL OR status = $3)
               ORDER BY planned_date NULLS LAST, created_at DESC"#,
        ).bind(org_id).bind(project_id).bind(status)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_billing_event).collect())
    }

    async fn update_billing_event_status(&self, id: Uuid, status: &str) -> AtlasResult<BillingEvent> {
        let row = sqlx::query(
            "UPDATE _atlas.billing_events SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Billing event {} not found", id)))?;
        Ok(row_to_billing_event(&row))
    }

    async fn complete_billing_event(
        &self, id: Uuid, actual_date: chrono::NaiveDate, completion_pct: f64,
    ) -> AtlasResult<BillingEvent> {
        let row = sqlx::query(
            r#"UPDATE _atlas.billing_events
               SET actual_date = $2, completion_pct = $3, status = 'ready', updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(actual_date).bind(completion_pct)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Billing event {} not found", id)))?;
        Ok(row_to_billing_event(&row))
    }

    async fn delete_billing_event(&self, org_id: Uuid, event_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.billing_events WHERE organization_id = $1 AND event_number = $2"
        ).bind(org_id).bind(event_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Billing event '{}' not found", event_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Project Invoices
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_invoice(
        &self, org_id: Uuid, invoice_number: &str, project_id: Uuid,
        project_number: Option<&str>, project_name: Option<&str>,
        invoice_type: &str, customer_id: Option<Uuid>, customer_name: Option<&str>,
        invoice_amount: f64, tax_amount: f64, retention_held: f64,
        total_amount: f64, currency_code: &str,
        billing_period_start: Option<chrono::NaiveDate>,
        billing_period_end: Option<chrono::NaiveDate>,
        invoice_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        billing_event_id: Option<Uuid>,
        customer_po_number: Option<&str>, contract_number: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectInvoiceHeader> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.project_invoice_headers
                (organization_id, invoice_number, project_id, project_number, project_name,
                 invoice_type, customer_id, customer_name,
                 invoice_amount, tax_amount, retention_held, total_amount, currency_code,
                 billing_period_start, billing_period_end,
                 invoice_date, due_date, billing_event_id,
                 customer_po_number, contract_number, notes,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,'{}'::jsonb,$22)
            RETURNING *"#,
        )
        .bind(org_id).bind(invoice_number).bind(project_id).bind(project_number).bind(project_name)
        .bind(invoice_type).bind(customer_id).bind(customer_name)
        .bind(invoice_amount).bind(tax_amount).bind(retention_held).bind(total_amount).bind(currency_code)
        .bind(billing_period_start).bind(billing_period_end)
        .bind(invoice_date).bind(due_date).bind(billing_event_id)
        .bind(customer_po_number).bind(contract_number).bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_invoice(&row))
    }

    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ProjectInvoiceHeader>> {
        let row = sqlx::query("SELECT * FROM _atlas.project_invoice_headers WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_invoice))
    }

    async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ProjectInvoiceHeader>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.project_invoice_headers WHERE organization_id = $1 AND invoice_number = $2"
        ).bind(org_id).bind(invoice_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_invoice))
    }

    async fn list_invoices(
        &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<ProjectInvoiceHeader>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.project_invoice_headers
               WHERE organization_id = $1
                 AND ($2::uuid IS NULL OR project_id = $2)
                 AND ($3::text IS NULL OR status = $3)
               ORDER BY invoice_date DESC, created_at DESC"#,
        ).bind(org_id).bind(project_id).bind(status)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_invoice).collect())
    }

    async fn update_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectInvoiceHeader> {
        let row = sqlx::query(
            "UPDATE _atlas.project_invoice_headers SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
        Ok(row_to_invoice(&row))
    }

    async fn reject_invoice(&self, id: Uuid, reason: &str) -> AtlasResult<ProjectInvoiceHeader> {
        let row = sqlx::query(
            r#"UPDATE _atlas.project_invoice_headers
               SET status = 'rejected', rejected_reason = $2, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
        Ok(row_to_invoice(&row))
    }

    async fn mark_invoice_posted(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
        let row = sqlx::query(
            r#"UPDATE _atlas.project_invoice_headers
               SET status = 'posted', gl_posted_flag = true, gl_posted_date = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
        Ok(row_to_invoice(&row))
    }

    async fn delete_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.project_invoice_headers WHERE organization_id = $1 AND invoice_number = $2"
        ).bind(org_id).bind(invoice_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Invoice '{}' not found", invoice_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Invoice Lines
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_invoice_line(
        &self, org_id: Uuid, invoice_header_id: Uuid, line_number: i32,
        line_source: &str, expenditure_item_id: Option<Uuid>,
        billing_event_id: Option<Uuid>,
        task_id: Option<Uuid>, task_number: Option<&str>, task_name: Option<&str>,
        description: Option<&str>,
        employee_id: Option<Uuid>, employee_name: Option<&str>,
        role_name: Option<&str>, expenditure_type: Option<&str>,
        quantity: f64, unit_of_measure: &str, bill_rate: f64,
        raw_cost_amount: f64, bill_amount: f64, markup_amount: f64,
        retention_amount: f64, tax_amount: f64,
        transaction_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ProjectInvoiceLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.project_invoice_lines
                (organization_id, invoice_header_id, line_number, line_source,
                 expenditure_item_id, billing_event_id, task_id, task_number, task_name,
                 description, employee_id, employee_name, role_name, expenditure_type,
                 quantity, unit_of_measure, bill_rate,
                 raw_cost_amount, bill_amount, markup_amount, retention_amount, tax_amount,
                 transaction_date, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,'{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(invoice_header_id).bind(line_number).bind(line_source)
        .bind(expenditure_item_id).bind(billing_event_id)
        .bind(task_id).bind(task_number).bind(task_name)
        .bind(description).bind(employee_id).bind(employee_name).bind(role_name).bind(expenditure_type)
        .bind(quantity).bind(unit_of_measure).bind(bill_rate)
        .bind(raw_cost_amount).bind(bill_amount).bind(markup_amount).bind(retention_amount).bind(tax_amount)
        .bind(transaction_date)
        .fetch_one(&self.pool).await?;
        Ok(row_to_invoice_line(&row))
    }

    async fn list_invoice_lines(&self, invoice_header_id: Uuid) -> AtlasResult<Vec<ProjectInvoiceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.project_invoice_lines WHERE invoice_header_id = $1 ORDER BY line_number"
        ).bind(invoice_header_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_invoice_line).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProjectBillingDashboard> {
        // Count billing configs
        let config_rows = sqlx::query(
            "SELECT billing_method FROM _atlas.project_billing_configs WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut total_projects_billable = 0i32;
        let mut total_contract_value = 0.0_f64;
        let mut by_billing_method = std::collections::HashMap::new();

        for row in &config_rows {
            total_projects_billable += 1;
            let amount: f64 = row.try_get("contract_amount").unwrap_or(0.0);
            total_contract_value += amount;
            let method: String = row.try_get("billing_method").unwrap_or_default();
            *by_billing_method.entry(method).or_insert(0i32) += 1;
        }

        // Count invoices
        let inv_rows = sqlx::query(
            "SELECT status, invoice_amount, retention_held FROM _atlas.project_invoice_headers WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut total_invoices = 0i32;
        let mut draft_invoices = 0i32;
        let mut submitted_invoices = 0i32;
        let mut approved_invoices = 0i32;
        let mut posted_invoices = 0i32;
        let overdue_invoices = 0i32;
        let mut total_billed = 0.0_f64;
        let mut total_retention_held = 0.0_f64;
        let mut by_invoice_status = std::collections::HashMap::new();

        for row in &inv_rows {
            total_invoices += 1;
            let status: String = row.try_get("status").unwrap_or_default();
            let amount: f64 = row.try_get("invoice_amount").unwrap_or(0.0);
            let ret: f64 = row.try_get("retention_held").unwrap_or(0.0);
            total_billed += amount;
            total_retention_held += ret;

            match status.as_str() {
                "draft" => draft_invoices += 1,
                "submitted" => submitted_invoices += 1,
                "approved" => approved_invoices += 1,
                "posted" => posted_invoices += 1,
                _ => {}
            }
            *by_invoice_status.entry(status).or_insert(0i32) += 1;
        }

        Ok(ProjectBillingDashboard {
            total_projects_billable,
            total_contract_value,
            total_billed,
            total_unbilled: (total_contract_value - total_billed).max(0.0),
            total_retention_held,
            total_retention_released: 0.0,
            total_invoices,
            draft_invoices,
            submitted_invoices,
            approved_invoices,
            posted_invoices,
            overdue_invoices,
            total_revenue_recognized: total_billed,
            by_billing_method: serde_json::to_value(&by_billing_method).unwrap_or_default(),
            by_invoice_status: serde_json::to_value(&by_invoice_status).unwrap_or_default(),
            billing_trend: serde_json::json!({}),
        })
    }
}
