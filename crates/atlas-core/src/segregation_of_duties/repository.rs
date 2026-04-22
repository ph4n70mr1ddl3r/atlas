//! Segregation of Duties Repository
//!
//! PostgreSQL storage for SoD rules, violations, mitigating controls,
//! and role assignments.

use atlas_shared::{
    SodRule, SodViolation, SodMitigatingControl, SodRoleAssignment, SodDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for Segregation of Duties data storage
#[async_trait]
pub trait SegregationOfDutiesRepository: Send + Sync {
    // ========================================================================
    // SoD Rules
    // ========================================================================

    async fn create_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        first_duties: Vec<String>,
        second_duties: Vec<String>,
        enforcement_mode: &str,
        risk_level: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SodRule>;

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SodRule>>;
    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<SodRule>>;
    async fn list_rules(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SodRule>>;
    async fn update_rule_status(&self, id: Uuid, is_active: bool) -> AtlasResult<SodRule>;
    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Role Assignments
    // ========================================================================

    async fn create_role_assignment(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role_name: &str,
        duty_code: &str,
        assigned_by: Option<Uuid>,
    ) -> AtlasResult<SodRoleAssignment>;

    async fn get_role_assignments_for_user(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<SodRoleAssignment>>;
    async fn list_role_assignments(&self, org_id: Uuid, user_id: Option<Uuid>) -> AtlasResult<Vec<SodRoleAssignment>>;
    async fn deactivate_role_assignment(&self, id: Uuid) -> AtlasResult<SodRoleAssignment>;

    // ========================================================================
    // Violations
    // ========================================================================

    async fn create_violation(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        rule_code: &str,
        user_id: Uuid,
        first_matched_duties: Vec<String>,
        second_matched_duties: Vec<String>,
    ) -> AtlasResult<SodViolation>;

    async fn get_violation(&self, id: Uuid) -> AtlasResult<Option<SodViolation>>;
    async fn list_violations(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        status: Option<&str>,
        risk_level: Option<&str>,
    ) -> AtlasResult<Vec<SodViolation>>;
    async fn update_violation_status(
        &self,
        id: Uuid,
        status: &str,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<SodViolation>;
    async fn find_existing_violation(
        &self,
        rule_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<Option<SodViolation>>;

    // ========================================================================
    // Mitigating Controls
    // ========================================================================

    async fn create_mitigating_control(
        &self,
        org_id: Uuid,
        violation_id: Uuid,
        control_name: &str,
        control_description: &str,
        control_owner_id: Option<Uuid>,
        review_frequency: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SodMitigatingControl>;

    async fn get_mitigating_controls_for_violation(&self, violation_id: Uuid) -> AtlasResult<Vec<SodMitigatingControl>>;
    async fn list_mitigating_controls(&self, org_id: Uuid) -> AtlasResult<Vec<SodMitigatingControl>>;
    async fn approve_mitigating_control(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<SodMitigatingControl>;
    async fn revoke_mitigating_control(&self, id: Uuid) -> AtlasResult<SodMitigatingControl>;

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SodDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresSegregationOfDutiesRepository {
    pool: PgPool,
}

impl PostgresSegregationOfDutiesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_rule(row: &sqlx::postgres::PgRow) -> SodRule {
    SodRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        first_duties: row.get("first_duties"),
        second_duties: row.get("second_duties"),
        enforcement_mode: row.get("enforcement_mode"),
        risk_level: row.get("risk_level"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_violation(row: &sqlx::postgres::PgRow) -> SodViolation {
    SodViolation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        rule_id: row.get("rule_id"),
        rule_code: row.get("rule_code"),
        user_id: row.get("user_id"),
        first_matched_duties: row.get("first_matched_duties"),
        second_matched_duties: row.get("second_matched_duties"),
        violation_status: row.get("violation_status"),
        detected_at: row.get("detected_at"),
        resolved_at: row.get("resolved_at"),
        resolved_by: row.get("resolved_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_mitigation(row: &sqlx::postgres::PgRow) -> SodMitigatingControl {
    SodMitigatingControl {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        violation_id: row.get("violation_id"),
        control_name: row.get("control_name"),
        control_description: row.get("control_description"),
        control_owner_id: row.get("control_owner_id"),
        review_frequency: row.get("review_frequency"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        status: row.get("status"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_assignment(row: &sqlx::postgres::PgRow) -> SodRoleAssignment {
    SodRoleAssignment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        user_id: row.get("user_id"),
        role_name: row.get("role_name"),
        duty_code: row.get("duty_code"),
        assigned_by: row.get("assigned_by"),
        assigned_at: row.get("assigned_at"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl SegregationOfDutiesRepository for PostgresSegregationOfDutiesRepository {
    async fn create_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        first_duties: Vec<String>,
        second_duties: Vec<String>,
        enforcement_mode: &str,
        risk_level: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SodRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sod_rules
                (organization_id, code, name, description, first_duties, second_duties,
                 enforcement_mode, risk_level, effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(serde_json::json!(first_duties)).bind(serde_json::json!(second_duties))
        .bind(enforcement_mode).bind(risk_level)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<SodRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.sod_rules WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<SodRule>> {
        let row = sqlx::query("SELECT * FROM _atlas.sod_rules WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn list_rules(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SodRule>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.sod_rules WHERE organization_id=$1 AND is_active=true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.sod_rules WHERE organization_id=$1 ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(|r| row_to_rule(r)).collect())
    }

    async fn update_rule_status(&self, id: Uuid, is_active: bool) -> AtlasResult<SodRule> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sod_rules SET is_active=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.sod_rules WHERE organization_id=$1 AND code=$2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_role_assignment(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role_name: &str,
        duty_code: &str,
        assigned_by: Option<Uuid>,
    ) -> AtlasResult<SodRoleAssignment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sod_role_assignments
                (organization_id, user_id, role_name, duty_code, assigned_by)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING *"#,
        )
        .bind(org_id).bind(user_id).bind(role_name).bind(duty_code).bind(assigned_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_assignment(&row))
    }

    async fn get_role_assignments_for_user(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<SodRoleAssignment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sod_role_assignments WHERE organization_id=$1 AND user_id=$2 AND is_active=true ORDER BY duty_code"
        )
        .bind(org_id).bind(user_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_assignment(r)).collect())
    }

    async fn list_role_assignments(&self, org_id: Uuid, user_id: Option<Uuid>) -> AtlasResult<Vec<SodRoleAssignment>> {
        let rows = if user_id.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.sod_role_assignments WHERE organization_id=$1 AND user_id=$2 AND is_active=true ORDER BY user_id, duty_code"
            )
            .bind(org_id).bind(user_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.sod_role_assignments WHERE organization_id=$1 AND is_active=true ORDER BY user_id, duty_code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(|r| row_to_assignment(r)).collect())
    }

    async fn deactivate_role_assignment(&self, id: Uuid) -> AtlasResult<SodRoleAssignment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sod_role_assignments SET is_active=false, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_assignment(&row))
    }

    async fn create_violation(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        rule_code: &str,
        user_id: Uuid,
        first_matched_duties: Vec<String>,
        second_matched_duties: Vec<String>,
    ) -> AtlasResult<SodViolation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sod_violations
                (organization_id, rule_id, rule_code, user_id,
                 first_matched_duties, second_matched_duties)
            VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING *"#,
        )
        .bind(org_id).bind(rule_id).bind(rule_code).bind(user_id)
        .bind(serde_json::json!(first_matched_duties))
        .bind(serde_json::json!(second_matched_duties))
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation(&row))
    }

    async fn get_violation(&self, id: Uuid) -> AtlasResult<Option<SodViolation>> {
        let row = sqlx::query("SELECT * FROM _atlas.sod_violations WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_violation(&r)))
    }

    async fn list_violations(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        status: Option<&str>,
        risk_level: Option<&str>,
    ) -> AtlasResult<Vec<SodViolation>> {
        let rows = sqlx::query(
            r#"SELECT v.* FROM _atlas.sod_violations v
            LEFT JOIN _atlas.sod_rules r ON v.rule_id = r.id
            WHERE v.organization_id=$1
              AND ($2::uuid IS NULL OR v.user_id=$2)
              AND ($3::text IS NULL OR v.violation_status=$3)
              AND ($4::text IS NULL OR r.risk_level=$4)
            ORDER BY v.detected_at DESC"#,
        )
        .bind(org_id).bind(user_id).bind(status).bind(risk_level)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_violation(r)).collect())
    }

    async fn update_violation_status(
        &self,
        id: Uuid,
        status: &str,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<SodViolation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sod_violations
            SET violation_status=$2, resolved_by=$3,
                resolved_at=CASE WHEN $2 IN ('resolved','mitigated','exception') THEN now() ELSE resolved_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(resolved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation(&row))
    }

    async fn find_existing_violation(
        &self,
        rule_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<Option<SodViolation>> {
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.sod_violations
            WHERE rule_id=$1 AND user_id=$2 AND violation_status='open'"#
        )
        .bind(rule_id).bind(user_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_violation(&r)))
    }

    async fn create_mitigating_control(
        &self,
        org_id: Uuid,
        violation_id: Uuid,
        control_name: &str,
        control_description: &str,
        control_owner_id: Option<Uuid>,
        review_frequency: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SodMitigatingControl> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sod_mitigating_controls
                (organization_id, violation_id, control_name, control_description,
                 control_owner_id, review_frequency, effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(violation_id).bind(control_name).bind(control_description)
        .bind(control_owner_id).bind(review_frequency)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_mitigation(&row))
    }

    async fn get_mitigating_controls_for_violation(&self, violation_id: Uuid) -> AtlasResult<Vec<SodMitigatingControl>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sod_mitigating_controls WHERE violation_id=$1 ORDER BY created_at"
        )
        .bind(violation_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_mitigation(r)).collect())
    }

    async fn list_mitigating_controls(&self, org_id: Uuid) -> AtlasResult<Vec<SodMitigatingControl>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.sod_mitigating_controls WHERE organization_id=$1 ORDER BY created_at DESC"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_mitigation(r)).collect())
    }

    async fn approve_mitigating_control(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<SodMitigatingControl> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sod_mitigating_controls
            SET approved_by=$2, approved_at=now(), status='active', updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_mitigation(&row))
    }

    async fn revoke_mitigating_control(&self, id: Uuid) -> AtlasResult<SodMitigatingControl> {
        let row = sqlx::query(
            r#"UPDATE _atlas.sod_mitigating_controls
            SET status='revoked', updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_mitigation(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SodDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active = true) as active
            FROM _atlas.sod_rules WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_rules: i64 = row.try_get("total").unwrap_or(0);
        let active_rules: i64 = row.try_get("active").unwrap_or(0);

        let vrow = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE violation_status = 'open') as open_v,
                COUNT(*) FILTER (WHERE violation_status = 'mitigated') as mitigated,
                COUNT(*) FILTER (WHERE violation_status = 'exception') as exception_v,
                COUNT(*) FILTER (WHERE violation_status = 'open' AND r.risk_level = 'high') as high_risk,
                COUNT(*) FILTER (WHERE violation_status = 'open' AND r.risk_level = 'medium') as medium_risk,
                COUNT(*) FILTER (WHERE violation_status = 'open' AND r.risk_level = 'low') as low_risk
            FROM _atlas.sod_violations v
            LEFT JOIN _atlas.sod_rules r ON v.rule_id = r.id
            WHERE v.organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_violations: i64 = vrow.try_get("total").unwrap_or(0);
        let open_violations: i64 = vrow.try_get("open_v").unwrap_or(0);
        let mitigated_violations: i64 = vrow.try_get("mitigated").unwrap_or(0);
        let exception_violations: i64 = vrow.try_get("exception_v").unwrap_or(0);
        let high_risk: i64 = vrow.try_get("high_risk").unwrap_or(0);
        let medium_risk: i64 = vrow.try_get("medium_risk").unwrap_or(0);
        let low_risk: i64 = vrow.try_get("low_risk").unwrap_or(0);

        let recent_rows = sqlx::query(
            r#"SELECT v.* FROM _atlas.sod_violations v
            WHERE v.organization_id=$1
            ORDER BY v.detected_at DESC LIMIT 10"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(SodDashboardSummary {
            total_rules: total_rules as i32,
            active_rules: active_rules as i32,
            total_violations: total_violations as i32,
            open_violations: open_violations as i32,
            mitigated_violations: mitigated_violations as i32,
            exception_violations: exception_violations as i32,
            violations_by_risk_level: serde_json::json!({
                "high": high_risk,
                "medium": medium_risk,
                "low": low_risk,
            }),
            recent_violations: recent_rows.iter().map(|r| row_to_violation(r)).collect(),
            rules_summary: serde_json::json!({}),
        })
    }
}
