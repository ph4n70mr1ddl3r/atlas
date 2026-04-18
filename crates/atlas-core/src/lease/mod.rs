//! Lease Accounting Module (ASC 842 / IFRS 16)
//!
//! Oracle Fusion Cloud ERP-inspired Lease Management.
//! Provides lease contract management, classification (operating/finance),
//! right-of-use (ROU) asset tracking, lease liability amortization,
//! payment scheduling with escalation, lease modifications, impairment,
//! and termination accounting.
//!
//! Oracle Fusion equivalent: Financials > Lease Management

mod repository;
pub mod engine;

pub use engine::LeaseAccountingEngine;
pub use repository::{LeaseAccountingRepository, PostgresLeaseAccountingRepository};
