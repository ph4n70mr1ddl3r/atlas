//! Manufacturing Execution Engine
//!
//! Business logic for Oracle Fusion SCM > Manufacturing.
//! Handles work definitions (BOM + Routing), work orders, operations,
//! material requirements, and production completions.

use crate::manufacturing::repository::ManufacturingRepository;
use atlas_shared::{
    AtlasError, AtlasResult,
    WorkDefinition, WorkDefinitionComponent, WorkDefinitionOperation,
    WorkOrder, WorkOrderOperation, WorkOrderMaterial,
    CreateWorkDefinitionRequest, CreateWorkOrderRequest,
    ReportCompletionRequest, IssueMaterialRequest,
    ManufacturingDashboard,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid work definition statuses
const VALID_DEFINITION_STATUSES: &[&str] = &["draft", "active", "inactive", "obsolete"];

/// Valid work order statuses (Oracle Fusion manufacturing lifecycle)
const VALID_WORK_ORDER_STATUSES: &[&str] = &[
    "draft", "released", "started", "completed", "closed", "cancelled",
];

/// Valid priorities
const VALID_PRIORITIES: &[&str] = &["low", "normal", "high", "urgent"];

/// Valid production types
const VALID_PRODUCTION_TYPES: &[&str] = &["discrete", "process", "repetitive"];

/// Valid work order operation statuses
const VALID_OPERATION_STATUSES: &[&str] = &[
    "pending", "in_queue", "running", "completed", "skipped", "error",
];

/// Valid material statuses
const VALID_MATERIAL_STATUSES: &[&str] = &[
    "pending", "partially_issued", "fully_issued", "returned", "short",
];

/// Manufacturing Execution Engine
pub struct ManufacturingEngine {
    repository: Arc<dyn ManufacturingRepository>,
}

impl ManufacturingEngine {
    pub fn new(repository: Arc<dyn ManufacturingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Work Definitions
    // ========================================================================

    /// Create a new work definition (BOM + Routing template)
    pub async fn create_work_definition(
        &self,
        org_id: Uuid,
        req: CreateWorkDefinitionRequest,
    ) -> AtlasResult<WorkDefinition> {
        // Validate production type
        let production_type = req.production_type.as_deref().unwrap_or("discrete");
        if !VALID_PRODUCTION_TYPES.contains(&production_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid production type '{}'. Must be one of: {}", production_type, VALID_PRODUCTION_TYPES.join(", ")
            )));
        }

        // Generate definition number if not provided
        let definition_number = req.definition_number.unwrap_or_else(|| {
            format!("WD-{}", Uuid::new_v4().to_string()[..8].to_uppercase())
        });

        info!("Creating work definition {} for org {}", definition_number, org_id);

        self.repository.create_work_definition(
            org_id, &definition_number, req.description.as_deref(),
            req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
            production_type,
            req.planning_type.as_deref().unwrap_or("make_to_order"),
            req.standard_lot_size.as_deref().unwrap_or("1"),
            req.unit_of_measure.as_deref().unwrap_or("EA"),
            req.lead_time_days.unwrap_or(0),
            req.cost_type.as_deref().unwrap_or("standard"),
            req.standard_cost.as_deref().unwrap_or("0"),
            req.effective_from, req.effective_to,
            req.created_by,
        ).await
    }

    /// Get a work definition by number
    pub async fn get_work_definition(&self, org_id: Uuid, definition_number: &str) -> AtlasResult<Option<WorkDefinition>> {
        self.repository.get_work_definition(org_id, definition_number).await
    }

    /// Get a work definition by ID
    pub async fn get_work_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkDefinition>> {
        self.repository.get_work_definition_by_id(id).await
    }

    /// List work definitions with optional status filter
    pub async fn list_work_definitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WorkDefinition>> {
        if let Some(s) = status {
            if !VALID_DEFINITION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_DEFINITION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_work_definitions(org_id, status).await
    }

    /// Activate a work definition (makes it available for creating work orders)
    pub async fn activate_work_definition(&self, id: Uuid) -> AtlasResult<WorkDefinition> {
        let def = self.repository.get_work_definition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", id)))?;

        if def.status != "draft" && def.status != "inactive" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate work definition in '{}' status. Must be 'draft' or 'inactive'.",
                def.status
            )));
        }

        // Check that the definition has at least one component and one operation
        let components = self.repository.list_work_definition_components(id).await?;
        let operations = self.repository.list_work_definition_operations(id).await?;

        if components.is_empty() {
            return Err(AtlasError::WorkflowError(
                "Cannot activate work definition without any components (BOM). Add at least one component.".to_string(),
            ));
        }
        if operations.is_empty() {
            return Err(AtlasError::WorkflowError(
                "Cannot activate work definition without any operations (Routing). Add at least one operation.".to_string(),
            ));
        }

        info!("Activating work definition {}", def.definition_number);
        self.repository.update_work_definition_status(id, "active").await
    }

    /// Deactivate a work definition
    pub async fn deactivate_work_definition(&self, id: Uuid) -> AtlasResult<WorkDefinition> {
        let def = self.repository.get_work_definition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", id)))?;

        if def.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deactivate work definition in '{}' status. Must be 'active'.",
                def.status
            )));
        }

        info!("Deactivating work definition {}", def.definition_number);
        self.repository.update_work_definition_status(id, "inactive").await
    }

    /// Delete a work definition (only draft)
    pub async fn delete_work_definition(&self, id: Uuid) -> AtlasResult<()> {
        let def = self.repository.get_work_definition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", id)))?;

        if def.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot delete work definition in '{}' status. Must be 'draft'.",
                def.status
            )));
        }

        info!("Deleting work definition {}", def.definition_number);
        self.repository.delete_work_definition(id).await
    }

    // ========================================================================
    // Work Definition Components (BOM)
    // ========================================================================

    /// Add a component to a work definition (BOM line)
    pub async fn add_work_definition_component(
        &self,
        org_id: Uuid,
        work_definition_id: Uuid,
        component_item_code: &str,
        quantity_required: &str,
        unit_of_measure: &str,
    ) -> AtlasResult<WorkDefinitionComponent> {
        if component_item_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Component item code is required".to_string()));
        }

        let qty: f64 = quantity_required.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity required must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity required must be positive".to_string()));
        }

        // Verify definition exists
        let def = self.repository.get_work_definition_by_id(work_definition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", work_definition_id)))?;

        if def.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot modify components of a non-draft work definition".to_string(),
            ));
        }

        // Determine next line number
        let existing = self.repository.list_work_definition_components(work_definition_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding component {} to work definition {}", component_item_code, def.definition_number);

        self.repository.add_work_definition_component(
            org_id, work_definition_id, line_number,
            None, component_item_code, None,
            quantity_required, unit_of_measure,
            "material", "0", "100",
            "push", None,
            "component_issue", None,
            None, None,
        ).await
    }

    /// List BOM components for a work definition
    pub async fn list_work_definition_components(&self, work_definition_id: Uuid) -> AtlasResult<Vec<WorkDefinitionComponent>> {
        self.repository.list_work_definition_components(work_definition_id).await
    }

    /// Remove a component from a work definition
    pub async fn delete_work_definition_component(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_work_definition_component(id).await
    }

    // ========================================================================
    // Work Definition Operations (Routing)
    // ========================================================================

    /// Add an operation to a work definition (routing step)
    pub async fn add_work_definition_operation(
        &self,
        org_id: Uuid,
        work_definition_id: Uuid,
        operation_sequence: i32,
        operation_name: &str,
        work_center_code: Option<&str>,
    ) -> AtlasResult<WorkDefinitionOperation> {
        if operation_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Operation name is required".to_string()));
        }
        if operation_sequence <= 0 {
            return Err(AtlasError::ValidationFailed("Operation sequence must be positive".to_string()));
        }

        let def = self.repository.get_work_definition_by_id(work_definition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", work_definition_id)))?;

        if def.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot modify operations of a non-draft work definition".to_string(),
            ));
        }

        info!("Adding operation {} (seq {}) to work definition {}", operation_name, operation_sequence, def.definition_number);

        self.repository.add_work_definition_operation(
            org_id, work_definition_id, operation_sequence,
            operation_name, None,
            work_center_code, None, None,
            "0", "0", "hour", "1",
            None, "machine", 1,
            "0", "0", "0",
            "standard", false, "manual",
            "100", "0",
        ).await
    }

    /// List routing operations for a work definition
    pub async fn list_work_definition_operations(&self, work_definition_id: Uuid) -> AtlasResult<Vec<WorkDefinitionOperation>> {
        self.repository.list_work_definition_operations(work_definition_id).await
    }

    /// Remove an operation from a work definition
    pub async fn delete_work_definition_operation(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_work_definition_operation(id).await
    }

    // ========================================================================
    // Work Orders
    // ========================================================================

    /// Create a new work order
    pub async fn create_work_order(
        &self,
        org_id: Uuid,
        req: CreateWorkOrderRequest,
    ) -> AtlasResult<WorkOrder> {
        // Validate quantity
        let qty: f64 = req.quantity_ordered.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity ordered must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity ordered must be positive".to_string()));
        }

        // Validate priority
        let priority = req.priority.as_deref().unwrap_or("normal");
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }

        let work_order_number = format!("WO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        // If a work definition is specified, copy BOM and routing
        let estimated_costs = if let Some(wd_id) = req.work_definition_id {
            let wd = self.repository.get_work_definition_by_id(wd_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Work definition {} not found", wd_id)))?;

            if wd.status != "active" {
                return Err(AtlasError::WorkflowError(format!(
                    "Cannot create work order from '{}' work definition. Must be 'active'.",
                    wd.status
                )));
            }

            let cost_per_unit: f64 = wd.standard_cost.parse().unwrap_or(0.0);
            let material_cost = cost_per_unit * qty * 0.6; // 60% material estimate
            let labor_cost = cost_per_unit * qty * 0.25;   // 25% labor
            let overhead_cost = cost_per_unit * qty * 0.15; // 15% overhead
            let total = material_cost + labor_cost + overhead_cost;

            Some((material_cost, labor_cost, overhead_cost, total))
        } else {
            None
        };

        let (mat_cost, lab_cost, oh_cost, total_cost) = estimated_costs.unwrap_or((0.0, 0.0, 0.0, 0.0));

        info!("Creating work order {} for org {}", work_order_number, org_id);

        let wo = self.repository.create_work_order(
            org_id, &work_order_number, req.description.as_deref(),
            req.work_definition_id,
            req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
            &req.quantity_ordered,
            req.unit_of_measure.as_deref().unwrap_or("EA"),
            req.scheduled_start_date, req.scheduled_completion_date, req.due_date,
            priority, req.production_line.as_deref(),
            req.work_center_code.as_deref(), req.warehouse_code.as_deref(),
            req.cost_type.as_deref().unwrap_or("standard"),
            &format!("{:.4}", mat_cost), &format!("{:.4}", lab_cost),
            &format!("{:.4}", oh_cost), &format!("{:.4}", total_cost),
            req.source_type.as_deref(), req.source_document_number.as_deref(),
            req.firm_planned.unwrap_or(false),
            req.company_id, req.plant_code.as_deref(),
            req.created_by,
        ).await?;

        // If work definition specified, copy BOM components and routing operations
        if let Some(wd_id) = req.work_definition_id {
            // Copy components as material requirements
            let components = self.repository.list_work_definition_components(wd_id).await?;
            for comp in &components {
                let required: f64 = comp.quantity_required.parse().unwrap_or(0.0);
                let adjusted = required * qty; // Scale by order quantity
                self.repository.create_work_order_material(
                    org_id, wo.id, comp.operation_sequence,
                    comp.component_item_id,
                    &comp.component_item_code,
                    comp.component_item_description.as_deref(),
                    &format!("{:.4}", adjusted),
                    &comp.unit_of_measure,
                    &comp.supply_type,
                    comp.supply_subinventory.as_deref(),
                    &comp.wip_supply_type,
                ).await?;
            }

            // Copy operations as work order operations
            let operations = self.repository.list_work_definition_operations(wd_id).await?;
            for op in &operations {
                self.repository.create_work_order_operation(
                    org_id, wo.id, op.operation_sequence,
                    &op.operation_name,
                    op.work_center_code.as_deref(),
                    op.work_center_name.as_deref(),
                    op.department_code.as_deref(),
                    &req.quantity_ordered, // initially all qty in queue
                    op.resource_code.as_deref(),
                    &op.resource_type,
                ).await?;
            }
        }

        Ok(wo)
    }

    /// Get a work order by number
    pub async fn get_work_order(&self, org_id: Uuid, work_order_number: &str) -> AtlasResult<Option<WorkOrder>> {
        self.repository.get_work_order(org_id, work_order_number).await
    }

    /// Get a work order by ID
    pub async fn get_work_order_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkOrder>> {
        self.repository.get_work_order_by_id(id).await
    }

    /// List work orders with optional status filter
    pub async fn list_work_orders(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WorkOrder>> {
        if let Some(s) = status {
            if !VALID_WORK_ORDER_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_WORK_ORDER_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_work_orders(org_id, status).await
    }

    /// Release a draft work order (moves to released, ready for production)
    pub async fn release_work_order(&self, id: Uuid) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;

        if wo.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot release work order in '{}' status. Must be 'draft'.",
                wo.status
            )));
        }

        info!("Releasing work order {}", wo.work_order_number);
        self.repository.update_work_order_status(id, "released").await
    }

    /// Start a released work order (production begins)
    pub async fn start_work_order(&self, id: Uuid) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;

        if wo.status != "released" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start work order in '{}' status. Must be 'released'.",
                wo.status
            )));
        }

        info!("Starting work order {}", wo.work_order_number);

        // Update first operation to running
        let operations = self.repository.list_work_order_operations(id).await?;
        if let Some(first_op) = operations.first() {
            self.repository.update_work_order_operation_status(first_op.id, "running").await?;
        }

        let updated = self.repository.update_work_order_status(id, "started").await?;
        self.repository.update_work_order_dates(id, Some(chrono::Utc::now().date_naive()), None).await
    }

    /// Complete a work order (production finished)
    pub async fn complete_work_order(&self, id: Uuid) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;

        if wo.status != "started" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete work order in '{}' status. Must be 'started'.",
                wo.status
            )));
        }

        info!("Completing work order {}", wo.work_order_number);

        // Complete all pending/running operations
        let operations = self.repository.list_work_order_operations(id).await?;
        for op in &operations {
            if op.status == "pending" || op.status == "in_queue" || op.status == "running" {
                self.repository.update_work_order_operation_status(op.id, "completed").await?;
            }
        }

        self.repository.update_work_order_status(id, "completed").await?;
        self.repository.update_work_order_dates(id, None, Some(chrono::Utc::now().date_naive())).await
    }

    /// Close a completed work order
    pub async fn close_work_order(&self, id: Uuid) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;

        if wo.status != "completed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close work order in '{}' status. Must be 'completed'.",
                wo.status
            )));
        }

        info!("Closing work order {}", wo.work_order_number);
        self.repository.update_work_order_status(id, "closed").await
    }

    /// Cancel a work order
    pub async fn cancel_work_order(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;

        if wo.status == "closed" || wo.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel work order in '{}' status.",
                wo.status
            )));
        }

        info!("Cancelling work order {} (reason: {:?})", wo.work_order_number, reason);
        self.repository.update_work_order_cancellation(id, reason).await?;
        self.repository.update_work_order_status(id, "cancelled").await
    }

    // ========================================================================
    // Production Reporting
    // ========================================================================

    /// Report production completion (against a specific operation or the order)
    pub async fn report_completion(
        &self,
        work_order_id: Uuid,
        req: ReportCompletionRequest,
    ) -> AtlasResult<WorkOrder> {
        let wo = self.repository.get_work_order_by_id(work_order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", work_order_id)))?;

        if wo.status != "started" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot report completion for work order in '{}' status. Must be 'started'.",
                wo.status
            )));
        }

        let completed: f64 = req.quantity_completed.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity completed must be a valid number".to_string(),
        ))?;
        let scrapped: f64 = req.quantity_scrapped.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity scrapped must be a valid number".to_string(),
        ))?;
        let ordered: f64 = wo.quantity_ordered.parse().unwrap_or(0.0);
        let already_completed: f64 = wo.quantity_completed.parse().unwrap_or(0.0);
        let already_scrapped: f64 = wo.quantity_scrapped.parse().unwrap_or(0.0);

        if completed < 0.0 || scrapped < 0.0 {
            return Err(AtlasError::ValidationFailed("Quantities cannot be negative".to_string()));
        }

        let total_completed = already_completed + completed;
        let total_scrapped = already_scrapped + scrapped;

        if total_completed + total_scrapped > ordered {
            return Err(AtlasError::ValidationFailed(format!(
                "Total completed ({}) + scrapped ({}) exceeds ordered quantity ({}).",
                total_completed, total_scrapped, ordered
            )));
        }

        // If an operation sequence is specified, update that operation
        if let Some(op_seq) = req.operation_sequence {
            let operations = self.repository.list_work_order_operations(work_order_id).await?;
            if let Some(op) = operations.iter().find(|o| o.operation_sequence == op_seq) {
                let op_completed: f64 = op.quantity_completed.parse().unwrap_or(0.0) + completed;
                let op_scrapped: f64 = op.quantity_scrapped.parse().unwrap_or(0.0) + scrapped;

                self.repository.update_work_order_operation_quantities(
                    op.id,
                    Some(&format!("{:.4}", op_completed)),
                    Some(&format!("{:.4}", op_scrapped)),
                    None,
                ).await?;

                // Move to next operation if this one is done
                let op_in_queue: f64 = op.quantity_in_queue.parse().unwrap_or(0.0);
                let op_running: f64 = op.quantity_running.parse().unwrap_or(0.0);
                let op_remaining = op_in_queue + op_running - completed - scrapped;
                if op_remaining <= 0.0 {
                    self.repository.update_work_order_operation_status(op.id, "completed").await?;
                    // Move next operation to running
                    if let Some(next_op) = operations.iter().find(|o| {
                        o.operation_sequence > op_seq && o.status == "pending"
                    }) {
                        self.repository.update_work_order_operation_status(next_op.id, "running").await?;
                    }
                } else {
                    self.repository.update_work_order_operation_status(op.id, "running").await?;
                }

                // Update actual costs on operation if provided
                if req.actual_run_hours.is_some() {
                    self.repository.update_work_order_operation_time(
                        op.id, None, req.actual_run_hours.as_deref(),
                    ).await?;
                }
                if req.actual_labor_cost.is_some() || req.actual_overhead_cost.is_some() {
                    self.repository.update_work_order_operation_costs(
                        op.id,
                        req.actual_labor_cost.as_deref(),
                        req.actual_overhead_cost.as_deref(),
                        None,
                    ).await?;
                }
            }
        }

        // Update work order quantities
        self.repository.update_work_order_quantities(
            work_order_id,
            Some(&format!("{:.4}", total_completed)),
            Some(&format!("{:.4}", total_scrapped)),
        ).await?;

        // Accumulate actual costs on the work order
        let new_labor: f64 = req.actual_labor_cost.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
        let new_overhead: f64 = req.actual_overhead_cost.as_deref().unwrap_or("0").parse().unwrap_or(0.0);
        let existing_labor: f64 = wo.actual_labor_cost.parse().unwrap_or(0.0);
        let existing_overhead: f64 = wo.actual_overhead_cost.parse().unwrap_or(0.0);
        let existing_material: f64 = wo.actual_material_cost.parse().unwrap_or(0.0);
        let new_total_labor = existing_labor + new_labor;
        let new_total_overhead = existing_overhead + new_overhead;
        let new_total = existing_material + new_total_labor + new_total_overhead;

        self.repository.update_work_order_actual_costs(
            work_order_id,
            None,
            Some(&format!("{:.4}", new_total_labor)),
            Some(&format!("{:.4}", new_total_overhead)),
            Some(&format!("{:.4}", new_total)),
        ).await?;

        // Check if fully completed — auto-complete if so
        if total_completed + total_scrapped >= ordered {
            info!("Work order {} fully completed", wo.work_order_number);
            self.repository.update_work_order_status(work_order_id, "completed").await?;
            self.repository.update_work_order_dates(work_order_id, None, Some(chrono::Utc::now().date_naive())).await
        } else {
            self.repository.get_work_order_by_id(work_order_id).await.map(|o| o.unwrap())
        }
    }

    /// Issue materials to a work order
    pub async fn issue_materials(
        &self,
        work_order_id: Uuid,
        issues: Vec<IssueMaterialRequest>,
    ) -> AtlasResult<Vec<WorkOrderMaterial>> {
        let wo = self.repository.get_work_order_by_id(work_order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Work order {} not found", work_order_id)))?;

        if wo.status != "released" && wo.status != "started" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot issue materials for work order in '{}' status. Must be 'released' or 'started'.",
                wo.status
            )));
        }

        let mut results = Vec::new();
        for issue in &issues {
            let mat = self.repository.get_work_order_material(issue.material_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Work order material {} not found", issue.material_id
                )))?;

            if mat.work_order_id != work_order_id {
                return Err(AtlasError::ValidationFailed(
                    "Material does not belong to the specified work order".to_string(),
                ));
            }

            let qty: f64 = issue.quantity_issued.parse().map_err(|_| AtlasError::ValidationFailed(
                "Quantity issued must be a valid number".to_string(),
            ))?;
            if qty <= 0.0 {
                return Err(AtlasError::ValidationFailed("Quantity issued must be positive".to_string()));
            }

            let required: f64 = mat.quantity_required.parse().unwrap_or(0.0);
            let already_issued: f64 = mat.quantity_issued.parse().unwrap_or(0.0);
            if already_issued + qty > required {
                return Err(AtlasError::ValidationFailed(format!(
                    "Cannot issue {} units. Only {} remaining of {} required.",
                    qty, required - already_issued, required
                )));
            }

            info!("Issuing {} units of {} to work order {}", qty, mat.component_item_code, wo.work_order_number);
            let updated = self.repository.update_work_order_material_issue(issue.material_id, &issue.quantity_issued).await?;
            results.push(updated);
        }

        Ok(results)
    }

    /// Return excess materials from a work order
    pub async fn return_material(
        &self,
        material_id: Uuid,
        quantity_returned: &str,
    ) -> AtlasResult<WorkOrderMaterial> {
        let mat = self.repository.get_work_order_material(material_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Material {} not found", material_id)))?;

        let qty: f64 = quantity_returned.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity returned must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Quantity returned must be positive".to_string()));
        }

        let issued: f64 = mat.quantity_issued.parse().unwrap_or(0.0);
        let already_returned: f64 = mat.quantity_returned.parse().unwrap_or(0.0);
        if already_returned + qty > issued {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot return {} units. Only {} were issued ({} already returned).",
                qty, issued - already_returned, already_returned
            )));
        }

        info!("Returning {} units of {} from work order", qty, mat.component_item_code);
        self.repository.update_work_order_material_return(material_id, quantity_returned).await
    }

    // ========================================================================
    // Work Order Operations
    // ========================================================================

    /// List operations for a work order
    pub async fn list_work_order_operations(&self, work_order_id: Uuid) -> AtlasResult<Vec<WorkOrderOperation>> {
        self.repository.list_work_order_operations(work_order_id).await
    }

    /// Move an operation to a specific status
    pub async fn update_operation_status(&self, id: Uuid, status: &str) -> AtlasResult<WorkOrderOperation> {
        if !VALID_OPERATION_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid operation status '{}'. Must be one of: {}", status, VALID_OPERATION_STATUSES.join(", ")
            )));
        }

        let op = self.repository.get_work_order_operation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Operation {} not found", id)))?;

        info!("Updating operation {} (seq {}) to status {}", op.operation_name, op.operation_sequence, status);
        self.repository.update_work_order_operation_status(id, status).await
    }

    // ========================================================================
    // Work Order Materials
    // ========================================================================

    /// List material requirements for a work order
    pub async fn list_work_order_materials(&self, work_order_id: Uuid) -> AtlasResult<Vec<WorkOrderMaterial>> {
        self.repository.list_work_order_materials(work_order_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get manufacturing dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ManufacturingDashboard> {
        self.repository.get_manufacturing_dashboard(org_id).await
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========================================================================
    // Stateful Mock Repository
    // ========================================================================

    struct MockState {
        definitions: HashMap<Uuid, (String, WorkDefinition)>, // (status, def)
        components: HashMap<Uuid, WorkDefinitionComponent>,
        def_operations: HashMap<Uuid, WorkDefinitionOperation>,
        orders: HashMap<Uuid, WorkOrder>,
        order_operations: HashMap<Uuid, WorkOrderOperation>,
        order_materials: HashMap<Uuid, WorkOrderMaterial>,
        next_counter: u64,
    }

    struct MockMfgRepo {
        state: Arc<Mutex<MockState>>,
    }

    impl MockMfgRepo {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    definitions: HashMap::new(),
                    components: HashMap::new(),
                    def_operations: HashMap::new(),
                    orders: HashMap::new(),
                    order_operations: HashMap::new(),
                    order_materials: HashMap::new(),
                    next_counter: 1,
                })),
            }
        }

        fn cloned(&self) -> Self {
            Self { state: self.state.clone() }
        }

        fn as_repo(self) -> Arc<dyn ManufacturingRepository> {
            Arc::new(self)
        }
    }

    fn make_definition(id: Uuid, org_id: Uuid, def_number: &str, status: &str) -> WorkDefinition {
        WorkDefinition {
            id, organization_id: org_id,
            definition_number: def_number.to_string(),
            description: None, item_id: None,
            item_code: Some("FINISHED-GOOD".to_string()),
            item_description: Some("Finished Good".to_string()),
            version: 1, status: status.to_string(),
            production_type: "discrete".to_string(),
            planning_type: "make_to_order".to_string(),
            standard_lot_size: "1".to_string(),
            unit_of_measure: "EA".to_string(),
            lead_time_days: 5,
            cost_type: "standard".to_string(),
            standard_cost: "100.0000".to_string(),
            effective_from: None, effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        }
    }

    fn make_order(id: Uuid, org_id: Uuid, wo_number: &str, status: &str, qty: &str) -> WorkOrder {
        WorkOrder {
            id, organization_id: org_id,
            work_order_number: wo_number.to_string(),
            description: None,
            work_definition_id: None,
            item_id: None,
            item_code: Some("FINISHED-GOOD".to_string()),
            item_description: Some("Finished Good".to_string()),
            quantity_ordered: qty.to_string(),
            quantity_completed: "0".to_string(),
            quantity_scrapped: "0".to_string(),
            quantity_in_queue: "0".to_string(),
            quantity_running: "0".to_string(),
            quantity_rejected: "0".to_string(),
            unit_of_measure: "EA".to_string(),
            scheduled_start_date: None, scheduled_completion_date: None,
            actual_start_date: None, actual_completion_date: None,
            due_date: None,
            status: status.to_string(),
            priority: "normal".to_string(),
            production_line: None, work_center_code: None, warehouse_code: None,
            cost_type: "standard".to_string(),
            estimated_material_cost: "60.0000".to_string(),
            estimated_labor_cost: "25.0000".to_string(),
            estimated_overhead_cost: "15.0000".to_string(),
            estimated_total_cost: "100.0000".to_string(),
            actual_material_cost: "0".to_string(),
            actual_labor_cost: "0".to_string(),
            actual_overhead_cost: "0".to_string(),
            actual_total_cost: "0".to_string(),
            source_type: None, source_document_number: None,
            source_document_line_id: None,
            firm_planned: false, company_id: None, plant_code: None,
            metadata: serde_json::json!({}),
            created_by: None,
            submitted_at: None, released_at: None, started_at: None,
            completed_at: None, closed_at: None, cancelled_at: None,
            cancellation_reason: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait::async_trait]
    impl ManufacturingRepository for MockMfgRepo {
        async fn create_work_definition(
            &self, org_id: Uuid, definition_number: &str, description: Option<&str>,
            item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
            production_type: &str, planning_type: &str,
            standard_lot_size: &str, unit_of_measure: &str, lead_time_days: i32,
            cost_type: &str, standard_cost: &str,
            effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<WorkDefinition> {
            let id = Uuid::new_v4();
            let def = WorkDefinition {
                id, organization_id: org_id,
                definition_number: definition_number.to_string(),
                description: description.map(|s| s.to_string()),
                item_id, item_code: item_code.map(|s| s.to_string()),
                item_description: item_description.map(|s| s.to_string()),
                version: 1, status: "draft".to_string(),
                production_type: production_type.to_string(),
                planning_type: planning_type.to_string(),
                standard_lot_size: standard_lot_size.to_string(),
                unit_of_measure: unit_of_measure.to_string(),
                lead_time_days,
                cost_type: cost_type.to_string(),
                standard_cost: standard_cost.to_string(),
                effective_from, effective_to,
                metadata: serde_json::json!({}),
                created_by,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().definitions.insert(id, ("draft".to_string(), def.clone()));
            Ok(def)
        }

        async fn get_work_definition(&self, org_id: Uuid, definition_number: &str) -> AtlasResult<Option<WorkDefinition>> {
            let state = self.state.lock().unwrap();
            Ok(state.definitions.values().find(|(_, d)| d.organization_id == org_id && d.definition_number == definition_number).map(|(_, d)| d.clone()))
        }

        async fn get_work_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkDefinition>> {
            Ok(self.state.lock().unwrap().definitions.get(&id).map(|(_, d)| d.clone()))
        }

        async fn list_work_definitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WorkDefinition>> {
            let state = self.state.lock().unwrap();
            Ok(state.definitions.values()
                .filter(|(_, d)| d.organization_id == org_id)
                .filter(|(_, d)| status.is_none_or(|s| d.status == s))
                .map(|(_, d)| d.clone())
                .collect())
        }

        async fn update_work_definition_status(&self, id: Uuid, status: &str) -> AtlasResult<WorkDefinition> {
            let mut state = self.state.lock().unwrap();
            if let Some((ref mut s, ref mut d)) = state.definitions.get_mut(&id) {
                *s = status.to_string();
                d.status = status.to_string();
                d.updated_at = chrono::Utc::now();
                Ok(d.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work definition {} not found", id)))
            }
        }

        async fn delete_work_definition(&self, id: Uuid) -> AtlasResult<()> {
            self.state.lock().unwrap().definitions.remove(&id);
            Ok(())
        }

        async fn add_work_definition_component(
            &self, org_id: Uuid, work_definition_id: Uuid, line_number: i32,
            component_item_id: Option<Uuid>, component_item_code: &str,
            component_item_description: Option<&str>,
            quantity_required: &str, unit_of_measure: &str,
            component_type: &str, scrap_percent: &str, yield_percent: &str,
            supply_type: &str, supply_subinventory: Option<&str>,
            wip_supply_type: &str, operation_sequence: Option<i32>,
            effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        ) -> AtlasResult<WorkDefinitionComponent> {
            let id = Uuid::new_v4();
            let comp = WorkDefinitionComponent {
                id, organization_id: org_id, work_definition_id, line_number,
                component_item_id,
                component_item_code: component_item_code.to_string(),
                component_item_description: component_item_description.map(|s| s.to_string()),
                quantity_required: quantity_required.to_string(),
                unit_of_measure: unit_of_measure.to_string(),
                component_type: component_type.to_string(),
                scrap_percent: scrap_percent.to_string(),
                yield_percent: yield_percent.to_string(),
                supply_type: supply_type.to_string(),
                supply_subinventory: supply_subinventory.map(|s| s.to_string()),
                wip_supply_type: wip_supply_type.to_string(),
                operation_sequence, effective_from, effective_to,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().components.insert(id, comp.clone());
            Ok(comp)
        }

        async fn list_work_definition_components(&self, work_definition_id: Uuid) -> AtlasResult<Vec<WorkDefinitionComponent>> {
            let state = self.state.lock().unwrap();
            Ok(state.components.values()
                .filter(|c| c.work_definition_id == work_definition_id)
                .cloned()
                .collect())
        }

        async fn delete_work_definition_component(&self, id: Uuid) -> AtlasResult<()> {
            self.state.lock().unwrap().components.remove(&id);
            Ok(())
        }

        async fn add_work_definition_operation(
            &self, org_id: Uuid, work_definition_id: Uuid, operation_sequence: i32,
            operation_name: &str, operation_description: Option<&str>,
            work_center_code: Option<&str>, work_center_name: Option<&str>,
            department_code: Option<&str>,
            setup_hours: &str, run_time_hours: &str, run_time_unit: &str,
            units_per_run: &str,
            resource_code: Option<&str>, resource_type: &str, resource_count: i32,
            standard_labor_cost: &str, standard_overhead_cost: &str,
            standard_machine_cost: &str,
            operation_type: &str, backflush_enabled: bool,
            count_point_type: &str, yield_percent: &str, scrap_percent: &str,
        ) -> AtlasResult<WorkDefinitionOperation> {
            let id = Uuid::new_v4();
            let op = WorkDefinitionOperation {
                id, organization_id: org_id, work_definition_id, operation_sequence,
                operation_name: operation_name.to_string(),
                operation_description: operation_description.map(|s| s.to_string()),
                work_center_code: work_center_code.map(|s| s.to_string()),
                work_center_name: work_center_name.map(|s| s.to_string()),
                department_code: department_code.map(|s| s.to_string()),
                setup_hours: setup_hours.to_string(),
                run_time_hours: run_time_hours.to_string(),
                run_time_unit: run_time_unit.to_string(),
                units_per_run: units_per_run.to_string(),
                resource_code: resource_code.map(|s| s.to_string()),
                resource_type: resource_type.to_string(),
                resource_count,
                standard_labor_cost: standard_labor_cost.to_string(),
                standard_overhead_cost: standard_overhead_cost.to_string(),
                standard_machine_cost: standard_machine_cost.to_string(),
                operation_type: operation_type.to_string(),
                backflush_enabled,
                count_point_type: count_point_type.to_string(),
                yield_percent: yield_percent.to_string(),
                scrap_percent: scrap_percent.to_string(),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().def_operations.insert(id, op.clone());
            Ok(op)
        }

        async fn list_work_definition_operations(&self, work_definition_id: Uuid) -> AtlasResult<Vec<WorkDefinitionOperation>> {
            let state = self.state.lock().unwrap();
            Ok(state.def_operations.values()
                .filter(|o| o.work_definition_id == work_definition_id)
                .cloned()
                .collect())
        }

        async fn delete_work_definition_operation(&self, id: Uuid) -> AtlasResult<()> {
            self.state.lock().unwrap().def_operations.remove(&id);
            Ok(())
        }

        async fn create_work_order(
            &self, org_id: Uuid, work_order_number: &str, description: Option<&str>,
            work_definition_id: Option<Uuid>,
            item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
            quantity_ordered: &str, unit_of_measure: &str,
            scheduled_start_date: Option<chrono::NaiveDate>,
            scheduled_completion_date: Option<chrono::NaiveDate>,
            due_date: Option<chrono::NaiveDate>,
            priority: &str, production_line: Option<&str>,
            work_center_code: Option<&str>, warehouse_code: Option<&str>,
            cost_type: &str,
            estimated_material_cost: &str, estimated_labor_cost: &str,
            estimated_overhead_cost: &str, estimated_total_cost: &str,
            source_type: Option<&str>, source_document_number: Option<&str>,
            firm_planned: bool, company_id: Option<Uuid>, plant_code: Option<&str>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<WorkOrder> {
            let id = Uuid::new_v4();
            let wo = WorkOrder {
                id, organization_id: org_id,
                work_order_number: work_order_number.to_string(),
                description: description.map(|s| s.to_string()),
                work_definition_id,
                item_id, item_code: item_code.map(|s| s.to_string()),
                item_description: item_description.map(|s| s.to_string()),
                quantity_ordered: quantity_ordered.to_string(),
                quantity_completed: "0".to_string(),
                quantity_scrapped: "0".to_string(),
                quantity_in_queue: "0".to_string(),
                quantity_running: "0".to_string(),
                quantity_rejected: "0".to_string(),
                unit_of_measure: unit_of_measure.to_string(),
                scheduled_start_date, scheduled_completion_date,
                actual_start_date: None, actual_completion_date: None,
                due_date,
                status: "draft".to_string(),
                priority: priority.to_string(),
                production_line: production_line.map(|s| s.to_string()),
                work_center_code: work_center_code.map(|s| s.to_string()),
                warehouse_code: warehouse_code.map(|s| s.to_string()),
                cost_type: cost_type.to_string(),
                estimated_material_cost: estimated_material_cost.to_string(),
                estimated_labor_cost: estimated_labor_cost.to_string(),
                estimated_overhead_cost: estimated_overhead_cost.to_string(),
                estimated_total_cost: estimated_total_cost.to_string(),
                actual_material_cost: "0".to_string(),
                actual_labor_cost: "0".to_string(),
                actual_overhead_cost: "0".to_string(),
                actual_total_cost: "0".to_string(),
                source_type: source_type.map(|s| s.to_string()),
                source_document_number: source_document_number.map(|s| s.to_string()),
                source_document_line_id: None,
                firm_planned, company_id, plant_code: plant_code.map(|s| s.to_string()),
                metadata: serde_json::json!({}),
                created_by,
                submitted_at: None, released_at: None, started_at: None,
                completed_at: None, closed_at: None, cancelled_at: None,
                cancellation_reason: None,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().orders.insert(id, wo.clone());
            Ok(wo)
        }

        async fn get_work_order(&self, org_id: Uuid, work_order_number: &str) -> AtlasResult<Option<WorkOrder>> {
            let state = self.state.lock().unwrap();
            Ok(state.orders.values().find(|o| o.organization_id == org_id && o.work_order_number == work_order_number).cloned())
        }

        async fn get_work_order_by_id(&self, id: Uuid) -> AtlasResult<Option<WorkOrder>> {
            Ok(self.state.lock().unwrap().orders.get(&id).cloned())
        }

        async fn list_work_orders(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WorkOrder>> {
            let state = self.state.lock().unwrap();
            Ok(state.orders.values()
                .filter(|o| o.organization_id == org_id)
                .filter(|o| status.is_none_or(|s| o.status == s))
                .cloned()
                .collect())
        }

        async fn update_work_order_status(&self, id: Uuid, status: &str) -> AtlasResult<WorkOrder> {
            let mut state = self.state.lock().unwrap();
            if let Some(wo) = state.orders.get_mut(&id) {
                wo.status = status.to_string();
                wo.updated_at = chrono::Utc::now();
                Ok(wo.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work order {} not found", id)))
            }
        }

        async fn update_work_order_quantities(
            &self, id: Uuid,
            quantity_completed: Option<&str>,
            quantity_scrapped: Option<&str>,
        ) -> AtlasResult<WorkOrder> {
            let mut state = self.state.lock().unwrap();
            if let Some(wo) = state.orders.get_mut(&id) {
                if let Some(qc) = quantity_completed { wo.quantity_completed = qc.to_string(); }
                if let Some(qs) = quantity_scrapped { wo.quantity_scrapped = qs.to_string(); }
                wo.updated_at = chrono::Utc::now();
                Ok(wo.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work order {} not found", id)))
            }
        }

        async fn update_work_order_actual_costs(
            &self, id: Uuid,
            actual_material_cost: Option<&str>,
            actual_labor_cost: Option<&str>,
            actual_overhead_cost: Option<&str>,
            actual_total_cost: Option<&str>,
        ) -> AtlasResult<WorkOrder> {
            let mut state = self.state.lock().unwrap();
            if let Some(wo) = state.orders.get_mut(&id) {
                if let Some(c) = actual_material_cost { wo.actual_material_cost = c.to_string(); }
                if let Some(c) = actual_labor_cost { wo.actual_labor_cost = c.to_string(); }
                if let Some(c) = actual_overhead_cost { wo.actual_overhead_cost = c.to_string(); }
                if let Some(c) = actual_total_cost { wo.actual_total_cost = c.to_string(); }
                wo.updated_at = chrono::Utc::now();
                Ok(wo.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work order {} not found", id)))
            }
        }

        async fn update_work_order_dates(
            &self, id: Uuid,
            actual_start_date: Option<chrono::NaiveDate>,
            actual_completion_date: Option<chrono::NaiveDate>,
        ) -> AtlasResult<WorkOrder> {
            let mut state = self.state.lock().unwrap();
            if let Some(wo) = state.orders.get_mut(&id) {
                if let Some(d) = actual_start_date { wo.actual_start_date = Some(d); }
                if let Some(d) = actual_completion_date { wo.actual_completion_date = Some(d); }
                wo.updated_at = chrono::Utc::now();
                Ok(wo.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work order {} not found", id)))
            }
        }

        async fn update_work_order_cancellation(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<WorkOrder> {
            let mut state = self.state.lock().unwrap();
            if let Some(wo) = state.orders.get_mut(&id) {
                wo.cancellation_reason = reason.map(|s| s.to_string());
                wo.updated_at = chrono::Utc::now();
                Ok(wo.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Work order {} not found", id)))
            }
        }

        async fn create_work_order_operation(
            &self, org_id: Uuid, work_order_id: Uuid, operation_sequence: i32,
            operation_name: &str, work_center_code: Option<&str>,
            work_center_name: Option<&str>, department_code: Option<&str>,
            quantity_in_queue: &str,
            resource_code: Option<&str>, resource_type: &str,
        ) -> AtlasResult<WorkOrderOperation> {
            let id = Uuid::new_v4();
            let op = WorkOrderOperation {
                id, organization_id: org_id, work_order_id, operation_sequence,
                operation_name: operation_name.to_string(),
                work_center_code: work_center_code.map(|s| s.to_string()),
                work_center_name: work_center_name.map(|s| s.to_string()),
                department_code: department_code.map(|s| s.to_string()),
                quantity_in_queue: quantity_in_queue.to_string(),
                quantity_running: "0".to_string(),
                quantity_completed: "0".to_string(),
                quantity_rejected: "0".to_string(),
                quantity_scrapped: "0".to_string(),
                scheduled_start_date: None, scheduled_completion_date: None,
                actual_start_date: None, actual_completion_date: None,
                actual_setup_hours: "0".to_string(),
                actual_run_hours: "0".to_string(),
                resource_code: resource_code.map(|s| s.to_string()),
                resource_type: resource_type.to_string(),
                status: "pending".to_string(),
                actual_labor_cost: "0".to_string(),
                actual_overhead_cost: "0".to_string(),
                actual_machine_cost: "0".to_string(),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().order_operations.insert(id, op.clone());
            Ok(op)
        }

        async fn get_work_order_operation(&self, id: Uuid) -> AtlasResult<Option<WorkOrderOperation>> {
            Ok(self.state.lock().unwrap().order_operations.get(&id).cloned())
        }

        async fn list_work_order_operations(&self, work_order_id: Uuid) -> AtlasResult<Vec<WorkOrderOperation>> {
            let state = self.state.lock().unwrap();
            let mut ops: Vec<_> = state.order_operations.values()
                .filter(|o| o.work_order_id == work_order_id)
                .cloned()
                .collect();
            ops.sort_by_key(|o| o.operation_sequence);
            Ok(ops)
        }

        async fn update_work_order_operation_status(&self, id: Uuid, status: &str) -> AtlasResult<WorkOrderOperation> {
            let mut state = self.state.lock().unwrap();
            if let Some(op) = state.order_operations.get_mut(&id) {
                op.status = status.to_string();
                op.updated_at = chrono::Utc::now();
                Ok(op.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Operation {} not found", id)))
            }
        }

        async fn update_work_order_operation_quantities(
            &self, id: Uuid,
            quantity_completed: Option<&str>,
            quantity_scrapped: Option<&str>,
            quantity_rejected: Option<&str>,
        ) -> AtlasResult<WorkOrderOperation> {
            let mut state = self.state.lock().unwrap();
            if let Some(op) = state.order_operations.get_mut(&id) {
                if let Some(qc) = quantity_completed { op.quantity_completed = qc.to_string(); }
                if let Some(qs) = quantity_scrapped { op.quantity_scrapped = qs.to_string(); }
                if let Some(qr) = quantity_rejected { op.quantity_rejected = qr.to_string(); }
                op.updated_at = chrono::Utc::now();
                Ok(op.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Operation {} not found", id)))
            }
        }

        async fn update_work_order_operation_time(
            &self, id: Uuid,
            actual_setup_hours: Option<&str>,
            actual_run_hours: Option<&str>,
        ) -> AtlasResult<WorkOrderOperation> {
            let mut state = self.state.lock().unwrap();
            if let Some(op) = state.order_operations.get_mut(&id) {
                if let Some(h) = actual_setup_hours { op.actual_setup_hours = h.to_string(); }
                if let Some(h) = actual_run_hours { op.actual_run_hours = h.to_string(); }
                op.updated_at = chrono::Utc::now();
                Ok(op.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Operation {} not found", id)))
            }
        }

        async fn update_work_order_operation_costs(
            &self, id: Uuid,
            actual_labor_cost: Option<&str>,
            actual_overhead_cost: Option<&str>,
            actual_machine_cost: Option<&str>,
        ) -> AtlasResult<WorkOrderOperation> {
            let mut state = self.state.lock().unwrap();
            if let Some(op) = state.order_operations.get_mut(&id) {
                if let Some(c) = actual_labor_cost { op.actual_labor_cost = c.to_string(); }
                if let Some(c) = actual_overhead_cost { op.actual_overhead_cost = c.to_string(); }
                if let Some(c) = actual_machine_cost { op.actual_machine_cost = c.to_string(); }
                op.updated_at = chrono::Utc::now();
                Ok(op.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Operation {} not found", id)))
            }
        }

        async fn update_work_order_operation_dates(
            &self, id: Uuid,
            actual_start_date: Option<chrono::NaiveDate>,
            actual_completion_date: Option<chrono::NaiveDate>,
        ) -> AtlasResult<WorkOrderOperation> {
            let mut state = self.state.lock().unwrap();
            if let Some(op) = state.order_operations.get_mut(&id) {
                if let Some(d) = actual_start_date { op.actual_start_date = Some(d); }
                if let Some(d) = actual_completion_date { op.actual_completion_date = Some(d); }
                op.updated_at = chrono::Utc::now();
                Ok(op.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Operation {} not found", id)))
            }
        }

        async fn create_work_order_material(
            &self, org_id: Uuid, work_order_id: Uuid, operation_sequence: Option<i32>,
            component_item_id: Option<Uuid>, component_item_code: &str,
            component_item_description: Option<&str>,
            quantity_required: &str, unit_of_measure: &str,
            supply_type: &str, supply_subinventory: Option<&str>,
            wip_supply_type: &str,
        ) -> AtlasResult<WorkOrderMaterial> {
            let id = Uuid::new_v4();
            let mat = WorkOrderMaterial {
                id, organization_id: org_id, work_order_id, operation_sequence,
                component_item_id,
                component_item_code: component_item_code.to_string(),
                component_item_description: component_item_description.map(|s| s.to_string()),
                quantity_required: quantity_required.to_string(),
                quantity_issued: "0".to_string(),
                quantity_returned: "0".to_string(),
                quantity_scrapped: "0".to_string(),
                unit_of_measure: unit_of_measure.to_string(),
                supply_type: supply_type.to_string(),
                supply_subinventory: supply_subinventory.map(|s| s.to_string()),
                wip_supply_type: wip_supply_type.to_string(),
                status: "pending".to_string(),
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.state.lock().unwrap().order_materials.insert(id, mat.clone());
            Ok(mat)
        }

        async fn get_work_order_material(&self, id: Uuid) -> AtlasResult<Option<WorkOrderMaterial>> {
            Ok(self.state.lock().unwrap().order_materials.get(&id).cloned())
        }

        async fn list_work_order_materials(&self, work_order_id: Uuid) -> AtlasResult<Vec<WorkOrderMaterial>> {
            let state = self.state.lock().unwrap();
            Ok(state.order_materials.values()
                .filter(|m| m.work_order_id == work_order_id)
                .cloned()
                .collect())
        }

        async fn update_work_order_material_issue(&self, id: Uuid, quantity_issued: &str) -> AtlasResult<WorkOrderMaterial> {
            let mut state = self.state.lock().unwrap();
            if let Some(mat) = state.order_materials.get_mut(&id) {
                let new_issued: f64 = mat.quantity_issued.parse().unwrap_or(0.0) + quantity_issued.parse().unwrap_or(0.0);
                let required: f64 = mat.quantity_required.parse().unwrap_or(0.0);
                mat.quantity_issued = format!("{:.4}", new_issued);
                mat.status = if new_issued >= required { "fully_issued" } else { "partially_issued" }.to_string();
                mat.updated_at = chrono::Utc::now();
                Ok(mat.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Material {} not found", id)))
            }
        }

        async fn update_work_order_material_return(&self, id: Uuid, quantity_returned: &str) -> AtlasResult<WorkOrderMaterial> {
            let mut state = self.state.lock().unwrap();
            if let Some(mat) = state.order_materials.get_mut(&id) {
                let new_returned: f64 = mat.quantity_returned.parse().unwrap_or(0.0) + quantity_returned.parse().unwrap_or(0.0);
                mat.quantity_returned = format!("{:.4}", new_returned);
                mat.updated_at = chrono::Utc::now();
                Ok(mat.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Material {} not found", id)))
            }
        }

        async fn update_work_order_material_status(&self, id: Uuid, status: &str) -> AtlasResult<WorkOrderMaterial> {
            let mut state = self.state.lock().unwrap();
            if let Some(mat) = state.order_materials.get_mut(&id) {
                mat.status = status.to_string();
                mat.updated_at = chrono::Utc::now();
                Ok(mat.clone())
            } else {
                Err(AtlasError::EntityNotFound(format!("Material {} not found", id)))
            }
        }

        async fn get_manufacturing_dashboard(&self, _org_id: Uuid) -> AtlasResult<ManufacturingDashboard> {
            Ok(ManufacturingDashboard {
                total_work_orders: 0, open_work_orders: 0, in_progress_work_orders: 0,
                completed_work_orders: 0, cancelled_work_orders: 0,
                total_definitions: 0, active_definitions: 0,
                overdue_orders: 0,
                total_estimated_cost: "0".to_string(),
                total_actual_cost: "0".to_string(),
                cost_variance_pct: "0.0".to_string(),
                orders_by_status: serde_json::json!({}),
                orders_by_priority: serde_json::json!({}),
                completion_rate_pct: "0.0".to_string(),
                on_time_completion_pct: "100.0".to_string(),
            })
        }
    }

    // ========================================================================
    // Work Definition Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_work_definition_success() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-001".to_string()),
            description: Some("Widget Assembly".to_string()),
            item_code: Some("WIDGET-100".to_string()),
            item_description: Some("Standard Widget".to_string()),
            production_type: Some("discrete".to_string()),
            standard_lot_size: Some("1".to_string()),
            lead_time_days: Some(5),
            standard_cost: Some("100".to_string()),
            ..Default::default()
        }).await;

        assert!(result.is_ok());
        let def = result.unwrap();
        assert_eq!(def.definition_number, "WD-001");
        assert_eq!(def.status, "draft");
        assert_eq!(def.production_type, "discrete");
        assert_eq!(def.lead_time_days, 5);
    }

    #[tokio::test]
    async fn test_create_work_definition_auto_number() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            item_code: Some("ITEM-01".to_string()),
            ..Default::default()
        }).await;

        assert!(result.is_ok());
        let def = result.unwrap();
        assert!(def.definition_number.starts_with("WD-"));
    }

    #[tokio::test]
    async fn test_create_work_definition_invalid_production_type() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            production_type: Some("invalid".to_string()),
            ..Default::default()
        }).await;

        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("Invalid production type"));
    }

    #[tokio::test]
    async fn test_activate_work_definition_requires_components_and_operations() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        // Create a definition with no components or operations
        let def = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-EMPTY".to_string()),
            ..Default::default()
        }).await.unwrap();

        // Should fail to activate without BOM + routing
        let result = engine.activate_work_definition(def.id).await;
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("without any components") || msg.contains("without any operations"));
    }

    #[tokio::test]
    async fn test_activate_work_definition_success() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let def = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-FULL".to_string()),
            standard_cost: Some("100".to_string()),
            ..Default::default()
        }).await.unwrap();

        // Add a component
        engine.add_work_definition_component(
            org_id, def.id, "RAW-MAT-01", "5", "EA",
        ).await.unwrap();

        // Add an operation
        engine.add_work_definition_operation(
            org_id, def.id, 10, "Assembly", Some("WC-ASSEMBLY"),
        ).await.unwrap();

        // Should now activate
        let result = engine.activate_work_definition(def.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "active");
    }

    #[tokio::test]
    async fn test_cannot_modify_active_definition() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let def = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-ACTIVE".to_string()),
            ..Default::default()
        }).await.unwrap();

        // Add component + operation then activate
        engine.add_work_definition_component(org_id, def.id, "COMP-01", "1", "EA").await.unwrap();
        engine.add_work_definition_operation(org_id, def.id, 10, "Cut", None).await.unwrap();
        engine.activate_work_definition(def.id).await.unwrap();

        // Should fail to add component to active definition
        let result = engine.add_work_definition_component(
            org_id, def.id, "COMP-02", "2", "EA",
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("non-draft"));
    }

    // ========================================================================
    // Work Order Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_work_order_success() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_order(org_id, CreateWorkOrderRequest {
            item_code: Some("WIDGET-100".to_string()),
            item_description: Some("Standard Widget".to_string()),
            quantity_ordered: "100".to_string(),
            priority: Some("high".to_string()),
            ..Default::default()
        }).await;

        assert!(result.is_ok());
        let wo = result.unwrap();
        assert!(wo.work_order_number.starts_with("WO-"));
        assert_eq!(wo.status, "draft");
        assert_eq!(wo.priority, "high");
    }

    #[tokio::test]
    async fn test_create_work_order_validates_quantity() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "0".to_string(),
            ..Default::default()
        }).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("positive"));

        let result = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "-5".to_string(),
            ..Default::default()
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_work_order_validates_priority() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let org_id = Uuid::new_v4();

        let result = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "10".to_string(),
            priority: Some("super_high".to_string()),
            ..Default::default()
        }).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid priority"));
    }

    #[tokio::test]
    async fn test_work_order_lifecycle_release_start_complete_close() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "50".to_string(),
            item_code: Some("ITEM-01".to_string()),
            ..Default::default()
        }).await.unwrap();

        // Release
        let wo = engine.release_work_order(wo.id).await.unwrap();
        assert_eq!(wo.status, "released");

        // Start
        let wo = engine.start_work_order(wo.id).await.unwrap();
        assert_eq!(wo.status, "started");
        assert!(wo.actual_start_date.is_some());

        // Complete
        let wo = engine.complete_work_order(wo.id).await.unwrap();
        assert_eq!(wo.status, "completed");
        assert!(wo.actual_completion_date.is_some());

        // Close
        let wo = engine.close_work_order(wo.id).await.unwrap();
        assert_eq!(wo.status, "closed");
    }

    #[tokio::test]
    async fn test_work_order_cannot_release_non_draft() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "10".to_string(),
            ..Default::default()
        }).await.unwrap();

        // Release first
        engine.release_work_order(wo.id).await.unwrap();

        // Try to release again
        let result = engine.release_work_order(wo.id).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'draft'"));
    }

    #[tokio::test]
    async fn test_cancel_work_order() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "10".to_string(),
            ..Default::default()
        }).await.unwrap();

        let result = engine.cancel_work_order(wo.id, Some("Customer cancelled")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "cancelled");
    }

    #[tokio::test]
    async fn test_cannot_cancel_closed_order() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "10".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();
        engine.start_work_order(wo.id).await.unwrap();
        engine.complete_work_order(wo.id).await.unwrap();
        engine.close_work_order(wo.id).await.unwrap();

        let result = engine.cancel_work_order(wo.id, None).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Cannot cancel"));
    }

    // ========================================================================
    // Production Reporting Tests
    // ========================================================================

    #[tokio::test]
    async fn test_report_completion_partial() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();
        engine.start_work_order(wo.id).await.unwrap();

        // Report partial completion of 40 units
        let result = engine.report_completion(wo.id, ReportCompletionRequest {
            operation_sequence: None,
            quantity_completed: "40".to_string(),
            quantity_scrapped: "5".to_string(),
            actual_run_hours: Some("2.5".to_string()),
            actual_labor_cost: Some("50".to_string()),
            actual_overhead_cost: None,
            completed_by: None,
        }).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.quantity_completed, "40.0000");
        assert_eq!(updated.quantity_scrapped, "5.0000");
        // Should not be completed yet
        assert_ne!(updated.status, "completed");
    }

    #[tokio::test]
    async fn test_report_completion_auto_completes_when_fully_done() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();
        engine.start_work_order(wo.id).await.unwrap();

        // Report full completion
        let result = engine.report_completion(wo.id, ReportCompletionRequest {
            operation_sequence: None,
            quantity_completed: "95".to_string(),
            quantity_scrapped: "5".to_string(),
            actual_run_hours: None,
            actual_labor_cost: None,
            actual_overhead_cost: None,
            completed_by: None,
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "completed");
    }

    #[tokio::test]
    async fn test_report_completion_exceeds_ordered() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();
        engine.start_work_order(wo.id).await.unwrap();

        let result = engine.report_completion(wo.id, ReportCompletionRequest {
            operation_sequence: None,
            quantity_completed: "101".to_string(),
            quantity_scrapped: "0".to_string(),
            ..Default::default()
        }).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("exceeds ordered"));
    }

    #[tokio::test]
    async fn test_report_completion_only_for_started_order() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        // Try to report on a draft order
        let result = engine.report_completion(wo.id, ReportCompletionRequest {
            operation_sequence: None,
            quantity_completed: "50".to_string(),
            quantity_scrapped: "0".to_string(),
            ..Default::default()
        }).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'started'"));
    }

    // ========================================================================
    // Material Management Tests
    // ========================================================================

    #[tokio::test]
    async fn test_issue_materials_success() {
        let repo = MockMfgRepo::new();
        let repo_clone = repo.cloned();
        let engine = ManufacturingEngine::new(repo.as_repo());
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();

        // Create a material requirement manually
        let mat = repo_clone.create_work_order_material(
            org_id, wo.id, None, None, "RAW-01", Some("Raw Material"),
            "200", "EA", "push", None, "component_issue",
        ).await.unwrap();

        // Issue 150 units
        let result = engine.issue_materials(wo.id, vec![
            IssueMaterialRequest {
                material_id: mat.id,
                quantity_issued: "150".to_string(),
            },
        ]).await;

        assert!(result.is_ok());
        let materials = result.unwrap();
        assert_eq!(materials[0].quantity_issued, "150.0000");
        assert_eq!(materials[0].status, "partially_issued");
    }

    #[tokio::test]
    async fn test_issue_materials_fully_issued() {
        let repo = MockMfgRepo::new();
        let repo_clone = repo.cloned();
        let engine = ManufacturingEngine::new(repo.as_repo());
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();

        let mat = repo_clone.create_work_order_material(
            org_id, wo.id, None, None, "RAW-01", Some("Raw Material"),
            "200", "EA", "push", None, "component_issue",
        ).await.unwrap();

        // Issue full 200 units
        let result = engine.issue_materials(wo.id, vec![
            IssueMaterialRequest {
                material_id: mat.id,
                quantity_issued: "200".to_string(),
            },
        ]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].status, "fully_issued");
    }

    #[tokio::test]
    async fn test_issue_materials_exceeds_required() {
        let repo = MockMfgRepo::new();
        let repo_clone = repo.cloned();
        let engine = ManufacturingEngine::new(repo.as_repo());
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();

        let mat = repo_clone.create_work_order_material(
            org_id, wo.id, None, None, "RAW-01", Some("Raw Material"),
            "200", "EA", "push", None, "component_issue",
        ).await.unwrap();

        // Try to issue more than required
        let result = engine.issue_materials(wo.id, vec![
            IssueMaterialRequest {
                material_id: mat.id,
                quantity_issued: "250".to_string(),
            },
        ]).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Cannot issue"));
    }

    #[tokio::test]
    async fn test_return_material_success() {
        let repo = MockMfgRepo::new();
        let repo_clone = repo.cloned();
        let engine = ManufacturingEngine::new(repo.as_repo());
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();

        let mat = repo_clone.create_work_order_material(
            org_id, wo.id, None, None, "RAW-01", Some("Raw Material"),
            "200", "EA", "push", None, "component_issue",
        ).await.unwrap();

        // Issue 200
        engine.issue_materials(wo.id, vec![
            IssueMaterialRequest { material_id: mat.id, quantity_issued: "200".to_string() },
        ]).await.unwrap();

        // Return 50
        let result = engine.return_material(mat.id, "50").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().quantity_returned, "50.0000");
    }

    #[tokio::test]
    async fn test_return_material_exceeds_issued() {
        let repo = MockMfgRepo::new();
        let repo_clone = repo.cloned();
        let engine = ManufacturingEngine::new(repo.as_repo());
        let org_id = Uuid::new_v4();

        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            quantity_ordered: "100".to_string(),
            ..Default::default()
        }).await.unwrap();

        engine.release_work_order(wo.id).await.unwrap();

        let mat = repo_clone.create_work_order_material(
            org_id, wo.id, None, None, "RAW-01", Some("Raw Material"),
            "200", "EA", "push", None, "component_issue",
        ).await.unwrap();

        // Issue only 100
        engine.issue_materials(wo.id, vec![
            IssueMaterialRequest { material_id: mat.id, quantity_issued: "100".to_string() },
        ]).await.unwrap();

        // Try to return 150 (more than issued)
        let result = engine.return_material(mat.id, "150").await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Cannot return"));
    }

    // ========================================================================
    // Work Order with Work Definition Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_work_order_copies_bom_and_routing() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        // Create and set up a work definition
        let def = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-BOM".to_string()),
            item_code: Some("FG-01".to_string()),
            standard_cost: Some("100".to_string()),
            ..Default::default()
        }).await.unwrap();

        engine.add_work_definition_component(org_id, def.id, "COMP-A", "2", "EA").await.unwrap();
        engine.add_work_definition_component(org_id, def.id, "COMP-B", "3", "EA").await.unwrap();
        engine.add_work_definition_operation(org_id, def.id, 10, "Cut", Some("WC-01")).await.unwrap();
        engine.add_work_definition_operation(org_id, def.id, 20, "Assemble", Some("WC-02")).await.unwrap();
        engine.activate_work_definition(def.id).await.unwrap();

        // Create a work order from the definition
        let wo = engine.create_work_order(org_id, CreateWorkOrderRequest {
            work_definition_id: Some(def.id),
            item_code: Some("FG-01".to_string()),
            quantity_ordered: "10".to_string(),
            ..Default::default()
        }).await.unwrap();

        // Verify BOM was copied (2 components × 10 qty each)
        let materials = engine.list_work_order_materials(wo.id).await.unwrap();
        assert_eq!(materials.len(), 2);

        // Verify routing was copied
        let operations = engine.list_work_order_operations(wo.id).await.unwrap();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0].operation_name, "Cut");
        assert_eq!(operations[1].operation_name, "Assemble");
    }

    #[tokio::test]
    async fn test_create_work_order_from_inactive_definition_fails() {
        let repo = MockMfgRepo::new();
        let engine = ManufacturingEngine::new(Arc::new(repo));
        let org_id = Uuid::new_v4();

        // Create but don't activate
        let def = engine.create_work_definition(org_id, CreateWorkDefinitionRequest {
            definition_number: Some("WD-INACTIVE".to_string()),
            ..Default::default()
        }).await.unwrap();

        let result = engine.create_work_order(org_id, CreateWorkOrderRequest {
            work_definition_id: Some(def.id),
            quantity_ordered: "10".to_string(),
            ..Default::default()
        }).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'active'"));
    }

    // ========================================================================
    // Dashboard Test
    // ========================================================================

    #[tokio::test]
    async fn test_get_dashboard() {
        let engine = ManufacturingEngine::new(Arc::new(MockMfgRepo::new()));
        let result = engine.get_dashboard(Uuid::new_v4()).await;
        assert!(result.is_ok());
        let dashboard = result.unwrap();
        assert_eq!(dashboard.total_work_orders, 0);
        assert_eq!(dashboard.active_definitions, 0);
    }
}
