//! Approval Delegation Repository
//!
//! PostgreSQL storage for delegation rules and history.

use atlas_shared::{
    ApprovalDelegationRule, DelegationHistoryEntry, DelegationDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for approval delegation storage
#[async_trait]
pub trait ApprovalDelegationRepository: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    async fn create_rule(
        &self,
        org_id: Uuid,
        delegator_id: Uuid,
        delegate_to_id: Uuid,
        rule_name: &str,
        description: Option<&str>,
        delegation_type: &str,
        categories: serde_json::Value,
        roles: serde_json::Value,
        entity_types: serde_json::Value,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        auto_activate: bool,
        auto_expire: bool,
        status: &str,
        activated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ApprovalDelegationRule>;

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ApprovalDelegationRule>>;
    async fn list_rules_for_delegator(&self, org_id: Uuid, delegator_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ApprovalDelegationRule>>;
    async fn list_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ApprovalDelegationRule>>;
    async fn cancel_rule(&self, id: Uuid, cancelled_by: Uuid, reason: Option<&str>) -> AtlasResult<()>;
    async fn activate_rule(&self, id: Uuid) -> AtlasResult<()>;
    async fn delete_rule(&self, id: Uuid) -> AtlasResult<()>;
    async fn activate_due_rules(&self) -> AtlasResult<Vec<Uuid>>;
    async fn expire_due_rules(&self) -> AtlasResult<Vec<Uuid>>;
    async fn find_active_delegate(&self, org_id: Uuid, approver_id: Uuid, entity_type: Option<&str>, approver_role: Option<&str>) -> AtlasResult<Option<Uuid>>;
    #[allow(clippy::too_many_arguments)]
    async fn record_delegation(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        original_approver_id: Uuid,
        delegated_to_id: Uuid,
        approval_step_id: Option<Uuid>,
        approval_request_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<DelegationHistoryEntry>;
    async fn list_delegation_history(&self, org_id: Uuid, user_id: Uuid, limit: i64) -> AtlasResult<Vec<DelegationHistoryEntry>>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DelegationDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresApprovalDelegationRepository {
    pool: PgPool,
}

impl PostgresApprovalDelegationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_rule(&self, row: &sqlx::postgres::PgRow) -> ApprovalDelegationRule {
        ApprovalDelegationRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            delegator_id: row.get("delegator_id"),
            delegate_to_id: row.get("delegate_to_id"),
            rule_name: row.get("rule_name"),
            description: row.get("description"),
            delegation_type: row.get("delegation_type"),
            categories: row.get("categories"),
            roles: row.get("roles"),
            entity_types: row.get("entity_types"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            is_active: row.get("is_active"),
            auto_activate: row.get("auto_activate"),
            auto_expire: row.get("auto_expire"),
            status: row.get("status"),
            activated_at: row.get("activated_at"),
            expired_at: row.get("expired_at"),
            cancelled_at: row.get("cancelled_at"),
            cancelled_by: row.get("cancelled_by"),
            cancellation_reason: row.get("cancellation_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_history(&self, row: &sqlx::postgres::PgRow) -> DelegationHistoryEntry {
        DelegationHistoryEntry {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            delegation_rule_id: row.get("delegation_rule_id"),
            original_approver_id: row.get("original_approver_id"),
            delegated_to_id: row.get("delegated_to_id"),
            approval_step_id: row.get("approval_step_id"),
            approval_request_id: row.get("approval_request_id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            action_taken: row.get("action_taken"),
            action_at: row.get("action_at"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl ApprovalDelegationRepository for PostgresApprovalDelegationRepository {
    #[allow(clippy::too_many_arguments)]
    async fn create_rule(
        &self,
        org_id: Uuid,
        delegator_id: Uuid,
        delegate_to_id: Uuid,
        rule_name: &str,
        description: Option<&str>,
        delegation_type: &str,
        categories: serde_json::Value,
        roles: serde_json::Value,
        entity_types: serde_json::Value,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        auto_activate: bool,
        auto_expire: bool,
        status: &str,
        activated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<ApprovalDelegationRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.approval_delegation_rules
                (organization_id, delegator_id, delegate_to_id, rule_name, description,
                 delegation_type, categories, roles, entity_types,
                 start_date, end_date, auto_activate, auto_expire, status, activated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(delegator_id)
        .bind(delegate_to_id)
        .bind(rule_name)
        .bind(description)
        .bind(delegation_type)
        .bind(&categories)
        .bind(&roles)
        .bind(&entity_types)
        .bind(start_date)
        .bind(end_date)
        .bind(auto_activate)
        .bind(auto_expire)
        .bind(status)
        .bind(activated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AtlasError::Conflict(format!("Duplicate delegation rule: {}", rule_name))
            } else {
                AtlasError::DatabaseError(e.to_string())
            }
        })?;

        Ok(self.row_to_rule(&row))
    }

    async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ApprovalDelegationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.approval_delegation_rules WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_rule(&r)))
    }

    async fn list_rules_for_delegator(&self, org_id: Uuid, delegator_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ApprovalDelegationRule>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND delegator_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(delegator_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND delegator_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(delegator_id)
            .fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_rule(r)).collect())
    }

    async fn list_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ApprovalDelegationRule>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.approval_delegation_rules WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_rule(r)).collect())
    }

    async fn cancel_rule(&self, id: Uuid, cancelled_by: Uuid, reason: Option<&str>) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.approval_delegation_rules 
            SET status = 'cancelled', is_active = false, cancelled_by = $2, 
                cancellation_reason = $3, cancelled_at = now(), updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id).bind(cancelled_by).bind(reason)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn activate_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_delegation_rules SET status = 'active', is_active = true, activated_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.approval_delegation_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn activate_due_rules(&self) -> AtlasResult<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            UPDATE _atlas.approval_delegation_rules
            SET status = 'active', activated_at = now(), updated_at = now()
            WHERE status = 'scheduled' 
              AND auto_activate = true 
              AND start_date <= CURRENT_DATE
            RETURNING id
            "#
        )
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get::<Uuid, _>("id")).collect())
    }

    async fn expire_due_rules(&self) -> AtlasResult<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            UPDATE _atlas.approval_delegation_rules
            SET status = 'expired', is_active = false, expired_at = now(), updated_at = now()
            WHERE status = 'active' 
              AND auto_expire = true 
              AND end_date < CURRENT_DATE
            RETURNING id
            "#
        )
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get::<Uuid, _>("id")).collect())
    }

    async fn find_active_delegate(
        &self,
        org_id: Uuid,
        approver_id: Uuid,
        entity_type: Option<&str>,
        approver_role: Option<&str>,
    ) -> AtlasResult<Option<Uuid>> {
        // First try to find a rule that delegates ALL approvals
        let row = sqlx::query(
            r#"
            SELECT delegate_to_id FROM _atlas.approval_delegation_rules
            WHERE organization_id = $1 
              AND delegator_id = $2 
              AND status = 'active' 
              AND is_active = true
              AND CURRENT_DATE BETWEEN start_date AND end_date
              AND delegation_type = 'all'
            LIMIT 1
            "#
        )
        .bind(org_id).bind(approver_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if let Some(r) = &row {
            return Ok(Some(r.get::<Uuid, _>("delegate_to_id")));
        }

        // Try by_entity if entity_type provided
        if let Some(et) = entity_type {
            let row = sqlx::query(
                r#"
                SELECT delegate_to_id FROM _atlas.approval_delegation_rules
                WHERE organization_id = $1 
                  AND delegator_id = $2 
                  AND status = 'active' 
                  AND is_active = true
                  AND CURRENT_DATE BETWEEN start_date AND end_date
                  AND delegation_type = 'by_entity'
                  AND entity_types @> $3::jsonb
                LIMIT 1
                "#
            )
            .bind(org_id).bind(approver_id)
            .bind(serde_json::json!([et]))
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

            if let Some(r) = &row {
                return Ok(Some(r.get::<Uuid, _>("delegate_to_id")));
            }
        }

        // Try by_role if role provided
        if let Some(role) = approver_role {
            let row = sqlx::query(
                r#"
                SELECT delegate_to_id FROM _atlas.approval_delegation_rules
                WHERE organization_id = $1 
                  AND delegator_id = $2 
                  AND status = 'active' 
                  AND is_active = true
                  AND CURRENT_DATE BETWEEN start_date AND end_date
                  AND delegation_type = 'by_role'
                  AND roles @> $3::jsonb
                LIMIT 1
                "#
            )
            .bind(org_id).bind(approver_id)
            .bind(serde_json::json!([role]))
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

            if let Some(r) = &row {
                return Ok(Some(r.get::<Uuid, _>("delegate_to_id")));
            }
        }

        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_delegation(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        original_approver_id: Uuid,
        delegated_to_id: Uuid,
        approval_step_id: Option<Uuid>,
        approval_request_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<DelegationHistoryEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.approval_delegation_history
                (organization_id, delegation_rule_id, original_approver_id, delegated_to_id,
                 approval_step_id, approval_request_id, entity_type, entity_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(org_id).bind(rule_id).bind(original_approver_id).bind(delegated_to_id)
        .bind(approval_step_id).bind(approval_request_id).bind(entity_type).bind(entity_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_history(&row))
    }

    async fn list_delegation_history(&self, org_id: Uuid, user_id: Uuid, limit: i64) -> AtlasResult<Vec<DelegationHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.approval_delegation_history
            WHERE organization_id = $1 AND (original_approver_id = $2 OR delegated_to_id = $2)
            ORDER BY created_at DESC
            LIMIT $3
            "#
        )
        .bind(org_id).bind(user_id).bind(limit)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_history(r)).collect())
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DelegationDashboard> {
        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND status = 'active'"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let scheduled_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND status = 'scheduled'"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let expired_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND status = 'expired'"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let today_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.approval_delegation_history WHERE organization_id = $1 AND created_at::date = CURRENT_DATE"
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Delegations by type
        let type_rows = sqlx::query(
            "SELECT delegation_type, COUNT(*) as count FROM _atlas.approval_delegation_rules WHERE organization_id = $1 AND status = 'active' GROUP BY delegation_type"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_type = serde_json::Map::new();
        for row in type_rows {
            let dt: String = row.get("delegation_type");
            let count: i64 = row.get("count");
            by_type.insert(dt, serde_json::Value::Number(count.into()));
        }

        // Recent delegations
        let recent_rows = sqlx::query(
            "SELECT * FROM _atlas.approval_delegation_history WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 10"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(DelegationDashboard {
            total_active_rules: active_count,
            total_scheduled_rules: scheduled_count,
            total_expired_rules: expired_count,
            total_delegations_today: today_count,
            delegations_by_type: serde_json::Value::Object(by_type),
            recent_delegations: recent_rows.iter().map(|r| self.row_to_history(r)).collect(),
        })
    }
}
