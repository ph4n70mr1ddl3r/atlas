//! Bank Guarantee Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Bank Guarantee Management.
//! Provides bank guarantee definitions (bid bonds, performance guarantees,
//! advance payment guarantees, etc.), lifecycle management from request
//! through issuance to release/expiration, amendment tracking, and
//! comprehensive dashboard reporting.
//!
//! Oracle Fusion equivalent: Treasury > Bank Guarantees

mod repository;
pub mod engine;

pub use engine::BankGuaranteeEngine;
pub use repository::{BankGuaranteeRepository, PostgresBankGuaranteeRepository};
