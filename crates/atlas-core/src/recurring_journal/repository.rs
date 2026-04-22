//! Recurring Journal Repository
//!
//! PostgreSQL storage for recurring journal schedules, template lines,
//! generations, and generated lines.

use atlas_shared::{
    RecurringJournalSchedule, RecurringJournalScheduleLine,
    RecurringJournalGeneration, RecurringJournalGenerationLine,
    RecurringJournalDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for recurring journal data storage
#[async_trait]
pub trait RecurringJournalRepository: Send + Sync {
    // Schedules
    async fn create_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str, description: Option<&str>,
        recurrence_type: &str, journal_type: &str, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        next_generation_date: Option<chrono::NaiveDate>,
        incremental_percent: Option<&str>, auto_post: bool,
        reversal_method: Option<&str>, ledger_id: Option<Uuid>,
        journal_category: Option<&str>, reference_template: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalSchedule>;
    async fn get_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<RecurringJournalSchedule>>;
    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RecurringJournalSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<RecurringJournalSchedule>;
    async fn update_schedule_generation_info(&self, id: Uuid, last_gen: chrono::NaiveDate, next_gen: Option<chrono::NaiveDate>, total: i32) -> AtlasResult<RecurringJournalSchedule>;
    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()>;

    // Schedule Lines
    async fn create_schedule_line(
        &self, org_id: Uuid, schedule_id: Uuid, line_number: i32, line_type: &str,
        account_code: &str, account_name: Option<&str>, description: Option<&str>,
        amount: &str, currency_code: &str, tax_code: Option<&str>,
        cost_center: Option<&str>, department_id: Option<Uuid>, project_id: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalScheduleLine>;
    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalScheduleLine>>;
    async fn delete_schedule_line(&self, id: Uuid) -> AtlasResult<()>;

    // Generations
    async fn create_generation(
        &self, org_id: Uuid, schedule_id: Uuid, generation_number: i32,
        generation_date: chrono::NaiveDate, period_name: Option<&str>,
        total_debit: &str, total_credit: &str, line_count: i32,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalGeneration>;
    async fn get_generation(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalGeneration>>;
    async fn list_generations(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalGeneration>>;
    async fn update_generation_status(&self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>, reversed_at: Option<chrono::DateTime<chrono::Utc>>, reversal_entry_id: Option<Uuid>) -> AtlasResult<RecurringJournalGeneration>;
    async fn get_latest_generation_number(&self, schedule_id: Uuid) -> AtlasResult<i32>;

    // Generation Lines
    async fn create_generation_line(
        &self, org_id: Uuid, generation_id: Uuid, schedule_line_id: Option<Uuid>,
        line_number: i32, line_type: &str, account_code: &str, account_name: Option<&str>,
        description: Option<&str>, amount: &str, currency_code: &str,
        tax_code: Option<&str>, cost_center: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalGenerationLine>;
    async fn list_generation_lines(&self, generation_id: Uuid) -> AtlasResult<Vec<RecurringJournalGenerationLine>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<RecurringJournalDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresRecurringJournalRepository {
    pool: PgPool,
}

impl PostgresRecurringJournalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_schedule(row: &sqlx::postgres::PgRow) -> RecurringJournalSchedule {
    
    use serde_json::Value;
    RecurringJournalSchedule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_number: row.get("schedule_number"),
        name: row.get("name"),
        description: row.get("description"),
        recurrence_type: row.get("recurrence_type"),
        journal_type: row.get("journal_type"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        last_generation_date: row.get("last_generation_date"),
        next_generation_date: row.get("next_generation_date"),
        total_generations: row.get("total_generations"),
        incremental_percent: row.try_get("incremental_percent").unwrap_or(None),
        auto_post: row.get("auto_post"),
        reversal_method: row.get("reversal_method"),
        ledger_id: row.get("ledger_id"),
        journal_category: row.get("journal_category"),
        reference_template: row.get("reference_template"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_schedule_line(row: &sqlx::postgres::PgRow) -> RecurringJournalScheduleLine {
    use serde_json::Value;
    RecurringJournalScheduleLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_id: row.get("schedule_id"),
        line_number: row.get("line_number"),
        line_type: row.get("line_type"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        amount: row.try_get("amount").unwrap_or(Value::Null).to_string(),
        currency_code: row.get("currency_code"),
        tax_code: row.get("tax_code"),
        cost_center: row.get("cost_center"),
        department_id: row.get("department_id"),
        project_id: row.get("project_id"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_generation(row: &sqlx::postgres::PgRow) -> RecurringJournalGeneration {
    use serde_json::Value;
    RecurringJournalGeneration {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_id: row.get("schedule_id"),
        generation_number: row.get("generation_number"),
        journal_entry_id: row.get("journal_entry_id"),
        journal_entry_number: row.get("journal_entry_number"),
        generation_date: row.get("generation_date"),
        period_name: row.get("period_name"),
        total_debit: row.try_get("total_debit").unwrap_or(Value::Null).to_string(),
        total_credit: row.try_get("total_credit").unwrap_or(Value::Null).to_string(),
        line_count: row.get("line_count"),
        status: row.get("status"),
        reversal_entry_id: row.get("reversal_entry_id"),
        reversed_at: row.get("reversed_at"),
        posted_at: row.get("posted_at"),
        generated_by: row.get("generated_by"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_generation_line(row: &sqlx::postgres::PgRow) -> RecurringJournalGenerationLine {
    use serde_json::Value;
    RecurringJournalGenerationLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        generation_id: row.get("generation_id"),
        schedule_line_id: row.get("schedule_line_id"),
        line_number: row.get("line_number"),
        line_type: row.get("line_type"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        amount: row.try_get("amount").unwrap_or(Value::Null).to_string(),
        currency_code: row.get("currency_code"),
        tax_code: row.get("tax_code"),
        cost_center: row.get("cost_center"),
        department_id: row.get("department_id"),
        project_id: row.get("project_id"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl RecurringJournalRepository for PostgresRecurringJournalRepository {
    async fn create_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str, description: Option<&str>,
        recurrence_type: &str, journal_type: &str, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        next_generation_date: Option<chrono::NaiveDate>,
        incremental_percent: Option<&str>, auto_post: bool,
        reversal_method: Option<&str>, ledger_id: Option<Uuid>,
        journal_category: Option<&str>, reference_template: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.recurring_journal_schedules
                (organization_id, schedule_number, name, description, recurrence_type,
                 journal_type, currency_code, status, effective_from, effective_to,
                 next_generation_date, incremental_percent, auto_post, reversal_method,
                 ledger_id, journal_category, reference_template, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,'draft',$8,$9,$10,$11::numeric,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_number).bind(name).bind(description)
        .bind(recurrence_type).bind(journal_type).bind(currency_code)
        .bind(effective_from).bind(effective_to)
        .bind(next_generation_date)
        .bind(incremental_percent).bind(auto_post).bind(reversal_method)
        .bind(ledger_id).bind(journal_category).bind(reference_template)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn get_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<RecurringJournalSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_schedules WHERE organization_id=$1 AND schedule_number=$2"
        )
        .bind(org_id).bind(schedule_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_schedules WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RecurringJournalSchedule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.recurring_journal_schedules
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY schedule_number"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<RecurringJournalSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.recurring_journal_schedules SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn update_schedule_generation_info(&self, id: Uuid, last_gen: chrono::NaiveDate, next_gen: Option<chrono::NaiveDate>, total: i32) -> AtlasResult<RecurringJournalSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.recurring_journal_schedules
            SET last_generation_date=$2, next_generation_date=$3,
                total_generations=$4, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(last_gen).bind(next_gen).bind(total)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.recurring_journal_schedules WHERE organization_id=$1 AND schedule_number=$2"
        )
        .bind(org_id).bind(schedule_number)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_schedule_line(
        &self, org_id: Uuid, schedule_id: Uuid, line_number: i32, line_type: &str,
        account_code: &str, account_name: Option<&str>, description: Option<&str>,
        amount: &str, currency_code: &str, tax_code: Option<&str>,
        cost_center: Option<&str>, department_id: Option<Uuid>, project_id: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalScheduleLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.recurring_journal_schedule_lines
                (organization_id, schedule_id, line_number, line_type, account_code,
                 account_name, description, amount, currency_code, tax_code, cost_center,
                 department_id, project_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9,$10,$11,$12,$13) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(line_number).bind(line_type)
        .bind(account_code).bind(account_name).bind(description)
        .bind(amount).bind(currency_code).bind(tax_code).bind(cost_center)
        .bind(department_id).bind(project_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line(&row))
    }

    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_schedule_lines WHERE schedule_id=$1 ORDER BY line_number"
        )
        .bind(schedule_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule_line).collect())
    }

    async fn delete_schedule_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.recurring_journal_schedule_lines WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_generation(
        &self, org_id: Uuid, schedule_id: Uuid, generation_number: i32,
        generation_date: chrono::NaiveDate, period_name: Option<&str>,
        total_debit: &str, total_credit: &str, line_count: i32,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalGeneration> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.recurring_journal_generations
                (organization_id, schedule_id, generation_number, generation_date,
                 period_name, total_debit, total_credit, line_count, generated_by)
            VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8,$9) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(generation_number)
        .bind(generation_date).bind(period_name)
        .bind(total_debit).bind(total_credit).bind(line_count)
        .bind(generated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_generation(&row))
    }

    async fn get_generation(&self, id: Uuid) -> AtlasResult<Option<RecurringJournalGeneration>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_generations WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_generation(&r)))
    }

    async fn list_generations(&self, schedule_id: Uuid) -> AtlasResult<Vec<RecurringJournalGeneration>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_generations WHERE schedule_id=$1 ORDER BY generation_number DESC"
        )
        .bind(schedule_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_generation).collect())
    }

    async fn update_generation_status(&self, id: Uuid, status: &str, posted_at: Option<chrono::DateTime<chrono::Utc>>, reversed_at: Option<chrono::DateTime<chrono::Utc>>, reversal_entry_id: Option<Uuid>) -> AtlasResult<RecurringJournalGeneration> {
        let row = sqlx::query(
            r#"UPDATE _atlas.recurring_journal_generations SET status=$2,
                posted_at=COALESCE($3, posted_at),
                reversed_at=COALESCE($4, reversed_at),
                reversal_entry_id=COALESCE($5, reversal_entry_id),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(posted_at).bind(reversed_at).bind(reversal_entry_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_generation(&row))
    }

    async fn get_latest_generation_number(&self, schedule_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(generation_number), 0) as max_gen FROM _atlas.recurring_journal_generations WHERE schedule_id=$1"
        )
        .bind(schedule_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max: i32 = row.try_get("max_gen").unwrap_or(0);
        Ok(max)
    }

    async fn create_generation_line(
        &self, org_id: Uuid, generation_id: Uuid, schedule_line_id: Option<Uuid>,
        line_number: i32, line_type: &str, account_code: &str, account_name: Option<&str>,
        description: Option<&str>, amount: &str, currency_code: &str,
        tax_code: Option<&str>, cost_center: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
    ) -> AtlasResult<RecurringJournalGenerationLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.recurring_journal_generation_lines
                (organization_id, generation_id, schedule_line_id, line_number,
                 line_type, account_code, account_name, description, amount,
                 currency_code, tax_code, cost_center, department_id, project_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10,$11,$12,$13,$14) RETURNING *"#,
        )
        .bind(org_id).bind(generation_id).bind(schedule_line_id)
        .bind(line_number).bind(line_type).bind(account_code)
        .bind(account_name).bind(description).bind(amount)
        .bind(currency_code).bind(tax_code).bind(cost_center)
        .bind(department_id).bind(project_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_generation_line(&row))
    }

    async fn list_generation_lines(&self, generation_id: Uuid) -> AtlasResult<Vec<RecurringJournalGenerationLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.recurring_journal_generation_lines WHERE generation_id=$1 ORDER BY line_number"
        )
        .bind(generation_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_generation_line).collect())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<RecurringJournalDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_count,
                COALESCE(SUM(total_generations), 0) as total_gens,
                0 as gens_this_month,
                COALESCE(SUM(total_generations), 0) as total_amount,
                COUNT(*) FILTER (WHERE status = 'active' AND next_generation_date <= CURRENT_DATE) as due_today,
                0 as overdue
            FROM _atlas.recurring_journal_schedules WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = row.try_get("active_count").unwrap_or(0);
        let draft: i64 = row.try_get("draft_count").unwrap_or(0);
        let gens: i64 = row.try_get("total_gens").unwrap_or(0);
        let due_today: i64 = row.try_get("due_today").unwrap_or(0);

        Ok(RecurringJournalDashboardSummary {
            total_active_schedules: active as i32,
            total_draft_schedules: draft as i32,
            total_generations: gens as i32,
            total_generations_this_month: 0,
            total_generated_amount: "0".to_string(),
            schedules_due_today: due_today as i32,
            schedules_overdue: 0,
            schedules_by_recurrence: serde_json::json!({}),
            schedules_by_status: serde_json::json!({}),
            recent_generations: vec![],
        })
    }
}
