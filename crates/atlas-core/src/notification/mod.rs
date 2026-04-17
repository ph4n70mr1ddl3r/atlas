//! Notification Engine
//!
//! Oracle Fusion-inspired notification system with bell-icon alerts,
//! workflow-action notifications, approval requests, escalation support,
//! and per-user notification preferences.

mod engine;
mod repository;

pub use engine::NotificationEngine;
pub use repository::{NotificationRepository, PostgresNotificationRepository};