//! Cost Allocation Repository
//!
//! PostgreSQL storage for allocation pools, bases, rules, runs, and lines.

use atlas_shared::{
    AllocationPool, AllocationBase, AllocationBaseValue,
    AllocationRule, AllocationRuleTarget,
    AllocationRun, AllocationRunLine,
    AllocationSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for cost allocation data storage
#[async_trait]
pub trait CostAllocationRepository: Send + Sync {
    // ── Allocation Pools ──────────────────────────────────────────────
    async fn create_pool(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        pool_type: &str,
        source_account_codes: serde_json::Value,
        source_department_id: Option<Uuid>,
        source_cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationPool>;

    async fn get_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationPool>>;
    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<AllocationPool>>;
    async fn list_pools(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationPool>>;
    async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Allocation Bases ──────────────────────────────────────────────
    async fn create_base(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        base_type: &str,
        financial_account_code: Option<&str>,
        unit_of_measure: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBase>;

    async fn get_base(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationBase>>;
    async fn get_base_by_id(&self, id: Uuid) -> AtlasResult<Option<AllocationBase>>;
    async fn list_bases(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationBase>>;
    async fn delete_base(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Base Values ───────────────────────────────────────────────────
    async fn upsert_base_value(
        &self,
        org_id: Uuid,
        base_id: Uuid,
        base_code: &str,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        value: &str,
        effective_date: chrono::NaiveDate,
        source: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBaseValue>;

    async fn get_base_values(
        &self,
        org_id: Uuid,
        base_id: Uuid,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AllocationBaseValue>>;

    async fn list_base_values(
        &self,
        org_id: Uuid,
        base_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AllocationBaseValue>>;

    // ── Allocation Rules ──────────────────────────────────────────────
    async fn create_rule(
        &self,
        org_id: Uuid,
        rule_number: &str,
        name: &str,
        description: Option<&str>,
        pool_id: Uuid,
        pool_code: &str,
        base_id: Uuid,
        base_code: &str,
        allocation_method: &str,
        journal_description: Option<&str>,
        offset_account_code: Option<&str>,
        currency_code: &str,
        is_reversing: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRule>;

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<AllocationRule>>;
    async fn get_rule_by_number(&self, org_id: Uuid, rule_number: &str) -> AtlasResult<Option<AllocationRule>>;
    async fn list_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AllocationRule>>;
    async fn update_rule_status(&self, id: Uuid, status: &str) -> AtlasResult<AllocationRule>;

    // ── Rule Targets ──────────────────────────────────────────────────
    async fn create_rule_target(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        line_number: i32,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        target_account_code: &str,
        fixed_percent: Option<&str>,
        fixed_amount: Option<&str>,
    ) -> AtlasResult<AllocationRuleTarget>;

    async fn list_rule_targets(&self, rule_id: Uuid) -> AtlasResult<Vec<AllocationRuleTarget>>;
    async fn delete_rule_targets(&self, rule_id: Uuid) -> AtlasResult<()>;

    // ── Allocation Runs ───────────────────────────────────────────────
    async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        rule_id: Uuid,
        rule_name: &str,
        rule_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        total_source_amount: &str,
        total_allocated_amount: &str,
        line_count: i32,
        run_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRun>;

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<AllocationRun>>;
    async fn list_runs(&self, org_id: Uuid, rule_id: Option<Uuid>) -> AtlasResult<Vec<AllocationRun>>;
    async fn update_run_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
        reversed_by_id: Option<Uuid>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<AllocationRun>;

    // ── Run Lines ─────────────────────────────────────────────────────
    async fn create_run_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        line_number: i32,
        line_type: &str,
        account_code: &str,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        amount: &str,
        base_value_used: Option<&str>,
        percent_of_total: Option<&str>,
        description: Option<&str>,
    ) -> AtlasResult<AllocationRunLine>;

    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<AllocationRunLine>>;

    // ── Dashboard ─────────────────────────────────────────────────────
    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<AllocationSummary>;
}

/// PostgreSQL implementation
pub struct PostgresCostAllocationRepository {
    pool: PgPool,
}

impl PostgresCostAllocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CostAllocationRepository for PostgresCostAllocationRepository {
    async fn create_pool(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        pool_type: &str,
        source_account_codes: serde_json::Value,
        source_department_id: Option<Uuid>,
        source_cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationPool> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_pools
                (organization_id, code, name, description, pool_type,
                 source_account_codes, source_department_id, source_cost_center,
                 is_active, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(pool_type)
        .bind(&source_account_codes).bind(source_department_id).bind(source_cost_center)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AllocationPool {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            pool_type: row.get("pool_type"),
            source_account_codes: row.get("source_account_codes"),
            source_department_id: row.get("source_department_id"),
            source_cost_center: row.get("source_cost_center"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationPool>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.allocation_pools WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| AllocationPool {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            pool_type: r.get("pool_type"),
            source_account_codes: r.get("source_account_codes"),
            source_department_id: r.get("source_department_id"),
            source_cost_center: r.get("source_cost_center"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<AllocationPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.allocation_pools WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| AllocationPool {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            pool_type: r.get("pool_type"),
            source_account_codes: r.get("source_account_codes"),
            source_department_id: r.get("source_department_id"),
            source_cost_center: r.get("source_cost_center"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_pools(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationPool>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.allocation_pools WHERE organization_id = $1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| AllocationPool {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            pool_type: r.get("pool_type"),
            source_account_codes: r.get("source_account_codes"),
            source_department_id: r.get("source_department_id"),
            source_cost_center: r.get("source_cost_center"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.allocation_pools WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Allocation Bases ──────────────────────────────────────────────

    async fn create_base(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        base_type: &str,
        financial_account_code: Option<&str>,
        unit_of_measure: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBase> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_bases
                (organization_id, code, name, description, base_type,
                 financial_account_code, unit_of_measure,
                 is_active, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(base_type)
        .bind(financial_account_code).bind(unit_of_measure)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AllocationBase {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            base_type: row.get("base_type"),
            financial_account_code: row.get("financial_account_code"),
            unit_of_measure: row.get("unit_of_measure"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_base(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationBase>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.allocation_bases WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| AllocationBase {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            base_type: r.get("base_type"),
            financial_account_code: r.get("financial_account_code"),
            unit_of_measure: r.get("unit_of_measure"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_base_by_id(&self, id: Uuid) -> AtlasResult<Option<AllocationBase>> {
        let row = sqlx::query("SELECT * FROM _atlas.allocation_bases WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| AllocationBase {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            base_type: r.get("base_type"),
            financial_account_code: r.get("financial_account_code"),
            unit_of_measure: r.get("unit_of_measure"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_bases(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationBase>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.allocation_bases WHERE organization_id = $1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| AllocationBase {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            base_type: r.get("base_type"),
            financial_account_code: r.get("financial_account_code"),
            unit_of_measure: r.get("unit_of_measure"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_base(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.allocation_bases WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Base Values ───────────────────────────────────────────────────

    async fn upsert_base_value(
        &self,
        org_id: Uuid,
        base_id: Uuid,
        base_code: &str,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        value: &str,
        effective_date: chrono::NaiveDate,
        source: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBaseValue> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_base_values
                (organization_id, base_id, base_code,
                 department_id, department_name, cost_center, project_id,
                 value, effective_date, source, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9, $10, $11)
            ON CONFLICT (base_id, COALESCE(department_id, '00000000-0000-0000-0000-000000000000'),
                                  COALESCE(cost_center, ''), effective_date)
            DO UPDATE SET value = EXCLUDED.value, source = EXCLUDED.source,
                          updated_at = now(), created_by = EXCLUDED.created_by
            RETURNING *
            "#,
        )
        .bind(org_id).bind(base_id).bind(base_code)
        .bind(department_id).bind(department_name).bind(cost_center).bind(project_id)
        .bind(value).bind(effective_date).bind(source).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AllocationBaseValue {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            base_id: row.get("base_id"),
            base_code: row.get("base_code"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            cost_center: row.get("cost_center"),
            project_id: row.get("project_id"),
            value: row.try_get("value").map(|v: serde_json::Value| v.to_string()).unwrap_or_else(|_| "0".to_string()),
            effective_date: row.get("effective_date"),
            source: row.get("source"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_base_values(
        &self,
        org_id: Uuid,
        base_id: Uuid,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AllocationBaseValue>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.allocation_base_values
            WHERE organization_id = $1 AND base_id = $2 AND effective_date <= $3
            ORDER BY effective_date DESC, department_name
            "#,
        )
        .bind(org_id).bind(base_id).bind(effective_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| AllocationBaseValue {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            base_id: r.get("base_id"),
            base_code: r.get("base_code"),
            department_id: r.get("department_id"),
            department_name: r.get("department_name"),
            cost_center: r.get("cost_center"),
            project_id: r.get("project_id"),
            value: r.try_get("value").map(|v: serde_json::Value| v.to_string()).unwrap_or_else(|_| "0".to_string()),
            effective_date: r.get("effective_date"),
            source: r.get("source"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn list_base_values(
        &self,
        org_id: Uuid,
        base_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AllocationBaseValue>> {
        let rows = if let Some(bid) = base_id {
            sqlx::query(
                "SELECT * FROM _atlas.allocation_base_values WHERE organization_id = $1 AND base_id = $2 ORDER BY effective_date DESC"
            )
            .bind(org_id).bind(bid)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.allocation_base_values WHERE organization_id = $1 ORDER BY effective_date DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| AllocationBaseValue {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            base_id: r.get("base_id"),
            base_code: r.get("base_code"),
            department_id: r.get("department_id"),
            department_name: r.get("department_name"),
            cost_center: r.get("cost_center"),
            project_id: r.get("project_id"),
            value: r.try_get("value").map(|v: serde_json::Value| v.to_string()).unwrap_or_else(|_| "0".to_string()),
            effective_date: r.get("effective_date"),
            source: r.get("source"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    // ── Allocation Rules ──────────────────────────────────────────────

    async fn create_rule(
        &self,
        org_id: Uuid,
        rule_number: &str,
        name: &str,
        description: Option<&str>,
        pool_id: Uuid,
        pool_code: &str,
        base_id: Uuid,
        base_code: &str,
        allocation_method: &str,
        journal_description: Option<&str>,
        offset_account_code: Option<&str>,
        currency_code: &str,
        is_reversing: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_rules
                (organization_id, rule_number, name, description,
                 pool_id, pool_code, base_id, base_code,
                 allocation_method, journal_description, offset_account_code,
                 status, current_version, currency_code, is_reversing, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    'draft', 1, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rule_number).bind(name).bind(description)
        .bind(pool_id).bind(pool_code).bind(base_id).bind(base_code)
        .bind(allocation_method).bind(journal_description).bind(offset_account_code)
        .bind(currency_code).bind(is_reversing).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_rule(&row))
    }

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<AllocationRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.allocation_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn get_rule_by_number(&self, org_id: Uuid, rule_number: &str) -> AtlasResult<Option<AllocationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.allocation_rules WHERE organization_id = $1 AND rule_number = $2"
        )
        .bind(org_id).bind(rule_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn list_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AllocationRule>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.allocation_rules
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY rule_number
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_rule).collect())
    }

    async fn update_rule_status(&self, id: Uuid, status: &str) -> AtlasResult<AllocationRule> {
        let row = sqlx::query(
            "UPDATE _atlas.allocation_rules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    // ── Rule Targets ──────────────────────────────────────────────────

    async fn create_rule_target(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        line_number: i32,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        target_account_code: &str,
        fixed_percent: Option<&str>,
        fixed_amount: Option<&str>,
    ) -> AtlasResult<AllocationRuleTarget> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_rule_targets
                (organization_id, rule_id, line_number,
                 department_id, department_name, cost_center,
                 project_id, project_name,
                 target_account_code, fixed_percent, fixed_amount, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::numeric, $11::numeric, true)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rule_id).bind(line_number)
        .bind(department_id).bind(department_name).bind(cost_center)
        .bind(project_id).bind(project_name)
        .bind(target_account_code).bind(fixed_percent).bind(fixed_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AllocationRuleTarget {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rule_id: row.get("rule_id"),
            line_number: row.get("line_number"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            cost_center: row.get("cost_center"),
            project_id: row.get("project_id"),
            project_name: row.get("project_name"),
            target_account_code: row.get("target_account_code"),
            fixed_percent: row.try_get("fixed_percent").map(|v: serde_json::Value| v.to_string()).ok(),
            fixed_amount: row.try_get("fixed_amount").map(|v: serde_json::Value| v.to_string()).ok(),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_rule_targets(&self, rule_id: Uuid) -> AtlasResult<Vec<AllocationRuleTarget>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.allocation_rule_targets WHERE rule_id = $1 ORDER BY line_number"
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| AllocationRuleTarget {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            rule_id: r.get("rule_id"),
            line_number: r.get("line_number"),
            department_id: r.get("department_id"),
            department_name: r.get("department_name"),
            cost_center: r.get("cost_center"),
            project_id: r.get("project_id"),
            project_name: r.get("project_name"),
            target_account_code: r.get("target_account_code"),
            fixed_percent: r.try_get("fixed_percent").map(|v: serde_json::Value| v.to_string()).ok(),
            fixed_amount: r.try_get("fixed_amount").map(|v: serde_json::Value| v.to_string()).ok(),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_rule_targets(&self, rule_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.allocation_rule_targets WHERE rule_id = $1")
            .bind(rule_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Allocation Runs ───────────────────────────────────────────────

    async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        rule_id: Uuid,
        rule_name: &str,
        rule_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        total_source_amount: &str,
        total_allocated_amount: &str,
        line_count: i32,
        run_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRun> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_runs
                (organization_id, run_number, rule_id, rule_name, rule_number,
                 period_start, period_end, total_source_amount, total_allocated_amount,
                 line_count, status, run_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric,
                    $10, 'draft', $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(run_number).bind(rule_id).bind(rule_name).bind(rule_number)
        .bind(period_start).bind(period_end).bind(total_source_amount).bind(total_allocated_amount)
        .bind(line_count).bind(run_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_run(&row))
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<AllocationRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.allocation_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn list_runs(&self, org_id: Uuid, rule_id: Option<Uuid>) -> AtlasResult<Vec<AllocationRun>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.allocation_runs
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR rule_id = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(rule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run).collect())
    }

    async fn update_run_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
        reversed_by_id: Option<Uuid>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<AllocationRun> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.allocation_runs
            SET status = $2,
                posted_by = COALESCE($3, posted_by),
                posted_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE posted_at END,
                reversed_by_id = COALESCE($4, reversed_by_id),
                reversal_reason = COALESCE($5, reversal_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(posted_by).bind(reversed_by_id).bind(reversal_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    // ── Run Lines ─────────────────────────────────────────────────────

    async fn create_run_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        line_number: i32,
        line_type: &str,
        account_code: &str,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        amount: &str,
        base_value_used: Option<&str>,
        percent_of_total: Option<&str>,
        description: Option<&str>,
    ) -> AtlasResult<AllocationRunLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.allocation_run_lines
                (organization_id, run_id, line_number, line_type, account_code,
                 department_id, department_name, cost_center, project_id,
                 amount, base_value_used, percent_of_total, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::numeric, $11::numeric, $12::numeric, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(run_id).bind(line_number).bind(line_type).bind(account_code)
        .bind(department_id).bind(department_name).bind(cost_center).bind(project_id)
        .bind(amount).bind(base_value_used).bind(percent_of_total).bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AllocationRunLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            run_id: row.get("run_id"),
            line_number: row.get("line_number"),
            line_type: row.get("line_type"),
            account_code: row.get("account_code"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            cost_center: row.get("cost_center"),
            project_id: row.get("project_id"),
            amount: row.try_get("amount").map(|v: serde_json::Value| v.to_string()).unwrap_or_else(|_| "0".to_string()),
            base_value_used: row.try_get("base_value_used").map(|v: serde_json::Value| v.to_string()).ok(),
            percent_of_total: row.try_get("percent_of_total").map(|v: serde_json::Value| v.to_string()).ok(),
            description: row.get("description"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<AllocationRunLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.allocation_run_lines WHERE run_id = $1 ORDER BY line_number"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|r| AllocationRunLine {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            run_id: r.get("run_id"),
            line_number: r.get("line_number"),
            line_type: r.get("line_type"),
            account_code: r.get("account_code"),
            department_id: r.get("department_id"),
            department_name: r.get("department_name"),
            cost_center: r.get("cost_center"),
            project_id: r.get("project_id"),
            amount: r.try_get("amount").map(|v: serde_json::Value| v.to_string()).unwrap_or_else(|_| "0".to_string()),
            base_value_used: r.try_get("base_value_used").map(|v: serde_json::Value| v.to_string()).ok(),
            percent_of_total: r.try_get("percent_of_total").map(|v: serde_json::Value| v.to_string()).ok(),
            description: r.get("description"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    // ── Dashboard ─────────────────────────────────────────────────────

    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<AllocationSummary> {
        let rules_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.allocation_rules WHERE organization_id = $1 AND status = 'active'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let pools_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.allocation_pools WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let runs_row = sqlx::query(
            r#"SELECT COUNT(*) as run_count,
                      COALESCE(SUM(total_allocated_amount), 0) as total_amount
               FROM _atlas.allocation_runs WHERE organization_id = $1"#
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_rules: i64 = rules_row.try_get("cnt").unwrap_or(0);
        let pool_count: i64 = pools_row.try_get("cnt").unwrap_or(0);
        let run_count: i64 = runs_row.try_get("run_count").unwrap_or(0);
        let total_amount: serde_json::Value = runs_row.try_get("total_amount").unwrap_or(serde_json::json!(0));

        Ok(AllocationSummary {
            active_rule_count: active_rules as i32,
            pool_count: pool_count as i32,
            run_count: run_count as i32,
            total_allocated_amount: total_amount.to_string(),
            runs_by_status: serde_json::json!({}),
            allocations_by_pool: serde_json::json!({}),
            top_rules: serde_json::json!({}),
        })
    }
}

fn row_to_rule(row: &sqlx::postgres::PgRow) -> AllocationRule {
    AllocationRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        rule_number: row.get("rule_number"),
        name: row.get("name"),
        description: row.get("description"),
        pool_id: row.get("pool_id"),
        pool_code: row.get("pool_code"),
        base_id: row.get("base_id"),
        base_code: row.get("base_code"),
        allocation_method: row.get("allocation_method"),
        journal_description: row.get("journal_description"),
        offset_account_code: row.get("offset_account_code"),
        status: row.get("status"),
        current_version: row.get("current_version"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        is_reversing: row.get("is_reversing"),
        currency_code: row.get("currency_code"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_run(row: &sqlx::postgres::PgRow) -> AllocationRun {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    AllocationRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_number: row.get("run_number"),
        rule_id: row.get("rule_id"),
        rule_name: row.get("rule_name"),
        rule_number: row.get("rule_number"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        total_source_amount: get_num(row, "total_source_amount"),
        total_allocated_amount: get_num(row, "total_allocated_amount"),
        line_count: row.get("line_count"),
        status: row.get("status"),
        journal_entry_id: row.get("journal_entry_id"),
        run_date: row.get("run_date"),
        reversed_by_id: row.get("reversed_by_id"),
        reversal_reason: row.get("reversal_reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        posted_by: row.get("posted_by"),
        posted_at: row.get("posted_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
