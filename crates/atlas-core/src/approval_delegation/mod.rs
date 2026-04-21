//! Approval Delegation Rules
//!
//! Oracle Fusion BPM Worklist > Rules > Configure Delegation
//!
//! Users can proactively set up delegation rules that say:
//! "While I'm on vacation from June 1-15, delegate all my approvals to Jane Smith."
//!
//! Supports:
//! - Full delegation (all approvals)
//! - Category-based delegation (e.g., only "purchase_orders")
//! - Role-based delegation (e.g., only approvals requiring role "finance_manager")
//! - Entity-type-based delegation
//! - Automatic activation/expiry based on date ranges

mod engine;
mod repository;

pub use engine::ApprovalDelegationEngine;
pub use repository::{ApprovalDelegationRepository, PostgresApprovalDelegationRepository};
