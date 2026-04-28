//! Enterprise Asset Management Repository
//!
//! PostgreSQL storage for asset locations, asset definitions, work orders,
//! preventive maintenance schedules, and maintenance dashboard.

use atlas_shared::{
    AssetDefinition, MaintenanceWorkOrder, PreventiveMaintenanceSchedule, MaintenanceDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for asset management data storage
#[async_trait]
pub trait AssetManagementRepository: Send + Sync {
    // Asset Locations
    async fn create_location(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_location_id: Option<Uuid>, location_type: &str, address: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<serde_json::Value>;
    async fn get_location(&self, id: Uuid) -> AtlasResult<Option<serde_json::Value>>;
    async fn get_location_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<serde_json::Value>>;
    async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<serde_json::Value>>;
    async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Asset Definitions
    async fn create_asset(
        &self, org_id: Uuid, asset_number: &str, name: &str, description: Option<&str>,
        asset_group: &str, asset_criticality: &str,
        location_id: Option<Uuid>, location_name: Option<&str>,
        parent_asset_id: Option<Uuid>,
        serial_number: &str, manufacturer: &str, model: &str,
        install_date: Option<chrono::NaiveDate>, warranty_expiry: Option<chrono::NaiveDate>,
        meter_reading: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDefinition>;
    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<AssetDefinition>>;
    async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<AssetDefinition>>;
    async fn list_assets(
        &self, org_id: Uuid, status: Option<&str>, asset_group: Option<&str>, criticality: Option<&str>,
    ) -> AtlasResult<Vec<AssetDefinition>>;
    async fn update_asset_status(&self, id: Uuid, status: &str) -> AtlasResult<AssetDefinition>;
    async fn update_asset_meter(&self, id: Uuid, meter_reading: serde_json::Value) -> AtlasResult<AssetDefinition>;
    async fn delete_asset(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<()>;

    // Work Orders
    async fn create_work_order(
        &self, org_id: Uuid, work_order_number: &str, title: &str, description: Option<&str>,
        work_order_type: &str, priority: &str,
        asset_id: Uuid, asset_number: &str, asset_name: &str, location_name: &str,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        scheduled_start: Option<chrono::NaiveDate>, scheduled_end: Option<chrono::NaiveDate>,
        estimated_hours: serde_json::Value, estimated_cost: &str,
        failure_code: &str, cause_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MaintenanceWorkOrder>;
    async fn get_work_order(&self, id: Uuid) -> AtlasResult<Option<MaintenanceWorkOrder>>;
    async fn get_work_order_by_number(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<Option<MaintenanceWorkOrder>>;
    async fn list_work_orders(
        &self, org_id: Uuid, status: Option<&str>, work_order_type: Option<&str>,
        priority: Option<&str>, asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<MaintenanceWorkOrder>>;
    async fn update_work_order_status(&self, id: Uuid, status: &str) -> AtlasResult<MaintenanceWorkOrder>;
    async fn complete_work_order(
        &self, id: Uuid, actual_cost: &str, actual_hours: serde_json::Value,
        downtime_hours: f64, resolution_code: &str, completion_notes: &str,
        materials: serde_json::Value, labor: serde_json::Value,
    ) -> AtlasResult<MaintenanceWorkOrder>;
    async fn delete_work_order(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<()>;

    // Preventive Maintenance Schedules
    async fn create_pm_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str, description: Option<&str>,
        asset_id: Uuid, asset_number: &str, asset_name: &str,
        schedule_type: &str, frequency: &str,
        interval_value: i32, interval_unit: &str,
        meter_type: &str, meter_threshold: serde_json::Value,
        work_order_template: serde_json::Value,
        estimated_duration_hours: f64, estimated_cost: &str,
        auto_generate: bool, lead_time_days: i32,
        effective_start: Option<chrono::NaiveDate>, effective_end: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PreventiveMaintenanceSchedule>;
    async fn get_pm_schedule(&self, id: Uuid) -> AtlasResult<Option<PreventiveMaintenanceSchedule>>;
    async fn get_pm_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<PreventiveMaintenanceSchedule>>;
    async fn list_pm_schedules(
        &self, org_id: Uuid, status: Option<&str>, asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PreventiveMaintenanceSchedule>>;
    async fn update_pm_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<PreventiveMaintenanceSchedule>;
    async fn delete_pm_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MaintenanceDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresAssetManagementRepository {
    pool: PgPool,
}

impl PostgresAssetManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper: map row to AssetDefinition
fn row_to_asset(row: &sqlx::postgres::PgRow) -> AssetDefinition {
    AssetDefinition {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        asset_number: row.try_get("asset_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        asset_group: row.try_get("asset_group").unwrap_or_default(),
        asset_criticality: row.try_get("asset_criticality").unwrap_or_default(),
        asset_status: row.try_get("asset_status").unwrap_or_default(),
        location_id: row.try_get("location_id").unwrap_or_default(),
        location_name: row.try_get("location_name").unwrap_or_default(),
        parent_asset_id: row.try_get("parent_asset_id").unwrap_or_default(),
        serial_number: row.try_get("serial_number").unwrap_or_default(),
        manufacturer: row.try_get("manufacturer").unwrap_or_default(),
        model: row.try_get("model").unwrap_or_default(),
        install_date: row.try_get("install_date").unwrap_or_default(),
        warranty_expiry: row.try_get("warranty_expiry").unwrap_or_default(),
        last_maintenance_date: row.try_get("last_maintenance_date").unwrap_or_default(),
        next_maintenance_date: row.try_get("next_maintenance_date").unwrap_or_default(),
        meter_reading: row.try_get("meter_reading").unwrap_or(Some(serde_json::json!({}))),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// Helper: map row to MaintenanceWorkOrder
fn row_to_work_order(row: &sqlx::postgres::PgRow) -> MaintenanceWorkOrder {
    MaintenanceWorkOrder {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        work_order_number: row.try_get("work_order_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        work_order_type: row.try_get("work_order_type").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        asset_id: row.try_get("asset_id").unwrap_or_default(),
        asset_number: row.try_get("asset_number").unwrap_or_default(),
        asset_name: row.try_get("asset_name").unwrap_or_default(),
        location_name: row.try_get("location_name").unwrap_or_default(),
        assigned_to: row.try_get("assigned_to").unwrap_or_default(),
        assigned_to_name: row.try_get("assigned_to_name").unwrap_or_default(),
        scheduled_start: row.try_get("scheduled_start").unwrap_or_default(),
        scheduled_end: row.try_get("scheduled_end").unwrap_or_default(),
        actual_start: row.try_get("actual_start").unwrap_or_default(),
        actual_end: row.try_get("actual_end").unwrap_or_default(),
        estimated_hours: row.try_get("estimated_hours").unwrap_or(Some(serde_json::json!({}))),
        actual_hours: row.try_get("actual_hours").unwrap_or(Some(serde_json::json!({}))),
        estimated_cost: row.try_get("estimated_cost").unwrap_or_default(),
        actual_cost: row.try_get("actual_cost").unwrap_or_default(),
        downtime_hours: row.try_get("downtime_hours").unwrap_or(0.0),
        failure_code: row.try_get("failure_code").unwrap_or_default(),
        cause_code: row.try_get("cause_code").unwrap_or_default(),
        resolution_code: row.try_get("resolution_code").unwrap_or_default(),
        materials: row.try_get("materials").unwrap_or(serde_json::json!([])),
        labor: row.try_get("labor").unwrap_or(serde_json::json!([])),
        completion_notes: row.try_get("completion_notes").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        closed_at: row.try_get("closed_at").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// Helper: map row to PreventiveMaintenanceSchedule
fn row_to_pm_schedule(row: &sqlx::postgres::PgRow) -> PreventiveMaintenanceSchedule {
    PreventiveMaintenanceSchedule {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        schedule_number: row.try_get("schedule_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        asset_id: row.try_get("asset_id").unwrap_or_default(),
        asset_number: row.try_get("asset_number").unwrap_or_default(),
        asset_name: row.try_get("asset_name").unwrap_or_default(),
        schedule_type: row.try_get("schedule_type").unwrap_or_default(),
        frequency: row.try_get("frequency").unwrap_or_default(),
        interval_value: row.try_get("interval_value").unwrap_or(1),
        interval_unit: row.try_get("interval_unit").unwrap_or_default(),
        meter_type: row.try_get("meter_type").unwrap_or_default(),
        meter_threshold: row.try_get("meter_threshold").unwrap_or(Some(serde_json::json!({}))),
        work_order_template: row.try_get("work_order_template").unwrap_or(Some(serde_json::json!({}))),
        estimated_duration_hours: row.try_get("estimated_duration_hours").unwrap_or(0.0),
        estimated_cost: row.try_get("estimated_cost").unwrap_or_default(),
        next_due_date: row.try_get("next_due_date").unwrap_or_default(),
        last_completed_date: row.try_get("last_completed_date").unwrap_or_default(),
        last_completed_wo: row.try_get("last_completed_wo").unwrap_or_default(),
        auto_generate: row.try_get("auto_generate").unwrap_or(false),
        lead_time_days: row.try_get("lead_time_days").unwrap_or(7),
        status: row.try_get("status").unwrap_or_default(),
        effective_start: row.try_get("effective_start").unwrap_or_default(),
        effective_end: row.try_get("effective_end").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl AssetManagementRepository for PostgresAssetManagementRepository {
    // ========================================================================
    // Asset Locations
    // ========================================================================

    async fn create_location(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_location_id: Option<Uuid>, location_type: &str, address: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<serde_json::Value> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.asset_locations
                (organization_id, code, name, description, parent_location_id,
                 location_type, address, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, '{}'::jsonb, $8)
            RETURNING id, organization_id, code, name, description, location_type, address, is_active, created_at"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_location_id).bind(location_type).bind(address).bind(created_by)
        .fetch_one(&self.pool).await?;

        Ok(serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "organizationId": row.try_get::<Uuid, _>("organization_id").unwrap_or_default(),
            "code": row.try_get::<String, _>("code").unwrap_or_default(),
            "name": row.try_get::<String, _>("name").unwrap_or_default(),
            "description": row.try_get::<Option<String>, _>("description").unwrap_or_default(),
            "locationType": row.try_get::<String, _>("location_type").unwrap_or_default(),
            "address": row.try_get::<Option<String>, _>("address").unwrap_or_default(),
            "isActive": row.try_get::<bool, _>("is_active").unwrap_or(true),
            "createdAt": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or(chrono::Utc::now()),
        }))
    }

    async fn get_location(&self, id: Uuid) -> AtlasResult<Option<serde_json::Value>> {
        let row = sqlx::query("SELECT * FROM _atlas.asset_locations WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(|r| serde_json::json!({
            "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
            "code": r.try_get::<String, _>("code").unwrap_or_default(),
            "name": r.try_get::<String, _>("name").unwrap_or_default(),
        })))
    }

    async fn get_location_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<serde_json::Value>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_locations WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(|r| serde_json::json!({
            "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
            "code": r.try_get::<String, _>("code").unwrap_or_default(),
        })))
    }

    async fn list_locations(&self, org_id: Uuid) -> AtlasResult<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.asset_locations WHERE organization_id = $1 AND is_active = true ORDER BY created_at"
        ).bind(org_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(|r| serde_json::json!({
            "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
            "code": r.try_get::<String, _>("code").unwrap_or_default(),
            "name": r.try_get::<String, _>("name").unwrap_or_default(),
            "locationType": r.try_get::<String, _>("location_type").unwrap_or_default(),
        })).collect())
    }

    async fn delete_location(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.asset_locations WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Location '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Asset Definitions
    // ========================================================================

    async fn create_asset(
        &self, org_id: Uuid, asset_number: &str, name: &str, description: Option<&str>,
        asset_group: &str, asset_criticality: &str,
        location_id: Option<Uuid>, location_name: Option<&str>,
        parent_asset_id: Option<Uuid>,
        serial_number: &str, manufacturer: &str, model: &str,
        install_date: Option<chrono::NaiveDate>, warranty_expiry: Option<chrono::NaiveDate>,
        meter_reading: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDefinition> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.asset_definitions
                (organization_id, asset_number, name, description,
                 asset_group, asset_criticality,
                 location_id, location_name, parent_asset_id,
                 serial_number, manufacturer, model,
                 install_date, warranty_expiry, meter_reading,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                    $13, $14, $15, '{}'::jsonb, $16)
            RETURNING *"#,
        )
        .bind(org_id).bind(asset_number).bind(name).bind(description)
        .bind(asset_group).bind(asset_criticality)
        .bind(location_id).bind(location_name).bind(parent_asset_id)
        .bind(serial_number).bind(manufacturer).bind(model)
        .bind(install_date).bind(warranty_expiry).bind(&meter_reading)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_asset(&row))
    }

    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<AssetDefinition>> {
        let row = sqlx::query("SELECT * FROM _atlas.asset_definitions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_asset))
    }

    async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<AssetDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_definitions WHERE organization_id = $1 AND asset_number = $2"
        ).bind(org_id).bind(asset_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_asset))
    }

    async fn list_assets(
        &self, org_id: Uuid, status: Option<&str>, asset_group: Option<&str>, criticality: Option<&str>,
    ) -> AtlasResult<Vec<AssetDefinition>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.asset_definitions
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR asset_status = $2)
                 AND ($3::text IS NULL OR asset_group = $3)
                 AND ($4::text IS NULL OR asset_criticality = $4)
               ORDER BY asset_criticality, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(asset_group).bind(criticality)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_asset).collect())
    }

    async fn update_asset_status(&self, id: Uuid, status: &str) -> AtlasResult<AssetDefinition> {
        let row = sqlx::query(
            "UPDATE _atlas.asset_definitions SET asset_status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Asset {} not found", id)))?;
        Ok(row_to_asset(&row))
    }

    async fn update_asset_meter(&self, id: Uuid, meter_reading: serde_json::Value) -> AtlasResult<AssetDefinition> {
        let row = sqlx::query(
            "UPDATE _atlas.asset_definitions SET meter_reading = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(&meter_reading)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Asset {} not found", id)))?;
        Ok(row_to_asset(&row))
    }

    async fn delete_asset(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.asset_definitions WHERE organization_id = $1 AND asset_number = $2"
        ).bind(org_id).bind(asset_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Asset '{}' not found", asset_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Work Orders
    // ========================================================================

    async fn create_work_order(
        &self, org_id: Uuid, work_order_number: &str, title: &str, description: Option<&str>,
        work_order_type: &str, priority: &str,
        asset_id: Uuid, asset_number: &str, asset_name: &str, location_name: &str,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        scheduled_start: Option<chrono::NaiveDate>, scheduled_end: Option<chrono::NaiveDate>,
        estimated_hours: serde_json::Value, estimated_cost: &str,
        failure_code: &str, cause_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MaintenanceWorkOrder> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.work_orders
                (organization_id, work_order_number, title, description,
                 work_order_type, priority, status,
                 asset_id, asset_number, asset_name, location_name,
                 assigned_to, assigned_to_name,
                 scheduled_start, scheduled_end,
                 estimated_hours, estimated_cost,
                 failure_code, cause_code,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, 'draft',
                    $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, '{}'::jsonb, $19)
            RETURNING *"#,
        )
        .bind(org_id).bind(work_order_number).bind(title).bind(description)
        .bind(work_order_type).bind(priority)
        .bind(asset_id).bind(asset_number).bind(asset_name).bind(location_name)
        .bind(assigned_to).bind(assigned_to_name)
        .bind(scheduled_start).bind(scheduled_end)
        .bind(&estimated_hours).bind(estimated_cost)
        .bind(failure_code).bind(cause_code)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_work_order(&row))
    }

    async fn get_work_order(&self, id: Uuid) -> AtlasResult<Option<MaintenanceWorkOrder>> {
        let row = sqlx::query("SELECT * FROM _atlas.work_orders WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_work_order))
    }

    async fn get_work_order_by_number(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<Option<MaintenanceWorkOrder>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.work_orders WHERE organization_id = $1 AND work_order_number = $2"
        ).bind(org_id).bind(wo_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_work_order))
    }

    async fn list_work_orders(
        &self, org_id: Uuid, status: Option<&str>, work_order_type: Option<&str>,
        priority: Option<&str>, asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<MaintenanceWorkOrder>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.work_orders
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR work_order_type = $3)
                 AND ($4::text IS NULL OR priority = $4)
                 AND ($5::uuid IS NULL OR asset_id = $5)
               ORDER BY
                 CASE priority WHEN 'urgent' THEN 1 WHEN 'high' THEN 2 WHEN 'normal' THEN 3 ELSE 4 END,
                 created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(work_order_type).bind(priority).bind(asset_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_work_order).collect())
    }

    async fn update_work_order_status(&self, id: Uuid, status: &str) -> AtlasResult<MaintenanceWorkOrder> {
        let row = sqlx::query(
            r#"UPDATE _atlas.work_orders SET status = $2,
                actual_start = CASE WHEN $3 THEN now() ELSE actual_start END,
                actual_end = CASE WHEN $4 THEN now() ELSE actual_end END,
                closed_at = CASE WHEN $5 THEN now() ELSE closed_at END,
                approved_at = CASE WHEN $6 THEN now() ELSE approved_at END,
                updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .bind(status == "in_progress")
        .bind(status == "completed")
        .bind(status == "closed")
        .bind(status == "approved")
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;
        Ok(row_to_work_order(&row))
    }

    async fn complete_work_order(
        &self, id: Uuid, actual_cost: &str, actual_hours: serde_json::Value,
        downtime_hours: f64, resolution_code: &str, completion_notes: &str,
        materials: serde_json::Value, labor: serde_json::Value,
    ) -> AtlasResult<MaintenanceWorkOrder> {
        let row = sqlx::query(
            r#"UPDATE _atlas.work_orders
               SET actual_cost = $2, actual_hours = $3, downtime_hours = $4,
                   resolution_code = $5, completion_notes = $6,
                   materials = $7, labor = $8,
                   status = 'completed', actual_end = now(),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(actual_cost).bind(&actual_hours).bind(downtime_hours)
        .bind(resolution_code).bind(completion_notes)
        .bind(&materials).bind(&labor)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Work order {} not found", id)))?;
        Ok(row_to_work_order(&row))
    }

    async fn delete_work_order(&self, org_id: Uuid, wo_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.work_orders WHERE organization_id = $1 AND work_order_number = $2"
        ).bind(org_id).bind(wo_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Work order '{}' not found", wo_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Preventive Maintenance Schedules
    // ========================================================================

    async fn create_pm_schedule(
        &self, org_id: Uuid, schedule_number: &str, name: &str, description: Option<&str>,
        asset_id: Uuid, asset_number: &str, asset_name: &str,
        schedule_type: &str, frequency: &str,
        interval_value: i32, interval_unit: &str,
        meter_type: &str, meter_threshold: serde_json::Value,
        work_order_template: serde_json::Value,
        estimated_duration_hours: f64, estimated_cost: &str,
        auto_generate: bool, lead_time_days: i32,
        effective_start: Option<chrono::NaiveDate>, effective_end: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PreventiveMaintenanceSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.preventive_maintenance_schedules
                (organization_id, schedule_number, name, description,
                 asset_id, asset_number, asset_name,
                 schedule_type, frequency, interval_value, interval_unit,
                 meter_type, meter_threshold, work_order_template,
                 estimated_duration_hours, estimated_cost,
                 auto_generate, lead_time_days,
                 effective_start, effective_end,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    '{}'::jsonb, $21)
            RETURNING *"#,
        )
        .bind(org_id).bind(schedule_number).bind(name).bind(description)
        .bind(asset_id).bind(asset_number).bind(asset_name)
        .bind(schedule_type).bind(frequency).bind(interval_value).bind(interval_unit)
        .bind(meter_type).bind(&meter_threshold).bind(&work_order_template)
        .bind(estimated_duration_hours).bind(estimated_cost)
        .bind(auto_generate).bind(lead_time_days)
        .bind(effective_start).bind(effective_end)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_pm_schedule(&row))
    }

    async fn get_pm_schedule(&self, id: Uuid) -> AtlasResult<Option<PreventiveMaintenanceSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.preventive_maintenance_schedules WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_pm_schedule))
    }

    async fn get_pm_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<PreventiveMaintenanceSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.preventive_maintenance_schedules WHERE organization_id = $1 AND schedule_number = $2"
        ).bind(org_id).bind(schedule_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_pm_schedule))
    }

    async fn list_pm_schedules(
        &self, org_id: Uuid, status: Option<&str>, asset_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PreventiveMaintenanceSchedule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.preventive_maintenance_schedules
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::uuid IS NULL OR asset_id = $3)
               ORDER BY next_due_date, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(asset_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_pm_schedule).collect())
    }

    async fn update_pm_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<PreventiveMaintenanceSchedule> {
        let row = sqlx::query(
            "UPDATE _atlas.preventive_maintenance_schedules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("PM schedule {} not found", id)))?;
        Ok(row_to_pm_schedule(&row))
    }

    async fn delete_pm_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.preventive_maintenance_schedules WHERE organization_id = $1 AND schedule_number = $2"
        ).bind(org_id).bind(schedule_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Schedule '{}' not found", schedule_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<MaintenanceDashboard> {
        // Count assets
        let asset_rows = sqlx::query(
            "SELECT asset_status, asset_criticality FROM _atlas.asset_definitions WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut active_assets = 0i32;
        let mut assets_in_repair = 0i32;
        let mut critical_assets = 0i32;
        let mut assets_by_criticality = std::collections::HashMap::new();

        for row in &asset_rows {
            let status: String = row.try_get("asset_status").unwrap_or_default();
            let crit: String = row.try_get("asset_criticality").unwrap_or_default();
            match status.as_str() {
                "active" => active_assets += 1,
                "in_repair" => assets_in_repair += 1,
                _ => {}
            }
            if crit == "critical" { critical_assets += 1; }
            *assets_by_criticality.entry(crit).or_insert(0i32) += 1;
        }

        // Count work orders
        let wo_rows = sqlx::query(
            "SELECT status, work_order_type, priority FROM _atlas.work_orders WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut open_work_orders = 0i32;
        let mut in_progress_work_orders = 0i32;
        let mut completed_work_orders = 0i32;
        let mut emergency_work_orders = 0i32;
        let mut preventive_work_orders = 0i32;
        let mut corrective_work_orders = 0i32;
        let mut wo_by_priority = std::collections::HashMap::new();
        let mut wo_by_type = std::collections::HashMap::new();

        for row in &wo_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let wo_type: String = row.try_get("work_order_type").unwrap_or_default();
            let pri: String = row.try_get("priority").unwrap_or_default();

            match status.as_str() {
                "draft" | "approved" => open_work_orders += 1,
                "in_progress" => in_progress_work_orders += 1,
                "completed" | "closed" => completed_work_orders += 1,
                _ => {}
            }
            match wo_type.as_str() {
                "emergency" => emergency_work_orders += 1,
                "preventive" => preventive_work_orders += 1,
                "corrective" => corrective_work_orders += 1,
                _ => {}
            }
            *wo_by_priority.entry(pri).or_insert(0i32) += 1;
            *wo_by_type.entry(wo_type).or_insert(0i32) += 1;
        }

        // Count schedules
        let sched_rows = sqlx::query(
            "SELECT status FROM _atlas.preventive_maintenance_schedules WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut active_schedules = 0i32;
        for row in &sched_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            if status == "active" { active_schedules += 1; }
        }

        // Compute overdue work orders
        let overdue_row = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.work_orders
               WHERE organization_id = $1
                 AND status IN ('draft', 'approved', 'in_progress')
                 AND scheduled_end < CURRENT_DATE"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let overdue_work_orders: i64 = overdue_row.try_get("cnt").unwrap_or(0);

        // Compute overdue schedules
        let overdue_sched_row = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.preventive_maintenance_schedules
               WHERE organization_id = $1
                 AND status = 'active'
                 AND next_due_date < CURRENT_DATE"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let overdue_schedules: i64 = overdue_sched_row.try_get("cnt").unwrap_or(0);

        // Compute avg completion time
        let avg_row = sqlx::query(
            r#"SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (actual_end - created_at))/86400), 0) as avg_days
               FROM _atlas.work_orders
               WHERE organization_id = $1 AND status IN ('completed', 'closed') AND actual_end IS NOT NULL"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let avg_completion_days: f64 = avg_row.try_get("avg_days").unwrap_or(0.0);

        // Compute total maintenance cost
        let cost_row = sqlx::query(
            r#"SELECT COALESCE(SUM(CAST(actual_cost AS NUMERIC)), 0) as total_cost
               FROM _atlas.work_orders WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let total_maintenance_cost: f64 = cost_row.try_get("total_cost").unwrap_or(0.0);

        // Compute total downtime
        let dt_row = sqlx::query(
            r#"SELECT COALESCE(SUM(downtime_hours), 0) as total_dt
               FROM _atlas.work_orders WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let total_downtime_hours: f64 = dt_row.try_get("total_dt").unwrap_or(0.0);

        // MTBF / MTTR
        let mtbf_row = sqlx::query(
            r#"SELECT COALESCE(AVG(CAST(meter_reading->>'value' AS NUMERIC)), 0) as mtbf
               FROM _atlas.asset_definitions
               WHERE organization_id = $1 AND asset_criticality IN ('high', 'critical')"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let mtbf_hours: f64 = mtbf_row.try_get("mtbf").unwrap_or(0.0);

        let mttr_row = sqlx::query(
            r#"SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (COALESCE(actual_end, now()) - COALESCE(actual_start, created_at)))/3600), 0) as mttr
               FROM _atlas.work_orders
               WHERE organization_id = $1 AND status IN ('completed', 'closed')"#,
        ).bind(org_id).fetch_one(&self.pool).await?;
        let mttr_hours: f64 = mttr_row.try_get("mttr").unwrap_or(0.0);

        Ok(MaintenanceDashboard {
            total_assets: asset_rows.len() as i32,
            active_assets,
            assets_in_repair,
            critical_assets,
            total_work_orders: wo_rows.len() as i32,
            open_work_orders,
            in_progress_work_orders,
            completed_work_orders,
            overdue_work_orders: overdue_work_orders as i32,
            emergency_work_orders,
            preventive_work_orders,
            corrective_work_orders,
            total_schedules: sched_rows.len() as i32,
            active_schedules,
            overdue_schedules: overdue_schedules as i32,
            avg_completion_days,
            total_maintenance_cost: format!("{:.2}", total_maintenance_cost),
            total_downtime_hours,
            mtbf_hours,
            mttr_hours,
            work_orders_by_priority: serde_json::to_value(&wo_by_priority).unwrap_or_default(),
            work_orders_by_type: serde_json::to_value(&wo_by_type).unwrap_or_default(),
            assets_by_criticality: serde_json::to_value(&assets_by_criticality).unwrap_or_default(),
            costs_by_month: serde_json::json!({}),
        })
    }
}
