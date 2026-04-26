//! KPI Repository
//!
//! PostgreSQL storage for KPI definitions, data points, dashboards, and widgets.

use atlas_shared::{
    KpiDefinition, KpiDataPoint, Dashboard, DashboardWidget, KpiDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for KPI & Analytics data storage
#[async_trait]
pub trait KpiRepository: Send + Sync {
    // KPI Definitions
    async fn create_kpi(
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
    ) -> AtlasResult<KpiDefinition>;

    async fn get_kpi(&self, id: Uuid) -> AtlasResult<Option<KpiDefinition>>;
    async fn get_kpi_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<KpiDefinition>>;
    async fn list_kpis(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<KpiDefinition>>;
    async fn delete_kpi(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // KPI Data Points
    async fn record_data_point(
        &self,
        org_id: Uuid,
        kpi_id: Uuid,
        value: &str,
        period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        recorded_by: Option<Uuid>,
    ) -> AtlasResult<KpiDataPoint>;

    async fn get_latest_data_point(&self, kpi_id: Uuid) -> AtlasResult<Option<KpiDataPoint>>;
    async fn list_data_points(
        &self,
        kpi_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<KpiDataPoint>>;
    async fn delete_data_point(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboards
    async fn create_dashboard(
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
    ) -> AtlasResult<Dashboard>;

    async fn get_dashboard(&self, id: Uuid) -> AtlasResult<Option<Dashboard>>;
    async fn get_dashboard_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Dashboard>>;
    async fn list_dashboards(&self, org_id: Uuid, owner_id: Option<Uuid>) -> AtlasResult<Vec<Dashboard>>;
    async fn delete_dashboard(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Dashboard Widgets
    async fn add_widget(
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
    ) -> AtlasResult<DashboardWidget>;

    async fn list_widgets(&self, dashboard_id: Uuid) -> AtlasResult<Vec<DashboardWidget>>;
    async fn delete_widget(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard Summary
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<KpiDashboardSummary>;
}

/// PostgreSQL implementation of KPI Repository
pub struct PostgresKpiRepository {
    pool: PgPool,
}

impl PostgresKpiRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_kpi(row: &sqlx::postgres::PgRow) -> KpiDefinition {
    KpiDefinition {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        unit_of_measure: row.try_get("unit_of_measure").unwrap_or_default(),
        direction: row.try_get("direction").unwrap_or_default(),
        target_value: row.try_get("target_value").unwrap_or_default(),
        warning_threshold: row.try_get("warning_threshold").unwrap_or_default(),
        critical_threshold: row.try_get("critical_threshold").unwrap_or_default(),
        data_source_query: row.try_get("data_source_query").unwrap_or_default(),
        evaluation_frequency: row.try_get("evaluation_frequency").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_data_point(row: &sqlx::postgres::PgRow) -> KpiDataPoint {
    KpiDataPoint {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        kpi_id: row.try_get("kpi_id").unwrap_or_default(),
        value: row.try_get("value").unwrap_or_default(),
        recorded_at: row.try_get("recorded_at").unwrap_or(chrono::Utc::now()),
        period_start: row.try_get("period_start").unwrap_or_default(),
        period_end: row.try_get("period_end").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "no_target".to_string()),
        notes: row.try_get("notes").unwrap_or_default(),
        recorded_by: row.try_get("recorded_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_dashboard(row: &sqlx::postgres::PgRow) -> Dashboard {
    Dashboard {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        is_shared: row.try_get("is_shared").unwrap_or(false),
        is_default: row.try_get("is_default").unwrap_or(false),
        layout_config: row.try_get("layout_config").unwrap_or(serde_json::json!({})),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_widget(row: &sqlx::postgres::PgRow) -> DashboardWidget {
    DashboardWidget {
        id: row.try_get("id").unwrap_or_default(),
        dashboard_id: row.try_get("dashboard_id").unwrap_or_default(),
        kpi_id: row.try_get("kpi_id").unwrap_or_default(),
        widget_type: row.try_get("widget_type").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        position_row: row.try_get("position_row").unwrap_or(0),
        position_col: row.try_get("position_col").unwrap_or(0),
        width: row.try_get("width").unwrap_or(1),
        height: row.try_get("height").unwrap_or(1),
        display_config: row.try_get("display_config").unwrap_or(serde_json::json!({})),
        is_visible: row.try_get("is_visible").unwrap_or(true),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl KpiRepository for PostgresKpiRepository {
    async fn create_kpi(
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
        let row = sqlx::query(
            r#"INSERT INTO _atlas.kpi_definitions
                (organization_id, code, name, description, category, unit_of_measure,
                 direction, target_value, warning_threshold, critical_threshold,
                 data_source_query, evaluation_frequency, is_active, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, true, '{}'::jsonb, $13)
            RETURNING *"#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(category).bind(unit_of_measure).bind(direction)
        .bind(target_value).bind(warning_threshold).bind(critical_threshold)
        .bind(data_source_query).bind(evaluation_frequency).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_kpi(&row))
    }

    async fn get_kpi(&self, id: Uuid) -> AtlasResult<Option<KpiDefinition>> {
        let row = sqlx::query("SELECT * FROM _atlas.kpi_definitions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_kpi))
    }

    async fn get_kpi_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<KpiDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.kpi_definitions WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_kpi))
    }

    async fn list_kpis(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<KpiDefinition>> {
        let rows = if let Some(cat) = category {
            sqlx::query(
                "SELECT * FROM _atlas.kpi_definitions WHERE organization_id = $1 AND category = $2 AND is_active = true ORDER BY created_at DESC"
            )
            .bind(org_id).bind(cat)
            .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.kpi_definitions WHERE organization_id = $1 AND is_active = true ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_kpi).collect())
    }

    async fn delete_kpi(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.kpi_definitions WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("KPI '{}' not found", code)));
        }
        Ok(())
    }

    async fn record_data_point(
        &self,
        org_id: Uuid,
        kpi_id: Uuid,
        value: &str,
        period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        recorded_by: Option<Uuid>,
    ) -> AtlasResult<KpiDataPoint> {
        // Compute status based on thresholds
        let kpi = self.get_kpi(kpi_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("KPI not found".to_string()))?;

        let status = compute_status(
            value,
            &kpi.direction,
            &kpi.target_value,
            kpi.warning_threshold.as_deref(),
            kpi.critical_threshold.as_deref(),
        );

        let row = sqlx::query(
            r#"INSERT INTO _atlas.kpi_data_points
                (organization_id, kpi_id, value, period_start, period_end, status, notes, recorded_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *"#
        )
        .bind(org_id).bind(kpi_id).bind(value)
        .bind(period_start).bind(period_end)
        .bind(&status).bind(notes).bind(recorded_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_data_point(&row))
    }

    async fn get_latest_data_point(&self, kpi_id: Uuid) -> AtlasResult<Option<KpiDataPoint>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.kpi_data_points WHERE kpi_id = $1 ORDER BY recorded_at DESC LIMIT 1"
        )
        .bind(kpi_id)
        .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_data_point))
    }

    async fn list_data_points(
        &self,
        kpi_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<KpiDataPoint>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.kpi_data_points WHERE kpi_id = $1 ORDER BY recorded_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(kpi_id).bind(limit).bind(offset)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_data_point).collect())
    }

    async fn delete_data_point(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.kpi_data_points WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Data point not found".to_string()));
        }
        Ok(())
    }

    async fn create_dashboard(
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
        let row = sqlx::query(
            r#"INSERT INTO _atlas.kpi_dashboards
                (organization_id, code, name, description, owner_id,
                 is_shared, is_default, layout_config, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}'::jsonb, $9)
            RETURNING *"#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(owner_id).bind(is_shared).bind(is_default)
        .bind(&layout_config).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_dashboard(&row))
    }

    async fn get_dashboard(&self, id: Uuid) -> AtlasResult<Option<Dashboard>> {
        let row = sqlx::query("SELECT * FROM _atlas.kpi_dashboards WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_dashboard))
    }

    async fn get_dashboard_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<Dashboard>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.kpi_dashboards WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_dashboard))
    }

    async fn list_dashboards(&self, org_id: Uuid, owner_id: Option<Uuid>) -> AtlasResult<Vec<Dashboard>> {
        let rows = if let Some(oid) = owner_id {
            sqlx::query(
                "SELECT * FROM _atlas.kpi_dashboards WHERE organization_id = $1 AND (owner_id = $2 OR is_shared = true) ORDER BY created_at DESC"
            )
            .bind(org_id).bind(oid)
            .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.kpi_dashboards WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_dashboard).collect())
    }

    async fn delete_dashboard(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.kpi_dashboards WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Dashboard '{}' not found", code)));
        }
        Ok(())
    }

    async fn add_widget(
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
        let row = sqlx::query(
            r#"INSERT INTO _atlas.kpi_dashboard_widgets
                (dashboard_id, kpi_id, widget_type, title,
                 position_row, position_col, width, height, display_config, is_visible)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
            RETURNING *"#
        )
        .bind(dashboard_id).bind(kpi_id).bind(widget_type).bind(title)
        .bind(position_row).bind(position_col).bind(width).bind(height)
        .bind(&display_config)
        .fetch_one(&self.pool).await?;
        Ok(row_to_widget(&row))
    }

    async fn list_widgets(&self, dashboard_id: Uuid) -> AtlasResult<Vec<DashboardWidget>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.kpi_dashboard_widgets WHERE dashboard_id = $1 AND is_visible = true ORDER BY position_row, position_col"
        )
        .bind(dashboard_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_widget).collect())
    }

    async fn delete_widget(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.kpi_dashboard_widgets WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Widget not found".to_string()));
        }
        Ok(())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<KpiDashboardSummary> {
        let kpis = self.list_kpis(org_id, None).await?;

        let mut on_track = 0i32;
        let mut warning = 0i32;
        let mut critical = 0i32;
        let mut no_data = 0i32;

        for kpi in &kpis {
            match self.get_latest_data_point(kpi.id).await? {
                Some(dp) => match dp.status.as_str() {
                    "on_track" => on_track += 1,
                    "warning" => warning += 1,
                    "critical" => critical += 1,
                    _ => on_track += 1,
                },
                None => no_data += 1,
            }
        }

        let total_kpis = kpis.len() as i32;
        let active_kpis = kpis.iter().filter(|k| k.is_active).count() as i32;

        // Category breakdown
        let mut categories: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for kpi in &kpis {
            *categories.entry(kpi.category.clone()).or_insert(0) += 1;
        }

        // Recent values (last 10 data points)
        let recent_rows = sqlx::query(
            r#"SELECT dp.*, kd.name as kpi_name, kd.code as kpi_code
               FROM _atlas.kpi_data_points dp
               JOIN _atlas.kpi_definitions kd ON dp.kpi_id = kd.id
               WHERE dp.organization_id = $1
               ORDER BY dp.recorded_at DESC LIMIT 10"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let recent_values: Vec<serde_json::Value> = recent_rows.iter().map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "kpiName": r.try_get::<String, _>("kpi_name").unwrap_or_default(),
                "kpiCode": r.try_get::<String, _>("kpi_code").unwrap_or_default(),
                "value": r.try_get::<String, _>("value").unwrap_or_default(),
                "status": r.try_get::<String, _>("status").unwrap_or_default(),
                "recordedAt": r.try_get::<chrono::DateTime<chrono::Utc>, _>("recorded_at").unwrap_or_default(),
            })
        }).collect();

        let dashboards = self.list_dashboards(org_id, None).await?;

        Ok(KpiDashboardSummary {
            total_kpis,
            active_kpis,
            on_track,
            warning,
            critical,
            no_data,
            total_dashboards: dashboards.len() as i32,
            kpis_by_category: serde_json::to_value(&categories).unwrap_or(serde_json::json!({})),
            recent_values: serde_json::json!(recent_values),
        })
    }
}

/// Compute the status of a KPI value against its thresholds
/// Thresholds are percentage values (e.g., 10 means 10% deviation allowed)
fn compute_status(
    value: &str,
    direction: &str,
    target: &str,
    warning: Option<&str>,
    critical: Option<&str>,
) -> String {
    let val: f64 = match value.parse() {
        Ok(v) => v,
        Err(_) => return "no_target".to_string(),
    };

    let target_val: f64 = match target.parse() {
        Ok(v) => v,
        Err(_) => return "no_target".to_string(),
    };

    if target_val == 0.0 {
        return "no_target".to_string();
    }

    let warning_pct = warning.and_then(|w| w.parse::<f64>().ok());
    let critical_pct = critical.and_then(|c| c.parse::<f64>().ok());

    let deviation_pct = match direction {
        "higher_is_better" => {
            // How far below target as % of target
            (target_val - val) / target_val.abs() * 100.0
        }
        "lower_is_better" => {
            // How far above target as % of target
            (val - target_val) / target_val.abs() * 100.0
        }
        "target_range" => {
            // Distance from target in either direction
            (val - target_val).abs() / target_val.abs() * 100.0
        }
        _ => return "no_target".to_string(),
    };

    // Check critical threshold first (higher threshold = more tolerant)
    if let Some(cp) = critical_pct {
        if deviation_pct > cp {
            return "critical".to_string();
        }
    }

    // Check warning threshold
    if let Some(wp) = warning_pct {
        if deviation_pct > wp {
            return "warning".to_string();
        }
    }

    "on_track".to_string()
}
