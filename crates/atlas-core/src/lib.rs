//! Atlas Core Engine
//! 
//! The declarative foundation of Atlas ERP. This module contains:
//! - Schema engine for dynamic entity definitions
//! - Workflow engine for state machine execution
//! - Validation engine for declarative rules
//! - Formula engine for computed fields
//! - Security engine for access control
//! - Audit engine for change tracking
//! - Configuration engine for hot-reload
//! - Event bus for inter-service communication
//! - Notification engine (Oracle Fusion bell-icon notifications)
//! - Approval engine (Oracle Fusion multi-level approvals)

pub mod schema;
pub mod workflow;
pub mod validation;
pub mod formula;
pub mod security;
pub mod audit;
pub mod config;
pub mod eventbus;
pub mod notification;
pub mod approval;
pub mod period_close;
pub mod currency;
pub mod tax;
pub mod intercompany;
pub mod reconciliation;
pub mod budget;
pub mod expense;
pub mod fixed_assets;
pub mod collections;
pub mod revenue;
pub mod payment;
pub mod subledger_accounting;
pub mod encumbrance;
pub mod cash_management;
pub mod sourcing;
pub mod lease;
pub mod project_costing;
pub mod cost_allocation;
pub mod financial_reporting;
pub mod withholding_tax;
pub mod multi_book;
pub mod procurement_contracts;
pub mod inventory;
pub mod customer_returns;
pub mod pricing;
pub mod sales_commission;
pub mod subscription;
pub mod treasury;
pub mod grant_management;
pub mod corporate_card;
pub mod financial_consolidation;
pub mod supplier_qualification;
pub mod recurring_journal;

pub use schema::*;
pub use workflow::{
    WorkflowEngine, StateMachine, GuardEvaluator, GuardResult,
    ActionExecutor, ActionResult,
    WorkflowState, StateHistoryEntry, TransitionResult,
    AvailableTransitions, TransitionInfo,
    repository::{WorkflowStateRepository, PostgresWorkflowStateRepository, InMemoryWorkflowStateRepository},
};

// Re-export the workflow engine's User type under a distinct path
// so downstream crates can import it without colliding with
// atlas_shared::User.
pub use workflow::engine::User as WorkflowUser;

pub use validation::*;
pub use formula::*;
pub use security::*;
pub use audit::*;
pub use config::*;
pub use eventbus::*;
pub use notification::{NotificationEngine, PostgresNotificationRepository as PostgresNotificationRepo};
pub use approval::{ApprovalEngine, PostgresApprovalRepository as PostgresApprovalRepo};
pub use period_close::{PeriodCloseEngine, PostgresPeriodCloseRepository as PostgresPeriodCloseRepo};
pub use currency::{CurrencyEngine, PostgresCurrencyRepository as PostgresCurrencyRepo};
pub use tax::{TaxEngine, PostgresTaxRepository as PostgresTaxRepo};
pub use intercompany::{IntercompanyEngine, PostgresIntercompanyRepository as PostgresIntercompanyRepo};
pub use reconciliation::{ReconciliationEngine, PostgresReconciliationRepository as PostgresReconciliationRepo};
pub use expense::{ExpenseEngine, PostgresExpenseRepository as PostgresExpenseRepo};
pub use budget::{BudgetEngine, PostgresBudgetRepository as PostgresBudgetRepo};
pub use fixed_assets::{FixedAssetEngine, PostgresFixedAssetRepository as PostgresFixedAssetRepo};
pub use collections::{CollectionsEngine, PostgresCollectionsRepository as PostgresCollectionsRepo};
pub use revenue::{RevenueEngine, PostgresRevenueRepository as PostgresRevenueRepo};
pub use payment::{PaymentEngine, PostgresPaymentRepository as PostgresPaymentRepo};
pub use subledger_accounting::{SubledgerAccountingEngine, PostgresSubledgerAccountingRepository as PostgresSubledgerAccountingRepo};
pub use encumbrance::{EncumbranceEngine, PostgresEncumbranceRepository as PostgresEncumbranceRepo};
pub use cash_management::{CashManagementEngine, PostgresCashManagementRepository as PostgresCashManagementRepo};
pub use sourcing::{SourcingEngine, PostgresSourcingRepository as PostgresSourcingRepo};
pub use lease::{LeaseAccountingEngine, PostgresLeaseAccountingRepository as PostgresLeaseAccountingRepo};
pub use project_costing::{ProjectCostingEngine, PostgresProjectCostingRepository as PostgresProjectCostingRepo};
pub use cost_allocation::{CostAllocationEngine, PostgresCostAllocationRepository as PostgresCostAllocationRepo};
pub use financial_reporting::{FinancialReportingEngine, PostgresFinancialReportingRepository as PostgresFinancialReportingRepo};
pub use withholding_tax::{WithholdingTaxEngine, PostgresWithholdingTaxRepository as PostgresWithholdingTaxRepo};
pub use multi_book::{MultiBookAccountingEngine, PostgresMultiBookAccountingRepository as PostgresMultiBookAccountingRepo};
pub use procurement_contracts::{ProcurementContractEngine, PostgresProcurementContractRepository as PostgresProcurementContractRepo};
pub use inventory::{InventoryEngine, PostgresInventoryRepository as PostgresInventoryRepo};
pub use customer_returns::{CustomerReturnsEngine, PostgresCustomerReturnsRepository as PostgresCustomerReturnsRepo};
pub use pricing::{PricingEngine, PostgresPricingRepository as PostgresPricingRepo};
pub use sales_commission::{SalesCommissionEngine, PostgresSalesCommissionRepository as PostgresSalesCommissionRepo};
pub use treasury::{TreasuryEngine, PostgresTreasuryRepository as PostgresTreasuryRepo};
pub use subscription::{SubscriptionEngine, PostgresSubscriptionRepository as PostgresSubscriptionRepo};
pub use grant_management::{GrantManagementEngine, PostgresGrantManagementRepository as PostgresGrantManagementRepo};
pub use corporate_card::{CorporateCardEngine, PostgresCorporateCardRepository as PostgresCorporateCardRepo};
pub use financial_consolidation::{FinancialConsolidationEngine, PostgresFinancialConsolidationRepository as PostgresFinancialConsolidationRepo};
pub use supplier_qualification::{SupplierQualificationEngine, PostgresSupplierQualificationRepository as PostgresSupplierQualificationRepo};
pub use recurring_journal::{RecurringJournalEngine, PostgresRecurringJournalRepository as PostgresRecurringJournalRepo};

mod mock_repos;
pub use mock_repos::*;
