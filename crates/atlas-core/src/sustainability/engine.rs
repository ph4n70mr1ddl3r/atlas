//! Sustainability Engine
//!
//! Manages facility tracking, GHG emissions, ESG metrics, sustainability goals,
//! and carbon offset management.
//!
//! Oracle Fusion Cloud equivalent: Sustainability > Environmental Accounting, ESG Reporting

use atlas_shared::{
    SustainabilityFacility, EmissionFactor, EnvironmentalActivity,
    EsgMetric, EsgMetricReading, SustainabilityGoal, CarbonOffset,
    SustainabilityDashboard,
    AtlasError, AtlasResult,
};
use super::SustainabilityRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_FACILITY_TYPES: &[&str] = &[
    "office", "manufacturing", "warehouse", "data_center", "retail", "other",
];

const VALID_FACILITY_STATUSES: &[&str] = &[
    "active", "inactive", "decommissioned",
];

const VALID_EMISSION_SCOPES: &[&str] = &[
    "scope_1", "scope_2", "scope_3",
];

const VALID_EMISSION_CATEGORIES: &[&str] = &[
    "stationary_combustion", "mobile_combustion", "fugitive_emissions",
    "process_emissions", "purchased_electricity", "purchased_steam",
    "purchased_cooling", "purchased_heating",
    "upstream_transportation", "downstream_transportation",
    "business_travel", "employee_commuting", "waste_generated",
    "purchased_goods", "capital_goods", "fuel_energy_related",
    "upstream_leased_assets", "downstream_leased_assets",
    "franchises", "investments", "other",
];

const VALID_GAS_TYPES: &[&str] = &[
    "co2", "ch4", "n2o", "hfcs", "pfcs", "sf6", "co2e",
];

const VALID_ACTIVITY_SOURCES: &[&str] = &[
    "manual_entry", "gl_journal", "invoice", "meter_reading", "estimation", "api_import",
];

const VALID_ACTIVITY_STATUSES: &[&str] = &[
    "draft", "confirmed", "verified", "adjusted",
];

const VALID_ESG_PILLARS: &[&str] = &[
    "environmental", "social", "governance",
];

const VALID_METRIC_DIRECTIONS: &[&str] = &[
    "lower_is_better", "higher_is_better",
];

const VALID_GOAL_TYPES: &[&str] = &[
    "emission_reduction", "energy_efficiency", "waste_reduction",
    "water_reduction", "carbon_neutral", "renewable_energy",
];

const VALID_GOAL_STATUSES: &[&str] = &[
    "on_track", "at_risk", "off_track", "achieved", "cancelled",
];

const VALID_GOAL_FRAMEWORKS: &[&str] = &[
    "SBTi", "UN_SDG", "Paris_Agreement", "EU_Green_Deal", "Custom",
];

const VALID_OFFSET_PROJECT_TYPES: &[&str] = &[
    "reforestation", "renewable_energy", "methane_capture",
    "direct_air_capture", "cookstove", "ocean_restoration", "other",
];

#[allow(dead_code)]
const VALID_OFFSET_STATUSES: &[&str] = &[
    "active", "retired", "expired", "cancelled",
];

const VALID_OFFSET_REGISTRIES: &[&str] = &[
    "Verra", "Gold_Standard", "ACR", "CAR", "Other",
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

/// Sustainability & ESG Management Engine
pub struct SustainabilityEngine {
    repository: Arc<dyn SustainabilityRepository>,
}

impl SustainabilityEngine {
    pub fn new(repository: Arc<dyn SustainabilityRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Facilities
    // ========================================================================

    /// Create a sustainability facility
    #[allow(clippy::too_many_arguments)]
    pub async fn create_facility(
        &self,
        org_id: Uuid,
        facility_code: &str,
        name: &str,
        description: Option<&str>,
        country_code: Option<&str>,
        region: Option<&str>,
        city: Option<&str>,
        address: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        facility_type: &str,
        industry_sector: Option<&str>,
        total_area_sqm: Option<f64>,
        employee_count: Option<i32>,
        operating_hours_per_year: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityFacility> {
        if facility_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Facility code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Facility name is required".to_string()));
        }
        validate_enum("facility_type", facility_type, VALID_FACILITY_TYPES)?;

        if self.repository.get_facility_by_code(org_id, facility_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Facility '{}' already exists", facility_code
            )));
        }

        info!("Creating sustainability facility '{}' ({}) for org {}", facility_code, name, org_id);
        self.repository.create_facility(
            org_id, facility_code, name, description,
            country_code, region, city, address,
            latitude, longitude, facility_type, industry_sector,
            total_area_sqm, employee_count,
            operating_hours_per_year.unwrap_or(8760),
            created_by,
        ).await
    }

    /// Get a facility by ID
    pub async fn get_facility(&self, id: Uuid) -> AtlasResult<Option<SustainabilityFacility>> {
        self.repository.get_facility(id).await
    }

    /// Get a facility by code
    pub async fn get_facility_by_code(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<Option<SustainabilityFacility>> {
        self.repository.get_facility_by_code(org_id, facility_code).await
    }

    /// List facilities with optional filter
    pub async fn list_facilities(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        facility_type: Option<&str>,
    ) -> AtlasResult<Vec<SustainabilityFacility>> {
        self.repository.list_facilities(org_id, status, facility_type).await
    }

    /// Update facility status
    pub async fn update_facility_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityFacility> {
        validate_enum("facility status", status, VALID_FACILITY_STATUSES)?;
        info!("Updating facility {} status to {}", id, status);
        self.repository.update_facility_status(id, status).await
    }

    /// Delete a facility by code
    pub async fn delete_facility(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<()> {
        info!("Deleting facility '{}' for org {}", facility_code, org_id);
        self.repository.delete_facility(org_id, facility_code).await
    }

    // ========================================================================
    // Emission Factors
    // ========================================================================

    /// Create an emission factor
    #[allow(clippy::too_many_arguments)]
    pub async fn create_emission_factor(
        &self,
        org_id: Uuid,
        factor_code: &str,
        name: &str,
        description: Option<&str>,
        scope: &str,
        category: &str,
        activity_type: &str,
        factor_value: f64,
        unit_of_measure: &str,
        gas_type: &str,
        factor_source: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        region_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EmissionFactor> {
        if factor_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Factor code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Factor name is required".to_string()));
        }
        validate_enum("scope", scope, VALID_EMISSION_SCOPES)?;
        validate_enum("category", category, VALID_EMISSION_CATEGORIES)?;
        validate_enum("gas_type", gas_type, VALID_GAS_TYPES)?;

        if factor_value <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Factor value must be positive".to_string(),
            ));
        }
        if !unit_of_measure.is_empty() && effective_from > effective_to.unwrap_or(effective_from) {
            return Err(AtlasError::ValidationFailed(
                "Effective from must be before effective to".to_string(),
            ));
        }

        if self.repository.get_emission_factor_by_code(org_id, factor_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Emission factor '{}' already exists", factor_code
            )));
        }

        info!("Creating emission factor '{}' ({}) for org {} [scope={}, category={}]",
              factor_code, name, org_id, scope, category);

        self.repository.create_emission_factor(
            org_id, factor_code, name, description,
            scope, category, activity_type, factor_value,
            unit_of_measure, gas_type, factor_source,
            effective_from, effective_to, region_code,
            created_by,
        ).await
    }

    /// Get an emission factor by ID
    pub async fn get_emission_factor(&self, id: Uuid) -> AtlasResult<Option<EmissionFactor>> {
        self.repository.get_emission_factor(id).await
    }

    /// Get an emission factor by code
    pub async fn get_emission_factor_by_code(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<Option<EmissionFactor>> {
        self.repository.get_emission_factor_by_code(org_id, factor_code).await
    }

    /// List emission factors with optional filters
    pub async fn list_emission_factors(
        &self,
        org_id: Uuid,
        scope: Option<&str>,
        category: Option<&str>,
        activity_type: Option<&str>,
    ) -> AtlasResult<Vec<EmissionFactor>> {
        self.repository.list_emission_factors(org_id, scope, category, activity_type).await
    }

    /// Delete an emission factor by code
    pub async fn delete_emission_factor(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<()> {
        info!("Deleting emission factor '{}' for org {}", factor_code, org_id);
        self.repository.delete_emission_factor(org_id, factor_code).await
    }

    // ========================================================================
    // Environmental Activities (Emissions / Consumption Logs)
    // ========================================================================

    /// Create an environmental activity entry
    #[allow(clippy::too_many_arguments)]
    pub async fn create_activity(
        &self,
        org_id: Uuid,
        activity_number: &str,
        facility_id: Option<Uuid>,
        facility_code: Option<&str>,
        activity_type: &str,
        scope: &str,
        category: Option<&str>,
        quantity: f64,
        unit_of_measure: &str,
        emission_factor_id: Option<Uuid>,
        co2e_kg: f64,
        co2_kg: Option<f64>,
        ch4_kg: Option<f64>,
        n2o_kg: Option<f64>,
        cost_amount: Option<f64>,
        cost_currency: Option<&str>,
        activity_date: chrono::NaiveDate,
        reporting_period: Option<&str>,
        source_type: Option<&str>,
        source_reference: Option<&str>,
        department_id: Option<Uuid>,
        project_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EnvironmentalActivity> {
        if activity_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Activity number is required".to_string()));
        }
        validate_enum("scope", scope, VALID_EMISSION_SCOPES)?;
        if let Some(st) = source_type {
            validate_enum("source_type", st, VALID_ACTIVITY_SOURCES)?;
        }

        if quantity <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive".to_string(),
            ));
        }

        // Verify emission factor if provided
        if let Some(ef_id) = emission_factor_id {
            self.repository.get_emission_factor(ef_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Emission factor {} not found", ef_id
                )))?;
        }

        // Verify facility if provided
        if let Some(f_id) = facility_id {
            self.repository.get_facility(f_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Facility {} not found", f_id
                )))?;
        }

        if self.repository.get_activity_by_number(org_id, activity_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Activity '{}' already exists", activity_number
            )));
        }

        info!("Creating environmental activity '{}' for org {} [scope={}, type={}, co2e={:.2}kg]",
              activity_number, org_id, scope, activity_type, co2e_kg);

        self.repository.create_activity(
            org_id, activity_number, facility_id, facility_code,
            activity_type, scope, category, quantity, unit_of_measure,
            emission_factor_id, co2e_kg, co2_kg, ch4_kg, n2o_kg,
            cost_amount, cost_currency, activity_date, reporting_period,
            source_type, source_reference, department_id, project_id,
            created_by,
        ).await
    }

    /// Get an activity by ID
    pub async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<EnvironmentalActivity>> {
        self.repository.get_activity(id).await
    }

    /// Get an activity by number
    pub async fn get_activity_by_number(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<Option<EnvironmentalActivity>> {
        self.repository.get_activity_by_number(org_id, activity_number).await
    }

    /// List activities with optional filters
    pub async fn list_activities(
        &self,
        org_id: Uuid,
        scope: Option<&str>,
        facility_id: Option<&Uuid>,
        activity_type: Option<&str>,
        reporting_period: Option<&str>,
    ) -> AtlasResult<Vec<EnvironmentalActivity>> {
        self.repository.list_activities(org_id, scope, facility_id, activity_type, reporting_period).await
    }

    /// Update activity status (e.g., verify an entry)
    pub async fn update_activity_status(&self, id: Uuid, status: &str) -> AtlasResult<EnvironmentalActivity> {
        validate_enum("activity status", status, VALID_ACTIVITY_STATUSES)?;
        info!("Updating activity {} status to {}", id, status);
        self.repository.update_activity_status(id, status).await
    }

    /// Delete an activity by number
    pub async fn delete_activity(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<()> {
        info!("Deleting activity '{}' for org {}", activity_number, org_id);
        self.repository.delete_activity(org_id, activity_number).await
    }

    // ========================================================================
    // ESG Metrics
    // ========================================================================

    /// Create an ESG metric definition
    #[allow(clippy::too_many_arguments)]
    pub async fn create_metric(
        &self,
        org_id: Uuid,
        metric_code: &str,
        name: &str,
        description: Option<&str>,
        pillar: &str,
        category: &str,
        unit_of_measure: &str,
        gri_standard: Option<&str>,
        sasb_standard: Option<&str>,
        tcfd_category: Option<&str>,
        eu_taxonomy_code: Option<&str>,
        target_value: Option<f64>,
        warning_threshold: Option<f64>,
        direction: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetric> {
        if metric_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Metric code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Metric name is required".to_string()));
        }
        validate_enum("pillar", pillar, VALID_ESG_PILLARS)?;
        validate_enum("direction", direction, VALID_METRIC_DIRECTIONS)?;
        if unit_of_measure.is_empty() {
            return Err(AtlasError::ValidationFailed("Unit of measure is required".to_string()));
        }

        if self.repository.get_metric_by_code(org_id, metric_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "ESG metric '{}' already exists", metric_code
            )));
        }

        info!("Creating ESG metric '{}' ({}) for org {} [pillar={}, category={}]",
              metric_code, name, org_id, pillar, category);

        self.repository.create_metric(
            org_id, metric_code, name, description,
            pillar, category, unit_of_measure,
            gri_standard, sasb_standard, tcfd_category, eu_taxonomy_code,
            target_value, warning_threshold, direction,
            created_by,
        ).await
    }

    /// Get a metric by ID
    pub async fn get_metric(&self, id: Uuid) -> AtlasResult<Option<EsgMetric>> {
        self.repository.get_metric(id).await
    }

    /// Get a metric by code
    pub async fn get_metric_by_code(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<Option<EsgMetric>> {
        self.repository.get_metric_by_code(org_id, metric_code).await
    }

    /// List metrics with optional filters
    pub async fn list_metrics(
        &self,
        org_id: Uuid,
        pillar: Option<&str>,
        category: Option<&str>,
    ) -> AtlasResult<Vec<EsgMetric>> {
        self.repository.list_metrics(org_id, pillar, category).await
    }

    /// Delete a metric by code
    pub async fn delete_metric(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<()> {
        info!("Deleting ESG metric '{}' for org {}", metric_code, org_id);
        self.repository.delete_metric(org_id, metric_code).await
    }

    // ========================================================================
    // ESG Metric Readings
    // ========================================================================

    /// Record an ESG metric reading
    pub async fn create_metric_reading(
        &self,
        org_id: Uuid,
        metric_id: Uuid,
        metric_value: f64,
        reading_date: chrono::NaiveDate,
        reporting_period: Option<&str>,
        facility_id: Option<Uuid>,
        notes: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetricReading> {
        // Verify metric exists
        self.repository.get_metric(metric_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "ESG metric {} not found", metric_id
            )))?;

        // Verify facility if provided
        if let Some(f_id) = facility_id {
            self.repository.get_facility(f_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Facility {} not found", f_id
                )))?;
        }

        info!("Recording ESG metric reading for metric {} [value={}, date={}]",
              metric_id, metric_value, reading_date);

        self.repository.create_metric_reading(
            org_id, metric_id, metric_value, reading_date,
            reporting_period, facility_id, notes, source,
            created_by,
        ).await
    }

    /// Get a metric reading by ID
    pub async fn get_metric_reading(&self, id: Uuid) -> AtlasResult<Option<EsgMetricReading>> {
        self.repository.get_metric_reading(id).await
    }

    /// List readings for a metric
    pub async fn list_metric_readings(
        &self,
        metric_id: Uuid,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<EsgMetricReading>> {
        self.repository.list_metric_readings(metric_id, from_date, to_date).await
    }

    /// Delete a metric reading
    pub async fn delete_metric_reading(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting metric reading {}", id);
        self.repository.delete_metric_reading(id).await
    }

    // ========================================================================
    // Sustainability Goals
    // ========================================================================

    /// Create a sustainability goal
    #[allow(clippy::too_many_arguments)]
    pub async fn create_goal(
        &self,
        org_id: Uuid,
        goal_code: &str,
        name: &str,
        description: Option<&str>,
        goal_type: &str,
        scope: Option<&str>,
        baseline_value: f64,
        baseline_year: i32,
        baseline_unit: &str,
        target_value: f64,
        target_year: i32,
        target_unit: &str,
        target_reduction_pct: Option<f64>,
        milestones: Option<serde_json::Value>,
        facility_id: Option<Uuid>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        framework: Option<&str>,
        framework_reference: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityGoal> {
        if goal_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Goal code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Goal name is required".to_string()));
        }
        validate_enum("goal_type", goal_type, VALID_GOAL_TYPES)?;
        if let Some(s) = scope {
            validate_enum("scope", s, VALID_EMISSION_SCOPES)?;
        }
        if let Some(fw) = framework {
            validate_enum("framework", fw, VALID_GOAL_FRAMEWORKS)?;
        }
        if !(1990..=2100).contains(&baseline_year) {
            return Err(AtlasError::ValidationFailed(
                "Baseline year must be between 1990 and 2100".to_string(),
            ));
        }
        if !(1990..=2100).contains(&target_year) {
            return Err(AtlasError::ValidationFailed(
                "Target year must be between 1990 and 2100".to_string(),
            ));
        }
        if target_year <= baseline_year {
            return Err(AtlasError::ValidationFailed(
                "Target year must be after baseline year".to_string(),
            ));
        }
        if baseline_unit.is_empty() {
            return Err(AtlasError::ValidationFailed("Baseline unit is required".to_string()));
        }
        if target_unit.is_empty() {
            return Err(AtlasError::ValidationFailed("Target unit is required".to_string()));
        }

        // Verify facility if provided
        if let Some(f_id) = facility_id {
            self.repository.get_facility(f_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Facility {} not found", f_id
                )))?;
        }

        if self.repository.get_goal_by_code(org_id, goal_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Goal '{}' already exists", goal_code
            )));
        }

        let progress = if baseline_value != 0.0 {
            let reduction = baseline_value - target_value;
            let current_reduction = baseline_value; // starts at baseline
            (current_reduction / reduction * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        info!("Creating sustainability goal '{}' ({}) for org {} [type={}, baseline={}, target={}]",
              goal_code, name, org_id, goal_type, baseline_value, target_value);

        self.repository.create_goal(
            org_id, goal_code, name, description,
            goal_type, scope,
            baseline_value, baseline_year, baseline_unit,
            target_value, target_year, target_unit,
            target_reduction_pct,
            milestones.unwrap_or(serde_json::json!([])),
            progress,
            facility_id, owner_id, owner_name,
            framework, framework_reference,
            effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get a goal by ID
    pub async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<SustainabilityGoal>> {
        self.repository.get_goal(id).await
    }

    /// Get a goal by code
    pub async fn get_goal_by_code(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<Option<SustainabilityGoal>> {
        self.repository.get_goal_by_code(org_id, goal_code).await
    }

    /// List goals with optional filters
    pub async fn list_goals(
        &self,
        org_id: Uuid,
        goal_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<SustainabilityGoal>> {
        self.repository.list_goals(org_id, goal_type, status).await
    }

    /// Update goal progress
    pub async fn update_goal_progress(&self, id: Uuid, current_value: f64) -> AtlasResult<SustainabilityGoal> {
        info!("Updating goal {} progress to {}", id, current_value);
        self.repository.update_goal_progress(id, current_value).await
    }

    /// Update goal status
    pub async fn update_goal_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityGoal> {
        validate_enum("goal status", status, VALID_GOAL_STATUSES)?;
        info!("Updating goal {} status to {}", id, status);
        self.repository.update_goal_status(id, status).await
    }

    /// Delete a goal by code
    pub async fn delete_goal(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<()> {
        info!("Deleting goal '{}' for org {}", goal_code, org_id);
        self.repository.delete_goal(org_id, goal_code).await
    }

    // ========================================================================
    // Carbon Offsets
    // ========================================================================

    /// Create a carbon offset
    #[allow(clippy::too_many_arguments)]
    pub async fn create_carbon_offset(
        &self,
        org_id: Uuid,
        offset_number: &str,
        name: &str,
        description: Option<&str>,
        project_name: &str,
        project_type: &str,
        project_location: Option<&str>,
        registry: Option<&str>,
        registry_id: Option<&str>,
        certification_standard: Option<&str>,
        quantity_tonnes: f64,
        unit_price: Option<f64>,
        total_cost: Option<f64>,
        currency_code: Option<&str>,
        vintage_year: i32,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        supplier_name: Option<&str>,
        supplier_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CarbonOffset> {
        if offset_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Offset number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Offset name is required".to_string()));
        }
        if project_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Project name is required".to_string()));
        }
        validate_enum("project_type", project_type, VALID_OFFSET_PROJECT_TYPES)?;
        if let Some(r) = registry {
            validate_enum("registry", r, VALID_OFFSET_REGISTRIES)?;
        }

        if quantity_tonnes <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive".to_string(),
            ));
        }
        if !(2000..=2100).contains(&vintage_year) {
            return Err(AtlasError::ValidationFailed(
                "Vintage year must be between 2000 and 2100".to_string(),
            ));
        }

        if self.repository.get_offset_by_number(org_id, offset_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Carbon offset '{}' already exists", offset_number
            )));
        }

        info!("Creating carbon offset '{}' ({}) for org {} [type={}, qty={:.2}t]",
              offset_number, name, org_id, project_type, quantity_tonnes);

        self.repository.create_carbon_offset(
            org_id, offset_number, name, description,
            project_name, project_type, project_location,
            registry, registry_id, certification_standard,
            quantity_tonnes, quantity_tonnes, // remaining = quantity initially
            unit_price, total_cost, currency_code,
            vintage_year, effective_from, effective_to,
            supplier_name, supplier_id, notes,
            created_by,
        ).await
    }

    /// Get a carbon offset by ID
    pub async fn get_carbon_offset(&self, id: Uuid) -> AtlasResult<Option<CarbonOffset>> {
        self.repository.get_carbon_offset(id).await
    }

    /// Get a carbon offset by number
    pub async fn get_offset_by_number(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<Option<CarbonOffset>> {
        self.repository.get_offset_by_number(org_id, offset_number).await
    }

    /// List carbon offsets with optional filters
    pub async fn list_carbon_offsets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        project_type: Option<&str>,
    ) -> AtlasResult<Vec<CarbonOffset>> {
        self.repository.list_carbon_offsets(org_id, status, project_type).await
    }

    /// Retire carbon offsets
    pub async fn retire_carbon_offset(
        &self,
        id: Uuid,
        retire_quantity: f64,
    ) -> AtlasResult<CarbonOffset> {
        if retire_quantity <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Retire quantity must be positive".to_string(),
            ));
        }
        info!("Retiring {:.2} tonnes from carbon offset {}", retire_quantity, id);
        self.repository.retire_carbon_offset(id, retire_quantity).await
    }

    /// Delete a carbon offset by number
    pub async fn delete_carbon_offset(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<()> {
        info!("Deleting carbon offset '{}' for org {}", offset_number, org_id);
        self.repository.delete_carbon_offset(org_id, offset_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the sustainability dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SustainabilityDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_facility_types() {
        assert!(VALID_FACILITY_TYPES.contains(&"office"));
        assert!(VALID_FACILITY_TYPES.contains(&"manufacturing"));
        assert!(VALID_FACILITY_TYPES.contains(&"warehouse"));
        assert!(VALID_FACILITY_TYPES.contains(&"data_center"));
        assert!(VALID_FACILITY_TYPES.contains(&"retail"));
        assert!(VALID_FACILITY_TYPES.contains(&"other"));
        assert!(!VALID_FACILITY_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_emission_scopes() {
        assert!(VALID_EMISSION_SCOPES.contains(&"scope_1"));
        assert!(VALID_EMISSION_SCOPES.contains(&"scope_2"));
        assert!(VALID_EMISSION_SCOPES.contains(&"scope_3"));
        assert!(!VALID_EMISSION_SCOPES.contains(&"scope_4"));
    }

    #[test]
    fn test_valid_emission_categories() {
        assert!(VALID_EMISSION_CATEGORIES.contains(&"stationary_combustion"));
        assert!(VALID_EMISSION_CATEGORIES.contains(&"purchased_electricity"));
        assert!(VALID_EMISSION_CATEGORIES.contains(&"business_travel"));
        assert!(VALID_EMISSION_CATEGORIES.contains(&"employee_commuting"));
        assert!(VALID_EMISSION_CATEGORIES.contains(&"purchased_goods"));
    }

    #[test]
    fn test_valid_gas_types() {
        assert!(VALID_GAS_TYPES.contains(&"co2"));
        assert!(VALID_GAS_TYPES.contains(&"ch4"));
        assert!(VALID_GAS_TYPES.contains(&"n2o"));
        assert!(VALID_GAS_TYPES.contains(&"co2e"));
        assert!(VALID_GAS_TYPES.contains(&"hfcs"));
    }

    #[test]
    fn test_valid_activity_sources() {
        assert!(VALID_ACTIVITY_SOURCES.contains(&"manual_entry"));
        assert!(VALID_ACTIVITY_SOURCES.contains(&"gl_journal"));
        assert!(VALID_ACTIVITY_SOURCES.contains(&"meter_reading"));
        assert!(VALID_ACTIVITY_SOURCES.contains(&"estimation"));
    }

    #[test]
    fn test_valid_activity_statuses() {
        assert!(VALID_ACTIVITY_STATUSES.contains(&"draft"));
        assert!(VALID_ACTIVITY_STATUSES.contains(&"confirmed"));
        assert!(VALID_ACTIVITY_STATUSES.contains(&"verified"));
        assert!(VALID_ACTIVITY_STATUSES.contains(&"adjusted"));
    }

    #[test]
    fn test_valid_esg_pillars() {
        assert!(VALID_ESG_PILLARS.contains(&"environmental"));
        assert!(VALID_ESG_PILLARS.contains(&"social"));
        assert!(VALID_ESG_PILLARS.contains(&"governance"));
        assert!(!VALID_ESG_PILLARS.contains(&"economic"));
    }

    #[test]
    fn test_valid_metric_directions() {
        assert!(VALID_METRIC_DIRECTIONS.contains(&"lower_is_better"));
        assert!(VALID_METRIC_DIRECTIONS.contains(&"higher_is_better"));
        assert!(!VALID_METRIC_DIRECTIONS.contains(&"neutral"));
    }

    #[test]
    fn test_valid_goal_types() {
        assert!(VALID_GOAL_TYPES.contains(&"emission_reduction"));
        assert!(VALID_GOAL_TYPES.contains(&"energy_efficiency"));
        assert!(VALID_GOAL_TYPES.contains(&"waste_reduction"));
        assert!(VALID_GOAL_TYPES.contains(&"water_reduction"));
        assert!(VALID_GOAL_TYPES.contains(&"carbon_neutral"));
        assert!(VALID_GOAL_TYPES.contains(&"renewable_energy"));
    }

    #[test]
    fn test_valid_goal_statuses() {
        assert!(VALID_GOAL_STATUSES.contains(&"on_track"));
        assert!(VALID_GOAL_STATUSES.contains(&"at_risk"));
        assert!(VALID_GOAL_STATUSES.contains(&"off_track"));
        assert!(VALID_GOAL_STATUSES.contains(&"achieved"));
        assert!(VALID_GOAL_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_offset_project_types() {
        assert!(VALID_OFFSET_PROJECT_TYPES.contains(&"reforestation"));
        assert!(VALID_OFFSET_PROJECT_TYPES.contains(&"renewable_energy"));
        assert!(VALID_OFFSET_PROJECT_TYPES.contains(&"methane_capture"));
        assert!(VALID_OFFSET_PROJECT_TYPES.contains(&"direct_air_capture"));
    }

    #[test]
    fn test_valid_offset_statuses() {
        assert!(VALID_OFFSET_STATUSES.contains(&"active"));
        assert!(VALID_OFFSET_STATUSES.contains(&"retired"));
        assert!(VALID_OFFSET_STATUSES.contains(&"expired"));
        assert!(VALID_OFFSET_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test", "scope_1", VALID_EMISSION_SCOPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("scope", "invalid", VALID_EMISSION_SCOPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("scope"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("scope", "", VALID_EMISSION_SCOPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockSustainabilityRepository;
    use chrono::NaiveDate;

    fn create_engine() -> SustainabilityEngine {
        SustainabilityEngine::new(Arc::new(MockSustainabilityRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    // --- Facility Tests ---

    #[tokio::test]
    async fn test_create_facility_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_facility(
            test_org_id(), "", "HQ Office", None,
            None, None, None, None, None, None,
            "office", None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_facility_validation_empty_name() {
        let engine = create_engine();
        let result = engine.create_facility(
            test_org_id(), "FAC-001", "", None,
            None, None, None, None, None, None,
            "office", None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("name")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_facility_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_facility(
            test_org_id(), "FAC-001", "HQ Office", None,
            None, None, None, None, None, None,
            "hospital", None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("facility_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_facility_success() {
        let engine = create_engine();
        let result = engine.create_facility(
            test_org_id(), "FAC-001", "HQ Office", Some("Main headquarters"),
            Some("US"), Some("California"), Some("San Francisco"), None, None, None,
            "office", Some("Technology"), Some(5000.0), Some(200), None,
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let fac = result.unwrap();
        assert_eq!(fac.facility_code, "FAC-001");
        assert_eq!(fac.name, "HQ Office");
        assert_eq!(fac.facility_type, "office");
    }

    #[tokio::test]
    async fn test_update_facility_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_facility_status(Uuid::new_v4(), "unknown").await;
        assert!(result.is_err());
    }

    // --- Emission Factor Tests ---

    #[tokio::test]
    async fn test_create_emission_factor_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_emission_factor(
            test_org_id(), "", "Natural Gas Factor", None,
            "scope_1", "stationary_combustion", "natural_gas",
            5.3, "therms", "co2e", None,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_emission_factor_validation_bad_scope() {
        let engine = create_engine();
        let result = engine.create_emission_factor(
            test_org_id(), "EF-001", "Factor", None,
            "scope_4", "stationary_combustion", "natural_gas",
            5.3, "therms", "co2e", None,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("scope")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_emission_factor_validation_bad_category() {
        let engine = create_engine();
        let result = engine.create_emission_factor(
            test_org_id(), "EF-001", "Factor", None,
            "scope_1", "nuclear_fusion", "natural_gas",
            5.3, "therms", "co2e", None,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("category")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_emission_factor_validation_negative_value() {
        let engine = create_engine();
        let result = engine.create_emission_factor(
            test_org_id(), "EF-001", "Factor", None,
            "scope_1", "stationary_combustion", "natural_gas",
            -5.3, "therms", "co2e", None,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_emission_factor_success() {
        let engine = create_engine();
        let result = engine.create_emission_factor(
            test_org_id(), "EF-001", "US Grid Electricity", Some("EPA eGRID 2024"),
            "scope_2", "purchased_electricity", "electricity_grid",
            0.3886, "kWh", "co2e", Some("EPA eGRID"),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, Some("US"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let ef = result.unwrap();
        assert_eq!(ef.factor_code, "EF-001");
        assert_eq!(ef.scope, "scope_2");
        assert!((ef.factor_value - 0.3886).abs() < 0.0001);
    }

    // --- Environmental Activity Tests ---

    #[tokio::test]
    async fn test_create_activity_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_activity(
            test_org_id(), "", None, None,
            "natural_gas", "scope_1", Some("stationary_combustion"),
            1000.0, "therms", None,
            5300.0, None, None, None,
            None, None,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), Some("2024-Q1"),
            Some("manual_entry"), None, None, None,
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_activity_validation_bad_scope() {
        let engine = create_engine();
        let result = engine.create_activity(
            test_org_id(), "ACT-001", None, None,
            "natural_gas", "scope_5", Some("stationary_combustion"),
            1000.0, "therms", None,
            5300.0, None, None, None,
            None, None,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), Some("2024-Q1"),
            None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("scope")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_activity_validation_negative_quantity() {
        let engine = create_engine();
        let result = engine.create_activity(
            test_org_id(), "ACT-001", None, None,
            "electricity", "scope_2", Some("purchased_electricity"),
            -500.0, "kWh", None,
            194.3, None, None, None,
            None, None,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), Some("2024-Q1"),
            None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_activity_success() {
        let engine = create_engine();
        let result = engine.create_activity(
            test_org_id(), "ACT-001", None, Some("FAC-001"),
            "electricity", "scope_2", Some("purchased_electricity"),
            10000.0, "kWh", None,
            3886.0, None, None, None,
            Some(850.0), Some("USD"),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), Some("2024-Q1"),
            Some("meter_reading"), None, None, None,
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let act = result.unwrap();
        assert_eq!(act.activity_number, "ACT-001");
        assert_eq!(act.scope, "scope_2");
        assert!((act.co2e_kg - 3886.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_update_activity_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_activity_status(Uuid::new_v4(), "deleted").await;
        assert!(result.is_err());
    }

    // --- ESG Metric Tests ---

    #[tokio::test]
    async fn test_create_metric_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_metric(
            test_org_id(), "", "Total Emissions", None,
            "environmental", "climate", "tonnes CO2e",
            None, None, None, None,
            None, None, "lower_is_better",
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_metric_validation_bad_pillar() {
        let engine = create_engine();
        let result = engine.create_metric(
            test_org_id(), "ESG-001", "Total Emissions", None,
            "economic", "climate", "tonnes CO2e",
            None, None, None, None,
            None, None, "lower_is_better",
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("pillar")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_metric_validation_bad_direction() {
        let engine = create_engine();
        let result = engine.create_metric(
            test_org_id(), "ESG-001", "Total Emissions", None,
            "environmental", "climate", "tonnes CO2e",
            None, None, None, None,
            None, None, "neutral",
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_metric_success() {
        let engine = create_engine();
        let result = engine.create_metric(
            test_org_id(), "ESG-001", "Scope 1+2 GHG Emissions", Some("Total direct and energy-indirect emissions"),
            "environmental", "climate", "tonnes CO2e",
            Some("GRI 305-4"), Some("EM-EP-110a.1"), Some("Metrics and Targets"), None,
            Some(5000.0), Some(6000.0), "lower_is_better",
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let m = result.unwrap();
        assert_eq!(m.metric_code, "ESG-001");
        assert_eq!(m.pillar, "environmental");
    }

    // --- Sustainability Goal Tests ---

    #[tokio::test]
    async fn test_create_goal_validation_empty_code() {
        let engine = create_engine();
        let result = engine.create_goal(
            test_org_id(), "", "Net Zero", None,
            "emission_reduction", Some("scope_1"),
            10000.0, 2020, "tonnes CO2e",
            0.0, 2030, "tonnes CO2e",
            Some(100.0), None, None,
            None, None, Some("SBTi"), None, None, None,
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_goal_validation_bad_type() {
        let engine = create_engine();
        let result = engine.create_goal(
            test_org_id(), "GOAL-001", "Net Zero", None,
            "plastic_free", Some("scope_1"),
            10000.0, 2020, "tonnes CO2e",
            0.0, 2030, "tonnes CO2e",
            Some(100.0), None, None,
            None, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("goal_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_goal_validation_target_before_baseline() {
        let engine = create_engine();
        let result = engine.create_goal(
            test_org_id(), "GOAL-001", "Net Zero", None,
            "emission_reduction", Some("scope_1"),
            10000.0, 2030, "tonnes CO2e",
            0.0, 2020, "tonnes CO2e",
            Some(100.0), None, None,
            None, None, None, None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Target year")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_goal_validation_bad_framework() {
        let engine = create_engine();
        let result = engine.create_goal(
            test_org_id(), "GOAL-001", "Net Zero", None,
            "emission_reduction", Some("scope_1"),
            10000.0, 2020, "tonnes CO2e",
            0.0, 2030, "tonnes CO2e",
            Some(100.0), None, None,
            None, None, Some("InvalidFramework"), None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("framework")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_goal_success() {
        let engine = create_engine();
        let result = engine.create_goal(
            test_org_id(), "GOAL-001", "50% Emission Reduction", Some("Science-based target"),
            "emission_reduction", Some("scope_1"),
            10000.0, 2020, "tonnes CO2e",
            5000.0, 2030, "tonnes CO2e",
            Some(50.0), Some(serde_json::json!([{"year": 2025, "value": 7500}])),
            None, Some(test_user_id()), Some("Sustainability Director"),
            Some("SBTi"), Some("SBTi-2024-001"),
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let goal = result.unwrap();
        assert_eq!(goal.goal_code, "GOAL-001");
        assert_eq!(goal.goal_type, "emission_reduction");
    }

    // --- Carbon Offset Tests ---

    #[tokio::test]
    async fn test_create_offset_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_carbon_offset(
            test_org_id(), "", "Amazon Reforestation", None,
            "Amazon Reforestation Project", "reforestation", Some("Brazil"),
            Some("Verra"), Some("VCS-1234"), None,
            1000.0, Some(15.0), Some(15000.0), Some("USD"),
            2023,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()),
            Some("GreenCarbon Inc."), None, None,
            None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_offset_validation_bad_project_type() {
        let engine = create_engine();
        let result = engine.create_carbon_offset(
            test_org_id(), "OFF-001", "Carbon Removal", None,
            "Ocean cleanup", "ocean_cleanup", None,
            None, None, None,
            500.0, None, None, None,
            2024,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None,
            None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("project_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_offset_validation_negative_quantity() {
        let engine = create_engine();
        let result = engine.create_carbon_offset(
            test_org_id(), "OFF-001", "Bad Offset", None,
            "Some Project", "reforestation", None,
            None, None, None,
            -100.0, None, None, None,
            2024,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None,
            None, None, None,
            None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_offset_success() {
        let engine = create_engine();
        let result = engine.create_carbon_offset(
            test_org_id(), "OFF-001", "Amazon Reforestation", Some("Verified carbon credits"),
            "Amazon Reforestation Project", "reforestation", Some("Brazil"),
            Some("Verra"), Some("VCS-1234"), Some("VCS"),
            1000.0, Some(15.0), Some(15000.0), Some("USD"),
            2023,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()),
            Some("GreenCarbon Inc."), None, Some("Purchased for 2024 offset"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let offset = result.unwrap();
        assert_eq!(offset.offset_number, "OFF-001");
        assert_eq!(offset.project_type, "reforestation");
        assert!((offset.quantity_tonnes - 1000.0).abs() < 0.01);
        assert!((offset.remaining_tonnes - 1000.0).abs() < 0.01);
    }
}
