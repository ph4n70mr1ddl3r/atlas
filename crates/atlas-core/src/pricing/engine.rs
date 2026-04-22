//! Advanced Pricing Engine Implementation
//!
//! Manages price lists, tiered pricing, discount rules, charge definitions,
//! pricing strategies, and price calculations.
//!
//! Oracle Fusion Cloud ERP equivalent: Order Management > Pricing > Advanced Pricing

use atlas_shared::{
    PriceList, PriceListLine, PriceTier, DiscountRule, ChargeDefinition,
    PricingStrategy, PriceCalculationLog,
    PriceCalculationResult, PriceCalculationStep,
    PricingDashboardSummary,
    AtlasError, AtlasResult,
};
use super::PricingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid price list types
const VALID_LIST_TYPES: &[&str] = &[
    "sale", "purchase", "transfer", "internal",
];

/// Valid pricing bases
const VALID_PRICING_BASES: &[&str] = &[
    "fixed", "cost_plus", "competitive", "tiered",
];

/// Valid discount types
const VALID_DISCOUNT_TYPES: &[&str] = &[
    "percentage", "fixed_amount", "fixed_price",
];

/// Valid application methods for discounts
const VALID_APPLICATION_METHODS: &[&str] = &[
    "line", "order", "group",
];

/// Valid stacking rules for discounts
const VALID_STACKING_RULES: &[&str] = &[
    "exclusive", "stackable", "best_price",
];

/// Valid charge types
const VALID_CHARGE_TYPES: &[&str] = &[
    "surcharge", "shipping", "handling", "insurance", "freight",
];

/// Valid charge categories
const VALID_CHARGE_CATEGORIES: &[&str] = &[
    "handling", "shipping", "insurance", "tax", "misc",
];

/// Valid charge calculation methods
const VALID_CHARGE_CALC_METHODS: &[&str] = &[
    "fixed", "percentage", "tiered", "formula",
];

/// Valid strategy types
const VALID_STRATEGY_TYPES: &[&str] = &[
    "price_list", "cost_plus", "competitive", "markup", "markdown",
];

/// Valid price types for tiers
const VALID_TIER_PRICE_TYPES: &[&str] = &[
    "fixed", "discount_from_list",
];

/// Advanced Pricing engine for managing price lists, discounts, charges, and calculations
pub struct PricingEngine {
    repository: Arc<dyn PricingRepository>,
}

impl PricingEngine {
    pub fn new(repository: Arc<dyn PricingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Price List Management
    // ========================================================================

    /// Create a new price list
    pub async fn create_price_list(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        currency_code: &str,
        list_type: &str,
        pricing_basis: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceList> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Price list code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Price list name is required".to_string(),
            ));
        }
        if !VALID_LIST_TYPES.contains(&list_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid list_type '{}'. Must be one of: {}", list_type, VALID_LIST_TYPES.join(", ")
            )));
        }
        if !VALID_PRICING_BASES.contains(&pricing_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pricing_basis '{}'. Must be one of: {}", pricing_basis, VALID_PRICING_BASES.join(", ")
            )));
        }
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "currency_code must be a 3-letter ISO currency code".to_string(),
            ));
        }

        info!("Creating price list '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_price_list(
            org_id, &code_upper, name, description, currency_code,
            list_type, pricing_basis, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a price list by code
    pub async fn get_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PriceList>> {
        self.repository.get_price_list(org_id, &code.to_uppercase()).await
    }

    /// List price lists with optional filters
    pub async fn list_price_lists(
        &self,
        org_id: Uuid,
        list_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PriceList>> {
        self.repository.list_price_lists(org_id, list_type, status).await
    }

    /// Activate a price list (draft -> active)
    pub async fn activate_price_list(&self, id: Uuid) -> AtlasResult<PriceList> {
        let pl = self.repository.get_price_list_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Price list {} not found", id)))?;

        if pl.status != "draft" && pl.status != "inactive" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate price list in '{}' status. Must be 'draft' or 'inactive'.", pl.status)
            ));
        }

        // Check that the price list has at least one line
        let lines = self.repository.list_price_list_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot activate a price list with no lines".to_string(),
            ));
        }

        info!("Activating price list {}", pl.code);
        self.repository.update_price_list_status(id, "active").await
    }

    /// Deactivate a price list
    pub async fn deactivate_price_list(&self, id: Uuid) -> AtlasResult<PriceList> {
        let pl = self.repository.get_price_list_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Price list {} not found", id)))?;

        if pl.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot deactivate price list in '{}' status. Must be 'active'.", pl.status)
            ));
        }

        info!("Deactivating price list {}", pl.code);
        self.repository.update_price_list_status(id, "inactive").await
    }

    /// Delete (soft-delete) a price list
    pub async fn delete_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting price list '{}' for org {}", code, org_id);
        self.repository.delete_price_list(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Price List Line Management
    // ========================================================================

    /// Add a line to a price list
    pub async fn add_price_list_line(
        &self,
        org_id: Uuid,
        price_list_id: Uuid,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        pricing_unit_of_measure: &str,
        list_price: &str,
        unit_price: &str,
        cost_price: &str,
        margin_percent: &str,
        minimum_quantity: &str,
        maximum_quantity: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceListLine> {
        let pl = self.repository.get_price_list_by_id(price_list_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Price list {} not found", price_list_id)))?;

        if pl.status == "expired" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to an expired price list".to_string(),
            ));
        }

        // Validate numeric fields
        let lp: f64 = list_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "list_price must be a valid number".to_string(),
        ))?;
        let up: f64 = unit_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "unit_price must be a valid number".to_string(),
        ))?;
        if lp < 0.0 || up < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Prices cannot be negative".to_string(),
            ));
        }

        let min_qty: f64 = minimum_quantity.parse().unwrap_or(1.0);
        if min_qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "minimum_quantity must be positive".to_string(),
            ));
        }

        // Get next line number
        let existing_lines = self.repository.list_price_list_lines(price_list_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        info!("Adding price list line {} to {}", line_number, pl.code);

        self.repository.create_price_list_line(
            org_id, price_list_id, line_number,
            item_id, item_code, item_description,
            pricing_unit_of_measure, list_price, unit_price, cost_price,
            margin_percent, minimum_quantity, maximum_quantity,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a price list line by ID
    pub async fn get_price_list_line(&self, id: Uuid) -> AtlasResult<Option<PriceListLine>> {
        self.repository.get_price_list_line(id).await
    }

    /// List lines for a price list
    pub async fn list_price_list_lines(&self, price_list_id: Uuid) -> AtlasResult<Vec<PriceListLine>> {
        self.repository.list_price_list_lines(price_list_id).await
    }

    /// Delete a price list line
    pub async fn delete_price_list_line(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_price_list_line(id).await
    }

    // ========================================================================
    // Price Tiers (Quantity Breaks)
    // ========================================================================

    /// Add a price tier to a price list line
    pub async fn add_price_tier(
        &self,
        org_id: Uuid,
        price_list_line_id: Uuid,
        from_quantity: &str,
        to_quantity: Option<&str>,
        price: &str,
        discount_percent: &str,
        price_type: &str,
    ) -> AtlasResult<PriceTier> {
        let line = self.repository.get_price_list_line(price_list_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Price list line {} not found", price_list_line_id)))?;

        if !VALID_TIER_PRICE_TYPES.contains(&price_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid price_type '{}'. Must be one of: {}", price_type, VALID_TIER_PRICE_TYPES.join(", ")
            )));
        }

        let from_qty: f64 = from_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "from_quantity must be a valid number".to_string(),
        ))?;
        let price_val: f64 = price.parse().map_err(|_| AtlasError::ValidationFailed(
            "price must be a valid number".to_string(),
        ))?;
        if from_qty < 0.0 || price_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "from_quantity and price must be non-negative".to_string(),
            ));
        }

        // Get next tier number
        let existing_tiers = self.repository.list_price_tiers(price_list_line_id).await?;
        let tier_number = (existing_tiers.len() + 1) as i32;

        info!("Adding price tier {} to line {}", tier_number, line.line_number);

        self.repository.create_price_tier(
            org_id, price_list_line_id, tier_number,
            from_quantity, to_quantity, price, discount_percent, price_type,
        ).await
    }

    /// List price tiers for a price list line
    pub async fn list_price_tiers(&self, price_list_line_id: Uuid) -> AtlasResult<Vec<PriceTier>> {
        self.repository.list_price_tiers(price_list_line_id).await
    }

    // ========================================================================
    // Discount Rule Management
    // ========================================================================

    /// Create a new discount rule
    pub async fn create_discount_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        discount_type: &str,
        discount_value: &str,
        application_method: &str,
        stacking_rule: &str,
        priority: i32,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        max_usage: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DiscountRule> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Discount rule code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Discount rule name is required".to_string(),
            ));
        }
        if !VALID_DISCOUNT_TYPES.contains(&discount_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid discount_type '{}'. Must be one of: {}", discount_type, VALID_DISCOUNT_TYPES.join(", ")
            )));
        }
        if !VALID_APPLICATION_METHODS.contains(&application_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid application_method '{}'. Must be one of: {}", application_method, VALID_APPLICATION_METHODS.join(", ")
            )));
        }
        if !VALID_STACKING_RULES.contains(&stacking_rule) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid stacking_rule '{}'. Must be one of: {}", stacking_rule, VALID_STACKING_RULES.join(", ")
            )));
        }

        let disc_val: f64 = discount_value.parse().map_err(|_| AtlasError::ValidationFailed(
            "discount_value must be a valid number".to_string(),
        ))?;
        if disc_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "discount_value cannot be negative".to_string(),
            ));
        }
        if discount_type == "percentage" && disc_val > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "percentage discount cannot exceed 100%".to_string(),
            ));
        }

        if let Some(max) = max_usage {
            if max < 0 {
                return Err(AtlasError::ValidationFailed(
                    "max_usage cannot be negative".to_string(),
                ));
            }
        }

        info!("Creating discount rule '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_discount_rule(
            org_id, &code_upper, name, description,
            discount_type, discount_value, application_method, stacking_rule,
            priority, condition, effective_from, effective_to,
            max_usage, created_by,
        ).await
    }

    /// Get a discount rule by code
    pub async fn get_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DiscountRule>> {
        self.repository.get_discount_rule(org_id, &code.to_uppercase()).await
    }

    /// List discount rules with optional status filter
    pub async fn list_discount_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DiscountRule>> {
        self.repository.list_discount_rules(org_id, status).await
    }

    /// Delete (soft-delete) a discount rule
    pub async fn delete_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting discount rule '{}' for org {}", code, org_id);
        self.repository.delete_discount_rule(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Charge Definition Management
    // ========================================================================

    /// Create a new charge definition
    pub async fn create_charge_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        charge_type: &str,
        charge_category: &str,
        calculation_method: &str,
        charge_amount: &str,
        charge_percent: &str,
        minimum_charge: &str,
        maximum_charge: Option<&str>,
        taxable: bool,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ChargeDefinition> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Charge definition code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Charge definition name is required".to_string(),
            ));
        }
        if !VALID_CHARGE_TYPES.contains(&charge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid charge_type '{}'. Must be one of: {}", charge_type, VALID_CHARGE_TYPES.join(", ")
            )));
        }
        if !VALID_CHARGE_CATEGORIES.contains(&charge_category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid charge_category '{}'. Must be one of: {}", charge_category, VALID_CHARGE_CATEGORIES.join(", ")
            )));
        }
        if !VALID_CHARGE_CALC_METHODS.contains(&calculation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation_method '{}'. Must be one of: {}", calculation_method, VALID_CHARGE_CALC_METHODS.join(", ")
            )));
        }

        info!("Creating charge definition '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_charge_definition(
            org_id, &code_upper, name, description,
            charge_type, charge_category, calculation_method,
            charge_amount, charge_percent, minimum_charge, maximum_charge,
            taxable, condition, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a charge definition by code
    pub async fn get_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ChargeDefinition>> {
        self.repository.get_charge_definition(org_id, &code.to_uppercase()).await
    }

    /// List charge definitions with optional type filter
    pub async fn list_charge_definitions(&self, org_id: Uuid, charge_type: Option<&str>) -> AtlasResult<Vec<ChargeDefinition>> {
        self.repository.list_charge_definitions(org_id, charge_type).await
    }

    /// Delete a charge definition
    pub async fn delete_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting charge definition '{}' for org {}", code, org_id);
        self.repository.delete_charge_definition(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Pricing Strategy Management
    // ========================================================================

    /// Create a new pricing strategy
    pub async fn create_pricing_strategy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        strategy_type: &str,
        priority: i32,
        condition: serde_json::Value,
        price_list_id: Option<Uuid>,
        markup_percent: &str,
        markdown_percent: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PricingStrategy> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Strategy code is required".to_string(),
            ));
        }
        if !VALID_STRATEGY_TYPES.contains(&strategy_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid strategy_type '{}'. Must be one of: {}", strategy_type, VALID_STRATEGY_TYPES.join(", ")
            )));
        }

        // Validate referenced price list exists if specified
        if let Some(pl_id) = price_list_id {
            let pl = self.repository.get_price_list_by_id(pl_id).await?;
            if pl.is_none() {
                return Err(AtlasError::EntityNotFound(
                    format!("Price list {} not found", pl_id)
                ));
            }
        }

        info!("Creating pricing strategy '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_pricing_strategy(
            org_id, &code_upper, name, description,
            strategy_type, priority, condition, price_list_id,
            markup_percent, markdown_percent,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a pricing strategy by code
    pub async fn get_pricing_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PricingStrategy>> {
        self.repository.get_pricing_strategy(org_id, &code.to_uppercase()).await
    }

    /// List pricing strategies
    pub async fn list_pricing_strategies(&self, org_id: Uuid) -> AtlasResult<Vec<PricingStrategy>> {
        self.repository.list_pricing_strategies(org_id).await
    }

    // ========================================================================
    // Price Calculation Engine
    // ========================================================================

    /// Calculate the price for a given item
    ///
    /// This method applies pricing strategies, price lists, tiered pricing,
    /// discount rules, and charge definitions to compute the final selling price.
    pub async fn calculate_price(
        &self,
        org_id: Uuid,
        item_code: &str,
        quantity: &str,
        currency_code: &str,
        entity_type: &str,
        entity_id: Uuid,
        line_id: Option<Uuid>,
        _customer_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceCalculationResult> {
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "quantity must be positive".to_string(),
            ));
        }

        let mut steps: Vec<PriceCalculationStep> = Vec::new();
        let mut applied_discount_rule_code: Option<String> = None;
        let mut applied_charge_code: Option<String> = None;
        let mut applied_price_list_code: Option<String> = None;
        let mut applied_strategy_code: Option<String> = None;
        let mut discount_rule_id: Option<Uuid> = None;
        let mut charge_def_id: Option<Uuid> = None;
        let mut strategy_id: Option<Uuid> = None;
        let mut price_list_id: Option<Uuid> = None;
        let mut tier_applied: Option<i32> = None;

        // Step 1: Find applicable strategy (by priority)
        let strategies = self.repository.list_pricing_strategies(org_id).await?;
        let today = chrono::Utc::now().date_naive();

        let strategy = strategies.into_iter()
            .filter(|s| s.is_active)
            .filter(|s| s.effective_from.is_none_or(|f| f <= today))
            .filter(|s| s.effective_to.is_none_or(|t| t >= today))
            .find(|s| {
                // Simple condition matching: if condition has item_codes, check if our item matches
                if let Some(codes) = s.condition.get("item_codes").and_then(|v| v.as_array()) {
                    codes.iter().any(|c| c.as_str() == Some(item_code))
                } else {
                    // No item filter means it applies to all
                    true
                }
            });

        // Step 2: Determine the base price from price list
        #[allow(unused_assignments)] let mut unit_list_price: f64 = 0.0;

        // First try strategy's price list
        let pl_id = if let Some(ref strat) = strategy {
            strategy_id = Some(strat.id);
            applied_strategy_code = Some(strat.code.clone());
            strat.price_list_id
        } else {
            None
        };

        let price_list_line = if let Some(plid) = pl_id {
            price_list_id = Some(plid);
            self.repository.find_price_list_line_by_item(plid, item_code).await?
        } else {
            // Fall back to first active sale price list that has this item
            let pls = self.repository.list_price_lists(org_id, Some("sale"), Some("active")).await?;
            let mut found = None;
            for pl in &pls {
                if let Some(line) = self.repository.find_price_list_line_by_item(pl.id, item_code).await? {
                    price_list_id = Some(pl.id);
                    applied_price_list_code = Some(pl.code.clone());
                    found = Some(line);
                    break;
                }
            }
            found
        };

        if let Some(ref pll) = price_list_line {
            unit_list_price = pll.unit_price.parse().unwrap_or(0.0);

            // Apply markup/markdown from strategy
            if let Some(ref strat) = strategy {
                let markup: f64 = strat.markup_percent.parse().unwrap_or(0.0);
                let markdown: f64 = strat.markdown_percent.parse().unwrap_or(0.0);
                if markup > 0.0 {
                    let before = unit_list_price;
                    unit_list_price = before * (1.0 + markup / 100.0);
                    steps.push(PriceCalculationStep {
                        step_type: "markup".to_string(),
                        description: format!("Apply {}% markup from strategy {}", markup, strat.code),
                        amount_before: format!("{:.4}", before),
                        amount_after: format!("{:.4}", unit_list_price),
                        rule_applied: Some(strat.code.clone()),
                    });
                }
                if markdown > 0.0 {
                    let before = unit_list_price;
                    unit_list_price = before * (1.0 - markdown / 100.0);
                    steps.push(PriceCalculationStep {
                        step_type: "markdown".to_string(),
                        description: format!("Apply {}% markdown from strategy {}", markdown, strat.code),
                        amount_before: format!("{:.4}", before),
                        amount_after: format!("{:.4}", unit_list_price),
                        rule_applied: Some(strat.code.clone()),
                    });
                }
            }

            // Step 3: Check for tiered pricing
            let tiers = self.repository.list_price_tiers(pll.id).await?;
            for tier in &tiers {
                let from_qty: f64 = tier.from_quantity.parse().unwrap_or(0.0);
                let to_qty: Option<f64> = tier.to_quantity.as_ref()
                    .and_then(|t| t.parse().ok());

                let in_range = qty >= from_qty && to_qty.is_none_or(|tq| qty <= tq);
                if in_range {
                    let before = unit_list_price;
                    if tier.price_type == "fixed" {
                        unit_list_price = tier.price.parse().unwrap_or(unit_list_price);
                    } else {
                        let disc_pct: f64 = tier.discount_percent.parse().unwrap_or(0.0);
                        unit_list_price *= 1.0 - disc_pct / 100.0;
                    }
                    tier_applied = Some(tier.tier_number);
                    steps.push(PriceCalculationStep {
                        step_type: "tier".to_string(),
                        description: format!("Apply tier {} pricing (qty >= {})", tier.tier_number, from_qty),
                        amount_before: format!("{:.4}", before),
                        amount_after: format!("{:.4}", unit_list_price),
                        rule_applied: None,
                    });
                    break;
                }
            }
        } else {
            return Err(AtlasError::EntityNotFound(
                format!("No price found for item '{}' in any active price list", item_code)
            ));
        }

        steps.push(PriceCalculationStep {
            step_type: "base_price".to_string(),
            description: format!("Base unit price for item {}", item_code),
            amount_before: "0.0000".to_string(),
            amount_after: format!("{:.4}", unit_list_price),
            rule_applied: applied_price_list_code.clone(),
        });

        // Step 4: Apply discount rules (sorted by priority, exclusive first)
        let discount_rules = self.repository.list_discount_rules(org_id, Some("active")).await?;
        let mut discount_amount: f64 = 0.0;

        let applicable_discounts: Vec<&DiscountRule> = discount_rules.iter()
            .filter(|d| d.is_active)
            .filter(|d| d.effective_from.is_none_or(|f| f <= today))
            .filter(|d| d.effective_to.is_none_or(|t| t >= today))
            .filter(|d| d.max_usage.is_none_or(|max| d.usage_count < max))
            .filter(|d| {
                // Check if discount applies to this item
                if let Some(codes) = d.condition.get("item_codes").and_then(|v| v.as_array()) {
                    codes.iter().any(|c| c.as_str() == Some(item_code))
                } else {
                    true
                }
            })
            .collect();

        // Find best discount
        let best_discount = applicable_discounts.iter().min_by_key(|d| d.priority);

        if let Some(disc) = best_discount {
            let disc_val: f64 = disc.discount_value.parse().unwrap_or(0.0);
            let before = unit_list_price;
            match disc.discount_type.as_str() {
                "percentage" => {
                    discount_amount = unit_list_price * disc_val / 100.0;
                }
                "fixed_amount" => {
                    discount_amount = disc_val;
                }
                "fixed_price" => {
                    discount_amount = unit_list_price - disc_val;
                }
                _ => {}
            }

            discount_rule_id = Some(disc.id);
            applied_discount_rule_code = Some(disc.code.clone());

            steps.push(PriceCalculationStep {
                step_type: "discount".to_string(),
                description: format!("Apply discount '{}' ({})", disc.name, disc.code),
                amount_before: format!("{:.4}", before),
                amount_after: format!("{:.4}", before - discount_amount),
                rule_applied: Some(disc.code.clone()),
            });

            // Increment usage
            self.repository.increment_discount_usage(disc.id).await?;
        }

        // Step 5: Apply charges
        let charges = self.repository.list_charge_definitions(org_id, None).await?;
        let mut charge_amount: f64 = 0.0;

        for charge in &charges {
            if !charge.is_active { continue; }
            if charge.effective_from.is_none_or(|f| f <= today) &&
               charge.effective_to.is_none_or(|t| t >= today) {
                let before_charge = charge_amount;
                match charge.calculation_method.as_str() {
                    "fixed" => {
                        charge_amount += charge.charge_amount.parse().unwrap_or(0.0);
                    }
                    "percentage" => {
                        let pct: f64 = charge.charge_percent.parse().unwrap_or(0.0);
                        charge_amount += unit_list_price * pct / 100.0;
                    }
                    _ => {}
                }

                // Enforce minimum
                let min_charge: f64 = charge.minimum_charge.parse().unwrap_or(0.0);
                if charge_amount < min_charge {
                    charge_amount = min_charge;
                }

                // Enforce maximum
                if let Some(max_str) = &charge.maximum_charge {
                    let max_charge: f64 = max_str.parse().unwrap_or(f64::MAX);
                    if charge_amount > max_charge {
                        charge_amount = max_charge;
                    }
                }

                charge_def_id = Some(charge.id);
                applied_charge_code = Some(charge.code.clone());

                steps.push(PriceCalculationStep {
                    step_type: "charge".to_string(),
                    description: format!("Apply charge '{}' ({})", charge.name, charge.code),
                    amount_before: format!("{:.4}", before_charge),
                    amount_after: format!("{:.4}", charge_amount),
                    rule_applied: Some(charge.code.clone()),
                });
                break; // Apply first matching charge only
            }
        }

        // Calculate final prices
        let unit_selling_price = unit_list_price - discount_amount;
        let extended_price = unit_selling_price * qty + charge_amount;

        // Log the calculation
        self.repository.create_calculation_log(
            org_id, entity_type, entity_id, line_id,
            price_list_line.as_ref().and_then(|l| l.item_id),
            Some(item_code), Some(quantity),
            &format!("{:.4}", unit_list_price),
            &format!("{:.4}", unit_selling_price),
            &format!("{:.4}", discount_amount),
            discount_rule_id,
            &format!("{:.4}", charge_amount),
            charge_def_id,
            strategy_id,
            price_list_id,
            serde_json::to_value(&steps).unwrap_or(serde_json::json!([])),
            currency_code,
            created_by,
        ).await?;

        Ok(PriceCalculationResult {
            list_price: format!("{:.4}", unit_list_price),
            discount_amount: format!("{:.4}", discount_amount),
            charge_amount: format!("{:.4}", charge_amount),
            unit_selling_price: format!("{:.4}", unit_selling_price),
            extended_price: format!("{:.4}", extended_price),
            currency_code: currency_code.to_string(),
            applied_discount_rule_code,
            applied_charge_code,
            applied_price_list_code,
            applied_strategy_code,
            tier_applied,
            calculation_steps: steps,
        })
    }

    /// Get price calculation logs
    pub async fn list_calculation_logs(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PriceCalculationLog>> {
        self.repository.list_calculation_logs(org_id, entity_type, entity_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get a pricing dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<PricingDashboardSummary> {
        let price_lists = self.repository.list_price_lists(org_id, None, None).await?;
        let discount_rules = self.repository.list_discount_rules(org_id, None).await?;
        let charge_defs = self.repository.list_charge_definitions(org_id, None).await?;
        let strategies = self.repository.list_pricing_strategies(org_id).await?;
        let calc_logs = self.repository.list_calculation_logs(org_id, None, None).await?;

        let today = chrono::Utc::now().date_naive();
        let total_calculations_today = calc_logs.iter()
            .filter(|l| l.calculation_date.date_naive() == today)
            .count() as i32;

        let active_price_lists = price_lists.iter().filter(|pl| pl.status == "active").count() as i32;
        let active_discount_rules = discount_rules.iter().filter(|dr| dr.status == "active" && dr.is_active).count() as i32;

        // Group by status
        let mut pl_by_status = serde_json::Map::new();
        for pl in &price_lists {
            let count = pl_by_status.entry(pl.status.clone())
                .or_insert(serde_json::Value::Number(0.into()));
            *count = serde_json::Value::Number((count.as_u64().unwrap_or(0) + 1).into());
        }

        let mut dr_by_type = serde_json::Map::new();
        for dr in &discount_rules {
            let count = dr_by_type.entry(dr.discount_type.clone())
                .or_insert(serde_json::Value::Number(0.into()));
            *count = serde_json::Value::Number((count.as_u64().unwrap_or(0) + 1).into());
        }

        let mut charges_by_type = serde_json::Map::new();
        for cd in &charge_defs {
            let count = charges_by_type.entry(cd.charge_type.clone())
                .or_insert(serde_json::Value::Number(0.into()));
            *count = serde_json::Value::Number((count.as_u64().unwrap_or(0) + 1).into());
        }

        Ok(PricingDashboardSummary {
            total_price_lists: price_lists.len() as i32,
            active_price_lists,
            total_discount_rules: discount_rules.len() as i32,
            active_discount_rules,
            total_charge_definitions: charge_defs.len() as i32,
            total_strategies: strategies.len() as i32,
            total_calculations_today,
            price_lists_by_status: serde_json::Value::Object(pl_by_status),
            discount_rules_by_type: serde_json::Value::Object(dr_by_type),
            charges_by_type: serde_json::Value::Object(charges_by_type),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_list_types() {
        assert!(VALID_LIST_TYPES.contains(&"sale"));
        assert!(VALID_LIST_TYPES.contains(&"purchase"));
        assert!(VALID_LIST_TYPES.contains(&"transfer"));
        assert!(VALID_LIST_TYPES.contains(&"internal"));
    }

    #[test]
    fn test_valid_discount_types() {
        assert!(VALID_DISCOUNT_TYPES.contains(&"percentage"));
        assert!(VALID_DISCOUNT_TYPES.contains(&"fixed_amount"));
        assert!(VALID_DISCOUNT_TYPES.contains(&"fixed_price"));
    }

    #[test]
    fn test_valid_charge_types() {
        assert!(VALID_CHARGE_TYPES.contains(&"surcharge"));
        assert!(VALID_CHARGE_TYPES.contains(&"shipping"));
        assert!(VALID_CHARGE_TYPES.contains(&"handling"));
        assert!(VALID_CHARGE_TYPES.contains(&"insurance"));
        assert!(VALID_CHARGE_TYPES.contains(&"freight"));
    }

    #[test]
    fn test_valid_strategy_types() {
        assert!(VALID_STRATEGY_TYPES.contains(&"price_list"));
        assert!(VALID_STRATEGY_TYPES.contains(&"cost_plus"));
        assert!(VALID_STRATEGY_TYPES.contains(&"competitive"));
        assert!(VALID_STRATEGY_TYPES.contains(&"markup"));
        assert!(VALID_STRATEGY_TYPES.contains(&"markdown"));
    }

    #[test]
    fn test_valid_stacking_rules() {
        assert!(VALID_STACKING_RULES.contains(&"exclusive"));
        assert!(VALID_STACKING_RULES.contains(&"stackable"));
        assert!(VALID_STACKING_RULES.contains(&"best_price"));
    }

    #[test]
    fn test_valid_charge_calc_methods() {
        assert!(VALID_CHARGE_CALC_METHODS.contains(&"fixed"));
        assert!(VALID_CHARGE_CALC_METHODS.contains(&"percentage"));
        assert!(VALID_CHARGE_CALC_METHODS.contains(&"tiered"));
        assert!(VALID_CHARGE_CALC_METHODS.contains(&"formula"));
    }
}
