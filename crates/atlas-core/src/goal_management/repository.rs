//! Goal Management Repository
//!
//! PostgreSQL storage for goal library categories, templates, plans,
//! goals, alignments, and notes.

use atlas_shared::{
    GoalLibraryCategory, GoalLibraryTemplate, GoalPlan, Goal,
    GoalAlignment, GoalNote, GoalManagementSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Goal Management data storage
#[async_trait]
pub trait GoalManagementRepository: Send + Sync {
    // Library Categories
    async fn create_library_category(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        display_order: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryCategory>;
    async fn get_library_category(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryCategory>>;
    async fn get_library_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalLibraryCategory>>;
    async fn list_library_categories(&self, org_id: Uuid) -> AtlasResult<Vec<GoalLibraryCategory>>;
    async fn delete_library_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Library Templates
    #[allow(clippy::too_many_arguments)]
    async fn create_library_template(
        &self,
        org_id: Uuid, category_id: Option<Uuid>, code: &str, name: &str,
        description: Option<&str>, goal_type: &str, success_criteria: Option<&str>,
        target_metric: Option<&str>, target_value: Option<&str>, uom: Option<&str>,
        suggested_weight: Option<&str>, estimated_duration_days: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryTemplate>;
    async fn get_library_template(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryTemplate>>;
    async fn get_library_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalLibraryTemplate>>;
    async fn list_library_templates(
        &self, org_id: Uuid, category_id: Option<Uuid>, goal_type: Option<&str>,
    ) -> AtlasResult<Vec<GoalLibraryTemplate>>;
    async fn delete_library_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Goal Plans
    #[allow(clippy::too_many_arguments)]
    async fn create_goal_plan(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        plan_type: &str, review_period_start: chrono::NaiveDate,
        review_period_end: chrono::NaiveDate,
        goal_creation_deadline: Option<chrono::NaiveDate>,
        allow_self_goals: bool, allow_team_goals: bool,
        max_weight_sum: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GoalPlan>;
    async fn get_goal_plan(&self, id: Uuid) -> AtlasResult<Option<GoalPlan>>;
    async fn get_goal_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalPlan>>;
    async fn list_goal_plans(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GoalPlan>>;
    async fn update_goal_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<GoalPlan>;
    async fn delete_goal_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Goals
    #[allow(clippy::too_many_arguments)]
    async fn create_goal(
        &self,
        org_id: Uuid, plan_id: Option<Uuid>, parent_goal_id: Option<Uuid>,
        library_template_id: Option<Uuid>, code: Option<&str>,
        name: &str, description: Option<&str>, goal_type: &str,
        category: Option<&str>, owner_id: Uuid, owner_type: &str,
        assigned_by: Option<Uuid>, success_criteria: Option<&str>,
        target_metric: Option<&str>, target_value: Option<&str>,
        uom: Option<&str>, weight: Option<&str>, priority: &str,
        start_date: Option<chrono::NaiveDate>,
        target_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<Goal>;
    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<Goal>>;
    async fn list_goals(
        &self, org_id: Uuid, plan_id: Option<Uuid>, owner_id: Option<Uuid>,
        goal_type: Option<&str>, status: Option<&str>, parent_goal_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Goal>>;
    async fn update_goal_progress(
        &self, id: Uuid, actual_value: Option<&str>, progress_pct: Option<&str>,
        status: Option<&str>, completed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Goal>;
    async fn delete_goal(&self, id: Uuid) -> AtlasResult<()>;

    // Alignments
    async fn create_goal_alignment(
        &self, org_id: Uuid, source_goal_id: Uuid, aligned_to_goal_id: Uuid,
        alignment_type: &str, description: Option<&str>,
    ) -> AtlasResult<GoalAlignment>;
    async fn list_goal_alignments(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalAlignment>>;
    async fn delete_goal_alignment(&self, id: Uuid) -> AtlasResult<()>;

    // Notes
    async fn create_goal_note(
        &self, org_id: Uuid, goal_id: Uuid, author_id: Uuid,
        note_type: &str, content: &str, visibility: &str,
    ) -> AtlasResult<GoalNote>;
    async fn list_goal_notes(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalNote>>;
    async fn delete_goal_note(&self, id: Uuid) -> AtlasResult<()>;

    // Summary
    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<GoalManagementSummary>;
}

/// PostgreSQL implementation
pub struct PostgresGoalManagementRepository {
    pool: PgPool,
}

impl PostgresGoalManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_category(row: &sqlx::postgres::PgRow) -> GoalLibraryCategory {
    GoalLibraryCategory {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        display_order: row.try_get("display_order").unwrap_or(0),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> GoalLibraryTemplate {
    GoalLibraryTemplate {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        category_id: row.try_get("category_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        goal_type: row.try_get("goal_type").unwrap_or_default(),
        success_criteria: row.try_get("success_criteria").unwrap_or_default(),
        target_metric: row.try_get("target_metric").unwrap_or_default(),
        target_value: row.try_get::<String, _>("target_value").ok(),
        uom: row.try_get("uom").unwrap_or_default(),
        suggested_weight: row.try_get::<String, _>("suggested_weight").ok(),
        estimated_duration_days: row.try_get("estimated_duration_days").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_plan(row: &sqlx::postgres::PgRow) -> GoalPlan {
    GoalPlan {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        plan_type: row.try_get("plan_type").unwrap_or_default(),
        review_period_start: row.try_get("review_period_start").unwrap_or_default(),
        review_period_end: row.try_get("review_period_end").unwrap_or_default(),
        goal_creation_deadline: row.try_get("goal_creation_deadline").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        allow_self_goals: row.try_get("allow_self_goals").unwrap_or(true),
        allow_team_goals: row.try_get("allow_team_goals").unwrap_or(true),
        max_weight_sum: row.try_get::<String, _>("max_weight_sum").ok(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_goal(row: &sqlx::postgres::PgRow) -> Goal {
    Goal {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        plan_id: row.try_get("plan_id").unwrap_or_default(),
        parent_goal_id: row.try_get("parent_goal_id").unwrap_or_default(),
        library_template_id: row.try_get("library_template_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        goal_type: row.try_get("goal_type").unwrap_or_default(),
        category: row.try_get("category").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_type: row.try_get("owner_type").unwrap_or_default(),
        assigned_by: row.try_get("assigned_by").unwrap_or_default(),
        success_criteria: row.try_get("success_criteria").unwrap_or_default(),
        target_metric: row.try_get("target_metric").unwrap_or_default(),
        target_value: row.try_get::<String, _>("target_value").ok(),
        actual_value: row.try_get::<String, _>("actual_value").ok(),
        uom: row.try_get("uom").unwrap_or_default(),
        progress_pct: row.try_get::<String, _>("progress_pct").ok(),
        weight: row.try_get::<String, _>("weight").ok(),
        status: row.try_get("status").unwrap_or_else(|_| "not_started".to_string()),
        priority: row.try_get("priority").unwrap_or_else(|_| "medium".to_string()),
        start_date: row.try_get("start_date").unwrap_or_default(),
        target_date: row.try_get("target_date").unwrap_or_default(),
        completed_date: row.try_get("completed_date").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_alignment(row: &sqlx::postgres::PgRow) -> GoalAlignment {
    GoalAlignment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        source_goal_id: row.try_get("source_goal_id").unwrap_or_default(),
        aligned_to_goal_id: row.try_get("aligned_to_goal_id").unwrap_or_default(),
        alignment_type: row.try_get("alignment_type").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_note(row: &sqlx::postgres::PgRow) -> GoalNote {
    GoalNote {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        goal_id: row.try_get("goal_id").unwrap_or_default(),
        author_id: row.try_get("author_id").unwrap_or_default(),
        note_type: row.try_get("note_type").unwrap_or_default(),
        content: row.try_get("content").unwrap_or_default(),
        visibility: row.try_get("visibility").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl GoalManagementRepository for PostgresGoalManagementRepository {
    // ========================================================================
    // Library Categories
    // ========================================================================

    async fn create_library_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        display_order: i32, _created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryCategory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goal_library_categories
                (organization_id, code, name, description, display_order, status, metadata)
            VALUES ($1, $2, $3, $4, $5, 'active', '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(display_order)
        .fetch_one(&self.pool).await?;
        Ok(row_to_category(&row))
    }

    async fn get_library_category(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryCategory>> {
        let row = sqlx::query("SELECT * FROM _atlas.goal_library_categories WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn get_library_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalLibraryCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.goal_library_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn list_library_categories(&self, org_id: Uuid) -> AtlasResult<Vec<GoalLibraryCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.goal_library_categories WHERE organization_id = $1 ORDER BY display_order, name"
        ).bind(org_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_category).collect())
    }

    async fn delete_library_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.goal_library_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Category '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Library Templates
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_library_template(
        &self, org_id: Uuid, category_id: Option<Uuid>, code: &str, name: &str,
        description: Option<&str>, goal_type: &str, success_criteria: Option<&str>,
        target_metric: Option<&str>, target_value: Option<&str>, uom: Option<&str>,
        suggested_weight: Option<&str>, estimated_duration_days: Option<i32>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryTemplate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goal_library_templates
                (organization_id, category_id, code, name, description, goal_type,
                 success_criteria, target_metric, target_value, uom,
                 suggested_weight, estimated_duration_days, status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::NUMERIC, $10,
                    $11::NUMERIC, $12, 'active', '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(category_id).bind(code).bind(name).bind(description)
        .bind(goal_type).bind(success_criteria).bind(target_metric).bind(target_value)
        .bind(uom).bind(suggested_weight).bind(estimated_duration_days)
        .fetch_one(&self.pool).await?;
        Ok(row_to_template(&row))
    }

    async fn get_library_template(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.goal_library_templates WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_template))
    }

    async fn get_library_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalLibraryTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.goal_library_templates WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_template))
    }

    async fn list_library_templates(
        &self, org_id: Uuid, category_id: Option<Uuid>, goal_type: Option<&str>,
    ) -> AtlasResult<Vec<GoalLibraryTemplate>> {
        let rows = if let Some(cat_id) = category_id {
            sqlx::query(
                "SELECT * FROM _atlas.goal_library_templates WHERE organization_id = $1 AND category_id = $2 ORDER BY name"
            ).bind(org_id).bind(cat_id).fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.goal_library_templates WHERE organization_id = $1 ORDER BY name"
            ).bind(org_id).fetch_all(&self.pool).await?
        };
        let mut templates: Vec<_> = rows.iter().map(row_to_template).collect();
        if let Some(gt) = goal_type {
            templates.retain(|t| t.goal_type == gt);
        }
        Ok(templates)
    }

    async fn delete_library_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.goal_library_templates WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Template '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Goal Plans
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_goal_plan(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        plan_type: &str, review_period_start: chrono::NaiveDate,
        review_period_end: chrono::NaiveDate,
        goal_creation_deadline: Option<chrono::NaiveDate>,
        allow_self_goals: bool, allow_team_goals: bool,
        max_weight_sum: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GoalPlan> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goal_plans
                (organization_id, code, name, description, plan_type,
                 review_period_start, review_period_end, goal_creation_deadline,
                 status, allow_self_goals, allow_team_goals, max_weight_sum,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft', $9, $10,
                    $11::NUMERIC, '{}'::jsonb, $12)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(plan_type)
        .bind(review_period_start).bind(review_period_end).bind(goal_creation_deadline)
        .bind(allow_self_goals).bind(allow_team_goals).bind(max_weight_sum)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_plan(&row))
    }

    async fn get_goal_plan(&self, id: Uuid) -> AtlasResult<Option<GoalPlan>> {
        let row = sqlx::query("SELECT * FROM _atlas.goal_plans WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_plan))
    }

    async fn get_goal_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GoalPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.goal_plans WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_plan))
    }

    async fn list_goal_plans(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GoalPlan>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.goal_plans WHERE organization_id = $1 AND status = $2 ORDER BY review_period_start DESC"
            ).bind(org_id).bind(s).fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.goal_plans WHERE organization_id = $1 ORDER BY review_period_start DESC"
            ).bind(org_id).fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_plan).collect())
    }

    async fn update_goal_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<GoalPlan> {
        let row = sqlx::query(
            "UPDATE _atlas.goal_plans SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_plan(&row))
    }

    async fn delete_goal_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.goal_plans WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Plan '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Goals
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    async fn create_goal(
        &self, org_id: Uuid, plan_id: Option<Uuid>, parent_goal_id: Option<Uuid>,
        library_template_id: Option<Uuid>, code: Option<&str>,
        name: &str, description: Option<&str>, goal_type: &str,
        category: Option<&str>, owner_id: Uuid, owner_type: &str,
        assigned_by: Option<Uuid>, success_criteria: Option<&str>,
        target_metric: Option<&str>, target_value: Option<&str>,
        uom: Option<&str>, weight: Option<&str>, priority: &str,
        start_date: Option<chrono::NaiveDate>,
        target_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<Goal> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goals
                (organization_id, plan_id, parent_goal_id, library_template_id,
                 code, name, description, goal_type, category,
                 owner_id, owner_type, assigned_by,
                 success_criteria, target_metric, target_value, uom,
                 weight, priority, status, progress_pct,
                 start_date, target_date, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14, $15::NUMERIC, $16,
                    $17::NUMERIC, $18, 'not_started', 0::NUMERIC,
                    $19, $20, '{}'::jsonb, $21)
            RETURNING *"#,
        )
        .bind(org_id).bind(plan_id).bind(parent_goal_id).bind(library_template_id)
        .bind(code).bind(name).bind(description).bind(goal_type).bind(category)
        .bind(owner_id).bind(owner_type).bind(assigned_by)
        .bind(success_criteria).bind(target_metric).bind(target_value).bind(uom)
        .bind(weight).bind(priority).bind(start_date).bind(target_date).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_goal(&row))
    }

    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<Goal>> {
        let row = sqlx::query("SELECT * FROM _atlas.goals WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_goal))
    }

    async fn list_goals(
        &self, org_id: Uuid, plan_id: Option<Uuid>, owner_id: Option<Uuid>,
        goal_type: Option<&str>, status: Option<&str>, parent_goal_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Goal>> {
        let mut query = String::from("SELECT * FROM _atlas.goals WHERE organization_id = $1");
        let mut bind_idx = 2u32;

        let has_plan = plan_id.is_some();
        if has_plan { query.push_str(&format!(" AND plan_id = ${}", bind_idx)); bind_idx += 1; }
        let has_owner = owner_id.is_some();
        if has_owner { query.push_str(&format!(" AND owner_id = ${}", bind_idx)); bind_idx += 1; }
        let has_type = goal_type.is_some();
        if has_type { query.push_str(&format!(" AND goal_type = ${}", bind_idx)); bind_idx += 1; }
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_parent = parent_goal_id.is_some();
        if has_parent { query.push_str(&format!(" AND parent_goal_id = ${}", bind_idx)); let _ = bind_idx; }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(pid) = plan_id { q = q.bind(pid); }
        if let Some(oid) = owner_id { q = q.bind(oid); }
        if let Some(gt) = goal_type { q = q.bind(gt); }
        if let Some(s) = status { q = q.bind(s); }
        if let Some(pgid) = parent_goal_id { q = q.bind(pgid); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_goal).collect())
    }

    async fn update_goal_progress(
        &self, id: Uuid, actual_value: Option<&str>, progress_pct: Option<&str>,
        status: Option<&str>, completed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Goal> {
        let row = sqlx::query(
            r#"UPDATE _atlas.goals SET
                actual_value = COALESCE($1::NUMERIC, actual_value),
                progress_pct = COALESCE($2::NUMERIC, progress_pct),
                status = COALESCE($3, status),
                completed_date = COALESCE($4, completed_date),
                updated_at = now()
            WHERE id = $5 RETURNING *"#,
        )
        .bind(actual_value).bind(progress_pct).bind(status)
        .bind(completed_date).bind(id)
        .fetch_one(&self.pool).await?;
        Ok(row_to_goal(&row))
    }

    async fn delete_goal(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.goals WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Goal {} not found", id)));
        }
        Ok(())
    }

    // ========================================================================
    // Alignments
    // ========================================================================

    async fn create_goal_alignment(
        &self, org_id: Uuid, source_goal_id: Uuid, aligned_to_goal_id: Uuid,
        alignment_type: &str, description: Option<&str>,
    ) -> AtlasResult<GoalAlignment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goal_alignments
                (organization_id, source_goal_id, aligned_to_goal_id, alignment_type, description, metadata)
            VALUES ($1, $2, $3, $4, $5, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(source_goal_id).bind(aligned_to_goal_id)
        .bind(alignment_type).bind(description)
        .fetch_one(&self.pool).await?;
        Ok(row_to_alignment(&row))
    }

    async fn list_goal_alignments(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalAlignment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.goal_alignments WHERE source_goal_id = $1 OR aligned_to_goal_id = $1 ORDER BY created_at"
        ).bind(goal_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_alignment).collect())
    }

    async fn delete_goal_alignment(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.goal_alignments WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Alignment not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Notes
    // ========================================================================

    async fn create_goal_note(
        &self, org_id: Uuid, goal_id: Uuid, author_id: Uuid,
        note_type: &str, content: &str, visibility: &str,
    ) -> AtlasResult<GoalNote> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.goal_notes
                (organization_id, goal_id, author_id, note_type, content, visibility, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(goal_id).bind(author_id).bind(note_type)
        .bind(content).bind(visibility)
        .fetch_one(&self.pool).await?;
        Ok(row_to_note(&row))
    }

    async fn list_goal_notes(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalNote>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.goal_notes WHERE goal_id = $1 ORDER BY created_at"
        ).bind(goal_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_note).collect())
    }

    async fn delete_goal_note(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.goal_notes WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Note not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Summary
    // ========================================================================

    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<GoalManagementSummary> {
        let status_rows = sqlx::query(
            "SELECT status, COUNT(*) as cnt FROM _atlas.goals WHERE organization_id = $1 GROUP BY status"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut not_started = 0i32;
        let mut in_progress = 0i32;
        let mut on_track = 0i32;
        let mut at_risk = 0i32;
        let mut completed = 0i32;
        let mut cancelled = 0i32;

        for r in &status_rows {
            let status: String = r.try_get("status").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            match status.as_str() {
                "not_started" => not_started = cnt as i32,
                "in_progress" => in_progress = cnt as i32,
                "on_track" => on_track = cnt as i32,
                "at_risk" => at_risk = cnt as i32,
                "completed" => completed = cnt as i32,
                "cancelled" => cancelled = cnt as i32,
                _ => {}
            }
        }
        let total_goals = not_started + in_progress + on_track + at_risk + completed + cancelled;

        // Average progress
        let avg_progress: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(progress_pct) FROM _atlas.goals WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(None);

        // Plans
        let plan_rows = sqlx::query(
            "SELECT status, COUNT(*) as cnt FROM _atlas.goal_plans WHERE organization_id = $1 GROUP BY status"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let total_plans: i32 = plan_rows.iter()
            .map(|r| r.try_get::<i64, _>("cnt").unwrap_or(0) as i32)
            .sum();
        let active_plans = plan_rows.iter()
            .filter(|r| r.try_get::<String, _>("status").unwrap_or_default() == "active")
            .map(|r| r.try_get::<i64, _>("cnt").unwrap_or(0) as i32)
            .sum();

        // Alignments count
        let alignment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.goal_alignments WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        Ok(GoalManagementSummary {
            total_goals,
            goals_not_started: not_started,
            goals_in_progress: in_progress,
            goals_on_track: on_track,
            goals_at_risk: at_risk,
            goals_completed: completed,
            goals_cancelled: cancelled,
            avg_progress_pct: avg_progress.map(|v| format!("{:.2}", v)),
            total_plans,
            active_plans,
            total_alignments: alignment_count as i32,
        })
    }
}
