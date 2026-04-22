//! Budget Repository
//!
//! PostgreSQL storage for budget definitions, versions, lines, and transfers.

use atlas_shared::{
    BudgetDefinition, BudgetVersion, BudgetLine, BudgetTransfer,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for budget data storage
#[async_trait]
pub trait BudgetRepository: Send + Sync {
    // Budget Definitions
    async fn create_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        calendar_id: Option<Uuid>,
        fiscal_year: Option<i32>,
        budget_type: &str,
        control_level: &str,
        allow_carry_forward: bool,
        allow_transfers: bool,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetDefinition>;

    async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BudgetDefinition>>;
    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<BudgetDefinition>>;
    async fn list_definitions(&self, org_id: Uuid) -> AtlasResult<Vec<BudgetDefinition>>;
    async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Budget Versions
    async fn create_version(
        &self,
        org_id: Uuid,
        definition_id: Uuid,
        version_number: i32,
        label: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetVersion>;

    async fn get_version(&self, id: Uuid) -> AtlasResult<Option<BudgetVersion>>;
    async fn get_active_version(&self, definition_id: Uuid) -> AtlasResult<Option<BudgetVersion>>;
    async fn list_versions(&self, definition_id: Uuid) -> AtlasResult<Vec<BudgetVersion>>;
    async fn get_next_version_number(&self, definition_id: Uuid) -> AtlasResult<i32>;
    async fn update_version_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<BudgetVersion>;
    async fn update_version_totals(
        &self,
        id: Uuid,
        total_budget: &str,
        total_committed: &str,
        total_actual: &str,
        total_variance: &str,
    ) -> AtlasResult<()>;

    // Budget Lines
    async fn create_line(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_name: Option<&str>,
        period_name: Option<&str>,
        period_start_date: Option<chrono::NaiveDate>,
        period_end_date: Option<chrono::NaiveDate>,
        fiscal_year: Option<i32>,
        quarter: Option<i32>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        budget_amount: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetLine>;

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<BudgetLine>>;
    async fn list_lines_by_version(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetLine>>;
    async fn find_line(
        &self,
        version_id: Uuid,
        account_code: &str,
        period_name: Option<&str>,
        department_id: Option<&Uuid>,
        cost_center: Option<&str>,
    ) -> AtlasResult<Option<BudgetLine>>;
    async fn update_line_amount(&self, id: Uuid, budget_amount: &str) -> AtlasResult<BudgetLine>;
    async fn delete_line(&self, id: Uuid) -> AtlasResult<()>;

    // Budget Transfers
    async fn create_transfer(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        transfer_number: &str,
        description: Option<&str>,
        from_account_code: &str,
        from_period_name: Option<&str>,
        from_department_id: Option<Uuid>,
        from_cost_center: Option<&str>,
        to_account_code: &str,
        to_period_name: Option<&str>,
        to_department_id: Option<Uuid>,
        to_cost_center: Option<&str>,
        amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetTransfer>;

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<BudgetTransfer>>;
    async fn list_transfers(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetTransfer>>;
    async fn update_transfer_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<BudgetTransfer>;
}

/// PostgreSQL implementation
pub struct PostgresBudgetRepository {
    pool: PgPool,
}

impl PostgresBudgetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_definition(&self, row: &sqlx::postgres::PgRow) -> BudgetDefinition {
        BudgetDefinition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            calendar_id: row.get("calendar_id"),
            fiscal_year: row.get("fiscal_year"),
            budget_type: row.get("budget_type"),
            control_level: row.get("control_level"),
            allow_carry_forward: row.get("allow_carry_forward"),
            allow_transfers: row.get("allow_transfers"),
            currency_code: row.get("currency_code"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_version(&self, row: &sqlx::postgres::PgRow) -> BudgetVersion {
        let total_budget: serde_json::Value = row.try_get("total_budget_amount").unwrap_or(serde_json::json!("0"));
        let total_committed: serde_json::Value = row.try_get("total_committed_amount").unwrap_or(serde_json::json!("0"));
        let total_actual: serde_json::Value = row.try_get("total_actual_amount").unwrap_or(serde_json::json!("0"));
        let total_variance: serde_json::Value = row.try_get("total_variance_amount").unwrap_or(serde_json::json!("0"));

        BudgetVersion {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            definition_id: row.get("definition_id"),
            version_number: row.get("version_number"),
            label: row.get("label"),
            status: row.get("status"),
            total_budget_amount: total_budget.to_string(),
            total_committed_amount: total_committed.to_string(),
            total_actual_amount: total_actual.to_string(),
            total_variance_amount: total_variance.to_string(),
            submitted_by: row.get("submitted_by"),
            submitted_at: row.get("submitted_at"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            notes: row.get("notes"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> BudgetLine {
        let budget_amount: serde_json::Value = row.try_get("budget_amount").unwrap_or(serde_json::json!("0"));
        let committed_amount: serde_json::Value = row.try_get("committed_amount").unwrap_or(serde_json::json!("0"));
        let actual_amount: serde_json::Value = row.try_get("actual_amount").unwrap_or(serde_json::json!("0"));
        let variance_amount: serde_json::Value = row.try_get("variance_amount").unwrap_or(serde_json::json!("0"));
        let variance_percent: serde_json::Value = row.try_get("variance_percent").unwrap_or(serde_json::json!("0"));
        let carry_forward: serde_json::Value = row.try_get("carry_forward_amount").unwrap_or(serde_json::json!("0"));
        let transferred_in: serde_json::Value = row.try_get("transferred_in_amount").unwrap_or(serde_json::json!("0"));
        let transferred_out: serde_json::Value = row.try_get("transferred_out_amount").unwrap_or(serde_json::json!("0"));

        BudgetLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            version_id: row.get("version_id"),
            line_number: row.get("line_number"),
            account_code: row.get("account_code"),
            account_name: row.get("account_name"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            fiscal_year: row.get("fiscal_year"),
            quarter: row.get("quarter"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            project_id: row.get("project_id"),
            project_name: row.get("project_name"),
            cost_center: row.get("cost_center"),
            budget_amount: budget_amount.to_string(),
            committed_amount: committed_amount.to_string(),
            actual_amount: actual_amount.to_string(),
            variance_amount: variance_amount.to_string(),
            variance_percent: variance_percent.to_string(),
            carry_forward_amount: carry_forward.to_string(),
            transferred_in_amount: transferred_in.to_string(),
            transferred_out_amount: transferred_out.to_string(),
            description: row.get("description"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_transfer(&self, row: &sqlx::postgres::PgRow) -> BudgetTransfer {
        let amount: serde_json::Value = row.try_get("amount").unwrap_or(serde_json::json!("0"));

        BudgetTransfer {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            version_id: row.get("version_id"),
            transfer_number: row.get("transfer_number"),
            description: row.get("description"),
            from_account_code: row.get("from_account_code"),
            from_period_name: row.get("from_period_name"),
            from_department_id: row.get("from_department_id"),
            from_cost_center: row.get("from_cost_center"),
            to_account_code: row.get("to_account_code"),
            to_period_name: row.get("to_period_name"),
            to_department_id: row.get("to_department_id"),
            to_cost_center: row.get("to_cost_center"),
            amount: amount.to_string(),
            status: row.get("status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl BudgetRepository for PostgresBudgetRepository {
    // ========================================================================
    // Budget Definitions
    // ========================================================================

    async fn create_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        calendar_id: Option<Uuid>,
        fiscal_year: Option<i32>,
        budget_type: &str,
        control_level: &str,
        allow_carry_forward: bool,
        allow_transfers: bool,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetDefinition> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.budget_definitions
                (organization_id, code, name, description,
                 calendar_id, fiscal_year, budget_type, control_level,
                 allow_carry_forward, allow_transfers, currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4,
                    calendar_id = $5, fiscal_year = $6,
                    budget_type = $7, control_level = $8,
                    allow_carry_forward = $9, allow_transfers = $10,
                    currency_code = $11, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(calendar_id).bind(fiscal_year).bind(budget_type).bind(control_level)
        .bind(allow_carry_forward).bind(allow_transfers).bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_definition(&row))
    }

    async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BudgetDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_definitions WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_definition(&r)))
    }

    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<BudgetDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_definitions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_definition(&r)))
    }

    async fn list_definitions(&self, org_id: Uuid) -> AtlasResult<Vec<BudgetDefinition>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.budget_definitions WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_definition(r)).collect())
    }

    async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.budget_definitions SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Budget Versions
    // ========================================================================

    async fn create_version(
        &self,
        org_id: Uuid,
        definition_id: Uuid,
        version_number: i32,
        label: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetVersion> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.budget_versions
                (organization_id, definition_id, version_number, label,
                 effective_from, effective_to, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(definition_id).bind(version_number).bind(label)
        .bind(effective_from).bind(effective_to).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_version(&row))
    }

    async fn get_version(&self, id: Uuid) -> AtlasResult<Option<BudgetVersion>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_versions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_version(&r)))
    }

    async fn get_active_version(&self, definition_id: Uuid) -> AtlasResult<Option<BudgetVersion>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_versions WHERE definition_id = $1 AND status = 'active' ORDER BY version_number DESC LIMIT 1"
        )
        .bind(definition_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_version(&r)))
    }

    async fn list_versions(&self, definition_id: Uuid) -> AtlasResult<Vec<BudgetVersion>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.budget_versions WHERE definition_id = $1 ORDER BY version_number DESC"
        )
        .bind(definition_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_version(r)).collect())
    }

    async fn get_next_version_number(&self, definition_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(version_number), 0) + 1 as next_ver FROM _atlas.budget_versions WHERE definition_id = $1"
        )
        .bind(definition_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let next: i32 = row.get("next_ver");
        Ok(next)
    }

    async fn update_version_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<BudgetVersion> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.budget_versions
            SET status = $2, submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $3 IS NOT NULL AND submitted_at IS NULL THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $4 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                rejected_reason = $5, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(submitted_by).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_version(&row))
    }

    async fn update_version_totals(
        &self,
        id: Uuid,
        total_budget: &str,
        total_committed: &str,
        total_actual: &str,
        total_variance: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.budget_versions
            SET total_budget_amount = $2::numeric, total_committed_amount = $3::numeric,
                total_actual_amount = $4::numeric, total_variance_amount = $5::numeric,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_budget).bind(total_committed).bind(total_actual).bind(total_variance)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Budget Lines
    // ========================================================================

    async fn create_line(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_name: Option<&str>,
        period_name: Option<&str>,
        period_start_date: Option<chrono::NaiveDate>,
        period_end_date: Option<chrono::NaiveDate>,
        fiscal_year: Option<i32>,
        quarter: Option<i32>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        budget_amount: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.budget_lines
                (organization_id, version_id, line_number,
                 account_code, account_name,
                 period_name, period_start_date, period_end_date,
                 fiscal_year, quarter,
                 department_id, department_name,
                 project_id, project_name,
                 cost_center, budget_amount, description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16::numeric, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(version_id).bind(line_number)
        .bind(account_code).bind(account_name)
        .bind(period_name).bind(period_start_date).bind(period_end_date)
        .bind(fiscal_year).bind(quarter)
        .bind(department_id).bind(department_name)
        .bind(project_id).bind(project_name)
        .bind(cost_center).bind(budget_amount).bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_line(&row))
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<BudgetLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_lines WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_line(&r)))
    }

    async fn list_lines_by_version(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.budget_lines WHERE version_id = $1 ORDER BY line_number"
        )
        .bind(version_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_line(r)).collect())
    }

    async fn find_line(
        &self,
        version_id: Uuid,
        account_code: &str,
        period_name: Option<&str>,
        department_id: Option<&Uuid>,
        cost_center: Option<&str>,
    ) -> AtlasResult<Option<BudgetLine>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.budget_lines
            WHERE version_id = $1 AND account_code = $2
              AND ($3::text IS NULL AND period_name IS NULL OR period_name = $3)
              AND ($4::uuid IS NULL AND department_id IS NULL OR department_id = $4)
              AND ($5::text IS NULL AND cost_center IS NULL OR cost_center = $5)
            LIMIT 1
            "#,
        )
        .bind(version_id).bind(account_code).bind(period_name).bind(department_id).bind(cost_center)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_line(&r)))
    }

    async fn update_line_amount(&self, id: Uuid, budget_amount: &str) -> AtlasResult<BudgetLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.budget_lines
            SET budget_amount = $2::numeric,
                variance_amount = budget_amount - actual_amount,
                variance_percent = CASE WHEN budget_amount != 0
                    THEN ((budget_amount - actual_amount) / budget_amount) * 100
                    ELSE 0 END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(budget_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_line(&row))
    }

    async fn delete_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.budget_lines WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Budget Transfers
    // ========================================================================

    async fn create_transfer(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        transfer_number: &str,
        description: Option<&str>,
        from_account_code: &str,
        from_period_name: Option<&str>,
        from_department_id: Option<Uuid>,
        from_cost_center: Option<&str>,
        to_account_code: &str,
        to_period_name: Option<&str>,
        to_department_id: Option<Uuid>,
        to_cost_center: Option<&str>,
        amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetTransfer> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.budget_transfers
                (organization_id, version_id, transfer_number, description,
                 from_account_code, from_period_name, from_department_id, from_cost_center,
                 to_account_code, to_period_name, to_department_id, to_cost_center,
                 amount, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::numeric, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(version_id).bind(transfer_number).bind(description)
        .bind(from_account_code).bind(from_period_name).bind(from_department_id).bind(from_cost_center)
        .bind(to_account_code).bind(to_period_name).bind(to_department_id).bind(to_cost_center)
        .bind(amount).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_transfer(&row))
    }

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<BudgetTransfer>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.budget_transfers WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transfer(&r)))
    }

    async fn list_transfers(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetTransfer>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.budget_transfers WHERE version_id = $1 ORDER BY created_at DESC"
        )
        .bind(version_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transfer(r)).collect())
    }

    async fn update_transfer_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<BudgetTransfer> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.budget_transfers
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                rejected_reason = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transfer(&row))
    }
}
