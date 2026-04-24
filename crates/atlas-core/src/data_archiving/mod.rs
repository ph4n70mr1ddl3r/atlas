//! Data Archiving and Retention Management Module
//!
//! Oracle Fusion Cloud: Tools > Information Lifecycle Management (ILM)
//!
//! Provides configurable retention policies, data archival, purge
//! management, legal holds, and restore capabilities for compliance
//! and data lifecycle governance.
//!
//! Key capabilities:
//! - Define retention policies per entity type with configurable retention periods
//! - Archive records based on age criteria and policies
//! - Purge records that have exceeded retention
//! - Legal holds to prevent archival/purge of records under litigation
//! - Restore archived records back to active state
//! - Full audit trail of every archive/purge/restore operation
//!
//! Oracle Fusion equivalent:
//!   Tools > Information Lifecycle Management > Retention Policies
//!   Tools > Information Lifecycle Management > Legal Holds

mod engine;
mod repository;

pub use engine::DataArchivingEngine;
pub use repository::{DataArchivingRepository, PostgresDataArchivingRepository};
