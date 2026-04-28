//! Enterprise Asset Management Engine
//!
//! Manages physical assets, work orders, preventive maintenance schedules,
//! and maintenance dashboard KPIs.
//!
//! Oracle Fusion Cloud equivalent: Maintenance Management > Work Orders,
//! Preventive Maintenance, Asset Definition

use atlas_shared::{
    AssetDefinition, MaintenanceWorkOrder, PreventiveMaintenanceSchedule, MaintenanceDashboard,
    AtlasError, AtlasResult,
};
use super::AssetManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_ASSET_GROUPS: &[&str] = &[
    "pump", "motor", "vehicle", "hvac", "electrical", "plumbing",
    "conveyor", "compressor", "generator", "it_equipment", "general",
];

const VALID_ASSET_CRITICALITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

const VALID_ASSET_STATUSES: &[&str] = &[
    "active", "inactive", "disposed", "in_repair",
];

const VALID_WORK_ORDER_TYPES: &[&str] = &[
    "corrective", "preventive", "emergency", "inspection", "project",
];

const VALID_PRIORITIES: &[&str] = &[
    "low", "normal", "high", "urgent",
];

const VALID_WORK_ORDER_STATUSES: &[&str] = &[
    "draft", "approved", "in_progress", "completed", "closed", "cancelled",
];

const VALID_FAILURE_CODES: &[&str] = &[
    "mechanical_failure", "electrical_failure", "corrosion", "wear",
    "overheating", "leak", "vibration", "contamination", "misalignment",
    "fatigue", "overload", "software_fault", "operator_error", "other",
];

const VALID_CAUSE_CODES: &[&str] = &[
    "normal_wear", "inadequate_maintenance", "improper_operation",
    "design_defect", "material_defect", "environmental", "overuse", "other",
];

const VALID_RESOLUTION_CODES: &[&str] = &[
    "repaired", "replaced", "adjusted", "calibrated", "lubricated",
    "cleaned", "refurbished", "no_action_needed", "other",
];

const VALID_SCHEDULE_TYPES: &[&str] = &[
    "time_based", "meter_based", "condition_based",
];

const VALID_FREQUENCIES: &[&str] = &[
    "daily", "weekly", "monthly", "quarterly", "semi_annual", "annual",
];

const VALID_INTERVAL_UNITS: &[&str] = &[
    "days", "weeks", "months", "hours", "miles", "km", "cycles",
];

const VALID_METER_TYPES: &[&str] = &[
    "hours", "miles", "km", "cycles",
];

const VALID_SCHEDULE_STATUSES: &[&str] = &[
    "active", "inactive", "completed",
];

const VALID_LOCATION_TYPES: &[&str] = &[
    "building", "floor", "room", "area", "outdoor", "warehouse",
    "production_line", "station",
];

/// Enterprise Asset Management Engine
pub struct EnterpriseAssetManagementEngine {
    repository: Arc<dyn AssetManagementRepository>,
}

impl EnterpriseAssetManagementEngine {
    pub fn new(repository: Arc<dyn AssetManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Asset Locations
    // ========================================================================

    /// Create an asset location
    pub async fn create_location(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_location_id: Option<Uuid>,
        location_type: Option<&str>,
        address: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<serde_json::Value> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Location code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Location name is required".to_string(),
            ));
        }
        let loc_type = location_type.unwrap_or("building");
        if !VALID_LOCATION_TYPES.contains(&loc_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid location_type '{}'. Must be one of: {}", loc_type, VALID_LOCATION_TYPES.join(", ")
            )));
        }
        if self.repository.get_location_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Location '{}' already exists", code_upper
            )));
        }

        info!("Creating asset location '{}' ({}) for org {}", code_upper, name, org_id);
        self.repository.create_location(
            org_id, &code_upper, name, description,
            parent_location_id, loc_type, address, created_by,
        ).await
    }

    /// Get a location by ID
    pub async fn get_location(&self, id: Uuid) -> AtlasResult<Option<serde_json::Value>> {
        self.repository.get_location(id).await
    }

    /// List locations for an organization
    pub async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<serde_json::Value>> {
        self.repository.list_locations(org_id).await
    }

    /// Delete a location by code
    pub async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting location '{}' for org {}", code, org_id);
        self.repository.delete_location(org_id, code).await
    }

    // ========================================================================
    // Asset Definitions
    // ========================================================================

    /// Create a physical asset definition
    #[allow(clippy::too_many_arguments)]
    pub async fn create_asset(
        &self,
        org_id: Uuid,
        asset_number: &str,
        name: &str,
        description: Option<&str>,
        asset_group: &str,
        asset_criticality: &str,
        location_id: Option<Uuid>,
        location_name: Option<&str>,
        parent_asset_id: Option<Uuid>,
        serial_number: Option<&str>,
        manufacturer: Option<&str>,
        model: Option<&str>,
        install_date: Option<chrono::NaiveDate>,
        warranty_expiry: Option<chrono::NaiveDate>,
        meter_reading: Option<serde_json::Value>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDefinition> {
        if asset_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Asset number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Asset name is required".to_string()));
        }
        if !VALID_ASSET_GROUPS.contains(&asset_group) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid asset_group '{}'. Must be one of: {}", asset_group, VALID_ASSET_GROUPS.join(", ")
            )));
        }
        if !VALID_ASSET_CRITICALITIES.contains(&asset_criticality) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid asset_criticality '{}'. Must be one of: {}",
                asset_criticality, VALID_ASSET_CRITICALITIES.join(", ")
            )));
        }

        if self.repository.get_asset_by_number(org_id, asset_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Asset '{}' already exists", asset_number
            )));
        }

        info!("Creating asset '{}' ({}) for org {} [group={}, criticality={}]",
              asset_number, name, org_id, asset_group, asset_criticality);

        self.repository.create_asset(
            org_id, asset_number, name, description,
            asset_group, asset_criticality,
            location_id, location_name,
            parent_asset_id,
            serial_number.unwrap_or(""),
            manufacturer.unwrap_or(""),
            model.unwrap_or(""),
            install_date, warranty_expiry,
            meter_reading.unwrap_or(serde_json::json!({})),
            created_by,
        ).await
    }

    /// Get an asset by ID
    pub async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<AssetDefinition>> {
        self.repository.get_asset(id).await
    }

    /// Get an asset by number
    pub async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<AssetDefinition>> {
        self.repository.get_asset_by_number(org_id, asset_number).await
    }

    /// List assets with optional filters
    pub async fn list_assets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        asset_group: Option<&str>,
        criticality: Option<&str>,
    ) -> AtlasResult<Vec<AssetDefinition>> {
        self.repository.list_assets(org_id, status, asset_group, criticality).await
    }

    /// Update asset status
    pub async fn update_asset_status(&self, id: Uuid, status: &str) -> AtlasResult<AssetDefinition> {
        if !VALID_ASSET_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid asset status '{}'. Must be one of: {}", status, VALID_ASSET_STATUSES.join(", ")
            )));
        }
        info!("Updating asset {} status to {}", id, status);
        self.repository.update_asset_status(id, status).await
    }

    /// Update asset meter reading
    pub async fn update_asset_meter(
        &self,
        id: Uuid,
        meter_reading: serde_json::Value,
    ) -> AtlasResult<AssetDefinition> {
        info!("Updating asset {} meter reading", id);
        self.repository.update_asset_meter(id, meter_reading).await
    }

    /// Delete an asset by number
    pub async fn delete_asset(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<()> {
        info!("Deleting asset '{}' for org {}", asset_number, org_id);
        self.repository.delete_asset(org_id, asset_number).await
    }

    // ========================================================================
    // Work Orders
    // ========================================================================

    /// Create a work order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_work_order(
        &self,
        org_id: Uuid,
        work_order_number: &str,
        title: &str,
        description: Option<&str>,
        work_order_type: &str,
        priority: &str,
        asset_id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        scheduled_start: Option<chrono::NaiveDate>,
        scheduled_end: Option<chrono::NaiveDate>,
        estimated_hours: Option<serde_json::Value>,
        estimated_cost: Option<&str>,
        failure_code: Option<&str>,
        cause_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MaintenanceWorkOrder> {
        if work_order_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Work order number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Work order title is required".to_string()));
        }
        if !VALID_WORK_ORDER_TYPES.contains(&work_order_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid work_order_type '{}'. Must be one of: {}",
                work_order_type, VALID_WORK_ORDER_TYPES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        if let Some(fc) = failure_code {
            if !VALID_FAILURE_CODES.contains(&fc) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid failure_code '{}'. Must be one of: {}", fc, VALID_FAILURE_CODES.join(", ")
                )));
            }
        }
        if let Some(cc) = cause_code {
            if !VALID_CAUSE_CODES.contains(&cc) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid cause_code '{}'. Must be one of: {}", cc, VALID_CAUSE_CODES.join(", ")
                )));
            }
        }
        if let (Some(start), Some(end)) = (scheduled_start, scheduled_end) {
            if start > end {
                return Err(AtlasError::ValidationFailed(
                    "Scheduled start must be before scheduled end".to_string(),
                ));
            }
        }

        // Verify asset exists and get its info
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Asset {} not found", asset_id)))?;

        if self.repository.get_work_order_by_number(org_id, work_order_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Work order '{}' already exists", work_order_number
            )));
        }

        info!("Creating work order '{}' ({}) for org {} [type={}, priority={}, asset={}]",
              work_order_number, title, org_id, work_order_type, priority, asset.asset_number);

        self.repository.create_work_order(
            org_id, work_order_number, title, description,
            work_order_type, priority,
            asset_id, &asset.asset_number, &asset.name, &asset.location_name,
            assigned_to, assigned_to_name,
            scheduled_start, scheduled_end,
            estimated_hours.unwrap_or(serde_json::json!({})),
            estimated_cost.unwrap_or("0.00"),
            failure_code.unwrap_or(""),
            cause_code.unwrap_or(""),
            created_by,
        ).await
    }

    /// Get a work order by ID
    pub async fn get_work_order(&self, id: Uuid) -> AtlasResult<Option<MaintenanceWorkOrder>> {
        self.repository.get_work_order(id).await
    }

    /// Get a work order by number
    pub async fn get_work_order_by_number(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<Option<MaintenanceWorkOrder>> {
        self.repository.get_work_order_by_number(org_id, wo_number).await
    }

    /// List work orders with optional filters
    pub async fn list_work_orders(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        work_order_type: Option<&str>,
        priority: Option<&str>,
        asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<MaintenanceWorkOrder>> {
        self.repository.list_work_orders(org_id, status, work_order_type, priority, asset_id).await
    }

    /// Update work order status
    pub async fn update_work_order_status(&self, id: Uuid, status: &str) -> AtlasResult<MaintenanceWorkOrder> {
        if !VALID_WORK_ORDER_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid work order status '{}'. Must be one of: {}", status, VALID_WORK_ORDER_STATUSES.join(", ")
            )));
        }
        info!("Updating work order {} status to {}", id, status);
        self.repository.update_work_order_status(id, status).await
    }

    /// Complete a work order with actuals
    #[allow(clippy::too_many_arguments)]
    pub async fn complete_work_order(
        &self,
        id: Uuid,
        actual_cost: Option<&str>,
        actual_hours: Option<serde_json::Value>,
        downtime_hours: Option<f64>,
        resolution_code: Option<&str>,
        completion_notes: Option<&str>,
        materials: Option<serde_json::Value>,
        labor: Option<serde_json::Value>,
    ) -> AtlasResult<MaintenanceWorkOrder> {
        if let Some(rc) = resolution_code {
            if !VALID_RESOLUTION_CODES.contains(&rc) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid resolution_code '{}'. Must be one of: {}",
                    rc, VALID_RESOLUTION_CODES.join(", ")
                )));
            }
        }
        info!("Completing work order {}", id);
        self.repository.complete_work_order(
            id,
            actual_cost.unwrap_or("0.00"),
            actual_hours.unwrap_or(serde_json::json!({})),
            downtime_hours.unwrap_or(0.0),
            resolution_code.unwrap_or(""),
            completion_notes.unwrap_or(""),
            materials.unwrap_or(serde_json::json!([])),
            labor.unwrap_or(serde_json::json!([])),
        ).await
    }

    /// Delete a work order by number
    pub async fn delete_work_order(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<()> {
        info!("Deleting work order '{}' for org {}", wo_number, org_id);
        self.repository.delete_work_order(org_id, wo_number).await
    }

    // ========================================================================
    // Preventive Maintenance Schedules
    // ========================================================================

    /// Create a preventive maintenance schedule
    #[allow(clippy::too_many_arguments)]
    pub async fn create_pm_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        asset_id: Uuid,
        schedule_type: &str,
        frequency: Option<&str>,
        interval_value: Option<i32>,
        interval_unit: Option<&str>,
        meter_type: Option<&str>,
        meter_threshold: Option<serde_json::Value>,
        work_order_template: Option<serde_json::Value>,
        estimated_duration_hours: Option<f64>,
        estimated_cost: Option<&str>,
        auto_generate: bool,
        lead_time_days: Option<i32>,
        effective_start: Option<chrono::NaiveDate>,
        effective_end: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PreventiveMaintenanceSchedule> {
        if schedule_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule name is required".to_string()));
        }
        if !VALID_SCHEDULE_TYPES.contains(&schedule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule_type '{}'. Must be one of: {}", schedule_type, VALID_SCHEDULE_TYPES.join(", ")
            )));
        }
        let freq = frequency.unwrap_or("monthly");
        if !VALID_FREQUENCIES.contains(&freq) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid frequency '{}'. Must be one of: {}", freq, VALID_FREQUENCIES.join(", ")
            )));
        }
        let iu = interval_unit.unwrap_or("months");
        if !VALID_INTERVAL_UNITS.contains(&iu) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid interval_unit '{}'. Must be one of: {}", iu, VALID_INTERVAL_UNITS.join(", ")
            )));
        }
        if let Some(mt) = meter_type {
            if !VALID_METER_TYPES.contains(&mt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid meter_type '{}'. Must be one of: {}", mt, VALID_METER_TYPES.join(", ")
                )));
            }
        }

        // Verify asset exists
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Asset {} not found", asset_id)))?;

        if self.repository.get_pm_schedule_by_number(org_id, schedule_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Schedule '{}' already exists", schedule_number
            )));
        }

        info!("Creating PM schedule '{}' ({}) for org {} [type={}, asset={}]",
              schedule_number, name, org_id, schedule_type, asset.asset_number);

        self.repository.create_pm_schedule(
            org_id, schedule_number, name, description,
            asset_id, &asset.asset_number, &asset.name,
            schedule_type, freq,
            interval_value.unwrap_or(1), iu,
            meter_type.unwrap_or(""),
            meter_threshold.unwrap_or(serde_json::json!({})),
            work_order_template.unwrap_or(serde_json::json!({})),
            estimated_duration_hours.unwrap_or(0.0),
            estimated_cost.unwrap_or("0.00"),
            auto_generate,
            lead_time_days.unwrap_or(7),
            effective_start, effective_end,
            created_by,
        ).await
    }

    /// Get a PM schedule by ID
    pub async fn get_pm_schedule(&self, id: Uuid) -> AtlasResult<Option<PreventiveMaintenanceSchedule>> {
        self.repository.get_pm_schedule(id).await
    }

    /// Get a PM schedule by number
    pub async fn get_pm_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<PreventiveMaintenanceSchedule>> {
        self.repository.get_pm_schedule_by_number(org_id, schedule_number).await
    }

    /// List PM schedules with optional filters
    pub async fn list_pm_schedules(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PreventiveMaintenanceSchedule>> {
        self.repository.list_pm_schedules(org_id, status, asset_id).await
    }

    /// Update PM schedule status
    pub async fn update_pm_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<PreventiveMaintenanceSchedule> {
        if !VALID_SCHEDULE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule status '{}'. Must be one of: {}", status, VALID_SCHEDULE_STATUSES.join(", ")
            )));
        }
        info!("Updating PM schedule {} status to {}", id, status);
        self.repository.update_pm_schedule_status(id, status).await
    }

    /// Delete a PM schedule by number
    pub async fn delete_pm_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        info!("Deleting PM schedule '{}' for org {}", schedule_number, org_id);
        self.repository.delete_pm_schedule(org_id, schedule_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the maintenance dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MaintenanceDashboard> {
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
    fn test_valid_asset_groups() {
        assert!(VALID_ASSET_GROUPS.contains(&"pump"));
        assert!(VALID_ASSET_GROUPS.contains(&"motor"));
        assert!(VALID_ASSET_GROUPS.contains(&"vehicle"));
        assert!(VALID_ASSET_GROUPS.contains(&"hvac"));
        assert!(VALID_ASSET_GROUPS.contains(&"general"));
        assert!(!VALID_ASSET_GROUPS.contains(&"invalid"));
    }

    #[test]
    fn test_valid_asset_criticalities() {
        assert!(VALID_ASSET_CRITICALITIES.contains(&"low"));
        assert!(VALID_ASSET_CRITICALITIES.contains(&"medium"));
        assert!(VALID_ASSET_CRITICALITIES.contains(&"high"));
        assert!(VALID_ASSET_CRITICALITIES.contains(&"critical"));
        assert!(!VALID_ASSET_CRITICALITIES.contains(&"extreme"));
    }

    #[test]
    fn test_valid_asset_statuses() {
        assert!(VALID_ASSET_STATUSES.contains(&"active"));
        assert!(VALID_ASSET_STATUSES.contains(&"inactive"));
        assert!(VALID_ASSET_STATUSES.contains(&"disposed"));
        assert!(VALID_ASSET_STATUSES.contains(&"in_repair"));
    }

    #[test]
    fn test_valid_work_order_types() {
        assert!(VALID_WORK_ORDER_TYPES.contains(&"corrective"));
        assert!(VALID_WORK_ORDER_TYPES.contains(&"preventive"));
        assert!(VALID_WORK_ORDER_TYPES.contains(&"emergency"));
        assert!(VALID_WORK_ORDER_TYPES.contains(&"inspection"));
        assert!(VALID_WORK_ORDER_TYPES.contains(&"project"));
    }

    #[test]
    fn test_valid_work_order_statuses() {
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"draft"));
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"approved"));
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"in_progress"));
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"completed"));
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"closed"));
        assert!(VALID_WORK_ORDER_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_schedule_types() {
        assert!(VALID_SCHEDULE_TYPES.contains(&"time_based"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"meter_based"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"condition_based"));
    }

    #[test]
    fn test_valid_frequencies() {
        assert!(VALID_FREQUENCIES.contains(&"daily"));
        assert!(VALID_FREQUENCIES.contains(&"weekly"));
        assert!(VALID_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_FREQUENCIES.contains(&"annual"));
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"normal"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"urgent"));
    }

    #[test]
    fn test_valid_failure_codes() {
        assert!(VALID_FAILURE_CODES.contains(&"mechanical_failure"));
        assert!(VALID_FAILURE_CODES.contains(&"electrical_failure"));
        assert!(VALID_FAILURE_CODES.contains(&"overheating"));
        assert!(VALID_FAILURE_CODES.contains(&"leak"));
    }

    #[test]
    fn test_valid_resolution_codes() {
        assert!(VALID_RESOLUTION_CODES.contains(&"repaired"));
        assert!(VALID_RESOLUTION_CODES.contains(&"replaced"));
        assert!(VALID_RESOLUTION_CODES.contains(&"adjusted"));
        assert!(VALID_RESOLUTION_CODES.contains(&"calibrated"));
    }

    #[test]
    fn test_valid_location_types() {
        assert!(VALID_LOCATION_TYPES.contains(&"building"));
        assert!(VALID_LOCATION_TYPES.contains(&"floor"));
        assert!(VALID_LOCATION_TYPES.contains(&"room"));
        assert!(VALID_LOCATION_TYPES.contains(&"production_line"));
    }

    #[test]
    fn test_valid_meter_types() {
        assert!(VALID_METER_TYPES.contains(&"hours"));
        assert!(VALID_METER_TYPES.contains(&"miles"));
        assert!(VALID_METER_TYPES.contains(&"km"));
        assert!(VALID_METER_TYPES.contains(&"cycles"));
    }

    #[test]
    fn test_valid_interval_units() {
        assert!(VALID_INTERVAL_UNITS.contains(&"days"));
        assert!(VALID_INTERVAL_UNITS.contains(&"weeks"));
        assert!(VALID_INTERVAL_UNITS.contains(&"months"));
        assert!(VALID_INTERVAL_UNITS.contains(&"hours"));
    }
}
