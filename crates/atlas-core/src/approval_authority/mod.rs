//! Approval Authority Limits Module
//!
//! Oracle Fusion Cloud BPM: Approval Configuration > Document Approval Limits
//!
//! Controls which transactions a user or role can approve based on dollar
//! amount thresholds, document types, and organizational scope (business
//! unit / cost center).
//!
//! Key capabilities:
//! - Define per-user and per-role approval amount limits by document type
//! - Enforce limits during approval workflows
//! - Hierarchical limit resolution (user-specific overrides role defaults)
//! - Effective dating for temporary limit changes
//! - Full audit trail of every authority check
//!
//! Oracle Fusion equivalent:
//!   BPM > Task Configuration > Document Approval Limits
//!   also known as "Signing Limits" or "Transaction Approval Authority"

mod engine;
mod repository;

pub use engine::ApprovalAuthorityEngine;
pub use repository::{ApprovalAuthorityRepository, PostgresApprovalAuthorityRepository};
