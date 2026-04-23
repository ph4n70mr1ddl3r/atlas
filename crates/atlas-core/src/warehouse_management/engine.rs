//! Warehouse Management Engine
//!
//! Oracle Fusion Cloud Warehouse Management

use atlas_shared::{
    Warehouse, WarehouseZone, PutAwayRule, WarehouseTask, PickWave, WarehouseDashboard,
    AtlasError, AtlasResult,
};
use super::WarehouseManagementRepository;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

/// Engine for managing warehouse operations
pub struct WarehouseManagementEngine {
    repository: Arc<dyn WarehouseManagementRepository>,
}

impl WarehouseManagementEngine {
    pub fn new(repository: Arc<dyn WarehouseManagementRepository>) -> Self {
        Self { repository }
    }

    // ─── Warehouses ─────────────────────────────────────────────────────────

    /// Create a new warehouse
    pub async fn create_warehouse(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        location_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Warehouse> {
        info!("Creating warehouse '{}' ({})", name, code);

        if code.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Warehouse code cannot be empty".to_string()));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Warehouse name cannot be empty".to_string()));
        }

        self.repository.create_warehouse(org_id, code, name, description, location_code, created_by).await
    }

    /// Get a warehouse by ID
    pub async fn get_warehouse(&self, id: Uuid) -> AtlasResult<Option<Warehouse>> {
        self.repository.get_warehouse(id).await
    }

    /// Get a warehouse by code
    pub async fn get_warehouse_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Warehouse>> {
        self.repository.get_warehouse_by_code(org_id, code).await
    }

    /// List warehouses for an organization
    pub async fn list_warehouses(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<Warehouse>> {
        self.repository.list_warehouses(org_id, active_only).await
    }

    /// Delete a warehouse
    pub async fn delete_warehouse(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting warehouse {}", id);
        self.repository.delete_warehouse(id).await
    }

    // ─── Zones ──────────────────────────────────────────────────────────────

    /// Create a zone within a warehouse
    pub async fn create_zone(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        code: &str,
        name: &str,
        zone_type: &str,
        description: Option<&str>,
        aisle_count: Option<i32>,
    ) -> AtlasResult<WarehouseZone> {
        info!("Creating zone '{}' ({}) in warehouse {}", name, code, warehouse_id);

        let valid_zone_types = ["receiving", "storage", "picking", "packing", "staging", "shipping"];
        if !valid_zone_types.contains(&zone_type) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid zone_type '{}'. Must be one of: {}", zone_type, valid_zone_types.join(", "))
            ));
        }

        // Verify warehouse exists
        let warehouse = self.repository.get_warehouse(warehouse_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)))?;

        if warehouse.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)));
        }

        self.repository.create_zone(org_id, warehouse_id, code, name, zone_type, description, aisle_count).await
    }

    /// Get a zone by ID
    pub async fn get_zone(&self, id: Uuid) -> AtlasResult<Option<WarehouseZone>> {
        self.repository.get_zone(id).await
    }

    /// List zones for a warehouse
    pub async fn list_zones(&self, warehouse_id: Uuid) -> AtlasResult<Vec<WarehouseZone>> {
        self.repository.list_zones(warehouse_id).await
    }

    /// Delete a zone
    pub async fn delete_zone(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_zone(id).await
    }

    // ─── Put-Away Rules ─────────────────────────────────────────────────────

    /// Create a put-away rule
    pub async fn create_put_away_rule(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        rule_name: &str,
        description: Option<&str>,
        priority: i32,
        item_category: Option<&str>,
        target_zone_type: &str,
        strategy: &str,
    ) -> AtlasResult<PutAwayRule> {
        info!("Creating put-away rule '{}' for warehouse {}", rule_name, warehouse_id);

        let valid_strategies = ["closest", "zone_rotation", "fixed_location"];
        if !valid_strategies.contains(&strategy) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid strategy '{}'. Must be one of: {}", strategy, valid_strategies.join(", "))
            ));
        }

        let valid_zone_types = ["receiving", "storage", "picking", "packing", "staging", "shipping"];
        if !valid_zone_types.contains(&target_zone_type) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid target_zone_type '{}'. Must be one of: {}", target_zone_type, valid_zone_types.join(", "))
            ));
        }

        // Verify warehouse exists
        let warehouse = self.repository.get_warehouse(warehouse_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)))?;

        if warehouse.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)));
        }

        self.repository.create_put_away_rule(org_id, warehouse_id, rule_name, description, priority, item_category, target_zone_type, strategy).await
    }

    /// Get a put-away rule by ID
    pub async fn get_put_away_rule(&self, id: Uuid) -> AtlasResult<Option<PutAwayRule>> {
        self.repository.get_put_away_rule(id).await
    }

    /// List put-away rules for a warehouse
    pub async fn list_put_away_rules(&self, warehouse_id: Uuid) -> AtlasResult<Vec<PutAwayRule>> {
        self.repository.list_put_away_rules(warehouse_id).await
    }

    /// Delete a put-away rule
    pub async fn delete_put_away_rule(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_put_away_rule(id).await
    }

    // ─── Warehouse Tasks ────────────────────────────────────────────────────

    /// Create a warehouse task
    #[allow(clippy::too_many_arguments)]
    pub async fn create_task(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        task_number: &str,
        task_type: &str,
        priority: &str,
        item_id: Option<Uuid>,
        item_description: Option<&str>,
        from_zone_id: Option<Uuid>,
        to_zone_id: Option<Uuid>,
        from_location: Option<&str>,
        to_location: Option<&str>,
        quantity: Option<String>,
        uom: Option<&str>,
        source_document: Option<&str>,
        source_document_id: Option<Uuid>,
        source_line_id: Option<Uuid>,
        wave_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WarehouseTask> {
        info!("Creating warehouse task '{}' of type '{}' in warehouse {}", task_number, task_type, warehouse_id);

        let valid_task_types = ["pick", "pack", "put_away", "load", "receive"];
        if !valid_task_types.contains(&task_type) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid task_type '{}'. Must be one of: {}", task_type, valid_task_types.join(", "))
            ));
        }

        let valid_priorities = ["low", "medium", "high", "urgent"];
        if !valid_priorities.contains(&priority) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid priority '{}'. Must be one of: {}", priority, valid_priorities.join(", "))
            ));
        }

        // Verify warehouse exists
        let warehouse = self.repository.get_warehouse(warehouse_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)))?;

        if warehouse.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)));
        }

        // Verify wave exists if specified
        if let Some(wid) = wave_id {
            let wave = self.repository.get_wave(wid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", wid)))?;
            if wave.status != "draft" && wave.status != "released" {
                return Err(AtlasError::WorkflowError(
                    format!("Cannot add task to wave in status '{}'. Wave must be draft or released.", wave.status)
                ));
            }
        }

        self.repository.create_task(
            org_id, warehouse_id, task_number, task_type, priority,
            item_id, item_description, from_zone_id, to_zone_id,
            from_location, to_location, quantity, uom,
            source_document, source_document_id, source_line_id,
            wave_id, created_by,
        ).await
    }

    /// Get a task by ID
    pub async fn get_task(&self, id: Uuid) -> AtlasResult<Option<WarehouseTask>> {
        self.repository.get_task(id).await
    }

    /// Get a task by task number
    pub async fn get_task_by_number(&self, org_id: Uuid, task_number: &str) -> AtlasResult<Option<WarehouseTask>> {
        self.repository.get_task_by_number(org_id, task_number).await
    }

    /// List tasks with optional filters
    pub async fn list_tasks(
        &self,
        org_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<&str>,
        task_type: Option<&str>,
    ) -> AtlasResult<Vec<WarehouseTask>> {
        self.repository.list_tasks(org_id, warehouse_id, status, task_type).await
    }

    /// Start a task (transition from pending to in_progress)
    pub async fn start_task(&self, id: Uuid, assigned_to: Option<Uuid>) -> AtlasResult<WarehouseTask> {
        info!("Starting warehouse task {}", id);

        let task = self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))?;

        if task.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot start task in status '{}'. Must be 'pending'.", task.status)
            ));
        }

        self.repository.update_task_status(id, "in_progress", assigned_to).await?;
        self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))
    }

    /// Complete a task
    pub async fn complete_task(&self, id: Uuid) -> AtlasResult<WarehouseTask> {
        info!("Completing warehouse task {}", id);

        let task = self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))?;

        if task.status != "in_progress" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete task in status '{}'. Must be 'in_progress'.", task.status)
            ));
        }

        self.repository.update_task_status(id, "completed", None).await?;
        self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))
    }

    /// Cancel a task
    pub async fn cancel_task(&self, id: Uuid) -> AtlasResult<WarehouseTask> {
        info!("Cancelling warehouse task {}", id);

        let task = self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))?;

        if task.status == "completed" || task.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel task in status '{}'.", task.status)
            ));
        }

        self.repository.update_task_status(id, "cancelled", None).await?;
        self.repository.get_task(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Task {}", id)))
    }

    /// Delete a task
    pub async fn delete_task(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_task(id).await
    }

    // ─── Pick Waves ─────────────────────────────────────────────────────────

    /// Create a pick wave
    pub async fn create_wave(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        wave_number: &str,
        priority: &str,
        cut_off_date: Option<chrono::NaiveDate>,
        shipping_method: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PickWave> {
        info!("Creating pick wave '{}' in warehouse {}", wave_number, warehouse_id);

        let valid_priorities = ["low", "medium", "high", "urgent"];
        if !valid_priorities.contains(&priority) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid priority '{}'. Must be one of: {}", priority, valid_priorities.join(", "))
            ));
        }

        // Verify warehouse exists
        let warehouse = self.repository.get_warehouse(warehouse_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)))?;

        if warehouse.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Warehouse {}", warehouse_id)));
        }

        self.repository.create_wave(org_id, warehouse_id, wave_number, priority, cut_off_date, shipping_method, created_by).await
    }

    /// Get a wave by ID
    pub async fn get_wave(&self, id: Uuid) -> AtlasResult<Option<PickWave>> {
        self.repository.get_wave(id).await
    }

    /// Get a wave by number
    pub async fn get_wave_by_number(&self, org_id: Uuid, wave_number: &str) -> AtlasResult<Option<PickWave>> {
        self.repository.get_wave_by_number(org_id, wave_number).await
    }

    /// List waves with optional filters
    pub async fn list_waves(
        &self,
        org_id: Uuid,
        warehouse_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PickWave>> {
        self.repository.list_waves(org_id, warehouse_id, status).await
    }

    /// Release a pick wave (draft -> released)
    pub async fn release_wave(&self, id: Uuid) -> AtlasResult<PickWave> {
        info!("Releasing pick wave {}", id);

        let wave = self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))?;

        if wave.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot release wave in status '{}'. Must be 'draft'.", wave.status)
            ));
        }

        self.repository.update_wave_status(id, "released").await?;
        self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))
    }

    /// Close a pick wave (mark as completed)
    pub async fn complete_wave(&self, id: Uuid) -> AtlasResult<PickWave> {
        info!("Completing pick wave {}", id);

        let wave = self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))?;

        if wave.status != "released" && wave.status != "in_progress" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete wave in status '{}'. Must be 'released' or 'in_progress'.", wave.status)
            ));
        }

        self.repository.update_wave_status(id, "completed").await?;
        self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))
    }

    /// Cancel a pick wave
    pub async fn cancel_wave(&self, id: Uuid) -> AtlasResult<PickWave> {
        info!("Cancelling pick wave {}", id);

        let wave = self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))?;

        if wave.status == "completed" || wave.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel wave in status '{}'.", wave.status)
            ));
        }

        self.repository.update_wave_status(id, "cancelled").await?;
        self.repository.get_wave(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Wave {}", id)))
    }

    /// Delete a wave
    pub async fn delete_wave(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_wave(id).await
    }

    // ─── Dashboard ──────────────────────────────────────────────────────────

    /// Get warehouse dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<WarehouseDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}
