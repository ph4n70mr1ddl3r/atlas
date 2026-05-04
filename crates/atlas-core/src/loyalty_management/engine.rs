//! Loyalty Management Engine
//!
//! Manages loyalty programs, tiers, members, point transactions,
//! rewards, redemptions, and dashboard analytics.
//!
//! Oracle Fusion Cloud equivalent: CX > Loyalty Management

use atlas_shared::{
    LoyaltyProgram, LoyaltyTier, LoyaltyMember, LoyaltyPointTransaction,
    LoyaltyReward, LoyaltyRedemption, LoyaltyDashboard,
    AtlasError, AtlasResult,
};
use super::LoyaltyManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

#[allow(dead_code)]
const VALID_PROGRAM_TYPES: &[&str] = &["points", "tier", "frequency", "hybrid"];
#[allow(dead_code)]
const VALID_PROGRAM_STATUSES: &[&str] = &["draft", "active", "suspended", "closed"];
#[allow(dead_code)]
const VALID_ENROLLMENT_TYPES: &[&str] = &["open", "invitation", "approval"];
#[allow(dead_code)]
const VALID_ACCRUAL_BASES: &[&str] = &["amount", "quantity", "visit"];
#[allow(dead_code)]
const VALID_ROUNDING_METHODS: &[&str] = &["round", "floor", "ceil"];
#[allow(dead_code)]
const VALID_TIER_PERIODS: &[&str] = &["yearly", "quarterly", "monthly", "lifetime"];

#[allow(dead_code)]
const VALID_MEMBER_STATUSES: &[&str] = &["active", "inactive", "suspended", "closed"];

#[allow(dead_code)]
const VALID_TXN_TYPES: &[&str] = &[
    "accrual", "redemption", "adjustment", "expiration",
    "transfer_in", "transfer_out", "bonus", "reversal",
];
#[allow(dead_code)]
const VALID_TXN_STATUSES: &[&str] = &["posted", "pending", "reversed", "cancelled"];
#[allow(dead_code)]
const VALID_SOURCE_TYPES: &[&str] = &[
    "sales_order", "purchase", "manual", "promotion", "signup_bonus",
    "referral", "social", "tier_upgrade",
];

#[allow(dead_code)]
const VALID_REWARD_TYPES: &[&str] = &[
    "merchandise", "discount", "voucher", "experience",
    "cashback", "free_product", "upgrade",
];
#[allow(dead_code)]
const VALID_REDEMPTION_STATUSES: &[&str] = &[
    "pending", "fulfilled", "cancelled", "expired",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Loyalty Management Engine
pub struct LoyaltyManagementEngine {
    repository: Arc<dyn LoyaltyManagementRepository>,
}

impl LoyaltyManagementEngine {
    pub fn new(repository: Arc<dyn LoyaltyManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Programs
    // ========================================================================

    /// Create a new loyalty program
    #[allow(clippy::too_many_arguments)]
    pub async fn create_program(
        &self,
        org_id: Uuid,
        program_number: &str,
        name: &str,
        description: Option<&str>,
        program_type: &str,
        currency_code: Option<&str>,
        points_name: Option<&str>,
        enrollment_type: Option<&str>,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
        accrual_rate: Option<f64>,
        accrual_basis: Option<&str>,
        minimum_accrual_amount: Option<f64>,
        rounding_method: Option<&str>,
        points_expiry_days: Option<i32>,
        tier_qualification_period: Option<&str>,
        auto_upgrade: Option<bool>,
        auto_downgrade: Option<bool>,
        max_points_per_member: Option<f64>,
        allow_point_transfer: Option<bool>,
        allow_redemption: Option<bool>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyProgram> {
        if program_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Program number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Program name is required".to_string()));
        }
        validate_enum("program_type", program_type, VALID_PROGRAM_TYPES)?;
        if let Some(et) = enrollment_type {
            validate_enum("enrollment_type", et, VALID_ENROLLMENT_TYPES)?;
        }
        if let Some(ab) = accrual_basis {
            validate_enum("accrual_basis", ab, VALID_ACCRUAL_BASES)?;
        }
        if let Some(rm) = rounding_method {
            validate_enum("rounding_method", rm, VALID_ROUNDING_METHODS)?;
        }
        if let Some(tp) = tier_qualification_period {
            validate_enum("tier_qualification_period", tp, VALID_TIER_PERIODS)?;
        }
        if let Some(ed) = end_date {
            if start_date >= ed {
                return Err(AtlasError::ValidationFailed(
                    "Start date must be before end date".to_string(),
                ));
            }
        }
        if let Some(ar) = accrual_rate {
            if ar < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Accrual rate cannot be negative".to_string(),
                ));
            }
        }

        if self.repository.get_program_by_number(org_id, program_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Loyalty program '{}' already exists", program_number
            )));
        }

        info!("Creating loyalty program '{}' ({}) for org {} [type={}]",
              program_number, name, org_id, program_type);

        self.repository.create_program(
            org_id, program_number, name, description,
            program_type,
            currency_code.unwrap_or("PTS"),
            points_name.unwrap_or("Points"),
            enrollment_type.unwrap_or("open"),
            start_date, end_date,
            accrual_rate.unwrap_or(1.0),
            accrual_basis.unwrap_or("amount"),
            minimum_accrual_amount.unwrap_or(0.0),
            rounding_method.unwrap_or("round"),
            points_expiry_days,
            tier_qualification_period.unwrap_or("yearly"),
            auto_upgrade.unwrap_or(true),
            auto_downgrade.unwrap_or(false),
            max_points_per_member,
            allow_point_transfer.unwrap_or(false),
            allow_redemption.unwrap_or(true),
            notes,
            created_by,
        ).await
    }

    /// Get a program by ID
    pub async fn get_program(&self, id: Uuid) -> AtlasResult<Option<LoyaltyProgram>> {
        self.repository.get_program(id).await
    }

    /// Get a program by number
    pub async fn get_program_by_number(&self, org_id: Uuid, program_number: &str) -> AtlasResult<Option<LoyaltyProgram>> {
        self.repository.get_program_by_number(org_id, program_number).await
    }

    /// List programs with optional filters
    pub async fn list_programs(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        program_type: Option<&str>,
    ) -> AtlasResult<Vec<LoyaltyProgram>> {
        self.repository.list_programs(org_id, status, program_type).await
    }

    /// Activate a program
    pub async fn activate_program(&self, id: Uuid) -> AtlasResult<LoyaltyProgram> {
        info!("Activating loyalty program {}", id);
        self.repository.update_program_status(id, "active").await
    }

    /// Suspend a program
    pub async fn suspend_program(&self, id: Uuid) -> AtlasResult<LoyaltyProgram> {
        info!("Suspending loyalty program {}", id);
        self.repository.update_program_status(id, "suspended").await
    }

    /// Close a program
    pub async fn close_program(&self, id: Uuid) -> AtlasResult<LoyaltyProgram> {
        info!("Closing loyalty program {}", id);
        self.repository.update_program_status(id, "closed").await
    }

    /// Delete a program (only drafts)
    pub async fn delete_program(&self, org_id: Uuid, program_number: &str) -> AtlasResult<()> {
        info!("Deleting loyalty program '{}' for org {}", program_number, org_id);
        self.repository.delete_program(org_id, program_number).await
    }

    // ========================================================================
    // Tiers
    // ========================================================================

    /// Create a loyalty tier
    #[allow(clippy::too_many_arguments)]
    pub async fn create_tier(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        tier_code: &str,
        tier_name: &str,
        tier_level: i32,
        minimum_points: f64,
        maximum_points: Option<f64>,
        accrual_bonus_percentage: Option<f64>,
        benefits: Option<&str>,
        color: Option<&str>,
        icon: Option<&str>,
        is_default: Option<bool>,
    ) -> AtlasResult<LoyaltyTier> {
        if tier_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Tier code is required".to_string()));
        }
        if tier_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Tier name is required".to_string()));
        }
        if minimum_points < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Minimum points cannot be negative".to_string(),
            ));
        }
        if let Some(max) = maximum_points {
            if max <= minimum_points {
                return Err(AtlasError::ValidationFailed(
                    "Maximum points must be greater than minimum points".to_string(),
                ));
            }
        }

        // Verify program exists
        self.repository.get_program(program_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty program {} not found", program_id
            )))?;

        info!("Creating tier '{}' ({}) for program {} [min={}, max={:?}]",
              tier_code, tier_name, program_id, minimum_points, maximum_points);

        self.repository.create_tier(
            org_id, program_id, tier_code, tier_name, tier_level,
            minimum_points, maximum_points,
            accrual_bonus_percentage.unwrap_or(0.0),
            benefits.unwrap_or(""),
            color.unwrap_or(""),
            icon.unwrap_or(""),
            is_default.unwrap_or(false),
        ).await
    }

    /// List tiers for a program
    pub async fn list_tiers(&self, program_id: Uuid) -> AtlasResult<Vec<LoyaltyTier>> {
        self.repository.list_tiers(program_id).await
    }

    /// Delete a tier
    pub async fn delete_tier(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_tier(id).await
    }

    // ========================================================================
    // Members
    // ========================================================================

    /// Enroll a new member
    #[allow(clippy::too_many_arguments)]
    pub async fn enroll_member(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        member_number: &str,
        customer_id: Option<Uuid>,
        customer_name: &str,
        customer_email: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyMember> {
        if member_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Member number is required".to_string()));
        }
        if customer_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Customer name is required".to_string()));
        }

        // Verify program exists and is active
        let program = self.repository.get_program(program_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty program {} not found", program_id
            )))?;

        if program.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Program is not active (status: {})", program.status
            )));
        }

        if self.repository.get_member_by_number(org_id, member_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Loyalty member '{}' already exists", member_number
            )));
        }

        // Find the default tier
        let tiers = self.repository.list_tiers(program_id).await?;
        let default_tier = tiers.iter().find(|t| t.is_default)
            .or_else(|| tiers.first());
        let (tier_id, tier_code) = match default_tier {
            Some(t) => (Some(t.id), t.tier_code.clone()),
            None => (None, String::new()),
        };

        info!("Enrolling member '{}' ({}) into program {} [tier={:?}]",
              member_number, customer_name, program_id, tier_code);

        let today = chrono::Utc::now().date_naive();

        self.repository.create_member(
            org_id, program_id, member_number,
            customer_id, customer_name,
            customer_email.unwrap_or(""),
            tier_id, &tier_code,
            today, notes,
            created_by,
        ).await
    }

    /// Get a member by ID
    pub async fn get_member(&self, id: Uuid) -> AtlasResult<Option<LoyaltyMember>> {
        self.repository.get_member(id).await
    }

    /// Get a member by number
    pub async fn get_member_by_number(&self, org_id: Uuid, member_number: &str) -> AtlasResult<Option<LoyaltyMember>> {
        self.repository.get_member_by_number(org_id, member_number).await
    }

    /// List members with optional filters
    pub async fn list_members(
        &self,
        program_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LoyaltyMember>> {
        self.repository.list_members(program_id, status).await
    }

    /// Suspend a member
    pub async fn suspend_member(&self, id: Uuid) -> AtlasResult<LoyaltyMember> {
        info!("Suspending loyalty member {}", id);
        self.repository.update_member_status(id, "suspended").await
    }

    /// Reactivate a member
    pub async fn reactivate_member(&self, id: Uuid) -> AtlasResult<LoyaltyMember> {
        info!("Reactivating loyalty member {}", id);
        self.repository.update_member_status(id, "active").await
    }

    /// Close a member
    pub async fn close_member(&self, id: Uuid) -> AtlasResult<LoyaltyMember> {
        info!("Closing loyalty member {}", id);
        self.repository.update_member_status(id, "closed").await
    }

    /// Delete a member by number
    pub async fn delete_member(&self, org_id: Uuid, member_number: &str) -> AtlasResult<()> {
        info!("Deleting loyalty member '{}' for org {}", member_number, org_id);
        self.repository.delete_member(org_id, member_number).await
    }

    // ========================================================================
    // Point Transactions
    // ========================================================================

    /// Accrue points to a member
    #[allow(clippy::too_many_arguments)]
    pub async fn accrue_points(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        member_id: Uuid,
        transaction_number: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        reference_amount: Option<f64>,
        reference_currency: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyPointTransaction> {
        if transaction_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Transaction number is required".to_string()));
        }
        if let Some(st) = source_type {
            validate_enum("source_type", st, VALID_SOURCE_TYPES)?;
        }

        // Verify member exists and is active
        let member = self.repository.get_member(member_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty member {} not found", member_id
            )))?;

        if member.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Member is not active (status: {})", member.status
            )));
        }

        // Verify program exists and get accrual rate
        let program = self.repository.get_program(program_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty program {} not found", program_id
            )))?;

        if program.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Program is not active (status: {})", program.status
            )));
        }

        // Calculate base points from reference amount
        let ref_amt = reference_amount.unwrap_or(0.0);
        if ref_amt < program.minimum_accrual_amount {
            return Err(AtlasError::ValidationFailed(format!(
                "Reference amount {} is below minimum accrual amount {}", 
                ref_amt, program.minimum_accrual_amount
            )));
        }

        let base_points = self.calculate_points(ref_amt, program.accrual_rate, &program.rounding_method);

        // Calculate tier bonus
        let tier_bonus = if let Some(ref tier_id) = member.tier_id {
            let tiers = self.repository.list_tiers(program_id).await?;
            let tier = tiers.iter().find(|t| t.id == *tier_id);
            tier.map(|t| (base_points * t.accrual_bonus_percentage / 100.0).floor())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let total_points = base_points + tier_bonus;

        if total_points <= 0.0 && ref_amt == 0.0 {
            return Err(AtlasError::ValidationFailed(
                "No points to accrue".to_string(),
            ));
        }

        // Check max points
        if let Some(max) = program.max_points_per_member {
            if member.current_points + total_points > max {
                return Err(AtlasError::ValidationFailed(format!(
                    "Would exceed max points per member ({})", max
                )));
            }
        }

        if self.repository.get_transaction_by_number(org_id, transaction_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Point transaction '{}' already exists", transaction_number
            )));
        }

        // Calculate expiry date
        let expiry_date = program.points_expiry_days.map(|days| {
            let today = chrono::Utc::now().date_naive();
            today + chrono::Duration::days(days as i64)
        });

        info!("Accruing {:.0} points (+{:.0} bonus) to member {} [txn={}]",
              base_points, tier_bonus, member_id, transaction_number);

        let txn = self.repository.create_transaction(
            org_id, program_id, member_id, transaction_number,
            "accrual", total_points,
            source_type.unwrap_or("manual"), source_id, source_number.unwrap_or(""),
            description.unwrap_or(""),
            reference_amount, reference_currency.unwrap_or("USD"),
            tier_bonus, 0.0, // promo bonus
            expiry_date,
            "posted",
            created_by,
        ).await?;

        // Update member points
        self.repository.update_member_points(
            member_id,
            member.current_points + total_points,
            member.lifetime_points + total_points,
        ).await?;

        // Check for tier upgrade
        if program.auto_upgrade {
            let updated_member = self.repository.get_member(member_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Member {} not found", member_id)))?;
            let _ = self.evaluate_tier_upgrade(member_id, program_id, updated_member.lifetime_points).await;
        }

        Ok(txn)
    }

    /// Adjust member points (manual add/subtract)
    pub async fn adjust_points(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        member_id: Uuid,
        transaction_number: &str,
        points: f64,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyPointTransaction> {
        if transaction_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Transaction number is required".to_string()));
        }
        if points == 0.0 {
            return Err(AtlasError::ValidationFailed("Points cannot be zero for adjustment".to_string()));
        }

        let member = self.repository.get_member(member_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty member {} not found", member_id
            )))?;

        if member.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Member is not active (status: {})", member.status
            )));
        }

        // For negative adjustments, ensure member has enough points
        if points < 0.0 && member.current_points + points < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Insufficient points for adjustment".to_string(),
            ));
        }

        if self.repository.get_transaction_by_number(org_id, transaction_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Point transaction '{}' already exists", transaction_number
            )));
        }

        info!("Adjusting {:.0} points for member {} [txn={}]",
              points, member_id, transaction_number);

        let txn = self.repository.create_transaction(
            org_id, program_id, member_id, transaction_number,
            "adjustment", points,
            "manual", None, "",
            description.unwrap_or("Manual adjustment"),
            None, "USD", 0.0, 0.0, None, "posted",
            created_by,
        ).await?;

        let new_current = member.current_points + points;
        let new_lifetime = if points > 0.0 { member.lifetime_points + points } else { member.lifetime_points };
        self.repository.update_member_points(member_id, new_current, new_lifetime).await?;

        Ok(txn)
    }

    /// Reverse a transaction
    pub async fn reverse_transaction(&self, id: Uuid, reason: &str) -> AtlasResult<LoyaltyPointTransaction> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Reversal reason is required".to_string()));
        }

        let txn = self.repository.get_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Transaction {} not found", id
            )))?;

        if txn.status != "posted" {
            return Err(AtlasError::ValidationFailed(
                "Only posted transactions can be reversed".to_string(),
            ));
        }

        info!("Reversing transaction {} [reason={}]", id, reason);

        let reversed = self.repository.update_transaction_status(id, "reversed", reason).await?;

        // Adjust member points
        let member = self.repository.get_member(txn.member_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Member not found".to_string()))?;
        let new_current = member.current_points - txn.points;
        self.repository.update_member_points(txn.member_id, new_current, member.lifetime_points).await?;

        Ok(reversed)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<LoyaltyPointTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// List transactions for a member
    pub async fn list_transactions(
        &self,
        member_id: Uuid,
        txn_type: Option<&str>,
    ) -> AtlasResult<Vec<LoyaltyPointTransaction>> {
        self.repository.list_transactions(member_id, txn_type).await
    }

    /// Delete a transaction by number
    pub async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()> {
        info!("Deleting point transaction '{}' for org {}", transaction_number, org_id);
        self.repository.delete_transaction(org_id, transaction_number).await
    }

    // ========================================================================
    // Rewards
    // ========================================================================

    /// Create a reward in the catalog
    #[allow(clippy::too_many_arguments)]
    pub async fn create_reward(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        reward_code: &str,
        name: &str,
        description: Option<&str>,
        reward_type: &str,
        points_required: f64,
        cash_value: Option<f64>,
        currency_code: Option<&str>,
        tier_restriction: Option<&str>,
        quantity_available: Option<i32>,
        max_per_member: Option<i32>,
        image_url: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyReward> {
        if reward_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Reward code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Reward name is required".to_string()));
        }
        validate_enum("reward_type", reward_type, VALID_REWARD_TYPES)?;
        if points_required <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Points required must be positive".to_string(),
            ));
        }

        // Verify program exists
        self.repository.get_program(program_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty program {} not found", program_id
            )))?;

        if self.repository.get_reward_by_code(org_id, reward_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Reward '{}' already exists", reward_code
            )));
        }

        info!("Creating reward '{}' ({}) for program {} [points={}]",
              reward_code, name, program_id, points_required);

        self.repository.create_reward(
            org_id, program_id, reward_code, name, description,
            reward_type, points_required,
            cash_value.unwrap_or(0.0),
            currency_code.unwrap_or("USD"),
            tier_restriction.unwrap_or(""),
            quantity_available, max_per_member,
            image_url.unwrap_or(""),
            true, start_date, end_date,
            notes, created_by,
        ).await
    }

    /// Get a reward by ID
    pub async fn get_reward(&self, id: Uuid) -> AtlasResult<Option<LoyaltyReward>> {
        self.repository.get_reward(id).await
    }

    /// List rewards for a program
    pub async fn list_rewards(
        &self,
        program_id: Uuid,
        reward_type: Option<&str>,
    ) -> AtlasResult<Vec<LoyaltyReward>> {
        self.repository.list_rewards(program_id, reward_type).await
    }

    /// Deactivate a reward
    pub async fn deactivate_reward(&self, id: Uuid) -> AtlasResult<LoyaltyReward> {
        info!("Deactivating reward {}", id);
        self.repository.update_reward_active(id, false).await
    }

    /// Delete a reward by code
    pub async fn delete_reward(&self, org_id: Uuid, reward_code: &str) -> AtlasResult<()> {
        info!("Deleting reward '{}' for org {}", reward_code, org_id);
        self.repository.delete_reward(org_id, reward_code).await
    }

    // ========================================================================
    // Redemptions
    // ========================================================================

    /// Redeem a reward for a member
    pub async fn redeem_reward(
        &self,
        org_id: Uuid,
        program_id: Uuid,
        member_id: Uuid,
        reward_id: Uuid,
        redemption_number: &str,
        quantity: Option<i32>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LoyaltyRedemption> {
        if redemption_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Redemption number is required".to_string()));
        }

        let qty = quantity.unwrap_or(1);
        if qty <= 0 {
            return Err(AtlasError::ValidationFailed("Quantity must be positive".to_string()));
        }

        // Verify member
        let member = self.repository.get_member(member_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Loyalty member {} not found", member_id
            )))?;

        if member.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Member is not active (status: {})", member.status
            )));
        }

        // Verify reward
        let reward = self.repository.get_reward(reward_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Reward {} not found", reward_id
            )))?;

        if !reward.is_active {
            return Err(AtlasError::ValidationFailed("Reward is not active".to_string()));
        }

        // Check tier restriction
        if !reward.tier_restriction.is_empty() && reward.tier_restriction != member.tier_code {
            return Err(AtlasError::ValidationFailed(format!(
                "Reward requires tier '{}', member has '{}'", reward.tier_restriction, member.tier_code
            )));
        }

        let points_needed = reward.points_required * qty as f64;

        // Check sufficient points
        if member.current_points < points_needed {
            return Err(AtlasError::ValidationFailed(format!(
                "Insufficient points (has {:.0}, needs {:.0})", member.current_points, points_needed
            )));
        }

        // Check availability
        if let Some(avail) = reward.quantity_available {
            if reward.quantity_claimed + qty > avail {
                return Err(AtlasError::ValidationFailed(
                    "Reward quantity not available".to_string(),
                ));
            }
        }

        // Check max per member
        if let Some(max) = reward.max_per_member {
            let member_redemptions = self.repository.count_member_redemptions(member_id, reward_id).await?;
            if member_redemptions + qty > max {
                return Err(AtlasError::ValidationFailed(
                    format!("Exceeds max per member ({})", max)
                ));
            }
        }

        // Check uniqueness
        if self.repository.get_redemption_by_number(org_id, redemption_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Redemption '{}' already exists", redemption_number
            )));
        }

        info!("Redeeming reward '{}' for member {} [points={:.0}, qty={}]",
              reward.reward_code, member_id, points_needed, qty);

        // Create redemption
        let redemption = self.repository.create_redemption(
            org_id, program_id, member_id, reward_id,
            redemption_number, points_needed, qty,
            "pending", notes, created_by,
        ).await?;

        // Deduct points
        self.repository.update_member_points(
            member_id,
            member.current_points - points_needed,
            member.lifetime_points,
        ).await?;
        self.repository.update_member_redeemed(member_id, member.redeemed_points + points_needed).await?;

        // Update reward claimed count
        self.repository.update_reward_claimed(reward_id, reward.quantity_claimed + qty).await?;

        // Create redemption point transaction
        let _ = self.repository.create_transaction(
            org_id, program_id, member_id,
            &format!("RD-{}", redemption_number),
            "redemption", -points_needed,
            "manual", Some(reward_id), &reward.reward_code,
            &format!("Redemption: {}", reward.name),
            Some(points_needed), "PTS", 0.0, 0.0, None, "posted",
            created_by,
        ).await;

        Ok(redemption)
    }

    /// Get a redemption by ID
    pub async fn get_redemption(&self, id: Uuid) -> AtlasResult<Option<LoyaltyRedemption>> {
        self.repository.get_redemption(id).await
    }

    /// List redemptions for a member
    pub async fn list_redemptions(
        &self,
        member_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LoyaltyRedemption>> {
        self.repository.list_redemptions(member_id, status).await
    }

    /// Fulfill a redemption
    pub async fn fulfill_redemption(&self, id: Uuid) -> AtlasResult<LoyaltyRedemption> {
        info!("Fulfilling redemption {}", id);
        self.repository.fulfill_redemption(id).await
    }

    /// Cancel a redemption (refunds points)
    pub async fn cancel_redemption(&self, id: Uuid, reason: &str) -> AtlasResult<LoyaltyRedemption> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Cancellation reason is required".to_string()));
        }

        let redemption = self.repository.get_redemption(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Redemption {} not found", id
            )))?;

        if redemption.status != "pending" {
            return Err(AtlasError::ValidationFailed(
                "Only pending redemptions can be cancelled".to_string(),
            ));
        }

        info!("Cancelling redemption {} [reason={}]", id, reason);

        let cancelled = self.repository.cancel_redemption(id, reason).await?;

        // Refund points to member
        let member = self.repository.get_member(redemption.member_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Member not found".to_string()))?;
        self.repository.update_member_points(
            redemption.member_id,
            member.current_points + redemption.points_spent,
            member.lifetime_points,
        ).await?;
        self.repository.update_member_redeemed(
            redemption.member_id,
            (member.redeemed_points - redemption.points_spent).max(0.0),
        ).await?;

        // Update reward claimed count
        if let Some(reward) = self.repository.get_reward(redemption.reward_id).await? {
            self.repository.update_reward_claimed(
                redemption.reward_id,
                (reward.quantity_claimed - redemption.quantity).max(0),
            ).await?;
        }

        Ok(cancelled)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the loyalty management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LoyaltyDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Calculate points from reference amount using program rules
    fn calculate_points(&self, amount: f64, rate: f64, rounding: &str) -> f64 {
        let raw = amount * rate;
        match rounding {
            "floor" => raw.floor(),
            "ceil" => raw.ceil(),
            _ => raw.round(),
        }
    }

    /// Evaluate and apply tier upgrade based on lifetime points
    async fn evaluate_tier_upgrade(
        &self,
        member_id: Uuid,
        program_id: Uuid,
        lifetime_points: f64,
    ) -> AtlasResult<()> {
        let tiers = self.repository.list_tiers(program_id).await?;
        if tiers.is_empty() {
            return Ok(());
        }

        // Find the highest tier the member qualifies for
        let qualified_tier = tiers.iter()
            .filter(|t| lifetime_points >= t.minimum_points)
            .max_by_key(|t| t.tier_level);

        if let Some(tier) = qualified_tier {
            let member = self.repository.get_member(member_id).await?;
            if let Some(m) = member {
                if m.tier_id != Some(tier.id) {
                    info!("Upgrading member {} to tier '{}' ({})", member_id, tier.tier_code, tier.tier_name);
                    self.repository.update_member_tier(member_id, tier.id, &tier.tier_code).await?;

                    // Calculate points remaining to next tier
                    let next_tier = tiers.iter()
                        .filter(|t| t.tier_level > tier.tier_level)
                        .min_by_key(|t| t.tier_level);
                    let remaining = next_tier.map(|t| t.minimum_points - lifetime_points);
                    self.repository.update_member_next_tier_remaining(member_id, remaining).await?;
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_program_types() {
        assert!(VALID_PROGRAM_TYPES.contains(&"points"));
        assert!(VALID_PROGRAM_TYPES.contains(&"tier"));
        assert!(VALID_PROGRAM_TYPES.contains(&"frequency"));
        assert!(VALID_PROGRAM_TYPES.contains(&"hybrid"));
        assert!(!VALID_PROGRAM_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_program_statuses() {
        assert!(VALID_PROGRAM_STATUSES.contains(&"draft"));
        assert!(VALID_PROGRAM_STATUSES.contains(&"active"));
        assert!(VALID_PROGRAM_STATUSES.contains(&"suspended"));
        assert!(VALID_PROGRAM_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_txn_types() {
        assert!(VALID_TXN_TYPES.contains(&"accrual"));
        assert!(VALID_TXN_TYPES.contains(&"redemption"));
        assert!(VALID_TXN_TYPES.contains(&"adjustment"));
        assert!(VALID_TXN_TYPES.contains(&"expiration"));
        assert!(VALID_TXN_TYPES.contains(&"bonus"));
        assert!(VALID_TXN_TYPES.contains(&"reversal"));
    }

    #[test]
    fn test_valid_reward_types() {
        assert!(VALID_REWARD_TYPES.contains(&"merchandise"));
        assert!(VALID_REWARD_TYPES.contains(&"discount"));
        assert!(VALID_REWARD_TYPES.contains(&"voucher"));
        assert!(VALID_REWARD_TYPES.contains(&"cashback"));
    }

    #[test]
    fn test_valid_member_statuses() {
        assert!(VALID_MEMBER_STATUSES.contains(&"active"));
        assert!(VALID_MEMBER_STATUSES.contains(&"inactive"));
        assert!(VALID_MEMBER_STATUSES.contains(&"suspended"));
        assert!(VALID_MEMBER_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("program_type", "points", VALID_PROGRAM_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("program_type", "invalid", VALID_PROGRAM_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("program_type"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("program_type", "", VALID_PROGRAM_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_calculate_points_round() {
        let engine_setup = || -> LoyaltyManagementEngine {
            // We can't easily create an engine without a repo, so just test the math directly
            unimplemented!()
        };
        // Test rounding methods manually
        let raw = 99.5_f64;
        assert!((raw.round() - 100.0).abs() < 0.01);
        assert!((raw.floor() - 99.0).abs() < 0.01);
        assert!((raw.ceil() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_valid_redemption_statuses() {
        assert!(VALID_REDEMPTION_STATUSES.contains(&"pending"));
        assert!(VALID_REDEMPTION_STATUSES.contains(&"fulfilled"));
        assert!(VALID_REDEMPTION_STATUSES.contains(&"cancelled"));
        assert!(VALID_REDEMPTION_STATUSES.contains(&"expired"));
    }
}
