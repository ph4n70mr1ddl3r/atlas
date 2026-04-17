//! Period Close Repository
//!
//! PostgreSQL storage for accounting calendars, periods, and close checklist.

use atlas_shared::{
    AccountingCalendar, AccountingPeriod, PeriodCloseChecklistItem,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for period close storage
#[async_trait]
pub trait PeriodCloseRepository: Send + Sync {
    // Calendar management
    async fn create_calendar(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        calendar_type: &str,
        fiscal_year_start_month: i32,
        periods_per_year: i32,
        has_adjusting_period: bool,
        current_fiscal_year: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingCalendar>;

    async fn get_calendar(&self, id: Uuid) -> AtlasResult<Option<AccountingCalendar>>;
    async fn list_calendars(&self, org_id: Uuid) -> AtlasResult<Vec<AccountingCalendar>>;
    async fn delete_calendar(&self, id: Uuid) -> AtlasResult<()>;

    // Period management
    async fn create_period(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        period_name: &str,
        period_number: i32,
        fiscal_year: i32,
        quarter: Option<i32>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        period_type: &str,
    ) -> AtlasResult<AccountingPeriod>;

    async fn get_period(&self, id: Uuid) -> AtlasResult<Option<AccountingPeriod>>;
    async fn get_period_by_date(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        date: chrono::NaiveDate,
    ) -> AtlasResult<Option<AccountingPeriod>>;
    async fn list_periods(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        fiscal_year: Option<i32>,
    ) -> AtlasResult<Vec<AccountingPeriod>>;

    async fn update_period_status(
        &self,
        id: Uuid,
        status: &str,
        changed_by: Option<Uuid>,
    ) -> AtlasResult<AccountingPeriod>;

    async fn update_subledger_status(
        &self,
        id: Uuid,
        subledger: &str,
        status: &str,
    ) -> AtlasResult<AccountingPeriod>;

    async fn increment_journal_count(&self, id: Uuid) -> AtlasResult<()>;

    // Checklist management
    async fn create_checklist_item(
        &self,
        org_id: Uuid,
        period_id: Uuid,
        task_name: &str,
        task_description: Option<&str>,
        task_order: i32,
        category: Option<&str>,
        subledger: Option<&str>,
        assigned_to: Option<Uuid>,
        due_date: Option<chrono::NaiveDate>,
        depends_on: Option<Uuid>,
    ) -> AtlasResult<PeriodCloseChecklistItem>;

    async fn list_checklist_items(&self, period_id: Uuid) -> AtlasResult<Vec<PeriodCloseChecklistItem>>;
    async fn update_checklist_item_status(
        &self,
        id: Uuid,
        status: &str,
        completed_by: Option<Uuid>,
    ) -> AtlasResult<PeriodCloseChecklistItem>;
    async fn delete_checklist_item(&self, id: Uuid) -> AtlasResult<()>;

    // Exceptions
    async fn grant_period_exception(
        &self,
        org_id: Uuid,
        period_id: Uuid,
        user_id: Uuid,
        allowed_actions: serde_json::Value,
        reason: Option<&str>,
        granted_by: Option<Uuid>,
        valid_until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<()>;

    async fn check_period_exception(
        &self,
        period_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<bool>;

    async fn revoke_period_exception(&self, period_id: Uuid, user_id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresPeriodCloseRepository {
    pool: PgPool,
}

impl PostgresPeriodCloseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_calendar(&self, row: &sqlx::postgres::PgRow) -> AccountingCalendar {
        AccountingCalendar {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            calendar_type: row.get("calendar_type"),
            fiscal_year_start_month: row.get("fiscal_year_start_month"),
            periods_per_year: row.get("periods_per_year"),
            has_adjusting_period: row.get("has_adjusting_period"),
            current_fiscal_year: row.get("current_fiscal_year"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }

    fn row_to_period(&self, row: &sqlx::postgres::PgRow) -> AccountingPeriod {
        AccountingPeriod {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            calendar_id: row.get("calendar_id"),
            period_name: row.get("period_name"),
            period_number: row.get("period_number"),
            fiscal_year: row.get("fiscal_year"),
            quarter: row.get("quarter"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            status: row.get("status"),
            status_changed_by: row.get("status_changed_by"),
            status_changed_at: row.get("status_changed_at"),
            closed_by: row.get("closed_by"),
            closed_at: row.get("closed_at"),
            period_type: row.get("period_type"),
            gl_status: row.get("gl_status"),
            ap_status: row.get("ap_status"),
            ar_status: row.get("ar_status"),
            fa_status: row.get("fa_status"),
            po_status: row.get("po_status"),
            total_debits: row_to_json_value(row, "total_debits"),
            total_credits: row_to_json_value(row, "total_credits"),
            net_activity: row_to_json_value(row, "net_activity"),
            beginning_balance: row_to_json_value(row, "beginning_balance"),
            ending_balance: row_to_json_value(row, "ending_balance"),
            journal_entry_count: row.get("journal_entry_count"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_checklist_item(&self, row: &sqlx::postgres::PgRow) -> PeriodCloseChecklistItem {
        PeriodCloseChecklistItem {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            period_id: row.get("period_id"),
            task_name: row.get("task_name"),
            task_description: row.get("task_description"),
            task_order: row.get("task_order"),
            category: row.get("category"),
            subledger: row.get("subledger"),
            status: row.get("status"),
            assigned_to: row.get("assigned_to"),
            due_date: row.get("due_date"),
            completed_by: row.get("completed_by"),
            completed_at: row.get("completed_at"),
            depends_on: row.get("depends_on"),
            notes: row.get("notes"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

fn row_to_json_value(row: &sqlx::postgres::PgRow, col: &str) -> serde_json::Value {
    use sqlx::Row;
    // Try numeric first, then string, fallback to 0
    row.try_get::<serde_json::Value, _>(col).unwrap_or(serde_json::json!(0))
}

#[async_trait]
impl PeriodCloseRepository for PostgresPeriodCloseRepository {
    async fn create_calendar(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        calendar_type: &str,
        fiscal_year_start_month: i32,
        periods_per_year: i32,
        has_adjusting_period: bool,
        current_fiscal_year: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingCalendar> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.accounting_calendars
                (organization_id, name, description, calendar_type, fiscal_year_start_month,
                 periods_per_year, has_adjusting_period, current_fiscal_year, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(calendar_type)
        .bind(fiscal_year_start_month)
        .bind(periods_per_year)
        .bind(has_adjusting_period)
        .bind(current_fiscal_year)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_calendar(&row))
    }

    async fn get_calendar(&self, id: Uuid) -> AtlasResult<Option<AccountingCalendar>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_calendars WHERE id = $1 AND is_active = true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_calendar(&r)))
    }

    async fn list_calendars(&self, org_id: Uuid) -> AtlasResult<Vec<AccountingCalendar>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.accounting_calendars WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_calendar(&r)).collect())
    }

    async fn delete_calendar(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.accounting_calendars SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_period(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        period_name: &str,
        period_number: i32,
        fiscal_year: i32,
        quarter: Option<i32>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        period_type: &str,
    ) -> AtlasResult<AccountingPeriod> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.accounting_periods
                (organization_id, calendar_id, period_name, period_number, fiscal_year,
                 quarter, start_date, end_date, period_type, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'not_opened')
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(calendar_id)
        .bind(period_name)
        .bind(period_number)
        .bind(fiscal_year)
        .bind(quarter)
        .bind(start_date)
        .bind(end_date)
        .bind(period_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_period(&row))
    }

    async fn get_period(&self, id: Uuid) -> AtlasResult<Option<AccountingPeriod>> {
        let row = sqlx::query("SELECT * FROM _atlas.accounting_periods WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_period(&r)))
    }

    async fn get_period_by_date(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        date: chrono::NaiveDate,
    ) -> AtlasResult<Option<AccountingPeriod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_periods WHERE organization_id = $1 AND calendar_id = $2 AND start_date <= $3 AND end_date >= $3",
        )
        .bind(org_id)
        .bind(calendar_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_period(&r)))
    }

    async fn list_periods(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        fiscal_year: Option<i32>,
    ) -> AtlasResult<Vec<AccountingPeriod>> {
        let rows = match fiscal_year {
            Some(fy) => {
                sqlx::query(
                    "SELECT * FROM _atlas.accounting_periods WHERE organization_id = $1 AND calendar_id = $2 AND fiscal_year = $3 ORDER BY period_number",
                )
                .bind(org_id)
                .bind(calendar_id)
                .bind(fy)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    "SELECT * FROM _atlas.accounting_periods WHERE organization_id = $1 AND calendar_id = $2 ORDER BY fiscal_year DESC, period_number",
                )
                .bind(org_id)
                .bind(calendar_id)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_period(&r)).collect())
    }

    async fn update_period_status(
        &self,
        id: Uuid,
        status: &str,
        changed_by: Option<Uuid>,
    ) -> AtlasResult<AccountingPeriod> {
        let closed_at = if status == "closed" || status == "permanently_closed" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let row = sqlx::query(
            r#"
            UPDATE _atlas.accounting_periods
            SET status = $2, status_changed_by = $3, status_changed_at = now(),
                closed_by = CASE WHEN $4 IS NOT NULL THEN $3 ELSE closed_by END,
                closed_at = CASE WHEN $4 IS NOT NULL THEN $4 ELSE closed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(changed_by)
        .bind(closed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_period(&row))
    }

    async fn update_subledger_status(
        &self,
        id: Uuid,
        subledger: &str,
        status: &str,
    ) -> AtlasResult<AccountingPeriod> {
        let column = match subledger {
            "gl" => "gl_status",
            "ap" => "ap_status",
            "ar" => "ar_status",
            "fa" => "fa_status",
            "po" => "po_status",
            _ => return Err(AtlasError::ValidationFailed(format!("Unknown subledger: {}", subledger))),
        };

        let sql = format!(
            "UPDATE _atlas.accounting_periods SET {} = $2, updated_at = now() WHERE id = $1 RETURNING *",
            column
        );

        let row = sqlx::query(&sql)
            .bind(id)
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_period(&row))
    }

    async fn increment_journal_count(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.accounting_periods SET journal_entry_count = journal_entry_count + 1, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Checklist methods

    async fn create_checklist_item(
        &self,
        org_id: Uuid,
        period_id: Uuid,
        task_name: &str,
        task_description: Option<&str>,
        task_order: i32,
        category: Option<&str>,
        subledger: Option<&str>,
        assigned_to: Option<Uuid>,
        due_date: Option<chrono::NaiveDate>,
        depends_on: Option<Uuid>,
    ) -> AtlasResult<PeriodCloseChecklistItem> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.period_close_checklist
                (organization_id, period_id, task_name, task_description, task_order,
                 category, subledger, assigned_to, due_date, depends_on, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'pending')
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(period_id)
        .bind(task_name)
        .bind(task_description)
        .bind(task_order)
        .bind(category)
        .bind(subledger)
        .bind(assigned_to)
        .bind(due_date)
        .bind(depends_on)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_checklist_item(&row))
    }

    async fn list_checklist_items(&self, period_id: Uuid) -> AtlasResult<Vec<PeriodCloseChecklistItem>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.period_close_checklist WHERE period_id = $1 ORDER BY task_order",
        )
        .bind(period_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_checklist_item(&r)).collect())
    }

    async fn update_checklist_item_status(
        &self,
        id: Uuid,
        status: &str,
        completed_by: Option<Uuid>,
    ) -> AtlasResult<PeriodCloseChecklistItem> {
        let completed_at = if status == "completed" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let row = sqlx::query(
            r#"
            UPDATE _atlas.period_close_checklist
            SET status = $2, completed_by = $3, completed_at = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(completed_by)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_checklist_item(&row))
    }

    async fn delete_checklist_item(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.period_close_checklist WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Exception methods

    async fn grant_period_exception(
        &self,
        org_id: Uuid,
        period_id: Uuid,
        user_id: Uuid,
        allowed_actions: serde_json::Value,
        reason: Option<&str>,
        granted_by: Option<Uuid>,
        valid_until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _atlas.period_close_exceptions
                (organization_id, period_id, user_id, allowed_actions, reason, granted_by, valid_until)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (period_id, user_id) DO UPDATE
                SET allowed_actions = $4, reason = $5, granted_by = $6, valid_until = $7
            "#,
        )
        .bind(org_id)
        .bind(period_id)
        .bind(user_id)
        .bind(&allowed_actions)
        .bind(reason)
        .bind(granted_by)
        .bind(valid_until)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn check_period_exception(
        &self,
        period_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM _atlas.period_close_exceptions
                WHERE period_id = $1 AND user_id = $2
                  AND (valid_until IS NULL OR valid_until > now())
            )
            "#,
        )
        .bind(period_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(exists)
    }

    async fn revoke_period_exception(&self, period_id: Uuid, user_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.period_close_exceptions WHERE period_id = $1 AND user_id = $2",
        )
        .bind(period_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
