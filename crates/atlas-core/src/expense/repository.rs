//! Expense Repository
//!
//! PostgreSQL storage for expense categories, policies, reports, and lines.

use atlas_shared::{
    ExpenseCategory, ExpensePolicy, ExpenseReport, ExpenseLine,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for expense data storage
#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    // Expense Categories
    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        receipt_required: bool,
        receipt_threshold: Option<&str>,
        is_per_diem: bool,
        default_per_diem_rate: Option<&str>,
        is_mileage: bool,
        default_mileage_rate: Option<&str>,
        expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseCategory>;

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExpenseCategory>>;
    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpenseCategory>>;
    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseCategory>>;
    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Expense Policies
    async fn create_policy(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        min_amount: Option<&str>,
        max_amount: Option<&str>,
        daily_limit: Option<&str>,
        report_limit: Option<&str>,
        requires_approval_on_violation: bool,
        violation_action: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicy>;

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<ExpensePolicy>>;
    async fn list_policies(&self, org_id: Uuid, category_id: Option<Uuid>) -> AtlasResult<Vec<ExpensePolicy>>;
    async fn delete_policy(&self, id: Uuid) -> AtlasResult<()>;

    // Expense Reports
    async fn create_report(
        &self,
        org_id: Uuid,
        report_number: &str,
        title: &str,
        description: Option<&str>,
        employee_id: Uuid,
        employee_name: Option<&str>,
        department_id: Option<Uuid>,
        purpose: Option<&str>,
        project_id: Option<Uuid>,
        currency_code: &str,
        trip_start_date: Option<chrono::NaiveDate>,
        trip_end_date: Option<chrono::NaiveDate>,
        cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseReport>;

    async fn get_report(&self, id: Uuid) -> AtlasResult<Option<ExpenseReport>>;
    async fn get_report_by_number(&self, org_id: Uuid, report_number: &str) -> AtlasResult<Option<ExpenseReport>>;
    async fn list_reports(&self, org_id: Uuid, employee_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ExpenseReport>>;
    async fn update_report_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejection_reason: Option<&str>,
        reimbursed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ExpenseReport>;
    async fn update_report_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        reimbursable_amount: &str,
        receipt_required_amount: &str,
        receipt_count: i32,
    ) -> AtlasResult<()>;

    // Expense Lines
    async fn create_line(
        &self,
        org_id: Uuid,
        report_id: Uuid,
        line_number: i32,
        expense_category_id: Option<Uuid>,
        expense_category_name: Option<&str>,
        expense_type: &str,
        description: Option<&str>,
        expense_date: chrono::NaiveDate,
        amount: &str,
        original_currency: Option<&str>,
        original_amount: Option<&str>,
        exchange_rate: Option<&str>,
        is_reimbursable: bool,
        has_receipt: bool,
        receipt_reference: Option<&str>,
        merchant_name: Option<&str>,
        location: Option<&str>,
        attendees: Option<serde_json::Value>,
        per_diem_days: Option<f64>,
        per_diem_rate: Option<&str>,
        mileage_distance: Option<f64>,
        mileage_rate: Option<&str>,
        mileage_unit: Option<&str>,
        mileage_from: Option<&str>,
        mileage_to: Option<&str>,
        policy_violation: bool,
        policy_violation_message: Option<&str>,
        expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseLine>;

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<ExpenseLine>>;
    async fn list_lines_by_report(&self, report_id: Uuid) -> AtlasResult<Vec<ExpenseLine>>;
    async fn delete_line(&self, id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresExpenseRepository {
    pool: PgPool,
}

impl PostgresExpenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_category(&self, row: &sqlx::postgres::PgRow) -> ExpenseCategory {
        let receipt_threshold: Option<serde_json::Value> = row.try_get("receipt_threshold").ok().flatten();
        let per_diem_rate: Option<serde_json::Value> = row.try_get("default_per_diem_rate").ok().flatten();
        let mileage_rate: Option<serde_json::Value> = row.try_get("default_mileage_rate").ok().flatten();

        ExpenseCategory {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            receipt_required: row.get("receipt_required"),
            receipt_threshold: receipt_threshold.map(|v| v.to_string()),
            is_per_diem: row.get("is_per_diem"),
            default_per_diem_rate: per_diem_rate.map(|v| v.to_string()),
            is_mileage: row.get("is_mileage"),
            default_mileage_rate: mileage_rate.map(|v| v.to_string()),
            expense_account_code: row.get("expense_account_code"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_policy(&self, row: &sqlx::postgres::PgRow) -> ExpensePolicy {
        let min_amount: Option<serde_json::Value> = row.try_get("min_amount").ok().flatten();
        let max_amount: Option<serde_json::Value> = row.try_get("max_amount").ok().flatten();
        let daily_limit: Option<serde_json::Value> = row.try_get("daily_limit").ok().flatten();
        let report_limit: Option<serde_json::Value> = row.try_get("report_limit").ok().flatten();

        ExpensePolicy {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            category_id: row.get("category_id"),
            min_amount: min_amount.map(|v| v.to_string()),
            max_amount: max_amount.map(|v| v.to_string()),
            daily_limit: daily_limit.map(|v| v.to_string()),
            report_limit: report_limit.map(|v| v.to_string()),
            requires_approval_on_violation: row.get("requires_approval_on_violation"),
            violation_action: row.get("violation_action"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_report(&self, row: &sqlx::postgres::PgRow) -> ExpenseReport {
        let total_amount: serde_json::Value = row.try_get("total_amount").unwrap_or(serde_json::json!("0"));
        let reimbursable_amount: serde_json::Value = row.try_get("reimbursable_amount").unwrap_or(serde_json::json!("0"));
        let receipt_required_amount: serde_json::Value = row.try_get("receipt_required_amount").unwrap_or(serde_json::json!("0"));

        ExpenseReport {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            report_number: row.get("report_number"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            employee_id: row.get("employee_id"),
            employee_name: row.get("employee_name"),
            department_id: row.get("department_id"),
            purpose: row.get("purpose"),
            project_id: row.get("project_id"),
            currency_code: row.get("currency_code"),
            total_amount: total_amount.to_string(),
            reimbursable_amount: reimbursable_amount.to_string(),
            receipt_required_amount: receipt_required_amount.to_string(),
            receipt_count: row.get("receipt_count"),
            trip_start_date: row.get("trip_start_date"),
            trip_end_date: row.get("trip_end_date"),
            cost_center: row.get("cost_center"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejection_reason: row.get("rejection_reason"),
            payment_method: row.get("payment_method"),
            payment_reference: row.get("payment_reference"),
            reimbursed_at: row.get("reimbursed_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> ExpenseLine {
        let amount: serde_json::Value = row.try_get("amount").unwrap_or(serde_json::json!("0"));
        let original_amount: Option<serde_json::Value> = row.try_get("original_amount").ok().flatten();
        let exchange_rate: Option<serde_json::Value> = row.try_get("exchange_rate").ok().flatten();
        let per_diem_rate: Option<serde_json::Value> = row.try_get("per_diem_rate").ok().flatten();
        let mileage_rate: Option<serde_json::Value> = row.try_get("mileage_rate").ok().flatten();

        ExpenseLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            report_id: row.get("report_id"),
            line_number: row.get("line_number"),
            expense_category_id: row.get("expense_category_id"),
            expense_category_name: row.get("expense_category_name"),
            expense_type: row.get("expense_type"),
            description: row.get("description"),
            expense_date: row.get("expense_date"),
            amount: amount.to_string(),
            original_currency: row.get("original_currency"),
            original_amount: original_amount.map(|v| v.to_string()),
            exchange_rate: exchange_rate.map(|v| v.to_string()),
            is_reimbursable: row.get("is_reimbursable"),
            has_receipt: row.get("has_receipt"),
            receipt_reference: row.get("receipt_reference"),
            merchant_name: row.get("merchant_name"),
            location: row.get("location"),
            attendees: row.get("attendees"),
            per_diem_days: row.try_get("per_diem_days").ok().flatten(),
            per_diem_rate: per_diem_rate.map(|v| v.to_string()),
            mileage_distance: row.try_get("mileage_distance").ok().flatten(),
            mileage_rate: mileage_rate.map(|v| v.to_string()),
            mileage_unit: row.get("mileage_unit"),
            mileage_from: row.get("mileage_from"),
            mileage_to: row.get("mileage_to"),
            policy_violation: row.get("policy_violation"),
            policy_violation_message: row.get("policy_violation_message"),
            expense_account_code: row.get("expense_account_code"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl ExpenseRepository for PostgresExpenseRepository {
    // ========================================================================
    // Expense Categories
    // ========================================================================

    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        receipt_required: bool,
        receipt_threshold: Option<&str>,
        is_per_diem: bool,
        default_per_diem_rate: Option<&str>,
        is_mileage: bool,
        default_mileage_rate: Option<&str>,
        expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseCategory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.expense_categories
                (organization_id, code, name, description,
                 receipt_required, receipt_threshold,
                 is_per_diem, default_per_diem_rate,
                 is_mileage, default_mileage_rate,
                 expense_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8::numeric, $9, $10::numeric, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4,
                    receipt_required = $5, receipt_threshold = $6::numeric,
                    is_per_diem = $7, default_per_diem_rate = $8::numeric,
                    is_mileage = $9, default_mileage_rate = $10::numeric,
                    expense_account_code = $11, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(receipt_required).bind(receipt_threshold)
        .bind(is_per_diem).bind(default_per_diem_rate)
        .bind(is_mileage).bind(default_mileage_rate)
        .bind(expense_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_category(&row))
    }

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExpenseCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_categories WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpenseCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_categories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.expense_categories WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_category(&r)).collect())
    }

    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.expense_categories SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Expense Policies
    // ========================================================================

    async fn create_policy(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        min_amount: Option<&str>,
        max_amount: Option<&str>,
        daily_limit: Option<&str>,
        report_limit: Option<&str>,
        requires_approval_on_violation: bool,
        violation_action: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicy> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.expense_policies
                (organization_id, name, description, category_id,
                 min_amount, max_amount, daily_limit, report_limit,
                 requires_approval_on_violation, violation_action,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8::numeric,
                    $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(name).bind(description).bind(category_id)
        .bind(min_amount).bind(max_amount).bind(daily_limit).bind(report_limit)
        .bind(requires_approval_on_violation).bind(violation_action)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_policy(&row))
    }

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<ExpensePolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_policies WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_policy(&r)))
    }

    async fn list_policies(&self, org_id: Uuid, category_id: Option<Uuid>) -> AtlasResult<Vec<ExpensePolicy>> {
        let rows = match category_id {
            Some(cid) => sqlx::query(
                "SELECT * FROM _atlas.expense_policies WHERE organization_id = $1 AND (category_id = $2 OR category_id IS NULL) AND is_active = true ORDER BY name"
            )
            .bind(org_id).bind(cid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.expense_policies WHERE organization_id = $1 AND is_active = true ORDER BY name"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_policy(&r)).collect())
    }

    async fn delete_policy(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.expense_policies SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Expense Reports
    // ========================================================================

    async fn create_report(
        &self,
        org_id: Uuid,
        report_number: &str,
        title: &str,
        description: Option<&str>,
        employee_id: Uuid,
        employee_name: Option<&str>,
        department_id: Option<Uuid>,
        purpose: Option<&str>,
        project_id: Option<Uuid>,
        currency_code: &str,
        trip_start_date: Option<chrono::NaiveDate>,
        trip_end_date: Option<chrono::NaiveDate>,
        cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseReport> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.expense_reports
                (organization_id, report_number, title, description,
                 employee_id, employee_name, department_id, purpose,
                 project_id, currency_code,
                 trip_start_date, trip_end_date, cost_center,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(report_number).bind(title).bind(description)
        .bind(employee_id).bind(employee_name).bind(department_id).bind(purpose)
        .bind(project_id).bind(currency_code)
        .bind(trip_start_date).bind(trip_end_date).bind(cost_center)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_report(&row))
    }

    async fn get_report(&self, id: Uuid) -> AtlasResult<Option<ExpenseReport>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_reports WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_report(&r)))
    }

    async fn get_report_by_number(&self, org_id: Uuid, report_number: &str) -> AtlasResult<Option<ExpenseReport>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_reports WHERE organization_id = $1 AND report_number = $2"
        )
        .bind(org_id).bind(report_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_report(&r)))
    }

    async fn list_reports(&self, org_id: Uuid, employee_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ExpenseReport>> {
        let rows = match (employee_id, status) {
            (Some(eid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.expense_reports WHERE organization_id = $1 AND employee_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(eid), None) => sqlx::query(
                "SELECT * FROM _atlas.expense_reports WHERE organization_id = $1 AND employee_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.expense_reports WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.expense_reports WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_report(&r)).collect())
    }

    async fn update_report_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejection_reason: Option<&str>,
        reimbursed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ExpenseReport> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.expense_reports
            SET status = $2, approved_by = $3, rejection_reason = $4,
                reimbursed_at = $5, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejection_reason).bind(reimbursed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_report(&row))
    }

    async fn update_report_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        reimbursable_amount: &str,
        receipt_required_amount: &str,
        receipt_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.expense_reports
            SET total_amount = $2::numeric, reimbursable_amount = $3::numeric,
                receipt_required_amount = $4::numeric, receipt_count = $5,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_amount).bind(reimbursable_amount).bind(receipt_required_amount).bind(receipt_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Expense Lines
    // ========================================================================

    async fn create_line(
        &self,
        org_id: Uuid,
        report_id: Uuid,
        line_number: i32,
        expense_category_id: Option<Uuid>,
        expense_category_name: Option<&str>,
        expense_type: &str,
        description: Option<&str>,
        expense_date: chrono::NaiveDate,
        amount: &str,
        original_currency: Option<&str>,
        original_amount: Option<&str>,
        exchange_rate: Option<&str>,
        is_reimbursable: bool,
        has_receipt: bool,
        receipt_reference: Option<&str>,
        merchant_name: Option<&str>,
        location: Option<&str>,
        attendees: Option<serde_json::Value>,
        per_diem_days: Option<f64>,
        per_diem_rate: Option<&str>,
        mileage_distance: Option<f64>,
        mileage_rate: Option<&str>,
        mileage_unit: Option<&str>,
        mileage_from: Option<&str>,
        mileage_to: Option<&str>,
        policy_violation: bool,
        policy_violation_message: Option<&str>,
        expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.expense_lines
                (organization_id, report_id, line_number,
                 expense_category_id, expense_category_name, expense_type,
                 description, expense_date,
                 amount, original_currency, original_amount, exchange_rate,
                 is_reimbursable, has_receipt, receipt_reference,
                 merchant_name, location, attendees,
                 per_diem_days, per_diem_rate,
                 mileage_distance, mileage_rate, mileage_unit,
                 mileage_from, mileage_to,
                 policy_violation, policy_violation_message,
                 expense_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::numeric, $10, $11::numeric, $12::numeric,
                    $13, $14, $15, $16, $17, $18,
                    $19, $20::numeric, $21, $22::numeric, $23,
                    $24, $25, $26, $27, $28, $29)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(report_id).bind(line_number)
        .bind(expense_category_id).bind(expense_category_name).bind(expense_type)
        .bind(description).bind(expense_date)
        .bind(amount).bind(original_currency).bind(original_amount).bind(exchange_rate)
        .bind(is_reimbursable).bind(has_receipt).bind(receipt_reference)
        .bind(merchant_name).bind(location).bind(attendees)
        .bind(per_diem_days).bind(per_diem_rate)
        .bind(mileage_distance).bind(mileage_rate).bind(mileage_unit)
        .bind(mileage_from).bind(mileage_to)
        .bind(policy_violation).bind(policy_violation_message)
        .bind(expense_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_line(&row))
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<ExpenseLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.expense_lines WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_line(&r)))
    }

    async fn list_lines_by_report(&self, report_id: Uuid) -> AtlasResult<Vec<ExpenseLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.expense_lines WHERE report_id = $1 ORDER BY line_number"
        )
        .bind(report_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_line(&r)).collect())
    }

    async fn delete_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.expense_lines WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
