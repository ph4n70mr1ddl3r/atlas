//! Transaction Calendar Module
//!
//! Oracle Fusion Cloud ERP-inspired Transaction Calendar Management.
//! Provides business day calendars with holiday management, used throughout
//! financials for date calculations (due dates, posting dates, discount dates,
//! cash forecasting, etc.).
//!
//! Key capabilities:
//! - Define calendars with configurable working day patterns
//! - Manage holidays and exception dates (non-working and special-working)
//! - Calculate next/previous business day
//! - Add N business days to a date
//! - Check if a date is a business day
//! - Full audit trail of date calculations
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Transaction Calendars

pub mod engine;
mod repository;

pub use engine::TransactionCalendarEngine;
pub use repository::{PostgresTransactionCalendarRepository, TransactionCalendarRepository};
