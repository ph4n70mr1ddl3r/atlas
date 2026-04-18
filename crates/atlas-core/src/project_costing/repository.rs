//! Project Costing Repository
//!
//! PostgreSQL storage for cost transactions, burden schedules,
//! cost adjustments, and cost distributions.

use atlas_shared::{
    ProjectCostTransaction, BurdenSchedule, BurdenScheduleLine,
    ProjectCostAdjustment, ProjectCostDistribution, ProjectCostingSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for project costing data storage
#[async_trait]
pub trait ProjectCostingRepository: Send + Sync {
    // Cost Transactions
    async fn create_cost_transaction(
        &self,
        org_id: Uuid,
        transaction_number: &str,
        project_id: Uuid,
        project_number: Option<&str>,
        task_id: Option<Uuid>,
        task_number: Option<&str>,
        cost_type: &str,
        raw_cost_amount: &str,
        burdened_cost_amount: &str,
        burden_amount: &str,
        currency_code: &str,
        transaction_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        employee_id: Option<Uuid>,
        employee_name: Option<&str>,
        expenditure_category: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_rate: Option<&str>,
        is_billable: bool,
        is_capitalizable: bool,
        original_transaction_id: Option<Uuid>,
        adjustment_type: Option<&str>,
        adjustment_reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostTransaction>;

    async fn get_cost_transaction(&self, id: Uuid) -> AtlasResult<Option<ProjectCostTransaction>>;
    async fn get_cost_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<ProjectCostTransaction>>;
    async fn list_cost_transactions(&self, org_id: Uuid, project_id: Option<Uuid>, cost_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<ProjectCostTransaction>>;
    async fn update_cost_transaction_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ProjectCostTransaction>;

    // Burden Schedules
    async fn create_burden_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        status: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_default: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BurdenSchedule>;

    async fn get_burden_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BurdenSchedule>>;
    async fn get_burden_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<BurdenSchedule>>;
    async fn list_burden_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<BurdenSchedule>>;
    async fn get_default_burden_schedule(&self, org_id: Uuid) -> AtlasResult<Option<BurdenSchedule>>;
    async fn update_burden_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<BurdenSchedule>;

    // Burden Schedule Lines
    async fn create_burden_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        line_number: i32,
        cost_type: &str,
        expenditure_category: Option<&str>,
        burden_rate_percent: &str,
        burden_account_code: Option<&str>,
    ) -> AtlasResult<BurdenScheduleLine>;

    async fn list_burden_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BurdenScheduleLine>>;
    async fn get_applicable_burden_rate(&self, schedule_id: Uuid, cost_type: &str, expenditure_category: Option<&str>) -> AtlasResult<Option<BurdenScheduleLine>>;

    // Cost Adjustments
    async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
        original_transaction_id: Uuid,
        adjustment_type: &str,
        adjustment_amount: &str,
        new_raw_cost: &str,
        new_burdened_cost: &str,
        reason: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        transfer_to_project_id: Option<Uuid>,
        transfer_to_task_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment>;

    async fn get_cost_adjustment(&self, id: Uuid) -> AtlasResult<Option<ProjectCostAdjustment>>;
    async fn list_cost_adjustments(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectCostAdjustment>>;
    async fn update_cost_adjustment_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        created_transaction_id: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment>;

    // Cost Distributions
    async fn create_cost_distribution(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        line_number: i32,
        debit_account_code: &str,
        credit_account_code: &str,
        amount: &str,
        distribution_type: &str,
        gl_date: chrono::NaiveDate,
    ) -> AtlasResult<ProjectCostDistribution>;

    async fn list_cost_distributions(&self, transaction_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>>;
    async fn list_unposted_distributions(&self, org_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>>;
    async fn mark_distribution_posted(&self, id: Uuid, gl_batch_id: Option<Uuid>) -> AtlasResult<ProjectCostDistribution>;

    // Dashboard
    async fn get_costing_summary(&self, org_id: Uuid) -> AtlasResult<ProjectCostingSummary>;
}

/// PostgreSQL implementation
pub struct PostgresProjectCostingRepository {
    pool: PgPool,
}

impl PostgresProjectCostingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_transaction(&self, row: &sqlx::postgres::PgRow) -> ProjectCostTransaction {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        ProjectCostTransaction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            transaction_number: row.get("transaction_number"),
            project_id: row.get("project_id"),
            project_number: row.get("project_number"),
            task_id: row.get("task_id"),
            task_number: row.get("task_number"),
            cost_type: row.get("cost_type"),
            raw_cost_amount: get_num(row, "raw_cost_amount"),
            burdened_cost_amount: get_num(row, "burdened_cost_amount"),
            burden_amount: get_num(row, "burden_amount"),
            currency_code: row.get("currency_code"),
            transaction_date: row.get("transaction_date"),
            gl_date: row.get("gl_date"),
            description: row.get("description"),
            supplier_id: row.get("supplier_id"),
            supplier_name: row.get("supplier_name"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            expenditure_category: row.get("expenditure_category"),
            quantity: row.try_get("quantity").unwrap_or(None),
            unit_of_measure: row.get("unit_of_measure"),
            unit_rate: row.try_get("unit_rate").unwrap_or(None),
            is_billable: row.get("is_billable"),
            is_capitalizable: row.get("is_capitalizable"),
            status: row.get("status"),
            distribution_id: row.get("distribution_id"),
            original_transaction_id: row.get("original_transaction_id"),
            adjustment_type: row.get("adjustment_type"),
            adjustment_reason: row.get("adjustment_reason"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            approved_by: row.get("approved_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_schedule(&self, row: &sqlx::postgres::PgRow) -> BurdenSchedule {
        BurdenSchedule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            status: row.get("status"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_default: row.get("is_default"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_schedule_line(&self, row: &sqlx::postgres::PgRow) -> BurdenScheduleLine {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        BurdenScheduleLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            schedule_id: row.get("schedule_id"),
            line_number: row.get("line_number"),
            cost_type: row.get("cost_type"),
            expenditure_category: row.get("expenditure_category"),
            burden_rate_percent: get_num(row, "burden_rate_percent"),
            burden_account_code: row.get("burden_account_code"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_adjustment(&self, row: &sqlx::postgres::PgRow) -> ProjectCostAdjustment {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        ProjectCostAdjustment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            adjustment_number: row.get("adjustment_number"),
            original_transaction_id: row.get("original_transaction_id"),
            adjustment_type: row.get("adjustment_type"),
            adjustment_amount: get_num(row, "adjustment_amount"),
            new_raw_cost: get_num(row, "new_raw_cost"),
            new_burdened_cost: get_num(row, "new_burdened_cost"),
            reason: row.get("reason"),
            description: row.get("description"),
            effective_date: row.get("effective_date"),
            transfer_to_project_id: row.get("transfer_to_project_id"),
            transfer_to_task_id: row.get("transfer_to_task_id"),
            status: row.get("status"),
            created_transaction_id: row.get("created_transaction_id"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_distribution(&self, row: &sqlx::postgres::PgRow) -> ProjectCostDistribution {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        ProjectCostDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            transaction_id: row.get("transaction_id"),
            line_number: row.get("line_number"),
            debit_account_code: row.get("debit_account_code"),
            credit_account_code: row.get("credit_account_code"),
            amount: get_num(row, "amount"),
            distribution_type: row.get("distribution_type"),
            gl_date: row.get("gl_date"),
            is_posted: row.get("is_posted"),
            gl_batch_id: row.get("gl_batch_id"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl ProjectCostingRepository for PostgresProjectCostingRepository {
    async fn create_cost_transaction(
        &self,
        org_id: Uuid,
        transaction_number: &str,
        project_id: Uuid,
        project_number: Option<&str>,
        task_id: Option<Uuid>,
        task_number: Option<&str>,
        cost_type: &str,
        raw_cost_amount: &str,
        burdened_cost_amount: &str,
        burden_amount: &str,
        currency_code: &str,
        transaction_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        employee_id: Option<Uuid>,
        employee_name: Option<&str>,
        expenditure_category: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_rate: Option<&str>,
        is_billable: bool,
        is_capitalizable: bool,
        original_transaction_id: Option<Uuid>,
        adjustment_type: Option<&str>,
        adjustment_reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.project_cost_transactions
                (organization_id, transaction_number, project_id, project_number,
                 task_id, task_number, cost_type,
                 raw_cost_amount, burdened_cost_amount, burden_amount,
                 currency_code, transaction_date, gl_date, description,
                 supplier_id, supplier_name, employee_id, employee_name,
                 expenditure_category, quantity, unit_of_measure, unit_rate,
                 is_billable, is_capitalizable,
                 original_transaction_id, adjustment_type, adjustment_reason,
                 status, created_by)
            VALUES ($1, $2, $3, $4,
                    $5, $6, $7,
                    $8::numeric, $9::numeric, $10::numeric,
                    $11, $12, $13, $14,
                    $15, $16, $17, $18,
                    $19, $20::numeric, $21, $22::numeric,
                    $23, $24,
                    $25, $26, $27,
                    'draft', $28)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transaction_number).bind(project_id).bind(project_number)
        .bind(task_id).bind(task_number).bind(cost_type)
        .bind(raw_cost_amount).bind(burdened_cost_amount).bind(burden_amount)
        .bind(currency_code).bind(transaction_date).bind(gl_date).bind(description)
        .bind(supplier_id).bind(supplier_name).bind(employee_id).bind(employee_name)
        .bind(expenditure_category).bind(quantity).bind(unit_of_measure).bind(unit_rate)
        .bind(is_billable).bind(is_capitalizable)
        .bind(original_transaction_id).bind(adjustment_type).bind(adjustment_reason)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_transaction(&row))
    }

    async fn get_cost_transaction(&self, id: Uuid) -> AtlasResult<Option<ProjectCostTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.project_cost_transactions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transaction(&r)))
    }

    async fn get_cost_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<ProjectCostTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.project_cost_transactions WHERE organization_id = $1 AND transaction_number = $2"
        )
        .bind(org_id).bind(transaction_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transaction(&r)))
    }

    async fn list_cost_transactions(&self, org_id: Uuid, project_id: Option<Uuid>, cost_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<ProjectCostTransaction>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.project_cost_transactions
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR project_id = $2)
              AND ($3::text IS NULL OR cost_type = $3)
              AND ($4::text IS NULL OR status = $4)
            ORDER BY transaction_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(project_id).bind(cost_type).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transaction(&r)).collect())
    }

    async fn update_cost_transaction_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ProjectCostTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.project_cost_transactions
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transaction(&row))
    }

    // Burden Schedules

    async fn create_burden_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        status: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_default: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BurdenSchedule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.burden_schedules
                (organization_id, code, name, description, status,
                 effective_from, effective_to, is_default, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(status)
        .bind(effective_from).bind(effective_to).bind(is_default).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_schedule(&row))
    }

    async fn get_burden_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BurdenSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.burden_schedules WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_schedule(&r)))
    }

    async fn get_burden_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<BurdenSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.burden_schedules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_schedule(&r)))
    }

    async fn list_burden_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<BurdenSchedule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.burden_schedules WHERE organization_id = $1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_schedule(&r)).collect())
    }

    async fn get_default_burden_schedule(&self, org_id: Uuid) -> AtlasResult<Option<BurdenSchedule>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.burden_schedules
            WHERE organization_id = $1 AND is_default = true AND status = 'active'
            ORDER BY effective_from DESC LIMIT 1
            "#
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_schedule(&r)))
    }

    async fn update_burden_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<BurdenSchedule> {
        let row = sqlx::query(
            "UPDATE _atlas.burden_schedules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_schedule(&row))
    }

    // Burden Schedule Lines

    async fn create_burden_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        line_number: i32,
        cost_type: &str,
        expenditure_category: Option<&str>,
        burden_rate_percent: &str,
        burden_account_code: Option<&str>,
    ) -> AtlasResult<BurdenScheduleLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.burden_schedule_lines
                (organization_id, schedule_id, line_number, cost_type,
                 expenditure_category, burden_rate_percent, burden_account_code)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(schedule_id).bind(line_number).bind(cost_type)
        .bind(expenditure_category).bind(burden_rate_percent).bind(burden_account_code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_schedule_line(&row))
    }

    async fn list_burden_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BurdenScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.burden_schedule_lines WHERE schedule_id = $1 ORDER BY line_number"
        )
        .bind(schedule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_schedule_line(&r)).collect())
    }

    async fn get_applicable_burden_rate(&self, schedule_id: Uuid, cost_type: &str, expenditure_category: Option<&str>) -> AtlasResult<Option<BurdenScheduleLine>> {
        // Try exact match on cost_type + expenditure_category first, then fall back to cost_type only
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.burden_schedule_lines
            WHERE schedule_id = $1 AND cost_type = $2 AND is_active = true
              AND (expenditure_category = $3 OR ($3 IS NULL AND expenditure_category IS NULL))
            LIMIT 1
            "#,
        )
        .bind(schedule_id).bind(cost_type).bind(expenditure_category)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if row.is_some() {
            return Ok(row.map(|r| self.row_to_schedule_line(&r)));
        }

        // Fallback: cost_type match only (category-agnostic line)
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.burden_schedule_lines
            WHERE schedule_id = $1 AND cost_type = $2 AND is_active = true
              AND expenditure_category IS NULL
            LIMIT 1
            "#,
        )
        .bind(schedule_id).bind(cost_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_schedule_line(&r)))
    }

    // Cost Adjustments

    async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        adjustment_number: &str,
        original_transaction_id: Uuid,
        adjustment_type: &str,
        adjustment_amount: &str,
        new_raw_cost: &str,
        new_burdened_cost: &str,
        reason: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        transfer_to_project_id: Option<Uuid>,
        transfer_to_task_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.project_cost_adjustments
                (organization_id, adjustment_number, original_transaction_id,
                 adjustment_type, adjustment_amount, new_raw_cost, new_burdened_cost,
                 reason, description, effective_date,
                 transfer_to_project_id, transfer_to_task_id,
                 status, created_by)
            VALUES ($1, $2, $3,
                    $4, $5::numeric, $6::numeric, $7::numeric,
                    $8, $9, $10,
                    $11, $12,
                    'pending', $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(adjustment_number).bind(original_transaction_id)
        .bind(adjustment_type).bind(adjustment_amount).bind(new_raw_cost).bind(new_burdened_cost)
        .bind(reason).bind(description).bind(effective_date)
        .bind(transfer_to_project_id).bind(transfer_to_task_id)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_adjustment(&row))
    }

    async fn get_cost_adjustment(&self, id: Uuid) -> AtlasResult<Option<ProjectCostAdjustment>> {
        let row = sqlx::query("SELECT * FROM _atlas.project_cost_adjustments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_adjustment(&r)))
    }

    async fn list_cost_adjustments(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectCostAdjustment>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.project_cost_adjustments
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_adjustment(&r)).collect())
    }

    async fn update_cost_adjustment_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        created_transaction_id: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.project_cost_adjustments
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE approved_at END,
                created_transaction_id = COALESCE($4, created_transaction_id),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(created_transaction_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_adjustment(&row))
    }

    // Cost Distributions

    async fn create_cost_distribution(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        line_number: i32,
        debit_account_code: &str,
        credit_account_code: &str,
        amount: &str,
        distribution_type: &str,
        gl_date: chrono::NaiveDate,
    ) -> AtlasResult<ProjectCostDistribution> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.project_cost_distributions
                (organization_id, transaction_id, line_number,
                 debit_account_code, credit_account_code,
                 amount, distribution_type, gl_date)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transaction_id).bind(line_number)
        .bind(debit_account_code).bind(credit_account_code)
        .bind(amount).bind(distribution_type).bind(gl_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_distribution(&row))
    }

    async fn list_cost_distributions(&self, transaction_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.project_cost_distributions WHERE transaction_id = $1 ORDER BY line_number"
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(&r)).collect())
    }

    async fn list_unposted_distributions(&self, org_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.project_cost_distributions WHERE organization_id = $1 AND is_posted = false ORDER BY gl_date"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(&r)).collect())
    }

    async fn mark_distribution_posted(&self, id: Uuid, gl_batch_id: Option<Uuid>) -> AtlasResult<ProjectCostDistribution> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.project_cost_distributions
            SET is_posted = true, gl_batch_id = COALESCE($2, gl_batch_id), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(gl_batch_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_distribution(&row))
    }

    // Dashboard

    async fn get_costing_summary(&self, org_id: Uuid) -> AtlasResult<ProjectCostingSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(DISTINCT project_id) as project_count,
                COALESCE(SUM(raw_cost_amount), 0) as total_raw,
                COALESCE(SUM(burdened_cost_amount), 0) as total_burdened,
                COALESCE(SUM(burden_amount), 0) as total_burden,
                COUNT(*) FILTER (WHERE status = 'capitalized') as capitalized_count,
                COALESCE(SUM(raw_cost_amount) FILTER (WHERE status = 'capitalized'), 0) as total_capitalized,
                COUNT(*) FILTER (WHERE is_billable = true AND status != 'draft') as billable_count,
                COALESCE(SUM(raw_cost_amount) FILTER (WHERE is_billable = true AND status != 'draft'), 0) as total_billed
            FROM _atlas.project_cost_transactions
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let project_count: i64 = row.try_get("project_count").unwrap_or(0);
        let total_raw: serde_json::Value = row.try_get("total_raw").unwrap_or(serde_json::json!(0));
        let total_burdened: serde_json::Value = row.try_get("total_burdened").unwrap_or(serde_json::json!(0));
        let total_burden: serde_json::Value = row.try_get("total_burden").unwrap_or(serde_json::json!(0));
        let total_capitalized: serde_json::Value = row.try_get("total_capitalized").unwrap_or(serde_json::json!(0));
        let total_billed: serde_json::Value = row.try_get("total_billed").unwrap_or(serde_json::json!(0));

        // Count pending adjustments
        let adj_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.project_cost_adjustments WHERE organization_id = $1 AND status = 'pending'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let pending_adj: i64 = adj_row.try_get("cnt").unwrap_or(0);

        // Count pending distributions
        let dist_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.project_cost_distributions WHERE organization_id = $1 AND is_posted = false"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let pending_dist: i64 = dist_row.try_get("cnt").unwrap_or(0);

        Ok(ProjectCostingSummary {
            project_count: project_count as i32,
            total_raw_costs: total_raw.to_string(),
            total_burdened_costs: total_burdened.to_string(),
            total_burden: total_burden.to_string(),
            total_capitalized: total_capitalized.to_string(),
            total_billed: total_billed.to_string(),
            costs_by_type: serde_json::json!({}),
            costs_by_project: serde_json::json!({}),
            costs_by_month: serde_json::json!({}),
            pending_adjustments: pending_adj as i32,
            pending_distributions: pending_dist as i32,
        })
    }
}
