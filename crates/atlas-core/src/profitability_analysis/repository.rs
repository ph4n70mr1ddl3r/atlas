//! Profitability Analysis Repository
//!
//! PostgreSQL storage for profitability segments, runs, lines, and templates.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Profitability segment definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfitabilitySegment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub segment_code: String,
    pub segment_name: String,
    pub segment_type: String,
    pub description: Option<String>,
    pub parent_segment_id: Option<Uuid>,
    pub is_active: bool,
    pub sort_order: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profitability analysis run
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfitabilityRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub run_name: String,
    pub analysis_type: String,
    pub period_from: chrono::NaiveDate,
    pub period_to: chrono::NaiveDate,
    pub currency_code: String,
    pub status: String,
    pub total_revenue: f64,
    pub total_cogs: f64,
    pub total_gross_margin: f64,
    pub total_operating_expenses: f64,
    pub total_operating_margin: f64,
    pub total_net_margin: f64,
    pub gross_margin_pct: f64,
    pub operating_margin_pct: f64,
    pub net_margin_pct: f64,
    pub segment_count: i32,
    pub comparison_run_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profitability run line (per-segment detail)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfitabilityRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub segment_id: Option<Uuid>,
    pub segment_code: Option<String>,
    pub segment_name: Option<String>,
    pub segment_type: Option<String>,
    pub line_number: i32,
    pub revenue: f64,
    pub cost_of_goods_sold: f64,
    pub gross_margin: f64,
    pub gross_margin_pct: f64,
    pub operating_expenses: f64,
    pub operating_margin: f64,
    pub operating_margin_pct: f64,
    pub other_income: f64,
    pub other_expense: f64,
    pub net_margin: f64,
    pub net_margin_pct: f64,
    pub revenue_contribution_pct: f64,
    pub margin_contribution_pct: f64,
    pub prior_period_revenue: f64,
    pub prior_period_cogs: f64,
    pub prior_period_net_margin: f64,
    pub revenue_change_pct: f64,
    pub margin_change_pct: f64,
    pub metadata: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profitability analysis template
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfitabilityTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_code: String,
    pub template_name: String,
    pub description: Option<String>,
    pub segment_type: String,
    pub includes_cogs: bool,
    pub includes_operating: bool,
    pub includes_other: bool,
    pub auto_calculate: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profitability dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfitabilityDashboard {
    pub total_segments: i64,
    pub active_segments: i64,
    pub total_runs: i64,
    pub completed_runs: i64,
    pub total_templates: i64,
    pub latest_run: Option<ProfitabilityRun>,
    pub top_margin_segments: Vec<ProfitabilityRunLine>,
    pub bottom_margin_segments: Vec<ProfitabilityRunLine>,
    pub by_segment_type: serde_json::Value,
}

/// Parameters for creating a segment
pub struct SegmentCreateParams {
    pub org_id: Uuid,
    pub segment_code: String,
    pub segment_name: String,
    pub segment_type: String,
    pub description: Option<String>,
    pub parent_segment_id: Option<Uuid>,
    pub sort_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a run
pub struct RunCreateParams {
    pub org_id: Uuid,
    pub run_number: String,
    pub run_name: String,
    pub analysis_type: String,
    pub period_from: chrono::NaiveDate,
    pub period_to: chrono::NaiveDate,
    pub currency_code: String,
    pub comparison_run_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a run line
pub struct RunLineCreateParams {
    pub org_id: Uuid,
    pub run_id: Uuid,
    pub segment_id: Option<Uuid>,
    pub segment_code: Option<String>,
    pub segment_name: Option<String>,
    pub segment_type: Option<String>,
    pub line_number: i32,
    pub revenue: f64,
    pub cost_of_goods_sold: f64,
    pub operating_expenses: f64,
    pub other_income: f64,
    pub other_expense: f64,
}

/// Parameters for creating a template
pub struct TemplateCreateParams {
    pub org_id: Uuid,
    pub template_code: String,
    pub template_name: String,
    pub description: Option<String>,
    pub segment_type: String,
    pub includes_cogs: Option<bool>,
    pub includes_operating: Option<bool>,
    pub includes_other: Option<bool>,
    pub auto_calculate: Option<bool>,
    pub created_by: Option<Uuid>,
}

// ============================================================================
// Repository Trait
// ============================================================================

#[async_trait]
pub trait ProfitabilityAnalysisRepository: Send + Sync {
    // Segments
    async fn create_segment(&self, params: &SegmentCreateParams) -> AtlasResult<ProfitabilitySegment>;
    async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<ProfitabilitySegment>>;
    async fn get_segment_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilitySegment>>;
    async fn list_segments(&self, org_id: Uuid, segment_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<ProfitabilitySegment>>;
    async fn delete_segment(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Runs
    async fn create_run(&self, params: &RunCreateParams) -> AtlasResult<ProfitabilityRun>;
    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityRun>>;
    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<ProfitabilityRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProfitabilityRun>>;
    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<ProfitabilityRun>;
    async fn update_run_totals(
        &self,
        id: Uuid,
        total_revenue: f64,
        total_cogs: f64,
        total_gross_margin: f64,
        total_operating_expenses: f64,
        total_operating_margin: f64,
        total_net_margin: f64,
        gross_margin_pct: f64,
        operating_margin_pct: f64,
        net_margin_pct: f64,
        segment_count: i32,
    ) -> AtlasResult<()>;
    async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()>;

    // Run Lines
    async fn create_run_line(&self, params: &RunLineCreateParams) -> AtlasResult<ProfitabilityRunLine>;
    async fn get_run_line(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityRunLine>>;
    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<ProfitabilityRunLine>>;
    async fn update_run_line_margins(
        &self,
        id: Uuid,
        gross_margin: f64,
        gross_margin_pct: f64,
        operating_margin: f64,
        operating_margin_pct: f64,
        net_margin: f64,
        net_margin_pct: f64,
        revenue_contribution_pct: f64,
        margin_contribution_pct: f64,
    ) -> AtlasResult<()>;
    async fn delete_run_line(&self, run_id: Uuid, line_id: Uuid) -> AtlasResult<()>;

    // Templates
    async fn create_template(&self, params: &TemplateCreateParams) -> AtlasResult<ProfitabilityTemplate>;
    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilityTemplate>>;
    async fn list_templates(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<ProfitabilityTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProfitabilityDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresProfitabilityAnalysisRepository {
    pool: PgPool,
}

impl PostgresProfitabilityAnalysisRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfitabilityAnalysisRepository for PostgresProfitabilityAnalysisRepository {
    // Segments
    async fn create_segment(&self, params: &SegmentCreateParams) -> AtlasResult<ProfitabilitySegment> {
        let row = sqlx::query_as::<_, ProfitabilitySegment>(
            r#"INSERT INTO _atlas.profitability_segments
               (organization_id, segment_code, segment_name, segment_type, description,
                parent_segment_id, sort_order, metadata, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#
        )
            .bind(params.org_id)
            .bind(&params.segment_code)
            .bind(&params.segment_name)
            .bind(&params.segment_type)
            .bind(&params.description)
            .bind(params.parent_segment_id)
            .bind(params.sort_order.unwrap_or(0))
            .bind(params.metadata.clone().unwrap_or(serde_json::json!({})))
            .bind(params.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<ProfitabilitySegment>> {
        let row = sqlx::query_as::<_, ProfitabilitySegment>(
            "SELECT * FROM _atlas.profitability_segments WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_segment_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilitySegment>> {
        let row = sqlx::query_as::<_, ProfitabilitySegment>(
            "SELECT * FROM _atlas.profitability_segments WHERE organization_id = $1 AND segment_code = $2"
        )
            .bind(org_id)
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_segments(&self, org_id: Uuid, segment_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<ProfitabilitySegment>> {
        let rows = if let Some(st) = segment_type {
            if let Some(active) = is_active {
                sqlx::query_as::<_, ProfitabilitySegment>(
                    "SELECT * FROM _atlas.profitability_segments WHERE organization_id = $1 AND segment_type = $2 AND is_active = $3 ORDER BY sort_order, segment_name"
                )
                    .bind(org_id).bind(st).bind(active)
                    .fetch_all(&self.pool).await
            } else {
                sqlx::query_as::<_, ProfitabilitySegment>(
                    "SELECT * FROM _atlas.profitability_segments WHERE organization_id = $1 AND segment_type = $2 ORDER BY sort_order, segment_name"
                )
                    .bind(org_id).bind(st)
                    .fetch_all(&self.pool).await
            }
        } else if let Some(active) = is_active {
            sqlx::query_as::<_, ProfitabilitySegment>(
                "SELECT * FROM _atlas.profitability_segments WHERE organization_id = $1 AND is_active = $2 ORDER BY sort_order, segment_name"
            )
                .bind(org_id).bind(active)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query_as::<_, ProfitabilitySegment>(
                "SELECT * FROM _atlas.profitability_segments WHERE organization_id = $1 ORDER BY sort_order, segment_name"
            )
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn delete_segment(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.profitability_segments WHERE organization_id = $1 AND segment_code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Runs
    async fn create_run(&self, params: &RunCreateParams) -> AtlasResult<ProfitabilityRun> {
        let row = sqlx::query_as::<_, ProfitabilityRun>(
            r#"INSERT INTO _atlas.profitability_runs
               (organization_id, run_number, run_name, analysis_type, period_from, period_to,
                currency_code, comparison_run_id, notes, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#
        )
            .bind(params.org_id)
            .bind(&params.run_number)
            .bind(&params.run_name)
            .bind(&params.analysis_type)
            .bind(params.period_from)
            .bind(params.period_to)
            .bind(&params.currency_code)
            .bind(params.comparison_run_id)
            .bind(&params.notes)
            .bind(params.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityRun>> {
        let row = sqlx::query_as::<_, ProfitabilityRun>(
            "SELECT * FROM _atlas.profitability_runs WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<ProfitabilityRun>> {
        let row = sqlx::query_as::<_, ProfitabilityRun>(
            "SELECT * FROM _atlas.profitability_runs WHERE organization_id = $1 AND run_number = $2"
        )
            .bind(org_id).bind(run_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProfitabilityRun>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, ProfitabilityRun>(
                "SELECT * FROM _atlas.profitability_runs WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query_as::<_, ProfitabilityRun>(
                "SELECT * FROM _atlas.profitability_runs WHERE organization_id = $1 ORDER BY created_at DESC"
            )
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<ProfitabilityRun> {
        let row = sqlx::query_as::<_, ProfitabilityRun>(
            "UPDATE _atlas.profitability_runs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_run_totals(
        &self,
        id: Uuid,
        total_revenue: f64,
        total_cogs: f64,
        total_gross_margin: f64,
        total_operating_expenses: f64,
        total_operating_margin: f64,
        total_net_margin: f64,
        gross_margin_pct: f64,
        operating_margin_pct: f64,
        net_margin_pct: f64,
        segment_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.profitability_runs SET
               total_revenue = $2, total_cogs = $3, total_gross_margin = $4,
               total_operating_expenses = $5, total_operating_margin = $6, total_net_margin = $7,
               gross_margin_pct = $8, operating_margin_pct = $9, net_margin_pct = $10,
               segment_count = $11, updated_at = now()
               WHERE id = $1"#
        )
            .bind(id)
            .bind(total_revenue).bind(total_cogs).bind(total_gross_margin)
            .bind(total_operating_expenses).bind(total_operating_margin).bind(total_net_margin)
            .bind(gross_margin_pct).bind(operating_margin_pct).bind(net_margin_pct)
            .bind(segment_count)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.profitability_runs WHERE organization_id = $1 AND run_number = $2 AND status = 'draft'")
            .bind(org_id).bind(run_number)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Run Lines
    async fn create_run_line(&self, params: &RunLineCreateParams) -> AtlasResult<ProfitabilityRunLine> {
        let row = sqlx::query_as::<_, ProfitabilityRunLine>(
            r#"INSERT INTO _atlas.profitability_run_lines
               (organization_id, run_id, segment_id, segment_code, segment_name, segment_type,
                line_number, revenue, cost_of_goods_sold, operating_expenses, other_income, other_expense)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#
        )
            .bind(params.org_id)
            .bind(params.run_id)
            .bind(params.segment_id)
            .bind(&params.segment_code)
            .bind(&params.segment_name)
            .bind(&params.segment_type)
            .bind(params.line_number)
            .bind(params.revenue)
            .bind(params.cost_of_goods_sold)
            .bind(params.operating_expenses)
            .bind(params.other_income)
            .bind(params.other_expense)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_run_line(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityRunLine>> {
        let row = sqlx::query_as::<_, ProfitabilityRunLine>(
            "SELECT * FROM _atlas.profitability_run_lines WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<ProfitabilityRunLine>> {
        let rows = sqlx::query_as::<_, ProfitabilityRunLine>(
            "SELECT * FROM _atlas.profitability_run_lines WHERE run_id = $1 ORDER BY line_number"
        )
            .bind(run_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_run_line_margins(
        &self,
        id: Uuid,
        gross_margin: f64,
        gross_margin_pct: f64,
        operating_margin: f64,
        operating_margin_pct: f64,
        net_margin: f64,
        net_margin_pct: f64,
        revenue_contribution_pct: f64,
        margin_contribution_pct: f64,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.profitability_run_lines SET
               gross_margin = $2, gross_margin_pct = $3,
               operating_margin = $4, operating_margin_pct = $5,
               net_margin = $6, net_margin_pct = $7,
               revenue_contribution_pct = $8, margin_contribution_pct = $9,
               updated_at = now()
               WHERE id = $1"#
        )
            .bind(id)
            .bind(gross_margin).bind(gross_margin_pct)
            .bind(operating_margin).bind(operating_margin_pct)
            .bind(net_margin).bind(net_margin_pct)
            .bind(revenue_contribution_pct).bind(margin_contribution_pct)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_run_line(&self, run_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.profitability_run_lines WHERE run_id = $1 AND id = $2")
            .bind(run_id).bind(line_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Templates
    async fn create_template(&self, params: &TemplateCreateParams) -> AtlasResult<ProfitabilityTemplate> {
        let row = sqlx::query_as::<_, ProfitabilityTemplate>(
            r#"INSERT INTO _atlas.profitability_templates
               (organization_id, template_code, template_name, description, segment_type,
                includes_cogs, includes_operating, includes_other, auto_calculate, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#
        )
            .bind(params.org_id)
            .bind(&params.template_code)
            .bind(&params.template_name)
            .bind(&params.description)
            .bind(&params.segment_type)
            .bind(params.includes_cogs.unwrap_or(true))
            .bind(params.includes_operating.unwrap_or(true))
            .bind(params.includes_other.unwrap_or(true))
            .bind(params.auto_calculate.unwrap_or(true))
            .bind(params.created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ProfitabilityTemplate>> {
        let row = sqlx::query_as::<_, ProfitabilityTemplate>(
            "SELECT * FROM _atlas.profitability_templates WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ProfitabilityTemplate>> {
        let row = sqlx::query_as::<_, ProfitabilityTemplate>(
            "SELECT * FROM _atlas.profitability_templates WHERE organization_id = $1 AND template_code = $2"
        )
            .bind(org_id).bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_templates(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<ProfitabilityTemplate>> {
        let rows = if let Some(active) = is_active {
            sqlx::query_as::<_, ProfitabilityTemplate>(
                "SELECT * FROM _atlas.profitability_templates WHERE organization_id = $1 AND is_active = $2 ORDER BY template_name"
            )
                .bind(org_id).bind(active)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query_as::<_, ProfitabilityTemplate>(
                "SELECT * FROM _atlas.profitability_templates WHERE organization_id = $1 ORDER BY template_name"
            )
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.profitability_templates WHERE organization_id = $1 AND template_code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProfitabilityDashboard> {
        let total_segments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.profitability_segments WHERE organization_id = $1"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let active_segments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.profitability_segments WHERE organization_id = $1 AND is_active = true"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let total_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.profitability_runs WHERE organization_id = $1"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let completed_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.profitability_runs WHERE organization_id = $1 AND status = 'completed'"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let total_templates: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.profitability_templates WHERE organization_id = $1"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let latest_run = sqlx::query_as::<_, ProfitabilityRun>(
            "SELECT * FROM _atlas.profitability_runs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
            .bind(org_id)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten();

        // Get top 5 by net margin from latest run
        let top_margin_segments = if let Some(ref run) = latest_run {
            sqlx::query_as::<_, ProfitabilityRunLine>(
                "SELECT * FROM _atlas.profitability_run_lines WHERE run_id = $1 ORDER BY net_margin_pct DESC LIMIT 5"
            )
                .bind(run.id)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        let bottom_margin_segments = if let Some(ref run) = latest_run {
            sqlx::query_as::<_, ProfitabilityRunLine>(
                "SELECT * FROM _atlas.profitability_run_lines WHERE run_id = $1 ORDER BY net_margin_pct ASC LIMIT 5"
            )
                .bind(run.id)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Count by segment type
        let type_rows = sqlx::query(
            "SELECT segment_type, COUNT(*) as cnt FROM _atlas.profitability_segments WHERE organization_id = $1 GROUP BY segment_type"
        )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

        let mut by_type = serde_json::Map::new();
        for row in type_rows {
            if let (Ok(st), Ok(cnt)) = (row.try_get::<String, _>("segment_type"), row.try_get::<i64, _>("cnt")) {
                by_type.insert(st, serde_json::Value::Number(cnt.into()));
            }
        }

        Ok(ProfitabilityDashboard {
            total_segments,
            active_segments,
            total_runs,
            completed_runs,
            total_templates,
            latest_run,
            top_margin_segments,
            bottom_margin_segments,
            by_segment_type: serde_json::Value::Object(by_type),
        })
    }
}
