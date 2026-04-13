//! Atlas Projects - Project Management
//! 
//! Provides project planning, task tracking, timesheet management,
//! milestone tracking, and resource allocation.

pub mod entities;
pub mod services;

pub use services::{ProjectService, TaskService};
