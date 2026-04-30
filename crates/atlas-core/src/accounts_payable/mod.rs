//! Accounts Payable Module
//!
//! Oracle Fusion Cloud ERP-inspired Accounts Payable.
//! Provides supplier invoice management with lines and distributions,
//! invoice workflow (draft → submitted → approved → paid), invoice holds,
//! payment processing, and AP aging reporting.
//!
//! Oracle Fusion equivalent: Financials > Payables > Invoices, Payments

mod repository;
pub mod engine;

pub use engine::AccountsPayableEngine;
pub use repository::{AccountsPayableRepository, PostgresAccountsPayableRepository};
