//! Demand Planning Repository
//!
//! PostgreSQL storage for demand planning data.

use atlas_shared::{
    DemandForecastMethod, DemandSchedule, DemandScheduleLine,
    DemandHistory, DemandConsumption, DemandAccuracy, DemandPlanningDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for Demand Planning data storage
#[async_trait]
pub trait DemandPlanningRepository: Send + Sync {
    // Forecast Methods
    async fn create_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        method_type: &str,
        parameters: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandForecastMethod>;
    async fn get_method(&self, id: Uuid) -> AtlasResult<Option<DemandForecastMethod>>;
    async fn get_method_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DemandForecastMethod>>;
    async fn list_methods(&self, org_id: Uuid) -> AtlasResult<Vec<DemandForecastMethod>>;
    async fn delete_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Demand Schedules
    async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        method_id: Option<Uuid>,
        method_name: Option<&str>,
        schedule_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        currency_code: &str,
        confidence_level: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandSchedule>;
    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<DemandSchedule>>;
    async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<DemandSchedule>>;
    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DemandSchedule>>;
    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<DemandSchedule>;
    async fn approve_schedule(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<DemandSchedule>;
    async fn update_schedule_totals(&self, id: Uuid) -> AtlasResult<DemandSchedule>;
    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()>;

    // Schedule Lines
    async fn add_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        line_number: i32,
        item_code: &str,
        item_name: Option<&str>,
        item_category: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        forecast_quantity: &str,
        unit_price: &str,
        confidence_pct: &str,
        notes: Option<&str>,
    ) -> AtlasResult<DemandScheduleLine>;
    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<DemandScheduleLine>>;
    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<DemandScheduleLine>>;
    async fn delete_schedule_line(&self, id: Uuid) -> AtlasResult<()>;

    // Demand History
    async fn create_history(
        &self,
        org_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        actual_date: chrono::NaiveDate,
        actual_quantity: &str,
        actual_value: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_line_id: Option<Uuid>,
    ) -> AtlasResult<DemandHistory>;
    async fn list_history(
        &self,
        org_id: Uuid,
        item_code: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<DemandHistory>>;
    async fn delete_history(&self, id: Uuid) -> AtlasResult<()>;

    // Forecast Consumption
    async fn create_consumption(
        &self,
        org_id: Uuid,
        schedule_line_id: Uuid,
        history_id: Option<Uuid>,
        consumed_quantity: &str,
        consumed_date: chrono::NaiveDate,
        source_type: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandConsumption>;
    async fn list_consumption(&self, schedule_line_id: Uuid) -> AtlasResult<Vec<DemandConsumption>>;
    async fn delete_consumption(&self, id: Uuid) -> AtlasResult<()>;

    // Accuracy
    async fn create_accuracy(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        schedule_line_id: Option<Uuid>,
        item_code: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        forecast_quantity: &str,
        actual_quantity: &str,
        absolute_error: &str,
        absolute_pct_error: &str,
        bias: &str,
        measurement_date: chrono::NaiveDate,
    ) -> AtlasResult<DemandAccuracy>;
    async fn list_accuracy(&self, schedule_id: Uuid) -> AtlasResult<Vec<DemandAccuracy>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DemandPlanningDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresDemandPlanningRepository {
    pool: PgPool,
}

impl PostgresDemandPlanningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DemandPlanningRepository for PostgresDemandPlanningRepository {
    // ========================================================================
    // Forecast Methods
    // ========================================================================

    async fn create_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        method_type: &str,
        parameters: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandForecastMethod> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_forecast_methods
                (organization_id, code, name, description, method_type, parameters, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(method_type).bind(&parameters).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_method(&row))
    }

    async fn get_method(&self, id: Uuid) -> AtlasResult<Option<DemandForecastMethod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.demand_forecast_methods WHERE id = $1 AND is_active = true",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_method(&r)))
    }

    async fn get_method_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DemandForecastMethod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.demand_forecast_methods WHERE organization_id = $1 AND code = $2 AND is_active = true",
        )
        .bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_method(&r)))
    }

    async fn list_methods(&self, org_id: Uuid) -> AtlasResult<Vec<DemandForecastMethod>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.demand_forecast_methods WHERE organization_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_method).collect())
    }

    async fn delete_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.demand_forecast_methods WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Demand Schedules
    // ========================================================================

    async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        method_id: Option<Uuid>,
        method_name: Option<&str>,
        schedule_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        currency_code: &str,
        confidence_level: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandSchedule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_schedules
                (organization_id, schedule_number, name, description,
                 method_id, method_name, schedule_type, start_date, end_date,
                 currency_code, confidence_level, owner_id, owner_name, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_number).bind(name).bind(description)
        .bind(method_id).bind(method_name).bind(schedule_type)
        .bind(start_date).bind(end_date)
        .bind(currency_code).bind(confidence_level)
        .bind(owner_id).bind(owner_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<DemandSchedule>> {
        let row = sqlx::query("SELECT * FROM _atlas.demand_schedules WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<DemandSchedule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.demand_schedules WHERE organization_id = $1 AND schedule_number = $2",
        )
        .bind(org_id).bind(schedule_number).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule(&r)))
    }

    async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DemandSchedule>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.demand_schedules WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.demand_schedules WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule).collect())
    }

    async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<DemandSchedule> {
        let row = sqlx::query(
            "UPDATE _atlas.demand_schedules SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn approve_schedule(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<DemandSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.demand_schedules
               SET status = 'approved', approved_by = $2, approved_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(approved_by).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn update_schedule_totals(&self, id: Uuid) -> AtlasResult<DemandSchedule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.demand_schedules
               SET total_forecast_quantity = (
                       SELECT COALESCE(SUM(forecast_quantity), 0) FROM _atlas.demand_schedule_lines WHERE schedule_id = $1
                   ),
                   total_forecast_value = (
                       SELECT COALESCE(SUM(forecast_value), 0) FROM _atlas.demand_schedule_lines WHERE schedule_id = $1
                   ),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule(&row))
    }

    async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.demand_schedules WHERE organization_id = $1 AND schedule_number = $2")
            .bind(org_id).bind(schedule_number).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Schedule Lines
    // ========================================================================

    async fn add_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        line_number: i32,
        item_code: &str,
        item_name: Option<&str>,
        item_category: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        forecast_quantity: &str,
        unit_price: &str,
        confidence_pct: &str,
        notes: Option<&str>,
    ) -> AtlasResult<DemandScheduleLine> {
        let qty: f64 = forecast_quantity.parse().unwrap_or(0.0);
        let price: f64 = unit_price.parse().unwrap_or(0.0);
        let value = qty * price;

        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_schedule_lines
                (organization_id, schedule_id, line_number, item_code, item_name,
                 item_category, warehouse_code, region, customer_group,
                 period_start, period_end, forecast_quantity, forecast_value,
                 unit_price, remaining_quantity, confidence_pct, notes)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$12,$15,$16) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(line_number).bind(item_code)
        .bind(item_name).bind(item_category).bind(warehouse_code)
        .bind(region).bind(customer_group)
        .bind(period_start).bind(period_end)
        .bind(qty).bind(value).bind(price)
        .bind(confidence_pct.parse::<f64>().unwrap_or(0.0))
        .bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line(&row))
    }

    async fn list_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<DemandScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.demand_schedule_lines WHERE schedule_id = $1 ORDER BY line_number",
        )
        .bind(schedule_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_schedule_line).collect())
    }

    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<DemandScheduleLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.demand_schedule_lines WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule_line(&r)))
    }

    async fn delete_schedule_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.demand_schedule_lines WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Demand History
    // ========================================================================

    async fn create_history(
        &self,
        org_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        actual_date: chrono::NaiveDate,
        actual_quantity: &str,
        actual_value: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_line_id: Option<Uuid>,
    ) -> AtlasResult<DemandHistory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_history
                (organization_id, item_code, item_name, warehouse_code, region,
                 customer_group, actual_date, actual_quantity, actual_value,
                 source_type, source_id, source_line_id)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) RETURNING *"#,
        )
        .bind(org_id).bind(item_code).bind(item_name).bind(warehouse_code)
        .bind(region).bind(customer_group).bind(actual_date)
        .bind(actual_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(actual_value.parse::<f64>().unwrap_or(0.0))
        .bind(source_type).bind(source_id).bind(source_line_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_history(&row))
    }

    async fn list_history(
        &self,
        org_id: Uuid,
        item_code: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<DemandHistory>> {
        let rows = match (item_code, start_date, end_date) {
            (Some(ic), Some(sd), Some(ed)) => sqlx::query(
                "SELECT * FROM _atlas.demand_history WHERE organization_id = $1 AND item_code = $2 AND actual_date >= $3 AND actual_date <= $4 ORDER BY actual_date DESC",
            ).bind(org_id).bind(ic).bind(sd).bind(ed).fetch_all(&self.pool).await,
            (Some(ic), None, None) => sqlx::query(
                "SELECT * FROM _atlas.demand_history WHERE organization_id = $1 AND item_code = $2 ORDER BY actual_date DESC",
            ).bind(org_id).bind(ic).fetch_all(&self.pool).await,
            (None, Some(sd), Some(ed)) => sqlx::query(
                "SELECT * FROM _atlas.demand_history WHERE organization_id = $1 AND actual_date >= $2 AND actual_date <= $3 ORDER BY actual_date DESC",
            ).bind(org_id).bind(sd).bind(ed).fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.demand_history WHERE organization_id = $1 ORDER BY actual_date DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_history).collect())
    }

    async fn delete_history(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.demand_history WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Forecast Consumption
    // ========================================================================

    async fn create_consumption(
        &self,
        org_id: Uuid,
        schedule_line_id: Uuid,
        history_id: Option<Uuid>,
        consumed_quantity: &str,
        consumed_date: chrono::NaiveDate,
        source_type: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DemandConsumption> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_consumption
                (organization_id, schedule_line_id, history_id,
                 consumed_quantity, consumed_date, source_type, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_line_id).bind(history_id)
        .bind(consumed_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(consumed_date).bind(source_type).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        // Update the schedule line consumed/remaining quantities
        sqlx::query(
            r#"UPDATE _atlas.demand_schedule_lines
               SET consumed_quantity = (
                       SELECT COALESCE(SUM(consumed_quantity), 0) FROM _atlas.demand_consumption WHERE schedule_line_id = $1
                   ),
                   remaining_quantity = forecast_quantity - (
                       SELECT COALESCE(SUM(consumed_quantity), 0) FROM _atlas.demand_consumption WHERE schedule_line_id = $1
                   ),
                   updated_at = now()
               WHERE id = $1"#,
        )
        .bind(schedule_line_id).execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_consumption(&row))
    }

    async fn list_consumption(&self, schedule_line_id: Uuid) -> AtlasResult<Vec<DemandConsumption>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.demand_consumption WHERE schedule_line_id = $1 ORDER BY consumed_date DESC",
        )
        .bind(schedule_line_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_consumption).collect())
    }

    async fn delete_consumption(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.demand_consumption WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Accuracy
    // ========================================================================

    async fn create_accuracy(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        schedule_line_id: Option<Uuid>,
        item_code: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        forecast_quantity: &str,
        actual_quantity: &str,
        absolute_error: &str,
        absolute_pct_error: &str,
        bias: &str,
        measurement_date: chrono::NaiveDate,
    ) -> AtlasResult<DemandAccuracy> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.demand_accuracy
                (organization_id, schedule_id, schedule_line_id, item_code,
                 period_start, period_end, forecast_quantity, actual_quantity,
                 absolute_error, absolute_pct_error, bias, measurement_date)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) RETURNING *"#,
        )
        .bind(org_id).bind(schedule_id).bind(schedule_line_id).bind(item_code)
        .bind(period_start).bind(period_end)
        .bind(forecast_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(actual_quantity.parse::<f64>().unwrap_or(0.0))
        .bind(absolute_error.parse::<f64>().unwrap_or(0.0))
        .bind(absolute_pct_error.parse::<f64>().unwrap_or(0.0))
        .bind(bias.parse::<f64>().unwrap_or(0.0))
        .bind(measurement_date)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_accuracy(&row))
    }

    async fn list_accuracy(&self, schedule_id: Uuid) -> AtlasResult<Vec<DemandAccuracy>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.demand_accuracy WHERE schedule_id = $1 ORDER BY period_start",
        )
        .bind(schedule_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_accuracy).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DemandPlanningDashboard> {
        use sqlx::Row;

        let sched_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('approved', 'active')) as active,
                COALESCE(SUM(total_forecast_quantity), 0) as total_qty,
                COALESCE(SUM(total_forecast_value), 0) as total_val
               FROM _atlas.demand_schedules WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let item_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT item_code) FROM _atlas.demand_schedule_lines WHERE organization_id = $1",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let avg_accuracy: f64 = sqlx::query_scalar(
            r#"SELECT COALESCE(AVG(absolute_pct_error), 0) FROM _atlas.demand_accuracy WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        // Accuracy = 100 - MAPE
        let accuracy_pct = 100.0 - avg_accuracy;

        // By status
        let status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.demand_schedules
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let schedules_by_status: serde_json::Value = status_rows.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        // Top forecast items
        let top_rows = sqlx::query(
            r#"SELECT item_code, item_name, SUM(forecast_quantity) as total_qty, SUM(forecast_value) as total_val
               FROM _atlas.demand_schedule_lines WHERE organization_id = $1
               GROUP BY item_code, item_name ORDER BY total_qty DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let top_forecast_items: serde_json::Value = top_rows.iter().map(|r| {
            serde_json::json!({
                "itemCode": r.get::<String, _>("item_code"),
                "itemName": r.get::<Option<String>, _>("item_name").unwrap_or_default(),
                "totalQuantity": format!("{:.2}", r.get::<f64, _>("total_qty")),
                "totalValue": format!("{:.2}", r.get::<f64, _>("total_val")),
            })
        }).collect();

        // Accuracy by method
        let method_rows = sqlx::query(
            r#"SELECT m.name as method_name, COALESCE(AVG(a.absolute_pct_error), 0) as avg_mape
               FROM _atlas.demand_schedules s
               JOIN _atlas.demand_forecast_methods m ON s.method_id = m.id
               LEFT JOIN _atlas.demand_accuracy a ON a.schedule_id = s.id
               WHERE s.organization_id = $1
               GROUP BY m.name"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let accuracy_by_method: serde_json::Value = method_rows.iter().map(|r| {
            let mape: f64 = r.get("avg_mape");
            serde_json::json!({
                "method": r.get::<String, _>("method_name"),
                "accuracyPct": format!("{:.1}", 100.0 - mape),
            })
        }).collect();

        Ok(DemandPlanningDashboard {
            total_schedules: sched_row.get::<i64, _>("total") as i32,
            active_schedules: sched_row.get::<i64, _>("active") as i32,
            total_forecast_items: item_count as i32,
            total_forecast_quantity: format!("{:.2}", sched_row.get::<f64, _>("total_qty")),
            total_forecast_value: format!("{:.2}", sched_row.get::<f64, _>("total_val")),
            avg_accuracy_pct: format!("{:.1}", accuracy_pct),
            schedules_by_status,
            top_forecast_items,
            accuracy_by_method,
        })
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_method(row: &sqlx::postgres::PgRow) -> DemandForecastMethod {
    DemandForecastMethod {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        method_type: row.get("method_type"),
        parameters: row.get("parameters"),
        is_active: row.get("is_active"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_schedule(row: &sqlx::postgres::PgRow) -> DemandSchedule {
    DemandSchedule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_number: row.get("schedule_number"),
        name: row.get("name"),
        description: row.get("description"),
        method_id: row.get("method_id"),
        method_name: row.get("method_name"),
        schedule_type: row.get("schedule_type"),
        status: row.get("status"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        currency_code: row.get("currency_code"),
        total_forecast_quantity: get_num(row, "total_forecast_quantity"),
        total_forecast_value: get_num(row, "total_forecast_value"),
        confidence_level: row.get("confidence_level"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        owner_id: row.get("owner_id"),
        owner_name: row.get("owner_name"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_schedule_line(row: &sqlx::postgres::PgRow) -> DemandScheduleLine {
    DemandScheduleLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_id: row.get("schedule_id"),
        line_number: row.get("line_number"),
        item_code: row.get("item_code"),
        item_name: row.get("item_name"),
        item_category: row.get("item_category"),
        warehouse_code: row.get("warehouse_code"),
        region: row.get("region"),
        customer_group: row.get("customer_group"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        forecast_quantity: get_num(row, "forecast_quantity"),
        forecast_value: get_num(row, "forecast_value"),
        unit_price: get_num(row, "unit_price"),
        consumed_quantity: get_num(row, "consumed_quantity"),
        remaining_quantity: get_num(row, "remaining_quantity"),
        confidence_pct: get_num(row, "confidence_pct"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_history(row: &sqlx::postgres::PgRow) -> DemandHistory {
    DemandHistory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        item_code: row.get("item_code"),
        item_name: row.get("item_name"),
        warehouse_code: row.get("warehouse_code"),
        region: row.get("region"),
        customer_group: row.get("customer_group"),
        actual_date: row.get("actual_date"),
        actual_quantity: get_num(row, "actual_quantity"),
        actual_value: get_num(row, "actual_value"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_line_id: row.get("source_line_id"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}

fn row_to_consumption(row: &sqlx::postgres::PgRow) -> DemandConsumption {
    DemandConsumption {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_line_id: row.get("schedule_line_id"),
        history_id: row.get("history_id"),
        consumed_quantity: get_num(row, "consumed_quantity"),
        consumed_date: row.get("consumed_date"),
        source_type: row.get("source_type"),
        notes: row.get("notes"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}

fn row_to_accuracy(row: &sqlx::postgres::PgRow) -> DemandAccuracy {
    DemandAccuracy {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        schedule_id: row.get("schedule_id"),
        schedule_line_id: row.get("schedule_line_id"),
        item_code: row.get("item_code"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        forecast_quantity: get_num(row, "forecast_quantity"),
        actual_quantity: get_num(row, "actual_quantity"),
        absolute_error: get_num(row, "absolute_error"),
        absolute_pct_error: get_num(row, "absolute_pct_error"),
        bias: get_num(row, "bias"),
        measurement_date: row.get("measurement_date"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}
