//! Sustainability Repository
//!
//! PostgreSQL storage for facilities, emission factors, environmental activities,
//! ESG metrics/readings, sustainability goals, carbon offsets, and dashboard.

use atlas_shared::{
    SustainabilityFacility, EmissionFactor, EnvironmentalActivity,
    EsgMetric, EsgMetricReading, SustainabilityGoal, CarbonOffset,
    SustainabilityDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for sustainability data storage
#[async_trait]
pub trait SustainabilityRepository: Send + Sync {
    // Facilities
    async fn create_facility(
        &self, org_id: Uuid, facility_code: &str, name: &str, description: Option<&str>,
        country_code: Option<&str>, region: Option<&str>, city: Option<&str>,
        address: Option<&str>, latitude: Option<f64>, longitude: Option<f64>,
        facility_type: &str, industry_sector: Option<&str>,
        total_area_sqm: Option<f64>, employee_count: Option<i32>,
        operating_hours_per_year: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityFacility>;
    async fn get_facility(&self, id: Uuid) -> AtlasResult<Option<SustainabilityFacility>>;
    async fn get_facility_by_code(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<Option<SustainabilityFacility>>;
    async fn list_facilities(&self, org_id: Uuid, status: Option<&str>, facility_type: Option<&str>) -> AtlasResult<Vec<SustainabilityFacility>>;
    async fn update_facility_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityFacility>;
    async fn delete_facility(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<()>;

    // Emission Factors
    async fn create_emission_factor(
        &self, org_id: Uuid, factor_code: &str, name: &str, description: Option<&str>,
        scope: &str, category: &str, activity_type: &str, factor_value: f64,
        unit_of_measure: &str, gas_type: &str, factor_source: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        region_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<EmissionFactor>;
    async fn get_emission_factor(&self, id: Uuid) -> AtlasResult<Option<EmissionFactor>>;
    async fn get_emission_factor_by_code(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<Option<EmissionFactor>>;
    async fn list_emission_factors(&self, org_id: Uuid, scope: Option<&str>, category: Option<&str>, activity_type: Option<&str>) -> AtlasResult<Vec<EmissionFactor>>;
    async fn delete_emission_factor(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<()>;

    // Environmental Activities
    async fn create_activity(
        &self, org_id: Uuid, activity_number: &str,
        facility_id: Option<Uuid>, facility_code: Option<&str>,
        activity_type: &str, scope: &str, category: Option<&str>,
        quantity: f64, unit_of_measure: &str,
        emission_factor_id: Option<Uuid>,
        co2e_kg: f64, co2_kg: Option<f64>, ch4_kg: Option<f64>, n2o_kg: Option<f64>,
        cost_amount: Option<f64>, cost_currency: Option<&str>,
        activity_date: chrono::NaiveDate, reporting_period: Option<&str>,
        source_type: Option<&str>, source_reference: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EnvironmentalActivity>;
    async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<EnvironmentalActivity>>;
    async fn get_activity_by_number(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<Option<EnvironmentalActivity>>;
    async fn list_activities(&self, org_id: Uuid, scope: Option<&str>, facility_id: Option<&Uuid>, activity_type: Option<&str>, reporting_period: Option<&str>) -> AtlasResult<Vec<EnvironmentalActivity>>;
    async fn update_activity_status(&self, id: Uuid, status: &str) -> AtlasResult<EnvironmentalActivity>;
    async fn delete_activity(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<()>;

    // ESG Metrics
    async fn create_metric(
        &self, org_id: Uuid, metric_code: &str, name: &str, description: Option<&str>,
        pillar: &str, category: &str, unit_of_measure: &str,
        gri_standard: Option<&str>, sasb_standard: Option<&str>,
        tcfd_category: Option<&str>, eu_taxonomy_code: Option<&str>,
        target_value: Option<f64>, warning_threshold: Option<f64>, direction: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetric>;
    async fn get_metric(&self, id: Uuid) -> AtlasResult<Option<EsgMetric>>;
    async fn get_metric_by_code(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<Option<EsgMetric>>;
    async fn list_metrics(&self, org_id: Uuid, pillar: Option<&str>, category: Option<&str>) -> AtlasResult<Vec<EsgMetric>>;
    async fn delete_metric(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<()>;

    // ESG Metric Readings
    async fn create_metric_reading(
        &self, org_id: Uuid, metric_id: Uuid, metric_value: f64,
        reading_date: chrono::NaiveDate, reporting_period: Option<&str>,
        facility_id: Option<Uuid>, notes: Option<&str>, source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetricReading>;
    async fn get_metric_reading(&self, id: Uuid) -> AtlasResult<Option<EsgMetricReading>>;
    async fn list_metric_readings(&self, metric_id: Uuid, from_date: Option<chrono::NaiveDate>, to_date: Option<chrono::NaiveDate>) -> AtlasResult<Vec<EsgMetricReading>>;
    async fn delete_metric_reading(&self, id: Uuid) -> AtlasResult<()>;

    // Sustainability Goals
    async fn create_goal(
        &self, org_id: Uuid, goal_code: &str, name: &str, description: Option<&str>,
        goal_type: &str, scope: Option<&str>,
        baseline_value: f64, baseline_year: i32, baseline_unit: &str,
        target_value: f64, target_year: i32, target_unit: &str,
        target_reduction_pct: Option<f64>, milestones: serde_json::Value,
        progress_pct: f64,
        facility_id: Option<Uuid>, owner_id: Option<Uuid>, owner_name: Option<&str>,
        framework: Option<&str>, framework_reference: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityGoal>;
    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<SustainabilityGoal>>;
    async fn get_goal_by_code(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<Option<SustainabilityGoal>>;
    async fn list_goals(&self, org_id: Uuid, goal_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<SustainabilityGoal>>;
    async fn update_goal_progress(&self, id: Uuid, current_value: f64) -> AtlasResult<SustainabilityGoal>;
    async fn update_goal_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityGoal>;
    async fn delete_goal(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<()>;

    // Carbon Offsets
    async fn create_carbon_offset(
        &self, org_id: Uuid, offset_number: &str, name: &str, description: Option<&str>,
        project_name: &str, project_type: &str, project_location: Option<&str>,
        registry: Option<&str>, registry_id: Option<&str>, certification_standard: Option<&str>,
        quantity_tonnes: f64, remaining_tonnes: f64,
        unit_price: Option<f64>, total_cost: Option<f64>, currency_code: Option<&str>,
        vintage_year: i32, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        supplier_name: Option<&str>, supplier_id: Option<Uuid>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CarbonOffset>;
    async fn get_carbon_offset(&self, id: Uuid) -> AtlasResult<Option<CarbonOffset>>;
    async fn get_offset_by_number(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<Option<CarbonOffset>>;
    async fn list_carbon_offsets(&self, org_id: Uuid, status: Option<&str>, project_type: Option<&str>) -> AtlasResult<Vec<CarbonOffset>>;
    async fn retire_carbon_offset(&self, id: Uuid, retire_quantity: f64) -> AtlasResult<CarbonOffset>;
    async fn delete_carbon_offset(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SustainabilityDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresSustainabilityRepository {
    pool: PgPool,
}

impl PostgresSustainabilityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper: map row to SustainabilityFacility
/// Helper to decode NUMERIC columns from PostgreSQL as f64
/// sqlx 0.7 without bigdecimal/rust_decimal features can't decode NUMERIC as f64 directly
fn get_numeric(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    // Try f64 first (works with DOUBLE PRECISION columns)
    if let Ok(v) = row.try_get::<f64, _>(column) {
        return v;
    }
    // Try serde_json::Value (works for NUMERIC via JSON conversion)
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() {
            return n;
        }
        if let Some(s) = v.as_str() {
            if let Ok(n) = s.parse::<f64>() {
                return n;
            }
        }
    }
    // Try String (NUMERIC comes as string in some sqlx configs)
    if let Ok(s) = row.try_get::<String, _>(column) {
        return s.parse::<f64>().unwrap_or(0.0);
    }
    0.0
}

fn get_optional_numeric(row: &sqlx::postgres::PgRow, column: &str) -> Option<f64> {
    // Try f64 first
    if let Ok(v) = row.try_get::<f64, _>(column) {
        return Some(v);
    }
    // Try serde_json::Value
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(s) = v.as_str() {
            return s.parse::<f64>().ok();
        }
    }
    // Try String
    if let Ok(s) = row.try_get::<String, _>(column) {
        return s.parse::<f64>().ok();
    }
    None
}

fn row_to_facility(row: &sqlx::postgres::PgRow) -> SustainabilityFacility {
    SustainabilityFacility {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        facility_code: row.try_get("facility_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        country_code: row.try_get("country_code").unwrap_or_default(),
        region: row.try_get("region").unwrap_or_default(),
        city: row.try_get("city").unwrap_or_default(),
        address: row.try_get("address").unwrap_or_default(),
        latitude: get_optional_numeric(row, "latitude"),
        longitude: get_optional_numeric(row, "longitude"),
        facility_type: row.try_get("facility_type").unwrap_or_default(),
        industry_sector: row.try_get("industry_sector").unwrap_or_default(),
        total_area_sqm: get_optional_numeric(row, "total_area_sqm"),
        employee_count: row.try_get("employee_count").unwrap_or_default(),
        operating_hours_per_year: row.try_get("operating_hours_per_year").unwrap_or(8760),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_emission_factor(row: &sqlx::postgres::PgRow) -> EmissionFactor {
    EmissionFactor {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        factor_code: row.try_get("factor_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        scope: row.try_get("scope").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        activity_type: row.try_get("activity_type").unwrap_or_default(),
        factor_value: get_numeric(row, "factor_value"),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        gas_type: row.try_get("gas_type").unwrap_or_default(),
        factor_source: row.try_get("factor_source").unwrap_or_default(),
        effective_from: row.try_get("effective_from").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        region_code: row.try_get("region_code").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_activity(row: &sqlx::postgres::PgRow) -> EnvironmentalActivity {
    EnvironmentalActivity {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        activity_number: row.try_get("activity_number").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        facility_code: row.try_get("facility_code").unwrap_or_default(),
        activity_type: row.try_get("activity_type").unwrap_or_default(),
        scope: row.try_get("scope").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        quantity: get_numeric(row, "quantity"),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        emission_factor_id: row.try_get("emission_factor_id").unwrap_or_default(),
        co2e_kg: get_numeric(row, "co2e_kg"),
        co2_kg: get_optional_numeric(row, "co2_kg"),
        ch4_kg: get_optional_numeric(row, "ch4_kg"),
        n2o_kg: get_optional_numeric(row, "n2o_kg"),
        cost_amount: get_optional_numeric(row, "cost_amount"),
        cost_currency: row.try_get("cost_currency").unwrap_or_default(),
        activity_date: row.try_get("activity_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        reporting_period: row.try_get("reporting_period").unwrap_or_default(),
        source_type: row.try_get("source_type").unwrap_or_default(),
        source_reference: row.try_get("source_reference").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        verified_by: row.try_get("verified_by").unwrap_or_default(),
        verified_at: row.try_get("verified_at").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_metric(row: &sqlx::postgres::PgRow) -> EsgMetric {
    EsgMetric {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        metric_code: row.try_get("metric_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        pillar: row.try_get("pillar").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        gri_standard: row.try_get("gri_standard").unwrap_or_default(),
        sasb_standard: row.try_get("sasb_standard").unwrap_or_default(),
        tcfd_category: row.try_get("tcfd_category").unwrap_or_default(),
        eu_taxonomy_code: row.try_get("eu_taxonomy_code").unwrap_or_default(),
        target_value: get_optional_numeric(row, "target_value"),
        warning_threshold: get_optional_numeric(row, "warning_threshold"),
        direction: row.try_get("direction").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_reading(row: &sqlx::postgres::PgRow) -> EsgMetricReading {
    EsgMetricReading {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        metric_id: row.try_get("metric_id").unwrap_or_default(),
        metric_value: get_numeric(row, "metric_value"),
        reading_date: row.try_get("reading_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        reporting_period: row.try_get("reporting_period").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        source: row.try_get("source").unwrap_or_default(),
        verified_by: row.try_get("verified_by").unwrap_or_default(),
        verified_at: row.try_get("verified_at").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_goal(row: &sqlx::postgres::PgRow) -> SustainabilityGoal {
    SustainabilityGoal {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        goal_code: row.try_get("goal_code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        goal_type: row.try_get("goal_type").unwrap_or_default(),
        scope: row.try_get("scope").unwrap_or_default(),
        baseline_value: get_numeric(row, "baseline_value"),
        baseline_year: row.try_get("baseline_year").unwrap_or(2024),
        baseline_unit: row.try_get("baseline_unit").unwrap_or_default(),
        target_value: get_numeric(row, "target_value"),
        target_year: row.try_get("target_year").unwrap_or(2030),
        target_unit: row.try_get("target_unit").unwrap_or_default(),
        target_reduction_pct: get_optional_numeric(row, "target_reduction_pct"),
        milestones: row.try_get("milestones").unwrap_or(serde_json::json!([])),
        current_value: get_numeric(row, "current_value"),
        progress_pct: get_numeric(row, "progress_pct"),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        framework: row.try_get("framework").unwrap_or_default(),
        framework_reference: row.try_get("framework_reference").unwrap_or_default(),
        effective_from: row.try_get("effective_from").unwrap_or_default(),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_offset(row: &sqlx::postgres::PgRow) -> CarbonOffset {
    CarbonOffset {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        offset_number: row.try_get("offset_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        project_name: row.try_get("project_name").unwrap_or_default(),
        project_type: row.try_get("project_type").unwrap_or_default(),
        project_location: row.try_get("project_location").unwrap_or_default(),
        registry: row.try_get("registry").unwrap_or_default(),
        registry_id: row.try_get("registry_id").unwrap_or_default(),
        certification_standard: row.try_get("certification_standard").unwrap_or_default(),
        quantity_tonnes: get_numeric(row, "quantity_tonnes"),
        remaining_tonnes: get_numeric(row, "remaining_tonnes"),
        unit_price: get_optional_numeric(row, "unit_price"),
        total_cost: get_optional_numeric(row, "total_cost"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        vintage_year: row.try_get("vintage_year").unwrap_or(2024),
        retired_quantity: get_numeric(row, "retired_quantity"),
        retired_date: row.try_get("retired_date").unwrap_or_default(),
        effective_from: row.try_get("effective_from").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        effective_to: row.try_get("effective_to").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        supplier_name: row.try_get("supplier_name").unwrap_or_default(),
        supplier_id: row.try_get("supplier_id").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl SustainabilityRepository for PostgresSustainabilityRepository {
    // ========================================================================
    // Facilities
    // ========================================================================

    async fn create_facility(
        &self, org_id: Uuid, facility_code: &str, name: &str, description: Option<&str>,
        country_code: Option<&str>, region: Option<&str>, city: Option<&str>,
        address: Option<&str>, latitude: Option<f64>, longitude: Option<f64>,
        facility_type: &str, industry_sector: Option<&str>,
        total_area_sqm: Option<f64>, employee_count: Option<i32>,
        operating_hours_per_year: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityFacility> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sustainability_facilities
                (organization_id, facility_code, name, description,
                 country_code, region, city, address, latitude, longitude,
                 facility_type, industry_sector, total_area_sqm, employee_count,
                 operating_hours_per_year, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, '{}'::jsonb, $16)
            RETURNING *"#,
        )
        .bind(org_id).bind(facility_code).bind(name).bind(description)
        .bind(country_code).bind(region).bind(city).bind(address)
        .bind(latitude).bind(longitude)
        .bind(facility_type).bind(industry_sector)
        .bind(total_area_sqm).bind(employee_count)
        .bind(operating_hours_per_year).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_facility(&row))
    }

    async fn get_facility(&self, id: Uuid) -> AtlasResult<Option<SustainabilityFacility>> {
        let row = sqlx::query("SELECT * FROM _atlas.sustainability_facilities WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_facility))
    }

    async fn get_facility_by_code(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<Option<SustainabilityFacility>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sustainability_facilities WHERE organization_id = $1 AND facility_code = $2"
        ).bind(org_id).bind(facility_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_facility))
    }

    async fn list_facilities(&self, org_id: Uuid, status: Option<&str>, facility_type: Option<&str>) -> AtlasResult<Vec<SustainabilityFacility>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.sustainability_facilities
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR facility_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(facility_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_facility).collect())
    }

    async fn update_facility_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityFacility> {
        let row = sqlx::query(
            "UPDATE _atlas.sustainability_facilities SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Facility {} not found", id)))?;
        Ok(row_to_facility(&row))
    }

    async fn delete_facility(&self, org_id: Uuid, facility_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.sustainability_facilities WHERE organization_id = $1 AND facility_code = $2"
        ).bind(org_id).bind(facility_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Facility '{}' not found", facility_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Emission Factors
    // ========================================================================

    async fn create_emission_factor(
        &self, org_id: Uuid, factor_code: &str, name: &str, description: Option<&str>,
        scope: &str, category: &str, activity_type: &str, factor_value: f64,
        unit_of_measure: &str, gas_type: &str, factor_source: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        region_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<EmissionFactor> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.emission_factors
                (organization_id, factor_code, name, description,
                 scope, category, activity_type, factor_value,
                 unit_of_measure, gas_type, factor_source,
                 effective_from, effective_to, region_code, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, '{}'::jsonb, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(factor_code).bind(name).bind(description)
        .bind(scope).bind(category).bind(activity_type).bind(factor_value)
        .bind(unit_of_measure).bind(gas_type).bind(factor_source)
        .bind(effective_from).bind(effective_to).bind(region_code)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_emission_factor(&row))
    }

    async fn get_emission_factor(&self, id: Uuid) -> AtlasResult<Option<EmissionFactor>> {
        let row = sqlx::query("SELECT * FROM _atlas.emission_factors WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_emission_factor))
    }

    async fn get_emission_factor_by_code(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<Option<EmissionFactor>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.emission_factors WHERE organization_id = $1 AND factor_code = $2"
        ).bind(org_id).bind(factor_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_emission_factor))
    }

    async fn list_emission_factors(&self, org_id: Uuid, scope: Option<&str>, category: Option<&str>, activity_type: Option<&str>) -> AtlasResult<Vec<EmissionFactor>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.emission_factors
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR scope = $2)
                 AND ($3::text IS NULL OR category = $3)
                 AND ($4::text IS NULL OR activity_type = $4)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(scope).bind(category).bind(activity_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_emission_factor).collect())
    }

    async fn delete_emission_factor(&self, org_id: Uuid, factor_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.emission_factors WHERE organization_id = $1 AND factor_code = $2"
        ).bind(org_id).bind(factor_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Emission factor '{}' not found", factor_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Environmental Activities
    // ========================================================================

    async fn create_activity(
        &self, org_id: Uuid, activity_number: &str,
        facility_id: Option<Uuid>, facility_code: Option<&str>,
        activity_type: &str, scope: &str, category: Option<&str>,
        quantity: f64, unit_of_measure: &str,
        emission_factor_id: Option<Uuid>,
        co2e_kg: f64, co2_kg: Option<f64>, ch4_kg: Option<f64>, n2o_kg: Option<f64>,
        cost_amount: Option<f64>, cost_currency: Option<&str>,
        activity_date: chrono::NaiveDate, reporting_period: Option<&str>,
        source_type: Option<&str>, source_reference: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EnvironmentalActivity> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.environmental_activities
                (organization_id, activity_number,
                 facility_id, facility_code,
                 activity_type, scope, category,
                 quantity, unit_of_measure, emission_factor_id,
                 co2e_kg, co2_kg, ch4_kg, n2o_kg,
                 cost_amount, cost_currency,
                 activity_date, reporting_period,
                 source_type, source_reference,
                 department_id, project_id,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18,
                    $19, $20, $21, $22, '{}'::jsonb, $23)
            RETURNING *"#,
        )
        .bind(org_id).bind(activity_number)
        .bind(facility_id).bind(facility_code)
        .bind(activity_type).bind(scope).bind(category)
        .bind(quantity).bind(unit_of_measure).bind(emission_factor_id)
        .bind(co2e_kg).bind(co2_kg).bind(ch4_kg).bind(n2o_kg)
        .bind(cost_amount).bind(cost_currency)
        .bind(activity_date).bind(reporting_period)
        .bind(source_type).bind(source_reference)
        .bind(department_id).bind(project_id)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_activity(&row))
    }

    async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<EnvironmentalActivity>> {
        let row = sqlx::query("SELECT * FROM _atlas.environmental_activities WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_activity))
    }

    async fn get_activity_by_number(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<Option<EnvironmentalActivity>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.environmental_activities WHERE organization_id = $1 AND activity_number = $2"
        ).bind(org_id).bind(activity_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_activity))
    }

    async fn list_activities(&self, org_id: Uuid, scope: Option<&str>, facility_id: Option<&Uuid>, activity_type: Option<&str>, reporting_period: Option<&str>) -> AtlasResult<Vec<EnvironmentalActivity>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.environmental_activities
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR scope = $2)
                 AND ($3::uuid IS NULL OR facility_id = $3)
                 AND ($4::text IS NULL OR activity_type = $4)
                 AND ($5::text IS NULL OR reporting_period = $5)
               ORDER BY activity_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(scope).bind(facility_id.copied()).bind(activity_type).bind(reporting_period)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_activity).collect())
    }

    async fn update_activity_status(&self, id: Uuid, status: &str) -> AtlasResult<EnvironmentalActivity> {
        let row = sqlx::query(
            r#"UPDATE _atlas.environmental_activities SET status = $2,
                verified_at = CASE WHEN $3 THEN now() ELSE verified_at END,
                updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(status == "verified")
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Activity {} not found", id)))?;
        Ok(row_to_activity(&row))
    }

    async fn delete_activity(&self, org_id: Uuid, activity_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.environmental_activities WHERE organization_id = $1 AND activity_number = $2"
        ).bind(org_id).bind(activity_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Activity '{}' not found", activity_number)));
        }
        Ok(())
    }

    // ========================================================================
    // ESG Metrics
    // ========================================================================

    async fn create_metric(
        &self, org_id: Uuid, metric_code: &str, name: &str, description: Option<&str>,
        pillar: &str, category: &str, unit_of_measure: &str,
        gri_standard: Option<&str>, sasb_standard: Option<&str>,
        tcfd_category: Option<&str>, eu_taxonomy_code: Option<&str>,
        target_value: Option<f64>, warning_threshold: Option<f64>, direction: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetric> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.esg_metrics
                (organization_id, metric_code, name, description,
                 pillar, category, unit_of_measure,
                 gri_standard, sasb_standard, tcfd_category, eu_taxonomy_code,
                 target_value, warning_threshold, direction, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, '{}'::jsonb, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(metric_code).bind(name).bind(description)
        .bind(pillar).bind(category).bind(unit_of_measure)
        .bind(gri_standard).bind(sasb_standard).bind(tcfd_category).bind(eu_taxonomy_code)
        .bind(target_value).bind(warning_threshold).bind(direction)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_metric(&row))
    }

    async fn get_metric(&self, id: Uuid) -> AtlasResult<Option<EsgMetric>> {
        let row = sqlx::query("SELECT * FROM _atlas.esg_metrics WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_metric))
    }

    async fn get_metric_by_code(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<Option<EsgMetric>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.esg_metrics WHERE organization_id = $1 AND metric_code = $2"
        ).bind(org_id).bind(metric_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_metric))
    }

    async fn list_metrics(&self, org_id: Uuid, pillar: Option<&str>, category: Option<&str>) -> AtlasResult<Vec<EsgMetric>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.esg_metrics
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR pillar = $2)
                 AND ($3::text IS NULL OR category = $3)
               ORDER BY pillar, category, created_at"#,
        )
        .bind(org_id).bind(pillar).bind(category)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_metric).collect())
    }

    async fn delete_metric(&self, org_id: Uuid, metric_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.esg_metrics WHERE organization_id = $1 AND metric_code = $2"
        ).bind(org_id).bind(metric_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Metric '{}' not found", metric_code)));
        }
        Ok(())
    }

    // ========================================================================
    // ESG Metric Readings
    // ========================================================================

    async fn create_metric_reading(
        &self, org_id: Uuid, metric_id: Uuid, metric_value: f64,
        reading_date: chrono::NaiveDate, reporting_period: Option<&str>,
        facility_id: Option<Uuid>, notes: Option<&str>, source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EsgMetricReading> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.esg_metric_readings
                (organization_id, metric_id, metric_value,
                 reading_date, reporting_period, facility_id,
                 notes, source, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}'::jsonb, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(metric_id).bind(metric_value)
        .bind(reading_date).bind(reporting_period).bind(facility_id)
        .bind(notes).bind(source).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_reading(&row))
    }

    async fn get_metric_reading(&self, id: Uuid) -> AtlasResult<Option<EsgMetricReading>> {
        let row = sqlx::query("SELECT * FROM _atlas.esg_metric_readings WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_reading))
    }

    async fn list_metric_readings(&self, metric_id: Uuid, from_date: Option<chrono::NaiveDate>, to_date: Option<chrono::NaiveDate>) -> AtlasResult<Vec<EsgMetricReading>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.esg_metric_readings
               WHERE metric_id = $1
                 AND ($2::date IS NULL OR reading_date >= $2)
                 AND ($3::date IS NULL OR reading_date <= $3)
               ORDER BY reading_date DESC"#,
        )
        .bind(metric_id).bind(from_date).bind(to_date)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_reading).collect())
    }

    async fn delete_metric_reading(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.esg_metric_readings WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Reading not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Sustainability Goals
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_goal(
        &self, org_id: Uuid, goal_code: &str, name: &str, description: Option<&str>,
        goal_type: &str, scope: Option<&str>,
        baseline_value: f64, baseline_year: i32, baseline_unit: &str,
        target_value: f64, target_year: i32, target_unit: &str,
        target_reduction_pct: Option<f64>, milestones: serde_json::Value,
        progress_pct: f64,
        facility_id: Option<Uuid>, owner_id: Option<Uuid>, owner_name: Option<&str>,
        framework: Option<&str>, framework_reference: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SustainabilityGoal> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sustainability_goals
                (organization_id, goal_code, name, description,
                 goal_type, scope,
                 baseline_value, baseline_year, baseline_unit,
                 target_value, target_year, target_unit,
                 target_reduction_pct, milestones,
                 current_value, progress_pct,
                 facility_id, owner_id, owner_name,
                 framework, framework_reference,
                 effective_from, effective_to,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, $21, $22, $23, '{}'::jsonb, $24)
            RETURNING *"#,
        )
        .bind(org_id).bind(goal_code).bind(name).bind(description)
        .bind(goal_type).bind(scope)
        .bind(baseline_value).bind(baseline_year).bind(baseline_unit)
        .bind(target_value).bind(target_year).bind(target_unit)
        .bind(target_reduction_pct).bind(&milestones)
        .bind(baseline_value) // current_value starts at baseline
        .bind(progress_pct)
        .bind(facility_id).bind(owner_id).bind(owner_name)
        .bind(framework).bind(framework_reference)
        .bind(effective_from).bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_goal(&row))
    }

    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<SustainabilityGoal>> {
        let row = sqlx::query("SELECT * FROM _atlas.sustainability_goals WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_goal))
    }

    async fn get_goal_by_code(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<Option<SustainabilityGoal>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sustainability_goals WHERE organization_id = $1 AND goal_code = $2"
        ).bind(org_id).bind(goal_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_goal))
    }

    async fn list_goals(&self, org_id: Uuid, goal_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<SustainabilityGoal>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.sustainability_goals
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR goal_type = $2)
                 AND ($3::text IS NULL OR status = $3)
               ORDER BY target_year, created_at"#,
        )
        .bind(org_id).bind(goal_type).bind(status)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_goal).collect())
    }

    async fn update_goal_progress(&self, id: Uuid, current_value: f64) -> AtlasResult<SustainabilityGoal> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sustainability_goals
               SET current_value = $2,
                   progress_pct = CASE
                       WHEN (baseline_value - target_value) = 0 THEN 0
                       ELSE LEAST(100.0, GREATEST(0.0, (baseline_value - $2) / (baseline_value - target_value) * 100.0))
                   END,
                   status = CASE
                       WHEN (baseline_value - target_value) = 0 THEN status
                       WHEN (baseline_value - $2) / (baseline_value - target_value) >= 1.0 THEN 'achieved'
                       ELSE status
                   END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(current_value)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Goal {} not found", id)))?;
        Ok(row_to_goal(&row))
    }

    async fn update_goal_status(&self, id: Uuid, status: &str) -> AtlasResult<SustainabilityGoal> {
        let row = sqlx::query(
            "UPDATE _atlas.sustainability_goals SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Goal {} not found", id)))?;
        Ok(row_to_goal(&row))
    }

    async fn delete_goal(&self, org_id: Uuid, goal_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.sustainability_goals WHERE organization_id = $1 AND goal_code = $2"
        ).bind(org_id).bind(goal_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Goal '{}' not found", goal_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Carbon Offsets
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_carbon_offset(
        &self, org_id: Uuid, offset_number: &str, name: &str, description: Option<&str>,
        project_name: &str, project_type: &str, project_location: Option<&str>,
        registry: Option<&str>, registry_id: Option<&str>, certification_standard: Option<&str>,
        quantity_tonnes: f64, remaining_tonnes: f64,
        unit_price: Option<f64>, total_cost: Option<f64>, currency_code: Option<&str>,
        vintage_year: i32, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        supplier_name: Option<&str>, supplier_id: Option<Uuid>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CarbonOffset> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.carbon_offsets
                (organization_id, offset_number, name, description,
                 project_name, project_type, project_location,
                 registry, registry_id, certification_standard,
                 quantity_tonnes, remaining_tonnes,
                 unit_price, total_cost, currency_code,
                 vintage_year, effective_from, effective_to,
                 supplier_name, supplier_id, notes,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, '{}'::jsonb, $22)
            RETURNING *"#,
        )
        .bind(org_id).bind(offset_number).bind(name).bind(description)
        .bind(project_name).bind(project_type).bind(project_location)
        .bind(registry).bind(registry_id).bind(certification_standard)
        .bind(quantity_tonnes).bind(remaining_tonnes)
        .bind(unit_price).bind(total_cost).bind(currency_code)
        .bind(vintage_year).bind(effective_from).bind(effective_to)
        .bind(supplier_name).bind(supplier_id).bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_offset(&row))
    }

    async fn get_carbon_offset(&self, id: Uuid) -> AtlasResult<Option<CarbonOffset>> {
        let row = sqlx::query("SELECT * FROM _atlas.carbon_offsets WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_offset))
    }

    async fn get_offset_by_number(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<Option<CarbonOffset>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.carbon_offsets WHERE organization_id = $1 AND offset_number = $2"
        ).bind(org_id).bind(offset_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_offset))
    }

    async fn list_carbon_offsets(&self, org_id: Uuid, status: Option<&str>, project_type: Option<&str>) -> AtlasResult<Vec<CarbonOffset>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.carbon_offsets
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR project_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(project_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_offset).collect())
    }

    async fn retire_carbon_offset(&self, id: Uuid, retire_quantity: f64) -> AtlasResult<CarbonOffset> {
        let row = sqlx::query(
            r#"UPDATE _atlas.carbon_offsets
               SET remaining_tonnes = remaining_tonnes - $2,
                   retired_quantity = retired_quantity + $2,
                   retired_date = CASE WHEN remaining_tonnes - $2 <= 0 THEN CURRENT_DATE ELSE retired_date END,
                   status = CASE WHEN remaining_tonnes - $2 <= 0 THEN 'retired' ELSE status END,
                   updated_at = now()
               WHERE id = $1 AND remaining_tonnes >= $2
               RETURNING *"#,
        )
        .bind(id).bind(retire_quantity)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::ValidationFailed(
            "Carbon offset not found or insufficient remaining quantity".to_string()
        ))?;
        Ok(row_to_offset(&row))
    }

    async fn delete_carbon_offset(&self, org_id: Uuid, offset_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.carbon_offsets WHERE organization_id = $1 AND offset_number = $2"
        ).bind(org_id).bind(offset_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Offset '{}' not found", offset_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SustainabilityDashboard> {
        // Count facilities
        let fac_rows = sqlx::query(
            "SELECT status FROM _atlas.sustainability_facilities WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_facilities = fac_rows.len() as i32;
        let active_facilities = fac_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .count() as i32;

        // Aggregate emissions
        let act_rows = sqlx::query(
            "SELECT scope, activity_type, co2e_kg FROM _atlas.environmental_activities WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut scope1: f64 = 0.0;
        let mut scope2: f64 = 0.0;
        let mut scope3: f64 = 0.0;
        let mut by_scope = std::collections::HashMap::new();
        let mut by_category = std::collections::HashMap::new();

        for row in &act_rows {
            let scope: String = row.try_get("scope").unwrap_or_default();
            let atype: String = row.try_get("activity_type").unwrap_or_default();
            let co2e: f64 = get_numeric(row, "co2e_kg");

            match scope.as_str() {
                "scope_1" => scope1 += co2e,
                "scope_2" => scope2 += co2e,
                "scope_3" => scope3 += co2e,
                _ => {}
            }
            *by_scope.entry(scope).or_insert(0.0f64) += co2e;
            *by_category.entry(atype).or_insert(0.0f64) += co2e;
        }

        // Energy, water, waste from activities with specific types
        let energy_rows = sqlx::query(
            r#"SELECT COALESCE(SUM(quantity), 0) as total FROM _atlas.environmental_activities
               WHERE organization_id = $1 AND activity_type IN ('electricity', 'natural_gas', 'diesel', 'gasoline')"#,
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();
        let total_energy: f64 = get_numeric(&energy_rows, "total");

        let water_rows = sqlx::query(
            r#"SELECT COALESCE(SUM(quantity), 0) as total FROM _atlas.environmental_activities
               WHERE organization_id = $1 AND activity_type = 'water'"#,
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();
        let total_water: f64 = get_numeric(&water_rows, "total");

        let waste_rows = sqlx::query(
            r#"SELECT COALESCE(SUM(quantity), 0) as total FROM _atlas.environmental_activities
               WHERE organization_id = $1 AND activity_type = 'waste'"#,
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();
        let total_waste: f64 = get_numeric(&waste_rows, "total");

        // Carbon offsets
        let offset_rows = sqlx::query(
            "SELECT remaining_tonnes, retired_quantity FROM _atlas.carbon_offsets WHERE organization_id = $1 AND status IN ('active', 'retired')"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_offsets: f64 = offset_rows.iter()
            .map(|r| get_numeric(r, "retired_quantity"))
            .sum();

        // Goals
        let goal_rows = sqlx::query(
            "SELECT status FROM _atlas.sustainability_goals WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let active_goals = goal_rows.len() as i32;
        let goals_on_track = goal_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "on_track")
            .count() as i32;
        let goals_achieved = goal_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "achieved")
            .count() as i32;

        let mut goals_by_status = std::collections::HashMap::new();
        for row in &goal_rows {
            let s: String = row.try_get("status").unwrap_or_default();
            *goals_by_status.entry(s).or_insert(0i32) += 1;
        }

        // ESG metrics count
        let metric_count: i32 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.esg_metrics WHERE organization_id = $1 AND status = 'active'"
        ).bind(org_id).fetch_one(&self.pool).await
            .map(|r| r.try_get("cnt").unwrap_or(0))
            .unwrap_or(0);

        let total_co2e = scope1 + scope2 + scope3;
        let net_emissions = (total_co2e / 1000.0) - total_offsets;

        Ok(SustainabilityDashboard {
            total_facilities,
            active_facilities,
            total_emissions_co2e_tonnes: total_co2e / 1000.0,
            scope1_emissions_tonnes: scope1 / 1000.0,
            scope2_emissions_tonnes: scope2 / 1000.0,
            scope3_emissions_tonnes: scope3 / 1000.0,
            total_energy_consumed_kwh: total_energy,
            renewable_energy_pct: 0.0,
            total_water_consumed_cubic_m: total_water,
            total_waste_generated_tonnes: total_waste,
            waste_diverted_pct: 0.0,
            total_offsets_tonnes: total_offsets,
            net_emissions_tonnes: net_emissions.max(0.0),
            active_goals,
            goals_on_track,
            goals_achieved,
            esg_metrics_count: metric_count,
            emissions_by_scope: serde_json::to_value(&by_scope).unwrap_or_default(),
            emissions_by_category: serde_json::to_value(&by_category).unwrap_or_default(),
            emissions_trend: serde_json::json!({}),
            goals_by_status: serde_json::to_value(&goals_by_status).unwrap_or_default(),
        })
    }
}
