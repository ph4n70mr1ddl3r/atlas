//! Interest Invoice Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Late Charge / Interest Invoice Management.
//! Provides interest rate schedule definitions, overdue invoice tracking,
//! automatic interest calculation, and interest invoice generation with
//! full lifecycle management.
//!
//! Oracle Fusion equivalent: Receivables > Late Charges

mod repository;
pub mod engine;

pub use engine::InterestInvoiceEngine;
pub use repository::{InterestInvoiceRepository, PostgresInterestInvoiceRepository};
