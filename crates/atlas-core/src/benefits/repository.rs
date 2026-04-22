//! Benefits Repository
//!
//! PostgreSQL storage for benefits plans, enrollments, and deductions.

use atlas_shared::{
    BenefitsPlan, BenefitsEnrollment, BenefitsDeduction,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for benefits data storage
#[async_trait]
pub trait BenefitsRepository: Send + Sync {
    // Benefits Plans
    async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        coverage_tiers: serde_json::Value,
        provider_name: Option<&str>,
        provider_plan_id: Option<&str>,
        plan_year_start: Option<chrono::NaiveDate>,
        plan_year_end: Option<chrono::NaiveDate>,
        open_enrollment_start: Option<chrono::NaiveDate>,
        open_enrollment_end: Option<chrono::NaiveDate>,
        allow_life_event_changes: bool,
        requires_eoi: bool,
        waiting_period_days: i32,
        max_dependents: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsPlan>;

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BenefitsPlan>>;
    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<BenefitsPlan>>;
    async fn list_plans(&self, org_id: Uuid, plan_type: Option<&str>) -> AtlasResult<Vec<BenefitsPlan>>;
    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Benefits Enrollments
    async fn create_enrollment(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        plan_id: Uuid,
        plan_code: Option<&str>,
        plan_name: Option<&str>,
        plan_type: Option<&str>,
        coverage_tier: &str,
        enrollment_type: &str,
        status: &str,
        effective_start_date: chrono::NaiveDate,
        effective_end_date: Option<chrono::NaiveDate>,
        employee_cost: &str,
        employer_cost: &str,
        total_cost: &str,
        deduction_frequency: &str,
        deduction_account_code: Option<&str>,
        employer_contribution_account_code: Option<&str>,
        dependents: serde_json::Value,
        life_event_reason: Option<&str>,
        life_event_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsEnrollment>;

    async fn get_enrollment(&self, id: Uuid) -> AtlasResult<Option<BenefitsEnrollment>>;
    async fn get_active_enrollment(&self, org_id: Uuid, employee_id: Uuid, plan_id: Uuid) -> AtlasResult<Option<BenefitsEnrollment>>;
    async fn list_enrollments(&self, org_id: Uuid, employee_id: Option<Uuid>, plan_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<BenefitsEnrollment>>;
    async fn update_enrollment_status(
        &self,
        id: Uuid,
        status: &str,
        processed_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BenefitsEnrollment>;
    async fn cancel_enrollment(
        &self,
        id: Uuid,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BenefitsEnrollment>;

    // Benefits Deductions
    async fn create_deduction(
        &self,
        org_id: Uuid,
        enrollment_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        plan_code: Option<&str>,
        plan_name: Option<&str>,
        employee_amount: &str,
        employer_amount: &str,
        total_amount: &str,
        pay_period_start: chrono::NaiveDate,
        pay_period_end: chrono::NaiveDate,
        deduction_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsDeduction>;

    async fn list_deductions(&self, org_id: Uuid, employee_id: Option<Uuid>, enrollment_id: Option<Uuid>) -> AtlasResult<Vec<BenefitsDeduction>>;
    async fn mark_deduction_processed(&self, id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresBenefitsRepository {
    pool: PgPool,
}

impl PostgresBenefitsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_plan(&self, row: &sqlx::postgres::PgRow) -> BenefitsPlan {
        BenefitsPlan {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            plan_type: row.get("plan_type"),
            coverage_tiers: row.get("coverage_tiers"),
            provider_name: row.get("provider_name"),
            provider_plan_id: row.get("provider_plan_id"),
            plan_year_start: row.get("plan_year_start"),
            plan_year_end: row.get("plan_year_end"),
            open_enrollment_start: row.get("open_enrollment_start"),
            open_enrollment_end: row.get("open_enrollment_end"),
            allow_life_event_changes: row.get("allow_life_event_changes"),
            requires_eoi: row.get("requires_eoi"),
            waiting_period_days: row.get("waiting_period_days"),
            max_dependents: row.try_get("max_dependents").ok().flatten(),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_enrollment(&self, row: &sqlx::postgres::PgRow) -> BenefitsEnrollment {
        let employee_cost: String = row.get("employee_cost");
        let employer_cost: String = row.get("employer_cost");
        let total_cost: String = row.get("total_cost");

        BenefitsEnrollment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            plan_id: row.get("plan_id"),
            plan_code: row.get("plan_code"),
            plan_name: row.get("plan_name"),
            plan_type: row.get("plan_type"),
            coverage_tier: row.get("coverage_tier"),
            enrollment_type: row.get("enrollment_type"),
            status: row.get("status"),
            effective_start_date: row.get("effective_start_date"),
            effective_end_date: row.get("effective_end_date"),
            employee_cost,
            employer_cost,
            total_cost,
            deduction_frequency: row.get("deduction_frequency"),
            deduction_account_code: row.get("deduction_account_code"),
            employer_contribution_account_code: row.get("employer_contribution_account_code"),
            dependents: row.get("dependents"),
            life_event_reason: row.get("life_event_reason"),
            life_event_date: row.get("life_event_date"),
            processed_by: row.get("processed_by"),
            processed_at: row.get("processed_at"),
            cancellation_reason: row.get("cancellation_reason"),
            cancelled_at: row.get("cancelled_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_deduction(&self, row: &sqlx::postgres::PgRow) -> BenefitsDeduction {
        let employee_amount: String = row.get("employee_amount");
        let employer_amount: String = row.get("employer_amount");
        let total_amount: String = row.get("total_amount");

        BenefitsDeduction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            enrollment_id: row.get("enrollment_id"),
            employee_id: row.get("employee_id"),
            plan_id: row.get("plan_id"),
            plan_code: row.get("plan_code"),
            plan_name: row.get("plan_name"),
            employee_amount,
            employer_amount,
            total_amount,
            pay_period_start: row.get("pay_period_start"),
            pay_period_end: row.get("pay_period_end"),
            deduction_account_code: row.get("deduction_account_code"),
            is_processed: row.get("is_processed"),
            processed_at: row.get("processed_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl BenefitsRepository for PostgresBenefitsRepository {
    // ========================================================================
    // Benefits Plans
    // ========================================================================

    async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        coverage_tiers: serde_json::Value,
        provider_name: Option<&str>,
        provider_plan_id: Option<&str>,
        plan_year_start: Option<chrono::NaiveDate>,
        plan_year_end: Option<chrono::NaiveDate>,
        open_enrollment_start: Option<chrono::NaiveDate>,
        open_enrollment_end: Option<chrono::NaiveDate>,
        allow_life_event_changes: bool,
        requires_eoi: bool,
        waiting_period_days: i32,
        max_dependents: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsPlan> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.benefits_plans
                (organization_id, code, name, description, plan_type,
                 coverage_tiers, provider_name, provider_plan_id,
                 plan_year_start, plan_year_end,
                 open_enrollment_start, open_enrollment_end,
                 allow_life_event_changes, requires_eoi,
                 waiting_period_days, max_dependents, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, plan_type = $5,
                    coverage_tiers = $6, provider_name = $7, provider_plan_id = $8,
                    plan_year_start = $9, plan_year_end = $10,
                    open_enrollment_start = $11, open_enrollment_end = $12,
                    allow_life_event_changes = $13, requires_eoi = $14,
                    waiting_period_days = $15, max_dependents = $16, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(plan_type)
        .bind(&coverage_tiers).bind(provider_name).bind(provider_plan_id)
        .bind(plan_year_start).bind(plan_year_end)
        .bind(open_enrollment_start).bind(open_enrollment_end)
        .bind(allow_life_event_changes).bind(requires_eoi)
        .bind(waiting_period_days).bind(max_dependents).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_plan(&row))
    }

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BenefitsPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.benefits_plans WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_plan(&r)))
    }

    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<BenefitsPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.benefits_plans WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_plan(&r)))
    }

    async fn list_plans(&self, org_id: Uuid, plan_type: Option<&str>) -> AtlasResult<Vec<BenefitsPlan>> {
        let rows = match plan_type {
            Some(pt) => sqlx::query(
                "SELECT * FROM _atlas.benefits_plans WHERE organization_id = $1 AND plan_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(pt)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.benefits_plans WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_plan(r)).collect())
    }

    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.benefits_plans SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Benefits Enrollments
    // ========================================================================

    async fn create_enrollment(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        plan_id: Uuid,
        plan_code: Option<&str>,
        plan_name: Option<&str>,
        plan_type: Option<&str>,
        coverage_tier: &str,
        enrollment_type: &str,
        status: &str,
        effective_start_date: chrono::NaiveDate,
        effective_end_date: Option<chrono::NaiveDate>,
        employee_cost: &str,
        employer_cost: &str,
        total_cost: &str,
        deduction_frequency: &str,
        deduction_account_code: Option<&str>,
        employer_contribution_account_code: Option<&str>,
        dependents: serde_json::Value,
        life_event_reason: Option<&str>,
        life_event_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsEnrollment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.benefits_enrollments
                (organization_id, employee_id, employee_name,
                 plan_id, plan_code, plan_name, plan_type,
                 coverage_tier, enrollment_type, status,
                 effective_start_date, effective_end_date,
                 employee_cost, employer_cost, total_cost,
                 deduction_frequency, deduction_account_code,
                 employer_contribution_account_code,
                 dependents, life_event_reason, life_event_date,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15,
                    $16, $17, $18, $19, $20, $21, $22)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(employee_id).bind(employee_name)
        .bind(plan_id).bind(plan_code).bind(plan_name).bind(plan_type)
        .bind(coverage_tier).bind(enrollment_type).bind(status)
        .bind(effective_start_date).bind(effective_end_date)
        .bind(employee_cost).bind(employer_cost).bind(total_cost)
        .bind(deduction_frequency).bind(deduction_account_code)
        .bind(employer_contribution_account_code)
        .bind(&dependents).bind(life_event_reason).bind(life_event_date)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_enrollment(&row))
    }

    async fn get_enrollment(&self, id: Uuid) -> AtlasResult<Option<BenefitsEnrollment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.benefits_enrollments WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_enrollment(&r)))
    }

    async fn get_active_enrollment(&self, org_id: Uuid, employee_id: Uuid, plan_id: Uuid) -> AtlasResult<Option<BenefitsEnrollment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND employee_id = $2 AND plan_id = $3 AND status IN ('active', 'pending')"
        )
        .bind(org_id).bind(employee_id).bind(plan_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_enrollment(&r)))
    }

    async fn list_enrollments(&self, org_id: Uuid, employee_id: Option<Uuid>, plan_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<BenefitsEnrollment>> {
        let rows = match (employee_id, plan_id, status) {
            (Some(eid), Some(pid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND employee_id = $2 AND plan_id = $3 AND status = $4 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid).bind(pid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND employee_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None, None) => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND employee_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            (None, Some(pid), None) => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND plan_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(pid)
            .fetch_all(&self.pool).await,
            (None, None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.benefits_enrollments WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_enrollment(r)).collect())
    }

    async fn update_enrollment_status(
        &self,
        id: Uuid,
        status: &str,
        processed_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BenefitsEnrollment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.benefits_enrollments
            SET status = $2, processed_by = COALESCE($3, processed_by),
                processed_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE processed_at END,
                cancellation_reason = COALESCE($4, cancellation_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(processed_by).bind(cancellation_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_enrollment(&row))
    }

    async fn cancel_enrollment(
        &self,
        id: Uuid,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BenefitsEnrollment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.benefits_enrollments
            SET status = 'cancelled', cancellation_reason = $2,
                cancelled_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(cancellation_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_enrollment(&row))
    }

    // ========================================================================
    // Benefits Deductions
    // ========================================================================

    async fn create_deduction(
        &self,
        org_id: Uuid,
        enrollment_id: Uuid,
        employee_id: Uuid,
        plan_id: Uuid,
        plan_code: Option<&str>,
        plan_name: Option<&str>,
        employee_amount: &str,
        employer_amount: &str,
        total_amount: &str,
        pay_period_start: chrono::NaiveDate,
        pay_period_end: chrono::NaiveDate,
        deduction_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenefitsDeduction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.benefits_deductions
                (organization_id, enrollment_id, employee_id, plan_id,
                 plan_code, plan_name,
                 employee_amount, employer_amount, total_amount,
                 pay_period_start, pay_period_end,
                 deduction_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(enrollment_id).bind(employee_id).bind(plan_id)
        .bind(plan_code).bind(plan_name)
        .bind(employee_amount).bind(employer_amount).bind(total_amount)
        .bind(pay_period_start).bind(pay_period_end)
        .bind(deduction_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_deduction(&row))
    }

    async fn list_deductions(&self, org_id: Uuid, employee_id: Option<Uuid>, enrollment_id: Option<Uuid>) -> AtlasResult<Vec<BenefitsDeduction>> {
        let rows = match (employee_id, enrollment_id) {
            (Some(eid), Some(enid)) => sqlx::query(
                "SELECT * FROM _atlas.benefits_deductions WHERE organization_id = $1 AND employee_id = $2 AND enrollment_id = $3 ORDER BY pay_period_start DESC"
            )
            .bind(org_id).bind(eid).bind(enid)
            .fetch_all(&self.pool).await,
            (Some(eid), None) => sqlx::query(
                "SELECT * FROM _atlas.benefits_deductions WHERE organization_id = $1 AND employee_id = $2 ORDER BY pay_period_start DESC"
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            (None, Some(enid)) => sqlx::query(
                "SELECT * FROM _atlas.benefits_deductions WHERE organization_id = $1 AND enrollment_id = $2 ORDER BY pay_period_start DESC"
            )
            .bind(org_id).bind(enid)
            .fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.benefits_deductions WHERE organization_id = $1 ORDER BY pay_period_start DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_deduction(r)).collect())
    }

    async fn mark_deduction_processed(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.benefits_deductions SET is_processed = true, processed_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
