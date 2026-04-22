//! Quality Management Repository
//!
//! PostgreSQL storage for inspection plans, inspections, non-conformance
//! reports, corrective actions, quality holds, and related data.

use atlas_shared::{
    QualityInspectionPlan, QualityInspectionPlanCriterion,
    QualityInspection, QualityInspectionResult,
    NonConformanceReport, CorrectiveAction,
    QualityHold, QualityDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for quality management data storage
#[async_trait]
pub trait QualityManagementRepository: Send + Sync {
    // ----------------------------------------------------------------
    // Inspection Plans
    // ----------------------------------------------------------------
    async fn create_plan(
        &self, org_id: Uuid, plan_code: &str, name: &str, description: Option<&str>,
        plan_type: &str, item_id: Option<Uuid>, item_code: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        inspection_trigger: &str, sampling_method: &str,
        sample_size_percent: Option<&str>, accept_number: Option<i32>,
        reject_number: Option<i32>, frequency: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionPlan>;

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualityInspectionPlan>>;
    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<QualityInspectionPlan>>;
    async fn list_plans(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualityInspectionPlan>>;
    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ----------------------------------------------------------------
    // Plan Criteria
    // ----------------------------------------------------------------
    async fn create_criterion(
        &self, org_id: Uuid, plan_id: Uuid, criterion_number: i32,
        name: &str, description: Option<&str>, characteristic: &str,
        measurement_type: &str, target_value: Option<&str>,
        lower_spec_limit: Option<&str>, upper_spec_limit: Option<&str>,
        unit_of_measure: Option<&str>, is_mandatory: bool,
        weight: &str, criticality: &str,
    ) -> AtlasResult<QualityInspectionPlanCriterion>;

    async fn list_criteria(&self, plan_id: Uuid) -> AtlasResult<Vec<QualityInspectionPlanCriterion>>;
    async fn delete_criterion(&self, id: Uuid) -> AtlasResult<()>;

    // ----------------------------------------------------------------
    // Inspections
    // ----------------------------------------------------------------
    async fn create_inspection(
        &self, org_id: Uuid, inspection_number: &str, plan_id: Uuid,
        source_type: &str, source_id: Option<Uuid>,
        source_number: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        lot_number: Option<&str>, quantity_inspected: &str,
        quantity_accepted: &str, quantity_rejected: &str,
        unit_of_measure: Option<&str>, inspector_id: Option<Uuid>,
        inspector_name: Option<&str>, inspection_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspection>;

    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<QualityInspection>>;
    async fn get_inspection_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<QualityInspection>>;
    async fn list_inspections(
        &self, org_id: Uuid, status: Option<&str>,
        plan_id: Option<Uuid>, limit: Option<i64>,
    ) -> AtlasResult<Vec<QualityInspection>>;
    async fn update_inspection_status(
        &self, id: Uuid, status: &str,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<QualityInspection>;
    async fn update_inspection_verdict(
        &self, id: Uuid, verdict: &str, score: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<QualityInspection>;

    // ----------------------------------------------------------------
    // Inspection Results
    // ----------------------------------------------------------------
    async fn create_result(
        &self, org_id: Uuid, inspection_id: Uuid, criterion_id: Option<Uuid>,
        criterion_name: &str, characteristic: &str,
        measurement_type: &str, observed_value: Option<&str>,
        target_value: Option<&str>, lower_spec_limit: Option<&str>,
        upper_spec_limit: Option<&str>, unit_of_measure: Option<&str>,
        result_status: &str, deviation: Option<&str>,
        notes: Option<&str>, evaluated_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionResult>;

    async fn list_results(&self, inspection_id: Uuid) -> AtlasResult<Vec<QualityInspectionResult>>;
    async fn update_result_status(
        &self, id: Uuid, status: &str, deviation: Option<&str>,
    ) -> AtlasResult<QualityInspectionResult>;

    // ----------------------------------------------------------------
    // Non-Conformance Reports
    // ----------------------------------------------------------------
    async fn create_ncr(
        &self, org_id: Uuid, ncr_number: &str, title: &str,
        description: Option<&str>, ncr_type: &str, severity: &str,
        origin: &str, source_type: Option<&str>, source_id: Option<Uuid>,
        source_number: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, supplier_id: Option<Uuid>,
        supplier_name: Option<&str>, detected_date: chrono::NaiveDate,
        detected_by: Option<&str>, responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NonConformanceReport>;

    async fn get_ncr(&self, id: Uuid) -> AtlasResult<Option<NonConformanceReport>>;
    async fn get_ncr_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<NonConformanceReport>>;
    async fn list_ncrs(
        &self, org_id: Uuid, status: Option<&str>,
        severity: Option<&str>, limit: Option<i64>,
    ) -> AtlasResult<Vec<NonConformanceReport>>;
    async fn update_ncr_status(
        &self, id: Uuid, status: &str,
        resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<NonConformanceReport>;
    async fn update_ncr_resolution(
        &self, id: Uuid, resolution_description: &str,
        resolution_type: &str, resolved_by: Option<&str>,
    ) -> AtlasResult<NonConformanceReport>;

    // ----------------------------------------------------------------
    // Corrective & Preventive Actions
    // ----------------------------------------------------------------
    async fn create_corrective_action(
        &self, org_id: Uuid, ncr_id: Uuid, action_number: &str,
        action_type: &str, title: &str, description: Option<&str>,
        root_cause: Option<&str>, corrective_action_desc: Option<&str>,
        preventive_action_desc: Option<&str>,
        assigned_to: Option<&str>, due_date: Option<chrono::NaiveDate>,
        priority: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<CorrectiveAction>;

    async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<CorrectiveAction>>;
    async fn list_corrective_actions(&self, ncr_id: Uuid) -> AtlasResult<Vec<CorrectiveAction>>;
    async fn update_corrective_action_status(
        &self, id: Uuid, status: &str,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        effectiveness_rating: Option<i32>,
    ) -> AtlasResult<CorrectiveAction>;

    // ----------------------------------------------------------------
    // Quality Holds
    // ----------------------------------------------------------------
    async fn create_hold(
        &self, org_id: Uuid, hold_number: &str, reason: &str,
        description: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, lot_number: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        source_type: Option<&str>, source_id: Option<Uuid>,
        source_number: Option<&str>, hold_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityHold>;

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<QualityHold>>;
    async fn list_holds(
        &self, org_id: Uuid, status: Option<&str>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<QualityHold>>;
    async fn update_hold_status(
        &self, id: Uuid, status: &str, released_by: Option<Uuid>,
        release_notes: Option<&str>,
    ) -> AtlasResult<QualityHold>;

    // ----------------------------------------------------------------
    // Dashboard
    // ----------------------------------------------------------------
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<QualityDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresQualityManagementRepository {
    pool: PgPool,
}

impl PostgresQualityManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    fn get_numeric(&self, row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
}

#[async_trait]
impl QualityManagementRepository for PostgresQualityManagementRepository {
    // ========================================================================
    // Inspection Plans
    // ========================================================================

    async fn create_plan(
        &self, org_id: Uuid, plan_code: &str, name: &str, description: Option<&str>,
        plan_type: &str, item_id: Option<Uuid>, item_code: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        inspection_trigger: &str, sampling_method: &str,
        sample_size_percent: Option<&str>, accept_number: Option<i32>,
        reject_number: Option<i32>, frequency: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionPlan> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_inspection_plans
                (organization_id, plan_code, name, description, plan_type,
                 item_id, item_code, supplier_id, supplier_name,
                 inspection_trigger, sampling_method, sample_size_percent,
                 accept_number, reject_number, frequency,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12::numeric,$13,$14,$15,$16,$17,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(plan_code).bind(name).bind(description)
        .bind(plan_type).bind(item_id).bind(item_code)
        .bind(supplier_id).bind(supplier_name)
        .bind(inspection_trigger).bind(sampling_method).bind(sample_size_percent)
        .bind(accept_number).bind(reject_number).bind(frequency)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_plan(&row))
    }

    async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualityInspectionPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_inspection_plans WHERE organization_id=$1 AND plan_code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_plan(&r)))
    }

    async fn get_plan_by_id(&self, id: Uuid) -> AtlasResult<Option<QualityInspectionPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_inspection_plans WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_plan(&r)))
    }

    async fn list_plans(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualityInspectionPlan>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.quality_inspection_plans WHERE organization_id=$1 AND is_active=true ORDER BY plan_code"
            ).bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.quality_inspection_plans WHERE organization_id=$1 ORDER BY plan_code"
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_plan).collect())
    }

    async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.quality_inspection_plans WHERE organization_id=$1 AND plan_code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Plan Criteria
    // ========================================================================

    async fn create_criterion(
        &self, org_id: Uuid, plan_id: Uuid, criterion_number: i32,
        name: &str, description: Option<&str>, characteristic: &str,
        measurement_type: &str, target_value: Option<&str>,
        lower_spec_limit: Option<&str>, upper_spec_limit: Option<&str>,
        unit_of_measure: Option<&str>, is_mandatory: bool,
        weight: &str, criticality: &str,
    ) -> AtlasResult<QualityInspectionPlanCriterion> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_plan_criteria
                (organization_id, plan_id, criterion_number, name, description,
                 characteristic, measurement_type, target_value,
                 lower_spec_limit, upper_spec_limit, unit_of_measure,
                 is_mandatory, weight, criticality)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10::numeric,$11,$12,$13::numeric,$14)
            RETURNING *"#,
        )
        .bind(org_id).bind(plan_id).bind(criterion_number).bind(name)
        .bind(description).bind(characteristic).bind(measurement_type)
        .bind(target_value).bind(lower_spec_limit).bind(upper_spec_limit)
        .bind(unit_of_measure).bind(is_mandatory).bind(weight).bind(criticality)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_criterion(&row))
    }

    async fn list_criteria(&self, plan_id: Uuid) -> AtlasResult<Vec<QualityInspectionPlanCriterion>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.quality_plan_criteria WHERE plan_id=$1 AND is_active=true ORDER BY criterion_number"
        )
        .bind(plan_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_criterion).collect())
    }

    async fn delete_criterion(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.quality_plan_criteria WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Inspections
    // ========================================================================

    async fn create_inspection(
        &self, org_id: Uuid, inspection_number: &str, plan_id: Uuid,
        source_type: &str, source_id: Option<Uuid>,
        source_number: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        lot_number: Option<&str>, quantity_inspected: &str,
        quantity_accepted: &str, quantity_rejected: &str,
        unit_of_measure: Option<&str>, inspector_id: Option<Uuid>,
        inspector_name: Option<&str>, inspection_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspection> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_inspections
                (organization_id, inspection_number, plan_id,
                 source_type, source_id, source_number,
                 item_id, item_code, item_description,
                 lot_number, quantity_inspected, quantity_accepted, quantity_rejected,
                 unit_of_measure, inspector_id, inspector_name, inspection_date,
                 created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::numeric,$12::numeric,$13::numeric,$14,$15,$16,$17,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(inspection_number).bind(plan_id)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(lot_number).bind(quantity_inspected).bind(quantity_accepted).bind(quantity_rejected)
        .bind(unit_of_measure).bind(inspector_id).bind(inspector_name).bind(inspection_date)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_inspection(&row))
    }

    async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<QualityInspection>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_inspections WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_inspection(&r)))
    }

    async fn get_inspection_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<QualityInspection>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_inspections WHERE organization_id=$1 AND inspection_number=$2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_inspection(&r)))
    }

    async fn list_inspections(
        &self, org_id: Uuid, status: Option<&str>,
        plan_id: Option<Uuid>, limit: Option<i64>,
    ) -> AtlasResult<Vec<QualityInspection>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.quality_inspections
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::uuid IS NULL OR plan_id=$3)
            ORDER BY created_at DESC LIMIT COALESCE($4, 100)"#,
        )
        .bind(org_id).bind(status).bind(plan_id).bind(limit)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_inspection).collect())
    }

    async fn update_inspection_status(
        &self, id: Uuid, status: &str,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<QualityInspection> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_inspections SET status=$2,
                completed_at=COALESCE($3, completed_at),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(completed_at)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_inspection(&row))
    }

    async fn update_inspection_verdict(
        &self, id: Uuid, verdict: &str, score: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<QualityInspection> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_inspections SET
                verdict=$2, overall_score=COALESCE($3::numeric, overall_score),
                notes=COALESCE($4, notes),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(verdict).bind(score).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_inspection(&row))
    }

    // ========================================================================
    // Inspection Results
    // ========================================================================

    async fn create_result(
        &self, org_id: Uuid, inspection_id: Uuid, criterion_id: Option<Uuid>,
        criterion_name: &str, characteristic: &str,
        measurement_type: &str, observed_value: Option<&str>,
        target_value: Option<&str>, lower_spec_limit: Option<&str>,
        upper_spec_limit: Option<&str>, unit_of_measure: Option<&str>,
        result_status: &str, deviation: Option<&str>,
        notes: Option<&str>, evaluated_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionResult> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_inspection_results
                (organization_id, inspection_id, criterion_id,
                 criterion_name, characteristic, measurement_type,
                 observed_value, target_value, lower_spec_limit,
                 upper_spec_limit, unit_of_measure, result_status,
                 deviation, notes, evaluated_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8::numeric,$9::numeric,$10::numeric,$11,$12,$13::numeric,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(inspection_id).bind(criterion_id)
        .bind(criterion_name).bind(characteristic).bind(measurement_type)
        .bind(observed_value).bind(target_value).bind(lower_spec_limit)
        .bind(upper_spec_limit).bind(unit_of_measure).bind(result_status)
        .bind(deviation).bind(notes).bind(evaluated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_result(&row))
    }

    async fn list_results(&self, inspection_id: Uuid) -> AtlasResult<Vec<QualityInspectionResult>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.quality_inspection_results WHERE inspection_id=$1 ORDER BY created_at"
        )
        .bind(inspection_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_result).collect())
    }

    async fn update_result_status(
        &self, id: Uuid, status: &str, deviation: Option<&str>,
    ) -> AtlasResult<QualityInspectionResult> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_inspection_results SET
                result_status=$2, deviation=COALESCE($3::numeric, deviation),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(deviation)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_result(&row))
    }

    // ========================================================================
    // Non-Conformance Reports
    // ========================================================================

    async fn create_ncr(
        &self, org_id: Uuid, ncr_number: &str, title: &str,
        description: Option<&str>, ncr_type: &str, severity: &str,
        origin: &str, source_type: Option<&str>, source_id: Option<Uuid>,
        source_number: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, supplier_id: Option<Uuid>,
        supplier_name: Option<&str>, detected_date: chrono::NaiveDate,
        detected_by: Option<&str>, responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NonConformanceReport> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_non_conformance_reports
                (organization_id, ncr_number, title, description,
                 ncr_type, severity, origin, source_type, source_id, source_number,
                 item_id, item_code, supplier_id, supplier_name,
                 detected_date, detected_by, responsible_party, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
            RETURNING *"#,
        )
        .bind(org_id).bind(ncr_number).bind(title).bind(description)
        .bind(ncr_type).bind(severity).bind(origin)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(item_id).bind(item_code).bind(supplier_id).bind(supplier_name)
        .bind(detected_date).bind(detected_by).bind(responsible_party).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_ncr(&row))
    }

    async fn get_ncr(&self, id: Uuid) -> AtlasResult<Option<NonConformanceReport>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_non_conformance_reports WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_ncr(&r)))
    }

    async fn get_ncr_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<NonConformanceReport>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_non_conformance_reports WHERE organization_id=$1 AND ncr_number=$2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_ncr(&r)))
    }

    async fn list_ncrs(
        &self, org_id: Uuid, status: Option<&str>,
        severity: Option<&str>, limit: Option<i64>,
    ) -> AtlasResult<Vec<NonConformanceReport>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.quality_non_conformance_reports
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::text IS NULL OR severity=$3)
            ORDER BY detected_date DESC LIMIT COALESCE($4, 100)"#,
        )
        .bind(org_id).bind(status).bind(severity).bind(limit)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_ncr).collect())
    }

    async fn update_ncr_status(
        &self, id: Uuid, status: &str,
        resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<NonConformanceReport> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_non_conformance_reports SET status=$2,
                resolved_at=COALESCE($3, resolved_at),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(resolved_at)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_ncr(&row))
    }

    async fn update_ncr_resolution(
        &self, id: Uuid, resolution_description: &str,
        resolution_type: &str, resolved_by: Option<&str>,
    ) -> AtlasResult<NonConformanceReport> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_non_conformance_reports SET
                resolution_description=$2, resolution_type=$3,
                resolved_by=COALESCE($4, resolved_by),
                resolved_at=CASE WHEN resolved_at IS NULL THEN now() ELSE resolved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(resolution_description).bind(resolution_type).bind(resolved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_ncr(&row))
    }

    // ========================================================================
    // Corrective Actions
    // ========================================================================

    async fn create_corrective_action(
        &self, org_id: Uuid, ncr_id: Uuid, action_number: &str,
        action_type: &str, title: &str, description: Option<&str>,
        root_cause: Option<&str>, corrective_action_desc: Option<&str>,
        preventive_action_desc: Option<&str>,
        assigned_to: Option<&str>, due_date: Option<chrono::NaiveDate>,
        priority: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<CorrectiveAction> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_corrective_actions
                (organization_id, ncr_id, action_number, action_type,
                 title, description, root_cause,
                 corrective_action_desc, preventive_action_desc,
                 assigned_to, due_date, priority, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(ncr_id).bind(action_number).bind(action_type)
        .bind(title).bind(description).bind(root_cause)
        .bind(corrective_action_desc).bind(preventive_action_desc)
        .bind(assigned_to).bind(due_date).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_corrective_action(&row))
    }

    async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<CorrectiveAction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_corrective_actions WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_corrective_action(&r)))
    }

    async fn list_corrective_actions(&self, ncr_id: Uuid) -> AtlasResult<Vec<CorrectiveAction>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.quality_corrective_actions WHERE ncr_id=$1 ORDER BY action_number"
        )
        .bind(ncr_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_corrective_action).collect())
    }

    async fn update_corrective_action_status(
        &self, id: Uuid, status: &str,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        effectiveness_rating: Option<i32>,
    ) -> AtlasResult<CorrectiveAction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_corrective_actions SET status=$2,
                completed_at=COALESCE($3, completed_at),
                effectiveness_rating=COALESCE($4, effectiveness_rating),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(completed_at).bind(effectiveness_rating)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_corrective_action(&row))
    }

    // ========================================================================
    // Quality Holds
    // ========================================================================

    async fn create_hold(
        &self, org_id: Uuid, hold_number: &str, reason: &str,
        description: Option<&str>, item_id: Option<Uuid>,
        item_code: Option<&str>, lot_number: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        source_type: Option<&str>, source_id: Option<Uuid>,
        source_number: Option<&str>, hold_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityHold> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.quality_holds
                (organization_id, hold_number, reason, description,
                 item_id, item_code, lot_number,
                 supplier_id, supplier_name,
                 source_type, source_id, source_number,
                 hold_type, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *"#,
        )
        .bind(org_id).bind(hold_number).bind(reason).bind(description)
        .bind(item_id).bind(item_code).bind(lot_number)
        .bind(supplier_id).bind(supplier_name)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(hold_type).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<QualityHold>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.quality_holds WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_hold(&r)))
    }

    async fn list_holds(
        &self, org_id: Uuid, status: Option<&str>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<QualityHold>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.quality_holds
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::uuid IS NULL OR item_id=$3)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(item_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_hold).collect())
    }

    async fn update_hold_status(
        &self, id: Uuid, status: &str, released_by: Option<Uuid>,
        release_notes: Option<&str>,
    ) -> AtlasResult<QualityHold> {
        let row = sqlx::query(
            r#"UPDATE _atlas.quality_holds SET status=$2,
                released_by=COALESCE($3, released_by),
                release_notes=COALESCE($4, release_notes),
                released_at=CASE WHEN $2='released' AND released_at IS NULL THEN now() ELSE released_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(released_by).bind(release_notes)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_hold(&row))
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<QualityDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM _atlas.quality_inspection_plans WHERE organization_id=$1 AND is_active=true) as active_plans,
                (SELECT COUNT(*) FROM _atlas.quality_inspections WHERE organization_id=$1 AND status IN ('planned','in_progress')) as pending_inspections,
                (SELECT COUNT(*) FROM _atlas.quality_inspections WHERE organization_id=$1 AND status='completed' AND verdict='pass') as passed_inspections,
                (SELECT COUNT(*) FROM _atlas.quality_inspections WHERE organization_id=$1 AND status='completed' AND verdict='fail') as failed_inspections,
                (SELECT COUNT(*) FROM _atlas.quality_non_conformance_reports WHERE organization_id=$1 AND status IN ('open','under_investigation')) as open_ncrs,
                (SELECT COUNT(*) FROM _atlas.quality_non_conformance_reports WHERE organization_id=$1) as total_ncrs,
                (SELECT COUNT(*) FROM _atlas.quality_corrective_actions WHERE organization_id=$1 AND status IN ('open','in_progress')) as open_corrective_actions,
                (SELECT COUNT(*) FROM _atlas.quality_corrective_actions WHERE organization_id=$1 AND status='completed') as completed_corrective_actions,
                (SELECT COUNT(*) FROM _atlas.quality_holds WHERE organization_id=$1 AND status='active') as active_holds,
                (SELECT COUNT(*) FROM _atlas.quality_non_conformance_reports WHERE organization_id=$1 AND severity='critical') as critical_ncrs"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let active_plans: i64 = row.try_get("active_plans").unwrap_or(0);
        let pending_inspections: i64 = row.try_get("pending_inspections").unwrap_or(0);
        let passed_inspections: i64 = row.try_get("passed_inspections").unwrap_or(0);
        let failed_inspections: i64 = row.try_get("failed_inspections").unwrap_or(0);
        let open_ncrs: i64 = row.try_get("open_ncrs").unwrap_or(0);
        let total_ncrs: i64 = row.try_get("total_ncrs").unwrap_or(0);
        let open_corrective_actions: i64 = row.try_get("open_corrective_actions").unwrap_or(0);
        let completed_corrective_actions: i64 = row.try_get("completed_corrective_actions").unwrap_or(0);
        let active_holds: i64 = row.try_get("active_holds").unwrap_or(0);
        let critical_ncrs: i64 = row.try_get("critical_ncrs").unwrap_or(0);

        let total_completed = passed_inspections + failed_inspections;
        let pass_rate = if total_completed > 0 {
            passed_inspections as f64 / total_completed as f64 * 100.0
        } else {
            0.0
        };

        let cap_rate = if total_ncrs > 0 {
            completed_corrective_actions as f64 / (open_corrective_actions + completed_corrective_actions).max(1) as f64 * 100.0
        } else {
            0.0
        };

        Ok(QualityDashboardSummary {
            total_active_plans: active_plans as i32,
            total_pending_inspections: pending_inspections as i32,
            total_passed_inspections: passed_inspections as i32,
            total_failed_inspections: failed_inspections as i32,
            inspection_pass_rate_percent: format!("{:.1}", pass_rate),
            total_open_ncrs: open_ncrs as i32,
            total_ncrs: total_ncrs as i32,
            critical_ncrs: critical_ncrs as i32,
            total_open_corrective_actions: open_corrective_actions as i32,
            total_completed_corrective_actions: completed_corrective_actions as i32,
            corrective_action_completion_rate_percent: format!("{:.1}", cap_rate),
            total_active_holds: active_holds as i32,
            inspections_by_verdict: serde_json::json!({}),
            ncrs_by_severity: serde_json::json!({}),
            ncrs_by_type: serde_json::json!({}),
        })
    }
}

// ========================================================================
// Row mapping helpers
// ========================================================================

fn row_to_plan(row: &sqlx::postgres::PgRow) -> QualityInspectionPlan {
    QualityInspectionPlan {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        plan_code: row.get("plan_code"),
        name: row.get("name"),
        description: row.get("description"),
        plan_type: row.get("plan_type"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        inspection_trigger: row.get("inspection_trigger"),
        sampling_method: row.get("sampling_method"),
        sample_size_percent: row_to_numeric(row, "sample_size_percent"),
        accept_number: row.get("accept_number"),
        reject_number: row.get("reject_number"),
        frequency: row.get("frequency"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        total_criteria: row.get("total_criteria"),
        total_inspections: row.get("total_inspections"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_criterion(row: &sqlx::postgres::PgRow) -> QualityInspectionPlanCriterion {
    QualityInspectionPlanCriterion {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        plan_id: row.get("plan_id"),
        criterion_number: row.get("criterion_number"),
        name: row.get("name"),
        description: row.get("description"),
        characteristic: row.get("characteristic"),
        measurement_type: row.get("measurement_type"),
        target_value: row_to_numeric(row, "target_value"),
        lower_spec_limit: row_to_numeric(row, "lower_spec_limit"),
        upper_spec_limit: row_to_numeric(row, "upper_spec_limit"),
        unit_of_measure: row.get("unit_of_measure"),
        is_mandatory: row.get("is_mandatory"),
        weight: row_to_numeric(row, "weight"),
        criticality: row.get("criticality"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_inspection(row: &sqlx::postgres::PgRow) -> QualityInspection {
    QualityInspection {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        inspection_number: row.get("inspection_number"),
        plan_id: row.get("plan_id"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        lot_number: row.get("lot_number"),
        quantity_inspected: row_to_numeric(row, "quantity_inspected"),
        quantity_accepted: row_to_numeric(row, "quantity_accepted"),
        quantity_rejected: row_to_numeric(row, "quantity_rejected"),
        unit_of_measure: row.get("unit_of_measure"),
        status: row.get("status"),
        verdict: row.get("verdict"),
        overall_score: row_to_numeric(row, "overall_score"),
        notes: row.get("notes"),
        inspector_id: row.get("inspector_id"),
        inspector_name: row.get("inspector_name"),
        inspection_date: row.get("inspection_date"),
        completed_at: row.get("completed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_result(row: &sqlx::postgres::PgRow) -> QualityInspectionResult {
    QualityInspectionResult {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        inspection_id: row.get("inspection_id"),
        criterion_id: row.get("criterion_id"),
        criterion_name: row.get("criterion_name"),
        characteristic: row.get("characteristic"),
        measurement_type: row.get("measurement_type"),
        observed_value: row_to_numeric(row, "observed_value"),
        target_value: row_to_numeric(row, "target_value"),
        lower_spec_limit: row_to_numeric(row, "lower_spec_limit"),
        upper_spec_limit: row_to_numeric(row, "upper_spec_limit"),
        unit_of_measure: row.get("unit_of_measure"),
        result_status: row.get("result_status"),
        deviation: row_to_numeric(row, "deviation"),
        notes: row.get("notes"),
        evaluated_by: row.get("evaluated_by"),
        evaluated_at: row.get("evaluated_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_ncr(row: &sqlx::postgres::PgRow) -> NonConformanceReport {
    NonConformanceReport {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        ncr_number: row.get("ncr_number"),
        title: row.get("title"),
        description: row.get("description"),
        ncr_type: row.get("ncr_type"),
        severity: row.get("severity"),
        origin: row.get("origin"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        detected_date: row.get("detected_date"),
        detected_by: row.get("detected_by"),
        responsible_party: row.get("responsible_party"),
        status: row.get("status"),
        resolution_description: row.get("resolution_description"),
        resolution_type: row.get("resolution_type"),
        resolved_by: row.get("resolved_by"),
        resolved_at: row.get("resolved_at"),
        total_corrective_actions: row.get("total_corrective_actions"),
        open_corrective_actions: row.get("open_corrective_actions"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_corrective_action(row: &sqlx::postgres::PgRow) -> CorrectiveAction {
    CorrectiveAction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        ncr_id: row.get("ncr_id"),
        action_number: row.get("action_number"),
        action_type: row.get("action_type"),
        title: row.get("title"),
        description: row.get("description"),
        root_cause: row.get("root_cause"),
        corrective_action_desc: row.get("corrective_action_desc"),
        preventive_action_desc: row.get("preventive_action_desc"),
        assigned_to: row.get("assigned_to"),
        due_date: row.get("due_date"),
        status: row.get("status"),
        completed_at: row.get("completed_at"),
        effectiveness_rating: row.get("effectiveness_rating"),
        priority: row.get("priority"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_hold(row: &sqlx::postgres::PgRow) -> QualityHold {
    QualityHold {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        hold_number: row.get("hold_number"),
        reason: row.get("reason"),
        description: row.get("description"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        lot_number: row.get("lot_number"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        hold_type: row.get("hold_type"),
        status: row.get("status"),
        released_by: row.get("released_by"),
        released_at: row.get("released_at"),
        release_notes: row.get("release_notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}
