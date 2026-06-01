//! Bank Statement Auto-Reconciliation Module
//!
//! Oracle Fusion Cloud ERP-inspired Bank Statement Auto-Reconciliation.
//! Provides statement import (MT940/BAI2/OFX simulation), configurable
//! matching rules, auto-matching engine, exception management, and
//! comprehensive reconciliation audit trail.
//!
//! Oracle Fusion equivalent: Cash Management > Bank Statements > Auto-Reconciliation

pub mod engine;
mod repository;

pub use engine::BankStatementReconciliationEngine;
pub use repository::{
    BankStatementReconciliationRepository, PostgresBankStatementReconciliationRepository,
};
