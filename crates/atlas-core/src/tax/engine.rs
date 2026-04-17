//! Tax Engine Implementation
//!
//! Manages tax regimes, jurisdictions, rates, determination rules,
//! and provides tax calculation for transactions.
//!
//! Oracle Fusion Cloud ERP equivalent: Tax > Tax Configuration and Calculation

use atlas_shared::{
    TaxRegime, TaxJurisdiction, TaxRate, TaxDeterminationRule, TaxLine,
    TaxCalculationRequest, TaxCalculationResult, TaxLineResult,
    AtlasError, AtlasResult,
};
use super::TaxRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid tax types
const VALID_TAX_TYPES: &[&str] = &[
    "sales_tax", "vat", "gst", "withholding", "excise", "customs",
];

/// Valid rate types
const VALID_RATE_TYPES: &[&str] = &[
    "standard", "reduced", "zero", "exempt",
];

/// Valid rounding rules
const VALID_ROUNDING_RULES: &[&str] = &[
    "nearest", "up", "down", "none",
];

/// Valid geographic levels
const VALID_GEOGRAPHIC_LEVELS: &[&str] = &[
    "country", "state", "county", "city", "region",
];

/// Tax engine for managing tax configuration and calculations
pub struct TaxEngine {
    repository: Arc<dyn TaxRepository>,
}

impl TaxEngine {
    pub fn new(repository: Arc<dyn TaxRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Tax Regime Management
    // ========================================================================

    /// Create a new tax regime
    pub async fn create_regime(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        default_inclusive: bool,
        allows_recovery: bool,
        rounding_rule: &str,
        rounding_precision: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegime> {
        // Validate
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Tax regime code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Tax regime name is required".to_string(),
            ));
        }
        if !VALID_TAX_TYPES.contains(&tax_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_type '{}'. Must be one of: {}", tax_type, VALID_TAX_TYPES.join(", ")
            )));
        }
        if !VALID_ROUNDING_RULES.contains(&rounding_rule) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rounding_rule '{}'. Must be one of: {}", rounding_rule, VALID_ROUNDING_RULES.join(", ")
            )));
        }
        if !(0..=6).contains(&rounding_precision) {
            return Err(AtlasError::ValidationFailed(
                "Rounding precision must be between 0 and 6".to_string(),
            ));
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Creating tax regime '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_regime(
            org_id, &code_upper, name, description, tax_type,
            default_inclusive, allows_recovery, rounding_rule,
            rounding_precision, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a tax regime by code
    pub async fn get_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxRegime>> {
        self.repository.get_regime(org_id, &code.to_uppercase()).await
    }

    /// List all tax regimes for an organization
    pub async fn list_regimes(&self, org_id: Uuid) -> AtlasResult<Vec<TaxRegime>> {
        self.repository.list_regimes(org_id).await
    }

    /// Deactivate a tax regime
    pub async fn delete_regime(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating tax regime '{}' for org {}", code, org_id);
        self.repository.delete_regime(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Tax Jurisdiction Management
    // ========================================================================

    /// Create a new tax jurisdiction
    pub async fn create_jurisdiction(
        &self,
        org_id: Uuid,
        regime_code: &str,
        code: &str,
        name: &str,
        geographic_level: &str,
        country_code: Option<&str>,
        state_code: Option<&str>,
        county: Option<&str>,
        city: Option<&str>,
        postal_code_pattern: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxJurisdiction> {
        // Validate regime exists
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;

        if code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Jurisdiction code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Jurisdiction name is required".to_string(),
            ));
        }
        if !VALID_GEOGRAPHIC_LEVELS.contains(&geographic_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid geographic_level '{}'. Must be one of: {}",
                geographic_level, VALID_GEOGRAPHIC_LEVELS.join(", ")
            )));
        }

        info!("Creating tax jurisdiction '{}' for regime {} org {}", code, regime_code, org_id);

        self.repository.create_jurisdiction(
            org_id, regime.id, &code.to_uppercase(), name,
            geographic_level, country_code, state_code, county, city,
            postal_code_pattern, created_by,
        ).await
    }

    /// List jurisdictions for a regime
    pub async fn list_jurisdictions(
        &self,
        org_id: Uuid,
        regime_code: Option<&str>,
    ) -> AtlasResult<Vec<TaxJurisdiction>> {
        match regime_code {
            Some(rc) => {
                let regime = self.get_regime(org_id, rc).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(
                        format!("Tax regime '{}' not found", rc)
                    ))?;
                self.repository.list_jurisdictions(org_id, Some(regime.id)).await
            }
            None => self.repository.list_jurisdictions(org_id, None).await,
        }
    }

    /// Get a jurisdiction by code
    pub async fn get_jurisdiction(&self, org_id: Uuid, regime_code: &str, code: &str) -> AtlasResult<Option<TaxJurisdiction>> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        self.repository.get_jurisdiction(org_id, regime.id, &code.to_uppercase()).await
    }

    /// Deactivate a jurisdiction
    pub async fn delete_jurisdiction(&self, org_id: Uuid, regime_code: &str, code: &str) -> AtlasResult<()> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        info!("Deactivating jurisdiction '{}' for org {}", code, org_id);
        self.repository.delete_jurisdiction(org_id, regime.id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Tax Rate Management
    // ========================================================================

    /// Create a new tax rate
    pub async fn create_tax_rate(
        &self,
        org_id: Uuid,
        regime_code: &str,
        jurisdiction_code: Option<&str>,
        code: &str,
        name: &str,
        rate_percentage: &str,
        rate_type: &str,
        tax_account_code: Option<&str>,
        recoverable: bool,
        recovery_percentage: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRate> {
        // Validate regime exists
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;

        // Validate jurisdiction if provided
        let jurisdiction_id = if let Some(jc) = jurisdiction_code {
            let juris = self.repository.get_jurisdiction(org_id, regime.id, &jc.to_uppercase()).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Tax jurisdiction '{}' not found in regime '{}'", jc, regime_code)
                ))?;
            Some(juris.id)
        } else {
            None
        };

        // Validate inputs
        let rate_pct: f64 = rate_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "rate_percentage must be a valid number".to_string(),
        ))?;
        if rate_pct < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "rate_percentage must be non-negative".to_string(),
            ));
        }

        if !VALID_RATE_TYPES.contains(&rate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate_type '{}'. Must be one of: {}", rate_type, VALID_RATE_TYPES.join(", ")
            )));
        }

        if let Some(to) = effective_to {
            if effective_from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        let recovery_pct = if let Some(rp) = recovery_percentage {
            let val: f64 = rp.parse().map_err(|_| AtlasError::ValidationFailed(
                "recovery_percentage must be a valid number".to_string(),
            ))?;
            if val < 0.0 || val > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "recovery_percentage must be between 0 and 100".to_string(),
                ));
            }
            Some(rp.to_string())
        } else {
            None
        };

        info!("Creating tax rate '{}' ({}) = {}% for org {}", code, name, rate_percentage, org_id);

        self.repository.create_tax_rate(
            org_id, regime.id, jurisdiction_id.as_ref(),
            code, name, rate_percentage, rate_type,
            tax_account_code, recoverable, recovery_pct.as_deref(),
            effective_from, effective_to, created_by,
        ).await
    }

    /// List tax rates for a regime
    pub async fn list_tax_rates(
        &self,
        org_id: Uuid,
        regime_code: &str,
    ) -> AtlasResult<Vec<TaxRate>> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        self.repository.list_tax_rates(org_id, regime.id).await
    }

    /// Get a specific tax rate
    pub async fn get_tax_rate(&self, org_id: Uuid, regime_code: &str, code: &str) -> AtlasResult<Option<TaxRate>> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        self.repository.get_tax_rate(org_id, regime.id, code).await
    }

    /// Get a tax rate by ID (used during calculation)
    pub async fn get_tax_rate_by_id(&self, id: Uuid) -> AtlasResult<Option<TaxRate>> {
        self.repository.get_tax_rate_by_id(id).await
    }

    /// Get tax rates effective on a specific date for determination
    pub async fn get_effective_tax_rates(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        on_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<TaxRate>> {
        self.repository.get_effective_tax_rates(org_id, regime_id, on_date).await
    }

    /// Deactivate a tax rate
    pub async fn delete_tax_rate(&self, org_id: Uuid, regime_code: &str, code: &str) -> AtlasResult<()> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        info!("Deactivating tax rate '{}' for org {}", code, org_id);
        self.repository.delete_tax_rate(org_id, regime.id, code).await
    }

    // ========================================================================
    // Tax Determination Rules
    // ========================================================================

    /// Create a determination rule
    pub async fn create_determination_rule(
        &self,
        org_id: Uuid,
        regime_code: &str,
        name: &str,
        description: Option<&str>,
        priority: i32,
        condition: serde_json::Value,
        action: serde_json::Value,
        stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxDeterminationRule> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;

        // Validate action has tax_rate_codes
        if let Some(codes) = action.get("tax_rate_codes") {
            if !codes.is_array() || codes.as_array().unwrap().is_empty() {
                return Err(AtlasError::ValidationFailed(
                    "action.tax_rate_codes must be a non-empty array".to_string(),
                ));
            }
        } else {
            return Err(AtlasError::ValidationFailed(
                "action must contain 'tax_rate_codes'".to_string(),
            ));
        }

        info!("Creating tax determination rule '{}' for regime {} org {}", name, regime_code, org_id);

        self.repository.create_determination_rule(
            org_id, regime.id, name, description, priority,
            condition, action, stop_on_match,
            effective_from, effective_to, created_by,
        ).await
    }

    /// List determination rules for a regime
    pub async fn list_determination_rules(
        &self,
        org_id: Uuid,
        regime_code: &str,
    ) -> AtlasResult<Vec<TaxDeterminationRule>> {
        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;
        self.repository.list_determination_rules(org_id, regime.id).await
    }

    /// Find matching determination rules for a given context
    pub async fn find_matching_rules(
        &self,
        org_id: Uuid,
        regime_id: Uuid,
        context: &serde_json::Value,
    ) -> AtlasResult<Vec<TaxDeterminationRule>> {
        let all_rules = self.repository.list_determination_rules(org_id, regime_id).await?;
        let today = chrono::Utc::now().date_naive();

        let mut matching = Vec::new();
        for rule in all_rules {
            if !rule.is_active { continue; }
            if let Some(from) = rule.effective_from {
                if today < from { continue; }
            }
            if let Some(to) = rule.effective_to {
                if today > to { continue; }
            }
            if self.matches_condition(&rule.condition, context) {
                matching.push(rule);
            }
        }

        // Sort by priority (lower first)
        matching.sort_by_key(|r| r.priority);
        Ok(matching)
    }

    /// Simple condition matching
    fn matches_condition(&self, condition: &serde_json::Value, context: &serde_json::Value) -> bool {
        if condition.is_object() {
            let cond_obj = condition.as_object().unwrap();
            if cond_obj.is_empty() {
                return true; // Empty condition matches everything
            }
            for (key, expected) in cond_obj {
                let actual = context.get(key);
                match actual {
                    None => return false,
                    Some(val) => {
                        // Support both direct value match and array membership
                        if expected.is_array() {
                            if !expected.as_array().unwrap().contains(val) {
                                return false;
                            }
                        } else if val != expected {
                            return false;
                        }
                    }
                }
            }
            true
        } else {
            // Non-object conditions always match
            true
        }
    }

    // ========================================================================
    // Tax Calculation
    // ========================================================================

    /// Calculate taxes for a transaction
    ///
    /// Applies determination rules to find applicable tax rates,
    /// then computes tax amounts for each line.
    pub async fn calculate_tax(
        &self,
        org_id: Uuid,
        request: TaxCalculationRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxCalculationResult> {
        let today = chrono::Utc::now().date_naive();
        let mut line_results: Vec<TaxLineResult> = Vec::new();
        let mut total_taxable = 0.0_f64;
        let mut total_tax = 0.0_f64;
        let mut total_recoverable = 0.0_f64;
        let mut total_non_recoverable = 0.0_f64;

        for line in &request.lines {
            let amount: f64 = line.amount.parse().map_err(|_| AtlasError::ValidationFailed(
                "Line amount must be a valid number".to_string(),
            ))?;

            // Determine which tax rates to apply
            let tax_rate_codes = if let Some(codes) = &line.tax_rate_codes {
                // Explicit codes provided
                codes.clone()
            } else {
                // Use determination rules
                self.determine_tax_rates(org_id, &request.context, &line, today).await?
            };

            if tax_rate_codes.is_empty() {
                // No taxes apply to this line
                continue;
            }

            for rate_code in &tax_rate_codes {
                let tax_rate = self.repository.get_tax_rate_by_code(org_id, rate_code).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(
                        format!("Tax rate '{}' not found", rate_code)
                    ))?;

                // Get the regime for rounding info
                let regime = self.repository.get_regime_by_id(tax_rate.regime_id).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(
                        "Tax regime not found".to_string()
                    ))?;

                let is_inclusive = line.is_inclusive.unwrap_or(regime.default_inclusive);
                let precision = regime.rounding_precision as i32;

                let (taxable_amount, tax_amount) = if is_inclusive {
                    // Tax is included in the amount: extract tax
                    // taxable = amount / (1 + rate/100)
                    let rate_pct: f64 = tax_rate.rate_percentage.parse().unwrap_or(0.0);
                    let taxable = amount / (1.0 + rate_pct / 100.0);
                    let tax = amount - taxable;
                    (taxable, tax)
                } else {
                    // Tax is added on top
                    let rate_pct: f64 = tax_rate.rate_percentage.parse().unwrap_or(0.0);
                    let tax = amount * rate_pct / 100.0;
                    (amount, tax)
                };

                // Apply rounding
                let tax_amount = self.round(tax_amount, &regime.rounding_rule, precision);
                let taxable_amount = self.round(taxable_amount, &regime.rounding_rule, precision);

                // Recovery calculation
                let (recoverable_amt, non_recoverable_amt) = if tax_rate.recoverable {
                    let rec_pct: f64 = tax_rate.recovery_percentage
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(100.0);
                    let rec = self.round(tax_amount * rec_pct / 100.0, &regime.rounding_rule, precision);
                    let non_rec = self.round(tax_amount - rec, &regime.rounding_rule, precision);
                    (Some(rec), Some(non_rec))
                } else {
                    (None, Some(tax_amount))
                };

                total_taxable += taxable_amount;
                total_tax += tax_amount;
                total_recoverable += recoverable_amt.unwrap_or(0.0);
                total_non_recoverable += non_recoverable_amt.unwrap_or(0.0);

                let rate_pct_str = tax_rate.rate_percentage.clone();

                line_results.push(TaxLineResult {
                    line_id: line.line_id,
                    regime_code: Some(regime.code),
                    jurisdiction_code: None, // populated from rate's jurisdiction
                    tax_rate_code: tax_rate.code,
                    tax_rate_name: tax_rate.name,
                    rate_percentage: rate_pct_str.clone(),
                    taxable_amount: format!("{:.2}", taxable_amount),
                    tax_amount: format!("{:.2}", tax_amount),
                    is_inclusive,
                    recoverable: tax_rate.recoverable,
                    recovery_percentage: tax_rate.recovery_percentage.clone(),
                    recoverable_amount: recoverable_amt.map(|v| format!("{:.2}", v)),
                    non_recoverable_amount: non_recoverable_amt.map(|v| format!("{:.2}", v)),
                });

                // Persist tax line if requested
                if request.persist {
                    if let Some(entity_id) = request.entity_id {
                        let inclusive_original = if is_inclusive {
                            Some(format!("{:.2}", amount))
                        } else {
                            None
                        };
                        let _ = self.repository.create_tax_line(
                            org_id,
                            &request.entity_type,
                            entity_id,
                            line.line_id,
                            Some(tax_rate.regime_id),
                            tax_rate.jurisdiction_id,
                            tax_rate.id,
                            &format!("{:.2}", taxable_amount),
                            &rate_pct_str,
                            &format!("{:.2}", tax_amount),
                            is_inclusive,
                            inclusive_original.as_deref(),
                            recoverable_amt.map(|v| format!("{:.2}", v)).as_deref(),
                            non_recoverable_amt.map(|v| format!("{:.2}", v)).as_deref(),
                            tax_rate.tax_account_code.as_deref(),
                            None,
                            created_by,
                        ).await;
                    }
                }
            }
        }

        Ok(TaxCalculationResult {
            lines: line_results,
            total_taxable_amount: format!("{:.2}", total_taxable),
            total_tax_amount: format!("{:.2}", total_tax),
            total_recoverable_amount: format!("{:.2}", total_recoverable),
            total_non_recoverable_amount: format!("{:.2}", total_non_recoverable),
        })
    }

    /// Determine applicable tax rate codes using determination rules
    async fn determine_tax_rates(
        &self,
        org_id: Uuid,
        context: &serde_json::Value,
        line: &atlas_shared::TaxCalculationLine,
        on_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<String>> {
        // Build line-level context by merging transaction context with line data
        let mut line_context = context.clone();
        if let Some(obj) = line_context.as_object_mut() {
            if let Some(pc) = &line.product_category {
                obj.insert("product_category".to_string(), serde_json::Value::String(pc.clone()));
            }
            if let Some(pcode) = &line.product_code {
                obj.insert("product_code".to_string(), serde_json::Value::String(pcode.clone()));
            }
            if let Some(sfc) = &line.ship_from_country {
                obj.insert("ship_from_country".to_string(), serde_json::Value::String(sfc.clone()));
            }
            if let Some(stc) = &line.ship_to_country {
                obj.insert("ship_to_country".to_string(), serde_json::Value::String(stc.clone()));
            }
            if let Some(sts) = &line.ship_to_state {
                obj.insert("ship_to_state".to_string(), serde_json::Value::String(sts.clone()));
            }
        }

        // Get all regimes and try determination rules
        let regimes = self.repository.list_regimes(org_id).await?;
        let mut rate_codes = Vec::new();

        for regime in &regimes {
            if !regime.is_active { continue; }
            if let Some(from) = regime.effective_from {
                if on_date < from { continue; }
            }
            if let Some(to) = regime.effective_to {
                if on_date > to { continue; }
            }

            let rules = self.find_matching_rules(org_id, regime.id, &line_context).await?;
            for rule in &rules {
                if let Some(codes) = rule.action.get("tax_rate_codes") {
                    if let Some(arr) = codes.as_array() {
                        for code in arr {
                            if let Some(s) = code.as_str() {
                                rate_codes.push(s.to_string());
                            }
                        }
                    }
                }
                if rule.stop_on_match {
                    break;
                }
            }
        }

        Ok(rate_codes)
    }

    /// Round a number according to the rounding rule
    fn round(&self, value: f64, rule: &str, precision: i32) -> f64 {
        let factor = 10_f64.powi(precision);
        match rule {
            "up" => (value * factor).ceil() / factor,
            "down" => (value * factor).floor() / factor,
            "nearest" => (value * factor).round() / factor,
            _ => value, // "none"
        }
    }

    // ========================================================================
    // Tax Reporting
    // ========================================================================

    /// Get tax lines for a specific entity
    pub async fn get_tax_lines(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> AtlasResult<Vec<TaxLine>> {
        self.repository.get_tax_lines(entity_type, entity_id).await
    }

    /// Generate a tax report for a period
    pub async fn generate_tax_report(
        &self,
        org_id: Uuid,
        regime_code: &str,
        jurisdiction_code: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxReport> {
        if period_start > period_end {
            return Err(AtlasError::ValidationFailed(
                "period_start must be before period_end".to_string(),
            ));
        }

        let regime = self.get_regime(org_id, regime_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Tax regime '{}' not found", regime_code)
            ))?;

        let jurisdiction_id = if let Some(jc) = jurisdiction_code {
            let juris = self.repository.get_jurisdiction(org_id, regime.id, &jc.to_uppercase()).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Tax jurisdiction '{}' not found", jc)
                ))?;
            Some(juris.id)
        } else {
            None
        };

        self.repository.generate_tax_report(
            org_id, regime.id, jurisdiction_id.as_ref(),
            period_start, period_end, created_by,
        ).await
    }

    /// List tax reports
    pub async fn list_tax_reports(
        &self,
        org_id: Uuid,
        regime_code: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::TaxReport>> {
        let regime_id = match regime_code {
            Some(rc) => {
                let regime = self.get_regime(org_id, rc).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(
                        format!("Tax regime '{}' not found", rc)
                    ))?;
                Some(regime.id)
            }
            None => None,
        };
        self.repository.list_tax_reports(org_id, regime_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tax_types() {
        assert!(VALID_TAX_TYPES.contains(&"sales_tax"));
        assert!(VALID_TAX_TYPES.contains(&"vat"));
        assert!(VALID_TAX_TYPES.contains(&"gst"));
        assert!(VALID_TAX_TYPES.contains(&"withholding"));
        assert!(VALID_TAX_TYPES.contains(&"excise"));
        assert!(VALID_TAX_TYPES.contains(&"customs"));
    }

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"standard"));
        assert!(VALID_RATE_TYPES.contains(&"reduced"));
        assert!(VALID_RATE_TYPES.contains(&"zero"));
        assert!(VALID_RATE_TYPES.contains(&"exempt"));
    }

    #[test]
    fn test_rounding() {
        let engine = TaxEngine::new(Arc::new(crate::mock_repos::MockTaxRepository));

        // Nearest - use values that round cleanly in f64
        assert_eq!(engine.round(1.006, "nearest", 2), 1.01);
        assert_eq!(engine.round(1.004, "nearest", 2), 1.0);

        // Up
        assert_eq!(engine.round(1.001, "up", 2), 1.01);

        // Down
        assert_eq!(engine.round(1.009, "down", 2), 1.0);

        // None
        assert_eq!(engine.round(1.0056789, "none", 2), 1.0056789);
    }

    #[test]
    fn test_condition_matching() {
        let engine = TaxEngine::new(Arc::new(crate::mock_repos::MockTaxRepository));

        // Empty condition matches everything
        let condition = serde_json::json!({});
        let context = serde_json::json!({"product_category": "goods"});
        assert!(engine.matches_condition(&condition, &context));

        // Direct match
        let condition = serde_json::json!({"product_category": "goods"});
        let context = serde_json::json!({"product_category": "goods", "ship_to_country": "US"});
        assert!(engine.matches_condition(&condition, &context));

        // Non-matching
        let context = serde_json::json!({"product_category": "services"});
        assert!(!engine.matches_condition(&condition, &context));

        // Array membership
        let condition = serde_json::json!({"product_category": ["goods", "digital"]});
        let context = serde_json::json!({"product_category": "digital"});
        assert!(engine.matches_condition(&condition, &context));

        // Multiple conditions
        let condition = serde_json::json!({"product_category": "goods", "ship_to_country": "US"});
        let context = serde_json::json!({"product_category": "goods", "ship_to_country": "US"});
        assert!(engine.matches_condition(&condition, &context));

        // Missing key
        let context = serde_json::json!({"product_category": "goods"});
        assert!(!engine.matches_condition(&condition, &context));
    }
}
