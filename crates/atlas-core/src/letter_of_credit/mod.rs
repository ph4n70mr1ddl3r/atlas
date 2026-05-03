//! Letter of Credit Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Letter of Credit Management.
//! Manages import/export letters of credit, amendments, document requirements,
//! shipments, presentations, and comprehensive dashboard reporting for
//! international trade finance operations.
//!
//! Oracle Fusion equivalent: Treasury > Trade Finance > Letters of Credit

mod repository;
pub mod engine;

pub use engine::LetterOfCreditEngine;
pub use repository::{LetterOfCreditRepository, PostgresLetterOfCreditRepository};
