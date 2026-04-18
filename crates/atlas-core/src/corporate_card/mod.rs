//! Corporate Card Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Corporate Card Management.
//! Manages corporate credit card programmes, card issuance to employees,
//! statement imports, transaction-to-expense matching, spending limit
//! overrides, and dispute handling.
//!
//! Oracle Fusion equivalent: Financials > Expenses > Corporate Cards

mod repository;
pub mod engine;

pub use engine::CorporateCardEngine;
pub use repository::{CorporateCardRepository, PostgresCorporateCardRepository};
