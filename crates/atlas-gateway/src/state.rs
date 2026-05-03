//! Application State
//! 
//! Shared state for all request handlers.

use atlas_core::{
    subscription::PostgresSubscriptionRepository,
    SubscriptionEngine,
    RevenueManagementEngine,
    revenue_management::PostgresRevenueManagementRepository,
    CashFlowForecastEngine,
    cash_flow_forecast::PostgresCashFlowForecastRepository,
    RegulatoryReportingEngine,
    regulatory_reporting::PostgresRegulatoryReportingRepository,
    AdvancePaymentEngine,
    advance_payment::PostgresAdvancePaymentRepository,
    CustomerDepositEngine,
    customer_deposit::PostgresCustomerDepositRepository,
    CashPositionEngine,
    cash_position::PostgresCashPositionRepository,
    AccountingHubEngine,
    accounting_hub::PostgresAccountingHubRepository,
    FinancialControlsEngine,
    financial_controls::PostgresFinancialControlsRepository,
    PaymentTermsEngine,
    payment_terms::PostgresPaymentTermsRepository,
    LockboxEngine,
    lockbox::PostgresLockboxRepository,
    ArAgingEngine,
    ar_aging::PostgresArAgingRepository,
    MassAdditionEngine,
    PostgresMassAdditionRepo,
    AssetReclassificationEngine,
    PostgresAssetReclassificationRepo,
    GlBudgetTransferEngine,
    PostgresGlBudgetTransferRepo,
    PaymentFormatEngine,
    PostgresPaymentFormatRepo,
    FinancialDimensionSetEngine,
    PostgresFinancialDimensionSetRepo,
    ReceiptWriteOffEngine,
    receipt_write_off::PostgresReceiptWriteOffRepository as PostgresReceiptWriteOffRepo,
    PrepaymentApplicationEngine,
    PostgresPrepaymentApplicationRepo,
    SuspenseAccountEngine,
    suspense_account::PostgresSuspenseAccountRepository,
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine,
    SecurityEngine, AuditEngine,
    NotificationEngine,
    ApprovalEngine,
    PeriodCloseEngine,
    CurrencyEngine,
    TaxEngine,
    IntercompanyEngine,
    ReconciliationEngine,
    ExpenseEngine,
    BudgetEngine,
    FixedAssetEngine,
    SubledgerAccountingEngine,
    EncumbranceEngine,
    CashManagementEngine,
    SourcingEngine,
    LeaseAccountingEngine,
    ProjectCostingEngine,
    CostAllocationEngine,
    FinancialReportingEngine,
    MultiBookAccountingEngine,
    ProcurementContractEngine,
    InventoryEngine,
    CustomerReturnsEngine,
    PricingEngine,
    SalesCommissionEngine,
    TreasuryEngine,
    GrantManagementEngine,
    SupplierQualificationEngine,
    RecurringJournalEngine,
    ManualJournalEngine,
    DescriptiveFlexfieldEngine,
    CrossValidationEngine,
    ScheduledProcessEngine,
    SegregationOfDutiesEngine,
    AllocationEngine,
    CurrencyRevaluationEngine,
    PurchaseRequisitionEngine,
    CorporateCardEngine,
    BenefitsEngine,
    PerformanceEngine,
    CreditManagementEngine,
    ProductInformationEngine,
    TransferPricingEngine,
    OrderManagementEngine,
    ApprovalDelegationEngine,
    ManufacturingEngine,
    WarehouseManagementEngine,
    AbsenceEngine,
    TimeAndLaborEngine,
    ApprovalAuthorityEngine,
    DataArchivingEngine,
    PayrollEngine,
    CompensationEngine,
    ServiceRequestEngine,
    LeadOpportunityEngine,
    DemandPlanningEngine,
    ShippingEngine,
    AutoInvoiceEngine,
    RecruitingEngine,
    RevenueEngine,
    MarketingEngine,
    ReceivingEngine,
    SupplierScorecardEngine,
    KpiEngine,
    AccountMonitorEngine,
    GoalManagementEngine,
    ContractLifecycleEngine,
    RiskManagementEngine,
    EnterpriseAssetManagementEngine,
    EngineeringChangeEngine,
    engineering_change_management::PostgresEngineeringChangeManagementRepository,
    ProductConfiguratorEngine,
    product_configurator::PostgresProductConfiguratorRepository,
    TransportationManagementEngine,
    transportation_management::PostgresTransportationManagementRepository,
    TerritoryManagementEngine,
    territory_management::PostgresTerritoryManagementRepository,
    SustainabilityEngine,
    sustainability::PostgresSustainabilityRepository,
    PromotionsManagementEngine,
    promotions_management::PostgresPromotionsManagementRepository,
    ProjectBillingEngine,
    project_billing::PostgresProjectBillingRepository,
    QualityManagementEngine,
    CostAccountingEngine,
    AccountsPayableEngine,
    accounts_payable::PostgresAccountsPayableRepository,
    SupplyChainPlanningEngine,
    supply_chain_planning::PostgresPlanningRepository,
    HealthSafetyEngine,
    health_safety::PostgresHealthSafetyRepository,
    FundsReservationEngine,
    funds_reservation::PostgresFundsReservationRepository,
    RebateManagementEngine,
    rebate_management::PostgresRebateManagementRepository,
    ProjectResourceManagementEngine,
    project_resource_management::PostgresProjectResourceManagementRepository,
    LoyaltyManagementEngine,
    loyalty_management::PostgresLoyaltyManagementRepository,
    GeneralLedgerEngine,
    AccountsReceivableEngine,
    PaymentEngine,
    NettingEngine,
    FinancialStatementEngine,
    JournalImportEngine,
    InflationAdjustmentEngine,
    inflation_adjustment::PostgresInflationAdjustmentRepository,
    ImpairmentManagementEngine,
    impairment_management::PostgresImpairmentManagementRepository,
    BankAccountTransferEngine,
    bank_account_transfer::PostgresBankAccountTransferRepository,
    TaxReportingEngine,
    tax_reporting::PostgresTaxReportingRepository,
    FinancialConsolidationEngine,
    financial_consolidation::PostgresFinancialConsolidationRepository,
    JointVentureEngine,
    joint_venture::PostgresJointVentureRepository,
    DeferredRevenueEngine,
    deferred_revenue::PostgresDeferredRevenueRepository,
    InterestInvoiceEngine,
    interest_invoice::PostgresInterestInvoiceRepository,
    ExpensePolicyComplianceEngine,
    expense_policy_compliance::PostgresExpensePolicyComplianceRepository,
    BankGuaranteeEngine,
    bank_guarantee::PostgresBankGuaranteeRepository,
    LetterOfCreditEngine,
    letter_of_credit::PostgresLetterOfCreditRepository as PostgresLetterOfCreditRepo,
    eventbus::NatsEventBus,
    schema::PostgresSchemaRepository,
    audit::PostgresAuditRepository,
    notification::PostgresNotificationRepository,
    approval::PostgresApprovalRepository,
    period_close::PostgresPeriodCloseRepository,
    currency::PostgresCurrencyRepository,
    tax::PostgresTaxRepository,
    intercompany::PostgresIntercompanyRepository,
    reconciliation::PostgresReconciliationRepository,
    expense::PostgresExpenseRepository,
    budget::PostgresBudgetRepository,
    fixed_assets::PostgresFixedAssetRepository,
    subledger_accounting::PostgresSubledgerAccountingRepository,
    encumbrance::PostgresEncumbranceRepository,
    cash_management::PostgresCashManagementRepository,
    sourcing::PostgresSourcingRepository,
    lease::PostgresLeaseAccountingRepository,
    project_costing::PostgresProjectCostingRepository,
    cost_allocation::PostgresCostAllocationRepository,
    financial_reporting::PostgresFinancialReportingRepository,
    multi_book::PostgresMultiBookAccountingRepository,
    procurement_contracts::PostgresProcurementContractRepository,
    inventory::PostgresInventoryRepository,
    customer_returns::PostgresCustomerReturnsRepository,
    pricing::PostgresPricingRepository,
    sales_commission::PostgresSalesCommissionRepository,
    treasury::PostgresTreasuryRepository,
    grant_management::PostgresGrantManagementRepository,
    supplier_qualification::PostgresSupplierQualificationRepository,
    recurring_journal::PostgresRecurringJournalRepository,
    manual_journal::PostgresManualJournalRepository,
    descriptive_flexfield::PostgresDescriptiveFlexfieldRepository,
    cross_validation::PostgresCrossValidationRepository,
    scheduled_process::PostgresScheduledProcessRepository,
    segregation_of_duties::PostgresSegregationOfDutiesRepository,
    allocation::PostgresAllocationRepository,
    currency_revaluation::PostgresCurrencyRevaluationRepository,
    purchase_requisition::PostgresPurchaseRequisitionRepository,
    corporate_card::PostgresCorporateCardRepository,
    benefits::PostgresBenefitsRepository,
    performance::PostgresPerformanceRepository,
    credit_management::PostgresCreditManagementRepository,
    product_information::PostgresProductInformationRepository,
    transfer_pricing::PostgresTransferPricingRepository,
    order_management::PostgresOrderManagementRepository,
    approval_delegation::PostgresApprovalDelegationRepository,
    manufacturing::PostgresManufacturingRepository,
    warehouse_management::PostgresWarehouseManagementRepository,
    absence::PostgresAbsenceRepository,
    time_and_labor::PostgresTimeAndLaborRepository,
    approval_authority::PostgresApprovalAuthorityRepository,
    data_archiving::PostgresDataArchivingRepository,
    payroll::PostgresPayrollRepository,
    compensation::PostgresCompensationRepository,
    service_request::PostgresServiceRequestRepository,
    lead_opportunity::PostgresLeadOpportunityRepository,
    demand_planning::PostgresDemandPlanningRepository,
    shipping::PostgresShippingRepository,
    recruiting::PostgresRecruitingRepository,
    marketing::PostgresMarketingRepository,
    receiving::PostgresReceivingRepository,
    supplier_scorecard::PostgresScorecardRepository,
    kpi::PostgresKpiRepository,
    account_monitor::PostgresAccountMonitorRepository,
    goal_management::PostgresGoalManagementRepository,
    contract_lifecycle::PostgresContractLifecycleRepository,
    risk_management::PostgresRiskManagementRepository,
    enterprise_asset_management::PostgresAssetManagementRepository,
};
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::info;

pub static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

/// Main application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub schema_engine: Arc<SchemaEngine>,
    pub workflow_engine: Arc<WorkflowEngine>,
    pub validation_engine: Arc<ValidationEngine>,
    pub formula_engine: Arc<FormulaEngine>,
    pub security_engine: Arc<SecurityEngine>,
    pub audit_engine: Arc<AuditEngine>,
    pub notification_engine: Arc<NotificationEngine>,
    pub approval_engine: Arc<ApprovalEngine>,
    pub period_close_engine: Arc<PeriodCloseEngine>,
    pub currency_engine: Arc<CurrencyEngine>,
    pub tax_engine: Arc<TaxEngine>,
    pub intercompany_engine: Arc<IntercompanyEngine>,
    pub reconciliation_engine: Arc<ReconciliationEngine>,
    pub expense_engine: Arc<ExpenseEngine>,
    pub budget_engine: Arc<BudgetEngine>,
    pub fixed_asset_engine: Arc<FixedAssetEngine>,
    pub sla_engine: Arc<SubledgerAccountingEngine>,
    pub encumbrance_engine: Arc<EncumbranceEngine>,
    pub cash_management_engine: Arc<CashManagementEngine>,
    pub sourcing_engine: Arc<SourcingEngine>,
    pub lease_accounting_engine: Arc<LeaseAccountingEngine>,
    pub project_costing_engine: Arc<ProjectCostingEngine>,
    pub cost_allocation_engine: Arc<CostAllocationEngine>,
    pub financial_reporting_engine: Arc<FinancialReportingEngine>,
    pub multi_book_engine: Arc<MultiBookAccountingEngine>,
    pub procurement_contract_engine: Arc<ProcurementContractEngine>,
    pub inventory_engine: Arc<InventoryEngine>,
    pub customer_returns_engine: Arc<CustomerReturnsEngine>,
    pub pricing_engine: Arc<PricingEngine>,
    pub sales_commission_engine: Arc<SalesCommissionEngine>,
    pub treasury_engine: Arc<TreasuryEngine>,
    pub grant_management_engine: Arc<GrantManagementEngine>,
    pub supplier_qualification_engine: Arc<SupplierQualificationEngine>,
    pub recurring_journal_engine: Arc<RecurringJournalEngine>,
    pub manual_journal_engine: Arc<ManualJournalEngine>,
    pub dff_engine: Arc<DescriptiveFlexfieldEngine>,
    pub cvr_engine: Arc<CrossValidationEngine>,
    pub scheduled_process_engine: Arc<ScheduledProcessEngine>,
    pub sod_engine: Arc<SegregationOfDutiesEngine>,
    pub allocation_engine: Arc<AllocationEngine>,
    pub currency_revaluation_engine: Arc<CurrencyRevaluationEngine>,
    pub purchase_requisition_engine: Arc<PurchaseRequisitionEngine>,
    pub corporate_card_engine: Arc<CorporateCardEngine>,
    pub benefits_engine: Arc<BenefitsEngine>,
    pub performance_engine: Arc<PerformanceEngine>,
    pub credit_management_engine: Arc<CreditManagementEngine>,
    pub product_information_engine: Arc<ProductInformationEngine>,
    pub transfer_pricing_engine: Arc<TransferPricingEngine>,
    pub order_management_engine: Arc<OrderManagementEngine>,
    pub approval_delegation_engine: Arc<ApprovalDelegationEngine>,
    pub manufacturing_engine: Arc<ManufacturingEngine>,
    pub warehouse_management_engine: Arc<WarehouseManagementEngine>,
    pub absence_engine: Arc<AbsenceEngine>,
    pub time_and_labor_engine: Arc<TimeAndLaborEngine>,
    pub approval_authority_engine: Arc<ApprovalAuthorityEngine>,
    pub data_archiving_engine: Arc<DataArchivingEngine>,
    pub payroll_engine: Arc<PayrollEngine>,
    pub compensation_engine: Arc<CompensationEngine>,
    pub service_request_engine: Arc<ServiceRequestEngine>,
    pub lead_opportunity_engine: Arc<LeadOpportunityEngine>,
    pub demand_planning_engine: Arc<DemandPlanningEngine>,
    pub shipping_engine: Arc<ShippingEngine>,
    pub autoinvoice_engine: Arc<AutoInvoiceEngine>,
    pub recruiting_engine: Arc<RecruitingEngine>,
    pub revenue_engine: Arc<RevenueEngine>,
    pub marketing_engine: Arc<MarketingEngine>,
    pub receiving_engine: Arc<ReceivingEngine>,
    pub scorecard_engine: Arc<SupplierScorecardEngine>,
    pub kpi_engine: Arc<KpiEngine>,
    pub account_monitor_engine: Arc<AccountMonitorEngine>,
    pub goal_management_engine: Arc<GoalManagementEngine>,
    pub clm_engine: Arc<ContractLifecycleEngine>,
    pub risk_management_engine: Arc<RiskManagementEngine>,
    pub eam_engine: Arc<EnterpriseAssetManagementEngine>,
    pub ecm_engine: Arc<EngineeringChangeEngine>,
    pub configurator_engine: Arc<ProductConfiguratorEngine>,
    pub transportation_engine: Arc<TransportationManagementEngine>,
    pub territory_engine: Arc<TerritoryManagementEngine>,
    pub sustainability_engine: Arc<SustainabilityEngine>,
    pub promotions_engine: Arc<PromotionsManagementEngine>,
    pub project_billing_engine: Arc<ProjectBillingEngine>,
    pub quality_engine: Arc<QualityManagementEngine>,
    pub cost_accounting_engine: Arc<CostAccountingEngine>,
    pub accounts_payable_engine: Arc<AccountsPayableEngine>,
    pub planning_engine: Arc<SupplyChainPlanningEngine>,
    pub health_safety_engine: Arc<HealthSafetyEngine>,
    pub funds_reservation_engine: Arc<FundsReservationEngine>,
    pub rebate_management_engine: Arc<RebateManagementEngine>,
    pub project_resource_engine: Arc<ProjectResourceManagementEngine>,
    pub loyalty_engine: Arc<LoyaltyManagementEngine>,
    pub general_ledger_engine: Arc<GeneralLedgerEngine>,
    pub accounts_receivable_engine: Arc<AccountsReceivableEngine>,
    pub payment_engine: Arc<PaymentEngine>,
    pub netting_engine: Arc<NettingEngine>,
    pub financial_statements_engine: Arc<FinancialStatementEngine>,
    pub journal_import_engine: Arc<JournalImportEngine>,
    pub inflation_adjustment_engine: Arc<InflationAdjustmentEngine>,
    pub impairment_management_engine: Arc<ImpairmentManagementEngine>,
    pub bank_transfer_engine: Arc<BankAccountTransferEngine>,
    pub tax_reporting_engine: Arc<TaxReportingEngine>,
    pub subscription_engine: Arc<SubscriptionEngine>,
    pub financial_consolidation_engine: Arc<FinancialConsolidationEngine>,
    pub joint_venture_engine: Arc<JointVentureEngine>,
    pub deferred_revenue_engine: Arc<DeferredRevenueEngine>,
    pub revenue_management_engine: Arc<RevenueManagementEngine>,
    pub cash_flow_forecast_engine: Arc<CashFlowForecastEngine>,
    pub regulatory_reporting_engine: Arc<RegulatoryReportingEngine>,
    pub advance_payment_engine: Arc<AdvancePaymentEngine>,
    pub customer_deposit_engine: Arc<CustomerDepositEngine>,
    pub cash_position_engine: Arc<CashPositionEngine>,
    pub accounting_hub_engine: Arc<AccountingHubEngine>,
    pub financial_controls_engine: Arc<FinancialControlsEngine>,
    pub payment_terms_engine: Arc<PaymentTermsEngine>,
    pub lockbox_engine: Arc<LockboxEngine>,
    pub ar_aging_engine: Arc<ArAgingEngine>,
    pub mass_addition_engine: Arc<MassAdditionEngine>,
    pub asset_reclassification_engine: Arc<AssetReclassificationEngine>,
    pub gl_budget_transfer_engine: Arc<GlBudgetTransferEngine>,
    pub payment_format_engine: Arc<PaymentFormatEngine>,
    pub financial_dimension_set_engine: Arc<FinancialDimensionSetEngine>,
    pub receipt_write_off_engine: Arc<ReceiptWriteOffEngine>,
    pub prepayment_application_engine: Arc<PrepaymentApplicationEngine>,
    pub suspense_account_engine: Arc<SuspenseAccountEngine>,
    pub interest_invoice_engine: Arc<InterestInvoiceEngine>,
    pub expense_policy_compliance_engine: Arc<ExpensePolicyComplianceEngine>,
    pub bank_guarantee_engine: Arc<BankGuaranteeEngine>,
    pub letter_of_credit_engine: Arc<LetterOfCreditEngine>,
    pub event_bus: Arc<NatsEventBus>,
    pub jwt_secret: String,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://atlas:atlas@localhost/atlas".to_string());
        
        // Create database pool with configurable settings
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .unwrap_or(20);
        
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .min_connections(2)
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .idle_timeout(std::time::Duration::from_secs(300))
            .connect(&database_url)
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to connect to database: {}", e)
            })?;
        
        info!("Connected to database");
        
        // Create schema engine - try PostgresRepository first
        let schema_engine = Arc::new(SchemaEngine::new(Arc::new(
            PostgresSchemaRepository::new(db_pool.clone())
        )));
        
        // Create audit engine with PostgresRepository
        let audit_engine = Arc::new(AuditEngine::new(Arc::new(
            PostgresAuditRepository::new(db_pool.clone())
        )));
        
        // Initialize other engines
        let workflow_engine = Arc::new(WorkflowEngine::new());
        let validation_engine = Arc::new(ValidationEngine::new());
        let formula_engine = Arc::new(FormulaEngine::new());
        let security_engine = Arc::new(SecurityEngine::new());
        
        // Initialize event bus (optional - gracefully handles NATS being unavailable)
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let event_bus = Arc::new(
            NatsEventBus::new(&nats_url, "atlas-gateway").await
                .unwrap_or_else(|_| NatsEventBus::noop("atlas-gateway"))
        );
        
        // Initialize notification engine
        let notification_engine = Arc::new(NotificationEngine::new(Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        )));
        
        // Initialize approval engine
        let approval_engine = Arc::new(ApprovalEngine::new(Arc::new(
            PostgresApprovalRepository::new(db_pool.clone())
        )));

        // Initialize period close engine
        let period_close_engine = Arc::new(PeriodCloseEngine::new(Arc::new(
            PostgresPeriodCloseRepository::new(db_pool.clone())
        )));

        // Initialize currency engine
        let currency_engine = Arc::new(CurrencyEngine::new(Arc::new(
            PostgresCurrencyRepository::new(db_pool.clone())
        )));

        // Initialize tax engine
        let tax_engine = Arc::new(TaxEngine::new(Arc::new(
            PostgresTaxRepository::new(db_pool.clone())
        )));

        // Initialize intercompany engine
        let intercompany_engine = Arc::new(IntercompanyEngine::new(Arc::new(
            PostgresIntercompanyRepository::new(db_pool.clone())
        )));

        // Initialize reconciliation engine
        let reconciliation_engine = Arc::new(ReconciliationEngine::new(Arc::new(
            PostgresReconciliationRepository::new(db_pool.clone())
        )));

        // Initialize expense engine
        let expense_engine = Arc::new(ExpenseEngine::new(Arc::new(
            PostgresExpenseRepository::new(db_pool.clone())
        )));

        // Initialize budget engine
        let budget_engine = Arc::new(BudgetEngine::new(Arc::new(
            PostgresBudgetRepository::new(db_pool.clone())
        )));

        // Initialize fixed asset engine
        let fixed_asset_engine = Arc::new(FixedAssetEngine::new(Arc::new(
            PostgresFixedAssetRepository::new(db_pool.clone())
        )));

        // Initialize subledger accounting engine
        let sla_engine = Arc::new(SubledgerAccountingEngine::new(Arc::new(
            PostgresSubledgerAccountingRepository::new(db_pool.clone())
        )));

        // Initialize encumbrance engine
        let encumbrance_engine = Arc::new(EncumbranceEngine::new(Arc::new(
            PostgresEncumbranceRepository::new(db_pool.clone())
        )));

        // Initialize cash management engine
        let cash_management_engine = Arc::new(CashManagementEngine::new(Arc::new(
            PostgresCashManagementRepository::new(db_pool.clone())
        )));

        // Initialize sourcing engine
        let sourcing_engine = Arc::new(SourcingEngine::new(Arc::new(
            PostgresSourcingRepository::new(db_pool.clone())
        )));

        // Initialize lease accounting engine
        let lease_accounting_engine = Arc::new(LeaseAccountingEngine::new(Arc::new(
            PostgresLeaseAccountingRepository::new(db_pool.clone())
        )));

        // Initialize project costing engine
        let project_costing_engine = Arc::new(ProjectCostingEngine::new(Arc::new(
            PostgresProjectCostingRepository::new(db_pool.clone())
        )));

        // Initialize cost allocation engine
        let cost_allocation_engine = Arc::new(CostAllocationEngine::new(Arc::new(
            PostgresCostAllocationRepository::new(db_pool.clone())
        )));

        // Initialize financial reporting engine
        let financial_reporting_engine = Arc::new(FinancialReportingEngine::new(Arc::new(
            PostgresFinancialReportingRepository::new(db_pool.clone())
        )));

        // Initialize multi-book accounting engine
        let multi_book_engine = Arc::new(MultiBookAccountingEngine::new(Arc::new(
            PostgresMultiBookAccountingRepository::new(db_pool.clone())
        )));

        // Initialize procurement contracts engine
        let procurement_contract_engine = Arc::new(ProcurementContractEngine::new(Arc::new(
            PostgresProcurementContractRepository::new(db_pool.clone())
        )));

        // Initialize inventory engine
        let inventory_engine = Arc::new(InventoryEngine::new(Arc::new(
            PostgresInventoryRepository::new(db_pool.clone())
        )));

        // Initialize customer returns engine
        let customer_returns_engine = Arc::new(CustomerReturnsEngine::new(Arc::new(
            PostgresCustomerReturnsRepository::new(db_pool.clone())
        )));

        // Initialize pricing engine
        let pricing_engine = Arc::new(PricingEngine::new(Arc::new(
            PostgresPricingRepository::new(db_pool.clone())
        )));

        // Initialize sales commission engine
        let sales_commission_engine = Arc::new(SalesCommissionEngine::new(Arc::new(
            PostgresSalesCommissionRepository::new(db_pool.clone())
        )));

        // Initialize treasury engine
        let treasury_engine = Arc::new(TreasuryEngine::new(Arc::new(
            PostgresTreasuryRepository::new(db_pool.clone())
        )));

        // Initialize grant management engine
        let grant_management_engine = Arc::new(GrantManagementEngine::new(Arc::new(
            PostgresGrantManagementRepository::new(db_pool.clone())
        )));

        // Initialize supplier qualification engine
        let supplier_qualification_engine = Arc::new(SupplierQualificationEngine::new(Arc::new(
            PostgresSupplierQualificationRepository::new(db_pool.clone())
        )));

        // Initialize recurring journal engine
        let recurring_journal_engine = Arc::new(RecurringJournalEngine::new(Arc::new(
            PostgresRecurringJournalRepository::new(db_pool.clone())
        )));

        // Initialize manual journal engine
        let manual_journal_engine = Arc::new(ManualJournalEngine::new(Arc::new(
            PostgresManualJournalRepository::new(db_pool.clone())
        )));

        // Initialize descriptive flexfield engine
        let dff_engine = Arc::new(DescriptiveFlexfieldEngine::new(Arc::new(
            PostgresDescriptiveFlexfieldRepository::new(db_pool.clone())
        )));

        // Initialize cross-validation rule engine
        let cvr_engine = Arc::new(CrossValidationEngine::new(Arc::new(
            PostgresCrossValidationRepository::new(db_pool.clone())
        )));

        // Initialize scheduled process engine
        let scheduled_process_engine = Arc::new(ScheduledProcessEngine::new(Arc::new(
            PostgresScheduledProcessRepository::new(db_pool.clone())
        )));

        // Initialize segregation of duties engine
        let sod_engine = Arc::new(SegregationOfDutiesEngine::new(Arc::new(
            PostgresSegregationOfDutiesRepository::new(db_pool.clone())
        )));

        // Initialize GL allocation engine
        let allocation_engine = Arc::new(AllocationEngine::new(Arc::new(
            PostgresAllocationRepository::new(db_pool.clone())
        )));

        // Initialize currency revaluation engine
        let currency_revaluation_engine = Arc::new(CurrencyRevaluationEngine::new(Arc::new(
            PostgresCurrencyRevaluationRepository::new(db_pool.clone())
        )));

        // Initialize purchase requisition engine
        let purchase_requisition_engine = Arc::new(PurchaseRequisitionEngine::new(Arc::new(
            PostgresPurchaseRequisitionRepository::new(db_pool.clone())
        )));

        // Initialize corporate card engine
        let corporate_card_engine = Arc::new(CorporateCardEngine::new(Arc::new(
            PostgresCorporateCardRepository::new(db_pool.clone())
        )));

        // Initialize benefits administration engine
        let benefits_engine = Arc::new(BenefitsEngine::new(Arc::new(
            PostgresBenefitsRepository::new(db_pool.clone())
        )));

        // Initialize performance management engine
        let performance_engine = Arc::new(PerformanceEngine::new(Arc::new(
            PostgresPerformanceRepository::new(db_pool.clone())
        )));

        // Initialize credit management engine
        let credit_management_engine = Arc::new(CreditManagementEngine::new(Arc::new(
            PostgresCreditManagementRepository::new(db_pool.clone())
        )));

        // Initialize product information management engine
        let product_information_engine = Arc::new(ProductInformationEngine::new(Arc::new(
            PostgresProductInformationRepository::new(db_pool.clone())
        )));

        // Initialize transfer pricing engine
        let transfer_pricing_engine = Arc::new(TransferPricingEngine::new(Arc::new(
            PostgresTransferPricingRepository::new(db_pool.clone())
        )));

        // Initialize order management engine
        let order_management_engine = Arc::new(OrderManagementEngine::new(Arc::new(
            PostgresOrderManagementRepository::new(db_pool.clone())
        )));

        // Initialize approval delegation engine
        let approval_delegation_engine = Arc::new(ApprovalDelegationEngine::new(Arc::new(
            PostgresApprovalDelegationRepository::new(db_pool.clone())
        )));

        // Initialize manufacturing execution engine
        let manufacturing_engine = Arc::new(ManufacturingEngine::new(Arc::new(
            PostgresManufacturingRepository::new(db_pool.clone())
        )));

        let warehouse_management_engine = Arc::new(WarehouseManagementEngine::new(Arc::new(
            PostgresWarehouseManagementRepository::new(db_pool.clone())
        )));

        let absence_engine = Arc::new(AbsenceEngine::new(Arc::new(
            PostgresAbsenceRepository::new(db_pool.clone())
        )));

        let time_and_labor_engine = Arc::new(TimeAndLaborEngine::new(Arc::new(
            PostgresTimeAndLaborRepository::new(db_pool.clone())
        )));

        // Initialize approval authority engine
        let approval_authority_engine = Arc::new(ApprovalAuthorityEngine::new(Arc::new(
            PostgresApprovalAuthorityRepository::new(db_pool.clone())
        )));

        // Initialize data archiving engine
        let data_archiving_engine = Arc::new(DataArchivingEngine::new(Arc::new(
            PostgresDataArchivingRepository::new(db_pool.clone())
        )));

        // Initialize payroll engine
        let payroll_engine = Arc::new(PayrollEngine::new(Arc::new(
            PostgresPayrollRepository::new(db_pool.clone())
        )));

        // Initialize compensation engine
        let compensation_engine = Arc::new(CompensationEngine::new(Arc::new(
            PostgresCompensationRepository::new(db_pool.clone())
        )));

        // Initialize service request engine
        let service_request_engine = Arc::new(ServiceRequestEngine::new(Arc::new(
            PostgresServiceRequestRepository::new(db_pool.clone())
        )));

        // Initialize lead and opportunity engine
        let lead_opportunity_engine = Arc::new(LeadOpportunityEngine::new(Arc::new(
            PostgresLeadOpportunityRepository::new(db_pool.clone())
        )));

        // Initialize demand planning engine
        let demand_planning_engine = Arc::new(DemandPlanningEngine::new(Arc::new(
            PostgresDemandPlanningRepository::new(db_pool.clone())
        )));

        // Initialize shipping execution engine
        let shipping_engine = Arc::new(ShippingEngine::new(Arc::new(
            PostgresShippingRepository::new(db_pool.clone())
        )));

        // Initialize AutoInvoice engine
        let autoinvoice_engine = Arc::new(AutoInvoiceEngine::new(Arc::new(
            atlas_core::autoinvoice::PostgresAutoInvoiceRepository::new(db_pool.clone())
        )));

        // Initialize Recruiting engine
        let recruiting_engine = Arc::new(RecruitingEngine::new(Arc::new(
            PostgresRecruitingRepository::new(db_pool.clone())
        )));

        // Initialize Revenue Recognition engine
        let revenue_engine = Arc::new(RevenueEngine::new(Arc::new(
            atlas_core::revenue::PostgresRevenueRepository::new(db_pool.clone())
        )));

        // Initialize Marketing Campaign engine
        let marketing_engine = Arc::new(MarketingEngine::new(Arc::new(
            PostgresMarketingRepository::new(db_pool.clone())
        )));

        // Initialize Receiving engine
        let receiving_engine = Arc::new(ReceivingEngine::new(Arc::new(
            PostgresReceivingRepository::new(db_pool.clone())
        )));

        // Initialize Supplier Scorecard engine
        let scorecard_engine = Arc::new(SupplierScorecardEngine::new(Arc::new(
            PostgresScorecardRepository::new(db_pool.clone())
        )));

        // Initialize KPI & Analytics engine
        let kpi_engine = Arc::new(KpiEngine::new(Arc::new(
            PostgresKpiRepository::new(db_pool.clone())
        )));

        // Initialize Account Monitor engine
        let account_monitor_engine = Arc::new(AccountMonitorEngine::new(Arc::new(
            PostgresAccountMonitorRepository::new(db_pool.clone())
        )));

        // Initialize Goal Management engine
        let goal_management_engine = Arc::new(GoalManagementEngine::new(Arc::new(
            PostgresGoalManagementRepository::new(db_pool.clone())
        )));

        // Initialize Contract Lifecycle Management engine
        let clm_engine = Arc::new(ContractLifecycleEngine::new(Arc::new(
            PostgresContractLifecycleRepository::new(db_pool.clone())
        )));

        // Initialize Risk Management engine
        let risk_management_engine = Arc::new(RiskManagementEngine::new(Arc::new(
            PostgresRiskManagementRepository::new(db_pool.clone())
        )));

        // Initialize Enterprise Asset Management engine
        let eam_engine = Arc::new(EnterpriseAssetManagementEngine::new(Arc::new(
            PostgresAssetManagementRepository::new(db_pool.clone())
        )));

        // Initialize Engineering Change Management engine
        let ecm_engine = Arc::new(EngineeringChangeEngine::new(Arc::new(
            PostgresEngineeringChangeManagementRepository::new(db_pool.clone())
        )));

        // Initialize Product Configurator engine
        let configurator_engine = Arc::new(ProductConfiguratorEngine::new(Arc::new(
            PostgresProductConfiguratorRepository::new(db_pool.clone())
        )));

        // Initialize Transportation Management engine
        let transportation_engine = Arc::new(TransportationManagementEngine::new(Arc::new(
            PostgresTransportationManagementRepository::new(db_pool.clone())
        )));

        // Initialize Territory Management engine
        let territory_engine = Arc::new(TerritoryManagementEngine::new(Arc::new(
            PostgresTerritoryManagementRepository::new(db_pool.clone())
        )));

        // Initialize Sustainability & ESG engine
        let sustainability_engine = Arc::new(SustainabilityEngine::new(Arc::new(
            PostgresSustainabilityRepository::new(db_pool.clone())
        )));

        // Initialize Promotions Management engine
        let promotions_engine = Arc::new(PromotionsManagementEngine::new(Arc::new(
            PostgresPromotionsManagementRepository::new(db_pool.clone())
        )));

        // Initialize Project Billing engine
        let project_billing_engine = Arc::new(ProjectBillingEngine::new(Arc::new(
            PostgresProjectBillingRepository::new(db_pool.clone())
        )));

        // Initialize Quality Management engine
        let quality_engine = Arc::new(QualityManagementEngine::new(Arc::new(
            atlas_core::quality_management::PostgresQualityManagementRepository::new(db_pool.clone())
        )));

        // Initialize Cost Accounting engine
        let cost_accounting_engine = Arc::new(CostAccountingEngine::new(Arc::new(
            atlas_core::cost_accounting::PostgresCostAccountingRepository::new(db_pool.clone())
        )));

        // Initialize Accounts Payable engine
        let accounts_payable_engine = Arc::new(AccountsPayableEngine::new(Arc::new(
            PostgresAccountsPayableRepository::new(db_pool.clone())
        )));

        // Initialize Supply Chain Planning engine
        let planning_engine = Arc::new(SupplyChainPlanningEngine::new(Arc::new(
            PostgresPlanningRepository::new(db_pool.clone())
        )));

        // Initialize Workplace Health & Safety engine
        let health_safety_engine = Arc::new(HealthSafetyEngine::new(Arc::new(
            PostgresHealthSafetyRepository::new(db_pool.clone())
        )));

        // Initialize Funds Reservation & Budgetary Control engine
        let funds_reservation_engine = Arc::new(FundsReservationEngine::new(Arc::new(
            PostgresFundsReservationRepository::new(db_pool.clone())
        )));

        // Initialize Rebate Management engine
        let rebate_management_engine = Arc::new(RebateManagementEngine::new(Arc::new(
            PostgresRebateManagementRepository::new(db_pool.clone())
        )));

        // Initialize Project Resource Management engine
        let project_resource_engine = Arc::new(ProjectResourceManagementEngine::new(Arc::new(
            PostgresProjectResourceManagementRepository::new(db_pool.clone())
        )));

        // Initialize Loyalty Management engine
        let loyalty_engine = Arc::new(LoyaltyManagementEngine::new(Arc::new(
            PostgresLoyaltyManagementRepository::new(db_pool.clone())
        )));

        // Initialize General Ledger engine
        let general_ledger_engine = Arc::new(GeneralLedgerEngine::new(Arc::new(
            atlas_core::general_ledger::PostgresGeneralLedgerRepository::new(db_pool.clone())
        )));

        // Initialize Accounts Receivable engine
        let accounts_receivable_engine = Arc::new(AccountsReceivableEngine::new(Arc::new(
            atlas_core::accounts_receivable::PostgresAccountsReceivableRepository::new(db_pool.clone())
        )));

        // Initialize Payment engine
        let payment_engine = Arc::new(PaymentEngine::new(Arc::new(
            atlas_core::payment::PostgresPaymentRepository::new(db_pool.clone())
        )));

        // Initialize Netting engine
        let netting_engine = Arc::new(NettingEngine::new(Arc::new(
            atlas_core::netting::PostgresNettingRepository::new(db_pool.clone())
        )));

        // Initialize Financial Statements engine
        let financial_statements_engine = Arc::new(FinancialStatementEngine::new(Arc::new(
            atlas_core::financial_statements::PostgresFinancialStatementRepository::new(db_pool.clone())
        )));

        // Initialize Journal Import engine
        let journal_import_engine = Arc::new(JournalImportEngine::new(Arc::new(
            atlas_core::journal_import::PostgresJournalImportRepository::new(db_pool.clone())
        )));

        // Initialize Inflation Adjustment engine
        let inflation_adjustment_engine = Arc::new(InflationAdjustmentEngine::new(Arc::new(
            PostgresInflationAdjustmentRepository::new(db_pool.clone())
        )));

        // Initialize Impairment Management engine
        let impairment_management_engine = Arc::new(ImpairmentManagementEngine::new(Arc::new(
            PostgresImpairmentManagementRepository::new(db_pool.clone())
        )));

        // Initialize Bank Account Transfer engine
        let bank_transfer_engine = Arc::new(BankAccountTransferEngine::new(Arc::new(
            PostgresBankAccountTransferRepository::new(db_pool.clone())
        )));

        // Initialize Tax Reporting engine
        let tax_reporting_engine = Arc::new(TaxReportingEngine::new(Arc::new(
            PostgresTaxReportingRepository::new(db_pool.clone())
        )));

        // Initialize Subscription Management engine
        let subscription_engine = Arc::new(SubscriptionEngine::new(Arc::new(
            PostgresSubscriptionRepository::new(db_pool.clone())
        )));

        // Initialize Financial Consolidation engine
        let financial_consolidation_engine = Arc::new(FinancialConsolidationEngine::new(Arc::new(
            PostgresFinancialConsolidationRepository::new(db_pool.clone())
        )));

        // Initialize Joint Venture Management engine
        let joint_venture_engine = Arc::new(JointVentureEngine::new(Arc::new(
            PostgresJointVentureRepository::new(db_pool.clone())
        )));

        // Initialize Deferred Revenue/Cost Management engine
        let deferred_revenue_engine = Arc::new(DeferredRevenueEngine::new(Arc::new(
            PostgresDeferredRevenueRepository::new(db_pool.clone())
        )));

        // Initialize Revenue Management (ASC 606) engine
        let revenue_management_engine = Arc::new(RevenueManagementEngine::new(Arc::new(
            PostgresRevenueManagementRepository::new(db_pool.clone())
        )));

        // Initialize Cash Flow Forecasting engine
        let cash_flow_forecast_engine = Arc::new(CashFlowForecastEngine::new(Arc::new(
            PostgresCashFlowForecastRepository::new(db_pool.clone())
        )));

        // Initialize Regulatory Reporting engine
        let regulatory_reporting_engine = Arc::new(RegulatoryReportingEngine::new(Arc::new(
            PostgresRegulatoryReportingRepository::new(db_pool.clone())
        )));

        // Initialize Advance Payment engine
        let advance_payment_engine = Arc::new(AdvancePaymentEngine::new(Arc::new(
            PostgresAdvancePaymentRepository::new(db_pool.clone())
        )));

        // Initialize Customer Deposit engine
        let customer_deposit_engine = Arc::new(CustomerDepositEngine::new(Arc::new(
            PostgresCustomerDepositRepository::new(db_pool.clone())
        )));

        // Initialize Cash Position engine
        let cash_position_engine = Arc::new(CashPositionEngine::new(Arc::new(
            PostgresCashPositionRepository::new(db_pool.clone())
        )));

        // Initialize Accounting Hub engine
        let accounting_hub_engine = Arc::new(AccountingHubEngine::new(Arc::new(
            PostgresAccountingHubRepository::new(db_pool.clone())
        )));

        // Initialize Financial Controls engine
        let financial_controls_engine = Arc::new(FinancialControlsEngine::new(Arc::new(
            PostgresFinancialControlsRepository::new(db_pool.clone())
        )));

        // Initialize Payment Terms engine
        let payment_terms_engine = Arc::new(PaymentTermsEngine::new(Arc::new(
            PostgresPaymentTermsRepository::new(db_pool.clone())
        )));

        // Initialize Lockbox Processing engine
        let lockbox_engine = Arc::new(LockboxEngine::new(Arc::new(
            PostgresLockboxRepository::new(db_pool.clone())
        )));

        // Initialize AR Aging Analysis engine
        let ar_aging_engine = Arc::new(ArAgingEngine::new(Arc::new(
            PostgresArAgingRepository::new(db_pool.clone())
        )));

        // Initialize Mass Additions engine (Oracle Fusion: Fixed Assets > Mass Additions)
        let mass_addition_engine = Arc::new(MassAdditionEngine::new(Arc::new(
            PostgresMassAdditionRepo::new(db_pool.clone())
        )));

        // Initialize Asset Reclassification engine (Oracle Fusion: Fixed Assets > Reclassification)
        let asset_reclassification_engine = Arc::new(AssetReclassificationEngine::new(Arc::new(
            PostgresAssetReclassificationRepo::new(db_pool.clone())
        )));

        // Initialize GL Budget Transfer engine (Oracle Fusion: GL > Budget Transfers)
        let gl_budget_transfer_engine = Arc::new(GlBudgetTransferEngine::new(Arc::new(
            PostgresGlBudgetTransferRepo::new(db_pool.clone())
        )));

        // Initialize Payment Format engine (Oracle Fusion: Payables > Payment Formats)
        let payment_format_engine = Arc::new(PaymentFormatEngine::new(Arc::new(
            PostgresPaymentFormatRepo::new(db_pool.clone())
        )));

        // Initialize Financial Dimension Set engine (Oracle Fusion: GL > Dimension Sets)
        let financial_dimension_set_engine = Arc::new(FinancialDimensionSetEngine::new(Arc::new(
            PostgresFinancialDimensionSetRepo::new(db_pool.clone())
        )));

        // Initialize Receipt Write-Off engine (Oracle Fusion: Receivables > Write-Offs)
        let receipt_write_off_engine = Arc::new(ReceiptWriteOffEngine::new(Arc::new(
            PostgresReceiptWriteOffRepo::new(db_pool.clone())
        )));

        // Initialize Prepayment Application engine (Oracle Fusion: Payables > Prepayments)
        let prepayment_application_engine = Arc::new(PrepaymentApplicationEngine::new(Arc::new(
            PostgresPrepaymentApplicationRepo::new(db_pool.clone())
        )));

        // Initialize Suspense Account Processing engine (Oracle Fusion: GL > Suspense Accounts)
        let suspense_account_engine = Arc::new(SuspenseAccountEngine::new(Arc::new(
            PostgresSuspenseAccountRepository::new(db_pool.clone())
        )));

        // Initialize Interest Invoice Management engine (Oracle Fusion: Receivables > Late Charges)
        let interest_invoice_engine = Arc::new(InterestInvoiceEngine::new(Arc::new(
            PostgresInterestInvoiceRepository::new(db_pool.clone())
        )));

        // Initialize Expense Policy Compliance engine (Oracle Fusion: Expenses > Policies > Compliance)
        let expense_policy_compliance_engine = Arc::new(ExpensePolicyComplianceEngine::new(Arc::new(
            PostgresExpensePolicyComplianceRepository::new(db_pool.clone())
        )));

        // Initialize Bank Guarantee Management engine (Oracle Fusion: Treasury > Bank Guarantees)
        let bank_guarantee_engine = Arc::new(BankGuaranteeEngine::new(Arc::new(
            PostgresBankGuaranteeRepository::new(db_pool.clone())
        )));

        // Initialize Letter of Credit Management engine (Oracle Fusion: Treasury > Trade Finance > LCs)
        let letter_of_credit_engine = Arc::new(LetterOfCreditEngine::new(Arc::new(
            PostgresLetterOfCreditRepo::new(db_pool.clone())
        )));

        // Load JWT secret from environment
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using development default");
                "dev-secret-key-please-change-in-production-1234567890".to_string()
            });
        
        // Validate JWT secret strength
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters long");
        }
        
        // Warn if using weak secrets
        if jwt_secret.contains("secret") || jwt_secret.contains("password") || jwt_secret.contains("change") || jwt_secret.contains("please-change") {
            tracing::warn!("JWT_SECRET appears to be a weak or placeholder value. Use a cryptographically random secret in production.");
            if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
                anyhow::bail!("JWT_SECRET appears to be a placeholder — refusing to start in production mode");
            }
        }
        
        // Load entity definitions from schema engine
        if let Err(e) = schema_engine.load_all().await {
            tracing::warn!("Failed to load entities from database: {} (tables may not be created yet)", e);
        }
        
        // Load workflows from entity definitions
        let entity_names = schema_engine.entity_names();
        for name in &entity_names {
            if let Some(entity) = schema_engine.get_entity(name) {
                if let Some(workflow) = &entity.workflow {
                    if let Err(e) = workflow_engine.load_workflow(workflow.clone()).await {
                        tracing::warn!("Failed to load workflow for {}: {}", name, e);
                    }
                }
            }
        }
        
        info!("Initialized {} entities and their workflows", entity_names.len());
        
        let state = Self {
            db_pool,
            schema_engine,
            workflow_engine,
            validation_engine,
            formula_engine,
            security_engine,
            audit_engine,
            notification_engine,
            approval_engine,
            period_close_engine,
            currency_engine,
            tax_engine,
            intercompany_engine,
            reconciliation_engine,
            expense_engine,
            budget_engine,
            fixed_asset_engine,
            sla_engine,
            encumbrance_engine,
            cash_management_engine,
            sourcing_engine,
            lease_accounting_engine,
            project_costing_engine,
            cost_allocation_engine,
            financial_reporting_engine,
            multi_book_engine,
            procurement_contract_engine,
            inventory_engine,
            customer_returns_engine,
            pricing_engine,
            sales_commission_engine,
            treasury_engine,
            grant_management_engine,
            supplier_qualification_engine,
            recurring_journal_engine,
            manual_journal_engine,
            dff_engine,
            cvr_engine,
            scheduled_process_engine,
            sod_engine,
            allocation_engine,
            currency_revaluation_engine,
            purchase_requisition_engine,
            corporate_card_engine,
            benefits_engine,
            performance_engine,
            credit_management_engine,
            product_information_engine,
            transfer_pricing_engine,
            order_management_engine,
            approval_delegation_engine,
            manufacturing_engine,
            warehouse_management_engine,
            absence_engine,
            time_and_labor_engine,
            approval_authority_engine,
            data_archiving_engine,
            payroll_engine,
            compensation_engine,
            service_request_engine,
            lead_opportunity_engine,
            demand_planning_engine,
            shipping_engine,
            autoinvoice_engine,
            recruiting_engine,
            revenue_engine,
            marketing_engine,
            receiving_engine,
            scorecard_engine,
            kpi_engine,
            account_monitor_engine,
            goal_management_engine,
            clm_engine,
            risk_management_engine,
            eam_engine,
            ecm_engine,
            configurator_engine,
            transportation_engine,
            territory_engine,
            sustainability_engine,
            promotions_engine,
            project_billing_engine,
            quality_engine,
            cost_accounting_engine,
            accounts_payable_engine,
            planning_engine,
            health_safety_engine,
            funds_reservation_engine,
            rebate_management_engine,
            project_resource_engine,
            loyalty_engine,
            general_ledger_engine,
            accounts_receivable_engine,
            payment_engine,
            netting_engine,
            financial_statements_engine,
            journal_import_engine,
            inflation_adjustment_engine,
            impairment_management_engine,
            bank_transfer_engine,
            tax_reporting_engine,
            subscription_engine,
            financial_consolidation_engine,
            joint_venture_engine,
            deferred_revenue_engine,
            revenue_management_engine,
            cash_flow_forecast_engine,
            regulatory_reporting_engine,
            advance_payment_engine,
            customer_deposit_engine,
            cash_position_engine,
            accounting_hub_engine,
            financial_controls_engine,
            payment_terms_engine,
            lockbox_engine,
            ar_aging_engine,
            mass_addition_engine,
            asset_reclassification_engine,
            gl_budget_transfer_engine,
            payment_format_engine,
            financial_dimension_set_engine,
            receipt_write_off_engine,
            prepayment_application_engine,
            suspense_account_engine,
            interest_invoice_engine,
            expense_policy_compliance_engine,
            bank_guarantee_engine,
            letter_of_credit_engine,
            event_bus,
            jwt_secret,
        };
        
        Ok(state)
    }
}
