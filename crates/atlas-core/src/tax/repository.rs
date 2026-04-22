//! Tax Repository
//!
//! PostgreSQL storage for tax regimes, jurisdictions, rates,
//! determination rules, tax lines, and reports.

use atlas_shared::{
    TaxRegime, TaxJurisdiction, TaxRate, TaxDeterminationRule, TaxLine,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for tax data storage
#[async_trait]
pub trait TaxRepository: Send + Sync {
    // Regimes
    async fn create_regime(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        default_inclusive: bool,
        allows_recovery: bool,
        rounding_rule: &str,
        rounding_precision: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegime>;

    async fn get_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxRegime>>;
    async fn get_regime_by_id(&self, id: Uuid) -> AtlasResult<Option<TaxRegime>>;
    async fn list_regimes(&self, org_id: Uuid) -> AtlasResult<Vec<TaxRegime>>;
    async fn delete_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Jurisdictions
    async fn create_jurisdiction(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        code: &str,
        name: &str,
        geographic_level: &str,
        country_code: Option<&str>,
        state_code: Option<&str>,
        county: Option<&str>,
        city: Option<&str>,
        postal_code_pattern: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxJurisdiction>;

    async fn get_jurisdiction(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<Option<TaxJurisdiction>>;
    async fn list_jurisdictions(&self, org_id: Uuid, regime_id: Option<Uuid>) -> AtlasResult<Vec<TaxJurisdiction>>;
    async fn delete_jurisdiction(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<()>;

    // Tax Rates
    async fn create_tax_rate(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        jurisdiction_id: Option<&Uuid>,
        code: &str,
        name: &str,
        rate_percentage: &str,
        rate_type: &str,
        tax_account_code: Option<&str>,
        recoverable: bool,
        recovery_percentage: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRate>;

    async fn get_tax_rate(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<Option<TaxRate>>;
    async fn get_tax_rate_by_id(&self, id: Uuid) -> AtlasResult<Option<TaxRate>>;
    async fn get_tax_rate_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxRate>>;
    async fn get_effective_tax_rates(&self, org_id: Uuid, regime_id: Uuid, on_date: chrono::NaiveDate) -> AtlasResult<Vec<TaxRate>>;
    async fn list_tax_rates(&self, org_id: Uuid, regime_id: Uuid) -> AtlasResult<Vec<TaxRate>>;
    async fn delete_tax_rate(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<()>;

    // Determination Rules
    async fn create_determination_rule(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        name: &str,
        description: Option<&str>,
        priority: i32,
        condition: serde_json::Value,
        action: serde_json::Value,
        stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxDeterminationRule>;

    async fn list_determination_rules(&self, org_id: Uuid, regime_id: Uuid) -> AtlasResult<Vec<TaxDeterminationRule>>;

    // Tax Lines
    async fn create_tax_line(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        line_id: Option<Uuid>,
        regime_id: Option<Uuid>,
        jurisdiction_id: Option<Uuid>,
        tax_rate_id: Uuid,
        taxable_amount: &str,
        tax_rate_percentage: &str,
        tax_amount: &str,
        is_inclusive: bool,
        original_amount: Option<&str>,
        recoverable_amount: Option<&str>,
        non_recoverable_amount: Option<&str>,
        tax_account_code: Option<&str>,
        determination_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxLine>;

    async fn get_tax_lines(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<TaxLine>>;

    // Reports
    async fn generate_tax_report(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        jurisdiction_id: Option<&Uuid>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxReport>;

    async fn list_tax_reports(&self, org_id: Uuid, regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxReport>>;
}

/// PostgreSQL implementation
pub struct PostgresTaxRepository {
    pool: PgPool,
}

impl PostgresTaxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_regime(&self, row: &sqlx::postgres::PgRow) -> TaxRegime {
        TaxRegime {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            tax_type: row.get("tax_type"),
            default_inclusive: row.get("default_inclusive"),
            allows_recovery: row.get("allows_recovery"),
            rounding_rule: row.get("rounding_rule"),
            rounding_precision: row.get("rounding_precision"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_jurisdiction(&self, row: &sqlx::postgres::PgRow) -> TaxJurisdiction {
        TaxJurisdiction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            regime_id: row.get("regime_id"),
            code: row.get("code"),
            name: row.get("name"),
            geographic_level: row.get("geographic_level"),
            country_code: row.get("country_code"),
            state_code: row.get("state_code"),
            county: row.get("county"),
            city: row.get("city"),
            postal_code_pattern: row.get("postal_code_pattern"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_tax_rate(&self, row: &sqlx::postgres::PgRow) -> TaxRate {
        let rate_val: serde_json::Value = row.try_get::<serde_json::Value, _>("rate_percentage")
            .unwrap_or(serde_json::json!("0"));
        let rec_val: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("recovery_percentage")
            .ok().flatten();

        TaxRate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            regime_id: row.get("regime_id"),
            jurisdiction_id: row.get("jurisdiction_id"),
            code: row.get("code"),
            name: row.get("name"),
            rate_percentage: rate_val.to_string(),
            rate_type: row.get("rate_type"),
            tax_account_code: row.get("tax_account_code"),
            recoverable: row.get("recoverable"),
            recovery_percentage: rec_val.map(|v| v.to_string()),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_determination_rule(&self, row: &sqlx::postgres::PgRow) -> TaxDeterminationRule {
        TaxDeterminationRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            regime_id: row.get("regime_id"),
            name: row.get("name"),
            description: row.get("description"),
            priority: row.get("priority"),
            condition: row.get("condition"),
            action: row.get("action"),
            stop_on_match: row.get("stop_on_match"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_tax_line(&self, row: &sqlx::postgres::PgRow) -> TaxLine {
        let taxable: serde_json::Value = row.try_get::<serde_json::Value, _>("taxable_amount")
            .unwrap_or(serde_json::json!("0"));
        let rate_pct: serde_json::Value = row.try_get::<serde_json::Value, _>("tax_rate_percentage")
            .unwrap_or(serde_json::json!("0"));
        let tax_amt: serde_json::Value = row.try_get::<serde_json::Value, _>("tax_amount")
            .unwrap_or(serde_json::json!("0"));
        let orig: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("original_amount")
            .ok().flatten();
        let recov: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("recoverable_amount")
            .ok().flatten();
        let non_recov: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("non_recoverable_amount")
            .ok().flatten();

        TaxLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            line_id: row.get("line_id"),
            regime_id: row.get("regime_id"),
            jurisdiction_id: row.get("jurisdiction_id"),
            tax_rate_id: row.get("tax_rate_id"),
            taxable_amount: taxable.to_string(),
            tax_rate_percentage: rate_pct.to_string(),
            tax_amount: tax_amt.to_string(),
            is_inclusive: row.get("is_inclusive"),
            original_amount: orig.map(|v| v.to_string()),
            recoverable_amount: recov.map(|v| v.to_string()),
            non_recoverable_amount: non_recov.map(|v| v.to_string()),
            tax_account_code: row.get("tax_account_code"),
            determination_rule_id: row.get("determination_rule_id"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl TaxRepository for PostgresTaxRepository {
    // ========================================================================
    // Regimes
    // ========================================================================

    async fn create_regime(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        default_inclusive: bool,
        allows_recovery: bool,
        rounding_rule: &str,
        rounding_precision: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegime> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_regimes
                (organization_id, code, name, description, tax_type,
                 default_inclusive, allows_recovery, rounding_rule, rounding_precision,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, tax_type = $5,
                    default_inclusive = $6, allows_recovery = $7,
                    rounding_rule = $8, rounding_precision = $9,
                    effective_from = $10, effective_to = $11, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(tax_type)
        .bind(default_inclusive).bind(allows_recovery).bind(rounding_rule).bind(rounding_precision)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_regime(&row))
    }

    async fn get_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxRegime>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_regimes WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_regime(&r)))
    }

    async fn get_regime_by_id(&self, id: Uuid) -> AtlasResult<Option<TaxRegime>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_regimes WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_regime(&r)))
    }

    async fn list_regimes(&self, org_id: Uuid) -> AtlasResult<Vec<TaxRegime>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_regimes WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_regime(r)).collect())
    }

    async fn delete_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.tax_regimes SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Jurisdictions
    // ========================================================================

    async fn create_jurisdiction(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        code: &str,
        name: &str,
        geographic_level: &str,
        country_code: Option<&str>,
        state_code: Option<&str>,
        county: Option<&str>,
        city: Option<&str>,
        postal_code_pattern: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxJurisdiction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_jurisdictions
                (organization_id, regime_id, code, name, geographic_level,
                 country_code, state_code, county, city, postal_code_pattern, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (organization_id, regime_id, code) DO UPDATE
                SET name = $4, geographic_level = $5, country_code = $6,
                    state_code = $7, county = $8, city = $9,
                    postal_code_pattern = $10, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(regime_id).bind(code).bind(name).bind(geographic_level)
        .bind(country_code).bind(state_code).bind(county).bind(city)
        .bind(postal_code_pattern).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_jurisdiction(&row))
    }

    async fn get_jurisdiction(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<Option<TaxJurisdiction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_jurisdictions WHERE organization_id = $1 AND regime_id = $2 AND code = $3 AND is_active = true"
        )
        .bind(org_id).bind(regime_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_jurisdiction(&r)))
    }

    async fn list_jurisdictions(&self, org_id: Uuid, regime_id: Option<Uuid>) -> AtlasResult<Vec<TaxJurisdiction>> {
        let rows = match regime_id {
            Some(rid) => sqlx::query(
                "SELECT * FROM _atlas.tax_jurisdictions WHERE organization_id = $1 AND regime_id = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(rid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.tax_jurisdictions WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_jurisdiction(r)).collect())
    }

    async fn delete_jurisdiction(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.tax_jurisdictions SET is_active = false, updated_at = now() WHERE organization_id = $1 AND regime_id = $2 AND code = $3"
        )
        .bind(org_id).bind(regime_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Tax Rates
    // ========================================================================

    async fn create_tax_rate(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        jurisdiction_id: Option<&Uuid>,
        code: &str,
        name: &str,
        rate_percentage: &str,
        rate_type: &str,
        tax_account_code: Option<&str>,
        recoverable: bool,
        recovery_percentage: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_rates
                (organization_id, regime_id, jurisdiction_id, code, name,
                 rate_percentage, rate_type, tax_account_code, recoverable,
                 recovery_percentage, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10::numeric, $11, $12, $13)
            ON CONFLICT (organization_id, regime_id, code) DO UPDATE
                SET name = $5, jurisdiction_id = $3, rate_percentage = $6::numeric,
                    rate_type = $7, tax_account_code = $8, recoverable = $9,
                    recovery_percentage = $10::numeric, effective_from = $11,
                    effective_to = $12, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(regime_id).bind(jurisdiction_id).bind(code).bind(name)
        .bind(rate_percentage).bind(rate_type).bind(tax_account_code).bind(recoverable)
        .bind(recovery_percentage).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_tax_rate(&row))
    }

    async fn get_tax_rate(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<Option<TaxRate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_rates WHERE organization_id = $1 AND regime_id = $2 AND code = $3 AND is_active = true"
        )
        .bind(org_id).bind(regime_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_tax_rate(&r)))
    }

    async fn get_tax_rate_by_id(&self, id: Uuid) -> AtlasResult<Option<TaxRate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_rates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_tax_rate(&r)))
    }

    async fn get_tax_rate_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxRate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_rates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_tax_rate(&r)))
    }

    async fn get_effective_tax_rates(&self, org_id: Uuid, regime_id: Uuid, on_date: chrono::NaiveDate) -> AtlasResult<Vec<TaxRate>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.tax_rates
            WHERE organization_id = $1 AND regime_id = $2 AND is_active = true
              AND effective_from <= $3 AND (effective_to IS NULL OR effective_to >= $3)
            ORDER BY code"#
        )
        .bind(org_id).bind(regime_id).bind(on_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_tax_rate(r)).collect())
    }

    async fn list_tax_rates(&self, org_id: Uuid, regime_id: Uuid) -> AtlasResult<Vec<TaxRate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_rates WHERE organization_id = $1 AND regime_id = $2 AND is_active = true ORDER BY code"
        )
        .bind(org_id).bind(regime_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_tax_rate(r)).collect())
    }

    async fn delete_tax_rate(&self, org_id: Uuid, regime_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.tax_rates SET is_active = false, updated_at = now() WHERE organization_id = $1 AND regime_id = $2 AND code = $3"
        )
        .bind(org_id).bind(regime_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Determination Rules
    // ========================================================================

    async fn create_determination_rule(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        name: &str,
        description: Option<&str>,
        priority: i32,
        condition: serde_json::Value,
        action: serde_json::Value,
        stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxDeterminationRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_determination_rules
                (organization_id, regime_id, name, description, priority,
                 condition, action, stop_on_match, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(regime_id).bind(name).bind(description).bind(priority)
        .bind(condition).bind(action).bind(stop_on_match)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_determination_rule(&row))
    }

    async fn list_determination_rules(&self, org_id: Uuid, regime_id: Uuid) -> AtlasResult<Vec<TaxDeterminationRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_determination_rules WHERE organization_id = $1 AND regime_id = $2 AND is_active = true ORDER BY priority"
        )
        .bind(org_id).bind(regime_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_determination_rule(r)).collect())
    }

    // ========================================================================
    // Tax Lines
    // ========================================================================

    async fn create_tax_line(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        line_id: Option<Uuid>,
        regime_id: Option<Uuid>,
        jurisdiction_id: Option<Uuid>,
        tax_rate_id: Uuid,
        taxable_amount: &str,
        tax_rate_percentage: &str,
        tax_amount: &str,
        is_inclusive: bool,
        original_amount: Option<&str>,
        recoverable_amount: Option<&str>,
        non_recoverable_amount: Option<&str>,
        tax_account_code: Option<&str>,
        determination_rule_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_lines
                (organization_id, entity_type, entity_id, line_id,
                 regime_id, jurisdiction_id, tax_rate_id,
                 taxable_amount, tax_rate_percentage, tax_amount,
                 is_inclusive, original_amount, recoverable_amount, non_recoverable_amount,
                 tax_account_code, determination_rule_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9::numeric, $10::numeric,
                    $11, $12::numeric, $13::numeric, $14::numeric,
                    $15, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entity_type).bind(entity_id).bind(line_id)
        .bind(regime_id).bind(jurisdiction_id).bind(tax_rate_id)
        .bind(taxable_amount).bind(tax_rate_percentage).bind(tax_amount)
        .bind(is_inclusive).bind(original_amount).bind(recoverable_amount).bind(non_recoverable_amount)
        .bind(tax_account_code).bind(determination_rule_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_tax_line(&row))
    }

    async fn get_tax_lines(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<TaxLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_lines WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at"
        )
        .bind(entity_type).bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_tax_line(r)).collect())
    }

    // ========================================================================
    // Reports
    // ========================================================================

    async fn generate_tax_report(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        jurisdiction_id: Option<&Uuid>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxReport> {
        // Aggregate tax lines for the period
        let agg = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(taxable_amount), 0) as total_taxable,
                COALESCE(SUM(tax_amount), 0) as total_tax,
                COALESCE(SUM(recoverable_amount), 0) as total_recoverable,
                COALESCE(SUM(non_recoverable_amount), 0) as total_non_recoverable,
                COUNT(*) as txn_count
            FROM _atlas.tax_lines
            WHERE organization_id = $1 AND regime_id = $2
              AND created_at >= $3 AND created_at <= $4
            "#,
        )
        .bind(org_id).bind(regime_id)
        .bind(period_start).bind(period_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_taxable: serde_json::Value = agg.try_get("total_taxable").unwrap_or(serde_json::json!("0"));
        let total_tax: serde_json::Value = agg.try_get("total_tax").unwrap_or(serde_json::json!("0"));
        let total_recoverable: serde_json::Value = agg.try_get("total_recoverable").unwrap_or(serde_json::json!("0"));
        let total_non_recoverable: serde_json::Value = agg.try_get("total_non_recoverable").unwrap_or(serde_json::json!("0"));
        let txn_count: i64 = agg.try_get("txn_count").unwrap_or(0);

        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_reports
                (organization_id, regime_id, jurisdiction_id,
                 period_start, period_end,
                 total_taxable_amount, total_tax_amount,
                 total_recoverable_amount, total_non_recoverable_amount,
                 transaction_count, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric, $9::numeric, $10, $11)
            ON CONFLICT (organization_id, regime_id, jurisdiction_id, period_start, period_end)
            DO UPDATE SET
                total_taxable_amount = $6::numeric, total_tax_amount = $7::numeric,
                total_recoverable_amount = $8::numeric, total_non_recoverable_amount = $9::numeric,
                transaction_count = $10, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(regime_id).bind(jurisdiction_id)
        .bind(period_start).bind(period_end)
        .bind(total_taxable.to_string())
        .bind(total_tax.to_string())
        .bind(total_recoverable.to_string())
        .bind(total_non_recoverable.to_string())
        .bind(txn_count as i32)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let report_total_taxable: serde_json::Value = row.try_get("total_taxable_amount").unwrap_or(serde_json::json!("0"));
        let report_total_tax: serde_json::Value = row.try_get("total_tax_amount").unwrap_or(serde_json::json!("0"));
        let report_total_recoverable: serde_json::Value = row.try_get("total_recoverable_amount").unwrap_or(serde_json::json!("0"));
        let report_total_non_recoverable: serde_json::Value = row.try_get("total_non_recoverable_amount").unwrap_or(serde_json::json!("0"));

        Ok(atlas_shared::TaxReport {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            regime_id: row.get("regime_id"),
            jurisdiction_id: row.get("jurisdiction_id"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            total_taxable_amount: report_total_taxable.to_string(),
            total_tax_amount: report_total_tax.to_string(),
            total_recoverable_amount: report_total_recoverable.to_string(),
            total_non_recoverable_amount: report_total_non_recoverable.to_string(),
            transaction_count: row.get("transaction_count"),
            status: row.get("status"),
            filed_by: row.get("filed_by"),
            filed_at: row.get("filed_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_tax_reports(&self, org_id: Uuid, regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxReport>> {
        let rows = match regime_id {
            Some(rid) => sqlx::query(
                "SELECT * FROM _atlas.tax_reports WHERE organization_id = $1 AND regime_id = $2 ORDER BY period_start DESC"
            )
            .bind(org_id).bind(rid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.tax_reports WHERE organization_id = $1 ORDER BY period_start DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|row| {
            let total_taxable: serde_json::Value = row.try_get("total_taxable_amount").unwrap_or(serde_json::json!("0"));
            let total_tax: serde_json::Value = row.try_get("total_tax_amount").unwrap_or(serde_json::json!("0"));
            let total_recoverable: serde_json::Value = row.try_get("total_recoverable_amount").unwrap_or(serde_json::json!("0"));
            let total_non_recoverable: serde_json::Value = row.try_get("total_non_recoverable_amount").unwrap_or(serde_json::json!("0"));

            atlas_shared::TaxReport {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                regime_id: row.get("regime_id"),
                jurisdiction_id: row.get("jurisdiction_id"),
                period_start: row.get("period_start"),
                period_end: row.get("period_end"),
                total_taxable_amount: total_taxable.to_string(),
                total_tax_amount: total_tax.to_string(),
                total_recoverable_amount: total_recoverable.to_string(),
                total_non_recoverable_amount: total_non_recoverable.to_string(),
                transaction_count: row.get("transaction_count"),
                status: row.get("status"),
                filed_by: row.get("filed_by"),
                filed_at: row.get("filed_at"),
                metadata: row.get("metadata"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect())
    }
}
