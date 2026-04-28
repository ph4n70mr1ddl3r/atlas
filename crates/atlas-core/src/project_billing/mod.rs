//! Project Billing Module
//!
//! Oracle Fusion Cloud: Project Management > Project Billing
//! Provides bill rate schedules, billing events, project invoices,
//! retention management, and billing analytics.
//!
//! Oracle Fusion equivalent: Project Billing, Project Invoices, Billing Events

mod repository;
pub mod engine;

pub use engine::ProjectBillingEngine;
pub use repository::{ProjectBillingRepository, PostgresProjectBillingRepository};
