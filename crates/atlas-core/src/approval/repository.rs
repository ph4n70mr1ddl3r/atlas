//! Approval Repository
//!
//! PostgreSQL storage for approval chains, requests, and steps.

use atlas_shared::{ApprovalChain, ApprovalRequest, ApprovalStep, AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for approval storage
#[async_trait]
pub trait ApprovalRepository: Send + Sync {
    // Chain management
    async fn create_chain(&self, chain: ApprovalChain) -> AtlasResult<ApprovalChain>;
    async fn get_chain(&self, id: Uuid) -> AtlasResult<Option<ApprovalChain>>;
    async fn get_chains_for_entity(&self, org_id: Uuid, entity_type: &str) -> AtlasResult<Vec<ApprovalChain>>;
    async fn delete_chain(&self, id: Uuid) -> AtlasResult<()>;

    // Request management
    #[allow(clippy::too_many_arguments)]
    async fn create_request(
        &self,
        org_id: Uuid,
        chain_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        total_levels: i32,
        requested_by: Uuid,
        title: Option<&str>,
        description: Option<&str>,
    ) -> AtlasResult<ApprovalRequest>;
    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ApprovalRequest>>;
    async fn get_requests_for_entity(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<ApprovalRequest>>;
    async fn complete_request(&self, id: Uuid, completed_by: Uuid, status: &str) -> AtlasResult<()>;
    async fn cancel_request(&self, id: Uuid) -> AtlasResult<()>;
    async fn advance_request_level(&self, id: Uuid, new_level: i32) -> AtlasResult<()>;

    // Step management
    #[allow(clippy::too_many_arguments)]
    async fn create_step(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        level: i32,
        approver_type: &str,
        approver_role: Option<&str>,
        approver_user_id: Option<Uuid>,
        auto_approve_after_hours: Option<i32>,
    ) -> AtlasResult<ApprovalStep>;
    async fn get_step(&self, id: Uuid) -> AtlasResult<Option<ApprovalStep>>;
    async fn approve_step(&self, id: Uuid, approved_by: Uuid, comment: Option<&str>) -> AtlasResult<()>;
    async fn reject_step(&self, id: Uuid, rejected_by: Uuid, comment: Option<&str>) -> AtlasResult<()>;
    async fn delegate_step(&self, id: Uuid, delegated_by: Uuid, delegated_to: Uuid) -> AtlasResult<()>;
    async fn get_pending_steps_for_user(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<ApprovalStep>>;
    async fn get_pending_steps_for_role(&self, org_id: Uuid, role: &str) -> AtlasResult<Vec<ApprovalStep>>;
    async fn find_expired_steps(&self) -> AtlasResult<Vec<ApprovalStep>>;
    async fn auto_approve_step(&self, id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresApprovalRepository {
    pool: PgPool,
}

impl PostgresApprovalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_chain(&self, row: &sqlx::postgres::PgRow) -> ApprovalChain {
        ApprovalChain {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            name: row.get("name"),
            description: row.get("description"),
            entity_type: row.get("entity_type"),
            condition_expression: row.get("condition_expression"),
            chain_definition: row.get("chain_definition"),
            escalation_enabled: row.get("escalation_enabled"),
            escalation_hours: row.get("escalation_hours"),
            escalation_to_roles: row.get("escalation_to_roles"),
            allow_delegation: row.get("allow_delegation"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_step(&self, row: &sqlx::postgres::PgRow) -> ApprovalStep {
        ApprovalStep {
            id: row.get("id"),
            approval_request_id: row.get("approval_request_id"),
            level: row.get("level"),
            approver_type: row.get("approver_type"),
            approver_role: row.get("approver_role"),
            approver_user_id: row.get("approver_user_id"),
            is_delegated: row.get("is_delegated"),
            delegated_by: row.get("delegated_by"),
            delegated_to: row.get("delegated_to"),
            status: row.get("status"),
            action_at: row.get("action_at"),
            action_by: row.get("action_by"),
            comment: row.get("comment"),
            auto_approve_after_hours: row.get("auto_approve_after_hours"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl ApprovalRepository for PostgresApprovalRepository {
    async fn create_chain(&self, chain: ApprovalChain) -> AtlasResult<ApprovalChain> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.approval_chains 
                (organization_id, name, description, entity_type, condition_expression, 
                 chain_definition, escalation_enabled, escalation_hours, escalation_to_roles,
                 allow_delegation, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(chain.organization_id)
        .bind(&chain.name)
        .bind(&chain.description)
        .bind(&chain.entity_type)
        .bind(&chain.condition_expression)
        .bind(&chain.chain_definition)
        .bind(chain.escalation_enabled)
        .bind(chain.escalation_hours)
        .bind(&chain.escalation_to_roles)
        .bind(chain.allow_delegation)
        .bind(chain.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_chain(&row))
    }

    async fn get_chain(&self, id: Uuid) -> AtlasResult<Option<ApprovalChain>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.approval_chains WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        
        Ok(row.map(|r| self.row_to_chain(&r)))
    }

    async fn get_chains_for_entity(&self, org_id: Uuid, entity_type: &str) -> AtlasResult<Vec<ApprovalChain>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.approval_chains WHERE organization_id = $1 AND entity_type = $2 AND is_active = true"
        )
        .bind(org_id)
        .bind(entity_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_chain(r)).collect())
    }

    async fn delete_chain(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.approval_chains WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_request(
        &self,
        org_id: Uuid,
        chain_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        total_levels: i32,
        requested_by: Uuid,
        title: Option<&str>,
        description: Option<&str>,
    ) -> AtlasResult<ApprovalRequest> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.approval_requests
                (organization_id, chain_id, entity_type, entity_id, current_level,
                 total_levels, status, requested_by, title, description, metadata)
            VALUES ($1, $2, $3, $4, 1, $5, 'pending', $6, $7, $8, '{}'::jsonb)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(chain_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(total_levels)
        .bind(requested_by)
        .bind(title)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Load steps separately
        let steps_rows = sqlx::query(
            "SELECT * FROM _atlas.approval_steps WHERE approval_request_id = $1 ORDER BY level"
        )
        .bind(row.get::<Uuid, _>("id"))
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(ApprovalRequest {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            chain_id: row.get("chain_id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            current_level: row.get("current_level"),
            total_levels: row.get("total_levels"),
            status: row.get("status"),
            requested_by: row.get("requested_by"),
            requested_at: row.get("requested_at"),
            completed_at: row.get("completed_at"),
            completed_by: row.get("completed_by"),
            title: row.get("title"),
            description: row.get("description"),
            metadata: row.get("metadata"),
            steps: steps_rows.iter().map(|r| self.row_to_step(r)).collect(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ApprovalRequest>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.approval_requests WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let steps_rows = sqlx::query(
                    "SELECT * FROM _atlas.approval_steps WHERE approval_request_id = $1 ORDER BY level"
                )
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default();

                Ok(Some(ApprovalRequest {
                    id: r.get("id"),
                    organization_id: r.get("organization_id"),
                    chain_id: r.get("chain_id"),
                    entity_type: r.get("entity_type"),
                    entity_id: r.get("entity_id"),
                    current_level: r.get("current_level"),
                    total_levels: r.get("total_levels"),
                    status: r.get("status"),
                    requested_by: r.get("requested_by"),
                    requested_at: r.get("requested_at"),
                    completed_at: r.get("completed_at"),
                    completed_by: r.get("completed_by"),
                    title: r.get("title"),
                    description: r.get("description"),
                    metadata: r.get("metadata"),
                    steps: steps_rows.iter().map(|sr| self.row_to_step(sr)).collect(),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_requests_for_entity(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<ApprovalRequest>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.approval_requests WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let steps_rows = sqlx::query(
                "SELECT * FROM _atlas.approval_steps WHERE approval_request_id = $1 ORDER BY level"
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

            results.push(ApprovalRequest {
                id,
                organization_id: row.get("organization_id"),
                chain_id: row.get("chain_id"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                current_level: row.get("current_level"),
                total_levels: row.get("total_levels"),
                status: row.get("status"),
                requested_by: row.get("requested_by"),
                requested_at: row.get("requested_at"),
                completed_at: row.get("completed_at"),
                completed_by: row.get("completed_by"),
                title: row.get("title"),
                description: row.get("description"),
                metadata: row.get("metadata"),
                steps: steps_rows.iter().map(|sr| self.row_to_step(sr)).collect(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(results)
    }

    async fn complete_request(&self, id: Uuid, completed_by: Uuid, status: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_requests SET status = $2, completed_at = now(), completed_by = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(status).bind(completed_by)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn cancel_request(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_requests SET status = 'cancelled', completed_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn advance_request_level(&self, id: Uuid, new_level: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_requests SET current_level = $2, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(new_level)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_step(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        level: i32,
        approver_type: &str,
        approver_role: Option<&str>,
        approver_user_id: Option<Uuid>,
        auto_approve_after_hours: Option<i32>,
    ) -> AtlasResult<ApprovalStep> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.approval_steps
                (organization_id, approval_request_id, level, approver_type, 
                 approver_role, approver_user_id, auto_approve_after_hours, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
            RETURNING *
            "#
        )
        .bind(org_id).bind(request_id).bind(level).bind(approver_type)
        .bind(approver_role).bind(approver_user_id).bind(auto_approve_after_hours)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_step(&row))
    }

    async fn get_step(&self, id: Uuid) -> AtlasResult<Option<ApprovalStep>> {
        let row = sqlx::query("SELECT * FROM _atlas.approval_steps WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_step(&r)))
    }

    async fn approve_step(&self, id: Uuid, approved_by: Uuid, comment: Option<&str>) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_steps SET status = 'approved', action_at = now(), action_by = $2, comment = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(approved_by).bind(comment)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn reject_step(&self, id: Uuid, rejected_by: Uuid, comment: Option<&str>) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_steps SET status = 'rejected', action_at = now(), action_by = $2, comment = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(rejected_by).bind(comment)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delegate_step(&self, id: Uuid, delegated_by: Uuid, delegated_to: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_steps SET is_delegated = true, delegated_by = $2, delegated_to = $3, approver_user_id = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(delegated_by).bind(delegated_to)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_pending_steps_for_user(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<ApprovalStep>> {
        let rows = sqlx::query(
            "SELECT s.* FROM _atlas.approval_steps s 
             JOIN _atlas.approval_requests r ON s.approval_request_id = r.id
             WHERE r.organization_id = $1 AND s.approver_user_id = $2 AND s.status = 'pending' AND r.status = 'pending'
             ORDER BY s.level ASC"
        )
        .bind(org_id).bind(user_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_step(r)).collect())
    }

    async fn get_pending_steps_for_role(&self, org_id: Uuid, role: &str) -> AtlasResult<Vec<ApprovalStep>> {
        let rows = sqlx::query(
            "SELECT s.* FROM _atlas.approval_steps s 
             JOIN _atlas.approval_requests r ON s.approval_request_id = r.id
             WHERE r.organization_id = $1 AND s.approver_role = $2 AND s.status = 'pending' AND r.status = 'pending'
             ORDER BY s.level ASC"
        )
        .bind(org_id).bind(role)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_step(r)).collect())
    }

    async fn find_expired_steps(&self) -> AtlasResult<Vec<ApprovalStep>> {
        let rows = sqlx::query(
            r#"
            SELECT s.* FROM _atlas.approval_steps s
            JOIN _atlas.approval_requests r ON s.approval_request_id = r.id
            WHERE s.status = 'pending' 
              AND s.auto_approve_after_hours IS NOT NULL
              AND s.created_at < now() - (s.auto_approve_after_hours || ' hours')::interval
              AND r.status = 'pending'
            "#
        )
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_step(r)).collect())
    }

    async fn auto_approve_step(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.approval_steps SET status = 'timed_out', action_at = now(), updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}