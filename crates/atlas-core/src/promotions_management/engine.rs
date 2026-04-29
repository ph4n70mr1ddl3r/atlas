//! Promotions Management Engine
//!
//! Oracle Fusion Trade Management > Trade Promotion.
//!
//! Features:
//! - Promotion CRUD with lifecycle (draft → active → on_hold → completed → cancelled)
//! - Promotional offers (discounts, buy-get, bundles, rebates, free items)
//! - Fund allocation and tracking (marketing development, cooperative, trade spend)
//! - Claims processing (accrual, settlement, deduction, lump sum)
//! - ROI analysis and budget utilization tracking
//! - Promotions dashboard with analytics
//!
//! Process:
//! 1. Create promotion with budget, dates, customer/product scope
//! 2. Add promotional offers (discount rules)
//! 3. Allocate funds by fund type
//! 4. Activate promotion
//! 5. Submit claims against promotion
//! 6. Approve/reject claims
//! 7. Settle (pay) approved claims
//! 8. Monitor ROI and budget utilization

use atlas_shared::AtlasError;
use super::PromotionsManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid promotion types
const VALID_PROMOTION_TYPES: &[&str] = &[
    "trade", "consumer", "channel", "co_op",
];

/// Valid promotion statuses and transitions
const VALID_STATUSES: &[&str] = &[
    "draft", "active", "on_hold", "completed", "cancelled",
];

/// Valid offer types
const VALID_OFFER_TYPES: &[&str] = &[
    "discount", "buy_get", "bundle", "free_item", "rebate",
];

/// Valid discount types
const VALID_DISCOUNT_TYPES: &[&str] = &[
    "percentage", "fixed_amount", "fixed_price",
];

/// Valid fund types
const VALID_FUND_TYPES: &[&str] = &[
    "marketing_development", "cooperative", "trade_spend", "display",
];

/// Valid claim types
const VALID_CLAIM_TYPES: &[&str] = &[
    "accrual", "settlement", "deduction", "lump_sum",
];

/// Valid claim statuses
const VALID_CLAIM_STATUSES: &[&str] = &[
    "submitted", "under_review", "approved", "rejected", "paid",
];

/// Promotions Management Engine
pub struct PromotionsManagementEngine {
    repository: Arc<dyn PromotionsManagementRepository>,
}

impl PromotionsManagementEngine {
    pub fn new(repository: Arc<dyn PromotionsManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Promotion CRUD
    // ========================================================================

    /// Create a new promotion
    pub async fn create_promotion(
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
    ) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let code = code.trim().to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Promotion code must be 1-50 characters".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Promotion name is required".to_string()));
        }
        if !VALID_PROMOTION_TYPES.contains(&promotion_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid promotion type '{}'. Must be one of: {}",
                promotion_type,
                VALID_PROMOTION_TYPES.join(", ")
            )));
        }
        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }
        let budget: f64 = budget_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Budget amount must be a valid number".to_string())
        })?;
        if budget < 0.0 {
            return Err(AtlasError::ValidationFailed("Budget amount cannot be negative".to_string()));
        }

        // Check uniqueness
        if self.repository.get_promotion_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Promotion '{}' already exists", code)));
        }

        info!("Creating promotion '{}' ({}) for org {}", name, code, org_id);
        self.repository.create_promotion(
            org_id, &code, name, description, promotion_type,
            start_date, end_date, customer_id, customer_name,
            territory_id, product_id, product_name, budget_amount,
            currency_code, owner_id, owner_name, created_by,
        ).await
    }

    /// Get a promotion by ID
    pub async fn get_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::PromoMgmtPromotion>> {
        self.repository.get_promotion(id).await
    }

    /// List promotions with optional filters
    pub async fn list_promotions(
        &self,
        org_id: Uuid,
        promotion_type: Option<&str>,
        status: Option<&str>,
        include_inactive: bool,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::PromoMgmtPromotion>> {
        if let Some(pt) = promotion_type {
            if !VALID_PROMOTION_TYPES.contains(&pt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid promotion type '{}'", pt
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'", s
                )));
            }
        }
        self.repository.list_promotions(org_id, promotion_type, status, include_inactive).await
    }

    /// Update a promotion
    pub async fn update_promotion(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        budget_amount: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let existing = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;

        // Validate dates
        let from = start_date.unwrap_or(existing.start_date);
        let to = end_date.unwrap_or(existing.end_date);
        if from >= to {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }

        if let Some(ba) = budget_amount {
            let budget: f64 = ba.parse().map_err(|_| {
                AtlasError::ValidationFailed("Budget amount must be a valid number".to_string())
            })?;
            if budget < 0.0 {
                return Err(AtlasError::ValidationFailed("Budget amount cannot be negative".to_string()));
            }
        }

        info!("Updating promotion {} ({})", id, existing.code);
        self.repository.update_promotion(
            id, name, description, start_date, end_date,
            budget_amount, owner_id, owner_name,
        ).await
    }

    /// Activate a promotion
    pub async fn activate_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let promotion = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        if promotion.status != "draft" && promotion.status != "on_hold" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot activate promotion in '{}' status. Must be 'draft' or 'on_hold'.", promotion.status)
            ));
        }
        info!("Activating promotion {}", promotion.code);
        self.repository.update_promotion_status(id, "active").await
    }

    /// Put a promotion on hold
    pub async fn hold_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let promotion = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        if promotion.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot hold promotion in '{}' status. Must be 'active'.", promotion.status)
            ));
        }
        info!("Putting promotion {} on hold", promotion.code);
        self.repository.update_promotion_status(id, "on_hold").await
    }

    /// Complete a promotion
    pub async fn complete_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let promotion = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        if promotion.status != "active" && promotion.status != "on_hold" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot complete promotion in '{}' status.", promotion.status)
            ));
        }
        info!("Completing promotion {}", promotion.code);
        self.repository.update_promotion_status(id, "completed").await
    }

    /// Cancel a promotion
    pub async fn cancel_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtPromotion> {
        let promotion = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        if promotion.status == "completed" || promotion.status == "cancelled" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot cancel promotion in '{}' status.", promotion.status)
            ));
        }
        info!("Cancelling promotion {}", promotion.code);
        self.repository.update_promotion_status(id, "cancelled").await
    }

    /// Delete a promotion (only in draft)
    pub async fn delete_promotion(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let promotion = self.repository.get_promotion(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", id)))?;
        if promotion.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only delete promotions in 'draft' status".to_string(),
            ));
        }
        info!("Deleting promotion {} ({})", promotion.code, id);
        self.repository.delete_promotion(id).await
    }

    // ========================================================================
    // Promotional Offers
    // ========================================================================

    /// Add an offer to a promotion
    pub async fn create_offer(
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
    ) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtOffer> {
        self.repository.get_promotion(promotion_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", promotion_id)))?;

        if !VALID_OFFER_TYPES.contains(&offer_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid offer type '{}'. Must be one of: {}",
                offer_type,
                VALID_OFFER_TYPES.join(", ")
            )));
        }
        if !VALID_DISCOUNT_TYPES.contains(&discount_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid discount type '{}'. Must be one of: {}",
                discount_type,
                VALID_DISCOUNT_TYPES.join(", ")
            )));
        }
        let val: f64 = discount_value.parse().map_err(|_| {
            AtlasError::ValidationFailed("Discount value must be a valid number".to_string())
        })?;
        if val < 0.0 {
            return Err(AtlasError::ValidationFailed("Discount value cannot be negative".to_string()));
        }
        if discount_type == "percentage" && val > 100.0 {
            return Err(AtlasError::ValidationFailed("Percentage discount cannot exceed 100".to_string()));
        }

        if let Some(mp) = minimum_purchase {
            let mpv: f64 = mp.parse().map_err(|_| AtlasError::ValidationFailed("Minimum purchase must be a number".to_string()))?;
            if mpv < 0.0 {
                return Err(AtlasError::ValidationFailed("Minimum purchase cannot be negative".to_string()));
            }
        }
        if let Some(md) = maximum_discount {
            let mdv: f64 = md.parse().map_err(|_| AtlasError::ValidationFailed("Maximum discount must be a number".to_string()))?;
            if mdv < 0.0 {
                return Err(AtlasError::ValidationFailed("Maximum discount cannot be negative".to_string()));
            }
        }

        info!("Adding {} offer to promotion {}", offer_type, promotion_id);
        self.repository.create_offer(
            org_id, promotion_id, offer_type, description,
            discount_type, discount_value, buy_quantity, get_quantity,
            minimum_purchase, maximum_discount, created_by,
        ).await
    }

    /// List offers for a promotion
    pub async fn list_offers(&self, promotion_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::PromoMgmtOffer>> {
        self.repository.list_offers(promotion_id).await
    }

    /// Get a single offer
    pub async fn get_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::PromoMgmtOffer>> {
        self.repository.get_offer(id).await
    }

    /// Delete an offer
    pub async fn delete_offer(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_offer(id).await
    }

    // ========================================================================
    // Fund Allocation
    // ========================================================================

    /// Allocate funds to a promotion
    pub async fn create_fund(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        fund_type: &str,
        allocated_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtFund> {
        self.repository.get_promotion(promotion_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", promotion_id)))?;

        if !VALID_FUND_TYPES.contains(&fund_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid fund type '{}'. Must be one of: {}",
                fund_type,
                VALID_FUND_TYPES.join(", ")
            )));
        }
        let alloc: f64 = allocated_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Allocated amount must be a valid number".to_string())
        })?;
        if alloc < 0.0 {
            return Err(AtlasError::ValidationFailed("Allocated amount cannot be negative".to_string()));
        }

        info!("Allocating {} {} funds to promotion {}", allocated_amount, fund_type, promotion_id);
        self.repository.create_fund(
            org_id, promotion_id, fund_type, allocated_amount,
            currency_code, created_by,
        ).await
    }

    /// List funds for a promotion
    pub async fn list_funds(&self, promotion_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::PromoMgmtFund>> {
        self.repository.list_funds(promotion_id).await
    }

    /// Update fund committed amount
    pub async fn update_fund_committed(&self, id: Uuid, committed_amount: &str) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtFund> {
        let comm: f64 = committed_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Committed amount must be a valid number".to_string())
        })?;
        if comm < 0.0 {
            return Err(AtlasError::ValidationFailed("Committed amount cannot be negative".to_string()));
        }
        self.repository.update_fund_committed(id, committed_amount).await
    }

    /// Update fund spent amount
    pub async fn update_fund_spent(&self, id: Uuid, spent_amount: &str) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtFund> {
        let spent: f64 = spent_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Spent amount must be a valid number".to_string())
        })?;
        if spent < 0.0 {
            return Err(AtlasError::ValidationFailed("Spent amount cannot be negative".to_string()));
        }
        self.repository.update_fund_spent(id, spent_amount).await
    }

    /// Delete a fund allocation
    pub async fn delete_fund(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_fund(id).await
    }

    // ========================================================================
    // Claims Processing
    // ========================================================================

    /// Submit a claim against a promotion
    pub async fn create_claim(
        &self,
        org_id: Uuid,
        promotion_id: Uuid,
        claim_type: &str,
        amount: &str,
        currency_code: &str,
        claim_date: chrono::NaiveDate,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtClaim> {
        let promotion = self.repository.get_promotion(promotion_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Promotion {} not found", promotion_id)))?;

        if promotion.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot submit claims against promotion in '{}' status", promotion.status)
            ));
        }

        if !VALID_CLAIM_TYPES.contains(&claim_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid claim type '{}'. Must be one of: {}",
                claim_type,
                VALID_CLAIM_TYPES.join(", ")
            )));
        }

        let amt: f64 = amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Claim amount must be a valid number".to_string())
        })?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Claim amount must be greater than zero".to_string()));
        }

        // Generate claim number
        let claim_number = format!("CLM-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating {} claim {} for promotion {}", claim_type, claim_number, promotion.code);
        self.repository.create_claim(
            org_id, promotion_id, &claim_number, claim_type,
            amount, currency_code, claim_date, customer_id,
            customer_name, description, created_by,
        ).await
    }

    /// Get a claim by ID
    pub async fn get_claim(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::PromoMgmtClaim>> {
        self.repository.get_claim(id).await
    }

    /// List claims for a promotion
    pub async fn list_claims(
        &self,
        promotion_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::PromoMgmtClaim>> {
        if let Some(s) = status {
            if !VALID_CLAIM_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!("Invalid claim status '{}'", s)));
            }
        }
        self.repository.list_claims(promotion_id, status).await
    }

    /// Review a claim (move to under_review)
    pub async fn review_claim(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtClaim> {
        let claim = self.repository.get_claim(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        if claim.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot review claim in '{}' status. Must be 'submitted'.", claim.status)
            ));
        }
        self.repository.update_claim_status(id, "under_review", None, None).await
    }

    /// Approve a claim
    pub async fn approve_claim(&self, id: Uuid, approved_amount: Option<&str>) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtClaim> {
        let claim = self.repository.get_claim(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        if claim.status != "submitted" && claim.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot approve claim in '{}' status.", claim.status)
            ));
        }
        if let Some(aa) = approved_amount {
            let a: f64 = aa.parse().map_err(|_| AtlasError::ValidationFailed("Approved amount must be a number".to_string()))?;
            if a <= 0.0 {
                return Err(AtlasError::ValidationFailed("Approved amount must be greater than zero".to_string()));
            }
        }
        info!("Approving claim {}", claim.claim_number);
        self.repository.update_claim_status(id, "approved", approved_amount, None).await
    }

    /// Reject a claim
    pub async fn reject_claim(&self, id: Uuid, reason: &str) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtClaim> {
        let claim = self.repository.get_claim(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        if claim.status != "submitted" && claim.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot reject claim in '{}' status.", claim.status)
            ));
        }
        if reason.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Rejection reason is required".to_string()));
        }
        info!("Rejecting claim {}: {}", claim.claim_number, reason);
        self.repository.update_claim_status(id, "rejected", None, Some(reason)).await
    }

    /// Settle (pay) a claim
    pub async fn settle_claim(&self, id: Uuid, paid_amount: &str) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtClaim> {
        let claim = self.repository.get_claim(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        if claim.status != "approved" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot settle claim in '{}' status. Must be 'approved'.", claim.status)
            ));
        }
        let paid: f64 = paid_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Paid amount must be a valid number".to_string())
        })?;
        if paid <= 0.0 {
            return Err(AtlasError::ValidationFailed("Paid amount must be greater than zero".to_string()));
        }
        let today = chrono::Utc::now().date_naive();
        info!("Settling claim {} for {}", claim.claim_number, paid_amount);
        self.repository.settle_claim(id, paid_amount, today).await
    }

    /// Delete a claim (only in submitted status)
    pub async fn delete_claim(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let claim = self.repository.get_claim(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Claim {} not found", id)))?;
        if claim.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                "Can only delete claims in 'submitted' status".to_string(),
            ));
        }
        self.repository.delete_claim(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the promotions management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::PromoMgmtDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_promotion_types() {
        assert!(VALID_PROMOTION_TYPES.contains(&"trade"));
        assert!(VALID_PROMOTION_TYPES.contains(&"consumer"));
        assert!(VALID_PROMOTION_TYPES.contains(&"channel"));
        assert!(VALID_PROMOTION_TYPES.contains(&"co_op"));
        assert!(!VALID_PROMOTION_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"active"));
        assert!(VALID_STATUSES.contains(&"on_hold"));
        assert!(VALID_STATUSES.contains(&"completed"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_offer_types() {
        assert!(VALID_OFFER_TYPES.contains(&"discount"));
        assert!(VALID_OFFER_TYPES.contains(&"buy_get"));
        assert!(VALID_OFFER_TYPES.contains(&"bundle"));
        assert!(VALID_OFFER_TYPES.contains(&"free_item"));
        assert!(VALID_OFFER_TYPES.contains(&"rebate"));
    }

    #[test]
    fn test_valid_discount_types() {
        assert!(VALID_DISCOUNT_TYPES.contains(&"percentage"));
        assert!(VALID_DISCOUNT_TYPES.contains(&"fixed_amount"));
        assert!(VALID_DISCOUNT_TYPES.contains(&"fixed_price"));
    }

    #[test]
    fn test_valid_fund_types() {
        assert!(VALID_FUND_TYPES.contains(&"marketing_development"));
        assert!(VALID_FUND_TYPES.contains(&"cooperative"));
        assert!(VALID_FUND_TYPES.contains(&"trade_spend"));
        assert!(VALID_FUND_TYPES.contains(&"display"));
    }

    #[test]
    fn test_valid_claim_types() {
        assert!(VALID_CLAIM_TYPES.contains(&"accrual"));
        assert!(VALID_CLAIM_TYPES.contains(&"settlement"));
        assert!(VALID_CLAIM_TYPES.contains(&"deduction"));
        assert!(VALID_CLAIM_TYPES.contains(&"lump_sum"));
    }

    #[test]
    fn test_valid_claim_statuses() {
        assert!(VALID_CLAIM_STATUSES.contains(&"submitted"));
        assert!(VALID_CLAIM_STATUSES.contains(&"under_review"));
        assert!(VALID_CLAIM_STATUSES.contains(&"approved"));
        assert!(VALID_CLAIM_STATUSES.contains(&"rejected"));
        assert!(VALID_CLAIM_STATUSES.contains(&"paid"));
    }

    #[test]
    fn test_budget_validation() {
        let budget = "-100".parse::<f64>();
        assert!(budget.unwrap() < 0.0);
        let budget = "50000".parse::<f64>();
        assert!(budget.unwrap() > 0.0);
    }

    #[test]
    fn test_percentage_discount_validation() {
        let discount_type = "percentage";
        let val = 150.0_f64;
        assert!(discount_type == "percentage" && val > 100.0);
        let val = 25.0_f64;
        assert!(discount_type == "percentage" && val <= 100.0);
    }

    #[test]
    fn test_date_validation() {
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert!(start < end);
        let same = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(start == same);
    }

    #[test]
    fn test_utilization_calculation() {
        let budget = 100000.0_f64;
        let spent = 45000.0_f64;
        let pct = (spent / budget) * 100.0;
        assert!((pct - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_code_uppercase() {
        let code = "spring-sale".to_string();
        let upper = code.trim().to_uppercase();
        assert_eq!(upper, "SPRING-SALE");
    }
}
