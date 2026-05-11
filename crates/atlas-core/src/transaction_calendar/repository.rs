//! Transaction Calendar Repository
//!
//! `PostgreSQL` storage for transaction calendars, exceptions, and calculation audit.

use atlas_shared::{
    TransactionCalendar, CalendarException, CalendarDateCalculation,
    TransactionCalendarDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for transaction calendar data storage
#[async_trait]
pub trait TransactionCalendarRepository: Send + Sync {
    // Calendars
    async fn create_calendar(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        working_days: &serde_json::Value,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionCalendar>;

    async fn get_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransactionCalendar>>;
    async fn get_calendar_by_id(&self, id: Uuid) -> AtlasResult<Option<TransactionCalendar>>;
    async fn list_calendars(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TransactionCalendar>>;
    async fn update_calendar_status(&self, id: Uuid, status: &str) -> AtlasResult<TransactionCalendar>;
    async fn delete_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Exceptions
    async fn create_exception(
        &self, org_id: Uuid, calendar_id: Uuid,
        exception_date: chrono::NaiveDate, exception_type: &str,
        name: &str, description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CalendarException>;

    async fn get_exception(&self, calendar_id: Uuid, exception_date: chrono::NaiveDate) -> AtlasResult<Option<CalendarException>>;
    async fn list_exceptions(&self, calendar_id: Uuid) -> AtlasResult<Vec<CalendarException>>;
    async fn list_exceptions_range(
        &self, calendar_id: Uuid,
        from_date: chrono::NaiveDate, to_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<CalendarException>>;
    async fn delete_exception(&self, id: Uuid) -> AtlasResult<()>;

    // Calculations
    async fn create_calculation(
        &self, org_id: Uuid, calendar_id: Uuid, calendar_code: &str,
        operation: &str, input_date: chrono::NaiveDate,
        result_date: Option<chrono::NaiveDate>, result_boolean: Option<bool>,
        business_days_added: Option<i32>,
        reference_type: Option<&str>, reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<CalendarDateCalculation>;

    async fn list_calculations(
        &self, org_id: Uuid, calendar_id: Option<Uuid>, limit: Option<i32>,
    ) -> AtlasResult<Vec<CalendarDateCalculation>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransactionCalendarDashboard>;
}

/// `PostgreSQL` implementation
pub struct PostgresTransactionCalendarRepository {
    pool: PgPool,
}

impl PostgresTransactionCalendarRepository {
    #[must_use] 
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_calendar(row: &sqlx::postgres::PgRow) -> TransactionCalendar {
    TransactionCalendar {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        working_days: row.get("working_days"),
        status: row.get("status"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_exception(row: &sqlx::postgres::PgRow) -> CalendarException {
    CalendarException {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        calendar_id: row.get("calendar_id"),
        exception_date: row.get("exception_date"),
        exception_type: row.get("exception_type"),
        name: row.get("name"),
        description: row.get("description"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_calculation(row: &sqlx::postgres::PgRow) -> CalendarDateCalculation {
    CalendarDateCalculation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        calendar_id: row.get("calendar_id"),
        calendar_code: row.get("calendar_code"),
        operation: row.get("operation"),
        input_date: row.get("input_date"),
        result_date: row.get("result_date"),
        result_boolean: row.get("result_boolean"),
        business_days_added: row.get("business_days_added"),
        reference_type: row.get("reference_type"),
        reference_id: row.get("reference_id"),
        calculated_at: row.get("calculated_at"),
        calculated_by: row.get("calculated_by"),
    }
}

#[async_trait]
impl TransactionCalendarRepository for PostgresTransactionCalendarRepository {
    async fn create_calendar(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        working_days: &serde_json::Value,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionCalendar> {
        let row = sqlx::query(
            r"INSERT INTO _atlas.transaction_calendars
                (organization_id, code, name, description, working_days,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *",
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(working_days).bind(effective_from).bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calendar(&row))
    }

    async fn get_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransactionCalendar>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transaction_calendars WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_calendar(&r)))
    }

    async fn get_calendar_by_id(&self, id: Uuid) -> AtlasResult<Option<TransactionCalendar>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transaction_calendars WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_calendar(&r)))
    }

    async fn list_calendars(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TransactionCalendar>> {
        let rows = sqlx::query(
            r"SELECT * FROM _atlas.transaction_calendars
            WHERE organization_id=$1
              AND ($2::text IS NULL OR status=$2)
            ORDER BY code",
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_calendar).collect())
    }

    async fn update_calendar_status(&self, id: Uuid, status: &str) -> AtlasResult<TransactionCalendar> {
        let row = sqlx::query(
            r"UPDATE _atlas.transaction_calendars SET status=$2, updated_at=now()
            WHERE id=$1 RETURNING *",
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calendar(&row))
    }

    async fn delete_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.transaction_calendars WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_exception(
        &self, org_id: Uuid, calendar_id: Uuid,
        exception_date: chrono::NaiveDate, exception_type: &str,
        name: &str, description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CalendarException> {
        let row = sqlx::query(
            r"INSERT INTO _atlas.calendar_exceptions
                (organization_id, calendar_id, exception_date, exception_type,
                 name, description, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *",
        )
        .bind(org_id).bind(calendar_id).bind(exception_date)
        .bind(exception_type).bind(name).bind(description)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_exception(&row))
    }

    async fn get_exception(&self, calendar_id: Uuid, exception_date: chrono::NaiveDate) -> AtlasResult<Option<CalendarException>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.calendar_exceptions WHERE calendar_id=$1 AND exception_date=$2"
        )
        .bind(calendar_id).bind(exception_date)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_exception(&r)))
    }

    async fn list_exceptions(&self, calendar_id: Uuid) -> AtlasResult<Vec<CalendarException>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.calendar_exceptions WHERE calendar_id=$1 ORDER BY exception_date"
        )
        .bind(calendar_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_exception).collect())
    }

    async fn list_exceptions_range(
        &self, calendar_id: Uuid,
        from_date: chrono::NaiveDate, to_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<CalendarException>> {
        let rows = sqlx::query(
            r"SELECT * FROM _atlas.calendar_exceptions
            WHERE calendar_id=$1 AND exception_date >= $2 AND exception_date <= $3
            ORDER BY exception_date",
        )
        .bind(calendar_id).bind(from_date).bind(to_date)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_exception).collect())
    }

    async fn delete_exception(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.calendar_exceptions WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_calculation(
        &self, org_id: Uuid, calendar_id: Uuid, calendar_code: &str,
        operation: &str, input_date: chrono::NaiveDate,
        result_date: Option<chrono::NaiveDate>, result_boolean: Option<bool>,
        business_days_added: Option<i32>,
        reference_type: Option<&str>, reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<CalendarDateCalculation> {
        let row = sqlx::query(
            r"INSERT INTO _atlas.calendar_date_calculations
                (organization_id, calendar_id, calendar_code, operation,
                 input_date, result_date, result_boolean, business_days_added,
                 reference_type, reference_id, calculated_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *",
        )
        .bind(org_id).bind(calendar_id).bind(calendar_code)
        .bind(operation).bind(input_date).bind(result_date)
        .bind(result_boolean).bind(business_days_added)
        .bind(reference_type).bind(reference_id).bind(calculated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_calculation(&row))
    }

    async fn list_calculations(
        &self, org_id: Uuid, calendar_id: Option<Uuid>, limit: Option<i32>,
    ) -> AtlasResult<Vec<CalendarDateCalculation>> {
        let limit_val = limit.unwrap_or(100);
        let rows = if calendar_id.is_some() {
            sqlx::query(
                r"SELECT * FROM _atlas.calendar_date_calculations
                WHERE organization_id=$1 AND calendar_id=$2
                ORDER BY calculated_at DESC LIMIT $3",
            )
            .bind(org_id).bind(calendar_id).bind(limit_val)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r"SELECT * FROM _atlas.calendar_date_calculations
                WHERE organization_id=$1
                ORDER BY calculated_at DESC LIMIT $2",
            )
            .bind(org_id).bind(limit_val)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_calculation).collect())
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransactionCalendarDashboard> {
        let row = sqlx::query(
            r"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.transaction_calendars WHERE organization_id = $1",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let active: i64 = row.try_get("active").unwrap_or(0);

        let exception_count: i64 = sqlx::query_scalar(
            r"SELECT COUNT(*) FROM _atlas.calendar_exceptions ce
            JOIN _atlas.transaction_calendars tc ON ce.calendar_id = tc.id
            WHERE tc.organization_id = $1",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let calculation_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.calendar_date_calculations WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_rows = sqlx::query(
            r"SELECT * FROM _atlas.calendar_date_calculations
            WHERE organization_id=$1
            ORDER BY calculated_at DESC LIMIT 10",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(TransactionCalendarDashboard {
            total_calendars: total as i32,
            active_calendars: active as i32,
            total_exceptions: exception_count as i32,
            total_calculations: calculation_count,
            recent_calculations: recent_rows.iter().map(row_to_calculation).collect(),
        })
    }
}
