//! Notification Engine Implementation
//!
//! Manages creation, querying, and delivery of notifications.
//! Inspired by Oracle Fusion's bell-icon notification system.

use atlas_shared::{Notification, CreateNotificationRequest, AtlasResult};
use super::NotificationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;


/// Notification engine for managing notifications
pub struct NotificationEngine {
    repository: Arc<dyn NotificationRepository>,
}

impl NotificationEngine {
    pub fn new(repository: Arc<dyn NotificationRepository>) -> Self {
        Self { repository }
    }

    /// Create a new notification
    pub async fn create(
        &self,
        org_id: Uuid,
        request: CreateNotificationRequest,
    ) -> AtlasResult<Notification> {
        info!("Creating notification: {} for user {:?}", request.title, request.user_id);

        let notification = self.repository.create(org_id, request).await?;
        Ok(notification)
    }

    /// Create notifications for all users with a given role
    #[allow(clippy::too_many_arguments)]
    pub async fn notify_role(
        &self,
        org_id: Uuid,
        role: &str,
        notification_type: &str,
        title: &str,
        message: Option<&str>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        action_url: Option<&str>,
    ) -> AtlasResult<Vec<Notification>> {
        info!("Sending notification to role: {} - {}", role, title);

        let user_ids = self.repository.get_users_by_role(org_id, role).await?;
        let mut notifications = Vec::new();

        for user_id in user_ids {
            let request = CreateNotificationRequest {
                user_id: Some(user_id),
                role: None,
                notification_type: notification_type.to_string(),
                priority: Some("normal".to_string()),
                title: title.to_string(),
                message: message.map(|s| s.to_string()),
                entity_type: entity_type.map(|s| s.to_string()),
                entity_id,
                action_url: action_url.map(|s| s.to_string()),
                workflow_name: None,
                from_state: None,
                to_state: None,
                action: None,
                performed_by: None,
                channels: None,
                metadata: None,
            };

            match self.repository.create(org_id, request).await {
                Ok(n) => notifications.push(n),
                Err(e) => {
                    tracing::warn!("Failed to send notification to user {}: {:?}", user_id, e);
                }
            }
        }

        Ok(notifications)
    }

    /// Create a workflow transition notification
    #[allow(clippy::too_many_arguments)]
    pub async fn notify_workflow_transition(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        action: &str,
        from_state: &str,
        to_state: &str,
        workflow_name: &str,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<Notification> {
        let title = format!("{}: {} → {}", action.replace('_', " "), 
            from_state.replace('_', " "), to_state.replace('_', " "));
        let message = Some(format!(
            "Record {} in {} has transitioned from {} to {} via action '{}'.",
            entity_id, entity_type, from_state, to_state, action
        ));
        let action_url = Some(format!("/{}/{}", entity_type, entity_id));

        let request = CreateNotificationRequest {
            user_id: Some(user_id),
            role: None,
            notification_type: "workflow_action".to_string(),
            priority: Some("normal".to_string()),
            title,
            message,
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id),
            action_url,
            workflow_name: Some(workflow_name.to_string()),
            from_state: Some(from_state.to_string()),
            to_state: Some(to_state.to_string()),
            action: Some(action.to_string()),
            performed_by,
            channels: None,
            metadata: None,
        };

        self.repository.create(org_id, request).await
    }

    /// Create an approval-required notification
    pub async fn notify_approval_required(
        &self,
        org_id: Uuid,
        approver_user_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        level: i32,
        title: &str,
    ) -> AtlasResult<Notification> {
        let message = Some(format!(
            "Your approval is requested at level {} for {} '{}'.",
            level, entity_type, title
        ));

        let request = CreateNotificationRequest {
            user_id: Some(approver_user_id),
            role: None,
            notification_type: "approval_required".to_string(),
            priority: Some("high".to_string()),
            title: format!("Approval Required: {}", title),
            message,
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id),
            action_url: Some(format!("/{}/{}", entity_type, entity_id)),
            workflow_name: None,
            from_state: None,
            to_state: None,
            action: None,
            performed_by: None,
            channels: None,
            metadata: None,
        };

        self.repository.create(org_id, request).await
    }

    /// Create an escalation notification
    #[allow(clippy::too_many_arguments)]
    pub async fn notify_escalation(
        &self,
        org_id: Uuid,
        escalated_to_role: &str,
        entity_type: &str,
        entity_id: Uuid,
        _original_approver: Uuid,
        hours_passed: i32,
        title: &str,
    ) -> AtlasResult<Vec<Notification>> {
        info!("Escalating approval for {} {} after {} hours", entity_type, entity_id, hours_passed);
        self.notify_role(
            org_id,
            escalated_to_role,
            "escalation",
            &format!("Escalated Approval: {}", title),
            Some(&format!(
                "Approval has been escalated after {} hours. Original approver may be unavailable.",
                hours_passed
            )),
            Some(entity_type),
            Some(entity_id),
            Some(&format!("/{}/{}", entity_type, entity_id)),
        ).await
    }

    /// Mark a notification as read
    pub async fn mark_read(&self, notification_id: Uuid) -> AtlasResult<()> {
        self.repository.mark_read(notification_id).await
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_read(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<u64> {
        self.repository.mark_all_read(org_id, user_id).await
    }

    /// Dismiss a notification
    pub async fn dismiss(&self, notification_id: Uuid) -> AtlasResult<()> {
        self.repository.dismiss(notification_id).await
    }

    /// Get unread count for a user
    pub async fn unread_count(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<i64> {
        self.repository.unread_count(org_id, user_id).await
    }

    /// Get notifications for a user with pagination
    pub async fn list(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        include_read: bool,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<Notification>> {
        self.repository.list(org_id, user_id, include_read, limit, offset).await
    }

    /// Delete expired notifications
    pub async fn cleanup_expired(&self) -> AtlasResult<u64> {
        self.repository.cleanup_expired().await
    }

    /// Process scheduled notifications (escalations, reminders)
    pub async fn process_scheduled(&self) -> AtlasResult<u64> {
        self.repository.send_scheduled().await
    }
}