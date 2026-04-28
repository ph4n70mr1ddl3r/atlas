//! Engineering Change Management Repository
//!
//! PostgreSQL storage for engineering change types, changes, change lines,
//! affected items, approvals, and dashboard.

use atlas_shared::{
    EngineeringChangeType, EngineeringChange, EngineeringChangeLine,
    EngineeringChangeAffectedItem, EngineeringChangeApproval, EcmDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for engineering change management data storage
#[async_trait]
pub trait EngineeringChangeManagementRepository: Send + Sync {
    // Change Types
    async fn create_change_type(
        &self, org_id: Uuid, type_code: &str, name: &str, description: Option<&str>,
        category: &str, approval_required: bool, default_priority: &str,
        number_prefix: &str, description_template: Option<&str>,
        statuses: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeType>;
    async fn get_change_type(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeType>>;
    async fn get_change_type_by_code(&self, org_id: Uuid, type_code: &str) -> AtlasResult<Option<EngineeringChangeType>>;
    async fn list_change_types(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<EngineeringChangeType>>;
    async fn delete_change_type(&self, org_id: Uuid, type_code: &str) -> AtlasResult<()>;

    // Changes
    #[allow(clippy::too_many_arguments)]
    async fn create_change(
        &self, org_id: Uuid, change_number: &str, change_type_id: Option<Uuid>,
        category: &str, title: &str, description: Option<&str>,
        change_reason: Option<&str>, change_reason_description: Option<&str>,
        priority: &str, status: &str, revision: &str,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
        target_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        resolution_code: Option<&str>,
        resolution_notes: Option<&str>,
        parent_change_id: Option<Uuid>,
        superseded_by_id: Option<Uuid>,
        impact_analysis: serde_json::Value,
        estimated_cost: Option<f64>,
        actual_cost: Option<f64>,
        currency_code: &str,
        estimated_hours: Option<f64>,
        actual_hours: Option<f64>,
        regulatory_impact: Option<&str>,
        safety_impact: Option<&str>,
        validation_required: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChange>;
    async fn get_change(&self, id: Uuid) -> AtlasResult<Option<EngineeringChange>>;
    async fn get_change_by_number(&self, org_id: Uuid, change_number: &str) -> AtlasResult<Option<EngineeringChange>>;
    async fn list_changes(
        &self, org_id: Uuid, status: Option<&str>, category: Option<&str>,
        priority: Option<&str>, assigned_to: Option<&Uuid>,
    ) -> AtlasResult<Vec<EngineeringChange>>;
    async fn update_change_status(
        &self, id: Uuid, status: &str,
        submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<EngineeringChange>;
    async fn update_change_with_resolution(
        &self, id: Uuid, status: &str,
        resolution_notes: Option<&str>,
        resolution_code: Option<&str>,
    ) -> AtlasResult<EngineeringChange>;
    async fn implement_change(
        &self, id: Uuid,
        actual_cost: Option<f64>,
        actual_hours: Option<f64>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<EngineeringChange>;
    async fn return_for_rework(
        &self, id: Uuid,
        comments: Option<&str>,
    ) -> AtlasResult<EngineeringChange>;
    async fn delete_change(&self, org_id: Uuid, change_number: &str) -> AtlasResult<()>;

    // Change Lines
    #[allow(clippy::too_many_arguments)]
    async fn create_change_line(
        &self, org_id: Uuid, change_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_name: Option<&str>,
        change_category: &str, field_name: Option<&str>,
        old_value: Option<&str>, new_value: Option<&str>,
        old_revision: Option<&str>, new_revision: Option<&str>,
        component_item_id: Option<Uuid>, component_item_number: Option<&str>,
        bom_quantity_old: Option<f64>, bom_quantity_new: Option<f64>,
        effectivity_date: Option<chrono::NaiveDate>,
        effectivity_end_date: Option<chrono::NaiveDate>,
        status: &str, completion_notes: Option<&str>,
        sequence_number: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeLine>;
    async fn get_change_line(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeLine>>;
    async fn list_change_lines(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeLine>>;
    async fn update_change_line_status(
        &self, id: Uuid, status: &str, completion_notes: Option<&str>,
    ) -> AtlasResult<EngineeringChangeLine>;
    async fn delete_change_line(&self, id: Uuid) -> AtlasResult<()>;

    // Affected Items
    #[allow(clippy::too_many_arguments)]
    async fn create_affected_item(
        &self, org_id: Uuid, change_id: Uuid, item_id: Uuid,
        item_number: &str, item_name: Option<&str>,
        impact_type: &str, impact_description: Option<&str>,
        current_revision: Option<&str>, new_revision: Option<&str>,
        disposition: Option<&str>,
        old_item_status: Option<&str>, new_item_status: Option<&str>,
        phase_in_date: Option<chrono::NaiveDate>,
        phase_out_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeAffectedItem>;
    async fn get_affected_item(&self, change_id: Uuid, item_id: Uuid) -> AtlasResult<Option<EngineeringChangeAffectedItem>>;
    async fn get_affected_item_by_id(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeAffectedItem>>;
    async fn list_affected_items(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeAffectedItem>>;
    async fn remove_affected_item(&self, id: Uuid) -> AtlasResult<()>;

    // Approvals
    #[allow(clippy::too_many_arguments)]
    async fn create_approval(
        &self, org_id: Uuid, change_id: Uuid, approval_level: i32,
        approver_id: Option<Uuid>, approver_name: Option<&str>,
        approver_role: Option<&str>, status: &str,
        action_date: Option<chrono::DateTime<chrono::Utc>>,
        comments: Option<&str>, delegated_from_id: Option<Uuid>,
        approval_conditions: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeApproval>;
    async fn list_approvals(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>>;
    async fn get_pending_approvals(&self, approver_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<EcmDashboard>;
}

/// PostgreSQL implementation of the Engineering Change Management repository
pub struct PostgresEngineeringChangeManagementRepository {
    pool: PgPool,
}

impl PostgresEngineeringChangeManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EngineeringChangeManagementRepository for PostgresEngineeringChangeManagementRepository {
    // ========================================================================
    // Change Types
    // ========================================================================

    async fn create_change_type(
        &self, org_id: Uuid, type_code: &str, name: &str, description: Option<&str>,
        category: &str, approval_required: bool, default_priority: &str,
        number_prefix: &str, description_template: Option<&str>,
        statuses: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeType> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.engineering_change_types
               (organization_id, type_code, name, description, category,
                approval_required, default_priority, number_prefix, description_template,
                statuses, metadata, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}'::jsonb, $11)
               RETURNING *"#,
        )
            .bind(org_id).bind(type_code).bind(name).bind(description)
            .bind(category).bind(approval_required).bind(default_priority)
            .bind(number_prefix).bind(description_template)
            .bind(&statuses).bind(created_by)
            .fetch_one(&self.pool).await?;

        Ok(row_to_change_type(&row))
    }

    async fn get_change_type(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_types WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_change_type))
    }

    async fn get_change_type_by_code(&self, org_id: Uuid, type_code: &str) -> AtlasResult<Option<EngineeringChangeType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_types WHERE organization_id = $1 AND type_code = $2",
        )
            .bind(org_id).bind(type_code)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_change_type))
    }

    async fn list_change_types(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<EngineeringChangeType>> {
        let rows = if let Some(cat) = category {
            sqlx::query(
                r#"SELECT * FROM _atlas.engineering_change_types
                   WHERE organization_id = $1 AND category = $2 AND status = 'active'
                   ORDER BY type_code"#,
            )
                .bind(org_id).bind(cat)
                .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                r#"SELECT * FROM _atlas.engineering_change_types
                   WHERE organization_id = $1 AND status = 'active'
                   ORDER BY type_code"#,
            )
                .bind(org_id)
                .fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_change_type).collect())
    }

    async fn delete_change_type(&self, org_id: Uuid, type_code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.engineering_change_types WHERE organization_id = $1 AND type_code = $2",
        )
            .bind(org_id).bind(type_code)
            .execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!(
                "Change type '{}' not found", type_code
            )));
        }
        Ok(())
    }

    // ========================================================================
    // Changes
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_change(
        &self, org_id: Uuid, change_number: &str, change_type_id: Option<Uuid>,
        category: &str, title: &str, description: Option<&str>,
        change_reason: Option<&str>, change_reason_description: Option<&str>,
        priority: &str, status: &str, revision: &str,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
        target_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        resolution_code: Option<&str>,
        resolution_notes: Option<&str>,
        parent_change_id: Option<Uuid>,
        superseded_by_id: Option<Uuid>,
        impact_analysis: serde_json::Value,
        estimated_cost: Option<f64>,
        actual_cost: Option<f64>,
        currency_code: &str,
        estimated_hours: Option<f64>,
        actual_hours: Option<f64>,
        regulatory_impact: Option<&str>,
        safety_impact: Option<&str>,
        validation_required: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChange> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.engineering_changes
               (organization_id, change_number, change_type_id, category, title, description,
                change_reason, change_reason_description, priority, status, revision,
                assigned_to, assigned_to_name, submitted_at, approved_at, implemented_at,
                target_date, effective_date, resolution_code, resolution_notes,
                parent_change_id, superseded_by_id, impact_analysis,
                estimated_cost, actual_cost, currency_code,
                estimated_hours, actual_hours, regulatory_impact, safety_impact,
                validation_required, metadata, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                       $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,'{}'::jsonb,$32)
               RETURNING *"#,
        )
            .bind(org_id).bind(change_number).bind(change_type_id).bind(category)
            .bind(title).bind(description).bind(change_reason).bind(change_reason_description)
            .bind(priority).bind(status).bind(revision)
            .bind(assigned_to).bind(assigned_to_name)
            .bind(submitted_at).bind(approved_at).bind(implemented_at)
            .bind(target_date).bind(effective_date)
            .bind(resolution_code).bind(resolution_notes)
            .bind(parent_change_id).bind(superseded_by_id)
            .bind(&impact_analysis)
            .bind(estimated_cost).bind(actual_cost)
            .bind(currency_code)
            .bind(estimated_hours).bind(actual_hours)
            .bind(regulatory_impact).bind(safety_impact)
            .bind(validation_required).bind(created_by)
            .fetch_one(&self.pool).await?;

        Ok(row_to_change(&row))
    }

    async fn get_change(&self, id: Uuid) -> AtlasResult<Option<EngineeringChange>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_changes WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_change))
    }

    async fn get_change_by_number(&self, org_id: Uuid, change_number: &str) -> AtlasResult<Option<EngineeringChange>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_changes WHERE organization_id = $1 AND change_number = $2",
        )
            .bind(org_id).bind(change_number)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_change))
    }

    async fn list_changes(
        &self, org_id: Uuid, status: Option<&str>, category: Option<&str>,
        priority: Option<&str>, assigned_to: Option<&Uuid>,
    ) -> AtlasResult<Vec<EngineeringChange>> {
        let mut query = String::from(
            "SELECT * FROM _atlas.engineering_changes WHERE organization_id = $1",
        );
        let mut param_idx = 2u32;

        if status.is_some() {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if category.is_some() {
            query.push_str(&format!(" AND category = ${}", param_idx));
            param_idx += 1;
        }
        if priority.is_some() {
            query.push_str(&format!(" AND priority = ${}", param_idx));
            param_idx += 1;
        }
        if assigned_to.is_some() {
            query.push_str(&format!(" AND assigned_to = ${}", param_idx));
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(c) = category { q = q.bind(c); }
        if let Some(p) = priority { q = q.bind(p); }
        if let Some(a) = assigned_to { q = q.bind(*a); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_change).collect())
    }

    async fn update_change_status(
        &self, id: Uuid, status: &str,
        submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<EngineeringChange> {
        let row = sqlx::query(
            r#"UPDATE _atlas.engineering_changes
               SET status = $2,
                   submitted_at = COALESCE($3, submitted_at),
                   approved_at = COALESCE($4, approved_at),
                   implemented_at = COALESCE($5, implemented_at),
                   updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
            .bind(id).bind(status)
            .bind(submitted_at).bind(approved_at).bind(implemented_at)
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Change {} not found", id)))?;

        Ok(row_to_change(&row))
    }

    async fn update_change_with_resolution(
        &self, id: Uuid, status: &str,
        resolution_notes: Option<&str>,
        resolution_code: Option<&str>,
    ) -> AtlasResult<EngineeringChange> {
        let row = sqlx::query(
            r#"UPDATE _atlas.engineering_changes
               SET status = $2, resolution_notes = $3, resolution_code = $4, updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
            .bind(id).bind(status).bind(resolution_notes).bind(resolution_code)
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Change {} not found", id)))?;

        Ok(row_to_change(&row))
    }

    async fn implement_change(
        &self, id: Uuid,
        actual_cost: Option<f64>,
        actual_hours: Option<f64>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<EngineeringChange> {
        let row = sqlx::query(
            r#"UPDATE _atlas.engineering_changes
               SET status = 'implemented',
                   actual_cost = COALESCE($2, actual_cost),
                   actual_hours = COALESCE($3, actual_hours),
                   implemented_at = COALESCE($4, now()),
                   resolution_code = 'implemented',
                   updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
            .bind(id).bind(actual_cost).bind(actual_hours).bind(implemented_at)
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Change {} not found", id)))?;

        Ok(row_to_change(&row))
    }

    async fn return_for_rework(
        &self, id: Uuid,
        comments: Option<&str>,
    ) -> AtlasResult<EngineeringChange> {
        let row = sqlx::query(
            r#"UPDATE _atlas.engineering_changes
               SET status = 'draft', resolution_notes = $2, submitted_at = NULL, updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
            .bind(id).bind(comments)
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Change {} not found", id)))?;

        Ok(row_to_change(&row))
    }

    async fn delete_change(&self, org_id: Uuid, change_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.engineering_changes WHERE organization_id = $1 AND change_number = $2",
        )
            .bind(org_id).bind(change_number)
            .execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!(
                "Change '{}' not found", change_number
            )));
        }
        Ok(())
    }

    // ========================================================================
    // Change Lines
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_change_line(
        &self, org_id: Uuid, change_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_name: Option<&str>,
        change_category: &str, field_name: Option<&str>,
        old_value: Option<&str>, new_value: Option<&str>,
        old_revision: Option<&str>, new_revision: Option<&str>,
        component_item_id: Option<Uuid>, component_item_number: Option<&str>,
        bom_quantity_old: Option<f64>, bom_quantity_new: Option<f64>,
        effectivity_date: Option<chrono::NaiveDate>,
        effectivity_end_date: Option<chrono::NaiveDate>,
        status: &str, completion_notes: Option<&str>,
        sequence_number: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.engineering_change_lines
               (organization_id, change_id, line_number, item_id, item_number, item_name,
                change_category, field_name, old_value, new_value, old_revision, new_revision,
                component_item_id, component_item_number,
                bom_quantity_old, bom_quantity_new, effectivity_date, effectivity_end_date,
                status, completion_notes, sequence_number, metadata, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,'{}'::jsonb,$22)
               RETURNING *"#,
        )
            .bind(org_id).bind(change_id).bind(line_number)
            .bind(item_id).bind(item_number).bind(item_name)
            .bind(change_category).bind(field_name)
            .bind(old_value).bind(new_value).bind(old_revision).bind(new_revision)
            .bind(component_item_id).bind(component_item_number)
            .bind(bom_quantity_old).bind(bom_quantity_new)
            .bind(effectivity_date).bind(effectivity_end_date)
            .bind(status).bind(completion_notes)
            .bind(sequence_number).bind(created_by)
            .fetch_one(&self.pool).await?;

        Ok(row_to_change_line(&row))
    }

    async fn get_change_line(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_lines WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_change_line))
    }

    async fn list_change_lines(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_lines WHERE change_id = $1 ORDER BY line_number",
        )
            .bind(change_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_change_line).collect())
    }

    async fn update_change_line_status(
        &self, id: Uuid, status: &str, completion_notes: Option<&str>,
    ) -> AtlasResult<EngineeringChangeLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.engineering_change_lines
               SET status = $2, completion_notes = $3, updated_at = now()
               WHERE id = $1
               RETURNING *"#,
        )
            .bind(id).bind(status).bind(completion_notes)
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Change line {} not found", id)))?;

        Ok(row_to_change_line(&row))
    }

    async fn delete_change_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.engineering_change_lines WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // ========================================================================
    // Affected Items
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_affected_item(
        &self, org_id: Uuid, change_id: Uuid, item_id: Uuid,
        item_number: &str, item_name: Option<&str>,
        impact_type: &str, impact_description: Option<&str>,
        current_revision: Option<&str>, new_revision: Option<&str>,
        disposition: Option<&str>,
        old_item_status: Option<&str>, new_item_status: Option<&str>,
        phase_in_date: Option<chrono::NaiveDate>,
        phase_out_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeAffectedItem> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.engineering_change_affected_items
               (organization_id, change_id, item_id, item_number, item_name,
                impact_type, impact_description, current_revision, new_revision,
                disposition, old_item_status, new_item_status,
                phase_in_date, phase_out_date, metadata, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'{}'::jsonb,$15)
               RETURNING *"#,
        )
            .bind(org_id).bind(change_id).bind(item_id).bind(item_number).bind(item_name)
            .bind(impact_type).bind(impact_description)
            .bind(current_revision).bind(new_revision).bind(disposition)
            .bind(old_item_status).bind(new_item_status)
            .bind(phase_in_date).bind(phase_out_date).bind(created_by)
            .fetch_one(&self.pool).await?;

        Ok(row_to_affected_item(&row))
    }

    async fn get_affected_item(&self, change_id: Uuid, item_id: Uuid) -> AtlasResult<Option<EngineeringChangeAffectedItem>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_affected_items WHERE change_id = $1 AND item_id = $2",
        )
            .bind(change_id).bind(item_id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_affected_item))
    }

    async fn get_affected_item_by_id(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeAffectedItem>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_affected_items WHERE id = $1",
        )
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_affected_item))
    }

    async fn list_affected_items(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeAffectedItem>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_affected_items WHERE change_id = $1 ORDER BY item_number",
        )
            .bind(change_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_affected_item).collect())
    }

    async fn remove_affected_item(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.engineering_change_affected_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // ========================================================================
    // Approvals
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_approval(
        &self, org_id: Uuid, change_id: Uuid, approval_level: i32,
        approver_id: Option<Uuid>, approver_name: Option<&str>,
        approver_role: Option<&str>, status: &str,
        action_date: Option<chrono::DateTime<chrono::Utc>>,
        comments: Option<&str>, delegated_from_id: Option<Uuid>,
        approval_conditions: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeApproval> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.engineering_change_approvals
               (organization_id, change_id, approval_level,
                approver_id, approver_name, approver_role, status,
                action_date, comments, delegated_from_id, approval_conditions, metadata, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'{}'::jsonb,$12)
               RETURNING *"#,
        )
            .bind(org_id).bind(change_id).bind(approval_level)
            .bind(approver_id).bind(approver_name).bind(approver_role)
            .bind(status).bind(action_date).bind(comments)
            .bind(delegated_from_id).bind(approval_conditions).bind(created_by)
            .fetch_one(&self.pool).await?;

        Ok(row_to_approval(&row))
    }

    async fn list_approvals(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_approvals WHERE change_id = $1 ORDER BY approval_level",
        )
            .bind(change_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_approval).collect())
    }

    async fn get_pending_approvals(&self, approver_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.engineering_change_approvals WHERE approver_id = $1 AND status = 'pending' ORDER BY created_at",
        )
            .bind(approver_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_approval).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<EcmDashboard> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.ecm_dashboard WHERE organization_id = $1",
        )
            .bind(org_id)
            .fetch_optional(&self.pool).await?;

        match row {
            Some(r) => Ok(EcmDashboard {
                total_changes: r.try_get::<Option<i32>, _>("total_changes").unwrap_or(None).unwrap_or(0),
                open_changes: r.try_get::<Option<i32>, _>("open_changes").unwrap_or(None).unwrap_or(0),
                pending_approval: r.try_get::<Option<i32>, _>("pending_approval").unwrap_or(None).unwrap_or(0),
                approved_changes: r.try_get::<Option<i32>, _>("approved_changes").unwrap_or(None).unwrap_or(0),
                implemented_changes: r.try_get::<Option<i32>, _>("implemented_changes").unwrap_or(None).unwrap_or(0),
                rejected_changes: r.try_get::<Option<i32>, _>("rejected_changes").unwrap_or(None).unwrap_or(0),
                ecr_count: r.try_get::<Option<i32>, _>("ecr_count").unwrap_or(None).unwrap_or(0),
                eco_count: r.try_get::<Option<i32>, _>("eco_count").unwrap_or(None).unwrap_or(0),
                ecn_count: r.try_get::<Option<i32>, _>("ecn_count").unwrap_or(None).unwrap_or(0),
                critical_open: r.try_get::<Option<i32>, _>("critical_open").unwrap_or(None).unwrap_or(0),
                high_open: r.try_get::<Option<i32>, _>("high_open").unwrap_or(None).unwrap_or(0),
                medium_open: r.try_get::<Option<i32>, _>("medium_open").unwrap_or(None).unwrap_or(0),
                low_open: r.try_get::<Option<i32>, _>("low_open").unwrap_or(None).unwrap_or(0),
                avg_days_to_implement: r.try_get::<Option<f64>, _>("avg_days_to_implement").unwrap_or(None).unwrap_or(0.0),
                avg_days_to_approve: r.try_get::<Option<f64>, _>("avg_days_to_approve").unwrap_or(None).unwrap_or(0.0),
                total_items_affected: r.try_get::<Option<i32>, _>("total_items_affected").unwrap_or(None).unwrap_or(0),
                total_estimated_cost: r.try_get::<Option<f64>, _>("total_estimated_cost").unwrap_or(None).unwrap_or(0.0),
                total_actual_cost: r.try_get::<Option<f64>, _>("total_actual_cost").unwrap_or(None).unwrap_or(0.0),
                changes_by_reason: r.try_get::<serde_json::Value, _>("changes_by_reason").unwrap_or(serde_json::json!({})),
                changes_by_status: r.try_get::<serde_json::Value, _>("changes_by_status").unwrap_or(serde_json::json!({})),
                changes_trend: r.try_get::<serde_json::Value, _>("changes_trend").unwrap_or(serde_json::json!([])),
            }),
            None => Ok(EcmDashboard {
                total_changes: 0, open_changes: 0, pending_approval: 0,
                approved_changes: 0, implemented_changes: 0, rejected_changes: 0,
                ecr_count: 0, eco_count: 0, ecn_count: 0,
                critical_open: 0, high_open: 0, medium_open: 0, low_open: 0,
                avg_days_to_implement: 0.0, avg_days_to_approve: 0.0,
                total_items_affected: 0, total_estimated_cost: 0.0, total_actual_cost: 0.0,
                changes_by_reason: serde_json::json!({}),
                changes_by_status: serde_json::json!({}),
                changes_trend: serde_json::json!([]),
            }),
        }
    }
}

// ============================================================================
// Row mapping helpers
// ============================================================================

fn row_to_change_type(row: &sqlx::postgres::PgRow) -> EngineeringChangeType {
    EngineeringChangeType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        type_code: row.get("type_code"),
        name: row.get("name"),
        description: row.get("description"),
        category: row.get("category"),
        approval_required: row.get("approval_required"),
        default_priority: row.get("default_priority"),
        number_prefix: row.get("number_prefix"),
        description_template: row.get("description_template"),
        status: row.get("status"),
        statuses: row.get("statuses"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_change(row: &sqlx::postgres::PgRow) -> EngineeringChange {
    EngineeringChange {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        change_number: row.get("change_number"),
        change_type_id: row.get("change_type_id"),
        category: row.get("category"),
        title: row.get("title"),
        description: row.get("description"),
        change_reason: row.get("change_reason"),
        change_reason_description: row.get("change_reason_description"),
        priority: row.get("priority"),
        status: row.get("status"),
        revision: row.get("revision"),
        assigned_to: row.get("assigned_to"),
        assigned_to_name: row.get("assigned_to_name"),
        submitted_at: row.get("submitted_at"),
        approved_at: row.get("approved_at"),
        implemented_at: row.get("implemented_at"),
        target_date: row.get("target_date"),
        effective_date: row.get("effective_date"),
        resolution_code: row.get("resolution_code"),
        resolution_notes: row.get("resolution_notes"),
        parent_change_id: row.get("parent_change_id"),
        superseded_by_id: row.get("superseded_by_id"),
        impact_analysis: row.get("impact_analysis"),
        estimated_cost: row.get("estimated_cost"),
        actual_cost: row.get("actual_cost"),
        currency_code: row.get("currency_code"),
        estimated_hours: row.get("estimated_hours"),
        actual_hours: row.get("actual_hours"),
        regulatory_impact: row.get("regulatory_impact"),
        safety_impact: row.get("safety_impact"),
        validation_required: row.get("validation_required"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_change_line(row: &sqlx::postgres::PgRow) -> EngineeringChangeLine {
    EngineeringChangeLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        change_id: row.get("change_id"),
        line_number: row.get("line_number"),
        item_id: row.get("item_id"),
        item_number: row.get("item_number"),
        item_name: row.get("item_name"),
        change_category: row.get("change_category"),
        field_name: row.get("field_name"),
        old_value: row.get("old_value"),
        new_value: row.get("new_value"),
        old_revision: row.get("old_revision"),
        new_revision: row.get("new_revision"),
        component_item_id: row.get("component_item_id"),
        component_item_number: row.get("component_item_number"),
        bom_quantity_old: row.get("bom_quantity_old"),
        bom_quantity_new: row.get("bom_quantity_new"),
        effectivity_date: row.get("effectivity_date"),
        effectivity_end_date: row.get("effectivity_end_date"),
        status: row.get("status"),
        completion_notes: row.get("completion_notes"),
        sequence_number: row.get("sequence_number"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_affected_item(row: &sqlx::postgres::PgRow) -> EngineeringChangeAffectedItem {
    EngineeringChangeAffectedItem {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        change_id: row.get("change_id"),
        item_id: row.get("item_id"),
        item_number: row.get("item_number"),
        item_name: row.get("item_name"),
        impact_type: row.get("impact_type"),
        impact_description: row.get("impact_description"),
        current_revision: row.get("current_revision"),
        new_revision: row.get("new_revision"),
        disposition: row.get("disposition"),
        old_item_status: row.get("old_item_status"),
        new_item_status: row.get("new_item_status"),
        phase_in_date: row.get("phase_in_date"),
        phase_out_date: row.get("phase_out_date"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_approval(row: &sqlx::postgres::PgRow) -> EngineeringChangeApproval {
    EngineeringChangeApproval {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        change_id: row.get("change_id"),
        approval_level: row.get("approval_level"),
        approver_id: row.get("approver_id"),
        approver_name: row.get("approver_name"),
        approver_role: row.get("approver_role"),
        status: row.get("status"),
        action_date: row.get("action_date"),
        comments: row.get("comments"),
        delegated_from_id: row.get("delegated_from_id"),
        approval_conditions: row.get("approval_conditions"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
