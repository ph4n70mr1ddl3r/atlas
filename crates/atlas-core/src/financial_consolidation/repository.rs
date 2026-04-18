//! Financial Consolidation Repository
//!
//! PostgreSQL storage for consolidation ledgers, entities, scenarios,
//! trial balance lines, elimination rules, adjustments, and translation rates.

use atlas_shared::{
    ConsolidationLedger, ConsolidationEntity, ConsolidationScenario,
    ConsolidationTrialBalanceLine, ConsolidationEliminationRule,
    ConsolidationAdjustment, ConsolidationTranslationRate,
    ConsolidationDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for financial consolidation data storage
#[async_trait]
pub trait FinancialConsolidationRepository: Send + Sync {
    // ── Consolidation Ledgers ───────────────────────────────────────
    async fn create_ledger(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        base_currency_code: &str, translation_method: &str,
        equity_elimination_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationLedger>;

    async fn get_ledger(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ConsolidationLedger>>;
    async fn get_ledger_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationLedger>>;
    async fn list_ledgers(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationLedger>>;

    // ── Consolidation Entities ──────────────────────────────────────
    async fn create_entity(
        &self, org_id: Uuid, ledger_id: Uuid, entity_id: Uuid,
        entity_name: &str, entity_code: &str, local_currency_code: &str,
        ownership_percentage: &str, consolidation_method: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEntity>;

    async fn get_entity(&self, ledger_id: Uuid, entity_code: &str) -> AtlasResult<Option<ConsolidationEntity>>;
    async fn get_entity_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationEntity>>;
    async fn list_entities(&self, ledger_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationEntity>>;

    // ── Consolidation Scenarios ─────────────────────────────────────
    async fn create_scenario(
        &self, org_id: Uuid, ledger_id: Uuid, scenario_number: &str,
        name: &str, description: Option<&str>,
        fiscal_year: i32, period_name: &str,
        period_start_date: chrono::NaiveDate, period_end_date: chrono::NaiveDate,
        translation_rate_type: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationScenario>;

    async fn get_scenario(&self, org_id: Uuid, scenario_number: &str) -> AtlasResult<Option<ConsolidationScenario>>;
    async fn get_scenario_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationScenario>>;
    async fn list_scenarios(
        &self, org_id: Uuid, ledger_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationScenario>>;
    async fn update_scenario_status(
        &self, id: Uuid, status: &str,
        approved_by: Option<Uuid>, posted_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationScenario>;
    async fn update_scenario_totals(
        &self, id: Uuid, total_entities: i32, total_eliminations: i32,
        total_adjustments: i32, total_debits: &str, total_credits: &str,
        is_balanced: bool,
    ) -> AtlasResult<()>;

    // ── Trial Balance Lines ─────────────────────────────────────────
    async fn create_trial_balance_line(
        &self, org_id: Uuid, scenario_id: Uuid,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        account_code: &str, account_name: Option<&str>,
        account_type: Option<&str>, financial_statement: Option<&str>,
        local_debit: &str, local_credit: &str, local_balance: &str,
        exchange_rate: Option<&str>,
        translated_debit: &str, translated_credit: &str, translated_balance: &str,
        elimination_debit: &str, elimination_credit: &str, elimination_balance: &str,
        minority_interest_debit: &str, minority_interest_credit: &str, minority_interest_balance: &str,
        consolidated_debit: &str, consolidated_credit: &str, consolidated_balance: &str,
        is_elimination_entry: bool, line_type: &str,
    ) -> AtlasResult<ConsolidationTrialBalanceLine>;

    async fn list_trial_balance(
        &self, scenario_id: Uuid, entity_id: Option<Uuid>,
        line_type: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationTrialBalanceLine>>;
    async fn delete_trial_balance_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()>;

    // ── Elimination Rules ───────────────────────────────────────────
    async fn create_elimination_rule(
        &self, org_id: Uuid, ledger_id: Uuid, rule_code: &str,
        name: &str, description: Option<&str>, elimination_type: &str,
        from_entity_id: Option<Uuid>, to_entity_id: Option<Uuid>,
        from_account_pattern: Option<&str>, to_account_pattern: Option<&str>,
        offset_account_code: &str, priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEliminationRule>;

    async fn get_elimination_rule(&self, ledger_id: Uuid, rule_code: &str) -> AtlasResult<Option<ConsolidationEliminationRule>>;
    async fn list_elimination_rules(&self, ledger_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationEliminationRule>>;

    // ── Adjustments ─────────────────────────────────────────────────
    async fn create_adjustment(
        &self, org_id: Uuid, scenario_id: Uuid, adjustment_number: &str,
        description: Option<&str>, account_code: &str, account_name: Option<&str>,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        debit: &str, credit: &str, adjustment_type: &str,
        reference: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationAdjustment>;

    async fn get_adjustment(&self, id: Uuid) -> AtlasResult<Option<ConsolidationAdjustment>>;
    async fn list_adjustments(&self, scenario_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ConsolidationAdjustment>>;
    async fn update_adjustment_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationAdjustment>;

    // ── Translation Rates ───────────────────────────────────────────
    async fn create_translation_rate(
        &self, org_id: Uuid, scenario_id: Uuid, entity_id: Uuid,
        from_currency: &str, to_currency: &str,
        rate_type: &str, exchange_rate: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<ConsolidationTranslationRate>;

    async fn get_translation_rate(
        &self, scenario_id: Uuid, entity_id: Uuid, rate_type: &str,
    ) -> AtlasResult<Option<ConsolidationTranslationRate>>;
    async fn list_translation_rates(&self, scenario_id: Uuid) -> AtlasResult<Vec<ConsolidationTranslationRate>>;

    // ── Dashboard ───────────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ConsolidationDashboardSummary>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresFinancialConsolidationRepository {
    pool: PgPool,
}

impl PostgresFinancialConsolidationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_ledger(row: &sqlx::postgres::PgRow) -> ConsolidationLedger {
    ConsolidationLedger {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        base_currency_code: row.get("base_currency_code"),
        translation_method: row.get("translation_method"),
        equity_elimination_method: row.get("equity_elimination_method"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_entity(row: &sqlx::postgres::PgRow) -> ConsolidationEntity {
    ConsolidationEntity {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        ledger_id: row.get("ledger_id"),
        entity_id: row.get("entity_id"),
        entity_name: row.get("entity_name"),
        entity_code: row.get("entity_code"),
        local_currency_code: row.get("local_currency_code"),
        ownership_percentage: get_numeric(row, "ownership_percentage"),
        consolidation_method: row.get("consolidation_method"),
        is_active: row.get("is_active"),
        include_in_consolidation: row.get("include_in_consolidation"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_scenario(row: &sqlx::postgres::PgRow) -> ConsolidationScenario {
    ConsolidationScenario {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        ledger_id: row.get("ledger_id"),
        scenario_number: row.get("scenario_number"),
        name: row.get("name"),
        description: row.get("description"),
        fiscal_year: row.get("fiscal_year"),
        period_name: row.get("period_name"),
        period_start_date: row.get("period_start_date"),
        period_end_date: row.get("period_end_date"),
        status: row.get("status"),
        translation_date: row.get("translation_date"),
        translation_rate_type: row.get("translation_rate_type"),
        total_entities: row.get("total_entities"),
        total_eliminations: row.get("total_eliminations"),
        total_adjustments: row.get("total_adjustments"),
        total_debits: get_numeric(row, "total_debits"),
        total_credits: get_numeric(row, "total_credits"),
        is_balanced: row.get("is_balanced"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        posted_by: row.get("posted_by"),
        posted_at: row.get("posted_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_tb_line(row: &sqlx::postgres::PgRow) -> ConsolidationTrialBalanceLine {
    ConsolidationTrialBalanceLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        entity_id: row.get("entity_id"),
        entity_code: row.get("entity_code"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        account_type: row.get("account_type"),
        financial_statement: row.get("financial_statement"),
        local_debit: get_numeric(row, "local_debit"),
        local_credit: get_numeric(row, "local_credit"),
        local_balance: get_numeric(row, "local_balance"),
        exchange_rate: row.try_get("exchange_rate").unwrap_or(None),
        translated_debit: get_numeric(row, "translated_debit"),
        translated_credit: get_numeric(row, "translated_credit"),
        translated_balance: get_numeric(row, "translated_balance"),
        elimination_debit: get_numeric(row, "elimination_debit"),
        elimination_credit: get_numeric(row, "elimination_credit"),
        elimination_balance: get_numeric(row, "elimination_balance"),
        minority_interest_debit: get_numeric(row, "minority_interest_debit"),
        minority_interest_credit: get_numeric(row, "minority_interest_credit"),
        minority_interest_balance: get_numeric(row, "minority_interest_balance"),
        consolidated_debit: get_numeric(row, "consolidated_debit"),
        consolidated_credit: get_numeric(row, "consolidated_credit"),
        consolidated_balance: get_numeric(row, "consolidated_balance"),
        is_elimination_entry: row.get("is_elimination_entry"),
        line_type: row.get("line_type"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_elimination_rule(row: &sqlx::postgres::PgRow) -> ConsolidationEliminationRule {
    ConsolidationEliminationRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        ledger_id: row.get("ledger_id"),
        rule_code: row.get("rule_code"),
        name: row.get("name"),
        description: row.get("description"),
        elimination_type: row.get("elimination_type"),
        from_entity_id: row.get("from_entity_id"),
        to_entity_id: row.get("to_entity_id"),
        from_account_pattern: row.get("from_account_pattern"),
        to_account_pattern: row.get("to_account_pattern"),
        offset_account_code: row.get("offset_account_code"),
        priority: row.get("priority"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_adjustment(row: &sqlx::postgres::PgRow) -> ConsolidationAdjustment {
    ConsolidationAdjustment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        adjustment_number: row.get("adjustment_number"),
        description: row.get("description"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        entity_id: row.get("entity_id"),
        entity_code: row.get("entity_code"),
        debit: get_numeric(row, "debit"),
        credit: get_numeric(row, "credit"),
        adjustment_type: row.get("adjustment_type"),
        reference: row.get("reference"),
        status: row.get("status"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_translation_rate(row: &sqlx::postgres::PgRow) -> ConsolidationTranslationRate {
    ConsolidationTranslationRate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        entity_id: row.get("entity_id"),
        from_currency: row.get("from_currency"),
        to_currency: row.get("to_currency"),
        rate_type: row.get("rate_type"),
        exchange_rate: get_numeric(row, "exchange_rate"),
        effective_date: row.get("effective_date"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl FinancialConsolidationRepository for PostgresFinancialConsolidationRepository {
    // ── Consolidation Ledgers ───────────────────────────────────────

    async fn create_ledger(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        base_currency_code: &str, translation_method: &str,
        equity_elimination_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationLedger> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_ledgers
                (organization_id, code, name, description, base_currency_code,
                 translation_method, equity_elimination_method, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(base_currency_code).bind(translation_method)
        .bind(equity_elimination_method).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_ledger(&row))
    }

    async fn get_ledger(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ConsolidationLedger>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_ledgers WHERE organization_id=$1 AND code=$2",
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_ledger(&r)))
    }

    async fn get_ledger_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationLedger>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_ledgers WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_ledger(&r)))
    }

    async fn list_ledgers(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationLedger>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_ledgers WHERE organization_id=$1 AND is_active=true ORDER BY code",
            )
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_ledgers WHERE organization_id=$1 ORDER BY code",
            )
        }
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_ledger(&r)).collect())
    }

    // ── Consolidation Entities ──────────────────────────────────────

    async fn create_entity(
        &self, org_id: Uuid, ledger_id: Uuid, entity_id: Uuid,
        entity_name: &str, entity_code: &str, local_currency_code: &str,
        ownership_percentage: &str, consolidation_method: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEntity> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_entities
                (organization_id, ledger_id, entity_id, entity_name, entity_code,
                 local_currency_code, ownership_percentage, consolidation_method,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(ledger_id).bind(entity_id)
        .bind(entity_name).bind(entity_code).bind(local_currency_code)
        .bind(ownership_percentage).bind(consolidation_method)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entity(&row))
    }

    async fn get_entity(&self, ledger_id: Uuid, entity_code: &str) -> AtlasResult<Option<ConsolidationEntity>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_entities WHERE ledger_id=$1 AND entity_code=$2",
        )
        .bind(ledger_id).bind(entity_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entity(&r)))
    }

    async fn get_entity_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationEntity>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_entities WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entity(&r)))
    }

    async fn list_entities(&self, ledger_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationEntity>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_entities WHERE ledger_id=$1 AND is_active=true AND include_in_consolidation=true ORDER BY entity_code",
            )
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_entities WHERE ledger_id=$1 ORDER BY entity_code",
            )
        }
        .bind(ledger_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_entity(&r)).collect())
    }

    // ── Consolidation Scenarios ─────────────────────────────────────

    async fn create_scenario(
        &self, org_id: Uuid, ledger_id: Uuid, scenario_number: &str,
        name: &str, description: Option<&str>,
        fiscal_year: i32, period_name: &str,
        period_start_date: chrono::NaiveDate, period_end_date: chrono::NaiveDate,
        translation_rate_type: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationScenario> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_scenarios
                (organization_id, ledger_id, scenario_number, name, description,
                 fiscal_year, period_name, period_start_date, period_end_date,
                 translation_rate_type, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(ledger_id).bind(scenario_number).bind(name)
        .bind(description).bind(fiscal_year).bind(period_name)
        .bind(period_start_date).bind(period_end_date)
        .bind(translation_rate_type).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scenario(&row))
    }

    async fn get_scenario(&self, org_id: Uuid, scenario_number: &str) -> AtlasResult<Option<ConsolidationScenario>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_scenarios WHERE organization_id=$1 AND scenario_number=$2",
        )
        .bind(org_id).bind(scenario_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_scenario(&r)))
    }

    async fn get_scenario_by_id(&self, id: Uuid) -> AtlasResult<Option<ConsolidationScenario>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_scenarios WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_scenario(&r)))
    }

    async fn list_scenarios(
        &self, org_id: Uuid, ledger_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationScenario>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.consolidation_scenarios
            WHERE organization_id=$1
              AND ($2::uuid IS NULL OR ledger_id=$2)
              AND ($3::text IS NULL OR status=$3)
            ORDER BY fiscal_year DESC, period_name"#,
        )
        .bind(org_id).bind(ledger_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_scenario(&r)).collect())
    }

    async fn update_scenario_status(
        &self, id: Uuid, status: &str,
        approved_by: Option<Uuid>, posted_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationScenario> {
        let row = sqlx::query(
            r#"UPDATE _atlas.consolidation_scenarios
            SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                posted_by=COALESCE($4, posted_by),
                posted_at=CASE WHEN $4 IS NOT NULL AND posted_at IS NULL THEN now() ELSE posted_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by).bind(posted_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scenario(&row))
    }

    async fn update_scenario_totals(
        &self, id: Uuid, total_entities: i32, total_eliminations: i32,
        total_adjustments: i32, total_debits: &str, total_credits: &str,
        is_balanced: bool,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.consolidation_scenarios
            SET total_entities=$2, total_eliminations=$3, total_adjustments=$4,
                total_debits=$5::numeric, total_credits=$6::numeric,
                is_balanced=$7, updated_at=now()
            WHERE id=$1"#,
        )
        .bind(id).bind(total_entities).bind(total_eliminations)
        .bind(total_adjustments).bind(total_debits).bind(total_credits)
        .bind(is_balanced)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Trial Balance Lines ─────────────────────────────────────────

    async fn create_trial_balance_line(
        &self, org_id: Uuid, scenario_id: Uuid,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        account_code: &str, account_name: Option<&str>,
        account_type: Option<&str>, financial_statement: Option<&str>,
        local_debit: &str, local_credit: &str, local_balance: &str,
        exchange_rate: Option<&str>,
        translated_debit: &str, translated_credit: &str, translated_balance: &str,
        elimination_debit: &str, elimination_credit: &str, elimination_balance: &str,
        minority_interest_debit: &str, minority_interest_credit: &str, minority_interest_balance: &str,
        consolidated_debit: &str, consolidated_credit: &str, consolidated_balance: &str,
        is_elimination_entry: bool, line_type: &str,
    ) -> AtlasResult<ConsolidationTrialBalanceLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_trial_balance
                (organization_id, scenario_id, entity_id, entity_code,
                 account_code, account_name, account_type, financial_statement,
                 local_debit, local_credit, local_balance,
                 exchange_rate,
                 translated_debit, translated_credit, translated_balance,
                 elimination_debit, elimination_credit, elimination_balance,
                 minority_interest_debit, minority_interest_credit, minority_interest_balance,
                 consolidated_debit, consolidated_credit, consolidated_balance,
                 is_elimination_entry, line_type)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,
                    $9::numeric,$10::numeric,$11::numeric,
                    $12::numeric,
                    $13::numeric,$14::numeric,$15::numeric,
                    $16::numeric,$17::numeric,$18::numeric,
                    $19::numeric,$20::numeric,$21::numeric,
                    $22::numeric,$23::numeric,$24::numeric,
                    $25,$26)
            RETURNING *"#,
        )
        .bind(org_id).bind(scenario_id).bind(entity_id).bind(entity_code)
        .bind(account_code).bind(account_name).bind(account_type).bind(financial_statement)
        .bind(local_debit).bind(local_credit).bind(local_balance)
        .bind(exchange_rate)
        .bind(translated_debit).bind(translated_credit).bind(translated_balance)
        .bind(elimination_debit).bind(elimination_credit).bind(elimination_balance)
        .bind(minority_interest_debit).bind(minority_interest_credit).bind(minority_interest_balance)
        .bind(consolidated_debit).bind(consolidated_credit).bind(consolidated_balance)
        .bind(is_elimination_entry).bind(line_type)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_tb_line(&row))
    }

    async fn list_trial_balance(
        &self, scenario_id: Uuid, entity_id: Option<Uuid>,
        line_type: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationTrialBalanceLine>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.consolidation_trial_balance
            WHERE scenario_id=$1
              AND ($2::uuid IS NULL OR entity_id=$2)
              AND ($3::text IS NULL OR line_type=$3)
            ORDER BY account_code, entity_code"#,
        )
        .bind(scenario_id).bind(entity_id).bind(line_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_tb_line(&r)).collect())
    }

    async fn delete_trial_balance_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.consolidation_trial_balance WHERE scenario_id=$1",
        )
        .bind(scenario_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Elimination Rules ───────────────────────────────────────────

    async fn create_elimination_rule(
        &self, org_id: Uuid, ledger_id: Uuid, rule_code: &str,
        name: &str, description: Option<&str>, elimination_type: &str,
        from_entity_id: Option<Uuid>, to_entity_id: Option<Uuid>,
        from_account_pattern: Option<&str>, to_account_pattern: Option<&str>,
        offset_account_code: &str, priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEliminationRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_elimination_rules
                (organization_id, ledger_id, rule_code, name, description,
                 elimination_type, from_entity_id, to_entity_id,
                 from_account_pattern, to_account_pattern,
                 offset_account_code, priority, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(ledger_id).bind(rule_code).bind(name).bind(description)
        .bind(elimination_type).bind(from_entity_id).bind(to_entity_id)
        .bind(from_account_pattern).bind(to_account_pattern)
        .bind(offset_account_code).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_elimination_rule(&row))
    }

    async fn get_elimination_rule(&self, ledger_id: Uuid, rule_code: &str) -> AtlasResult<Option<ConsolidationEliminationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_elimination_rules WHERE ledger_id=$1 AND rule_code=$2",
        )
        .bind(ledger_id).bind(rule_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_elimination_rule(&r)))
    }

    async fn list_elimination_rules(&self, ledger_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationEliminationRule>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_elimination_rules WHERE ledger_id=$1 AND is_active=true ORDER BY priority, rule_code",
            )
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.consolidation_elimination_rules WHERE ledger_id=$1 ORDER BY priority, rule_code",
            )
        }
        .bind(ledger_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_elimination_rule(&r)).collect())
    }

    // ── Adjustments ─────────────────────────────────────────────────

    async fn create_adjustment(
        &self, org_id: Uuid, scenario_id: Uuid, adjustment_number: &str,
        description: Option<&str>, account_code: &str, account_name: Option<&str>,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        debit: &str, credit: &str, adjustment_type: &str,
        reference: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationAdjustment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_adjustments
                (organization_id, scenario_id, adjustment_number, description,
                 account_code, account_name, entity_id, entity_code,
                 debit, credit, adjustment_type, reference, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,
                    $9::numeric,$10::numeric,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(scenario_id).bind(adjustment_number).bind(description)
        .bind(account_code).bind(account_name).bind(entity_id).bind(entity_code)
        .bind(debit).bind(credit).bind(adjustment_type).bind(reference).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_adjustment(&row))
    }

    async fn get_adjustment(&self, id: Uuid) -> AtlasResult<Option<ConsolidationAdjustment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_adjustments WHERE id=$1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_adjustment(&r)))
    }

    async fn list_adjustments(&self, scenario_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ConsolidationAdjustment>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.consolidation_adjustments
            WHERE scenario_id=$1
              AND ($2::text IS NULL OR status=$2)
            ORDER BY adjustment_number"#,
        )
        .bind(scenario_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_adjustment(&r)).collect())
    }

    async fn update_adjustment_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationAdjustment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.consolidation_adjustments
            SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_adjustment(&row))
    }

    // ── Translation Rates ───────────────────────────────────────────

    async fn create_translation_rate(
        &self, org_id: Uuid, scenario_id: Uuid, entity_id: Uuid,
        from_currency: &str, to_currency: &str,
        rate_type: &str, exchange_rate: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<ConsolidationTranslationRate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.consolidation_translation_rates
                (organization_id, scenario_id, entity_id,
                 from_currency, to_currency, rate_type,
                 exchange_rate, effective_date)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8)
            RETURNING *"#,
        )
        .bind(org_id).bind(scenario_id).bind(entity_id)
        .bind(from_currency).bind(to_currency).bind(rate_type)
        .bind(exchange_rate).bind(effective_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_translation_rate(&row))
    }

    async fn get_translation_rate(
        &self, scenario_id: Uuid, entity_id: Uuid, rate_type: &str,
    ) -> AtlasResult<Option<ConsolidationTranslationRate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.consolidation_translation_rates WHERE scenario_id=$1 AND entity_id=$2 AND rate_type=$3",
        )
        .bind(scenario_id).bind(entity_id).bind(rate_type)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_translation_rate(&r)))
    }

    async fn list_translation_rates(&self, scenario_id: Uuid) -> AtlasResult<Vec<ConsolidationTranslationRate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.consolidation_translation_rates WHERE scenario_id=$1 ORDER BY entity_id, rate_type",
        )
        .bind(scenario_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_translation_rate(&r)).collect())
    }

    // ── Dashboard ───────────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ConsolidationDashboardSummary> {
        let ledger_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.consolidation_ledgers WHERE organization_id=$1 AND is_active=true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let scenario_stats = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status IN ('draft','in_progress','pending_review')) as active_scenarios,
                MAX(CASE WHEN status IN ('posted','approved') THEN created_at END) as last_consolidation,
                MAX(CASE WHEN status IN ('posted','approved') THEN status END) as last_status
            FROM _atlas.consolidation_scenarios WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_scenarios: i64 = scenario_stats.try_get("active_scenarios").unwrap_or(0);
        let last_consolidation: Option<chrono::DateTime<chrono::Utc>> = scenario_stats.try_get("last_consolidation").unwrap_or(None);
        let last_status: Option<String> = scenario_stats.try_get("last_status").unwrap_or(None);

        let entity_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.consolidation_entities WHERE organization_id=$1 AND is_active=true AND include_in_consolidation=true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let rule_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _atlas.consolidation_elimination_rules WHERE organization_id=$1 AND is_active=true",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Scenarios by status
        let status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.consolidation_scenarios
            WHERE organization_id=$1 GROUP BY status"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut scenarios_by_status = serde_json::Map::new();
        for row in &status_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            scenarios_by_status.insert(status, serde_json::json!(cnt));
        }

        // Entities by consolidation method
        let method_rows = sqlx::query(
            r#"SELECT consolidation_method, COUNT(*) as cnt FROM _atlas.consolidation_entities
            WHERE organization_id=$1 AND is_active=true GROUP BY consolidation_method"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut entities_by_method = serde_json::Map::new();
        for row in &method_rows {
            let method: String = row.try_get("consolidation_method").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            entities_by_method.insert(method, serde_json::json!(cnt));
        }

        Ok(ConsolidationDashboardSummary {
            total_ledgers: ledger_count as i32,
            total_active_scenarios: active_scenarios as i32,
            total_entities: entity_count as i32,
            total_elimination_rules: rule_count as i32,
            last_consolidation_date: last_consolidation.map(|d| d.to_rfc3339()),
            last_consolidation_status: last_status,
            scenarios_by_status: serde_json::Value::Object(scenarios_by_status),
            entities_by_method: serde_json::Value::Object(entities_by_method),
            consolidation_completion_percent: "0".to_string(),
        })
    }
}
