//! Financial Controls Repository
//!
//! PostgreSQL storage for control monitor rules, violations, and dashboards.

use atlas_shared::{
    ControlMonitorRule, ControlViolation, FinancialControlsDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Financial Controls data storage
#[async_trait]
pub trait FinancialControlsRepository: Send + Sync {
    // Monitor Rules
    async fn create_rule(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: &str, risk_level: &str, control_type: &str,
        conditions: serde_json::Value, threshold_value: Option<&str>,
        target_entity: &str, target_fields: serde_json::Value,
        actions: serde_json::Value, auto_resolve: bool,
        check_schedule: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlMonitorRule>;

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ControlMonitorRule>>;
    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<ControlMonitorRule>>;
    async fn list_rules(&self, org_id: Uuid, category: Option<&str>, risk_level: Option<&str>) -> AtlasResult<Vec<ControlMonitorRule>>;
    async fn list_active_rules(&self, org_id: Uuid, check_schedule: Option<&str>) -> AtlasResult<Vec<ControlMonitorRule>>;
    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn update_rule_stats(&self, id: Uuid, total_violations: i32, total_resolved: i32, last_violation_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<()>;

    // Violations
    async fn create_violation(
        &self,
        org_id: Uuid, rule_id: Uuid, rule_code: Option<&str>, rule_name: Option<&str>,
        violation_number: &str, entity_type: &str, entity_id: Option<Uuid>,
        description: &str, findings: serde_json::Value, risk_level: &str,
        status: &str, related_entities: serde_json::Value,
    ) -> AtlasResult<ControlViolation>;

    async fn get_violation(&self, id: Uuid) -> AtlasResult<Option<ControlViolation>>;
    async fn get_violation_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ControlViolation>>;
    async fn list_violations(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>,
        rule_id: Option<Uuid>, assigned_to: Option<Uuid>,
    ) -> AtlasResult<Vec<ControlViolation>>;
    async fn update_violation_status(
        &self, id: Uuid, status: &str, assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>, resolution_notes: Option<&str>,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<ControlViolation>;
    async fn escalate_violation(
        &self, id: Uuid, escalated_to: Option<Uuid>, escalated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ControlViolation>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<FinancialControlsDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresFinancialControlsRepository {
    pool: PgPool,
}

impl PostgresFinancialControlsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

macro_rules! row_to_rule {
    ($row:expr) => {{
        ControlMonitorRule {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            category: $row.get("category"),
            risk_level: $row.get("risk_level"),
            control_type: $row.get("control_type"),
            conditions: $row.get("conditions"),
            threshold_value: $row.get("threshold_value"),
            target_entity: $row.get("target_entity"),
            target_fields: $row.get("target_fields"),
            actions: $row.get("actions"),
            auto_resolve: $row.get("auto_resolve"),
            check_schedule: $row.get("check_schedule"),
            is_active: $row.get("is_active"),
            effective_from: $row.get("effective_from"),
            effective_to: $row.get("effective_to"),
            last_check_at: $row.get("last_check_at"),
            last_violation_at: $row.get("last_violation_at"),
            total_violations: $row.try_get::<i32, _>("total_violations").unwrap_or(0),
            total_resolved: $row.try_get::<i32, _>("total_resolved").unwrap_or(0),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_violation {
    ($row:expr) => {{
        ControlViolation {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            rule_id: $row.get("rule_id"),
            rule_code: $row.get("rule_code"),
            rule_name: $row.get("rule_name"),
            violation_number: $row.get("violation_number"),
            entity_type: $row.get("entity_type"),
            entity_id: $row.get("entity_id"),
            description: $row.get("description"),
            findings: $row.get("findings"),
            risk_level: $row.get("risk_level"),
            status: $row.get("status"),
            assigned_to: $row.get("assigned_to"),
            assigned_to_name: $row.get("assigned_to_name"),
            resolution_notes: $row.get("resolution_notes"),
            resolved_by: $row.get("resolved_by"),
            resolved_at: $row.get("resolved_at"),
            escalated_to: $row.get("escalated_to"),
            escalated_at: $row.get("escalated_at"),
            related_entities: $row.get("related_entities"),
            detected_at: $row.get("detected_at"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

#[async_trait]
impl FinancialControlsRepository for PostgresFinancialControlsRepository {
    async fn create_rule(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: &str, risk_level: &str, control_type: &str,
        conditions: serde_json::Value, threshold_value: Option<&str>,
        target_entity: &str, target_fields: serde_json::Value,
        actions: serde_json::Value, auto_resolve: bool,
        check_schedule: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlMonitorRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.control_monitor_rules
                (organization_id, code, name, description, category, risk_level, control_type,
                 conditions, threshold_value, target_entity, target_fields, actions,
                 auto_resolve, check_schedule, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(category).bind(risk_level).bind(control_type)
        .bind(&conditions).bind(threshold_value).bind(target_entity)
        .bind(&target_fields).bind(&actions).bind(auto_resolve)
        .bind(check_schedule).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule!(row))
    }

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ControlMonitorRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.control_monitor_rules WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule!(r)))
    }

    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<ControlMonitorRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.control_monitor_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule!(r)))
    }

    async fn list_rules(&self, org_id: Uuid, category: Option<&str>, risk_level: Option<&str>) -> AtlasResult<Vec<ControlMonitorRule>> {
        let mut query = String::from("SELECT * FROM _atlas.control_monitor_rules WHERE organization_id = $1 AND is_active = true");
        let mut param_idx = 2;
        if category.is_some() { query.push_str(&format!(" AND category = ${}", param_idx)); param_idx += 1; }
        if risk_level.is_some() { query.push_str(&format!(" AND risk_level = ${}", param_idx)); }
        query.push_str(" ORDER BY risk_level, code");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(c) = category { q = q.bind(c); }
        if let Some(r) = risk_level { q = q.bind(r); }

        let rows = q.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_rule!(r)).collect())
    }

    async fn list_active_rules(&self, org_id: Uuid, check_schedule: Option<&str>) -> AtlasResult<Vec<ControlMonitorRule>> {
        let rows = if let Some(schedule) = check_schedule {
            sqlx::query(
                r#"SELECT * FROM _atlas.control_monitor_rules
                WHERE organization_id = $1 AND is_active = true AND check_schedule = $2
                  AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
                  AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
                ORDER BY risk_level, code"#,
            )
            .bind(org_id).bind(schedule)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                r#"SELECT * FROM _atlas.control_monitor_rules
                WHERE organization_id = $1 AND is_active = true
                  AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
                  AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
                ORDER BY risk_level, code"#,
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_rule!(r)).collect())
    }

    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.control_monitor_rules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_rule_stats(&self, id: Uuid, total_violations: i32, total_resolved: i32, last_violation_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.control_monitor_rules
            SET total_violations = $1, total_resolved = $2, last_violation_at = $3, updated_at = now()
            WHERE id = $4"#,
        )
        .bind(total_violations).bind(total_resolved).bind(last_violation_at).bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_violation(
        &self,
        org_id: Uuid, rule_id: Uuid, rule_code: Option<&str>, rule_name: Option<&str>,
        violation_number: &str, entity_type: &str, entity_id: Option<Uuid>,
        description: &str, findings: serde_json::Value, risk_level: &str,
        status: &str, related_entities: serde_json::Value,
    ) -> AtlasResult<ControlViolation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.control_violations
                (organization_id, rule_id, rule_code, rule_name, violation_number,
                 entity_type, entity_id, description, findings, risk_level,
                 status, related_entities)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *"#,
        )
        .bind(org_id).bind(rule_id).bind(rule_code).bind(rule_name).bind(violation_number)
        .bind(entity_type).bind(entity_id).bind(description).bind(&findings).bind(risk_level)
        .bind(status).bind(&related_entities)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation!(row))
    }

    async fn get_violation(&self, id: Uuid) -> AtlasResult<Option<ControlViolation>> {
        let row = sqlx::query("SELECT * FROM _atlas.control_violations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_violation!(r)))
    }

    async fn get_violation_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ControlViolation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.control_violations WHERE organization_id = $1 AND violation_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_violation!(r)))
    }

    async fn list_violations(
        &self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>,
        rule_id: Option<Uuid>, assigned_to: Option<Uuid>,
    ) -> AtlasResult<Vec<ControlViolation>> {
        let mut query = String::from("SELECT * FROM _atlas.control_violations WHERE organization_id = $1");
        let mut param_idx = 2;
        if status.is_some() { query.push_str(&format!(" AND status = ${}", param_idx)); param_idx += 1; }
        if risk_level.is_some() { query.push_str(&format!(" AND risk_level = ${}", param_idx)); param_idx += 1; }
        if rule_id.is_some() { query.push_str(&format!(" AND rule_id = ${}", param_idx)); param_idx += 1; }
        if assigned_to.is_some() { query.push_str(&format!(" AND assigned_to = ${}", param_idx)); }
        query.push_str(" ORDER BY detected_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(r) = risk_level { q = q.bind(r); }
        if let Some(r) = rule_id { q = q.bind(r); }
        if let Some(a) = assigned_to { q = q.bind(a); }

        let rows = q.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_violation!(r)).collect())
    }

    async fn update_violation_status(
        &self, id: Uuid, status: &str, assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>, resolution_notes: Option<&str>,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<ControlViolation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.control_violations
            SET status = $1, assigned_to = COALESCE($2, assigned_to),
                assigned_to_name = COALESCE($3, assigned_to_name),
                resolution_notes = COALESCE($4, resolution_notes),
                resolved_by = COALESCE($5, resolved_by),
                resolved_at = CASE WHEN $1 IN ('resolved', 'false_positive', 'waived') THEN now() ELSE resolved_at END,
                updated_at = now()
            WHERE id = $6
            RETURNING *"#,
        )
        .bind(status).bind(assigned_to).bind(assigned_to_name)
        .bind(resolution_notes).bind(resolved_by).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation!(row))
    }

    async fn escalate_violation(
        &self, id: Uuid, escalated_to: Option<Uuid>, escalated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ControlViolation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.control_violations
            SET status = 'escalated', escalated_to = $1, escalated_at = $2, updated_at = now()
            WHERE id = $3
            RETURNING *"#,
        )
        .bind(escalated_to).bind(escalated_at).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation!(row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<FinancialControlsDashboardSummary> {
        let rules = sqlx::query(
            "SELECT * FROM _atlas.control_monitor_rules WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let violations = sqlx::query(
            "SELECT * FROM _atlas.control_violations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_rules = rules.len() as i32;
        let active_rules = rules.iter().filter(|r| r.get::<bool, _>("is_active")).count() as i32;
        let total_violations = violations.len() as i32;
        let open_violations = violations.iter().filter(|v| v.get::<String, _>("status") == "open").count() as i32;
        let resolved_violations = violations.iter().filter(|v| v.get::<String, _>("status") == "resolved").count() as i32;
        let escalated_violations = violations.iter().filter(|v| v.get::<String, _>("status") == "escalated").count() as i32;
        let false_positive = violations.iter().filter(|v| v.get::<String, _>("status") == "false_positive").count() as i32;
        let critical = violations.iter().filter(|v| v.get::<String, _>("risk_level") == "critical").count() as i32;
        let high = violations.iter().filter(|v| v.get::<String, _>("risk_level") == "high").count() as i32;
        let medium = violations.iter().filter(|v| v.get::<String, _>("risk_level") == "medium").count() as i32;
        let low = violations.iter().filter(|v| v.get::<String, _>("risk_level") == "low").count() as i32;

        let by_category: serde_json::Value = violations.iter()
            .map(|v| {
                let rule_id: Uuid = v.get("rule_id");
                let cat = rules.iter().find(|r| r.get::<Uuid, _>("id") == rule_id)
                    .map(|r| r.get::<String, _>("category"))
                    .unwrap_or_else(|| "unknown".to_string());
                (cat, 1)
            })
            .fold(std::collections::HashMap::<String, i32>::new(), |mut acc, (k, v)| {
                *acc.entry(k).or_insert(0) += v;
                acc
            })
            .into_iter()
            .map(|(k, v)| serde_json::json!({"category": k, "count": v}))
            .collect();

        let by_rule: serde_json::Value = violations.iter()
            .map(|v| v.get::<String, _>("rule_code"))
            .fold(std::collections::HashMap::<String, i32>::new(), |mut acc, code| {
                *acc.entry(code).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(k, v)| serde_json::json!({"rule": k, "count": v}))
            .collect();

        // Calculate avg resolution time (simplified)
        let avg_resolution: Option<f64> = None; // Would need resolved_at - detected_at

        Ok(FinancialControlsDashboardSummary {
            total_rules,
            active_rules,
            total_violations,
            open_violations,
            resolved_violations,
            escalated_violations,
            false_positive_violations: false_positive,
            critical_violations: critical,
            high_violations: high,
            medium_violations: medium,
            low_violations: low,
            violations_by_category: by_category,
            violations_by_rule: by_rule,
            avg_resolution_time_hours: avg_resolution,
        })
    }
}
