//! Atlas HCM - Human Capital Management
//! 
//! Employee management, payroll, time & attendance, performance.
//! 
//! This module uses declarative entities defined in the database.
//! All business logic is configured declaratively and hot-reloaded.

pub mod entities;
pub mod services;

pub use entities::*;
