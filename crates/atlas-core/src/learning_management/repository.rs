//! Learning Management Repository
//!
//! PostgreSQL storage for learning items, categories, enrollments,
//! learning paths, path items, assignments, and dashboard analytics.

use atlas_shared::{
    LearningItem, LearningCategory, LearningEnrollment,
    LearningPath, LearningPathItem, LearningAssignment, LearningDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Learning Management data storage
#[async_trait]
pub trait LearningManagementRepository: Send + Sync {
    // Learning Items
    async fn create_learning_item(
        &self,
        org_id: Uuid, code: &str, title: &str, description: Option<&str>,
        item_type: &str, format: &str, category: Option<&str>,
        provider: Option<&str>, duration_hours: Option<f64>,
        currency_code: Option<&str>, cost: Option<&str>,
        credits: Option<&str>, credit_type: Option<&str>,
        validity_months: Option<i32>, recertification_required: bool,
        max_enrollments: Option<i32>, created_by: Option<Uuid>,
    ) -> AtlasResult<LearningItem>;
    async fn get_learning_item(&self, id: Uuid) -> AtlasResult<Option<LearningItem>>;
    async fn get_learning_item_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningItem>>;
    async fn list_learning_items(
        &self, org_id: Uuid, item_type: Option<&str>, status: Option<&str>,
        category: Option<&str>,
    ) -> AtlasResult<Vec<LearningItem>>;
    async fn update_learning_item_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningItem>;
    async fn delete_learning_item(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Learning Categories
    async fn create_learning_category(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, display_order: i32,
    ) -> AtlasResult<LearningCategory>;
    async fn get_learning_category(&self, id: Uuid) -> AtlasResult<Option<LearningCategory>>;
    async fn get_learning_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningCategory>>;
    async fn list_learning_categories(&self, org_id: Uuid, parent_id: Option<Uuid>) -> AtlasResult<Vec<LearningCategory>>;
    async fn delete_learning_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Learning Enrollments
    async fn create_learning_enrollment(
        &self,
        org_id: Uuid, learning_item_id: Uuid, person_id: Uuid,
        person_name: Option<&str>, enrollment_type: &str,
        enrolled_by: Option<Uuid>, enrollment_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningEnrollment>;
    async fn get_learning_enrollment(&self, id: Uuid) -> AtlasResult<Option<LearningEnrollment>>;
    async fn list_learning_enrollments(
        &self, org_id: Uuid, learning_item_id: Option<Uuid>,
        person_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<LearningEnrollment>>;
    async fn update_enrollment_progress(
        &self, id: Uuid, progress_pct: Option<&str>, score: Option<&str>,
        status: Option<&str>, completion_date: Option<chrono::NaiveDate>,
        certification_expiry: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningEnrollment>;
    async fn delete_learning_enrollment(&self, id: Uuid) -> AtlasResult<()>;

    // Learning Paths
    async fn create_learning_path(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        path_type: &str, target_role: Option<&str>, target_job_id: Option<Uuid>,
        estimated_duration_hours: Option<f64>, created_by: Option<Uuid>,
    ) -> AtlasResult<LearningPath>;
    async fn get_learning_path(&self, id: Uuid) -> AtlasResult<Option<LearningPath>>;
    async fn get_learning_path_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningPath>>;
    async fn list_learning_paths(
        &self, org_id: Uuid, status: Option<&str>, path_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningPath>>;
    async fn update_learning_path_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningPath>;
    async fn update_learning_path_total_items(&self, id: Uuid, total: i32) -> AtlasResult<LearningPath>;
    async fn delete_learning_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Learning Path Items
    async fn create_learning_path_item(
        &self,
        org_id: Uuid, learning_path_id: Uuid, learning_item_id: Uuid,
        sequence_number: i32, is_required: bool, milestone_name: Option<&str>,
    ) -> AtlasResult<LearningPathItem>;
    async fn list_learning_path_items(&self, learning_path_id: Uuid) -> AtlasResult<Vec<LearningPathItem>>;
    async fn delete_learning_path_item(&self, id: Uuid) -> AtlasResult<()>;

    // Learning Assignments
    async fn create_learning_assignment(
        &self,
        org_id: Uuid, learning_item_id: Option<Uuid>, learning_path_id: Option<Uuid>,
        title: &str, description: Option<&str>, assignment_type: &str,
        target_id: Option<Uuid>, assigned_by: Option<Uuid>, priority: &str,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningAssignment>;
    async fn get_learning_assignment(&self, id: Uuid) -> AtlasResult<Option<LearningAssignment>>;
    async fn list_learning_assignments(
        &self, org_id: Uuid, status: Option<&str>, assignment_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningAssignment>>;
    async fn update_learning_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningAssignment>;
    async fn delete_learning_assignment(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_learning_dashboard(&self, org_id: Uuid) -> AtlasResult<LearningDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresLearningManagementRepository {
    pool: PgPool,
}

impl PostgresLearningManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row mapping helpers

fn row_to_learning_item(row: &sqlx::postgres::PgRow) -> LearningItem {
    LearningItem {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        item_type: row.try_get("item_type").unwrap_or_else(|_| "course".to_string()),
        format: row.try_get("format").unwrap_or_else(|_| "self_paced".to_string()),
        category: row.try_get("category").unwrap_or_default(),
        provider: row.try_get("provider").unwrap_or_default(),
        duration_hours: row.try_get::<f64, _>("duration_hours").ok(),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        cost: row.try_get::<String, _>("cost").ok(),
        credits: row.try_get::<String, _>("credits").ok(),
        credit_type: row.try_get("credit_type").unwrap_or_default(),
        validity_months: row.try_get("validity_months").unwrap_or_default(),
        recertification_required: row.try_get("recertification_required").unwrap_or(false),
        max_enrollments: row.try_get("max_enrollments").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_category(row: &sqlx::postgres::PgRow) -> LearningCategory {
    LearningCategory {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        parent_category_id: row.try_get("parent_category_id").unwrap_or_default(),
        display_order: row.try_get("display_order").unwrap_or(0),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_enrollment(row: &sqlx::postgres::PgRow) -> LearningEnrollment {
    LearningEnrollment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        learning_item_id: row.try_get("learning_item_id").unwrap_or_default(),
        person_id: row.try_get("person_id").unwrap_or_default(),
        person_name: row.try_get("person_name").unwrap_or_default(),
        enrollment_type: row.try_get("enrollment_type").unwrap_or_else(|_| "self".to_string()),
        enrolled_by: row.try_get("enrolled_by").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "enrolled".to_string()),
        progress_pct: row.try_get::<String, _>("progress_pct").ok(),
        score: row.try_get::<String, _>("score").ok(),
        enrollment_date: row.try_get("enrollment_date").unwrap_or_default(),
        completion_date: row.try_get("completion_date").unwrap_or_default(),
        due_date: row.try_get("due_date").unwrap_or_default(),
        certification_expiry: row.try_get("certification_expiry").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_learning_path(row: &sqlx::postgres::PgRow) -> LearningPath {
    LearningPath {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        path_type: row.try_get("path_type").unwrap_or_else(|_| "sequential".to_string()),
        target_role: row.try_get("target_role").unwrap_or_default(),
        target_job_id: row.try_get("target_job_id").unwrap_or_default(),
        estimated_duration_hours: row.try_get::<f64, _>("estimated_duration_hours").ok(),
        total_items: row.try_get("total_items").unwrap_or(0),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_path_item(row: &sqlx::postgres::PgRow) -> LearningPathItem {
    LearningPathItem {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        learning_path_id: row.try_get("learning_path_id").unwrap_or_default(),
        learning_item_id: row.try_get("learning_item_id").unwrap_or_default(),
        sequence_number: row.try_get("sequence_number").unwrap_or(1),
        is_required: row.try_get("is_required").unwrap_or(true),
        milestone_name: row.try_get("milestone_name").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_assignment(row: &sqlx::postgres::PgRow) -> LearningAssignment {
    LearningAssignment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        learning_item_id: row.try_get("learning_item_id").unwrap_or_default(),
        learning_path_id: row.try_get("learning_path_id").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        assignment_type: row.try_get("assignment_type").unwrap_or_else(|_| "individual".to_string()),
        target_id: row.try_get("target_id").unwrap_or_default(),
        assigned_by: row.try_get("assigned_by").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_else(|_| "medium".to_string()),
        due_date: row.try_get("due_date").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl LearningManagementRepository for PostgresLearningManagementRepository {
    // ========================================================================
    // Learning Items
    // ========================================================================

    async fn create_learning_item(
        &self, org_id: Uuid, code: &str, title: &str, description: Option<&str>,
        item_type: &str, format: &str, category: Option<&str>,
        provider: Option<&str>, duration_hours: Option<f64>,
        currency_code: Option<&str>, cost: Option<&str>,
        credits: Option<&str>, credit_type: Option<&str>,
        validity_months: Option<i32>, recertification_required: bool,
        max_enrollments: Option<i32>, created_by: Option<Uuid>,
    ) -> AtlasResult<LearningItem> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_items
                (organization_id, code, title, description, item_type, format,
                 category, provider, duration_hours, currency_code, cost,
                 credits, credit_type, validity_months, recertification_required,
                 max_enrollments, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::NUMERIC,
                    $12::NUMERIC, $13, $14, $15, $16, '{}'::jsonb, $17)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(title).bind(description)
        .bind(item_type).bind(format).bind(category).bind(provider)
        .bind(duration_hours).bind(currency_code).bind(cost)
        .bind(credits).bind(credit_type).bind(validity_months)
        .bind(recertification_required).bind(max_enrollments).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_learning_item(&row))
    }

    async fn get_learning_item(&self, id: Uuid) -> AtlasResult<Option<LearningItem>> {
        let row = sqlx::query("SELECT * FROM _atlas.learning_items WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_learning_item))
    }

    async fn get_learning_item_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningItem>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.learning_items WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_learning_item))
    }

    async fn list_learning_items(
        &self, org_id: Uuid, item_type: Option<&str>, status: Option<&str>,
        category: Option<&str>,
    ) -> AtlasResult<Vec<LearningItem>> {
        let mut query = String::from("SELECT * FROM _atlas.learning_items WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_type = item_type.is_some();
        if has_type { query.push_str(&format!(" AND item_type = ${}", bind_idx)); bind_idx += 1; }
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_cat = category.is_some();
        if has_cat { query.push_str(&format!(" AND category = ${}", bind_idx)); }
        query.push_str(" ORDER BY title");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(t) = item_type { q = q.bind(t); }
        if let Some(s) = status { q = q.bind(s); }
        if let Some(c) = category { q = q.bind(c); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_learning_item).collect())
    }

    async fn update_learning_item_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningItem> {
        let row = sqlx::query(
            "UPDATE _atlas.learning_items SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_learning_item(&row))
    }

    async fn delete_learning_item(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.learning_items WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Learning item '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Learning Categories
    // ========================================================================

    async fn create_learning_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, display_order: i32,
    ) -> AtlasResult<LearningCategory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_categories
                (organization_id, code, name, description, parent_category_id,
                 display_order, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, 'active', '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_category_id).bind(display_order)
        .fetch_one(&self.pool).await?;
        Ok(row_to_category(&row))
    }

    async fn get_learning_category(&self, id: Uuid) -> AtlasResult<Option<LearningCategory>> {
        let row = sqlx::query("SELECT * FROM _atlas.learning_categories WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn get_learning_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.learning_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn list_learning_categories(&self, org_id: Uuid, parent_id: Option<Uuid>) -> AtlasResult<Vec<LearningCategory>> {
        let rows = if let Some(pid) = parent_id {
            sqlx::query(
                "SELECT * FROM _atlas.learning_categories WHERE organization_id = $1 AND parent_category_id = $2 ORDER BY display_order, name"
            ).bind(org_id).bind(pid).fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.learning_categories WHERE organization_id = $1 ORDER BY display_order, name"
            ).bind(org_id).fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_category).collect())
    }

    async fn delete_learning_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.learning_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Learning category '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Learning Enrollments
    // ========================================================================

    async fn create_learning_enrollment(
        &self, org_id: Uuid, learning_item_id: Uuid, person_id: Uuid,
        person_name: Option<&str>, enrollment_type: &str,
        enrolled_by: Option<Uuid>, enrollment_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningEnrollment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_enrollments
                (organization_id, learning_item_id, person_id, person_name,
                 enrollment_type, enrolled_by, status, progress_pct,
                 enrollment_date, due_date, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, 'enrolled', 0::NUMERIC, $7, $8, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(learning_item_id).bind(person_id).bind(person_name)
        .bind(enrollment_type).bind(enrolled_by).bind(enrollment_date).bind(due_date)
        .fetch_one(&self.pool).await?;
        Ok(row_to_enrollment(&row))
    }

    async fn get_learning_enrollment(&self, id: Uuid) -> AtlasResult<Option<LearningEnrollment>> {
        let row = sqlx::query("SELECT * FROM _atlas.learning_enrollments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_enrollment))
    }

    async fn list_learning_enrollments(
        &self, org_id: Uuid, learning_item_id: Option<Uuid>,
        person_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<LearningEnrollment>> {
        let mut query = String::from("SELECT * FROM _atlas.learning_enrollments WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_item = learning_item_id.is_some();
        if has_item { query.push_str(&format!(" AND learning_item_id = ${}", bind_idx)); bind_idx += 1; }
        let has_person = person_id.is_some();
        if has_person { query.push_str(&format!(" AND person_id = ${}", bind_idx)); bind_idx += 1; }
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(li) = learning_item_id { q = q.bind(li); }
        if let Some(pi) = person_id { q = q.bind(pi); }
        if let Some(s) = status { q = q.bind(s); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_enrollment).collect())
    }

    async fn update_enrollment_progress(
        &self, id: Uuid, progress_pct: Option<&str>, score: Option<&str>,
        status: Option<&str>, completion_date: Option<chrono::NaiveDate>,
        certification_expiry: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningEnrollment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.learning_enrollments SET
                progress_pct = COALESCE($1::NUMERIC, progress_pct),
                score = COALESCE($2::NUMERIC, score),
                status = COALESCE($3, status),
                completion_date = COALESCE($4, completion_date),
                certification_expiry = COALESCE($5, certification_expiry),
                updated_at = now()
            WHERE id = $6 RETURNING *"#,
        )
        .bind(progress_pct).bind(score).bind(status)
        .bind(completion_date).bind(certification_expiry).bind(id)
        .fetch_one(&self.pool).await?;
        Ok(row_to_enrollment(&row))
    }

    async fn delete_learning_enrollment(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.learning_enrollments WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Enrollment not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Learning Paths
    // ========================================================================

    async fn create_learning_path(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        path_type: &str, target_role: Option<&str>, target_job_id: Option<Uuid>,
        estimated_duration_hours: Option<f64>, created_by: Option<Uuid>,
    ) -> AtlasResult<LearningPath> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_paths
                (organization_id, code, name, description, path_type,
                 target_role, target_job_id, estimated_duration_hours,
                 total_items, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, '{}'::jsonb, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(path_type)
        .bind(target_role).bind(target_job_id).bind(estimated_duration_hours)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_learning_path(&row))
    }

    async fn get_learning_path(&self, id: Uuid) -> AtlasResult<Option<LearningPath>> {
        let row = sqlx::query("SELECT * FROM _atlas.learning_paths WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_learning_path))
    }

    async fn get_learning_path_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LearningPath>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.learning_paths WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_learning_path))
    }

    async fn list_learning_paths(
        &self, org_id: Uuid, status: Option<&str>, path_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningPath>> {
        let mut query = String::from("SELECT * FROM _atlas.learning_paths WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_type = path_type.is_some();
        if has_type { query.push_str(&format!(" AND path_type = ${}", bind_idx)); }
        query.push_str(" ORDER BY name");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(pt) = path_type { q = q.bind(pt); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_learning_path).collect())
    }

    async fn update_learning_path_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningPath> {
        let row = sqlx::query(
            "UPDATE _atlas.learning_paths SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_learning_path(&row))
    }

    async fn update_learning_path_total_items(&self, id: Uuid, total: i32) -> AtlasResult<LearningPath> {
        let row = sqlx::query(
            "UPDATE _atlas.learning_paths SET total_items = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(total).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_learning_path(&row))
    }

    async fn delete_learning_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.learning_paths WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Learning path '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Learning Path Items
    // ========================================================================

    async fn create_learning_path_item(
        &self, org_id: Uuid, learning_path_id: Uuid, learning_item_id: Uuid,
        sequence_number: i32, is_required: bool, milestone_name: Option<&str>,
    ) -> AtlasResult<LearningPathItem> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_path_items
                (organization_id, learning_path_id, learning_item_id,
                 sequence_number, is_required, milestone_name, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(learning_path_id).bind(learning_item_id)
        .bind(sequence_number).bind(is_required).bind(milestone_name)
        .fetch_one(&self.pool).await?;
        Ok(row_to_path_item(&row))
    }

    async fn list_learning_path_items(&self, learning_path_id: Uuid) -> AtlasResult<Vec<LearningPathItem>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.learning_path_items WHERE learning_path_id = $1 ORDER BY sequence_number"
        ).bind(learning_path_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_path_item).collect())
    }

    async fn delete_learning_path_item(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.learning_path_items WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Path item not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Learning Assignments
    // ========================================================================

    async fn create_learning_assignment(
        &self, org_id: Uuid, learning_item_id: Option<Uuid>, learning_path_id: Option<Uuid>,
        title: &str, description: Option<&str>, assignment_type: &str,
        target_id: Option<Uuid>, assigned_by: Option<Uuid>, priority: &str,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningAssignment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.learning_assignments
                (organization_id, learning_item_id, learning_path_id,
                 title, description, assignment_type, target_id,
                 assigned_by, priority, due_date, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(learning_item_id).bind(learning_path_id)
        .bind(title).bind(description).bind(assignment_type)
        .bind(target_id).bind(assigned_by).bind(priority).bind(due_date)
        .fetch_one(&self.pool).await?;
        Ok(row_to_assignment(&row))
    }

    async fn get_learning_assignment(&self, id: Uuid) -> AtlasResult<Option<LearningAssignment>> {
        let row = sqlx::query("SELECT * FROM _atlas.learning_assignments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_assignment))
    }

    async fn list_learning_assignments(
        &self, org_id: Uuid, status: Option<&str>, assignment_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningAssignment>> {
        let mut query = String::from("SELECT * FROM _atlas.learning_assignments WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_type = assignment_type.is_some();
        if has_type { query.push_str(&format!(" AND assignment_type = ${}", bind_idx)); }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(at) = assignment_type { q = q.bind(at); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_assignment).collect())
    }

    async fn update_learning_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningAssignment> {
        let row = sqlx::query(
            "UPDATE _atlas.learning_assignments SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_assignment(&row))
    }

    async fn delete_learning_assignment(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.learning_assignments WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Assignment not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_learning_dashboard(&self, org_id: Uuid) -> AtlasResult<LearningDashboard> {
        // Learning items by type
        let type_rows = sqlx::query(
            "SELECT item_type, COUNT(*) as cnt FROM _atlas.learning_items WHERE organization_id = $1 GROUP BY item_type"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let mut items_by_type = serde_json::Map::new();
        let mut total_items = 0i32;
        for r in &type_rows {
            let t: String = r.try_get("item_type").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            total_items += cnt as i32;
            items_by_type.insert(t, serde_json::json!(cnt));
        }

        let active_items: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.learning_items WHERE organization_id = $1 AND status = 'active'"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        // Enrollments by status
        let status_rows = sqlx::query(
            "SELECT status, COUNT(*) as cnt FROM _atlas.learning_enrollments WHERE organization_id = $1 GROUP BY status"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let mut enrollments_by_status = serde_json::Map::new();
        let mut total_enrollments = 0i32;
        let mut completed_count = 0i32;
        for r in &status_rows {
            let s: String = r.try_get("status").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            total_enrollments += cnt as i32;
            if s == "completed" { completed_count = cnt as i32; }
            enrollments_by_status.insert(s, serde_json::json!(cnt));
        }

        let completion_rate = if total_enrollments > 0 {
            Some(format!("{:.1}", (completed_count as f64 / total_enrollments as f64) * 100.0))
        } else {
            None
        };

        let total_learning_paths: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.learning_paths WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let total_active_assignments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.learning_assignments WHERE organization_id = $1 AND status = 'active'"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let overdue_enrollments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.learning_enrollments WHERE organization_id = $1 AND due_date < CURRENT_DATE AND status NOT IN ('completed', 'withdrawn', 'expired')"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let avg_score: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(score) FROM _atlas.learning_enrollments WHERE organization_id = $1 AND score IS NOT NULL"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(None);

        Ok(LearningDashboard {
            total_learning_items: total_items,
            active_items: active_items as i32,
            items_by_type: serde_json::Value::Object(items_by_type),
            total_enrollments,
            enrollments_by_status: serde_json::Value::Object(enrollments_by_status),
            completion_rate,
            total_learning_paths: total_learning_paths as i32,
            total_active_assignments: total_active_assignments as i32,
            overdue_enrollments: overdue_enrollments as i32,
            avg_score: avg_score.map(|v| format!("{:.1}", v)),
        })
    }
}
