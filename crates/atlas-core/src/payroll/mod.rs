//! Payroll Management Module
//!
//! Oracle Fusion Cloud HCM Global Payroll implementation.
//! Provides payroll definitions, element management (earnings & deductions),
//! payroll run lifecycle, pay slip generation, and payroll calculations.
//!
//! Oracle Fusion equivalent: Payroll > Payroll Definitions, Elements, Payroll Runs, Pay Slips

mod engine;
mod repository;

pub use engine::PayrollEngine;
pub use repository::{PayrollRepository, PostgresPayrollRepository};
