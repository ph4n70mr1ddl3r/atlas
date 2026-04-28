//! Succession Planning Repository
//!
//! PostgreSQL storage for succession plans, candidates, talent pools,
//! pool members, talent reviews, assessments, and career paths.

use atlas_shared::{
    SuccessionPlan, SuccessionCandidate, TalentPool, TalentPoolMember,
    TalentReview, TalentReviewAssessment, CareerPath, SuccessionDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Succession Planning data storage
#[async_trait]
pub trait SuccessionPlanningRepository: Send + Sync {
    // ========================================================================
    // Succession Plans
    // ========================================================================
    async fn create_succession_plan(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        plan_type: &str, position_id: Option<Uuid>, position_title: Option<&str>,
        job_id: Option<Uuid>, department_id: Option<Uuid>,
        current_incumbent_id: Option<Uuid>, current_incumbent_name: Option<&str>,
        risk_level: &str, urgency: &str, effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionPlan>;
    async fn get_succession_plan(&self, id: Uuid) -> AtlasResult<Option<SuccessionPlan>>;
    async fn get_succession_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SuccessionPlan>>;
    async fn list_succession_plans(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>,
    ) -> AtlasResult<Vec<SuccessionPlan>>;
    async fn update_succession_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionPlan>;
    async fn delete_succession_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Succession Candidates
    // ========================================================================
    async fn create_succession_candidate(
        &self,
        org_id: Uuid, plan_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        employee_number: Option<&str>, readiness: &str, ranking: Option<i32>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        flight_risk: Option<&str>, development_notes: Option<&str>,
        recommended_actions: Option<&str>, status: &str, added_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionCandidate>;
    async fn get_succession_candidate(&self, id: Uuid) -> AtlasResult<Option<SuccessionCandidate>>;
    async fn list_succession_candidates(&self, plan_id: Uuid) -> AtlasResult<Vec<SuccessionCandidate>>;
    async fn update_candidate_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionCandidate>;
    async fn update_candidate_readiness(&self, id: Uuid, readiness: &str) -> AtlasResult<SuccessionCandidate>;
    async fn delete_succession_candidate(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Talent Pools
    // ========================================================================
    async fn create_talent_pool(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_type: &str, owner_id: Option<Uuid>, max_members: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentPool>;
    async fn get_talent_pool(&self, id: Uuid) -> AtlasResult<Option<TalentPool>>;
    async fn get_talent_pool_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TalentPool>>;
    async fn list_talent_pools(
        &self, org_id: Uuid, status: Option<&str>, pool_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentPool>>;
    async fn update_talent_pool_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPool>;
    async fn delete_talent_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Talent Pool Members
    // ========================================================================
    async fn create_talent_pool_member(
        &self,
        org_id: Uuid, pool_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        readiness: &str, development_plan: Option<&str>, notes: Option<&str>,
        added_date: Option<chrono::NaiveDate>, review_date: Option<chrono::NaiveDate>,
        added_by: Option<Uuid>,
    ) -> AtlasResult<TalentPoolMember>;
    async fn get_talent_pool_member(&self, id: Uuid) -> AtlasResult<Option<TalentPoolMember>>;
    async fn list_talent_pool_members(&self, pool_id: Uuid) -> AtlasResult<Vec<TalentPoolMember>>;
    async fn update_pool_member_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPoolMember>;
    async fn delete_talent_pool_member(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Talent Reviews
    // ========================================================================
    async fn create_talent_review(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        review_type: &str, facilitator_id: Option<Uuid>,
        department_id: Option<Uuid>, review_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentReview>;
    async fn get_talent_review(&self, id: Uuid) -> AtlasResult<Option<TalentReview>>;
    async fn get_talent_review_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TalentReview>>;
    async fn list_talent_reviews(
        &self, org_id: Uuid, status: Option<&str>, review_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentReview>>;
    async fn update_talent_review_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentReview>;
    async fn delete_talent_review(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Talent Review Assessments
    // ========================================================================
    async fn create_talent_review_assessment(
        &self,
        org_id: Uuid, review_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        nine_box_position: Option<&str>, strengths: Option<&str>,
        weaknesses: Option<&str>, career_aspiration: Option<&str>,
        development_needs: Option<&str>, succession_readiness: Option<&str>,
        assessor_id: Option<Uuid>, notes: Option<&str>,
    ) -> AtlasResult<TalentReviewAssessment>;
    async fn get_talent_review_assessment(&self, id: Uuid) -> AtlasResult<Option<TalentReviewAssessment>>;
    async fn list_talent_review_assessments(&self, review_id: Uuid) -> AtlasResult<Vec<TalentReviewAssessment>>;
    async fn delete_talent_review_assessment(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Career Paths
    // ========================================================================
    async fn create_career_path(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        path_type: &str, from_job_id: Option<Uuid>, from_job_title: Option<&str>,
        to_job_id: Option<Uuid>, to_job_title: Option<&str>,
        typical_duration_months: Option<i32>,
        required_competencies: Option<&str>, required_certifications: Option<&str>,
        development_activities: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CareerPath>;
    async fn get_career_path(&self, id: Uuid) -> AtlasResult<Option<CareerPath>>;
    async fn get_career_path_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CareerPath>>;
    async fn list_career_paths(
        &self, org_id: Uuid, status: Option<&str>, path_type: Option<&str>,
    ) -> AtlasResult<Vec<CareerPath>>;
    async fn update_career_path_status(&self, id: Uuid, status: &str) -> AtlasResult<CareerPath>;
    async fn delete_career_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Dashboard
    // ========================================================================
    async fn get_succession_dashboard(&self, org_id: Uuid) -> AtlasResult<SuccessionDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresSuccessionPlanningRepository {
    pool: PgPool,
}

impl PostgresSuccessionPlanningRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row mapping helpers

fn row_to_plan(row: &sqlx::postgres::PgRow) -> SuccessionPlan {
    SuccessionPlan {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        plan_type: row.try_get("plan_type").unwrap_or_else(|_| "position".to_string()),
        position_id: row.try_get("position_id").unwrap_or_default(),
        position_title: row.try_get("position_title").unwrap_or_default(),
        job_id: row.try_get("job_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        current_incumbent_id: row.try_get("current_incumbent_id").unwrap_or_default(),
        current_incumbent_name: row.try_get("current_incumbent_name").unwrap_or_default(),
        risk_level: row.try_get("risk_level").unwrap_or_else(|_| "medium".to_string()),
        urgency: row.try_get("urgency").unwrap_or_else(|_| "medium_term".to_string()),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        effective_date: row.try_get("effective_date").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_candidate(row: &sqlx::postgres::PgRow) -> SuccessionCandidate {
    SuccessionCandidate {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        plan_id: row.try_get("plan_id").unwrap_or_default(),
        person_id: row.try_get("person_id").unwrap_or_default(),
        person_name: row.try_get("person_name").unwrap_or_default(),
        employee_number: row.try_get("employee_number").unwrap_or_default(),
        readiness: row.try_get("readiness").unwrap_or_else(|_| "not_ready".to_string()),
        ranking: row.try_get("ranking").unwrap_or_default(),
        performance_rating: row.try_get("performance_rating").unwrap_or_default(),
        potential_rating: row.try_get("potential_rating").unwrap_or_default(),
        flight_risk: row.try_get("flight_risk").unwrap_or_default(),
        development_notes: row.try_get("development_notes").unwrap_or_default(),
        recommended_actions: row.try_get("recommended_actions").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "proposed".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        added_by: row.try_get("added_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_pool(row: &sqlx::postgres::PgRow) -> TalentPool {
    TalentPool {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        pool_type: row.try_get("pool_type").unwrap_or_else(|_| "high_potential".to_string()),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        max_members: row.try_get("max_members").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_pool_member(row: &sqlx::postgres::PgRow) -> TalentPoolMember {
    TalentPoolMember {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        pool_id: row.try_get("pool_id").unwrap_or_default(),
        person_id: row.try_get("person_id").unwrap_or_default(),
        person_name: row.try_get("person_name").unwrap_or_default(),
        performance_rating: row.try_get("performance_rating").unwrap_or_default(),
        potential_rating: row.try_get("potential_rating").unwrap_or_default(),
        readiness: row.try_get("readiness").unwrap_or_else(|_| "not_ready".to_string()),
        development_plan: row.try_get("development_plan").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        added_date: row.try_get("added_date").unwrap_or_default(),
        review_date: row.try_get("review_date").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        added_by: row.try_get("added_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_review(row: &sqlx::postgres::PgRow) -> TalentReview {
    TalentReview {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        review_type: row.try_get("review_type").unwrap_or_else(|_| "nine_box".to_string()),
        facilitator_id: row.try_get("facilitator_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        review_date: row.try_get("review_date").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "scheduled".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_assessment(row: &sqlx::postgres::PgRow) -> TalentReviewAssessment {
    TalentReviewAssessment {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        review_id: row.try_get("review_id").unwrap_or_default(),
        person_id: row.try_get("person_id").unwrap_or_default(),
        person_name: row.try_get("person_name").unwrap_or_default(),
        performance_rating: row.try_get("performance_rating").unwrap_or_default(),
        potential_rating: row.try_get("potential_rating").unwrap_or_default(),
        nine_box_position: row.try_get("nine_box_position").unwrap_or_default(),
        strengths: row.try_get("strengths").unwrap_or_default(),
        weaknesses: row.try_get("weaknesses").unwrap_or_default(),
        career_aspiration: row.try_get("career_aspiration").unwrap_or_default(),
        development_needs: row.try_get("development_needs").unwrap_or_default(),
        succession_readiness: row.try_get("succession_readiness").unwrap_or_default(),
        assessor_id: row.try_get("assessor_id").unwrap_or_default(),
        notes: row.try_get("notes").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_career_path(row: &sqlx::postgres::PgRow) -> CareerPath {
    CareerPath {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        path_type: row.try_get("path_type").unwrap_or_else(|_| "linear".to_string()),
        from_job_id: row.try_get("from_job_id").unwrap_or_default(),
        from_job_title: row.try_get("from_job_title").unwrap_or_default(),
        to_job_id: row.try_get("to_job_id").unwrap_or_default(),
        to_job_title: row.try_get("to_job_title").unwrap_or_default(),
        typical_duration_months: row.try_get("typical_duration_months").unwrap_or_default(),
        required_competencies: row.try_get("required_competencies").unwrap_or_default(),
        required_certifications: row.try_get("required_certifications").unwrap_or_default(),
        development_activities: row.try_get("development_activities").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl SuccessionPlanningRepository for PostgresSuccessionPlanningRepository {
    // ========================================================================
    // Succession Plans
    // ========================================================================

    async fn create_succession_plan(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        plan_type: &str, position_id: Option<Uuid>, position_title: Option<&str>,
        job_id: Option<Uuid>, department_id: Option<Uuid>,
        current_incumbent_id: Option<Uuid>, current_incumbent_name: Option<&str>,
        risk_level: &str, urgency: &str, effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionPlan> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.succession_plans
                (organization_id, code, name, description, plan_type,
                 position_id, position_title, job_id, department_id,
                 current_incumbent_id, current_incumbent_name,
                 risk_level, urgency, effective_date, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, '{}'::jsonb, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(plan_type)
        .bind(position_id).bind(position_title).bind(job_id).bind(department_id)
        .bind(current_incumbent_id).bind(current_incumbent_name)
        .bind(risk_level).bind(urgency).bind(effective_date).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_plan(&row))
    }

    async fn get_succession_plan(&self, id: Uuid) -> AtlasResult<Option<SuccessionPlan>> {
        let row = sqlx::query("SELECT * FROM _atlas.succession_plans WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_plan))
    }

    async fn get_succession_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SuccessionPlan>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.succession_plans WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_plan))
    }

    async fn list_succession_plans(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>,
    ) -> AtlasResult<Vec<SuccessionPlan>> {
        let mut query = String::from("SELECT * FROM _atlas.succession_plans WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_risk = risk_level.is_some();
        if has_risk { query.push_str(&format!(" AND risk_level = ${}", bind_idx)); }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(r) = risk_level { q = q.bind(r); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_plan).collect())
    }

    async fn update_succession_plan_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionPlan> {
        let row = sqlx::query(
            "UPDATE _atlas.succession_plans SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_plan(&row))
    }

    async fn delete_succession_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.succession_plans WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Succession plan '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Succession Candidates
    // ========================================================================

    async fn create_succession_candidate(
        &self, org_id: Uuid, plan_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        employee_number: Option<&str>, readiness: &str, ranking: Option<i32>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        flight_risk: Option<&str>, development_notes: Option<&str>,
        recommended_actions: Option<&str>, status: &str, added_by: Option<Uuid>,
    ) -> AtlasResult<SuccessionCandidate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.succession_candidates
                (organization_id, plan_id, person_id, person_name, employee_number,
                 readiness, ranking, performance_rating, potential_rating,
                 flight_risk, development_notes, recommended_actions,
                 status, metadata, added_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, '{}'::jsonb, $14)
            RETURNING *"#,
        )
        .bind(org_id).bind(plan_id).bind(person_id).bind(person_name)
        .bind(employee_number).bind(readiness).bind(ranking)
        .bind(performance_rating).bind(potential_rating)
        .bind(flight_risk).bind(development_notes).bind(recommended_actions)
        .bind(status).bind(added_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_candidate(&row))
    }

    async fn get_succession_candidate(&self, id: Uuid) -> AtlasResult<Option<SuccessionCandidate>> {
        let row = sqlx::query("SELECT * FROM _atlas.succession_candidates WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_candidate))
    }

    async fn list_succession_candidates(&self, plan_id: Uuid) -> AtlasResult<Vec<SuccessionCandidate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.succession_candidates WHERE plan_id = $1 ORDER BY ranking NULLS LAST, created_at"
        ).bind(plan_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_candidate).collect())
    }

    async fn update_candidate_status(&self, id: Uuid, status: &str) -> AtlasResult<SuccessionCandidate> {
        let row = sqlx::query(
            "UPDATE _atlas.succession_candidates SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_candidate(&row))
    }

    async fn update_candidate_readiness(&self, id: Uuid, readiness: &str) -> AtlasResult<SuccessionCandidate> {
        let row = sqlx::query(
            "UPDATE _atlas.succession_candidates SET readiness = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(readiness).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_candidate(&row))
    }

    async fn delete_succession_candidate(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.succession_candidates WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Candidate not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Talent Pools
    // ========================================================================

    async fn create_talent_pool(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        pool_type: &str, owner_id: Option<Uuid>, max_members: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentPool> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.talent_pools
                (organization_id, code, name, description, pool_type,
                 owner_id, max_members, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, '{}'::jsonb, $8)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(pool_type)
        .bind(owner_id).bind(max_members).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_pool(&row))
    }

    async fn get_talent_pool(&self, id: Uuid) -> AtlasResult<Option<TalentPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.talent_pools WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_pool))
    }

    async fn get_talent_pool_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TalentPool>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.talent_pools WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_pool))
    }

    async fn list_talent_pools(
        &self, org_id: Uuid, status: Option<&str>, pool_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentPool>> {
        let mut query = String::from("SELECT * FROM _atlas.talent_pools WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_type = pool_type.is_some();
        if has_type { query.push_str(&format!(" AND pool_type = ${}", bind_idx)); }
        query.push_str(" ORDER BY name");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(pt) = pool_type { q = q.bind(pt); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_pool).collect())
    }

    async fn update_talent_pool_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPool> {
        let row = sqlx::query(
            "UPDATE _atlas.talent_pools SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_pool(&row))
    }

    async fn delete_talent_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.talent_pools WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Talent pool '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Talent Pool Members
    // ========================================================================

    async fn create_talent_pool_member(
        &self, org_id: Uuid, pool_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        readiness: &str, development_plan: Option<&str>, notes: Option<&str>,
        added_date: Option<chrono::NaiveDate>, review_date: Option<chrono::NaiveDate>,
        added_by: Option<Uuid>,
    ) -> AtlasResult<TalentPoolMember> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.talent_pool_members
                (organization_id, pool_id, person_id, person_name,
                 performance_rating, potential_rating, readiness,
                 development_plan, notes, added_date, review_date,
                 metadata, added_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    '{}'::jsonb, $12)
            RETURNING *"#,
        )
        .bind(org_id).bind(pool_id).bind(person_id).bind(person_name)
        .bind(performance_rating).bind(potential_rating).bind(readiness)
        .bind(development_plan).bind(notes).bind(added_date).bind(review_date)
        .bind(added_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_pool_member(&row))
    }

    async fn get_talent_pool_member(&self, id: Uuid) -> AtlasResult<Option<TalentPoolMember>> {
        let row = sqlx::query("SELECT * FROM _atlas.talent_pool_members WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_pool_member))
    }

    async fn list_talent_pool_members(&self, pool_id: Uuid) -> AtlasResult<Vec<TalentPoolMember>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.talent_pool_members WHERE pool_id = $1 ORDER BY created_at"
        ).bind(pool_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_pool_member).collect())
    }

    async fn update_pool_member_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentPoolMember> {
        let row = sqlx::query(
            "UPDATE _atlas.talent_pool_members SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_pool_member(&row))
    }

    async fn delete_talent_pool_member(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.talent_pool_members WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Pool member not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Talent Reviews
    // ========================================================================

    async fn create_talent_review(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        review_type: &str, facilitator_id: Option<Uuid>,
        department_id: Option<Uuid>, review_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TalentReview> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.talent_reviews
                (organization_id, code, name, description, review_type,
                 facilitator_id, department_id, review_date, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}'::jsonb, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(review_type)
        .bind(facilitator_id).bind(department_id).bind(review_date).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_review(&row))
    }

    async fn get_talent_review(&self, id: Uuid) -> AtlasResult<Option<TalentReview>> {
        let row = sqlx::query("SELECT * FROM _atlas.talent_reviews WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_review))
    }

    async fn get_talent_review_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TalentReview>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.talent_reviews WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_review))
    }

    async fn list_talent_reviews(
        &self, org_id: Uuid, status: Option<&str>, review_type: Option<&str>,
    ) -> AtlasResult<Vec<TalentReview>> {
        let mut query = String::from("SELECT * FROM _atlas.talent_reviews WHERE organization_id = $1");
        let mut bind_idx = 2u32;
        let has_status = status.is_some();
        if has_status { query.push_str(&format!(" AND status = ${}", bind_idx)); bind_idx += 1; }
        let has_type = review_type.is_some();
        if has_type { query.push_str(&format!(" AND review_type = ${}", bind_idx)); }
        query.push_str(" ORDER BY review_date DESC NULLS LAST, created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(rt) = review_type { q = q.bind(rt); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_review).collect())
    }

    async fn update_talent_review_status(&self, id: Uuid, status: &str) -> AtlasResult<TalentReview> {
        let row = sqlx::query(
            "UPDATE _atlas.talent_reviews SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_review(&row))
    }

    async fn delete_talent_review(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.talent_reviews WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Talent review '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Talent Review Assessments
    // ========================================================================

    async fn create_talent_review_assessment(
        &self, org_id: Uuid, review_id: Uuid, person_id: Uuid, person_name: Option<&str>,
        performance_rating: Option<&str>, potential_rating: Option<&str>,
        nine_box_position: Option<&str>, strengths: Option<&str>,
        weaknesses: Option<&str>, career_aspiration: Option<&str>,
        development_needs: Option<&str>, succession_readiness: Option<&str>,
        assessor_id: Option<Uuid>, notes: Option<&str>,
    ) -> AtlasResult<TalentReviewAssessment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.talent_review_assessments
                (organization_id, review_id, person_id, person_name,
                 performance_rating, potential_rating, nine_box_position,
                 strengths, weaknesses, career_aspiration, development_needs,
                 succession_readiness, assessor_id, notes, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12, $13, $14, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(review_id).bind(person_id).bind(person_name)
        .bind(performance_rating).bind(potential_rating).bind(nine_box_position)
        .bind(strengths).bind(weaknesses).bind(career_aspiration)
        .bind(development_needs).bind(succession_readiness)
        .bind(assessor_id).bind(notes)
        .fetch_one(&self.pool).await?;
        Ok(row_to_assessment(&row))
    }

    async fn get_talent_review_assessment(&self, id: Uuid) -> AtlasResult<Option<TalentReviewAssessment>> {
        let row = sqlx::query("SELECT * FROM _atlas.talent_review_assessments WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_assessment))
    }

    async fn list_talent_review_assessments(&self, review_id: Uuid) -> AtlasResult<Vec<TalentReviewAssessment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.talent_review_assessments WHERE review_id = $1 ORDER BY created_at"
        ).bind(review_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_assessment).collect())
    }

    async fn delete_talent_review_assessment(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.talent_review_assessments WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Assessment not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Career Paths
    // ========================================================================

    async fn create_career_path(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        path_type: &str, from_job_id: Option<Uuid>, from_job_title: Option<&str>,
        to_job_id: Option<Uuid>, to_job_title: Option<&str>,
        typical_duration_months: Option<i32>,
        required_competencies: Option<&str>, required_certifications: Option<&str>,
        development_activities: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CareerPath> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.career_paths
                (organization_id, code, name, description, path_type,
                 from_job_id, from_job_title, to_job_id, to_job_title,
                 typical_duration_months, required_competencies,
                 required_certifications, development_activities,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, '{}'::jsonb, $14)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(path_type)
        .bind(from_job_id).bind(from_job_title).bind(to_job_id).bind(to_job_title)
        .bind(typical_duration_months).bind(required_competencies)
        .bind(required_certifications).bind(development_activities)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_career_path(&row))
    }

    async fn get_career_path(&self, id: Uuid) -> AtlasResult<Option<CareerPath>> {
        let row = sqlx::query("SELECT * FROM _atlas.career_paths WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_career_path))
    }

    async fn get_career_path_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CareerPath>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.career_paths WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_career_path))
    }

    async fn list_career_paths(
        &self, org_id: Uuid, status: Option<&str>, path_type: Option<&str>,
    ) -> AtlasResult<Vec<CareerPath>> {
        let mut query = String::from("SELECT * FROM _atlas.career_paths WHERE organization_id = $1");
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
        Ok(rows.iter().map(row_to_career_path).collect())
    }

    async fn update_career_path_status(&self, id: Uuid, status: &str) -> AtlasResult<CareerPath> {
        let row = sqlx::query(
            "UPDATE _atlas.career_paths SET status = $1, updated_at = now() WHERE id = $2 RETURNING *"
        ).bind(status).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_career_path(&row))
    }

    async fn delete_career_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.career_paths WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Career path '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_succession_dashboard(&self, org_id: Uuid) -> AtlasResult<SuccessionDashboard> {
        // Plans by risk
        let risk_rows = sqlx::query(
            "SELECT risk_level, COUNT(*) as cnt FROM _atlas.succession_plans WHERE organization_id = $1 GROUP BY risk_level"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let mut plans_by_risk = serde_json::Map::new();
        let mut total_plans = 0i32;
        for r in &risk_rows {
            let risk: String = r.try_get("risk_level").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            total_plans += cnt as i32;
            plans_by_risk.insert(risk, serde_json::json!(cnt));
        }

        // Active plans count
        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.succession_plans WHERE organization_id = $1 AND status = 'active'"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);
        let active_plans = active_count as i32;

        // Plans by urgency
        let urgency_rows = sqlx::query(
            "SELECT urgency, COUNT(*) as cnt FROM _atlas.succession_plans WHERE organization_id = $1 GROUP BY urgency"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let mut plans_by_urgency = serde_json::Map::new();
        for r in &urgency_rows {
            let urgency: String = r.try_get("urgency").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            plans_by_urgency.insert(urgency, serde_json::json!(cnt));
        }

        // Total candidates + by readiness
        let total_candidates: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.succession_candidates WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let readiness_rows = sqlx::query(
            "SELECT readiness, COUNT(*) as cnt FROM _atlas.succession_candidates WHERE organization_id = $1 GROUP BY readiness"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();
        let mut candidates_by_readiness = serde_json::Map::new();
        for r in &readiness_rows {
            let readiness: String = r.try_get("readiness").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            candidates_by_readiness.insert(readiness, serde_json::json!(cnt));
        }

        // Talent pools
        let total_talent_pools: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.talent_pools WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let total_pool_members: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.talent_pool_members WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        // Reviews
        let total_reviews: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.talent_reviews WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        // Career paths
        let total_career_paths: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.career_paths WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        // Coverage %: plans with at least 1 ready_now candidate / total active plans
        let covered: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(DISTINCT sp.id) FROM _atlas.succession_plans sp
               JOIN _atlas.succession_candidates sc ON sc.plan_id = sp.id
               WHERE sp.organization_id = $1 AND sp.status = 'active'
                 AND sc.readiness = 'ready_now' AND sc.status = 'approved'"#
        ).bind(org_id).fetch_one(&self.pool).await.unwrap_or(0);

        let coverage_pct = if active_plans > 0 {
            Some(format!("{:.1}", (covered as f64 / active_plans as f64) * 100.0))
        } else {
            None
        };

        Ok(SuccessionDashboard {
            total_succession_plans: total_plans,
            active_plans,
            plans_by_risk: serde_json::Value::Object(plans_by_risk),
            plans_by_urgency: serde_json::Value::Object(plans_by_urgency),
            total_candidates: total_candidates as i32,
            candidates_by_readiness: serde_json::Value::Object(candidates_by_readiness),
            total_talent_pools: total_talent_pools as i32,
            total_pool_members: total_pool_members as i32,
            total_reviews: total_reviews as i32,
            total_career_paths: total_career_paths as i32,
            coverage_pct,
        })
    }
}
