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

// --- Core Engines ---
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

// Re-export core engines at the top level
pub use schema::*;
pub use workflow::{
    WorkflowEngine, StateMachine, GuardEvaluator, GuardResult,
    ActionExecutor, ActionResult,
    WorkflowState, StateHistoryEntry, TransitionResult,
    AvailableTransitions, TransitionInfo,
    repository::{WorkflowStateRepository, PostgresWorkflowStateRepository, InMemoryWorkflowStateRepository},
};
pub use workflow::engine::User as WorkflowUser;
pub use validation::*;
pub use formula::*;
pub use security::*;
pub use audit::*;
pub use config::*;
pub use eventbus::*;
pub use notification::{NotificationEngine, PostgresNotificationRepository as PostgresNotificationRepo};
pub use approval::{ApprovalEngine, PostgresApprovalRepository as PostgresApprovalRepo};

// --- Domain Modules (Grouped for better organization) ---

pub mod financials {
    pub use crate::period_close::{PeriodCloseEngine, PostgresPeriodCloseRepository as PostgresPeriodCloseRepo};
    pub use crate::currency::{CurrencyEngine, PostgresCurrencyRepository as PostgresCurrencyRepo};
    pub use crate::tax::{TaxEngine, PostgresTaxRepository as PostgresTaxRepo};
    pub use crate::intercompany::{IntercompanyEngine, PostgresIntercompanyRepository as PostgresIntercompanyRepo};
    pub use crate::reconciliation::{ReconciliationEngine, PostgresReconciliationRepository as PostgresReconciliationRepo};
    pub use crate::expense::{ExpenseEngine, PostgresExpenseRepository as PostgresExpenseRepo};
    pub use crate::budget::{BudgetEngine, PostgresBudgetRepository as PostgresBudgetRepo};
    pub use crate::fixed_assets::{FixedAssetEngine, PostgresFixedAssetRepository as PostgresFixedAssetRepo};
    pub use crate::collections::{CollectionsEngine, PostgresCollectionsRepository as PostgresCollectionsRepo};
    pub use crate::revenue::{RevenueEngine, PostgresRevenueRepository as PostgresRevenueRepo};
    pub use crate::payment::{PaymentEngine, PostgresPaymentRepository as PostgresPaymentRepo};
    pub use crate::subledger_accounting::{SubledgerAccountingEngine, PostgresSubledgerAccountingRepository as PostgresSubledgerAccountingRepo};
    pub use crate::encumbrance::{EncumbranceEngine, PostgresEncumbranceRepository as PostgresEncumbranceRepo};
    pub use crate::cash_management::{CashManagementEngine, PostgresCashManagementRepository as PostgresCashManagementRepo};
    pub use crate::withholding_tax::{WithholdingTaxEngine, PostgresWithholdingTaxRepository as PostgresWithholdingTaxRepo};
    pub use crate::multi_book::{MultiBookAccountingEngine, PostgresMultiBookAccountingRepository as PostgresMultiBookAccountingRepo};
    pub use crate::financial_consolidation::{FinancialConsolidationEngine, PostgresFinancialConsolidationRepository as PostgresFinancialConsolidationRepo};
    pub use crate::recurring_journal::{RecurringJournalEngine, PostgresRecurringJournalRepository as PostgresRecurringJournalRepo};
    pub use crate::manual_journal::{ManualJournalEngine, PostgresManualJournalRepository as PostgresManualJournalRepo};
    pub use crate::document_sequencing::{DocumentSequencingEngine, PostgresDocumentSequencingRepository as PostgresDocumentSequencingRepo};
    pub use crate::transaction_calendar::{TransactionCalendarEngine, PostgresTransactionCalendarRepository as PostgresTransactionCalendarRepo};
    pub use crate::allocation::{AllocationEngine, PostgresAllocationRepository as PostgresAllocationRepo};
    pub use crate::currency_revaluation::{CurrencyRevaluationEngine, PostgresCurrencyRevaluationRepository as PostgresCurrencyRevaluationRepo};
    pub use crate::autoinvoice::{AutoInvoiceEngine, PostgresAutoInvoiceRepository as PostgresAutoInvoiceRepo};
    pub use crate::credit_management::{CreditManagementEngine, PostgresCreditManagementRepository as PostgresCreditManagementRepo};
    pub use crate::treasury::{TreasuryEngine, PostgresTreasuryRepository as PostgresTreasuryRepo};
    pub use crate::grant_management::{GrantManagementEngine, PostgresGrantManagementRepository as PostgresGrantManagementRepo};
    pub use crate::corporate_card::{CorporateCardEngine, PostgresCorporateCardRepository as PostgresCorporateCardRepo};
    pub use crate::account_monitor::{AccountMonitorEngine, PostgresAccountMonitorRepository as PostgresAccountMonitorRepo};
    pub use crate::accounts_payable::{AccountsPayableEngine, PostgresAccountsPayableRepository as PostgresAccountsPayableRepo};
    pub use crate::accounts_receivable::{AccountsReceivableEngine, PostgresAccountsReceivableRepository as PostgresAccountsReceivableRepo};
    pub use crate::general_ledger::{GeneralLedgerEngine, PostgresGeneralLedgerRepository as PostgresGeneralLedgerRepo};
    pub use crate::asset_depreciation::{AssetDepreciationEngine, PostgresAssetDepreciationRepository as PostgresAssetDepreciationRepo};
    pub use crate::netting::{NettingEngine, PostgresNettingRepository as PostgresNettingRepo};
    pub use crate::financial_statements::{FinancialStatementEngine, PostgresFinancialStatementRepository as PostgresFinancialStatementRepo};
    pub use crate::journal_import::{JournalImportEngine, PostgresJournalImportRepository as PostgresJournalImportRepo};
    pub use crate::inflation_adjustment::{InflationAdjustmentEngine, PostgresInflationAdjustmentRepository as PostgresInflationAdjustmentRepo};
    pub use crate::impairment_management::{ImpairmentManagementEngine, PostgresImpairmentManagementRepository as PostgresImpairmentManagementRepo};
    pub use crate::bank_account_transfer::{BankAccountTransferEngine, PostgresBankAccountTransferRepository as PostgresBankAccountTransferRepo};
    pub use crate::tax_reporting::{TaxReportingEngine, PostgresTaxReportingRepository as PostgresTaxReportingRepo};
    pub use crate::deferred_revenue::{DeferredRevenueEngine, PostgresDeferredRevenueRepository as PostgresDeferredRevenueRepo};
    pub use crate::accounting_hub::{AccountingHubEngine, PostgresAccountingHubRepository as PostgresAccountingHubRepo};
    pub use crate::financial_controls::{FinancialControlsEngine, PostgresFinancialControlsRepository as PostgresFinancialControlsRepo};
    pub use crate::revenue_management::{RevenueManagementEngine, PostgresRevenueManagementRepository as PostgresRevenueManagementRepo};
    pub use crate::cash_flow_forecast::{CashFlowForecastEngine, PostgresCashFlowForecastRepository as PostgresCashFlowForecastRepo};
    pub use crate::regulatory_reporting::{RegulatoryReportingEngine, PostgresRegulatoryReportingRepository as PostgresRegulatoryReportingRepo};
    pub use crate::advance_payment::{AdvancePaymentEngine, PostgresAdvancePaymentRepository as PostgresAdvancePaymentRepo};
    pub use crate::customer_deposit::{CustomerDepositEngine, PostgresCustomerDepositRepository as PostgresCustomerDepositRepo};
    pub use crate::cash_position::{CashPositionEngine, PostgresCashPositionRepository as PostgresCashPositionRepo};
    pub use crate::payment_terms::{PaymentTermsEngine, PostgresPaymentTermsRepository as PostgresPaymentTermsRepo};
    pub use crate::lockbox::{LockboxEngine, PostgresLockboxRepository as PostgresLockboxRepo};
    pub use crate::ar_aging::{ArAgingEngine, PostgresArAgingRepository as PostgresArAgingRepo};
    pub use crate::ap_aging::{ApAgingEngine, PostgresApAgingRepository as PostgresApAgingRepo};
    pub use crate::financial_ratio::{FinancialRatioEngine, PostgresFinancialRatioRepository as PostgresFinancialRatioRepo};
    pub use crate::receipt_write_off::{ReceiptWriteOffEngine, PostgresReceiptWriteOffRepository as PostgresReceiptWriteOffRepo};
    pub use crate::mass_additions::{MassAdditionEngine, PostgresMassAdditionRepository as PostgresMassAdditionRepo};
    pub use crate::asset_reclassification::{AssetReclassificationEngine, PostgresAssetReclassificationRepository as PostgresAssetReclassificationRepo};
    pub use crate::gl_budget_transfer::{GlBudgetTransferEngine, PostgresGlBudgetTransferRepository as PostgresGlBudgetTransferRepo};
    pub use crate::payment_format::{PaymentFormatEngine, PostgresPaymentFormatRepository as PostgresPaymentFormatRepo};
    pub use crate::financial_dimension_set::{FinancialDimensionSetEngine, PostgresFinancialDimensionSetRepository as PostgresFinancialDimensionSetRepo};
    pub use crate::prepayment_application::{PrepaymentApplicationEngine, PostgresPrepaymentApplicationRepository as PostgresPrepaymentApplicationRepo};
    pub use crate::asset_retirement::{AssetRetirementEngine, PostgresAssetRetirementRepository as PostgresAssetRetirementRepo};
    pub use crate::cip_capitalization::{CipCapitalizationEngine, PostgresCipCapitalizationRepository as PostgresCipCapitalizationRepo};
    pub use crate::available_funds::{AvailableFundsEngine, PostgresAvailableFundsRepository as PostgresAvailableFundsRepo};
    pub use crate::statistical_accounting::{StatisticalAccountingEngine, PostgresStatisticalAccountingRepository as PostgresStatisticalAccountingRepo};
    pub use crate::receivables_factoring::{ReceivablesFactoringEngine, PostgresReceivablesFactoringRepository as PostgresReceivablesFactoringRepo};
    pub use crate::account_hierarchy::{AccountHierarchyEngine, PostgresAccountHierarchyRepository as PostgresAccountHierarchyRepo};
    pub use crate::suspense_account::{SuspenseAccountEngine, PostgresSuspenseAccountRepository as PostgresSuspenseAccountRepo};
    pub use crate::interest_invoice::{InterestInvoiceEngine, PostgresInterestInvoiceRepository as PostgresInterestInvoiceRepo};
    pub use crate::expense_policy_compliance::{ExpensePolicyComplianceEngine, PostgresExpensePolicyComplianceRepository as PostgresExpensePolicyComplianceRepo};
    pub use crate::bank_guarantee::{BankGuaranteeEngine, PostgresBankGuaranteeRepository as PostgresBankGuaranteeRepo};
    pub use crate::letter_of_credit::{LetterOfCreditEngine, PostgresLetterOfCreditRepository as PostgresLetterOfCreditRepo};
    pub use crate::hedge_management::{HedgeManagementEngine, repository::PostgresHedgeManagementRepository as PostgresHedgeManagementRepo};
    pub use crate::payment_risk::{PaymentRiskEngine, repository::PostgresPaymentRiskRepository as PostgresPaymentRiskRepo};
    pub use crate::tax_registration::{TaxRegistrationEngine, PostgresTaxRegistrationRepository as PostgresTaxRegistrationRepo};
    pub use crate::cash_concentration::{CashConcentrationEngine, PostgresCashConcentrationRepository as PostgresCashConcentrationRepo};
    pub use crate::customer_statement::{CustomerStatementEngine, PostgresCustomerStatementRepository as PostgresCustomerStatementRepo};
    pub use crate::remittance_batch::{RemittanceBatchEngine, PostgresRemittanceBatchRepository as PostgresRemittanceBatchRepo};
    pub use crate::chargeback_management::{ChargebackManagementEngine, PostgresChargebackManagementRepository as PostgresChargebackManagementRepo};
    pub use crate::finance_charge::{FinanceChargeEngine, PostgresFinanceChargeRepository};
    pub use crate::profitability_analysis::{ProfitabilityAnalysisEngine, PostgresProfitabilityAnalysisRepository as PostgresProfitabilityAnalysisRepo};
    pub use crate::recurring_invoice::{RecurringInvoiceEngine, PostgresRecurringInvoiceRepository as PostgresRecurringInvoiceRepo};
    pub use crate::payment_settlement::{PaymentSettlementEngine, PostgresPaymentSettlementRepository as PostgresPaymentSettlementRepo};
    pub use crate::payment_process_request::{PaymentProcessRequestEngine, PostgresPaymentProcessRequestRepository};
    pub use crate::invoice_batch::{InvoiceBatchEngine, PostgresInvoiceBatchRepository};
    pub use crate::doubtful_account_allowance::{DoubtfulAccountAllowanceEngine, PostgresDoubtfulAccountAllowanceRepository as PostgresDoubtfulAccountAllowanceRepo};
    pub use crate::invoice_matching::{InvoiceMatchingEngine, PostgresInvoiceMatchingRepository};
    pub use crate::distribution_set::{DistributionSetEngine, PostgresDistributionSetRepository};
    pub use crate::auto_offset::{AutoOffsetEngine, PostgresAutoOffsetRepository as PostgresAutoOffsetRepo};
    pub use crate::cash_flow_statement::{CashFlowStatementEngine, PostgresCashFlowStatementRepository as PostgresCashFlowStatementRepo};
    pub use crate::bank_statement_reconciliation::{BankStatementReconciliationEngine, PostgresBankStatementReconciliationRepository as PostgresBankStatementReconciliationRepo};
    pub use crate::third_party_payment::{ThirdPartyPaymentEngine, PostgresThirdPartyPaymentRepository as PostgresThirdPartyPaymentRepo};
    pub use crate::average_balance::{AverageBalanceEngine, PostgresAverageBalanceRepository as PostgresAverageBalanceRepo};
    pub use crate::cash_receipt::{CashReceiptEngine, PostgresCashReceiptRepository as PostgresCashReceiptRepo};
    pub use crate::direct_debit_mandate::{DirectDebitMandateEngine, PostgresDirectDebitMandateRepository as PostgresDirectDebitMandateRepo};
    pub use crate::multi_period_accounting::{MultiPeriodAccountingEngine, MpaRepository, PostgresMpaRepository as PostgresMpaRepo};
    pub use crate::dunning_letter_management::{DunningLetterManagementEngine, PostgresDunningLetterManagementRepository as PostgresDunningLetterManagementRepo};
}

pub mod hcm {
    pub use crate::absence::{AbsenceEngine, PostgresAbsenceRepository as PostgresAbsenceRepo};
    pub use crate::time_and_labor::{TimeAndLaborEngine, PostgresTimeAndLaborRepository as PostgresTimeAndLaborRepo};
    pub use crate::payroll::{PayrollEngine, PostgresPayrollRepository as PostgresPayrollRepo};
    pub use crate::compensation::{CompensationEngine, PostgresCompensationRepository as PostgresCompensationRepository};
    pub use crate::benefits::{BenefitsEngine, PostgresBenefitsRepository as PostgresBenefitsRepo};
    pub use crate::performance::{PerformanceEngine, PostgresPerformanceRepository as PostgresPerformanceRepo};
    pub use crate::recruiting::{RecruitingEngine, PostgresRecruitingRepository as PostgresRecruitingRepo};
    pub use crate::learning_management::{LearningManagementEngine, PostgresLearningManagementRepository as PostgresLearningManagementRepo};
    pub use crate::succession_planning::{SuccessionPlanningEngine, PostgresSuccessionPlanningRepository as PostgresSuccessionPlanningRepo};
    pub use crate::goal_management::{GoalManagementEngine, PostgresGoalManagementRepository as PostgresGoalManagementRepo};
    pub use crate::approval_authority::{ApprovalAuthorityEngine, PostgresApprovalAuthorityRepository as PostgresApprovalAuthorityRepo};
    pub use crate::approval_delegation::{ApprovalDelegationEngine, PostgresApprovalDelegationRepository as PostgresApprovalDelegationRepo};
    pub use crate::data_archiving::{DataArchivingEngine, PostgresDataArchivingRepository as PostgresDataArchivingRepo};
}

pub mod scm {
    pub use crate::sourcing::{SourcingEngine, PostgresSourcingRepository as PostgresSourcingRepo};
    pub use crate::procurement_contracts::{ProcurementContractEngine, PostgresProcurementContractRepository as PostgresProcurementContractRepo};
    pub use crate::inventory::{InventoryEngine, PostgresInventoryRepository as PostgresInventoryRepo};
    pub use crate::customer_returns::{CustomerReturnsEngine, PostgresCustomerReturnsRepository as PostgresCustomerReturnsRepo};
    pub use crate::pricing::{PricingEngine, PostgresPricingRepository as PostgresPricingRepo};
    pub use crate::purchase_requisition::{PurchaseRequisitionEngine, PostgresPurchaseRequisitionRepository as PostgresPurchaseRequisitionRepo};
    pub use crate::product_information::{ProductInformationEngine, PostgresProductInformationRepository as PostgresProductInformationRepo};
    pub use crate::quality_management::{QualityManagementEngine, PostgresQualityManagementRepository as PostgresQualityManagementRepo};
    pub use crate::order_management::{OrderManagementEngine, PostgresOrderManagementRepository as PostgresOrderManagementRepo};
    pub use crate::manufacturing::{ManufacturingEngine, PostgresManufacturingRepository as PostgresManufacturingRepo};
    pub use crate::warehouse_management::{WarehouseManagementEngine, PostgresWarehouseManagementRepository as PostgresWarehouseManagementRepo};
    pub use crate::shipping::{ShippingEngine, PostgresShippingRepository as PostgresShippingRepo};
    pub use crate::receiving::{ReceivingEngine, PostgresReceivingRepository as PostgresReceivingRepo};
    pub use crate::supplier_qualification::{SupplierQualificationEngine, PostgresSupplierQualificationRepository as PostgresSupplierQualificationRepo};
    pub use crate::supplier_scorecard::{SupplierScorecardEngine, PostgresScorecardRepository as PostgresScorecardRepo};
    pub use crate::landed_cost::{LandedCostEngine, PostgresLandedCostRepository as PostgresLandedCostRepo};
    pub use crate::contract_lifecycle::{ContractLifecycleEngine, PostgresContractLifecycleRepository as PostgresContractLifecycleRepo};
    pub use crate::demand_planning::{DemandPlanningEngine, PostgresDemandPlanningRepository as PostgresDemandPlanningRepo};
    pub use crate::supply_chain_planning::{SupplyChainPlanningEngine, PostgresPlanningRepository as PostgresPlanningRepo};
    pub use crate::product_configurator::{ProductConfiguratorEngine, PostgresProductConfiguratorRepository as PostgresProductConfiguratorRepo};
    pub use crate::transportation_management::{TransportationManagementEngine, PostgresTransportationManagementRepository as PostgresTransportationManagementRepo};
    pub use crate::channel_revenue::{ChannelRevenueEngine, PostgresChannelRevenueRepository as PostgresChannelRevenueRepo};
    pub use crate::rebate_management::{RebateManagementEngine, PostgresRebateManagementRepository as PostgresRebateManagementRepo};
}

pub mod crm {
    pub use crate::sales_commission::{SalesCommissionEngine, PostgresSalesCommissionRepository as PostgresSalesCommissionRepo};
    pub use crate::subscription::{SubscriptionEngine, PostgresSubscriptionRepository as PostgresSubscriptionRepo};
    pub use crate::lead_opportunity::{LeadOpportunityEngine, PostgresLeadOpportunityRepository as PostgresLeadOpportunityRepo};
    pub use crate::marketing::{MarketingEngine, PostgresMarketingRepository as PostgresMarketingRepo};
    pub use crate::service_request::{ServiceRequestEngine, PostgresServiceRequestRepository as PostgresServiceRequestRepo};
    pub use crate::loyalty_management::{LoyaltyManagementEngine, PostgresLoyaltyManagementRepository as PostgresLoyaltyManagementRepo};
    pub use crate::promotions_management::{PromotionsManagementEngine, PostgresPromotionsManagementRepository as PostgresPromotionsManagementRepo};
}

pub mod projects {
    pub use crate::project_costing::{ProjectCostingEngine, PostgresProjectCostingRepository as PostgresProjectCostingRepo};
    pub use crate::project_resource_management::{ProjectResourceManagementEngine, PostgresProjectResourceManagementRepository as PostgresProjectResourceManagementRepo};
    pub use crate::joint_venture::{JointVentureEngine, PostgresJointVentureRepository as PostgresJointVentureRepo};
    pub use crate::project_billing::{ProjectBillingEngine, PostgresProjectBillingRepository as PostgresProjectBillingRepo};
}

pub mod shared {
    pub use crate::lease::{LeaseAccountingEngine, PostgresLeaseAccountingRepository as PostgresLeaseAccountingRepo};
    pub use crate::cost_allocation::{CostAllocationEngine, PostgresCostAllocationRepository as PostgresCostAllocationRepo};
    pub use crate::financial_reporting::{FinancialReportingEngine, PostgresFinancialReportingRepository as PostgresFinancialReportingRepo};
    pub use crate::descriptive_flexfield::{DescriptiveFlexfieldEngine, PostgresDescriptiveFlexfieldRepository as PostgresDescriptiveFlexfieldRepo};
    pub use crate::cross_validation::{CrossValidationEngine, PostgresCrossValidationRepository as PostgresCrossValidationRepo};
    pub use crate::scheduled_process::{ScheduledProcessEngine, PostgresScheduledProcessRepository as PostgresScheduledProcessRepo};
    pub use crate::segregation_of_duties::{SegregationOfDutiesEngine, PostgresSegregationOfDutiesRepository as PostgresSegregationOfDutiesRepo};
    pub use crate::kpi::{KpiEngine, PostgresKpiRepository as PostgresKpiRepo};
    pub use crate::enterprise_asset_management::{EnterpriseAssetManagementEngine, PostgresAssetManagementRepository as PostgresAssetManagementRepo};
    pub use crate::risk_management::{RiskManagementEngine, PostgresRiskManagementRepository as PostgresRiskManagementRepo};
    pub use crate::sustainability::{SustainabilityEngine, PostgresSustainabilityRepository as PostgresSustainabilityRepo};
    pub use crate::engineering_change_management::{EngineeringChangeEngine, PostgresEngineeringChangeManagementRepository as PostgresEcmRepository};
    pub use crate::health_safety::{HealthSafetyEngine, PostgresHealthSafetyRepository as PostgresHealthSafetyRepo};
    pub use crate::transfer_pricing::{TransferPricingEngine, PostgresTransferPricingRepository as PostgresTransferPricingRepo};
    pub use crate::cost_accounting::{CostAccountingEngine, PostgresCostAccountingRepository as PostgresCostAccountingRepo};
    pub use crate::funds_reservation::{FundsReservationEngine, PostgresFundsReservationRepository as PostgresFundsReservationRepo};
    pub use crate::territory_management::{TerritoryManagementEngine, PostgresTerritoryManagementRepository as PostgresTerritoryManagementRepo};
}

// Re-export domain modules at top level for backward compatibility
pub use financials::*;
pub use hcm::*;
pub use scm::*;
pub use crm::*;
pub use projects::*;
pub use shared::*;

// --- Module Declarations ---

// Financials
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
pub mod withholding_tax;
pub mod multi_book;
pub mod financial_consolidation;
pub mod recurring_journal;
pub mod manual_journal;
pub mod document_sequencing;
pub mod transaction_calendar;
pub mod allocation;
pub mod currency_revaluation;
pub mod autoinvoice;
pub mod credit_management;
pub mod treasury;
pub mod grant_management;
pub mod corporate_card;
pub mod account_monitor;
pub mod accounts_payable;
pub mod accounts_receivable;
pub mod general_ledger;
pub mod asset_depreciation;
pub mod netting;
pub mod financial_statements;
pub mod journal_import;
pub mod inflation_adjustment;
pub mod impairment_management;
pub mod bank_account_transfer;
pub mod tax_reporting;
pub mod deferred_revenue;
pub mod accounting_hub;
pub mod financial_controls;
pub mod revenue_management;
pub mod cash_flow_forecast;
pub mod cash_flow_statement;
pub mod regulatory_reporting;
pub mod advance_payment;
pub mod customer_deposit;
pub mod cash_position;
pub mod payment_terms;
pub mod lockbox;
pub mod ar_aging;
pub mod ap_aging;
pub mod financial_ratio;
pub mod receipt_write_off;
pub mod mass_additions;
pub mod asset_reclassification;
pub mod gl_budget_transfer;
pub mod payment_format;
pub mod financial_dimension_set;
pub mod prepayment_application;
pub mod asset_retirement;
pub mod cip_capitalization;
pub mod available_funds;
pub mod statistical_accounting;
pub mod receivables_factoring;
pub mod account_hierarchy;
pub mod suspense_account;
pub mod interest_invoice;
pub mod expense_policy_compliance;
pub mod bank_guarantee;
pub mod letter_of_credit;
pub mod hedge_management;
pub mod payment_risk;
pub mod tax_registration;
pub mod cash_concentration;
pub mod customer_statement;
pub mod remittance_batch;
pub mod chargeback_management;
pub mod finance_charge;
pub mod profitability_analysis;
pub mod recurring_invoice;
pub mod payment_settlement;
pub mod payment_process_request;
pub mod invoice_batch;
pub mod doubtful_account_allowance;
pub mod invoice_matching;
pub mod distribution_set;
pub mod auto_offset;
pub mod bank_statement_reconciliation;
pub mod third_party_payment;
pub mod average_balance;
pub mod cash_receipt;
pub mod direct_debit_mandate;
pub mod multi_period_accounting;
pub mod dunning_letter_management;

// HCM
pub mod absence;
pub mod time_and_labor;
pub mod payroll;
pub mod compensation;
pub mod benefits;
pub mod performance;
pub mod recruiting;
pub mod learning_management;
pub mod succession_planning;
pub mod goal_management;
pub mod approval_authority;
pub mod approval_delegation;
pub mod data_archiving;

// SCM
pub mod sourcing;
pub mod procurement_contracts;
pub mod inventory;
pub mod customer_returns;
pub mod pricing;
pub mod purchase_requisition;
pub mod product_information;
pub mod quality_management;
pub mod order_management;
pub mod manufacturing;
pub mod warehouse_management;
pub mod shipping;
pub mod receiving;
pub mod supplier_qualification;
pub mod supplier_scorecard;
pub mod landed_cost;
pub mod contract_lifecycle;
pub mod demand_planning;
pub mod supply_chain_planning;
pub mod product_configurator;
pub mod transportation_management;
pub mod channel_revenue;
pub mod rebate_management;

// CRM
pub mod sales_commission;
pub mod subscription;
pub mod lead_opportunity;
pub mod marketing;
pub mod service_request;
pub mod loyalty_management;
pub mod promotions_management;

// Projects
pub mod project_costing;
pub mod project_resource_management;
pub mod joint_venture;
pub mod project_billing;

// Shared
pub mod lease;
pub mod cost_allocation;
pub mod financial_reporting;
pub mod descriptive_flexfield;
pub mod cross_validation;
pub mod scheduled_process;
pub mod segregation_of_duties;
pub mod kpi;
pub mod enterprise_asset_management;
pub mod risk_management;
pub mod sustainability;
pub mod engineering_change_management;
pub mod health_safety;
pub mod transfer_pricing;
pub mod cost_accounting;
pub mod funds_reservation;
pub mod territory_management;

mod mock_repos;
pub use mock_repos::*;
