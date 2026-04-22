//! Subscription Management Engine
//!
//! Manages subscription products, subscription lifecycle, amendments, billing schedules,
//! proration calculations, and ASC 606 / IFRS 15 revenue recognition scheduling.
//!
//! Subscription lifecycle: draft → active → suspended / cancelled / expired / renewed
//! Amendment lifecycle: draft → applied → cancelled
//! Billing schedule: generated on activation and amendments, tracks invoicing status
//! Revenue schedule: generated for ASC 606 compliance, tracks deferred vs recognized revenue
//!
//! Oracle Fusion Cloud ERP equivalent: Subscription Management

use atlas_shared::{
    SubscriptionProduct, SubscriptionPriceTier, Subscription, SubscriptionAmendment,
    SubscriptionBillingLine, SubscriptionRevenueLine, SubscriptionDashboardSummary,
    AtlasError, AtlasResult,
};
use super::SubscriptionRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid product types
#[allow(dead_code)]
const VALID_PRODUCT_TYPES: &[&str] = &["service", "software", "physical", "bundle"];

/// Valid billing frequencies
#[allow(dead_code)]
const VALID_BILLING_FREQUENCIES: &[&str] = &["monthly", "quarterly", "semi_annual", "annual", "one_time"];

/// Valid subscription statuses
#[allow(dead_code)]
const VALID_SUBSCRIPTION_STATUSES: &[&str] = &[
    "draft", "active", "suspended", "cancelled", "expired", "renewed",
];

/// Valid amendment types
#[allow(dead_code)]
const VALID_AMENDMENT_TYPES: &[&str] = &[
    "price_change", "quantity_change", "upgrade", "downgrade",
    "renewal", "cancellation", "suspension", "reactivation",
];

/// Valid amendment statuses
#[allow(dead_code)]
const VALID_AMENDMENT_STATUSES: &[&str] = &["draft", "applied", "cancelled"];

/// Valid tier types
#[allow(dead_code)]
const VALID_TIER_TYPES: &[&str] = &["flat", "volume", "tiered", "stairstep"];

/// Valid billing alignments
#[allow(dead_code)]
const VALID_BILLING_ALIGNMENTS: &[&str] = &["start_date", "first_of_month", "anniversary"];

/// Number of periods per year for each billing frequency
fn periods_per_year(frequency: &str) -> i32 {
    match frequency {
        "monthly" => 12,
        "quarterly" => 4,
        "semi_annual" => 2,
        "annual" => 1,
        "one_time" => 1,
        _ => 1,
    }
}

/// Number of months per billing period
fn months_per_period(frequency: &str) -> i32 {
    match frequency {
        "monthly" => 1,
        "quarterly" => 3,
        "semi_annual" => 6,
        "annual" => 12,
        "one_time" => 12,
        _ => 12,
    }
}

/// Subscription Management engine
pub struct SubscriptionEngine {
    repository: Arc<dyn SubscriptionRepository>,
}

impl SubscriptionEngine {
    pub fn new(repository: Arc<dyn SubscriptionRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Product Catalog Management
    // ========================================================================

    /// Create a new subscription product
    pub async fn create_product(
        &self,
        org_id: Uuid,
        product_code: &str,
        name: &str,
        description: Option<&str>,
        product_type: &str,
        billing_frequency: &str,
        default_duration_months: i32,
        is_auto_renew: bool,
        cancellation_notice_days: i32,
        setup_fee: &str,
        tier_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionProduct> {
        if product_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Product code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Product name is required".to_string()));
        }
        if !VALID_PRODUCT_TYPES.contains(&product_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid product type '{}'. Must be one of: {}",
                product_type,
                VALID_PRODUCT_TYPES.join(", ")
            )));
        }
        if !VALID_BILLING_FREQUENCIES.contains(&billing_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing frequency '{}'. Must be one of: {}",
                billing_frequency,
                VALID_BILLING_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_TIER_TYPES.contains(&tier_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tier type '{}'. Must be one of: {}",
                tier_type,
                VALID_TIER_TYPES.join(", ")
            )));
        }
        if default_duration_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Default duration must be positive".to_string(),
            ));
        }
        let setup: f64 = setup_fee.parse().map_err(|_| AtlasError::ValidationFailed(
            "Setup fee must be a valid number".to_string(),
        ))?;
        if setup < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Setup fee cannot be negative".to_string(),
            ));
        }

        info!(
            "Creating subscription product {} ({}) for org {}",
            product_code, name, org_id
        );

        self.repository
            .create_product(
                org_id,
                product_code,
                name,
                description,
                product_type,
                billing_frequency,
                default_duration_months,
                is_auto_renew,
                cancellation_notice_days,
                setup_fee,
                tier_type,
                created_by,
            )
            .await
    }

    /// Get a product by code
    pub async fn get_product(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SubscriptionProduct>> {
        self.repository.get_product(org_id, code).await
    }

    /// List products
    pub async fn list_products(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SubscriptionProduct>> {
        self.repository.list_products(org_id, active_only).await
    }

    /// Delete a product
    pub async fn delete_product(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_product(org_id, code).await
    }

    // ========================================================================
    // Price Tier Management
    // ========================================================================

    /// Add a price tier to a product
    pub async fn create_price_tier(
        &self,
        org_id: Uuid,
        product_id: Uuid,
        tier_name: Option<&str>,
        min_quantity: &str,
        max_quantity: Option<&str>,
        unit_price: &str,
        discount_percent: &str,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<SubscriptionPriceTier> {
        // Validate product exists
        let product = self
            .repository
            .get_product_by_id(product_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Product {} not found", product_id)))?;

        let min_q: f64 = min_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Min quantity must be a valid number".to_string(),
        ))?;
        let price: f64 = unit_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit price must be a valid number".to_string(),
        ))?;
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed("Unit price cannot be negative".to_string()));
        }
        if min_q < 0.0 {
            return Err(AtlasError::ValidationFailed("Min quantity cannot be negative".to_string()));
        }
        if let Some(max) = max_quantity {
            let max_q: f64 = max.parse().map_err(|_| AtlasError::ValidationFailed(
                "Max quantity must be a valid number".to_string(),
            ))?;
            if max_q < min_q {
                return Err(AtlasError::ValidationFailed(
                    "Max quantity cannot be less than min quantity".to_string(),
                ));
            }
        }

        info!(
            "Adding price tier to product {} ({})",
            product.product_code, product.name
        );

        self.repository
            .create_price_tier(
                org_id,
                product_id,
                tier_name,
                min_quantity,
                max_quantity,
                unit_price,
                discount_percent,
                currency_code,
                effective_from,
                effective_to,
            )
            .await
    }

    /// Get the applicable price for a product at a given quantity
    pub async fn get_applicable_price(
        &self,
        org_id: Uuid,
        product_id: Uuid,
        quantity: f64,
    ) -> AtlasResult<f64> {
        let tiers = self.repository.list_price_tiers(org_id, product_id).await?;

        if tiers.is_empty() {
            return Ok(0.0);
        }

        // Find the matching tier
        for tier in &tiers {
            let min_q: f64 = tier.min_quantity.parse().unwrap_or(0.0);
            let max_q: f64 = tier
                .max_quantity
                .as_deref()
                .and_then(|v| v.parse().ok())
                .unwrap_or(f64::MAX);

            if quantity >= min_q && quantity <= max_q {
                let unit_price: f64 = tier.unit_price.parse().unwrap_or(0.0);
                let discount: f64 = tier.discount_percent.parse().unwrap_or(0.0);
                return Ok(unit_price * (1.0 - discount / 100.0));
            }
        }

        // Default to first tier's price if no match
        let first_price: f64 = tiers[0].unit_price.parse().unwrap_or(0.0);
        Ok(first_price)
    }

    // ========================================================================
    // Subscription Lifecycle
    // ========================================================================

    /// Create a new subscription in draft status
    pub async fn create_subscription(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        product_id: Uuid,
        description: Option<&str>,
        start_date: chrono::NaiveDate,
        duration_months: i32,
        billing_frequency: Option<&str>,
        billing_day_of_month: Option<i32>,
        billing_alignment: Option<&str>,
        currency_code: &str,
        quantity: &str,
        discount_percent: &str,
        is_auto_renew: bool,
        sales_rep_id: Option<Uuid>,
        sales_rep_name: Option<&str>,
        gl_revenue_account: Option<&str>,
        gl_deferred_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Subscription> {
        if duration_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Duration must be positive".to_string(),
            ));
        }

        let quantity_val: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        if quantity_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive".to_string(),
            ));
        }

        // Validate product exists and get defaults
        let product = self
            .repository
            .get_product_by_id(product_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Product {} not found", product_id)))?;

        if !product.is_active {
            return Err(AtlasError::ValidationFailed(format!(
                "Product '{}' is not active",
                product.name
            )));
        }

        let freq = billing_frequency.unwrap_or(&product.billing_frequency);
        if !VALID_BILLING_FREQUENCIES.contains(&freq) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing frequency '{}'",
                freq
            )));
        }

        let alignment = billing_alignment.unwrap_or("start_date");
        if !VALID_BILLING_ALIGNMENTS.contains(&alignment) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing alignment '{}'",
                alignment
            )));
        }

        // Calculate pricing
        let unit_price = self
            .get_applicable_price(org_id, product_id, quantity_val)
            .await?;
        let discount: f64 = discount_percent.parse().unwrap_or(0.0);
        let effective_price = unit_price * (1.0 - discount / 100.0);
        let recurring_amount = effective_price * quantity_val;
        let total_periods = (duration_months as f64 / months_per_period(freq) as f64).ceil() as i32;
        let total_contract_value = recurring_amount * total_periods as f64;
        let setup: f64 = product.setup_fee.parse().unwrap_or(0.0);

        let end_date = start_date
            .checked_add_months(chrono::Months::new(duration_months as u32))
            .ok_or_else(|| AtlasError::ValidationFailed("Invalid date range".to_string()))?;

        let renewal_date = if is_auto_renew {
            Some(end_date)
        } else {
            None
        };

        let billing_day = billing_day_of_month.unwrap_or(1);
        let sub_number = format!("SUB-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!(
            "Creating subscription {} for customer {} (product: {})",
            sub_number,
            customer_id,
            product.product_code
        );

        let subscription = self
            .repository
            .create_subscription(
                org_id,
                &sub_number,
                customer_id,
                customer_name,
                product_id,
                Some(&product.product_code),
                Some(&product.name),
                description,
                "draft",
                start_date,
                Some(end_date),
                renewal_date.as_ref(),
                freq,
                billing_day,
                alignment,
                currency_code,
                quantity,
                &format!("{:.2}", effective_price),
                &format!("{:.2}", unit_price),
                discount_percent,
                &format!("{:.2}", setup),
                &format!("{:.2}", recurring_amount),
                &format!("{:.2}", total_contract_value),
                "0",
                "0",
                duration_months,
                is_auto_renew,
                None,  // cancellation_date
                None,  // cancellation_reason
                None,  // suspension_reason
                sales_rep_id,
                sales_rep_name,
                gl_revenue_account,
                gl_deferred_account,
                created_by,
            )
            .await?;

        Ok(subscription)
    }

    /// Get a subscription by ID
    pub async fn get_subscription(&self, id: Uuid) -> AtlasResult<Option<Subscription>> {
        self.repository.get_subscription(id).await
    }

    /// Get a subscription by number
    pub async fn get_subscription_by_number(
        &self,
        org_id: Uuid,
        number: &str,
    ) -> AtlasResult<Option<Subscription>> {
        self.repository.get_subscription_by_number(org_id, number).await
    }

    /// List subscriptions with optional filters
    pub async fn list_subscriptions(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Subscription>> {
        if let Some(s) = status {
            if !VALID_SUBSCRIPTION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_SUBSCRIPTION_STATUSES.join(", ")
                )));
            }
        }
        self.repository
            .list_subscriptions(org_id, status, customer_id)
            .await
    }

    /// Activate a draft subscription
    /// Generates the billing schedule and revenue schedule
    pub async fn activate_subscription(&self, sub_id: Uuid) -> AtlasResult<Subscription> {
        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate subscription in '{}' status. Must be 'draft'.",
                sub.status
            )));
        }

        // Generate billing schedule
        let billing_lines = self.generate_billing_schedule(&sub)?;
        for line in &billing_lines {
            self.repository
                .create_billing_line(
                    line.organization_id,
                    line.subscription_id,
                    line.schedule_number,
                    line.billing_date,
                    line.period_start,
                    line.period_end,
                    &line.amount,
                    &line.proration_amount,
                    &line.total_amount,
                )
                .await?;
        }

        // Generate revenue schedule (ASC 606)
        let revenue_lines = self.generate_revenue_schedule(&sub, &billing_lines)?;
        for line in &revenue_lines {
            self.repository
                .create_revenue_line(
                    line.organization_id,
                    line.subscription_id,
                    line.billing_schedule_id,
                    &line.period_name,
                    line.period_start,
                    line.period_end,
                    &line.revenue_amount,
                    &line.deferred_amount,
                    &line.recognized_to_date,
                    "deferred",
                )
                .await?;
        }

        info!("Activated subscription {} with {} billing periods and {} revenue periods", 
              sub.subscription_number, billing_lines.len(), revenue_lines.len());

        self.repository
            .update_subscription_status(sub_id, "active", None, None, None)
            .await
    }

    /// Suspend an active subscription
    pub async fn suspend_subscription(
        &self,
        sub_id: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<Subscription> {
        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot suspend subscription in '{}' status. Must be 'active'.",
                sub.status
            )));
        }

        info!("Suspended subscription {} ({})", sub.subscription_number, reason.unwrap_or("No reason provided"));
        self.repository
            .update_subscription_status(sub_id, "suspended", None, None, reason)
            .await
    }

    /// Reactivate a suspended subscription
    pub async fn reactivate_subscription(&self, sub_id: Uuid) -> AtlasResult<Subscription> {
        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reactivate subscription in '{}' status. Must be 'suspended'.",
                sub.status
            )));
        }

        info!("Reactivated subscription {}", sub.subscription_number);
        self.repository
            .update_subscription_status(sub_id, "active", None, None, None)
            .await
    }

    /// Cancel a subscription
    pub async fn cancel_subscription(
        &self,
        sub_id: Uuid,
        cancellation_date: chrono::NaiveDate,
        reason: Option<&str>,
    ) -> AtlasResult<Subscription> {
        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "active" && sub.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel subscription in '{}' status. Must be 'active' or 'suspended'.",
                sub.status
            )));
        }

        info!(
            "Cancelled subscription {} effective {}",
            sub.subscription_number, cancellation_date
        );

        self.repository
            .update_subscription_status(
                sub_id,
                "cancelled",
                Some(cancellation_date),
                reason,
                None,
            )
            .await
    }

    /// Renew a subscription (extend the end date)
    pub async fn renew_subscription(
        &self,
        sub_id: Uuid,
        new_duration_months: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Subscription> {
        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot renew subscription in '{}' status. Must be 'active'.",
                sub.status
            )));
        }

        let duration = new_duration_months.unwrap_or(sub.duration_months);
        let current_end = sub.end_date.unwrap_or(sub.start_date);
        let new_end = current_end
            .checked_add_months(chrono::Months::new(duration as u32))
            .ok_or_else(|| AtlasError::ValidationFailed("Invalid date range for renewal".to_string()))?;

        let new_renewal_date = if sub.is_auto_renew { Some(new_end) } else { None };

        // Create amendment for the renewal
        let amendment_number = format!("AMD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        self.repository
            .create_amendment(
                sub.organization_id,
                sub_id,
                &amendment_number,
                "renewal",
                Some(&format!("Renewed for {} months", duration)),
                None,
                None,
                None,
                None,
                Some(&sub.recurring_amount),
                Some(&sub.recurring_amount),
                sub.end_date.as_ref(),
                Some(&new_end),
                chrono::Utc::now().date_naive(),
                "0",
                "0",
                "draft",
                created_by,
            )
            .await?;

        // Update subscription end date and status
        let updated = self
            .repository
            .update_subscription_dates(sub_id, Some(new_end), new_renewal_date.as_ref())
            .await?;

        // Generate new billing and revenue schedules for the renewal period
        let renewal_billing = self.generate_billing_schedule_from(
            &sub,
            current_end,
            new_end,
        )?;
        for line in &renewal_billing {
            self.repository
                .create_billing_line(
                    line.organization_id,
                    line.subscription_id,
                    line.schedule_number,
                    line.billing_date,
                    line.period_start,
                    line.period_end,
                    &line.amount,
                    &line.proration_amount,
                    &line.total_amount,
                )
                .await?;
        }

        let renewal_revenue = self.generate_revenue_schedule(&sub, &renewal_billing)?;
        for line in &renewal_revenue {
            self.repository
                .create_revenue_line(
                    line.organization_id,
                    line.subscription_id,
                    line.billing_schedule_id,
                    &line.period_name,
                    line.period_start,
                    line.period_end,
                    &line.revenue_amount,
                    &line.deferred_amount,
                    &line.recognized_to_date,
                    "deferred",
                )
                .await?;
        }

        info!(
            "Renewed subscription {} for {} months, new end date: {}",
            sub.subscription_number, duration, new_end
        );

        Ok(updated)
    }

    // ========================================================================
    // Amendments
    // ========================================================================

    /// Create an amendment to change price or quantity on an active subscription
    pub async fn create_amendment(
        &self,
        sub_id: Uuid,
        amendment_type: &str,
        description: Option<&str>,
        new_quantity: Option<&str>,
        new_unit_price: Option<&str>,
        new_end_date: Option<chrono::NaiveDate>,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SubscriptionAmendment> {
        if !VALID_AMENDMENT_TYPES.contains(&amendment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid amendment type '{}'. Must be one of: {}",
                amendment_type,
                VALID_AMENDMENT_TYPES.join(", ")
            )));
        }

        let sub = self
            .repository
            .get_subscription(sub_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Subscription {} not found", sub_id)))?;

        if sub.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot amend subscription in '{}' status. Must be 'active'.",
                sub.status
            )));
        }

        // Calculate proration if mid-period change
        let (proration_credit, proration_charge) =
            self.calculate_proration(&sub, new_quantity, new_unit_price, effective_date)?;

        let new_recurring = if let (Some(q), Some(p)) = (new_quantity, new_unit_price) {
            let q: f64 = q.parse().unwrap_or(0.0);
            let p: f64 = p.parse().unwrap_or(0.0);
            Some(format!("{:.2}", q * p))
        } else if let Some(q) = new_quantity {
            let q: f64 = q.parse().unwrap_or(0.0);
            let price: f64 = sub.unit_price.parse().unwrap_or(0.0);
            Some(format!("{:.2}", q * price))
        } else {
            None
        };

        let amendment_number = format!("AMD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!(
            "Creating amendment {} ({}) for subscription {}",
            amendment_number, amendment_type, sub.subscription_number
        );

        self.repository
            .create_amendment(
                sub.organization_id,
                sub_id,
                &amendment_number,
                amendment_type,
                description,
                new_quantity.map(|_| sub.quantity.as_str()),
                new_quantity,
                new_unit_price.map(|_| sub.unit_price.as_str()),
                new_unit_price,
                Some(&sub.recurring_amount),
                new_recurring.as_deref(),
                sub.end_date.as_ref(),
                new_end_date.as_ref(),
                effective_date,
                &format!("{:.2}", proration_credit),
                &format!("{:.2}", proration_charge),
                "draft",
                created_by,
            )
            .await
    }

    /// Apply a draft amendment
    pub async fn apply_amendment(&self, amendment_id: Uuid, applied_by: Option<Uuid>) -> AtlasResult<SubscriptionAmendment> {
        let amendment = self
            .repository
            .get_amendment(amendment_id)
            .await?
            .ok_or_else(|| {
                AtlasError::EntityNotFound(format!("Amendment {} not found", amendment_id))
            })?;

        if amendment.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot apply amendment in '{}' status. Must be 'draft'.",
                amendment.status
            )));
        }

        // Update the subscription with amendment changes
        if let (Some(new_qty), Some(new_price)) = (&amendment.new_quantity, &amendment.new_unit_price) {
            let q: f64 = new_qty.parse().unwrap_or(0.0);
            let p: f64 = new_price.parse().unwrap_or(0.0);
            let recurring = q * p;
            self.repository
                .update_subscription_pricing(
                    amendment.subscription_id,
                    new_qty,
                    new_price,
                    &format!("{:.2}", recurring),
                )
                .await?;
        }

        // Create proration billing lines if applicable
        let credit: f64 = amendment
            .proration_credit
            .as_deref()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let charge: f64 = amendment
            .proration_charge
            .as_deref()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);

        if (credit - 0.0).abs() > f64::EPSILON || (charge - 0.0).abs() > f64::EPSILON {
            // Get existing billing lines count for schedule_number
            let existing = self
                .repository
                .list_billing_lines(amendment.subscription_id)
                .await?;
            let next_num = existing.len() as i32 + 1;

            self.repository
                .create_billing_line(
                    amendment.organization_id,
                    amendment.subscription_id,
                    next_num,
                    amendment.effective_date,
                    amendment.effective_date,
                    amendment.effective_date,
                    &format!("{:.2}", charge),
                    &format!("{:.2}", credit),
                    &format!("{:.2}", charge - credit),
                )
                .await?;
        }

        info!("Applied amendment {} to subscription", amendment.amendment_number);

        self.repository
            .update_amendment_status(amendment_id, "applied", applied_by)
            .await
    }

    /// Cancel a draft amendment
    pub async fn cancel_amendment(&self, amendment_id: Uuid) -> AtlasResult<SubscriptionAmendment> {
        let amendment = self
            .repository
            .get_amendment(amendment_id)
            .await?
            .ok_or_else(|| {
                AtlasError::EntityNotFound(format!("Amendment {} not found", amendment_id))
            })?;

        if amendment.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel amendment in '{}' status. Must be 'draft'.",
                amendment.status
            )));
        }

        info!("Cancelled amendment {}", amendment.amendment_number);
        self.repository
            .update_amendment_status(amendment_id, "cancelled", None)
            .await
    }

    /// List amendments for a subscription
    pub async fn list_amendments(&self, subscription_id: Uuid) -> AtlasResult<Vec<SubscriptionAmendment>> {
        self.repository.list_amendments(subscription_id).await
    }

    // ========================================================================
    // Billing & Revenue Queries
    // ========================================================================

    /// List billing schedule for a subscription
    pub async fn list_billing_schedule(
        &self,
        subscription_id: Uuid,
    ) -> AtlasResult<Vec<SubscriptionBillingLine>> {
        self.repository.list_billing_lines(subscription_id).await
    }

    /// List revenue schedule for a subscription
    pub async fn list_revenue_schedule(
        &self,
        subscription_id: Uuid,
    ) -> AtlasResult<Vec<SubscriptionRevenueLine>> {
        self.repository.list_revenue_lines(subscription_id).await
    }

    /// Recognize revenue for a specific period
    pub async fn recognize_revenue(&self, revenue_line_id: Uuid) -> AtlasResult<SubscriptionRevenueLine> {
        let line = self
            .repository
            .get_revenue_line(revenue_line_id)
            .await?
            .ok_or_else(|| {
                AtlasError::EntityNotFound(format!("Revenue line {} not found", revenue_line_id))
            })?;

        if line.status != "deferred" && line.status != "partially_recognized" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot recognize revenue in '{}' status.",
                line.status
            )));
        }

        info!("Recognized revenue for period {}", line.period_name);
        self.repository
            .update_revenue_line_status(revenue_line_id, "recognized")
            .await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get subscription dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SubscriptionDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Schedule Generation (Internal)
    // ========================================================================

    /// Generate the full billing schedule for a subscription
    fn generate_billing_schedule(
        &self,
        sub: &Subscription,
    ) -> AtlasResult<Vec<SubscriptionBillingLine>> {
        let mut lines = Vec::new();
        let recurring: f64 = sub.recurring_amount.parse().unwrap_or(0.0);

        let _periods = periods_per_year(&sub.billing_frequency);
        let months_per = months_per_period(&sub.billing_frequency);
        let total_periods = (sub.duration_months as f64 / months_per as f64).ceil() as i32;

        let mut period_start = sub.start_date;

        for i in 0..total_periods {
            let period_end = period_start
                .checked_add_months(chrono::Months::new(months_per as u32))
                .and_then(|d| d.pred_opt())
                .unwrap_or(period_start);

            let billing_date = self.calculate_billing_date(period_start, sub.billing_day_of_month);

            lines.push(SubscriptionBillingLine {
                id: Uuid::new_v4(),
                organization_id: sub.organization_id,
                subscription_id: sub.id,
                schedule_number: i + 1,
                billing_date,
                period_start,
                period_end,
                amount: format!("{:.2}", recurring),
                proration_amount: "0.00".to_string(),
                total_amount: format!("{:.2}", recurring),
                invoice_id: None,
                invoice_number: None,
                status: "pending".to_string(),
                paid_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });

            period_start = period_end
                .succ_opt()
                .unwrap_or(period_start);
        }

        Ok(lines)
    }

    /// Generate billing schedule from a given start date to end date (for renewals)
    fn generate_billing_schedule_from(
        &self,
        sub: &Subscription,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> AtlasResult<Vec<SubscriptionBillingLine>> {
        let mut lines = Vec::new();
        let recurring: f64 = sub.recurring_amount.parse().unwrap_or(0.0);
        let months_per = months_per_period(&sub.billing_frequency);

        let mut period_start = start;
        let mut schedule_num = 1;

        while period_start < end {
            let period_end = std::cmp::min(
                period_start
                    .checked_add_months(chrono::Months::new(months_per as u32))
                    .and_then(|d| d.pred_opt())
                    .unwrap_or(end),
                end,
            );

            let billing_date = self.calculate_billing_date(period_start, sub.billing_day_of_month);

            // Proration for last period if it's shorter than a full period
            let full_period_days = months_per as i32 * 30;
            let actual_days = (period_end - period_start).num_days() + 1;
            let proration_factor = if actual_days < full_period_days as i64 {
                actual_days as f64 / full_period_days as f64
            } else {
                1.0
            };

            let amount = recurring * proration_factor;

            lines.push(SubscriptionBillingLine {
                id: Uuid::new_v4(),
                organization_id: sub.organization_id,
                subscription_id: sub.id,
                schedule_number: schedule_num,
                billing_date,
                period_start,
                period_end,
                amount: format!("{:.2}", amount),
                proration_amount: if proration_factor < 1.0 {
                    format!("{:.2}", recurring - amount)
                } else {
                    "0.00".to_string()
                },
                total_amount: format!("{:.2}", amount),
                invoice_id: None,
                invoice_number: None,
                status: "pending".to_string(),
                paid_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });

            schedule_num += 1;
            period_start = period_end.succ_opt().unwrap_or(end);
        }

        Ok(lines)
    }

    /// Generate the revenue recognition schedule (ASC 606 / IFRS 15)
    fn generate_revenue_schedule(
        &self,
        sub: &Subscription,
        billing_lines: &[SubscriptionBillingLine],
    ) -> AtlasResult<Vec<SubscriptionRevenueLine>> {
        let mut lines = Vec::new();
        let total_contract: f64 = sub.total_contract_value.parse().unwrap_or(0.0);
        let total_periods = billing_lines.len() as f64;

        if total_periods == 0.0 {
            return Ok(lines);
        }

        let revenue_per_period = total_contract / total_periods;
        let mut recognized_to_date = 0.0;

        for (i, billing) in billing_lines.iter().enumerate() {
            let is_last = i == billing_lines.len() - 1;
            // For the last period, adjust to account for rounding
            let revenue = if is_last {
                total_contract - recognized_to_date
            } else {
                revenue_per_period
            };

            let period_name = billing.period_start.format("%Y-%m").to_string();

            recognized_to_date += revenue;
            let deferred = total_contract - recognized_to_date;

            lines.push(SubscriptionRevenueLine {
                id: Uuid::new_v4(),
                organization_id: sub.organization_id,
                subscription_id: sub.id,
                billing_schedule_id: Some(billing.id),
                period_name,
                period_start: billing.period_start,
                period_end: billing.period_end,
                revenue_amount: format!("{:.2}", revenue),
                deferred_amount: format!("{:.2}", deferred),
                recognized_to_date: format!("{:.2}", recognized_to_date),
                status: "deferred".to_string(),
                recognized_at: None,
                journal_entry_id: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });
        }

        Ok(lines)
    }

    /// Calculate the billing date for a period start
    fn calculate_billing_date(
        &self,
        period_start: chrono::NaiveDate,
        billing_day: i32,
    ) -> chrono::NaiveDate {
        let month: u32 = period_start.format("%m").to_string().parse().unwrap_or(1);
        let year: i32 = period_start.format("%Y").to_string().parse().unwrap_or(2024);
        let day = std::cmp::min(billing_day, 28);
        chrono::NaiveDate::from_ymd_opt(year, month, day as u32)
            .unwrap_or(period_start)
    }

    /// Calculate proration credit/charge for a mid-period change
    fn calculate_proration(
        &self,
        sub: &Subscription,
        new_quantity: Option<&str>,
        new_unit_price: Option<&str>,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<(f64, f64)> {
        let current_recurring: f64 = sub.recurring_amount.parse().unwrap_or(0.0);

        let new_recurring = if let (Some(q), Some(p)) = (new_quantity, new_unit_price) {
            let q: f64 = q.parse().unwrap_or(0.0);
            let p: f64 = p.parse().unwrap_or(0.0);
            q * p
        } else if let Some(q) = new_quantity {
            let q: f64 = q.parse().unwrap_or(0.0);
            let price: f64 = sub.unit_price.parse().unwrap_or(0.0);
            q * price
        } else if let Some(p) = new_unit_price {
            let qty: f64 = sub.quantity.parse().unwrap_or(0.0);
            let p: f64 = p.parse().unwrap_or(0.0);
            qty * p
        } else {
            return Ok((0.0, 0.0));
        };

        // Calculate days remaining in current period
        let months_per = months_per_period(&sub.billing_frequency);
        let period_end = effective_date
            .checked_add_months(chrono::Months::new(months_per as u32))
            .and_then(|d| d.pred_opt())
            .unwrap_or(effective_date);

        let total_days = (period_end - sub.start_date).num_days().max(1) as f64;
        let remaining_days = (period_end - effective_date).num_days().max(0) as f64;
        let fraction = remaining_days / total_days;

        // Credit for unused portion at old rate, charge for remaining at new rate
        let credit = current_recurring * fraction;
        let charge = new_recurring * fraction;

        Ok((credit, charge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_product_types() {
        assert!(VALID_PRODUCT_TYPES.contains(&"service"));
        assert!(VALID_PRODUCT_TYPES.contains(&"software"));
        assert!(VALID_PRODUCT_TYPES.contains(&"physical"));
        assert!(VALID_PRODUCT_TYPES.contains(&"bundle"));
    }

    #[test]
    fn test_valid_billing_frequencies() {
        assert!(VALID_BILLING_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_BILLING_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_BILLING_FREQUENCIES.contains(&"semi_annual"));
        assert!(VALID_BILLING_FREQUENCIES.contains(&"annual"));
        assert!(VALID_BILLING_FREQUENCIES.contains(&"one_time"));
    }

    #[test]
    fn test_valid_subscription_statuses() {
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"draft"));
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"active"));
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"suspended"));
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"cancelled"));
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"expired"));
        assert!(VALID_SUBSCRIPTION_STATUSES.contains(&"renewed"));
    }

    #[test]
    fn test_valid_amendment_types() {
        assert!(VALID_AMENDMENT_TYPES.contains(&"price_change"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"quantity_change"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"upgrade"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"downgrade"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"renewal"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"cancellation"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"suspension"));
        assert!(VALID_AMENDMENT_TYPES.contains(&"reactivation"));
    }

    #[test]
    fn test_periods_per_year() {
        assert_eq!(periods_per_year("monthly"), 12);
        assert_eq!(periods_per_year("quarterly"), 4);
        assert_eq!(periods_per_year("semi_annual"), 2);
        assert_eq!(periods_per_year("annual"), 1);
        assert_eq!(periods_per_year("one_time"), 1);
    }

    #[test]
    fn test_months_per_period() {
        assert_eq!(months_per_period("monthly"), 1);
        assert_eq!(months_per_period("quarterly"), 3);
        assert_eq!(months_per_period("semi_annual"), 6);
        assert_eq!(months_per_period("annual"), 12);
    }

    #[test]
    fn test_billing_schedule_generation_monthly_12_months() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-TEST01".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "draft".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "monthly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "100.00".to_string(),
            list_price: "100.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "100.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let lines = engine.generate_billing_schedule(&sub).unwrap();
        assert_eq!(lines.len(), 12, "Expected 12 monthly billing lines");
        assert_eq!(lines[0].amount, "100.00");
        assert_eq!(lines[0].billing_date, chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(lines[11].billing_date, chrono::NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
    }

    #[test]
    fn test_billing_schedule_generation_quarterly_12_months() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-TEST02".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "draft".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "quarterly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "300.00".to_string(),
            list_price: "300.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "300.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let lines = engine.generate_billing_schedule(&sub).unwrap();
        assert_eq!(lines.len(), 4, "Expected 4 quarterly billing lines");
        assert_eq!(lines[0].amount, "300.00");
    }

    #[test]
    fn test_billing_schedule_generation_annual() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-TEST03".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "draft".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "annual".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "1200.00".to_string(),
            list_price: "1200.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "1200.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let lines = engine.generate_billing_schedule(&sub).unwrap();
        assert_eq!(lines.len(), 1, "Expected 1 annual billing line");
        assert_eq!(lines[0].amount, "1200.00");
    }

    #[test]
    fn test_revenue_schedule_generation() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-TEST04".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "draft".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "monthly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "100.00".to_string(),
            list_price: "100.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "100.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let billing = engine.generate_billing_schedule(&sub).unwrap();
        let revenue = engine.generate_revenue_schedule(&sub, &billing).unwrap();

        assert_eq!(revenue.len(), 12, "Expected 12 revenue lines");
        assert_eq!(revenue[0].revenue_amount, "100.00");
        assert_eq!(revenue[0].status, "deferred");
        assert_eq!(revenue[0].period_name, "2024-01");

        // Last line should account for rounding
        let last = &revenue[11];
        let total_recognized: f64 = last.recognized_to_date.parse().unwrap();
        assert!(
            (total_recognized - 1200.0).abs() < 0.01,
            "Total recognized should be ~1200.00, got {:.2}",
            total_recognized
        );

        // Check deferred decreases monotonically
        for i in 1..revenue.len() {
            let prev_defer: f64 = revenue[i - 1].deferred_amount.parse().unwrap();
            let curr_defer: f64 = revenue[i].deferred_amount.parse().unwrap();
            assert!(
                curr_defer <= prev_defer,
                "Deferred should decrease: {} > {} at index {}",
                curr_defer,
                prev_defer,
                i
            );
        }
    }

    #[test]
    fn test_revenue_schedule_quarterly() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-TEST05".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "draft".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "quarterly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "5".to_string(),
            unit_price: "50.00".to_string(),
            list_price: "50.00".to_string(),
            discount_percent: "10".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "225.00".to_string(), // 5 * 50 * 0.9
            total_contract_value: "900.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let billing = engine.generate_billing_schedule(&sub).unwrap();
        let revenue = engine.generate_revenue_schedule(&sub, &billing).unwrap();

        assert_eq!(revenue.len(), 4, "Expected 4 quarterly revenue lines");
        let total: f64 = revenue.iter().map(|r| r.revenue_amount.parse::<f64>().unwrap()).sum();
        assert!(
            (total - 900.0).abs() < 0.01,
            "Total revenue should be ~900.00, got {:.2}",
            total
        );
    }

    #[test]
    fn test_proration_calculation() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-PRORATE".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "active".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "monthly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "100.00".to_string(),
            list_price: "100.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "100.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Upgrade mid-month: quantity change from 1 to 2 on Jan 16
        let (credit, charge) = engine
            .calculate_proration(
                &sub,
                Some("2"),
                Some("100.00"),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            )
            .unwrap();

        // Charge should be greater than credit (upgrade)
        assert!(charge > credit, "Upgrade: charge ({:.2}) should be > credit ({:.2})", charge, credit);
        assert!(credit >= 0.0, "Credit should be non-negative");
        assert!(charge >= 0.0, "Charge should be non-negative");
    }

    #[test]
    fn test_proration_no_change() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let sub = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            subscription_number: "SUB-NOCHANGE".to_string(),
            customer_id: Uuid::new_v4(),
            customer_name: None,
            product_id: Uuid::new_v4(),
            product_code: None,
            product_name: None,
            description: None,
            status: "active".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            renewal_date: None,
            billing_frequency: "monthly".to_string(),
            billing_day_of_month: 1,
            billing_alignment: "start_date".to_string(),
            currency_code: "USD".to_string(),
            quantity: "1".to_string(),
            unit_price: "100.00".to_string(),
            list_price: "100.00".to_string(),
            discount_percent: "0".to_string(),
            setup_fee: "0".to_string(),
            recurring_amount: "100.00".to_string(),
            total_contract_value: "1200.00".to_string(),
            total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(),
            duration_months: 12,
            is_auto_renew: false,
            cancellation_date: None,
            cancellation_reason: None,
            suspension_reason: None,
            sales_rep_id: None,
            sales_rep_name: None,
            gl_revenue_account: None,
            gl_deferred_account: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (credit, charge) = engine
            .calculate_proration(
                &sub,
                None,
                None,
                chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            )
            .unwrap();
        assert_eq!(credit, 0.0);
        assert_eq!(charge, 0.0);
    }

    #[test]
    fn test_calculate_billing_date() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        let date = engine.calculate_billing_date(
            chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            15,
        );
        assert_eq!(date, chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn test_calculate_billing_date_clamp() {
        let engine = SubscriptionEngine::new(Arc::new(crate::MockSubscriptionRepository));
        // Day 31 should be clamped to 28 (safest approach for February)
        let date = engine.calculate_billing_date(
            chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            31,
        );
        assert_eq!(date, chrono::NaiveDate::from_ymd_opt(2024, 2, 28).unwrap());
    }
}
