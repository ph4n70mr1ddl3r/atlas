//! Subscription Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Subscription Management.
//! Provides subscription product catalog management, subscription lifecycle
//! (create, activate, suspend, cancel, renew), amendments (price/quantity changes),
//! billing schedule generation, proration calculations, and ASC 606 revenue scheduling.
//!
//! Oracle Fusion equivalent: Financials > Subscription Management > Subscriptions

mod repository;
pub mod engine;

pub use engine::SubscriptionEngine;
pub use repository::{SubscriptionRepository, PostgresSubscriptionRepository};
