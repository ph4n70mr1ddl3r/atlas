//! Funds Reservation & Budgetary Control Module
//!
//! Oracle Fusion Cloud: Financials > Budgetary Control > Funds Reservation
//! Provides:
//! - Fund reservations against approved budgets
//! - Fund availability checking (advisory or absolute control)
//! - Fund consumption when actual transactions post
//! - Fund reservation release (partial or full)
//! - Budgetary control dashboard with analytics
//!
//! Oracle Fusion equivalent: Budgetary Control > Funds Reservation, Fund Check

mod repository;
pub mod engine;

pub use engine::FundsReservationEngine;
pub use repository::{FundsReservationRepository, PostgresFundsReservationRepository};
