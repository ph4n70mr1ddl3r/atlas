//! KPI & Embedded Analytics Engine
//!
//! Manages KPI definitions, data point recording, dashboards, and widgets.
//! Inspired by Oracle Fusion OTBI (Oracle Transactional Business Intelligence).
//!
//! Oracle Fusion equivalent: Analytics > KPI Library, Dashboards

use atlas_shared::{
    KpiDefinition, KpiDataPoint, Dashboard, DashboardWidget, KpiDashboardSummary,
    AtlasError, AtlasResult,
};
use super::KpiRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid direction values
const VALID_DIRECTIONS: &[&str] = &[
    "higher_is_better", "lower_is_better", "target_range",
];

/// Valid evaluation frequencies
const VALID_FREQUENCIES: &[&str] = &[
    "manual", "hourly", "daily", "weekly", "monthly",
];

/// Valid widget types
const VALID_WIDGET_TYPES: &[&str] = &[
    "kpi_card", "chart", "table", "gauge", "trend",
];

/// Valid units of measure
const VALID_UNITS: &[&str] = &[
    "number", "currency", "percent", "ratio", "count", "days", "hours",
];

/// KPI & Analytics Engine
pub struct KpiEngine {
    repository: Arc<dyn KpiRepository>,
}

impl KpiEngine {
    pub fn new(repository: Arc<dyn KpiRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // KPI Definition Management
    // ========================================================================

    /// Create a new KPI definition
    #[allow(clippy::too_many_arguments)]
    pub async fn create_kpi(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        unit_of_measure: &str,
        direction: &str,
        target_value: &str,
        warning_threshold: Option<&str>,
        critical_threshold: Option<&str>,
        data_source_query: Option<&str>,
        evaluation_frequency: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<KpiDefinition> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "KPI code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "KPI name is required".to_string(),
            ));
        }
        if category.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "KPI category is required".to_string(),
            ));
        }
        if !VALID_DIRECTIONS.contains(&direction) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid direction '{}'. Must be one of: {}", direction, VALID_DIRECTIONS.join(", ")
            )));
        }
        if !VALID_FREQUENCIES.contains(&evaluation_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid evaluation_frequency '{}'. Must be one of: {}", evaluation_frequency, VALID_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_UNITS.contains(&unit_of_measure) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid unit_of_measure '{}'. Must be one of: {}", unit_of_measure, VALID_UNITS.join(", ")
            )));
        }
        if target_value.parse::<f64>().is_err() {
            return Err(AtlasError::ValidationFailed(
                "Target value must be a valid number".to_string(),
            ));
        }
        if let Some(ref wt) = warning_threshold {
            if wt.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Warning threshold must be a valid number".to_string(),
                ));
            }
        }
        if let Some(ref ct) = critical_threshold {
            if ct.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Critical threshold must be a valid number".to_string(),
                ));
            }
        }

        // Check for duplicate code
        if self.repository.get_kpi_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "KPI with code '{}' already exists", code_upper
            )));
        }

        info!("Creating KPI '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_kpi(
            org_id, &code_upper, name, description, category, unit_of_measure,
            direction, target_value, warning_threshold, critical_threshold,
            data_source_query, evaluation_frequency, created_by,
        ).await
    }

    /// Get a KPI definition by ID
    pub async fn get_kpi(&self, id: Uuid) -> AtlasResult<Option<KpiDefinition>> {
        self.repository.get_kpi(id).await
    }

    /// Get a KPI definition by code
    pub async fn get_kpi_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<KpiDefinition>> {
        self.repository.get_kpi_by_code(org_id, code).await
    }

    /// List KPIs for an organization, optionally filtered by category
    pub async fn list_kpis(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<KpiDefinition>> {
        self.repository.list_kpis(org_id, category).await
    }

    /// Delete a KPI definition by code
    pub async fn delete_kpi(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting KPI '{}' for org {}", code, org_id);
        self.repository.delete_kpi(org_id, code).await
    }

    // ========================================================================
    // KPI Data Points
    // ========================================================================

    /// Record a new data point for a KPI
    #[allow(clippy::too_many_arguments)]
    pub async fn record_data_point(
        &self,
        org_id: Uuid,
        kpi_id: Uuid,
        value: &str,
        period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        recorded_by: Option<Uuid>,
    ) -> AtlasResult<KpiDataPoint> {
        if value.parse::<f64>().is_err() {
            return Err(AtlasError::ValidationFailed(
                "Value must be a valid number".to_string(),
            ));
        }

        // Verify KPI exists
        let kpi = self.repository.get_kpi(kpi_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("KPI {} not found", kpi_id)))?;

        if !kpi.is_active {
            return Err(AtlasError::ValidationFailed(
                "Cannot record data for inactive KPI".to_string(),
            ));
        }

        info!("Recording data point for KPI '{}' ({}): value={}", kpi.code, kpi.name, value);

        self.repository.record_data_point(
            org_id, kpi_id, value, period_start, period_end, notes, recorded_by,
        ).await
    }

    /// Get the latest data point for a KPI
    pub async fn get_latest_data_point(&self, kpi_id: Uuid) -> AtlasResult<Option<KpiDataPoint>> {
        self.repository.get_latest_data_point(kpi_id).await
    }

    /// List data points for a KPI with pagination
    pub async fn list_data_points(
        &self,
        kpi_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<KpiDataPoint>> {
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        self.repository.list_data_points(kpi_id, limit, offset).await
    }

    /// Delete a data point
    pub async fn delete_data_point(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_data_point(id).await
    }

    // ========================================================================
    // Dashboard Management
    // ========================================================================

    /// Create a new dashboard
    #[allow(clippy::too_many_arguments)]
    pub async fn create_dashboard(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        owner_id: Option<Uuid>,
        is_shared: bool,
        is_default: bool,
        layout_config: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Dashboard> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Dashboard code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Dashboard name is required".to_string(),
            ));
        }

        // Check for duplicate code
        if self.repository.get_dashboard_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Dashboard with code '{}' already exists", code_upper
            )));
        }

        info!("Creating dashboard '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_dashboard(
            org_id, &code_upper, name, description, owner_id,
            is_shared, is_default, layout_config, created_by,
        ).await
    }

    /// Get a dashboard by ID
    pub async fn get_dashboard(&self, id: Uuid) -> AtlasResult<Option<Dashboard>> {
        self.repository.get_dashboard(id).await
    }

    /// List dashboards for an organization
    pub async fn list_dashboards(
        &self,
        org_id: Uuid,
        owner_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Dashboard>> {
        self.repository.list_dashboards(org_id, owner_id).await
    }

    /// Delete a dashboard by code
    pub async fn delete_dashboard(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting dashboard '{}' for org {}", code, org_id);
        self.repository.delete_dashboard(org_id, code).await
    }

    // ========================================================================
    // Dashboard Widgets
    // ========================================================================

    /// Add a widget to a dashboard
    #[allow(clippy::too_many_arguments)]
    pub async fn add_widget(
        &self,
        dashboard_id: Uuid,
        kpi_id: Option<Uuid>,
        widget_type: &str,
        title: &str,
        position_row: i32,
        position_col: i32,
        width: i32,
        height: i32,
        display_config: serde_json::Value,
    ) -> AtlasResult<DashboardWidget> {
        if !VALID_WIDGET_TYPES.contains(&widget_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid widget_type '{}'. Must be one of: {}", widget_type, VALID_WIDGET_TYPES.join(", ")
            )));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Widget title is required".to_string(),
            ));
        }
        if width < 1 || height < 1 {
            return Err(AtlasError::ValidationFailed(
                "Widget width and height must be at least 1".to_string(),
            ));
        }

        // Verify dashboard exists
        let _dashboard = self.repository.get_dashboard(dashboard_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dashboard {} not found", dashboard_id)))?;

        // Verify KPI exists if specified
        if let Some(kid) = kpi_id {
            let _kpi = self.repository.get_kpi(kid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("KPI {} not found", kid)))?;
        }

        info!("Adding {} widget '{}' to dashboard {}", widget_type, title, dashboard_id);

        self.repository.add_widget(
            dashboard_id, kpi_id, widget_type, title,
            position_row, position_col, width, height, display_config,
        ).await
    }

    /// List widgets for a dashboard
    pub async fn list_widgets(&self, dashboard_id: Uuid) -> AtlasResult<Vec<DashboardWidget>> {
        self.repository.list_widgets(dashboard_id).await
    }

    /// Delete a widget
    pub async fn delete_widget(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_widget(id).await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get the KPI analytics dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<KpiDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}
