//! Rebate Management Repository
//!
//! PostgreSQL storage for rebate agreements, tiers, transactions, accruals,
//! settlements, and dashboard analytics.

use atlas_shared::{
    RebateAgreement, RebateTier, RebateTransaction, RebateAccrual,
    RebateSettlement, RebateSettlementLine, RebateDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for rebate management data storage
#[async_trait]
pub trait RebateManagementRepository: Send + Sync {
    // Agreements
    async fn create_agreement(
        &self, org_id: Uuid, agreement_number: &str, name: &str, description: Option<&str>,
        rebate_type: &str, direction: &str, partner_type: &str,
        partner_id: Option<Uuid>, partner_name: Option<&str>, partner_number: Option<&str>,
        product_category: Option<&str>, product_id: Option<Uuid>, product_name: Option<&str>,
        uom: Option<&str>, currency_code: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        calculation_method: &str,
        accrual_account: Option<&str>, liability_account: Option<&str>, expense_account: Option<&str>,
        payment_terms: Option<&str>, settlement_frequency: Option<&str>,
        minimum_amount: f64, maximum_amount: Option<f64>,
        auto_accrue: bool, requires_approval: bool,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAgreement>;
    async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<RebateAgreement>>;
    async fn get_agreement_by_number(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<Option<RebateAgreement>>;
    async fn list_agreements(&self, org_id: Uuid, status: Option<&str>, rebate_type: Option<&str>, partner_type: Option<&str>) -> AtlasResult<Vec<RebateAgreement>>;
    async fn update_agreement_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateAgreement>;
    async fn delete_agreement(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<()>;

    // Tiers
    async fn create_tier(
        &self, org_id: Uuid, agreement_id: Uuid, tier_number: i32,
        from_value: f64, to_value: Option<f64>, rebate_rate: f64,
        rate_type: &str, description: Option<&str>,
    ) -> AtlasResult<RebateTier>;
    async fn list_tiers(&self, agreement_id: Uuid) -> AtlasResult<Vec<RebateTier>>;
    async fn delete_tier(&self, id: Uuid) -> AtlasResult<()>;

    // Transactions
    async fn create_transaction(
        &self, org_id: Uuid, agreement_id: Uuid, transaction_number: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        product_id: Option<Uuid>, product_name: Option<&str>,
        quantity: f64, unit_price: f64, transaction_amount: f64,
        currency_code: &str, applicable_rate: f64, rebate_amount: f64,
        tier_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateTransaction>;
    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<RebateTransaction>>;
    async fn get_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<RebateTransaction>>;
    async fn list_transactions(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<RebateTransaction>;
    async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()>;

    // Accruals
    async fn create_accrual(
        &self, org_id: Uuid, agreement_id: Uuid, accrual_number: &str,
        accrual_date: chrono::NaiveDate, accrual_period: Option<&str>,
        accumulated_quantity: f64, accumulated_amount: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, accrued_amount: f64,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAccrual>;
    async fn get_accrual(&self, id: Uuid) -> AtlasResult<Option<RebateAccrual>>;
    async fn get_accrual_by_number(&self, org_id: Uuid, accrual_number: &str) -> AtlasResult<Option<RebateAccrual>>;
    async fn list_accruals(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateAccrual>>;
    async fn update_accrual_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateAccrual>;
    async fn delete_accrual(&self, org_id: Uuid, accrual_number: &str) -> AtlasResult<()>;

    // Settlements
    async fn create_settlement(
        &self, org_id: Uuid, agreement_id: Uuid, settlement_number: &str,
        settlement_date: chrono::NaiveDate,
        settlement_period_from: Option<chrono::NaiveDate>,
        settlement_period_to: Option<chrono::NaiveDate>,
        total_qualifying_amount: f64, total_qualifying_quantity: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, settlement_amount: f64,
        currency_code: &str, settlement_type: &str, payment_method: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateSettlement>;
    async fn get_settlement(&self, id: Uuid) -> AtlasResult<Option<RebateSettlement>>;
    async fn get_settlement_by_number(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<Option<RebateSettlement>>;
    async fn list_settlements(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateSettlement>>;
    async fn update_settlement_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateSettlement>;
    async fn approve_settlement(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<RebateSettlement>;
    async fn pay_settlement(&self, id: Uuid) -> AtlasResult<RebateSettlement>;
    async fn delete_settlement(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<()>;

    // Settlement Lines
    async fn create_settlement_line(&self, settlement_id: Uuid, transaction_id: Uuid, amount: f64) -> AtlasResult<RebateSettlementLine>;
    async fn list_settlement_lines(&self, settlement_id: Uuid) -> AtlasResult<Vec<RebateSettlementLine>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RebateDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresRebateManagementRepository {
    pool: PgPool,
}

impl PostgresRebateManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper numeric decoding
fn get_numeric(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    if let Ok(v) = row.try_get::<f64, _>(column) { return v; }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() { return n; }
        if let Some(s) = v.as_str() { if let Ok(n) = s.parse::<f64>() { return n; } }
    }
    if let Ok(s) = row.try_get::<String, _>(column) { return s.parse::<f64>().unwrap_or(0.0); }
    0.0
}

fn get_optional_numeric(row: &sqlx::postgres::PgRow, column: &str) -> Option<f64> {
    if let Ok(v) = row.try_get::<f64, _>(column) { return Some(v); }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() { return Some(n); }
        if let Some(s) = v.as_str() { return s.parse::<f64>().ok(); }
    }
    if let Ok(s) = row.try_get::<String, _>(column) { return s.parse::<f64>().ok(); }
    None
}

fn row_to_agreement(row: &sqlx::postgres::PgRow) -> RebateAgreement {
    RebateAgreement {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        agreement_number: row.try_get("agreement_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        rebate_type: row.try_get("rebate_type").unwrap_or_default(),
        direction: row.try_get("direction").unwrap_or_default(),
        partner_type: row.try_get("partner_type").unwrap_or_default(),
        partner_id: row.try_get("partner_id").unwrap_or_default(),
        partner_name: row.try_get("partner_name").unwrap_or_default(),
        partner_number: row.try_get("partner_number").unwrap_or_default(),
        product_category: row.try_get("product_category").unwrap_or_default(),
        product_id: row.try_get("product_id").unwrap_or_default(),
        product_name: row.try_get("product_name").unwrap_or_default(),
        uom: row.try_get("uom").unwrap_or_default(),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: row.try_get("end_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        status: row.try_get("status").unwrap_or_default(),
        calculation_method: row.try_get("calculation_method").unwrap_or_default(),
        accrual_account: row.try_get("accrual_account").unwrap_or_default(),
        liability_account: row.try_get("liability_account").unwrap_or_default(),
        expense_account: row.try_get("expense_account").unwrap_or_default(),
        payment_terms: row.try_get("payment_terms").unwrap_or_default(),
        settlement_frequency: row.try_get("settlement_frequency").unwrap_or_default(),
        minimum_amount: get_numeric(row, "minimum_amount"),
        maximum_amount: get_optional_numeric(row, "maximum_amount"),
        auto_accrue: row.try_get("auto_accrue").unwrap_or(true),
        requires_approval: row.try_get("requires_approval").unwrap_or(true),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_tier(row: &sqlx::postgres::PgRow) -> RebateTier {
    RebateTier {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        agreement_id: row.try_get("agreement_id").unwrap_or_default(),
        tier_number: row.try_get("tier_number").unwrap_or_default(),
        from_value: get_numeric(row, "from_value"),
        to_value: get_optional_numeric(row, "to_value"),
        rebate_rate: get_numeric(row, "rebate_rate"),
        rate_type: row.try_get("rate_type").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> RebateTransaction {
    RebateTransaction {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        agreement_id: row.try_get("agreement_id").unwrap_or_default(),
        transaction_number: row.try_get("transaction_number").unwrap_or_default(),
        source_type: row.try_get("source_type").unwrap_or_default(),
        source_id: row.try_get("source_id").unwrap_or_default(),
        source_number: row.try_get("source_number").unwrap_or_default(),
        transaction_date: row.try_get("transaction_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        product_id: row.try_get("product_id").unwrap_or_default(),
        product_name: row.try_get("product_name").unwrap_or_default(),
        quantity: get_numeric(row, "quantity"),
        unit_price: get_numeric(row, "unit_price"),
        transaction_amount: get_numeric(row, "transaction_amount"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        applicable_rate: get_numeric(row, "applicable_rate"),
        rebate_amount: get_numeric(row, "rebate_amount"),
        status: row.try_get("status").unwrap_or_default(),
        tier_id: row.try_get("tier_id").unwrap_or_default(),
        excluded_reason: row.try_get("excluded_reason").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_accrual(row: &sqlx::postgres::PgRow) -> RebateAccrual {
    RebateAccrual {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        agreement_id: row.try_get("agreement_id").unwrap_or_default(),
        accrual_number: row.try_get("accrual_number").unwrap_or_default(),
        accrual_date: row.try_get("accrual_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        accrual_period: row.try_get("accrual_period").unwrap_or_default(),
        accumulated_quantity: get_numeric(row, "accumulated_quantity"),
        accumulated_amount: get_numeric(row, "accumulated_amount"),
        applicable_tier_id: row.try_get("applicable_tier_id").unwrap_or_default(),
        applicable_rate: get_numeric(row, "applicable_rate"),
        accrued_amount: get_numeric(row, "accrued_amount"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        gl_posted: row.try_get("gl_posted").unwrap_or(false),
        gl_journal_id: row.try_get("gl_journal_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_settlement(row: &sqlx::postgres::PgRow) -> RebateSettlement {
    RebateSettlement {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        agreement_id: row.try_get("agreement_id").unwrap_or_default(),
        settlement_number: row.try_get("settlement_number").unwrap_or_default(),
        settlement_date: row.try_get("settlement_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        settlement_period_from: row.try_get("settlement_period_from").unwrap_or_default(),
        settlement_period_to: row.try_get("settlement_period_to").unwrap_or_default(),
        total_qualifying_amount: get_numeric(row, "total_qualifying_amount"),
        total_qualifying_quantity: get_numeric(row, "total_qualifying_quantity"),
        applicable_tier_id: row.try_get("applicable_tier_id").unwrap_or_default(),
        applicable_rate: get_numeric(row, "applicable_rate"),
        settlement_amount: get_numeric(row, "settlement_amount"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        settlement_type: row.try_get("settlement_type").unwrap_or_default(),
        payment_method: row.try_get("payment_method").unwrap_or_default(),
        payment_reference: row.try_get("payment_reference").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        paid_at: row.try_get("paid_at").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_settlement_line(row: &sqlx::postgres::PgRow) -> RebateSettlementLine {
    RebateSettlementLine {
        id: row.try_get("id").unwrap_or_default(),
        settlement_id: row.try_get("settlement_id").unwrap_or_default(),
        transaction_id: row.try_get("transaction_id").unwrap_or_default(),
        settlement_amount: get_numeric(row, "settlement_amount"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl RebateManagementRepository for PostgresRebateManagementRepository {
    // ========================================================================
    // Agreements
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_agreement(
        &self, org_id: Uuid, agreement_number: &str, name: &str, description: Option<&str>,
        rebate_type: &str, direction: &str, partner_type: &str,
        partner_id: Option<Uuid>, partner_name: Option<&str>, partner_number: Option<&str>,
        product_category: Option<&str>, product_id: Option<Uuid>, product_name: Option<&str>,
        uom: Option<&str>, currency_code: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        calculation_method: &str,
        accrual_account: Option<&str>, liability_account: Option<&str>, expense_account: Option<&str>,
        payment_terms: Option<&str>, settlement_frequency: Option<&str>,
        minimum_amount: f64, maximum_amount: Option<f64>,
        auto_accrue: bool, requires_approval: bool,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAgreement> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_agreements
                (organization_id, agreement_number, name, description,
                 rebate_type, direction, partner_type,
                 partner_id, partner_name, partner_number,
                 product_category, product_id, product_name,
                 uom, currency_code, start_date, end_date,
                 calculation_method, accrual_account, liability_account, expense_account,
                 payment_terms, settlement_frequency,
                 minimum_amount, maximum_amount, auto_accrue, requires_approval,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, $28, '{}'::jsonb, $29)
            RETURNING *"#,
        )
        .bind(org_id).bind(agreement_number).bind(name).bind(description)
        .bind(rebate_type).bind(direction).bind(partner_type)
        .bind(partner_id).bind(partner_name).bind(partner_number)
        .bind(product_category).bind(product_id).bind(product_name)
        .bind(uom).bind(currency_code).bind(start_date).bind(end_date)
        .bind(calculation_method)
        .bind(accrual_account).bind(liability_account).bind(expense_account)
        .bind(payment_terms).bind(settlement_frequency)
        .bind(minimum_amount).bind(maximum_amount).bind(auto_accrue).bind(requires_approval)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_agreement(&row))
    }

    async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<RebateAgreement>> {
        let row = sqlx::query("SELECT * FROM _atlas.rebate_agreements WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_agreement))
    }

    async fn get_agreement_by_number(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<Option<RebateAgreement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.rebate_agreements WHERE organization_id = $1 AND agreement_number = $2"
        ).bind(org_id).bind(agreement_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_agreement))
    }

    async fn list_agreements(&self, org_id: Uuid, status: Option<&str>, rebate_type: Option<&str>, partner_type: Option<&str>) -> AtlasResult<Vec<RebateAgreement>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.rebate_agreements
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR rebate_type = $3)
                 AND ($4::text IS NULL OR partner_type = $4)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(rebate_type).bind(partner_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_agreement).collect())
    }

    async fn update_agreement_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateAgreement> {
        let row = sqlx::query(
            "UPDATE _atlas.rebate_agreements SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Rebate agreement {} not found", id)))?;
        Ok(row_to_agreement(&row))
    }

    async fn delete_agreement(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.rebate_agreements WHERE organization_id = $1 AND agreement_number = $2 AND status = 'draft'"
        ).bind(org_id).bind(agreement_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Draft agreement '{}' not found", agreement_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Tiers
    // ========================================================================

    async fn create_tier(
        &self, org_id: Uuid, agreement_id: Uuid, tier_number: i32,
        from_value: f64, to_value: Option<f64>, rebate_rate: f64,
        rate_type: &str, description: Option<&str>,
    ) -> AtlasResult<RebateTier> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_tiers
                (organization_id, agreement_id, tier_number,
                 from_value, to_value, rebate_rate, rate_type, description, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(agreement_id).bind(tier_number)
        .bind(from_value).bind(to_value).bind(rebate_rate)
        .bind(rate_type).bind(description)
        .fetch_one(&self.pool).await?;
        Ok(row_to_tier(&row))
    }

    async fn list_tiers(&self, agreement_id: Uuid) -> AtlasResult<Vec<RebateTier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.rebate_tiers WHERE agreement_id = $1 ORDER BY tier_number"
        ).bind(agreement_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_tier).collect())
    }

    async fn delete_tier(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.rebate_tiers WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Tier not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_transaction(
        &self, org_id: Uuid, agreement_id: Uuid, transaction_number: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        product_id: Option<Uuid>, product_name: Option<&str>,
        quantity: f64, unit_price: f64, transaction_amount: f64,
        currency_code: &str, applicable_rate: f64, rebate_amount: f64,
        tier_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateTransaction> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_transactions
                (organization_id, agreement_id, transaction_number,
                 source_type, source_id, source_number, transaction_date,
                 product_id, product_name, quantity, unit_price, transaction_amount,
                 currency_code, applicable_rate, rebate_amount, tier_id, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, '{}'::jsonb, $17)
            RETURNING *"#,
        )
        .bind(org_id).bind(agreement_id).bind(transaction_number)
        .bind(source_type).bind(source_id).bind(source_number).bind(transaction_date)
        .bind(product_id).bind(product_name).bind(quantity).bind(unit_price).bind(transaction_amount)
        .bind(currency_code).bind(applicable_rate).bind(rebate_amount).bind(tier_id)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<RebateTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.rebate_transactions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_transaction))
    }

    async fn get_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<RebateTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.rebate_transactions WHERE organization_id = $1 AND transaction_number = $2"
        ).bind(org_id).bind(transaction_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_transaction))
    }

    async fn list_transactions(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateTransaction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.rebate_transactions
               WHERE agreement_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY transaction_date DESC, created_at DESC"#,
        ).bind(agreement_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_transaction).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<RebateTransaction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.rebate_transactions SET status = $2, excluded_reason = COALESCE($3, excluded_reason), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Transaction {} not found", id)))?;
        Ok(row_to_transaction(&row))
    }

    async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.rebate_transactions WHERE organization_id = $1 AND transaction_number = $2"
        ).bind(org_id).bind(transaction_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Transaction '{}' not found", transaction_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Accruals
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_accrual(
        &self, org_id: Uuid, agreement_id: Uuid, accrual_number: &str,
        accrual_date: chrono::NaiveDate, accrual_period: Option<&str>,
        accumulated_quantity: f64, accumulated_amount: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, accrued_amount: f64,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAccrual> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_accruals
                (organization_id, agreement_id, accrual_number, accrual_date, accrual_period,
                 accumulated_quantity, accumulated_amount,
                 applicable_tier_id, applicable_rate, accrued_amount,
                 currency_code, notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}'::jsonb, $13)
            RETURNING *"#,
        )
        .bind(org_id).bind(agreement_id).bind(accrual_number).bind(accrual_date).bind(accrual_period)
        .bind(accumulated_quantity).bind(accumulated_amount)
        .bind(applicable_tier_id).bind(applicable_rate).bind(accrued_amount)
        .bind(currency_code).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_accrual(&row))
    }

    async fn get_accrual(&self, id: Uuid) -> AtlasResult<Option<RebateAccrual>> {
        let row = sqlx::query("SELECT * FROM _atlas.rebate_accruals WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_accrual))
    }

    async fn get_accrual_by_number(&self, org_id: Uuid, accrual_number: &str) -> AtlasResult<Option<RebateAccrual>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.rebate_accruals WHERE organization_id = $1 AND accrual_number = $2"
        ).bind(org_id).bind(accrual_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_accrual))
    }

    async fn list_accruals(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateAccrual>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.rebate_accruals
               WHERE agreement_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY accrual_date DESC"#,
        ).bind(agreement_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_accrual).collect())
    }

    async fn update_accrual_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateAccrual> {
        let row = sqlx::query(
            "UPDATE _atlas.rebate_accruals SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Accrual {} not found", id)))?;
        Ok(row_to_accrual(&row))
    }

    async fn delete_accrual(&self, org_id: Uuid, accrual_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.rebate_accruals WHERE organization_id = $1 AND accrual_number = $2"
        ).bind(org_id).bind(accrual_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Accrual '{}' not found", accrual_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Settlements
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_settlement(
        &self, org_id: Uuid, agreement_id: Uuid, settlement_number: &str,
        settlement_date: chrono::NaiveDate,
        settlement_period_from: Option<chrono::NaiveDate>,
        settlement_period_to: Option<chrono::NaiveDate>,
        total_qualifying_amount: f64, total_qualifying_quantity: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, settlement_amount: f64,
        currency_code: &str, settlement_type: &str, payment_method: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RebateSettlement> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_settlements
                (organization_id, agreement_id, settlement_number, settlement_date,
                 settlement_period_from, settlement_period_to,
                 total_qualifying_amount, total_qualifying_quantity,
                 applicable_tier_id, applicable_rate, settlement_amount,
                 currency_code, settlement_type, payment_method,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, '{}'::jsonb, $16)
            RETURNING *"#,
        )
        .bind(org_id).bind(agreement_id).bind(settlement_number).bind(settlement_date)
        .bind(settlement_period_from).bind(settlement_period_to)
        .bind(total_qualifying_amount).bind(total_qualifying_quantity)
        .bind(applicable_tier_id).bind(applicable_rate).bind(settlement_amount)
        .bind(currency_code).bind(settlement_type).bind(payment_method)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_settlement(&row))
    }

    async fn get_settlement(&self, id: Uuid) -> AtlasResult<Option<RebateSettlement>> {
        let row = sqlx::query("SELECT * FROM _atlas.rebate_settlements WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_settlement))
    }

    async fn get_settlement_by_number(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<Option<RebateSettlement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.rebate_settlements WHERE organization_id = $1 AND settlement_number = $2"
        ).bind(org_id).bind(settlement_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_settlement))
    }

    async fn list_settlements(&self, agreement_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<RebateSettlement>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.rebate_settlements
               WHERE agreement_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY settlement_date DESC"#,
        ).bind(agreement_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_settlement).collect())
    }

    async fn update_settlement_status(&self, id: Uuid, status: &str) -> AtlasResult<RebateSettlement> {
        let row = sqlx::query(
            "UPDATE _atlas.rebate_settlements SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Settlement {} not found", id)))?;
        Ok(row_to_settlement(&row))
    }

    async fn approve_settlement(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<RebateSettlement> {
        let row = sqlx::query(
            r#"UPDATE _atlas.rebate_settlements
               SET status = 'approved', approved_by = $2, approved_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Settlement {} not found", id)))?;
        Ok(row_to_settlement(&row))
    }

    async fn pay_settlement(&self, id: Uuid) -> AtlasResult<RebateSettlement> {
        let row = sqlx::query(
            r#"UPDATE _atlas.rebate_settlements
               SET status = 'paid', paid_at = now(), updated_at = now()
               WHERE id = $1 AND status = 'approved' RETURNING *"#,
        ).bind(id)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::ValidationFailed("Settlement not found or not approved".to_string()))?;
        Ok(row_to_settlement(&row))
    }

    async fn delete_settlement(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.rebate_settlements WHERE organization_id = $1 AND settlement_number = $2 AND status = 'pending'"
        ).bind(org_id).bind(settlement_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Pending settlement '{}' not found", settlement_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Settlement Lines
    // ========================================================================

    async fn create_settlement_line(&self, settlement_id: Uuid, transaction_id: Uuid, amount: f64) -> AtlasResult<RebateSettlementLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.rebate_settlement_lines (settlement_id, transaction_id, settlement_amount, metadata)
               VALUES ($1, $2, $3, '{}'::jsonb) RETURNING *"#,
        ).bind(settlement_id).bind(transaction_id).bind(amount)
        .fetch_one(&self.pool).await?;
        Ok(row_to_settlement_line(&row))
    }

    async fn list_settlement_lines(&self, settlement_id: Uuid) -> AtlasResult<Vec<RebateSettlementLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.rebate_settlement_lines WHERE settlement_id = $1 ORDER BY created_at"
        ).bind(settlement_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_settlement_line).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RebateDashboard> {
        let agreements = sqlx::query(
            "SELECT status, rebate_type FROM _atlas.rebate_agreements WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_agreements = agreements.len() as i64;
        let active_agreements = agreements.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i64;

        let mut by_type = serde_json::Map::new();
        for row in &agreements {
            let rt = row.try_get::<String, _>("rebate_type").unwrap_or_default();
            let count = by_type.entry(rt).or_insert(serde_json::Value::from(0));
            *count = serde_json::Value::from(count.as_i64().unwrap_or(0) + 1);
        }

        let txn_stats = sqlx::query(
            r#"SELECT COUNT(*) as cnt, COALESCE(SUM(transaction_amount), 0) as total_amount
               FROM _atlas.rebate_transactions WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_transactions: i64 = txn_stats.try_get("cnt").unwrap_or(0);
        let total_qualifying_amount: f64 = get_numeric(&txn_stats, "total_amount");

        let acc_stats = sqlx::query(
            r#"SELECT COALESCE(SUM(accrued_amount), 0) as total_accrued FROM _atlas.rebate_accruals
               WHERE organization_id = $1 AND status IN ('posted', 'settled')"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_accrued_amount: f64 = get_numeric(&acc_stats, "total_accrued");

        let set_stats = sqlx::query(
            r#"SELECT COALESCE(SUM(settlement_amount), 0) as total_settled,
                      COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_count
               FROM _atlas.rebate_settlements WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_settled_amount: f64 = get_numeric(&set_stats, "total_settled");
        let pending_settlements: i64 = set_stats.try_get("pending_count").unwrap_or(0);

        Ok(RebateDashboard {
            organization_id: org_id,
            total_agreements,
            active_agreements,
            total_transactions,
            total_qualifying_amount,
            total_accrued_amount,
            total_settled_amount,
            pending_settlements,
            agreements_by_type: serde_json::Value::Object(by_type),
            top_rebate_agreements: serde_json::json!([]),
            recent_settlements: serde_json::json!([]),
        })
    }
}
