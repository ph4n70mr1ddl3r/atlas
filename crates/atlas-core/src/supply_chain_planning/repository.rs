//! Supply Chain Planning Repository
//!
//! PostgreSQL storage for planning scenarios, parameters, supply/demand entries,
//! planned orders, planning exceptions, and dashboard data.

use atlas_shared::{
    PlanningScenario, PlanningParameter, SupplyDemandEntry,
    PlannedOrder, PlanningException, PlanningDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for supply chain planning data storage
#[async_trait]
pub trait PlanningRepository: Send + Sync {
    // Scenarios
    async fn create_scenario(
        &self, org_id: Uuid, scenario_number: &str, name: &str, description: Option<&str>,
        scenario_type: &str, planning_horizon_days: i32,
        planning_start_date: Option<chrono::NaiveDate>, planning_end_date: Option<chrono::NaiveDate>,
        include_existing_supply: bool, include_on_hand: bool, include_wip: bool,
        auto_firm: bool, auto_firm_days: Option<i32>, net_shortages_only: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningScenario>;
    async fn get_scenario(&self, id: Uuid) -> AtlasResult<Option<PlanningScenario>>;
    async fn get_scenario_by_number(&self, org_id: Uuid, scenario_number: &str) -> AtlasResult<Option<PlanningScenario>>;
    async fn list_scenarios(&self, org_id: Uuid, scenario_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<PlanningScenario>>;
    async fn update_scenario_status(&self, id: Uuid, status: &str) -> AtlasResult<PlanningScenario>;
    async fn update_scenario_results(&self, id: Uuid, status: &str, total_planned_orders: i32, total_exceptions: i32) -> AtlasResult<PlanningScenario>;

    // Planning Parameters
    async fn upsert_planning_parameter(
        &self, org_id: Uuid, item_id: Uuid, item_name: Option<&str>, item_number: Option<&str>,
        planner_code: Option<&str>, planning_method: &str, make_buy: &str, lead_time_days: i32,
        safety_stock_quantity: &str, min_order_quantity: &str,
        max_order_quantity: Option<&str>, fixed_order_quantity: Option<&str>,
        lot_size_policy: &str, order_multiple: &str,
        default_supplier_id: Option<Uuid>, default_supplier_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningParameter>;
    async fn get_planning_parameter_by_item(&self, org_id: Uuid, item_id: Uuid) -> AtlasResult<Option<PlanningParameter>>;
    async fn list_planning_parameters(&self, org_id: Uuid) -> AtlasResult<Vec<PlanningParameter>>;
    async fn delete_planning_parameter(&self, org_id: Uuid, item_id: Uuid) -> AtlasResult<()>;

    // Supply/Demand
    async fn create_supply_demand_entry(
        &self, org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        entry_type: &str, source_type: &str,
        source_id: Option<Uuid>, source_number: Option<&str>,
        quantity: &str, quantity_remaining: &str,
        due_date: chrono::NaiveDate, priority: i32, status: &str,
    ) -> AtlasResult<SupplyDemandEntry>;
    async fn list_supply_demand_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<Vec<SupplyDemandEntry>>;
    async fn list_supply_demand_by_scenario_filtered(&self, scenario_id: Uuid, entry_type: Option<&str>) -> AtlasResult<Vec<SupplyDemandEntry>>;

    // Planned Orders
    async fn create_planned_order(
        &self, org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        order_number: &str, order_type: &str, status: &str,
        quantity: &str, quantity_firmed: &str,
        due_date: chrono::NaiveDate, start_date: Option<chrono::NaiveDate>,
        need_date: Option<chrono::NaiveDate>,
        planner_notes: Option<&str>, planning_priority: i32, order_action: &str,
        suggested_supplier_id: Option<Uuid>, suggested_supplier_name: Option<&str>,
        suggested_source_type: Option<&str>, suggested_source_id: Option<Uuid>,
        firm_deadline: Option<chrono::NaiveDate>,
        pegging_demand_id: Option<Uuid>,
    ) -> AtlasResult<PlannedOrder>;
    async fn get_planned_order(&self, id: Uuid) -> AtlasResult<Option<PlannedOrder>>;
    async fn list_planned_orders(&self, scenario_id: Uuid, status: Option<&str>, order_type: Option<&str>) -> AtlasResult<Vec<PlannedOrder>>;
    async fn update_planned_order_status(&self, id: Uuid, status: &str, quantity_firmed: Option<&str>, planner_notes: Option<&str>) -> AtlasResult<PlannedOrder>;
    async fn delete_planned_orders_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()>;

    // Exceptions
    async fn create_exception(
        &self, org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        exception_type: &str, severity: &str, message: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        affected_quantity: Option<&str>, affected_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<PlanningException>;
    async fn get_exception(&self, id: Uuid) -> AtlasResult<Option<PlanningException>>;
    async fn list_exceptions(&self, scenario_id: Uuid, severity: Option<&str>, resolution_status: Option<&str>) -> AtlasResult<Vec<PlanningException>>;
    async fn update_exception_resolution(&self, id: Uuid, resolution_status: &str, resolution_notes: Option<&str>, resolved_by: Option<Uuid>) -> AtlasResult<PlanningException>;
    async fn delete_exceptions_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PlanningDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresPlanningRepository {
    pool: PgPool,
}

impl PostgresPlanningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row mappers

fn row_to_scenario(row: &sqlx::postgres::PgRow) -> PlanningScenario {
    PlanningScenario {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_number: row.get("scenario_number"),
        name: row.get("name"),
        description: row.get("description"),
        scenario_type: row.get("scenario_type"),
        status: row.get("status"),
        planning_horizon_days: row.get("planning_horizon_days"),
        planning_start_date: row.get("planning_start_date"),
        planning_end_date: row.get("planning_end_date"),
        include_existing_supply: row.get("include_existing_supply"),
        include_on_hand: row.get("include_on_hand"),
        include_work_in_progress: row.get("include_work_in_progress"),
        auto_firm: row.get("auto_firm"),
        auto_firm_days: row.get("auto_firm_days"),
        net_shortages_only: row.get("net_shortages_only"),
        total_planned_orders: row.get("total_planned_orders"),
        total_exceptions: row.get("total_exceptions"),
        completed_at: row.get("completed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_parameter(row: &sqlx::postgres::PgRow) -> PlanningParameter {
    PlanningParameter {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        item_number: row.get("item_number"),
        planner_code: row.get("planner_code"),
        planning_method: row.get("planning_method"),
        make_buy: row.get("make_buy"),
        lead_time_days: row.get("lead_time_days"),
        safety_stock_quantity: get_num(row, "safety_stock_quantity"),
        min_order_quantity: get_num(row, "min_order_quantity"),
        max_order_quantity: row.try_get::<Option<f64>, _>("max_order_quantity").ok().flatten().map(|v| format!("{:.2}", v)),
        fixed_order_quantity: row.try_get::<Option<f64>, _>("fixed_order_quantity").ok().flatten().map(|v| format!("{:.2}", v)),
        fixed_lot_multiplier: get_num(row, "fixed_lot_multiplier"),
        order_multiple: get_num(row, "order_multiple"),
        planning_time_fence_days: row.get("planning_time_fence_days"),
        release_time_fence_days: row.get("release_time_fence_days"),
        shrinkage_rate: get_num(row, "shrinkage_rate"),
        lot_size_policy: row.get("lot_size_policy"),
        period_order_quantity_days: row.get("period_order_quantity_days"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_name: row.get("source_name"),
        default_supplier_id: row.get("default_supplier_id"),
        default_supplier_name: row.get("default_supplier_name"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_supply_demand(row: &sqlx::postgres::PgRow) -> SupplyDemandEntry {
    SupplyDemandEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        item_number: row.get("item_number"),
        entry_type: row.get("entry_type"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        quantity: get_num(row, "quantity"),
        quantity_remaining: get_num(row, "quantity_remaining"),
        due_date: row.get("due_date"),
        priority: row.get("priority"),
        status: row.get("status"),
        pegged_to_id: row.get("pegged_to_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_planned_order(row: &sqlx::postgres::PgRow) -> PlannedOrder {
    PlannedOrder {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        item_number: row.get("item_number"),
        order_number: row.get("order_number"),
        order_type: row.get("order_type"),
        status: row.get("status"),
        quantity: get_num(row, "quantity"),
        quantity_firmed: get_num(row, "quantity_firmed"),
        due_date: row.get("due_date"),
        start_date: row.get("start_date"),
        need_date: row.get("need_date"),
        planner_notes: row.get("planner_notes"),
        planning_priority: row.get("planning_priority"),
        order_action: row.get("order_action"),
        suggested_supplier_id: row.get("suggested_supplier_id"),
        suggested_supplier_name: row.get("suggested_supplier_name"),
        suggested_source_type: row.get("suggested_source_type"),
        suggested_source_id: row.get("suggested_source_id"),
        firm_deadline: row.get("firm_deadline"),
        pegging_demand_id: row.get("pegging_demand_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_exception(row: &sqlx::postgres::PgRow) -> PlanningException {
    PlanningException {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        scenario_id: row.get("scenario_id"),
        item_id: row.get("item_id"),
        item_name: row.get("item_name"),
        item_number: row.get("item_number"),
        exception_type: row.get("exception_type"),
        severity: row.get("severity"),
        message: row.get("message"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        affected_quantity: row.try_get::<Option<f64>, _>("affected_quantity").ok().flatten().map(|v| format!("{:.2}", v)),
        affected_date: row.get("affected_date"),
        resolution_status: row.get("resolution_status"),
        resolution_notes: row.get("resolution_notes"),
        resolved_by: row.get("resolved_by"),
        resolved_at: row.get("resolved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl PlanningRepository for PostgresPlanningRepository {
    // ========================================================================
    // Scenarios
    // ========================================================================

    async fn create_scenario(
        &self,
        org_id: Uuid, scenario_number: &str, name: &str, description: Option<&str>,
        scenario_type: &str, planning_horizon_days: i32,
        planning_start_date: Option<chrono::NaiveDate>, planning_end_date: Option<chrono::NaiveDate>,
        include_existing_supply: bool, include_on_hand: bool, include_wip: bool,
        auto_firm: bool, auto_firm_days: Option<i32>, net_shortages_only: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningScenario> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.planning_scenarios
                (organization_id, scenario_number, name, description,
                 scenario_type, planning_horizon_days,
                 planning_start_date, planning_end_date,
                 include_existing_supply, include_on_hand, include_work_in_progress,
                 auto_firm, auto_firm_days, net_shortages_only, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(scenario_number).bind(name).bind(description)
        .bind(scenario_type).bind(planning_horizon_days)
        .bind(planning_start_date).bind(planning_end_date)
        .bind(include_existing_supply).bind(include_on_hand).bind(include_wip)
        .bind(auto_firm).bind(auto_firm_days).bind(net_shortages_only)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scenario(&row))
    }

    async fn get_scenario(&self, id: Uuid) -> AtlasResult<Option<PlanningScenario>> {
        let row = sqlx::query("SELECT * FROM _atlas.planning_scenarios WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_scenario(&r)))
    }

    async fn get_scenario_by_number(&self, org_id: Uuid, scenario_number: &str) -> AtlasResult<Option<PlanningScenario>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.planning_scenarios WHERE organization_id = $1 AND scenario_number = $2"
        )
        .bind(org_id).bind(scenario_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_scenario(&r)))
    }

    async fn list_scenarios(&self, org_id: Uuid, scenario_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<PlanningScenario>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.planning_scenarios
            WHERE organization_id = $1
              AND ($2::text IS NULL OR scenario_type = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(scenario_type).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_scenario).collect())
    }

    async fn update_scenario_status(&self, id: Uuid, status: &str) -> AtlasResult<PlanningScenario> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.planning_scenarios
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scenario(&row))
    }

    async fn update_scenario_results(&self, id: Uuid, status: &str, total_planned_orders: i32, total_exceptions: i32) -> AtlasResult<PlanningScenario> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.planning_scenarios
            SET status = $2,
                total_planned_orders = $3,
                total_exceptions = $4,
                completed_at = now(),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(total_planned_orders).bind(total_exceptions)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_scenario(&row))
    }

    // ========================================================================
    // Planning Parameters
    // ========================================================================

    async fn upsert_planning_parameter(
        &self,
        org_id: Uuid, item_id: Uuid, item_name: Option<&str>, item_number: Option<&str>,
        planner_code: Option<&str>, planning_method: &str, make_buy: &str, lead_time_days: i32,
        safety_stock_quantity: &str, min_order_quantity: &str,
        max_order_quantity: Option<&str>, fixed_order_quantity: Option<&str>,
        lot_size_policy: &str, order_multiple: &str,
        default_supplier_id: Option<Uuid>, default_supplier_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanningParameter> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.planning_parameters
                (organization_id, item_id, item_name, item_number, planner_code,
                 planning_method, make_buy, lead_time_days,
                 safety_stock_quantity, min_order_quantity,
                 max_order_quantity, fixed_order_quantity,
                 lot_size_policy, order_multiple,
                 default_supplier_id, default_supplier_name, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::double precision, $10::double precision,
                    $11::double precision, $12::double precision,
                    $13, $14::double precision,
                    $15, $16, $17)
            ON CONFLICT (organization_id, item_id) DO UPDATE SET
                item_name = COALESCE($3, planning_parameters.item_name),
                item_number = COALESCE($4, planning_parameters.item_number),
                planner_code = COALESCE($5, planning_parameters.planner_code),
                planning_method = $6, make_buy = $7, lead_time_days = $8,
                safety_stock_quantity = $9::double precision,
                min_order_quantity = $10::double precision,
                max_order_quantity = $11::double precision,
                fixed_order_quantity = $12::double precision,
                lot_size_policy = $13, order_multiple = $14::double precision,
                default_supplier_id = $15,
                default_supplier_name = $16,
                is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(item_id).bind(item_name).bind(item_number)
        .bind(planner_code).bind(planning_method).bind(make_buy).bind(lead_time_days)
        .bind(safety_stock_quantity).bind(min_order_quantity)
        .bind(max_order_quantity).bind(fixed_order_quantity)
        .bind(lot_size_policy).bind(order_multiple)
        .bind(default_supplier_id).bind(default_supplier_name)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_parameter(&row))
    }

    async fn get_planning_parameter_by_item(&self, org_id: Uuid, item_id: Uuid) -> AtlasResult<Option<PlanningParameter>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.planning_parameters WHERE organization_id = $1 AND item_id = $2 AND is_active = true"
        )
        .bind(org_id).bind(item_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_parameter(&r)))
    }

    async fn list_planning_parameters(&self, org_id: Uuid) -> AtlasResult<Vec<PlanningParameter>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.planning_parameters WHERE organization_id = $1 AND is_active = true ORDER BY item_name"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_parameter).collect())
    }

    async fn delete_planning_parameter(&self, org_id: Uuid, item_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.planning_parameters SET is_active = false, updated_at = now() WHERE organization_id = $1 AND item_id = $2"
        )
        .bind(org_id).bind(item_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Supply/Demand Entries
    // ========================================================================

    async fn create_supply_demand_entry(
        &self,
        org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        entry_type: &str, source_type: &str,
        source_id: Option<Uuid>, source_number: Option<&str>,
        quantity: &str, quantity_remaining: &str,
        due_date: chrono::NaiveDate, priority: i32, status: &str,
    ) -> AtlasResult<SupplyDemandEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.supply_demand_entries
                (organization_id, scenario_id, item_id, item_name, item_number,
                 entry_type, source_type, source_id, source_number,
                 quantity, quantity_remaining, due_date, priority, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::double precision, $11::double precision, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(scenario_id).bind(item_id).bind(item_name).bind(item_number)
        .bind(entry_type).bind(source_type).bind(source_id).bind(source_number)
        .bind(quantity).bind(quantity_remaining).bind(due_date).bind(priority).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_supply_demand(&row))
    }

    async fn list_supply_demand_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<Vec<SupplyDemandEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.supply_demand_entries WHERE scenario_id = $1 ORDER BY item_id, due_date"
        )
        .bind(scenario_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_supply_demand).collect())
    }

    async fn list_supply_demand_by_scenario_filtered(&self, scenario_id: Uuid, entry_type: Option<&str>) -> AtlasResult<Vec<SupplyDemandEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.supply_demand_entries
            WHERE scenario_id = $1
              AND ($2::text IS NULL OR entry_type = $2)
            ORDER BY due_date
            "#,
        )
        .bind(scenario_id).bind(entry_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_supply_demand).collect())
    }

    // ========================================================================
    // Planned Orders
    // ========================================================================

    async fn create_planned_order(
        &self,
        org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        order_number: &str, order_type: &str, status: &str,
        quantity: &str, quantity_firmed: &str,
        due_date: chrono::NaiveDate, start_date: Option<chrono::NaiveDate>,
        need_date: Option<chrono::NaiveDate>,
        planner_notes: Option<&str>, planning_priority: i32, order_action: &str,
        suggested_supplier_id: Option<Uuid>, suggested_supplier_name: Option<&str>,
        suggested_source_type: Option<&str>, suggested_source_id: Option<Uuid>,
        firm_deadline: Option<chrono::NaiveDate>,
        pegging_demand_id: Option<Uuid>,
    ) -> AtlasResult<PlannedOrder> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.planned_orders
                (organization_id, scenario_id, item_id, item_name, item_number,
                 order_number, order_type, status,
                 quantity, quantity_firmed,
                 due_date, start_date, need_date,
                 planner_notes, planning_priority, order_action,
                 suggested_supplier_id, suggested_supplier_name,
                 suggested_source_type, suggested_source_id,
                 firm_deadline, pegging_demand_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::double precision, $10::double precision,
                    $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(scenario_id).bind(item_id).bind(item_name).bind(item_number)
        .bind(order_number).bind(order_type).bind(status)
        .bind(quantity).bind(quantity_firmed)
        .bind(due_date).bind(start_date).bind(need_date)
        .bind(planner_notes).bind(planning_priority).bind(order_action)
        .bind(suggested_supplier_id).bind(suggested_supplier_name)
        .bind(suggested_source_type).bind(suggested_source_id)
        .bind(firm_deadline).bind(pegging_demand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_planned_order(&row))
    }

    async fn get_planned_order(&self, id: Uuid) -> AtlasResult<Option<PlannedOrder>> {
        let row = sqlx::query("SELECT * FROM _atlas.planned_orders WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_planned_order(&r)))
    }

    async fn list_planned_orders(&self, scenario_id: Uuid, status: Option<&str>, order_type: Option<&str>) -> AtlasResult<Vec<PlannedOrder>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.planned_orders
            WHERE scenario_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR order_type = $3)
            ORDER BY due_date, planning_priority
            "#,
        )
        .bind(scenario_id).bind(status).bind(order_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_planned_order).collect())
    }

    async fn update_planned_order_status(&self, id: Uuid, status: &str, quantity_firmed: Option<&str>, planner_notes: Option<&str>) -> AtlasResult<PlannedOrder> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.planned_orders
            SET status = $2,
                quantity_firmed = COALESCE($3::double precision, quantity_firmed),
                planner_notes = COALESCE($4, planner_notes),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(quantity_firmed).bind(planner_notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_planned_order(&row))
    }

    async fn delete_planned_orders_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.planned_orders WHERE scenario_id = $1")
            .bind(scenario_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Exceptions
    // ========================================================================

    async fn create_exception(
        &self,
        org_id: Uuid, scenario_id: Option<Uuid>, item_id: Uuid,
        item_name: Option<&str>, item_number: Option<&str>,
        exception_type: &str, severity: &str, message: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        affected_quantity: Option<&str>, affected_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<PlanningException> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.planning_exceptions
                (organization_id, scenario_id, item_id, item_name, item_number,
                 exception_type, severity, message,
                 source_type, source_id, source_number,
                 affected_quantity, affected_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9, $10, $11, $12::double precision, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(scenario_id).bind(item_id).bind(item_name).bind(item_number)
        .bind(exception_type).bind(severity).bind(message)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(affected_quantity).bind(affected_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_exception(&row))
    }

    async fn get_exception(&self, id: Uuid) -> AtlasResult<Option<PlanningException>> {
        let row = sqlx::query("SELECT * FROM _atlas.planning_exceptions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_exception(&r)))
    }

    async fn list_exceptions(&self, scenario_id: Uuid, severity: Option<&str>, resolution_status: Option<&str>) -> AtlasResult<Vec<PlanningException>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.planning_exceptions
            WHERE scenario_id = $1
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::text IS NULL OR resolution_status = $3)
            ORDER BY
                CASE severity
                    WHEN 'critical' THEN 1
                    WHEN 'error' THEN 2
                    WHEN 'warning' THEN 3
                    WHEN 'info' THEN 4
                END,
                created_at DESC
            "#,
        )
        .bind(scenario_id).bind(severity).bind(resolution_status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_exception).collect())
    }

    async fn update_exception_resolution(
        &self,
        id: Uuid, resolution_status: &str, resolution_notes: Option<&str>, resolved_by: Option<Uuid>,
    ) -> AtlasResult<PlanningException> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.planning_exceptions
            SET resolution_status = $2,
                resolution_notes = COALESCE($3, resolution_notes),
                resolved_by = $4,
                resolved_at = now(),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(resolution_status).bind(resolution_notes).bind(resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_exception(&row))
    }

    async fn delete_exceptions_by_scenario(&self, scenario_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.planning_exceptions WHERE scenario_id = $1")
            .bind(scenario_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PlanningDashboard> {
        let total_scenarios: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planning_scenarios WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_scenarios: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planning_scenarios WHERE organization_id = $1 AND status IN ('draft', 'running')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_planned_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planned_orders WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let unfirm_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planned_orders WHERE organization_id = $1 AND status = 'unfirm'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let firmed_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planned_orders WHERE organization_id = $1 AND status = 'firmed'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_exceptions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planning_exceptions WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let critical_exceptions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planning_exceptions WHERE organization_id = $1 AND severity IN ('critical', 'error')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let open_exceptions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.planning_exceptions WHERE organization_id = $1 AND resolution_status = 'open'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_items_planned: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT item_id) FROM _atlas.planning_parameters WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let items_with_shortage: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT item_id) FROM _atlas.planning_exceptions WHERE organization_id = $1 AND exception_type = 'shortage' AND resolution_status = 'open'"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PlanningDashboard {
            organization_id: org_id,
            total_scenarios,
            active_scenarios,
            total_planned_orders,
            unfirm_orders,
            firmed_orders,
            total_exceptions,
            critical_exceptions,
            open_exceptions,
            total_items_planned,
            items_with_shortage,
        })
    }
}
