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
pub mod approval;
pub mod audit;
pub mod config;
pub mod eventbus;
pub mod formula;
pub mod notification;
pub mod schema;
pub mod security;
pub mod validation;
pub mod workflow;

// Re-export core engines at the top level
pub use approval::{ApprovalEngine, PostgresApprovalRepository as PostgresApprovalRepo};
pub use audit::*;
pub use config::*;
pub use eventbus::*;
pub use formula::*;
pub use notification::{
    NotificationEngine, PostgresNotificationRepository as PostgresNotificationRepo,
};
pub use schema::*;
pub use security::*;
pub use validation::*;
pub use workflow::engine::User as WorkflowUser;
pub use workflow::{
    repository::{
        InMemoryWorkflowStateRepository, PostgresWorkflowStateRepository, WorkflowStateRepository,
    },
    ActionExecutor, ActionResult, AvailableTransitions, GuardEvaluator, GuardResult,
    StateHistoryEntry, StateMachine, TransitionInfo, TransitionResult, WorkflowEngine,
    WorkflowState,
};

// --- Domain Modules (Grouped for better organization) ---

pub mod financials {
    pub use crate::account_hierarchy::{
        AccountHierarchyEngine, PostgresAccountHierarchyRepository as PostgresAccountHierarchyRepo,
    };
    pub use crate::account_monitor::{
        AccountMonitorEngine, PostgresAccountMonitorRepository as PostgresAccountMonitorRepo,
    };
    pub use crate::accounting_hub::{
        AccountingHubEngine, PostgresAccountingHubRepository as PostgresAccountingHubRepo,
    };
    pub use crate::accounts_payable::{
        AccountsPayableEngine, PostgresAccountsPayableRepository as PostgresAccountsPayableRepo,
    };
    pub use crate::accounts_receivable::{
        AccountsReceivableEngine,
        PostgresAccountsReceivableRepository as PostgresAccountsReceivableRepo,
    };
    pub use crate::advance_payment::{
        AdvancePaymentEngine, PostgresAdvancePaymentRepository as PostgresAdvancePaymentRepo,
    };
    pub use crate::allocation::{
        AllocationEngine, PostgresAllocationRepository as PostgresAllocationRepo,
    };
    pub use crate::ap_aging::{ApAgingEngine, PostgresApAgingRepository as PostgresApAgingRepo};
    pub use crate::ar_aging::{ArAgingEngine, PostgresArAgingRepository as PostgresArAgingRepo};
    pub use crate::asset_depreciation::{
        AssetDepreciationEngine,
        PostgresAssetDepreciationRepository as PostgresAssetDepreciationRepo,
    };
    pub use crate::asset_reclassification::{
        AssetReclassificationEngine,
        PostgresAssetReclassificationRepository as PostgresAssetReclassificationRepo,
    };
    pub use crate::asset_retirement::{
        AssetRetirementEngine, PostgresAssetRetirementRepository as PostgresAssetRetirementRepo,
    };
    pub use crate::auto_offset::{
        AutoOffsetEngine, PostgresAutoOffsetRepository as PostgresAutoOffsetRepo,
    };
    pub use crate::autoinvoice::{
        AutoInvoiceEngine, PostgresAutoInvoiceRepository as PostgresAutoInvoiceRepo,
    };
    pub use crate::available_funds::{
        AvailableFundsEngine, PostgresAvailableFundsRepository as PostgresAvailableFundsRepo,
    };
    pub use crate::average_balance::{
        AverageBalanceEngine, PostgresAverageBalanceRepository as PostgresAverageBalanceRepo,
    };
    pub use crate::bank_account_transfer::{
        BankAccountTransferEngine,
        PostgresBankAccountTransferRepository as PostgresBankAccountTransferRepo,
    };
    pub use crate::bank_guarantee::{
        BankGuaranteeEngine, PostgresBankGuaranteeRepository as PostgresBankGuaranteeRepo,
    };
    pub use crate::bank_statement_reconciliation::{
        BankStatementReconciliationEngine,
        PostgresBankStatementReconciliationRepository as PostgresBankStatementReconciliationRepo,
    };
    pub use crate::budget::{BudgetEngine, PostgresBudgetRepository as PostgresBudgetRepo};
    pub use crate::cash_concentration::{
        CashConcentrationEngine,
        PostgresCashConcentrationRepository as PostgresCashConcentrationRepo,
    };
    pub use crate::cash_flow_forecast::{
        CashFlowForecastEngine, PostgresCashFlowForecastRepository as PostgresCashFlowForecastRepo,
    };
    pub use crate::cash_flow_statement::{
        CashFlowStatementEngine,
        PostgresCashFlowStatementRepository as PostgresCashFlowStatementRepo,
    };
    pub use crate::cash_management::{
        CashManagementEngine, PostgresCashManagementRepository as PostgresCashManagementRepo,
    };
    pub use crate::cash_position::{
        CashPositionEngine, PostgresCashPositionRepository as PostgresCashPositionRepo,
    };
    pub use crate::cash_receipt::{
        CashReceiptEngine, PostgresCashReceiptRepository as PostgresCashReceiptRepo,
    };
    pub use crate::chargeback_management::{
        ChargebackManagementEngine,
        PostgresChargebackManagementRepository as PostgresChargebackManagementRepo,
    };
    pub use crate::cip_capitalization::{
        CipCapitalizationEngine,
        PostgresCipCapitalizationRepository as PostgresCipCapitalizationRepo,
    };
    pub use crate::collections::{
        CollectionsEngine, PostgresCollectionsRepository as PostgresCollectionsRepo,
    };
    pub use crate::corporate_card::{
        CorporateCardEngine, PostgresCorporateCardRepository as PostgresCorporateCardRepo,
    };
    pub use crate::credit_management::{
        CreditManagementEngine, PostgresCreditManagementRepository as PostgresCreditManagementRepo,
    };
    pub use crate::currency::{CurrencyEngine, PostgresCurrencyRepository as PostgresCurrencyRepo};
    pub use crate::currency_revaluation::{
        CurrencyRevaluationEngine,
        PostgresCurrencyRevaluationRepository as PostgresCurrencyRevaluationRepo,
    };
    pub use crate::customer_deposit::{
        CustomerDepositEngine, PostgresCustomerDepositRepository as PostgresCustomerDepositRepo,
    };
    pub use crate::customer_statement::{
        CustomerStatementEngine,
        PostgresCustomerStatementRepository as PostgresCustomerStatementRepo,
    };
    pub use crate::deferred_revenue::{
        DeferredRevenueEngine, PostgresDeferredRevenueRepository as PostgresDeferredRevenueRepo,
    };
    pub use crate::direct_debit_mandate::{
        DirectDebitMandateEngine,
        PostgresDirectDebitMandateRepository as PostgresDirectDebitMandateRepo,
    };
    pub use crate::distribution_set::{DistributionSetEngine, PostgresDistributionSetRepository};
    pub use crate::document_sequencing::{
        DocumentSequencingEngine,
        PostgresDocumentSequencingRepository as PostgresDocumentSequencingRepo,
    };
    pub use crate::doubtful_account_allowance::{
        DoubtfulAccountAllowanceEngine,
        PostgresDoubtfulAccountAllowanceRepository as PostgresDoubtfulAccountAllowanceRepo,
    };
    pub use crate::dunning_letter_management::{
        DunningLetterManagementEngine,
        PostgresDunningLetterManagementRepository as PostgresDunningLetterManagementRepo,
    };
    pub use crate::encumbrance::{
        EncumbranceEngine, PostgresEncumbranceRepository as PostgresEncumbranceRepo,
    };
    pub use crate::expense::{ExpenseEngine, PostgresExpenseRepository as PostgresExpenseRepo};
    pub use crate::expense_policy_compliance::{
        ExpensePolicyComplianceEngine,
        PostgresExpensePolicyComplianceRepository as PostgresExpensePolicyComplianceRepo,
    };
    pub use crate::finance_charge::{FinanceChargeEngine, PostgresFinanceChargeRepository};
    pub use crate::financial_consolidation::{
        FinancialConsolidationEngine,
        PostgresFinancialConsolidationRepository as PostgresFinancialConsolidationRepo,
    };
    pub use crate::financial_controls::{
        FinancialControlsEngine,
        PostgresFinancialControlsRepository as PostgresFinancialControlsRepo,
    };
    pub use crate::financial_dimension_set::{
        FinancialDimensionSetEngine,
        PostgresFinancialDimensionSetRepository as PostgresFinancialDimensionSetRepo,
    };
    pub use crate::financial_ratio::{
        FinancialRatioEngine, PostgresFinancialRatioRepository as PostgresFinancialRatioRepo,
    };
    pub use crate::financial_statements::{
        FinancialStatementEngine,
        PostgresFinancialStatementRepository as PostgresFinancialStatementRepo,
    };
    pub use crate::fixed_assets::{
        FixedAssetEngine, PostgresFixedAssetRepository as PostgresFixedAssetRepo,
    };
    pub use crate::general_ledger::{
        GeneralLedgerEngine, PostgresGeneralLedgerRepository as PostgresGeneralLedgerRepo,
    };
    pub use crate::gl_budget_transfer::{
        GlBudgetTransferEngine, PostgresGlBudgetTransferRepository as PostgresGlBudgetTransferRepo,
    };
    pub use crate::grant_management::{
        GrantManagementEngine, PostgresGrantManagementRepository as PostgresGrantManagementRepo,
    };
    pub use crate::hedge_management::{
        repository::PostgresHedgeManagementRepository as PostgresHedgeManagementRepo,
        HedgeManagementEngine,
    };
    pub use crate::impairment_management::{
        ImpairmentManagementEngine,
        PostgresImpairmentManagementRepository as PostgresImpairmentManagementRepo,
    };
    pub use crate::inflation_adjustment::{
        InflationAdjustmentEngine,
        PostgresInflationAdjustmentRepository as PostgresInflationAdjustmentRepo,
    };
    pub use crate::intercompany::{
        IntercompanyEngine, PostgresIntercompanyRepository as PostgresIntercompanyRepo,
    };
    pub use crate::interest_invoice::{
        InterestInvoiceEngine, PostgresInterestInvoiceRepository as PostgresInterestInvoiceRepo,
    };
    pub use crate::invoice_batch::{InvoiceBatchEngine, PostgresInvoiceBatchRepository};
    pub use crate::invoice_matching::{InvoiceMatchingEngine, PostgresInvoiceMatchingRepository};
    pub use crate::journal_import::{
        JournalImportEngine, PostgresJournalImportRepository as PostgresJournalImportRepo,
    };
    pub use crate::letter_of_credit::{
        LetterOfCreditEngine, PostgresLetterOfCreditRepository as PostgresLetterOfCreditRepo,
    };
    pub use crate::lockbox::{LockboxEngine, PostgresLockboxRepository as PostgresLockboxRepo};
    pub use crate::manual_journal::{
        ManualJournalEngine, PostgresManualJournalRepository as PostgresManualJournalRepo,
    };
    pub use crate::mass_additions::{
        MassAdditionEngine, PostgresMassAdditionRepository as PostgresMassAdditionRepo,
    };
    pub use crate::multi_book::{
        MultiBookAccountingEngine,
        PostgresMultiBookAccountingRepository as PostgresMultiBookAccountingRepo,
    };
    pub use crate::multi_period_accounting::{
        MpaRepository, MultiPeriodAccountingEngine, PostgresMpaRepository as PostgresMpaRepo,
    };
    pub use crate::netting::{NettingEngine, PostgresNettingRepository as PostgresNettingRepo};
    pub use crate::payment::{PaymentEngine, PostgresPaymentRepository as PostgresPaymentRepo};
    pub use crate::payment_format::{
        PaymentFormatEngine, PostgresPaymentFormatRepository as PostgresPaymentFormatRepo,
    };
    pub use crate::payment_process_request::{
        PaymentProcessRequestEngine, PostgresPaymentProcessRequestRepository,
    };
    pub use crate::payment_risk::{
        repository::PostgresPaymentRiskRepository as PostgresPaymentRiskRepo, PaymentRiskEngine,
    };
    pub use crate::payment_settlement::{
        PaymentSettlementEngine,
        PostgresPaymentSettlementRepository as PostgresPaymentSettlementRepo,
    };
    pub use crate::payment_terms::{
        PaymentTermsEngine, PostgresPaymentTermsRepository as PostgresPaymentTermsRepo,
    };
    pub use crate::period_close::{
        PeriodCloseEngine, PostgresPeriodCloseRepository as PostgresPeriodCloseRepo,
    };
    pub use crate::prepayment_application::{
        PostgresPrepaymentApplicationRepository as PostgresPrepaymentApplicationRepo,
        PrepaymentApplicationEngine,
    };
    pub use crate::profitability_analysis::{
        PostgresProfitabilityAnalysisRepository as PostgresProfitabilityAnalysisRepo,
        ProfitabilityAnalysisEngine,
    };
    pub use crate::receipt_write_off::{
        PostgresReceiptWriteOffRepository as PostgresReceiptWriteOffRepo, ReceiptWriteOffEngine,
    };
    pub use crate::receivables_factoring::{
        PostgresReceivablesFactoringRepository as PostgresReceivablesFactoringRepo,
        ReceivablesFactoringEngine,
    };
    pub use crate::reconciliation::{
        PostgresReconciliationRepository as PostgresReconciliationRepo, ReconciliationEngine,
    };
    pub use crate::recurring_invoice::{
        PostgresRecurringInvoiceRepository as PostgresRecurringInvoiceRepo, RecurringInvoiceEngine,
    };
    pub use crate::recurring_journal::{
        PostgresRecurringJournalRepository as PostgresRecurringJournalRepo, RecurringJournalEngine,
    };
    pub use crate::regulatory_reporting::{
        PostgresRegulatoryReportingRepository as PostgresRegulatoryReportingRepo,
        RegulatoryReportingEngine,
    };
    pub use crate::remittance_batch::{
        PostgresRemittanceBatchRepository as PostgresRemittanceBatchRepo, RemittanceBatchEngine,
    };
    pub use crate::revenue::{PostgresRevenueRepository as PostgresRevenueRepo, RevenueEngine};
    pub use crate::revenue_management::{
        PostgresRevenueManagementRepository as PostgresRevenueManagementRepo,
        RevenueManagementEngine,
    };
    pub use crate::statistical_accounting::{
        PostgresStatisticalAccountingRepository as PostgresStatisticalAccountingRepo,
        StatisticalAccountingEngine,
    };
    pub use crate::subledger_accounting::{
        PostgresSubledgerAccountingRepository as PostgresSubledgerAccountingRepo,
        SubledgerAccountingEngine,
    };
    pub use crate::suspense_account::{
        PostgresSuspenseAccountRepository as PostgresSuspenseAccountRepo, SuspenseAccountEngine,
    };
    pub use crate::tax::{PostgresTaxRepository as PostgresTaxRepo, TaxEngine};
    pub use crate::tax_registration::{
        PostgresTaxRegistrationRepository as PostgresTaxRegistrationRepo, TaxRegistrationEngine,
    };
    pub use crate::tax_reporting::{
        PostgresTaxReportingRepository as PostgresTaxReportingRepo, TaxReportingEngine,
    };
    pub use crate::third_party_payment::{
        PostgresThirdPartyPaymentRepository as PostgresThirdPartyPaymentRepo,
        ThirdPartyPaymentEngine,
    };
    pub use crate::transaction_calendar::{
        PostgresTransactionCalendarRepository as PostgresTransactionCalendarRepo,
        TransactionCalendarEngine,
    };
    pub use crate::treasury::{PostgresTreasuryRepository as PostgresTreasuryRepo, TreasuryEngine};
    pub use crate::withholding_tax::{
        PostgresWithholdingTaxRepository as PostgresWithholdingTaxRepo, WithholdingTaxEngine,
    };
}

pub mod hcm {
    pub use crate::absence::{AbsenceEngine, PostgresAbsenceRepository as PostgresAbsenceRepo};
    pub use crate::approval_authority::{
        ApprovalAuthorityEngine,
        PostgresApprovalAuthorityRepository as PostgresApprovalAuthorityRepo,
    };
    pub use crate::approval_delegation::{
        ApprovalDelegationEngine,
        PostgresApprovalDelegationRepository as PostgresApprovalDelegationRepo,
    };
    pub use crate::benefits::{BenefitsEngine, PostgresBenefitsRepository as PostgresBenefitsRepo};
    pub use crate::compensation::{CompensationEngine, PostgresCompensationRepository};
    pub use crate::data_archiving::{
        DataArchivingEngine, PostgresDataArchivingRepository as PostgresDataArchivingRepo,
    };
    pub use crate::goal_management::{
        GoalManagementEngine, PostgresGoalManagementRepository as PostgresGoalManagementRepo,
    };
    pub use crate::learning_management::{
        LearningManagementEngine,
        PostgresLearningManagementRepository as PostgresLearningManagementRepo,
    };
    pub use crate::payroll::{PayrollEngine, PostgresPayrollRepository as PostgresPayrollRepo};
    pub use crate::performance::{
        PerformanceEngine, PostgresPerformanceRepository as PostgresPerformanceRepo,
    };
    pub use crate::recruiting::{
        PostgresRecruitingRepository as PostgresRecruitingRepo, RecruitingEngine,
    };
    pub use crate::succession_planning::{
        PostgresSuccessionPlanningRepository as PostgresSuccessionPlanningRepo,
        SuccessionPlanningEngine,
    };
    pub use crate::time_and_labor::{
        PostgresTimeAndLaborRepository as PostgresTimeAndLaborRepo, TimeAndLaborEngine,
    };
}

pub mod scm {
    pub use crate::channel_revenue::{
        ChannelRevenueEngine, PostgresChannelRevenueRepository as PostgresChannelRevenueRepo,
    };
    pub use crate::contract_lifecycle::{
        ContractLifecycleEngine,
        PostgresContractLifecycleRepository as PostgresContractLifecycleRepo,
    };
    pub use crate::customer_returns::{
        CustomerReturnsEngine, PostgresCustomerReturnsRepository as PostgresCustomerReturnsRepo,
    };
    pub use crate::demand_planning::{
        DemandPlanningEngine, PostgresDemandPlanningRepository as PostgresDemandPlanningRepo,
    };
    pub use crate::inventory::{
        InventoryEngine, PostgresInventoryRepository as PostgresInventoryRepo,
    };
    pub use crate::landed_cost::{
        LandedCostEngine, PostgresLandedCostRepository as PostgresLandedCostRepo,
    };
    pub use crate::manufacturing::{
        ManufacturingEngine, PostgresManufacturingRepository as PostgresManufacturingRepo,
    };
    pub use crate::order_management::{
        OrderManagementEngine, PostgresOrderManagementRepository as PostgresOrderManagementRepo,
    };
    pub use crate::pricing::{PostgresPricingRepository as PostgresPricingRepo, PricingEngine};
    pub use crate::procurement_contracts::{
        PostgresProcurementContractRepository as PostgresProcurementContractRepo,
        ProcurementContractEngine,
    };
    pub use crate::product_configurator::{
        PostgresProductConfiguratorRepository as PostgresProductConfiguratorRepo,
        ProductConfiguratorEngine,
    };
    pub use crate::product_information::{
        PostgresProductInformationRepository as PostgresProductInformationRepo,
        ProductInformationEngine,
    };
    pub use crate::purchase_requisition::{
        PostgresPurchaseRequisitionRepository as PostgresPurchaseRequisitionRepo,
        PurchaseRequisitionEngine,
    };
    pub use crate::quality_management::{
        PostgresQualityManagementRepository as PostgresQualityManagementRepo,
        QualityManagementEngine,
    };
    pub use crate::rebate_management::{
        PostgresRebateManagementRepository as PostgresRebateManagementRepo, RebateManagementEngine,
    };
    pub use crate::receiving::{
        PostgresReceivingRepository as PostgresReceivingRepo, ReceivingEngine,
    };
    pub use crate::shipping::{PostgresShippingRepository as PostgresShippingRepo, ShippingEngine};
    pub use crate::sourcing::{PostgresSourcingRepository as PostgresSourcingRepo, SourcingEngine};
    pub use crate::supplier_qualification::{
        PostgresSupplierQualificationRepository as PostgresSupplierQualificationRepo,
        SupplierQualificationEngine,
    };
    pub use crate::supplier_scorecard::{
        PostgresScorecardRepository as PostgresScorecardRepo, SupplierScorecardEngine,
    };
    pub use crate::supply_chain_planning::{
        PostgresPlanningRepository as PostgresPlanningRepo, SupplyChainPlanningEngine,
    };
    pub use crate::transportation_management::{
        PostgresTransportationManagementRepository as PostgresTransportationManagementRepo,
        TransportationManagementEngine,
    };
    pub use crate::warehouse_management::{
        PostgresWarehouseManagementRepository as PostgresWarehouseManagementRepo,
        WarehouseManagementEngine,
    };
}

pub mod crm {
    pub use crate::lead_opportunity::{
        LeadOpportunityEngine, PostgresLeadOpportunityRepository as PostgresLeadOpportunityRepo,
    };
    pub use crate::loyalty_management::{
        LoyaltyManagementEngine,
        PostgresLoyaltyManagementRepository as PostgresLoyaltyManagementRepo,
    };
    pub use crate::marketing::{
        MarketingEngine, PostgresMarketingRepository as PostgresMarketingRepo,
    };
    pub use crate::promotions_management::{
        PostgresPromotionsManagementRepository as PostgresPromotionsManagementRepo,
        PromotionsManagementEngine,
    };
    pub use crate::sales_commission::{
        PostgresSalesCommissionRepository as PostgresSalesCommissionRepo, SalesCommissionEngine,
    };
    pub use crate::service_request::{
        PostgresServiceRequestRepository as PostgresServiceRequestRepo, ServiceRequestEngine,
    };
    pub use crate::subscription::{
        PostgresSubscriptionRepository as PostgresSubscriptionRepo, SubscriptionEngine,
    };
}

pub mod projects {
    pub use crate::joint_venture::{
        JointVentureEngine, PostgresJointVentureRepository as PostgresJointVentureRepo,
    };
    pub use crate::project_billing::{
        PostgresProjectBillingRepository as PostgresProjectBillingRepo, ProjectBillingEngine,
    };
    pub use crate::project_costing::{
        PostgresProjectCostingRepository as PostgresProjectCostingRepo, ProjectCostingEngine,
    };
    pub use crate::project_resource_management::{
        PostgresProjectResourceManagementRepository as PostgresProjectResourceManagementRepo,
        ProjectResourceManagementEngine,
    };
}

pub mod shared {
    pub use crate::cost_accounting::{
        CostAccountingEngine, PostgresCostAccountingRepository as PostgresCostAccountingRepo,
    };
    pub use crate::cost_allocation::{
        CostAllocationEngine, PostgresCostAllocationRepository as PostgresCostAllocationRepo,
    };
    pub use crate::cross_validation::{
        CrossValidationEngine, PostgresCrossValidationRepository as PostgresCrossValidationRepo,
    };
    pub use crate::descriptive_flexfield::{
        DescriptiveFlexfieldEngine,
        PostgresDescriptiveFlexfieldRepository as PostgresDescriptiveFlexfieldRepo,
    };
    pub use crate::engineering_change_management::{
        EngineeringChangeEngine,
        PostgresEngineeringChangeManagementRepository as PostgresEcmRepository,
    };
    pub use crate::enterprise_asset_management::{
        EnterpriseAssetManagementEngine,
        PostgresAssetManagementRepository as PostgresAssetManagementRepo,
    };
    pub use crate::financial_reporting::{
        FinancialReportingEngine,
        PostgresFinancialReportingRepository as PostgresFinancialReportingRepo,
    };
    pub use crate::funds_reservation::{
        FundsReservationEngine, PostgresFundsReservationRepository as PostgresFundsReservationRepo,
    };
    pub use crate::health_safety::{
        HealthSafetyEngine, PostgresHealthSafetyRepository as PostgresHealthSafetyRepo,
    };
    pub use crate::kpi::{KpiEngine, PostgresKpiRepository as PostgresKpiRepo};
    pub use crate::lease::{
        LeaseAccountingEngine, PostgresLeaseAccountingRepository as PostgresLeaseAccountingRepo,
    };
    pub use crate::risk_management::{
        PostgresRiskManagementRepository as PostgresRiskManagementRepo, RiskManagementEngine,
    };
    pub use crate::scheduled_process::{
        PostgresScheduledProcessRepository as PostgresScheduledProcessRepo, ScheduledProcessEngine,
    };
    pub use crate::segregation_of_duties::{
        PostgresSegregationOfDutiesRepository as PostgresSegregationOfDutiesRepo,
        SegregationOfDutiesEngine,
    };
    pub use crate::sustainability::{
        PostgresSustainabilityRepository as PostgresSustainabilityRepo, SustainabilityEngine,
    };
    pub use crate::territory_management::{
        PostgresTerritoryManagementRepository as PostgresTerritoryManagementRepo,
        TerritoryManagementEngine,
    };
    pub use crate::transfer_pricing::{
        PostgresTransferPricingRepository as PostgresTransferPricingRepo, TransferPricingEngine,
    };
}

// Re-export domain modules at top level for backward compatibility
pub use crm::*;
pub use financials::*;
pub use hcm::*;
pub use projects::*;
pub use scm::*;
pub use shared::*;

// --- Module Declarations ---

// Financials
pub mod account_hierarchy;
pub mod account_monitor;
pub mod accounting_hub;
pub mod accounts_payable;
pub mod accounts_receivable;
pub mod advance_payment;
pub mod allocation;
pub mod ap_aging;
pub mod ar_aging;
pub mod asset_depreciation;
pub mod asset_reclassification;
pub mod asset_retirement;
pub mod auto_offset;
pub mod autoinvoice;
pub mod available_funds;
pub mod average_balance;
pub mod bank_account_transfer;
pub mod bank_guarantee;
pub mod bank_statement_reconciliation;
pub mod budget;
pub mod cash_concentration;
pub mod cash_flow_forecast;
pub mod cash_flow_statement;
pub mod cash_management;
pub mod cash_position;
pub mod cash_receipt;
pub mod chargeback_management;
pub mod cip_capitalization;
pub mod collections;
pub mod corporate_card;
pub mod credit_management;
pub mod currency;
pub mod currency_revaluation;
pub mod customer_deposit;
pub mod customer_statement;
pub mod deferred_revenue;
pub mod direct_debit_mandate;
pub mod distribution_set;
pub mod document_sequencing;
pub mod doubtful_account_allowance;
pub mod dunning_letter_management;
pub mod encumbrance;
pub mod expense;
pub mod expense_policy_compliance;
pub mod finance_charge;
pub mod financial_consolidation;
pub mod financial_controls;
pub mod financial_dimension_set;
pub mod financial_ratio;
pub mod financial_statements;
pub mod fixed_assets;
pub mod general_ledger;
pub mod gl_budget_transfer;
pub mod grant_management;
pub mod hedge_management;
pub mod impairment_management;
pub mod inflation_adjustment;
pub mod intercompany;
pub mod interest_invoice;
pub mod invoice_batch;
pub mod invoice_matching;
pub mod journal_import;
pub mod letter_of_credit;
pub mod lockbox;
pub mod manual_journal;
pub mod mass_additions;
pub mod multi_book;
pub mod multi_period_accounting;
pub mod netting;
pub mod payment;
pub mod payment_format;
pub mod payment_process_request;
pub mod payment_risk;
pub mod payment_settlement;
pub mod payment_terms;
pub mod period_close;
pub mod prepayment_application;
pub mod profitability_analysis;
pub mod receipt_write_off;
pub mod receivables_factoring;
pub mod reconciliation;
pub mod recurring_invoice;
pub mod recurring_journal;
pub mod regulatory_reporting;
pub mod remittance_batch;
pub mod revenue;
pub mod revenue_management;
pub mod statistical_accounting;
pub mod subledger_accounting;
pub mod suspense_account;
pub mod tax;
pub mod tax_registration;
pub mod tax_reporting;
pub mod third_party_payment;
pub mod transaction_calendar;
pub mod treasury;
pub mod withholding_tax;

// HCM
pub mod absence;
pub mod approval_authority;
pub mod approval_delegation;
pub mod benefits;
pub mod compensation;
pub mod data_archiving;
pub mod goal_management;
pub mod learning_management;
pub mod payroll;
pub mod performance;
pub mod recruiting;
pub mod succession_planning;
pub mod time_and_labor;

// SCM
pub mod channel_revenue;
pub mod contract_lifecycle;
pub mod customer_returns;
pub mod demand_planning;
pub mod inventory;
pub mod landed_cost;
pub mod manufacturing;
pub mod order_management;
pub mod pricing;
pub mod procurement_contracts;
pub mod product_configurator;
pub mod product_information;
pub mod purchase_requisition;
pub mod quality_management;
pub mod rebate_management;
pub mod receiving;
pub mod shipping;
pub mod sourcing;
pub mod supplier_qualification;
pub mod supplier_scorecard;
pub mod supply_chain_planning;
pub mod transportation_management;
pub mod warehouse_management;

// CRM
pub mod lead_opportunity;
pub mod loyalty_management;
pub mod marketing;
pub mod promotions_management;
pub mod sales_commission;
pub mod service_request;
pub mod subscription;

// Projects
pub mod joint_venture;
pub mod project_billing;
pub mod project_costing;
pub mod project_resource_management;

// Shared
pub mod cost_accounting;
pub mod cost_allocation;
pub mod cross_validation;
pub mod descriptive_flexfield;
pub mod engineering_change_management;
pub mod enterprise_asset_management;
pub mod financial_reporting;
pub mod funds_reservation;
pub mod health_safety;
pub mod kpi;
pub mod lease;
pub mod risk_management;
pub mod scheduled_process;
pub mod segregation_of_duties;
pub mod sustainability;
pub mod territory_management;
pub mod transfer_pricing;

mod mock_repos;
pub use mock_repos::*;
