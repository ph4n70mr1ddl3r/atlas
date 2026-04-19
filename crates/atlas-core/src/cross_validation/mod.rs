//! Cross-Validation Rules Module
//!
//! Oracle Fusion Cloud ERP-inspired Cross-Validation Rules (CVR).
//! Prevents invalid combinations of Chart of Accounts segment values.
//!
//! Key capabilities:
//! - Define "deny" rules that block invalid account combinations
//! - Define "allow" rules that explicitly permit specific combinations
//! - Pattern matching with wildcards ("%") for each segment
//! - Priority-based rule evaluation (lower = evaluated first)
//! - Effective date ranges for rule activation/deactivation
//! - Full validation API for checking account combinations
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Chart of Accounts >
//!   Cross-Validation Rules

mod repository;
pub mod engine;

pub use engine::CrossValidationEngine;
pub use repository::{CrossValidationRepository, PostgresCrossValidationRepository};
