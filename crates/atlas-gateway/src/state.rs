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
};
use std::sync::Arc;
use once_cell::sync::OnceCell;
use tracing::info;

pub static APP_STATE: OnceCell<Arc<AppState>> = OnceCell::new();

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
        if jwt_secret.contains("secret") || jwt_secret.contains("password") || jwt_secret.contains("change") {
            tracing::warn!("JWT_SECRET appears to be a weak or placeholder value. Use a cryptographically random secret in production.");
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
            event_bus,
            jwt_secret,
        };
        
        Ok(state)
    }
}
