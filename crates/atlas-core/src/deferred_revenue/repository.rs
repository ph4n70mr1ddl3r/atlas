//! Deferred Revenue Repository
//!
//! PostgreSQL storage for deferral templates, schedules, and schedule lines.

use atlas_shared::{
    DeferralTemplate, DeferralSchedule, DeferralScheduleLine, DeferralDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Deferred Revenue/Cost Management
#[async_trait]
pub trait DeferredRevenueRepository: Send + Sync {
    // Templates
    async fn create_template(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        deferral_type: &str, recognition_method: &str,
        deferral_account_code: &str, recognition_account_code: &str,
        contra_account_code: Option<&str>,
        default_periods: i32, period_type: &str,
        start_date_basis: &str, end_date_basis: &str,
        prorate_partial_periods: bool, auto_generate_schedule: bool, auto_post: bool,
        rounding_threshold: Option<&str>, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DeferralTemplate>;

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DeferralTemplate>>;
    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<DeferralTemplate>>;
    async fn list_templates(&self, org_id: Uuid, deferral_type: Option<&str>) -> AtlasResult<Vec<DeferralTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Schedules
    async fn create_schedule(
        &self,
        org_id: Uuid, schedule_number: &str, template_id: Uuid, template_code: Option<&str>,
        deferral_type: &str, source_type: &str, source_id: Option<Uuid>,
        source_number: Option<&str>, source_line_id: Option<Uuid>,
        description: Option<&str>,
        total_amount: &str, recognized_amount: &str, remaining_amount: &str,
        currency_code: &str,
        deferral_account_code: &str, recognition_account_code: &str,
        contra_account_code: Option<&str>,
        recognition_method: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        total_periods: i32, status: &str,
        original_journal_entry_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DeferralSchedule>;

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<DeferralSchedule>>;
    async fn get_schedule_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<DeferralSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>, deferral_type: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<DeferralSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str, hold_reason: Option<&str>) -> AtlasResult<DeferralSchedule>;
    async fn update_schedule_amounts(&self, id: Uuid, recognized_amount: &str, remaining_amount: &str, completed_periods: i32) -> AtlasResult<()>;

    // Schedule Lines
    async fn create_schedule_line(
        &self,
        org_id: Uuid, schedule_id: Uuid, line_number: i32,
        period_name: Option<&str>, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, days_in_period: i32,
        amount: &str, status: &str,
    ) -> AtlasResult<DeferralScheduleLine>;

    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<DeferralScheduleLine>>;
    async fn get_pending_lines(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<DeferralScheduleLine>>;
    async fn update_line_status(&self, id: Uuid, status: &str, recognized_amount: &str, recognition_date: Option<chrono::NaiveDate>, journal_entry_id: Option<Uuid>) -> AtlasResult<DeferralScheduleLine>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<DeferralDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresDeferredRevenueRepository {
    pool: PgPool,
}

impl PostgresDeferredRevenueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

macro_rules! row_to_template {
    ($row:expr) => {{
        DeferralTemplate {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            deferral_type: $row.get("deferral_type"),
            recognition_method: $row.get("recognition_method"),
            deferral_account_code: $row.get("deferral_account_code"),
            recognition_account_code: $row.get("recognition_account_code"),
            contra_account_code: $row.get("contra_account_code"),
            default_periods: $row.get("default_periods"),
            period_type: $row.get("period_type"),
            start_date_basis: $row.get("start_date_basis"),
            end_date_basis: $row.get("end_date_basis"),
            prorate_partial_periods: $row.get("prorate_partial_periods"),
            auto_generate_schedule: $row.get("auto_generate_schedule"),
            auto_post: $row.get("auto_post"),
            rounding_threshold: $row.try_get::<f64, _>("rounding_threshold").ok().map(|v| format!("{:.2}", v)),
            currency_code: $row.get("currency_code"),
            is_active: $row.get("is_active"),
            effective_from: $row.get("effective_from"),
            effective_to: $row.get("effective_to"),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_schedule {
    ($row:expr) => {{
        DeferralSchedule {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            schedule_number: $row.get("schedule_number"),
            template_id: $row.get("template_id"),
            template_code: $row.get("template_code"),
            deferral_type: $row.get("deferral_type"),
            source_type: $row.get("source_type"),
            source_id: $row.get("source_id"),
            source_number: $row.get("source_number"),
            source_line_id: $row.get("source_line_id"),
            description: $row.get("description"),
            total_amount: $row.try_get::<f64, _>("total_amount").map(|v| format!("{:.2}", v)).unwrap_or_else(|_| "0.00".to_string()),
            recognized_amount: $row.try_get::<f64, _>("recognized_amount").map(|v| format!("{:.2}", v)).unwrap_or_else(|_| "0.00".to_string()),
            remaining_amount: $row.try_get::<f64, _>("remaining_amount").map(|v| format!("{:.2}", v)).unwrap_or_else(|_| "0.00".to_string()),
            currency_code: $row.get("currency_code"),
            deferral_account_code: $row.get("deferral_account_code"),
            recognition_account_code: $row.get("recognition_account_code"),
            contra_account_code: $row.get("contra_account_code"),
            recognition_method: $row.get("recognition_method"),
            start_date: $row.get("start_date"),
            end_date: $row.get("end_date"),
            total_periods: $row.get("total_periods"),
            completed_periods: $row.get("completed_periods"),
            status: $row.get("status"),
            hold_reason: $row.get("hold_reason"),
            original_journal_entry_id: $row.get("original_journal_entry_id"),
            last_recognition_date: $row.get("last_recognition_date"),
            completion_date: $row.get("completion_date"),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_schedule_line {
    ($row:expr) => {{
        DeferralScheduleLine {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            schedule_id: $row.get("schedule_id"),
            line_number: $row.get("line_number"),
            period_name: $row.get("period_name"),
            period_start_date: $row.get("period_start_date"),
            period_end_date: $row.get("period_end_date"),
            days_in_period: $row.get("days_in_period"),
            amount: $row.try_get::<f64, _>("amount").map(|v| format!("{:.2}", v)).unwrap_or_else(|_| "0.00".to_string()),
            recognized_amount: $row.try_get::<f64, _>("recognized_amount").map(|v| format!("{:.2}", v)).unwrap_or_else(|_| "0.00".to_string()),
            status: $row.get("status"),
            recognition_date: $row.get("recognition_date"),
            journal_entry_id: $row.get("journal_entry_id"),
            reversal_journal_entry_id: $row.get("reversal_journal_entry_id"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

#[async_trait]
impl DeferredRevenueRepository for PostgresDeferredRevenueRepository {
    async fn create_template(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        deferral_type: &str, recognition_method: &str,
        deferral_account_code: &str, recognition_account_code: &str,
        contra_account_code: Option<&str>,
        default_periods: i32, period_type: &str,
        start_date_basis: &str, end_date_basis: &str,
        prorate_partial_periods: bool, auto_generate_schedule: bool, auto_post: bool,
        rounding_threshold: Option<&str>, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DeferralTemplate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.deferral_templates
                (organization_id, code, name, description, deferral_type, recognition_method,
                 deferral_account_code, recognition_account_code, contra_account_code,
                 default_periods, period_type, start_date_basis, end_date_basis,
                 prorate_partial_periods, auto_generate_schedule, auto_post,
                 rounding_threshold, currency_code, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(deferral_type).bind(recognition_method)
        .bind(deferral_account_code).bind(recognition_account_code).bind(contra_account_code)
        .bind(default_periods).bind(period_type).bind(start_date_basis).bind(end_date_basis)
        .bind(prorate_partial_periods).bind(auto_generate_schedule).bind(auto_post)
        .bind(rounding_threshold.and_then(|v| v.parse::<f64>().ok()))
        .bind(currency_code).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_template!(row))
    }

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DeferralTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.deferral_templates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template!(r)))
    }

    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<DeferralTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.deferral_templates WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template!(r)))
    }

    async fn list_templates(&self, org_id: Uuid, deferral_type: Option<&str>) -> AtlasResult<Vec<DeferralTemplate>> {
        let rows = if let Some(dt) = deferral_type {
            sqlx::query(
                "SELECT * FROM _atlas.deferral_templates WHERE organization_id = $1 AND deferral_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(dt)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.deferral_templates WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_template!(r)).collect())
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.deferral_templates SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_schedule(
        &self,
        org_id: Uuid, schedule_number: &str, template_id: Uuid, template_code: Option<&str>,
        deferral_type: &str, source_type: &str, source_id: Option<Uuid>,
        source_number: Option<&str>, source_line_id: Option<Uuid>,
        description: Option<&str>,
        total_amount: &str, recognized_amount: &str, remaining_amount: &str,
        currency_code: &str,
        deferral_account_code: &str, recognition_account_code: &str,
        contra_account_code: Option<&str>,
        recognition_method: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        total_periods: i32, status: &str,
        original_journal_entry_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DeferralSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.deferral_schedules
                (organization_id, schedule_number, template_id, template_code,
                 deferral_type, source_type, source_id, source_number, source_line_id,
                 description, total_amount, recognized_amount, remaining_amount,
                 currency_code, deferral_account_code, recognition_account_code, contra_account_code,
                 recognition_method, start_date, end_date, total_periods, status,
                 original_journal_entry_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_number).bind(template_id).bind(template_code)
        .bind(deferral_type).bind(source_type).bind(source_id).bind(source_number).bind(source_line_id)
        .bind(description)
        .bind(total_amount.parse::<f64>().unwrap_or(0.0))
        .bind(recognized_amount.parse::<f64>().unwrap_or(0.0))
        .bind(remaining_amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code)
        .bind(deferral_account_code).bind(recognition_account_code).bind(contra_account_code)
        .bind(recognition_method)
        .bind(start_date).bind(end_date).bind(total_periods).bind(status)
        .bind(original_journal_entry_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_schedule!(row))
    }

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<DeferralSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.deferral_schedules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule!(r)))
    }

    async fn get_schedule_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<DeferralSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.deferral_schedules WHERE organization_id = $1 AND schedule_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule!(r)))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>, deferral_type: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<DeferralSchedule>> {
        let mut query = String::from("SELECT * FROM _atlas.deferral_schedules WHERE organization_id = $1");
        let mut param_idx = 2;
        if status.is_some() { query.push_str(&format!(" AND status = ${}", param_idx)); param_idx += 1; }
        if deferral_type.is_some() { query.push_str(&format!(" AND deferral_type = ${}", param_idx)); param_idx += 1; }
        if source_type.is_some() { query.push_str(&format!(" AND source_type = ${}", param_idx)); }
        query.push_str(" ORDER BY start_date DESC, created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(d) = deferral_type { q = q.bind(d); }
        if let Some(s) = source_type { q = q.bind(s); }

        let rows = q.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_schedule!(r)).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str, hold_reason: Option<&str>) -> AtlasResult<DeferralSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.deferral_schedules
            SET status = $1, hold_reason = $2, updated_at = now(),
                completion_date = CASE WHEN $1 = 'completed' THEN now() ELSE completion_date END
            WHERE id = $3
            RETURNING *"#,
        )
        .bind(status).bind(hold_reason).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule!(row))
    }

    async fn update_schedule_amounts(&self, id: Uuid, recognized_amount: &str, remaining_amount: &str, completed_periods: i32) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.deferral_schedules
            SET recognized_amount = $1, remaining_amount = $2, completed_periods = $3,
                last_recognition_date = CURRENT_DATE, updated_at = now()
            WHERE id = $4"#,
        )
        .bind(recognized_amount.parse::<f64>().unwrap_or(0.0))
        .bind(remaining_amount.parse::<f64>().unwrap_or(0.0))
        .bind(completed_periods).bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_schedule_line(
        &self,
        org_id: Uuid, schedule_id: Uuid, line_number: i32,
        period_name: Option<&str>, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, days_in_period: i32,
        amount: &str, status: &str,
    ) -> AtlasResult<DeferralScheduleLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.deferral_schedule_lines
                (organization_id, schedule_id, line_number, period_name,
                 period_start_date, period_end_date, days_in_period,
                 amount, recognized_amount, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(line_number).bind(period_name)
        .bind(period_start_date).bind(period_end_date).bind(days_in_period)
        .bind(amount.parse::<f64>().unwrap_or(0.0)).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line!(row))
    }

    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<DeferralScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.deferral_schedule_lines WHERE schedule_id = $1 ORDER BY line_number"
        )
        .bind(schedule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_schedule_line!(r)).collect())
    }

    async fn get_pending_lines(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<DeferralScheduleLine>> {
        let rows = sqlx::query(
            r#"SELECT l.* FROM _atlas.deferral_schedule_lines l
            JOIN _atlas.deferral_schedules s ON l.schedule_id = s.id
            WHERE s.organization_id = $1 AND l.status = 'pending'
              AND l.period_end_date <= $2 AND s.status = 'active'
            ORDER BY l.period_start_date"#,
        )
        .bind(org_id).bind(as_of_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_schedule_line!(r)).collect())
    }

    async fn update_line_status(&self, id: Uuid, status: &str, recognized_amount: &str, recognition_date: Option<chrono::NaiveDate>, journal_entry_id: Option<Uuid>) -> AtlasResult<DeferralScheduleLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.deferral_schedule_lines
            SET status = $1, recognized_amount = $2, recognition_date = $3, journal_entry_id = $4, updated_at = now()
            WHERE id = $5
            RETURNING *"#,
        )
        .bind(status)
        .bind(recognized_amount.parse::<f64>().unwrap_or(0.0))
        .bind(recognition_date).bind(journal_entry_id).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line!(row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<DeferralDashboardSummary> {
        let schedules = sqlx::query(
            "SELECT * FROM _atlas.deferral_schedules WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut total_schedules = 0i32;
        let mut active_schedules = 0i32;
        let mut completed_schedules = 0i32;
        let mut on_hold_schedules = 0i32;
        let mut total_deferred = 0.0f64;
        let mut total_recognized = 0.0f64;
        let mut total_remaining = 0.0f64;
        let mut revenue_deferred = 0.0f64;
        let mut cost_deferred = 0.0f64;
        let mut pending_count = 0i32;
        let mut pending_amount = 0.0f64;

        for s in &schedules {
            total_schedules += 1;
            let remaining: f64 = s.try_get::<f64, _>("remaining_amount").unwrap_or(0.0);
            let total: f64 = s.try_get::<f64, _>("total_amount").unwrap_or(0.0);
            let recognized: f64 = s.try_get::<f64, _>("recognized_amount").unwrap_or(0.0);

            total_deferred += total;
            total_recognized += recognized;
            total_remaining += remaining;

            let status: String = s.get("status");
            match status.as_str() {
                "active" => active_schedules += 1,
                "completed" => completed_schedules += 1,
                "on_hold" => on_hold_schedules += 1,
                _ => {}
            }

            let dtype: String = s.get("deferral_type");
            if dtype == "revenue" {
                revenue_deferred += remaining;
            } else {
                cost_deferred += remaining;
            }

            if status == "active" && remaining > 0.0 {
                pending_count += 1;
                pending_amount += remaining;
            }
        }

        Ok(DeferralDashboardSummary {
            total_schedules,
            active_schedules,
            completed_schedules,
            on_hold_schedules,
            total_deferred_amount: format!("{:.2}", total_deferred),
            total_recognized_amount: format!("{:.2}", total_recognized),
            total_remaining_amount: format!("{:.2}", total_remaining),
            pending_recognition_count: pending_count,
            pending_recognition_amount: format!("{:.2}", pending_amount),
            revenue_deferred: format!("{:.2}", revenue_deferred),
            cost_deferred: format!("{:.2}", cost_deferred),
        })
    }
}
