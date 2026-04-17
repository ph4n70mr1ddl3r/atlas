//! Encumbrance Repository
//!
//! PostgreSQL storage for encumbrance types, entries, lines,
//! liquidations, and carry-forward processing.

use atlas_shared::{
    EncumbranceType, EncumbranceEntry, EncumbranceLine,
    EncumbranceLiquidation, EncumbranceCarryForward,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for encumbrance management data storage
#[async_trait]
pub trait EncumbranceRepository: Send + Sync {
    // ========================================================================
    // Encumbrance Types
    // ========================================================================

    async fn create_encumbrance_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        allow_manual_entry: bool,
        default_encumbrance_account_code: Option<&str>,
        allow_carry_forward: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceType>;

    async fn get_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<EncumbranceType>>;
    async fn get_encumbrance_type_by_id(&self, id: Uuid) -> AtlasResult<Option<EncumbranceType>>;
    async fn list_encumbrance_types(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceType>>;
    async fn delete_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Encumbrance Entries
    // ========================================================================

    async fn create_entry(
        &self,
        org_id: Uuid,
        entry_number: &str,
        encumbrance_type_id: Uuid,
        encumbrance_type_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        encumbrance_date: chrono::NaiveDate,
        original_amount: &str,
        current_amount: &str,
        currency_code: &str,
        status: &str,
        fiscal_year: Option<i32>,
        period_name: Option<&str>,
        expiry_date: Option<chrono::NaiveDate>,
        budget_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceEntry>;

    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<EncumbranceEntry>>;
    async fn get_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<EncumbranceEntry>>;
    async fn list_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        encumbrance_type_code: Option<&str>,
        source_type: Option<&str>,
        fiscal_year: Option<i32>,
    ) -> AtlasResult<Vec<EncumbranceEntry>>;
    async fn update_entry_amounts(
        &self,
        id: Uuid,
        current_amount: &str,
        liquidated_amount: &str,
        adjusted_amount: &str,
        status: &str,
    ) -> AtlasResult<EncumbranceEntry>;
    async fn update_entry_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        cancelled_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<EncumbranceEntry>;

    // ========================================================================
    // Encumbrance Lines
    // ========================================================================

    async fn create_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_description: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        original_amount: &str,
        current_amount: &str,
        encumbrance_account_code: Option<&str>,
        source_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLine>;

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<EncumbranceLine>>;
    async fn list_lines_by_entry(&self, entry_id: Uuid) -> AtlasResult<Vec<EncumbranceLine>>;
    async fn update_line_amounts(
        &self,
        id: Uuid,
        current_amount: &str,
        liquidated_amount: &str,
    ) -> AtlasResult<EncumbranceLine>;
    async fn delete_line(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Liquidations
    // ========================================================================

    async fn create_liquidation(
        &self,
        org_id: Uuid,
        liquidation_number: &str,
        encumbrance_entry_id: Uuid,
        encumbrance_line_id: Option<Uuid>,
        liquidation_type: &str,
        liquidation_amount: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        liquidation_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLiquidation>;

    async fn get_liquidation(&self, id: Uuid) -> AtlasResult<Option<EncumbranceLiquidation>>;
    async fn list_liquidations(
        &self,
        org_id: Uuid,
        entry_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<EncumbranceLiquidation>>;
    async fn update_liquidation_status(
        &self,
        id: Uuid,
        status: &str,
        reversed_by_id: Option<Uuid>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<EncumbranceLiquidation>;

    // ========================================================================
    // Carry-Forward
    // ========================================================================

    async fn create_carry_forward(
        &self,
        org_id: Uuid,
        batch_number: &str,
        from_fiscal_year: i32,
        to_fiscal_year: i32,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceCarryForward>;

    async fn get_carry_forward(&self, id: Uuid) -> AtlasResult<Option<EncumbranceCarryForward>>;
    async fn list_carry_forwards(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceCarryForward>>;
    async fn update_carry_forward_status(
        &self,
        id: Uuid,
        status: &str,
        entry_count: i32,
        total_amount: &str,
        processed_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceCarryForward>;
}

/// PostgreSQL implementation
pub struct PostgresEncumbranceRepository {
    pool: PgPool,
}

impl PostgresEncumbranceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_encumbrance_type(row: &sqlx::postgres::PgRow) -> EncumbranceType {
    EncumbranceType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        category: row.get("category"),
        is_enabled: row.get("is_enabled"),
        allow_manual_entry: row.get("allow_manual_entry"),
        default_encumbrance_account_code: row.get("default_encumbrance_account_code"),
        allow_carry_forward: row.get("allow_carry_forward"),
        priority: row.get("priority"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_entry(row: &sqlx::postgres::PgRow) -> EncumbranceEntry {
    EncumbranceEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        entry_number: row.get("entry_number"),
        encumbrance_type_id: row.get("encumbrance_type_id"),
        encumbrance_type_code: row.get("encumbrance_type_code"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        description: row.get("description"),
        encumbrance_date: row.get("encumbrance_date"),
        original_amount: get_num(row, "original_amount"),
        current_amount: get_num(row, "current_amount"),
        liquidated_amount: get_num(row, "liquidated_amount"),
        adjusted_amount: get_num(row, "adjusted_amount"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        fiscal_year: row.get("fiscal_year"),
        period_name: row.get("period_name"),
        is_carry_forward: row.get("is_carry_forward"),
        carried_forward_from_id: row.get("carried_forward_from_id"),
        expiry_date: row.get("expiry_date"),
        budget_line_id: row.get("budget_line_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        approved_by: row.get("approved_by"),
        cancelled_by: row.get("cancelled_by"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_line(row: &sqlx::postgres::PgRow) -> EncumbranceLine {
    EncumbranceLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        entry_id: row.get("entry_id"),
        line_number: row.get("line_number"),
        account_code: row.get("account_code"),
        account_description: row.get("account_description"),
        department_id: row.get("department_id"),
        department_name: row.get("department_name"),
        project_id: row.get("project_id"),
        project_name: row.get("project_name"),
        cost_center: row.get("cost_center"),
        original_amount: get_num(row, "original_amount"),
        current_amount: get_num(row, "current_amount"),
        liquidated_amount: get_num(row, "liquidated_amount"),
        encumbrance_account_code: row.get("encumbrance_account_code"),
        source_line_id: row.get("source_line_id"),
        attribute_category: row.get("attribute_category"),
        attribute1: row.get("attribute1"),
        attribute2: row.get("attribute2"),
        attribute3: row.get("attribute3"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_liquidation(row: &sqlx::postgres::PgRow) -> EncumbranceLiquidation {
    EncumbranceLiquidation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        liquidation_number: row.get("liquidation_number"),
        encumbrance_entry_id: row.get("encumbrance_entry_id"),
        encumbrance_line_id: row.get("encumbrance_line_id"),
        liquidation_type: row.get("liquidation_type"),
        liquidation_amount: get_num(row, "liquidation_amount"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        description: row.get("description"),
        liquidation_date: row.get("liquidation_date"),
        status: row.get("status"),
        reversed_by_id: row.get("reversed_by_id"),
        reversal_reason: row.get("reversal_reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_carry_forward(row: &sqlx::postgres::PgRow) -> EncumbranceCarryForward {
    EncumbranceCarryForward {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        from_fiscal_year: row.get("from_fiscal_year"),
        to_fiscal_year: row.get("to_fiscal_year"),
        status: row.get("status"),
        entry_count: row.get("entry_count"),
        total_amount: get_num(row, "total_amount"),
        description: row.get("description"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        processed_by: row.get("processed_by"),
        processed_at: row.get("processed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl EncumbranceRepository for PostgresEncumbranceRepository {
    // ========================================================================
    // Encumbrance Types
    // ========================================================================

    async fn create_encumbrance_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        allow_manual_entry: bool,
        default_encumbrance_account_code: Option<&str>,
        allow_carry_forward: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceType> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.encumbrance_types
                (organization_id, code, name, description, category,
                 allow_manual_entry, default_encumbrance_account_code,
                 allow_carry_forward, priority, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, category = $5,
                    allow_manual_entry = $6,
                    default_encumbrance_account_code = $7,
                    allow_carry_forward = $8, priority = $9,
                    is_enabled = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(category)
        .bind(allow_manual_entry).bind(default_encumbrance_account_code)
        .bind(allow_carry_forward).bind(priority).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_encumbrance_type(&row))
    }

    async fn get_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<EncumbranceType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.encumbrance_types WHERE organization_id = $1 AND code = $2 AND is_enabled = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_encumbrance_type(&r)))
    }

    async fn get_encumbrance_type_by_id(&self, id: Uuid) -> AtlasResult<Option<EncumbranceType>> {
        let row = sqlx::query("SELECT * FROM _atlas.encumbrance_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_encumbrance_type(&r)))
    }

    async fn list_encumbrance_types(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceType>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.encumbrance_types WHERE organization_id = $1 AND is_enabled = true ORDER BY priority, code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_encumbrance_type(&r)).collect())
    }

    async fn delete_encumbrance_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.encumbrance_types SET is_enabled = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Encumbrance Entries
    // ========================================================================

    async fn create_entry(
        &self,
        org_id: Uuid,
        entry_number: &str,
        encumbrance_type_id: Uuid,
        encumbrance_type_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        encumbrance_date: chrono::NaiveDate,
        original_amount: &str,
        current_amount: &str,
        currency_code: &str,
        status: &str,
        fiscal_year: Option<i32>,
        period_name: Option<&str>,
        expiry_date: Option<chrono::NaiveDate>,
        budget_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.encumbrance_entries
                (organization_id, entry_number, encumbrance_type_id, encumbrance_type_code,
                 source_type, source_id, source_number, description,
                 encumbrance_date, original_amount, current_amount,
                 currency_code, status, fiscal_year, period_name,
                 expiry_date, budget_line_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::numeric, $11::numeric, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entry_number).bind(encumbrance_type_id).bind(encumbrance_type_code)
        .bind(source_type).bind(source_id).bind(source_number).bind(description)
        .bind(encumbrance_date).bind(original_amount).bind(current_amount)
        .bind(currency_code).bind(status).bind(fiscal_year).bind(period_name)
        .bind(expiry_date).bind(budget_line_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_entry(&row))
    }

    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<EncumbranceEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.encumbrance_entries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entry(&r)))
    }

    async fn get_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<EncumbranceEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.encumbrance_entries WHERE organization_id = $1 AND entry_number = $2"
        )
        .bind(org_id).bind(entry_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entry(&r)))
    }

    async fn list_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        encumbrance_type_code: Option<&str>,
        source_type: Option<&str>,
        fiscal_year: Option<i32>,
    ) -> AtlasResult<Vec<EncumbranceEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.encumbrance_entries
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR encumbrance_type_code = $3)
              AND ($4::text IS NULL OR source_type = $4)
              AND ($5::int IS NULL OR fiscal_year = $5)
            ORDER BY encumbrance_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(encumbrance_type_code).bind(source_type).bind(fiscal_year)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_entry(&r)).collect())
    }

    async fn update_entry_amounts(
        &self,
        id: Uuid,
        current_amount: &str,
        liquidated_amount: &str,
        adjusted_amount: &str,
        status: &str,
    ) -> AtlasResult<EncumbranceEntry> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.encumbrance_entries
            SET current_amount = $2::numeric, liquidated_amount = $3::numeric,
                adjusted_amount = $4::numeric, status = $5, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(current_amount).bind(liquidated_amount).bind(adjusted_amount).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entry(&row))
    }

    async fn update_entry_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        cancelled_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<EncumbranceEntry> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.encumbrance_entries
            SET status = $2,
                approved_by = CASE WHEN $2 = 'active' THEN $3 ELSE approved_by END,
                cancelled_by = CASE WHEN $2 = 'cancelled' THEN $4 ELSE cancelled_by END,
                cancelled_at = CASE WHEN $2 = 'cancelled' THEN now() ELSE cancelled_at END,
                cancellation_reason = CASE WHEN $2 = 'cancelled' THEN $5 ELSE cancellation_reason END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(cancelled_by).bind(cancellation_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entry(&row))
    }

    // ========================================================================
    // Encumbrance Lines
    // ========================================================================

    async fn create_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_description: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        original_amount: &str,
        current_amount: &str,
        encumbrance_account_code: Option<&str>,
        source_line_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.encumbrance_lines
                (organization_id, entry_id, line_number, account_code, account_description,
                 department_id, department_name, project_id, project_name, cost_center,
                 original_amount, current_amount, encumbrance_account_code,
                 source_line_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entry_id).bind(line_number).bind(account_code).bind(account_description)
        .bind(department_id).bind(department_name).bind(project_id).bind(project_name).bind(cost_center)
        .bind(original_amount).bind(current_amount).bind(encumbrance_account_code)
        .bind(source_line_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_line(&row))
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<EncumbranceLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.encumbrance_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_line(&r)))
    }

    async fn list_lines_by_entry(&self, entry_id: Uuid) -> AtlasResult<Vec<EncumbranceLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.encumbrance_lines WHERE entry_id = $1 ORDER BY line_number"
        )
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_line(&r)).collect())
    }

    async fn update_line_amounts(
        &self,
        id: Uuid,
        current_amount: &str,
        liquidated_amount: &str,
    ) -> AtlasResult<EncumbranceLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.encumbrance_lines
            SET current_amount = $2::numeric, liquidated_amount = $3::numeric,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(current_amount).bind(liquidated_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_line(&row))
    }

    async fn delete_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.encumbrance_lines WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Liquidations
    // ========================================================================

    async fn create_liquidation(
        &self,
        org_id: Uuid,
        liquidation_number: &str,
        encumbrance_entry_id: Uuid,
        encumbrance_line_id: Option<Uuid>,
        liquidation_type: &str,
        liquidation_amount: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        description: Option<&str>,
        liquidation_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceLiquidation> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.encumbrance_liquidations
                (organization_id, liquidation_number, encumbrance_entry_id,
                 encumbrance_line_id, liquidation_type, liquidation_amount,
                 source_type, source_id, source_number, description,
                 liquidation_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(liquidation_number).bind(encumbrance_entry_id)
        .bind(encumbrance_line_id).bind(liquidation_type).bind(liquidation_amount)
        .bind(source_type).bind(source_id).bind(source_number).bind(description)
        .bind(liquidation_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_liquidation(&row))
    }

    async fn get_liquidation(&self, id: Uuid) -> AtlasResult<Option<EncumbranceLiquidation>> {
        let row = sqlx::query("SELECT * FROM _atlas.encumbrance_liquidations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_liquidation(&r)))
    }

    async fn list_liquidations(
        &self,
        org_id: Uuid,
        entry_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<EncumbranceLiquidation>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.encumbrance_liquidations
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR encumbrance_entry_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY liquidation_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(entry_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_liquidation(&r)).collect())
    }

    async fn update_liquidation_status(
        &self,
        id: Uuid,
        status: &str,
        reversed_by_id: Option<Uuid>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<EncumbranceLiquidation> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.encumbrance_liquidations
            SET status = $2, reversed_by_id = $3, reversal_reason = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(reversed_by_id).bind(reversal_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_liquidation(&row))
    }

    // ========================================================================
    // Carry-Forward
    // ========================================================================

    async fn create_carry_forward(
        &self,
        org_id: Uuid,
        batch_number: &str,
        from_fiscal_year: i32,
        to_fiscal_year: i32,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceCarryForward> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.encumbrance_carry_forwards
                (organization_id, batch_number, from_fiscal_year, to_fiscal_year,
                 description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_number).bind(from_fiscal_year).bind(to_fiscal_year)
        .bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_carry_forward(&row))
    }

    async fn get_carry_forward(&self, id: Uuid) -> AtlasResult<Option<EncumbranceCarryForward>> {
        let row = sqlx::query("SELECT * FROM _atlas.encumbrance_carry_forwards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_carry_forward(&r)))
    }

    async fn list_carry_forwards(&self, org_id: Uuid) -> AtlasResult<Vec<EncumbranceCarryForward>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.encumbrance_carry_forwards WHERE organization_id = $1 ORDER BY created_at DESC"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_carry_forward(&r)).collect())
    }

    async fn update_carry_forward_status(
        &self,
        id: Uuid,
        status: &str,
        entry_count: i32,
        total_amount: &str,
        processed_by: Option<Uuid>,
    ) -> AtlasResult<EncumbranceCarryForward> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.encumbrance_carry_forwards
            SET status = $2, entry_count = $3, total_amount = $4::numeric,
                processed_by = $5,
                processed_at = CASE WHEN $2 = 'completed' THEN now() ELSE processed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(entry_count).bind(total_amount).bind(processed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_carry_forward(&row))
    }
}
