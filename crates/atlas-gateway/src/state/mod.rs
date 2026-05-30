//! Application State
//! 
//! Shared state for all request handlers.

mod core;
mod financials;
mod hcm;
mod scm;
mod crm;
mod projects;
mod shared_domain;

pub use self::core::CoreState;
pub use self::financials::FinancialsState;
pub use self::hcm::HcmState;
pub use self::scm::ScmState;
pub use self::crm::CrmState;
pub use self::projects::ProjectsState;
pub use self::shared_domain::SharedState;

use atlas_core::{
    eventbus::NatsEventBus,
    schema::PostgresSchemaRepository,
    audit::PostgresAuditRepository,
    PostgresNotificationRepo,
    PostgresApprovalRepo,
    financials::*,
    hcm::*,
    scm::*,
    crm::*,
    projects::*,
    shared::*,
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine,
    SecurityEngine, AuditEngine,
    NotificationEngine,
    ApprovalEngine,
    DocumentSequencingEngine,
    TransactionCalendarEngine,
};
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::info;

pub static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

/// Main application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub core: CoreState,
    pub financials: FinancialsState,
    pub hcm: HcmState,
    pub scm: ScmState,
    pub crm: CrmState,
    pub projects: ProjectsState,
    pub shared: SharedState,
    pub event_bus: Arc<NatsEventBus>,
    pub jwt_secret: String,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://atlas:atlas@localhost/atlas".to_string());
        
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .unwrap_or(20);
        
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .min_connections(2)
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .idle_timeout(std::time::Duration::from_mins(5))
            .connect(&database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {e}"))?;
        
        info!("Connected to database");
        
        let schema_engine = Arc::new(SchemaEngine::new(Arc::new(PostgresSchemaRepository::new(db_pool.clone()))));
        let audit_engine = Arc::new(AuditEngine::new(Arc::new(PostgresAuditRepository::new(db_pool.clone()))));
        let workflow_engine = Arc::new(WorkflowEngine::new());
        let validation_engine = Arc::new(ValidationEngine::new());
        let formula_engine = Arc::new(FormulaEngine::new());
        let security_engine = Arc::new(SecurityEngine::new());
        let notification_engine = Arc::new(NotificationEngine::new(Arc::new(PostgresNotificationRepo::new(db_pool.clone()))));
        let approval_engine = Arc::new(ApprovalEngine::new(Arc::new(PostgresApprovalRepo::new(db_pool.clone()))));
        let document_sequencing_engine = Arc::new(DocumentSequencingEngine::new(Arc::new(PostgresDocumentSequencingRepo::new(db_pool.clone()))));
        let transaction_calendar_engine = Arc::new(TransactionCalendarEngine::new(Arc::new(PostgresTransactionCalendarRepo::new(db_pool.clone()))));

        let core = CoreState {
            schema_engine: schema_engine.clone(),
            workflow_engine: workflow_engine.clone(),
            validation_engine,
            formula_engine,
            security_engine,
            audit_engine,
            notification_engine,
            approval_engine,
            document_sequencing_engine,
            transaction_calendar_engine,
        };

        let financials = FinancialsState {
            period_close_engine: Arc::new(PeriodCloseEngine::new(Arc::new(PostgresPeriodCloseRepo::new(db_pool.clone())))),
            currency_engine: Arc::new(CurrencyEngine::new(Arc::new(PostgresCurrencyRepo::new(db_pool.clone())))),
            tax_engine: Arc::new(TaxEngine::new(Arc::new(PostgresTaxRepo::new(db_pool.clone())))),
            intercompany_engine: Arc::new(IntercompanyEngine::new(Arc::new(PostgresIntercompanyRepo::new(db_pool.clone())))),
            reconciliation_engine: Arc::new(ReconciliationEngine::new(Arc::new(PostgresReconciliationRepo::new(db_pool.clone())))),
            expense_engine: Arc::new(ExpenseEngine::new(Arc::new(PostgresExpenseRepo::new(db_pool.clone())))),
            budget_engine: Arc::new(BudgetEngine::new(Arc::new(PostgresBudgetRepo::new(db_pool.clone())))),
            fixed_asset_engine: Arc::new(FixedAssetEngine::new(Arc::new(PostgresFixedAssetRepo::new(db_pool.clone())))),
            sla_engine: Arc::new(SubledgerAccountingEngine::new(Arc::new(PostgresSubledgerAccountingRepo::new(db_pool.clone())))),
            encumbrance_engine: Arc::new(EncumbranceEngine::new(Arc::new(PostgresEncumbranceRepo::new(db_pool.clone())))),
            cash_management_engine: Arc::new(CashManagementEngine::new(Arc::new(PostgresCashManagementRepo::new(db_pool.clone())))),
            multi_book_engine: Arc::new(MultiBookAccountingEngine::new(Arc::new(PostgresMultiBookAccountingRepo::new(db_pool.clone())))),
            recurring_journal_engine: Arc::new(RecurringJournalEngine::new(Arc::new(PostgresRecurringJournalRepo::new(db_pool.clone())))),
            manual_journal_engine: Arc::new(ManualJournalEngine::new(Arc::new(PostgresManualJournalRepo::new(db_pool.clone())))),
            allocation_engine: Arc::new(AllocationEngine::new(Arc::new(PostgresAllocationRepo::new(db_pool.clone())))),
            currency_revaluation_engine: Arc::new(CurrencyRevaluationEngine::new(Arc::new(PostgresCurrencyRevaluationRepo::new(db_pool.clone())))),
            corporate_card_engine: Arc::new(CorporateCardEngine::new(Arc::new(PostgresCorporateCardRepo::new(db_pool.clone())))),
            credit_management_engine: Arc::new(CreditManagementEngine::new(Arc::new(PostgresCreditManagementRepo::new(db_pool.clone())))),
            autoinvoice_engine: Arc::new(AutoInvoiceEngine::new(Arc::new(PostgresAutoInvoiceRepo::new(db_pool.clone())))),
            revenue_engine: Arc::new(RevenueEngine::new(Arc::new(PostgresRevenueRepo::new(db_pool.clone())))),
            account_monitor_engine: Arc::new(AccountMonitorEngine::new(Arc::new(PostgresAccountMonitorRepo::new(db_pool.clone())))),
            accounts_payable_engine: Arc::new(AccountsPayableEngine::new(Arc::new(PostgresAccountsPayableRepo::new(db_pool.clone())))),
            general_ledger_engine: Arc::new(GeneralLedgerEngine::new(Arc::new(PostgresGeneralLedgerRepo::new(db_pool.clone())))),
            accounts_receivable_engine: Arc::new(AccountsReceivableEngine::new(Arc::new(PostgresAccountsReceivableRepo::new(db_pool.clone())))),
            payment_engine: Arc::new(PaymentEngine::new(Arc::new(PostgresPaymentRepo::new(db_pool.clone())))),
            netting_engine: Arc::new(NettingEngine::new(Arc::new(PostgresNettingRepo::new(db_pool.clone())))),
            financial_statements_engine: Arc::new(FinancialStatementEngine::new(Arc::new(PostgresFinancialStatementRepo::new(db_pool.clone())))),
            journal_import_engine: Arc::new(JournalImportEngine::new(Arc::new(PostgresJournalImportRepo::new(db_pool.clone())))),
            inflation_adjustment_engine: Arc::new(InflationAdjustmentEngine::new(Arc::new(PostgresInflationAdjustmentRepo::new(db_pool.clone())))),
            impairment_management_engine: Arc::new(ImpairmentManagementEngine::new(Arc::new(PostgresImpairmentManagementRepo::new(db_pool.clone())))),
            bank_transfer_engine: Arc::new(BankAccountTransferEngine::new(Arc::new(PostgresBankAccountTransferRepo::new(db_pool.clone())))),
            tax_reporting_engine: Arc::new(TaxReportingEngine::new(Arc::new(PostgresTaxReportingRepo::new(db_pool.clone())))),
            financial_consolidation_engine: Arc::new(FinancialConsolidationEngine::new(Arc::new(PostgresFinancialConsolidationRepo::new(db_pool.clone())))),
            deferred_revenue_engine: Arc::new(DeferredRevenueEngine::new(Arc::new(PostgresDeferredRevenueRepo::new(db_pool.clone())))),
            revenue_management_engine: Arc::new(RevenueManagementEngine::new(Arc::new(PostgresRevenueManagementRepo::new(db_pool.clone())))),
            cash_flow_forecast_engine: Arc::new(CashFlowForecastEngine::new(Arc::new(PostgresCashFlowForecastRepo::new(db_pool.clone())))),
            regulatory_reporting_engine: Arc::new(RegulatoryReportingEngine::new(Arc::new(PostgresRegulatoryReportingRepo::new(db_pool.clone())))),
            advance_payment_engine: Arc::new(AdvancePaymentEngine::new(Arc::new(PostgresAdvancePaymentRepo::new(db_pool.clone())))),
            customer_deposit_engine: Arc::new(CustomerDepositEngine::new(Arc::new(PostgresCustomerDepositRepo::new(db_pool.clone())))),
            cash_position_engine: Arc::new(CashPositionEngine::new(Arc::new(PostgresCashPositionRepo::new(db_pool.clone())))),
            accounting_hub_engine: Arc::new(AccountingHubEngine::new(Arc::new(PostgresAccountingHubRepo::new(db_pool.clone())))),
            financial_controls_engine: Arc::new(FinancialControlsEngine::new(Arc::new(PostgresFinancialControlsRepo::new(db_pool.clone())))),
            payment_terms_engine: Arc::new(PaymentTermsEngine::new(Arc::new(PostgresPaymentTermsRepo::new(db_pool.clone())))),
            lockbox_engine: Arc::new(LockboxEngine::new(Arc::new(PostgresLockboxRepo::new(db_pool.clone())))),
            ar_aging_engine: Arc::new(ArAgingEngine::new(Arc::new(PostgresArAgingRepo::new(db_pool.clone())))),
            mass_addition_engine: Arc::new(MassAdditionEngine::new(Arc::new(PostgresMassAdditionRepo::new(db_pool.clone())))),
            asset_reclassification_engine: Arc::new(AssetReclassificationEngine::new(Arc::new(PostgresAssetReclassificationRepo::new(db_pool.clone())))),
            gl_budget_transfer_engine: Arc::new(GlBudgetTransferEngine::new(Arc::new(PostgresGlBudgetTransferRepo::new(db_pool.clone())))),
            payment_format_engine: Arc::new(PaymentFormatEngine::new(Arc::new(PostgresPaymentFormatRepo::new(db_pool.clone())))),
            financial_dimension_set_engine: Arc::new(FinancialDimensionSetEngine::new(Arc::new(PostgresFinancialDimensionSetRepo::new(db_pool.clone())))),
            receipt_write_off_engine: Arc::new(ReceiptWriteOffEngine::new(Arc::new(PostgresReceiptWriteOffRepo::new(db_pool.clone())))),
            prepayment_application_engine: Arc::new(PrepaymentApplicationEngine::new(Arc::new(PostgresPrepaymentApplicationRepo::new(db_pool.clone())))),
            suspense_account_engine: Arc::new(SuspenseAccountEngine::new(Arc::new(PostgresSuspenseAccountRepo::new(db_pool.clone())))),
            interest_invoice_engine: Arc::new(InterestInvoiceEngine::new(Arc::new(PostgresInterestInvoiceRepo::new(db_pool.clone())))),
            expense_policy_compliance_engine: Arc::new(ExpensePolicyComplianceEngine::new(Arc::new(PostgresExpensePolicyComplianceRepo::new(db_pool.clone())))),
            bank_guarantee_engine: Arc::new(BankGuaranteeEngine::new(Arc::new(PostgresBankGuaranteeRepo::new(db_pool.clone())))),
            letter_of_credit_engine: Arc::new(LetterOfCreditEngine::new(Arc::new(PostgresLetterOfCreditRepo::new(db_pool.clone())))),
            hedge_management_engine: Arc::new(HedgeManagementEngine::new(Arc::new(PostgresHedgeManagementRepo::new(db_pool.clone())))),
            payment_risk_engine: Arc::new(PaymentRiskEngine::new(Arc::new(PostgresPaymentRiskRepo::new(db_pool.clone())))),
            cash_concentration_engine: Arc::new(CashConcentrationEngine::new(Arc::new(PostgresCashConcentrationRepo::new(db_pool.clone())))),
            customer_statement_engine: Arc::new(CustomerStatementEngine::new(Arc::new(PostgresCustomerStatementRepo::new(db_pool.clone())))),
            remittance_batch_engine: Arc::new(RemittanceBatchEngine::new(Arc::new(PostgresRemittanceBatchRepo::new(db_pool.clone())))),
            chargeback_engine: Arc::new(ChargebackManagementEngine::new(Arc::new(PostgresChargebackManagementRepo::new(db_pool.clone())))),
            finance_charge_engine: Arc::new(FinanceChargeEngine::new(Arc::new(PostgresFinanceChargeRepository::new(db_pool.clone())))),
            profitability_engine: Arc::new(ProfitabilityAnalysisEngine::new(Arc::new(PostgresProfitabilityAnalysisRepo::new(db_pool.clone())))),
            recurring_invoice_engine: Arc::new(RecurringInvoiceEngine::new(Arc::new(PostgresRecurringInvoiceRepo::new(db_pool.clone())))),
            payment_settlement_engine: Arc::new(PaymentSettlementEngine::new(Arc::new(PostgresPaymentSettlementRepo::new(db_pool.clone())))),
            payment_process_request_engine: Arc::new(PaymentProcessRequestEngine::new(Arc::new(PostgresPaymentProcessRequestRepository::new(db_pool.clone())))),
            invoice_batch_engine: Arc::new(InvoiceBatchEngine::new(Arc::new(PostgresInvoiceBatchRepository::new(db_pool.clone())))),
            withholding_tax_engine: Arc::new(WithholdingTaxEngine::new(Arc::new(PostgresWithholdingTaxRepo::new(db_pool.clone())))),
            tax_registration_engine: Arc::new(TaxRegistrationEngine::new(Arc::new(PostgresTaxRegistrationRepo::new(db_pool.clone())))),
            doubtful_account_engine: Arc::new(DoubtfulAccountAllowanceEngine::new(Arc::new(PostgresDoubtfulAccountAllowanceRepo::new(db_pool.clone())))),
            invoice_matching_engine: Arc::new(InvoiceMatchingEngine::new(Arc::new(PostgresInvoiceMatchingRepository::new(db_pool.clone())))),
            distribution_set_engine: Arc::new(DistributionSetEngine::new(Arc::new(PostgresDistributionSetRepository::new(db_pool.clone())))),
            third_party_payment_engine: Arc::new(ThirdPartyPaymentEngine::new(Arc::new(PostgresThirdPartyPaymentRepo::new(db_pool.clone())))),
            auto_offset_engine: Arc::new(AutoOffsetEngine::new(Arc::new(PostgresAutoOffsetRepo::new(db_pool.clone())))),
            cash_flow_statement_engine: Arc::new(CashFlowStatementEngine::new(Arc::new(PostgresCashFlowStatementRepo::new(db_pool.clone())))),
            average_balance_engine: Arc::new(AverageBalanceEngine::new(Arc::new(PostgresAverageBalanceRepo::new(db_pool.clone())))),
            cash_receipt_engine: Arc::new(CashReceiptEngine::new(Arc::new(PostgresCashReceiptRepo::new(db_pool.clone())))),
            direct_debit_mandate_engine: Arc::new(DirectDebitMandateEngine::new(Arc::new(PostgresDirectDebitMandateRepo::new(db_pool.clone())))),
            dunning_letter_management_engine: Arc::new(DunningLetterManagementEngine::new(Arc::new(PostgresDunningLetterManagementRepo::new(db_pool.clone())))),
            statistical_accounting_engine: Arc::new(StatisticalAccountingEngine::new(Arc::new(PostgresStatisticalAccountingRepo::new(db_pool.clone())))),
            receivables_factoring_engine: Arc::new(ReceivablesFactoringEngine::new(Arc::new(PostgresReceivablesFactoringRepo::new(db_pool.clone())))),
            asset_retirement_engine: Arc::new(AssetRetirementEngine::new(Arc::new(PostgresAssetRetirementRepo::new(db_pool.clone())))),
            mpa_engine: Arc::new(MultiPeriodAccountingEngine::new(Arc::new(PostgresMpaRepo::new(db_pool.clone())))),
            treasury_engine: Arc::new(TreasuryEngine::new(Arc::new(PostgresTreasuryRepo::new(db_pool.clone())))),
            grant_management_engine: Arc::new(GrantManagementEngine::new(Arc::new(PostgresGrantManagementRepo::new(db_pool.clone())))),
        };

        let hcm = HcmState {
            absence_engine: Arc::new(AbsenceEngine::new(Arc::new(PostgresAbsenceRepo::new(db_pool.clone())))),
            time_and_labor_engine: Arc::new(TimeAndLaborEngine::new(Arc::new(PostgresTimeAndLaborRepo::new(db_pool.clone())))),
            payroll_engine: Arc::new(PayrollEngine::new(Arc::new(PostgresPayrollRepo::new(db_pool.clone())))),
            compensation_engine: Arc::new(CompensationEngine::new(Arc::new(PostgresCompensationRepository::new(db_pool.clone())))),
            benefits_engine: Arc::new(BenefitsEngine::new(Arc::new(PostgresBenefitsRepo::new(db_pool.clone())))),
            performance_engine: Arc::new(PerformanceEngine::new(Arc::new(PostgresPerformanceRepo::new(db_pool.clone())))),
            recruiting_engine: Arc::new(RecruitingEngine::new(Arc::new(PostgresRecruitingRepo::new(db_pool.clone())))),
            learning_management_engine: Arc::new(LearningManagementEngine::new(Arc::new(PostgresLearningManagementRepo::new(db_pool.clone())))),
            succession_planning_engine: Arc::new(SuccessionPlanningEngine::new(Arc::new(PostgresSuccessionPlanningRepo::new(db_pool.clone())))),
            goal_management_engine: Arc::new(GoalManagementEngine::new(Arc::new(PostgresGoalManagementRepo::new(db_pool.clone())))),
            approval_authority_engine: Arc::new(ApprovalAuthorityEngine::new(Arc::new(PostgresApprovalAuthorityRepo::new(db_pool.clone())))),
            approval_delegation_engine: Arc::new(ApprovalDelegationEngine::new(Arc::new(PostgresApprovalDelegationRepo::new(db_pool.clone())))),
            data_archiving_engine: Arc::new(DataArchivingEngine::new(Arc::new(PostgresDataArchivingRepo::new(db_pool.clone())))),
        };

        let scm = ScmState {
            sourcing_engine: Arc::new(SourcingEngine::new(Arc::new(PostgresSourcingRepo::new(db_pool.clone())))),
            procurement_contract_engine: Arc::new(ProcurementContractEngine::new(Arc::new(PostgresProcurementContractRepo::new(db_pool.clone())))),
            inventory_engine: Arc::new(InventoryEngine::new(Arc::new(PostgresInventoryRepo::new(db_pool.clone())))),
            customer_returns_engine: Arc::new(CustomerReturnsEngine::new(Arc::new(PostgresCustomerReturnsRepo::new(db_pool.clone())))),
            pricing_engine: Arc::new(PricingEngine::new(Arc::new(PostgresPricingRepo::new(db_pool.clone())))),
            purchase_requisition_engine: Arc::new(PurchaseRequisitionEngine::new(Arc::new(PostgresPurchaseRequisitionRepo::new(db_pool.clone())))),
            product_information_engine: Arc::new(ProductInformationEngine::new(Arc::new(PostgresProductInformationRepo::new(db_pool.clone())))),
            quality_engine: Arc::new(QualityManagementEngine::new(Arc::new(PostgresQualityManagementRepo::new(db_pool.clone())))),
            order_management_engine: Arc::new(OrderManagementEngine::new(Arc::new(PostgresOrderManagementRepo::new(db_pool.clone())))),
            manufacturing_engine: Arc::new(ManufacturingEngine::new(Arc::new(PostgresManufacturingRepo::new(db_pool.clone())))),
            warehouse_management_engine: Arc::new(WarehouseManagementEngine::new(Arc::new(PostgresWarehouseManagementRepo::new(db_pool.clone())))),
            shipping_engine: Arc::new(ShippingEngine::new(Arc::new(PostgresShippingRepo::new(db_pool.clone())))),
            receiving_engine: Arc::new(ReceivingEngine::new(Arc::new(PostgresReceivingRepo::new(db_pool.clone())))),
            supplier_qualification_engine: Arc::new(SupplierQualificationEngine::new(Arc::new(PostgresSupplierQualificationRepo::new(db_pool.clone())))),
            scorecard_engine: Arc::new(SupplierScorecardEngine::new(Arc::new(PostgresScorecardRepo::new(db_pool.clone())))),
            landed_cost_engine: Arc::new(LandedCostEngine::new(Arc::new(PostgresLandedCostRepo::new(db_pool.clone())))),
            clm_engine: Arc::new(ContractLifecycleEngine::new(Arc::new(PostgresContractLifecycleRepo::new(db_pool.clone())))),
            demand_planning_engine: Arc::new(DemandPlanningEngine::new(Arc::new(PostgresDemandPlanningRepo::new(db_pool.clone())))),
            planning_engine: Arc::new(SupplyChainPlanningEngine::new(Arc::new(PostgresPlanningRepo::new(db_pool.clone())))),
            configurator_engine: Arc::new(ProductConfiguratorEngine::new(Arc::new(PostgresProductConfiguratorRepo::new(db_pool.clone())))),
            transportation_engine: Arc::new(TransportationManagementEngine::new(Arc::new(PostgresTransportationManagementRepo::new(db_pool.clone())))),
            channel_revenue_engine: Arc::new(ChannelRevenueEngine::new(Arc::new(PostgresChannelRevenueRepo::new(db_pool.clone())))),
            rebate_management_engine: Arc::new(RebateManagementEngine::new(Arc::new(PostgresRebateManagementRepo::new(db_pool.clone())))),
        };

        let crm = CrmState {
            sales_commission_engine: Arc::new(SalesCommissionEngine::new(Arc::new(PostgresSalesCommissionRepo::new(db_pool.clone())))),
            subscription_engine: Arc::new(SubscriptionEngine::new(Arc::new(PostgresSubscriptionRepo::new(db_pool.clone())))),
            lead_opportunity_engine: Arc::new(LeadOpportunityEngine::new(Arc::new(PostgresLeadOpportunityRepo::new(db_pool.clone())))),
            marketing_engine: Arc::new(MarketingEngine::new(Arc::new(PostgresMarketingRepo::new(db_pool.clone())))),
            service_request_engine: Arc::new(ServiceRequestEngine::new(Arc::new(PostgresServiceRequestRepo::new(db_pool.clone())))),
            loyalty_engine: Arc::new(LoyaltyManagementEngine::new(Arc::new(PostgresLoyaltyManagementRepo::new(db_pool.clone())))),
            promotions_engine: Arc::new(PromotionsManagementEngine::new(Arc::new(PostgresPromotionsManagementRepo::new(db_pool.clone())))),
        };

        let projects = ProjectsState {
            project_costing_engine: Arc::new(ProjectCostingEngine::new(Arc::new(PostgresProjectCostingRepo::new(db_pool.clone())))),
            project_resource_engine: Arc::new(ProjectResourceManagementEngine::new(Arc::new(PostgresProjectResourceManagementRepo::new(db_pool.clone())))),
            joint_venture_engine: Arc::new(JointVentureEngine::new(Arc::new(PostgresJointVentureRepo::new(db_pool.clone())))),
            project_billing_engine: Arc::new(ProjectBillingEngine::new(Arc::new(PostgresProjectBillingRepo::new(db_pool.clone())))),
        };

        let shared = SharedState {
            lease_accounting_engine: Arc::new(LeaseAccountingEngine::new(Arc::new(PostgresLeaseAccountingRepo::new(db_pool.clone())))),
            cost_allocation_engine: Arc::new(CostAllocationEngine::new(Arc::new(PostgresCostAllocationRepo::new(db_pool.clone())))),
            financial_reporting_engine: Arc::new(FinancialReportingEngine::new(Arc::new(PostgresFinancialReportingRepo::new(db_pool.clone())))),
            dff_engine: Arc::new(DescriptiveFlexfieldEngine::new(Arc::new(PostgresDescriptiveFlexfieldRepo::new(db_pool.clone())))),
            cvr_engine: Arc::new(CrossValidationEngine::new(Arc::new(PostgresCrossValidationRepo::new(db_pool.clone())))),
            scheduled_process_engine: Arc::new(ScheduledProcessEngine::new(Arc::new(PostgresScheduledProcessRepo::new(db_pool.clone())))),
            sod_engine: Arc::new(SegregationOfDutiesEngine::new(Arc::new(PostgresSegregationOfDutiesRepo::new(db_pool.clone())))),
            kpi_engine: Arc::new(KpiEngine::new(Arc::new(PostgresKpiRepo::new(db_pool.clone())))),
            eam_engine: Arc::new(EnterpriseAssetManagementEngine::new(Arc::new(PostgresAssetManagementRepo::new(db_pool.clone())))),
            risk_management_engine: Arc::new(RiskManagementEngine::new(Arc::new(PostgresRiskManagementRepo::new(db_pool.clone())))),
            sustainability_engine: Arc::new(SustainabilityEngine::new(Arc::new(PostgresSustainabilityRepo::new(db_pool.clone())))),
            ecm_engine: Arc::new(EngineeringChangeEngine::new(Arc::new(PostgresEcmRepository::new(db_pool.clone())))),
            health_safety_engine: Arc::new(HealthSafetyEngine::new(Arc::new(PostgresHealthSafetyRepo::new(db_pool.clone())))),
            transfer_pricing_engine: Arc::new(TransferPricingEngine::new(Arc::new(PostgresTransferPricingRepo::new(db_pool.clone())))),
            cost_accounting_engine: Arc::new(CostAccountingEngine::new(Arc::new(PostgresCostAccountingRepo::new(db_pool.clone())))),
            funds_reservation_engine: Arc::new(FundsReservationEngine::new(Arc::new(PostgresFundsReservationRepo::new(db_pool.clone())))),
            territory_engine: Arc::new(TerritoryManagementEngine::new(Arc::new(PostgresTerritoryManagementRepo::new(db_pool.clone())))),
        };

        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
        let event_bus = Arc::new(NatsEventBus::new(&nats_url, "atlas-gateway").await.unwrap_or_else(|_| NatsEventBus::noop("atlas-gateway")));

        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-key-please-change-in-production-1234567890".to_string());
        if jwt_secret.len() < 32 { anyhow::bail!("JWT_SECRET must be at least 32 characters long"); }

        if let Err(e) = schema_engine.load_all().await { tracing::warn!("Failed to load entities: {e}"); }
        let entity_names = schema_engine.entity_names();
        for name in &entity_names {
            if let Some(entity) = schema_engine.get_entity(name) {
                if let Some(workflow) = &entity.workflow {
                    if let Err(e) = workflow_engine.load_workflow(workflow.clone()).await { tracing::warn!("Failed to load workflow for {name}: {e}"); }
                }
            }
        }
        info!("Initialized {} entities", entity_names.len());

        Ok(Self { db_pool, core, financials, hcm, scm, crm, projects, shared, event_bus, jwt_secret })
    }
}
