//! Journal Import Module (Oracle Fusion GL > Import Journals)
//!
//! Oracle Fusion Cloud ERP-inspired Journal Import.
//! Allows organizations to import journal entries from external systems,
//! subledgers, and flat files into the General Ledger.
//!
//! Features:
//! - Import format definitions with column mappings
//! - Data validation (account codes, balancing, required fields)
//! - Error handling with row-level detail
//! - Import workflow (upload → validate → import → post)
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Import Journals

mod repository;
pub mod engine;

pub use engine::JournalImportEngine;
pub use repository::{JournalImportRepository, PostgresJournalImportRepository};
