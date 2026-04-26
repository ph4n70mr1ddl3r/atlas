//! Payroll Repository
//!
//! PostgreSQL storage for payroll definitions, elements, entries, runs, and pay slips.

use atlas_shared::{
    PayrollDefinition, PayrollElement, PayrollElementEntry,
    PayrollRun, PaySlip, PaySlipLine, PayrollDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for payroll data storage
#[async_trait]
pub trait PayrollRepository: Send + Sync {
    // Payroll Definitions
    async fn create_payroll(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        pay_frequency: &str,
        currency_code: &str,
        salary_expense_account: Option<&str>,
        liability_account: Option<&str>,
        employer_tax_account: Option<&str>,
        payment_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollDefinition>;

    async fn get_payroll(&self, id: Uuid) -> AtlasResult<Option<PayrollDefinition>>;
    async fn get_payroll_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<PayrollDefinition>>;
    async fn list_payrolls(&self, org_id: Uuid) -> AtlasResult<Vec<PayrollDefinition>>;
    async fn delete_payroll(&self, id: Uuid) -> AtlasResult<()>;

    // Payroll Elements
    async fn create_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        category: &str,
        calculation_method: &str,
        default_value: Option<&str>,
        is_recurring: bool,
        has_employer_contribution: bool,
        employer_contribution_rate: Option<&str>,
        gl_account_code: Option<&str>,
        is_pretax: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElement>;

    async fn get_element(&self, id: Uuid) -> AtlasResult<Option<PayrollElement>>;
    async fn get_element_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PayrollElement>>;
    async fn list_elements(&self, org_id: Uuid, element_type: Option<&str>) -> AtlasResult<Vec<PayrollElement>>;
    async fn delete_element(&self, id: Uuid) -> AtlasResult<()>;

    // Element Entries (employee assignments)
    async fn create_entry(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        element_id: Uuid,
        element_code: &str,
        element_name: &str,
        element_type: &str,
        entry_value: &str,
        remaining_periods: Option<i32>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElementEntry>;

    async fn get_entries_by_employee(&self, employee_id: Uuid) -> AtlasResult<Vec<PayrollElementEntry>>;
    async fn delete_entry(&self, id: Uuid) -> AtlasResult<()>;

    // Payroll Runs
    async fn create_run(
        &self,
        org_id: Uuid,
        payroll_id: Uuid,
        run_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        pay_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollRun>;

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<PayrollRun>>;
    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<PayrollRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PayrollRun>>;
    async fn update_run_status(&self, id: Uuid, status: &str, action_by: Option<Uuid>) -> AtlasResult<PayrollRun>;
    async fn update_run_totals(
        &self,
        id: Uuid,
        total_gross: &str,
        total_deductions: &str,
        total_net: &str,
        total_employer_cost: &str,
        employee_count: i32,
    ) -> AtlasResult<()>;

    // Pay Slips
    async fn create_pay_slip(
        &self,
        org_id: Uuid,
        payroll_run_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        gross_earnings: &str,
        total_deductions: &str,
        net_pay: &str,
        employer_cost: &str,
        currency_code: &str,
        payment_method: Option<&str>,
        bank_account_last4: Option<&str>,
    ) -> AtlasResult<PaySlip>;

    async fn get_pay_slip(&self, id: Uuid) -> AtlasResult<Option<PaySlip>>;
    async fn list_pay_slips_by_run(&self, payroll_run_id: Uuid) -> AtlasResult<Vec<PaySlip>>;
    async fn list_pay_slips_by_employee(&self, employee_id: Uuid) -> AtlasResult<Vec<PaySlip>>;

    // Pay Slip Lines
    async fn create_pay_slip_line(
        &self,
        pay_slip_id: Uuid,
        element_code: &str,
        element_name: &str,
        element_type: &str,
        category: &str,
        hours_or_units: Option<&str>,
        rate: Option<&str>,
        amount: &str,
        is_pretax: bool,
        is_employer: bool,
        gl_account_code: Option<&str>,
    ) -> AtlasResult<PaySlipLine>;

    async fn list_pay_slip_lines(&self, pay_slip_id: Uuid) -> AtlasResult<Vec<PaySlipLine>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PayrollDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresPayrollRepository {
    pool: PgPool,
}

impl PostgresPayrollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PayrollRepository for PostgresPayrollRepository {
    // ========================================================================
    // Payroll Definitions
    // ========================================================================

    async fn create_payroll(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        pay_frequency: &str,
        currency_code: &str,
        salary_expense_account: Option<&str>,
        liability_account: Option<&str>,
        employer_tax_account: Option<&str>,
        payment_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollDefinition> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payroll_definitions
                (organization_id, name, description, pay_frequency, currency_code,
                 salary_expense_account, liability_account, employer_tax_account,
                 payment_account, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(name).bind(description).bind(pay_frequency)
        .bind(currency_code).bind(salary_expense_account)
        .bind(liability_account).bind(employer_tax_account)
        .bind(payment_account).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PayrollDefinition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            pay_frequency: row.get("pay_frequency"),
            currency_code: row.get("currency_code"),
            salary_expense_account: row.get("salary_expense_account"),
            liability_account: row.get("liability_account"),
            employer_tax_account: row.get("employer_tax_account"),
            payment_account: row.get("payment_account"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    async fn get_payroll(&self, id: Uuid) -> AtlasResult<Option<PayrollDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_definitions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PayrollDefinition {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            name: r.get("name"),
            description: r.get("description"),
            pay_frequency: r.get("pay_frequency"),
            currency_code: r.get("currency_code"),
            salary_expense_account: r.get("salary_expense_account"),
            liability_account: r.get("liability_account"),
            employer_tax_account: r.get("employer_tax_account"),
            payment_account: r.get("payment_account"),
            is_active: r.get("is_active"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }))
    }

    async fn get_payroll_by_name(&self, org_id: Uuid, name: &str) -> AtlasResult<Option<PayrollDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_definitions WHERE organization_id = $1 AND name = $2 AND is_active = true"
        )
        .bind(org_id).bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PayrollDefinition {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            name: r.get("name"),
            description: r.get("description"),
            pay_frequency: r.get("pay_frequency"),
            currency_code: r.get("currency_code"),
            salary_expense_account: r.get("salary_expense_account"),
            liability_account: r.get("liability_account"),
            employer_tax_account: r.get("employer_tax_account"),
            payment_account: r.get("payment_account"),
            is_active: r.get("is_active"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }))
    }

    async fn list_payrolls(&self, org_id: Uuid) -> AtlasResult<Vec<PayrollDefinition>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.payroll_definitions WHERE organization_id = $1 AND is_active = true ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PayrollDefinition {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            name: r.get("name"),
            description: r.get("description"),
            pay_frequency: r.get("pay_frequency"),
            currency_code: r.get("currency_code"),
            salary_expense_account: r.get("salary_expense_account"),
            liability_account: r.get("liability_account"),
            employer_tax_account: r.get("employer_tax_account"),
            payment_account: r.get("payment_account"),
            is_active: r.get("is_active"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect())
    }

    async fn delete_payroll(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.payroll_definitions SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Payroll Elements
    // ========================================================================

    async fn create_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        category: &str,
        calculation_method: &str,
        default_value: Option<&str>,
        is_recurring: bool,
        has_employer_contribution: bool,
        employer_contribution_rate: Option<&str>,
        gl_account_code: Option<&str>,
        is_pretax: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payroll_elements
                (organization_id, code, name, description, element_type, category,
                 calculation_method, default_value, is_recurring,
                 has_employer_contribution, employer_contribution_rate,
                 gl_account_code, is_pretax,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9, $10,
                    $11::numeric, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(element_type).bind(category).bind(calculation_method)
        .bind(default_value).bind(is_recurring)
        .bind(has_employer_contribution).bind(employer_contribution_rate)
        .bind(gl_account_code).bind(is_pretax)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PayrollElement {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            element_type: row.get("element_type"),
            category: row.get("category"),
            calculation_method: row.get("calculation_method"),
            default_value: row.try_get("default_value").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            is_recurring: row.get("is_recurring"),
            has_employer_contribution: row.get("has_employer_contribution"),
            employer_contribution_rate: row.try_get("employer_contribution_rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            gl_account_code: row.get("gl_account_code"),
            is_pretax: row.get("is_pretax"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    async fn get_element(&self, id: Uuid) -> AtlasResult<Option<PayrollElement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_elements WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PayrollElement {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            element_type: r.get("element_type"),
            category: r.get("category"),
            calculation_method: r.get("calculation_method"),
            default_value: r.try_get("default_value").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            is_recurring: r.get("is_recurring"),
            has_employer_contribution: r.get("has_employer_contribution"),
            employer_contribution_rate: r.try_get("employer_contribution_rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            gl_account_code: r.get("gl_account_code"),
            is_pretax: r.get("is_pretax"),
            is_active: r.get("is_active"),
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }))
    }

    async fn get_element_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PayrollElement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_elements WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PayrollElement {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            element_type: r.get("element_type"),
            category: r.get("category"),
            calculation_method: r.get("calculation_method"),
            default_value: r.try_get("default_value").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            is_recurring: r.get("is_recurring"),
            has_employer_contribution: r.get("has_employer_contribution"),
            employer_contribution_rate: r.try_get("employer_contribution_rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            gl_account_code: r.get("gl_account_code"),
            is_pretax: r.get("is_pretax"),
            is_active: r.get("is_active"),
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }))
    }

    async fn list_elements(&self, org_id: Uuid, element_type: Option<&str>) -> AtlasResult<Vec<PayrollElement>> {
        let rows = match element_type {
            Some(et) => sqlx::query(
                "SELECT * FROM _atlas.payroll_elements WHERE organization_id = $1 AND element_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(et)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.payroll_elements WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PayrollElement {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            code: r.get("code"),
            name: r.get("name"),
            description: r.get("description"),
            element_type: r.get("element_type"),
            category: r.get("category"),
            calculation_method: r.get("calculation_method"),
            default_value: r.try_get("default_value").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            is_recurring: r.get("is_recurring"),
            has_employer_contribution: r.get("has_employer_contribution"),
            employer_contribution_rate: r.try_get("employer_contribution_rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            gl_account_code: r.get("gl_account_code"),
            is_pretax: r.get("is_pretax"),
            is_active: r.get("is_active"),
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect())
    }

    async fn delete_element(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.payroll_elements SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Element Entries
    // ========================================================================

    async fn create_entry(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        element_id: Uuid,
        element_code: &str,
        element_name: &str,
        element_type: &str,
        entry_value: &str,
        remaining_periods: Option<i32>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElementEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payroll_element_entries
                (organization_id, employee_id, element_id, element_code, element_name,
                 element_type, entry_value, remaining_periods,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(employee_id).bind(element_id)
        .bind(element_code).bind(element_name).bind(element_type)
        .bind(entry_value).bind(remaining_periods)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PayrollElementEntry {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            employee_id: row.get("employee_id"),
            element_id: row.get("element_id"),
            element_code: row.get("element_code"),
            element_name: row.get("element_name"),
            element_type: row.get("element_type"),
            entry_value: row.try_get("entry_value").ok()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            remaining_periods: row.get("remaining_periods"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    async fn get_entries_by_employee(&self, employee_id: Uuid) -> AtlasResult<Vec<PayrollElementEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.payroll_element_entries WHERE employee_id = $1 AND is_active = true ORDER BY element_type, element_code"
        )
        .bind(employee_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PayrollElementEntry {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            employee_id: r.get("employee_id"),
            element_id: r.get("element_id"),
            element_code: r.get("element_code"),
            element_name: r.get("element_name"),
            element_type: r.get("element_type"),
            entry_value: r.try_get("entry_value").ok()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
            remaining_periods: r.get("remaining_periods"),
            is_active: r.get("is_active"),
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            metadata: r.get("metadata"),
        }).collect())
    }

    async fn delete_entry(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.payroll_element_entries SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Payroll Runs
    // ========================================================================

    async fn create_run(
        &self,
        org_id: Uuid,
        payroll_id: Uuid,
        run_number: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        pay_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollRun> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.payroll_runs
                (organization_id, payroll_id, run_number, status,
                 period_start, period_end, pay_date,
                 total_gross, total_deductions, total_net, total_employer_cost,
                 employee_count, created_by)
            VALUES ($1, $2, $3, 'open', $4, $5, $6, '0', '0', '0', '0', 0, $7)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payroll_id).bind(run_number)
        .bind(period_start).bind(period_end).bind(pay_date)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_run(&row))
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<PayrollRun>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_runs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<PayrollRun>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.payroll_runs WHERE organization_id = $1 AND run_number = $2"
        )
        .bind(org_id).bind(run_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PayrollRun>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.payroll_runs WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.payroll_runs WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run).collect())
    }

    async fn update_run_status(&self, id: Uuid, status: &str, action_by: Option<Uuid>) -> AtlasResult<PayrollRun> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.payroll_runs
            SET status = $2,
                confirmed_by = CASE WHEN $2 IN ('confirmed') THEN $3 ELSE confirmed_by END,
                confirmed_at = CASE WHEN $2 IN ('confirmed') THEN now() ELSE confirmed_at END,
                paid_by = CASE WHEN $2 IN ('paid') THEN $3 ELSE paid_by END,
                paid_at = CASE WHEN $2 IN ('paid') THEN now() ELSE paid_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(action_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    async fn update_run_totals(
        &self,
        id: Uuid,
        total_gross: &str,
        total_deductions: &str,
        total_net: &str,
        total_employer_cost: &str,
        employee_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.payroll_runs
            SET total_gross = $2::numeric, total_deductions = $3::numeric,
                total_net = $4::numeric, total_employer_cost = $5::numeric,
                employee_count = $6, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_gross).bind(total_deductions)
        .bind(total_net).bind(total_employer_cost).bind(employee_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Pay Slips
    // ========================================================================

    async fn create_pay_slip(
        &self,
        org_id: Uuid,
        payroll_run_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        gross_earnings: &str,
        total_deductions: &str,
        net_pay: &str,
        employer_cost: &str,
        currency_code: &str,
        payment_method: Option<&str>,
        bank_account_last4: Option<&str>,
    ) -> AtlasResult<PaySlip> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pay_slips
                (organization_id, payroll_run_id, employee_id, employee_name,
                 gross_earnings, total_deductions, net_pay, employer_cost,
                 currency_code, payment_method, bank_account_last4)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8::numeric,
                    $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payroll_run_id).bind(employee_id).bind(employee_name)
        .bind(gross_earnings).bind(total_deductions).bind(net_pay).bind(employer_cost)
        .bind(currency_code).bind(payment_method).bind(bank_account_last4)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PaySlip {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            payroll_run_id: row.get("payroll_run_id"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            gross_earnings: numeric_to_string(&row, "gross_earnings"),
            total_deductions: numeric_to_string(&row, "total_deductions"),
            net_pay: numeric_to_string(&row, "net_pay"),
            employer_cost: numeric_to_string(&row, "employer_cost"),
            currency_code: row.get("currency_code"),
            payment_method: row.get("payment_method"),
            bank_account_last4: row.get("bank_account_last4"),
            lines: vec![],
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    async fn get_pay_slip(&self, id: Uuid) -> AtlasResult<Option<PaySlip>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pay_slips WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(row_to_slip))
    }

    async fn list_pay_slips_by_run(&self, payroll_run_id: Uuid) -> AtlasResult<Vec<PaySlip>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pay_slips WHERE payroll_run_id = $1 ORDER BY employee_name"
        )
        .bind(payroll_run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_slip).collect())
    }

    async fn list_pay_slips_by_employee(&self, employee_id: Uuid) -> AtlasResult<Vec<PaySlip>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pay_slips WHERE employee_id = $1 ORDER BY created_at DESC"
        )
        .bind(employee_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_slip).collect())
    }

    // ========================================================================
    // Pay Slip Lines
    // ========================================================================

    async fn create_pay_slip_line(
        &self,
        pay_slip_id: Uuid,
        element_code: &str,
        element_name: &str,
        element_type: &str,
        category: &str,
        hours_or_units: Option<&str>,
        rate: Option<&str>,
        amount: &str,
        is_pretax: bool,
        is_employer: bool,
        gl_account_code: Option<&str>,
    ) -> AtlasResult<PaySlipLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pay_slip_lines
                (pay_slip_id, element_code, element_name, element_type, category,
                 hours_or_units, rate, amount, is_pretax, is_employer, gl_account_code)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(pay_slip_id).bind(element_code).bind(element_name)
        .bind(element_type).bind(category)
        .bind(hours_or_units).bind(rate).bind(amount)
        .bind(is_pretax).bind(is_employer).bind(gl_account_code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PaySlipLine {
            id: row.get("id"),
            pay_slip_id: row.get("pay_slip_id"),
            element_code: row.get("element_code"),
            element_name: row.get("element_name"),
            element_type: row.get("element_type"),
            category: row.get("category"),
            hours_or_units: row.try_get("hours_or_units").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            rate: row.try_get("rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            amount: numeric_to_string(&row, "amount"),
            is_pretax: row.get("is_pretax"),
            is_employer: row.get("is_employer"),
            gl_account_code: row.get("gl_account_code"),
            created_at: row.get("created_at"),
        })
    }

    async fn list_pay_slip_lines(&self, pay_slip_id: Uuid) -> AtlasResult<Vec<PaySlipLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pay_slip_lines WHERE pay_slip_id = $1 ORDER BY element_type, id"
        )
        .bind(pay_slip_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PaySlipLine {
            id: r.get("id"),
            pay_slip_id: r.get("pay_slip_id"),
            element_code: r.get("element_code"),
            element_name: r.get("element_name"),
            element_type: r.get("element_type"),
            category: r.get("category"),
            hours_or_units: r.try_get("hours_or_units").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            rate: r.try_get("rate").ok().flatten()
                .map(|v: serde_json::Value| v.to_string().trim_matches('"').to_string()),
            amount: numeric_to_string(r, "amount"),
            is_pretax: r.get("is_pretax"),
            is_employer: r.get("is_employer"),
            gl_account_code: r.get("gl_account_code"),
            created_at: r.get("created_at"),
        }).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PayrollDashboard> {
        // Get totals from recent runs
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(total_gross), 0) as total_gross,
                COALESCE(SUM(total_deductions), 0) as total_deductions,
                COALESCE(SUM(total_net), 0) as total_net,
                COALESCE(SUM(total_employer_cost), 0) as total_employer_cost,
                COALESCE(SUM(employee_count), 0) as employee_count
            FROM _atlas.payroll_runs
            WHERE organization_id = $1 AND status IN ('confirmed', 'paid')
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PayrollDashboard {
            total_gross: numeric_to_string(&row, "total_gross"),
            total_deductions: numeric_to_string(&row, "total_deductions"),
            total_net: numeric_to_string(&row, "total_net"),
            total_employer_cost: numeric_to_string(&row, "total_employer_cost"),
            employee_count: row.try_get("employee_count").unwrap_or(0),
            payroll_runs_this_period: 0,
            recent_runs: vec![],
            top_earnings_by_category: serde_json::json!({}),
            top_deductions_by_category: serde_json::json!({}),
        })
    }
}

fn numeric_to_string(row: &sqlx::postgres::PgRow, field: &str) -> String {
    row.try_get::<serde_json::Value, _>(field)
        .ok()
        .map(|v| v.to_string().trim_matches('"').to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn row_to_run(row: &sqlx::postgres::PgRow) -> PayrollRun {
    PayrollRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        payroll_id: row.get("payroll_id"),
        run_number: row.get("run_number"),
        status: row.get("status"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        pay_date: row.get("pay_date"),
        total_gross: numeric_to_string(row, "total_gross"),
        total_deductions: numeric_to_string(row, "total_deductions"),
        total_net: numeric_to_string(row, "total_net"),
        total_employer_cost: numeric_to_string(row, "total_employer_cost"),
        employee_count: row.get("employee_count"),
        confirmed_by: row.get("confirmed_by"),
        confirmed_at: row.get("confirmed_at"),
        paid_by: row.get("paid_by"),
        paid_at: row.get("paid_at"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        metadata: row.get("metadata"),
    }
}

fn row_to_slip(row: &sqlx::postgres::PgRow) -> PaySlip {
    PaySlip {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        payroll_run_id: row.get("payroll_run_id"),
        employee_id: row.get("employee_id"),
        employee_name: row.get("employee_name"),
        gross_earnings: numeric_to_string(row, "gross_earnings"),
        total_deductions: numeric_to_string(row, "total_deductions"),
        net_pay: numeric_to_string(row, "net_pay"),
        employer_cost: numeric_to_string(row, "employer_cost"),
        currency_code: row.get("currency_code"),
        payment_method: row.get("payment_method"),
        bank_account_last4: row.get("bank_account_last4"),
        lines: vec![],
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        metadata: row.get("metadata"),
    }
}
