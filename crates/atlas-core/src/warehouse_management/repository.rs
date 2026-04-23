//! Warehouse Management Repository
//!
//! PostgreSQL storage for warehouse management data.

use atlas_shared::{
    Warehouse, WarehouseZone, PutAwayRule, WarehouseTask, PickWave, WarehouseDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for warehouse management storage
#[async_trait]
pub trait WarehouseManagementRepository: Send + Sync {
    // Warehouses
    async fn create_warehouse(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        location_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Warehouse>;

    async fn get_warehouse(&self, id: Uuid) -> AtlasResult<Option<Warehouse>>;
    async fn get_warehouse_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Warehouse>>;
    async fn list_warehouses(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<Warehouse>>;
    async fn delete_warehouse(&self, id: Uuid) -> AtlasResult<()>;

    // Zones
    async fn create_zone(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        code: &str,
        name: &str,
        zone_type: &str,
        description: Option<&str>,
        aisle_count: Option<i32>,
    ) -> AtlasResult<WarehouseZone>;

    async fn get_zone(&self, id: Uuid) -> AtlasResult<Option<WarehouseZone>>;
    async fn list_zones(&self, warehouse_id: Uuid) -> AtlasResult<Vec<WarehouseZone>>;
    async fn delete_zone(&self, id: Uuid) -> AtlasResult<()>;

    // Put-away Rules
    async fn create_put_away_rule(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        rule_name: &str,
        description: Option<&str>,
        priority: i32,
        item_category: Option<&str>,
        target_zone_type: &str,
        strategy: &str,
    ) -> AtlasResult<PutAwayRule>;

    async fn get_put_away_rule(&self, id: Uuid) -> AtlasResult<Option<PutAwayRule>>;
    async fn list_put_away_rules(&self, warehouse_id: Uuid) -> AtlasResult<Vec<PutAwayRule>>;
    async fn delete_put_away_rule(&self, id: Uuid) -> AtlasResult<()>;

    // Warehouse Tasks
    async fn create_task(
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
    ) -> AtlasResult<WarehouseTask>;

    async fn get_task(&self, id: Uuid) -> AtlasResult<Option<WarehouseTask>>;
    async fn get_task_by_number(&self, org_id: Uuid, task_number: &str) -> AtlasResult<Option<WarehouseTask>>;
    async fn list_tasks(&self, org_id: Uuid, warehouse_id: Option<Uuid>, status: Option<&str>, task_type: Option<&str>) -> AtlasResult<Vec<WarehouseTask>>;
    async fn update_task_status(&self, id: Uuid, status: &str, assigned_to: Option<Uuid>) -> AtlasResult<()>;
    async fn delete_task(&self, id: Uuid) -> AtlasResult<()>;

    // Pick Waves
    async fn create_wave(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        wave_number: &str,
        priority: &str,
        cut_off_date: Option<chrono::NaiveDate>,
        shipping_method: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PickWave>;

    async fn get_wave(&self, id: Uuid) -> AtlasResult<Option<PickWave>>;
    async fn get_wave_by_number(&self, org_id: Uuid, wave_number: &str) -> AtlasResult<Option<PickWave>>;
    async fn list_waves(&self, org_id: Uuid, warehouse_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<PickWave>>;
    async fn update_wave_status(&self, id: Uuid, status: &str) -> AtlasResult<()>;
    async fn update_wave_task_counts(&self, id: Uuid, total: i32, completed: i32) -> AtlasResult<()>;
    async fn delete_wave(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<WarehouseDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresWarehouseManagementRepository {
    pool: PgPool,
}

impl PostgresWarehouseManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WarehouseManagementRepository for PostgresWarehouseManagementRepository {
    // ─── Warehouses ─────────────────────────────────────────────────────────

    async fn create_warehouse(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        location_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Warehouse> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.warehouses (organization_id, code, name, description, location_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(code)
        .bind(name)
        .bind(description)
        .bind(location_code)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AtlasError::Conflict(format!("Warehouse code '{}' already exists", code))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;

        Ok(row_to_warehouse(&row))
    }

    async fn get_warehouse(&self, id: Uuid) -> AtlasResult<Option<Warehouse>> {
        let row = sqlx::query("SELECT * FROM _atlas.warehouses WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_warehouse(&r)))
    }

    async fn get_warehouse_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Warehouse>> {
        let row = sqlx::query("SELECT * FROM _atlas.warehouses WHERE organization_id = $1 AND code = $2")
            .bind(org_id)
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_warehouse(&r)))
    }

    async fn list_warehouses(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<Warehouse>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.warehouses WHERE organization_id = $1 AND is_active = true ORDER BY name")
                .bind(org_id)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query("SELECT * FROM _atlas.warehouses WHERE organization_id = $1 ORDER BY name")
                .bind(org_id)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_warehouse(r)).collect())
    }

    async fn delete_warehouse(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.warehouses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Warehouse {}", id)));
        }
        Ok(())
    }

    // ─── Zones ──────────────────────────────────────────────────────────────

    async fn create_zone(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        code: &str,
        name: &str,
        zone_type: &str,
        description: Option<&str>,
        aisle_count: Option<i32>,
    ) -> AtlasResult<WarehouseZone> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.warehouse_zones (organization_id, warehouse_id, code, name, zone_type, description, aisle_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(warehouse_id)
        .bind(code)
        .bind(name)
        .bind(zone_type)
        .bind(description)
        .bind(aisle_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AtlasError::Conflict(format!("Zone code '{}' already exists in this warehouse", code))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;

        Ok(row_to_zone(&row))
    }

    async fn get_zone(&self, id: Uuid) -> AtlasResult<Option<WarehouseZone>> {
        let row = sqlx::query("SELECT * FROM _atlas.warehouse_zones WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_zone(&r)))
    }

    async fn list_zones(&self, warehouse_id: Uuid) -> AtlasResult<Vec<WarehouseZone>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.warehouse_zones WHERE warehouse_id = $1 AND is_active = true ORDER BY zone_type, name",
        )
        .bind(warehouse_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_zone(r)).collect())
    }

    async fn delete_zone(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.warehouse_zones WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Zone {}", id)));
        }
        Ok(())
    }

    // ─── Put-Away Rules ─────────────────────────────────────────────────────

    async fn create_put_away_rule(
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.put_away_rules (organization_id, warehouse_id, rule_name, description, priority, item_category, target_zone_type, strategy)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(warehouse_id)
        .bind(rule_name)
        .bind(description)
        .bind(priority)
        .bind(item_category)
        .bind(target_zone_type)
        .bind(strategy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_put_away_rule(&row))
    }

    async fn get_put_away_rule(&self, id: Uuid) -> AtlasResult<Option<PutAwayRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.put_away_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_put_away_rule(&r)))
    }

    async fn list_put_away_rules(&self, warehouse_id: Uuid) -> AtlasResult<Vec<PutAwayRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.put_away_rules WHERE warehouse_id = $1 AND is_active = true ORDER BY priority",
        )
        .bind(warehouse_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_put_away_rule(r)).collect())
    }

    async fn delete_put_away_rule(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.put_away_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Put-away rule {}", id)));
        }
        Ok(())
    }

    // ─── Warehouse Tasks ────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    async fn create_task(
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
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.warehouse_tasks
                (organization_id, warehouse_id, task_number, task_type, priority,
                 item_id, item_description, from_zone_id, to_zone_id,
                 from_location, to_location, quantity, uom,
                 source_document, source_document_id, source_line_id,
                 wave_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(warehouse_id)
        .bind(task_number)
        .bind(task_type)
        .bind(priority)
        .bind(item_id)
        .bind(item_description)
        .bind(from_zone_id)
        .bind(to_zone_id)
        .bind(from_location)
        .bind(to_location)
        .bind(quantity)
        .bind(uom)
        .bind(source_document)
        .bind(source_document_id)
        .bind(source_line_id)
        .bind(wave_id)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AtlasError::Conflict(format!("Task number '{}' already exists", task_number))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;

        // If task is part of a wave, update wave total_tasks count
        if let Some(wid) = wave_id {
            let _ = sqlx::query(
                "UPDATE _atlas.pick_waves SET total_tasks = total_tasks + 1, updated_at = now() WHERE id = $1",
            )
            .bind(wid)
            .execute(&self.pool)
            .await;
        }

        Ok(row_to_task(&row))
    }

    async fn get_task(&self, id: Uuid) -> AtlasResult<Option<WarehouseTask>> {
        let row = sqlx::query("SELECT * FROM _atlas.warehouse_tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_task(&r)))
    }

    async fn get_task_by_number(&self, org_id: Uuid, task_number: &str) -> AtlasResult<Option<WarehouseTask>> {
        let row = sqlx::query("SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND task_number = $2")
            .bind(org_id)
            .bind(task_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_task(&r)))
    }

    async fn list_tasks(&self, org_id: Uuid, warehouse_id: Option<Uuid>, status: Option<&str>, task_type: Option<&str>) -> AtlasResult<Vec<WarehouseTask>> {
        let rows = match (warehouse_id, status, task_type) {
            (Some(wh), Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND warehouse_id = $2 AND status = $3 AND task_type = $4 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).bind(s).bind(t).fetch_all(&self.pool).await,
            (Some(wh), Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND warehouse_id = $2 AND status = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).bind(s).fetch_all(&self.pool).await,
            (Some(wh), None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND warehouse_id = $2 AND task_type = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).bind(t).fetch_all(&self.pool).await,
            (Some(wh), None, None) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND warehouse_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).fetch_all(&self.pool).await,
            (None, Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND status = $2 AND task_type = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).bind(t).fetch_all(&self.pool).await,
            (None, Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND task_type = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(t).fetch_all(&self.pool).await,
            (None, None, None) => sqlx::query(
                "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_task(r)).collect())
    }

    async fn update_task_status(&self, id: Uuid, status: &str, assigned_to: Option<Uuid>) -> AtlasResult<()> {
        let now = chrono::Utc::now();
        let (started_at, completed_at) = match status {
            "in_progress" => (Some(now), None),
            "completed" => (None, Some(now)),
            _ => (None, None),
        };

        let result = sqlx::query(
            r#"
            UPDATE _atlas.warehouse_tasks
            SET status = $2, assigned_to = COALESCE($3, assigned_to),
                started_at = COALESCE($4, started_at),
                completed_at = COALESCE($5, completed_at),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(assigned_to)
        .bind(started_at)
        .bind(completed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Task {}", id)));
        }

        // If task completed and part of a wave, update wave completed count
        if status == "completed" {
            let task = self.get_task(id).await?;
            if let Some(task) = task {
                if let Some(wave_id) = task.wave_id {
                    let _ = sqlx::query(
                        "UPDATE _atlas.pick_waves SET completed_tasks = completed_tasks + 1, updated_at = now() WHERE id = $1",
                    )
                    .bind(wave_id)
                    .execute(&self.pool)
                    .await;
                }
            }
        }

        Ok(())
    }

    async fn delete_task(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.warehouse_tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Task {}", id)));
        }
        Ok(())
    }

    // ─── Pick Waves ─────────────────────────────────────────────────────────

    async fn create_wave(
        &self,
        org_id: Uuid,
        warehouse_id: Uuid,
        wave_number: &str,
        priority: &str,
        cut_off_date: Option<chrono::NaiveDate>,
        shipping_method: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PickWave> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pick_waves (organization_id, warehouse_id, wave_number, priority, cut_off_date, shipping_method, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(warehouse_id)
        .bind(wave_number)
        .bind(priority)
        .bind(cut_off_date)
        .bind(shipping_method)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AtlasError::Conflict(format!("Wave number '{}' already exists", wave_number))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;

        Ok(row_to_wave(&row))
    }

    async fn get_wave(&self, id: Uuid) -> AtlasResult<Option<PickWave>> {
        let row = sqlx::query("SELECT * FROM _atlas.pick_waves WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_wave(&r)))
    }

    async fn get_wave_by_number(&self, org_id: Uuid, wave_number: &str) -> AtlasResult<Option<PickWave>> {
        let row = sqlx::query("SELECT * FROM _atlas.pick_waves WHERE organization_id = $1 AND wave_number = $2")
            .bind(org_id)
            .bind(wave_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_wave(&r)))
    }

    async fn list_waves(&self, org_id: Uuid, warehouse_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<PickWave>> {
        let rows = match (warehouse_id, status) {
            (Some(wh), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.pick_waves WHERE organization_id = $1 AND warehouse_id = $2 AND status = $3 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).bind(s).fetch_all(&self.pool).await,
            (Some(wh), None) => sqlx::query(
                "SELECT * FROM _atlas.pick_waves WHERE organization_id = $1 AND warehouse_id = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(wh).fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.pick_waves WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.pick_waves WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_wave(r)).collect())
    }

    async fn update_wave_status(&self, id: Uuid, status: &str) -> AtlasResult<()> {
        let now = chrono::Utc::now();
        let (released_at, completed_at) = match status {
            "released" | "in_progress" => (Some(now), None),
            "completed" => (None, Some(now)),
            _ => (None, None),
        };

        let result = sqlx::query(
            r#"
            UPDATE _atlas.pick_waves
            SET status = $2, released_at = COALESCE($3, released_at),
                completed_at = COALESCE($4, completed_at), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(released_at)
        .bind(completed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Wave {}", id)));
        }
        Ok(())
    }

    async fn update_wave_task_counts(&self, id: Uuid, total: i32, completed: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.pick_waves SET total_tasks = $2, completed_tasks = $3, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(total)
        .bind(completed)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_wave(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.pick_waves WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Wave {}", id)));
        }
        Ok(())
    }

    // ─── Dashboard ──────────────────────────────────────────────────────────

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<WarehouseDashboard> {
        let total_warehouses: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouses WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_warehouses: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouses WHERE organization_id = $1 AND is_active = true",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_zones: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouse_zones WHERE organization_id = $1 AND is_active = true",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_pending_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND status = 'pending'",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_in_progress_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND status = 'in_progress'",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_completed_tasks_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.warehouse_tasks WHERE organization_id = $1 AND status = 'completed' AND completed_at::date = CURRENT_DATE",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_active_waves: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.pick_waves WHERE organization_id = $1 AND status IN ('draft', 'released', 'in_progress')",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Tasks by type
        let type_rows = sqlx::query(
            "SELECT task_type, COUNT(*) as count FROM _atlas.warehouse_tasks WHERE organization_id = $1 GROUP BY task_type",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut tasks_by_type = serde_json::Map::new();
        for row in type_rows {
            let tt: String = row.get("task_type");
            let count: i64 = row.get("count");
            tasks_by_type.insert(tt, serde_json::Value::Number(count.into()));
        }

        // Tasks by priority
        let priority_rows = sqlx::query(
            "SELECT priority, COUNT(*) as count FROM _atlas.warehouse_tasks WHERE organization_id = $1 GROUP BY priority",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut tasks_by_priority = serde_json::Map::new();
        for row in priority_rows {
            let p: String = row.get("priority");
            let count: i64 = row.get("count");
            tasks_by_priority.insert(p, serde_json::Value::Number(count.into()));
        }

        // Wave completion percentage
        let wave_completion_pct: String = sqlx::query_scalar(
            r#"
            SELECT COALESCE(
                (SUM(completed_tasks)::decimal / NULLIF(SUM(total_tasks), 0)::decimal * 100)::text,
                '0.0'
            )
            FROM _atlas.pick_waves WHERE organization_id = $1 AND status IN ('released', 'in_progress', 'completed')
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Recent tasks
        let recent_rows = sqlx::query(
            "SELECT * FROM _atlas.warehouse_tasks WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 10",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(WarehouseDashboard {
            total_warehouses,
            active_warehouses,
            total_zones,
            total_pending_tasks,
            total_in_progress_tasks,
            total_completed_tasks_today,
            total_active_waves,
            tasks_by_type: serde_json::Value::Object(tasks_by_type),
            tasks_by_priority: serde_json::Value::Object(tasks_by_priority),
            wave_completion_pct,
            recent_tasks: recent_rows.iter().map(|r| row_to_task(r)).collect(),
        })
    }
}

// ─── Row mapping helpers ────────────────────────────────────────────────────────

fn row_to_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_warehouse(row: &sqlx::postgres::PgRow) -> Warehouse {
    Warehouse {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        location_code: row.get("location_code"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_zone(row: &sqlx::postgres::PgRow) -> WarehouseZone {
    WarehouseZone {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        warehouse_id: row.get("warehouse_id"),
        code: row.get("code"),
        name: row.get("name"),
        zone_type: row.get("zone_type"),
        description: row.get("description"),
        aisle_count: row.get("aisle_count"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_put_away_rule(row: &sqlx::postgres::PgRow) -> PutAwayRule {
    PutAwayRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        warehouse_id: row.get("warehouse_id"),
        rule_name: row.get("rule_name"),
        description: row.get("description"),
        priority: row.get("priority"),
        item_category: row.get("item_category"),
        target_zone_type: row.get("target_zone_type"),
        strategy: row.get("strategy"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_task(row: &sqlx::postgres::PgRow) -> WarehouseTask {
    WarehouseTask {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        warehouse_id: row.get("warehouse_id"),
        task_number: row.get("task_number"),
        task_type: row.get("task_type"),
        status: row.get("status"),
        priority: row.get("priority"),
        source_document: row.get("source_document"),
        source_document_id: row.get("source_document_id"),
        source_line_id: row.get("source_line_id"),
        item_id: row.get("item_id"),
        item_description: row.get("item_description"),
        from_zone_id: row.get("from_zone_id"),
        to_zone_id: row.get("to_zone_id"),
        from_location: row.get("from_location"),
        to_location: row.get("to_location"),
        quantity: Some(row_to_numeric(row, "quantity")),
        uom: row.get("uom"),
        assigned_to: row.get("assigned_to"),
        wave_id: row.get("wave_id"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_wave(row: &sqlx::postgres::PgRow) -> PickWave {
    PickWave {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        warehouse_id: row.get("warehouse_id"),
        wave_number: row.get("wave_number"),
        status: row.get("status"),
        priority: row.get("priority"),
        cut_off_date: row.get("cut_off_date"),
        shipping_method: row.get("shipping_method"),
        total_tasks: row.get("total_tasks"),
        completed_tasks: row.get("completed_tasks"),
        released_at: row.get("released_at"),
        completed_at: row.get("completed_at"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
