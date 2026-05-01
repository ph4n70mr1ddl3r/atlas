//! Financial Statement Repository
//!
//! Storage interface for financial statement data.

use atlas_shared::{
    FinancialStatementDefinition, FinancialStatement,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Account balance for financial statement generation
#[derive(Debug, Clone)]
pub struct AccountBalance {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub subtype: Option<String>,
    pub balance: f64,
}

/// Repository trait for financial statement data storage
#[async_trait]
pub trait FinancialStatementRepository: Send + Sync {
    // Report definitions
    async fn create_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        report_type: &str,
        currency_code: &str,
        include_comparative: bool,
        comparative_period_count: i32,
        row_definitions: serde_json::Value,
        column_definitions: serde_json::Value,
        period_name: Option<&str>,
        fiscal_year: Option<i32>,
        is_system: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatementDefinition>;

    async fn get_definition(&self, id: Uuid) -> AtlasResult<Option<FinancialStatementDefinition>>;
    async fn get_definition_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialStatementDefinition>>;
    async fn list_definitions(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatementDefinition>>;

    // Account balances (from GL)
    async fn get_account_balances(
        &self,
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
        period_name: Option<&str>,
    ) -> AtlasResult<Vec<AccountBalance>>;

    // Generated statements
    async fn save_statement(&self, statement: &FinancialStatement) -> AtlasResult<()>;
    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<FinancialStatement>>;
    async fn list_statements(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatement>>;
}

/// PostgreSQL implementation
pub struct PostgresFinancialStatementRepository {
    pool: PgPool,
}

impl PostgresFinancialStatementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_definition(row: &sqlx::postgres::PgRow) -> FinancialStatementDefinition {
    FinancialStatementDefinition {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        report_type: row.get("report_type"),
        currency_code: row.get("currency_code"),
        include_comparative: row.get("include_comparative"),
        comparative_period_count: row.try_get("comparative_period_count").unwrap_or(0),
        row_definitions: row.try_get("row_definitions").unwrap_or(serde_json::json!([])),
        column_definitions: row.try_get("column_definitions").unwrap_or(serde_json::json!([])),
        period_name: row.get("period_name"),
        fiscal_year: row.get("fiscal_year"),
        is_system: row.get("is_system"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl FinancialStatementRepository for PostgresFinancialStatementRepository {
    async fn create_definition(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        report_type: &str, currency_code: &str,
        include_comparative: bool, comparative_period_count: i32,
        row_definitions: serde_json::Value, column_definitions: serde_json::Value,
        period_name: Option<&str>, fiscal_year: Option<i32>,
        is_system: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatementDefinition> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_definitions
                (organization_id, code, name, description, report_type,
                 currency_code, include_comparative, comparative_period_count,
                 row_definitions, column_definitions,
                 period_name, fiscal_year, is_system, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(report_type)
        .bind(currency_code).bind(include_comparative).bind(comparative_period_count)
        .bind(&row_definitions).bind(&column_definitions)
        .bind(period_name).bind(fiscal_year).bind(is_system).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_definition(&row))
    }

    async fn get_definition(&self, id: Uuid) -> AtlasResult<Option<FinancialStatementDefinition>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_report_definitions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_definition(&r)))
    }

    async fn get_definition_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialStatementDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.financial_report_definitions WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_definition(&r)))
    }

    async fn list_definitions(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatementDefinition>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.financial_report_definitions
            WHERE organization_id = $1
              AND ($2::text IS NULL OR report_type = $2)
              AND is_active = true
            ORDER BY report_type, code
            "#,
        )
        .bind(org_id).bind(report_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_definition).collect())
    }

    async fn get_account_balances(
        &self,
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
        _period_name: Option<&str>,
    ) -> AtlasResult<Vec<AccountBalance>> {
        let rows = sqlx::query(
            r#"
            SELECT
                a.account_code,
                a.account_name,
                a.account_type,
                a.subtype,
                COALESCE(SUM(jl.accounted_dr), 0) - COALESCE(SUM(jl.accounted_cr), 0) as balance
            FROM _atlas.gl_accounts a
            LEFT JOIN _atlas.gl_journal_lines jl ON jl.account_code = a.account_code
            LEFT JOIN _atlas.gl_journal_entries je ON jl.journal_entry_id = je.id AND je.status = 'posted' AND je.gl_date <= $2
            WHERE a.organization_id = $1 AND a.is_active = true
            GROUP BY a.account_code, a.account_name, a.account_type, a.subtype
            HAVING COALESCE(SUM(jl.accounted_dr), 0) - COALESCE(SUM(jl.accounted_cr), 0) <> 0
            ORDER BY a.account_code
            "#,
        )
        .bind(org_id).bind(as_of_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| AccountBalance {
            account_code: r.get("account_code"),
            account_name: r.get("account_name"),
            account_type: r.get("account_type"),
            subtype: r.get("subtype"),
            balance: r.try_get("balance").unwrap_or(0.0),
        }).collect())
    }

    async fn save_statement(&self, stmt: &FinancialStatement) -> AtlasResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _atlas.financial_statements
                (id, organization_id, definition_id, report_name, report_type,
                 as_of_date, period_name, fiscal_year, currency_code,
                 lines, totals, is_balanced, generated_at, generated_by, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(stmt.id).bind(stmt.organization_id).bind(stmt.definition_id)
        .bind(&stmt.report_name).bind(&stmt.report_type)
        .bind(stmt.as_of_date).bind(&stmt.period_name).bind(stmt.fiscal_year)
        .bind(&stmt.currency_code)
        .bind(serde_json::to_value(&stmt.lines).unwrap_or(serde_json::json!([])))
        .bind(&stmt.totals)
        .bind(stmt.is_balanced)
        .bind(stmt.generated_at).bind(stmt.generated_by)
        .bind(&stmt.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<FinancialStatement>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_statements WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| FinancialStatement {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            definition_id: r.get("definition_id"),
            report_name: r.get("report_name"),
            report_type: r.get("report_type"),
            as_of_date: r.get("as_of_date"),
            period_name: r.get("period_name"),
            fiscal_year: r.get("fiscal_year"),
            currency_code: r.get("currency_code"),
            lines: serde_json::from_value(r.try_get("lines").unwrap_or(serde_json::json!([]))).unwrap_or_default(),
            totals: r.try_get("totals").unwrap_or(serde_json::json!({})),
            is_balanced: r.get("is_balanced"),
            generated_at: r.get("generated_at"),
            generated_by: r.get("generated_by"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: r.get("created_at"),
        }))
    }

    async fn list_statements(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatement>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.financial_statements
            WHERE organization_id = $1
              AND ($2::text IS NULL OR report_type = $2)
            ORDER BY generated_at DESC
            "#,
        )
        .bind(org_id).bind(report_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| FinancialStatement {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            definition_id: r.get("definition_id"),
            report_name: r.get("report_name"),
            report_type: r.get("report_type"),
            as_of_date: r.get("as_of_date"),
            period_name: r.get("period_name"),
            fiscal_year: r.get("fiscal_year"),
            currency_code: r.get("currency_code"),
            lines: serde_json::from_value(r.try_get("lines").unwrap_or(serde_json::json!([]))).unwrap_or_default(),
            totals: r.try_get("totals").unwrap_or(serde_json::json!({})),
            is_balanced: r.get("is_balanced"),
            generated_at: r.get("generated_at"),
            generated_by: r.get("generated_by"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: r.get("created_at"),
        }).collect())
    }
}
