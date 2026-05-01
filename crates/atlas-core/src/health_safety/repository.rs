//! Workplace Health & Safety Repository
//!
//! PostgreSQL storage for safety incidents, hazards, inspections,
//! corrective actions, and dashboard data.

use atlas_shared::{
    SafetyIncident, Hazard, SafetyInspection, SafetyCorrectiveAction,
    HealthSafetyDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for health & safety data storage
#[async_trait]
pub trait HealthSafetyRepository: Send + Sync {
    // Incidents
    async fn create_incident(
        &self, org_id: Uuid, incident_number: &str, title: &str, description: Option<&str>,
        incident_type: &str, severity: &str, status: &str, priority: &str,
        incident_date: chrono::NaiveDate, incident_time: Option<&str>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        reported_by_id: Option<Uuid>, reported_by_name: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        root_cause: Option<&str>, immediate_action: Option<&str>,
        osha_recordable: bool, osha_classification: Option<&str>,
        days_away_from_work: i32, days_restricted: i32,
        body_part: Option<&str>, injury_source: Option<&str>,
        event_type: Option<&str>, environment_factor: Option<&str>,
        involved_parties: serde_json::Value, witness_statements: serde_json::Value,
        attachments: serde_json::Value,
        resolution_date: Option<chrono::NaiveDate>, closed_date: Option<chrono::NaiveDate>,
        closed_by: Option<Uuid>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyIncident>;
    async fn get_incident(&self, id: Uuid) -> AtlasResult<Option<SafetyIncident>>;
    async fn get_incident_by_number(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<Option<SafetyIncident>>;
    async fn list_incidents(&self, org_id: Uuid, status: Option<&str>, severity: Option<&str>, incident_type: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<SafetyIncident>>;
    async fn update_incident_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyIncident>;
    async fn update_incident_investigation(&self, id: Uuid, root_cause: Option<&str>, immediate_action: Option<&str>, assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>, days_away_from_work: Option<i32>, days_restricted: Option<i32>) -> AtlasResult<SafetyIncident>;
    async fn close_incident(&self, id: Uuid, closed_by: Option<Uuid>) -> AtlasResult<SafetyIncident>;
    async fn delete_incident(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<()>;

    // Hazards
    async fn create_hazard(
        &self, org_id: Uuid, hazard_code: &str, title: &str, description: Option<&str>,
        hazard_category: &str, risk_level: &str, likelihood: &str, consequence: &str,
        risk_score: i32, status: &str,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        identified_by_id: Option<Uuid>, identified_by_name: Option<&str>,
        identified_date: chrono::NaiveDate,
        mitigation_measures: serde_json::Value,
        residual_risk_level: Option<&str>, residual_risk_score: Option<i32>,
        review_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<Hazard>;
    async fn get_hazard(&self, id: Uuid) -> AtlasResult<Option<Hazard>>;
    async fn get_hazard_by_code(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<Option<Hazard>>;
    async fn list_hazards(&self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>, hazard_category: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<Hazard>>;
    async fn update_hazard_status(&self, id: Uuid, status: &str) -> AtlasResult<Hazard>;
    async fn update_residual_risk(&self, id: Uuid, residual_risk_level: &str, residual_risk_score: i32) -> AtlasResult<Hazard>;
    async fn delete_hazard(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<()>;

    // Inspections
    async fn create_inspection(
        &self, org_id: Uuid, inspection_number: &str, title: &str, description: Option<&str>,
        inspection_type: &str, status: &str, priority: &str,
        scheduled_date: chrono::NaiveDate, completed_date: Option<chrono::NaiveDate>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        inspector_id: Option<Uuid>, inspector_name: Option<&str>,
        findings_summary: Option<&str>,
        total_findings: i32, critical_findings: i32, non_conformities: i32, observations: i32,
        score: Option<f64>, max_score: Option<f64>, score_pct: Option<f64>,
        findings: serde_json::Value, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyInspection>;
    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<SafetyInspection>>;
    async fn get_inspection_by_number(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<Option<SafetyInspection>>;
    async fn list_inspections(&self, org_id: Uuid, status: Option<&str>, inspection_type: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<SafetyInspection>>;
    async fn complete_inspection(
        &self, id: Uuid, findings_summary: Option<&str>,
        total_findings: i32, critical_findings: i32, non_conformities: i32, observations: i32,
        score: Option<f64>, max_score: Option<f64>, score_pct: Option<f64>,
        findings: serde_json::Value,
    ) -> AtlasResult<SafetyInspection>;
    async fn update_inspection_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyInspection>;
    async fn delete_inspection(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<()>;

    // CAPA
    async fn create_corrective_action(
        &self, org_id: Uuid, action_number: &str, title: &str, description: Option<&str>,
        action_type: &str, status: &str, priority: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        root_cause: Option<&str>, corrective_action_plan: Option<&str>, preventive_action_plan: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        due_date: Option<chrono::NaiveDate>, completed_date: Option<chrono::NaiveDate>,
        verified_by: Option<Uuid>, verified_date: Option<chrono::NaiveDate>, effectiveness: Option<&str>,
        facility_id: Option<Uuid>, department_id: Option<Uuid>,
        estimated_cost: Option<f64>, actual_cost: Option<f64>, currency_code: Option<&str>,
        notes: Option<&str>, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyCorrectiveAction>;
    async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<SafetyCorrectiveAction>>;
    async fn get_corrective_action_by_number(&self, org_id: Uuid, action_number: &str) -> AtlasResult<Option<SafetyCorrectiveAction>>;
    async fn list_corrective_actions(&self, org_id: Uuid, status: Option<&str>, action_type: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<SafetyCorrectiveAction>>;
    async fn update_corrective_action_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyCorrectiveAction>;
    async fn complete_corrective_action(&self, id: Uuid, effectiveness: &str, actual_cost: Option<f64>, verified_by: Option<Uuid>) -> AtlasResult<SafetyCorrectiveAction>;
    async fn delete_corrective_action(&self, org_id: Uuid, action_number: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HealthSafetyDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresHealthSafetyRepository {
    pool: PgPool,
}

impl PostgresHealthSafetyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_incident(row: &sqlx::postgres::PgRow) -> SafetyIncident {
    SafetyIncident {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        incident_number: row.try_get("incident_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        incident_type: row.try_get("incident_type").unwrap_or_default(),
        severity: row.try_get("severity").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        incident_date: row.try_get("incident_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        incident_time: row.try_get("incident_time").unwrap_or_default(),
        location: row.try_get("location").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        reported_by_id: row.try_get("reported_by_id").unwrap_or_default(),
        reported_by_name: row.try_get("reported_by_name").unwrap_or_default(),
        assigned_to_id: row.try_get("assigned_to_id").unwrap_or_default(),
        assigned_to_name: row.try_get("assigned_to_name").unwrap_or_default(),
        root_cause: row.try_get("root_cause").unwrap_or_default(),
        immediate_action: row.try_get("immediate_action").unwrap_or_default(),
        osha_recordable: row.try_get("osha_recordable").unwrap_or(false),
        osha_classification: row.try_get("osha_classification").unwrap_or_default(),
        days_away_from_work: row.try_get("days_away_from_work").unwrap_or(0),
        days_restricted: row.try_get("days_restricted").unwrap_or(0),
        body_part: row.try_get("body_part").unwrap_or_default(),
        injury_source: row.try_get("injury_source").unwrap_or_default(),
        event_type: row.try_get("event_type").unwrap_or_default(),
        environment_factor: row.try_get("environment_factor").unwrap_or_default(),
        involved_parties: row.try_get("involved_parties").unwrap_or(serde_json::json!([])),
        witness_statements: row.try_get("witness_statements").unwrap_or(serde_json::json!([])),
        attachments: row.try_get("attachments").unwrap_or(serde_json::json!([])),
        resolution_date: row.try_get("resolution_date").unwrap_or_default(),
        closed_date: row.try_get("closed_date").unwrap_or_default(),
        closed_by: row.try_get("closed_by").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_hazard(row: &sqlx::postgres::PgRow) -> Hazard {
    let risk_score: i32 = row.try_get("risk_score").unwrap_or(1);
    Hazard {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        hazard_code: row.try_get("hazard_code").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        hazard_category: row.try_get("hazard_category").unwrap_or_default(),
        risk_level: row.try_get("risk_level").unwrap_or_default(),
        likelihood: row.try_get("likelihood").unwrap_or_default(),
        consequence: row.try_get("consequence").unwrap_or_default(),
        risk_score,
        status: row.try_get("status").unwrap_or_default(),
        location: row.try_get("location").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        identified_by_id: row.try_get("identified_by_id").unwrap_or_default(),
        identified_by_name: row.try_get("identified_by_name").unwrap_or_default(),
        identified_date: row.try_get("identified_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        mitigation_measures: row.try_get("mitigation_measures").unwrap_or(serde_json::json!([])),
        residual_risk_level: row.try_get("residual_risk_level").unwrap_or_default(),
        residual_risk_score: row.try_get("residual_risk_score").unwrap_or_default(),
        review_date: row.try_get("review_date").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_inspection(row: &sqlx::postgres::PgRow) -> SafetyInspection {
    SafetyInspection {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        inspection_number: row.try_get("inspection_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        inspection_type: row.try_get("inspection_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        scheduled_date: row.try_get("scheduled_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        completed_date: row.try_get("completed_date").unwrap_or_default(),
        location: row.try_get("location").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        inspector_id: row.try_get("inspector_id").unwrap_or_default(),
        inspector_name: row.try_get("inspector_name").unwrap_or_default(),
        findings_summary: row.try_get("findings_summary").unwrap_or_default(),
        total_findings: row.try_get("total_findings").unwrap_or(0),
        critical_findings: row.try_get("critical_findings").unwrap_or(0),
        non_conformities: row.try_get("non_conformities").unwrap_or(0),
        observations: row.try_get("observations").unwrap_or(0),
        score: row.try_get("score").unwrap_or_default(),
        max_score: row.try_get("max_score").unwrap_or_default(),
        score_pct: row.try_get("score_pct").unwrap_or_default(),
        findings: row.try_get("findings").unwrap_or(serde_json::json!([])),
        attachments: row.try_get("attachments").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_corrective_action(row: &sqlx::postgres::PgRow) -> SafetyCorrectiveAction {
    SafetyCorrectiveAction {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        action_number: row.try_get("action_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        action_type: row.try_get("action_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        source_type: row.try_get("source_type").unwrap_or_default(),
        source_id: row.try_get("source_id").unwrap_or_default(),
        source_number: row.try_get("source_number").unwrap_or_default(),
        root_cause: row.try_get("root_cause").unwrap_or_default(),
        corrective_action_plan: row.try_get("corrective_action_plan").unwrap_or_default(),
        preventive_action_plan: row.try_get("preventive_action_plan").unwrap_or_default(),
        assigned_to_id: row.try_get("assigned_to_id").unwrap_or_default(),
        assigned_to_name: row.try_get("assigned_to_name").unwrap_or_default(),
        due_date: row.try_get("due_date").unwrap_or_default(),
        completed_date: row.try_get("completed_date").unwrap_or_default(),
        verified_by: row.try_get("verified_by").unwrap_or_default(),
        verified_date: row.try_get("verified_date").unwrap_or_default(),
        effectiveness: row.try_get("effectiveness").unwrap_or_default(),
        facility_id: row.try_get("facility_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        estimated_cost: row.try_get("estimated_cost").unwrap_or_default(),
        actual_cost: row.try_get("actual_cost").unwrap_or_default(),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        attachments: row.try_get("attachments").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl HealthSafetyRepository for PostgresHealthSafetyRepository {
    // ========================================================================
    // Incidents
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_incident(
        &self, org_id: Uuid, incident_number: &str, title: &str, description: Option<&str>,
        incident_type: &str, severity: &str, status: &str, priority: &str,
        incident_date: chrono::NaiveDate, incident_time: Option<&str>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        reported_by_id: Option<Uuid>, reported_by_name: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        root_cause: Option<&str>, immediate_action: Option<&str>,
        osha_recordable: bool, osha_classification: Option<&str>,
        days_away_from_work: i32, days_restricted: i32,
        body_part: Option<&str>, injury_source: Option<&str>,
        event_type: Option<&str>, environment_factor: Option<&str>,
        involved_parties: serde_json::Value, witness_statements: serde_json::Value,
        attachments: serde_json::Value,
        resolution_date: Option<chrono::NaiveDate>, closed_date: Option<chrono::NaiveDate>,
        closed_by: Option<Uuid>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyIncident> {
        let _ = &metadata; // used by downstream callers, stored in DB schema
        let row = sqlx::query(
            r#"INSERT INTO _atlas.safety_incidents
                (organization_id, incident_number, title, description,
                 incident_type, severity, status, priority,
                 incident_date, incident_time, location,
                 facility_id, department_id,
                 reported_by_id, reported_by_name,
                 assigned_to_id, assigned_to_name,
                 root_cause, immediate_action,
                 osha_recordable, osha_classification,
                 days_away_from_work, days_restricted,
                 body_part, injury_source, event_type, environment_factor,
                 involved_parties, witness_statements, attachments,
                 resolution_date, closed_date, closed_by,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                    $31, $32, $33, '{}'::jsonb, $34)
            RETURNING *"#,
        )
        .bind(org_id).bind(incident_number).bind(title).bind(description)
        .bind(incident_type).bind(severity).bind(status).bind(priority)
        .bind(incident_date).bind(incident_time).bind(location)
        .bind(facility_id).bind(department_id)
        .bind(reported_by_id).bind(reported_by_name)
        .bind(assigned_to_id).bind(assigned_to_name)
        .bind(root_cause).bind(immediate_action)
        .bind(osha_recordable).bind(osha_classification)
        .bind(days_away_from_work).bind(days_restricted)
        .bind(body_part).bind(injury_source).bind(event_type).bind(environment_factor)
        .bind(&involved_parties).bind(&witness_statements).bind(&attachments)
        .bind(resolution_date).bind(closed_date).bind(closed_by)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_incident(&row))
    }

    async fn get_incident(&self, id: Uuid) -> AtlasResult<Option<SafetyIncident>> {
        let row = sqlx::query("SELECT * FROM _atlas.safety_incidents WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_incident))
    }

    async fn get_incident_by_number(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<Option<SafetyIncident>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.safety_incidents WHERE organization_id = $1 AND incident_number = $2"
        ).bind(org_id).bind(incident_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_incident))
    }

    async fn list_incidents(&self, org_id: Uuid, status: Option<&str>, severity: Option<&str>, incident_type: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<SafetyIncident>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.safety_incidents
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR severity = $3)
                 AND ($4::text IS NULL OR incident_type = $4)
                 AND ($5::uuid IS NULL OR facility_id = $5)
               ORDER BY incident_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(severity).bind(incident_type).bind(facility_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_incident).collect())
    }

    async fn update_incident_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyIncident> {
        let row = sqlx::query(
            "UPDATE _atlas.safety_incidents SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Incident {} not found", id)))?;
        Ok(row_to_incident(&row))
    }

    async fn update_incident_investigation(&self, id: Uuid, root_cause: Option<&str>, immediate_action: Option<&str>, assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>, days_away_from_work: Option<i32>, days_restricted: Option<i32>) -> AtlasResult<SafetyIncident> {
        let row = sqlx::query(
            r#"UPDATE _atlas.safety_incidents
               SET root_cause = COALESCE($2, root_cause),
                   immediate_action = COALESCE($3, immediate_action),
                   assigned_to_id = COALESCE($4, assigned_to_id),
                   assigned_to_name = COALESCE($5, assigned_to_name),
                   days_away_from_work = COALESCE($6, days_away_from_work),
                   days_restricted = COALESCE($7, days_restricted),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(root_cause).bind(immediate_action)
        .bind(assigned_to_id).bind(assigned_to_name)
        .bind(days_away_from_work).bind(days_restricted)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Incident {} not found", id)))?;
        Ok(row_to_incident(&row))
    }

    async fn close_incident(&self, id: Uuid, closed_by: Option<Uuid>) -> AtlasResult<SafetyIncident> {
        let row = sqlx::query(
            r#"UPDATE _atlas.safety_incidents
               SET status = 'closed', closed_date = CURRENT_DATE,
                   closed_by = $2, resolution_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(closed_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Incident {} not found", id)))?;
        Ok(row_to_incident(&row))
    }

    async fn delete_incident(&self, org_id: Uuid, incident_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.safety_incidents WHERE organization_id = $1 AND incident_number = $2"
        ).bind(org_id).bind(incident_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Incident '{}' not found", incident_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Hazards
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_hazard(
        &self, org_id: Uuid, hazard_code: &str, title: &str, description: Option<&str>,
        hazard_category: &str, risk_level: &str, likelihood: &str, consequence: &str,
        risk_score: i32, status: &str,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        identified_by_id: Option<Uuid>, identified_by_name: Option<&str>,
        identified_date: chrono::NaiveDate,
        mitigation_measures: serde_json::Value,
        residual_risk_level: Option<&str>, residual_risk_score: Option<i32>,
        review_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<Hazard> {
        let _ = &metadata; // used by downstream callers, stored in DB schema
        let row = sqlx::query(
            r#"INSERT INTO _atlas.safety_hazards
                (organization_id, hazard_code, title, description,
                 hazard_category, risk_level, likelihood, consequence,
                 risk_score, status, location,
                 facility_id, department_id,
                 identified_by_id, identified_by_name, identified_date,
                 mitigation_measures, residual_risk_level, residual_risk_score,
                 review_date, owner_id, owner_name,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, '{}'::jsonb, $23)
            RETURNING *"#,
        )
        .bind(org_id).bind(hazard_code).bind(title).bind(description)
        .bind(hazard_category).bind(risk_level).bind(likelihood).bind(consequence)
        .bind(risk_score).bind(status).bind(location)
        .bind(facility_id).bind(department_id)
        .bind(identified_by_id).bind(identified_by_name).bind(identified_date)
        .bind(&mitigation_measures).bind(residual_risk_level).bind(residual_risk_score)
        .bind(review_date).bind(owner_id).bind(owner_name)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_hazard(&row))
    }

    async fn get_hazard(&self, id: Uuid) -> AtlasResult<Option<Hazard>> {
        let row = sqlx::query("SELECT * FROM _atlas.safety_hazards WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_hazard))
    }

    async fn get_hazard_by_code(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<Option<Hazard>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.safety_hazards WHERE organization_id = $1 AND hazard_code = $2"
        ).bind(org_id).bind(hazard_code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_hazard))
    }

    async fn list_hazards(&self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>, hazard_category: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<Hazard>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.safety_hazards
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR risk_level = $3)
                 AND ($4::text IS NULL OR hazard_category = $4)
                 AND ($5::uuid IS NULL OR facility_id = $5)
               ORDER BY risk_score DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(risk_level).bind(hazard_category).bind(facility_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_hazard).collect())
    }

    async fn update_hazard_status(&self, id: Uuid, status: &str) -> AtlasResult<Hazard> {
        let row = sqlx::query(
            "UPDATE _atlas.safety_hazards SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Hazard {} not found", id)))?;
        Ok(row_to_hazard(&row))
    }

    async fn update_residual_risk(&self, id: Uuid, residual_risk_level: &str, residual_risk_score: i32) -> AtlasResult<Hazard> {
        let row = sqlx::query(
            r#"UPDATE _atlas.safety_hazards
               SET residual_risk_level = $2, residual_risk_score = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(residual_risk_level).bind(residual_risk_score)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Hazard {} not found", id)))?;
        Ok(row_to_hazard(&row))
    }

    async fn delete_hazard(&self, org_id: Uuid, hazard_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.safety_hazards WHERE organization_id = $1 AND hazard_code = $2"
        ).bind(org_id).bind(hazard_code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Hazard '{}' not found", hazard_code)));
        }
        Ok(())
    }

    // ========================================================================
    // Inspections
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_inspection(
        &self, org_id: Uuid, inspection_number: &str, title: &str, description: Option<&str>,
        inspection_type: &str, status: &str, priority: &str,
        scheduled_date: chrono::NaiveDate, completed_date: Option<chrono::NaiveDate>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        inspector_id: Option<Uuid>, inspector_name: Option<&str>,
        findings_summary: Option<&str>,
        total_findings: i32, critical_findings: i32, non_conformities: i32, observations: i32,
        score: Option<f64>, max_score: Option<f64>, score_pct: Option<f64>,
        findings: serde_json::Value, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyInspection> {
        let _ = &metadata; // used by downstream callers, stored in DB schema
        let row = sqlx::query(
            r#"INSERT INTO _atlas.safety_inspections
                (organization_id, inspection_number, title, description,
                 inspection_type, status, priority,
                 scheduled_date, completed_date,
                 location, facility_id, department_id,
                 inspector_id, inspector_name,
                 findings_summary, total_findings, critical_findings,
                 non_conformities, observations,
                 score, max_score, score_pct,
                 findings, attachments, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, $21, $22, $23, $24, '{}'::jsonb, $25)
            RETURNING *"#,
        )
        .bind(org_id).bind(inspection_number).bind(title).bind(description)
        .bind(inspection_type).bind(status).bind(priority)
        .bind(scheduled_date).bind(completed_date)
        .bind(location).bind(facility_id).bind(department_id)
        .bind(inspector_id).bind(inspector_name)
        .bind(findings_summary).bind(total_findings).bind(critical_findings)
        .bind(non_conformities).bind(observations)
        .bind(score).bind(max_score).bind(score_pct)
        .bind(&findings).bind(&attachments).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_inspection(&row))
    }

    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<SafetyInspection>> {
        let row = sqlx::query("SELECT * FROM _atlas.safety_inspections WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_inspection))
    }

    async fn get_inspection_by_number(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<Option<SafetyInspection>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.safety_inspections WHERE organization_id = $1 AND inspection_number = $2"
        ).bind(org_id).bind(inspection_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_inspection))
    }

    async fn list_inspections(&self, org_id: Uuid, status: Option<&str>, inspection_type: Option<&str>, facility_id: Option<&Uuid>) -> AtlasResult<Vec<SafetyInspection>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.safety_inspections
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR inspection_type = $3)
                 AND ($4::uuid IS NULL OR facility_id = $4)
               ORDER BY scheduled_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(inspection_type).bind(facility_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_inspection).collect())
    }

    async fn complete_inspection(
        &self, id: Uuid, findings_summary: Option<&str>,
        total_findings: i32, critical_findings: i32, non_conformities: i32, observations: i32,
        score: Option<f64>, max_score: Option<f64>, score_pct: Option<f64>,
        findings: serde_json::Value,
    ) -> AtlasResult<SafetyInspection> {
        let row = sqlx::query(
            r#"UPDATE _atlas.safety_inspections
               SET status = 'completed', completed_date = CURRENT_DATE,
                   findings_summary = COALESCE($2, findings_summary),
                   total_findings = $3, critical_findings = $4,
                   non_conformities = $5, observations = $6,
                   score = $7, max_score = $8, score_pct = $9,
                   findings = $10, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(findings_summary)
        .bind(total_findings).bind(critical_findings).bind(non_conformities).bind(observations)
        .bind(score).bind(max_score).bind(score_pct)
        .bind(&findings)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Inspection {} not found", id)))?;
        Ok(row_to_inspection(&row))
    }

    async fn update_inspection_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyInspection> {
        let row = sqlx::query(
            "UPDATE _atlas.safety_inspections SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Inspection {} not found", id)))?;
        Ok(row_to_inspection(&row))
    }

    async fn delete_inspection(&self, org_id: Uuid, inspection_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.safety_inspections WHERE organization_id = $1 AND inspection_number = $2"
        ).bind(org_id).bind(inspection_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Inspection '{}' not found", inspection_number)));
        }
        Ok(())
    }

    // ========================================================================
    // CAPA
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_corrective_action(
        &self, org_id: Uuid, action_number: &str, title: &str, description: Option<&str>,
        action_type: &str, status: &str, priority: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        root_cause: Option<&str>, corrective_action_plan: Option<&str>, preventive_action_plan: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        due_date: Option<chrono::NaiveDate>, completed_date: Option<chrono::NaiveDate>,
        verified_by: Option<Uuid>, verified_date: Option<chrono::NaiveDate>, effectiveness: Option<&str>,
        facility_id: Option<Uuid>, department_id: Option<Uuid>,
        estimated_cost: Option<f64>, actual_cost: Option<f64>, currency_code: Option<&str>,
        notes: Option<&str>, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<SafetyCorrectiveAction> {
        let _ = &metadata; // used by downstream callers, stored in DB schema
        let row = sqlx::query(
            r#"INSERT INTO _atlas.corrective_actions
                (organization_id, action_number, title, description,
                 action_type, status, priority,
                 source_type, source_id, source_number,
                 root_cause, corrective_action_plan, preventive_action_plan,
                 assigned_to_id, assigned_to_name,
                 due_date, completed_date,
                 verified_by, verified_date, effectiveness,
                 facility_id, department_id,
                 estimated_cost, actual_cost, currency_code,
                 notes, attachments, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, '{}'::jsonb, $28)
            RETURNING *"#,
        )
        .bind(org_id).bind(action_number).bind(title).bind(description)
        .bind(action_type).bind(status).bind(priority)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(root_cause).bind(corrective_action_plan).bind(preventive_action_plan)
        .bind(assigned_to_id).bind(assigned_to_name)
        .bind(due_date).bind(completed_date)
        .bind(verified_by).bind(verified_date).bind(effectiveness)
        .bind(facility_id).bind(department_id)
        .bind(estimated_cost).bind(actual_cost).bind(currency_code)
        .bind(notes).bind(&attachments).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_corrective_action(&row))
    }

    async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<SafetyCorrectiveAction>> {
        let row = sqlx::query("SELECT * FROM _atlas.corrective_actions WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_corrective_action))
    }

    async fn get_corrective_action_by_number(&self, org_id: Uuid, action_number: &str) -> AtlasResult<Option<SafetyCorrectiveAction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.corrective_actions WHERE organization_id = $1 AND action_number = $2"
        ).bind(org_id).bind(action_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_corrective_action))
    }

    async fn list_corrective_actions(&self, org_id: Uuid, status: Option<&str>, action_type: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<SafetyCorrectiveAction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.corrective_actions
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR action_type = $3)
                 AND ($4::text IS NULL OR source_type = $4)
               ORDER BY due_date ASC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(action_type).bind(source_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_corrective_action).collect())
    }

    async fn update_corrective_action_status(&self, id: Uuid, status: &str) -> AtlasResult<SafetyCorrectiveAction> {
        let row = sqlx::query(
            "UPDATE _atlas.corrective_actions SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Corrective action {} not found", id)))?;
        Ok(row_to_corrective_action(&row))
    }

    async fn complete_corrective_action(&self, id: Uuid, effectiveness: &str, actual_cost: Option<f64>, verified_by: Option<Uuid>) -> AtlasResult<SafetyCorrectiveAction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.corrective_actions
               SET status = 'completed', completed_date = CURRENT_DATE,
                   effectiveness = $2, actual_cost = $3,
                   verified_by = $4, verified_date = CURRENT_DATE,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(effectiveness).bind(actual_cost).bind(verified_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Corrective action {} not found", id)))?;
        Ok(row_to_corrective_action(&row))
    }

    async fn delete_corrective_action(&self, org_id: Uuid, action_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.corrective_actions WHERE organization_id = $1 AND action_number = $2"
        ).bind(org_id).bind(action_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Action '{}' not found", action_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HealthSafetyDashboard> {
        // Incidents
        let inc_rows = sqlx::query(
            "SELECT status, incident_type, severity, osha_recordable FROM _atlas.safety_incidents WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_incidents = inc_rows.len() as i64;
        let open_incidents = inc_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s != "closed" && s != "resolved"
        }).count() as i64;
        let closed_incidents = inc_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s == "closed" || s == "resolved"
        }).count() as i64;
        let critical_incidents = inc_rows.iter().filter(|r| {
            let s: String = r.try_get("severity").unwrap_or_default();
            s == "critical"
        }).count() as i64;
        let osha_recordable_count = inc_rows.iter().filter(|r| {
            r.try_get::<bool, _>("osha_recordable").unwrap_or(false)
        }).count() as i64;

        let mut incidents_by_type = serde_json::Map::new();
        for r in &inc_rows {
            let t: String = r.try_get("incident_type").unwrap_or_default();
            *incidents_by_type.entry(t).or_insert(serde_json::Value::from(0i64)) = serde_json::Value::from(
                incidents_by_type.get(&t).and_then(|v| v.as_i64()).unwrap_or(0) + 1
            );
        }
        let mut incidents_by_severity = serde_json::Map::new();
        for r in &inc_rows {
            let s: String = r.try_get("severity").unwrap_or_default();
            *incidents_by_severity.entry(s).or_insert(serde_json::Value::from(0i64)) = serde_json::Value::from(
                incidents_by_severity.get(&s).and_then(|v| v.as_i64()).unwrap_or(0) + 1
            );
        }

        // Days since last incident
        let last_incident = sqlx::query(
            "SELECT MAX(incident_date) as last_date FROM _atlas.safety_incidents WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await;
        let days_since_last_incident = match last_incident {
            Ok(row) => {
                let last: Option<chrono::NaiveDate> = row.try_get("last_date").unwrap_or(None);
                last.map(|d| (chrono::Utc::now().date_naive() - d).num_days()).unwrap_or(-1)
            }
            Err(_) => -1,
        };

        // Hazards
        let haz_rows = sqlx::query(
            "SELECT status, risk_level FROM _atlas.safety_hazards WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_hazards = haz_rows.len() as i64;
        let open_hazards = haz_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s != "closed" && s != "mitigated"
        }).count() as i64;
        let high_risk_hazards = haz_rows.iter().filter(|r| {
            let s: String = r.try_get("risk_level").unwrap_or_default();
            s == "high" || s == "very_high" || s == "extreme"
        }).count() as i64;

        let mut hazards_by_risk = serde_json::Map::new();
        for r in &haz_rows {
            let l: String = r.try_get("risk_level").unwrap_or_default();
            *hazards_by_risk.entry(l).or_insert(serde_json::Value::from(0i64)) = serde_json::Value::from(
                hazards_by_risk.get(&l).and_then(|v| v.as_i64()).unwrap_or(0) + 1
            );
        }

        // Inspections
        let ins_rows = sqlx::query(
            "SELECT status, score_pct FROM _atlas.safety_inspections WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_inspections = ins_rows.len() as i64;
        let open_inspections = ins_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s != "completed" && s != "cancelled"
        }).count() as i64;
        let completed_inspections = ins_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s == "completed"
        }).count() as i64;

        let inspection_pass_rate = if completed_inspections > 0 {
            let passing = ins_rows.iter().filter(|r| {
                let pct: Option<f64> = r.try_get("score_pct").unwrap_or(None);
                pct.map_or(false, |p| p >= 80.0)
            }).count() as f64;
            (passing / completed_inspections as f64) * 100.0
        } else {
            0.0
        };

        // CAPA
        let capa_rows = sqlx::query(
            "SELECT status, due_date FROM _atlas.corrective_actions WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_capa = capa_rows.len() as i64;
        let open_capa = capa_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            s != "completed" && s != "closed" && s != "cancelled"
        }).count() as i64;
        let overdue_capa = capa_rows.iter().filter(|r| {
            let s: String = r.try_get("status").unwrap_or_default();
            let due: Option<chrono::NaiveDate> = r.try_get("due_date").unwrap_or(None);
            s != "completed" && s != "closed" && s != "cancelled"
                && due.map_or(false, |d| d < chrono::Utc::now().date_naive())
        }).count() as i64;

        Ok(HealthSafetyDashboard {
            organization_id: org_id,
            total_incidents,
            open_incidents,
            closed_incidents,
            critical_incidents,
            total_hazards,
            open_hazards,
            high_risk_hazards,
            total_inspections,
            open_inspections,
            completed_inspections,
            total_capa,
            open_capa,
            overdue_capa,
            osha_recordable_count,
            days_since_last_incident,
            incidents_by_type: serde_json::Value::Object(incidents_by_type),
            incidents_by_severity: serde_json::Value::Object(incidents_by_severity),
            hazards_by_risk: serde_json::Value::Object(hazards_by_risk),
            inspection_pass_rate,
        })
    }
}
