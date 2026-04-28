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
pub mod manual_journal;
pub mod document_sequencing;
pub mod descriptive_flexfield;
pub mod cross_validation;
pub mod scheduled_process;
pub mod segregation_of_duties;
pub mod allocation;
pub mod currency_revaluation;
pub mod purchase_requisition;
pub mod benefits;
pub mod autoinvoice;
pub mod performance;
pub mod credit_management;
pub mod product_information;
pub mod quality_management;
pub mod transfer_pricing;
pub mod order_management;
pub mod approval_delegation;
pub mod manufacturing;
pub mod warehouse_management;
pub mod absence;
pub mod time_and_labor;
pub mod approval_authority;
pub mod data_archiving;
pub mod payroll;
pub mod compensation;
pub mod service_request;
pub mod lead_opportunity;
pub mod demand_planning;
pub mod shipping;
pub mod recruiting;
pub mod marketing;
pub mod receiving;
pub mod supplier_scorecard;
pub mod kpi;
pub mod account_monitor;
pub mod goal_management;
pub mod landed_cost;
pub mod contract_lifecycle;
pub mod succession_planning;
pub mod learning_management;
pub mod joint_venture;
pub mod enterprise_asset_management;
pub mod risk_management;

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
pub use manual_journal::{ManualJournalEngine, PostgresManualJournalRepository as PostgresManualJournalRepo};
pub use document_sequencing::{DocumentSequencingEngine, PostgresDocumentSequencingRepository as PostgresDocumentSequencingRepo};
pub use descriptive_flexfield::{DescriptiveFlexfieldEngine, PostgresDescriptiveFlexfieldRepository as PostgresDescriptiveFlexfieldRepo};
pub use cross_validation::{CrossValidationEngine, PostgresCrossValidationRepository as PostgresCrossValidationRepo};
pub use scheduled_process::{ScheduledProcessEngine, PostgresScheduledProcessRepository as PostgresScheduledProcessRepo};
pub use segregation_of_duties::{SegregationOfDutiesEngine, PostgresSegregationOfDutiesRepository as PostgresSegregationOfDutiesRepo};
pub use allocation::{AllocationEngine, PostgresAllocationRepository as PostgresAllocationRepo};
pub use currency_revaluation::{CurrencyRevaluationEngine, PostgresCurrencyRevaluationRepository as PostgresCurrencyRevaluationRepo};
pub use purchase_requisition::{PurchaseRequisitionEngine, PostgresPurchaseRequisitionRepository as PostgresPurchaseRequisitionRepo};
pub use benefits::{BenefitsEngine, PostgresBenefitsRepository as PostgresBenefitsRepo};
pub use autoinvoice::{AutoInvoiceEngine, PostgresAutoInvoiceRepository as PostgresAutoInvoiceRepo};
pub use performance::{PerformanceEngine, PostgresPerformanceRepository as PostgresPerformanceRepo};
pub use credit_management::{CreditManagementEngine, PostgresCreditManagementRepository as PostgresCreditManagementRepo};
pub use product_information::{ProductInformationEngine, PostgresProductInformationRepository as PostgresProductInformationRepo};
pub use quality_management::{QualityManagementEngine, PostgresQualityManagementRepository as PostgresQualityManagementRepo};
pub use transfer_pricing::{TransferPricingEngine, PostgresTransferPricingRepository as PostgresTransferPricingRepo};
pub use order_management::{OrderManagementEngine, PostgresOrderManagementRepository as PostgresOrderManagementRepo};
pub use approval_delegation::{ApprovalDelegationEngine, PostgresApprovalDelegationRepository as PostgresApprovalDelegationRepo};
pub use manufacturing::{ManufacturingEngine, PostgresManufacturingRepository as PostgresManufacturingRepo};
pub use warehouse_management::{WarehouseManagementEngine, PostgresWarehouseManagementRepository as PostgresWarehouseManagementRepo};
pub use absence::{AbsenceEngine, PostgresAbsenceRepository as PostgresAbsenceRepo};
pub use time_and_labor::{TimeAndLaborEngine, PostgresTimeAndLaborRepository as PostgresTimeAndLaborRepo};
pub use approval_authority::{ApprovalAuthorityEngine, PostgresApprovalAuthorityRepository as PostgresApprovalAuthorityRepo};
pub use data_archiving::{DataArchivingEngine, PostgresDataArchivingRepository as PostgresDataArchivingRepo};
pub use payroll::{PayrollEngine, PostgresPayrollRepository as PostgresPayrollRepo};
pub use compensation::{CompensationEngine, PostgresCompensationRepository as PostgresCompensationRepository};
pub use service_request::{ServiceRequestEngine, PostgresServiceRequestRepository as PostgresServiceRequestRepo};
pub use lead_opportunity::{LeadOpportunityEngine, PostgresLeadOpportunityRepository as PostgresLeadOpportunityRepo};
pub use demand_planning::{DemandPlanningEngine, PostgresDemandPlanningRepository as PostgresDemandPlanningRepo};
pub use shipping::{ShippingEngine, PostgresShippingRepository as PostgresShippingRepo};
pub use recruiting::{RecruitingEngine, PostgresRecruitingRepository as PostgresRecruitingRepo};
pub use marketing::{MarketingEngine, PostgresMarketingRepository as PostgresMarketingRepo};
pub use receiving::{ReceivingEngine, PostgresReceivingRepository as PostgresReceivingRepo};
pub use supplier_scorecard::{SupplierScorecardEngine, PostgresScorecardRepository as PostgresScorecardRepo};
pub use kpi::{KpiEngine, PostgresKpiRepository as PostgresKpiRepo};
pub use account_monitor::{AccountMonitorEngine, PostgresAccountMonitorRepository as PostgresAccountMonitorRepo};
pub use goal_management::{GoalManagementEngine, PostgresGoalManagementRepository as PostgresGoalManagementRepo};
pub use landed_cost::{LandedCostEngine, PostgresLandedCostRepository as PostgresLandedCostRepo};
pub use contract_lifecycle::{ContractLifecycleEngine, PostgresContractLifecycleRepository as PostgresContractLifecycleRepo};
pub use succession_planning::{SuccessionPlanningEngine, PostgresSuccessionPlanningRepository as PostgresSuccessionPlanningRepo};
pub use learning_management::{LearningManagementEngine, PostgresLearningManagementRepository as PostgresLearningManagementRepo};
pub use joint_venture::{JointVentureEngine, PostgresJointVentureRepository as PostgresJointVentureRepo};
pub use risk_management::{RiskManagementEngine, PostgresRiskManagementRepository as PostgresRiskManagementRepo};
pub use enterprise_asset_management::{EnterpriseAssetManagementEngine, PostgresAssetManagementRepository as PostgresAssetManagementRepo};

mod mock_repos;
pub use mock_repos::*;
