//! Finance Charge Repository
//!
//! `PostgreSQL` storage for finance charge terms, assessment runs, and charge invoices.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Finance charge term definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeTerm {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub term_code: String,
    pub term_name: String,
    pub description: Option<String>,
    pub charge_type: String, // percentage, flat_fee, tiered
    pub charge_rate: Option<f64>,
    pub minimum_charge: Option<f64>,
    pub maximum_charge: Option<f64>,
    pub grace_period_days: i32,
    pub currency_code: String,
    pub calculation_basis: String, // daily, monthly, annual
    pub include_tax: bool,
    pub compound_charges: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub auto_assess: bool,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge tier for tiered charge types
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub term_id: Uuid,
    pub from_days_overdue: i32,
    pub to_days_overdue: Option<i32>,
    pub charge_rate: f64,
    pub flat_fee: Option<f64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge assessment run
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub run_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub term_id: Option<Uuid>,
    pub term_code: Option<String>,
    pub currency_code: String,
    pub total_invoices_assessed: i32,
    pub total_charges_assessed: f64,
    pub status: String, // draft, submitted, approved, applied, cancelled
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge line (individual charge per invoice in a run)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    pub days_overdue: i32,
    pub invoice_amount: f64,
    pub outstanding_amount: f64,
    pub charge_type: String,
    pub charge_rate: f64,
    pub charge_amount: f64,
    pub currency_code: String,
    pub term_id: Option<Uuid>,
    pub term_code: Option<String>,
    pub charge_invoice_id: Option<Uuid>,
    pub charge_invoice_number: Option<String>,
    pub status: String, // pending, charged, waived, cancelled
    pub waived_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge invoice (the actual invoice generated)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeInvoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_invoice_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub currency_code: String,
    pub total_charge_amount: f64,
    pub amount_applied: f64,
    pub amount_remaining: f64,
    pub status: String, // open, paid, cancelled, reversed
    pub run_id: Option<Uuid>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge activity audit log
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String, // run, line, invoice, term
    pub entity_id: Uuid,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Finance charge dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChargeSummary {
    pub total_terms: i64,
    pub active_terms: i64,
    pub total_runs: i64,
    pub pending_runs: i64,
    pub completed_runs: i64,
    pub total_charges_assessed: f64,
    pub total_charges_collected: f64,
    pub total_charges_outstanding: f64,
    pub invoices_assessed: i64,
    pub by_charge_type: serde_json::Value,
    pub by_status: serde_json::Value,
}

// ============================================================================
// Create Parameters
// ============================================================================

pub struct FinanceChargeTermCreateParams {
    pub org_id: Uuid,
    pub term_code: String,
    pub term_name: String,
    pub description: Option<String>,
    pub charge_type: String,
    pub charge_rate: Option<f64>,
    pub minimum_charge: Option<f64>,
    pub maximum_charge: Option<f64>,
    pub grace_period_days: i32,
    pub currency_code: String,
    pub calculation_basis: String,
    pub include_tax: bool,
    pub compound_charges: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub auto_assess: bool,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub created_by: Option<Uuid>,
}

pub struct FinanceChargeRunCreateParams {
    pub org_id: Uuid,
    pub run_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub term_id: Option<Uuid>,
    pub term_code: Option<String>,
    pub currency_code: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

pub struct FinanceChargeLineCreateParams {
    pub org_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    pub days_overdue: i32,
    pub invoice_amount: f64,
    pub outstanding_amount: f64,
    pub charge_type: String,
    pub charge_rate: f64,
    pub charge_amount: f64,
    pub currency_code: String,
    pub term_id: Option<Uuid>,
    pub term_code: Option<String>,
}

pub struct FinanceChargeInvoiceCreateParams {
    pub org_id: Uuid,
    pub charge_invoice_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub currency_code: String,
    pub total_charge_amount: f64,
    pub run_id: Option<Uuid>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

// ============================================================================
// Repository Trait
// ============================================================================

#[async_trait]
pub trait FinanceChargeRepository: Send + Sync {
    // Terms
    async fn create_term(&self, params: &FinanceChargeTermCreateParams) -> AtlasResult<FinanceChargeTerm>;
    async fn get_term(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeTerm>>;
    async fn get_term_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinanceChargeTerm>>;
    async fn list_terms(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinanceChargeTerm>>;
    async fn update_term_status(&self, id: Uuid, is_active: bool) -> AtlasResult<FinanceChargeTerm>;

    // Tiers
    async fn create_tier(&self, org_id: Uuid, term_id: Uuid, from_days: i32, to_days: Option<i32>, rate: f64, flat_fee: Option<f64>) -> AtlasResult<FinanceChargeTier>;
    async fn list_tiers(&self, term_id: Uuid) -> AtlasResult<Vec<FinanceChargeTier>>;

    // Runs
    async fn create_run(&self, params: &FinanceChargeRunCreateParams) -> AtlasResult<FinanceChargeRun>;
    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeRun>>;
    async fn get_run_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FinanceChargeRun>>;
    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<FinanceChargeRun>;
    async fn update_run_totals(&self, id: Uuid, total_invoices: i32, total_charges: f64) -> AtlasResult<()>;
    async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()>;
    async fn get_next_run_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Lines
    async fn create_line(&self, params: &FinanceChargeLineCreateParams) -> AtlasResult<FinanceChargeLine>;
    async fn list_lines(&self, run_id: Uuid) -> AtlasResult<Vec<FinanceChargeLine>>;
    async fn update_line_status(&self, id: Uuid, status: &str, waived_reason: Option<&str>) -> AtlasResult<FinanceChargeLine>;
    async fn update_line_charge_invoice(&self, id: Uuid, charge_invoice_id: Uuid, charge_invoice_number: Option<&str>) -> AtlasResult<()>;
    async fn delete_lines_for_run(&self, run_id: Uuid) -> AtlasResult<()>;

    // Invoices
    async fn create_invoice(&self, params: &FinanceChargeInvoiceCreateParams) -> AtlasResult<FinanceChargeInvoice>;
    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeInvoice>>;
    async fn get_invoice_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeInvoice>>;
    async fn list_invoices(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<FinanceChargeInvoice>>;
    async fn update_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<FinanceChargeInvoice>;
    async fn get_next_invoice_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Activities
    async fn create_activity(
        &self, org_id: Uuid, entity_type: &str, entity_id: Uuid,
        activity_type: &str, description: Option<&str>,
        old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>,
    ) -> AtlasResult<FinanceChargeActivity>;
    async fn list_activities(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<FinanceChargeActivity>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FinanceChargeSummary>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresFinanceChargeRepository {
    pool: PgPool,
}

impl PostgresFinanceChargeRepository {
    #[must_use] 
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FinanceChargeRepository for PostgresFinanceChargeRepository {
    // ========================================================================
    // Terms
    // ========================================================================

    async fn create_term(&self, params: &FinanceChargeTermCreateParams) -> AtlasResult<FinanceChargeTerm> {
        let row = sqlx::query_as::<_, FinanceChargeTerm>(
            r"INSERT INTO _atlas.finance_charge_terms
               (organization_id, term_code, term_name, description,
                charge_type, charge_rate, minimum_charge, maximum_charge,
                grace_period_days, currency_code, calculation_basis,
                include_tax, compound_charges,
                effective_from, effective_to, is_active, auto_assess,
                revenue_account_code, receivable_account_code, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(&params.term_code)
        .bind(&params.term_name)
        .bind(&params.description)
        .bind(&params.charge_type)
        .bind(params.charge_rate)
        .bind(params.minimum_charge)
        .bind(params.maximum_charge)
        .bind(params.grace_period_days)
        .bind(&params.currency_code)
        .bind(&params.calculation_basis)
        .bind(params.include_tax)
        .bind(params.compound_charges)
        .bind(params.effective_from)
        .bind(params.effective_to)
        .bind(params.is_active)
        .bind(params.auto_assess)
        .bind(&params.revenue_account_code)
        .bind(&params.receivable_account_code)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_term(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeTerm>> {
        let row = sqlx::query_as::<_, FinanceChargeTerm>(
            "SELECT * FROM _atlas.finance_charge_terms WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_term_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinanceChargeTerm>> {
        let row = sqlx::query_as::<_, FinanceChargeTerm>(
            "SELECT * FROM _atlas.finance_charge_terms WHERE organization_id = $1 AND term_code = $2",
        )
        .bind(org_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_terms(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinanceChargeTerm>> {
        let rows = sqlx::query_as::<_, FinanceChargeTerm>(
            r"SELECT * FROM _atlas.finance_charge_terms
               WHERE organization_id = $1
               AND ($2::boolean IS NULL OR is_active = $2)
               ORDER BY term_code",
        )
        .bind(org_id)
        .bind(is_active)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_term_status(&self, id: Uuid, is_active: bool) -> AtlasResult<FinanceChargeTerm> {
        let row = sqlx::query_as::<_, FinanceChargeTerm>(
            "UPDATE _atlas.finance_charge_terms SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    // ========================================================================
    // Tiers
    // ========================================================================

    async fn create_tier(&self, org_id: Uuid, term_id: Uuid, from_days: i32, to_days: Option<i32>, rate: f64, flat_fee: Option<f64>) -> AtlasResult<FinanceChargeTier> {
        let row = sqlx::query_as::<_, FinanceChargeTier>(
            r"INSERT INTO _atlas.finance_charge_tiers
               (organization_id, term_id, from_days_overdue, to_days_overdue, charge_rate, flat_fee)
               VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
        )
        .bind(org_id)
        .bind(term_id)
        .bind(from_days)
        .bind(to_days)
        .bind(rate)
        .bind(flat_fee)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_tiers(&self, term_id: Uuid) -> AtlasResult<Vec<FinanceChargeTier>> {
        let rows = sqlx::query_as::<_, FinanceChargeTier>(
            "SELECT * FROM _atlas.finance_charge_tiers WHERE term_id = $1 ORDER BY from_days_overdue",
        )
        .bind(term_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    // ========================================================================
    // Runs
    // ========================================================================

    async fn create_run(&self, params: &FinanceChargeRunCreateParams) -> AtlasResult<FinanceChargeRun> {
        let seq = self.get_next_run_number(params.org_id).await.unwrap_or(1);
        let run_number = format!("FCR-{seq:06}");

        let row = sqlx::query_as::<_, FinanceChargeRun>(
            r"INSERT INTO _atlas.finance_charge_runs
               (organization_id, run_number, run_date, gl_date,
                term_id, term_code, currency_code,
                total_invoices_assessed, total_charges_assessed,
                status, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,0,0.0,'draft',$8,$9)
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(&run_number)
        .bind(params.run_date)
        .bind(params.gl_date)
        .bind(params.term_id)
        .bind(&params.term_code)
        .bind(&params.currency_code)
        .bind(&params.notes)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeRun>> {
        let row = sqlx::query_as::<_, FinanceChargeRun>(
            "SELECT * FROM _atlas.finance_charge_runs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_run_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeRun>> {
        let row = sqlx::query_as::<_, FinanceChargeRun>(
            "SELECT * FROM _atlas.finance_charge_runs WHERE organization_id = $1 AND run_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FinanceChargeRun>> {
        let rows = sqlx::query_as::<_, FinanceChargeRun>(
            r"SELECT * FROM _atlas.finance_charge_runs
               WHERE organization_id = $1
               AND ($2::text IS NULL OR status = $2)
               ORDER BY run_date DESC, run_number",
        )
        .bind(org_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<FinanceChargeRun> {
        let row = sqlx::query_as::<_, FinanceChargeRun>(
            r"UPDATE _atlas.finance_charge_runs
               SET status = $2,
                   submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                   submitted_by = CASE WHEN $2 = 'submitted' THEN created_by ELSE submitted_by END,
                   approved_at = CASE WHEN $2 IN ('approved','applied') THEN now() ELSE approved_at END,
                   approved_by = CASE WHEN $2 IN ('approved','applied') THEN created_by ELSE approved_by END,
                   updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_run_totals(&self, id: Uuid, total_invoices: i32, total_charges: f64) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.finance_charge_runs SET total_invoices_assessed = $2, total_charges_assessed = $3, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(total_invoices)
        .bind(total_charges)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.finance_charge_runs WHERE organization_id = $1 AND run_number = $2 AND status = 'draft'",
        )
        .bind(org_id)
        .bind(run_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Finance charge run not found or not in draft status".to_string()));
        }
        Ok(())
    }

    async fn get_next_run_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(run_number FROM 'FCR-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.finance_charge_runs WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // ========================================================================
    // Lines
    // ========================================================================

    async fn create_line(&self, params: &FinanceChargeLineCreateParams) -> AtlasResult<FinanceChargeLine> {
        let row = sqlx::query_as::<_, FinanceChargeLine>(
            r"INSERT INTO _atlas.finance_charge_lines
               (organization_id, run_id, line_number,
                customer_id, customer_number, customer_name,
                invoice_id, invoice_number, invoice_date, invoice_due_date,
                days_overdue, invoice_amount, outstanding_amount,
                charge_type, charge_rate, charge_amount, currency_code,
                term_id, term_code, status)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,'pending')
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(params.run_id)
        .bind(params.line_number)
        .bind(params.customer_id)
        .bind(&params.customer_number)
        .bind(&params.customer_name)
        .bind(params.invoice_id)
        .bind(&params.invoice_number)
        .bind(params.invoice_date)
        .bind(params.invoice_due_date)
        .bind(params.days_overdue)
        .bind(params.invoice_amount)
        .bind(params.outstanding_amount)
        .bind(&params.charge_type)
        .bind(params.charge_rate)
        .bind(params.charge_amount)
        .bind(&params.currency_code)
        .bind(params.term_id)
        .bind(&params.term_code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_lines(&self, run_id: Uuid) -> AtlasResult<Vec<FinanceChargeLine>> {
        let rows = sqlx::query_as::<_, FinanceChargeLine>(
            "SELECT * FROM _atlas.finance_charge_lines WHERE run_id = $1 ORDER BY line_number",
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_line_status(&self, id: Uuid, status: &str, waived_reason: Option<&str>) -> AtlasResult<FinanceChargeLine> {
        let row = sqlx::query_as::<_, FinanceChargeLine>(
            r"UPDATE _atlas.finance_charge_lines
               SET status = $2, waived_reason = COALESCE($3, waived_reason), updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .bind(waived_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_line_charge_invoice(&self, id: Uuid, charge_invoice_id: Uuid, charge_invoice_number: Option<&str>) -> AtlasResult<()> {
        sqlx::query(
            r"UPDATE _atlas.finance_charge_lines
               SET charge_invoice_id = $2, charge_invoice_number = $3, status = 'charged', updated_at = now()
               WHERE id = $1",
        )
        .bind(id)
        .bind(charge_invoice_id)
        .bind(charge_invoice_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_lines_for_run(&self, run_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.finance_charge_lines WHERE run_id = $1")
            .bind(run_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Invoices
    // ========================================================================

    async fn create_invoice(&self, params: &FinanceChargeInvoiceCreateParams) -> AtlasResult<FinanceChargeInvoice> {
        let row = sqlx::query_as::<_, FinanceChargeInvoice>(
            r"INSERT INTO _atlas.finance_charge_invoices
               (organization_id, charge_invoice_number,
                customer_id, customer_number, customer_name,
                invoice_date, gl_date, due_date, currency_code,
                total_charge_amount, amount_applied, amount_remaining,
                status, run_id,
                revenue_account_code, receivable_account_code,
                notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,0.0,$10,'open',$11,$12,$13,$14,$15)
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(&params.charge_invoice_number)
        .bind(params.customer_id)
        .bind(&params.customer_number)
        .bind(&params.customer_name)
        .bind(params.invoice_date)
        .bind(params.gl_date)
        .bind(params.due_date)
        .bind(&params.currency_code)
        .bind(params.total_charge_amount)
        .bind(params.run_id)
        .bind(&params.revenue_account_code)
        .bind(&params.receivable_account_code)
        .bind(&params.notes)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeInvoice>> {
        let row = sqlx::query_as::<_, FinanceChargeInvoice>(
            "SELECT * FROM _atlas.finance_charge_invoices WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_invoice_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeInvoice>> {
        let row = sqlx::query_as::<_, FinanceChargeInvoice>(
            "SELECT * FROM _atlas.finance_charge_invoices WHERE organization_id = $1 AND charge_invoice_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_invoices(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<FinanceChargeInvoice>> {
        let rows = sqlx::query_as::<_, FinanceChargeInvoice>(
            r"SELECT * FROM _atlas.finance_charge_invoices
               WHERE organization_id = $1
               AND ($2::text IS NULL OR status = $2)
               AND ($3::uuid IS NULL OR customer_id = $3)
               ORDER BY invoice_date DESC, charge_invoice_number",
        )
        .bind(org_id)
        .bind(status)
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<FinanceChargeInvoice> {
        let row = sqlx::query_as::<_, FinanceChargeInvoice>(
            "UPDATE _atlas.finance_charge_invoices SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_next_invoice_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(charge_invoice_number FROM 'FCI-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.finance_charge_invoices WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // ========================================================================
    // Activities
    // ========================================================================

    async fn create_activity(
        &self, org_id: Uuid, entity_type: &str, entity_id: Uuid,
        activity_type: &str, description: Option<&str>,
        old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>,
    ) -> AtlasResult<FinanceChargeActivity> {
        let row = sqlx::query_as::<_, FinanceChargeActivity>(
            r"INSERT INTO _atlas.finance_charge_activities
               (organization_id, entity_type, entity_id, activity_type,
                description, old_status, new_status, performed_by, performed_by_name)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING *",
        )
        .bind(org_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(activity_type)
        .bind(description)
        .bind(old_status)
        .bind(new_status)
        .bind(performed_by)
        .bind(performed_by_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_activities(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<FinanceChargeActivity>> {
        let rows = sqlx::query_as::<_, FinanceChargeActivity>(
            "SELECT * FROM _atlas.finance_charge_activities WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FinanceChargeSummary> {
        let total_terms: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.finance_charge_terms WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_terms: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.finance_charge_terms WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.finance_charge_runs WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let pending_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.finance_charge_runs WHERE organization_id = $1 AND status IN ('draft','submitted')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let completed_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.finance_charge_runs WHERE organization_id = $1 AND status IN ('approved','applied')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_assessed: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_charges_assessed), 0) FROM _atlas.finance_charge_runs WHERE organization_id = $1 AND status IN ('approved','applied')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_collected: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_charge_amount), 0) FROM _atlas.finance_charge_invoices WHERE organization_id = $1 AND status = 'paid'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_outstanding: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount_remaining), 0) FROM _atlas.finance_charge_invoices WHERE organization_id = $1 AND status = 'open'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let invoices_assessed: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_invoices_assessed), 0) FROM _atlas.finance_charge_runs WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_charge_type = self.get_run_grouped_count("term_code", org_id).await?;
        let by_status = self.get_grouped_count("status", org_id).await?;

        Ok(FinanceChargeSummary {
            total_terms,
            active_terms,
            total_runs,
            pending_runs,
            completed_runs,
            total_charges_assessed: total_assessed,
            total_charges_collected: total_collected,
            total_charges_outstanding: total_outstanding,
            invoices_assessed,
            by_charge_type,
            by_status,
        })
    }
}

impl PostgresFinanceChargeRepository {
    async fn get_grouped_count(&self, column: &str, org_id: Uuid) -> AtlasResult<serde_json::Value> {
        let query = format!(
            "SELECT {column} as key, COUNT(*) as cnt, COALESCE(SUM(total_charge_amount), 0) as total FROM _atlas.finance_charge_invoices WHERE organization_id = $1 GROUP BY {column}"
        );
        let rows = sqlx::query(&query)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut result = serde_json::Map::new();
        for row in rows {
            let key: String = row.try_get("key").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            let total: f64 = row.try_get("total").unwrap_or(0.0);
            result.insert(key, serde_json::json!({"count": cnt, "amount": total}));
        }
        Ok(serde_json::Value::Object(result))
    }

    async fn get_run_grouped_count(&self, column: &str, org_id: Uuid) -> AtlasResult<serde_json::Value> {
        let query = format!(
            "SELECT {column} as key, COUNT(*) as cnt, COALESCE(SUM(total_charges_assessed), 0) as total FROM _atlas.finance_charge_runs WHERE organization_id = $1 GROUP BY {column}"
        );
        let rows = sqlx::query(&query)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut result = serde_json::Map::new();
        for row in rows {
            let key: String = row.try_get("key").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            let total: f64 = row.try_get("total").unwrap_or(0.0);
            result.insert(key, serde_json::json!({"count": cnt, "amount": total}));
        }
        Ok(serde_json::Value::Object(result))
    }
}
