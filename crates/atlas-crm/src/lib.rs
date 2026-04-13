//! Atlas CRM - Customer Relationship Management
//! 
//! Provides customer lifecycle management, lead tracking, opportunity
//! pipeline, contact management, and service case handling.

pub mod entities;
pub mod services;

pub use services::{CustomerService, LeadService};
