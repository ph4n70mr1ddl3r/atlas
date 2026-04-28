//! Product Configurator Repository
//!
//! PostgreSQL storage for configuration models, features, options, rules,
//! instances, and dashboard analytics.

use atlas_shared::{
    ConfigModel, ConfigFeature, ConfigOption, ConfigRule,
    ConfigInstance, ConfiguratorDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for product configurator data storage
#[async_trait]
pub trait ProductConfiguratorRepository: Send + Sync {
    // ========================================================================
    // Configuration Models
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_model(
        &self,
        org_id: Uuid, model_number: &str, name: &str, description: Option<&str>,
        base_product_id: Option<Uuid>, base_product_number: Option<&str>, base_product_name: Option<&str>,
        model_type: &str, status: &str, version: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        default_config: serde_json::Value, validation_mode: &str,
        ui_layout: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigModel>;

    async fn get_model(&self, id: Uuid) -> AtlasResult<Option<ConfigModel>>;
    async fn get_model_by_number(&self, org_id: Uuid, model_number: &str) -> AtlasResult<Option<ConfigModel>>;
    async fn list_models(
        &self, org_id: Uuid, status: Option<&str>, model_type: Option<&str>,
    ) -> AtlasResult<Vec<ConfigModel>>;
    async fn update_model_status(&self, id: Uuid, status: &str) -> AtlasResult<ConfigModel>;
    async fn delete_model(&self, org_id: Uuid, model_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Configuration Features
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_feature(
        &self,
        org_id: Uuid, model_id: Uuid, feature_code: &str, name: &str,
        description: Option<&str>, feature_type: &str, is_required: bool,
        display_order: i32, ui_hints: serde_json::Value,
    ) -> AtlasResult<ConfigFeature>;

    async fn get_feature(&self, id: Uuid) -> AtlasResult<Option<ConfigFeature>>;
    async fn list_features(&self, model_id: Uuid) -> AtlasResult<Vec<ConfigFeature>>;
    async fn delete_feature(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Configuration Options
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_option(
        &self,
        org_id: Uuid, feature_id: Uuid, option_code: &str, name: &str,
        description: Option<&str>, option_type: &str,
        price_adjustment: f64, cost_adjustment: f64, lead_time_days: i32,
        is_default: bool, is_available: bool, display_order: i32,
    ) -> AtlasResult<ConfigOption>;

    async fn get_option(&self, id: Uuid) -> AtlasResult<Option<ConfigOption>>;
    async fn list_options(&self, feature_id: Uuid) -> AtlasResult<Vec<ConfigOption>>;
    async fn update_option_availability(&self, id: Uuid, is_available: bool) -> AtlasResult<ConfigOption>;
    async fn delete_option(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Configuration Rules
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_rule(
        &self,
        org_id: Uuid, model_id: Uuid, rule_code: &str, name: &str,
        description: Option<&str>, rule_type: &str,
        source_feature_id: Option<Uuid>, source_option_id: Option<Uuid>,
        target_feature_id: Option<Uuid>, target_option_id: Option<Uuid>,
        condition_expression: Option<&str>, severity: &str,
        is_active: bool, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigRule>;

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ConfigRule>>;
    async fn get_rule_by_code(&self, model_id: Uuid, rule_code: &str) -> AtlasResult<Option<ConfigRule>>;
    async fn list_rules(&self, model_id: Uuid, rule_type: Option<&str>) -> AtlasResult<Vec<ConfigRule>>;
    async fn update_rule_active(&self, id: Uuid, is_active: bool) -> AtlasResult<ConfigRule>;
    async fn delete_rule(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Configuration Instances
    // ========================================================================
    #[allow(clippy::too_many_arguments)]
    async fn create_instance(
        &self,
        org_id: Uuid, instance_number: &str, model_id: Uuid,
        model_number: Option<&str>, name: Option<&str>, description: Option<&str>,
        status: &str, selections: serde_json::Value,
        validation_errors: serde_json::Value, validation_warnings: serde_json::Value,
        base_price: f64, total_price: f64, currency_code: &str,
        config_hash: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        configured_by: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigInstance>;

    async fn get_instance(&self, id: Uuid) -> AtlasResult<Option<ConfigInstance>>;
    async fn get_instance_by_number(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<Option<ConfigInstance>>;
    async fn list_instances(
        &self, org_id: Uuid, status: Option<&str>, model_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<ConfigInstance>>;
    async fn update_instance_status(&self, id: Uuid, status: &str) -> AtlasResult<ConfigInstance>;
    async fn update_instance_validation(
        &self, id: Uuid, validation_errors: serde_json::Value, validation_warnings: serde_json::Value,
    ) -> AtlasResult<ConfigInstance>;
    async fn update_instance_selections(
        &self, id: Uuid, selections: serde_json::Value, total_price: f64, config_hash: Option<&str>,
    ) -> AtlasResult<ConfigInstance>;
    async fn update_instance_approval(
        &self, id: Uuid, approved_by: Option<Uuid>,
    ) -> AtlasResult<ConfigInstance>;
    async fn link_instance_to_order(
        &self, id: Uuid, sales_order_id: Option<Uuid>,
        sales_order_number: Option<&str>, sales_order_line: Option<i32>,
    ) -> AtlasResult<ConfigInstance>;
    async fn delete_instance(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<()>;

    // ========================================================================
    // Dashboard
    // ========================================================================
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ConfiguratorDashboard>;
}

// ============================================================================
// Row mappers
// ============================================================================

fn row_to_model(row: &sqlx::postgres::PgRow) -> ConfigModel {
    ConfigModel {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        model_number: row.try_get("model_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        base_product_id: row.try_get("base_product_id").unwrap_or_default(),
        base_product_number: row.try_get("base_product_number").unwrap_or_default(),
        base_product_name: row.try_get("base_product_name").unwrap_or_default(),
        model_type: row.try_get("model_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        version: row.try_get("version").unwrap_or(1),
        effective_from: row.try_get("effective_from").unwrap_or_default(),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        default_config: row.try_get("default_config").unwrap_or(serde_json::json!({})),
        validation_mode: row.try_get("validation_mode").unwrap_or_else(|_| "strict".to_string()),
        ui_layout: row.try_get("ui_layout").unwrap_or(serde_json::json!({})),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_feature(row: &sqlx::postgres::PgRow) -> ConfigFeature {
    ConfigFeature {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        model_id: row.try_get("model_id").unwrap_or_default(),
        feature_code: row.try_get("feature_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        feature_type: row.try_get("feature_type").unwrap_or_else(|_| "single_select".to_string()),
        is_required: row.try_get("is_required").unwrap_or(false),
        display_order: row.try_get("display_order").unwrap_or(0),
        ui_hints: row.try_get("ui_hints").unwrap_or(serde_json::json!({})),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_option(row: &sqlx::postgres::PgRow) -> ConfigOption {
    ConfigOption {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        feature_id: row.try_get("feature_id").unwrap_or_default(),
        option_code: row.try_get("option_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        option_type: row.try_get("option_type").unwrap_or_else(|_| "standard".to_string()),
        price_adjustment: row.try_get("price_adjustment").unwrap_or(0.0),
        cost_adjustment: row.try_get("cost_adjustment").unwrap_or(0.0),
        lead_time_days: row.try_get("lead_time_days").unwrap_or(0),
        is_default: row.try_get("is_default").unwrap_or(false),
        is_available: row.try_get("is_available").unwrap_or(true),
        display_order: row.try_get("display_order").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_rule(row: &sqlx::postgres::PgRow) -> ConfigRule {
    ConfigRule {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        model_id: row.try_get("model_id").unwrap_or_default(),
        rule_code: row.try_get("rule_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        rule_type: row.try_get("rule_type").unwrap_or_default(),
        source_feature_id: row.try_get("source_feature_id").unwrap_or_default(),
        source_option_id: row.try_get("source_option_id").unwrap_or_default(),
        target_feature_id: row.try_get("target_feature_id").unwrap_or_default(),
        target_option_id: row.try_get("target_option_id").unwrap_or_default(),
        condition_expression: row.try_get("condition_expression").unwrap_or_default(),
        severity: row.try_get("severity").unwrap_or_else(|_| "error".to_string()),
        is_active: row.try_get("is_active").unwrap_or(true),
        priority: row.try_get("priority").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_instance(row: &sqlx::postgres::PgRow) -> ConfigInstance {
    ConfigInstance {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        instance_number: row.try_get("instance_number").unwrap_or_default(),
        model_id: row.try_get("model_id").unwrap_or_default(),
        model_number: row.try_get("model_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        selections: row.try_get("selections").unwrap_or(serde_json::json!({})),
        validation_errors: row.try_get("validation_errors").unwrap_or(serde_json::json!([])),
        validation_warnings: row.try_get("validation_warnings").unwrap_or(serde_json::json!([])),
        base_price: row.try_get("base_price").unwrap_or(0.0),
        total_price: row.try_get("total_price").unwrap_or(0.0),
        currency_code: row.try_get("currency_code").unwrap_or_else(|_| "USD".to_string()),
        config_hash: row.try_get("config_hash").unwrap_or_default(),
        effective_date: row.try_get("effective_date").unwrap_or_default(),
        valid_from: row.try_get("valid_from").unwrap_or_default(),
        valid_to: row.try_get("valid_to").unwrap_or_default(),
        sales_order_id: row.try_get("sales_order_id").unwrap_or_default(),
        sales_order_number: row.try_get("sales_order_number").unwrap_or_default(),
        sales_order_line: row.try_get("sales_order_line").unwrap_or_default(),
        configured_by: row.try_get("configured_by").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresProductConfiguratorRepository {
    pool: PgPool,
}

impl PostgresProductConfiguratorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductConfiguratorRepository for PostgresProductConfiguratorRepository {
    // ========================================================================
    // Configuration Models
    // ========================================================================

    async fn create_model(
        &self,
        org_id: Uuid, model_number: &str, name: &str, description: Option<&str>,
        base_product_id: Option<Uuid>, base_product_number: Option<&str>, base_product_name: Option<&str>,
        model_type: &str, status: &str, version: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        default_config: serde_json::Value, validation_mode: &str,
        ui_layout: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigModel> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.config_models
                (organization_id, model_number, name, description,
                 base_product_id, base_product_number, base_product_name,
                 model_type, status, version,
                 effective_from, effective_to, default_config, validation_mode,
                 ui_layout, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,'{}'::jsonb,$16)
            RETURNING *"#,
        )
        .bind(org_id).bind(model_number).bind(name).bind(description)
        .bind(base_product_id).bind(base_product_number).bind(base_product_name)
        .bind(model_type).bind(status).bind(version)
        .bind(effective_from).bind(effective_to).bind(&default_config).bind(validation_mode)
        .bind(&ui_layout).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_model(&row))
    }

    async fn get_model(&self, id: Uuid) -> AtlasResult<Option<ConfigModel>> {
        let row = sqlx::query("SELECT * FROM _atlas.config_models WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_model))
    }

    async fn get_model_by_number(&self, org_id: Uuid, model_number: &str) -> AtlasResult<Option<ConfigModel>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.config_models WHERE organization_id = $1 AND model_number = $2"
        ).bind(org_id).bind(model_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_model))
    }

    async fn list_models(
        &self, org_id: Uuid, status: Option<&str>, model_type: Option<&str>,
    ) -> AtlasResult<Vec<ConfigModel>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.config_models
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR model_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(model_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_model).collect())
    }

    async fn update_model_status(&self, id: Uuid, status: &str) -> AtlasResult<ConfigModel> {
        let row = sqlx::query(
            "UPDATE _atlas.config_models SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config model {} not found", id)))?;
        Ok(row_to_model(&row))
    }

    async fn delete_model(&self, org_id: Uuid, model_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.config_models WHERE organization_id = $1 AND model_number = $2"
        ).bind(org_id).bind(model_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Config model '{}' not found", model_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Configuration Features
    // ========================================================================

    async fn create_feature(
        &self,
        org_id: Uuid, model_id: Uuid, feature_code: &str, name: &str,
        description: Option<&str>, feature_type: &str, is_required: bool,
        display_order: i32, ui_hints: serde_json::Value,
    ) -> AtlasResult<ConfigFeature> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.config_features
                (organization_id, model_id, feature_code, name, description,
                 feature_type, is_required, display_order, ui_hints, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(model_id).bind(feature_code).bind(name).bind(description)
        .bind(feature_type).bind(is_required).bind(display_order).bind(&ui_hints)
        .fetch_one(&self.pool).await?;
        Ok(row_to_feature(&row))
    }

    async fn get_feature(&self, id: Uuid) -> AtlasResult<Option<ConfigFeature>> {
        let row = sqlx::query("SELECT * FROM _atlas.config_features WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_feature))
    }

    async fn list_features(&self, model_id: Uuid) -> AtlasResult<Vec<ConfigFeature>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.config_features WHERE model_id = $1 ORDER BY display_order"
        ).bind(model_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_feature).collect())
    }

    async fn delete_feature(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.config_features WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Feature not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Configuration Options
    // ========================================================================

    async fn create_option(
        &self,
        org_id: Uuid, feature_id: Uuid, option_code: &str, name: &str,
        description: Option<&str>, option_type: &str,
        price_adjustment: f64, cost_adjustment: f64, lead_time_days: i32,
        is_default: bool, is_available: bool, display_order: i32,
    ) -> AtlasResult<ConfigOption> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.config_options
                (organization_id, feature_id, option_code, name, description,
                 option_type, price_adjustment, cost_adjustment, lead_time_days,
                 is_default, is_available, display_order, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,'{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(feature_id).bind(option_code).bind(name).bind(description)
        .bind(option_type).bind(price_adjustment).bind(cost_adjustment).bind(lead_time_days)
        .bind(is_default).bind(is_available).bind(display_order)
        .fetch_one(&self.pool).await?;
        Ok(row_to_option(&row))
    }

    async fn get_option(&self, id: Uuid) -> AtlasResult<Option<ConfigOption>> {
        let row = sqlx::query("SELECT * FROM _atlas.config_options WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_option))
    }

    async fn list_options(&self, feature_id: Uuid) -> AtlasResult<Vec<ConfigOption>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.config_options WHERE feature_id = $1 ORDER BY display_order"
        ).bind(feature_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_option).collect())
    }

    async fn update_option_availability(&self, id: Uuid, is_available: bool) -> AtlasResult<ConfigOption> {
        let row = sqlx::query(
            "UPDATE _atlas.config_options SET is_available = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_available)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config option {} not found", id)))?;
        Ok(row_to_option(&row))
    }

    async fn delete_option(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.config_options WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Config option not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Configuration Rules
    // ========================================================================

    async fn create_rule(
        &self,
        org_id: Uuid, model_id: Uuid, rule_code: &str, name: &str,
        description: Option<&str>, rule_type: &str,
        source_feature_id: Option<Uuid>, source_option_id: Option<Uuid>,
        target_feature_id: Option<Uuid>, target_option_id: Option<Uuid>,
        condition_expression: Option<&str>, severity: &str,
        is_active: bool, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.config_rules
                (organization_id, model_id, rule_code, name, description,
                 rule_type, source_feature_id, source_option_id,
                 target_feature_id, target_option_id,
                 condition_expression, severity, is_active, priority, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'{}'::jsonb,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(model_id).bind(rule_code).bind(name).bind(description)
        .bind(rule_type).bind(source_feature_id).bind(source_option_id)
        .bind(target_feature_id).bind(target_option_id)
        .bind(condition_expression).bind(severity).bind(is_active).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_rule(&row))
    }

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ConfigRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.config_rules WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_rule))
    }

    async fn get_rule_by_code(&self, model_id: Uuid, rule_code: &str) -> AtlasResult<Option<ConfigRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.config_rules WHERE model_id = $1 AND rule_code = $2"
        ).bind(model_id).bind(rule_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_rule))
    }

    async fn list_rules(&self, model_id: Uuid, rule_type: Option<&str>) -> AtlasResult<Vec<ConfigRule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.config_rules
               WHERE model_id = $1
                 AND ($2::text IS NULL OR rule_type = $2)
               ORDER BY priority DESC, created_at"#,
        )
        .bind(model_id).bind(rule_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_rule).collect())
    }

    async fn update_rule_active(&self, id: Uuid, is_active: bool) -> AtlasResult<ConfigRule> {
        let row = sqlx::query(
            "UPDATE _atlas.config_rules SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config rule {} not found", id)))?;
        Ok(row_to_rule(&row))
    }

    async fn delete_rule(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.config_rules WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Config rule not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Configuration Instances
    // ========================================================================

    async fn create_instance(
        &self,
        org_id: Uuid, instance_number: &str, model_id: Uuid,
        model_number: Option<&str>, name: Option<&str>, description: Option<&str>,
        status: &str, selections: serde_json::Value,
        validation_errors: serde_json::Value, validation_warnings: serde_json::Value,
        base_price: f64, total_price: f64, currency_code: &str,
        config_hash: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        configured_by: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.config_instances
                (organization_id, instance_number, model_id, model_number,
                 name, description, status, selections,
                 validation_errors, validation_warnings,
                 base_price, total_price, currency_code, config_hash,
                 effective_date, configured_by, metadata, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,'{}'::jsonb,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(instance_number).bind(model_id).bind(model_number)
        .bind(name).bind(description).bind(status).bind(&selections)
        .bind(&validation_errors).bind(&validation_warnings)
        .bind(base_price).bind(total_price).bind(currency_code).bind(config_hash)
        .bind(effective_date).bind(configured_by).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_instance(&row))
    }

    async fn get_instance(&self, id: Uuid) -> AtlasResult<Option<ConfigInstance>> {
        let row = sqlx::query("SELECT * FROM _atlas.config_instances WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_instance))
    }

    async fn get_instance_by_number(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<Option<ConfigInstance>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.config_instances WHERE organization_id = $1 AND instance_number = $2"
        ).bind(org_id).bind(instance_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_instance))
    }

    async fn list_instances(
        &self, org_id: Uuid, status: Option<&str>, model_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<ConfigInstance>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.config_instances
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::uuid IS NULL OR model_id = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(model_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_instance).collect())
    }

    async fn update_instance_status(&self, id: Uuid, status: &str) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.config_instances
               SET status = $2, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config instance {} not found", id)))?;
        Ok(row_to_instance(&row))
    }

    async fn update_instance_validation(
        &self, id: Uuid, validation_errors: serde_json::Value, validation_warnings: serde_json::Value,
    ) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.config_instances
               SET validation_errors = $2, validation_warnings = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(&validation_errors).bind(&validation_warnings)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config instance {} not found", id)))?;
        Ok(row_to_instance(&row))
    }

    async fn update_instance_selections(
        &self, id: Uuid, selections: serde_json::Value, total_price: f64, config_hash: Option<&str>,
    ) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.config_instances
               SET selections = $2, total_price = $3, config_hash = $4, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(&selections).bind(total_price).bind(config_hash)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config instance {} not found", id)))?;
        Ok(row_to_instance(&row))
    }

    async fn update_instance_approval(
        &self, id: Uuid, approved_by: Option<Uuid>,
    ) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.config_instances
               SET approved_by = $2, approved_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config instance {} not found", id)))?;
        Ok(row_to_instance(&row))
    }

    async fn link_instance_to_order(
        &self, id: Uuid, sales_order_id: Option<Uuid>,
        sales_order_number: Option<&str>, sales_order_line: Option<i32>,
    ) -> AtlasResult<ConfigInstance> {
        let row = sqlx::query(
            r#"UPDATE _atlas.config_instances
               SET sales_order_id = $2, sales_order_number = $3,
                   sales_order_line = $4, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(sales_order_id).bind(sales_order_number).bind(sales_order_line)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Config instance {} not found", id)))?;
        Ok(row_to_instance(&row))
    }

    async fn delete_instance(&self, org_id: Uuid, instance_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.config_instances WHERE organization_id = $1 AND instance_number = $2"
        ).bind(org_id).bind(instance_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Config instance '{}' not found", instance_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ConfiguratorDashboard> {
        let model_rows = sqlx::query(
            "SELECT status, model_type FROM _atlas.config_models WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_models = model_rows.len() as i32;
        let active_models = model_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;

        let mut models_by_status = std::collections::HashMap::new();
        for r in &model_rows {
            let s = r.try_get::<String, _>("status").unwrap_or_default();
            *models_by_status.entry(s).or_insert(0i32) += 1;
        }

        let inst_rows = sqlx::query(
            "SELECT status, total_price FROM _atlas.config_instances WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_configurations = inst_rows.len() as i32;
        let valid_configurations = inst_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "valid")
            .count() as i32;
        let invalid_configurations = inst_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "invalid")
            .count() as i32;
        let ordered_configurations = inst_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "ordered")
            .count() as i32;

        let total_configured_value: f64 = inst_rows.iter()
            .map(|r| r.try_get("total_price").unwrap_or(0.0)).sum();
        let avg_configuration_price = if total_configurations > 0 {
            total_configured_value / total_configurations as f64
        } else {
            0.0
        };

        let mut configurations_by_status = std::collections::HashMap::new();
        for r in &inst_rows {
            let s = r.try_get::<String, _>("status").unwrap_or_default();
            *configurations_by_status.entry(s).or_insert(0i32) += 1;
        }

        let rule_rows = sqlx::query(
            "SELECT is_active FROM _atlas.config_rules WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_rules = rule_rows.len() as i32;
        let active_rules = rule_rows.iter()
            .filter(|r| r.try_get::<bool, _>("is_active").unwrap_or(true))
            .count() as i32;

        Ok(ConfiguratorDashboard {
            total_models,
            active_models,
            total_configurations,
            valid_configurations,
            invalid_configurations,
            ordered_configurations,
            total_rules,
            active_rules,
            avg_configuration_price,
            total_configured_value,
            models_by_status: serde_json::to_value(models_by_status).unwrap_or(serde_json::json!({})),
            configurations_by_status: serde_json::to_value(configurations_by_status).unwrap_or(serde_json::json!({})),
            top_configured_models: serde_json::json!([]),
        })
    }
}
