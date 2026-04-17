//! Currency Repository
//!
//! PostgreSQL storage for currencies, exchange rates, and conversion history.

use atlas_shared::{
    CurrencyDefinition, ExchangeRate,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for currency data storage
#[async_trait]
pub trait CurrencyRepository: Send + Sync {
    // Currency definitions
    async fn create_currency(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        symbol: Option<&str>,
        precision: i32,
        is_base_currency: bool,
    ) -> AtlasResult<CurrencyDefinition>;

    async fn get_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyDefinition>>;
    async fn list_currencies(&self, org_id: Uuid) -> AtlasResult<Vec<CurrencyDefinition>>;
    async fn get_base_currency(&self, org_id: Uuid) -> AtlasResult<Option<CurrencyDefinition>>;
    async fn delete_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Exchange rates
    async fn upsert_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        rate: &str,
        effective_date: chrono::NaiveDate,
        inverse_rate: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExchangeRate>;

    async fn get_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>>;

    async fn get_latest_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        on_or_before: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>>;

    async fn list_rates(
        &self,
        org_id: Uuid,
        from_currency: Option<&str>,
        to_currency: Option<&str>,
        rate_type: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<ExchangeRate>>;

    async fn delete_exchange_rate(&self, id: Uuid) -> AtlasResult<()>;

    // Conversion history
    async fn record_conversion(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        from_currency: &str,
        to_currency: &str,
        from_amount: &str,
        to_amount: &str,
        exchange_rate: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
        gain_loss_amount: Option<&str>,
        gain_loss_type: Option<&str>,
        triangulation_currency: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Uuid>;
}

/// PostgreSQL implementation
pub struct PostgresCurrencyRepository {
    pool: PgPool,
}

impl PostgresCurrencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_currency(&self, row: &sqlx::postgres::PgRow) -> CurrencyDefinition {
        CurrencyDefinition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            symbol: row.get("symbol"),
            precision: row.get("precision"),
            is_base_currency: row.get("is_base_currency"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_rate(&self, row: &sqlx::postgres::PgRow) -> ExchangeRate {
        let rate_val: serde_json::Value = row.try_get::<serde_json::Value, _>("rate")
            .unwrap_or(serde_json::json!("0"));
        let inv_val: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("inverse_rate")
            .ok()
            .flatten();

        ExchangeRate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            from_currency: row.get("from_currency"),
            to_currency: row.get("to_currency"),
            rate_type: row.get("rate_type"),
            rate: rate_val.to_string(),
            effective_date: row.get("effective_date"),
            inverse_rate: inv_val.map(|v| v.to_string()),
            source: row.get("source"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl CurrencyRepository for PostgresCurrencyRepository {
    async fn create_currency(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        symbol: Option<&str>,
        precision: i32,
        is_base_currency: bool,
    ) -> AtlasResult<CurrencyDefinition> {
        // If this is being set as base currency, unset any existing base
        if is_base_currency {
            sqlx::query(
                "UPDATE _atlas.currencies SET is_base_currency = false WHERE organization_id = $1 AND is_base_currency = true"
            )
            .bind(org_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        }

        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.currencies (organization_id, code, name, symbol, precision, is_base_currency)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, symbol = $4, precision = $5, is_base_currency = $6, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(code)
        .bind(name)
        .bind(symbol)
        .bind(precision)
        .bind(is_base_currency)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_currency(&row))
    }

    async fn get_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.currencies WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_currency(&r)))
    }

    async fn list_currencies(&self, org_id: Uuid) -> AtlasResult<Vec<CurrencyDefinition>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.currencies WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_currency(&r)).collect())
    }

    async fn get_base_currency(&self, org_id: Uuid) -> AtlasResult<Option<CurrencyDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.currencies WHERE organization_id = $1 AND is_base_currency = true AND is_active = true"
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_currency(&r)))
    }

    async fn delete_currency(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.currencies SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id)
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn upsert_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        rate: &str,
        effective_date: chrono::NaiveDate,
        inverse_rate: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExchangeRate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.exchange_rates
                (organization_id, from_currency, to_currency, rate_type, rate, effective_date, inverse_rate, source, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6, $7::numeric, $8, $9)
            ON CONFLICT (organization_id, from_currency, to_currency, rate_type, effective_date)
            DO UPDATE SET rate = $5::numeric, inverse_rate = $7::numeric, source = $8, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(from_currency)
        .bind(to_currency)
        .bind(rate_type)
        .bind(rate)
        .bind(effective_date)
        .bind(inverse_rate)
        .bind(source)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_rate(&row))
    }

    async fn get_exchange_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>> {
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.exchange_rates
            WHERE organization_id = $1 AND from_currency = $2 AND to_currency = $3
              AND rate_type = $4 AND effective_date = $5"#,
        )
        .bind(org_id)
        .bind(from_currency)
        .bind(to_currency)
        .bind(rate_type)
        .bind(effective_date)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_rate(&r)))
    }

    async fn get_latest_rate(
        &self,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        on_or_before: chrono::NaiveDate,
    ) -> AtlasResult<Option<ExchangeRate>> {
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.exchange_rates
            WHERE organization_id = $1 AND from_currency = $2 AND to_currency = $3
              AND rate_type = $4 AND effective_date <= $5
            ORDER BY effective_date DESC LIMIT 1"#,
        )
        .bind(org_id)
        .bind(from_currency)
        .bind(to_currency)
        .bind(rate_type)
        .bind(on_or_before)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_rate(&r)))
    }

    async fn list_rates(
        &self,
        org_id: Uuid,
        from_currency: Option<&str>,
        to_currency: Option<&str>,
        rate_type: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<ExchangeRate>> {
        let mut query_str = String::from(
            "SELECT * FROM _atlas.exchange_rates WHERE organization_id = $1"
        );
        let mut param_idx = 1;

        let bind_from = from_currency.is_some();
        let bind_to = to_currency.is_some();
        let bind_type = rate_type.is_some();
        let bind_date = effective_date.is_some();

        if bind_from { param_idx += 1; query_str.push_str(&format!(" AND from_currency = ${}", param_idx)); }
        if bind_to { param_idx += 1; query_str.push_str(&format!(" AND to_currency = ${}", param_idx)); }
        if bind_type { param_idx += 1; query_str.push_str(&format!(" AND rate_type = ${}", param_idx)); }
        if bind_date { param_idx += 1; query_str.push_str(&format!(" AND effective_date = ${}", param_idx)); }

        param_idx += 1;
        let limit_idx = param_idx;
        param_idx += 1;
        let offset_idx = param_idx;

        query_str.push_str(&format!(" ORDER BY effective_date DESC, from_currency, to_currency LIMIT ${} OFFSET ${}", limit_idx, offset_idx));

        let mut query = sqlx::query(&query_str).bind(org_id);
        if let Some(f) = from_currency { query = query.bind(f); }
        if let Some(t) = to_currency { query = query.bind(t); }
        if let Some(rt) = rate_type { query = query.bind(rt); }
        if let Some(d) = effective_date { query = query.bind(d); }
        query = query.bind(limit).bind(offset);

        let rows = query.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_rate(&r)).collect())
    }

    async fn delete_exchange_rate(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.exchange_rates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn record_conversion(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        from_currency: &str,
        to_currency: &str,
        from_amount: &str,
        to_amount: &str,
        exchange_rate: &str,
        rate_type: &str,
        effective_date: chrono::NaiveDate,
        gain_loss_amount: Option<&str>,
        gain_loss_type: Option<&str>,
        triangulation_currency: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.currency_conversions
                (organization_id, entity_type, entity_id, from_currency, to_currency,
                 from_amount, to_amount, exchange_rate, rate_type, effective_date,
                 gain_loss_amount, gain_loss_type, triangulation_currency, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8::numeric, $9, $10,
                    $11::numeric, $12, $13, $14)
            RETURNING id
            "#,
        )
        .bind(org_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(from_currency)
        .bind(to_currency)
        .bind(from_amount)
        .bind(to_amount)
        .bind(exchange_rate)
        .bind(rate_type)
        .bind(effective_date)
        .bind(gain_loss_amount)
        .bind(gain_loss_type)
        .bind(triangulation_currency)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let id: Uuid = row.get("id");
        Ok(id)
    }
}
