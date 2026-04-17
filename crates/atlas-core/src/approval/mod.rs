//! Approval Engine
//!
//! Oracle Fusion-inspired multi-level approval chains with:
//! - Sequential approval levels
//! - Role-based and user-specific approvers
//! - Timed escalation (auto-approve or escalate after hours)
//! - Delegation support (delegate your approval to another user)

mod engine;
mod repository;

pub use engine::ApprovalEngine;
pub use repository::{ApprovalRepository, PostgresApprovalRepository};