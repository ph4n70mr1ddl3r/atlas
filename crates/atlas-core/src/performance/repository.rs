//! Performance Management Repository
//!
//! PostgreSQL storage for rating models, review cycles, competencies,
//! performance documents, goals, competency assessments, and feedback.

use atlas_shared::{
    PerformanceRatingModel, PerformanceReviewCycle, PerformanceCompetency,
    PerformanceDocument, PerformanceGoal, CompetencyAssessment,
    PerformanceFeedback,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for performance management data storage
#[async_trait]
pub trait PerformanceRepository: Send + Sync {
    // Rating Models
    async fn create_rating_model(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        rating_scale: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceRatingModel>;
    async fn get_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceRatingModel>>;
    async fn list_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<PerformanceRatingModel>>;
    async fn delete_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Review Cycles
    async fn create_review_cycle(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        cycle_type: &str, rating_model_id: Option<Uuid>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        goal_setting_start: Option<chrono::NaiveDate>, goal_setting_end: Option<chrono::NaiveDate>,
        self_evaluation_start: Option<chrono::NaiveDate>, self_evaluation_end: Option<chrono::NaiveDate>,
        manager_evaluation_start: Option<chrono::NaiveDate>, manager_evaluation_end: Option<chrono::NaiveDate>,
        calibration_date: Option<chrono::NaiveDate>,
        require_goals: bool, require_competencies: bool,
        min_goals: i32, max_goals: i32, goal_weight_total: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceReviewCycle>;
    async fn get_review_cycle(&self, id: Uuid) -> AtlasResult<Option<PerformanceReviewCycle>>;
    async fn list_review_cycles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PerformanceReviewCycle>>;
    async fn update_cycle_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceReviewCycle>;

    // Competencies
    async fn create_competency(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: Option<&str>, rating_model_id: Option<Uuid>,
        behavioral_indicators: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceCompetency>;
    async fn get_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceCompetency>>;
    async fn list_competencies(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<PerformanceCompetency>>;
    async fn delete_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Documents
    async fn create_document(
        &self, org_id: Uuid, review_cycle_id: Uuid, employee_id: Uuid,
        employee_name: Option<&str>, manager_id: Option<Uuid>,
        manager_name: Option<&str>, document_number: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceDocument>;
    async fn get_document(&self, id: Uuid) -> AtlasResult<Option<PerformanceDocument>>;
    async fn get_document_by_cycle_employee(&self, org_id: Uuid, cycle_id: Uuid, employee_id: Uuid) -> AtlasResult<Option<PerformanceDocument>>;
    async fn list_documents(
        &self, org_id: Uuid, review_cycle_id: Option<Uuid>,
        employee_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<PerformanceDocument>>;
    async fn update_document_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceDocument>;
    async fn update_self_evaluation(&self, id: Uuid, overall_rating: Option<&str>, comments: Option<&str>) -> AtlasResult<PerformanceDocument>;
    async fn update_manager_evaluation(&self, id: Uuid, overall_rating: Option<&str>, comments: Option<&str>) -> AtlasResult<PerformanceDocument>;
    async fn finalize_document(&self, id: Uuid, final_rating: Option<&str>, final_comments: Option<&str>) -> AtlasResult<PerformanceDocument>;

    // Goals
    async fn create_goal(
        &self, org_id: Uuid, document_id: Uuid, employee_id: Uuid,
        goal_name: &str, description: Option<&str>, goal_category: Option<&str>,
        weight: &str, target_metric: Option<&str>,
        start_date: Option<chrono::NaiveDate>, due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceGoal>;
    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<PerformanceGoal>>;
    async fn list_goals(&self, document_id: Uuid) -> AtlasResult<Vec<PerformanceGoal>>;
    async fn list_goals_by_cycle(&self, org_id: Uuid, cycle_id: Uuid) -> AtlasResult<Vec<PerformanceGoal>>;
    async fn update_goal_status(
        &self, id: Uuid, status: &str, actual_result: Option<&str>,
        completed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<PerformanceGoal>;
    async fn update_goal_self_rating(&self, id: Uuid, rating: &str, comments: Option<&str>) -> AtlasResult<PerformanceGoal>;
    async fn update_goal_manager_rating(&self, id: Uuid, rating: &str, comments: Option<&str>) -> AtlasResult<PerformanceGoal>;
    async fn delete_goal(&self, id: Uuid) -> AtlasResult<()>;

    // Competency Assessments
    async fn upsert_competency_assessment(
        &self, org_id: Uuid, document_id: Uuid, employee_id: Uuid,
        competency_id: Uuid, rating_type: &str, rating: &str,
        comments: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CompetencyAssessment>;
    async fn list_competency_assessments(&self, document_id: Uuid) -> AtlasResult<Vec<CompetencyAssessment>>;

    // Feedback
    async fn create_feedback(
        &self, org_id: Uuid, document_id: Option<Uuid>, employee_id: Uuid,
        from_user_id: Uuid, from_user_name: Option<&str>,
        feedback_type: &str, subject: Option<&str>, content: &str,
        visibility: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceFeedback>;
    async fn get_feedback(&self, id: Uuid) -> AtlasResult<Option<PerformanceFeedback>>;
    async fn list_feedback(&self, org_id: Uuid, employee_id: Option<Uuid>, document_id: Option<Uuid>) -> AtlasResult<Vec<PerformanceFeedback>>;
    async fn update_feedback_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceFeedback>;
}

/// PostgreSQL implementation
pub struct PostgresPerformanceRepository {
    pool: PgPool,
}

impl PostgresPerformanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn num_to_str(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

#[async_trait]
impl PerformanceRepository for PostgresPerformanceRepository {
    // ========================================================================
    // Rating Models
    // ========================================================================

    async fn create_rating_model(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        rating_scale: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceRatingModel> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_rating_models
                (organization_id, code, name, description, rating_scale, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, rating_scale = $5, is_active = true, updated_at = now()
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(&rating_scale).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PerformanceRatingModel {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            rating_scale: row.try_get("rating_scale").unwrap_or(serde_json::json!([])),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceRatingModel>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.performance_rating_models WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PerformanceRatingModel {
            id: r.get("id"), organization_id: r.get("organization_id"),
            code: r.get("code"), name: r.get("name"), description: r.get("description"),
            rating_scale: r.try_get("rating_scale").unwrap_or(serde_json::json!([])),
            is_active: r.get("is_active"), created_by: r.get("created_by"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        }))
    }

    async fn list_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<PerformanceRatingModel>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.performance_rating_models WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PerformanceRatingModel {
            id: r.get("id"), organization_id: r.get("organization_id"),
            code: r.get("code"), name: r.get("name"), description: r.get("description"),
            rating_scale: r.try_get("rating_scale").unwrap_or(serde_json::json!([])),
            is_active: r.get("is_active"), created_by: r.get("created_by"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.performance_rating_models SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Review Cycles
    // ========================================================================

    async fn create_review_cycle(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        cycle_type: &str, rating_model_id: Option<Uuid>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        goal_setting_start: Option<chrono::NaiveDate>, goal_setting_end: Option<chrono::NaiveDate>,
        self_evaluation_start: Option<chrono::NaiveDate>, self_evaluation_end: Option<chrono::NaiveDate>,
        manager_evaluation_start: Option<chrono::NaiveDate>, manager_evaluation_end: Option<chrono::NaiveDate>,
        calibration_date: Option<chrono::NaiveDate>,
        require_goals: bool, require_competencies: bool,
        min_goals: i32, max_goals: i32, goal_weight_total: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceReviewCycle> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_review_cycles
                (organization_id, name, description, cycle_type, rating_model_id,
                 start_date, end_date, goal_setting_start, goal_setting_end,
                 self_evaluation_start, self_evaluation_end,
                 manager_evaluation_start, manager_evaluation_end, calibration_date,
                 require_goals, require_competencies, min_goals, max_goals,
                 goal_weight_total, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19::numeric, $20)
            RETURNING *"#,
        )
        .bind(org_id).bind(name).bind(description).bind(cycle_type).bind(rating_model_id)
        .bind(start_date).bind(end_date)
        .bind(goal_setting_start).bind(goal_setting_end)
        .bind(self_evaluation_start).bind(self_evaluation_end)
        .bind(manager_evaluation_start).bind(manager_evaluation_end)
        .bind(calibration_date)
        .bind(require_goals).bind(require_competencies)
        .bind(min_goals).bind(max_goals)
        .bind(goal_weight_total).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_cycle(&row))
    }

    async fn get_review_cycle(&self, id: Uuid) -> AtlasResult<Option<PerformanceReviewCycle>> {
        let row = sqlx::query("SELECT * FROM _atlas.performance_review_cycles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cycle(&r)))
    }

    async fn list_review_cycles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PerformanceReviewCycle>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.performance_review_cycles
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY start_date DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cycle).collect())
    }

    async fn update_cycle_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceReviewCycle> {
        let row = sqlx::query(
            "UPDATE _atlas.performance_review_cycles SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle(&row))
    }

    // ========================================================================
    // Competencies
    // ========================================================================

    async fn create_competency(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: Option<&str>, rating_model_id: Option<Uuid>,
        behavioral_indicators: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceCompetency> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_competencies
                (organization_id, code, name, description, category, rating_model_id, behavioral_indicators, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, category = $5, rating_model_id = $6,
                    behavioral_indicators = $7, is_active = true, updated_at = now()
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(category)
        .bind(rating_model_id).bind(&behavioral_indicators).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(PerformanceCompetency {
            id: row.get("id"), organization_id: row.get("organization_id"),
            code: row.get("code"), name: row.get("name"), description: row.get("description"),
            category: row.get("category"), rating_model_id: row.get("rating_model_id"),
            behavioral_indicators: row.try_get("behavioral_indicators").unwrap_or(serde_json::json!([])),
            is_active: row.get("is_active"), created_by: row.get("created_by"),
            created_at: row.get("created_at"), updated_at: row.get("updated_at"),
        })
    }

    async fn get_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PerformanceCompetency>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.performance_competencies WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| PerformanceCompetency {
            id: r.get("id"), organization_id: r.get("organization_id"),
            code: r.get("code"), name: r.get("name"), description: r.get("description"),
            category: r.get("category"), rating_model_id: r.get("rating_model_id"),
            behavioral_indicators: r.try_get("behavioral_indicators").unwrap_or(serde_json::json!([])),
            is_active: r.get("is_active"), created_by: r.get("created_by"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        }))
    }

    async fn list_competencies(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<PerformanceCompetency>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.performance_competencies
            WHERE organization_id = $1 AND is_active = true
              AND ($2::text IS NULL OR category = $2)
            ORDER BY code"#,
        )
        .bind(org_id).bind(category)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| PerformanceCompetency {
            id: r.get("id"), organization_id: r.get("organization_id"),
            code: r.get("code"), name: r.get("name"), description: r.get("description"),
            category: r.get("category"), rating_model_id: r.get("rating_model_id"),
            behavioral_indicators: r.try_get("behavioral_indicators").unwrap_or(serde_json::json!([])),
            is_active: r.get("is_active"), created_by: r.get("created_by"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_competency(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.performance_competencies SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Documents
    // ========================================================================

    async fn create_document(
        &self, org_id: Uuid, review_cycle_id: Uuid, employee_id: Uuid,
        employee_name: Option<&str>, manager_id: Option<Uuid>,
        manager_name: Option<&str>, document_number: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceDocument> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_documents
                (organization_id, review_cycle_id, employee_id, employee_name,
                 manager_id, manager_name, document_number, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *"#,
        )
        .bind(org_id).bind(review_cycle_id).bind(employee_id).bind(employee_name)
        .bind(manager_id).bind(manager_name).bind(document_number).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_doc(&row))
    }

    async fn get_document(&self, id: Uuid) -> AtlasResult<Option<PerformanceDocument>> {
        let row = sqlx::query("SELECT * FROM _atlas.performance_documents WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_doc(&r)))
    }

    async fn get_document_by_cycle_employee(&self, org_id: Uuid, cycle_id: Uuid, employee_id: Uuid) -> AtlasResult<Option<PerformanceDocument>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.performance_documents WHERE organization_id = $1 AND review_cycle_id = $2 AND employee_id = $3"
        )
        .bind(org_id).bind(cycle_id).bind(employee_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_doc(&r)))
    }

    async fn list_documents(
        &self, org_id: Uuid, review_cycle_id: Option<Uuid>,
        employee_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<PerformanceDocument>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.performance_documents
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR review_cycle_id = $2)
              AND ($3::uuid IS NULL OR employee_id = $3)
              AND ($4::text IS NULL OR status = $4)
            ORDER BY employee_name"#,
        )
        .bind(org_id).bind(review_cycle_id).bind(employee_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_doc).collect())
    }

    async fn update_document_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceDocument> {
        let row = sqlx::query(
            "UPDATE _atlas.performance_documents SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_doc(&row))
    }

    async fn update_self_evaluation(&self, id: Uuid, overall_rating: Option<&str>, comments: Option<&str>) -> AtlasResult<PerformanceDocument> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_documents
            SET self_overall_rating = $2::numeric, self_comments = $3, updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(overall_rating).bind(comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_doc(&row))
    }

    async fn update_manager_evaluation(&self, id: Uuid, overall_rating: Option<&str>, comments: Option<&str>) -> AtlasResult<PerformanceDocument> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_documents
            SET manager_overall_rating = $2::numeric, manager_comments = $3, updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(overall_rating).bind(comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_doc(&row))
    }

    async fn finalize_document(&self, id: Uuid, final_rating: Option<&str>, final_comments: Option<&str>) -> AtlasResult<PerformanceDocument> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_documents
            SET status = 'completed', final_rating = $2::numeric, final_comments = $3,
                overall_rating = COALESCE($2::numeric, manager_overall_rating),
                updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(final_rating).bind(final_comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_doc(&row))
    }

    // ========================================================================
    // Goals
    // ========================================================================

    async fn create_goal(
        &self, org_id: Uuid, document_id: Uuid, employee_id: Uuid,
        goal_name: &str, description: Option<&str>, goal_category: Option<&str>,
        weight: &str, target_metric: Option<&str>,
        start_date: Option<chrono::NaiveDate>, due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceGoal> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_goals
                (organization_id, document_id, employee_id, goal_name, description,
                 goal_category, weight, target_metric, start_date, due_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11)
            RETURNING *"#,
        )
        .bind(org_id).bind(document_id).bind(employee_id).bind(goal_name).bind(description)
        .bind(goal_category).bind(weight).bind(target_metric)
        .bind(start_date).bind(due_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_goal(&row))
    }

    async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<PerformanceGoal>> {
        let row = sqlx::query("SELECT * FROM _atlas.performance_goals WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_goal(&r)))
    }

    async fn list_goals(&self, document_id: Uuid) -> AtlasResult<Vec<PerformanceGoal>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.performance_goals WHERE document_id = $1 ORDER BY created_at"
        )
        .bind(document_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_goal).collect())
    }

    async fn list_goals_by_cycle(&self, org_id: Uuid, cycle_id: Uuid) -> AtlasResult<Vec<PerformanceGoal>> {
        let rows = sqlx::query(
            r#"SELECT g.* FROM _atlas.performance_goals g
            JOIN _atlas.performance_documents d ON g.document_id = d.id
            WHERE g.organization_id = $1 AND d.review_cycle_id = $2
            ORDER BY g.created_at"#,
        )
        .bind(org_id).bind(cycle_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_goal).collect())
    }

    async fn update_goal_status(
        &self, id: Uuid, status: &str, actual_result: Option<&str>,
        completed_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<PerformanceGoal> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_goals
            SET status = $2, actual_result = COALESCE($3, actual_result),
                completed_date = COALESCE($4, completed_date), updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(actual_result).bind(completed_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_goal(&row))
    }

    async fn update_goal_self_rating(&self, id: Uuid, rating: &str, comments: Option<&str>) -> AtlasResult<PerformanceGoal> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_goals
            SET self_rating = $2::numeric, self_comments = $3, updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(rating).bind(comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_goal(&row))
    }

    async fn update_goal_manager_rating(&self, id: Uuid, rating: &str, comments: Option<&str>) -> AtlasResult<PerformanceGoal> {
        let row = sqlx::query(
            r#"UPDATE _atlas.performance_goals
            SET manager_rating = $2::numeric, manager_comments = $3, updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(rating).bind(comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_goal(&row))
    }

    async fn delete_goal(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.performance_goals WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Competency Assessments
    // ========================================================================

    async fn upsert_competency_assessment(
        &self, org_id: Uuid, document_id: Uuid, employee_id: Uuid,
        competency_id: Uuid, rating_type: &str, rating: &str,
        comments: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CompetencyAssessment> {
        let rating_col = match rating_type {
            "self" => "self_rating",
            "manager" => "manager_rating",
            "calibration" => "calibration_rating",
            _ => "self_rating",
        };
        let comments_col = match rating_type {
            "self" => "self_comments",
            "manager" => "manager_comments",
            "calibration" => "calibration_comments",
            _ => "self_comments",
        };

        let query = format!(
            r#"INSERT INTO _atlas.performance_competency_assessments
                (organization_id, document_id, employee_id, competency_id, {}, {}, created_by)
            VALUES ($1, $2, $3, $4, $5::numeric, $6, $7)
            ON CONFLICT (document_id, competency_id) DO UPDATE
                SET {} = $5::numeric, {} = $6, updated_at = now()
            RETURNING *"#,
            rating_col, comments_col, rating_col, comments_col
        );

        let row = sqlx::query(&query)
            .bind(org_id).bind(document_id).bind(employee_id)
            .bind(competency_id).bind(rating).bind(comments).bind(created_by)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_assessment(&row))
    }

    async fn list_competency_assessments(&self, document_id: Uuid) -> AtlasResult<Vec<CompetencyAssessment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.performance_competency_assessments WHERE document_id = $1 ORDER BY created_at"
        )
        .bind(document_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_assessment).collect())
    }

    // ========================================================================
    // Feedback
    // ========================================================================

    async fn create_feedback(
        &self, org_id: Uuid, document_id: Option<Uuid>, employee_id: Uuid,
        from_user_id: Uuid, from_user_name: Option<&str>,
        feedback_type: &str, subject: Option<&str>, content: &str,
        visibility: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceFeedback> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.performance_feedback
                (organization_id, document_id, employee_id, from_user_id, from_user_name,
                 feedback_type, subject, content, visibility, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *"#,
        )
        .bind(org_id).bind(document_id).bind(employee_id).bind(from_user_id).bind(from_user_name)
        .bind(feedback_type).bind(subject).bind(content).bind(visibility).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_feedback(&row))
    }

    async fn get_feedback(&self, id: Uuid) -> AtlasResult<Option<PerformanceFeedback>> {
        let row = sqlx::query("SELECT * FROM _atlas.performance_feedback WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_feedback(&r)))
    }

    async fn list_feedback(&self, org_id: Uuid, employee_id: Option<Uuid>, document_id: Option<Uuid>) -> AtlasResult<Vec<PerformanceFeedback>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.performance_feedback
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR employee_id = $2)
              AND ($3::uuid IS NULL OR document_id = $3)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(employee_id).bind(document_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_feedback).collect())
    }

    async fn update_feedback_status(&self, id: Uuid, status: &str) -> AtlasResult<PerformanceFeedback> {
        let row = sqlx::query(
            "UPDATE _atlas.performance_feedback SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_feedback(&row))
    }
}

// ========================================================================
// Row mapping helpers
// ========================================================================

fn row_to_cycle(row: &sqlx::postgres::PgRow) -> PerformanceReviewCycle {
    PerformanceReviewCycle {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        name: row.get("name"),
        description: row.get("description"),
        cycle_type: row.get("cycle_type"),
        status: row.get("status"),
        rating_model_id: row.get("rating_model_id"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        goal_setting_start: row.get("goal_setting_start"),
        goal_setting_end: row.get("goal_setting_end"),
        self_evaluation_start: row.get("self_evaluation_start"),
        self_evaluation_end: row.get("self_evaluation_end"),
        manager_evaluation_start: row.get("manager_evaluation_start"),
        manager_evaluation_end: row.get("manager_evaluation_end"),
        calibration_date: row.get("calibration_date"),
        require_goals: row.get("require_goals"),
        require_competencies: row.get("require_competencies"),
        min_goals: row.get("min_goals"),
        max_goals: row.get("max_goals"),
        goal_weight_total: num_to_str(row, "goal_weight_total"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_doc(row: &sqlx::postgres::PgRow) -> PerformanceDocument {
    fn opt_num(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
        let v: Option<serde_json::Value> = row.try_get(col).ok();
        v.map(|val| val.to_string())
    }
    PerformanceDocument {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        review_cycle_id: row.get("review_cycle_id"),
        employee_id: row.get("employee_id"),
        employee_name: row.get("employee_name"),
        manager_id: row.get("manager_id"),
        manager_name: row.get("manager_name"),
        document_number: row.get("document_number"),
        status: row.get("status"),
        overall_rating: opt_num(row, "overall_rating"),
        overall_rating_label: row.get("overall_rating_label"),
        self_overall_rating: opt_num(row, "self_overall_rating"),
        self_comments: row.get("self_comments"),
        manager_overall_rating: opt_num(row, "manager_overall_rating"),
        manager_comments: row.get("manager_comments"),
        calibration_rating: opt_num(row, "calibration_rating"),
        calibration_comments: row.get("calibration_comments"),
        final_rating: opt_num(row, "final_rating"),
        final_comments: row.get("final_comments"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_goal(row: &sqlx::postgres::PgRow) -> PerformanceGoal {
    fn opt_num(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
        let v: Option<serde_json::Value> = row.try_get(col).ok();
        v.map(|val| val.to_string())
    }
    PerformanceGoal {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        document_id: row.get("document_id"),
        employee_id: row.get("employee_id"),
        goal_name: row.get("goal_name"),
        description: row.get("description"),
        goal_category: row.get("goal_category"),
        status: row.get("status"),
        weight: num_to_str(row, "weight"),
        target_metric: row.get("target_metric"),
        actual_result: row.get("actual_result"),
        self_rating: opt_num(row, "self_rating"),
        self_comments: row.get("self_comments"),
        manager_rating: opt_num(row, "manager_rating"),
        manager_comments: row.get("manager_comments"),
        start_date: row.get("start_date"),
        due_date: row.get("due_date"),
        completed_date: row.get("completed_date"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_assessment(row: &sqlx::postgres::PgRow) -> CompetencyAssessment {
    fn opt_num(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
        let v: Option<serde_json::Value> = row.try_get(col).ok();
        v.map(|val| val.to_string())
    }
    CompetencyAssessment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        document_id: row.get("document_id"),
        employee_id: row.get("employee_id"),
        competency_id: row.get("competency_id"),
        self_rating: opt_num(row, "self_rating"),
        self_comments: row.get("self_comments"),
        manager_rating: opt_num(row, "manager_rating"),
        manager_comments: row.get("manager_comments"),
        calibration_rating: opt_num(row, "calibration_rating"),
        calibration_comments: row.get("calibration_comments"),
        final_rating: opt_num(row, "final_rating"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_feedback(row: &sqlx::postgres::PgRow) -> PerformanceFeedback {
    PerformanceFeedback {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        document_id: row.get("document_id"),
        employee_id: row.get("employee_id"),
        from_user_id: row.get("from_user_id"),
        from_user_name: row.get("from_user_name"),
        feedback_type: row.get("feedback_type"),
        subject: row.get("subject"),
        content: row.get("content"),
        visibility: row.get("visibility"),
        status: row.get("status"),
        acknowledged_at: row.get("acknowledged_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
