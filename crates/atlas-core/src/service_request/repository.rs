//! Service Request Repository
//!
//! PostgreSQL storage for service categories, requests, updates,
//! and assignments.

use atlas_shared::{
    ServiceCategory, ServiceRequest, ServiceRequestUpdate,
    ServiceRequestAssignment, ServiceRequestDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for service request data storage
#[async_trait]
pub trait ServiceRequestRepository: Send + Sync {
    // Categories
    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        default_priority: Option<&str>,
        default_sla_hours: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceCategory>;

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ServiceCategory>>;
    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<ServiceCategory>>;
    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ServiceCategory>>;
    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Requests
    async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        title: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        category_name: Option<&str>,
        priority: &str,
        status: &str,
        request_type: &str,
        channel: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        serial_number: Option<&str>,
        sla_due_date: Option<chrono::NaiveDate>,
        parent_request_id: Option<Uuid>,
        related_object_type: Option<&str>,
        related_object_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceRequest>;

    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ServiceRequest>>;
    async fn get_request_by_number(&self, org_id: Uuid, request_number: &str) -> AtlasResult<Option<ServiceRequest>>;
    async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        priority: Option<&str>,
        customer_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ServiceRequest>>;
    async fn update_request_status(
        &self,
        id: Uuid,
        status: &str,
        resolved_at: Option<chrono::DateTime<chrono::Utc>>,
        closed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ServiceRequest>;
    async fn update_request_resolution(
        &self,
        id: Uuid,
        resolution: &str,
        resolution_code: &str,
        resolved_at: chrono::DateTime<chrono::Utc>,
    ) -> AtlasResult<ServiceRequest>;
    async fn update_request_assignment(
        &self,
        id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
    ) -> AtlasResult<()>;

    // Updates
    async fn create_update(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        update_type: &str,
        author_id: Option<Uuid>,
        author_name: Option<&str>,
        subject: Option<&str>,
        body: &str,
        is_internal: bool,
    ) -> AtlasResult<ServiceRequestUpdate>;
    async fn list_updates(&self, request_id: Uuid, include_internal: bool) -> AtlasResult<Vec<ServiceRequestUpdate>>;

    // Assignments
    async fn create_assignment(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        assigned_by: Option<Uuid>,
        assigned_by_name: Option<&str>,
        assignment_type: &str,
    ) -> AtlasResult<ServiceRequestAssignment>;
    async fn list_assignments(&self, request_id: Uuid) -> AtlasResult<Vec<ServiceRequestAssignment>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ServiceRequestDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresServiceRequestRepository {
    pool: PgPool,
}

impl PostgresServiceRequestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_category(&self, row: &sqlx::postgres::PgRow) -> ServiceCategory {
        ServiceCategory {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            parent_category_id: row.get("parent_category_id"),
            default_priority: row.get("default_priority"),
            default_sla_hours: row.get("default_sla_hours"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_request(&self, row: &sqlx::postgres::PgRow) -> ServiceRequest {
        ServiceRequest {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            request_number: row.get("request_number"),
            title: row.get("title"),
            description: row.get("description"),
            category_id: row.get("category_id"),
            category_name: row.get("category_name"),
            priority: row.get("priority"),
            status: row.get("status"),
            request_type: row.get("request_type"),
            channel: row.get("channel"),
            customer_id: row.get("customer_id"),
            customer_name: row.get("customer_name"),
            contact_id: row.get("contact_id"),
            contact_name: row.get("contact_name"),
            assigned_to: row.get("assigned_to"),
            assigned_to_name: row.get("assigned_to_name"),
            assigned_group: row.get("assigned_group"),
            product_id: row.get("product_id"),
            product_name: row.get("product_name"),
            serial_number: row.get("serial_number"),
            resolution: row.get("resolution"),
            resolution_code: row.get("resolution_code"),
            sla_due_date: row.get("sla_due_date"),
            sla_breached: row.get("sla_breached"),
            parent_request_id: row.get("parent_request_id"),
            related_object_type: row.get("related_object_type"),
            related_object_id: row.get("related_object_id"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            resolved_at: row.get("resolved_at"),
            closed_at: row.get("closed_at"),
        }
    }

    fn row_to_update(&self, row: &sqlx::postgres::PgRow) -> ServiceRequestUpdate {
        ServiceRequestUpdate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            request_id: row.get("request_id"),
            update_type: row.get("update_type"),
            author_id: row.get("author_id"),
            author_name: row.get("author_name"),
            subject: row.get("subject"),
            body: row.get("body"),
            is_internal: row.get("is_internal"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_assignment(&self, row: &sqlx::postgres::PgRow) -> ServiceRequestAssignment {
        ServiceRequestAssignment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            request_id: row.get("request_id"),
            assigned_to: row.get("assigned_to"),
            assigned_to_name: row.get("assigned_to_name"),
            assigned_group: row.get("assigned_group"),
            assigned_by: row.get("assigned_by"),
            assigned_by_name: row.get("assigned_by_name"),
            assignment_type: row.get("assignment_type"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl ServiceRequestRepository for PostgresServiceRequestRepository {
    // ========================================================================
    // Categories
    // ========================================================================

    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        default_priority: Option<&str>,
        default_sla_hours: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceCategory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.service_categories
                (organization_id, code, name, description, parent_category_id,
                 default_priority, default_sla_hours, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, parent_category_id = $5,
                    default_priority = $6, default_sla_hours = $7, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_category_id).bind(default_priority).bind(default_sla_hours)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_category(&row))
    }

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ServiceCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.service_categories WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<ServiceCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.service_categories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ServiceCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.service_categories WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_category(r)).collect())
    }

    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.service_categories SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Service Requests
    // ========================================================================

    async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        title: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        category_name: Option<&str>,
        priority: &str,
        status: &str,
        request_type: &str,
        channel: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        serial_number: Option<&str>,
        sla_due_date: Option<chrono::NaiveDate>,
        parent_request_id: Option<Uuid>,
        related_object_type: Option<&str>,
        related_object_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceRequest> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.service_requests
                (organization_id, request_number, title, description,
                 category_id, category_name, priority, status, request_type, channel,
                 customer_id, customer_name, contact_id, contact_name,
                 assigned_to, assigned_to_name, assigned_group,
                 product_id, product_name, serial_number,
                 sla_due_date, parent_request_id,
                 related_object_type, related_object_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(request_number).bind(title).bind(description)
        .bind(category_id).bind(category_name).bind(priority).bind(status)
        .bind(request_type).bind(channel)
        .bind(customer_id).bind(customer_name).bind(contact_id).bind(contact_name)
        .bind(assigned_to).bind(assigned_to_name).bind(assigned_group)
        .bind(product_id).bind(product_name).bind(serial_number)
        .bind(sla_due_date).bind(parent_request_id)
        .bind(related_object_type).bind(related_object_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_request(&row))
    }

    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ServiceRequest>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.service_requests WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_request(&r)))
    }

    async fn get_request_by_number(&self, org_id: Uuid, request_number: &str) -> AtlasResult<Option<ServiceRequest>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.service_requests WHERE organization_id = $1 AND request_number = $2"
        )
        .bind(org_id).bind(request_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_request(&r)))
    }

    async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        priority: Option<&str>,
        customer_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ServiceRequest>> {
        let mut query_str = String::from(
            "SELECT * FROM _atlas.service_requests WHERE organization_id = $1"
        );
        let mut bind_idx = 2;

        let status_val;
        let priority_val;

        if let Some(s) = status {
            query_str.push_str(&format!(" AND status = ${}", bind_idx));
            status_val = s.to_string();
            bind_idx += 1;
        } else {
            status_val = String::new();
        }
        if let Some(p) = priority {
            query_str.push_str(&format!(" AND priority = ${}", bind_idx));
            priority_val = p.to_string();
            bind_idx += 1;
        } else {
            priority_val = String::new();
        }
        if customer_id.is_some() {
            query_str.push_str(&format!(" AND customer_id = ${}", bind_idx));
            bind_idx += 1;
        }
        if assigned_to.is_some() {
            query_str.push_str(&format!(" AND (assigned_to = ${} OR assigned_group IS NOT NULL)", bind_idx));
            bind_idx += 1;
        }
        if category_id.is_some() {
            query_str.push_str(&format!(" AND category_id = ${}", bind_idx));
        }

        query_str.push_str(" ORDER BY created_at DESC");

        let mut query = sqlx::query(&query_str).bind(org_id);
        if !status_val.is_empty() { query = query.bind(&status_val); }
        if !priority_val.is_empty() { query = query.bind(&priority_val); }
        if let Some(cid) = customer_id { query = query.bind(cid); }
        if let Some(aid) = assigned_to { query = query.bind(aid); }
        if let Some(catid) = category_id { query = query.bind(catid); }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_request(r)).collect())
    }

    async fn update_request_status(
        &self,
        id: Uuid,
        status: &str,
        resolved_at: Option<chrono::DateTime<chrono::Utc>>,
        closed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ServiceRequest> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.service_requests
            SET status = $2, resolved_at = COALESCE($3, resolved_at),
                closed_at = COALESCE($4, closed_at), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(resolved_at).bind(closed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_request(&row))
    }

    async fn update_request_resolution(
        &self,
        id: Uuid,
        resolution: &str,
        resolution_code: &str,
        resolved_at: chrono::DateTime<chrono::Utc>,
    ) -> AtlasResult<ServiceRequest> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.service_requests
            SET status = 'resolved', resolution = $2, resolution_code = $3,
                resolved_at = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(resolution).bind(resolution_code).bind(resolved_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_request(&row))
    }

    async fn update_request_assignment(
        &self,
        id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.service_requests
            SET assigned_to = $2, assigned_to_name = $3, assigned_group = $4, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(assigned_to).bind(assigned_to_name).bind(assigned_group)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Updates / Communications
    // ========================================================================

    async fn create_update(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        update_type: &str,
        author_id: Option<Uuid>,
        author_name: Option<&str>,
        subject: Option<&str>,
        body: &str,
        is_internal: bool,
    ) -> AtlasResult<ServiceRequestUpdate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.service_request_updates
                (organization_id, request_id, update_type, author_id, author_name,
                 subject, body, is_internal)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(request_id).bind(update_type)
        .bind(author_id).bind(author_name).bind(subject).bind(body).bind(is_internal)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_update(&row))
    }

    async fn list_updates(&self, request_id: Uuid, include_internal: bool) -> AtlasResult<Vec<ServiceRequestUpdate>> {
        let rows = if include_internal {
            sqlx::query(
                "SELECT * FROM _atlas.service_request_updates WHERE request_id = $1 ORDER BY created_at ASC"
            )
            .bind(request_id)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.service_request_updates WHERE request_id = $1 AND is_internal = false ORDER BY created_at ASC"
            )
            .bind(request_id)
            .fetch_all(&self.pool).await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_update(r)).collect())
    }

    // ========================================================================
    // Assignments
    // ========================================================================

    async fn create_assignment(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        assigned_by: Option<Uuid>,
        assigned_by_name: Option<&str>,
        assignment_type: &str,
    ) -> AtlasResult<ServiceRequestAssignment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.service_request_assignments
                (organization_id, request_id, assigned_to, assigned_to_name,
                 assigned_group, assigned_by, assigned_by_name, assignment_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(request_id).bind(assigned_to).bind(assigned_to_name)
        .bind(assigned_group).bind(assigned_by).bind(assigned_by_name).bind(assignment_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_assignment(&row))
    }

    async fn list_assignments(&self, request_id: Uuid) -> AtlasResult<Vec<ServiceRequestAssignment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.service_request_assignments WHERE request_id = $1 ORDER BY created_at DESC"
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_assignment(r)).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ServiceRequestDashboard> {
        // Get counts by status
        let status_rows = sqlx::query(
            r#"
            SELECT status, COUNT(*) as count
            FROM _atlas.service_requests
            WHERE organization_id = $1
            GROUP BY status
            "#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut total_open = 0i32;
        let mut total_resolved = 0i32;
        let mut total_closed = 0i32;
        let mut by_status = serde_json::Map::new();

        for row in &status_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_status.insert(status.clone(), serde_json::json!(count));
            match status.as_str() {
                "open" | "in_progress" | "pending_customer" => total_open += count as i32,
                "resolved" => total_resolved += count as i32,
                "closed" => total_closed += count as i32,
                _ => {}
            }
        }

        // Get unassigned count
        let unassigned: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.service_requests WHERE organization_id = $1 AND assigned_to IS NULL AND status NOT IN ('closed', 'cancelled')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // SLA breached count
        let sla_breached: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.service_requests WHERE organization_id = $1 AND sla_breached = true AND status NOT IN ('closed', 'cancelled')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // By priority
        let priority_rows = sqlx::query(
            "SELECT priority, COUNT(*) as count FROM _atlas.service_requests WHERE organization_id = $1 GROUP BY priority"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_priority = serde_json::Map::new();
        for row in &priority_rows {
            let priority: String = row.try_get("priority").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_priority.insert(priority, serde_json::json!(count));
        }

        // By category
        let category_rows = sqlx::query(
            r#"
            SELECT COALESCE(category_name, 'Uncategorized') as cat, COUNT(*) as count
            FROM _atlas.service_requests
            WHERE organization_id = $1
            GROUP BY category_name
            "#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_category = serde_json::Map::new();
        for row in &category_rows {
            let cat: String = row.try_get("cat").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_category.insert(cat, serde_json::json!(count));
        }

        // By channel
        let channel_rows = sqlx::query(
            "SELECT channel, COUNT(*) as count FROM _atlas.service_requests WHERE organization_id = $1 GROUP BY channel"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_channel = serde_json::Map::new();
        for row in &channel_rows {
            let ch: String = row.try_get("channel").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_channel.insert(ch, serde_json::json!(count));
        }

        // Average resolution time - cast to text to avoid NUMERIC/f64 mismatch
        let avg_hours: f64 = {
            let row = sqlx::query(
                r#"
                SELECT COALESCE(EXTRACT(EPOCH FROM AVG(resolved_at - created_at)) / 3600.0, 0)::text as avg_hours
                FROM _atlas.service_requests
                WHERE organization_id = $1 AND resolved_at IS NOT NULL
                "#
            )
            .bind(org_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

            match row {
                Some(r) => {
                    let val: String = r.try_get("avg_hours").unwrap_or_default();
                    val.parse().unwrap_or(0.0)
                }
                None => 0.0,
            }
        };

        Ok(ServiceRequestDashboard {
            total_open,
            total_resolved,
            total_closed,
            total_unassigned: unassigned as i32,
            sla_breached_count: sla_breached as i32,
            by_priority: serde_json::Value::Object(by_priority),
            by_status: serde_json::Value::Object(by_status),
            by_category: serde_json::Value::Object(by_category),
            by_channel: serde_json::Value::Object(by_channel),
            average_resolution_hours: format!("{:.2}", avg_hours),
        })
    }
}
