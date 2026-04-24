//! Compensation Management Repository
//!
//! PostgreSQL storage for compensation management data.

use atlas_shared::{
    CompensationPlan, CompensationComponent, CompensationCycle,
    CompensationBudgetPool, CompensationWorksheet, CompensationWorksheetLine,
    CompensationStatement, CompensationDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for Compensation Management data storage
#[async_trait]
pub trait CompensationRepository: Send + Sync {
    // Plans
    async fn create_plan(
        &self,
        org_id: Uuid,
        plan_code: &str,
        plan_name: &str,
        description: Option<&str>,
        plan_type: &str,
        effective_start_date: Option<chrono::NaiveDate>,
        effective_end_date: Option<chrono::NaiveDate>,
        eligibility_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationPlan>;

    async fn get_plan(&self, id: Uuid) -> AtlasResult<Option<CompensationPlan>>;
    async fn get_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CompensationPlan>>;
    async fn list_plans(&self, org_id: Uuid) -> AtlasResult<Vec<CompensationPlan>>;
    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Components
    async fn create_component(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        component_name: &str,
        component_type: &str,
        description: Option<&str>,
        is_recurring: bool,
        frequency: Option<&str>,
    ) -> AtlasResult<CompensationComponent>;
    async fn list_components(&self, plan_id: Uuid) -> AtlasResult<Vec<CompensationComponent>>;
    async fn delete_component(&self, id: Uuid) -> AtlasResult<()>;

    // Cycles
    async fn create_cycle(
        &self,
        org_id: Uuid,
        cycle_name: &str,
        description: Option<&str>,
        cycle_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationCycle>;
    async fn get_cycle(&self, id: Uuid) -> AtlasResult<Option<CompensationCycle>>;
    async fn list_cycles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationCycle>>;
    async fn update_cycle_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationCycle>;
    async fn update_cycle_totals(&self, id: Uuid, total_approved: &str, total_employees: i32) -> AtlasResult<()>;
    async fn delete_cycle(&self, id: Uuid) -> AtlasResult<()>;

    // Budget Pools
    async fn create_budget_pool(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_name: &str,
        pool_type: &str,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationBudgetPool>;
    async fn get_budget_pool(&self, id: Uuid) -> AtlasResult<Option<CompensationBudgetPool>>;
    async fn list_budget_pools(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationBudgetPool>>;
    async fn delete_budget_pool(&self, id: Uuid) -> AtlasResult<()>;

    // Worksheets
    async fn create_worksheet(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_id: Option<Uuid>,
        manager_id: Uuid,
        manager_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheet>;
    async fn get_worksheet(&self, id: Uuid) -> AtlasResult<Option<CompensationWorksheet>>;
    async fn list_worksheets(&self, cycle_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationWorksheet>>;
    async fn update_worksheet_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationWorksheet>;
    async fn update_worksheet_totals(
        &self,
        id: Uuid,
        total_employees: i32,
        total_current_salary: &str,
        total_proposed_salary: &str,
        total_merit: &str,
        total_bonus: &str,
        total_equity: &str,
        total_compensation_change: &str,
    ) -> AtlasResult<()>;
    async fn delete_worksheet(&self, id: Uuid) -> AtlasResult<()>;

    // Worksheet Lines
    async fn create_worksheet_line(
        &self,
        org_id: Uuid,
        worksheet_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        job_title: Option<&str>,
        department_name: Option<&str>,
        current_base_salary: &str,
        proposed_base_salary: &str,
        salary_change_amount: &str,
        salary_change_percent: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        total_compensation: &str,
        performance_rating: Option<&str>,
        compa_ratio: &str,
        manager_comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheetLine>;
    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<CompensationWorksheetLine>>;
    async fn list_worksheet_lines(&self, worksheet_id: Uuid) -> AtlasResult<Vec<CompensationWorksheetLine>>;
    async fn update_worksheet_line(
        &self,
        id: Uuid,
        proposed_base_salary: &str,
        salary_change_amount: &str,
        salary_change_percent: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        total_compensation: &str,
        compa_ratio: &str,
        manager_comments: Option<&str>,
    ) -> AtlasResult<CompensationWorksheetLine>;
    async fn update_line_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationWorksheetLine>;
    async fn delete_worksheet_line(&self, id: Uuid) -> AtlasResult<()>;

    // Statements
    async fn upsert_statement(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        statement_date: chrono::NaiveDate,
        base_salary: &str,
        merit_increase: &str,
        bonus: &str,
        equity: &str,
        benefits_value: &str,
        total_compensation: &str,
        total_direct_compensation: &str,
        total_indirect_compensation: &str,
        change_from_previous: &str,
        change_percent: &str,
        currency_code: &str,
        components: serde_json::Value,
    ) -> AtlasResult<CompensationStatement>;
    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CompensationStatement>>;
    async fn get_statement_by_employee(&self, cycle_id: Uuid, employee_id: Uuid) -> AtlasResult<Option<CompensationStatement>>;
    async fn list_statements(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationStatement>>;
    async fn publish_statement(&self, id: Uuid) -> AtlasResult<CompensationStatement>;
    async fn delete_statement(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CompensationDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresCompensationRepository {
    pool: PgPool,
}

impl PostgresCompensationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CompensationRepository for PostgresCompensationRepository {
    // Plans
    async fn create_plan(
        &self,
        org_id: Uuid,
        plan_code: &str,
        plan_name: &str,
        description: Option<&str>,
        plan_type: &str,
        effective_start_date: Option<chrono::NaiveDate>,
        effective_end_date: Option<chrono::NaiveDate>,
        eligibility_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationPlan> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_plans
                (organization_id, plan_code, plan_name, description, plan_type,
                 effective_start_date, effective_end_date, eligibility_criteria, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(org_id).bind(plan_code).bind(plan_name).bind(description)
        .bind(plan_type).bind(effective_start_date).bind(effective_end_date)
        .bind(&eligibility_criteria).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_plan(&row))
    }

    async fn get_plan(&self, id: Uuid) -> AtlasResult<Option<CompensationPlan>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_plans WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_plan(&r)))
    }

    async fn get_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CompensationPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.compensation_plans WHERE organization_id = $1 AND plan_code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_plan(&r)))
    }

    async fn list_plans(&self, org_id: Uuid) -> AtlasResult<Vec<CompensationPlan>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.compensation_plans WHERE organization_id = $1 AND is_active = true ORDER BY plan_name"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_plan).collect())
    }

    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_plans WHERE organization_id = $1 AND plan_code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Components
    async fn create_component(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        component_name: &str,
        component_type: &str,
        description: Option<&str>,
        is_recurring: bool,
        frequency: Option<&str>,
    ) -> AtlasResult<CompensationComponent> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_components
                (organization_id, plan_id, component_name, component_type, description, is_recurring, frequency)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(org_id).bind(plan_id).bind(component_name).bind(component_type)
        .bind(description).bind(is_recurring).bind(frequency)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_component(&row))
    }

    async fn list_components(&self, plan_id: Uuid) -> AtlasResult<Vec<CompensationComponent>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.compensation_components WHERE plan_id = $1 AND is_active = true ORDER BY component_name"
        )
        .bind(plan_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_component).collect())
    }

    async fn delete_component(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_components WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Cycles
    async fn create_cycle(
        &self,
        org_id: Uuid,
        cycle_name: &str,
        description: Option<&str>,
        cycle_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationCycle> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_cycles
                (organization_id, cycle_name, description, cycle_type,
                 start_date, end_date, total_budget, currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(org_id).bind(cycle_name).bind(description).bind(cycle_type)
        .bind(start_date).bind(end_date)
        .bind(total_budget.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_cycle(&row))
    }

    async fn get_cycle(&self, id: Uuid) -> AtlasResult<Option<CompensationCycle>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_cycles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cycle(&r)))
    }

    async fn list_cycles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationCycle>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.compensation_cycles WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.compensation_cycles WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_cycle).collect())
    }

    async fn update_cycle_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationCycle> {
        let row = sqlx::query(
            "UPDATE _atlas.compensation_cycles SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle(&row))
    }

    async fn update_cycle_totals(&self, id: Uuid, total_approved: &str, total_employees: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.compensation_cycles SET total_approved = $2, total_employees = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .bind(total_approved.parse::<f64>().unwrap_or(0.0))
        .bind(total_employees)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_cycle(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_cycles WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Budget Pools
    async fn create_budget_pool(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_name: &str,
        pool_type: &str,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationBudgetPool> {
        let budget_val = total_budget.parse::<f64>().unwrap_or(0.0);
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_budget_pools
                (organization_id, cycle_id, pool_name, pool_type,
                 manager_id, manager_name, department_id, department_name,
                 total_budget, remaining_budget, currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(org_id).bind(cycle_id).bind(pool_name).bind(pool_type)
        .bind(manager_id).bind(manager_name)
        .bind(department_id).bind(department_name)
        .bind(budget_val).bind(currency_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_budget_pool(&row))
    }

    async fn get_budget_pool(&self, id: Uuid) -> AtlasResult<Option<CompensationBudgetPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_budget_pools WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_budget_pool(&r)))
    }

    async fn list_budget_pools(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationBudgetPool>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.compensation_budget_pools WHERE cycle_id = $1 ORDER BY pool_name"
        )
        .bind(cycle_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_budget_pool).collect())
    }

    async fn delete_budget_pool(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_budget_pools WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Worksheets
    async fn create_worksheet(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_id: Option<Uuid>,
        manager_id: Uuid,
        manager_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheet> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_worksheets
                (organization_id, cycle_id, pool_id, manager_id, manager_name, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(org_id).bind(cycle_id).bind(pool_id)
        .bind(manager_id).bind(manager_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_worksheet(&row))
    }

    async fn get_worksheet(&self, id: Uuid) -> AtlasResult<Option<CompensationWorksheet>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_worksheets WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_worksheet(&r)))
    }

    async fn list_worksheets(&self, cycle_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationWorksheet>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.compensation_worksheets WHERE cycle_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(cycle_id).bind(s)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.compensation_worksheets WHERE cycle_id = $1 ORDER BY created_at DESC"
            )
            .bind(cycle_id)
            .fetch_all(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_worksheet).collect())
    }

    async fn update_worksheet_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationWorksheet> {
        let row = sqlx::query(
            "UPDATE _atlas.compensation_worksheets SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_worksheet(&row))
    }

    async fn update_worksheet_totals(
        &self,
        id: Uuid,
        total_employees: i32,
        total_current_salary: &str,
        total_proposed_salary: &str,
        total_merit: &str,
        total_bonus: &str,
        total_equity: &str,
        total_compensation_change: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.compensation_worksheets
            SET total_employees = $2, total_current_salary = $3, total_proposed_salary = $4,
                total_merit = $5, total_bonus = $6, total_equity = $7,
                total_compensation_change = $8, updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id).bind(total_employees)
        .bind(total_current_salary.parse::<f64>().unwrap_or(0.0))
        .bind(total_proposed_salary.parse::<f64>().unwrap_or(0.0))
        .bind(total_merit.parse::<f64>().unwrap_or(0.0))
        .bind(total_bonus.parse::<f64>().unwrap_or(0.0))
        .bind(total_equity.parse::<f64>().unwrap_or(0.0))
        .bind(total_compensation_change.parse::<f64>().unwrap_or(0.0))
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_worksheet(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_worksheets WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Worksheet Lines
    async fn create_worksheet_line(
        &self,
        org_id: Uuid,
        worksheet_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        job_title: Option<&str>,
        department_name: Option<&str>,
        current_base_salary: &str,
        proposed_base_salary: &str,
        salary_change_amount: &str,
        salary_change_percent: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        total_compensation: &str,
        performance_rating: Option<&str>,
        compa_ratio: &str,
        manager_comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheetLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_worksheet_lines
                (organization_id, worksheet_id, employee_id, employee_name,
                 job_title, department_name, current_base_salary, proposed_base_salary,
                 salary_change_amount, salary_change_percent,
                 merit_amount, bonus_amount, equity_amount, total_compensation,
                 performance_rating, compa_ratio, manager_comments, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#
        )
        .bind(org_id).bind(worksheet_id).bind(employee_id).bind(employee_name)
        .bind(job_title).bind(department_name)
        .bind(current_base_salary.parse::<f64>().unwrap_or(0.0))
        .bind(proposed_base_salary.parse::<f64>().unwrap_or(0.0))
        .bind(salary_change_amount.parse::<f64>().unwrap_or(0.0))
        .bind(salary_change_percent.parse::<f64>().unwrap_or(0.0))
        .bind(merit_amount.parse::<f64>().unwrap_or(0.0))
        .bind(bonus_amount.parse::<f64>().unwrap_or(0.0))
        .bind(equity_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_compensation.parse::<f64>().unwrap_or(0.0))
        .bind(performance_rating)
        .bind(compa_ratio.parse::<f64>().unwrap_or(0.0))
        .bind(manager_comments).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_worksheet_line(&row))
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<CompensationWorksheetLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_worksheet_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_worksheet_line(&r)))
    }

    async fn list_worksheet_lines(&self, worksheet_id: Uuid) -> AtlasResult<Vec<CompensationWorksheetLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.compensation_worksheet_lines WHERE worksheet_id = $1 ORDER BY employee_name"
        )
        .bind(worksheet_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_worksheet_line).collect())
    }

    async fn update_worksheet_line(
        &self,
        id: Uuid,
        proposed_base_salary: &str,
        salary_change_amount: &str,
        salary_change_percent: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        total_compensation: &str,
        compa_ratio: &str,
        manager_comments: Option<&str>,
    ) -> AtlasResult<CompensationWorksheetLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.compensation_worksheet_lines
            SET proposed_base_salary = $2, salary_change_amount = $3, salary_change_percent = $4,
                merit_amount = $5, bonus_amount = $6, equity_amount = $7,
                total_compensation = $8, compa_ratio = $9, manager_comments = $10,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(proposed_base_salary.parse::<f64>().unwrap_or(0.0))
        .bind(salary_change_amount.parse::<f64>().unwrap_or(0.0))
        .bind(salary_change_percent.parse::<f64>().unwrap_or(0.0))
        .bind(merit_amount.parse::<f64>().unwrap_or(0.0))
        .bind(bonus_amount.parse::<f64>().unwrap_or(0.0))
        .bind(equity_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_compensation.parse::<f64>().unwrap_or(0.0))
        .bind(compa_ratio.parse::<f64>().unwrap_or(0.0))
        .bind(manager_comments)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_worksheet_line(&row))
    }

    async fn update_line_status(&self, id: Uuid, status: &str) -> AtlasResult<CompensationWorksheetLine> {
        let row = sqlx::query(
            "UPDATE _atlas.compensation_worksheet_lines SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_worksheet_line(&row))
    }

    async fn delete_worksheet_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_worksheet_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Statements
    async fn upsert_statement(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        statement_date: chrono::NaiveDate,
        base_salary: &str,
        merit_increase: &str,
        bonus: &str,
        equity: &str,
        benefits_value: &str,
        total_compensation: &str,
        total_direct_compensation: &str,
        total_indirect_compensation: &str,
        change_from_previous: &str,
        change_percent: &str,
        currency_code: &str,
        components: serde_json::Value,
    ) -> AtlasResult<CompensationStatement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.compensation_statements
                (organization_id, cycle_id, employee_id, employee_name, statement_date,
                 base_salary, merit_increase, bonus, equity, benefits_value,
                 total_compensation, total_direct_compensation, total_indirect_compensation,
                 change_from_previous, change_percent, currency_code, components)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (cycle_id, employee_id) DO UPDATE SET
                employee_name = EXCLUDED.employee_name,
                statement_date = EXCLUDED.statement_date,
                base_salary = EXCLUDED.base_salary,
                merit_increase = EXCLUDED.merit_increase,
                bonus = EXCLUDED.bonus,
                equity = EXCLUDED.equity,
                benefits_value = EXCLUDED.benefits_value,
                total_compensation = EXCLUDED.total_compensation,
                total_direct_compensation = EXCLUDED.total_direct_compensation,
                total_indirect_compensation = EXCLUDED.total_indirect_compensation,
                change_from_previous = EXCLUDED.change_from_previous,
                change_percent = EXCLUDED.change_percent,
                currency_code = EXCLUDED.currency_code,
                components = EXCLUDED.components,
                updated_at = now()
            RETURNING *
            "#
        )
        .bind(org_id).bind(cycle_id).bind(employee_id).bind(employee_name)
        .bind(statement_date)
        .bind(base_salary.parse::<f64>().unwrap_or(0.0))
        .bind(merit_increase.parse::<f64>().unwrap_or(0.0))
        .bind(bonus.parse::<f64>().unwrap_or(0.0))
        .bind(equity.parse::<f64>().unwrap_or(0.0))
        .bind(benefits_value.parse::<f64>().unwrap_or(0.0))
        .bind(total_compensation.parse::<f64>().unwrap_or(0.0))
        .bind(total_direct_compensation.parse::<f64>().unwrap_or(0.0))
        .bind(total_indirect_compensation.parse::<f64>().unwrap_or(0.0))
        .bind(change_from_previous.parse::<f64>().unwrap_or(0.0))
        .bind(change_percent.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(&components)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_statement(&row))
    }

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CompensationStatement>> {
        let row = sqlx::query("SELECT * FROM _atlas.compensation_statements WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_statement(&r)))
    }

    async fn get_statement_by_employee(&self, cycle_id: Uuid, employee_id: Uuid) -> AtlasResult<Option<CompensationStatement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.compensation_statements WHERE cycle_id = $1 AND employee_id = $2"
        )
        .bind(cycle_id).bind(employee_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_statement(&r)))
    }

    async fn list_statements(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationStatement>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.compensation_statements WHERE cycle_id = $1 ORDER BY employee_name"
        )
        .bind(cycle_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_statement).collect())
    }

    async fn publish_statement(&self, id: Uuid) -> AtlasResult<CompensationStatement> {
        let row = sqlx::query(
            "UPDATE _atlas.compensation_statements SET status = 'published', published_at = now(), updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_statement(&row))
    }

    async fn delete_statement(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.compensation_statements WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CompensationDashboard> {
        let plan_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.compensation_plans WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let cycle_row = sqlx::query(
            r#"
            SELECT COUNT(*) as total,
                   COUNT(*) FILTER (WHERE status IN ('active', 'allocation', 'review')) as active,
                   COALESCE(SUM(total_budget), 0) as total_budget,
                   COALESCE(SUM(total_approved), 0) as total_approved,
                   COALESCE(SUM(total_employees), 0) as total_employees
            FROM _atlas.compensation_cycles
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let ws_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE w.status = 'draft') as pending,
                COUNT(*) FILTER (WHERE w.status = 'approved') as completed
            FROM _atlas.compensation_worksheets w
            JOIN _atlas.compensation_cycles c ON c.id = w.cycle_id
            WHERE c.organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        let total_budget: f64 = cycle_row.try_get("total_budget").unwrap_or(0.0);
        let total_approved: f64 = cycle_row.try_get("total_approved").unwrap_or(0.0);
        let budget_util = if total_budget > 0.0 { (total_approved / total_budget) * 100.0 } else { 0.0 };

        Ok(CompensationDashboard {
            active_plans: plan_row.get::<i64, _>("cnt") as i32,
            active_cycles: cycle_row.get::<i64, _>("active") as i32,
            total_budget: format!("{:.2}", total_budget),
            total_allocated: format!("{:.2}", total_approved),
            total_approved: format!("{:.2}", total_approved),
            total_employees_in_cycle: cycle_row.get::<i64, _>("total_employees") as i32,
            pending_worksheets: ws_row.get::<i64, _>("pending") as i32,
            completed_worksheets: ws_row.get::<i64, _>("completed") as i32,
            average_salary_increase_percent: "0.00".to_string(),
            budget_utilization_percent: format!("{:.2}", budget_util),
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn get_num4(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.4}", v)
}

fn row_to_plan(row: &sqlx::postgres::PgRow) -> CompensationPlan {
    CompensationPlan {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        plan_code: row.get("plan_code"),
        plan_name: row.get("plan_name"),
        description: row.get("description"),
        plan_type: row.get("plan_type"),
        status: row.get("status"),
        effective_start_date: row.get("effective_start_date"),
        effective_end_date: row.get("effective_end_date"),
        eligibility_criteria: row.get("eligibility_criteria"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_component(row: &sqlx::postgres::PgRow) -> CompensationComponent {
    CompensationComponent {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        plan_id: row.get("plan_id"),
        component_name: row.get("component_name"),
        component_type: row.get("component_type"),
        description: row.get("description"),
        is_recurring: row.get("is_recurring"),
        frequency: row.get("frequency"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cycle(row: &sqlx::postgres::PgRow) -> CompensationCycle {
    CompensationCycle {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cycle_name: row.get("cycle_name"),
        description: row.get("description"),
        cycle_type: row.get("cycle_type"),
        status: row.get("status"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        allocation_start_date: row.get("allocation_start_date"),
        allocation_end_date: row.get("allocation_end_date"),
        review_start_date: row.get("review_start_date"),
        review_end_date: row.get("review_end_date"),
        total_budget: get_num(row, "total_budget"),
        total_allocated: get_num(row, "total_allocated"),
        total_approved: get_num(row, "total_approved"),
        total_employees: row.get("total_employees"),
        currency_code: row.get("currency_code"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_budget_pool(row: &sqlx::postgres::PgRow) -> CompensationBudgetPool {
    CompensationBudgetPool {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cycle_id: row.get("cycle_id"),
        pool_name: row.get("pool_name"),
        pool_type: row.get("pool_type"),
        manager_id: row.get("manager_id"),
        manager_name: row.get("manager_name"),
        department_id: row.get("department_id"),
        department_name: row.get("department_name"),
        total_budget: get_num(row, "total_budget"),
        allocated_amount: get_num(row, "allocated_amount"),
        approved_amount: get_num(row, "approved_amount"),
        remaining_budget: get_num(row, "remaining_budget"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_worksheet(row: &sqlx::postgres::PgRow) -> CompensationWorksheet {
    CompensationWorksheet {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cycle_id: row.get("cycle_id"),
        pool_id: row.get("pool_id"),
        manager_id: row.get("manager_id"),
        manager_name: row.get("manager_name"),
        status: row.get("status"),
        total_employees: row.get("total_employees"),
        total_current_salary: get_num(row, "total_current_salary"),
        total_proposed_salary: get_num(row, "total_proposed_salary"),
        total_merit: get_num(row, "total_merit"),
        total_bonus: get_num(row, "total_bonus"),
        total_equity: get_num(row, "total_equity"),
        total_compensation_change: get_num(row, "total_compensation_change"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_worksheet_line(row: &sqlx::postgres::PgRow) -> CompensationWorksheetLine {
    CompensationWorksheetLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        worksheet_id: row.get("worksheet_id"),
        employee_id: row.get("employee_id"),
        employee_name: row.get("employee_name"),
        job_title: row.get("job_title"),
        department_name: row.get("department_name"),
        current_base_salary: get_num(row, "current_base_salary"),
        proposed_base_salary: get_num(row, "proposed_base_salary"),
        salary_change_amount: get_num(row, "salary_change_amount"),
        salary_change_percent: get_num4(row, "salary_change_percent"),
        merit_amount: get_num(row, "merit_amount"),
        bonus_amount: get_num(row, "bonus_amount"),
        equity_amount: get_num(row, "equity_amount"),
        total_compensation: get_num(row, "total_compensation"),
        performance_rating: row.get("performance_rating"),
        compa_ratio: get_num4(row, "compa_ratio"),
        status: row.get("status"),
        manager_comments: row.get("manager_comments"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_statement(row: &sqlx::postgres::PgRow) -> CompensationStatement {
    CompensationStatement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cycle_id: row.get("cycle_id"),
        employee_id: row.get("employee_id"),
        employee_name: row.get("employee_name"),
        statement_date: row.get("statement_date"),
        base_salary: get_num(row, "base_salary"),
        merit_increase: get_num(row, "merit_increase"),
        bonus: get_num(row, "bonus"),
        equity: get_num(row, "equity"),
        benefits_value: get_num(row, "benefits_value"),
        total_compensation: get_num(row, "total_compensation"),
        total_direct_compensation: get_num(row, "total_direct_compensation"),
        total_indirect_compensation: get_num(row, "total_indirect_compensation"),
        change_from_previous: get_num(row, "change_from_previous"),
        change_percent: get_num4(row, "change_percent"),
        currency_code: row.get("currency_code"),
        components: row.get("components"),
        status: row.get("status"),
        published_at: row.get("published_at"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
