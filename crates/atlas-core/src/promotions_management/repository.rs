//! Promotions Management Repository
//!
//! PostgreSQL storage for promotions data.

use atlas_shared::{
    PromoMgmtPromotion, PromoMgmtOffer, PromoMgmtFund, PromoMgmtClaim, PromoMgmtDashboard,
    AtlasResult, AtlasError,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for promotions data storage
#[async_trait]
pub trait PromotionsManagementRepository: Send + Sync {
    // Promotion CRUD
    async fn create_promotion(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        promotion_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        territory_id: Option<Uuid>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        budget_amount: &str,
        currency_code: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtPromotion>;
    async fn get_promotion(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtPromotion>>;
    async fn get_promotion_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PromoMgmtPromotion>>;
    async fn list_promotions(
        &self,
        org_id: Uuid,
        promotion_type: Option<&str>,
        status: Option<&str>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<PromoMgmtPromotion>>;
    async fn update_promotion(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        budget_amount: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
    ) -> AtlasResult<PromoMgmtPromotion>;
    async fn update_promotion_status(&self, id: Uuid, status: &str) -> AtlasResult<PromoMgmtPromotion>;
    async fn update_promotion_spent(&self, id: Uuid, spent_amount: &str) -> AtlasResult<PromoMgmtPromotion>;
    async fn delete_promotion(&self, id: Uuid) -> AtlasResult<()>;

    // Offers
    async fn create_offer(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        offer_type: &str,
        description: Option<&str>,
        discount_type: &str,
        discount_value: &str,
        buy_quantity: Option<i32>,
        get_quantity: Option<i32>,
        minimum_purchase: Option<&str>,
        maximum_discount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtOffer>;
    async fn list_offers(&self, promotion_id: Uuid) -> AtlasResult<Vec<PromoMgmtOffer>>;
    async fn get_offer(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtOffer>>;
    async fn delete_offer(&self, id: Uuid) -> AtlasResult<()>;

    // Funds
    async fn create_fund(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        fund_type: &str,
        allocated_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtFund>;
    async fn list_funds(&self, promotion_id: Uuid) -> AtlasResult<Vec<PromoMgmtFund>>;
    async fn update_fund_committed(&self, id: Uuid, committed_amount: &str) -> AtlasResult<PromoMgmtFund>;
    async fn update_fund_spent(&self, id: Uuid, spent_amount: &str) -> AtlasResult<PromoMgmtFund>;
    async fn delete_fund(&self, id: Uuid) -> AtlasResult<()>;

    // Claims
    async fn create_claim(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        claim_number: &str,
        claim_type: &str,
        amount: &str,
        currency_code: &str,
        claim_date: chrono::NaiveDate,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtClaim>;
    async fn get_claim(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtClaim>>;
    async fn list_claims(&self, promotion_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PromoMgmtClaim>>;
    async fn update_claim_status(&self, id: Uuid, status: &str, approved_amount: Option<&str>, rejection_reason: Option<&str>) -> AtlasResult<PromoMgmtClaim>;
    async fn settle_claim(&self, id: Uuid, paid_amount: &str, settlement_date: chrono::NaiveDate) -> AtlasResult<PromoMgmtClaim>;
    async fn delete_claim(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PromoMgmtDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresPromotionsManagementRepository {
    pool: PgPool,
}

impl PostgresPromotionsManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PromotionsManagementRepository for PostgresPromotionsManagementRepository {
    async fn create_promotion(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        promotion_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        territory_id: Option<Uuid>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        budget_amount: &str,
        currency_code: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtPromotion> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.promotions
                (organization_id, code, name, description, promotion_type,
                 start_date, end_date, customer_id, customer_name, territory_id,
                 product_id, product_name, budget_amount, currency_code,
                 owner_id, owner_name, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
               RETURNING id, organization_id, code, name, description, promotion_type, status,
                 start_date, end_date, customer_id, customer_name, territory_id,
                 product_id, product_name,
                 budget_amount::text as budget_amount, spent_amount::text as spent_amount,
                 currency_code, owner_id, owner_name, is_active, metadata,
                 created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(promotion_type)
        .bind(start_date).bind(end_date).bind(customer_id).bind(customer_name)
        .bind(territory_id).bind(product_id).bind(product_name)
        .bind(budget_amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code)
        .bind(owner_id).bind(owner_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_promotion(&row))
    }

    async fn get_promotion(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtPromotion>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, promotion_type, status, start_date, end_date, customer_id, customer_name, territory_id, product_id, product_name, budget_amount::text as budget_amount, spent_amount::text as spent_amount, currency_code, owner_id, owner_name, is_active, metadata, created_by, created_at, updated_at FROM _atlas.promotions WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_promo_mgmt_promotion(&r)))
    }

    async fn get_promotion_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PromoMgmtPromotion>> {
        let row = sqlx::query(
            "SELECT id, organization_id, code, name, description, promotion_type, status, start_date, end_date, customer_id, customer_name, territory_id, product_id, product_name, budget_amount::text as budget_amount, spent_amount::text as spent_amount, currency_code, owner_id, owner_name, is_active, metadata, created_by, created_at, updated_at FROM _atlas.promotions WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_promo_mgmt_promotion(&r)))
    }

    async fn list_promotions(
        &self,
        org_id: Uuid,
        promotion_type: Option<&str>,
        status: Option<&str>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<PromoMgmtPromotion>> {
        let cols = "id, organization_id, code, name, description, promotion_type, status, start_date, end_date, customer_id, customer_name, territory_id, product_id, product_name, budget_amount::text as budget_amount, spent_amount::text as spent_amount, currency_code, owner_id, owner_name, is_active, metadata, created_by, created_at, updated_at";
        let rows = match (promotion_type, status, include_inactive) {
            (Some(pt), Some(s), true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND promotion_type = $2 AND status = $3 ORDER BY name", cols))
                .bind(org_id).bind(pt).bind(s).fetch_all(&self.pool).await,
            (Some(pt), Some(s), false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND promotion_type = $2 AND status = $3 AND is_active = true ORDER BY name", cols))
                .bind(org_id).bind(pt).bind(s).fetch_all(&self.pool).await,
            (Some(pt), None, false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND promotion_type = $2 AND is_active = true ORDER BY name", cols))
                .bind(org_id).bind(pt).fetch_all(&self.pool).await,
            (Some(pt), None, true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND promotion_type = $2 ORDER BY name", cols))
                .bind(org_id).bind(pt).fetch_all(&self.pool).await,
            (None, Some(s), false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND status = $2 AND is_active = true ORDER BY name", cols))
                .bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, Some(s), true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND status = $2 ORDER BY name", cols))
                .bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None, false) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 AND is_active = true ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
            (None, None, true) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotions WHERE organization_id = $1 ORDER BY name", cols))
                .bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_promo_mgmt_promotion).collect())
    }

    async fn update_promotion(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        budget_amount: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
    ) -> AtlasResult<PromoMgmtPromotion> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotions SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                start_date = CASE WHEN $4::boolean THEN $5 ELSE start_date END,
                end_date = CASE WHEN $6::boolean THEN $7 ELSE end_date END,
                budget_amount = CASE WHEN $8::boolean THEN $9 ELSE budget_amount END,
                owner_id = CASE WHEN $10::boolean THEN $11 ELSE owner_id END,
                owner_name = COALESCE($12, owner_name),
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, code, name, description, promotion_type, status,
                 start_date, end_date, customer_id, customer_name, territory_id,
                 product_id, product_name,
                 budget_amount::text as budget_amount, spent_amount::text as spent_amount,
                 currency_code, owner_id, owner_name, is_active, metadata,
                 created_by, created_at, updated_at"#,
        )
        .bind(id).bind(name).bind(description)
        .bind(start_date.is_some()).bind(start_date)
        .bind(end_date.is_some()).bind(end_date)
        .bind(budget_amount.is_some()).bind(budget_amount.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(owner_id.is_some()).bind(owner_id).bind(owner_name)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_promotion(&row))
    }

    async fn update_promotion_status(&self, id: Uuid, status: &str) -> AtlasResult<PromoMgmtPromotion> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotions SET status = $2, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, code, name, description, promotion_type, status,
                 start_date, end_date, customer_id, customer_name, territory_id,
                 product_id, product_name,
                 budget_amount::text as budget_amount, spent_amount::text as spent_amount,
                 currency_code, owner_id, owner_name, is_active, metadata,
                 created_by, created_at, updated_at"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_promotion(&row))
    }

    async fn update_promotion_spent(&self, id: Uuid, spent_amount: &str) -> AtlasResult<PromoMgmtPromotion> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotions SET spent_amount = $2::numeric, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, code, name, description, promotion_type, status,
                 start_date, end_date, customer_id, customer_name, territory_id,
                 product_id, product_name,
                 budget_amount::text as budget_amount, spent_amount::text as spent_amount,
                 currency_code, owner_id, owner_name, is_active, metadata,
                 created_by, created_at, updated_at"#,
        )
        .bind(id).bind(spent_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_promotion(&row))
    }

    async fn delete_promotion(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.promotions WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Offers
    async fn create_offer(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        offer_type: &str,
        description: Option<&str>,
        discount_type: &str,
        discount_value: &str,
        buy_quantity: Option<i32>,
        get_quantity: Option<i32>,
        minimum_purchase: Option<&str>,
        maximum_discount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtOffer> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.promotion_offers
                (organization_id, promotion_id, offer_type, description,
                 discount_type, discount_value, buy_quantity, get_quantity,
                 minimum_purchase, maximum_discount, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING id, organization_id, promotion_id, offer_type, description,
                 discount_type, discount_value::text as discount_value,
                 buy_quantity, get_quantity,
                 minimum_purchase::text as minimum_purchase,
                 maximum_discount::text as maximum_discount,
                 is_active, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(promotion_id).bind(offer_type).bind(description)
        .bind(discount_type).bind(discount_value.parse::<f64>().unwrap_or(0.0))
        .bind(buy_quantity).bind(get_quantity)
        .bind(minimum_purchase.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(maximum_discount.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_offer(&row))
    }

    async fn list_offers(&self, promotion_id: Uuid) -> AtlasResult<Vec<PromoMgmtOffer>> {
        let rows = sqlx::query(
            "SELECT id, organization_id, promotion_id, offer_type, description, discount_type, discount_value::text as discount_value, buy_quantity, get_quantity, minimum_purchase::text as minimum_purchase, maximum_discount::text as maximum_discount, is_active, created_by, created_at, updated_at FROM _atlas.promotion_offers WHERE promotion_id = $1 AND is_active = true ORDER BY created_at",
        )
        .bind(promotion_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_promo_mgmt_offer).collect())
    }

    async fn get_offer(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtOffer>> {
        let row = sqlx::query(
            "SELECT id, organization_id, promotion_id, offer_type, description, discount_type, discount_value::text as discount_value, buy_quantity, get_quantity, minimum_purchase::text as minimum_purchase, maximum_discount::text as maximum_discount, is_active, created_by, created_at, updated_at FROM _atlas.promotion_offers WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_promo_mgmt_offer(&r)))
    }

    async fn delete_offer(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.promotion_offers WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Funds
    async fn create_fund(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        fund_type: &str,
        allocated_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtFund> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.promotion_funds
                (organization_id, promotion_id, fund_type, allocated_amount, currency_code, created_by)
               VALUES ($1,$2,$3,$4,$5,$6)
               RETURNING id, organization_id, promotion_id, fund_type,
                 allocated_amount::text as allocated_amount,
                 committed_amount::text as committed_amount,
                 spent_amount::text as spent_amount,
                 currency_code, is_active, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(promotion_id).bind(fund_type)
        .bind(allocated_amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_fund(&row))
    }

    async fn list_funds(&self, promotion_id: Uuid) -> AtlasResult<Vec<PromoMgmtFund>> {
        let rows = sqlx::query(
            "SELECT id, organization_id, promotion_id, fund_type, allocated_amount::text as allocated_amount, committed_amount::text as committed_amount, spent_amount::text as spent_amount, currency_code, is_active, created_by, created_at, updated_at FROM _atlas.promotion_funds WHERE promotion_id = $1 AND is_active = true ORDER BY fund_type",
        )
        .bind(promotion_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_promo_mgmt_fund).collect())
    }

    async fn update_fund_committed(&self, id: Uuid, committed_amount: &str) -> AtlasResult<PromoMgmtFund> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_funds SET committed_amount = $2::numeric, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, promotion_id, fund_type,
                 allocated_amount::text as allocated_amount,
                 committed_amount::text as committed_amount,
                 spent_amount::text as spent_amount,
                 currency_code, is_active, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(committed_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_fund(&row))
    }

    async fn update_fund_spent(&self, id: Uuid, spent_amount: &str) -> AtlasResult<PromoMgmtFund> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_funds SET spent_amount = $2::numeric, updated_at = now() WHERE id = $1
               RETURNING id, organization_id, promotion_id, fund_type,
                 allocated_amount::text as allocated_amount,
                 committed_amount::text as committed_amount,
                 spent_amount::text as spent_amount,
                 currency_code, is_active, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(spent_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_fund(&row))
    }

    async fn delete_fund(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.promotion_funds WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Claims
    async fn create_claim(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        claim_number: &str,
        claim_type: &str,
        amount: &str,
        currency_code: &str,
        claim_date: chrono::NaiveDate,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PromoMgmtClaim> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.promotion_claims
                (organization_id, promotion_id, claim_number, claim_type, amount,
                 currency_code, claim_date, customer_id, customer_name, description, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING id, organization_id, promotion_id, claim_number, claim_type, status,
                 amount::text as amount, approved_amount::text as approved_amount,
                 paid_amount::text as paid_amount, currency_code, claim_date,
                 settlement_date, customer_id, customer_name, description,
                 rejection_reason, created_by, created_at, updated_at"#,
        )
        .bind(org_id).bind(promotion_id).bind(claim_number).bind(claim_type)
        .bind(amount.parse::<f64>().unwrap_or(0.0))
        .bind(currency_code).bind(claim_date)
        .bind(customer_id).bind(customer_name).bind(description).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_claim(&row))
    }

    async fn get_claim(&self, id: Uuid) -> AtlasResult<Option<PromoMgmtClaim>> {
        let row = sqlx::query(
            "SELECT id, organization_id, promotion_id, claim_number, claim_type, status, amount::text as amount, approved_amount::text as approved_amount, paid_amount::text as paid_amount, currency_code, claim_date, settlement_date, customer_id, customer_name, description, rejection_reason, created_by, created_at, updated_at FROM _atlas.promotion_claims WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_promo_mgmt_claim(&r)))
    }

    async fn list_claims(&self, promotion_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PromoMgmtClaim>> {
        let cols = "id, organization_id, promotion_id, claim_number, claim_type, status, amount::text as amount, approved_amount::text as approved_amount, paid_amount::text as paid_amount, currency_code, claim_date, settlement_date, customer_id, customer_name, description, rejection_reason, created_by, created_at, updated_at";
        let rows = match status {
            Some(s) => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotion_claims WHERE promotion_id = $1 AND status = $2 ORDER BY claim_date DESC", cols))
                .bind(promotion_id).bind(s).fetch_all(&self.pool).await,
            None => sqlx::query(&format!(
                "SELECT {} FROM _atlas.promotion_claims WHERE promotion_id = $1 ORDER BY claim_date DESC", cols))
                .bind(promotion_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_promo_mgmt_claim).collect())
    }

    async fn update_claim_status(
        &self,
        id: Uuid,
        status: &str,
        approved_amount: Option<&str>,
        rejection_reason: Option<&str>,
    ) -> AtlasResult<PromoMgmtClaim> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_claims SET
                status = $2,
                approved_amount = COALESCE($3::numeric, approved_amount),
                rejection_reason = COALESCE($4, rejection_reason),
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, promotion_id, claim_number, claim_type, status,
                 amount::text as amount, approved_amount::text as approved_amount,
                 paid_amount::text as paid_amount, currency_code, claim_date,
                 settlement_date, customer_id, customer_name, description,
                 rejection_reason, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(status)
        .bind(approved_amount.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(rejection_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_claim(&row))
    }

    async fn settle_claim(&self, id: Uuid, paid_amount: &str, settlement_date: chrono::NaiveDate) -> AtlasResult<PromoMgmtClaim> {
        let row = sqlx::query(
            r#"UPDATE _atlas.promotion_claims SET
                status = 'paid', paid_amount = $2::numeric, settlement_date = $3,
                updated_at = now()
               WHERE id = $1
               RETURNING id, organization_id, promotion_id, claim_number, claim_type, status,
                 amount::text as amount, approved_amount::text as approved_amount,
                 paid_amount::text as paid_amount, currency_code, claim_date,
                 settlement_date, customer_id, customer_name, description,
                 rejection_reason, created_by, created_at, updated_at"#,
        )
        .bind(id).bind(paid_amount.parse::<f64>().unwrap_or(0.0)).bind(settlement_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_promo_mgmt_claim(&row))
    }

    async fn delete_claim(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.promotion_claims WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PromoMgmtDashboard> {
        let promo_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COALESCE(SUM(budget_amount), 0) as total_budget,
                COALESCE(SUM(spent_amount), 0) as total_spent
               FROM _atlas.promotions WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let claim_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total_claims,
                COUNT(*) FILTER (WHERE status IN ('submitted', 'under_review')) as pending_claims
               FROM _atlas.promotion_claims WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        let total_budget: f64 = promo_row.try_get("total_budget").unwrap_or(0.0);
        let total_spent: f64 = promo_row.try_get("total_spent").unwrap_or(0.0);
        let util_pct = if total_budget > 0.0 { (total_spent / total_budget) * 100.0 } else { 0.0 };

        let by_status_row = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.promotions
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_type_row = sqlx::query(
            r#"SELECT promotion_type, COUNT(*) as cnt FROM _atlas.promotions
               WHERE organization_id = $1 GROUP BY promotion_type ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_status: serde_json::Value = by_status_row.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        let by_type: serde_json::Value = by_type_row.iter().map(|r| {
            serde_json::json!({
                "type": r.get::<String, _>("promotion_type"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        Ok(PromoMgmtDashboard {
            total_promotions: promo_row.get::<i64, _>("total") as i32,
            active_promotions: promo_row.get::<i64, _>("active") as i32,
            total_budget: format!("{:.2}", total_budget),
            total_spent: format!("{:.2}", total_spent),
            utilization_percent: format!("{:.1}", util_pct),
            total_claims: claim_row.get::<i64, _>("total_claims") as i32,
            pending_claims: claim_row.get::<i64, _>("pending_claims") as i32,
            by_status,
            by_type,
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn row_to_promo_mgmt_promotion(row: &sqlx::postgres::PgRow) -> PromoMgmtPromotion {
    let budget_str: String = row.try_get("budget_amount").unwrap_or_else(|_| "0".to_string());
    let spent_str: String = row.try_get("spent_amount").unwrap_or_else(|_| "0".to_string());
    let budget: f64 = budget_str.parse().unwrap_or(0.0);
    let spent: f64 = spent_str.parse().unwrap_or(0.0);
    PromoMgmtPromotion {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        promotion_type: row.get("promotion_type"),
        status: row.get("status"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        territory_id: row.get("territory_id"),
        product_id: row.get("product_id"),
        product_name: row.get("product_name"),
        budget_amount: format!("{:.2}", budget),
        spent_amount: format!("{:.2}", spent),
        currency_code: row.get("currency_code"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_promo_mgmt_offer(row: &sqlx::postgres::PgRow) -> PromoMgmtOffer {
    let val_str: String = row.try_get("discount_value").unwrap_or_else(|_| "0".to_string());
    let min_str: Option<String> = row.try_get("minimum_purchase").ok();
    let max_str: Option<String> = row.try_get("maximum_discount").ok();
    PromoMgmtOffer {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        promotion_id: row.get("promotion_id"),
        offer_type: row.get("offer_type"),
        description: row.get("description"),
        discount_type: row.get("discount_type"),
        discount_value: format!("{:.2}", val_str.parse::<f64>().unwrap_or(0.0)),
        buy_quantity: row.get("buy_quantity"),
        get_quantity: row.get("get_quantity"),
        minimum_purchase: min_str.map(|s| format!("{:.2}", s.parse::<f64>().unwrap_or(0.0))),
        maximum_discount: max_str.map(|s| format!("{:.2}", s.parse::<f64>().unwrap_or(0.0))),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_promo_mgmt_fund(row: &sqlx::postgres::PgRow) -> PromoMgmtFund {
    let alloc: String = row.try_get("allocated_amount").unwrap_or_else(|_| "0".to_string());
    let comm: String = row.try_get("committed_amount").unwrap_or_else(|_| "0".to_string());
    let spent: String = row.try_get("spent_amount").unwrap_or_else(|_| "0".to_string());
    PromoMgmtFund {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        promotion_id: row.get("promotion_id"),
        fund_type: row.get("fund_type"),
        allocated_amount: format!("{:.2}", alloc.parse::<f64>().unwrap_or(0.0)),
        committed_amount: format!("{:.2}", comm.parse::<f64>().unwrap_or(0.0)),
        spent_amount: format!("{:.2}", spent.parse::<f64>().unwrap_or(0.0)),
        currency_code: row.get("currency_code"),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_promo_mgmt_claim(row: &sqlx::postgres::PgRow) -> PromoMgmtClaim {
    let amt: String = row.try_get("amount").unwrap_or_else(|_| "0".to_string());
    let app: Option<String> = row.try_get("approved_amount").ok();
    let paid: Option<String> = row.try_get("paid_amount").ok();
    PromoMgmtClaim {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        promotion_id: row.get("promotion_id"),
        claim_number: row.get("claim_number"),
        claim_type: row.get("claim_type"),
        status: row.get("status"),
        amount: format!("{:.2}", amt.parse::<f64>().unwrap_or(0.0)),
        approved_amount: app.map(|s| format!("{:.2}", s.parse::<f64>().unwrap_or(0.0))),
        paid_amount: paid.map(|s| format!("{:.2}", s.parse::<f64>().unwrap_or(0.0))),
        currency_code: row.get("currency_code"),
        claim_date: row.get("claim_date"),
        settlement_date: row.get("settlement_date"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        description: row.get("description"),
        rejection_reason: row.get("rejection_reason"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
