//! Document Sequencing Module
//!
//! Oracle Fusion Cloud ERP-inspired Document Sequencing.
//! Provides automatic sequential document numbering for regulatory compliance,
//! supporting gapless and gap-permitted sequences, configurable reset frequency,
//! prefix/suffix formatting, and full audit trail of every number generated.
//!
//! Key capabilities:
//! - Define document sequences with gapless or gap-permitted modes
//! - Assign sequences to document categories, business units, and ledgers
//! - Generate formatted document numbers with prefix, suffix, and padding
//! - Automatic reset at period boundaries (daily, monthly, quarterly, annually)
//! - Complete audit trail for compliance reporting
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Document Sequencing

mod repository;
pub mod engine;

pub use engine::DocumentSequencingEngine;
pub use repository::{DocumentSequencingRepository, PostgresDocumentSequencingRepository};
