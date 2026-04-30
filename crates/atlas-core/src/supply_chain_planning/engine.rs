//! Supply Chain Planning Engine (MRP)
//!
//! Oracle Fusion Cloud: Supply Chain Management > Supply Chain Planning
//!
//! Features:
//! - Planning Scenarios (MRP, distribution, demand, production planning)
//! - Planning Parameters (item-level: lead time, safety stock, lot sizing)
//! - Supply/Demand netting (on-hand, POs, WOs, sales orders, forecasts)
//! - Planned Order generation (buy/make/transfer suggestions)
//! - Planning Exception management (shortages, late orders, excess)
//! - Planning Dashboard
//!
//! MRP Process:
//! 1. Define planning parameters per item
//! 2. Create a planning scenario
//! 3. Load supply/demand data into scenario
//! 4. Run MRP (net supply vs demand, generate planned orders)
//! 5. Review and firm planned orders
//! 6. Resolve planning exceptions

use atlas_shared::{
    PlanningScenario, PlanningParameter, SupplyDemandEntry,
    PlannedOrder, PlanningException, PlanningDashboard,
    AtlasError, AtlasResult,
};
use super::PlanningRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_SCENARIO_TYPES: &[&str] = &[
    "mrp", "distribution_planning", "demand_planning", "production_planning",
];

const VALID_SCENARIO_STATUSES: &[&str] = &[
    "draft", "running", "completed", "error", "cancelled",
];

const VALID_PLANNING_METHODS: &[&str] = &[
    "mrp", "min_max", "reorder_point", "kanban", "not_planned",
];

const VALID_MAKE_BUY: &[&str] = &["make", "buy"];

const VALID_LOT_POLICIES: &[&str] = &[
    "fixed_quantity", "lot_for_lot", "period_order_quantity", "min_max",
];

const VALID_ENTRY_TYPES: &[&str] = &["supply", "demand"];

const VALID_SOURCE_TYPES: &[&str] = &[
    "on_hand", "purchase_order", "work_order", "transfer_order",
    "sales_order", "forecast", "safety_stock",
];

const VALID_ORDER_TYPES: &[&str] = &["buy", "make", "transfer"];

const VALID_ORDER_STATUSES: &[&str] = &[
    "unfirm", "firmed", "released", "cancelled", "completed",
];

const VALID_ORDER_ACTIONS: &[&str] = &[
    "new", "reschedule_in", "reschedule_out", "cancel", "expedite",
];

const VALID_EXCEPTION_TYPES: &[&str] = &[
    "late_order", "early_order", "excess_supply", "shortage",
    "past_due_demand", "order_past_due", "over_planned", "under_planned",
    "cancel_suggestion", "reschedule_suggestion",
];

const VALID_SEVERITIES: &[&str] = &["info", "warning", "error", "critical"];

const VALID_RESOLUTION_STATUSES: &[&str] = &[
    "open", "acknowledged", "resolved", "dismissed",
];

fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!("{} is required", field)));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Supply Chain Planning Engine
pub struct SupplyChainPlanningEngine {
    repository: Arc<dyn PlanningRepository>,
}

impl SupplyChainPlanningEngine {
    pub fn new(repository: Arc<dyn PlanningRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Planning Scenarios
    // ========================================================================

    /// Create a new planning scenario
    pub async fn create_scenario(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        scenario_type: &str,
        planning_horizon_days: i32,
        planning_start_date: Option<chrono::NaiveDate>,
        include_existing_supply: bool,
        include_on_hand: bool,
        include_wip: bool,
        auto_firm: bool,
        auto_firm_days: Option<i32>,
        net_shortages_only: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningScenario> {
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Scenario name is required".to_string()));
        }
        validate_enum("scenario type", scenario_type, VALID_SCENARIO_TYPES)?;
        if planning_horizon_days < 1 {
            return Err(AtlasError::ValidationFailed(
                "Planning horizon must be at least 1 day".to_string(),
            ));
        }

        let scenario_number = format!("SCP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let planning_end_date = planning_start_date.map(|d| {
            d + chrono::Duration::days(planning_horizon_days as i64)
        });

        info!("Creating planning scenario {} for org {}", scenario_number, org_id);

        self.repository.create_scenario(
            org_id, &scenario_number, name, description, scenario_type,
            planning_horizon_days, planning_start_date, planning_end_date,
            include_existing_supply, include_on_hand, include_wip,
            auto_firm, auto_firm_days, net_shortages_only, created_by,
        ).await
    }

    /// Get a planning scenario by ID
    pub async fn get_scenario(&self, id: Uuid) -> AtlasResult<Option<PlanningScenario>> {
        self.repository.get_scenario(id).await
    }

    /// Get a planning scenario by number
    pub async fn get_scenario_by_number(
        &self,
        org_id: Uuid,
        scenario_number: &str,
    ) -> AtlasResult<Option<PlanningScenario>> {
        self.repository.get_scenario_by_number(org_id, scenario_number).await
    }

    /// List planning scenarios
    pub async fn list_scenarios(
        &self,
        org_id: Uuid,
        scenario_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PlanningScenario>> {
        if let Some(st) = scenario_type {
            validate_enum("scenario type", st, VALID_SCENARIO_TYPES)?;
        }
        if let Some(s) = status {
            validate_enum("status", s, VALID_SCENARIO_STATUSES)?;
        }
        self.repository.list_scenarios(org_id, scenario_type, status).await
    }

    /// Run MRP for a scenario: nets supply vs demand, generates planned orders and exceptions
    pub async fn run_mrp(&self, scenario_id: Uuid) -> AtlasResult<PlanningScenario> {
        let mut scenario = self.repository.get_scenario(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planning scenario {} not found", scenario_id)
            ))?;

        if scenario.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot run scenario in '{}' status. Must be 'draft'.", scenario.status)
            ));
        }

        info!("Starting MRP run for scenario {}", scenario.scenario_number);

        // Mark as running
        scenario = self.repository.update_scenario_status(scenario_id, "running").await?;

        // Clear existing planned orders and exceptions for this scenario
        self.repository.delete_planned_orders_by_scenario(scenario_id).await?;
        self.repository.delete_exceptions_by_scenario(scenario_id).await?;

        // Get all supply/demand entries for this scenario
        let entries = self.repository.list_supply_demand_by_scenario(scenario_id).await?;

        // Group by item_id
        let mut item_ids: Vec<Uuid> = entries.iter().map(|e| e.item_id).collect();
        item_ids.sort();
        item_ids.dedup();

        let mut total_orders = 0i32;
        let mut total_exceptions = 0i32;

        for item_id in &item_ids {
            let item_entries: Vec<&SupplyDemandEntry> =
                entries.iter().filter(|e| e.item_id == *item_id).collect();

            // Separate supply and demand, sort by due_date
            let mut supply: Vec<&SupplyDemandEntry> = item_entries
                .iter()
                .filter(|e| e.entry_type == "supply")
                .copied()
                .collect();
            supply.sort_by_key(|e| e.due_date);

            let mut demand: Vec<&SupplyDemandEntry> = item_entries
                .iter()
                .filter(|e| e.entry_type == "demand")
                .copied()
                .collect();
            demand.sort_by_key(|e| e.due_date);

            // Get planning parameters
            let params = self.repository.get_planning_parameter_by_item(
                scenario.organization_id, *item_id,
            ).await?;

            let lead_time = params.as_ref().map(|p| p.lead_time_days).unwrap_or(0);
            let safety_stock: f64 = params.as_ref()
                .map(|p| p.safety_stock_quantity.parse().unwrap_or(0.0))
                .unwrap_or(0.0);
            let min_order: f64 = params.as_ref()
                .map(|p| p.min_order_quantity.parse().unwrap_or(0.0))
                .unwrap_or(0.0);
            let order_multiple: f64 = params.as_ref()
                .map(|p| p.order_multiple.parse().unwrap_or(1.0))
                .unwrap_or(1.0_f64).max(1.0);
            let make_buy = params.as_ref()
                .map(|p| p.make_buy.as_str())
                .unwrap_or("buy");
            let lot_policy = params.as_ref()
                .map(|p| p.lot_size_policy.as_str())
                .unwrap_or("lot_for_lot");

            let item_name = item_entries.first().and_then(|e| e.item_name.clone());
            let item_number = item_entries.first().and_then(|e| e.item_number.clone());

            // Calculate total supply and demand
            let total_supply: f64 = supply.iter()
                .filter(|e| e.status == "open")
                .map(|e| e.quantity_remaining.parse().unwrap_or(0.0))
                .sum();
            let total_demand: f64 = demand.iter()
                .filter(|e| e.status == "open")
                .map(|e| e.quantity_remaining.parse().unwrap_or(0.0))
                .sum();

            let net_position = total_supply - total_demand - safety_stock;

            // Generate shortage exception if net position is negative
            if net_position < 0.0 {
                let shortage_qty = net_position.abs();
                let _ex = self.repository.create_exception(
                    scenario.organization_id,
                    Some(scenario_id),
                    *item_id,
                    item_name.as_deref(),
                    item_number.as_deref(),
                    "shortage",
                    "critical",
                    &format!(
                        "Item {} has a net shortage of {:.2} (supply: {:.2}, demand: {:.2}, safety stock: {:.2})",
                        item_number.as_deref().unwrap_or("unknown"),
                        shortage_qty, total_supply, total_demand, safety_stock
                    ),
                    None, None, None,
                    Some(&format!("{:.2}", shortage_qty)),
                    scenario.planning_start_date,
                ).await?;
                total_exceptions += 1;
            }

            // Generate excess exception if net position is very positive
            if net_position > total_demand * 0.5 && total_demand > 0.0 {
                let _ex = self.repository.create_exception(
                    scenario.organization_id,
                    Some(scenario_id),
                    *item_id,
                    item_name.as_deref(),
                    item_number.as_deref(),
                    "excess_supply",
                    "warning",
                    &format!(
                        "Item {} has excess supply of {:.2} above demand + safety stock",
                        item_number.as_deref().unwrap_or("unknown"),
                        net_position
                    ),
                    None, None, None,
                    Some(&format!("{:.2}", net_position)),
                    scenario.planning_start_date,
                ).await?;
                total_exceptions += 1;
            }

            // Net supply against demand chronologically
            let mut supply_pool = total_supply;
            for dem in &demand {
                if dem.status != "open" { continue; }
                let dem_qty: f64 = dem.quantity_remaining.parse().unwrap_or(0.0);
                supply_pool -= dem_qty;

                if supply_pool < 0.0 && supply_pool + dem_qty >= 0.0 {
                    // This demand causes the shortage - generate planned order
                    let needed = supply_pool.abs();
                    let order_qty = Self::calculate_order_quantity(
                        needed, min_order, order_multiple, lot_policy,
                    );

                    if order_qty > 0.0 {
                        let due_date = dem.due_date;
                        let start_date = due_date - chrono::Duration::days(lead_time as i64);
                        let order_number = format!("PO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
                        let order_type = match make_buy {
                            "make" => "make",
                            _ => "buy",
                        };

                        let supplier_id = params.as_ref().and_then(|p| p.default_supplier_id);
                        let supplier_name = params.as_ref().and_then(|p| p.default_supplier_name.clone());

                        let firm_deadline = start_date;
                        let _order = self.repository.create_planned_order(
                            scenario.organization_id,
                            Some(scenario_id),
                            *item_id,
                            item_name.as_deref(),
                            item_number.as_deref(),
                            &order_number,
                            order_type,
                            "unfirm",
                            &format!("{:.2}", order_qty),
                            "0",
                            due_date,
                            Some(start_date),
                            Some(dem.due_date),
                            None,
                            5,
                            "new",
                            supplier_id,
                            supplier_name.as_deref(),
                            None, None,
                            Some(firm_deadline),
                            Some(dem.id),
                        ).await?;
                        total_orders += 1;
                    }
                }

                // Check for past due demand
                if let Some(start) = scenario.planning_start_date {
                    if dem.due_date < start {
                        let _ex = self.repository.create_exception(
                            scenario.organization_id,
                            Some(scenario_id),
                            *item_id,
                            item_name.as_deref(),
                            item_number.as_deref(),
                            "past_due_demand",
                            "error",
                            &format!(
                                "Item {} has past-due demand of {:.2} due on {}",
                                item_number.as_deref().unwrap_or("unknown"),
                                dem_qty,
                                dem.due_date
                            ),
                            Some(&dem.source_type),
                            dem.source_id,
                            dem.source_number.as_deref(),
                            Some(&format!("{:.2}", dem_qty)),
                            Some(dem.due_date),
                        ).await?;
                        total_exceptions += 1;
                    }
                }
            }

            // Check for safety stock replenishment
            if net_position < 0.0 && demand.is_empty() {
                // No demand but need safety stock
                let needed = net_position.abs();
                let order_qty = Self::calculate_order_quantity(
                    needed, min_order, order_multiple, lot_policy,
                );
                if order_qty > 0.0 {
                    let order_number = format!("PO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
                    let due_date = scenario.planning_start_date
                        .unwrap_or_else(|| chrono::Utc::now().date_naive());
                    let start_date = due_date - chrono::Duration::days(lead_time as i64);
                    let supplier_id = params.as_ref().and_then(|p| p.default_supplier_id);
                    let supplier_name = params.as_ref().and_then(|p| p.default_supplier_name.clone());

                    let _order = self.repository.create_planned_order(
                        scenario.organization_id,
                        Some(scenario_id),
                        *item_id,
                        item_name.as_deref(),
                        item_number.as_deref(),
                        &order_number,
                        match make_buy { "make" => "make", _ => "buy" },
                        "unfirm",
                        &format!("{:.2}", order_qty),
                        "0",
                        due_date,
                        Some(start_date),
                        None,
                        None,
                        5,
                        "new",
                        supplier_id,
                        supplier_name.as_deref(),
                        None, None,
                        Some(start_date),
                        None,
                    ).await?;
                    total_orders += 1;
                }
            }
        }

        // Update scenario totals and mark completed
        let completed = self.repository.update_scenario_results(
            scenario_id,
            "completed",
            total_orders,
            total_exceptions,
        ).await?;

        info!("MRP run completed for scenario {}: {} planned orders, {} exceptions",
            completed.scenario_number, total_orders, total_exceptions);

        Ok(completed)
    }

    /// Calculate order quantity based on lot sizing policy
    fn calculate_order_quantity(
        needed: f64,
        min_order: f64,
        order_multiple: f64,
        lot_policy: &str,
    ) -> f64 {
        if needed <= 0.0 { return 0.0; }
        let base = needed.max(min_order);
        match lot_policy {
            "lot_for_lot" => needed,
            "fixed_quantity" => base,
            "min_max" => base,
            _ => {
                // Round up to order_multiple
                if order_multiple > 0.0 {
                    (base / order_multiple).ceil() * order_multiple
                } else {
                    base
                }
            }
        }
    }

    /// Cancel a planning scenario
    pub async fn cancel_scenario(&self, scenario_id: Uuid) -> AtlasResult<PlanningScenario> {
        let scenario = self.repository.get_scenario(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planning scenario {} not found", scenario_id)
            ))?;

        if scenario.status == "completed" || scenario.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel scenario in '{}' status", scenario.status)
            ));
        }

        info!("Cancelling planning scenario {}", scenario.scenario_number);
        self.repository.update_scenario_status(scenario_id, "cancelled").await
    }

    // ========================================================================
    // Planning Parameters
    // ========================================================================

    /// Create or update planning parameters for an item
    pub async fn upsert_planning_parameter(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        item_name: Option<&str>,
        item_number: Option<&str>,
        planner_code: Option<&str>,
        planning_method: &str,
        make_buy: &str,
        lead_time_days: i32,
        safety_stock_quantity: &str,
        min_order_quantity: &str,
        max_order_quantity: Option<&str>,
        fixed_order_quantity: Option<&str>,
        lot_size_policy: &str,
        order_multiple: Option<&str>,
        default_supplier_id: Option<Uuid>,
        default_supplier_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningParameter> {
        validate_enum("planning method", planning_method, VALID_PLANNING_METHODS)?;
        validate_enum("make/buy", make_buy, VALID_MAKE_BUY)?;
        validate_enum("lot size policy", lot_size_policy, VALID_LOT_POLICIES)?;

        if lead_time_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Lead time cannot be negative".to_string(),
            ));
        }
        let ss: f64 = safety_stock_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Safety stock must be a valid number".to_string(),
        ))?;
        if ss < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Safety stock cannot be negative".to_string(),
            ));
        }
        let moq: f64 = min_order_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Min order quantity must be a valid number".to_string(),
        ))?;
        if moq < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Min order quantity cannot be negative".to_string(),
            ));
        }

        info!("Upserting planning parameters for item {} in org {}", item_id, org_id);

        self.repository.upsert_planning_parameter(
            org_id, item_id, item_name, item_number, planner_code,
            planning_method, make_buy, lead_time_days,
            &format!("{:.2}", ss),
            &format!("{:.2}", moq),
            max_order_quantity,
            fixed_order_quantity,
            lot_size_policy,
            order_multiple.unwrap_or("1"),
            default_supplier_id, default_supplier_name,
            created_by,
        ).await
    }

    /// Get planning parameters for an item
    pub async fn get_planning_parameter(
        &self,
        org_id: Uuid,
        item_id: Uuid,
    ) -> AtlasResult<Option<PlanningParameter>> {
        self.repository.get_planning_parameter_by_item(org_id, item_id).await
    }

    /// List all planning parameters for an org
    pub async fn list_planning_parameters(
        &self,
        org_id: Uuid,
    ) -> AtlasResult<Vec<PlanningParameter>> {
        self.repository.list_planning_parameters(org_id).await
    }

    /// Delete planning parameters for an item
    pub async fn delete_planning_parameter(&self, org_id: Uuid, item_id: Uuid) -> AtlasResult<()> {
        info!("Deleting planning parameters for item {} in org {}", item_id, org_id);
        self.repository.delete_planning_parameter(org_id, item_id).await
    }

    // ========================================================================
    // Supply/Demand Entries
    // ========================================================================

    /// Add a supply/demand entry
    pub async fn create_supply_demand_entry(
        &self,
        org_id: Uuid,
        scenario_id: Option<Uuid>,
        item_id: Uuid,
        item_name: Option<&str>,
        item_number: Option<&str>,
        entry_type: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        quantity: &str,
        due_date: chrono::NaiveDate,
        priority: Option<i32>,
    ) -> AtlasResult<SupplyDemandEntry> {
        validate_enum("entry type", entry_type, VALID_ENTRY_TYPES)?;
        validate_enum("source type", source_type, VALID_SOURCE_TYPES)?;

        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive".to_string(),
            ));
        }

        self.repository.create_supply_demand_entry(
            org_id, scenario_id, item_id, item_name, item_number,
            entry_type, source_type, source_id, source_number,
            &format!("{:.2}", qty),
            &format!("{:.2}", qty),
            due_date,
            priority.unwrap_or(5),
            "open",
        ).await
    }

    /// List supply/demand entries for a scenario
    pub async fn list_supply_demand(
        &self,
        scenario_id: Uuid,
        entry_type: Option<&str>,
    ) -> AtlasResult<Vec<SupplyDemandEntry>> {
        if let Some(et) = entry_type {
            validate_enum("entry type", et, VALID_ENTRY_TYPES)?;
        }
        self.repository.list_supply_demand_by_scenario_filtered(
            scenario_id, entry_type,
        ).await
    }

    // ========================================================================
    // Planned Orders
    // ========================================================================

    /// Get a planned order by ID
    pub async fn get_planned_order(&self, id: Uuid) -> AtlasResult<Option<PlannedOrder>> {
        self.repository.get_planned_order(id).await
    }

    /// List planned orders for a scenario
    pub async fn list_planned_orders(
        &self,
        scenario_id: Uuid,
        status: Option<&str>,
        order_type: Option<&str>,
    ) -> AtlasResult<Vec<PlannedOrder>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_ORDER_STATUSES)?;
        }
        if let Some(ot) = order_type {
            validate_enum("order type", ot, VALID_ORDER_TYPES)?;
        }
        self.repository.list_planned_orders(scenario_id, status, order_type).await
    }

    /// Firm a planned order (commit to executing it)
    pub async fn firm_planned_order(&self, order_id: Uuid) -> AtlasResult<PlannedOrder> {
        let order = self.repository.get_planned_order(order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planned order {} not found", order_id)
            ))?;

        if order.status != "unfirm" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot firm order in '{}' status. Must be 'unfirm'.", order.status)
            ));
        }

        info!("Firming planned order {}", order.order_number);
        let qty: f64 = order.quantity.parse().unwrap_or(0.0);
        self.repository.update_planned_order_status(
            order_id, "firmed", Some(&format!("{:.2}", qty)), None,
        ).await
    }

    /// Cancel a planned order
    pub async fn cancel_planned_order(&self, order_id: Uuid) -> AtlasResult<PlannedOrder> {
        let order = self.repository.get_planned_order(order_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planned order {} not found", order_id)
            ))?;

        if order.status == "released" || order.status == "completed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel order in '{}' status", order.status)
            ));
        }

        info!("Cancelling planned order {}", order.order_number);
        self.repository.update_planned_order_status(
            order_id, "cancelled", None, Some("Cancelled by planner"),
        ).await
    }

    // ========================================================================
    // Planning Exceptions
    // ========================================================================

    /// List planning exceptions for a scenario
    pub async fn list_exceptions(
        &self,
        scenario_id: Uuid,
        severity: Option<&str>,
        resolution_status: Option<&str>,
    ) -> AtlasResult<Vec<PlanningException>> {
        if let Some(s) = severity {
            validate_enum("severity", s, VALID_SEVERITIES)?;
        }
        if let Some(rs) = resolution_status {
            validate_enum("resolution status", rs, VALID_RESOLUTION_STATUSES)?;
        }
        self.repository.list_exceptions(scenario_id, severity, resolution_status).await
    }

    /// Resolve a planning exception
    pub async fn resolve_exception(
        &self,
        exception_id: Uuid,
        resolution: &str,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<PlanningException> {
        let ex = self.repository.get_exception(exception_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planning exception {} not found", exception_id)
            ))?;

        if ex.resolution_status == "resolved" {
            return Err(AtlasError::WorkflowError(
                "Exception is already resolved".to_string()
            ));
        }
        if resolution.trim().is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Resolution notes are required".to_string(),
            ));
        }

        info!("Resolving planning exception {} for item {}", exception_id, ex.item_id);
        self.repository.update_exception_resolution(
            exception_id, "resolved", Some(resolution), resolved_by,
        ).await
    }

    /// Dismiss a planning exception
    pub async fn dismiss_exception(
        &self,
        exception_id: Uuid,
        reason: Option<&str>,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<PlanningException> {
        let ex = self.repository.get_exception(exception_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Planning exception {} not found", exception_id)
            ))?;

        if ex.resolution_status == "resolved" {
            return Err(AtlasError::WorkflowError(
                "Exception is already resolved".to_string()
            ));
        }

        info!("Dismissing planning exception {}", exception_id);
        self.repository.update_exception_resolution(
            exception_id, "dismissed", reason, resolved_by,
        ).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get planning dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PlanningDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_scenario_types() {
        assert!(VALID_SCENARIO_TYPES.contains(&"mrp"));
        assert!(VALID_SCENARIO_TYPES.contains(&"distribution_planning"));
        assert!(VALID_SCENARIO_TYPES.contains(&"demand_planning"));
        assert!(VALID_SCENARIO_TYPES.contains(&"production_planning"));
    }

    #[test]
    fn test_valid_planning_methods() {
        assert!(VALID_PLANNING_METHODS.contains(&"mrp"));
        assert!(VALID_PLANNING_METHODS.contains(&"min_max"));
        assert!(VALID_PLANNING_METHODS.contains(&"reorder_point"));
        assert!(VALID_PLANNING_METHODS.contains(&"kanban"));
    }

    #[test]
    fn test_valid_order_types() {
        assert!(VALID_ORDER_TYPES.contains(&"buy"));
        assert!(VALID_ORDER_TYPES.contains(&"make"));
        assert!(VALID_ORDER_TYPES.contains(&"transfer"));
    }

    #[test]
    fn test_valid_exception_types() {
        assert!(VALID_EXCEPTION_TYPES.contains(&"shortage"));
        assert!(VALID_EXCEPTION_TYPES.contains(&"excess_supply"));
        assert!(VALID_EXCEPTION_TYPES.contains(&"late_order"));
        assert!(VALID_EXCEPTION_TYPES.contains(&"past_due_demand"));
    }

    #[test]
    fn test_calculate_order_quantity_lot_for_lot() {
        let qty = SupplyChainPlanningEngine::calculate_order_quantity(
            100.0, 50.0, 10.0, "lot_for_lot",
        );
        assert_eq!(qty, 100.0);
    }

    #[test]
    fn test_calculate_order_quantity_with_minimum() {
        let qty = SupplyChainPlanningEngine::calculate_order_quantity(
            30.0, 50.0, 10.0, "fixed_quantity",
        );
        assert_eq!(qty, 50.0); // min order wins
    }

    #[test]
    fn test_calculate_order_quantity_with_multiple() {
        let qty = SupplyChainPlanningEngine::calculate_order_quantity(
            95.0, 0.0, 25.0, "period_order_quantity",
        );
        assert_eq!(qty, 100.0); // round up to next multiple of 25
    }

    #[test]
    fn test_calculate_order_quantity_zero_needed() {
        let qty = SupplyChainPlanningEngine::calculate_order_quantity(
            0.0, 50.0, 10.0, "lot_for_lot",
        );
        assert_eq!(qty, 0.0);
    }

    #[test]
    fn test_calculate_order_quantity_negative() {
        let qty = SupplyChainPlanningEngine::calculate_order_quantity(
            -10.0, 50.0, 10.0, "lot_for_lot",
        );
        assert_eq!(qty, 0.0);
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test", "mrp", VALID_PLANNING_METHODS).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        assert!(validate_enum("test", "invalid", VALID_PLANNING_METHODS).is_err());
    }

    #[test]
    fn test_validate_enum_empty() {
        assert!(validate_enum("test", "", VALID_PLANNING_METHODS).is_err());
    }
}
