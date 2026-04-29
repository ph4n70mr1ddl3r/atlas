//! Risk Management Repository
//!
//! PostgreSQL storage for risk categories, risks, controls, mappings,
//! control tests, issues, and dashboard summary.

use atlas_shared::{
    RiskCategory, RiskEntry, ControlEntry, RiskControlMapping,
    ControlTest, RiskIssue, RiskDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for risk management data storage
#[async_trait]
pub trait RiskManagementRepository: Send + Sync {
    // Risk Categories
    async fn create_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, sort_order: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<RiskCategory>;
    async fn get_category(&self, id: Uuid) -> AtlasResult<Option<RiskCategory>>;
    async fn get_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskCategory>>;
    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<RiskCategory>>;
    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Risk Register
    async fn create_risk(
        &self, org_id: Uuid, risk_number: &str, title: &str, description: Option<&str>,
        category_id: Option<Uuid>, risk_source: &str, likelihood: i32, impact: i32,
        risk_level: &str, owner_id: Option<Uuid>, owner_name: Option<&str>,
        response_strategy: Option<&str>, business_units: serde_json::Value,
        related_entity_type: Option<&str>, related_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskEntry>;
    async fn get_risk(&self, id: Uuid) -> AtlasResult<Option<RiskEntry>>;
    async fn get_risk_by_number(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<Option<RiskEntry>>;
    async fn list_risks(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>, risk_source: Option<&str>,
    ) -> AtlasResult<Vec<RiskEntry>>;
    async fn update_risk_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskEntry>;
    async fn assess_risk(
        &self, id: Uuid, likelihood: i32, impact: i32, risk_level: &str,
        residual_likelihood: Option<i32>, residual_impact: Option<i32>,
    ) -> AtlasResult<RiskEntry>;
    async fn delete_risk(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<()>;

    // Control Registry
    async fn create_control(
        &self, org_id: Uuid, control_number: &str, title: &str, description: Option<&str>,
        control_type: &str, control_nature: &str, frequency: &str,
        objective: Option<&str>, test_procedures: Option<&str>,
        owner_id: Option<Uuid>, owner_name: Option<&str>, is_key_control: bool,
        business_processes: serde_json::Value, regulatory_frameworks: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlEntry>;
    async fn get_control(&self, id: Uuid) -> AtlasResult<Option<ControlEntry>>;
    async fn get_control_by_number(&self, org_id: Uuid, control_number: &str) -> AtlasResult<Option<ControlEntry>>;
    async fn list_controls(
        &self, org_id: Uuid, status: Option<&str>, control_type: Option<&str>,
    ) -> AtlasResult<Vec<ControlEntry>>;
    async fn update_control_status(&self, id: Uuid, status: &str) -> AtlasResult<ControlEntry>;
    async fn update_control_effectiveness(&self, id: Uuid, effectiveness: &str) -> AtlasResult<ControlEntry>;
    async fn delete_control(&self, org_id: Uuid, control_number: &str) -> AtlasResult<()>;

    // Risk-Control Mappings
    async fn create_risk_control_mapping(
        &self, org_id: Uuid, risk_id: Uuid, control_id: Uuid,
        mitigation_effectiveness: &str, description: Option<&str>, mapped_by: Option<Uuid>,
    ) -> AtlasResult<RiskControlMapping>;
    async fn list_risk_mappings(&self, risk_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>>;
    async fn list_control_mappings(&self, control_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>>;
    async fn delete_mapping(&self, id: Uuid) -> AtlasResult<()>;

    // Control Tests
    async fn create_control_test(
        &self, org_id: Uuid, control_id: Uuid, test_number: &str, test_plan: &str,
        test_period_start: chrono::NaiveDate, test_period_end: chrono::NaiveDate,
        tester_id: Option<Uuid>, tester_name: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ControlTest>;
    async fn get_control_test(&self, id: Uuid) -> AtlasResult<Option<ControlTest>>;
    async fn list_control_tests(&self, control_id: Uuid) -> AtlasResult<Vec<ControlTest>>;
    async fn update_control_test_status(&self, id: Uuid, status: &str) -> AtlasResult<ControlTest>;
    async fn complete_control_test(
        &self, id: Uuid, result: &str, findings: Option<&str>,
        deficiency_severity: Option<&str>, sample_size: Option<i32>, sample_exceptions: Option<i32>,
    ) -> AtlasResult<ControlTest>;
    async fn delete_control_test(&self, org_id: Uuid, test_number: &str) -> AtlasResult<()>;

    // Issues
    async fn create_issue(
        &self, org_id: Uuid, issue_number: &str, title: &str, description: &str,
        source: &str, risk_id: Option<Uuid>, control_id: Option<Uuid>,
        control_test_id: Option<Uuid>, severity: &str, priority: &str,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        remediation_plan: Option<&str>, remediation_due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskIssue>;
    async fn get_issue(&self, id: Uuid) -> AtlasResult<Option<RiskIssue>>;
    async fn get_issue_by_number(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<Option<RiskIssue>>;
    async fn list_issues(
        &self, org_id: Uuid, status: Option<&str>, severity: Option<&str>,
    ) -> AtlasResult<Vec<RiskIssue>>;
    async fn update_issue_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskIssue>;
    async fn resolve_issue(
        &self, id: Uuid, root_cause: Option<&str>, corrective_actions: Option<&str>,
    ) -> AtlasResult<RiskIssue>;
    async fn delete_issue(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RiskDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresRiskManagementRepository {
    pool: PgPool,
}

impl PostgresRiskManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper: map row to RiskCategory
fn row_to_category(row: &sqlx::postgres::PgRow) -> RiskCategory {
    RiskCategory {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        parent_category_id: row.try_get("parent_category_id").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
        sort_order: row.try_get("sort_order").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// Helper: map row to RiskEntry
fn row_to_risk(row: &sqlx::postgres::PgRow) -> RiskEntry {
    RiskEntry {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        risk_number: row.try_get("risk_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        category_id: row.try_get("category_id").unwrap_or_default(),
        risk_source: row.try_get("risk_source").unwrap_or_default(),
        likelihood: row.try_get("likelihood").unwrap_or(3),
        impact: row.try_get("impact").unwrap_or(3),
        risk_score: row.try_get("risk_score").unwrap_or(9),
        risk_level: row.try_get("risk_level").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        business_units: row.try_get("business_units").unwrap_or(serde_json::json!([])),
        response_strategy: row.try_get("response_strategy").unwrap_or_default(),
        residual_likelihood: row.try_get("residual_likelihood").unwrap_or_default(),
        residual_impact: row.try_get("residual_impact").unwrap_or_default(),
        identified_date: row.try_get("identified_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        last_assessed_date: row.try_get("last_assessed_date").unwrap_or_default(),
        next_review_date: row.try_get("next_review_date").unwrap_or_default(),
        closed_date: row.try_get("closed_date").unwrap_or_default(),
        related_entity_type: row.try_get("related_entity_type").unwrap_or_default(),
        related_entity_id: row.try_get("related_entity_id").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

// Helper: map row to ControlEntry
fn row_to_control(row: &sqlx::postgres::PgRow) -> ControlEntry {
    ControlEntry {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        control_number: row.try_get("control_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        control_type: row.try_get("control_type").unwrap_or_default(),
        control_nature: row.try_get("control_nature").unwrap_or_default(),
        frequency: row.try_get("frequency").unwrap_or_default(),
        objective: row.try_get("objective").unwrap_or_default(),
        test_procedures: row.try_get("test_procedures").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        is_key_control: row.try_get("is_key_control").unwrap_or(false),
        effectiveness: row.try_get("effectiveness").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        business_processes: row.try_get("business_processes").unwrap_or(serde_json::json!([])),
        regulatory_frameworks: row.try_get("regulatory_frameworks").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_mapping(row: &sqlx::postgres::PgRow) -> RiskControlMapping {
    RiskControlMapping {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        risk_id: row.try_get("risk_id").unwrap_or_default(),
        control_id: row.try_get("control_id").unwrap_or_default(),
        mitigation_effectiveness: row.try_get("mitigation_effectiveness").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        mapped_by: row.try_get("mapped_by").unwrap_or_default(),
        mapped_at: row.try_get("mapped_at").unwrap_or(chrono::Utc::now()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_test(row: &sqlx::postgres::PgRow) -> ControlTest {
    ControlTest {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        control_id: row.try_get("control_id").unwrap_or_default(),
        test_number: row.try_get("test_number").unwrap_or_default(),
        test_plan: row.try_get("test_plan").unwrap_or_default(),
        test_period_start: row.try_get("test_period_start").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        test_period_end: row.try_get("test_period_end").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        tester_id: row.try_get("tester_id").unwrap_or_default(),
        tester_name: row.try_get("tester_name").unwrap_or_default(),
        result: row.try_get("result").unwrap_or_default(),
        findings: row.try_get("findings").unwrap_or_default(),
        deficiency_severity: row.try_get("deficiency_severity").unwrap_or_default(),
        evidence_document_ids: row.try_get("evidence_document_ids").unwrap_or(serde_json::json!([])),
        sample_size: row.try_get("sample_size").unwrap_or_default(),
        sample_exceptions: row.try_get("sample_exceptions").unwrap_or_default(),
        started_at: row.try_get("started_at").unwrap_or_default(),
        completed_at: row.try_get("completed_at").unwrap_or_default(),
        reviewer_id: row.try_get("reviewer_id").unwrap_or_default(),
        reviewer_name: row.try_get("reviewer_name").unwrap_or_default(),
        reviewed_at: row.try_get("reviewed_at").unwrap_or_default(),
        review_status: row.try_get("review_status").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_issue(row: &sqlx::postgres::PgRow) -> RiskIssue {
    RiskIssue {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        issue_number: row.try_get("issue_number").unwrap_or_default(),
        title: row.try_get("title").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        source: row.try_get("source").unwrap_or_default(),
        risk_id: row.try_get("risk_id").unwrap_or_default(),
        control_id: row.try_get("control_id").unwrap_or_default(),
        control_test_id: row.try_get("control_test_id").unwrap_or_default(),
        severity: row.try_get("severity").unwrap_or_default(),
        priority: row.try_get("priority").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        owner_name: row.try_get("owner_name").unwrap_or_default(),
        remediation_plan: row.try_get("remediation_plan").unwrap_or_default(),
        remediation_due_date: row.try_get("remediation_due_date").unwrap_or_default(),
        remediation_completed_date: row.try_get("remediation_completed_date").unwrap_or_default(),
        root_cause: row.try_get("root_cause").unwrap_or_default(),
        corrective_actions: row.try_get("corrective_actions").unwrap_or_default(),
        identified_date: row.try_get("identified_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        resolved_date: row.try_get("resolved_date").unwrap_or_default(),
        closed_date: row.try_get("closed_date").unwrap_or_default(),
        regulatory_reference: row.try_get("regulatory_reference").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl RiskManagementRepository for PostgresRiskManagementRepository {
    // ========================================================================
    // Risk Categories
    // ========================================================================

    async fn create_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, sort_order: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<RiskCategory> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.risk_categories
                (organization_id, code, name, description, parent_category_id, sort_order, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb, $7)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_category_id).bind(sort_order).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_category(&row))
    }

    async fn get_category(&self, id: Uuid) -> AtlasResult<Option<RiskCategory>> {
        let row = sqlx::query("SELECT * FROM _atlas.risk_categories WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn get_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.risk_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_category))
    }

    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<RiskCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.risk_categories WHERE organization_id = $1 AND is_active = true ORDER BY sort_order, created_at"
        ).bind(org_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_category).collect())
    }

    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.risk_categories WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Category '{}' not found", code)));
        }
        Ok(())
    }

    // ========================================================================
    // Risk Register
    // ========================================================================

    async fn create_risk(
        &self, org_id: Uuid, risk_number: &str, title: &str, description: Option<&str>,
        category_id: Option<Uuid>, risk_source: &str, likelihood: i32, impact: i32,
        risk_level: &str, owner_id: Option<Uuid>, owner_name: Option<&str>,
        response_strategy: Option<&str>, business_units: serde_json::Value,
        related_entity_type: Option<&str>, related_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.risk_register
                (organization_id, risk_number, title, description, category_id,
                 risk_source, likelihood, impact, risk_level, status,
                 owner_id, owner_name, response_strategy, business_units,
                 related_entity_type, related_entity_id, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'identified',
                    $10, $11, $12, $13, $14, $15, '{}'::jsonb, $16)
            RETURNING *"#,
        )
        .bind(org_id).bind(risk_number).bind(title).bind(description)
        .bind(category_id).bind(risk_source).bind(likelihood).bind(impact)
        .bind(risk_level).bind(owner_id).bind(owner_name)
        .bind(response_strategy).bind(&business_units)
        .bind(related_entity_type).bind(related_entity_id).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_risk(&row))
    }

    async fn get_risk(&self, id: Uuid) -> AtlasResult<Option<RiskEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.risk_register WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_risk))
    }

    async fn get_risk_by_number(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<Option<RiskEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.risk_register WHERE organization_id = $1 AND risk_number = $2"
        ).bind(org_id).bind(risk_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_risk))
    }

    async fn list_risks(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>, risk_source: Option<&str>,
    ) -> AtlasResult<Vec<RiskEntry>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.risk_register
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR risk_level = $3)
                 AND ($4::text IS NULL OR risk_source = $4)
               ORDER BY risk_score DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(risk_level).bind(risk_source)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_risk).collect())
    }

    async fn update_risk_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskEntry> {
        let _closed_date = if status == "closed" { "CURRENT_DATE" } else { "NULL" };
        let query = r#"UPDATE _atlas.risk_register SET status = $2, closed_date = CASE WHEN $3 THEN CURRENT_DATE ELSE closed_date END, updated_at = now()
               WHERE id = $1 RETURNING *"#.to_string();
        let row = sqlx::query(&query)
            .bind(id).bind(status).bind(status == "closed")
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Risk {} not found", id)))?;
        Ok(row_to_risk(&row))
    }

    async fn assess_risk(
        &self, id: Uuid, likelihood: i32, impact: i32, risk_level: &str,
        residual_likelihood: Option<i32>, residual_impact: Option<i32>,
    ) -> AtlasResult<RiskEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.risk_register
               SET likelihood = $2, impact = $3, risk_level = $4,
                   residual_likelihood = $5, residual_impact = $6,
                   last_assessed_date = CURRENT_DATE, status = 'assessed',
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(likelihood).bind(impact).bind(risk_level)
        .bind(residual_likelihood).bind(residual_impact)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Risk {} not found", id)))?;
        Ok(row_to_risk(&row))
    }

    async fn delete_risk(&self, org_id: Uuid, risk_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.risk_register WHERE organization_id = $1 AND risk_number = $2"
        ).bind(org_id).bind(risk_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Risk '{}' not found", risk_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Control Registry
    // ========================================================================

    async fn create_control(
        &self, org_id: Uuid, control_number: &str, title: &str, description: Option<&str>,
        control_type: &str, control_nature: &str, frequency: &str,
        objective: Option<&str>, test_procedures: Option<&str>,
        owner_id: Option<Uuid>, owner_name: Option<&str>, is_key_control: bool,
        business_processes: serde_json::Value, regulatory_frameworks: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.control_registry
                (organization_id, control_number, title, description,
                 control_type, control_nature, frequency,
                 objective, test_procedures, owner_id, owner_name,
                 is_key_control, business_processes, regulatory_frameworks,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, '{}'::jsonb, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(control_number).bind(title).bind(description)
        .bind(control_type).bind(control_nature).bind(frequency)
        .bind(objective).bind(test_procedures).bind(owner_id).bind(owner_name)
        .bind(is_key_control).bind(&business_processes).bind(&regulatory_frameworks)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_control(&row))
    }

    async fn get_control(&self, id: Uuid) -> AtlasResult<Option<ControlEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.control_registry WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_control))
    }

    async fn get_control_by_number(&self, org_id: Uuid, control_number: &str) -> AtlasResult<Option<ControlEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.control_registry WHERE organization_id = $1 AND control_number = $2"
        ).bind(org_id).bind(control_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_control))
    }

    async fn list_controls(
        &self, org_id: Uuid, status: Option<&str>, control_type: Option<&str>,
    ) -> AtlasResult<Vec<ControlEntry>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.control_registry
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR control_type = $3)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(control_type)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_control).collect())
    }

    async fn update_control_status(&self, id: Uuid, status: &str) -> AtlasResult<ControlEntry> {
        let row = sqlx::query(
            "UPDATE _atlas.control_registry SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Control {} not found", id)))?;
        Ok(row_to_control(&row))
    }

    async fn update_control_effectiveness(&self, id: Uuid, effectiveness: &str) -> AtlasResult<ControlEntry> {
        let row = sqlx::query(
            "UPDATE _atlas.control_registry SET effectiveness = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(effectiveness)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Control {} not found", id)))?;
        Ok(row_to_control(&row))
    }

    async fn delete_control(&self, org_id: Uuid, control_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.control_registry WHERE organization_id = $1 AND control_number = $2"
        ).bind(org_id).bind(control_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Control '{}' not found", control_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Risk-Control Mappings
    // ========================================================================

    async fn create_risk_control_mapping(
        &self, org_id: Uuid, risk_id: Uuid, control_id: Uuid,
        mitigation_effectiveness: &str, description: Option<&str>, mapped_by: Option<Uuid>,
    ) -> AtlasResult<RiskControlMapping> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.risk_control_mappings
                (organization_id, risk_id, control_id, mitigation_effectiveness, description, mapped_by, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb)
            RETURNING *"#,
        )
        .bind(org_id).bind(risk_id).bind(control_id)
        .bind(mitigation_effectiveness).bind(description).bind(mapped_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_mapping(&row))
    }

    async fn list_risk_mappings(&self, risk_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.risk_control_mappings WHERE risk_id = $1 AND status = 'active' ORDER BY created_at"
        ).bind(risk_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_mapping).collect())
    }

    async fn list_control_mappings(&self, control_id: Uuid) -> AtlasResult<Vec<RiskControlMapping>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.risk_control_mappings WHERE control_id = $1 AND status = 'active' ORDER BY created_at"
        ).bind(control_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_mapping).collect())
    }

    async fn delete_mapping(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.risk_control_mappings WHERE id = $1")
            .bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Mapping not found".to_string()));
        }
        Ok(())
    }

    // ========================================================================
    // Control Tests
    // ========================================================================

    async fn create_control_test(
        &self, org_id: Uuid, control_id: Uuid, test_number: &str, test_plan: &str,
        test_period_start: chrono::NaiveDate, test_period_end: chrono::NaiveDate,
        tester_id: Option<Uuid>, tester_name: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ControlTest> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.control_tests
                (organization_id, control_id, test_number, test_plan,
                 test_period_start, test_period_end, tester_id, tester_name,
                 evidence_document_ids, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '[]'::jsonb, '{}'::jsonb, $9)
            RETURNING *"#,
        )
        .bind(org_id).bind(control_id).bind(test_number).bind(test_plan)
        .bind(test_period_start).bind(test_period_end)
        .bind(tester_id).bind(tester_name).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_test(&row))
    }

    async fn get_control_test(&self, id: Uuid) -> AtlasResult<Option<ControlTest>> {
        let row = sqlx::query("SELECT * FROM _atlas.control_tests WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_test))
    }

    async fn list_control_tests(&self, control_id: Uuid) -> AtlasResult<Vec<ControlTest>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.control_tests WHERE control_id = $1 ORDER BY created_at DESC"
        ).bind(control_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_test).collect())
    }

    async fn update_control_test_status(&self, id: Uuid, status: &str) -> AtlasResult<ControlTest> {
        let _started = if status == "in_progress" { "now()" } else { "started_at" };
        let query = r#"UPDATE _atlas.control_tests SET status = $2, started_at = CASE WHEN $3 THEN now() ELSE started_at END, updated_at = now()
               WHERE id = $1 RETURNING *"#.to_string();
        let row = sqlx::query(&query)
            .bind(id).bind(status).bind(status == "in_progress")
            .fetch_one(&self.pool).await
            .map_err(|_| AtlasError::EntityNotFound(format!("Control test {} not found", id)))?;
        Ok(row_to_test(&row))
    }

    async fn complete_control_test(
        &self, id: Uuid, result: &str, findings: Option<&str>,
        deficiency_severity: Option<&str>, sample_size: Option<i32>, sample_exceptions: Option<i32>,
    ) -> AtlasResult<ControlTest> {
        let row = sqlx::query(
            r#"UPDATE _atlas.control_tests
               SET result = $2, findings = $3, deficiency_severity = $4,
                   sample_size = $5, sample_exceptions = $6,
                   status = 'completed', completed_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(result).bind(findings).bind(deficiency_severity)
        .bind(sample_size).bind(sample_exceptions)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Control test {} not found", id)))?;
        Ok(row_to_test(&row))
    }

    async fn delete_control_test(&self, org_id: Uuid, test_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.control_tests WHERE organization_id = $1 AND test_number = $2"
        ).bind(org_id).bind(test_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Test '{}' not found", test_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Issues
    // ========================================================================

    async fn create_issue(
        &self, org_id: Uuid, issue_number: &str, title: &str, description: &str,
        source: &str, risk_id: Option<Uuid>, control_id: Option<Uuid>,
        control_test_id: Option<Uuid>, severity: &str, priority: &str,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        remediation_plan: Option<&str>, remediation_due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RiskIssue> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.risk_issues
                (organization_id, issue_number, title, description, source,
                 risk_id, control_id, control_test_id, severity, priority,
                 owner_id, owner_name, remediation_plan, remediation_due_date,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, '{}'::jsonb, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(issue_number).bind(title).bind(description).bind(source)
        .bind(risk_id).bind(control_id).bind(control_test_id)
        .bind(severity).bind(priority)
        .bind(owner_id).bind(owner_name).bind(remediation_plan).bind(remediation_due_date)
        .bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_issue(&row))
    }

    async fn get_issue(&self, id: Uuid) -> AtlasResult<Option<RiskIssue>> {
        let row = sqlx::query("SELECT * FROM _atlas.risk_issues WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_issue))
    }

    async fn get_issue_by_number(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<Option<RiskIssue>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.risk_issues WHERE organization_id = $1 AND issue_number = $2"
        ).bind(org_id).bind(issue_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_issue))
    }

    async fn list_issues(
        &self, org_id: Uuid, status: Option<&str>, severity: Option<&str>,
    ) -> AtlasResult<Vec<RiskIssue>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.risk_issues
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::text IS NULL OR severity = $3)
               ORDER BY
                 CASE severity WHEN 'critical' THEN 1 WHEN 'high' THEN 2 WHEN 'medium' THEN 3 ELSE 4 END,
                 created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(severity)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_issue).collect())
    }

    async fn update_issue_status(&self, id: Uuid, status: &str) -> AtlasResult<RiskIssue> {
        let row = sqlx::query(
            r#"UPDATE _atlas.risk_issues SET status = $2,
                resolved_date = CASE WHEN $3 THEN CURRENT_DATE ELSE resolved_date END,
                closed_date = CASE WHEN $4 THEN CURRENT_DATE ELSE closed_date END,
                remediation_completed_date = CASE WHEN $5 THEN CURRENT_DATE ELSE remediation_completed_date END,
                updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .bind(status == "resolved")
        .bind(status == "closed")
        .bind(status == "remediation_in_progress")
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Issue {} not found", id)))?;
        Ok(row_to_issue(&row))
    }

    async fn resolve_issue(
        &self, id: Uuid, root_cause: Option<&str>, corrective_actions: Option<&str>,
    ) -> AtlasResult<RiskIssue> {
        let row = sqlx::query(
            r#"UPDATE _atlas.risk_issues
               SET status = 'resolved', root_cause = $2, corrective_actions = $3,
                   resolved_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(root_cause).bind(corrective_actions)
        .fetch_one(&self.pool).await
        .map_err(|_| AtlasError::EntityNotFound(format!("Issue {} not found", id)))?;
        Ok(row_to_issue(&row))
    }

    async fn delete_issue(&self, org_id: Uuid, issue_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.risk_issues WHERE organization_id = $1 AND issue_number = $2"
        ).bind(org_id).bind(issue_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Issue '{}' not found", issue_number)));
        }
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RiskDashboard> {
        // Count risks
        let risk_rows = sqlx::query(
            "SELECT status, risk_level, risk_source FROM _atlas.risk_register WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut open_risks = 0i32;
        let mut mitigated_risks = 0i32;
        let mut accepted_risks = 0i32;
        let mut critical_risks = 0i32;
        let mut high_risks = 0i32;
        let mut medium_risks = 0i32;
        let mut low_risks = 0i32;
        let mut risks_by_source = std::collections::HashMap::new();
        let mut risks_by_level = std::collections::HashMap::new();

        for row in &risk_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let level: String = row.try_get("risk_level").unwrap_or_default();
            let source: String = row.try_get("risk_source").unwrap_or_default();

            match status.as_str() {
                "identified" | "assessed" => open_risks += 1,
                "mitigated" => mitigated_risks += 1,
                "accepted" => accepted_risks += 1,
                _ => {}
            }
            match level.as_str() {
                "critical" => critical_risks += 1,
                "high" => high_risks += 1,
                "medium" => medium_risks += 1,
                "low" => low_risks += 1,
                _ => {}
            }
            *risks_by_source.entry(source).or_insert(0i32) += 1;
            *risks_by_level.entry(level).or_insert(0i32) += 1;
        }

        // Count controls
        let ctrl_rows = sqlx::query(
            "SELECT status, effectiveness FROM _atlas.control_registry WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut active_controls = 0i32;
        let mut effective_controls = 0i32;
        let mut ineffective_controls = 0i32;
        let mut not_tested_controls = 0i32;
        let mut ctrl_eff = std::collections::HashMap::new();

        for row in &ctrl_rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let eff: String = row.try_get("effectiveness").unwrap_or_default();
            if status == "active" { active_controls += 1; }
            match eff.as_str() {
                "effective" => effective_controls += 1,
                "ineffective" => ineffective_controls += 1,
                "not_tested" => not_tested_controls += 1,
                _ => {}
            }
            *ctrl_eff.entry(eff).or_insert(0i32) += 1;
        }

        // Count tests
        let test_rows = sqlx::query(
            "SELECT result FROM _atlas.control_tests WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut passed_tests = 0i32;
        let mut failed_tests = 0i32;
        for row in &test_rows {
            let result: String = row.try_get("result").unwrap_or_default();
            match result.as_str() {
                "pass" => passed_tests += 1,
                "fail" => failed_tests += 1,
                _ => {}
            }
        }

        // Count issues
        let issue_rows = sqlx::query(
            "SELECT severity, status, remediation_due_date FROM _atlas.risk_issues WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let mut open_issues = 0i32;
        let mut critical_issues = 0i32;
        let mut overdue_remediations = 0i32;
        let today = chrono::Utc::now().date_naive();

        for row in &issue_rows {
            let severity: String = row.try_get("severity").unwrap_or_default();
            let status: String = row.try_get("status").unwrap_or_default();
            let due: Option<chrono::NaiveDate> = row.try_get("remediation_due_date").unwrap_or_default();

            if !matches!(status.as_str(), "resolved" | "closed" | "accepted") {
                open_issues += 1;
            }
            if severity == "critical" { critical_issues += 1; }
            if let Some(d) = due {
                if d < today && !matches!(status.as_str(), "resolved" | "closed") {
                    overdue_remediations += 1;
                }
            }
        }

        Ok(RiskDashboard {
            total_risks: risk_rows.len() as i32,
            open_risks,
            mitigated_risks,
            accepted_risks,
            critical_risks,
            high_risks,
            medium_risks,
            low_risks,
            total_controls: ctrl_rows.len() as i32,
            active_controls,
            effective_controls,
            ineffective_controls,
            not_tested_controls,
            total_tests: test_rows.len() as i32,
            passed_tests,
            failed_tests,
            open_issues,
            critical_issues,
            overdue_remediations,
            risks_by_source: serde_json::to_value(&risks_by_source).unwrap_or_default(),
            risks_by_level: serde_json::to_value(&risks_by_level).unwrap_or_default(),
            control_effectiveness_summary: serde_json::to_value(&ctrl_eff).unwrap_or_default(),
        })
    }
}
