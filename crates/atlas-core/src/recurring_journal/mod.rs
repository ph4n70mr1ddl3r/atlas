//! Recurring Journals Module
//!
//! Oracle Fusion Cloud ERP-inspired Recurring Journals.
//! Provides recurring journal schedule definition, template lines,
//! automatic generation of journal entries on schedule, and generation history tracking.
//!
//! Supports three journal types:
//! - **Standard**: Fixed amounts that repeat every period
//! - **Skeleton**: Template structure; amounts are provided at generation time
//! - **Incremental**: Amounts increase by a configurable percentage each period
//!
//! Oracle Fusion equivalent: General Ledger > Journals > Recurring Journals

mod repository;
pub mod engine;

pub use engine::RecurringJournalEngine;
pub use repository::{RecurringJournalRepository, PostgresRecurringJournalRepository};
