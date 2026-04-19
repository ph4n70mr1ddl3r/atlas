//! Manual Journal Entries Module
//!
//! Oracle Fusion Cloud ERP-inspired Manual Journal Entries.
//! Provides journal batch and entry management with full lifecycle
//! support: Draft → Submitted → Approved → Posted → Reversed.
//!
//! Key capabilities:
//! - Create journal batches containing multiple journal entries
//! - Add balanced debit/credit lines to each entry
//! - Submit, approve, post, and reverse journals
//! - Automatic balance validation (total debits = total credits)
//! - Period-aware posting controls
//!
//! Oracle Fusion equivalent: General Ledger > Journals > New Journal

mod repository;
pub mod engine;

pub use engine::ManualJournalEngine;
pub use repository::{ManualJournalRepository, PostgresManualJournalRepository};
