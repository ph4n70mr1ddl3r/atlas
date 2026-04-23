//! Absence Repository
//!
//! PostgreSQL storage for absence types, plans, entries, balances, and history.

use atlas_shared::{
    AbsenceType, AbsencePlan, AbsenceBalance, AbsenceEntry, AbsenceEntryHistory,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for absence data storage
#[async_trait]
pub trait AbsenceRepository: Send + Sync {
    // Absence Types
    async fn create_absence_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        plan_type: &str,
        requires_approval: bool,
        requires_documentation: bool,
        auto_approve_below_days: &str,
        allow_negative_balance: bool,
        allow_half_day: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceType>;

    async fn get_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsenceType>>;
    async fn list_absence_types(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<AbsenceType>>;
    async fn delete_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Absence Plans
    async fn create_absence_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        absence_type_id: Uuid,
        accrual_frequency: &str,
        accrual_rate: &str,
        accrual_unit: &str,
        carry_over_max: Option<String>,
        carry_over_expiry_months: Option<i32>,
        max_balance: Option<String>,
        probation_period_days: i32,
        prorate_first_year: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsencePlan>;

    async fn get_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsencePlan>>;
    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<AbsencePlan>>;
    async fn list_absence_plans(&self, org_id: Uuid, absence_type_id: Option<Uuid>) -> AtlasResult<Vec<AbsencePlan>>;
    async fn delete_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Absence Entries
    async fn create_entry(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        absence_type_id: Uuid,
        plan_id: Option<Uuid>,
        entry_number: &str,
        status: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        duration_days: &str,
        duration_hours: Option<String>,
        is_half_day: bool,
        half_day_period: Option<&str>,
        reason: Option<&str>,
        comments: Option<&str>,
        documentation_provided: bool,
        approved_by: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceEntry>;

    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<AbsenceEntry>>;
    async fn list_entries(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        absence_type_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<AbsenceEntry>>;
    async fn update_entry_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        cancelled_reason: Option<&str>,
    ) -> AtlasResult<AbsenceEntry>;
    async fn find_overlapping_entries(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AbsenceEntry>>;

    // Absence Balances
    async fn create_balance(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        accrued: &str,
        taken: &str,
        adjusted: &str,
        carried_over: &str,
        remaining: &str,
    ) -> AtlasResult<AbsenceBalance>;
    async fn get_balance(
        &self,
        employee_id: Uuid,
        plan_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<Option<AbsenceBalance>>;
    async fn get_balance_for_previous_period(
        &self,
        employee_id: Uuid,
        plan_id: Uuid,
        current_period_start: chrono::NaiveDate,
    ) -> AtlasResult<Option<AbsenceBalance>>;
    async fn list_balances(&self, org_id: Uuid, employee_id: Uuid) -> AtlasResult<Vec<AbsenceBalance>>;
    async fn update_balance(
        &self,
        id: Uuid,
        taken: &str,
        adjusted: &str,
        remaining: &str,
    ) -> AtlasResult<()>;

    // History
    async fn add_history(
        &self,
        entry_id: Uuid,
        action: &str,
        from_status: Option<&str>,
        to_status: Option<&str>,
        performed_by: Option<Uuid>,
        comment: Option<&str>,
    ) -> AtlasResult<()>;
    async fn get_entry_history(&self, entry_id: Uuid) -> AtlasResult<Vec<AbsenceEntryHistory>>;
}

/// PostgreSQL implementation
pub struct PostgresAbsenceRepository {
    pool: PgPool,
}

impl PostgresAbsenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_absence_type(&self, row: &sqlx::postgres::PgRow) -> AbsenceType {
        let auto_approve: String = row.get("auto_approve_below_days");
        AbsenceType {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            category: row.get("category"),
            plan_type: row.get("plan_type"),
            requires_approval: row.get("requires_approval"),
            requires_documentation: row.get("requires_documentation"),
            auto_approve_below_days: auto_approve,
            allow_negative_balance: row.get("allow_negative_balance"),
            allow_half_day: row.get("allow_half_day"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_absence_plan(&self, row: &sqlx::postgres::PgRow) -> AbsencePlan {
        let accrual_rate: String = row.get("accrual_rate");
        let carry_over_max: Option<String> = row.try_get("carry_over_max").ok().flatten();
        let max_balance: Option<String> = row.try_get("max_balance").ok().flatten();
        AbsencePlan {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            absence_type_id: row.get("absence_type_id"),
            accrual_frequency: row.get("accrual_frequency"),
            accrual_rate,
            accrual_unit: row.get("accrual_unit"),
            carry_over_max,
            carry_over_expiry_months: row.try_get("carry_over_expiry_months").ok().flatten(),
            max_balance,
            probation_period_days: row.get("probation_period_days"),
            prorate_first_year: row.get("prorate_first_year"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_absence_balance(&self, row: &sqlx::postgres::PgRow) -> AbsenceBalance {
        AbsenceBalance {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            employee_id: row.get("employee_id"),
            plan_id: row.get("plan_id"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            accrued: row.get("accrued"),
            taken: row.get("taken"),
            adjusted: row.get("adjusted"),
            carried_over: row.get("carried_over"),
            remaining: row.get("remaining"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_absence_entry(&self, row: &sqlx::postgres::PgRow) -> AbsenceEntry {
        AbsenceEntry {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            absence_type_id: row.get("absence_type_id"),
            plan_id: row.get("plan_id"),
            entry_number: row.get("entry_number"),
            status: row.get("status"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            duration_days: row.get("duration_days"),
            duration_hours: row.try_get("duration_hours").ok().flatten(),
            is_half_day: row.get("is_half_day"),
            half_day_period: row.get("half_day_period"),
            reason: row.get("reason"),
            comments: row.get("comments"),
            documentation_provided: row.get("documentation_provided"),
            submitted_at: row.get("submitted_at"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            cancelled_reason: row.get("cancelled_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_history(&self, row: &sqlx::postgres::PgRow) -> AbsenceEntryHistory {
        AbsenceEntryHistory {
            id: row.get("id"),
            entry_id: row.get("entry_id"),
            action: row.get("action"),
            from_status: row.get("from_status"),
            to_status: row.get("to_status"),
            performed_by: row.get("performed_by"),
            comment: row.get("comment"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl AbsenceRepository for PostgresAbsenceRepository {
    // ========================================================================
    // Absence Types
    // ========================================================================

    async fn create_absence_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        plan_type: &str,
        requires_approval: bool,
        requires_documentation: bool,
        auto_approve_below_days: &str,
        allow_negative_balance: bool,
        allow_half_day: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceType> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.absence_types
                (organization_id, code, name, description, category, plan_type,
                 requires_approval, requires_documentation, auto_approve_below_days,
                 allow_negative_balance, allow_half_day, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, category = $5, plan_type = $6,
                    requires_approval = $7, requires_documentation = $8,
                    auto_approve_below_days = $9, allow_negative_balance = $10,
                    allow_half_day = $11, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(category)
        .bind(plan_type).bind(requires_approval).bind(requires_documentation)
        .bind(auto_approve_below_days).bind(allow_negative_balance)
        .bind(allow_half_day).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_absence_type(&row))
    }

    async fn get_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsenceType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.absence_types WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_type(&r)))
    }

    async fn list_absence_types(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<AbsenceType>> {
        let rows = match category {
            Some(c) => sqlx::query(
                "SELECT * FROM _atlas.absence_types WHERE organization_id = $1 AND category = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(c)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.absence_types WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_absence_type(r)).collect())
    }

    async fn delete_absence_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.absence_types SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Absence Plans
    // ========================================================================

    async fn create_absence_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        absence_type_id: Uuid,
        accrual_frequency: &str,
        accrual_rate: &str,
        accrual_unit: &str,
        carry_over_max: Option<String>,
        carry_over_expiry_months: Option<i32>,
        max_balance: Option<String>,
        probation_period_days: i32,
        prorate_first_year: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsencePlan> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.absence_plans
                (organization_id, code, name, description, absence_type_id,
                 accrual_frequency, accrual_rate, accrual_unit,
                 carry_over_max, carry_over_expiry_months, max_balance,
                 probation_period_days, prorate_first_year, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, absence_type_id = $5,
                    accrual_frequency = $6, accrual_rate = $7, accrual_unit = $8,
                    carry_over_max = $9, carry_over_expiry_months = $10,
                    max_balance = $11, probation_period_days = $12,
                    prorate_first_year = $13, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(absence_type_id).bind(accrual_frequency).bind(accrual_rate)
        .bind(accrual_unit)
        .bind(carry_over_max.as_deref()).bind(carry_over_expiry_months)
        .bind(max_balance.as_deref())
        .bind(probation_period_days).bind(prorate_first_year).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_absence_plan(&row))
    }

    async fn get_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AbsencePlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.absence_plans WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_plan(&r)))
    }

    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<AbsencePlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.absence_plans WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_plan(&r)))
    }

    async fn list_absence_plans(&self, org_id: Uuid, absence_type_id: Option<Uuid>) -> AtlasResult<Vec<AbsencePlan>> {
        let rows = match absence_type_id {
            Some(tid) => sqlx::query(
                "SELECT * FROM _atlas.absence_plans WHERE organization_id = $1 AND absence_type_id = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(tid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.absence_plans WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_absence_plan(r)).collect())
    }

    async fn delete_absence_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.absence_plans SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Absence Entries
    // ========================================================================

    async fn create_entry(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        absence_type_id: Uuid,
        plan_id: Option<Uuid>,
        entry_number: &str,
        status: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        duration_days: &str,
        duration_hours: Option<String>,
        is_half_day: bool,
        half_day_period: Option<&str>,
        reason: Option<&str>,
        comments: Option<&str>,
        documentation_provided: bool,
        approved_by: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AbsenceEntry> {
        let approved_by_uuid: Option<Uuid> = approved_by
            .and_then(|s| if s == "system" { None } else { Uuid::parse_str(s).ok() });

        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.absence_entries
                (organization_id, employee_id, employee_name,
                 absence_type_id, plan_id, entry_number, status,
                 start_date, end_date, duration_days, duration_hours,
                 is_half_day, half_day_period, reason, comments,
                 documentation_provided, approved_by,
                 approved_at, submitted_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, $15, $16,
                    $17, CASE WHEN $7 = 'approved' THEN now() ELSE NULL END,
                    CASE WHEN $7 IN ('submitted', 'approved') THEN now() ELSE NULL END,
                    $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(employee_id).bind(employee_name)
        .bind(absence_type_id).bind(plan_id).bind(entry_number).bind(status)
        .bind(start_date).bind(end_date).bind(duration_days).bind(duration_hours)
        .bind(is_half_day).bind(half_day_period).bind(reason).bind(comments)
        .bind(documentation_provided).bind(approved_by_uuid).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_absence_entry(&row))
    }

    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<AbsenceEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.absence_entries WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_entry(&r)))
    }

    async fn list_entries(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        absence_type_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<AbsenceEntry>> {
        let rows = match (employee_id, absence_type_id, status) {
            (Some(eid), Some(tid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 AND employee_id = $2 AND absence_type_id = $3 AND status = $4 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid).bind(tid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 AND employee_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None, None) => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 AND employee_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            (None, Some(tid), None) => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 AND absence_type_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(tid)
            .fetch_all(&self.pool).await,
            (None, None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.absence_entries WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_absence_entry(r)).collect())
    }

    async fn update_entry_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
        cancelled_reason: Option<&str>,
    ) -> AtlasResult<AbsenceEntry> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.absence_entries
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                rejected_reason = COALESCE($4, rejected_reason),
                cancelled_reason = COALESCE($5, cancelled_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .bind(rejected_reason).bind(cancelled_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_absence_entry(&row))
    }

    async fn find_overlapping_entries(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AbsenceEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.absence_entries
            WHERE organization_id = $1
              AND employee_id = $2
              AND status NOT IN ('cancelled', 'rejected')
              AND start_date <= $4 AND end_date >= $3
            "#,
        )
        .bind(org_id).bind(employee_id).bind(start_date).bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_absence_entry(r)).collect())
    }

    // ========================================================================
    // Absence Balances
    // ========================================================================

    async fn create_balance(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        accrued: &str,
        taken: &str,
        adjusted: &str,
        carried_over: &str,
        remaining: &str,
    ) -> AtlasResult<AbsenceBalance> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.absence_balances
                (organization_id, employee_id, plan_id,
                 period_start, period_end,
                 accrued, taken, adjusted, carried_over, remaining)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (employee_id, plan_id, period_start, period_end) DO UPDATE
                SET accrued = $6, taken = $7, adjusted = $8,
                    carried_over = $9, remaining = $10, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(employee_id).bind(plan_id)
        .bind(period_start).bind(period_end)
        .bind(accrued).bind(taken).bind(adjusted).bind(carried_over).bind(remaining)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_absence_balance(&row))
    }

    async fn get_balance(
        &self,
        employee_id: Uuid,
        plan_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<Option<AbsenceBalance>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.absence_balances WHERE employee_id = $1 AND plan_id = $2 AND period_start = $3 AND period_end = $4"
        )
        .bind(employee_id).bind(plan_id).bind(period_start).bind(period_end)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_balance(&r)))
    }

    async fn get_balance_for_previous_period(
        &self,
        employee_id: Uuid,
        plan_id: Uuid,
        current_period_start: chrono::NaiveDate,
    ) -> AtlasResult<Option<AbsenceBalance>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.absence_balances
            WHERE employee_id = $1 AND plan_id = $2 AND period_end < $3
            ORDER BY period_end DESC LIMIT 1
            "#,
        )
        .bind(employee_id).bind(plan_id).bind(current_period_start)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_absence_balance(&r)))
    }

    async fn list_balances(&self, org_id: Uuid, employee_id: Uuid) -> AtlasResult<Vec<AbsenceBalance>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.absence_balances WHERE organization_id = $1 AND employee_id = $2 ORDER BY period_start DESC"
        )
        .bind(org_id).bind(employee_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_absence_balance(r)).collect())
    }

    async fn update_balance(
        &self,
        id: Uuid,
        taken: &str,
        adjusted: &str,
        remaining: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.absence_balances SET taken = $2, adjusted = $3, remaining = $4, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(taken).bind(adjusted).bind(remaining)
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
        entry_id: Uuid,
        action: &str,
        from_status: Option<&str>,
        to_status: Option<&str>,
        performed_by: Option<Uuid>,
        comment: Option<&str>,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _atlas.absence_entry_history
                (entry_id, action, from_status, to_status, performed_by, comment)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(entry_id).bind(action).bind(from_status).bind(to_status)
        .bind(performed_by).bind(comment)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_entry_history(&self, entry_id: Uuid) -> AtlasResult<Vec<AbsenceEntryHistory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.absence_entry_history WHERE entry_id = $1 ORDER BY created_at ASC"
        )
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_history(r)).collect())
    }
}
