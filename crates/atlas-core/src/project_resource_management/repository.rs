//! Project Resource Management Repository
//!
//! PostgreSQL storage for resource profiles, requests, assignments,
//! utilization entries, and dashboard analytics.

use atlas_shared::{
    ResourceProfile, ResourceRequest, ResourceAssignment, UtilizationEntry,
    ResourceDashboard, AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for project resource management data storage
#[async_trait]
pub trait ProjectResourceManagementRepository: Send + Sync {
    // Profiles
    async fn create_profile(
        &self, org_id: Uuid, resource_number: &str, name: &str, email: &str,
        resource_type: &str, department: &str, job_title: &str,
        skills: &str, certifications: &str,
        availability_status: &str, available_hours_per_week: f64,
        cost_rate: f64, cost_rate_currency: &str,
        bill_rate: f64, bill_rate_currency: &str,
        location: &str, manager_id: Option<Uuid>, manager_name: &str,
        hire_date: Option<chrono::NaiveDate>, notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceProfile>;
    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<ResourceProfile>>;
    async fn get_profile_by_number(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<Option<ResourceProfile>>;
    async fn list_profiles(&self, org_id: Uuid, availability_status: Option<&str>, resource_type: Option<&str>, department: Option<&str>) -> AtlasResult<Vec<ResourceProfile>>;
    async fn update_availability_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceProfile>;
    async fn delete_profile(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<()>;

    // Requests
    async fn create_request(
        &self, org_id: Uuid, request_number: &str, project_id: Option<Uuid>,
        project_name: &str, project_number: &str,
        requested_role: &str, required_skills: &str, priority: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        hours_per_week: f64, total_planned_hours: f64,
        max_cost_rate: Option<f64>, currency_code: &str,
        resource_type_preference: &str, location_requirement: &str,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceRequest>;
    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ResourceRequest>>;
    async fn get_request_by_number(&self, org_id: Uuid, request_number: &str) -> AtlasResult<Option<ResourceRequest>>;
    async fn list_requests(&self, org_id: Uuid, status: Option<&str>, priority: Option<&str>, project_id: Option<Uuid>) -> AtlasResult<Vec<ResourceRequest>>;
    async fn update_request_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceRequest>;
    async fn fulfill_request(&self, id: Uuid, fulfilled_by: Uuid) -> AtlasResult<ResourceRequest>;
    async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()>;

    // Assignments
    async fn create_assignment(
        &self, org_id: Uuid, assignment_number: &str, resource_id: Uuid,
        resource_name: &str, resource_email: &str,
        project_id: Option<Uuid>, project_name: &str, project_number: &str,
        request_id: Option<Uuid>, role: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        planned_hours: f64, cost_rate: f64, bill_rate: f64, currency_code: &str,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceAssignment>;
    async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<ResourceAssignment>>;
    async fn get_assignment_by_number(&self, org_id: Uuid, assignment_number: &str) -> AtlasResult<Option<ResourceAssignment>>;
    async fn list_assignments(&self, org_id: Uuid, status: Option<&str>, resource_id: Option<Uuid>, project_id: Option<Uuid>) -> AtlasResult<Vec<ResourceAssignment>>;
    async fn update_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceAssignment>;
    async fn update_assignment_hours(&self, id: Uuid, actual_hours: f64, remaining_hours: f64, utilization_percentage: f64) -> AtlasResult<()>;
    async fn delete_assignment(&self, org_id: Uuid, assignment_number: &str) -> AtlasResult<()>;

    // Utilization
    async fn create_utilization_entry(
        &self, org_id: Uuid, assignment_id: Uuid, resource_id: Uuid,
        entry_date: chrono::NaiveDate, hours_worked: f64,
        description: &str, billable: bool,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<UtilizationEntry>;
    async fn get_utilization_entry(&self, id: Uuid) -> AtlasResult<Option<UtilizationEntry>>;
    async fn list_utilization_entries(&self, org_id: Uuid, assignment_id: Option<Uuid>, resource_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<UtilizationEntry>>;
    async fn update_utilization_status(&self, id: Uuid, status: &str) -> AtlasResult<UtilizationEntry>;
    async fn approve_utilization_entry(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<UtilizationEntry>;
    async fn delete_utilization_entry(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ResourceDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresProjectResourceManagementRepository {
    pool: PgPool,
}

impl PostgresProjectResourceManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper numeric decoding
fn get_numeric(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    if let Ok(v) = row.try_get::<f64, _>(column) { return v; }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() { return n; }
        if let Some(s) = v.as_str() { if let Ok(n) = s.parse::<f64>() { return n; } }
    }
    if let Ok(s) = row.try_get::<String, _>(column) { return s.parse::<f64>().unwrap_or(0.0); }
    0.0
}

fn get_optional_numeric(row: &sqlx::postgres::PgRow, column: &str) -> Option<f64> {
    if let Ok(v) = row.try_get::<f64, _>(column) { return Some(v); }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() { return Some(n); }
        if let Some(s) = v.as_str() { return s.parse::<f64>().ok(); }
    }
    if let Ok(s) = row.try_get::<String, _>(column) { return s.parse::<f64>().ok(); }
    None
}

fn row_to_profile(row: &sqlx::postgres::PgRow) -> ResourceProfile {
    ResourceProfile {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        resource_number: row.try_get("resource_number").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        resource_type: row.try_get("resource_type").unwrap_or_default(),
        department: row.try_get("department").unwrap_or_default(),
        job_title: row.try_get("job_title").unwrap_or_default(),
        skills: row.try_get("skills").unwrap_or_default(),
        certifications: row.try_get("certifications").unwrap_or_default(),
        availability_status: row.try_get("availability_status").unwrap_or_default(),
        available_hours_per_week: get_numeric(row, "available_hours_per_week"),
        cost_rate: get_numeric(row, "cost_rate"),
        cost_rate_currency: row.try_get("cost_rate_currency").unwrap_or_default(),
        bill_rate: get_numeric(row, "bill_rate"),
        bill_rate_currency: row.try_get("bill_rate_currency").unwrap_or_default(),
        location: row.try_get("location").unwrap_or_default(),
        manager_id: row.try_get("manager_id").unwrap_or_default(),
        manager_name: row.try_get("manager_name").unwrap_or_default(),
        hire_date: row.try_get("hire_date").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_request(row: &sqlx::postgres::PgRow) -> ResourceRequest {
    ResourceRequest {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        request_number: row.try_get("request_number").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        project_name: row.try_get("project_name").unwrap_or_default(),
        project_number: row.try_get("project_number").unwrap_or_default(),
        requested_role: row.try_get("requested_role").unwrap_or_default(),
        required_skills: row.try_get("required_skills").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: row.try_get("end_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        hours_per_week: get_numeric(row, "hours_per_week"),
        total_planned_hours: get_numeric(row, "total_planned_hours"),
        max_cost_rate: get_optional_numeric(row, "max_cost_rate"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        resource_type_preference: row.try_get("resource_type_preference").unwrap_or_default(),
        location_requirement: row.try_get("location_requirement").unwrap_or_default(),
        fulfilled_by: row.try_get("fulfilled_by").unwrap_or_default(),
        fulfilled_at: row.try_get("fulfilled_at").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_assignment(row: &sqlx::postgres::PgRow) -> ResourceAssignment {
    ResourceAssignment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        assignment_number: row.try_get("assignment_number").unwrap_or_default(),
        resource_id: row.try_get("resource_id").unwrap_or_default(),
        resource_name: row.try_get("resource_name").unwrap_or_default(),
        resource_email: row.try_get("resource_email").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        project_name: row.try_get("project_name").unwrap_or_default(),
        project_number: row.try_get("project_number").unwrap_or_default(),
        request_id: row.try_get("request_id").unwrap_or_default(),
        role: row.try_get("role").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        start_date: row.try_get("start_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: row.try_get("end_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        planned_hours: get_numeric(row, "planned_hours"),
        actual_hours: get_numeric(row, "actual_hours"),
        remaining_hours: get_numeric(row, "remaining_hours"),
        utilization_percentage: get_numeric(row, "utilization_percentage"),
        cost_rate: get_numeric(row, "cost_rate"),
        bill_rate: get_numeric(row, "bill_rate"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_utilization(row: &sqlx::postgres::PgRow) -> UtilizationEntry {
    UtilizationEntry {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        assignment_id: row.try_get("assignment_id").unwrap_or_default(),
        resource_id: row.try_get("resource_id").unwrap_or_default(),
        entry_date: row.try_get("entry_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        hours_worked: get_numeric(row, "hours_worked"),
        description: row.try_get("description").unwrap_or_default(),
        billable: row.try_get("billable").unwrap_or(true),
        status: row.try_get("status").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        approved_at: row.try_get("approved_at").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl ProjectResourceManagementRepository for PostgresProjectResourceManagementRepository {
    // ========================================================================
    // Profiles
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_profile(
        &self, org_id: Uuid, resource_number: &str, name: &str, email: &str,
        resource_type: &str, department: &str, job_title: &str,
        skills: &str, certifications: &str,
        availability_status: &str, available_hours_per_week: f64,
        cost_rate: f64, cost_rate_currency: &str,
        bill_rate: f64, bill_rate_currency: &str,
        location: &str, manager_id: Option<Uuid>, manager_name: &str,
        hire_date: Option<chrono::NaiveDate>, notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceProfile> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.resource_profiles
                (organization_id, resource_number, name, email,
                 resource_type, department, job_title,
                 skills, certifications, availability_status,
                 available_hours_per_week,
                 cost_rate, cost_rate_currency, bill_rate, bill_rate_currency,
                 location, manager_id, manager_name, hire_date,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, '{}'::jsonb, $21)
            RETURNING *"#,
        )
        .bind(org_id).bind(resource_number).bind(name).bind(email)
        .bind(resource_type).bind(department).bind(job_title)
        .bind(skills).bind(certifications).bind(availability_status)
        .bind(available_hours_per_week)
        .bind(cost_rate).bind(cost_rate_currency).bind(bill_rate).bind(bill_rate_currency)
        .bind(location).bind(manager_id).bind(manager_name).bind(hire_date)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_profile(&row))
    }

    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<ResourceProfile>> {
        let row = sqlx::query("SELECT * FROM _atlas.resource_profiles WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_profile))
    }

    async fn get_profile_by_number(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<Option<ResourceProfile>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.resource_profiles WHERE organization_id = $1 AND resource_number = $2"
        ).bind(org_id).bind(resource_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_profile))
    }

    async fn list_profiles(&self, org_id: Uuid, availability_status: Option<&str>, resource_type: Option<&str>, department: Option<&str>) -> AtlasResult<Vec<ResourceProfile>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.resource_profiles
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR availability_status = $2)
                 AND ($3::text IS NULL OR resource_type = $3)
                 AND ($4::text IS NULL OR department = $4)
               ORDER BY name"#,
        )
        .bind(org_id).bind(availability_status).bind(resource_type).bind(department)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_profile).collect())
    }

    async fn update_availability_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceProfile> {
        let row = sqlx::query(
            "UPDATE _atlas.resource_profiles SET availability_status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Resource profile {} not found", id)))?;
        Ok(row_to_profile(&row))
    }

    async fn delete_profile(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.resource_profiles WHERE organization_id = $1 AND resource_number = $2"
        ).bind(org_id).bind(resource_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Resource profile '{}' not found", resource_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Requests
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_request(
        &self, org_id: Uuid, request_number: &str, project_id: Option<Uuid>,
        project_name: &str, project_number: &str,
        requested_role: &str, required_skills: &str, priority: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        hours_per_week: f64, total_planned_hours: f64,
        max_cost_rate: Option<f64>, currency_code: &str,
        resource_type_preference: &str, location_requirement: &str,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceRequest> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.resource_requests
                (organization_id, request_number, project_id,
                 project_name, project_number,
                 requested_role, required_skills, priority, status,
                 start_date, end_date, hours_per_week, total_planned_hours,
                 max_cost_rate, currency_code,
                 resource_type_preference, location_requirement,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft',
                    $9, $10, $11, $12, $13, $14, $15, $16,
                    $17, '{}'::jsonb, $18)
            RETURNING *"#,
        )
        .bind(org_id).bind(request_number).bind(project_id)
        .bind(project_name).bind(project_number)
        .bind(requested_role).bind(required_skills).bind(priority)
        .bind(start_date).bind(end_date).bind(hours_per_week).bind(total_planned_hours)
        .bind(max_cost_rate).bind(currency_code)
        .bind(resource_type_preference).bind(location_requirement)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_request(&row))
    }

    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ResourceRequest>> {
        let row = sqlx::query("SELECT * FROM _atlas.resource_requests WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_request))
    }

    async fn get_request_by_number(&self, org_id: Uuid, request_number: &str) -> AtlasResult<Option<ResourceRequest>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.resource_requests WHERE organization_id = $1 AND request_number = $2"
        ).bind(org_id).bind(request_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_request))
    }

    async fn list_requests(&self, org_id: Uuid, status: Option<&str>, priority: Option<&str>, project_id: Option<Uuid>) -> AtlasResult<Vec<ResourceRequest>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.resource_requests
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR priority = $3)
                 AND ($4::uuid IS NULL OR project_id = $4)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(priority).bind(project_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_request).collect())
    }

    async fn update_request_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceRequest> {
        let row = sqlx::query(
            "UPDATE _atlas.resource_requests SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Resource request {} not found", id)))?;
        Ok(row_to_request(&row))
    }

    async fn fulfill_request(&self, id: Uuid, fulfilled_by: Uuid) -> AtlasResult<ResourceRequest> {
        let row = sqlx::query(
            r#"UPDATE _atlas.resource_requests
               SET status = 'fulfilled', fulfilled_by = $2, fulfilled_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(fulfilled_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Resource request {} not found", id)))?;
        Ok(row_to_request(&row))
    }

    async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.resource_requests WHERE organization_id = $1 AND request_number = $2 AND status = 'draft'"
        ).bind(org_id).bind(request_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Draft request '{}' not found", request_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Assignments
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_assignment(
        &self, org_id: Uuid, assignment_number: &str, resource_id: Uuid,
        resource_name: &str, resource_email: &str,
        project_id: Option<Uuid>, project_name: &str, project_number: &str,
        request_id: Option<Uuid>, role: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        planned_hours: f64, cost_rate: f64, bill_rate: f64, currency_code: &str,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceAssignment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.resource_assignments
                (organization_id, assignment_number, resource_id,
                 resource_name, resource_email,
                 project_id, project_name, project_number,
                 request_id, role, status,
                 start_date, end_date, planned_hours,
                 actual_hours, remaining_hours, utilization_percentage,
                 cost_rate, bill_rate, currency_code,
                 notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'planned',
                    $11, $12, $13, 0, $13, 0,
                    $14, $15, $16, $17, '{}'::jsonb, $18)
            RETURNING *"#,
        )
        .bind(org_id).bind(assignment_number).bind(resource_id)
        .bind(resource_name).bind(resource_email)
        .bind(project_id).bind(project_name).bind(project_number)
        .bind(request_id).bind(role)
        .bind(start_date).bind(end_date).bind(planned_hours)
        .bind(cost_rate).bind(bill_rate).bind(currency_code)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_assignment(&row))
    }

    async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<ResourceAssignment>> {
        let row = sqlx::query("SELECT * FROM _atlas.resource_assignments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_assignment))
    }

    async fn get_assignment_by_number(&self, org_id: Uuid, assignment_number: &str) -> AtlasResult<Option<ResourceAssignment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.resource_assignments WHERE organization_id = $1 AND assignment_number = $2"
        ).bind(org_id).bind(assignment_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_assignment))
    }

    async fn list_assignments(&self, org_id: Uuid, status: Option<&str>, resource_id: Option<Uuid>, project_id: Option<Uuid>) -> AtlasResult<Vec<ResourceAssignment>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.resource_assignments
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::uuid IS NULL OR resource_id = $3)
                 AND ($4::uuid IS NULL OR project_id = $4)
               ORDER BY start_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(resource_id).bind(project_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_assignment).collect())
    }

    async fn update_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<ResourceAssignment> {
        let row = sqlx::query(
            "UPDATE _atlas.resource_assignments SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Resource assignment {} not found", id)))?;
        Ok(row_to_assignment(&row))
    }

    async fn update_assignment_hours(&self, id: Uuid, actual_hours: f64, remaining_hours: f64, utilization_percentage: f64) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.resource_assignments
               SET actual_hours = $2, remaining_hours = $3, utilization_percentage = $4, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id).bind(actual_hours).bind(remaining_hours).bind(utilization_percentage)
        .execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_assignment(&self, org_id: Uuid, assignment_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.resource_assignments WHERE organization_id = $1 AND assignment_number = $2 AND status IN ('planned', 'cancelled')"
        ).bind(org_id).bind(assignment_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!(
                "Planned/cancelled assignment '{}' not found", assignment_number
            )));
        }
        Ok(())
    }

    // ========================================================================
    // Utilization Entries
    // ========================================================================

    async fn create_utilization_entry(
        &self, org_id: Uuid, assignment_id: Uuid, resource_id: Uuid,
        entry_date: chrono::NaiveDate, hours_worked: f64,
        description: &str, billable: bool,
        notes: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<UtilizationEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.utilization_entries
                (organization_id, assignment_id, resource_id,
                 entry_date, hours_worked, description, billable,
                 status, notes, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'submitted', $8, '{}'::jsonb, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(assignment_id).bind(resource_id)
        .bind(entry_date).bind(hours_worked).bind(description).bind(billable)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_utilization(&row))
    }

    async fn get_utilization_entry(&self, id: Uuid) -> AtlasResult<Option<UtilizationEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.utilization_entries WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_utilization))
    }

    async fn list_utilization_entries(&self, org_id: Uuid, assignment_id: Option<Uuid>, resource_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<UtilizationEntry>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.utilization_entries
               WHERE organization_id = $1
                 AND ($2::uuid IS NULL OR assignment_id = $2)
                 AND ($3::uuid IS NULL OR resource_id = $3)
                 AND ($4::text IS NULL OR status = $4)
               ORDER BY entry_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(assignment_id).bind(resource_id).bind(status)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_utilization).collect())
    }

    async fn update_utilization_status(&self, id: Uuid, status: &str) -> AtlasResult<UtilizationEntry> {
        let row = sqlx::query(
            "UPDATE _atlas.utilization_entries SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Utilization entry {} not found", id)))?;
        Ok(row_to_utilization(&row))
    }

    async fn approve_utilization_entry(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<UtilizationEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.utilization_entries
               SET status = 'approved', approved_by = $2, approved_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Utilization entry {} not found", id)))?;
        Ok(row_to_utilization(&row))
    }

    async fn delete_utilization_entry(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.utilization_entries WHERE id = $1 AND status = 'submitted'"
        ).bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(
                "Submitted utilization entry not found or already processed".to_string()
            ));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ResourceDashboard> {
        let profiles = sqlx::query(
            "SELECT resource_type, availability_status, department FROM _atlas.resource_profiles WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_resources = profiles.len() as i64;
        let available_resources = profiles.iter()
            .filter(|r| r.try_get::<String, _>("availability_status").unwrap_or_default() == "available")
            .count() as i64;
        let fully_allocated_resources = profiles.iter()
            .filter(|r| r.try_get::<String, _>("availability_status").unwrap_or_default() == "fully_allocated")
            .count() as i64;

        let mut by_type = serde_json::Map::new();
        let mut by_dept = serde_json::Map::new();
        for row in &profiles {
            let rt = row.try_get::<String, _>("resource_type").unwrap_or_default();
            let count = by_type.entry(rt).or_insert(serde_json::Value::from(0));
            *count = serde_json::Value::from(count.as_i64().unwrap_or(0) + 1);

            let dept = row.try_get::<String, _>("department").unwrap_or_default();
            if !dept.is_empty() {
                let count = by_dept.entry(dept).or_insert(serde_json::Value::from(0));
                *count = serde_json::Value::from(count.as_i64().unwrap_or(0) + 1);
            }
        }

        let open_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.resource_requests WHERE organization_id = $1 AND status IN ('draft', 'submitted')"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let active_assignments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.resource_assignments WHERE organization_id = $1 AND status IN ('planned', 'active')"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let hour_stats = sqlx::query(
            r#"SELECT COALESCE(SUM(planned_hours), 0) as total_planned,
                      COALESCE(SUM(actual_hours), 0) as total_actual,
                      COALESCE(AVG(utilization_percentage), 0) as avg_util
               FROM _atlas.resource_assignments
               WHERE organization_id = $1 AND status IN ('planned', 'active')"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap();

        let total_planned_hours: f64 = get_numeric(&hour_stats, "total_planned");
        let total_actual_hours: f64 = get_numeric(&hour_stats, "total_actual");
        let average_utilization: f64 = get_numeric(&hour_stats, "avg_util");

        Ok(ResourceDashboard {
            organization_id: org_id,
            total_resources,
            available_resources,
            fully_allocated_resources,
            open_requests,
            active_assignments,
            average_utilization,
            total_planned_hours,
            total_actual_hours,
            resources_by_type: serde_json::Value::Object(by_type),
            resources_by_department: serde_json::Value::Object(by_dept),
            top_resources_by_utilization: serde_json::json!([]),
            recent_assignments: serde_json::json!([]),
        })
    }
}
