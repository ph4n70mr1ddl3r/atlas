//! Atlas Financials
//! 
//! Provides financial management including general ledger, invoicing,
//! purchase orders, budgeting, and financial reporting.

pub mod entities;
pub mod services;

pub use services::{PurchaseOrderService, InvoiceService, GeneralLedgerService};
