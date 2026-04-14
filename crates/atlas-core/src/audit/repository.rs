//! Audit Repository

use atlas_shared::{AuditEntry, AuditAction, AtlasResult};
use super::AuditQuery;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for audit entries
#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn insert(&self, entry: &AuditEntry) -> AtlasResult<()>;
    async fn query(&self, query: &AuditQuery) -> AtlasResult<Vec<AuditEntry>>;
    async fn get_by_id(&self, id: Uuid) -> AtlasResult<Option<AuditEntry>>;
    async fn get_by_ids(&self, ids: &[Uuid]) -> AtlasResult<Vec<AuditEntry>>;
}

/// PostgreSQL implementation of AuditRepository
pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn insert(&self, entry: &AuditEntry) -> AtlasResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _atlas.audit_log (
                id, entity_type, entity_id, action,
                old_data, new_data, changed_by, changed_at,
                session_id, ip_address, user_agent
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(entry.id)
        .bind(&entry.entity_type)
        .bind(entry.entity_id)
        .bind(format!("{:?}", entry.action))
        .bind(&entry.old_data)
        .bind(&entry.new_data)
        .bind(entry.changed_by)
        .bind(entry.changed_at)
        .bind(entry.session_id)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn query(&self, query: &AuditQuery) -> AtlasResult<Vec<AuditEntry>> {
        let mut sql = String::from(
            "SELECT * FROM _atlas.audit_log WHERE 1=1"
        );
        
        let mut param_idx = 1u32;
        
        if query.entity_type.is_some() {
            sql.push_str(&format!(" AND entity_type = ${}", param_idx));
            param_idx += 1;
        }
        if query.entity_id.is_some() {
            sql.push_str(&format!(" AND entity_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.user_id.is_some() {
            sql.push_str(&format!(" AND changed_by = ${}", param_idx));
            param_idx += 1;
        }
        if query.from_date.is_some() {
            sql.push_str(&format!(" AND changed_at >= ${}", param_idx));
            param_idx += 1;
        }
        if query.to_date.is_some() {
            sql.push_str(&format!(" AND changed_at <= ${}", param_idx));
            param_idx += 1;
        }
        
        sql.push_str(" ORDER BY changed_at DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        let mut q = sqlx::query_as::<_, AuditLogRow>(&sql);
        
        if let Some(ref entity_type) = query.entity_type {
            q = q.bind(entity_type);
        }
        if let Some(entity_id) = query.entity_id {
            q = q.bind(entity_id);
        }
        if let Some(user_id) = query.user_id {
            q = q.bind(user_id);
        }
        if let Some(from_date) = query.from_date {
            q = q.bind(from_date);
        }
        if let Some(to_date) = query.to_date {
            q = q.bind(to_date);
        }
        
        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn get_by_id(&self, id: Uuid) -> AtlasResult<Option<AuditEntry>> {
        let row = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM _atlas.audit_log WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn get_by_ids(&self, ids: &[Uuid]) -> AtlasResult<Vec<AuditEntry>> {
        let entries = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM _atlas.audit_log WHERE id = ANY($1) ORDER BY changed_at"
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(entries.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    action: String,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
    changed_by: Option<Uuid>,
    changed_at: DateTime<Utc>,
    session_id: Option<Uuid>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl From<AuditLogRow> for AuditEntry {
    fn from(row: AuditLogRow) -> Self {
        let action = match row.action.as_str() {
            "Create" => AuditAction::Create,
            "Read" => AuditAction::Read,
            "Update" => AuditAction::Update,
            "Delete" => AuditAction::Delete,
            "ExecuteAction" => AuditAction::ExecuteAction,
            "Login" => AuditAction::Login,
            "Logout" => AuditAction::Logout,
            _ => AuditAction::Update,
        };
        
        AuditEntry {
            id: row.id,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            action,
            old_data: row.old_data,
            new_data: row.new_data,
            changed_by: row.changed_by,
            changed_at: row.changed_at,
            session_id: row.session_id,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
        }
    }
}
