//! Scheduled Process Module
//!
//! Oracle Fusion Cloud ERP-inspired Enterprise Scheduler Service.
//! Provides process template management, process submission (immediate and scheduled),
//! recurring job scheduling, execution monitoring, and detailed process logging.
//!
//! Key capabilities:
//! - Define process templates with parameters, timeout, and retry policies
//! - Submit processes for immediate or deferred execution
//! - Schedule recurring process runs (daily, weekly, monthly, cron)
//! - Monitor process status with heartbeat tracking and progress
//! - Detailed execution log with step-level granularity
//! - Cancel running processes, retry failed ones
//!
//! Oracle Fusion equivalent: Navigator > Tools > Scheduled Processes

mod engine;
mod repository;

pub use engine::ScheduledProcessEngine;
pub use repository::{ScheduledProcessRepository, PostgresScheduledProcessRepository};
