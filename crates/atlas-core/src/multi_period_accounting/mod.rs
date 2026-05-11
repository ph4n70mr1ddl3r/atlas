//! Multi-Period Accounting (MPA) Module
//!
//! Oracle Fusion Cloud ERP-inspired Multi-Period Accounting for General Ledger.
//! Distributes journal entry amounts across multiple accounting periods:
//! - Create reusable MPA templates with distribution methods (equal, custom, days)
//! - Apply templates to create MPA schedules with per-period allocation lines
//! - Recognize or reverse individual period allocations
//! - Track MPA schedule lifecycle (draft → active → `completed/cancelled/on_hold`)
//! - Dashboard with aggregated MPA statistics
//!
//! Schedule statuses: draft → active → `completed/cancelled/on_hold`
//! Schedule line statuses: pending → recognized → reversed
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Multi-Period Accounting

mod engine;

pub use engine::MultiPeriodAccountingEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// MPA Template (reusable distribution definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpaTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_name: String,
    pub description: Option<String>,
    pub distribution_method: String,
    pub number_of_periods: i32,
    pub period_type: String,
    pub deferred_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub status: String,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// MPA Template Line (custom distribution percentage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpaTemplateLine {
    pub id: Uuid,
    pub template_id: Uuid,
    pub period_sequence: i32,
    pub percentage: String,
    pub offset_days: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// MPA Schedule (actual allocation applied to a journal entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpaSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub source_journal_entry_id: Option<Uuid>,
    pub source_journal_line_id: Option<Uuid>,
    pub total_amount: String,
    pub recognized_amount: String,
    pub remaining_amount: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub currency_code: String,
    pub company_code: Option<String>,
    pub cost_center: Option<String>,
    pub account_segment: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// MPA Schedule Line (individual period allocation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpaScheduleLine {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub period_sequence: i32,
    pub period_name: Option<String>,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub amount: String,
    pub percentage: String,
    pub status: String,
    pub journal_entry_id: Option<Uuid>,
    pub recognized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// MPA Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpaDashboard {
    pub total_templates: i64,
    pub active_templates: i64,
    pub total_schedules: i64,
    pub draft_schedules: i64,
    pub active_schedules: i64,
    pub completed_schedules: i64,
    pub cancelled_schedules: i64,
    pub on_hold_schedules: i64,
    pub total_scheduled_amount: String,
    pub total_recognized_amount: String,
    pub total_remaining_amount: String,
    pub pending_lines: i64,
    pub recognized_lines: i64,
    pub reversed_lines: i64,
}

/// Repository trait for MPA persistence
#[async_trait]
pub trait MpaRepository: Send + Sync {
    // Templates
    async fn create_template(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        distribution_method: &str, number_of_periods: i32, period_type: &str,
        deferred_account_code: Option<&str>, expense_account_code: Option<&str>,
        currency_code: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<MpaTemplate>;

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<MpaTemplate>>;
    async fn get_template_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<MpaTemplate>>;
    async fn list_templates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaTemplate>>;
    async fn update_template_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaTemplate>;

    // Template Lines
    async fn add_template_line(&self, template_id: Uuid, period_sequence: i32, percentage: &str, offset_days: i32) -> AtlasResult<MpaTemplateLine>;
    async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<MpaTemplateLine>>;
    async fn delete_template_lines(&self, template_id: Uuid) -> AtlasResult<()>;

    // Schedules
    async fn create_schedule(
        &self, org_id: Uuid, schedule_number: &str, description: Option<&str>,
        template_id: Option<Uuid>, source_journal_entry_id: Option<Uuid>,
        source_journal_line_id: Option<Uuid>, total_amount: &str,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        currency_code: &str, company_code: Option<&str>,
        cost_center: Option<&str>, account_segment: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MpaSchedule>;

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<MpaSchedule>>;
    async fn get_schedule_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<MpaSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaSchedule>;
    async fn update_schedule_amounts(&self, id: Uuid, recognized: &str, remaining: &str) -> AtlasResult<()>;

    // Schedule Lines
    async fn create_schedule_line(
        &self, schedule_id: Uuid, period_sequence: i32,
        period_name: Option<&str>, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, amount: &str, percentage: &str,
    ) -> AtlasResult<MpaScheduleLine>;

    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<MpaScheduleLine>>;
    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<MpaScheduleLine>>;
    async fn update_schedule_line_status(&self, id: Uuid, status: &str, journal_entry_id: Option<Uuid>) -> AtlasResult<MpaScheduleLine>;
    async fn count_pending_lines(&self, schedule_id: Uuid) -> AtlasResult<i64>;
    async fn sum_recognized_for_schedule(&self, schedule_id: Uuid) -> AtlasResult<String>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MpaDashboard>;
}

// ============================================================================
// Row mapping helpers
// ============================================================================

fn row_to_template(row: &sqlx::postgres::PgRow) -> MpaTemplate {
    MpaTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_name: row.get("template_name"),
        description: row.get("description"),
        distribution_method: row.get("distribution_method"),
        number_of_periods: row.get("number_of_periods"),
        period_type: row.get("period_type"),
        deferred_account_code: row.get("deferred_account_code"),
        expense_account_code: row.get("expense_account_code"),
        status: row.get("status"),
        currency_code: row.get("currency_code"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_template_line(row: &sqlx::postgres::PgRow) -> MpaTemplateLine {
    MpaTemplateLine {
        id: row.get("id"),
        template_id: row.get("template_id"),
        period_sequence: row.get("period_sequence"),
        percentage: row.get("percentage"),
        offset_days: row.get("offset_days"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}

fn row_to_schedule(row: &sqlx::postgres::PgRow) -> MpaSchedule {
    MpaSchedule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_number: row.get("schedule_number"),
        description: row.get("description"),
        template_id: row.get("template_id"),
        source_journal_entry_id: row.get("source_journal_entry_id"),
        source_journal_line_id: row.get("source_journal_line_id"),
        total_amount: row.get("total_amount"),
        recognized_amount: row.get("recognized_amount"),
        remaining_amount: row.get("remaining_amount"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        status: row.get("status"),
        currency_code: row.get("currency_code"),
        company_code: row.get("company_code"),
        cost_center: row.get("cost_center"),
        account_segment: row.get("account_segment"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_schedule_line(row: &sqlx::postgres::PgRow) -> MpaScheduleLine {
    MpaScheduleLine {
        id: row.get("id"),
        schedule_id: row.get("schedule_id"),
        period_sequence: row.get("period_sequence"),
        period_name: row.get("period_name"),
        period_start_date: row.get("period_start_date"),
        period_end_date: row.get("period_end_date"),
        amount: row.get("amount"),
        percentage: row.get("percentage"),
        status: row.get("status"),
        journal_entry_id: row.get("journal_entry_id"),
        recognized_at: row.get("recognized_at"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// `PostgreSQL` implementation
#[allow(dead_code)]
pub struct PostgresMpaRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresMpaRepository { #[must_use] 
pub const fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl MpaRepository for PostgresMpaRepository {
    // Templates
    async fn create_template(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        method: &str, periods: i32, period_type: &str,
        deferred: Option<&str>, expense: Option<&str>,
        currency: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<MpaTemplate> {
        let row = sqlx::query(
            r"INSERT INTO financials.mpa_templates
                (organization_id, template_name, description, distribution_method,
                 number_of_periods, period_type, deferred_account_code, expense_account_code,
                 currency_code, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *",
        )
        .bind(org_id).bind(name).bind(description).bind(method)
        .bind(periods).bind(period_type).bind(deferred).bind(expense)
        .bind(currency).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<MpaTemplate>> {
        let row = sqlx::query("SELECT * FROM financials.mpa_templates WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_template_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<MpaTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM financials.mpa_templates WHERE organization_id = $1 AND template_name = $2"
        ).bind(org_id).bind(name)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaTemplate>> {
        let rows = sqlx::query(
            r"SELECT * FROM financials.mpa_templates
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC",
        ).bind(org_id).bind(status)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn update_template_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaTemplate> {
        let row = sqlx::query(
            "UPDATE financials.mpa_templates SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn add_template_line(&self, template_id: Uuid, seq: i32, pct: &str, offset: i32) -> AtlasResult<MpaTemplateLine> {
        let row = sqlx::query(
            r"INSERT INTO financials.mpa_template_lines
                (template_id, period_sequence, percentage, offset_days)
            VALUES ($1,$2,$3,$4) RETURNING *",
        ).bind(template_id).bind(seq).bind(pct).bind(offset)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template_line(&row))
    }

    async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<MpaTemplateLine>> {
        let rows = sqlx::query(
            "SELECT * FROM financials.mpa_template_lines WHERE template_id = $1 ORDER BY period_sequence"
        ).bind(template_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template_line).collect())
    }

    async fn delete_template_lines(&self, template_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM financials.mpa_template_lines WHERE template_id = $1")
            .bind(template_id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Schedules
    async fn create_schedule(
        &self, org_id: Uuid, num: &str, desc: Option<&str>,
        template_id: Option<Uuid>, src_je: Option<Uuid>, src_jl: Option<Uuid>,
        total: &str, start: chrono::NaiveDate, end: Option<chrono::NaiveDate>,
        currency: &str, company: Option<&str>, cc: Option<&str>, acct: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MpaSchedule> {
        let row = sqlx::query(
            r"INSERT INTO financials.mpa_schedules
                (organization_id, schedule_number, description, template_id,
                 source_journal_entry_id, source_journal_line_id,
                 total_amount, remaining_amount, start_date, end_date,
                 currency_code, company_code, cost_center, account_segment, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING *",
        )
        .bind(org_id).bind(num).bind(desc).bind(template_id)
        .bind(src_je).bind(src_jl).bind(total).bind(total)
        .bind(start).bind(end).bind(currency)
        .bind(company).bind(cc).bind(acct).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<MpaSchedule>> {
        let row = sqlx::query("SELECT * FROM financials.mpa_schedules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn get_schedule_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<MpaSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM financials.mpa_schedules WHERE organization_id = $1 AND schedule_number = $2"
        ).bind(org_id).bind(num)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MpaSchedule>> {
        let rows = sqlx::query(
            r"SELECT * FROM financials.mpa_schedules
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC",
        ).bind(org_id).bind(status)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<MpaSchedule> {
        let row = sqlx::query(
            "UPDATE financials.mpa_schedules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn update_schedule_amounts(&self, id: Uuid, recognized: &str, remaining: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE financials.mpa_schedules SET recognized_amount = $2, remaining_amount = $3, updated_at = now() WHERE id = $1",
        ).bind(id).bind(recognized).bind(remaining)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Schedule Lines
    async fn create_schedule_line(
        &self, schedule_id: Uuid, seq: i32, name: Option<&str>,
        start: chrono::NaiveDate, end: chrono::NaiveDate, amount: &str, pct: &str,
    ) -> AtlasResult<MpaScheduleLine> {
        let row = sqlx::query(
            r"INSERT INTO financials.mpa_schedule_lines
                (schedule_id, period_sequence, period_name, period_start_date, period_end_date,
                 amount, percentage)
            VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *",
        ).bind(schedule_id).bind(seq).bind(name).bind(start).bind(end).bind(amount).bind(pct)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line(&row))
    }

    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<MpaScheduleLine>> {
        let row = sqlx::query("SELECT * FROM financials.mpa_schedule_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule_line(&r)))
    }

    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<MpaScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM financials.mpa_schedule_lines WHERE schedule_id = $1 ORDER BY period_sequence"
        ).bind(schedule_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule_line).collect())
    }

    async fn update_schedule_line_status(&self, id: Uuid, status: &str, je_id: Option<Uuid>) -> AtlasResult<MpaScheduleLine> {
        let row = sqlx::query(
            r"UPDATE financials.mpa_schedule_lines
            SET status = $2, journal_entry_id = $3,
                recognized_at = CASE WHEN $2 = 'recognized' THEN now() ELSE recognized_at END,
                updated_at = now()
            WHERE id = $1 RETURNING *",
        ).bind(id).bind(status).bind(je_id)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line(&row))
    }

    async fn count_pending_lines(&self, schedule_id: Uuid) -> AtlasResult<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM financials.mpa_schedule_lines WHERE schedule_id = $1 AND status = 'pending'"
        ).bind(schedule_id)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.try_get("cnt").unwrap_or(0))
    }

    async fn sum_recognized_for_schedule(&self, schedule_id: Uuid) -> AtlasResult<String> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(amount::numeric), 0)::float8 as total FROM financials.mpa_schedule_lines WHERE schedule_id = $1 AND status = 'recognized'"
        ).bind(schedule_id)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let total: f64 = row.try_get("total").unwrap_or(0.0);
        Ok(format!("{total:.2}"))
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MpaDashboard> {
        let row = sqlx::query(
            r"SELECT
                COALESCE((SELECT COUNT(*) FROM financials.mpa_templates WHERE organization_id = $1), 0) as total_templates,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_templates WHERE organization_id = $1 AND status = 'active'), 0) as active_templates,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1), 0) as total_schedules,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1 AND status = 'draft'), 0) as draft_schedules,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1 AND status = 'active'), 0) as active_schedules,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1 AND status = 'completed'), 0) as completed_schedules,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1 AND status = 'cancelled'), 0) as cancelled_schedules,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedules WHERE organization_id = $1 AND status = 'on_hold'), 0) as on_hold_schedules,
                COALESCE((SELECT SUM(total_amount::numeric) FROM financials.mpa_schedules WHERE organization_id = $1), 0)::float8 as total_scheduled_amount,
                COALESCE((SELECT SUM(recognized_amount::numeric) FROM financials.mpa_schedules WHERE organization_id = $1), 0)::float8 as total_recognized_amount,
                COALESCE((SELECT SUM(remaining_amount::numeric) FROM financials.mpa_schedules WHERE organization_id = $1), 0)::float8 as total_remaining_amount,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedule_lines sl JOIN financials.mpa_schedules s ON s.id = sl.schedule_id WHERE s.organization_id = $1 AND sl.status = 'pending'), 0) as pending_lines,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedule_lines sl JOIN financials.mpa_schedules s ON s.id = sl.schedule_id WHERE s.organization_id = $1 AND sl.status = 'recognized'), 0) as recognized_lines,
                COALESCE((SELECT COUNT(*) FROM financials.mpa_schedule_lines sl JOIN financials.mpa_schedules s ON s.id = sl.schedule_id WHERE s.organization_id = $1 AND sl.status = 'reversed'), 0) as reversed_lines",
        ).bind(org_id)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(MpaDashboard {
            total_templates: row.try_get("total_templates").unwrap_or(0),
            active_templates: row.try_get("active_templates").unwrap_or(0),
            total_schedules: row.try_get("total_schedules").unwrap_or(0),
            draft_schedules: row.try_get("draft_schedules").unwrap_or(0),
            active_schedules: row.try_get("active_schedules").unwrap_or(0),
            completed_schedules: row.try_get("completed_schedules").unwrap_or(0),
            cancelled_schedules: row.try_get("cancelled_schedules").unwrap_or(0),
            on_hold_schedules: row.try_get("on_hold_schedules").unwrap_or(0),
            total_scheduled_amount: row.try_get::<f64,_>("total_scheduled_amount").map_or_else(|_| "0.00".into(), |v| format!("{v:.2}")),
            total_recognized_amount: row.try_get::<f64,_>("total_recognized_amount").map_or_else(|_| "0.00".into(), |v| format!("{v:.2}")),
            total_remaining_amount: row.try_get::<f64,_>("total_remaining_amount").map_or_else(|_| "0.00".into(), |v| format!("{v:.2}")),
            pending_lines: row.try_get("pending_lines").unwrap_or(0),
            recognized_lines: row.try_get("recognized_lines").unwrap_or(0),
            reversed_lines: row.try_get("reversed_lines").unwrap_or(0),
        })
    }
}
