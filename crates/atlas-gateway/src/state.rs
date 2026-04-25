//! Application State
//! 
//! Shared state for all request handlers.

use atlas_core::{
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
            event_bus,
            jwt_secret,
        };
        
        Ok(state)
    }
}
