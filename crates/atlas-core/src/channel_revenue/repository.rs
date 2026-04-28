//! Channel Revenue Management Repository
//!
//! PostgreSQL storage for trade promotions, promotion lines, funds,
//! claims, settlements, and dashboard analytics.

use atlas_shared::{
    TradePromotion, TradePromotionLine, PromotionFund,
    TradeClaim, TradeSettlement, ChannelRevenueDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for channel revenue data storage
#[async_trait]
pub trait ChannelRevenueRepository: Send + Sync {
    // ========================================================================
    // Trade Promotions
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_promotion(
        &self,
        org_id: Uuid, promotion_number: &str, name: &str, description: Option<&str>,
        promotion_type: &str, status: &str, priority: Option<&str>,
        category: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        fund_id: Option<Uuid>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        sell_in_start_date: Option<chrono::NaiveDate>, sell_in_end_date: Option<chrono::NaiveDate>,
        sell_out_start_date: Option<chrono::NaiveDate>, sell_out_end_date: Option<chrono::NaiveDate>,
        product_category: Option<&str>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        customer_segment: Option<&str>, territory: Option<&str>,
        expected_revenue: f64, planned_budget: f64,
        currency_code: &str,
        discount_pct: Option<f64>, discount_amount: Option<f64>,
        volume_threshold: Option<f64>, volume_uom: Option<&str>,
        tier_config: serde_json::Value,
        objectives: Option<&str>, terms_and_conditions: Option<&str>,
        approval_status: &str,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotion>;

    async fn get_promotion(&self, id: Uuid) -> AtlasResult<Option<TradePromotion>>;
    async fn get_promotion_by_number(&self, org_id: Uuid, promotion_number: &str) -> AtlasResult<Option<TradePromotion>>;
    async fn list_promotions(
        &self, org_id: Uuid, status: Option<&str>, promotion_type: Option<&str>,
        partner_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<TradePromotion>>;
    async fn update_promotion_status(&self, id: Uuid, status: &str) -> AtlasResult<TradePromotion>;
    async fn update_promotion_approval(
        &self, id: Uuid, approval_status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotion>;
    async fn update_promotion_spend(
        &self, id: Uuid, actual_spend: f64, accrued_amount: f64,
    ) -> AtlasResult<TradePromotion>;
    async fn delete_promotion(&self, org_id: Uuid, promotion_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Trade Promotion Lines
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_promotion_line(
        &self,
        org_id: Uuid, promotion_id: Uuid, line_number: i32,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        product_category: Option<&str>,
        discount_type: &str, discount_value: f64,
        unit_of_measure: Option<&str>,
        quantity_from: Option<f64>, quantity_to: Option<f64>,
        planned_quantity: f64, planned_amount: f64,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotionLine>;

    async fn get_promotion_line(&self, id: Uuid) -> AtlasResult<Option<TradePromotionLine>>;
    async fn list_promotion_lines(&self, promotion_id: Uuid) -> AtlasResult<Vec<TradePromotionLine>>;
    async fn update_promotion_line_actuals(
        &self, id: Uuid, actual_quantity: f64, actual_amount: f64, accrual_amount: f64,
    ) -> AtlasResult<TradePromotionLine>;
    async fn delete_promotion_line(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Promotion Funds
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_fund(
        &self,
        org_id: Uuid, fund_number: &str, name: &str, description: Option<&str>,
        fund_type: &str, status: &str,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        total_budget: f64, currency_code: &str,
        fund_year: Option<i32>, fund_quarter: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromotionFund>;

    async fn get_fund(&self, id: Uuid) -> AtlasResult<Option<PromotionFund>>;
    async fn get_fund_by_number(&self, org_id: Uuid, fund_number: &str) -> AtlasResult<Option<PromotionFund>>;
    async fn list_funds(
        &self, org_id: Uuid, status: Option<&str>, fund_type: Option<&str>,
    ) -> AtlasResult<Vec<PromotionFund>>;
    async fn update_fund_status(&self, id: Uuid, status: &str) -> AtlasResult<PromotionFund>;
    async fn update_fund_budget(
        &self, id: Uuid, total_budget: f64,
    ) -> AtlasResult<PromotionFund>;
    async fn update_fund_utilization(
        &self, id: Uuid, allocated_amount: f64, committed_amount: f64,
        utilized_amount: f64, available_amount: f64,
    ) -> AtlasResult<PromotionFund>;
    async fn delete_fund(&self, org_id: Uuid, fund_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Trade Claims
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_claim(
        &self,
        org_id: Uuid, claim_number: &str,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        fund_id: Option<Uuid>, fund_number: Option<&str>,
        claim_type: &str, status: &str, priority: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        claim_date: chrono::NaiveDate,
        sell_in_from: Option<chrono::NaiveDate>, sell_in_to: Option<chrono::NaiveDate>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        quantity: f64, unit_of_measure: Option<&str>, unit_price: Option<f64>,
        claimed_amount: f64, currency_code: &str,
        invoice_number: Option<&str>, invoice_date: Option<chrono::NaiveDate>,
        reference_document: Option<&str>,
        proof_of_performance: serde_json::Value,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradeClaim>;

    async fn get_claim(&self, id: Uuid) -> AtlasResult<Option<TradeClaim>>;
    async fn get_claim_by_number(&self, org_id: Uuid, claim_number: &str) -> AtlasResult<Option<TradeClaim>>;
    async fn list_claims(
        &self, org_id: Uuid, status: Option<&str>, claim_type: Option<&str>,
        promotion_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<TradeClaim>>;
    async fn update_claim_status(
        &self, id: Uuid, status: &str,
        approved_amount: Option<f64>,
        rejection_reason: Option<&str>,
        resolution_notes: Option<&str>,
    ) -> AtlasResult<TradeClaim>;
    async fn update_claim_payment(
        &self, id: Uuid, paid_amount: f64,
    ) -> AtlasResult<TradeClaim>;
    async fn delete_claim(&self, org_id: Uuid, claim_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Trade Settlements
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_settlement(
        &self,
        org_id: Uuid, settlement_number: &str,
        claim_id: Option<Uuid>, claim_number: Option<&str>,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        settlement_type: &str, status: &str,
        settlement_date: chrono::NaiveDate, settlement_amount: f64,
        currency_code: &str,
        payment_method: Option<&str>, payment_reference: Option<&str>,
        bank_account: Option<&str>, gl_account: Option<&str>, cost_center: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradeSettlement>;

    async fn get_settlement(&self, id: Uuid) -> AtlasResult<Option<TradeSettlement>>;
    async fn get_settlement_by_number(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<Option<TradeSettlement>>;
    async fn list_settlements(
        &self, org_id: Uuid, status: Option<&str>, settlement_type: Option<&str>,
    ) -> AtlasResult<Vec<TradeSettlement>>;
    async fn update_settlement_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<TradeSettlement>;
    async fn delete_settlement(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Dashboard
    // ========================================================================
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ChannelRevenueDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresChannelRevenueRepository {
    pool: PgPool,
}

impl PostgresChannelRevenueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row mappers
fn row_to_promotion(row: &sqlx::postgres::PgRow) -> TradePromotion {
    TradePromotion {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        promotion_number: row.try_get("promotion_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        promotion_type: row.try_get("promotion_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        partner_id: row.try_get("partner_id").unwrap_or_default(),
        partner_number: row.try_get("partner_number").unwrap_or_default(),
        partner_name: row.try_get("partner_name").unwrap_or_default(),
        fund_id: row.try_get("fund_id").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: row.try_get("end_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        sell_in_start_date: row.try_get("sell_in_start_date").unwrap_or_default(),
        sell_in_end_date: row.try_get("sell_in_end_date").unwrap_or_default(),
        sell_out_start_date: row.try_get("sell_out_start_date").unwrap_or_default(),
        sell_out_end_date: row.try_get("sell_out_end_date").unwrap_or_default(),
        product_category: row.try_get("product_category").unwrap_or_default(),
        product_id: row.try_get("product_id").unwrap_or_default(),
        product_number: row.try_get("product_number").unwrap_or_default(),
        product_name: row.try_get("product_name").unwrap_or_default(),
        customer_segment: row.try_get("customer_segment").unwrap_or_default(),
        territory: row.try_get("territory").unwrap_or_default(),
        expected_revenue: row.try_get("expected_revenue").unwrap_or(0.0),
        planned_budget: row.try_get("planned_budget").unwrap_or(0.0),
        actual_spend: row.try_get("actual_spend").unwrap_or(0.0),
        accrued_amount: row.try_get("accrued_amount").unwrap_or(0.0),
        claimed_amount: row.try_get("claimed_amount").unwrap_or(0.0),
        settled_amount: row.try_get("settled_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        discount_pct: row.try_get("discount_pct").unwrap_or_default(),
        discount_amount: row.try_get("discount_amount").unwrap_or_default(),
        volume_threshold: row.try_get("volume_threshold").unwrap_or_default(),
        volume_uom: row.try_get("volume_uom").unwrap_or_default(),
        tier_config: row.try_get("tier_config").unwrap_or(serde_json::json!({})),
        objectives: row.try_get("objectives").unwrap_or_default(),
        terms_and_conditions: row.try_get("terms_and_conditions").unwrap_or_default(),
        approval_status: row.try_get("approval_status").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        effective_from: row.try_get("effective_from").unwrap_or_default(),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_promotion_line(row: &sqlx::postgres::PgRow) -> TradePromotionLine {
    TradePromotionLine {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        promotion_id: row.try_get("promotion_id").unwrap_or_default(),
        line_number: row.try_get("line_number").unwrap_or_default(),
        product_id: row.try_get("product_id").unwrap_or_default(),
        product_number: row.try_get("product_number").unwrap_or_default(),
        product_name: row.try_get("product_name").unwrap_or_default(),
        product_category: row.try_get("product_category").unwrap_or_default(),
        discount_type: row.try_get("discount_type").unwrap_or_default(),
        discount_value: row.try_get("discount_value").unwrap_or(0.0),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        quantity_from: row.try_get("quantity_from").unwrap_or_default(),
        quantity_to: row.try_get("quantity_to").unwrap_or_default(),
        planned_quantity: row.try_get("planned_quantity").unwrap_or(0.0),
        actual_quantity: row.try_get("actual_quantity").unwrap_or(0.0),
        planned_amount: row.try_get("planned_amount").unwrap_or(0.0),
        actual_amount: row.try_get("actual_amount").unwrap_or(0.0),
        accrual_amount: row.try_get("accrual_amount").unwrap_or(0.0),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_fund(row: &sqlx::postgres::PgRow) -> PromotionFund {
    PromotionFund {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        fund_number: row.try_get("fund_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        fund_type: row.try_get("fund_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        partner_id: row.try_get("partner_id").unwrap_or_default(),
        partner_number: row.try_get("partner_number").unwrap_or_default(),
        partner_name: row.try_get("partner_name").unwrap_or_default(),
        total_budget: row.try_get("total_budget").unwrap_or(0.0),
        allocated_amount: row.try_get("allocated_amount").unwrap_or(0.0),
        committed_amount: row.try_get("committed_amount").unwrap_or(0.0),
        utilized_amount: row.try_get("utilized_amount").unwrap_or(0.0),
        available_amount: row.try_get("available_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        fund_year: row.try_get("fund_year").unwrap_or_default(),
        fund_quarter: row.try_get("fund_quarter").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or_default(),
        end_date: row.try_get("end_date").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        approval_status: row.try_get("approval_status").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_claim(row: &sqlx::postgres::PgRow) -> TradeClaim {
    TradeClaim {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        claim_number: row.try_get("claim_number").unwrap_or_default(),
        promotion_id: row.try_get("promotion_id").unwrap_or_default(),
        promotion_number: row.try_get("promotion_number").unwrap_or_default(),
        fund_id: row.try_get("fund_id").unwrap_or_default(),
        fund_number: row.try_get("fund_number").unwrap_or_default(),
        claim_type: row.try_get("claim_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        partner_id: row.try_get("partner_id").unwrap_or_default(),
        partner_number: row.try_get("partner_number").unwrap_or_default(),
        partner_name: row.try_get("partner_name").unwrap_or_default(),
        claim_date: row.try_get("claim_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        sell_in_from: row.try_get("sell_in_from").unwrap_or_default(),
        sell_in_to: row.try_get("sell_in_to").unwrap_or_default(),
        product_id: row.try_get("product_id").unwrap_or_default(),
        product_number: row.try_get("product_number").unwrap_or_default(),
        product_name: row.try_get("product_name").unwrap_or_default(),
        quantity: row.try_get("quantity").unwrap_or(0.0),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        unit_price: row.try_get("unit_price").unwrap_or_default(),
        claimed_amount: row.try_get("claimed_amount").unwrap_or(0.0),
        approved_amount: row.try_get("approved_amount").unwrap_or(0.0),
        paid_amount: row.try_get("paid_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        invoice_number: row.try_get("invoice_number").unwrap_or_default(),
        invoice_date: row.try_get("invoice_date").unwrap_or_default(),
        reference_document: row.try_get("reference_document").unwrap_or_default(),
        proof_of_performance: row.try_get("proof_of_performance").unwrap_or(serde_json::json!({})),
        rejection_reason: row.try_get("rejection_reason").unwrap_or_default(),
        resolution_notes: row.try_get("resolution_notes").unwrap_or_default(),
        assigned_to: row.try_get("assigned_to").unwrap_or_default(),
        assigned_to_name: row.try_get("assigned_to_name").unwrap_or_default(),
        submitted_at: row.try_get("submitted_at").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        paid_at: row.try_get("paid_at").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_settlement(row: &sqlx::postgres::PgRow) -> TradeSettlement {
    TradeSettlement {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        settlement_number: row.try_get("settlement_number").unwrap_or_default(),
        claim_id: row.try_get("claim_id").unwrap_or_default(),
        claim_number: row.try_get("claim_number").unwrap_or_default(),
        promotion_id: row.try_get("promotion_id").unwrap_or_default(),
        promotion_number: row.try_get("promotion_number").unwrap_or_default(),
        partner_id: row.try_get("partner_id").unwrap_or_default(),
        partner_number: row.try_get("partner_number").unwrap_or_default(),
        partner_name: row.try_get("partner_name").unwrap_or_default(),
        settlement_type: row.try_get("settlement_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        settlement_date: row.try_get("settlement_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        settlement_amount: row.try_get("settlement_amount").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        payment_method: row.try_get("payment_method").unwrap_or_default(),
        payment_reference: row.try_get("payment_reference").unwrap_or_default(),
        bank_account: row.try_get("bank_account").unwrap_or_default(),
        gl_account: row.try_get("gl_account").unwrap_or_default(),
        cost_center: row.try_get("cost_center").unwrap_or_default(),
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

#[async_trait]
impl ChannelRevenueRepository for PostgresChannelRevenueRepository {
    // ========================================================================
    // Trade Promotions
    // ========================================================================

    async fn create_promotion(
        &self,
        org_id: Uuid, promotion_number: &str, name: &str, description: Option<&str>,
        promotion_type: &str, status: &str, priority: Option<&str>,
        category: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        fund_id: Option<Uuid>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        sell_in_start_date: Option<chrono::NaiveDate>, sell_in_end_date: Option<chrono::NaiveDate>,
        sell_out_start_date: Option<chrono::NaiveDate>, sell_out_end_date: Option<chrono::NaiveDate>,
        product_category: Option<&str>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        customer_segment: Option<&str>, territory: Option<&str>,
        expected_revenue: f64, planned_budget: f64,
        currency_code: &str,
        discount_pct: Option<f64>, discount_amount: Option<f64>,
        volume_threshold: Option<f64>, volume_uom: Option<&str>,
        tier_config: serde_json::Value,
        objectives: Option<&str>, terms_and_conditions: Option<&str>,
        approval_status: &str,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotion> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.trade_promotions
                (organization_id, promotion_number, name, description,
                 promotion_type, status, priority, category,
                 partner_id, partner_number, partner_name, fund_id,
                 start_date, end_date,
                 sell_in_start_date, sell_in_end_date,
                 sell_out_start_date, sell_out_end_date,
                 product_category, product_id, product_number, product_name,
                 customer_segment, territory,
                 expected_revenue, planned_budget, currency_code,
                 discount_pct, discount_amount,
                 volume_threshold, volume_uom, tier_config,
                 objectives, terms_and_conditions, approval_status,
                 owner_id, owner_name,
                 effective_from, effective_to,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,
                    $19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,
                    $35,$36,$37,$38,$39,$40)
            RETURNING *"#,
        )
        .bind(org_id).bind(promotion_number).bind(name).bind(description)
        .bind(promotion_type).bind(status).bind(priority).bind(category)
        .bind(partner_id).bind(partner_number).bind(partner_name).bind(fund_id)
        .bind(start_date).bind(end_date)
        .bind(sell_in_start_date).bind(sell_in_end_date)
        .bind(sell_out_start_date).bind(sell_out_end_date)
        .bind(product_category).bind(product_id).bind(product_number).bind(product_name)
        .bind(customer_segment).bind(territory)
        .bind(expected_revenue).bind(planned_budget).bind(currency_code)
        .bind(discount_pct).bind(discount_amount)
        .bind(volume_threshold).bind(volume_uom).bind(&tier_config)
        .bind(objectives).bind(terms_and_conditions).bind(approval_status)
        .bind(owner_id).bind(owner_name)
        .bind(effective_from).bind(effective_to)
        .bind(serde_json::json!({})).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_promotion(&row))
    }

    async fn get_promotion(&self, id: Uuid) -> AtlasResult<Option<TradePromotion>> {
        let row = sqlx::query("SELECT * FROM _atlas.trade_promotions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_promotion))
    }

    async fn get_promotion_by_number(&self, org_id: Uuid, promotion_number: &str) -> AtlasResult<Option<TradePromotion>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.trade_promotions WHERE organization_id = $1 AND promotion_number = $2"
        ).bind(org_id).bind(promotion_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_promotion))
    }

    async fn list_promotions(
        &self, org_id: Uuid, status: Option<&str>, promotion_type: Option<&str>,
        partner_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<TradePromotion>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.trade_promotions
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR promotion_type = $3)
                 AND ($4::uuid IS NULL OR partner_id = $4)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(promotion_type).bind(partner_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_promotion).collect())
    }

    async fn update_promotion_status(&self, id: Uuid, status: &str) -> AtlasResult<TradePromotion> {
        let row = sqlx::query(
            "UPDATE _atlas.trade_promotions SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        Ok(row_to_promotion(&row))
    }

    async fn update_promotion_approval(
        &self, id: Uuid, approval_status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotion> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_promotions
               SET approval_status = $2, approved_by = $3,
                   approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(approval_status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        Ok(row_to_promotion(&row))
    }

    async fn update_promotion_spend(
        &self, id: Uuid, actual_spend: f64, accrued_amount: f64,
    ) -> AtlasResult<TradePromotion> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_promotions
               SET actual_spend = $2, accrued_amount = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(actual_spend).bind(accrued_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        Ok(row_to_promotion(&row))
    }

    async fn delete_promotion(&self, org_id: Uuid, promotion_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.trade_promotions WHERE organization_id = $1 AND promotion_number = $2"
        ).bind(org_id).bind(promotion_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Promotion '{}' not found", promotion_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Trade Promotion Lines
    // ========================================================================

    async fn create_promotion_line(
        &self,
        org_id: Uuid, promotion_id: Uuid, line_number: i32,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        product_category: Option<&str>,
        discount_type: &str, discount_value: f64,
        unit_of_measure: Option<&str>,
        quantity_from: Option<f64>, quantity_to: Option<f64>,
        planned_quantity: f64, planned_amount: f64,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradePromotionLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.trade_promotion_lines
                (organization_id, promotion_id, line_number,
                 product_id, product_number, product_name, product_category,
                 discount_type, discount_value, unit_of_measure,
                 quantity_from, quantity_to, planned_quantity, planned_amount,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'{}'::jsonb,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(promotion_id).bind(line_number)
        .bind(product_id).bind(product_number).bind(product_name).bind(product_category)
        .bind(discount_type).bind(discount_value).bind(unit_of_measure)
        .bind(quantity_from).bind(quantity_to).bind(planned_quantity).bind(planned_amount)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_promotion_line(&row))
    }

    async fn get_promotion_line(&self, id: Uuid) -> AtlasResult<Option<TradePromotionLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.trade_promotion_lines WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_promotion_line))
    }

    async fn list_promotion_lines(&self, promotion_id: Uuid) -> AtlasResult<Vec<TradePromotionLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.trade_promotion_lines WHERE promotion_id = $1 ORDER BY line_number"
        ).bind(promotion_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_promotion_line).collect())
    }

    async fn update_promotion_line_actuals(
        &self, id: Uuid, actual_quantity: f64, actual_amount: f64, accrual_amount: f64,
    ) -> AtlasResult<TradePromotionLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_promotion_lines
               SET actual_quantity = $2, actual_amount = $3, accrual_amount = $4, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(actual_quantity).bind(actual_amount).bind(accrual_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Promotion line {} not found", id)))?;
        Ok(row_to_promotion_line(&row))
    }

    async fn delete_promotion_line(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.trade_promotion_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Promotion line not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Promotion Funds
    // ========================================================================

    async fn create_fund(
        &self,
        org_id: Uuid, fund_number: &str, name: &str, description: Option<&str>,
        fund_type: &str, status: &str,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        total_budget: f64, currency_code: &str,
        fund_year: Option<i32>, fund_quarter: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromotionFund> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.promotion_funds
                (organization_id, fund_number, name, description,
                 fund_type, status,
                 partner_id, partner_number, partner_name,
                 total_budget, available_amount, currency_code,
                 fund_year, fund_quarter, start_date, end_date,
                 owner_id, owner_name, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10,$11,$12,$13,$14,$15,$16,$17,'{}'::jsonb,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(fund_number).bind(name).bind(description)
        .bind(fund_type).bind(status)
        .bind(partner_id).bind(partner_number).bind(partner_name)
        .bind(total_budget).bind(currency_code)
        .bind(fund_year).bind(fund_quarter).bind(start_date).bind(end_date)
        .bind(owner_id).bind(owner_name).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_fund(&row))
    }

    async fn get_fund(&self, id: Uuid) -> AtlasResult<Option<PromotionFund>> {
        let row = sqlx::query("SELECT * FROM _atlas.promotion_funds WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_fund))
    }

    async fn get_fund_by_number(&self, org_id: Uuid, fund_number: &str) -> AtlasResult<Option<PromotionFund>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.promotion_funds WHERE organization_id = $1 AND fund_number = $2"
        ).bind(org_id).bind(fund_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_fund))
    }

    async fn list_funds(
        &self, org_id: Uuid, status: Option<&str>, fund_type: Option<&str>,
    ) -> AtlasResult<Vec<PromotionFund>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.promotion_funds
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR fund_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(fund_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_fund).collect())
    }

    async fn update_fund_status(&self, id: Uuid, status: &str) -> AtlasResult<PromotionFund> {
        let row = sqlx::query(
            "UPDATE _atlas.promotion_funds SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Fund {} not found", id)))?;
        Ok(row_to_fund(&row))
    }

    async fn update_fund_budget(&self, id: Uuid, total_budget: f64) -> AtlasResult<PromotionFund> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_funds
               SET total_budget = $2, available_amount = $2 - allocated_amount,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(total_budget)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Fund {} not found", id)))?;
        Ok(row_to_fund(&row))
    }

    async fn update_fund_utilization(
        &self, id: Uuid, allocated_amount: f64, committed_amount: f64,
        utilized_amount: f64, available_amount: f64,
    ) -> AtlasResult<PromotionFund> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_funds
               SET allocated_amount = $2, committed_amount = $3,
                   utilized_amount = $4, available_amount = $5,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(allocated_amount).bind(committed_amount)
        .bind(utilized_amount).bind(available_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Fund {} not found", id)))?;
        Ok(row_to_fund(&row))
    }

    async fn delete_fund(&self, org_id: Uuid, fund_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.promotion_funds WHERE organization_id = $1 AND fund_number = $2"
        ).bind(org_id).bind(fund_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Fund '{}' not found", fund_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Trade Claims
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_claim(
        &self,
        org_id: Uuid, claim_number: &str,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        fund_id: Option<Uuid>, fund_number: Option<&str>,
        claim_type: &str, status: &str, priority: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        claim_date: chrono::NaiveDate,
        sell_in_from: Option<chrono::NaiveDate>, sell_in_to: Option<chrono::NaiveDate>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        quantity: f64, unit_of_measure: Option<&str>, unit_price: Option<f64>,
        claimed_amount: f64, currency_code: &str,
        invoice_number: Option<&str>, invoice_date: Option<chrono::NaiveDate>,
        reference_document: Option<&str>,
        proof_of_performance: serde_json::Value,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradeClaim> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.trade_claims
                (organization_id, claim_number,
                 promotion_id, promotion_number, fund_id, fund_number,
                 claim_type, status, priority,
                 partner_id, partner_number, partner_name,
                 claim_date, sell_in_from, sell_in_to,
                 product_id, product_number, product_name,
                 quantity, unit_of_measure, unit_price,
                 claimed_amount, currency_code,
                 invoice_number, invoice_date, reference_document,
                 proof_of_performance,
                 assigned_to, assigned_to_name,
                 metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,
                    $16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,'{}'::jsonb,$31)
            RETURNING *"#,
        )
        .bind(org_id).bind(claim_number)
        .bind(promotion_id).bind(promotion_number).bind(fund_id).bind(fund_number)
        .bind(claim_type).bind(status).bind(priority)
        .bind(partner_id).bind(partner_number).bind(partner_name)
        .bind(claim_date).bind(sell_in_from).bind(sell_in_to)
        .bind(product_id).bind(product_number).bind(product_name)
        .bind(quantity).bind(unit_of_measure).bind(unit_price)
        .bind(claimed_amount).bind(currency_code)
        .bind(invoice_number).bind(invoice_date).bind(reference_document)
        .bind(&proof_of_performance)
        .bind(assigned_to).bind(assigned_to_name)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_claim(&row))
    }

    async fn get_claim(&self, id: Uuid) -> AtlasResult<Option<TradeClaim>> {
        let row = sqlx::query("SELECT * FROM _atlas.trade_claims WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_claim))
    }

    async fn get_claim_by_number(&self, org_id: Uuid, claim_number: &str) -> AtlasResult<Option<TradeClaim>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.trade_claims WHERE organization_id = $1 AND claim_number = $2"
        ).bind(org_id).bind(claim_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_claim))
    }

    async fn list_claims(
        &self, org_id: Uuid, status: Option<&str>, claim_type: Option<&str>,
        promotion_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<TradeClaim>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.trade_claims
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR claim_type = $3)
                 AND ($4::uuid IS NULL OR promotion_id = $4)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(claim_type).bind(promotion_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_claim).collect())
    }

    async fn update_claim_status(
        &self, id: Uuid, status: &str,
        approved_amount: Option<f64>,
        rejection_reason: Option<&str>,
        resolution_notes: Option<&str>,
    ) -> AtlasResult<TradeClaim> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_claims
               SET status = $2,
                   approved_amount = COALESCE($3, approved_amount),
                   rejection_reason = COALESCE($4, rejection_reason),
                   resolution_notes = COALESCE($5, resolution_notes),
                   submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                   approved_at = CASE WHEN $2 IN ('approved','partially_approved') THEN now() ELSE approved_at END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .bind(approved_amount).bind(rejection_reason).bind(resolution_notes)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        Ok(row_to_claim(&row))
    }

    async fn update_claim_payment(
        &self, id: Uuid, paid_amount: f64,
    ) -> AtlasResult<TradeClaim> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_claims
               SET paid_amount = $2,
                   paid_at = CASE WHEN $2 >= claimed_amount THEN now() ELSE paid_at END,
                   status = CASE WHEN $2 >= claimed_amount THEN 'paid' ELSE status END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(paid_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        Ok(row_to_claim(&row))
    }

    async fn delete_claim(&self, org_id: Uuid, claim_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.trade_claims WHERE organization_id = $1 AND claim_number = $2"
        ).bind(org_id).bind(claim_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Claim '{}' not found", claim_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Trade Settlements
    // ========================================================================

    async fn create_settlement(
        &self,
        org_id: Uuid, settlement_number: &str,
        claim_id: Option<Uuid>, claim_number: Option<&str>,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        settlement_type: &str, status: &str,
        settlement_date: chrono::NaiveDate, settlement_amount: f64,
        currency_code: &str,
        payment_method: Option<&str>, payment_reference: Option<&str>,
        bank_account: Option<&str>, gl_account: Option<&str>, cost_center: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TradeSettlement> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.trade_settlements
                (organization_id, settlement_number,
                 claim_id, claim_number, promotion_id, promotion_number,
                 partner_id, partner_number, partner_name,
                 settlement_type, status,
                 settlement_date, settlement_amount, currency_code,
                 payment_method, payment_reference,
                 bank_account, gl_account, cost_center,
                 notes, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,'{}'::jsonb,$21)
            RETURNING *"#,
        )
        .bind(org_id).bind(settlement_number)
        .bind(claim_id).bind(claim_number).bind(promotion_id).bind(promotion_number)
        .bind(partner_id).bind(partner_number).bind(partner_name)
        .bind(settlement_type).bind(status)
        .bind(settlement_date).bind(settlement_amount).bind(currency_code)
        .bind(payment_method).bind(payment_reference)
        .bind(bank_account).bind(gl_account).bind(cost_center)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_settlement(&row))
    }

    async fn get_settlement(&self, id: Uuid) -> AtlasResult<Option<TradeSettlement>> {
        let row = sqlx::query("SELECT * FROM _atlas.trade_settlements WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_settlement))
    }

    async fn get_settlement_by_number(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<Option<TradeSettlement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.trade_settlements WHERE organization_id = $1 AND settlement_number = $2"
        ).bind(org_id).bind(settlement_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_settlement))
    }

    async fn list_settlements(
        &self, org_id: Uuid, status: Option<&str>, settlement_type: Option<&str>,
    ) -> AtlasResult<Vec<TradeSettlement>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.trade_settlements
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR settlement_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(settlement_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_settlement).collect())
    }

    async fn update_settlement_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
    ) -> AtlasResult<TradeSettlement> {
        let row = sqlx::query(
            r#"UPDATE _atlas.trade_settlements
               SET status = $2,
                   approved_by = COALESCE($3, approved_by),
                   approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                   paid_at = CASE WHEN $2 = 'completed' THEN now() ELSE paid_at END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Settlement {} not found", id)))?;
        Ok(row_to_settlement(&row))
    }

    async fn delete_settlement(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.trade_settlements WHERE organization_id = $1 AND settlement_number = $2"
        ).bind(org_id).bind(settlement_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Settlement '{}' not found", settlement_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ChannelRevenueDashboard> {
        // Promotions
        let promo_rows = sqlx::query(
            "SELECT status, promotion_type, planned_budget, actual_spend, expected_revenue FROM _atlas.trade_promotions WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_promotions = promo_rows.len() as i32;
        let active_promotions = promo_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;
        let total_planned_budget: f64 = promo_rows.iter()
            .map(|r| r.try_get("planned_budget").unwrap_or(0.0)).sum();
        let total_actual_spend: f64 = promo_rows.iter()
            .map(|r| r.try_get("actual_spend").unwrap_or(0.0)).sum();
        let total_expected_revenue: f64 = promo_rows.iter()
            .map(|r| r.try_get("expected_revenue").unwrap_or(0.0)).sum();

        let mut by_status = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();
        for row in &promo_rows {
            let s: String = row.try_get("status").unwrap_or_default();
            let t: String = row.try_get("promotion_type").unwrap_or_default();
            *by_status.entry(s).or_insert(0i32) += 1;
            *by_type.entry(t).or_insert(0i32) += 1;
        }

        // Claims
        let claim_rows = sqlx::query(
            "SELECT status, claimed_amount, approved_amount, paid_amount FROM _atlas.trade_claims WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_claims = claim_rows.len() as i32;
        let pending_claims = claim_rows.iter()
            .filter(|r| {
                let s: String = r.try_get("status").unwrap_or_default();
                s == "draft" || s == "submitted" || s == "under_review"
            }).count() as i32;
        let approved_claims = claim_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "approved")
            .count() as i32;
        let rejected_claims = claim_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "rejected")
            .count() as i32;
        let total_claimed_amount: f64 = claim_rows.iter().map(|r| r.try_get("claimed_amount").unwrap_or(0.0)).sum();
        let total_approved_amount: f64 = claim_rows.iter().map(|r| r.try_get("approved_amount").unwrap_or(0.0)).sum();
        let total_paid_amount: f64 = claim_rows.iter().map(|r| r.try_get("paid_amount").unwrap_or(0.0)).sum();

        let mut claims_by_status = std::collections::HashMap::new();
        for row in &claim_rows {
            let s: String = row.try_get("status").unwrap_or_default();
            *claims_by_status.entry(s).or_insert(0i32) += 1;
        }

        // Funds
        let fund_rows = sqlx::query(
            "SELECT status, total_budget, utilized_amount FROM _atlas.promotion_funds WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_funds = fund_rows.len() as i32;
        let active_funds = fund_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;
        let total_fund_budget: f64 = fund_rows.iter().map(|r| r.try_get("total_budget").unwrap_or(0.0)).sum();
        let total_fund_utilized: f64 = fund_rows.iter().map(|r| r.try_get("utilized_amount").unwrap_or(0.0)).sum();

        // Settlements
        let sett_rows = sqlx::query(
            "SELECT status, settlement_amount FROM _atlas.trade_settlements WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_settlements = sett_rows.len() as i32;
        let pending_settlements = sett_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "pending")
            .count() as i32;
        let completed_settlements = sett_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "completed")
            .count() as i32;
        let total_settlement_amount: f64 = sett_rows.iter().map(|r| r.try_get("settlement_amount").unwrap_or(0.0)).sum();

        let budget_utilization = if total_planned_budget > 0.0 {
            (total_actual_spend / total_planned_budget) * 100.0
        } else { 0.0 };

        let roi = if total_actual_spend > 0.0 {
            ((total_expected_revenue - total_actual_spend) / total_actual_spend) * 100.0
        } else { 0.0 };

        let fund_utilization = if total_fund_budget > 0.0 {
            (total_fund_utilized / total_fund_budget) * 100.0
        } else { 0.0 };

        Ok(ChannelRevenueDashboard {
            total_promotions,
            active_promotions,
            total_planned_budget,
            total_actual_spend,
            budget_utilization_pct: budget_utilization,
            total_expected_revenue,
            roi_pct: roi,
            total_claims,
            pending_claims,
            approved_claims,
            rejected_claims,
            total_claimed_amount,
            total_approved_amount,
            total_paid_amount,
            total_funds,
            active_funds,
            total_fund_budget,
            total_fund_utilized,
            fund_utilization_pct: fund_utilization,
            total_settlements,
            pending_settlements,
            completed_settlements,
            total_settlement_amount,
            promotions_by_status: serde_json::to_value(by_status).unwrap_or(serde_json::json!({})),
            promotions_by_type: serde_json::to_value(by_type).unwrap_or(serde_json::json!({})),
            claims_by_status: serde_json::to_value(claims_by_status).unwrap_or(serde_json::json!({})),
            spend_trend: serde_json::json!([]),
            top_partners: serde_json::json!([]),
        })
    }
}
