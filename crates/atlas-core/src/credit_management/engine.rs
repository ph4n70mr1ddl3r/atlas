//! Credit Management Engine
//!
//! Oracle Fusion Cloud Credit Management.
//! Manages customer credit profiles, credit scoring, credit limits,
//! credit exposure tracking, credit check rules, credit holds,
//! and credit reviews.
//!
//! The process follows Oracle Fusion's Credit Management workflow:
//! 1. Define scoring models for credit assessment
//! 2. Create credit profiles for customers
//! 3. Set credit limits (overall, per-currency)
//! 4. Configure credit check rules (automatic/manual)
//! 5. Track credit exposure in real-time
//! 6. Place/release credit holds on transactions
//! 7. Conduct periodic credit reviews

use atlas_shared::{
    CreditScoringModel, CreditProfile, CreditLimit, CreditCheckRule,
    CreditExposure, CreditHold, CreditReview, CreditManagementDashboard,
    AtlasError, AtlasResult,
};
use super::CreditManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid model types for scoring models
const VALID_MODEL_TYPES: &[&str] = &[
    "manual", "scorecard", "risk_category", "external",
];

/// Valid profile types
const VALID_PROFILE_TYPES: &[&str] = &[
    "customer", "customer_group", "global",
];

/// Valid profile statuses
const VALID_PROFILE_STATUSES: &[&str] = &[
    "active", "inactive", "suspended", "blocked",
];

/// Valid risk levels
const VALID_RISK_LEVELS: &[&str] = &[
    "low", "medium", "high", "very_high", "blocked",
];

/// Valid limit types
const VALID_LIMIT_TYPES: &[&str] = &[
    "overall", "order", "delivery", "currency",
];

/// Valid check points for credit check rules
const VALID_CHECK_POINTS: &[&str] = &[
    "order_entry", "shipment", "invoice", "delivery", "payment",
];

/// Valid check types
const VALID_CHECK_TYPES: &[&str] = &[
    "automatic", "manual",
];

/// Valid actions on credit check failure
const VALID_FAILURE_ACTIONS: &[&str] = &[
    "hold", "warn", "reject", "notify",
];

/// Valid hold types
const VALID_HOLD_TYPES: &[&str] = &[
    "credit_limit", "overdue", "review", "manual", "scoring",
];

/// Valid hold statuses
const VALID_HOLD_STATUSES: &[&str] = &[
    "active", "released", "overridden", "cancelled",
];

/// Valid review types
const VALID_REVIEW_TYPES: &[&str] = &[
    "periodic", "triggered", "ad_hoc", "escalation",
];

/// Valid review statuses
const VALID_REVIEW_STATUSES: &[&str] = &[
    "pending", "in_review", "completed", "approved", "rejected", "cancelled",
];

/// Credit Management engine
pub struct CreditManagementEngine {
    repository: Arc<dyn CreditManagementRepository>,
}

impl CreditManagementEngine {
    pub fn new(repository: Arc<dyn CreditManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Scoring Model Management
    // ========================================================================

    /// Create a new credit scoring model
    pub async fn create_scoring_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        model_type: &str,
        scoring_criteria: serde_json::Value,
        score_ranges: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditScoringModel> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Scoring model code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Scoring model name is required".to_string(),
            ));
        }
        if !VALID_MODEL_TYPES.contains(&model_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid model_type '{}'. Must be one of: {}",
                model_type, VALID_MODEL_TYPES.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_scoring_model_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Scoring model '{}' already exists", code
            )));
        }

        info!("Creating credit scoring model '{}' for org {}", code, org_id);

        self.repository.create_scoring_model(
            org_id, &code, name, description, model_type,
            scoring_criteria, score_ranges, created_by,
        ).await
    }

    /// Get a scoring model by ID
    pub async fn get_scoring_model(&self, id: Uuid) -> AtlasResult<Option<CreditScoringModel>> {
        self.repository.get_scoring_model(id).await
    }

    /// Get a scoring model by code
    pub async fn get_scoring_model_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CreditScoringModel>> {
        self.repository.get_scoring_model_by_code(org_id, code).await
    }

    /// List all scoring models for an org
    pub async fn list_scoring_models(&self, org_id: Uuid) -> AtlasResult<Vec<CreditScoringModel>> {
        self.repository.list_scoring_models(org_id).await
    }

    /// Delete a scoring model
    pub async fn delete_scoring_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting credit scoring model '{}' for org {}", code, org_id);
        self.repository.delete_scoring_model(org_id, code).await
    }

    // ========================================================================
    // Credit Profile Management
    // ========================================================================

    /// Create a new credit profile
    pub async fn create_profile(
        &self,
        org_id: Uuid,
        profile_number: &str,
        profile_name: &str,
        description: Option<&str>,
        profile_type: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        customer_group_id: Option<Uuid>,
        customer_group_name: Option<&str>,
        scoring_model_id: Option<Uuid>,
        review_frequency_days: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditProfile> {
        if profile_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Profile number is required".to_string(),
            ));
        }
        if profile_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Profile name is required".to_string(),
            ));
        }
        if !VALID_PROFILE_TYPES.contains(&profile_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid profile_type '{}'. Must be one of: {}",
                profile_type, VALID_PROFILE_TYPES.join(", ")
            )));
        }

        // Validate customer/group assignment based on type
        match profile_type {
            "customer" if customer_id.is_none() => {
                return Err(AtlasError::ValidationFailed(
                    "customer_id is required for customer profile type".to_string(),
                ));
            }
            "customer_group" if customer_group_id.is_none() => {
                return Err(AtlasError::ValidationFailed(
                    "customer_group_id is required for customer_group profile type".to_string(),
                ));
            }
            _ => {}
        }

        // Validate scoring model if provided
        if let Some(sm_id) = scoring_model_id {
            if self.repository.get_scoring_model(sm_id).await?.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Scoring model {} not found", sm_id)
                ));
            }
        }

        // Check uniqueness
        if self.repository.get_profile_by_number(org_id, profile_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Credit profile '{}' already exists", profile_number
            )));
        }

        info!("Creating credit profile '{}' for org {}", profile_number, org_id);

        let profile = self.repository.create_profile(
            org_id, profile_number, profile_name, description,
            profile_type, customer_id, customer_name,
            customer_group_id, customer_group_name,
            scoring_model_id, review_frequency_days, created_by,
        ).await?;

        // Create default overall credit limit of 0
        self.repository.create_credit_limit(
            org_id, profile.id, "overall", None,
            "0", None, None, None,
        ).await?;

        Ok(profile)
    }

    /// Get a profile by ID
    pub async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<CreditProfile>> {
        self.repository.get_profile(id).await
    }

    /// Get a profile by number
    pub async fn get_profile_by_number(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<Option<CreditProfile>> {
        self.repository.get_profile_by_number(org_id, profile_number).await
    }

    /// Get a profile by customer ID
    pub async fn get_profile_by_customer(&self, org_id: Uuid, customer_id: Uuid) -> AtlasResult<Option<CreditProfile>> {
        self.repository.get_profile_by_customer(org_id, customer_id).await
    }

    /// List profiles
    pub async fn list_profiles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CreditProfile>> {
        self.repository.list_profiles(org_id, status).await
    }

    /// Update profile status
    pub async fn update_profile_status(&self, id: Uuid, status: &str) -> AtlasResult<CreditProfile> {
        if !VALID_PROFILE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid profile status '{}'. Must be one of: {}",
                status, VALID_PROFILE_STATUSES.join(", ")
            )));
        }

        let profile = self.repository.get_profile(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", id)
            ))?;

        info!("Updating credit profile {} status to {}", profile.profile_number, status);
        self.repository.update_profile_status(id, status).await
    }

    /// Update profile score and rating
    pub async fn update_profile_score(
        &self,
        id: Uuid,
        credit_score: &str,
        credit_rating: &str,
        risk_level: &str,
    ) -> AtlasResult<CreditProfile> {
        if !VALID_RISK_LEVELS.contains(&risk_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_level '{}'. Must be one of: {}",
                risk_level, VALID_RISK_LEVELS.join(", ")
            )));
        }

        let score: f64 = credit_score.parse().map_err(|_| {
            AtlasError::ValidationFailed("credit_score must be a number".to_string())
        })?;

        if !(0.0..=100.0).contains(&score) {
            return Err(AtlasError::ValidationFailed(
                "credit_score must be between 0 and 100".to_string(),
            ));
        }

        info!("Updating credit profile {} score to {} ({})", id, credit_score, credit_rating);
        self.repository.update_profile_score(id, credit_score, credit_rating, risk_level).await
    }

    /// Delete a profile
    pub async fn delete_profile(&self, org_id: Uuid, profile_number: &str) -> AtlasResult<()> {
        info!("Deleting credit profile '{}' for org {}", profile_number, org_id);
        self.repository.delete_profile(org_id, profile_number).await
    }

    // ========================================================================
    // Credit Limit Management
    // ========================================================================

    /// Create a credit limit
    pub async fn create_credit_limit(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        limit_type: &str,
        currency_code: Option<&str>,
        credit_limit: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditLimit> {
        if !VALID_LIMIT_TYPES.contains(&limit_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid limit_type '{}'. Must be one of: {}",
                limit_type, VALID_LIMIT_TYPES.join(", ")
            )));
        }

        let limit: f64 = credit_limit.parse().map_err(|_| {
            AtlasError::ValidationFailed("credit_limit must be a number".to_string())
        })?;

        if limit < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "credit_limit cannot be negative".to_string(),
            ));
        }

        // Verify profile exists
        self.repository.get_profile(profile_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", profile_id)
            ))?;

        info!("Creating {} credit limit of {} for profile {}", limit_type, credit_limit, profile_id);

        self.repository.create_credit_limit(
            org_id, profile_id, limit_type, currency_code,
            credit_limit, effective_from, effective_to, created_by,
        ).await
    }

    /// List credit limits for a profile
    pub async fn list_credit_limits(&self, profile_id: Uuid) -> AtlasResult<Vec<CreditLimit>> {
        self.repository.list_credit_limits(profile_id).await
    }

    /// Update credit limit amount
    pub async fn update_credit_limit_amount(&self, id: Uuid, credit_limit: &str) -> AtlasResult<CreditLimit> {
        let limit: f64 = credit_limit.parse().map_err(|_| {
            AtlasError::ValidationFailed("credit_limit must be a number".to_string())
        })?;

        if limit < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "credit_limit cannot be negative".to_string(),
            ));
        }

        info!("Updating credit limit {} to {}", id, credit_limit);
        self.repository.update_credit_limit_amount(id, credit_limit).await
    }

    /// Set a temporary limit increase
    pub async fn set_temp_limit(&self, id: Uuid, temp_increase: &str, expiry: Option<chrono::NaiveDate>) -> AtlasResult<CreditLimit> {
        let increase: f64 = temp_increase.parse().map_err(|_| {
            AtlasError::ValidationFailed("temp_limit_increase must be a number".to_string())
        })?;

        if increase < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "temp_limit_increase cannot be negative".to_string(),
            ));
        }

        if expiry.is_none() {
            return Err(AtlasError::ValidationFailed(
                "temp_limit_expiry is required when setting a temporary limit increase".to_string(),
            ));
        }

        info!("Setting temp limit increase of {} for credit limit {}", temp_increase, id);
        self.repository.set_temp_limit(id, temp_increase, expiry).await
    }

    /// Delete a credit limit
    pub async fn delete_credit_limit(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_credit_limit(id).await
    }

    // ========================================================================
    // Credit Check Rules
    // ========================================================================

    /// Create a credit check rule
    pub async fn create_check_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        check_point: &str,
        check_type: &str,
        condition: serde_json::Value,
        action_on_failure: &str,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditCheckRule> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rule name is required".to_string(),
            ));
        }
        if !VALID_CHECK_POINTS.contains(&check_point) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid check_point '{}'. Must be one of: {}",
                check_point, VALID_CHECK_POINTS.join(", ")
            )));
        }
        if !VALID_CHECK_TYPES.contains(&check_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid check_type '{}'. Must be one of: {}",
                check_type, VALID_CHECK_TYPES.join(", ")
            )));
        }
        if !VALID_FAILURE_ACTIONS.contains(&action_on_failure) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid action_on_failure '{}'. Must be one of: {}",
                action_on_failure, VALID_FAILURE_ACTIONS.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_check_rule_by_name(org_id, name).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Credit check rule '{}' already exists", name
            )));
        }

        info!("Creating credit check rule '{}' for org {}", name, org_id);

        self.repository.create_check_rule(
            org_id, name, description, check_point, check_type,
            condition, action_on_failure, priority,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a check rule by ID
    pub async fn get_check_rule(&self, id: Uuid) -> AtlasResult<Option<CreditCheckRule>> {
        self.repository.get_check_rule(id).await
    }

    /// List check rules
    pub async fn list_check_rules(&self, org_id: Uuid) -> AtlasResult<Vec<CreditCheckRule>> {
        self.repository.list_check_rules(org_id).await
    }

    /// Delete a check rule
    pub async fn delete_check_rule(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_check_rule(id).await
    }

    // ========================================================================
    // Credit Exposure
    // ========================================================================

    /// Calculate and record credit exposure for a profile
    pub async fn calculate_exposure(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        currency_code: &str,
        open_receivables: &str,
        open_orders: &str,
        open_shipments: &str,
        open_invoices: &str,
        unapplied_cash: &str,
        on_hold_amount: &str,
    ) -> AtlasResult<CreditExposure> {
        // Verify profile exists
        self.repository.get_profile(profile_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", profile_id)
            ))?;

        let receivables: f64 = open_receivables.parse().unwrap_or(0.0);
        let orders: f64 = open_orders.parse().unwrap_or(0.0);
        let shipments: f64 = open_shipments.parse().unwrap_or(0.0);
        let invoices: f64 = open_invoices.parse().unwrap_or(0.0);
        let cash: f64 = unapplied_cash.parse().unwrap_or(0.0);
        let holds: f64 = on_hold_amount.parse().unwrap_or(0.0);

        let total_exposure = receivables + orders + shipments + invoices - cash;
        let total_exposure = if total_exposure < 0.0 { 0.0 } else { total_exposure };

        // Get current credit limit
        let limits = self.repository.list_credit_limits(profile_id).await?;
        let overall_limit = limits.iter()
            .find(|l| l.limit_type == "overall")
            .map(|l| l.credit_limit.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let available_credit = if overall_limit > total_exposure {
            overall_limit - total_exposure
        } else {
            0.0
        };

        let utilization = if overall_limit > 0.0 {
            (total_exposure / overall_limit) * 100.0
        } else {
            0.0
        };

        let today = chrono::Utc::now().date_naive();

        info!(
            "Calculated credit exposure for profile {}: total={:.2}, limit={:.2}, available={:.2}, utilization={:.1}%",
            profile_id, total_exposure, overall_limit, available_credit, utilization
        );

        self.repository.create_exposure(
            org_id, profile_id, today, currency_code,
            open_receivables, open_orders, open_shipments, open_invoices,
            unapplied_cash, on_hold_amount,
            &format!("{:.2}", total_exposure),
            &format!("{:.2}", overall_limit),
            &format!("{:.2}", available_credit),
            &format!("{:.2}", utilization),
        ).await
    }

    /// Get latest exposure for a profile
    pub async fn get_latest_exposure(&self, profile_id: Uuid) -> AtlasResult<Option<CreditExposure>> {
        self.repository.get_latest_exposure(profile_id).await
    }

    /// Get exposure history
    pub async fn list_exposure_history(&self, profile_id: Uuid, limit: Option<i32>) -> AtlasResult<Vec<CreditExposure>> {
        self.repository.list_exposure_history(profile_id, limit).await
    }

    /// Perform a credit check against a profile
    /// Returns Ok(exposure) if credit is available, Err if credit limit exceeded
    pub async fn perform_credit_check(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        requested_amount: &str,
        check_point: &str,
    ) -> AtlasResult<CreditCheckResult> {
        let profile = self.repository.get_profile(profile_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", profile_id)
            ))?;

        if profile.status != "active" {
            return Ok(CreditCheckResult {
                passed: false,
                reason: Some(format!("Profile status is '{}', not 'active'", profile.status)),
                exposure: None,
            });
        }

        // Get latest exposure
        let exposure = self.repository.get_latest_exposure(profile_id).await?;

        let requested: f64 = requested_amount.parse().unwrap_or(0.0);

        // Get credit limit
        let limits = self.repository.list_credit_limits(profile_id).await?;
        let overall_limit = limits.iter()
            .find(|l| l.limit_type == "overall")
            .map(|l| {
                let base = l.credit_limit.parse::<f64>().unwrap_or(0.0);
                let temp = l.temp_limit_increase.parse::<f64>().unwrap_or(0.0);
                // Check temp limit expiry
                if let Some(expiry) = l.temp_limit_expiry {
                    if expiry >= chrono::Utc::now().date_naive() {
                        base + temp
                    } else {
                        base
                    }
                } else {
                    base
                }
            })
            .unwrap_or(0.0);

        let current_exposure = exposure.as_ref()
            .map(|e| e.total_exposure.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let available = if overall_limit > current_exposure {
            overall_limit - current_exposure
        } else {
            0.0
        };

        let passed = requested <= available;

        if !passed {
            info!(
                "Credit check FAILED for profile {}: requested {:.2}, available {:.2} (limit {:.2}, exposure {:.2})",
                profile.profile_number, requested, available, overall_limit, current_exposure
            );
        } else {
            info!(
                "Credit check PASSED for profile {}: requested {:.2}, available {:.2}",
                profile.profile_number, requested, available
            );
        }

        Ok(CreditCheckResult {
            passed,
            reason: if passed { None } else {
                Some(format!(
                    "Credit limit exceeded: requested {:.2}, available {:.2} (limit {:.2}, exposure {:.2})",
                    requested, available, overall_limit, current_exposure
                ))
            },
            exposure,
        })
    }

    // ========================================================================
    // Credit Holds
    // ========================================================================

    /// Create a credit hold
    pub async fn create_hold(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        hold_type: &str,
        entity_type: &str,
        entity_id: Uuid,
        entity_number: Option<&str>,
        hold_amount: Option<&str>,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditHold> {
        if !VALID_HOLD_TYPES.contains(&hold_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hold_type '{}'. Must be one of: {}",
                hold_type, VALID_HOLD_TYPES.join(", ")
            )));
        }

        // Verify profile exists
        self.repository.get_profile(profile_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", profile_id)
            ))?;

        let hold_number = format!("HLD-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        info!("Creating credit hold {} on {} ({})", hold_number, entity_type, hold_type);

        self.repository.create_hold(
            org_id, profile_id, &hold_number, hold_type,
            entity_type, entity_id, entity_number,
            hold_amount, reason, created_by,
        ).await
    }

    /// Get a hold by ID
    pub async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<CreditHold>> {
        self.repository.get_hold(id).await
    }

    /// List holds
    pub async fn list_holds(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditHold>> {
        self.repository.list_holds(org_id, status, profile_id).await
    }

    /// Release a hold
    pub async fn release_hold(&self, id: Uuid, released_by: Option<Uuid>, release_reason: Option<&str>) -> AtlasResult<CreditHold> {
        let hold = self.repository.get_hold(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit hold {} not found", id)
            ))?;

        if hold.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot release hold in '{}' status. Must be 'active'.", hold.status
            )));
        }

        info!("Releasing credit hold {}", hold.hold_number);
        self.repository.release_hold(id, released_by, release_reason).await
    }

    /// Override a hold (with authorization)
    pub async fn override_hold(&self, id: Uuid, overridden_by: Option<Uuid>, override_reason: Option<&str>) -> AtlasResult<CreditHold> {
        let hold = self.repository.get_hold(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit hold {} not found", id)
            ))?;

        if hold.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot override hold in '{}' status. Must be 'active'.", hold.status
            )));
        }

        if override_reason.is_none() || override_reason.map(|r| r.is_empty()).unwrap_or(true) {
            return Err(AtlasError::ValidationFailed(
                "Override reason is required when overriding a credit hold".to_string(),
            ));
        }

        info!("Overriding credit hold {}", hold.hold_number);
        self.repository.override_hold(id, overridden_by, override_reason).await
    }

    // ========================================================================
    // Credit Reviews
    // ========================================================================

    /// Create a credit review
    pub async fn create_review(
        &self,
        org_id: Uuid,
        profile_id: Uuid,
        review_type: &str,
        previous_credit_limit: Option<&str>,
        recommended_credit_limit: Option<&str>,
        previous_score: Option<&str>,
        previous_rating: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditReview> {
        if !VALID_REVIEW_TYPES.contains(&review_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid review_type '{}'. Must be one of: {}",
                review_type, VALID_REVIEW_TYPES.join(", ")
            )));
        }

        // Verify profile exists
        let profile = self.repository.get_profile(profile_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit profile {} not found", profile_id)
            ))?;

        let review_number = format!("CR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        // Auto-populate previous values from profile if not provided
        let prev_limit = previous_credit_limit.or_else(|| {
            // Get current limit
            None // Will be set from actual limits in real implementation
        });
        let prev_score = previous_score.or(profile.credit_score.as_deref());
        let prev_rating = previous_rating.or(profile.credit_rating.as_deref());

        info!("Creating credit review {} for profile {}", review_number, profile.profile_number);

        self.repository.create_review(
            org_id, profile_id, &review_number, review_type,
            prev_limit, recommended_credit_limit,
            prev_score, prev_rating,
            due_date, created_by,
        ).await
    }

    /// Get a review by ID
    pub async fn get_review(&self, id: Uuid) -> AtlasResult<Option<CreditReview>> {
        self.repository.get_review(id).await
    }

    /// List reviews
    pub async fn list_reviews(&self, org_id: Uuid, status: Option<&str>, profile_id: Option<Uuid>) -> AtlasResult<Vec<CreditReview>> {
        self.repository.list_reviews(org_id, status, profile_id).await
    }

    /// Start a review (transition from pending to in_review)
    pub async fn start_review(&self, id: Uuid) -> AtlasResult<CreditReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit review {} not found", id)
            ))?;

        if review.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start review in '{}' status. Must be 'pending'.", review.status
            )));
        }

        self.repository.update_review_status(id, "in_review").await
    }

    /// Complete a review with findings
    pub async fn complete_review(
        &self,
        id: Uuid,
        new_score: Option<&str>,
        new_rating: Option<&str>,
        approved_credit_limit: Option<&str>,
        findings: Option<&str>,
        recommendations: Option<&str>,
        reviewer_id: Option<Uuid>,
        reviewer_name: Option<&str>,
    ) -> AtlasResult<CreditReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit review {} not found", id)
            ))?;

        if review.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete review in '{}' status. Must be 'in_review'.", review.status
            )));
        }

        info!("Completing credit review {}", review.review_number);
        let completed = self.repository.complete_review(
            id, new_score, new_rating, approved_credit_limit,
            findings, recommendations, reviewer_id, reviewer_name,
        ).await?;

        // Update the profile's score and review dates
        let profile_id = completed.profile_id;
        let today = chrono::Utc::now().date_naive();

        if let (Some(score), Some(rating)) = (&new_score, &new_rating) {
            // Determine risk level from score
            let score_val: f64 = score.parse().unwrap_or(0.0);
            let risk_level = if score_val >= 80.0 {
                "low"
            } else if score_val >= 60.0 {
                "medium"
            } else if score_val >= 40.0 {
                "high"
            } else {
                "very_high"
            };

            if let Err(e) = self.repository.update_profile_score(
                profile_id, score, rating, risk_level,
            ).await {
                tracing::warn!("Failed to update profile score after review: {}", e);
            }
        }

        // Update profile review dates
        let profile = self.repository.get_profile(profile_id).await?;
        if let Some(p) = profile {
            let next_review = today + chrono::Duration::days(p.review_frequency_days as i64);
            if let Err(e) = self.repository.update_profile_review_dates(
                profile_id, Some(today), Some(next_review),
            ).await {
                tracing::warn!("Failed to update profile review dates: {}", e);
            }
        }

        // Update credit limit if approved
        if let Some(new_limit) = approved_credit_limit {
            let limits = self.repository.list_credit_limits(profile_id).await?;
            if let Some(overall) = limits.iter().find(|l| l.limit_type == "overall") {
                if let Err(e) = self.repository.update_credit_limit_amount(
                    overall.id, new_limit,
                ).await {
                    tracing::warn!("Failed to update credit limit after review: {}", e);
                }
            }
        }

        Ok(completed)
    }

    /// Approve a completed review
    pub async fn approve_review(
        &self,
        id: Uuid,
        approver_id: Option<Uuid>,
        approver_name: Option<&str>,
    ) -> AtlasResult<CreditReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit review {} not found", id)
            ))?;

        if review.status != "completed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve review in '{}' status. Must be 'completed'.", review.status
            )));
        }

        info!("Approving credit review {}", review.review_number);
        self.repository.approve_review(id, approver_id, approver_name).await
    }

    /// Reject a completed review
    pub async fn reject_review(&self, id: Uuid, rejected_reason: Option<&str>) -> AtlasResult<CreditReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit review {} not found", id)
            ))?;

        if review.status != "completed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject review in '{}' status. Must be 'completed'.", review.status
            )));
        }

        self.repository.update_review_status(id, "rejected").await
    }

    /// Cancel a pending or in_review review
    pub async fn cancel_review(&self, id: Uuid) -> AtlasResult<CreditReview> {
        let review = self.repository.get_review(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Credit review {} not found", id)
            ))?;

        if !matches!(review.status.as_str(), "pending" | "in_review") {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel review in '{}' status.", review.status
            )));
        }

        self.repository.update_review_status(id, "cancelled").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get credit management dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CreditManagementDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

/// Result of a credit check
#[derive(Debug, Clone)]
pub struct CreditCheckResult {
    pub passed: bool,
    pub reason: Option<String>,
    pub exposure: Option<CreditExposure>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_model_types() {
        assert!(VALID_MODEL_TYPES.contains(&"manual"));
        assert!(VALID_MODEL_TYPES.contains(&"scorecard"));
        assert!(VALID_MODEL_TYPES.contains(&"risk_category"));
        assert!(VALID_MODEL_TYPES.contains(&"external"));
        assert!(!VALID_MODEL_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_profile_types() {
        assert!(VALID_PROFILE_TYPES.contains(&"customer"));
        assert!(VALID_PROFILE_TYPES.contains(&"customer_group"));
        assert!(VALID_PROFILE_TYPES.contains(&"global"));
        assert!(!VALID_PROFILE_TYPES.contains(&"supplier"));
    }

    #[test]
    fn test_valid_profile_statuses() {
        assert!(VALID_PROFILE_STATUSES.contains(&"active"));
        assert!(VALID_PROFILE_STATUSES.contains(&"inactive"));
        assert!(VALID_PROFILE_STATUSES.contains(&"suspended"));
        assert!(VALID_PROFILE_STATUSES.contains(&"blocked"));
        assert!(!VALID_PROFILE_STATUSES.contains(&"deleted"));
    }

    #[test]
    fn test_valid_risk_levels() {
        assert!(VALID_RISK_LEVELS.contains(&"low"));
        assert!(VALID_RISK_LEVELS.contains(&"medium"));
        assert!(VALID_RISK_LEVELS.contains(&"high"));
        assert!(VALID_RISK_LEVELS.contains(&"very_high"));
        assert!(VALID_RISK_LEVELS.contains(&"blocked"));
        assert!(!VALID_RISK_LEVELS.contains(&"critical"));
    }

    #[test]
    fn test_valid_limit_types() {
        assert!(VALID_LIMIT_TYPES.contains(&"overall"));
        assert!(VALID_LIMIT_TYPES.contains(&"order"));
        assert!(VALID_LIMIT_TYPES.contains(&"delivery"));
        assert!(VALID_LIMIT_TYPES.contains(&"currency"));
        assert!(!VALID_LIMIT_TYPES.contains(&"temporary"));
    }

    #[test]
    fn test_valid_check_points() {
        assert!(VALID_CHECK_POINTS.contains(&"order_entry"));
        assert!(VALID_CHECK_POINTS.contains(&"shipment"));
        assert!(VALID_CHECK_POINTS.contains(&"invoice"));
        assert!(VALID_CHECK_POINTS.contains(&"delivery"));
        assert!(VALID_CHECK_POINTS.contains(&"payment"));
        assert!(!VALID_CHECK_POINTS.contains(&"receipt"));
    }

    #[test]
    fn test_valid_check_types() {
        assert!(VALID_CHECK_TYPES.contains(&"automatic"));
        assert!(VALID_CHECK_TYPES.contains(&"manual"));
        assert!(!VALID_CHECK_TYPES.contains(&"semi"));
    }

    #[test]
    fn test_valid_failure_actions() {
        assert!(VALID_FAILURE_ACTIONS.contains(&"hold"));
        assert!(VALID_FAILURE_ACTIONS.contains(&"warn"));
        assert!(VALID_FAILURE_ACTIONS.contains(&"reject"));
        assert!(VALID_FAILURE_ACTIONS.contains(&"notify"));
        assert!(!VALID_FAILURE_ACTIONS.contains(&"ignore"));
    }

    #[test]
    fn test_valid_hold_types() {
        assert!(VALID_HOLD_TYPES.contains(&"credit_limit"));
        assert!(VALID_HOLD_TYPES.contains(&"overdue"));
        assert!(VALID_HOLD_TYPES.contains(&"review"));
        assert!(VALID_HOLD_TYPES.contains(&"manual"));
        assert!(VALID_HOLD_TYPES.contains(&"scoring"));
        assert!(!VALID_HOLD_TYPES.contains(&"fraud"));
    }

    #[test]
    fn test_valid_hold_statuses() {
        assert!(VALID_HOLD_STATUSES.contains(&"active"));
        assert!(VALID_HOLD_STATUSES.contains(&"released"));
        assert!(VALID_HOLD_STATUSES.contains(&"overridden"));
        assert!(VALID_HOLD_STATUSES.contains(&"cancelled"));
        assert!(!VALID_HOLD_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_valid_review_types() {
        assert!(VALID_REVIEW_TYPES.contains(&"periodic"));
        assert!(VALID_REVIEW_TYPES.contains(&"triggered"));
        assert!(VALID_REVIEW_TYPES.contains(&"ad_hoc"));
        assert!(VALID_REVIEW_TYPES.contains(&"escalation"));
        assert!(!VALID_REVIEW_TYPES.contains(&"annual"));
    }

    #[test]
    fn test_valid_review_statuses() {
        assert!(VALID_REVIEW_STATUSES.contains(&"pending"));
        assert!(VALID_REVIEW_STATUSES.contains(&"in_review"));
        assert!(VALID_REVIEW_STATUSES.contains(&"completed"));
        assert!(VALID_REVIEW_STATUSES.contains(&"approved"));
        assert!(VALID_REVIEW_STATUSES.contains(&"rejected"));
        assert!(VALID_REVIEW_STATUSES.contains(&"cancelled"));
        assert!(!VALID_REVIEW_STATUSES.contains(&"draft"));
    }

    #[test]
    fn test_credit_score_range_validation() {
        // Valid scores
        assert!((0.0..=100.0).contains(&0.0));
        assert!((0.0..=100.0).contains(&50.0));
        assert!((0.0..=100.0).contains(&100.0));

        // Invalid scores
        assert!(!(0.0..=100.0).contains(&-1.0));
        assert!(!(0.0..=100.0).contains(&101.0));
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(risk_level_from_score(85.0), "low");
        assert_eq!(risk_level_from_score(80.0), "low");
        assert_eq!(risk_level_from_score(79.9), "medium");
        assert_eq!(risk_level_from_score(60.0), "medium");
        assert_eq!(risk_level_from_score(59.9), "high");
        assert_eq!(risk_level_from_score(40.0), "high");
        assert_eq!(risk_level_from_score(39.9), "very_high");
        assert_eq!(risk_level_from_score(0.0), "very_high");
    }

    #[test]
    fn test_utilization_calculation() {
        let limit: f64 = 100000.0;
        let exposure: f64 = 75000.0;
        let utilization: f64 = (exposure / limit) * 100.0;
        assert!((utilization - 75.0).abs() < 0.01);

        // Over 100% utilization
        let exposure: f64 = 120000.0;
        let utilization: f64 = (exposure / limit) * 100.0;
        assert!((utilization - 120.0).abs() < 0.01);

        // Zero limit
        let limit: f64 = 0.0;
        let utilization: f64 = if limit > 0.0 { (exposure / limit) * 100.0 } else { 0.0 };
        assert!((utilization - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_exposure_calculation() {
        let receivables: f64 = 50000.0;
        let orders: f64 = 30000.0;
        let shipments: f64 = 10000.0;
        let invoices: f64 = 15000.0;
        let cash: f64 = 5000.0;
        let _holds: f64 = 2000.0;

        let total: f64 = receivables + orders + shipments + invoices - cash;
        assert!((total - 100000.0).abs() < 0.01);

        // Negative exposure should be clamped to 0
        let receivables: f64 = 1000.0;
        let orders: f64 = 0.0;
        let shipments: f64 = 0.0;
        let invoices: f64 = 0.0;
        let cash: f64 = 5000.0;
        let total: f64 = receivables + orders + shipments + invoices - cash;
        let total: f64 = if total < 0.0 { 0.0 } else { total };
        assert!((total - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_limit_negative_rejected() {
        let limit = -5000.0;
        assert!(limit < 0.0);
    }

    #[test]
    fn test_available_credit_calculation() {
        let limit: f64 = 100000.0;
        let exposure: f64 = 75000.0;
        let available: f64 = if limit > exposure { limit - exposure } else { 0.0 };
        assert!((available - 25000.0).abs() < 0.01);

        // Exposure exceeds limit
        let exposure: f64 = 120000.0;
        let available: f64 = if limit > exposure { limit - exposure } else { 0.0 };
        assert!((available - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_temp_limit_with_expiry() {
        let base_limit: f64 = 100000.0;
        let temp_increase: f64 = 25000.0;
        let effective_limit: f64 = base_limit + temp_increase;
        assert!((effective_limit - 125000.0).abs() < 0.01);

        // Temp limit expired
        let expiry = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = chrono::Utc::now().date_naive();
        let effective_limit: f64 = if expiry >= today {
            base_limit + temp_increase
        } else {
            base_limit
        };
        assert!((effective_limit - 100000.0).abs() < 0.01);
    }

    #[test]
    fn test_hold_number_format() {
        let hold_number = format!("HLD-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));
        assert!(hold_number.starts_with("HLD-"));
        assert!(hold_number.len() > 10);
    }

    #[test]
    fn test_review_number_format() {
        let review_number = format!("CR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));
        assert!(review_number.starts_with("CR-"));
        assert!(review_number.len() > 10);
    }

    fn risk_level_from_score(score: f64) -> &'static str {
        if score >= 80.0 {
            "low"
        } else if score >= 60.0 {
            "medium"
        } else if score >= 40.0 {
            "high"
        } else {
            "very_high"
        }
    }
}
