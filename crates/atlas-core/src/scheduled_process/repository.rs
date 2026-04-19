//! Scheduled Process Repository
//!
//! PostgreSQL storage for process templates, process instances,
//! recurrence schedules, and execution logs.

use atlas_shared::{
    ScheduledProcess, ScheduledProcessTemplate, ScheduledProcessRecurrence,
    ScheduledProcessLog, ScheduledProcessDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for scheduled process data storage
#[async_trait]
pub trait ScheduledProcessRepository: Send + Sync {
    // Templates
    #[allow(clippy::too_many_arguments)]
    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        process_type: &str, executor_type: &str,
        executor_config: serde_json::Value, parameters: serde_json::Value,
        default_parameters: serde_json::Value, timeout_minutes: i32,
        max_retries: i32, retry_delay_minutes: i32, requires_approval: bool,
        approval_chain_id: Option<Uuid>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessTemplate>;

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ScheduledProcessTemplate>>;
    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessTemplate>>;
    async fn list_templates(&self, org_id: Uuid, process_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<ScheduledProcessTemplate>>;
    async fn update_template_status(&self, id: Uuid, is_active: bool) -> AtlasResult<ScheduledProcessTemplate>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Processes
    #[allow(clippy::too_many_arguments)]
    async fn create_process(
        &self, org_id: Uuid, template_id: Option<Uuid>, template_code: Option<&str>,
        process_name: &str, process_type: &str, description: Option<&str>,
        status: &str, priority: &str, scheduled_start_at: Option<DateTime<Utc>>,
        timeout_minutes: i32, max_retries: i32,
        parameters: serde_json::Value, submitted_by: Uuid,
    ) -> AtlasResult<ScheduledProcess>;

    async fn get_process(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcess>>;
    async fn list_processes(
        &self, org_id: Uuid, status: Option<&str>, submitted_by: Option<Uuid>,
        process_type: Option<&str>, limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcess>>;

    #[allow(clippy::too_many_arguments)]
    async fn update_process_status(
        &self, id: Uuid, status: &str, started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>, cancelled_at: Option<DateTime<Utc>>,
        cancelled_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcess>;

    #[allow(clippy::too_many_arguments)]
    async fn complete_process(
        &self, id: Uuid, status: &str, completed_at: Option<DateTime<Utc>>,
        result_summary: Option<&str>, output_file_url: Option<&str>,
        log_output: Option<&str>, progress_percent: Option<i32>,
    ) -> AtlasResult<ScheduledProcess>;

    #[allow(clippy::too_many_arguments)]
    async fn fail_process(
        &self, id: Uuid, status: &str, completed_at: Option<DateTime<Utc>>,
        error_message: Option<&str>, log_output: Option<&str>,
    ) -> AtlasResult<ScheduledProcess>;

    #[allow(clippy::too_many_arguments)]
    async fn cancel_process(
        &self, id: Uuid, status: &str, cancelled_at: Option<DateTime<Utc>>,
        cancelled_by: Option<Uuid>, cancel_reason: Option<&str>,
    ) -> AtlasResult<ScheduledProcess>;

    async fn retry_process(&self, id: Uuid, retry_count: i32) -> AtlasResult<ScheduledProcess>;
    async fn update_progress(&self, id: Uuid, progress_percent: i32) -> AtlasResult<ScheduledProcess>;
    async fn update_heartbeat(&self, id: Uuid) -> AtlasResult<ScheduledProcess>;
    async fn update_process_recurrence(&self, id: Uuid, recurrence_id: Uuid) -> AtlasResult<()>;
    async fn find_timed_out_processes(&self) -> AtlasResult<Vec<ScheduledProcess>>;

    // Recurrences
    #[allow(clippy::too_many_arguments)]
    async fn create_recurrence(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        template_id: Uuid, template_code: Option<&str>,
        parameters: serde_json::Value, recurrence_type: &str,
        recurrence_config: serde_json::Value,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        next_run_at: Option<DateTime<Utc>>, max_runs: Option<i32>,
        submitted_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessRecurrence>;

    async fn get_recurrence(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessRecurrence>>;
    async fn list_recurrences(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<ScheduledProcessRecurrence>>;
    async fn update_recurrence_status(&self, id: Uuid, is_active: bool) -> AtlasResult<ScheduledProcessRecurrence>;
    async fn update_recurrence_after_run(
        &self, id: Uuid, last_run_at: Option<DateTime<Utc>>,
        next_run_at: Option<DateTime<Utc>>, run_count: i32,
    ) -> AtlasResult<()>;
    async fn delete_recurrence(&self, id: Uuid) -> AtlasResult<()>;
    async fn find_due_recurrences(&self, now: DateTime<Utc>) -> AtlasResult<Vec<ScheduledProcessRecurrence>>;

    // Logs
    async fn create_log(
        &self, org_id: Uuid, process_id: Uuid, log_level: &str,
        message: &str, details: Option<serde_json::Value>,
        step_name: Option<&str>, duration_ms: Option<i32>,
    ) -> AtlasResult<ScheduledProcessLog>;

    async fn list_logs(
        &self, process_id: Uuid, log_level: Option<&str>, limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcessLog>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ScheduledProcessDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresScheduledProcessRepository {
    pool: PgPool,
}

impl PostgresScheduledProcessRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> ScheduledProcessTemplate {
    ScheduledProcessTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        process_type: row.get("process_type"),
        executor_type: row.get("executor_type"),
        executor_config: row.get("executor_config"),
        parameters: row.get("parameters"),
        default_parameters: row.get("default_parameters"),
        timeout_minutes: row.get("timeout_minutes"),
        max_retries: row.get("max_retries"),
        retry_delay_minutes: row.get("retry_delay_minutes"),
        requires_approval: row.get("requires_approval"),
        approval_chain_id: row.get("approval_chain_id"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_process(row: &sqlx::postgres::PgRow) -> ScheduledProcess {
    ScheduledProcess {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        template_code: row.get("template_code"),
        process_name: row.get("process_name"),
        process_type: row.get("process_type"),
        description: row.get("description"),
        status: row.get("status"),
        priority: row.get("priority"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        scheduled_start_at: row.get("scheduled_start_at"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        cancelled_at: row.get("cancelled_at"),
        cancelled_by: row.get("cancelled_by"),
        cancel_reason: row.get("cancel_reason"),
        last_heartbeat_at: row.get("last_heartbeat_at"),
        retry_count: row.get("retry_count"),
        max_retries: row.get("max_retries"),
        timeout_minutes: row.get("timeout_minutes"),
        progress_percent: row.get("progress_percent"),
        parameters: row.get("parameters"),
        result_summary: row.get("result_summary"),
        output_file_url: row.get("output_file_url"),
        output_format: row.get("output_format"),
        log_output: row.get("log_output"),
        error_message: row.get("error_message"),
        parent_process_id: row.get("parent_process_id"),
        recurrence_id: row.get("recurrence_id"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_recurrence(row: &sqlx::postgres::PgRow) -> ScheduledProcessRecurrence {
    ScheduledProcessRecurrence {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        name: row.get("name"),
        description: row.get("description"),
        template_id: row.get("template_id"),
        template_code: row.get("template_code"),
        parameters: row.get("parameters"),
        recurrence_type: row.get("recurrence_type"),
        recurrence_config: row.get("recurrence_config"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        next_run_at: row.get("next_run_at"),
        last_run_at: row.get("last_run_at"),
        run_count: row.get("run_count"),
        max_runs: row.get("max_runs"),
        is_active: row.get("is_active"),
        submitted_by: row.get("submitted_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_log(row: &sqlx::postgres::PgRow) -> ScheduledProcessLog {
    ScheduledProcessLog {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        process_id: row.get("process_id"),
        log_level: row.get("log_level"),
        message: row.get("message"),
        details: row.get("details"),
        step_name: row.get("step_name"),
        duration_ms: row.get("duration_ms"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl ScheduledProcessRepository for PostgresScheduledProcessRepository {
    // ---- Templates ----

    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        process_type: &str, executor_type: &str,
        executor_config: serde_json::Value, parameters: serde_json::Value,
        default_parameters: serde_json::Value, timeout_minutes: i32,
        max_retries: i32, retry_delay_minutes: i32, requires_approval: bool,
        approval_chain_id: Option<Uuid>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessTemplate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.scheduled_process_templates
                (organization_id, code, name, description, process_type, executor_type,
                 executor_config, parameters, default_parameters, timeout_minutes,
                 max_retries, retry_delay_minutes, requires_approval, approval_chain_id,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(process_type).bind(executor_type)
        .bind(&executor_config).bind(&parameters).bind(&default_parameters)
        .bind(timeout_minutes).bind(max_retries).bind(retry_delay_minutes)
        .bind(requires_approval).bind(approval_chain_id)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ScheduledProcessTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.scheduled_process_templates WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.scheduled_process_templates WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid, process_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<ScheduledProcessTemplate>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_process_templates
            WHERE organization_id=$1
              AND ($2::text IS NULL OR process_type=$2)
              AND ($3::bool IS NULL OR is_active=$3)
            ORDER BY code"#,
        )
        .bind(org_id).bind(process_type).bind(is_active)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_template(r)).collect())
    }

    async fn update_template_status(&self, id: Uuid, is_active: bool) -> AtlasResult<ScheduledProcessTemplate> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_process_templates SET is_active=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.scheduled_process_templates WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ---- Processes ----

    async fn create_process(
        &self, org_id: Uuid, template_id: Option<Uuid>, template_code: Option<&str>,
        process_name: &str, process_type: &str, description: Option<&str>,
        status: &str, priority: &str, scheduled_start_at: Option<DateTime<Utc>>,
        timeout_minutes: i32, max_retries: i32,
        parameters: serde_json::Value, submitted_by: Uuid,
    ) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.scheduled_processes
                (organization_id, template_id, template_code, process_name, process_type,
                 description, status, priority, scheduled_start_at, timeout_minutes,
                 max_retries, parameters, submitted_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(template_id).bind(template_code)
        .bind(process_name).bind(process_type).bind(description)
        .bind(status).bind(priority).bind(scheduled_start_at)
        .bind(timeout_minutes).bind(max_retries)
        .bind(&parameters).bind(submitted_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn get_process(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcess>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.scheduled_processes WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_process(&r)))
    }

    async fn list_processes(
        &self, org_id: Uuid, status: Option<&str>, submitted_by: Option<Uuid>,
        process_type: Option<&str>, limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcess>> {
        let limit_val = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_processes
            WHERE organization_id=$1
              AND ($2::text IS NULL OR status=$2)
              AND ($3::uuid IS NULL OR submitted_by=$3)
              AND ($4::text IS NULL OR process_type=$4)
            ORDER BY submitted_at DESC LIMIT $5"#,
        )
        .bind(org_id).bind(status).bind(submitted_by).bind(process_type).bind(limit_val)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_process(r)).collect())
    }

    async fn update_process_status(
        &self, id: Uuid, status: &str, started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>, cancelled_at: Option<DateTime<Utc>>,
        cancelled_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET status=$2, started_at=COALESCE($3, started_at),
                completed_at=COALESCE($4, completed_at),
                cancelled_at=COALESCE($5, cancelled_at),
                cancelled_by=COALESCE($6, cancelled_by),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(started_at).bind(completed_at)
        .bind(cancelled_at).bind(cancelled_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn complete_process(
        &self, id: Uuid, status: &str, completed_at: Option<DateTime<Utc>>,
        result_summary: Option<&str>, output_file_url: Option<&str>,
        log_output: Option<&str>, progress_percent: Option<i32>,
    ) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET status=$2, completed_at=COALESCE($3, now()),
                result_summary=$4, output_file_url=$5, log_output=$6,
                progress_percent=COALESCE($7, progress_percent),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(completed_at)
        .bind(result_summary).bind(output_file_url).bind(log_output)
        .bind(progress_percent)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn fail_process(
        &self, id: Uuid, status: &str, completed_at: Option<DateTime<Utc>>,
        error_message: Option<&str>, log_output: Option<&str>,
    ) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET status=$2, completed_at=COALESCE($3, now()),
                error_message=$4, log_output=$5, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(completed_at)
        .bind(error_message).bind(log_output)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn cancel_process(
        &self, id: Uuid, status: &str, cancelled_at: Option<DateTime<Utc>>,
        cancelled_by: Option<Uuid>, cancel_reason: Option<&str>,
    ) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET status=$2, cancelled_at=COALESCE($3, now()),
                cancelled_by=$4, cancel_reason=$5, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(cancelled_at)
        .bind(cancelled_by).bind(cancel_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn retry_process(&self, id: Uuid, retry_count: i32) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET status='pending', retry_count=$2,
                started_at=NULL, completed_at=NULL,
                error_message=NULL, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(retry_count)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn update_progress(&self, id: Uuid, progress_percent: i32) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET progress_percent=$2, last_heartbeat_at=now(), updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(progress_percent)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn update_heartbeat(&self, id: Uuid) -> AtlasResult<ScheduledProcess> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_processes
            SET last_heartbeat_at=now(), updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_process(&row))
    }

    async fn update_process_recurrence(&self, id: Uuid, recurrence_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.scheduled_processes SET recurrence_id=$2, updated_at=now() WHERE id=$1"
        )
        .bind(id).bind(recurrence_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_timed_out_processes(&self) -> AtlasResult<Vec<ScheduledProcess>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_processes
            WHERE status='running'
              AND started_at IS NOT NULL
              AND started_at < now() - (timeout_minutes || ' minutes')::interval
            "#,
        )
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_process(r)).collect())
    }

    // ---- Recurrences ----

    async fn create_recurrence(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        template_id: Uuid, template_code: Option<&str>,
        parameters: serde_json::Value, recurrence_type: &str,
        recurrence_config: serde_json::Value,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        next_run_at: Option<DateTime<Utc>>, max_runs: Option<i32>,
        submitted_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledProcessRecurrence> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.scheduled_process_recurrences
                (organization_id, name, description, template_id, template_code,
                 parameters, recurrence_type, recurrence_config, start_date, end_date,
                 next_run_at, max_runs, submitted_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(name).bind(description)
        .bind(template_id).bind(template_code)
        .bind(&parameters).bind(recurrence_type).bind(&recurrence_config)
        .bind(start_date).bind(end_date)
        .bind(next_run_at).bind(max_runs).bind(submitted_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_recurrence(&row))
    }

    async fn get_recurrence(&self, id: Uuid) -> AtlasResult<Option<ScheduledProcessRecurrence>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.scheduled_process_recurrences WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_recurrence(&r)))
    }

    async fn list_recurrences(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<ScheduledProcessRecurrence>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_process_recurrences
            WHERE organization_id=$1 AND ($2::bool IS NULL OR is_active=$2)
            ORDER BY name"#,
        )
        .bind(org_id).bind(is_active)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_recurrence(r)).collect())
    }

    async fn update_recurrence_status(&self, id: Uuid, is_active: bool) -> AtlasResult<ScheduledProcessRecurrence> {
        let row = sqlx::query(
            r#"UPDATE _atlas.scheduled_process_recurrences SET is_active=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_recurrence(&row))
    }

    async fn update_recurrence_after_run(
        &self, id: Uuid, last_run_at: Option<DateTime<Utc>>,
        next_run_at: Option<DateTime<Utc>>, run_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.scheduled_process_recurrences
            SET last_run_at=$2, next_run_at=$3, run_count=$4, updated_at=now()
            WHERE id=$1"#,
        )
        .bind(id).bind(last_run_at).bind(next_run_at).bind(run_count)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_recurrence(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.scheduled_process_recurrences WHERE id=$1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_due_recurrences(&self, now: DateTime<Utc>) -> AtlasResult<Vec<ScheduledProcessRecurrence>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_process_recurrences
            WHERE is_active=true
              AND next_run_at IS NOT NULL
              AND next_run_at <= $1
              AND (end_date IS NULL OR end_date >= CURRENT_DATE)
            ORDER BY next_run_at"#,
        )
        .bind(now)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_recurrence(r)).collect())
    }

    // ---- Logs ----

    async fn create_log(
        &self, org_id: Uuid, process_id: Uuid, log_level: &str,
        message: &str, details: Option<serde_json::Value>,
        step_name: Option<&str>, duration_ms: Option<i32>,
    ) -> AtlasResult<ScheduledProcessLog> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.scheduled_process_logs
                (organization_id, process_id, log_level, message, details, step_name, duration_ms)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *"#,
        )
        .bind(org_id).bind(process_id).bind(log_level)
        .bind(message).bind(details).bind(step_name).bind(duration_ms)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_log(&row))
    }

    async fn list_logs(
        &self, process_id: Uuid, log_level: Option<&str>, limit: Option<i32>,
    ) -> AtlasResult<Vec<ScheduledProcessLog>> {
        let limit_val = limit.unwrap_or(200);
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_process_logs
            WHERE process_id=$1 AND ($2::text IS NULL OR log_level=$2)
            ORDER BY created_at ASC LIMIT $3"#,
        )
        .bind(process_id).bind(log_level).bind(limit_val)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_log(r)).collect())
    }

    // ---- Dashboard ----

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ScheduledProcessDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'scheduled') as scheduled,
                COUNT(*) FILTER (WHERE status = 'running') as running,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled
            FROM _atlas.scheduled_processes WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let pending: i64 = row.try_get("pending").unwrap_or(0);
        let running: i64 = row.try_get("running").unwrap_or(0);
        let completed: i64 = row.try_get("completed").unwrap_or(0);
        let failed: i64 = row.try_get("failed").unwrap_or(0);
        let cancelled: i64 = row.try_get("cancelled").unwrap_or(0);
        let scheduled: i64 = row.try_get("scheduled").unwrap_or(0);

        let active_recurrences: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.scheduled_process_recurrences WHERE organization_id=$1 AND is_active=true"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_rows = sqlx::query(
            r#"SELECT * FROM _atlas.scheduled_processes
            WHERE organization_id=$1
            ORDER BY submitted_at DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let type_rows = sqlx::query(
            r#"SELECT process_type, COUNT(*) as count FROM _atlas.scheduled_processes
            WHERE organization_id=$1 GROUP BY process_type"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_type = serde_json::Map::new();
        for r in &type_rows {
            let pt: String = r.get("process_type");
            let count: i64 = r.get("count");
            by_type.insert(pt, serde_json::Value::Number(count.into()));
        }

        Ok(ScheduledProcessDashboardSummary {
            total_processes: total as i32,
            pending_processes: pending as i32,
            running_processes: running as i32,
            completed_processes: completed as i32,
            failed_processes: failed as i32,
            cancelled_processes: cancelled as i32,
            scheduled_processes: scheduled as i32,
            active_recurrences: active_recurrences as i32,
            recent_processes: recent_rows.iter().map(|r| row_to_process(r)).collect(),
            processes_by_type: serde_json::Value::Object(by_type),
        })
    }
}
