//! Allocation Repository
//!
//! PostgreSQL storage for GL allocation pools, bases, rules, and runs.

use atlas_shared::{
    GlAllocationPool, GlAllocationBasis, GlAllocationBasisDetail,
    GlAllocationRule, GlAllocationTargetLine,
    GlAllocationRun, GlAllocationRunLine,
    GlAllocationDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for GL allocation storage
#[async_trait]
pub trait AllocationRepository: Send + Sync {
    // ── Pools ──────────────────────────────────────────────────
    async fn create_pool(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_type: &str,
        source_account_code: Option<&str>,
        source_account_range_from: Option<&str>,
        source_account_range_to: Option<&str>,
        source_department_id: Option<Uuid>,
        source_project_id: Option<Uuid>,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationPool>;
    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationPool>>;
    async fn get_pool_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationPool>>;
    async fn list_pools(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationPool>>;
    async fn update_pool_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationPool>;
    async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Bases ───────────────────────────────────────────────────
    async fn create_basis(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        basis_type: &str, unit_of_measure: Option<&str>,
        is_manual: bool, source_account_code: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasis>;
    async fn get_basis_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationBasis>>;
    async fn get_basis_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationBasis>>;
    async fn list_bases(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationBasis>>;
    async fn update_basis_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationBasis>;
    async fn delete_basis(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Basis Details ──────────────────────────────────────────
    async fn create_basis_detail(
        &self,
        org_id: Uuid, basis_id: Uuid,
        target_department_id: Option<Uuid>,
        target_department_name: Option<&str>,
        target_cost_center: Option<&str>,
        target_project_id: Option<Uuid>,
        target_project_name: Option<&str>,
        target_account_code: Option<&str>,
        basis_amount: &str,
        period_name: Option<&str>,
        period_start_date: Option<chrono::NaiveDate>,
        period_end_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasisDetail>;
    async fn get_basis_detail_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationBasisDetail>>;
    async fn list_basis_details(&self, basis_id: Uuid, period_name: Option<&str>) -> AtlasResult<Vec<GlAllocationBasisDetail>>;
    async fn update_basis_detail_amount(&self, id: Uuid, basis_amount: &str) -> AtlasResult<GlAllocationBasisDetail>;
    async fn update_basis_detail_percentage(&self, id: Uuid, percentage: &str) -> AtlasResult<()>;
    async fn delete_basis_detail(&self, id: Uuid) -> AtlasResult<()>;

    // ── Rules ───────────────────────────────────────────────────
    async fn create_rule(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_id: Uuid, pool_code: &str, basis_id: Uuid, basis_code: &str,
        allocation_method: &str, offset_method: &str,
        offset_account_code: Option<&str>,
        journal_batch_prefix: Option<&str>,
        round_to_largest: bool,
        minimum_threshold: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRule>;
    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationRule>>;
    async fn get_rule_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationRule>>;
    async fn list_rules(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationRule>>;
    async fn update_rule_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationRule>;
    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Target Lines ────────────────────────────────────────────
    async fn create_target_line(
        &self,
        org_id: Uuid, rule_id: Uuid, line_number: i32,
        target_department_id: Option<Uuid>,
        target_department_name: Option<&str>,
        target_cost_center: Option<&str>,
        target_project_id: Option<Uuid>,
        target_project_name: Option<&str>,
        target_account_code: &str,
        target_account_name: Option<&str>,
        fixed_percentage: Option<&str>,
        is_active: bool,
    ) -> AtlasResult<GlAllocationTargetLine>;
    async fn list_target_lines(&self, rule_id: Uuid) -> AtlasResult<Vec<GlAllocationTargetLine>>;

    // ── Runs ────────────────────────────────────────────────────
    async fn create_run(
        &self,
        org_id: Uuid, run_number: &str,
        rule_id: Uuid, rule_code: &str, rule_name: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        pool_amount: &str,
        allocation_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRun>;
    async fn get_run_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlAllocationRun>>;
    async fn update_run_status(&self, id: Uuid, status: &str, acted_by: Option<Uuid>) -> AtlasResult<GlAllocationRun>;
    async fn update_run_totals(&self, id: Uuid) -> AtlasResult<GlAllocationRun>;

    // ── Run Lines ───────────────────────────────────────────────
    async fn create_run_line(
        &self,
        org_id: Uuid, run_id: Uuid, line_number: i32,
        target_department_id: Option<Uuid>,
        target_department_name: Option<&str>,
        target_cost_center: Option<&str>,
        target_project_id: Option<Uuid>,
        target_project_name: Option<&str>,
        target_account_code: &str,
        target_account_name: Option<&str>,
        source_account_code: Option<&str>,
        basis_amount: &str,
        basis_percentage: &str,
        allocated_amount: &str,
        offset_amount: &str,
        line_type: &str,
    ) -> AtlasResult<GlAllocationRunLine>;

    // ── Dashboard ───────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<GlAllocationDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresAllocationRepository {
    pool: PgPool,
}

impl PostgresAllocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_pool(&self, row: &sqlx::postgres::PgRow) -> GlAllocationPool {
        GlAllocationPool {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            pool_type: row.get("pool_type"),
            source_account_code: row.get("source_account_code"),
            source_account_range_from: row.get("source_account_range_from"),
            source_account_range_to: row.get("source_account_range_to"),
            source_department_id: row.get("source_department_id"),
            source_project_id: row.get("source_project_id"),
            currency_code: row.get("currency_code"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_basis(&self, row: &sqlx::postgres::PgRow) -> GlAllocationBasis {
        GlAllocationBasis {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            basis_type: row.get("basis_type"),
            unit_of_measure: row.get("unit_of_measure"),
            is_manual: row.get("is_manual"),
            source_account_code: row.get("source_account_code"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_basis_detail(&self, row: &sqlx::postgres::PgRow) -> GlAllocationBasisDetail {
        GlAllocationBasisDetail {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            basis_id: row.get("basis_id"),
            target_department_id: row.get("target_department_id"),
            target_department_name: row.get("target_department_name"),
            target_cost_center: row.get("target_cost_center"),
            target_project_id: row.get("target_project_id"),
            target_project_name: row.get("target_project_name"),
            target_account_code: row.get("target_account_code"),
            basis_amount: row.get("basis_amount"),
            percentage: row.get("percentage"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_target_line(&self, row: &sqlx::postgres::PgRow) -> GlAllocationTargetLine {
        GlAllocationTargetLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rule_id: row.get("rule_id"),
            line_number: row.get("line_number"),
            target_department_id: row.get("target_department_id"),
            target_department_name: row.get("target_department_name"),
            target_cost_center: row.get("target_cost_center"),
            target_project_id: row.get("target_project_id"),
            target_project_name: row.get("target_project_name"),
            target_account_code: row.get("target_account_code"),
            target_account_name: row.get("target_account_name"),
            fixed_percentage: row.get("fixed_percentage"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_rule(&self, row: &sqlx::postgres::PgRow) -> GlAllocationRule {
        GlAllocationRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            pool_id: row.get("pool_id"),
            pool_code: row.get("pool_code"),
            basis_id: row.get("basis_id"),
            basis_code: row.get("basis_code"),
            allocation_method: row.get("allocation_method"),
            offset_method: row.get("offset_method"),
            offset_account_code: row.get("offset_account_code"),
            journal_batch_prefix: row.get("journal_batch_prefix"),
            round_to_largest: row.get("round_to_largest"),
            minimum_threshold: row.get("minimum_threshold"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            target_lines: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_run(&self, row: &sqlx::postgres::PgRow) -> GlAllocationRun {
        GlAllocationRun {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            run_number: row.get("run_number"),
            rule_id: row.get("rule_id"),
            rule_code: row.get("rule_code"),
            rule_name: row.get("rule_name"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            pool_amount: row.get("pool_amount"),
            allocation_method: row.get("allocation_method"),
            total_allocated: row.get("total_allocated"),
            total_offset: row.get("total_offset"),
            rounding_difference: row.get("rounding_difference"),
            target_count: row.get("target_count"),
            journal_batch_id: row.get("journal_batch_id"),
            journal_batch_name: row.get("journal_batch_name"),
            status: row.get("status"),
            run_date: row.get("run_date"),
            posted_at: row.get("posted_at"),
            reversed_at: row.get("reversed_at"),
            posted_by: row.get("posted_by"),
            results: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_run_line(&self, row: &sqlx::postgres::PgRow) -> GlAllocationRunLine {
        GlAllocationRunLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            run_id: row.get("run_id"),
            line_number: row.get("line_number"),
            target_department_id: row.get("target_department_id"),
            target_department_name: row.get("target_department_name"),
            target_cost_center: row.get("target_cost_center"),
            target_project_id: row.get("target_project_id"),
            target_project_name: row.get("target_project_name"),
            target_account_code: row.get("target_account_code"),
            target_account_name: row.get("target_account_name"),
            source_account_code: row.get("source_account_code"),
            basis_amount: row.get("basis_amount"),
            basis_percentage: row.get("basis_percentage"),
            allocated_amount: row.get("allocated_amount"),
            offset_amount: row.get("offset_amount"),
            line_type: row.get("line_type"),
            journal_line_id: row.get("journal_line_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl AllocationRepository for PostgresAllocationRepository {
    // ── Pools ──────────────────────────────────────────────────

    async fn create_pool(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_type: &str, source_account_code: Option<&str>,
        source_account_range_from: Option<&str>, source_account_range_to: Option<&str>,
        source_department_id: Option<Uuid>, source_project_id: Option<Uuid>,
        currency_code: &str, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationPool> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_pools
                (organization_id, code, name, description, pool_type,
                 source_account_code, source_account_range_from, source_account_range_to,
                 source_department_id, source_project_id, currency_code,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(pool_type)
        .bind(source_account_code).bind(source_account_range_from).bind(source_account_range_to)
        .bind(source_department_id).bind(source_project_id).bind(currency_code)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_pool(&row))
    }

    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_pools WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_pool(&r)))
    }

    async fn get_pool_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_pools WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_pool(&r)))
    }

    async fn list_pools(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationPool>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_pools WHERE organization_id = $1 AND is_active = true ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_pools WHERE organization_id = $1 ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_pool(r)).collect())
    }

    async fn update_pool_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationPool> {
        let row = sqlx::query(
            "UPDATE _atlas.gl_allocation_pools SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_pool(&row))
    }

    async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.gl_allocation_pools WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Bases ─────────────────────────────────────────────────

    async fn create_basis(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        basis_type: &str, unit_of_measure: Option<&str>, is_manual: bool,
        source_account_code: Option<&str>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasis> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_bases
                (organization_id, code, name, description, basis_type,
                 unit_of_measure, is_manual, source_account_code,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(basis_type)
        .bind(unit_of_measure).bind(is_manual).bind(source_account_code)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_basis(&row))
    }

    async fn get_basis_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationBasis>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_bases WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_basis(&r)))
    }

    async fn get_basis_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationBasis>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_bases WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_basis(&r)))
    }

    async fn list_bases(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationBasis>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_bases WHERE organization_id = $1 AND is_active = true ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_bases WHERE organization_id = $1 ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_basis(r)).collect())
    }

    async fn update_basis_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationBasis> {
        let row = sqlx::query(
            "UPDATE _atlas.gl_allocation_bases SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_basis(&row))
    }

    async fn delete_basis(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.gl_allocation_bases WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Basis Details ─────────────────────────────────────────

    async fn create_basis_detail(
        &self, org_id: Uuid, basis_id: Uuid,
        target_department_id: Option<Uuid>, target_department_name: Option<&str>,
        target_cost_center: Option<&str>, target_project_id: Option<Uuid>,
        target_project_name: Option<&str>, target_account_code: Option<&str>,
        basis_amount: &str, period_name: Option<&str>,
        period_start_date: Option<chrono::NaiveDate>, period_end_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasisDetail> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_basis_details
                (organization_id, basis_id,
                 target_department_id, target_department_name, target_cost_center,
                 target_project_id, target_project_name, target_account_code,
                 basis_amount, percentage, period_name, period_start_date, period_end_date, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'0',$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(basis_id)
        .bind(target_department_id).bind(target_department_name).bind(target_cost_center)
        .bind(target_project_id).bind(target_project_name).bind(target_account_code)
        .bind(basis_amount).bind(period_name).bind(period_start_date).bind(period_end_date)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_basis_detail(&row))
    }

    async fn get_basis_detail_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationBasisDetail>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_basis_details WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_basis_detail(&r)))
    }

    async fn list_basis_details(&self, basis_id: Uuid, period_name: Option<&str>) -> AtlasResult<Vec<GlAllocationBasisDetail>> {
        let rows = if period_name.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.gl_allocation_basis_details WHERE basis_id = $1 AND period_name = $2 AND is_active = true ORDER BY id"
            ).bind(basis_id).bind(period_name)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.gl_allocation_basis_details WHERE basis_id = $1 AND is_active = true ORDER BY id"
            ).bind(basis_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_basis_detail(r)).collect())
    }

    async fn update_basis_detail_amount(&self, id: Uuid, basis_amount: &str) -> AtlasResult<GlAllocationBasisDetail> {
        let row = sqlx::query(
            "UPDATE _atlas.gl_allocation_basis_details SET basis_amount = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(basis_amount)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_basis_detail(&row))
    }

    async fn update_basis_detail_percentage(&self, id: Uuid, percentage: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.gl_allocation_basis_details SET percentage = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(percentage)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_basis_detail(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.gl_allocation_basis_details WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Rules ─────────────────────────────────────────────────

    async fn create_rule(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_id: Uuid, pool_code: &str, basis_id: Uuid, basis_code: &str,
        allocation_method: &str, offset_method: &str, offset_account_code: Option<&str>,
        journal_batch_prefix: Option<&str>, round_to_largest: bool,
        minimum_threshold: Option<&str>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_rules
                (organization_id, code, name, description,
                 pool_id, pool_code, basis_id, basis_code,
                 allocation_method, offset_method, offset_account_code,
                 journal_batch_prefix, round_to_largest, minimum_threshold,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(pool_id).bind(pool_code).bind(basis_id).bind(basis_code)
        .bind(allocation_method).bind(offset_method).bind(offset_account_code)
        .bind(journal_batch_prefix).bind(round_to_largest).bind(minimum_threshold)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_rule(&row))
    }

    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_rules WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let rule_id: Uuid = r.get("id");
                let mut rule = self.row_to_rule(&r);
                let lines = sqlx::query(
                    "SELECT * FROM _atlas.gl_allocation_target_lines WHERE rule_id = $1 ORDER BY line_number"
                ).bind(rule_id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                rule.target_lines = lines.iter().map(|l| self.row_to_target_line(l)).collect();
                Ok(Some(rule))
            }
            None => Ok(None),
        }
    }

    async fn get_rule_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_rules WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let rule_id: Uuid = r.get("id");
                let mut rule = self.row_to_rule(&r);
                let lines = sqlx::query(
                    "SELECT * FROM _atlas.gl_allocation_target_lines WHERE rule_id = $1 ORDER BY line_number"
                ).bind(rule_id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                rule.target_lines = lines.iter().map(|l| self.row_to_target_line(l)).collect();
                Ok(Some(rule))
            }
            None => Ok(None),
        }
    }

    async fn list_rules(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationRule>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_rules WHERE organization_id = $1 AND is_active = true ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_rules WHERE organization_id = $1 ORDER BY code")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            let mut rule = self.row_to_rule(&row);
            let rule_id: Uuid = row.get("id");
            let lines = sqlx::query(
                "SELECT * FROM _atlas.gl_allocation_target_lines WHERE rule_id = $1 ORDER BY line_number"
            ).bind(rule_id).fetch_all(&self.pool).await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            rule.target_lines = lines.iter().map(|l| self.row_to_target_line(l)).collect();
            rules.push(rule);
        }
        Ok(rules)
    }

    async fn update_rule_active(&self, id: Uuid, is_active: bool) -> AtlasResult<GlAllocationRule> {
        let row = sqlx::query(
            "UPDATE _atlas.gl_allocation_rules SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_rule(&row))
    }

    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        // Delete target lines first
        sqlx::query(
            r#"DELETE FROM _atlas.gl_allocation_target_lines WHERE rule_id IN
               (SELECT id FROM _atlas.gl_allocation_rules WHERE organization_id = $1 AND code = $2)"#
        ).bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM _atlas.gl_allocation_rules WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Target Lines ──────────────────────────────────────────

    async fn create_target_line(
        &self, org_id: Uuid, rule_id: Uuid, line_number: i32,
        target_department_id: Option<Uuid>, target_department_name: Option<&str>,
        target_cost_center: Option<&str>, target_project_id: Option<Uuid>,
        target_project_name: Option<&str>, target_account_code: &str,
        target_account_name: Option<&str>, fixed_percentage: Option<&str>,
        is_active: bool,
    ) -> AtlasResult<GlAllocationTargetLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_target_lines
                (organization_id, rule_id, line_number,
                 target_department_id, target_department_name, target_cost_center,
                 target_project_id, target_project_name, target_account_code,
                 target_account_name, fixed_percentage, is_active)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING *"#,
        )
        .bind(org_id).bind(rule_id).bind(line_number)
        .bind(target_department_id).bind(target_department_name).bind(target_cost_center)
        .bind(target_project_id).bind(target_project_name).bind(target_account_code)
        .bind(target_account_name).bind(fixed_percentage).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_target_line(&row))
    }

    async fn list_target_lines(&self, rule_id: Uuid) -> AtlasResult<Vec<GlAllocationTargetLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.gl_allocation_target_lines WHERE rule_id = $1 ORDER BY line_number"
        ).bind(rule_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_target_line(r)).collect())
    }

    // ── Runs ─────────────────────────────────────────────────

    async fn create_run(
        &self, org_id: Uuid, run_number: &str,
        rule_id: Uuid, rule_code: &str, rule_name: &str,
        period_name: &str, period_start_date: chrono::NaiveDate, period_end_date: chrono::NaiveDate,
        pool_amount: &str, allocation_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRun> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_runs
                (organization_id, run_number, rule_id, rule_code, rule_name,
                 period_name, period_start_date, period_end_date,
                 pool_amount, allocation_method, total_allocated, total_offset,
                 rounding_difference, target_count, status, run_date, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'0','0','0',0,'draft',CURRENT_DATE,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(run_number).bind(rule_id).bind(rule_code).bind(rule_name)
        .bind(period_name).bind(period_start_date).bind(period_end_date)
        .bind(pool_amount).bind(allocation_method).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_run(&row))
    }

    async fn get_run_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_allocation_runs WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut run = self.row_to_run(&r);
                let lines = sqlx::query(
                    "SELECT * FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 ORDER BY line_number"
                ).bind(id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                run.results = lines.iter().map(|l| self.row_to_run_line(l)).collect();
                Ok(Some(run))
            }
            None => Ok(None),
        }
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlAllocationRun>> {
        let rows = if status.is_some() {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_runs WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC")
                .bind(org_id).bind(status)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.gl_allocation_runs WHERE organization_id = $1 ORDER BY created_at DESC")
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut runs = Vec::new();
        for row in rows {
            let mut run = self.row_to_run(&row);
            let run_id: Uuid = row.get("id");
            let lines = sqlx::query(
                "SELECT * FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 ORDER BY line_number"
            ).bind(run_id).fetch_all(&self.pool).await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            run.results = lines.iter().map(|l| self.row_to_run_line(l)).collect();
            runs.push(run);
        }
        Ok(runs)
    }

    async fn update_run_status(&self, id: Uuid, status: &str, acted_by: Option<Uuid>) -> AtlasResult<GlAllocationRun> {
        let row = if status == "posted" {
            sqlx::query(
                r#"UPDATE _atlas.gl_allocation_runs SET status = $2, posted_at = now(), posted_by = $3, updated_at = now() WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).bind(acted_by)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else if status == "reversed" {
            sqlx::query(
                r#"UPDATE _atlas.gl_allocation_runs SET status = $2, reversed_at = now(), updated_at = now() WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "UPDATE _atlas.gl_allocation_runs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(self.row_to_run(&row))
    }

    async fn update_run_totals(&self, id: Uuid) -> AtlasResult<GlAllocationRun> {
        // Calculate totals from run lines
        let row = sqlx::query(
            r#"UPDATE _atlas.gl_allocation_runs SET
                total_allocated = (SELECT COALESCE(SUM(allocated_amount::numeric), 0) FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 AND line_type = 'allocation'),
                total_offset = (SELECT COALESCE(SUM(allocated_amount::numeric), 0) FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 AND line_type = 'offset'),
                rounding_difference = (SELECT pool_amount::numeric FROM _atlas.gl_allocation_runs WHERE id = $1) -
                    (SELECT COALESCE(SUM(allocated_amount::numeric), 0) FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 AND line_type = 'allocation'),
                target_count = (SELECT COUNT(*)::int FROM _atlas.gl_allocation_run_lines WHERE run_id = $1 AND line_type = 'allocation'),
                updated_at = now()
            WHERE id = $1 RETURNING *"#,
        ).bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_run(&row))
    }

    // ── Run Lines ─────────────────────────────────────────────

    async fn create_run_line(
        &self, org_id: Uuid, run_id: Uuid, line_number: i32,
        target_department_id: Option<Uuid>, target_department_name: Option<&str>,
        target_cost_center: Option<&str>, target_project_id: Option<Uuid>,
        target_project_name: Option<&str>, target_account_code: &str,
        target_account_name: Option<&str>, source_account_code: Option<&str>,
        basis_amount: &str, basis_percentage: &str, allocated_amount: &str,
        offset_amount: &str, line_type: &str,
    ) -> AtlasResult<GlAllocationRunLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_allocation_run_lines
                (organization_id, run_id, line_number,
                 target_department_id, target_department_name, target_cost_center,
                 target_project_id, target_project_name, target_account_code,
                 target_account_name, source_account_code,
                 basis_amount, basis_percentage, allocated_amount, offset_amount, line_type)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            RETURNING *"#,
        )
        .bind(org_id).bind(run_id).bind(line_number)
        .bind(target_department_id).bind(target_department_name).bind(target_cost_center)
        .bind(target_project_id).bind(target_project_name).bind(target_account_code)
        .bind(target_account_name).bind(source_account_code)
        .bind(basis_amount).bind(basis_percentage).bind(allocated_amount)
        .bind(offset_amount).bind(line_type)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_run_line(&row))
    }

    // ── Dashboard ─────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<GlAllocationDashboardSummary> {
        let pool_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE is_active) as active FROM _atlas.gl_allocation_pools WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let basis_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE is_active) as active FROM _atlas.gl_allocation_bases WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let rule_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE is_active) as active FROM _atlas.gl_allocation_rules WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let run_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE status = 'posted') as posted, COUNT(*) FILTER (WHERE status = 'draft') as draft, COALESCE(SUM(total_allocated::numeric), 0) as total_allocated FROM _atlas.gl_allocation_runs WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let pool_type_rows = sqlx::query(
            "SELECT pool_type, COUNT(*) as cnt FROM _atlas.gl_allocation_pools WHERE organization_id = $1 GROUP BY pool_type"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let method_rows = sqlx::query(
            "SELECT allocation_method, COUNT(*) as cnt FROM _atlas.gl_allocation_rules WHERE organization_id = $1 GROUP BY allocation_method"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut pools_by_type = serde_json::Map::new();
        for r in &pool_type_rows {
            let pt: String = r.get("pool_type");
            let cnt: i64 = r.get("cnt");
            pools_by_type.insert(pt, serde_json::json!(cnt));
        }

        let mut rules_by_method = serde_json::Map::new();
        for r in &method_rows {
            let am: String = r.get("allocation_method");
            let cnt: i64 = r.get("cnt");
            rules_by_method.insert(am, serde_json::json!(cnt));
        }

        let total_pools: i64 = pool_row.try_get("total").unwrap_or(0);
        let active_pools: i64 = pool_row.try_get("active").unwrap_or(0);
        let total_bases: i64 = basis_row.try_get("total").unwrap_or(0);
        let active_bases: i64 = basis_row.try_get("active").unwrap_or(0);
        let total_rules: i64 = rule_row.try_get("total").unwrap_or(0);
        let active_rules: i64 = rule_row.try_get("active").unwrap_or(0);
        let total_runs: i64 = run_row.try_get("total").unwrap_or(0);
        let posted_runs: i64 = run_row.try_get("posted").unwrap_or(0);
        let draft_runs: i64 = run_row.try_get("draft").unwrap_or(0);
        let total_allocated: serde_json::Value = run_row.try_get("total_allocated").unwrap_or(serde_json::json!("0"));

        Ok(GlAllocationDashboardSummary {
            total_pools: total_pools as i32,
            active_pools: active_pools as i32,
            total_bases: total_bases as i32,
            active_bases: active_bases as i32,
            total_rules: total_rules as i32,
            active_rules: active_rules as i32,
            total_runs: total_runs as i32,
            posted_runs: posted_runs as i32,
            draft_runs: draft_runs as i32,
            total_allocated_amount: total_allocated.to_string(),
            pools_by_type: serde_json::Value::Object(pools_by_type),
            rules_by_method: serde_json::Value::Object(rules_by_method),
        })
    }
}