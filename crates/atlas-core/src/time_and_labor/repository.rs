//! Time and Labor Repository
//!
//! PostgreSQL storage for work schedules, overtime rules, time cards,
//! time entries, history, and labor distributions.

use atlas_shared::{
    WorkSchedule, OvertimeRule, TimeCard, TimeEntry, TimeCardHistory,
    LaborDistribution, AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for time and labor data storage
#[async_trait]
pub trait TimeAndLaborRepository: Send + Sync {
    // Work Schedules
    async fn create_work_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        schedule_type: &str,
        standard_hours_per_day: &str,
        standard_hours_per_week: &str,
        work_days_per_week: i32,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        break_duration_minutes: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WorkSchedule>;

    async fn get_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WorkSchedule>>;
    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkSchedule>>;
    async fn list_work_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<WorkSchedule>>;
    async fn delete_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Overtime Rules
    async fn create_overtime_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        threshold_type: &str,
        daily_threshold_hours: &str,
        weekly_threshold_hours: &str,
        overtime_multiplier: &str,
        double_time_threshold_hours: Option<&str>,
        double_time_multiplier: &str,
        include_holidays: bool,
        include_weekends: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<OvertimeRule>;

    async fn get_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<OvertimeRule>>;
    async fn get_overtime_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<OvertimeRule>>;
    async fn list_overtime_rules(&self, org_id: Uuid) -> AtlasResult<Vec<OvertimeRule>>;
    async fn delete_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Time Cards
    async fn create_time_card(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        card_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        schedule_id: Option<Uuid>,
        overtime_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeCard>;

    async fn get_time_card(&self, id: Uuid) -> AtlasResult<Option<TimeCard>>;
    async fn get_time_card_by_number(&self, org_id: Uuid, card_number: &str) -> AtlasResult<Option<TimeCard>>;
    async fn list_time_cards(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TimeCard>>;
    async fn update_time_card_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<TimeCard>;
    async fn update_time_card_totals(
        &self,
        id: Uuid,
        regular_hours: &str,
        overtime_hours: &str,
        double_time_hours: &str,
        total_hours: &str,
    ) -> AtlasResult<()>;

    // Time Entries
    async fn create_time_entry(
        &self,
        org_id: Uuid,
        time_card_id: Uuid,
        entry_date: chrono::NaiveDate,
        entry_type: &str,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        duration_hours: &str,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        task_name: Option<&str>,
        location: Option<&str>,
        cost_center: Option<&str>,
        labor_category: Option<&str>,
        comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeEntry>;

    async fn get_time_entry(&self, id: Uuid) -> AtlasResult<Option<TimeEntry>>;
    async fn list_time_entries_by_card(&self, time_card_id: Uuid) -> AtlasResult<Vec<TimeEntry>>;
    async fn delete_time_entry(&self, id: Uuid) -> AtlasResult<()>;

    // Time Card History
    async fn add_history(
        &self,
        time_card_id: Uuid,
        action: &str,
        from_status: Option<&str>,
        to_status: Option<&str>,
        performed_by: Option<Uuid>,
        comment: Option<&str>,
    ) -> AtlasResult<()>;

    async fn get_time_card_history(&self, time_card_id: Uuid) -> AtlasResult<Vec<TimeCardHistory>>;

    // Labor Distributions
    async fn create_labor_distribution(
        &self,
        org_id: Uuid,
        time_entry_id: Uuid,
        distribution_percent: &str,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        gl_account_code: Option<&str>,
        allocated_hours: &str,
    ) -> AtlasResult<LaborDistribution>;

    async fn list_labor_distributions_by_entry(&self, time_entry_id: Uuid) -> AtlasResult<Vec<LaborDistribution>>;
    async fn delete_labor_distribution(&self, id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresTimeAndLaborRepository {
    pool: PgPool,
}

impl PostgresTimeAndLaborRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_schedule(&self, row: &sqlx::postgres::PgRow) -> WorkSchedule {
        WorkSchedule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            schedule_type: row.get("schedule_type"),
            standard_hours_per_day: row.get("standard_hours_per_day"),
            standard_hours_per_week: row.get("standard_hours_per_week"),
            work_days_per_week: row.get("work_days_per_week"),
            start_time: row.try_get("start_time").ok().flatten(),
            end_time: row.try_get("end_time").ok().flatten(),
            break_duration_minutes: row.get("break_duration_minutes"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_overtime_rule(&self, row: &sqlx::postgres::PgRow) -> OvertimeRule {
        OvertimeRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            threshold_type: row.get("threshold_type"),
            daily_threshold_hours: row.get("daily_threshold_hours"),
            weekly_threshold_hours: row.get("weekly_threshold_hours"),
            overtime_multiplier: row.get("overtime_multiplier"),
            double_time_threshold_hours: row.try_get("double_time_threshold_hours").ok().flatten(),
            double_time_multiplier: row.get("double_time_multiplier"),
            include_holidays: row.get("include_holidays"),
            include_weekends: row.get("include_weekends"),
            is_active: row.get("is_active"),
            effective_from: row.try_get("effective_from").ok().flatten(),
            effective_to: row.try_get("effective_to").ok().flatten(),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_time_card(&self, row: &sqlx::postgres::PgRow) -> TimeCard {
        TimeCard {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            card_number: row.get("card_number"),
            status: row.get("status"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            total_regular_hours: row.get("total_regular_hours"),
            total_overtime_hours: row.get("total_overtime_hours"),
            total_double_time_hours: row.get("total_double_time_hours"),
            total_hours: row.get("total_hours"),
            schedule_id: row.get("schedule_id"),
            overtime_rule_id: row.get("overtime_rule_id"),
            submitted_at: row.get("submitted_at"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            comments: row.get("comments"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_time_entry(&self, row: &sqlx::postgres::PgRow) -> TimeEntry {
        TimeEntry {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            time_card_id: row.get("time_card_id"),
            entry_date: row.get("entry_date"),
            entry_type: row.get("entry_type"),
            start_time: row.try_get("start_time").ok().flatten(),
            end_time: row.try_get("end_time").ok().flatten(),
            duration_hours: row.get("duration_hours"),
            project_id: row.get("project_id"),
            project_name: row.get("project_name"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            task_name: row.get("task_name"),
            location: row.get("location"),
            cost_center: row.get("cost_center"),
            labor_category: row.get("labor_category"),
            comments: row.get("comments"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_history(&self, row: &sqlx::postgres::PgRow) -> TimeCardHistory {
        TimeCardHistory {
            id: row.get("id"),
            time_card_id: row.get("time_card_id"),
            action: row.get("action"),
            from_status: row.get("from_status"),
            to_status: row.get("to_status"),
            performed_by: row.get("performed_by"),
            comment: row.get("comment"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_distribution(&self, row: &sqlx::postgres::PgRow) -> LaborDistribution {
        LaborDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            time_entry_id: row.get("time_entry_id"),
            distribution_percent: row.get("distribution_percent"),
            cost_center: row.get("cost_center"),
            project_id: row.get("project_id"),
            project_name: row.get("project_name"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            gl_account_code: row.get("gl_account_code"),
            allocated_hours: row.get("allocated_hours"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl TimeAndLaborRepository for PostgresTimeAndLaborRepository {
    // ========================================================================
    // Work Schedules
    // ========================================================================

    async fn create_work_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        schedule_type: &str,
        standard_hours_per_day: &str,
        standard_hours_per_week: &str,
        work_days_per_week: i32,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        break_duration_minutes: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WorkSchedule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.work_schedules
                (organization_id, code, name, description, schedule_type,
                 standard_hours_per_day, standard_hours_per_week, work_days_per_week,
                 start_time, end_time, break_duration_minutes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, schedule_type = $5,
                    standard_hours_per_day = $6, standard_hours_per_week = $7,
                    work_days_per_week = $8, start_time = $9, end_time = $10,
                    break_duration_minutes = $11, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(schedule_type)
        .bind(standard_hours_per_day).bind(standard_hours_per_week).bind(work_days_per_week)
        .bind(start_time).bind(end_time).bind(break_duration_minutes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_schedule(&row))
    }

    async fn get_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WorkSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.work_schedules WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_schedule(&r)))
    }

    async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.work_schedules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_schedule(&r)))
    }

    async fn list_work_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<WorkSchedule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.work_schedules WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_schedule(r)).collect())
    }

    async fn delete_work_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.work_schedules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Overtime Rules
    // ========================================================================

    async fn create_overtime_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        threshold_type: &str,
        daily_threshold_hours: &str,
        weekly_threshold_hours: &str,
        overtime_multiplier: &str,
        double_time_threshold_hours: Option<&str>,
        double_time_multiplier: &str,
        include_holidays: bool,
        include_weekends: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<OvertimeRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.overtime_rules
                (organization_id, code, name, description, threshold_type,
                 daily_threshold_hours, weekly_threshold_hours, overtime_multiplier,
                 double_time_threshold_hours, double_time_multiplier,
                 include_holidays, include_weekends,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, threshold_type = $5,
                    daily_threshold_hours = $6, weekly_threshold_hours = $7,
                    overtime_multiplier = $8, double_time_threshold_hours = $9,
                    double_time_multiplier = $10, include_holidays = $11,
                    include_weekends = $12, effective_from = $13, effective_to = $14,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(threshold_type)
        .bind(daily_threshold_hours).bind(weekly_threshold_hours).bind(overtime_multiplier)
        .bind(double_time_threshold_hours).bind(double_time_multiplier)
        .bind(include_holidays).bind(include_weekends)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_overtime_rule(&row))
    }

    async fn get_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<OvertimeRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.overtime_rules WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_overtime_rule(&r)))
    }

    async fn get_overtime_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<OvertimeRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.overtime_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_overtime_rule(&r)))
    }

    async fn list_overtime_rules(&self, org_id: Uuid) -> AtlasResult<Vec<OvertimeRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.overtime_rules WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_overtime_rule(r)).collect())
    }

    async fn delete_overtime_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.overtime_rules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Time Cards
    // ========================================================================

    async fn create_time_card(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        card_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        schedule_id: Option<Uuid>,
        overtime_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeCard> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.time_cards
                (organization_id, employee_id, employee_name, card_number,
                 period_start, period_end, schedule_id, overtime_rule_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (organization_id, employee_id, period_start, period_end) DO UPDATE
                SET employee_name = $3, schedule_id = $7, overtime_rule_id = $8,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(employee_id).bind(employee_name).bind(card_number)
        .bind(period_start).bind(period_end)
        .bind(schedule_id).bind(overtime_rule_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_time_card(&row))
    }

    async fn get_time_card(&self, id: Uuid) -> AtlasResult<Option<TimeCard>> {
        let row = sqlx::query("SELECT * FROM _atlas.time_cards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_time_card(&r)))
    }

    async fn get_time_card_by_number(&self, org_id: Uuid, card_number: &str) -> AtlasResult<Option<TimeCard>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.time_cards WHERE organization_id = $1 AND card_number = $2"
        )
        .bind(org_id).bind(card_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_time_card(&r)))
    }

    async fn list_time_cards(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TimeCard>> {
        let rows = match (employee_id, status) {
            (Some(eid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.time_cards WHERE organization_id = $1 AND employee_id = $2 AND status = $3 ORDER BY period_start DESC"
            )
            .bind(org_id).bind(eid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None) => sqlx::query(
                "SELECT * FROM _atlas.time_cards WHERE organization_id = $1 AND employee_id = $2 ORDER BY period_start DESC"
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.time_cards WHERE organization_id = $1 AND status = $2 ORDER BY period_start DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.time_cards WHERE organization_id = $1 ORDER BY period_start DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_time_card(r)).collect())
    }

    async fn update_time_card_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<TimeCard> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.time_cards
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                rejected_reason = COALESCE($4, rejected_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_time_card(&row))
    }

    async fn update_time_card_totals(
        &self,
        id: Uuid,
        regular_hours: &str,
        overtime_hours: &str,
        double_time_hours: &str,
        total_hours: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.time_cards
            SET total_regular_hours = $2, total_overtime_hours = $3,
                total_double_time_hours = $4, total_hours = $5, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(regular_hours).bind(overtime_hours)
        .bind(double_time_hours).bind(total_hours)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Time Entries
    // ========================================================================

    async fn create_time_entry(
        &self,
        org_id: Uuid,
        time_card_id: Uuid,
        entry_date: chrono::NaiveDate,
        entry_type: &str,
        start_time: Option<chrono::NaiveTime>,
        end_time: Option<chrono::NaiveTime>,
        duration_hours: &str,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        task_name: Option<&str>,
        location: Option<&str>,
        cost_center: Option<&str>,
        labor_category: Option<&str>,
        comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TimeEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.time_entries
                (organization_id, time_card_id, entry_date, entry_type,
                 start_time, end_time, duration_hours,
                 project_id, project_name, department_id, department_name,
                 task_name, location, cost_center, labor_category, comments, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(time_card_id).bind(entry_date).bind(entry_type)
        .bind(start_time).bind(end_time).bind(duration_hours)
        .bind(project_id).bind(project_name).bind(department_id).bind(department_name)
        .bind(task_name).bind(location).bind(cost_center).bind(labor_category)
        .bind(comments).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_time_entry(&row))
    }

    async fn get_time_entry(&self, id: Uuid) -> AtlasResult<Option<TimeEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.time_entries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_time_entry(&r)))
    }

    async fn list_time_entries_by_card(&self, time_card_id: Uuid) -> AtlasResult<Vec<TimeEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.time_entries WHERE time_card_id = $1 ORDER BY entry_date, start_time"
        )
        .bind(time_card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_time_entry(r)).collect())
    }

    async fn delete_time_entry(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.time_entries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // History
    // ========================================================================

    async fn add_history(
        &self,
        time_card_id: Uuid,
        action: &str,
        from_status: Option<&str>,
        to_status: Option<&str>,
        performed_by: Option<Uuid>,
        comment: Option<&str>,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _atlas.time_card_history
                (time_card_id, action, from_status, to_status, performed_by, comment)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(time_card_id).bind(action).bind(from_status).bind(to_status)
        .bind(performed_by).bind(comment)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_time_card_history(&self, time_card_id: Uuid) -> AtlasResult<Vec<TimeCardHistory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.time_card_history WHERE time_card_id = $1 ORDER BY created_at ASC"
        )
        .bind(time_card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_history(r)).collect())
    }

    // ========================================================================
    // Labor Distributions
    // ========================================================================

    async fn create_labor_distribution(
        &self,
        org_id: Uuid,
        time_entry_id: Uuid,
        distribution_percent: &str,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        gl_account_code: Option<&str>,
        allocated_hours: &str,
    ) -> AtlasResult<LaborDistribution> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.labor_distributions
                (organization_id, time_entry_id, distribution_percent,
                 cost_center, project_id, project_name, department_id, department_name,
                 gl_account_code, allocated_hours)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(time_entry_id).bind(distribution_percent)
        .bind(cost_center).bind(project_id).bind(project_name)
        .bind(department_id).bind(department_name).bind(gl_account_code)
        .bind(allocated_hours)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_distribution(&row))
    }

    async fn list_labor_distributions_by_entry(&self, time_entry_id: Uuid) -> AtlasResult<Vec<LaborDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.labor_distributions WHERE time_entry_id = $1 ORDER BY created_at"
        )
        .bind(time_entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(r)).collect())
    }

    async fn delete_labor_distribution(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.labor_distributions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
