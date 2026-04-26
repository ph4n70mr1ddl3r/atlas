//! Supplier Scorecard Repository
//!
//! PostgreSQL storage for supplier scorecard data.

use atlas_shared::{
    ScorecardTemplate, ScorecardCategory, SupplierScorecard, ScorecardLine,
    SupplierPerformanceReview, ReviewActionItem, SupplierScorecardDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[async_trait]
pub trait ScorecardRepository: Send + Sync {
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, evaluation_period: &str, created_by: Option<Uuid>) -> AtlasResult<ScorecardTemplate>;
    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ScorecardTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ScorecardTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<ScorecardTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    async fn create_category(&self, org_id: Uuid, template_id: Uuid, code: &str, name: &str, description: Option<&str>, weight: &str, sort_order: i32, scoring_model: &str, target_score: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<ScorecardCategory>;
    async fn list_categories(&self, template_id: Uuid) -> AtlasResult<Vec<ScorecardCategory>>;
    async fn delete_category(&self, id: Uuid) -> AtlasResult<()>;

    async fn create_scorecard(&self, org_id: Uuid, template_id: Uuid, scorecard_number: &str, supplier_id: Uuid, supplier_name: Option<&str>, supplier_number: Option<&str>, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, notes: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<SupplierScorecard>;
    async fn get_scorecard(&self, id: Uuid) -> AtlasResult<Option<SupplierScorecard>>;
    async fn get_scorecard_by_number(&self, org_id: Uuid, scorecard_number: &str) -> AtlasResult<Option<SupplierScorecard>>;
    async fn list_scorecards(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierScorecard>>;
    async fn update_scorecard_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierScorecard>;
    async fn submit_scorecard(&self, id: Uuid, overall_score: &str, overall_grade: &str, reviewer_id: Option<Uuid>, reviewer_name: Option<&str>) -> AtlasResult<SupplierScorecard>;
    async fn approve_scorecard(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<SupplierScorecard>;
    async fn delete_scorecard(&self, org_id: Uuid, scorecard_number: &str) -> AtlasResult<()>;

    async fn add_scorecard_line(&self, org_id: Uuid, scorecard_id: Uuid, category_id: Uuid, line_number: i32, kpi_name: &str, kpi_description: Option<&str>, weight: &str, target_value: Option<&str>, actual_value: Option<&str>, score: &str, weighted_score: &str, evidence: Option<&str>, notes: Option<&str>) -> AtlasResult<ScorecardLine>;
    async fn list_scorecard_lines(&self, scorecard_id: Uuid) -> AtlasResult<Vec<ScorecardLine>>;
    async fn delete_scorecard_line(&self, id: Uuid) -> AtlasResult<()>;

    async fn create_review(&self, org_id: Uuid, review_number: &str, supplier_id: Uuid, supplier_name: Option<&str>, scorecard_id: Option<Uuid>, review_type: &str, review_period: Option<&str>, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, created_by: Option<Uuid>) -> AtlasResult<SupplierPerformanceReview>;
    async fn get_review(&self, id: Uuid) -> AtlasResult<Option<SupplierPerformanceReview>>;
    async fn get_review_by_number(&self, org_id: Uuid, review_number: &str) -> AtlasResult<Option<SupplierPerformanceReview>>;
    async fn list_reviews(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierPerformanceReview>>;
    async fn update_review_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierPerformanceReview>;
    async fn complete_review(&self, id: Uuid, current_score: Option<&str>, rating: Option<&str>, strengths: Option<&str>, improvement_areas: Option<&str>, action_items: Option<&str>, previous_score: Option<&str>, score_change: Option<&str>, reviewer_id: Option<Uuid>, reviewer_name: Option<&str>) -> AtlasResult<SupplierPerformanceReview>;
    async fn delete_review(&self, org_id: Uuid, review_number: &str) -> AtlasResult<()>;

    async fn create_action_item(&self, org_id: Uuid, review_id: Uuid, action_number: i32, description: &str, assignee_id: Option<Uuid>, assignee_name: Option<&str>, priority: &str, due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<ReviewActionItem>;
    async fn get_action_item(&self, id: Uuid) -> AtlasResult<Option<ReviewActionItem>>;
    async fn list_action_items(&self, review_id: Uuid) -> AtlasResult<Vec<ReviewActionItem>>;
    async fn complete_action_item(&self, id: Uuid) -> AtlasResult<ReviewActionItem>;
    async fn delete_action_item(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SupplierScorecardDashboard>;
}

pub struct PostgresScorecardRepository { pool: PgPool }

impl PostgresScorecardRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> ScorecardTemplate {
    ScorecardTemplate {
        id: row.get("id"), organization_id: row.get("organization_id"),
        code: row.get("code"), name: row.get("name"),
        description: row.get("description"), evaluation_period: row.get("evaluation_period"),
        is_active: row.get("is_active"), total_weight: get_num(row, "total_weight"),
        metadata: row.get("metadata"), created_by: row.get("created_by"),
        created_at: row.get("created_at"), updated_at: row.get("updated_at"),
    }
}

fn row_to_category(row: &sqlx::postgres::PgRow) -> ScorecardCategory {
    ScorecardCategory {
        id: row.get("id"), organization_id: row.get("organization_id"),
        template_id: row.get("template_id"), code: row.get("code"), name: row.get("name"),
        description: row.get("description"), weight: get_num(row, "weight"),
        sort_order: row.get("sort_order"), scoring_model: row.get("scoring_model"),
        target_score: row.get::<Option<f64>, _>("target_score").map(|v| format!("{:.2}", v)), metadata: row.get("metadata"),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_scorecard(row: &sqlx::postgres::PgRow) -> SupplierScorecard {
    SupplierScorecard {
        id: row.get("id"), organization_id: row.get("organization_id"),
        template_id: row.get("template_id"), scorecard_number: row.get("scorecard_number"),
        supplier_id: row.get("supplier_id"), supplier_name: row.get("supplier_name"),
        supplier_number: row.get("supplier_number"),
        evaluation_period_start: row.get("evaluation_period_start"),
        evaluation_period_end: row.get("evaluation_period_end"),
        status: row.get("status"), overall_score: get_num(row, "overall_score"),
        overall_grade: row.get("overall_grade"), reviewer_id: row.get("reviewer_id"),
        reviewer_name: row.get("reviewer_name"), review_date: row.get("review_date"),
        approved_by: row.get("approved_by"), approved_at: row.get("approved_at"),
        notes: row.get("notes"), metadata: row.get("metadata"),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_line(row: &sqlx::postgres::PgRow) -> ScorecardLine {
    ScorecardLine {
        id: row.get("id"), organization_id: row.get("organization_id"),
        scorecard_id: row.get("scorecard_id"), category_id: row.get("category_id"),
        line_number: row.get("line_number"), kpi_name: row.get("kpi_name"),
        kpi_description: row.get("kpi_description"), weight: get_num(row, "weight"),
        target_value: row.get::<Option<f64>, _>("target_value").map(|v| format!("{:.2}", v)), actual_value: row.get::<Option<f64>, _>("actual_value").map(|v| format!("{:.2}", v)),
        score: get_num(row, "score"), weighted_score: get_num(row, "weighted_score"),
        evidence: row.get("evidence"), notes: row.get("notes"),
        metadata: row.get("metadata"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_review(row: &sqlx::postgres::PgRow) -> SupplierPerformanceReview {
    SupplierPerformanceReview {
        id: row.get("id"), organization_id: row.get("organization_id"),
        review_number: row.get("review_number"), supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"), scorecard_id: row.get("scorecard_id"),
        review_type: row.get("review_type"), review_period: row.get("review_period"),
        period_start: row.get("period_start"), period_end: row.get("period_end"),
        previous_score: row.get::<Option<f64>, _>("previous_score").map(|v| format!("{:.2}", v)), current_score: row.get::<Option<f64>, _>("current_score").map(|v| format!("{:.2}", v)),
        score_change: row.get::<Option<f64>, _>("score_change").map(|v| format!("{:.2}", v)), rating: row.get("rating"),
        strengths: row.get("strengths"), improvement_areas: row.get("improvement_areas"),
        action_items: row.get("action_items"), follow_up_date: row.get("follow_up_date"),
        status: row.get("status"), reviewer_id: row.get("reviewer_id"),
        reviewer_name: row.get("reviewer_name"), completed_at: row.get("completed_at"),
        metadata: row.get("metadata"), created_by: row.get("created_by"),
        created_at: row.get("created_at"), updated_at: row.get("updated_at"),
    }
}

fn row_to_action_item(row: &sqlx::postgres::PgRow) -> ReviewActionItem {
    ReviewActionItem {
        id: row.get("id"), organization_id: row.get("organization_id"),
        review_id: row.get("review_id"), action_number: row.get("action_number"),
        description: row.get("description"), assignee_id: row.get("assignee_id"),
        assignee_name: row.get("assignee_name"), priority: row.get("priority"),
        due_date: row.get("due_date"), status: row.get("status"),
        completed_at: row.get("completed_at"), notes: row.get("notes"),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

macro_rules! db_err {
    ($e:expr) => { atlas_shared::AtlasError::DatabaseError($e.to_string()) };
}

#[async_trait]
impl ScorecardRepository for PostgresScorecardRepository {
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, evaluation_period: &str, created_by: Option<Uuid>) -> AtlasResult<ScorecardTemplate> {
        let row = sqlx::query(
            "INSERT INTO _atlas.scorecard_templates (organization_id, code, name, description, evaluation_period, created_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"
        ).bind(org_id).bind(code).bind(name).bind(description).bind(evaluation_period).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ScorecardTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.scorecard_templates WHERE id = $1 AND is_active = true")
            .bind(id).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ScorecardTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.scorecard_templates WHERE organization_id = $1 AND code = $2 AND is_active = true")
            .bind(org_id).bind(code).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<ScorecardTemplate>> {
        let rows = sqlx::query("SELECT * FROM _atlas.scorecard_templates WHERE organization_id = $1 AND is_active = true ORDER BY name")
            .bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.scorecard_templates WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn create_category(&self, org_id: Uuid, template_id: Uuid, code: &str, name: &str, description: Option<&str>, weight: &str, sort_order: i32, scoring_model: &str, target_score: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<ScorecardCategory> {
        let row = sqlx::query(
            "INSERT INTO _atlas.scorecard_categories (organization_id, template_id, code, name, description, weight, sort_order, scoring_model, target_score, created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
        ).bind(org_id).bind(template_id).bind(code).bind(name).bind(description)
        .bind(weight.parse::<f64>().unwrap_or(0.0)).bind(sort_order).bind(scoring_model)
        .bind(target_score.map(|s| s.parse::<f64>().unwrap_or(0.0))).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_category(&row))
    }

    async fn list_categories(&self, template_id: Uuid) -> AtlasResult<Vec<ScorecardCategory>> {
        let rows = sqlx::query("SELECT * FROM _atlas.scorecard_categories WHERE template_id = $1 ORDER BY sort_order")
            .bind(template_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_category).collect())
    }

    async fn delete_category(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.scorecard_categories WHERE id = $1")
            .bind(id).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn create_scorecard(&self, org_id: Uuid, template_id: Uuid, scorecard_number: &str, supplier_id: Uuid, supplier_name: Option<&str>, supplier_number: Option<&str>, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, notes: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<SupplierScorecard> {
        let row = sqlx::query(
            "INSERT INTO _atlas.supplier_scorecards (organization_id, template_id, scorecard_number, supplier_id, supplier_name, supplier_number, evaluation_period_start, evaluation_period_end, notes, created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
        ).bind(org_id).bind(template_id).bind(scorecard_number).bind(supplier_id)
        .bind(supplier_name).bind(supplier_number).bind(period_start).bind(period_end)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_scorecard(&row))
    }

    async fn get_scorecard(&self, id: Uuid) -> AtlasResult<Option<SupplierScorecard>> {
        let row = sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_scorecard(&r)))
    }

    async fn get_scorecard_by_number(&self, org_id: Uuid, scorecard_number: &str) -> AtlasResult<Option<SupplierScorecard>> {
        let row = sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE organization_id = $1 AND scorecard_number = $2")
            .bind(org_id).bind(scorecard_number).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_scorecard(&r)))
    }

    async fn list_scorecards(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierScorecard>> {
        let rows = match (supplier_id, status) {
            (Some(sid), Some(s)) => sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND supplier_id=$2 AND status=$3 ORDER BY created_at DESC").bind(org_id).bind(sid).bind(s).fetch_all(&self.pool).await,
            (Some(sid), None) => sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND supplier_id=$2 ORDER BY created_at DESC").bind(org_id).bind(sid).fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND status=$2 ORDER BY created_at DESC").bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None) => sqlx::query("SELECT * FROM _atlas.supplier_scorecards WHERE organization_id=$1 ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_scorecard).collect())
    }

    async fn update_scorecard_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierScorecard> {
        let row = sqlx::query("UPDATE _atlas.supplier_scorecards SET status=$2, updated_at=now() WHERE id=$1 RETURNING *")
            .bind(id).bind(status).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_scorecard(&row))
    }

    async fn submit_scorecard(&self, id: Uuid, overall_score: &str, overall_grade: &str, reviewer_id: Option<Uuid>, reviewer_name: Option<&str>) -> AtlasResult<SupplierScorecard> {
        let row = sqlx::query(
            "UPDATE _atlas.supplier_scorecards SET status='submitted', overall_score=$2, overall_grade=$3, reviewer_id=$4, reviewer_name=$5, review_date=now(), updated_at=now() WHERE id=$1 RETURNING *"
        ).bind(id).bind(overall_score.parse::<f64>().unwrap_or(0.0)).bind(overall_grade)
        .bind(reviewer_id).bind(reviewer_name)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_scorecard(&row))
    }

    async fn approve_scorecard(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<SupplierScorecard> {
        let row = sqlx::query(
            "UPDATE _atlas.supplier_scorecards SET status='approved', approved_by=$2, approved_at=now(), updated_at=now() WHERE id=$1 RETURNING *"
        ).bind(id).bind(approved_by).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_scorecard(&row))
    }

    async fn delete_scorecard(&self, org_id: Uuid, scorecard_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND scorecard_number=$2")
            .bind(org_id).bind(scorecard_number).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn add_scorecard_line(&self, org_id: Uuid, scorecard_id: Uuid, category_id: Uuid, line_number: i32, kpi_name: &str, kpi_description: Option<&str>, weight: &str, target_value: Option<&str>, actual_value: Option<&str>, score: &str, weighted_score: &str, evidence: Option<&str>, notes: Option<&str>) -> AtlasResult<ScorecardLine> {
        let row = sqlx::query(
            "INSERT INTO _atlas.scorecard_lines (organization_id, scorecard_id, category_id, line_number, kpi_name, kpi_description, weight, target_value, actual_value, score, weighted_score, evidence, notes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING *"
        ).bind(org_id).bind(scorecard_id).bind(category_id).bind(line_number)
        .bind(kpi_name).bind(kpi_description)
        .bind(weight.parse::<f64>().unwrap_or(0.0))
        .bind(target_value.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(actual_value.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(score.parse::<f64>().unwrap_or(0.0))
        .bind(weighted_score.parse::<f64>().unwrap_or(0.0))
        .bind(evidence).bind(notes)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_line(&row))
    }

    async fn list_scorecard_lines(&self, scorecard_id: Uuid) -> AtlasResult<Vec<ScorecardLine>> {
        let rows = sqlx::query("SELECT * FROM _atlas.scorecard_lines WHERE scorecard_id=$1 ORDER BY line_number")
            .bind(scorecard_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_line).collect())
    }

    async fn delete_scorecard_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.scorecard_lines WHERE id=$1")
            .bind(id).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn create_review(&self, org_id: Uuid, review_number: &str, supplier_id: Uuid, supplier_name: Option<&str>, scorecard_id: Option<Uuid>, review_type: &str, review_period: Option<&str>, period_start: chrono::NaiveDate, period_end: chrono::NaiveDate, created_by: Option<Uuid>) -> AtlasResult<SupplierPerformanceReview> {
        let row = sqlx::query(
            "INSERT INTO _atlas.supplier_performance_reviews (organization_id, review_number, supplier_id, supplier_name, scorecard_id, review_type, review_period, period_start, period_end, created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
        ).bind(org_id).bind(review_number).bind(supplier_id).bind(supplier_name)
        .bind(scorecard_id).bind(review_type).bind(review_period)
        .bind(period_start).bind(period_end).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_review(&row))
    }

    async fn get_review(&self, id: Uuid) -> AtlasResult<Option<SupplierPerformanceReview>> {
        let row = sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_review(&r)))
    }

    async fn get_review_by_number(&self, org_id: Uuid, review_number: &str) -> AtlasResult<Option<SupplierPerformanceReview>> {
        let row = sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 AND review_number=$2")
            .bind(org_id).bind(review_number).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_review(&r)))
    }

    async fn list_reviews(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierPerformanceReview>> {
        let rows = match (supplier_id, status) {
            (Some(sid), Some(s)) => sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 AND supplier_id=$2 AND status=$3 ORDER BY created_at DESC").bind(org_id).bind(sid).bind(s).fetch_all(&self.pool).await,
            (Some(sid), None) => sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 AND supplier_id=$2 ORDER BY created_at DESC").bind(org_id).bind(sid).fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 AND status=$2 ORDER BY created_at DESC").bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None) => sqlx::query("SELECT * FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await,
        }.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_review).collect())
    }

    async fn update_review_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierPerformanceReview> {
        let row = sqlx::query("UPDATE _atlas.supplier_performance_reviews SET status=$2, updated_at=now() WHERE id=$1 RETURNING *")
            .bind(id).bind(status).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_review(&row))
    }

    async fn complete_review(&self, id: Uuid, current_score: Option<&str>, rating: Option<&str>, strengths: Option<&str>, improvement_areas: Option<&str>, action_items: Option<&str>, previous_score: Option<&str>, score_change: Option<&str>, reviewer_id: Option<Uuid>, reviewer_name: Option<&str>) -> AtlasResult<SupplierPerformanceReview> {
        let row = sqlx::query(
            "UPDATE _atlas.supplier_performance_reviews SET status='completed', current_score=$2, rating=$3, strengths=$4, improvement_areas=$5, action_items=$6, previous_score=$7, score_change=$8, reviewer_id=$9, reviewer_name=$10, completed_at=now(), updated_at=now() WHERE id=$1 RETURNING *"
        ).bind(id)
        .bind(current_score.map(|s| s.parse::<f64>().unwrap_or(0.0)))
        .bind(rating).bind(strengths).bind(improvement_areas).bind(action_items)
        .bind(previous_score.map(|s| s.parse::<f64>().unwrap_or(0.0)))
        .bind(score_change.map(|s| s.parse::<f64>().unwrap_or(0.0)))
        .bind(reviewer_id).bind(reviewer_name)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_review(&row))
    }

    async fn delete_review(&self, org_id: Uuid, review_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.supplier_performance_reviews WHERE organization_id=$1 AND review_number=$2")
            .bind(org_id).bind(review_number).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn create_action_item(&self, org_id: Uuid, review_id: Uuid, action_number: i32, description: &str, assignee_id: Option<Uuid>, assignee_name: Option<&str>, priority: &str, due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<ReviewActionItem> {
        let row = sqlx::query(
            "INSERT INTO _atlas.review_action_items (organization_id, review_id, action_number, description, assignee_id, assignee_name, priority, due_date, created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING *"
        ).bind(org_id).bind(review_id).bind(action_number).bind(description)
        .bind(assignee_id).bind(assignee_name).bind(priority).bind(due_date).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_action_item(&row))
    }

    async fn get_action_item(&self, id: Uuid) -> AtlasResult<Option<ReviewActionItem>> {
        let row = sqlx::query("SELECT * FROM _atlas.review_action_items WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row.map(|r| row_to_action_item(&r)))
    }

    async fn list_action_items(&self, review_id: Uuid) -> AtlasResult<Vec<ReviewActionItem>> {
        let rows = sqlx::query("SELECT * FROM _atlas.review_action_items WHERE review_id=$1 ORDER BY action_number")
            .bind(review_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(rows.iter().map(row_to_action_item).collect())
    }

    async fn complete_action_item(&self, id: Uuid) -> AtlasResult<ReviewActionItem> {
        let row = sqlx::query(
            "UPDATE _atlas.review_action_items SET status='completed', completed_at=now(), updated_at=now() WHERE id=$1 RETURNING *"
        ).bind(id).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(row_to_action_item(&row))
    }

    async fn delete_action_item(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.review_action_items WHERE id=$1")
            .bind(id).execute(&self.pool).await.map_err(|e| db_err!(e))?;
        Ok(())
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SupplierScorecardDashboard> {
        let tpl_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.scorecard_templates WHERE organization_id=$1 AND is_active=true"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;

        let summary = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE status IN ('draft','in_review')) as pending, COALESCE(AVG(overall_score),0) as avg_score FROM _atlas.supplier_scorecards WHERE organization_id=$1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| db_err!(e))?;

        let status_rows = sqlx::query(
            "SELECT status, COUNT(*) as cnt FROM _atlas.supplier_scorecards WHERE organization_id=$1 GROUP BY status ORDER BY cnt DESC"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;

        let scorecards_by_status: serde_json::Value = status_rows.iter().map(|r| {
            serde_json::json!({"status": r.get::<String,_>("status"), "count": r.get::<i64,_>("cnt")})
        }).collect();

        let grade_rows = sqlx::query(
            "SELECT overall_grade, COUNT(*) as cnt FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND overall_grade IS NOT NULL GROUP BY overall_grade ORDER BY cnt DESC"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;

        let scorecards_by_grade: serde_json::Value = grade_rows.iter().map(|r| {
            serde_json::json!({"grade": r.get::<String,_>("overall_grade"), "count": r.get::<i64,_>("cnt")})
        }).collect();

        let top_rows = sqlx::query(
            "SELECT supplier_name, overall_score, overall_grade FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND status='approved' AND overall_score > 0 ORDER BY overall_score DESC LIMIT 5"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;

        let top_performers: serde_json::Value = top_rows.iter().map(|r| {
            serde_json::json!({"supplierName": r.get::<Option<String>,_>("supplier_name").unwrap_or_default(), "score": r.get::<f64,_>("overall_score"), "grade": r.get::<Option<String>,_>("overall_grade").unwrap_or_default()})
        }).collect();

        let bottom_rows = sqlx::query(
            "SELECT supplier_name, overall_score, overall_grade FROM _atlas.supplier_scorecards WHERE organization_id=$1 AND status='approved' AND overall_score > 0 ORDER BY overall_score ASC LIMIT 5"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;

        let bottom_performers: serde_json::Value = bottom_rows.iter().map(|r| {
            serde_json::json!({"supplierName": r.get::<Option<String>,_>("supplier_name").unwrap_or_default(), "score": r.get::<f64,_>("overall_score"), "grade": r.get::<Option<String>,_>("overall_grade").unwrap_or_default()})
        }).collect();

        let recent_rows = sqlx::query(
            "SELECT id, scorecard_number, supplier_name, overall_score, overall_grade, status, created_at FROM _atlas.supplier_scorecards WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 10"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| db_err!(e))?;

        let recent_reviews: serde_json::Value = recent_rows.iter().map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid,_>("id").to_string(),
                "scorecardNumber": r.get::<String,_>("scorecard_number"),
                "supplierName": r.get::<Option<String>,_>("supplier_name").unwrap_or_default(),
                "score": r.get::<f64,_>("overall_score"),
                "grade": r.get::<Option<String>,_>("overall_grade").unwrap_or_default(),
                "status": r.get::<String,_>("status"),
                "createdAt": r.get::<chrono::DateTime<chrono::Utc>,_>("created_at").to_rfc3339(),
            })
        }).collect();

        Ok(SupplierScorecardDashboard {
            total_templates: tpl_count as i32,
            total_scorecards: summary.get::<i64,_>("total") as i32,
            pending_reviews: summary.get::<i64,_>("pending") as i32,
            average_score: format!("{:.2}", summary.get::<f64, _>("avg_score")),
            scorecards_by_status,
            scorecards_by_grade,
            top_performers,
            bottom_performers,
            recent_reviews,
        })
    }
}

