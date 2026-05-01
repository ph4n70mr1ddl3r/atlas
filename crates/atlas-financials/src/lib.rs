//! Atlas Financials
//! 
//! Provides financial management modules inspired by Oracle Fusion Cloud ERP:
//! - General Ledger (Chart of Accounts, Journal Entries)
//! - Accounts Payable (AP Invoices, Payments, Holds)
//! - Accounts Receivable (AR Transactions, Receipts, Credit Memos, Adjustments)
//! - Fixed Assets (Categories, Books, Assets, Depreciation, Transfers, Retirements)
//! - Cost Management (Cost Books, Elements, Profiles, Standard Costs, Adjustments, Variances)
//! - Budgeting & Planning
//! - Expense Reports

pub mod entities;
pub mod services;

pub use services::{
    PurchaseOrderService,
    InvoiceService,
    GeneralLedgerService,
    AccountsPayableService,
    AccountsReceivableService,
    FixedAssetsService,
    CostManagementService,
};
