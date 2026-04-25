//! Lead and Opportunity Management Module
//!
//! Oracle Fusion Cloud CX Sales implementation.
//! Manages sales leads, opportunity pipeline, sales activities,
//! lead scoring, lead-to-opportunity conversion, and pipeline analytics.
//!
//! Oracle Fusion equivalent: CX Sales > Leads & Opportunities

mod engine;
mod repository;

pub use engine::LeadOpportunityEngine;
pub use repository::{LeadOpportunityRepository, PostgresLeadOpportunityRepository};
