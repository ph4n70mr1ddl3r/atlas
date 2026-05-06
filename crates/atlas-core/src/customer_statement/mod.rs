//! Customer Account Statement / Balance Forward Billing Module
//!
//! Oracle Fusion Cloud ERP-inspired Balance Forward Billing.
//! Generates consolidated customer statements showing opening balance,
//! new charges, payments/credits, closing balance, and aging breakdown.
//!
//! Oracle Fusion equivalent: Financials > Receivables > Billing > Balance Forward Billing

pub mod repository;
pub mod engine;

pub use engine::CustomerStatementEngine;
pub use repository::{CustomerStatementRepository, PostgresCustomerStatementRepository};
