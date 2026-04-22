//! Treasury Management Repository
//!
//! PostgreSQL storage for counterparties, treasury deals, and settlements.

use atlas_shared::{
    TreasuryCounterparty, TreasuryDeal, TreasurySettlement, TreasuryDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for treasury management data storage
#[async_trait]
pub trait TreasuryRepository: Send + Sync {
    // Counterparties
    async fn create_counterparty(
        &self,
        org_id: Uuid,
        counterparty_code: &str,
        name: &str,
        counterparty_type: &str,
        country_code: Option<&str>,
        credit_rating: Option<&str>,
        credit_limit: Option<&str>,
        settlement_currency: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryCounterparty>;

    async fn get_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TreasuryCounterparty>>;
    async fn get_counterparty_by_id(&self, id: Uuid) -> AtlasResult<Option<TreasuryCounterparty>>;
    async fn list_counterparties(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<TreasuryCounterparty>>;
    async fn delete_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Deals
    async fn create_deal(
        &self,
        org_id: Uuid,
        deal_number: &str,
        deal_type: &str,
        description: Option<&str>,
        counterparty_id: Uuid,
        counterparty_name: Option<&str>,
        currency_code: &str,
        principal_amount: &str,
        interest_rate: Option<&str>,
        interest_basis: Option<&str>,
        start_date: chrono::NaiveDate,
        maturity_date: chrono::NaiveDate,
        term_days: i32,
        fx_buy_currency: Option<&str>,
        fx_buy_amount: Option<&str>,
        fx_sell_currency: Option<&str>,
        fx_sell_amount: Option<&str>,
        fx_rate: Option<&str>,
        gl_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryDeal>;

    async fn get_deal(&self, id: Uuid) -> AtlasResult<Option<TreasuryDeal>>;
    async fn get_deal_by_number(&self, org_id: Uuid, deal_number: &str) -> AtlasResult<Option<TreasuryDeal>>;
    async fn list_deals(&self, org_id: Uuid, deal_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<TreasuryDeal>>;
    async fn update_deal_status(
        &self,
        id: Uuid,
        status: &str,
        authorized_by: Option<Uuid>,
        settled_at: Option<chrono::DateTime<chrono::Utc>>,
        matured_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<TreasuryDeal>;
    async fn update_deal_interest(&self, id: Uuid, accrued_interest: &str, settlement_amount: Option<&str>) -> AtlasResult<()>;

    // Settlements
    async fn create_settlement(
        &self,
        org_id: Uuid,
        deal_id: Uuid,
        settlement_number: &str,
        settlement_type: &str,
        settlement_date: chrono::NaiveDate,
        principal_amount: &str,
        interest_amount: &str,
        total_amount: &str,
        payment_reference: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasurySettlement>;

    async fn list_settlements(&self, deal_id: Uuid) -> AtlasResult<Vec<TreasurySettlement>>;
    async fn update_settlement_status(&self, id: Uuid, status: &str) -> AtlasResult<TreasurySettlement>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<TreasuryDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresTreasuryRepository {
    pool: PgPool,
}

impl PostgresTreasuryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_counterparty(&self, row: &sqlx::postgres::PgRow) -> TreasuryCounterparty {
        TreasuryCounterparty {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            counterparty_code: row.get("counterparty_code"),
            name: row.get("name"),
            counterparty_type: row.get("counterparty_type"),
            country_code: row.get("country_code"),
            credit_rating: row.get("credit_rating"),
            credit_limit: row.try_get("credit_limit").unwrap_or(None),
            settlement_currency: row.get("settlement_currency"),
            contact_name: row.get("contact_name"),
            contact_email: row.get("contact_email"),
            contact_phone: row.get("contact_phone"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_deal(&self, row: &sqlx::postgres::PgRow) -> TreasuryDeal {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        TreasuryDeal {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            deal_number: row.get("deal_number"),
            deal_type: row.get("deal_type"),
            description: row.get("description"),
            counterparty_id: row.get("counterparty_id"),
            counterparty_name: row.get("counterparty_name"),
            currency_code: row.get("currency_code"),
            principal_amount: get_num(row, "principal_amount"),
            interest_rate: row.try_get("interest_rate").unwrap_or(None),
            interest_basis: row.get("interest_basis"),
            start_date: row.get("start_date"),
            maturity_date: row.get("maturity_date"),
            term_days: row.get("term_days"),
            fx_buy_currency: row.get("fx_buy_currency"),
            fx_buy_amount: row.try_get("fx_buy_amount").unwrap_or(None),
            fx_sell_currency: row.get("fx_sell_currency"),
            fx_sell_amount: row.try_get("fx_sell_amount").unwrap_or(None),
            fx_rate: row.try_get("fx_rate").unwrap_or(None),
            accrued_interest: get_num(row, "accrued_interest"),
            settlement_amount: row.try_get("settlement_amount").unwrap_or(None),
            gl_account_code: row.get("gl_account_code"),
            status: row.get("status"),
            authorized_by: row.get("authorized_by"),
            authorized_at: row.get("authorized_at"),
            settled_at: row.get("settled_at"),
            matured_at: row.get("matured_at"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_settlement(&self, row: &sqlx::postgres::PgRow) -> TreasurySettlement {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        TreasurySettlement {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            deal_id: row.get("deal_id"),
            settlement_number: row.get("settlement_number"),
            settlement_type: row.get("settlement_type"),
            settlement_date: row.get("settlement_date"),
            principal_amount: get_num(row, "principal_amount"),
            interest_amount: get_num(row, "interest_amount"),
            total_amount: get_num(row, "total_amount"),
            payment_reference: row.get("payment_reference"),
            journal_entry_id: row.get("journal_entry_id"),
            status: row.get("status"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl TreasuryRepository for PostgresTreasuryRepository {
    async fn create_counterparty(
        &self,
        org_id: Uuid,
        counterparty_code: &str,
        name: &str,
        counterparty_type: &str,
        country_code: Option<&str>,
        credit_rating: Option<&str>,
        credit_limit: Option<&str>,
        settlement_currency: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryCounterparty> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.treasury_counterparties
                (organization_id, counterparty_code, name, counterparty_type,
                 country_code, credit_rating, credit_limit, settlement_currency,
                 contact_name, contact_email, contact_phone, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(counterparty_code).bind(name).bind(counterparty_type)
        .bind(country_code).bind(credit_rating).bind(credit_limit)
        .bind(settlement_currency)
        .bind(contact_name).bind(contact_email).bind(contact_phone)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_counterparty(&row))
    }

    async fn get_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TreasuryCounterparty>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.treasury_counterparties WHERE organization_id = $1 AND counterparty_code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_counterparty(&r)))
    }

    async fn get_counterparty_by_id(&self, id: Uuid) -> AtlasResult<Option<TreasuryCounterparty>> {
        let row = sqlx::query("SELECT * FROM _atlas.treasury_counterparties WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_counterparty(&r)))
    }

    async fn list_counterparties(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<TreasuryCounterparty>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.treasury_counterparties WHERE organization_id = $1 AND is_active = true ORDER BY counterparty_code"
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.treasury_counterparties WHERE organization_id = $1 ORDER BY counterparty_code"
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_counterparty(r)).collect())
    }

    async fn delete_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.treasury_counterparties WHERE organization_id = $1 AND counterparty_code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_deal(
        &self,
        org_id: Uuid,
        deal_number: &str,
        deal_type: &str,
        description: Option<&str>,
        counterparty_id: Uuid,
        counterparty_name: Option<&str>,
        currency_code: &str,
        principal_amount: &str,
        interest_rate: Option<&str>,
        interest_basis: Option<&str>,
        start_date: chrono::NaiveDate,
        maturity_date: chrono::NaiveDate,
        term_days: i32,
        fx_buy_currency: Option<&str>,
        fx_buy_amount: Option<&str>,
        fx_sell_currency: Option<&str>,
        fx_sell_amount: Option<&str>,
        fx_rate: Option<&str>,
        gl_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryDeal> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.treasury_deals
                (organization_id, deal_number, deal_type, description,
                 counterparty_id, counterparty_name,
                 currency_code, principal_amount,
                 interest_rate, interest_basis,
                 start_date, maturity_date, term_days,
                 fx_buy_currency, fx_buy_amount,
                 fx_sell_currency, fx_sell_amount, fx_rate,
                 accrued_interest, gl_account_code, created_by)
            VALUES ($1, $2, $3, $4,
                    $5, $6,
                    $7, $8::numeric,
                    $9::numeric, $10,
                    $11, $12, $13,
                    $14, $15::numeric,
                    $16, $17::numeric, $18::numeric,
                    0, $19, $20)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(deal_number).bind(deal_type).bind(description)
        .bind(counterparty_id).bind(counterparty_name)
        .bind(currency_code).bind(principal_amount)
        .bind(interest_rate).bind(interest_basis)
        .bind(start_date).bind(maturity_date).bind(term_days)
        .bind(fx_buy_currency).bind(fx_buy_amount)
        .bind(fx_sell_currency).bind(fx_sell_amount).bind(fx_rate)
        .bind(gl_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_deal(&row))
    }

    async fn get_deal(&self, id: Uuid) -> AtlasResult<Option<TreasuryDeal>> {
        let row = sqlx::query("SELECT * FROM _atlas.treasury_deals WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_deal(&r)))
    }

    async fn get_deal_by_number(&self, org_id: Uuid, deal_number: &str) -> AtlasResult<Option<TreasuryDeal>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.treasury_deals WHERE organization_id = $1 AND deal_number = $2"
        )
        .bind(org_id).bind(deal_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_deal(&r)))
    }

    async fn list_deals(&self, org_id: Uuid, deal_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<TreasuryDeal>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.treasury_deals
            WHERE organization_id = $1
              AND ($2::text IS NULL OR deal_type = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY deal_number
            "#,
        )
        .bind(org_id).bind(deal_type).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_deal(r)).collect())
    }

    async fn update_deal_status(
        &self,
        id: Uuid,
        status: &str,
        authorized_by: Option<Uuid>,
        settled_at: Option<chrono::DateTime<chrono::Utc>>,
        matured_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<TreasuryDeal> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.treasury_deals
            SET status = $2,
                authorized_by = COALESCE($3, authorized_by),
                authorized_at = CASE WHEN $3 IS NOT NULL AND authorized_at IS NULL THEN now() ELSE authorized_at END,
                settled_at = COALESCE($4, settled_at),
                matured_at = COALESCE($5, matured_at),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(authorized_by).bind(settled_at).bind(matured_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_deal(&row))
    }

    async fn update_deal_interest(&self, id: Uuid, accrued_interest: &str, settlement_amount: Option<&str>) -> AtlasResult<()> {
        if let Some(sa) = settlement_amount {
            sqlx::query(
                r#"
                UPDATE _atlas.treasury_deals
                SET accrued_interest = $2::numeric,
                    settlement_amount = $3::numeric,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(id).bind(accrued_interest).bind(sa)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        } else {
            sqlx::query(
                r#"
                UPDATE _atlas.treasury_deals
                SET accrued_interest = $2::numeric,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(id).bind(accrued_interest)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    async fn create_settlement(
        &self,
        org_id: Uuid,
        deal_id: Uuid,
        settlement_number: &str,
        settlement_type: &str,
        settlement_date: chrono::NaiveDate,
        principal_amount: &str,
        interest_amount: &str,
        total_amount: &str,
        payment_reference: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasurySettlement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.treasury_settlements
                (organization_id, deal_id, settlement_number, settlement_type,
                 settlement_date, principal_amount, interest_amount, total_amount,
                 payment_reference, created_by)
            VALUES ($1, $2, $3, $4, $5,
                    $6::numeric, $7::numeric, $8::numeric,
                    $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(deal_id).bind(settlement_number).bind(settlement_type)
        .bind(settlement_date)
        .bind(principal_amount).bind(interest_amount).bind(total_amount)
        .bind(payment_reference).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_settlement(&row))
    }

    async fn list_settlements(&self, deal_id: Uuid) -> AtlasResult<Vec<TreasurySettlement>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.treasury_settlements WHERE deal_id = $1 ORDER BY settlement_date"
        )
        .bind(deal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_settlement(r)).collect())
    }

    async fn update_settlement_status(&self, id: Uuid, status: &str) -> AtlasResult<TreasurySettlement> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.treasury_settlements
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_settlement(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<TreasuryDashboardSummary> {
        let rows = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status IN ('authorized', 'settled')) as total_active,
                COALESCE(SUM(principal_amount) FILTER (WHERE deal_type = 'investment' AND status IN ('authorized', 'settled')), 0) as total_inv,
                COALESCE(SUM(principal_amount) FILTER (WHERE deal_type = 'borrowing' AND status IN ('authorized', 'settled')), 0) as total_bor,
                COALESCE(SUM(principal_amount) FILTER (WHERE deal_type IN ('fx_spot', 'fx_forward') AND status IN ('authorized', 'settled')), 0) as total_fx,
                COALESCE(SUM(accrued_interest) FILTER (WHERE status IN ('authorized', 'settled')), 0) as total_interest,
                COUNT(*) FILTER (WHERE maturity_date <= CURRENT_DATE + INTERVAL '7 days' AND status IN ('authorized', 'settled')) as maturing_7,
                COUNT(*) FILTER (WHERE maturity_date <= CURRENT_DATE + INTERVAL '30 days' AND status IN ('authorized', 'settled')) as maturing_30,
                COUNT(*) FILTER (WHERE deal_type = 'investment' AND status IN ('authorized', 'settled')) as inv_count,
                COUNT(*) FILTER (WHERE deal_type = 'borrowing' AND status IN ('authorized', 'settled')) as bor_count,
                COUNT(*) FILTER (WHERE deal_type IN ('fx_spot', 'fx_forward') AND status IN ('authorized', 'settled')) as fx_count
            FROM _atlas.treasury_deals
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = rows.try_get("total_active").unwrap_or(0);
        let maturing_7: i64 = rows.try_get("maturing_7").unwrap_or(0);
        let maturing_30: i64 = rows.try_get("maturing_30").unwrap_or(0);
        let inv_count: i64 = rows.try_get("inv_count").unwrap_or(0);
        let bor_count: i64 = rows.try_get("bor_count").unwrap_or(0);
        let fx_count: i64 = rows.try_get("fx_count").unwrap_or(0);

        let inv: serde_json::Value = rows.try_get("total_inv").unwrap_or(serde_json::json!(0));
        let bor: serde_json::Value = rows.try_get("total_bor").unwrap_or(serde_json::json!(0));
        let fx: serde_json::Value = rows.try_get("total_fx").unwrap_or(serde_json::json!(0));
        let interest: serde_json::Value = rows.try_get("total_interest").unwrap_or(serde_json::json!(0));

        // Count active counterparties
        let cp_count: i64 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.treasury_counterparties WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        .get("cnt");

        Ok(TreasuryDashboardSummary {
            total_active_deals: active as i32,
            total_investments: inv.to_string(),
            total_borrowings: bor.to_string(),
            total_fx_exposure: fx.to_string(),
            total_accrued_interest: interest.to_string(),
            deals_maturing_7_days: maturing_7 as i32,
            deals_maturing_30_days: maturing_30 as i32,
            investment_count: inv_count as i32,
            borrowing_count: bor_count as i32,
            fx_deal_count: fx_count as i32,
            active_counterparties: cp_count as i32,
            deals_by_status: serde_json::json!({}),
            deals_by_type: serde_json::json!({}),
            maturity_profile: serde_json::json!({}),
        })
    }
}
