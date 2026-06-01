//! Bank Account Transfer Module
//!
//! Oracle Fusion Cloud ERP-inspired internal bank account transfers.
//! Manages transfers between organization bank accounts with approval
//! workflows, cross-currency support, and audit trails.
//!
//! Oracle Fusion equivalent: Financials > Cash Management > Bank Account Transfers

pub mod engine;
mod repository;

pub use engine::BankAccountTransferEngine;
pub use repository::{BankAccountTransferRepository, PostgresBankAccountTransferRepository};
