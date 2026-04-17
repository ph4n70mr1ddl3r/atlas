//! Notification Repository
//!
//! PostgreSQL-backed storage for notifications.

use atlas_shared::{Notification, CreateNotificationRequest, AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for notification storage
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, org_id: Uuid, request: CreateNotificationRequest) -> AtlasResult<Notification>;
    async fn mark_read(&self, notification_id: Uuid) -> AtlasResult<()>;
    async fn mark_all_read(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<u64>;
    async fn dismiss(&self, notification_id: Uuid) -> AtlasResult<()>;
    async fn unread_count(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<i64>;
    async fn list(&self, org_id: Uuid, user_id: Uuid, include_read: bool, limit: i64, offset: i64) -> AtlasResult<Vec<Notification>>;
    async fn get_users_by_role(&self, org_id: Uuid, role: &str) -> AtlasResult<Vec<Uuid>>;
    async fn cleanup_expired(&self) -> AtlasResult<u64>;
    async fn send_scheduled(&self) -> AtlasResult<u64>;
}

/// PostgreSQL implementation of notification storage
pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Map a database row to a Notification struct
    fn row_to_notification(row: &sqlx::postgres::PgRow) -> Result<Notification, sqlx::Error> {
        use sqlx::Row;
        Ok(Notification {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            user_id: row.try_get("user_id")?,
            notification_type: row.try_get("notification_type")?,
            priority: row.try_get("priority")?,
            title: row.try_get("title")?,
            message: row.try_get("message")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            action_url: row.try_get("action_url")?,
            workflow_name: row.try_get("workflow_name")?,
            from_state: row.try_get("from_state")?,
            to_state: row.try_get("to_state")?,
            action: row.try_get("action")?,
            performed_by: row.try_get("performed_by")?,
            is_read: row.try_get("is_read")?,
            read_at: row.try_get("read_at")?,
            is_dismissed: row.try_get("is_dismissed")?,
            dismissed_at: row.try_get("dismissed_at")?,
            channels: row.try_get("channels")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            expires_at: row.try_get("expires_at")?,
        })
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(&self, org_id: Uuid, request: CreateNotificationRequest) -> AtlasResult<Notification> {
        let user_id = request.user_id.unwrap_or(Uuid::nil());
        let priority = request.priority.unwrap_or_else(|| "normal".to_string());
        let channels = request.channels.unwrap_or_else(|| serde_json::json!(["in_app"]));

        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.notifications 
                (organization_id, user_id, notification_type, priority, title, message,
                 entity_type, entity_id, action_url, workflow_name, from_state, to_state,
                 action, performed_by, channels, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .bind(&request.notification_type)
        .bind(&priority)
        .bind(&request.title)
        .bind(&request.message)
        .bind(&request.entity_type)
        .bind(request.entity_id)
        .bind(&request.action_url)
        .bind(&request.workflow_name)
        .bind(&request.from_state)
        .bind(&request.to_state)
        .bind(&request.action)
        .bind(request.performed_by)
        .bind(&channels)
        .bind(request.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Self::row_to_notification(&row).map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn mark_read(&self, notification_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.notifications SET is_read = true, read_at = now() WHERE id = $1"
        )
        .bind(notification_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn mark_all_read(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<u64> {
        let result = sqlx::query(
            "UPDATE _atlas.notifications SET is_read = true, read_at = now() 
             WHERE organization_id = $1 AND user_id = $2 AND is_read = false AND is_dismissed = false"
        )
        .bind(org_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(result.rows_affected())
    }

    async fn dismiss(&self, notification_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.notifications SET is_dismissed = true, dismissed_at = now() WHERE id = $1"
        )
        .bind(notification_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn unread_count(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM _atlas.notifications 
             WHERE organization_id = $1 AND user_id = $2 AND is_read = false AND is_dismissed = false"
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(count.0)
    }

    async fn list(&self, org_id: Uuid, user_id: Uuid, include_read: bool, limit: i64, offset: i64) -> AtlasResult<Vec<Notification>> {
        let rows = if include_read {
            sqlx::query(
                "SELECT * FROM _atlas.notifications 
                 WHERE organization_id = $1 AND user_id = $2 AND is_dismissed = false
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(org_id).bind(user_id).bind(limit).bind(offset)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.notifications 
                 WHERE organization_id = $1 AND user_id = $2 AND is_read = false AND is_dismissed = false
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(org_id).bind(user_id).bind(limit).bind(offset)
            .fetch_all(&self.pool).await
        };

        let rows = rows.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        rows.iter()
            .map(|r| Self::row_to_notification(r).map_err(|e| AtlasError::DatabaseError(e.to_string())))
            .collect()
    }

    async fn get_users_by_role(&self, org_id: Uuid, role: &str) -> AtlasResult<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM _atlas.users 
             WHERE organization_id = $1 AND is_active = true 
             AND roles @> $2::jsonb"
        )
        .bind(org_id)
        .bind(serde_json::json!([role]))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn cleanup_expired(&self) -> AtlasResult<u64> {
        let result = sqlx::query(
            "DELETE FROM _atlas.notifications WHERE expires_at IS NOT NULL AND expires_at < now()"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(result.rows_affected())
    }

    async fn send_scheduled(&self) -> AtlasResult<u64> {
        let result = sqlx::query(
            "UPDATE _atlas.notifications SET sent_at = now() 
             WHERE scheduled_for IS NOT NULL AND sent_at IS NULL AND scheduled_for <= now()"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(result.rows_affected())
    }
}