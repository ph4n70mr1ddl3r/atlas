//! Cross-Validation Rule Repository
//!
//! PostgreSQL storage for cross-validation rules, lines, and dashboard summary.

use atlas_shared::{
    CrossValidationRule, CrossValidationRuleLine, CrossValidationDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for cross-validation rule storage
#[async_trait]
pub trait CrossValidationRepository: Send + Sync {
    // Rules
    async fn create_rule(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        rule_type: &str, error_message: &str, priority: i32,
        segment_names: Vec<String>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CrossValidationRule>;

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CrossValidationRule>>;
    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<CrossValidationRule>>;
    async fn list_rules(&self, org_id: Uuid, enabled_only: bool) -> AtlasResult<Vec<CrossValidationRule>>;
    async fn update_rule_enabled(&self, id: Uuid, is_enabled: bool) -> AtlasResult<CrossValidationRule>;
    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Lines
    async fn create_rule_line(
        &self,
        org_id: Uuid, rule_id: Uuid, line_type: &str,
        patterns: &[String], display_order: i32,
    ) -> AtlasResult<CrossValidationRuleLine>;

    async fn list_rule_lines(&self, rule_id: Uuid) -> AtlasResult<Vec<CrossValidationRuleLine>>;
    async fn delete_rule_line(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CrossValidationDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresCrossValidationRepository {
    pool: PgPool,
}

impl PostgresCrossValidationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_rule(row: &sqlx::postgres::PgRow) -> CrossValidationRule {
    let segment_names: serde_json::Value = row.get("segment_names");
    let segment_names: Vec<String> = serde_json::from_value(segment_names)
        .unwrap_or_default();

    CrossValidationRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        rule_type: row.get("rule_type"),
        error_message: row.get("error_message"),
        is_enabled: row.get("is_enabled"),
        priority: row.get("priority"),
        segment_names,
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_rule_line(row: &sqlx::postgres::PgRow) -> CrossValidationRuleLine {
    let patterns: serde_json::Value = row.get("patterns");
    let patterns: Vec<String> = serde_json::from_value(patterns)
        .unwrap_or_default();

    CrossValidationRuleLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        rule_id: row.get("rule_id"),
        line_type: row.get("line_type"),
        patterns,
        display_order: row.get("display_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl CrossValidationRepository for PostgresCrossValidationRepository {
    async fn create_rule(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        rule_type: &str, error_message: &str, priority: i32,
        segment_names: Vec<String>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CrossValidationRule> {
        let segment_names_json = serde_json::to_value(&segment_names)
            .map_err(|e| AtlasError::Internal(e.to_string()))?;

        let row = sqlx::query(
            r#"INSERT INTO _atlas.cross_validation_rules
                (organization_id, code, name, description, rule_type, error_message,
                 is_enabled, priority, segment_names, effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,true,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(rule_type).bind(error_message).bind(priority)
        .bind(&segment_names_json)
        .bind(effective_from).bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CrossValidationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cross_validation_rules WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<CrossValidationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cross_validation_rules WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn list_rules(&self, org_id: Uuid, enabled_only: bool) -> AtlasResult<Vec<CrossValidationRule>> {
        let rows = if enabled_only {
            sqlx::query(
                "SELECT * FROM _atlas.cross_validation_rules WHERE organization_id=$1 AND is_enabled=true ORDER BY priority, code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.cross_validation_rules WHERE organization_id=$1 ORDER BY priority, code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_rule).collect())
    }

    async fn update_rule_enabled(&self, id: Uuid, is_enabled: bool) -> AtlasResult<CrossValidationRule> {
        let row = sqlx::query(
            "UPDATE _atlas.cross_validation_rules SET is_enabled=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(is_enabled)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.cross_validation_rules WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_rule_line(
        &self,
        org_id: Uuid, rule_id: Uuid, line_type: &str,
        patterns: &[String], display_order: i32,
    ) -> AtlasResult<CrossValidationRuleLine> {
        let patterns_json = serde_json::to_value(patterns)
            .map_err(|e| AtlasError::Internal(e.to_string()))?;

        let row = sqlx::query(
            r#"INSERT INTO _atlas.cross_validation_rule_lines
                (organization_id, rule_id, line_type, patterns, display_order)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING *"#,
        )
        .bind(org_id).bind(rule_id).bind(line_type)
        .bind(&patterns_json).bind(display_order)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule_line(&row))
    }

    async fn list_rule_lines(&self, rule_id: Uuid) -> AtlasResult<Vec<CrossValidationRuleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cross_validation_rule_lines WHERE rule_id=$1 ORDER BY line_type, display_order"
        )
        .bind(rule_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_rule_line).collect())
    }

    async fn delete_rule_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cross_validation_rule_lines WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CrossValidationDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_enabled) as enabled,
                COUNT(*) FILTER (WHERE rule_type = 'deny') as deny_count,
                COUNT(*) FILTER (WHERE rule_type = 'allow') as allow_count
            FROM _atlas.cross_validation_rules WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let enabled: i64 = row.try_get("enabled").unwrap_or(0);
        let deny_count: i64 = row.try_get("deny_count").unwrap_or(0);
        let allow_count: i64 = row.try_get("allow_count").unwrap_or(0);

        let line_count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM _atlas.cross_validation_rule_lines l
            JOIN _atlas.cross_validation_rules r ON l.rule_id = r.id
            WHERE r.organization_id = $1"#
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(CrossValidationDashboardSummary {
            total_rules: total as i32,
            enabled_rules: enabled as i32,
            deny_rules: deny_count as i32,
            allow_rules: allow_count as i32,
            total_lines: line_count as i32,
            rules_by_type: serde_json::json!({
                "deny": deny_count,
                "allow": allow_count,
            }),
        })
    }
}
