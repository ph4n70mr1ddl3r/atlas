//! Loyalty Management Repository
//!
//! PostgreSQL storage for loyalty programs, tiers, members, point transactions,
//! rewards, redemptions, and dashboard analytics.

use atlas_shared::{
    LoyaltyProgram, LoyaltyTier, LoyaltyMember, LoyaltyPointTransaction,
    LoyaltyReward, LoyaltyRedemption, LoyaltyDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for loyalty management data storage
#[async_trait]
pub trait LoyaltyManagementRepository: Send + Sync {
    // Programs
    async fn create_program(
        &self, org_id: Uuid, program_number: &str, name: &str, description: Option<&str>,
        program_type: &str, currency_code: &str, points_name: &str, enrollment_type: &str,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        accrual_rate: f64, accrual_basis: &str, minimum_accrual_amount: f64,
        rounding_method: &str, points_expiry_days: Option<i32>,
        tier_qualification_period: &str, auto_upgrade: bool, auto_downgrade: bool,
        max_points_per_member: Option<f64>, allow_point_transfer: bool, allow_redemption: bool,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyProgram>;
    async fn get_program(&self, id: Uuid) -> AtlasResult<Option<LoyaltyProgram>>;
    async fn get_program_by_number(&self, org_id: Uuid, program_number: &str) -> AtlasResult<Option<LoyaltyProgram>>;
    async fn list_programs(&self, org_id: Uuid, status: Option<&str>, program_type: Option<&str>) -> AtlasResult<Vec<LoyaltyProgram>>;
    async fn update_program_status(&self, id: Uuid, status: &str) -> AtlasResult<LoyaltyProgram>;
    async fn delete_program(&self, org_id: Uuid, program_number: &str) -> AtlasResult<()>;

    // Tiers
    async fn create_tier(
        &self, org_id: Uuid, program_id: Uuid, tier_code: &str, tier_name: &str,
        tier_level: i32, minimum_points: f64, maximum_points: Option<f64>,
        accrual_bonus_percentage: f64, benefits: &str, color: &str, icon: &str,
        is_default: bool,
    ) -> AtlasResult<LoyaltyTier>;
    async fn list_tiers(&self, program_id: Uuid) -> AtlasResult<Vec<LoyaltyTier>>;
    async fn delete_tier(&self, id: Uuid) -> AtlasResult<()>;

    // Members
    async fn create_member(
        &self, org_id: Uuid, program_id: Uuid, member_number: &str,
        customer_id: Option<Uuid>, customer_name: &str, customer_email: &str,
        tier_id: Option<Uuid>, tier_code: &str, enrollment_date: chrono::NaiveDate,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyMember>;
    async fn get_member(&self, id: Uuid) -> AtlasResult<Option<LoyaltyMember>>;
    async fn get_member_by_number(&self, org_id: Uuid, member_number: &str) -> AtlasResult<Option<LoyaltyMember>>;
    async fn list_members(&self, program_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LoyaltyMember>>;
    async fn update_member_status(&self, id: Uuid, status: &str) -> AtlasResult<LoyaltyMember>;
    async fn update_member_points(&self, id: Uuid, current_points: f64, lifetime_points: f64) -> AtlasResult<()>;
    async fn update_member_redeemed(&self, id: Uuid, redeemed_points: f64) -> AtlasResult<()>;
    async fn update_member_tier(&self, id: Uuid, tier_id: Uuid, tier_code: &str) -> AtlasResult<()>;
    async fn update_member_next_tier_remaining(&self, id: Uuid, remaining: Option<f64>) -> AtlasResult<()>;
    async fn delete_member(&self, org_id: Uuid, member_number: &str) -> AtlasResult<()>;

    // Point Transactions
    async fn create_transaction(
        &self, org_id: Uuid, program_id: Uuid, member_id: Uuid,
        transaction_number: &str, transaction_type: &str, points: f64,
        source_type: &str, source_id: Option<Uuid>, source_number: &str,
        description: &str, reference_amount: Option<f64>, reference_currency: &str,
        tier_bonus_applied: f64, promo_bonus_applied: f64, expiry_date: Option<chrono::NaiveDate>,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyPointTransaction>;
    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<LoyaltyPointTransaction>>;
    async fn get_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<LoyaltyPointTransaction>>;
    async fn list_transactions(&self, member_id: Uuid, txn_type: Option<&str>) -> AtlasResult<Vec<LoyaltyPointTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, reason: &str) -> AtlasResult<LoyaltyPointTransaction>;
    async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()>;

    // Rewards
    async fn create_reward(
        &self, org_id: Uuid, program_id: Uuid, reward_code: &str, name: &str,
        description: Option<&str>, reward_type: &str, points_required: f64,
        cash_value: f64, currency_code: &str, tier_restriction: &str,
        quantity_available: Option<i32>, max_per_member: Option<i32>,
        image_url: &str, is_active: bool,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyReward>;
    async fn get_reward(&self, id: Uuid) -> AtlasResult<Option<LoyaltyReward>>;
    async fn get_reward_by_code(&self, org_id: Uuid, reward_code: &str) -> AtlasResult<Option<LoyaltyReward>>;
    async fn list_rewards(&self, program_id: Uuid, reward_type: Option<&str>) -> AtlasResult<Vec<LoyaltyReward>>;
    async fn update_reward_active(&self, id: Uuid, is_active: bool) -> AtlasResult<LoyaltyReward>;
    async fn update_reward_claimed(&self, id: Uuid, quantity_claimed: i32) -> AtlasResult<()>;
    async fn delete_reward(&self, org_id: Uuid, reward_code: &str) -> AtlasResult<()>;

    // Redemptions
    async fn create_redemption(
        &self, org_id: Uuid, program_id: Uuid, member_id: Uuid, reward_id: Uuid,
        redemption_number: &str, points_spent: f64, quantity: i32, status: &str,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyRedemption>;
    async fn get_redemption(&self, id: Uuid) -> AtlasResult<Option<LoyaltyRedemption>>;
    async fn get_redemption_by_number(&self, org_id: Uuid, redemption_number: &str) -> AtlasResult<Option<LoyaltyRedemption>>;
    async fn list_redemptions(&self, member_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LoyaltyRedemption>>;
    async fn fulfill_redemption(&self, id: Uuid) -> AtlasResult<LoyaltyRedemption>;
    async fn cancel_redemption(&self, id: Uuid, reason: &str) -> AtlasResult<LoyaltyRedemption>;
    async fn count_member_redemptions(&self, member_id: Uuid, reward_id: Uuid) -> AtlasResult<i32>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LoyaltyDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresLoyaltyManagementRepository {
    pool: PgPool,
}

impl PostgresLoyaltyManagementRepository {
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
    None
}

fn row_to_program(row: &sqlx::postgres::PgRow) -> LoyaltyProgram {
    LoyaltyProgram {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_number: row.try_get("program_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        program_type: row.try_get("program_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        points_name: row.try_get("points_name").unwrap_or_default(),
        enrollment_type: row.try_get("enrollment_type").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: row.try_get("end_date").unwrap_or_default(),
        accrual_rate: get_numeric(row, "accrual_rate"),
        accrual_basis: row.try_get("accrual_basis").unwrap_or_default(),
        minimum_accrual_amount: get_numeric(row, "minimum_accrual_amount"),
        rounding_method: row.try_get("rounding_method").unwrap_or_default(),
        points_expiry_days: row.try_get("points_expiry_days").unwrap_or_default(),
        tier_qualification_period: row.try_get("tier_qualification_period").unwrap_or_default(),
        auto_upgrade: row.try_get("auto_upgrade").unwrap_or(true),
        auto_downgrade: row.try_get("auto_downgrade").unwrap_or(false),
        max_points_per_member: get_optional_numeric(row, "max_points_per_member"),
        allow_point_transfer: row.try_get("allow_point_transfer").unwrap_or(false),
        allow_redemption: row.try_get("allow_redemption").unwrap_or(true),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_tier(row: &sqlx::postgres::PgRow) -> LoyaltyTier {
    LoyaltyTier {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_id: row.try_get("program_id").unwrap_or_default(),
        tier_code: row.try_get("tier_code").unwrap_or_default(),
        tier_name: row.try_get("tier_name").unwrap_or_default(),
        tier_level: row.try_get("tier_level").unwrap_or_default(),
        minimum_points: get_numeric(row, "minimum_points"),
        maximum_points: get_optional_numeric(row, "maximum_points"),
        accrual_bonus_percentage: get_numeric(row, "accrual_bonus_percentage"),
        benefits: row.try_get("benefits").unwrap_or_default(),
        color: row.try_get("color").unwrap_or_default(),
        icon: row.try_get("icon").unwrap_or_default(),
        is_default: row.try_get("is_default").unwrap_or(false),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_member(row: &sqlx::postgres::PgRow) -> LoyaltyMember {
    LoyaltyMember {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_id: row.try_get("program_id").unwrap_or_default(),
        member_number: row.try_get("member_number").unwrap_or_default(),
        customer_id: row.try_get("customer_id").unwrap_or_default(),
        customer_name: row.try_get("customer_name").unwrap_or_default(),
        customer_email: row.try_get("customer_email").unwrap_or_default(),
        tier_id: row.try_get("tier_id").unwrap_or_default(),
        tier_code: row.try_get("tier_code").unwrap_or_default(),
        current_points: get_numeric(row, "current_points"),
        lifetime_points: get_numeric(row, "lifetime_points"),
        redeemed_points: get_numeric(row, "redeemed_points"),
        expired_points: get_numeric(row, "expired_points"),
        enrollment_date: row.try_get("enrollment_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        status: row.try_get("status").unwrap_or_default(),
        last_activity_date: row.try_get("last_activity_date").unwrap_or_default(),
        next_tier_points_remaining: get_optional_numeric(row, "next_tier_points_remaining"),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> LoyaltyPointTransaction {
    LoyaltyPointTransaction {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_id: row.try_get("program_id").unwrap_or_default(),
        member_id: row.try_get("member_id").unwrap_or_default(),
        transaction_number: row.try_get("transaction_number").unwrap_or_default(),
        transaction_type: row.try_get("transaction_type").unwrap_or_default(),
        points: get_numeric(row, "points"),
        source_type: row.try_get("source_type").unwrap_or_default(),
        source_id: row.try_get("source_id").unwrap_or_default(),
        source_number: row.try_get("source_number").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        reference_amount: get_optional_numeric(row, "reference_amount"),
        reference_currency: row.try_get("reference_currency").unwrap_or_default(),
        tier_bonus_applied: get_numeric(row, "tier_bonus_applied"),
        promo_bonus_applied: get_numeric(row, "promo_bonus_applied"),
        expiry_date: row.try_get("expiry_date").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        reversal_reason: row.try_get("reversal_reason").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_reward(row: &sqlx::postgres::PgRow) -> LoyaltyReward {
    LoyaltyReward {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_id: row.try_get("program_id").unwrap_or_default(),
        reward_code: row.try_get("reward_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        reward_type: row.try_get("reward_type").unwrap_or_default(),
        points_required: get_numeric(row, "points_required"),
        cash_value: get_numeric(row, "cash_value"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        tier_restriction: row.try_get("tier_restriction").unwrap_or_default(),
        quantity_available: row.try_get("quantity_available").unwrap_or_default(),
        quantity_claimed: row.try_get("quantity_claimed").unwrap_or_default(),
        max_per_member: row.try_get("max_per_member").unwrap_or_default(),
        image_url: row.try_get("image_url").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
        start_date: row.try_get("start_date").unwrap_or_default(),
        end_date: row.try_get("end_date").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_redemption(row: &sqlx::postgres::PgRow) -> LoyaltyRedemption {
    LoyaltyRedemption {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        program_id: row.try_get("program_id").unwrap_or_default(),
        member_id: row.try_get("member_id").unwrap_or_default(),
        reward_id: row.try_get("reward_id").unwrap_or_default(),
        redemption_number: row.try_get("redemption_number").unwrap_or_default(),
        points_spent: get_numeric(row, "points_spent"),
        quantity: row.try_get("quantity").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        fulfilled_at: row.try_get("fulfilled_at").unwrap_or_default(),
        cancelled_reason: row.try_get("cancelled_reason").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl LoyaltyManagementRepository for PostgresLoyaltyManagementRepository {
    // ========================================================================
    // Programs
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_program(
        &self, org_id: Uuid, program_number: &str, name: &str, description: Option<&str>,
        program_type: &str, currency_code: &str, points_name: &str, enrollment_type: &str,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        accrual_rate: f64, accrual_basis: &str, minimum_accrual_amount: f64,
        rounding_method: &str, points_expiry_days: Option<i32>,
        tier_qualification_period: &str, auto_upgrade: bool, auto_downgrade: bool,
        max_points_per_member: Option<f64>, allow_point_transfer: bool, allow_redemption: bool,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyProgram> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_programs
                (organization_id, program_number, name, description,
                 program_type, currency_code, points_name, enrollment_type,
                 start_date, end_date, accrual_rate, accrual_basis,
                 minimum_accrual_amount, rounding_method, points_expiry_days,
                 tier_qualification_period, auto_upgrade, auto_downgrade,
                 max_points_per_member, allow_point_transfer, allow_redemption,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, '{}'::jsonb, $23)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_number).bind(name).bind(description.unwrap_or(""))
        .bind(program_type).bind(currency_code).bind(points_name).bind(enrollment_type)
        .bind(start_date).bind(end_date).bind(accrual_rate).bind(accrual_basis)
        .bind(minimum_accrual_amount).bind(rounding_method).bind(points_expiry_days)
        .bind(tier_qualification_period).bind(auto_upgrade).bind(auto_downgrade)
        .bind(max_points_per_member).bind(allow_point_transfer).bind(allow_redemption)
        .bind(notes.unwrap_or("")).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_program(&row))
    }

    async fn get_program(&self, id: Uuid) -> AtlasResult<Option<LoyaltyProgram>> {
        let row = sqlx::query("SELECT * FROM _atlas.loyalty_programs WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_program))
    }

    async fn get_program_by_number(&self, org_id: Uuid, program_number: &str) -> AtlasResult<Option<LoyaltyProgram>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.loyalty_programs WHERE organization_id = $1 AND program_number = $2"
        ).bind(org_id).bind(program_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_program))
    }

    async fn list_programs(&self, org_id: Uuid, status: Option<&str>, program_type: Option<&str>) -> AtlasResult<Vec<LoyaltyProgram>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.loyalty_programs
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR program_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(program_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_program).collect())
    }

    async fn update_program_status(&self, id: Uuid, status: &str) -> AtlasResult<LoyaltyProgram> {
        let row = sqlx::query(
            "UPDATE _atlas.loyalty_programs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Loyalty program {} not found", id)))?;
        Ok(row_to_program(&row))
    }

    async fn delete_program(&self, org_id: Uuid, program_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.loyalty_programs WHERE organization_id = $1 AND program_number = $2 AND status = 'draft'"
        ).bind(org_id).bind(program_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Draft program '{}' not found", program_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Tiers
    // ========================================================================

    async fn create_tier(
        &self, org_id: Uuid, program_id: Uuid, tier_code: &str, tier_name: &str,
        tier_level: i32, minimum_points: f64, maximum_points: Option<f64>,
        accrual_bonus_percentage: f64, benefits: &str, color: &str, icon: &str,
        is_default: bool,
    ) -> AtlasResult<LoyaltyTier> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_tiers
                (organization_id, program_id, tier_code, tier_name, tier_level,
                 minimum_points, maximum_points, accrual_bonus_percentage,
                 benefits, color, icon, is_default, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(tier_code).bind(tier_name).bind(tier_level)
        .bind(minimum_points).bind(maximum_points).bind(accrual_bonus_percentage)
        .bind(benefits).bind(color).bind(icon).bind(is_default)
        .fetch_one(&self.pool).await?;
        Ok(row_to_tier(&row))
    }

    async fn list_tiers(&self, program_id: Uuid) -> AtlasResult<Vec<LoyaltyTier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.loyalty_tiers WHERE program_id = $1 ORDER BY tier_level"
        ).bind(program_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_tier).collect())
    }

    async fn delete_tier(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.loyalty_tiers WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Tier not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Members
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_member(
        &self, org_id: Uuid, program_id: Uuid, member_number: &str,
        customer_id: Option<Uuid>, customer_name: &str, customer_email: &str,
        tier_id: Option<Uuid>, tier_code: &str, enrollment_date: chrono::NaiveDate,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyMember> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_members
                (organization_id, program_id, member_number,
                 customer_id, customer_name, customer_email,
                 tier_id, tier_code, enrollment_date,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}'::jsonb, $11)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(member_number)
        .bind(customer_id).bind(customer_name).bind(customer_email)
        .bind(tier_id).bind(tier_code).bind(enrollment_date)
        .bind(notes.unwrap_or("")).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_member(&row))
    }

    async fn get_member(&self, id: Uuid) -> AtlasResult<Option<LoyaltyMember>> {
        let row = sqlx::query("SELECT * FROM _atlas.loyalty_members WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_member))
    }

    async fn get_member_by_number(&self, org_id: Uuid, member_number: &str) -> AtlasResult<Option<LoyaltyMember>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.loyalty_members WHERE organization_id = $1 AND member_number = $2"
        ).bind(org_id).bind(member_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_member))
    }

    async fn list_members(&self, program_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LoyaltyMember>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.loyalty_members
               WHERE program_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY enrollment_date DESC"#,
        ).bind(program_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_member).collect())
    }

    async fn update_member_status(&self, id: Uuid, status: &str) -> AtlasResult<LoyaltyMember> {
        let row = sqlx::query(
            "UPDATE _atlas.loyalty_members SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Member {} not found", id)))?;
        Ok(row_to_member(&row))
    }

    async fn update_member_points(&self, id: Uuid, current_points: f64, lifetime_points: f64) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.loyalty_members SET current_points = $2, lifetime_points = $3,
               last_activity_date = CURRENT_DATE, updated_at = now() WHERE id = $1"#,
        ).bind(id).bind(current_points).bind(lifetime_points)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn update_member_redeemed(&self, id: Uuid, redeemed_points: f64) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.loyalty_members SET redeemed_points = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(redeemed_points).execute(&self.pool).await?;
        Ok(())
    }

    async fn update_member_tier(&self, id: Uuid, tier_id: Uuid, tier_code: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.loyalty_members SET tier_id = $2, tier_code = $3, updated_at = now() WHERE id = $1"#
        ).bind(id).bind(tier_id).bind(tier_code).execute(&self.pool).await?;
        Ok(())
    }

    async fn update_member_next_tier_remaining(&self, id: Uuid, remaining: Option<f64>) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.loyalty_members SET next_tier_points_remaining = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(remaining).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_member(&self, org_id: Uuid, member_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.loyalty_members WHERE organization_id = $1 AND member_number = $2"
        ).bind(org_id).bind(member_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Member '{}' not found", member_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Point Transactions
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_transaction(
        &self, org_id: Uuid, program_id: Uuid, member_id: Uuid,
        transaction_number: &str, transaction_type: &str, points: f64,
        source_type: &str, source_id: Option<Uuid>, source_number: &str,
        description: &str, reference_amount: Option<f64>, reference_currency: &str,
        tier_bonus_applied: f64, promo_bonus_applied: f64, expiry_date: Option<chrono::NaiveDate>,
        status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyPointTransaction> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_point_transactions
                (organization_id, program_id, member_id, transaction_number,
                 transaction_type, points, source_type, source_id, source_number,
                 description, reference_amount, reference_currency,
                 tier_bonus_applied, promo_bonus_applied, expiry_date,
                 status, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, '{}'::jsonb, $17)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(member_id).bind(transaction_number)
        .bind(transaction_type).bind(points).bind(source_type).bind(source_id).bind(source_number)
        .bind(description).bind(reference_amount).bind(reference_currency)
        .bind(tier_bonus_applied).bind(promo_bonus_applied).bind(expiry_date)
        .bind(status).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<LoyaltyPointTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.loyalty_point_transactions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_transaction))
    }

    async fn get_transaction_by_number(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<LoyaltyPointTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.loyalty_point_transactions WHERE organization_id = $1 AND transaction_number = $2"
        ).bind(org_id).bind(transaction_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_transaction))
    }

    async fn list_transactions(&self, member_id: Uuid, txn_type: Option<&str>) -> AtlasResult<Vec<LoyaltyPointTransaction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.loyalty_point_transactions
               WHERE member_id = $1 AND ($2::text IS NULL OR transaction_type = $2)
               ORDER BY created_at DESC"#,
        ).bind(member_id).bind(txn_type).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_transaction).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, reason: &str) -> AtlasResult<LoyaltyPointTransaction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.loyalty_point_transactions SET status = $2, reversal_reason = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Transaction {} not found", id)))?;
        Ok(row_to_transaction(&row))
    }

    async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.loyalty_point_transactions WHERE organization_id = $1 AND transaction_number = $2"
        ).bind(org_id).bind(transaction_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Transaction '{}' not found", transaction_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Rewards
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_reward(
        &self, org_id: Uuid, program_id: Uuid, reward_code: &str, name: &str,
        description: Option<&str>, reward_type: &str, points_required: f64,
        cash_value: f64, currency_code: &str, tier_restriction: &str,
        quantity_available: Option<i32>, max_per_member: Option<i32>,
        image_url: &str, is_active: bool,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyReward> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_rewards
                (organization_id, program_id, reward_code, name, description,
                 reward_type, points_required, cash_value, currency_code,
                 tier_restriction, quantity_available, max_per_member,
                 image_url, is_active, start_date, end_date,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                    $13, $14, $15, $16, $17, '{}'::jsonb, $18)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(reward_code).bind(name).bind(description.unwrap_or(""))
        .bind(reward_type).bind(points_required).bind(cash_value).bind(currency_code)
        .bind(tier_restriction).bind(quantity_available).bind(max_per_member)
        .bind(image_url).bind(is_active).bind(start_date).bind(end_date)
        .bind(notes.unwrap_or("")).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_reward(&row))
    }

    async fn get_reward(&self, id: Uuid) -> AtlasResult<Option<LoyaltyReward>> {
        let row = sqlx::query("SELECT * FROM _atlas.loyalty_rewards WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_reward))
    }

    async fn get_reward_by_code(&self, org_id: Uuid, reward_code: &str) -> AtlasResult<Option<LoyaltyReward>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.loyalty_rewards WHERE organization_id = $1 AND reward_code = $2"
        ).bind(org_id).bind(reward_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_reward))
    }

    async fn list_rewards(&self, program_id: Uuid, reward_type: Option<&str>) -> AtlasResult<Vec<LoyaltyReward>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.loyalty_rewards
               WHERE program_id = $1 AND ($2::text IS NULL OR reward_type = $2)
               ORDER BY points_required ASC"#,
        ).bind(program_id).bind(reward_type).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_reward).collect())
    }

    async fn update_reward_active(&self, id: Uuid, is_active: bool) -> AtlasResult<LoyaltyReward> {
        let row = sqlx::query(
            "UPDATE _atlas.loyalty_rewards SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Reward {} not found", id)))?;
        Ok(row_to_reward(&row))
    }

    async fn update_reward_claimed(&self, id: Uuid, quantity_claimed: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.loyalty_rewards SET quantity_claimed = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(quantity_claimed).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_reward(&self, org_id: Uuid, reward_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.loyalty_rewards WHERE organization_id = $1 AND reward_code = $2"
        ).bind(org_id).bind(reward_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Reward '{}' not found", reward_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Redemptions
    // ========================================================================

    async fn create_redemption(
        &self, org_id: Uuid, program_id: Uuid, member_id: Uuid, reward_id: Uuid,
        redemption_number: &str, points_spent: f64, quantity: i32, status: &str,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyRedemption> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.loyalty_redemptions
                (organization_id, program_id, member_id, reward_id,
                 redemption_number, points_spent, quantity, status,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '{}'::jsonb, $10)
            RETURNING *"#,
        )
        .bind(org_id).bind(program_id).bind(member_id).bind(reward_id)
        .bind(redemption_number).bind(points_spent).bind(quantity).bind(status)
        .bind(notes.unwrap_or("")).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_redemption(&row))
    }

    async fn get_redemption(&self, id: Uuid) -> AtlasResult<Option<LoyaltyRedemption>> {
        let row = sqlx::query("SELECT * FROM _atlas.loyalty_redemptions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_redemption))
    }

    async fn get_redemption_by_number(&self, org_id: Uuid, redemption_number: &str) -> AtlasResult<Option<LoyaltyRedemption>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.loyalty_redemptions WHERE organization_id = $1 AND redemption_number = $2"
        ).bind(org_id).bind(redemption_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_redemption))
    }

    async fn list_redemptions(&self, member_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LoyaltyRedemption>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.loyalty_redemptions
               WHERE member_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY created_at DESC"#,
        ).bind(member_id).bind(status).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_redemption).collect())
    }

    async fn fulfill_redemption(&self, id: Uuid) -> AtlasResult<LoyaltyRedemption> {
        let row = sqlx::query(
            r#"UPDATE _atlas.loyalty_redemptions
               SET status = 'fulfilled', fulfilled_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Redemption {} not found", id)))?;
        Ok(row_to_redemption(&row))
    }

    async fn cancel_redemption(&self, id: Uuid, reason: &str) -> AtlasResult<LoyaltyRedemption> {
        let row = sqlx::query(
            r#"UPDATE _atlas.loyalty_redemptions
               SET status = 'cancelled', cancelled_reason = $2, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Redemption {} not found", id)))?;
        Ok(row_to_redemption(&row))
    }

    async fn count_member_redemptions(&self, member_id: Uuid, reward_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM(quantity), 0) as cnt FROM _atlas.loyalty_redemptions
               WHERE member_id = $1 AND reward_id = $2 AND status != 'cancelled'"#
        ).bind(member_id).bind(reward_id).fetch_one(&self.pool).await?;
        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count as i32)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LoyaltyDashboard> {
        let programs = sqlx::query(
            "SELECT status FROM _atlas.loyalty_programs WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_programs = programs.len() as i64;
        let active_programs = programs.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i64;

        let member_stats = sqlx::query(
            r#"SELECT COUNT(*) as cnt,
                      COUNT(CASE WHEN status = 'active' THEN 1 END) as active_cnt
               FROM _atlas.loyalty_members WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_members: i64 = member_stats.try_get("cnt").unwrap_or(0);
        let active_members: i64 = member_stats.try_get("active_cnt").unwrap_or(0);

        let txn_stats = sqlx::query(
            r#"SELECT
                COALESCE(SUM(CASE WHEN transaction_type = 'accrual' THEN points ELSE 0 END), 0) as issued,
                COALESCE(SUM(CASE WHEN transaction_type = 'redemption' THEN ABS(points) ELSE 0 END), 0) as redeemed,
                COALESCE(SUM(CASE WHEN transaction_type = 'expiration' THEN ABS(points) ELSE 0 END), 0) as expired
               FROM _atlas.loyalty_point_transactions WHERE organization_id = $1 AND status = 'posted'"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_points_issued: f64 = get_numeric(&txn_stats, "issued");
        let total_points_redeemed: f64 = get_numeric(&txn_stats, "redeemed");
        let total_points_expired: f64 = get_numeric(&txn_stats, "expired");

        let red_stats = sqlx::query(
            r#"SELECT COUNT(*) as total,
                      COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending
               FROM _atlas.loyalty_redemptions WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_redemptions: i64 = red_stats.try_get("total").unwrap_or(0);
        let pending_redemptions: i64 = red_stats.try_get("pending").unwrap_or(0);

        // Members by tier
        let tier_stats = sqlx::query(
            r#"SELECT tier_code, COUNT(*) as cnt FROM _atlas.loyalty_members
               WHERE organization_id = $1 AND status = 'active'
               GROUP BY tier_code"#
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut by_tier = serde_json::Map::new();
        for row in &tier_stats {
            let tc: String = row.try_get("tier_code").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            by_tier.insert(tc, serde_json::Value::from(cnt));
        }

        Ok(LoyaltyDashboard {
            organization_id: org_id,
            total_programs,
            active_programs,
            total_members,
            active_members,
            total_points_issued,
            total_points_redeemed,
            total_points_expired,
            total_redemptions,
            pending_redemptions,
            members_by_tier: serde_json::Value::Object(by_tier),
            top_members: serde_json::json!([]),
            recent_transactions: serde_json::json!([]),
        })
    }
}
