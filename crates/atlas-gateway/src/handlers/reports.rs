//! Report generation and data import/export handlers

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::handlers::records::sanitize_identifier;
use std::sync::Arc;
use tracing::{info, error};
use sqlx::{Row, Column};

// ============================================================================
// Report Generation
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub report_type: String,
    pub generated_at: String,
    pub data: serde_json::Value,
}

/// Generate a summary report for an entity
pub async fn generate_entity_report(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
) -> Result<Json<ReportResponse>, StatusCode> {
    info!("Generating report for entity: {}", entity);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    let table_name = sanitize_identifier(table_name).map_err(|e| {
        error!("Invalid table name for reports: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Get total count
    let count_row = sqlx::query(
        format!("SELECT COUNT(*) as count FROM \"{}\" WHERE deleted_at IS NULL", table_name).as_str()
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Report count error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get recent records
    let recent = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT 10",
            table_name
        ).as_str()
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Report recent error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let recent_records: Vec<serde_json::Value> = recent.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        for i in 0..row.columns().len() {
            let name = row.columns()[i].name();
            let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
            obj.insert(name.to_string(), value);
        }
        serde_json::Value::Object(obj)
    }).collect();

    // If there's a workflow_state column, get counts by state
    let by_state = if entity_def.workflow.is_some() {
        let state_rows = sqlx::query(
            format!(
                "SELECT workflow_state, COUNT(*) as count FROM \"{}\" WHERE deleted_at IS NULL GROUP BY workflow_state",
                table_name
            ).as_str()
        )
        .fetch_all(&state.db_pool)
        .await
        .unwrap_or_default();

        let mut states = serde_json::Map::new();
        for row in state_rows {
            let state: String = row.try_get("workflow_state").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            states.insert(state, serde_json::json!(count));
        }
        serde_json::Value::Object(states)
    } else {
        serde_json::Value::Null
    };

    Ok(Json(ReportResponse {
        report_type: format!("{}_summary", entity),
        generated_at: chrono::Utc::now().to_rfc3339(),
        data: serde_json::json!({
            "entity": entity,
            "total_records": total,
            "by_state": by_state,
            "recent_records": recent_records,
            "fields": entity_def.fields.iter().map(|f| &f.name).collect::<Vec<_>>(),
        }),
    }))
}

/// Generate a dashboard overview report
pub async fn dashboard_report(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ReportResponse>, StatusCode> {
    info!("Generating dashboard report");

    let entities = state.schema_engine.entity_names();
    let mut entity_counts = serde_json::Map::new();

    for entity_name in &entities {
        if let Some(def) = state.schema_engine.get_entity(entity_name) {
            let table = def.table_name.as_deref().unwrap_or(entity_name);
            let Ok(table) = sanitize_identifier(table) else {
                continue;
            };
            let count = sqlx::query(
                format!("SELECT COUNT(*) as count FROM \"{}\" WHERE deleted_at IS NULL", table).as_str()
            )
            .fetch_one(&state.db_pool)
            .await;

            if let Ok(row) = count {
                let total: i64 = row.try_get("count").unwrap_or(0);
                entity_counts.insert(entity_name.clone(), serde_json::json!(total));
            }
        }
    }

    // Get recent audit entries
    let recent_audit = state.audit_engine.query(&atlas_core::audit::AuditQuery {
        entity_type: None,
        entity_id: None,
        action: None,
        user_id: None,
        from_date: Some(chrono::Utc::now() - chrono::Duration::days(7)),
        to_date: Some(chrono::Utc::now()),
        limit: Some(20),
        offset: None,
    }).await.unwrap_or_default();

    Ok(Json(ReportResponse {
        report_type: "dashboard_overview".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        data: serde_json::json!({
            "entity_counts": entity_counts,
            "total_entities": entities.len(),
            "recent_activity_count": recent_audit.len(),
        }),
    }))
}

// ============================================================================
// Data Import / Export
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub entity: String,
    pub format: String, // "json" or "csv"
    pub data: serde_json::Value,
    #[serde(default)]
    pub upsert: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub entity: String,
    pub imported: usize,
    pub errors: Vec<String>,
}

/// Import data for an entity
pub async fn import_data(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, StatusCode> {
    info!("Importing data for entity: {} (format: {})", payload.entity, payload.format);

    let entity_def = state.schema_engine.get_entity(&payload.entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = entity_def.table_name.as_deref().unwrap_or(&payload.entity);

    let records = match payload.format.as_str() {
        "json" => {
            // Expect an array of objects
            payload.data.as_array()
                .cloned()
                .unwrap_or_default()
        }
        _ => {
            return Ok(Json(ImportResponse {
                entity: payload.entity,
                imported: 0,
                errors: vec![format!("Unsupported format: {}", payload.format)],
            }));
        }
    };

    let mut imported = 0;
    let mut errors = vec![];

    for record in &records {
        if let Some(obj) = record.as_object() {
            let fields: Vec<String> = obj.keys().cloned().collect();
            let placeholders: Vec<String> = (1..=fields.len())
                .map(|i| format!("${}", i))
                .collect();

            let query = if payload.upsert {
                // Simple insert (upsert would need conflict targets)
                format!(
                    "INSERT INTO \"{}\" ({}) VALUES ({}) ON CONFLICT DO NOTHING",
                    table_name,
                    fields.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "),
                    placeholders.join(", ")
                )
            } else {
                format!(
                    "INSERT INTO \"{}\" ({}) VALUES ({})",
                    table_name,
                    fields.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "),
                    placeholders.join(", ")
                )
            };

            let mut db_query = sqlx::query(&query);
            for key in &fields {
                let value = obj.get(key).unwrap_or(&serde_json::Value::Null);
                db_query = db_query.bind(value);
            }

            match db_query.execute(&state.db_pool).await {
                Ok(_) => imported += 1,
                Err(e) => errors.push(format!("Row {}: {}", imported + errors.len() + 1, e)),
            }
        }
    }

    Ok(Json(ImportResponse {
        entity: payload.entity,
        imported,
        errors,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub entity: String,
    pub format: String,
    pub count: usize,
    pub data: serde_json::Value,
}

/// Export data for an entity
pub async fn export_data(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Query(params): Query<ExportParams>,
) -> Result<Json<ExportResponse>, StatusCode> {
    info!("Exporting data for entity: {}", entity);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    let format = params.format.unwrap_or_else(|| "json".to_string());

    let rows = sqlx::query(
        format!("SELECT * FROM \"{}\" WHERE deleted_at IS NULL ORDER BY created_at DESC", table_name).as_str()
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Export query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let records: Vec<serde_json::Value> = rows.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        for i in 0..row.columns().len() {
            let name = row.columns()[i].name();
            let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
            obj.insert(name.to_string(), value);
        }
        serde_json::Value::Object(obj)
    }).collect();

    Ok(Json(ExportResponse {
        entity,
        format,
        count: records.len(),
        data: serde_json::Value::Array(records),
    }))
}
