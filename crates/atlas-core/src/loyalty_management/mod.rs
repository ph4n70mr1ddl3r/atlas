//! Loyalty Management Module
//!
//! Oracle Fusion Cloud: CX > Loyalty Management
//! Provides:
//! - Loyalty program management (points-based, tier-based, frequency-based)
//! - Loyalty tier definitions with automatic tier progression
//! - Member enrollment and lifecycle management
//! - Point transactions (accrual, redemption, adjustment, expiration)
//! - Reward catalog management
//! - Reward redemption processing
//! - Loyalty analytics dashboard
//!
//! Oracle Fusion equivalent: CX > Loyalty Management > Programs, Members, Rewards

mod repository;
pub mod engine;

pub use engine::LoyaltyManagementEngine;
pub use repository::{LoyaltyManagementRepository, PostgresLoyaltyManagementRepository};
