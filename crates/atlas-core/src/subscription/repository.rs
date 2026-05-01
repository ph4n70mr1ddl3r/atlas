//! Subscription Management Repository
//!
//! PostgreSQL storage for subscription products, subscriptions, amendments,
//! billing schedules, and revenue schedules.

use atlas_shared::{
    SubscriptionProduct, SubscriptionPriceTier, Subscription, SubscriptionAmendment,
    SubscriptionBillingLine, SubscriptionRevenueLine, SubscriptionDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for subscription management data storage
#[async_trait]
pub trait SubscriptionRepository: Send + Sync {
    // Products
    async fn create_product(
        &self, org_id: Uuid, product_code: &str, name: &str, description: Option<&str>,
        product_type: &str, billing_frequency: &str, default_duration_months: i32,
        is_auto_renew: bool, cancellation_notice_days: i32, setup_fee: &str,
        tier_type: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionProduct>;
    async fn get_product(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SubscriptionProduct>>;
    async fn get_product_by_id(&self, id: Uuid) -> AtlasResult<Option<SubscriptionProduct>>;
    async fn list_products(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SubscriptionProduct>>;
    async fn delete_product(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Price Tiers
    async fn create_price_tier(
        &self, org_id: Uuid, product_id: Uuid, tier_name: Option<&str>,
        min_quantity: &str, max_quantity: Option<&str>, unit_price: &str,
        discount_percent: &str, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<SubscriptionPriceTier>;
    async fn list_price_tiers(&self, org_id: Uuid, product_id: Uuid) -> AtlasResult<Vec<SubscriptionPriceTier>>;

    // Subscriptions
    #[allow(clippy::too_many_arguments)]
    async fn create_subscription(
        &self, org_id: Uuid, subscription_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, product_id: Uuid, product_code: Option<&str>,
        product_name: Option<&str>, description: Option<&str>, status: &str,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        renewal_date: Option<&chrono::NaiveDate>, billing_frequency: &str,
        billing_day_of_month: i32, billing_alignment: &str, currency_code: &str,
        quantity: &str, unit_price: &str, list_price: &str, discount_percent: &str,
        setup_fee: &str, recurring_amount: &str, total_contract_value: &str,
        total_billed: &str, total_revenue_recognized: &str, duration_months: i32,
        is_auto_renew: bool, cancellation_date: Option<chrono::NaiveDate>,
        cancellation_reason: Option<&str>, suspension_reason: Option<&str>,
        sales_rep_id: Option<Uuid>, sales_rep_name: Option<&str>,
        gl_revenue_account: Option<&str>, gl_deferred_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Subscription>;
    async fn get_subscription(&self, id: Uuid) -> AtlasResult<Option<Subscription>>;
    async fn get_subscription_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<Subscription>>;
    async fn list_subscriptions(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<Subscription>>;
    async fn update_subscription_status(&self, id: Uuid, status: &str, cancellation_date: Option<chrono::NaiveDate>, cancellation_reason: Option<&str>, suspension_reason: Option<&str>) -> AtlasResult<Subscription>;
    async fn update_subscription_dates(&self, id: Uuid, end_date: Option<chrono::NaiveDate>, renewal_date: Option<&chrono::NaiveDate>) -> AtlasResult<Subscription>;
    async fn update_subscription_pricing(&self, id: Uuid, quantity: &str, unit_price: &str, recurring_amount: &str) -> AtlasResult<()>;

    // Amendments
    #[allow(clippy::too_many_arguments)]
    async fn create_amendment(
        &self, org_id: Uuid, subscription_id: Uuid, amendment_number: &str,
        amendment_type: &str, description: Option<&str>,
        old_quantity: Option<&str>, new_quantity: Option<&str>,
        old_unit_price: Option<&str>, new_unit_price: Option<&str>,
        old_recurring_amount: Option<&str>, new_recurring_amount: Option<&str>,
        old_end_date: Option<&chrono::NaiveDate>, new_end_date: Option<&chrono::NaiveDate>,
        effective_date: chrono::NaiveDate, proration_credit: &str, proration_charge: &str,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionAmendment>;
    async fn get_amendment(&self, id: Uuid) -> AtlasResult<Option<SubscriptionAmendment>>;
    async fn list_amendments(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionAmendment>>;
    async fn update_amendment_status(&self, id: Uuid, status: &str, applied_by: Option<Uuid>) -> AtlasResult<SubscriptionAmendment>;

    // Billing Schedule
    async fn create_billing_line(
        &self, org_id: Uuid, subscription_id: Uuid, schedule_number: i32,
        billing_date: chrono::NaiveDate, period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate, amount: &str, proration_amount: &str, total_amount: &str,
    ) -> AtlasResult<SubscriptionBillingLine>;
    async fn list_billing_lines(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionBillingLine>>;

    // Revenue Schedule
    async fn create_revenue_line(
        &self, org_id: Uuid, subscription_id: Uuid, billing_schedule_id: Option<Uuid>,
        period_name: &str, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        revenue_amount: &str, deferred_amount: &str, recognized_to_date: &str, status: &str,
    ) -> AtlasResult<SubscriptionRevenueLine>;
    async fn get_revenue_line(&self, id: Uuid) -> AtlasResult<Option<SubscriptionRevenueLine>>;
    async fn list_revenue_lines(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionRevenueLine>>;
    async fn update_revenue_line_status(&self, id: Uuid, status: &str) -> AtlasResult<SubscriptionRevenueLine>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SubscriptionDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresSubscriptionRepository {
    pool: PgPool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn get_numeric(&self, row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: f64 = row.try_get(col).unwrap_or(0.0);
        format!("{:.2}", v)
    }

    fn row_to_product(&self, row: &sqlx::postgres::PgRow) -> SubscriptionProduct {
        SubscriptionProduct {
            id: row.get("id"), organization_id: row.get("organization_id"),
            product_code: row.get("product_code"), name: row.get("name"),
            description: row.get("description"), product_type: row.get("product_type"),
            billing_frequency: row.get("billing_frequency"),
            default_duration_months: row.get("default_duration_months"),
            is_auto_renew: row.get("is_auto_renew"),
            cancellation_notice_days: row.get("cancellation_notice_days"),
            setup_fee: self.get_numeric(row, "setup_fee"),
            tier_type: row.get("tier_type"), is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_subscription(&self, row: &sqlx::postgres::PgRow) -> Subscription {
        Subscription {
            id: row.get("id"), organization_id: row.get("organization_id"),
            subscription_number: row.get("subscription_number"),
            customer_id: row.get("customer_id"), customer_name: row.get("customer_name"),
            product_id: row.get("product_id"), product_code: row.get("product_code"),
            product_name: row.get("product_name"), description: row.get("description"),
            status: row.get("status"), start_date: row.get("start_date"),
            end_date: row.get("end_date"), renewal_date: row.get("renewal_date"),
            billing_frequency: row.get("billing_frequency"),
            billing_day_of_month: row.get("billing_day_of_month"),
            billing_alignment: row.get("billing_alignment"),
            currency_code: row.get("currency_code"),
            quantity: self.get_numeric(row, "quantity"),
            unit_price: self.get_numeric(row, "unit_price"),
            list_price: self.get_numeric(row, "list_price"),
            discount_percent: self.get_numeric(row, "discount_percent"),
            setup_fee: self.get_numeric(row, "setup_fee"),
            recurring_amount: self.get_numeric(row, "recurring_amount"),
            total_contract_value: self.get_numeric(row, "total_contract_value"),
            total_billed: self.get_numeric(row, "total_billed"),
            total_revenue_recognized: self.get_numeric(row, "total_revenue_recognized"),
            duration_months: row.get("duration_months"), is_auto_renew: row.get("is_auto_renew"),
            cancellation_date: row.get("cancellation_date"),
            cancellation_reason: row.get("cancellation_reason"),
            suspension_reason: row.get("suspension_reason"),
            sales_rep_id: row.get("sales_rep_id"), sales_rep_name: row.get("sales_rep_name"),
            gl_revenue_account: row.get("gl_revenue_account"),
            gl_deferred_account: row.get("gl_deferred_account"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn create_product(&self, org_id: Uuid, product_code: &str, name: &str,
        description: Option<&str>, product_type: &str, billing_frequency: &str,
        default_duration_months: i32, is_auto_renew: bool, cancellation_notice_days: i32,
        setup_fee: &str, tier_type: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionProduct> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscription_products
                (organization_id, product_code, name, description, product_type,
                 billing_frequency, default_duration_months, is_auto_renew,
                 cancellation_notice_days, setup_fee, tier_type, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10::numeric,$11,$12) RETURNING *"#,
        ).bind(org_id).bind(product_code).bind(name).bind(description)
        .bind(product_type).bind(billing_frequency).bind(default_duration_months)
        .bind(is_auto_renew).bind(cancellation_notice_days).bind(setup_fee)
        .bind(tier_type).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_product(&row))
    }

    async fn get_product(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SubscriptionProduct>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscription_products WHERE organization_id=$1 AND product_code=$2")
            .bind(org_id).bind(code).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_product(&r)))
    }

    async fn get_product_by_id(&self, id: Uuid) -> AtlasResult<Option<SubscriptionProduct>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscription_products WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_product(&r)))
    }

    async fn list_products(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SubscriptionProduct>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.subscription_products WHERE organization_id=$1 AND is_active=true ORDER BY product_code")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.subscription_products WHERE organization_id=$1 ORDER BY product_code")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_product(r)).collect())
    }

    async fn delete_product(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.subscription_products WHERE organization_id=$1 AND product_code=$2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_price_tier(&self, org_id: Uuid, product_id: Uuid, tier_name: Option<&str>,
        min_quantity: &str, max_quantity: Option<&str>, unit_price: &str,
        discount_percent: &str, currency_code: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<SubscriptionPriceTier> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscription_price_tiers
                (organization_id, product_id, tier_name, min_quantity, max_quantity,
                 unit_price, discount_percent, currency_code, effective_from, effective_to)
            VALUES ($1,$2,$3,$4::numeric,$5::numeric,$6::numeric,$7::numeric,$8,$9,$10) RETURNING *"#,
        ).bind(org_id).bind(product_id).bind(tier_name).bind(min_quantity)
        .bind(max_quantity).bind(unit_price).bind(discount_percent).bind(currency_code)
        .bind(effective_from).bind(effective_to)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(SubscriptionPriceTier {
            id: row.get("id"), organization_id: row.get("organization_id"),
            product_id: row.get("product_id"), tier_name: row.get("tier_name"),
            min_quantity: self.get_numeric(&row, "min_quantity"),
            max_quantity: row.try_get("max_quantity").unwrap_or(None),
            unit_price: self.get_numeric(&row, "unit_price"),
            discount_percent: self.get_numeric(&row, "discount_percent"),
            currency_code: row.get("currency_code"),
            effective_from: row.get("effective_from"), effective_to: row.get("effective_to"),
            is_active: row.get("is_active"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_price_tiers(&self, org_id: Uuid, product_id: Uuid) -> AtlasResult<Vec<SubscriptionPriceTier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subscription_price_tiers WHERE organization_id=$1 AND product_id=$2 AND is_active=true ORDER BY min_quantity"
        ).bind(org_id).bind(product_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| SubscriptionPriceTier {
            id: r.get("id"), organization_id: r.get("organization_id"),
            product_id: r.get("product_id"), tier_name: r.get("tier_name"),
            min_quantity: self.get_numeric(r, "min_quantity"),
            max_quantity: r.try_get("max_quantity").unwrap_or(None),
            unit_price: self.get_numeric(r, "unit_price"),
            discount_percent: self.get_numeric(r, "discount_percent"),
            currency_code: r.get("currency_code"),
            effective_from: r.get("effective_from"), effective_to: r.get("effective_to"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_subscription(&self, org_id: Uuid, subscription_number: &str,
        customer_id: Uuid, customer_name: Option<&str>, product_id: Uuid,
        product_code: Option<&str>, product_name: Option<&str>, description: Option<&str>,
        status: &str, start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        renewal_date: Option<&chrono::NaiveDate>, billing_frequency: &str,
        billing_day_of_month: i32, billing_alignment: &str, currency_code: &str,
        quantity: &str, unit_price: &str, list_price: &str, discount_percent: &str,
        setup_fee: &str, recurring_amount: &str, total_contract_value: &str,
        total_billed: &str, total_revenue_recognized: &str, duration_months: i32,
        is_auto_renew: bool, cancellation_date: Option<chrono::NaiveDate>,
        cancellation_reason: Option<&str>, suspension_reason: Option<&str>,
        sales_rep_id: Option<Uuid>, sales_rep_name: Option<&str>,
        gl_revenue_account: Option<&str>, gl_deferred_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Subscription> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscriptions
                (organization_id, subscription_number, customer_id, customer_name,
                 product_id, product_code, product_name, description, status,
                 start_date, end_date, renewal_date, billing_frequency,
                 billing_day_of_month, billing_alignment, currency_code,
                 quantity, unit_price, list_price, discount_percent, setup_fee,
                 recurring_amount, total_contract_value, total_billed,
                 total_revenue_recognized, duration_months, is_auto_renew,
                 cancellation_date, cancellation_reason, suspension_reason,
                 sales_rep_id, sales_rep_name, gl_revenue_account, gl_deferred_account, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,
                    $17::numeric,$18::numeric,$19::numeric,$20::numeric,$21::numeric,
                    $22::numeric,$23::numeric,$24::numeric,$25::numeric,$26,$27,
                    $28,$29,$30,$31,$32,$33,$34,$35) RETURNING *"#,
        ).bind(org_id).bind(subscription_number).bind(customer_id).bind(customer_name)
        .bind(product_id).bind(product_code).bind(product_name).bind(description)
        .bind(status).bind(start_date).bind(end_date).bind(renewal_date)
        .bind(billing_frequency).bind(billing_day_of_month).bind(billing_alignment)
        .bind(currency_code).bind(quantity).bind(unit_price).bind(list_price)
        .bind(discount_percent).bind(setup_fee).bind(recurring_amount)
        .bind(total_contract_value).bind(total_billed).bind(total_revenue_recognized)
        .bind(duration_months).bind(is_auto_renew).bind(cancellation_date)
        .bind(cancellation_reason).bind(suspension_reason).bind(sales_rep_id)
        .bind(sales_rep_name).bind(gl_revenue_account).bind(gl_deferred_account)
        .bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_subscription(&row))
    }

    async fn get_subscription(&self, id: Uuid) -> AtlasResult<Option<Subscription>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscriptions WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_subscription(&r)))
    }

    async fn get_subscription_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<Subscription>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscriptions WHERE organization_id=$1 AND subscription_number=$2")
            .bind(org_id).bind(number).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_subscription(&r)))
    }

    async fn list_subscriptions(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<Subscription>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.subscriptions
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2) AND ($3::uuid IS NULL OR customer_id=$3)
            ORDER BY subscription_number"#,
        ).bind(org_id).bind(status).bind(customer_id)
        .fetch_all(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_subscription(r)).collect())
    }

    async fn update_subscription_status(&self, id: Uuid, status: &str,
        cancellation_date: Option<chrono::NaiveDate>, cancellation_reason: Option<&str>,
        suspension_reason: Option<&str>,
    ) -> AtlasResult<Subscription> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subscriptions SET status=$2,
                cancellation_date=COALESCE($3, cancellation_date),
                cancellation_reason=COALESCE($4, cancellation_reason),
                suspension_reason=COALESCE($5, suspension_reason),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(cancellation_date).bind(cancellation_reason).bind(suspension_reason)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_subscription(&row))
    }

    async fn update_subscription_dates(&self, id: Uuid, end_date: Option<chrono::NaiveDate>,
        renewal_date: Option<&chrono::NaiveDate>,
    ) -> AtlasResult<Subscription> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subscriptions SET end_date=$2, renewal_date=$3, updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(end_date).bind(renewal_date)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_subscription(&row))
    }

    async fn update_subscription_pricing(&self, id: Uuid, quantity: &str, unit_price: &str, recurring_amount: &str) -> AtlasResult<()> {
        sqlx::query(r#"UPDATE _atlas.subscriptions SET quantity=$2::numeric, unit_price=$3::numeric, recurring_amount=$4::numeric, updated_at=now() WHERE id=$1"#)
            .bind(id).bind(quantity).bind(unit_price).bind(recurring_amount)
            .execute(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_amendment(&self, org_id: Uuid, subscription_id: Uuid, amendment_number: &str,
        amendment_type: &str, description: Option<&str>,
        old_quantity: Option<&str>, new_quantity: Option<&str>,
        old_unit_price: Option<&str>, new_unit_price: Option<&str>,
        old_recurring_amount: Option<&str>, new_recurring_amount: Option<&str>,
        old_end_date: Option<&chrono::NaiveDate>, new_end_date: Option<&chrono::NaiveDate>,
        effective_date: chrono::NaiveDate, proration_credit: &str, proration_charge: &str,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionAmendment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscription_amendments
                (organization_id, subscription_id, amendment_number, amendment_type,
                 description, old_quantity, new_quantity, old_unit_price, new_unit_price,
                 old_recurring_amount, new_recurring_amount, old_end_date, new_end_date,
                 effective_date, proration_credit, proration_charge, status, created_by)
            VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8::numeric,$9::numeric,
                    $10::numeric,$11::numeric,$12,$13,$14,$15::numeric,$16::numeric,$17,$18)
            RETURNING *"#,
        ).bind(org_id).bind(subscription_id).bind(amendment_number).bind(amendment_type)
        .bind(description).bind(old_quantity).bind(new_quantity).bind(old_unit_price).bind(new_unit_price)
        .bind(old_recurring_amount).bind(new_recurring_amount).bind(old_end_date).bind(new_end_date)
        .bind(effective_date).bind(proration_credit).bind(proration_charge).bind(status).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_amendment(&row))
    }

    async fn get_amendment(&self, id: Uuid) -> AtlasResult<Option<SubscriptionAmendment>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscription_amendments WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_amendment(&r)))
    }

    async fn list_amendments(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionAmendment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subscription_amendments WHERE subscription_id=$1 ORDER BY created_at"
        ).bind(subscription_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_amendment).collect())
    }

    async fn update_amendment_status(&self, id: Uuid, status: &str, applied_by: Option<Uuid>) -> AtlasResult<SubscriptionAmendment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subscription_amendments SET status=$2,
                applied_by=COALESCE($3, applied_by),
                applied_at=CASE WHEN $3 IS NOT NULL AND applied_at IS NULL THEN now() ELSE applied_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(applied_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_amendment(&row))
    }

    async fn create_billing_line(&self, org_id: Uuid, subscription_id: Uuid, schedule_number: i32,
        billing_date: chrono::NaiveDate, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        amount: &str, proration_amount: &str, total_amount: &str,
    ) -> AtlasResult<SubscriptionBillingLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscription_billing_schedule
                (organization_id, subscription_id, schedule_number, billing_date,
                 period_start, period_end, amount, proration_amount, total_amount)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8::numeric,$9::numeric) RETURNING *"#,
        ).bind(org_id).bind(subscription_id).bind(schedule_number)
        .bind(billing_date).bind(period_start).bind(period_end)
        .bind(amount).bind(proration_amount).bind(total_amount)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_billing_line(&row))
    }

    async fn list_billing_lines(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionBillingLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subscription_billing_schedule WHERE subscription_id=$1 ORDER BY schedule_number"
        ).bind(subscription_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_billing_line).collect())
    }

    async fn create_revenue_line(&self, org_id: Uuid, subscription_id: Uuid,
        billing_schedule_id: Option<Uuid>, period_name: &str,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        revenue_amount: &str, deferred_amount: &str, recognized_to_date: &str, status: &str,
    ) -> AtlasResult<SubscriptionRevenueLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subscription_revenue_schedule
                (organization_id, subscription_id, billing_schedule_id, period_name,
                 period_start, period_end, revenue_amount, deferred_amount, recognized_to_date, status)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8::numeric,$9::numeric,$10) RETURNING *"#,
        ).bind(org_id).bind(subscription_id).bind(billing_schedule_id).bind(period_name)
        .bind(period_start).bind(period_end).bind(revenue_amount).bind(deferred_amount)
        .bind(recognized_to_date).bind(status)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_revenue_line(&row))
    }

    async fn get_revenue_line(&self, id: Uuid) -> AtlasResult<Option<SubscriptionRevenueLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.subscription_revenue_schedule WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_revenue_line(&r)))
    }

    async fn list_revenue_lines(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionRevenueLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subscription_revenue_schedule WHERE subscription_id=$1 ORDER BY period_start"
        ).bind(subscription_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_revenue_line).collect())
    }

    async fn update_revenue_line_status(&self, id: Uuid, status: &str) -> AtlasResult<SubscriptionRevenueLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subscription_revenue_schedule SET status=$2,
                recognized_at=CASE WHEN $2='recognized' AND recognized_at IS NULL THEN now() ELSE recognized_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_revenue_line(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SubscriptionDashboardSummary> {
        let rows = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COUNT(DISTINCT CASE WHEN status = 'active' AND customer_id IS NOT NULL THEN customer_id END) as distinct_customers,
                COALESCE(SUM(recurring_amount) FILTER (WHERE status = 'active'), 0::numeric)::float8 as mrr,
                COUNT(DISTINCT customer_id) FILTER (WHERE status = 'active') as total_subscribers,
                COALESCE(SUM(total_contract_value) FILTER (WHERE status = 'active'), 0::numeric)::float8 as tcv,
                COALESCE(SUM(total_billed) FILTER (WHERE status = 'active'), 0::numeric)::float8 as total_billed,
                COALESCE(SUM(total_revenue_recognized), 0::numeric)::float8 as total_rev,
                COUNT(*) FILTER (WHERE renewal_date <= CURRENT_DATE + INTERVAL '30 days' AND status = 'active' AND is_auto_renew = true) as renewals_30,
                COUNT(*) FILTER (WHERE created_at >= date_trunc('month', CURRENT_DATE) AND status IN ('active','draft')) as new_this_month,
                COUNT(*) FILTER (WHERE cancellation_date >= date_trunc('month', CURRENT_DATE)) as cancelled_this_month
            FROM _atlas.subscriptions WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = rows.try_get("active_count").unwrap_or(0);
        let subscribers: i64 = rows.try_get("total_subscribers").unwrap_or(0);
        let renewals_30: i64 = rows.try_get("renewals_30").unwrap_or(0);
        let new_month: i64 = rows.try_get("new_this_month").unwrap_or(0);
        let cancelled_month: i64 = rows.try_get("cancelled_this_month").unwrap_or(0);

        let mrr: f64 = rows.try_get("mrr").unwrap_or(0.0);
        let tcv: f64 = rows.try_get("tcv").unwrap_or(0.0);
        let billed: f64 = rows.try_get("total_billed").unwrap_or(0.0);
        let rev: f64 = rows.try_get("total_rev").unwrap_or(0.0);

        let arr_val = mrr * 12.0;

        Ok(SubscriptionDashboardSummary {
            total_active_subscriptions: active as i32,
            total_subscribers: subscribers as i32,
            total_monthly_recurring_revenue: format!("{:.2}", mrr),
            total_annual_recurring_revenue: format!("{:.2}", arr_val),
            total_contract_value: format!("{:.2}", tcv),
            total_billed: format!("{:.2}", billed),
            total_revenue_recognized: format!("{:.2}", rev),
            total_deferred_revenue: "0".to_string(),
            churn_rate_percent: "0".to_string(),
            renewals_due_30_days: renewals_30 as i32,
            new_subscriptions_this_month: new_month as i32,
            cancelled_this_month: cancelled_month as i32,
            subscriptions_by_status: serde_json::json!({}),
            revenue_by_product: serde_json::json!({}),
        })
    }
}

fn row_to_amendment(row: &sqlx::postgres::PgRow) -> SubscriptionAmendment {
    SubscriptionAmendment {
        id: row.get("id"), organization_id: row.get("organization_id"),
        subscription_id: row.get("subscription_id"),
        amendment_number: row.get("amendment_number"),
        amendment_type: row.get("amendment_type"),
        description: row.get("description"),
        old_quantity: row.try_get("old_quantity").unwrap_or(None),
        new_quantity: row.try_get("new_quantity").unwrap_or(None),
        old_unit_price: row.try_get("old_unit_price").unwrap_or(None),
        new_unit_price: row.try_get("new_unit_price").unwrap_or(None),
        old_recurring_amount: row.try_get("old_recurring_amount").unwrap_or(None),
        new_recurring_amount: row.try_get("new_recurring_amount").unwrap_or(None),
        old_end_date: row.get("old_end_date"),
        new_end_date: row.get("new_end_date"),
        effective_date: row.get("effective_date"),
        proration_credit: row.try_get("proration_credit").unwrap_or(None),
        proration_charge: row.try_get("proration_charge").unwrap_or(None),
        status: row.get("status"),
        applied_at: row.get("applied_at"),
        applied_by: row.get("applied_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_billing_line(row: &sqlx::postgres::PgRow) -> SubscriptionBillingLine {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: f64 = row.try_get(col).unwrap_or(0.0);
        format!("{:.2}", v)
    }
    SubscriptionBillingLine {
        id: row.get("id"), organization_id: row.get("organization_id"),
        subscription_id: row.get("subscription_id"),
        schedule_number: row.get("schedule_number"),
        billing_date: row.get("billing_date"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        amount: get_num(row, "amount"),
        proration_amount: get_num(row, "proration_amount"),
        total_amount: get_num(row, "total_amount"),
        invoice_id: row.get("invoice_id"),
        invoice_number: row.get("invoice_number"),
        status: row.get("status"),
        paid_at: row.get("paid_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_revenue_line(row: &sqlx::postgres::PgRow) -> SubscriptionRevenueLine {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: f64 = row.try_get(col).unwrap_or(0.0);
        format!("{:.2}", v)
    }
    SubscriptionRevenueLine {
        id: row.get("id"), organization_id: row.get("organization_id"),
        subscription_id: row.get("subscription_id"),
        billing_schedule_id: row.get("billing_schedule_id"),
        period_name: row.get("period_name"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        revenue_amount: get_num(row, "revenue_amount"),
        deferred_amount: get_num(row, "deferred_amount"),
        recognized_to_date: get_num(row, "recognized_to_date"),
        status: row.get("status"),
        recognized_at: row.get("recognized_at"),
        journal_entry_id: row.get("journal_entry_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
