//! Bank Account Transfer Module
//!
//! Oracle Fusion Cloud ERP-inspired internal bank account transfers.
//! Manages transfers between organization bank accounts with approval
//! workflows, cross-currency support, and audit trails.
//!
//! Oracle Fusion equivalent: Financials > Cash Management > Bank Account Transfers

mod repository;
pub mod engine;

pub use engine::BankAccountTransferEngine;
pub use repository::{BankAccountTransferRepository, PostgresBankAccountTransferRepository};
