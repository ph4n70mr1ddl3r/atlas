//! Payment Risk & Fraud Detection Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Risk Management.
//! Provides duplicate payment detection, risk scoring, velocity checks,
//! sanctions screening, supplier risk assessment, behavioral analysis,
//! and fraud alert management.
//!
//! Oracle Fusion equivalent: Financials > Payables > Payment Risk

pub mod engine;
pub mod repository;

pub use engine::PaymentRiskEngine;
pub use repository::{PaymentRiskRepository, PostgresPaymentRiskRepository};
