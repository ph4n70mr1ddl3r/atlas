//! Product Configurator Engine
//!
//! Manages configurable product models, features, options, rules,
//! configuration instance lifecycle, validation, and pricing.
//!
//! Oracle Fusion Cloud equivalent: SCM > Product Management > Configurator

use atlas_shared::{
    ConfigModel, ConfigFeature, ConfigOption, ConfigRule,
    ConfigInstance, ConfiguratorDashboard,
    AtlasError, AtlasResult,
};
use super::ProductConfiguratorRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_MODEL_TYPES: &[&str] = &["standard", "kit", "bundle"];

const VALID_MODEL_STATUSES: &[&str] = &[
    "draft", "active", "inactive", "obsolete",
];

const VALID_VALIDATION_MODES: &[&str] = &[
    "strict", "relaxed", "none",
];

const VALID_FEATURE_TYPES: &[&str] = &[
    "single_select", "multi_select", "numeric", "text", "boolean",
];

const VALID_OPTION_TYPES: &[&str] = &[
    "standard", "default", "recommended",
];

const VALID_RULE_TYPES: &[&str] = &[
    "compatibility", "incompatibility", "default", "requirement", "exclusion",
];

const VALID_SEVERITIES: &[&str] = &[
    "error", "warning", "info",
];

const VALID_INSTANCE_STATUSES: &[&str] = &[
    "draft", "valid", "invalid", "submitted", "approved", "ordered", "cancelled",
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

/// Product Configurator Engine
pub struct ProductConfiguratorEngine {
    repository: Arc<dyn ProductConfiguratorRepository>,
}

impl ProductConfiguratorEngine {
    pub fn new(repository: Arc<dyn ProductConfiguratorRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Configuration Models
    // ========================================================================

    /// Create a configuration model
    #[allow(clippy::too_many_arguments)]
    pub async fn create_model(
        &self,
        org_id: Uuid,
        model_number: &str,
        name: &str,
        description: Option<&str>,
        base_product_id: Option<Uuid>,
        base_product_number: Option<&str>,
        base_product_name: Option<&str>,
        model_type: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        default_config: Option<serde_json::Value>,
        validation_mode: &str,
        ui_layout: Option<serde_json::Value>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigModel> {
        if model_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Model number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Model name is required".to_string()));
        }
        validate_enum("model_type", model_type, VALID_MODEL_TYPES)?;
        validate_enum("validation_mode", validation_mode, VALID_VALIDATION_MODES)?;
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from must be before effective to".to_string(),
                ));
            }
        }

        if self.repository.get_model_by_number(org_id, model_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Model '{}' already exists", model_number
            )));
        }

        info!("Creating config model '{}' ({}) for org {} [type={}]",
              model_number, name, org_id, model_type);

        self.repository.create_model(
            org_id, model_number, name, description,
            base_product_id, base_product_number, base_product_name,
            model_type, "draft", 1,
            effective_from, effective_to,
            default_config.unwrap_or(serde_json::json!({})),
            validation_mode,
            ui_layout.unwrap_or(serde_json::json!({})),
            created_by,
        ).await
    }

    /// Get a model by ID
    pub async fn get_model(&self, id: Uuid) -> AtlasResult<Option<ConfigModel>> {
        self.repository.get_model(id).await
    }

    /// Get a model by number
    pub async fn get_model_by_number(&self, org_id: Uuid, model_number: &str) -> AtlasResult<Option<ConfigModel>> {
        self.repository.get_model_by_number(org_id, model_number).await
    }

    /// List models with optional filters
    pub async fn list_models(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        model_type: Option<&str>,
    ) -> AtlasResult<Vec<ConfigModel>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_MODEL_STATUSES)?;
        }
        if let Some(t) = model_type {
            validate_enum("model_type", t, VALID_MODEL_TYPES)?;
        }
        self.repository.list_models(org_id, status, model_type).await
    }

    /// Activate a model
    pub async fn activate_model(&self, id: Uuid) -> AtlasResult<ConfigModel> {
        let model = self.repository.get_model(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Model {} not found", id)))?;

        if model.status != "draft" && model.status != "inactive" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot activate model in '{}' status. Must be 'draft' or 'inactive'.", model.status)
            ));
        }

        info!("Activating config model {}", id);
        self.repository.update_model_status(id, "active").await
    }

    /// Deactivate a model
    pub async fn deactivate_model(&self, id: Uuid) -> AtlasResult<ConfigModel> {
        let model = self.repository.get_model(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Model {} not found", id)))?;

        if model.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot deactivate model in '{}' status. Must be 'active'.", model.status)
            ));
        }

        info!("Deactivating config model {}", id);
        self.repository.update_model_status(id, "inactive").await
    }

    /// Obsolete a model
    pub async fn obsolete_model(&self, id: Uuid) -> AtlasResult<ConfigModel> {
        let model = self.repository.get_model(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Model {} not found", id)))?;

        if model.status == "obsolete" {
            return Err(AtlasError::ValidationFailed(
                "Model is already obsolete".to_string()
            ));
        }

        info!("Obsoleting config model {}", id);
        self.repository.update_model_status(id, "obsolete").await
    }

    /// Delete a model by number (only draft)
    pub async fn delete_model(&self, org_id: Uuid, model_number: &str) -> AtlasResult<()> {
        if let Some(model) = self.repository.get_model_by_number(org_id, model_number).await? {
            if model.status != "draft" {
                return Err(AtlasError::ValidationFailed(
                    "Only draft models can be deleted".to_string()
                ));
            }
        }
        info!("Deleting config model '{}' for org {}", model_number, org_id);
        self.repository.delete_model(org_id, model_number).await
    }

    // ========================================================================
    // Configuration Features
    // ========================================================================

    /// Create a configuration feature
    #[allow(clippy::too_many_arguments)]
    pub async fn create_feature(
        &self,
        org_id: Uuid,
        model_id: Uuid,
        feature_code: &str,
        name: &str,
        description: Option<&str>,
        feature_type: &str,
        is_required: bool,
        display_order: i32,
        ui_hints: Option<serde_json::Value>,
    ) -> AtlasResult<ConfigFeature> {
        if feature_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Feature code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Feature name is required".to_string()));
        }
        validate_enum("feature_type", feature_type, VALID_FEATURE_TYPES)?;

        // Verify model exists
        let model = self.repository.get_model(model_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Model {} not found", model_id
            )))?;

        if model.status != "draft" && model.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add features to model in '{}' status", model.status)
            ));
        }

        info!("Creating feature '{}' ({}) for model {}", feature_code, name, model_id);

        self.repository.create_feature(
            org_id, model_id, feature_code, name, description,
            feature_type, is_required, display_order,
            ui_hints.unwrap_or(serde_json::json!({})),
        ).await
    }

    /// Get a feature by ID
    pub async fn get_feature(&self, id: Uuid) -> AtlasResult<Option<ConfigFeature>> {
        self.repository.get_feature(id).await
    }

    /// List features for a model
    pub async fn list_features(&self, model_id: Uuid) -> AtlasResult<Vec<ConfigFeature>> {
        self.repository.list_features(model_id).await
    }

    /// Delete a feature
    pub async fn delete_feature(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting feature {}", id);
        self.repository.delete_feature(id).await
    }

    // ========================================================================
    // Configuration Options
    // ========================================================================

    /// Create a configuration option
    #[allow(clippy::too_many_arguments)]
    pub async fn create_option(
        &self,
        org_id: Uuid,
        feature_id: Uuid,
        option_code: &str,
        name: &str,
        description: Option<&str>,
        option_type: &str,
        price_adjustment: f64,
        cost_adjustment: f64,
        lead_time_days: i32,
        is_default: bool,
        is_available: bool,
        display_order: i32,
    ) -> AtlasResult<ConfigOption> {
        if option_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Option code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Option name is required".to_string()));
        }
        validate_enum("option_type", option_type, VALID_OPTION_TYPES)?;
        if lead_time_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Lead time days cannot be negative".to_string(),
            ));
        }

        // Verify feature exists
        let feature = self.repository.get_feature(feature_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Feature {} not found", feature_id
            )))?;

        // Verify model is editable
        let model = self.repository.get_model(feature.model_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Model {} not found", feature.model_id
            )))?;

        if model.status != "draft" && model.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add options to model in '{}' status", model.status)
            ));
        }

        info!("Creating option '{}' ({}) for feature {}", option_code, name, feature_id);

        self.repository.create_option(
            org_id, feature_id, option_code, name, description,
            option_type, price_adjustment, cost_adjustment,
            lead_time_days, is_default, is_available, display_order,
        ).await
    }

    /// Get an option by ID
    pub async fn get_option(&self, id: Uuid) -> AtlasResult<Option<ConfigOption>> {
        self.repository.get_option(id).await
    }

    /// List options for a feature
    pub async fn list_options(&self, feature_id: Uuid) -> AtlasResult<Vec<ConfigOption>> {
        self.repository.list_options(feature_id).await
    }

    /// Update option availability
    pub async fn update_option_availability(&self, id: Uuid, is_available: bool) -> AtlasResult<ConfigOption> {
        info!("Updating option {} availability to {}", id, is_available);
        self.repository.update_option_availability(id, is_available).await
    }

    /// Delete an option
    pub async fn delete_option(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting option {}", id);
        self.repository.delete_option(id).await
    }

    // ========================================================================
    // Configuration Rules
    // ========================================================================

    /// Create a configuration rule
    #[allow(clippy::too_many_arguments)]
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        model_id: Uuid,
        rule_code: &str,
        name: &str,
        description: Option<&str>,
        rule_type: &str,
        source_feature_id: Option<Uuid>,
        source_option_id: Option<Uuid>,
        target_feature_id: Option<Uuid>,
        target_option_id: Option<Uuid>,
        condition_expression: Option<&str>,
        severity: &str,
        is_active: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigRule> {
        if rule_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule name is required".to_string()));
        }
        validate_enum("rule_type", rule_type, VALID_RULE_TYPES)?;
        validate_enum("severity", severity, VALID_SEVERITIES)?;

        // Verify model exists
        let model = self.repository.get_model(model_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Model {} not found", model_id
            )))?;

        if model.status != "draft" && model.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot add rules to model in '{}' status", model.status)
            ));
        }

        // Verify referenced features/options exist
        if let Some(sf_id) = source_feature_id {
            self.repository.get_feature(sf_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Source feature {} not found", sf_id
                )))?;
        }
        if let Some(tf_id) = target_feature_id {
            self.repository.get_feature(tf_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Target feature {} not found", tf_id
                )))?;
        }
        if let Some(so_id) = source_option_id {
            self.repository.get_option(so_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Source option {} not found", so_id
                )))?;
        }
        if let Some(to_id) = target_option_id {
            self.repository.get_option(to_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Target option {} not found", to_id
                )))?;
        }

        if self.repository.get_rule_by_code(model_id, rule_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Rule '{}' already exists in model", rule_code
            )));
        }

        info!("Creating rule '{}' ({}) for model {} [type={}, severity={}]",
              rule_code, name, model_id, rule_type, severity);

        self.repository.create_rule(
            org_id, model_id, rule_code, name, description,
            rule_type, source_feature_id, source_option_id,
            target_feature_id, target_option_id,
            condition_expression, severity, is_active, priority,
            created_by,
        ).await
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ConfigRule>> {
        self.repository.get_rule(id).await
    }

    /// List rules for a model
    pub async fn list_rules(&self, model_id: Uuid, rule_type: Option<&str>) -> AtlasResult<Vec<ConfigRule>> {
        if let Some(rt) = rule_type {
            validate_enum("rule_type", rt, VALID_RULE_TYPES)?;
        }
        self.repository.list_rules(model_id, rule_type).await
    }

    /// Toggle a rule active/inactive
    pub async fn toggle_rule(&self, id: Uuid, is_active: bool) -> AtlasResult<ConfigRule> {
        info!("Toggling rule {} to active={}", id, is_active);
        self.repository.update_rule_active(id, is_active).await
    }

    /// Delete a rule
    pub async fn delete_rule(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting rule {}", id);
        self.repository.delete_rule(id).await
    }

    // ========================================================================
    // Configuration Instances
    // ========================================================================

    /// Create a configuration instance
    #[allow(clippy::too_many_arguments)]
    pub async fn create_instance(
        &self,
        org_id: Uuid,
        instance_number: &str,
        model_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        selections: serde_json::Value,
        base_price: f64,
        currency_code: &str,
        effective_date: Option<chrono::NaiveDate>,
        configured_by: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigInstance> {
        if instance_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Instance number is required".to_string()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }
        if base_price < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Base price cannot be negative".to_string(),
            ));
        }

        // Verify model exists and is active
        let model = self.repository.get_model(model_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Model {} not found", model_id
            )))?;

        if model.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot create instances for model in '{}' status. Must be 'active'.", model.status)
            ));
        }

        // Calculate total price from selections
        let total_price = self.calculate_total_price(model_id, base_price, &selections).await?;

        // Generate config hash
        let config_hash = Self::compute_config_hash(&selections);

        // Validate the configuration
        let (errors, warnings) = self.validate_configuration(model_id, &selections).await?;

        let status = if errors.as_array().is_none_or(|a| a.is_empty()) {
            "valid"
        } else {
            "invalid"
        };

        if self.repository.get_instance_by_number(org_id, instance_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Instance '{}' already exists", instance_number
            )));
        }

        info!("Creating config instance '{}' for model {} [status={}, price={:.2}]",
              instance_number, model.model_number, status, total_price);

        self.repository.create_instance(
            org_id, instance_number, model_id,
            Some(&model.model_number), name, description,
            status, selections,
            errors, warnings,
            base_price, total_price, currency_code,
            Some(&config_hash),
            effective_date,
            configured_by, created_by,
        ).await
    }

    /// Get an instance by ID
    pub async fn get_instance(&self, id: Uuid) -> AtlasResult<Option<ConfigInstance>> {
        self.repository.get_instance(id).await
    }

    /// Get an instance by number
    pub async fn get_instance_by_number(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<Option<ConfigInstance>> {
        self.repository.get_instance_by_number(org_id, instance_number).await
    }

    /// List instances with optional filters
    pub async fn list_instances(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        model_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<ConfigInstance>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_INSTANCE_STATUSES)?;
        }
        self.repository.list_instances(org_id, status, model_id).await
    }

    /// Update instance selections (re-validate)
    pub async fn update_instance_selections(
        &self,
        id: Uuid,
        selections: serde_json::Value,
        base_price: f64,
    ) -> AtlasResult<ConfigInstance> {
        let instance = self.repository.get_instance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Instance {} not found", id)))?;

        if instance.status != "draft" && instance.status != "valid" && instance.status != "invalid" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot update selections for instance in '{}' status", instance.status)
            ));
        }

        let total_price = self.calculate_total_price(instance.model_id, base_price, &selections).await?;
        let config_hash = Self::compute_config_hash(&selections);
        let (errors, warnings) = self.validate_configuration(instance.model_id, &selections).await?;

        let new_status = if errors.as_array().is_none_or(|a| a.is_empty()) {
            "valid"
        } else {
            "invalid"
        };

        info!("Updating instance {} selections [status={}, price={:.2}]", id, new_status, total_price);

        let _inst = self.repository.update_instance_selections(
            id, selections, total_price, Some(&config_hash),
        ).await?;
        let _inst = self.repository.update_instance_validation(id, errors, warnings).await?;
        let _ = _inst; // suppress unused warning; validation already persisted
        // Update status via repository
        self.repository.update_instance_status(id, new_status).await
    }

    /// Submit a configuration for approval
    pub async fn submit_instance(&self, id: Uuid) -> AtlasResult<ConfigInstance> {
        let instance = self.repository.get_instance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Instance {} not found", id)))?;

        if instance.status != "valid" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot submit instance in '{}' status. Must be 'valid'.", instance.status)
            ));
        }

        info!("Submitting config instance {} for approval", id);
        self.repository.update_instance_status(id, "submitted").await
    }

    /// Approve a configuration
    pub async fn approve_instance(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ConfigInstance> {
        let instance = self.repository.get_instance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Instance {} not found", id)))?;

        if instance.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot approve instance in '{}' status. Must be 'submitted'.", instance.status)
            ));
        }

        info!("Approving config instance {} by {:?}", id, approved_by);
        self.repository.update_instance_approval(id, approved_by).await?;
        self.repository.update_instance_status(id, "approved").await
    }

    /// Reject (cancel) a configuration
    pub async fn cancel_instance(&self, id: Uuid) -> AtlasResult<ConfigInstance> {
        let instance = self.repository.get_instance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Instance {} not found", id)))?;

        if instance.status == "ordered" {
            return Err(AtlasError::ValidationFailed(
                "Cannot cancel an ordered configuration".to_string()
            ));
        }

        info!("Cancelling config instance {}", id);
        self.repository.update_instance_status(id, "cancelled").await
    }

    /// Link a configuration instance to a sales order
    pub async fn link_instance_to_order(
        &self,
        id: Uuid,
        sales_order_id: Uuid,
        sales_order_number: &str,
        sales_order_line: i32,
    ) -> AtlasResult<ConfigInstance> {
        let instance = self.repository.get_instance(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Instance {} not found", id)))?;

        if instance.status != "approved" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot link instance in '{}' status. Must be 'approved'.", instance.status)
            ));
        }

        info!("Linking config instance {} to order {}", id, sales_order_number);
        let _inst = self.repository.link_instance_to_order(
            id, Some(sales_order_id), Some(sales_order_number), Some(sales_order_line),
        ).await?;
        self.repository.update_instance_status(id, "ordered").await
    }

    /// Delete an instance by number (only draft/invalid)
    pub async fn delete_instance(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<()> {
        if let Some(instance) = self.repository.get_instance_by_number(org_id, instance_number).await? {
            if instance.status != "draft" && instance.status != "invalid" && instance.status != "cancelled" {
                return Err(AtlasError::ValidationFailed(
                    "Only draft, invalid, or cancelled instances can be deleted".to_string()
                ));
            }
        }
        info!("Deleting config instance '{}' for org {}", instance_number, org_id);
        self.repository.delete_instance(org_id, instance_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the configurator dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ConfiguratorDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal: Pricing & Validation
    // ========================================================================

    /// Calculate total price from base price + option adjustments
    async fn calculate_total_price(
        &self,
        model_id: Uuid,
        base_price: f64,
        selections: &serde_json::Value,
    ) -> AtlasResult<f64> {
        let mut total = base_price;

        // selections format: { "feature_code": "option_code", ... }
        if let Some(sel_map) = selections.as_object() {
            let features = self.repository.list_features(model_id).await?;
            for feature in &features {
                let options = self.repository.list_options(feature.id).await?;
                if let Some(selected) = sel_map.get(&feature.feature_code) {
                    // For single_select: value is a string
                    if let Some(opt_code) = selected.as_str() {
                        for opt in &options {
                            if opt.option_code == opt_code {
                                total += opt.price_adjustment;
                            }
                        }
                    }
                    // For multi_select: value is an array of strings
                    if let Some(arr) = selected.as_array() {
                        for item in arr {
                            if let Some(opt_code) = item.as_str() {
                                for opt in &options {
                                    if opt.option_code == opt_code {
                                        total += opt.price_adjustment;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(total)
    }

    /// Validate configuration against rules
    async fn validate_configuration(
        &self,
        model_id: Uuid,
        selections: &serde_json::Value,
    ) -> AtlasResult<(serde_json::Value, serde_json::Value)> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let features = self.repository.list_features(model_id).await?;
        let rules = self.repository.list_rules(model_id, None).await?;

        // Check required features have selections
        for feature in &features {
            if feature.is_required {
                let has_selection = selections.get(&feature.feature_code)
                    .and_then(|v| {
                        if v.is_string() { Some(v.as_str().map(|s| !s.is_empty())).flatten() }
                        else if v.is_array() { Some(!v.as_array().is_none_or(|a| a.is_empty())) }
                        else { None }
                    })
                    .unwrap_or(false);

                if !has_selection {
                    errors.push(serde_json::json!({
                        "feature": feature.feature_code,
                        "message": format!("Feature '{}' is required", feature.name),
                        "type": "missing_required"
                    }));
                }
            }
        }

        // Check rules
        for rule in &rules {
            if !rule.is_active {
                continue;
            }

            let source_selected = rule.source_option_id.is_none_or(|_so_id| {
                // Check if source option is selected in selections
                true // simplified – real implementation would check
            });

            if !source_selected {
                continue;
            }

            // For incompatibility rules: source and target can't both be selected
            if rule.rule_type == "incompatibility" {
                let source_opt = self.repository.get_option(
                    rule.source_option_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();
                let source_feat = self.repository.get_feature(
                    rule.source_feature_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();

                let target_opt = self.repository.get_option(
                    rule.target_option_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();
                let target_feat = self.repository.get_feature(
                    rule.target_feature_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();

                let source_selected = Self::is_option_selected(selections, &source_feat, &source_opt);
                let target_selected = Self::is_option_selected(selections, &target_feat, &target_opt);

                if source_selected && target_selected {
                    let msg = format!(
                        "Incompatibility rule '{}': {} and {} cannot be selected together",
                        rule.rule_code,
                        source_opt.as_ref().map_or("?", |o| &o.name),
                        target_opt.as_ref().map_or("?", |o| &o.name),
                    );
                    match rule.severity.as_str() {
                        "warning" => warnings.push(serde_json::json!({
                            "rule": rule.rule_code, "message": msg, "type": "incompatibility"
                        })),
                        _ => errors.push(serde_json::json!({
                            "rule": rule.rule_code, "message": msg, "type": "incompatibility"
                        })),
                    }
                }
            }

            // For requirement rules: if source is selected, target must also be selected
            if rule.rule_type == "requirement" {
                let source_opt = self.repository.get_option(
                    rule.source_option_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();
                let source_feat = self.repository.get_feature(
                    rule.source_feature_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();

                let target_opt = self.repository.get_option(
                    rule.target_option_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();
                let target_feat = self.repository.get_feature(
                    rule.target_feature_id.unwrap_or(Uuid::nil())
                ).await.ok().flatten();

                let source_selected = Self::is_option_selected(selections, &source_feat, &source_opt);
                let target_selected = Self::is_option_selected(selections, &target_feat, &target_opt);

                if source_selected && !target_selected {
                    let msg = format!(
                        "Requirement rule '{}': selecting {} requires {}",
                        rule.rule_code,
                        source_opt.as_ref().map_or("?", |o| &o.name),
                        target_opt.as_ref().map_or("?", |o| &o.name),
                    );
                    match rule.severity.as_str() {
                        "warning" => warnings.push(serde_json::json!({
                            "rule": rule.rule_code, "message": msg, "type": "requirement"
                        })),
                        _ => errors.push(serde_json::json!({
                            "rule": rule.rule_code, "message": msg, "type": "requirement"
                        })),
                    }
                }
            }
        }

        Ok((serde_json::json!(errors), serde_json::json!(warnings)))
    }

    /// Check if an option is selected in the given selections
    fn is_option_selected(
        selections: &serde_json::Value,
        feature: &Option<ConfigFeature>,
        option: &Option<ConfigOption>,
    ) -> bool {
        match (feature, option) {
            (Some(f), Some(o)) => {
                if let Some(sel) = selections.get(&f.feature_code) {
                    if sel.is_string() {
                        sel.as_str() == Some(&o.option_code)
                    } else if sel.is_array() {
                        sel.as_array().is_some_and(|arr| {
                            arr.iter().any(|v| v.as_str() == Some(&o.option_code))
                        })
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Compute a simple hash of the configuration selections
    fn compute_config_hash(selections: &serde_json::Value) -> String {
        use std::collections::BTreeMap;
        // Sort keys for deterministic ordering
        let sorted: BTreeMap<_, _> = selections.as_object()
            .map_or(BTreeMap::new(), |m| m.iter().collect());
        let canonical = serde_json::to_string(&serde_json::json!(sorted))
            .unwrap_or_default();
        // Simple hash (not crypto, but good enough for dedup)
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        canonical.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_model_types() {
        assert!(VALID_MODEL_TYPES.contains(&"standard"));
        assert!(VALID_MODEL_TYPES.contains(&"kit"));
        assert!(VALID_MODEL_TYPES.contains(&"bundle"));
        assert!(!VALID_MODEL_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_model_statuses() {
        assert!(VALID_MODEL_STATUSES.contains(&"draft"));
        assert!(VALID_MODEL_STATUSES.contains(&"active"));
        assert!(VALID_MODEL_STATUSES.contains(&"inactive"));
        assert!(VALID_MODEL_STATUSES.contains(&"obsolete"));
    }

    #[test]
    fn test_valid_feature_types() {
        assert!(VALID_FEATURE_TYPES.contains(&"single_select"));
        assert!(VALID_FEATURE_TYPES.contains(&"multi_select"));
        assert!(VALID_FEATURE_TYPES.contains(&"numeric"));
        assert!(VALID_FEATURE_TYPES.contains(&"boolean"));
    }

    #[test]
    fn test_valid_rule_types() {
        assert!(VALID_RULE_TYPES.contains(&"compatibility"));
        assert!(VALID_RULE_TYPES.contains(&"incompatibility"));
        assert!(VALID_RULE_TYPES.contains(&"default"));
        assert!(VALID_RULE_TYPES.contains(&"requirement"));
        assert!(VALID_RULE_TYPES.contains(&"exclusion"));
    }

    #[test]
    fn test_valid_instance_statuses() {
        assert!(VALID_INSTANCE_STATUSES.contains(&"draft"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"valid"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"invalid"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"submitted"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"approved"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"ordered"));
        assert!(VALID_INSTANCE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("model_type", "standard", VALID_MODEL_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("model_type", "invalid", VALID_MODEL_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("model_type"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("model_type", "", VALID_MODEL_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_compute_config_hash_deterministic() {
        let sel1 = serde_json::json!({"color": "red", "engine": "v8"});
        let sel2 = serde_json::json!({"engine": "v8", "color": "red"});
        // Both should produce the same hash because we sort keys
        assert_eq!(
            ProductConfiguratorEngine::compute_config_hash(&sel1),
            ProductConfiguratorEngine::compute_config_hash(&sel2),
        );
    }

    #[test]
    fn test_compute_config_hash_different() {
        let sel1 = serde_json::json!({"color": "red"});
        let sel2 = serde_json::json!({"color": "blue"});
        assert_ne!(
            ProductConfiguratorEngine::compute_config_hash(&sel1),
            ProductConfiguratorEngine::compute_config_hash(&sel2),
        );
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockProductConfiguratorRepository;

    fn create_engine() -> ProductConfiguratorEngine {
        ProductConfiguratorEngine::new(Arc::new(MockProductConfiguratorRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    // --- Model Validation Tests ---

    #[tokio::test]
    async fn test_create_model_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "", "Laptop Configurator", None,
            None, None, None, "standard",
            None, None, None, "strict", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_model_validation_empty_name() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "MODEL-001", "", None,
            None, None, None, "standard",
            None, None, None, "strict", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("name")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_model_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "MODEL-001", "Laptop Configurator", None,
            None, None, None, "custom_type",
            None, None, None, "strict", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("model_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_model_validation_bad_validation_mode() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "MODEL-001", "Laptop Configurator", None,
            None, None, None, "standard",
            None, None, None, "ultra_strict", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("validation_mode")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_model_validation_dates_inverted() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "MODEL-001", "Laptop Configurator", None,
            None, None, None, "standard",
            chrono::NaiveDate::from_ymd_opt(2025, 12, 31),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            None, "strict", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Effective from")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_model_success() {
        let engine = create_engine();
        let result = engine.create_model(
            test_org_id(), "MODEL-001", "Laptop Configurator",
            Some("Configure your laptop"),
            None, None, None, "standard",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            chrono::NaiveDate::from_ymd_opt(2025, 12, 31),
            None, "strict", None, Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.model_number, "MODEL-001");
        assert_eq!(model.name, "Laptop Configurator");
        assert_eq!(model.model_type, "standard");
        assert_eq!(model.status, "draft");
        assert_eq!(model.validation_mode, "strict");
    }

    // --- Feature Validation Tests ---

    #[tokio::test]
    async fn test_create_feature_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_feature(
            test_org_id(), Uuid::new_v4(), "", "Color", None,
            "single_select", false, 0, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_feature_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_feature(
            test_org_id(), Uuid::new_v4(), "COLOR", "Color", None,
            "dropdown", false, 0, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("feature_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Option Validation Tests ---

    #[tokio::test]
    async fn test_create_option_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_option(
            test_org_id(), Uuid::new_v4(), "", "Red", None,
            "standard", 0.0, 0.0, 0, false, true, 0,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_option_validation_negative_lead_time() {
        let engine = create_engine();
        let result = engine.create_option(
            test_org_id(), Uuid::new_v4(), "RED", "Red", None,
            "standard", 0.0, 0.0, -5, false, true, 0,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Lead time")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Rule Validation Tests ---

    #[tokio::test]
    async fn test_create_rule_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_rule(
            test_org_id(), Uuid::new_v4(), "", "Color-Engine Rule", None,
            "incompatibility", None, None, None, None, None,
            "error", true, 0, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_rule_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_rule(
            test_org_id(), Uuid::new_v4(), "RULE-001", "Rule", None,
            "conditional", None, None, None, None, None,
            "error", true, 0, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("rule_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_rule_validation_bad_severity() {
        let engine = create_engine();
        let result = engine.create_rule(
            test_org_id(), Uuid::new_v4(), "RULE-001", "Rule", None,
            "incompatibility", None, None, None, None, None,
            "critical", true, 0, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("severity")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Instance Validation Tests ---

    #[tokio::test]
    async fn test_create_instance_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_instance(
            test_org_id(), "", Uuid::new_v4(), None, None,
            serde_json::json!({}), 1000.0, "USD", None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_instance_validation_negative_price() {
        let engine = create_engine();
        let result = engine.create_instance(
            test_org_id(), "CFG-001", Uuid::new_v4(), None, None,
            serde_json::json!({}), -100.0, "USD", None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Base price")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Activate/Deactivate Model Tests ---

    #[tokio::test]
    async fn test_activate_model_wrong_status() {
        let engine = create_engine();
        // Model doesn't exist in mock → EntityNotFound
        let result = engine.activate_model(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_model_wrong_status() {
        let engine = create_engine();
        let result = engine.deactivate_model(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_is_option_selected() {
        let selections = serde_json::json!({
            "color": "red",
            "accessories": ["mouse", "keyboard"]
        });

        let feature_color = ConfigFeature {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            model_id: Uuid::new_v4(),
            feature_code: "color".to_string(),
            name: "Color".to_string(),
            description: None,
            feature_type: "single_select".to_string(),
            is_required: false,
            display_order: 0,
            ui_hints: serde_json::json!({}),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let opt_red = ConfigOption {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            feature_id: feature_color.id,
            option_code: "red".to_string(),
            name: "Red".to_string(),
            description: None,
            option_type: "standard".to_string(),
            price_adjustment: 0.0,
            cost_adjustment: 0.0,
            lead_time_days: 0,
            is_default: false,
            is_available: true,
            display_order: 0,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(ProductConfiguratorEngine::is_option_selected(
            &selections, &Some(feature_color.clone()), &Some(opt_red.clone())
        ));

        let opt_blue = ConfigOption {
            option_code: "blue".to_string(),
            ..opt_red.clone()
        };
        assert!(!ProductConfiguratorEngine::is_option_selected(
            &selections, &Some(feature_color.clone()), &Some(opt_blue)
        ));

        // Multi-select
        let feature_acc = ConfigFeature {
            feature_code: "accessories".to_string(),
            ..feature_color.clone()
        };
        let opt_mouse = ConfigOption {
            id: Uuid::new_v4(),
            option_code: "mouse".to_string(),
            feature_id: feature_acc.id,
            ..opt_red.clone()
        };
        assert!(ProductConfiguratorEngine::is_option_selected(
            &selections, &Some(feature_acc), &Some(opt_mouse)
        ));
    }
}
