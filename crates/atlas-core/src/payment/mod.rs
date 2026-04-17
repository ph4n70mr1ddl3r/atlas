//! Payment Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Management.
//! Provides payment terms, payment batches, payment processing,
//! scheduled payments, void/reissue, and remittance advice.
//!
//! Oracle Fusion equivalent: Financials > Payables > Payments

mod repository;
pub mod engine;

pub use engine::PaymentEngine;
pub use repository::{PaymentRepository, PostgresPaymentRepository};
