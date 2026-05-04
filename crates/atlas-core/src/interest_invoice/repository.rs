//! Interest Invoice Repository
//!
//! PostgreSQL storage for interest rate schedules, overdue invoices,
//! calculation runs, calculation lines, interest invoices, and invoice lines.

use atlas_shared::{
    InterestRateSchedule, OverdueInvoice, InterestCalculationRun,
    InterestCalculationLine, InterestInvoice, InterestInvoiceLine,
    InterestInvoiceDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for interest invoice data storage
#[async_trait]
pub trait InterestInvoiceRepository: Send + Sync {
    // Interest Rate Schedules
    async fn create_schedule(
        &self, org_id: Uuid, schedule_code: &str, name: &str, description: Option<&str>,
        annual_rate: &str, compounding_frequency: &str, charge_type: &str,
        grace_period_days: i32, minimum_charge: &str, maximum_charge: Option<&str>,
        currency_code: &str, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InterestRateSchedule>;
    async fn get_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<Option<InterestRateSchedule>>;
    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestRateSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestRateSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<InterestRateSchedule>;
    async fn delete_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<()>;

    // Overdue Invoices
    async fn register_overdue_invoice(
        &self, org_id: Uuid, invoice_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, original_amount: &str, outstanding_amount: &str,
        due_date: chrono::NaiveDate, overdue_days: i32, currency_code: &str,
    ) -> AtlasResult<OverdueInvoice>;
    async fn get_overdue_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<OverdueInvoice>>;
    async fn get_overdue_invoice_by_id(&self, id: Uuid) -> AtlasResult<Option<OverdueInvoice>>;
    async fn list_overdue_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<OverdueInvoice>>;
    async fn update_overdue_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<OverdueInvoice>;
    async fn update_overdue_interest(&self, id: Uuid, total_interest: &str, last_interest_date: chrono::NaiveDate) -> AtlasResult<OverdueInvoice>;

    // Calculation Runs
    async fn create_calculation_run(
        &self, org_id: Uuid, run_number: &str, description: Option<&str>,
        calculation_date: chrono::NaiveDate, schedule_id: Option<Uuid>,
        currency_code: &str, generated_by: Option<Uuid>,
    ) -> AtlasResult<InterestCalculationRun>;
    async fn get_calculation_run(&self, id: Uuid) -> AtlasResult<Option<InterestCalculationRun>>;
    async fn list_calculation_runs(&self, org_id: Uuid) -> AtlasResult<Vec<InterestCalculationRun>>;
    async fn update_calculation_run_totals(&self, id: Uuid, invoices_processed: i32, total_interest: &str) -> AtlasResult<InterestCalculationRun>;
    async fn update_calculation_run_status(&self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<InterestCalculationRun>;
    async fn get_latest_run_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Calculation Lines
    async fn create_calculation_line(
        &self, org_id: Uuid, run_id: Uuid, overdue_invoice_id: Option<Uuid>,
        invoice_number: &str, customer_id: Uuid, customer_name: Option<&str>,
        outstanding_amount: &str, overdue_days: i32, annual_rate_used: &str,
        interest_amount: &str, currency_code: &str,
    ) -> AtlasResult<InterestCalculationLine>;
    async fn list_calculation_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InterestCalculationLine>>;
    async fn update_calculation_line_status(&self, id: Uuid, status: &str, invoice_id: Option<Uuid>) -> AtlasResult<InterestCalculationLine>;

    // Interest Invoices
    async fn create_interest_invoice(
        &self, org_id: Uuid, invoice_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, calculation_run_id: Option<Uuid>,
        invoice_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        total_interest_amount: &str, currency_code: &str, line_count: i32,
        gl_account_code: Option<&str>, reference_invoice_number: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<InterestInvoice>;
    async fn get_interest_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<InterestInvoice>>;
    async fn get_interest_invoice_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestInvoice>>;
    async fn list_interest_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestInvoice>>;
    async fn update_interest_invoice_status(
        &self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>,
        reversed_at: Option<chrono::DateTime<chrono::Utc>>, reversal_invoice_id: Option<Uuid>,
    ) -> AtlasResult<InterestInvoice>;
    async fn get_latest_invoice_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Interest Invoice Lines
    async fn create_interest_invoice_line(
        &self, org_id: Uuid, interest_invoice_id: Uuid, calculation_line_id: Option<Uuid>,
        line_number: i32, line_type: &str, description: Option<&str>,
        reference_invoice_number: Option<&str>, overdue_days: Option<i32>,
        outstanding_amount: Option<&str>, annual_rate_used: Option<&str>,
        interest_amount: &str, currency_code: &str, gl_account_code: Option<&str>,
    ) -> AtlasResult<InterestInvoiceLine>;
    async fn list_interest_invoice_lines(&self, interest_invoice_id: Uuid) -> AtlasResult<Vec<InterestInvoiceLine>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InterestInvoiceDashboard>;
}


/// Read a NUMERIC column as text string from a PostgreSQL row.
/// PostgreSQL NUMERIC type doesn't map directly to Rust types via SQLx,
/// so we use the pgrowlocks::get pattern with explicit text casting.
fn get_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<String, _>(col).unwrap_or_else(|_| "0.00".to_string())
}

fn get_numeric_text_precise(row: &sqlx::postgres::PgRow, col: &str, _precision: usize) -> String {
    row.try_get::<String, _>(col).unwrap_or_else(|_| "0".to_string())
}

fn get_optional_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(col).unwrap_or(None)
}

/// PostgreSQL implementation
pub struct PostgresInterestInvoiceRepository {
    pool: PgPool,
}

impl PostgresInterestInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_schedule(row: &sqlx::postgres::PgRow) -> InterestRateSchedule {
    use serde_json::Value;
    InterestRateSchedule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_code: row.get("schedule_code"),
        name: row.get("name"),
        description: row.get("description"),
        annual_rate: get_numeric_text_precise(row, "annual_rate", 6),
        compounding_frequency: row.get("compounding_frequency"),
        charge_type: row.get("charge_type"),
        grace_period_days: row.get("grace_period_days"),
        minimum_charge: get_numeric_text(row, "minimum_charge"),
        maximum_charge: get_optional_numeric_text(row, "maximum_charge"),
        currency_code: row.get("currency_code"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_overdue_invoice(row: &sqlx::postgres::PgRow) -> OverdueInvoice {
    use serde_json::Value;
    OverdueInvoice {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        invoice_number: row.get("invoice_number"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        original_amount: get_numeric_text(row, "original_amount"),
        outstanding_amount: get_numeric_text(row, "outstanding_amount"),
        due_date: row.get("due_date"),
        overdue_days: row.get("overdue_days"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        last_interest_date: row.get("last_interest_date"),
        total_interest_charged: get_numeric_text(row, "total_interest_charged"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_calc_run(row: &sqlx::postgres::PgRow) -> InterestCalculationRun {
    use serde_json::Value;
    InterestCalculationRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_number: row.get("run_number"),
        description: row.get("description"),
        calculation_date: row.get("calculation_date"),
        schedule_id: row.get("schedule_id"),
        total_invoices_processed: row.get("total_invoices_processed"),
        total_interest_calculated: get_numeric_text(row, "total_interest_calculated"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        generated_by: row.get("generated_by"),
        posted_at: row.get("posted_at"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_calc_line(row: &sqlx::postgres::PgRow) -> InterestCalculationLine {
    use serde_json::Value;
    InterestCalculationLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_id: row.get("run_id"),
        overdue_invoice_id: row.get("overdue_invoice_id"),
        invoice_number: row.get("invoice_number"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        outstanding_amount: get_numeric_text(row, "outstanding_amount"),
        overdue_days: row.get("overdue_days"),
        annual_rate_used: get_numeric_text_precise(row, "annual_rate_used", 6),
        interest_amount: get_numeric_text(row, "interest_amount"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        interest_invoice_id: row.get("interest_invoice_id"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_interest_invoice(row: &sqlx::postgres::PgRow) -> InterestInvoice {
    use serde_json::Value;
    InterestInvoice {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        invoice_number: row.get("invoice_number"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        calculation_run_id: row.get("calculation_run_id"),
        invoice_date: row.get("invoice_date"),
        due_date: row.get("due_date"),
        total_interest_amount: get_numeric_text(row, "total_interest_amount"),
        currency_code: row.get("currency_code"),
        line_count: row.get("line_count"),
        status: row.get("status"),
        gl_account_code: row.get("gl_account_code"),
        posted_at: row.get("posted_at"),
        reversed_at: row.get("reversed_at"),
        reversal_invoice_id: row.get("reversal_invoice_id"),
        reference_invoice_number: row.get("reference_invoice_number"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_invoice_line(row: &sqlx::postgres::PgRow) -> InterestInvoiceLine {
    use serde_json::Value;
    InterestInvoiceLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        interest_invoice_id: row.get("interest_invoice_id"),
        calculation_line_id: row.get("calculation_line_id"),
        line_number: row.get("line_number"),
        line_type: row.get("line_type"),
        description: row.get("description"),
        reference_invoice_number: row.get("reference_invoice_number"),
        overdue_days: row.get("overdue_days"),
        outstanding_amount: get_optional_numeric_text(row, "outstanding_amount"),
        annual_rate_used: get_optional_numeric_text(row, "annual_rate_used"),
        interest_amount: get_numeric_text(row, "interest_amount"),
        currency_code: row.get("currency_code"),
        gl_account_code: row.get("gl_account_code"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl InterestInvoiceRepository for PostgresInterestInvoiceRepository {
    // ========================================================================
    // Interest Rate Schedules
    // ========================================================================

    async fn create_schedule(
        &self, org_id: Uuid, schedule_code: &str, name: &str, description: Option<&str>,
        annual_rate: &str, compounding_frequency: &str, charge_type: &str,
        grace_period_days: i32, minimum_charge: &str, maximum_charge: Option<&str>,
        currency_code: &str, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InterestRateSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interest_rate_schedules
                (organization_id, schedule_code, name, description, annual_rate,
                 compounding_frequency, charge_type, grace_period_days, minimum_charge,
                 maximum_charge, currency_code, effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_code).bind(name).bind(description)
        .bind(annual_rate).bind(compounding_frequency).bind(charge_type)
        .bind(grace_period_days).bind(minimum_charge).bind(maximum_charge)
        .bind(currency_code).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn get_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<Option<InterestRateSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.interest_rate_schedules WHERE organization_id=$1 AND schedule_code=$2"
        )
        .bind(org_id).bind(schedule_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestRateSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.interest_rate_schedules WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestRateSchedule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.interest_rate_schedules
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY schedule_code"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<InterestRateSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interest_rate_schedules SET status=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn delete_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.interest_rate_schedules WHERE organization_id=$1 AND schedule_code=$2"
        )
        .bind(org_id).bind(schedule_code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Overdue Invoices
    // ========================================================================

    async fn register_overdue_invoice(
        &self, org_id: Uuid, invoice_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, original_amount: &str, outstanding_amount: &str,
        due_date: chrono::NaiveDate, overdue_days: i32, currency_code: &str,
    ) -> AtlasResult<OverdueInvoice> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.overdue_invoices
                (organization_id, invoice_number, customer_id, customer_name,
                 original_amount, outstanding_amount, due_date, overdue_days, currency_code)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(invoice_number).bind(customer_id).bind(customer_name)
        .bind(original_amount).bind(outstanding_amount)
        .bind(due_date).bind(overdue_days).bind(currency_code)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_overdue_invoice(&row))
    }

    async fn get_overdue_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<OverdueInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.overdue_invoices WHERE organization_id=$1 AND invoice_number=$2"
        )
        .bind(org_id).bind(invoice_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_overdue_invoice(&r)))
    }

    async fn get_overdue_invoice_by_id(&self, id: Uuid) -> AtlasResult<Option<OverdueInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.overdue_invoices WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_overdue_invoice(&r)))
    }

    async fn list_overdue_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<OverdueInvoice>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.overdue_invoices
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY overdue_days DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_overdue_invoice).collect())
    }

    async fn update_overdue_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<OverdueInvoice> {
        let row = sqlx::query(
            r#"UPDATE _atlas.overdue_invoices SET status=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_overdue_invoice(&row))
    }

    async fn update_overdue_interest(&self, id: Uuid, total_interest: &str, last_interest_date: chrono::NaiveDate) -> AtlasResult<OverdueInvoice> {
        let row = sqlx::query(
            r#"UPDATE _atlas.overdue_invoices
            SET total_interest_charged=$2, last_interest_date=$3, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(total_interest).bind(last_interest_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_overdue_invoice(&row))
    }

    // ========================================================================
    // Calculation Runs
    // ========================================================================

    async fn create_calculation_run(
        &self, org_id: Uuid, run_number: &str, description: Option<&str>,
        calculation_date: chrono::NaiveDate, schedule_id: Option<Uuid>,
        currency_code: &str, generated_by: Option<Uuid>,
    ) -> AtlasResult<InterestCalculationRun> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interest_calculation_runs
                (organization_id, run_number, description, calculation_date,
                 schedule_id, currency_code, generated_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *"#,
        )
        .bind(org_id).bind(run_number).bind(description)
        .bind(calculation_date).bind(schedule_id)
        .bind(currency_code).bind(generated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calc_run(&row))
    }

    async fn get_calculation_run(&self, id: Uuid) -> AtlasResult<Option<InterestCalculationRun>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.interest_calculation_runs WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_calc_run(&r)))
    }

    async fn list_calculation_runs(&self, org_id: Uuid) -> AtlasResult<Vec<InterestCalculationRun>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.interest_calculation_runs WHERE organization_id=$1 ORDER BY calculation_date DESC"
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_calc_run).collect())
    }

    async fn update_calculation_run_totals(&self, id: Uuid, invoices_processed: i32, total_interest: &str) -> AtlasResult<InterestCalculationRun> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interest_calculation_runs
            SET total_invoices_processed=$2, total_interest_calculated=$3, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(invoices_processed).bind(total_interest)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calc_run(&row))
    }

    async fn update_calculation_run_status(&self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<InterestCalculationRun> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interest_calculation_runs SET status=$2,
                posted_at=COALESCE($3, posted_at), updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(posted_at)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calc_run(&row))
    }

    async fn get_latest_run_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(run_number AS INTEGER)), 0) as max_run FROM _atlas.interest_calculation_runs WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max: i32 = row.try_get("max_run").unwrap_or(0);
        Ok(max)
    }

    // ========================================================================
    // Calculation Lines
    // ========================================================================

    async fn create_calculation_line(
        &self, org_id: Uuid, run_id: Uuid, overdue_invoice_id: Option<Uuid>,
        invoice_number: &str, customer_id: Uuid, customer_name: Option<&str>,
        outstanding_amount: &str, overdue_days: i32, annual_rate_used: &str,
        interest_amount: &str, currency_code: &str,
    ) -> AtlasResult<InterestCalculationLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interest_calculation_lines
                (organization_id, run_id, overdue_invoice_id, invoice_number,
                 customer_id, customer_name, outstanding_amount, overdue_days,
                 annual_rate_used, interest_amount, currency_code)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(run_id).bind(overdue_invoice_id).bind(invoice_number)
        .bind(customer_id).bind(customer_name).bind(outstanding_amount)
        .bind(overdue_days).bind(annual_rate_used).bind(interest_amount)
        .bind(currency_code)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calc_line(&row))
    }

    async fn list_calculation_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InterestCalculationLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.interest_calculation_lines WHERE run_id=$1 ORDER BY invoice_number"
        )
        .bind(run_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_calc_line).collect())
    }

    async fn update_calculation_line_status(&self, id: Uuid, status: &str, invoice_id: Option<Uuid>) -> AtlasResult<InterestCalculationLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interest_calculation_lines SET status=$2,
                interest_invoice_id=COALESCE($3, interest_invoice_id), updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(invoice_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calc_line(&row))
    }

    // ========================================================================
    // Interest Invoices
    // ========================================================================

    async fn create_interest_invoice(
        &self, org_id: Uuid, invoice_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, calculation_run_id: Option<Uuid>,
        invoice_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        total_interest_amount: &str, currency_code: &str, line_count: i32,
        gl_account_code: Option<&str>, reference_invoice_number: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<InterestInvoice> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interest_invoices
                (organization_id, invoice_number, customer_id, customer_name,
                 calculation_run_id, invoice_date, due_date, total_interest_amount,
                 currency_code, line_count, gl_account_code, reference_invoice_number,
                 notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *"#,
        )
        .bind(org_id).bind(invoice_number).bind(customer_id).bind(customer_name)
        .bind(calculation_run_id).bind(invoice_date).bind(due_date)
        .bind(total_interest_amount).bind(currency_code).bind(line_count)
        .bind(gl_account_code).bind(reference_invoice_number)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_interest_invoice(&row))
    }

    async fn get_interest_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<InterestInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.interest_invoices WHERE organization_id=$1 AND invoice_number=$2"
        )
        .bind(org_id).bind(invoice_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_interest_invoice(&r)))
    }

    async fn get_interest_invoice_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestInvoice>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.interest_invoices WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_interest_invoice(&r)))
    }

    async fn list_interest_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestInvoice>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.interest_invoices
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY invoice_date DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_interest_invoice).collect())
    }

    async fn update_interest_invoice_status(
        &self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>,
        reversed_at: Option<chrono::DateTime<chrono::Utc>>, reversal_invoice_id: Option<Uuid>,
    ) -> AtlasResult<InterestInvoice> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interest_invoices SET status=$2,
                posted_at=COALESCE($3, posted_at),
                reversed_at=COALESCE($4, reversed_at),
                reversal_invoice_id=COALESCE($5, reversal_invoice_id),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(posted_at).bind(reversed_at).bind(reversal_invoice_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_interest_invoice(&row))
    }

    async fn get_latest_invoice_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(invoice_number AS INTEGER)), 0) as max_inv FROM _atlas.interest_invoices WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max: i32 = row.try_get("max_inv").unwrap_or(0);
        Ok(max)
    }

    // ========================================================================
    // Interest Invoice Lines
    // ========================================================================

    async fn create_interest_invoice_line(
        &self, org_id: Uuid, interest_invoice_id: Uuid, calculation_line_id: Option<Uuid>,
        line_number: i32, line_type: &str, description: Option<&str>,
        reference_invoice_number: Option<&str>, overdue_days: Option<i32>,
        outstanding_amount: Option<&str>, annual_rate_used: Option<&str>,
        interest_amount: &str, currency_code: &str, gl_account_code: Option<&str>,
    ) -> AtlasResult<InterestInvoiceLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interest_invoice_lines
                (organization_id, interest_invoice_id, calculation_line_id,
                 line_number, line_type, description, reference_invoice_number,
                 overdue_days, outstanding_amount, annual_rate_used, interest_amount,
                 currency_code, gl_account_code)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(interest_invoice_id).bind(calculation_line_id)
        .bind(line_number).bind(line_type).bind(description)
        .bind(reference_invoice_number).bind(overdue_days)
        .bind(outstanding_amount)
        .bind(annual_rate_used)
        .bind(interest_amount).bind(currency_code).bind(gl_account_code)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_invoice_line(&row))
    }

    async fn list_interest_invoice_lines(&self, interest_invoice_id: Uuid) -> AtlasResult<Vec<InterestInvoiceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.interest_invoice_lines WHERE interest_invoice_id=$1 ORDER BY line_number"
        )
        .bind(interest_invoice_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_invoice_line).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InterestInvoiceDashboard> {
        // (no external imports needed)
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_schedules,
                (SELECT COUNT(*) FROM _atlas.overdue_invoices WHERE organization_id = $1 AND status = 'open') as overdue_inv,
                (SELECT COALESCE(SUM(outstanding_amount::numeric), 0)::text FROM _atlas.overdue_invoices WHERE organization_id = $1 AND status = 'open') as overdue_amt,
                (SELECT COALESCE(SUM(total_interest_calculated::numeric), 0)::text FROM _atlas.interest_calculation_runs
                    WHERE organization_id = $1 AND EXTRACT(YEAR FROM calculation_date) = EXTRACT(YEAR FROM CURRENT_DATE)) as interest_ytd,
                (SELECT COUNT(*) FROM _atlas.interest_invoices WHERE organization_id = $1 AND status = 'draft') as pending_inv,
                (SELECT COALESCE(SUM(total_interest_amount::numeric), 0)::text FROM _atlas.interest_invoices WHERE organization_id = $1 AND status = 'draft') as pending_amt,
                (SELECT COALESCE(AVG(overdue_days), 0) FROM _atlas.overdue_invoices WHERE organization_id = $1 AND status = 'open')::text as avg_days
            FROM _atlas.interest_rate_schedules WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = row.try_get("active_schedules").unwrap_or(0);
        let overdue_inv: i64 = row.try_get("overdue_inv").unwrap_or(0);
        let overdue_amt: String = row.try_get("overdue_amt").unwrap_or_else(|_| "0".to_string());
        let interest_ytd: String = row.try_get("interest_ytd").unwrap_or_else(|_| "0".to_string());
        let pending_inv: i64 = row.try_get("pending_inv").unwrap_or(0);
        let pending_amt: String = row.try_get("pending_amt").unwrap_or_else(|_| "0".to_string());
        let avg_days: String = row.try_get("avg_days").unwrap_or_else(|_| "0".to_string());

        Ok(InterestInvoiceDashboard {
            total_active_schedules: active as i32,
            total_overdue_invoices: overdue_inv as i32,
            total_overdue_amount: overdue_amt,
            total_interest_ytd: interest_ytd,
            total_pending_invoices: pending_inv as i32,
            total_pending_amount: pending_amt,
            avg_overdue_days: avg_days,
        })
    }
}
