//! Approval Authority Limits Repository
//!
//! PostgreSQL storage for approval authority limits and check audit trail.

use atlas_shared::{
    ApprovalAuthorityLimit, AuthorityCheckAudit,
    ApprovalAuthorityDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for approval authority data storage
#[async_trait]
pub trait ApprovalAuthorityRepository: Send + Sync {
    // Limit CRUD
    #[allow(clippy::too_many_arguments)]
    async fn create_limit(
        &self,
        org_id: Uuid,
        limit_code: &str,
        name: &str,
        description: Option<&str>,
        owner_type: &str,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
        document_type: &str,
        approval_limit_amount: &str,
        currency_code: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApprovalAuthorityLimit>;

    async fn get_limit(&self, id: Uuid) -> AtlasResult<Option<ApprovalAuthorityLimit>>;
    async fn get_limit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ApprovalAuthorityLimit>>;
    async fn list_limits(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        owner_type: Option<&str>,
        document_type: Option<&str>,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalAuthorityLimit>>;
    async fn update_limit_status(&self, id: Uuid, status: &str) -> AtlasResult<ApprovalAuthorityLimit>;
    async fn delete_limit(&self, id: Uuid) -> AtlasResult<()>;

    /// Find applicable limits matching the given criteria (used by engine for resolution).
    #[allow(clippy::too_many_arguments)]
    async fn find_applicable_limits(
        &self,
        org_id: Uuid,
        owner_type: &str,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
        document_type: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalAuthorityLimit>>;

    // Check audit
    #[allow(clippy::too_many_arguments)]
    async fn create_check_audit(
        &self,
        org_id: Uuid,
        limit_id: Option<Uuid>,
        checked_user_id: Uuid,
        checked_role: Option<&str>,
        document_type: &str,
        document_id: Option<Uuid>,
        requested_amount: &str,
        applicable_limit: &str,
        result: &str,
        reason: Option<&str>,
    ) -> AtlasResult<AuthorityCheckAudit>;

    async fn list_check_audits(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        document_type: Option<&str>,
        result: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<AuthorityCheckAudit>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ApprovalAuthorityDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresApprovalAuthorityRepository {
    pool: PgPool,
}

impl PostgresApprovalAuthorityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_limit(row: &sqlx::postgres::PgRow) -> ApprovalAuthorityLimit {
    ApprovalAuthorityLimit {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        limit_code: row.get("limit_code"),
        name: row.get("name"),
        description: row.get("description"),
        owner_type: row.get("owner_type"),
        user_id: row.get("user_id"),
        role_name: row.get("role_name"),
        document_type: row.get("document_type"),
        approval_limit_amount: row.get("approval_limit_amount"),
        currency_code: row.get("currency_code"),
        business_unit_id: row.get("business_unit_id"),
        cost_center: row.get("cost_center"),
        status: row.get("status"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_audit(row: &sqlx::postgres::PgRow) -> AuthorityCheckAudit {
    AuthorityCheckAudit {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        limit_id: row.get("limit_id"),
        checked_user_id: row.get("checked_user_id"),
        checked_role: row.get("checked_role"),
        document_type: row.get("document_type"),
        document_id: row.get("document_id"),
        requested_amount: row.get("requested_amount"),
        applicable_limit: row.get("applicable_limit"),
        result: row.get("result"),
        reason: row.get("reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl ApprovalAuthorityRepository for PostgresApprovalAuthorityRepository {
    async fn create_limit(
        &self,
        org_id: Uuid,
        limit_code: &str,
        name: &str,
        description: Option<&str>,
        owner_type: &str,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
        document_type: &str,
        approval_limit_amount: &str,
        currency_code: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApprovalAuthorityLimit> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.approval_authority_limits
                (organization_id, limit_code, name, description,
                 owner_type, user_id, role_name, document_type,
                 approval_limit_amount, currency_code,
                 business_unit_id, cost_center,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(limit_code).bind(name).bind(description)
        .bind(owner_type).bind(user_id).bind(role_name)
        .bind(document_type).bind(approval_limit_amount).bind(currency_code)
        .bind(business_unit_id).bind(cost_center)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_limit(&row))
    }

    async fn get_limit(&self, id: Uuid) -> AtlasResult<Option<ApprovalAuthorityLimit>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.approval_authority_limits WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_limit(&r)))
    }

    async fn get_limit_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ApprovalAuthorityLimit>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.approval_authority_limits WHERE organization_id = $1 AND limit_code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_limit(&r)))
    }

    async fn list_limits(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        owner_type: Option<&str>,
        document_type: Option<&str>,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalAuthorityLimit>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.approval_authority_limits
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR owner_type = $3)
              AND ($4::text IS NULL OR document_type = $4)
              AND ($5::uuid IS NULL OR user_id = $5)
              AND ($6::text IS NULL OR role_name = $6)
            ORDER BY limit_code"#,
        )
        .bind(org_id).bind(status).bind(owner_type)
        .bind(document_type).bind(user_id).bind(role_name)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_limit).collect())
    }

    async fn update_limit_status(&self, id: Uuid, status: &str) -> AtlasResult<ApprovalAuthorityLimit> {
        let row = sqlx::query(
            r#"UPDATE _atlas.approval_authority_limits
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_limit(&row))
    }

    async fn delete_limit(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.approval_authority_limits WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_applicable_limits(
        &self,
        org_id: Uuid,
        owner_type: &str,
        user_id: Option<Uuid>,
        role_name: Option<&str>,
        document_type: &str,
        business_unit_id: Option<Uuid>,
        cost_center: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalAuthorityLimit>> {
        // Fetch limits matching the owner and document type.
        // BU-scoped limits are preferred; global limits (no BU) are fallback.
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.approval_authority_limits
            WHERE organization_id = $1
              AND owner_type = $2
              AND (
                  ($3::uuid IS NOT NULL AND user_id = $3)
                  OR
                  ($4::text IS NOT NULL AND role_name = $4)
              )
              AND document_type = $5
              AND (
                  business_unit_id IS NULL
                  OR ($6::uuid IS NOT NULL AND business_unit_id = $6)
              )
              AND (
                  cost_center IS NULL
                  OR ($7::text IS NOT NULL AND cost_center = $7)
              )
            ORDER BY approval_limit_amount DESC"#,
        )
        .bind(org_id).bind(owner_type)
        .bind(user_id).bind(role_name)
        .bind(document_type).bind(business_unit_id).bind(cost_center)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_limit).collect())
    }

    async fn create_check_audit(
        &self,
        org_id: Uuid,
        limit_id: Option<Uuid>,
        checked_user_id: Uuid,
        checked_role: Option<&str>,
        document_type: &str,
        document_id: Option<Uuid>,
        requested_amount: &str,
        applicable_limit: &str,
        result: &str,
        reason: Option<&str>,
    ) -> AtlasResult<AuthorityCheckAudit> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.approval_authority_check_audit
                (organization_id, limit_id, checked_user_id, checked_role,
                 document_type, document_id, requested_amount,
                 applicable_limit, result, reason)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"#,
        )
        .bind(org_id).bind(limit_id).bind(checked_user_id).bind(checked_role)
        .bind(document_type).bind(document_id).bind(requested_amount)
        .bind(applicable_limit).bind(result).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_audit(&row))
    }

    async fn list_check_audits(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        document_type: Option<&str>,
        result: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<AuthorityCheckAudit>> {
        let limit_val = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.approval_authority_check_audit
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR checked_user_id = $2)
              AND ($3::text IS NULL OR document_type = $3)
              AND ($4::text IS NULL OR result = $4)
            ORDER BY created_at DESC
            LIMIT $5"#,
        )
        .bind(org_id).bind(user_id).bind(document_type).bind(result)
        .bind(limit_val)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_audit).collect())
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ApprovalAuthorityDashboard> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE owner_type = 'user') as user_limits,
                COUNT(*) FILTER (WHERE owner_type = 'role') as role_limits
            FROM _atlas.approval_authority_limits WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let active: i64 = row.try_get("active").unwrap_or(0);

        // By document type
        let dt_rows = sqlx::query(
            r#"SELECT document_type, COUNT(*) as cnt
            FROM _atlas.approval_authority_limits
            WHERE organization_id = $1 AND status = 'active'
            GROUP BY document_type"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_doc_type = serde_json::Map::new();
        for r in &dt_rows {
            let dt: String = r.get("document_type");
            let cnt: i64 = r.get("cnt");
            by_doc_type.insert(dt, serde_json::Value::Number(cnt.into()));
        }

        // By owner type
        let ot_rows = sqlx::query(
            r#"SELECT owner_type, COUNT(*) as cnt
            FROM _atlas.approval_authority_limits
            WHERE organization_id = $1 AND status = 'active'
            GROUP BY owner_type"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_owner = serde_json::Map::new();
        for r in &ot_rows {
            let ot: String = r.get("owner_type");
            let cnt: i64 = r.get("cnt");
            by_owner.insert(ot, serde_json::Value::Number(cnt.into()));
        }

        // Check audit stats
        let audit_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total_checks,
                COUNT(*) FILTER (WHERE result = 'approved') as approved_checks,
                COUNT(*) FILTER (WHERE result = 'denied') as denied_checks
            FROM _atlas.approval_authority_check_audit WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_checks: i64 = audit_row.try_get("total_checks").unwrap_or(0);
        let approved_checks: i64 = audit_row.try_get("approved_checks").unwrap_or(0);
        let denied_checks: i64 = audit_row.try_get("denied_checks").unwrap_or(0);

        // Recent checks
        let recent_rows = sqlx::query(
            r#"SELECT * FROM _atlas.approval_authority_check_audit
            WHERE organization_id = $1
            ORDER BY created_at DESC LIMIT 10"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(ApprovalAuthorityDashboard {
            total_limits: total,
            active_limits: active,
            limits_by_document_type: serde_json::Value::Object(by_doc_type),
            limits_by_owner_type: serde_json::Value::Object(by_owner),
            recent_checks: recent_rows.iter().map(row_to_audit).collect(),
            total_checks,
            approved_checks,
            denied_checks,
        })
    }
}
