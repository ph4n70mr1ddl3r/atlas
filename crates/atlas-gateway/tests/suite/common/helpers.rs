//! Helper functions and types for E2E tests

use std::sync::Arc;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};

use atlas_core::{
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine,
    SecurityEngine, AuditEngine,
    eventbus::NatsEventBus,
    ManualJournalEngine,
    ScheduledProcessEngine,
    ApprovalDelegationEngine,
    WarehouseManagementEngine,
    ApprovalAuthorityEngine,
    DataArchivingEngine,
    RecruitingEngine,
    MarketingEngine,
    ReceivingEngine,
    SupplierScorecardEngine,
    KpiEngine,
    AccountMonitorEngine,
};
use atlas_shared::{
    EntityDefinition, FieldDefinition, FieldType, WorkflowDefinition,
    StateDefinition, StateType, TransitionDefinition,
};
use uuid::Uuid;

pub const TEST_JWT_SECRET: &str = "test-jwt-secret-key-for-e2e-testing-minimum-32-chars";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub org_id: String,
    pub exp: i64,
}

pub fn admin_claims() -> Claims {
    Claims {
        sub: "00000000-0000-0000-0000-000000000002".to_string(),
        email: "admin@atlas.local".to_string(),
        name: "System Administrator".to_string(),
        roles: vec!["admin".to_string(), "system".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn user_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4().to_string(),
        email: "user@example.com".to_string(),
        name: "Test User".to_string(),
        roles: vec!["user".to_string()],
        org_id: "00000000-0000-0000-0000-000000000001".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
    }
}

pub fn make_test_token(claims: &Claims) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("failed to encode test JWT")
}

pub fn auth_header(claims: &Claims) -> (String, String) {
    let token = make_test_token(claims);
    ("Authorization".to_string(), format!("Bearer {}", token))
}

pub async fn build_test_app() -> Router {
    let state = build_test_state().await;
    build_router(state)
}

pub fn build_router(state: Arc<atlas_gateway::AppState>) -> Router {
    let cors = CorsLayer::new().allow_methods(Any).allow_headers(Any).allow_origin(Any);
    Router::new()
        .nest("/api/v1", atlas_gateway::handlers::api_routes())
        .nest("/api/admin", atlas_gateway::handlers::admin_routes())
        .route("/health", axum::routing::get(atlas_gateway::handlers::health_check))
        .route("/metrics", axum::routing::get(atlas_gateway::handlers::metrics))
        .route("/api/v1/auth/login", axum::routing::post(atlas_gateway::handlers::login))
        .layer(cors)
        .with_state(state)
}

pub async fn build_test_state() -> Arc<atlas_gateway::AppState> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://atlas:atlas@localhost:5432/atlas".to_string());

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let schema_engine = Arc::new(SchemaEngine::new(Arc::new(
        atlas_core::schema::PostgresSchemaRepository::new(db_pool.clone()),
    )));
    let audit_engine = Arc::new(AuditEngine::new(Arc::new(
        atlas_core::audit::PostgresAuditRepository::new(db_pool.clone()),
    )));
    let workflow_engine = Arc::new(WorkflowEngine::new());
    let validation_engine = Arc::new(ValidationEngine::new());
    let formula_engine = Arc::new(FormulaEngine::new());
    let security_engine = Arc::new(SecurityEngine::new());
    let event_bus = Arc::new(NatsEventBus::noop("atlas-gateway-test"));

    let notification_engine = Arc::new(atlas_core::NotificationEngine::new(Arc::new(
        atlas_core::notification::PostgresNotificationRepository::new(db_pool.clone()),
    )));
    let approval_engine = Arc::new(atlas_core::ApprovalEngine::new(Arc::new(
        atlas_core::approval::PostgresApprovalRepository::new(db_pool.clone()),
    )));

    let period_close_engine = Arc::new(atlas_core::PeriodCloseEngine::new(Arc::new(
        atlas_core::period_close::PostgresPeriodCloseRepository::new(db_pool.clone()),
    )));

    let currency_engine = Arc::new(atlas_core::CurrencyEngine::new(Arc::new(
        atlas_core::currency::PostgresCurrencyRepository::new(db_pool.clone()),
    )));

    let tax_engine = Arc::new(atlas_core::TaxEngine::new(Arc::new(
        atlas_core::tax::PostgresTaxRepository::new(db_pool.clone()),
    )));

    let intercompany_engine = Arc::new(atlas_core::IntercompanyEngine::new(Arc::new(
        atlas_core::intercompany::PostgresIntercompanyRepository::new(db_pool.clone()),
    )));

    let reconciliation_engine = Arc::new(atlas_core::ReconciliationEngine::new(Arc::new(
        atlas_core::reconciliation::PostgresReconciliationRepository::new(db_pool.clone()),
    )));

    let expense_engine = Arc::new(atlas_core::ExpenseEngine::new(Arc::new(
        atlas_core::expense::PostgresExpenseRepository::new(db_pool.clone()),
    )));

    let budget_engine = Arc::new(atlas_core::BudgetEngine::new(Arc::new(
        atlas_core::budget::PostgresBudgetRepository::new(db_pool.clone()),
    )));

    let fixed_asset_engine = Arc::new(atlas_core::FixedAssetEngine::new(Arc::new(
        atlas_core::fixed_assets::PostgresFixedAssetRepository::new(db_pool.clone()),
    )));

    let sla_engine = Arc::new(atlas_core::SubledgerAccountingEngine::new(Arc::new(
        atlas_core::subledger_accounting::PostgresSubledgerAccountingRepository::new(db_pool.clone()),
    )));

    let encumbrance_engine = Arc::new(atlas_core::EncumbranceEngine::new(Arc::new(
        atlas_core::encumbrance::PostgresEncumbranceRepository::new(db_pool.clone()),
    )));

    let cash_management_engine = Arc::new(atlas_core::CashManagementEngine::new(Arc::new(
        atlas_core::cash_management::PostgresCashManagementRepository::new(db_pool.clone()),
    )));

    let sourcing_engine = Arc::new(atlas_core::SourcingEngine::new(Arc::new(
        atlas_core::sourcing::PostgresSourcingRepository::new(db_pool.clone()),
    )));

    let lease_accounting_engine = Arc::new(atlas_core::LeaseAccountingEngine::new(Arc::new(
        atlas_core::lease::PostgresLeaseAccountingRepository::new(db_pool.clone()),
    )));

    let project_costing_engine = Arc::new(atlas_core::ProjectCostingEngine::new(Arc::new(
        atlas_core::project_costing::PostgresProjectCostingRepository::new(db_pool.clone()),
    )));

    let cost_allocation_engine = Arc::new(atlas_core::CostAllocationEngine::new(Arc::new(
        atlas_core::cost_allocation::PostgresCostAllocationRepository::new(db_pool.clone()),
    )));

    let financial_reporting_engine = Arc::new(atlas_core::FinancialReportingEngine::new(Arc::new(
        atlas_core::financial_reporting::PostgresFinancialReportingRepository::new(db_pool.clone()),
    )));

    let multi_book_engine = Arc::new(atlas_core::MultiBookAccountingEngine::new(Arc::new(
        atlas_core::multi_book::PostgresMultiBookAccountingRepository::new(db_pool.clone()),
    )));

    let procurement_contract_engine = Arc::new(atlas_core::ProcurementContractEngine::new(Arc::new(
        atlas_core::procurement_contracts::PostgresProcurementContractRepository::new(db_pool.clone()),
    )));

    let customer_returns_engine = Arc::new(atlas_core::CustomerReturnsEngine::new(Arc::new(
        atlas_core::customer_returns::PostgresCustomerReturnsRepository::new(db_pool.clone()),
    )));

    let inventory_engine = Arc::new(atlas_core::InventoryEngine::new(Arc::new(
        atlas_core::inventory::PostgresInventoryRepository::new(db_pool.clone()),
    )));

    let pricing_engine = Arc::new(atlas_core::PricingEngine::new(Arc::new(
        atlas_core::pricing::PostgresPricingRepository::new(db_pool.clone()),
    )));

    let sales_commission_engine = Arc::new(atlas_core::SalesCommissionEngine::new(Arc::new(
        atlas_core::sales_commission::PostgresSalesCommissionRepository::new(db_pool.clone()),
    )));

    let treasury_engine = Arc::new(atlas_core::TreasuryEngine::new(Arc::new(
        atlas_core::treasury::PostgresTreasuryRepository::new(db_pool.clone()),
    )));

    let grant_management_engine = Arc::new(atlas_core::GrantManagementEngine::new(Arc::new(
        atlas_core::grant_management::PostgresGrantManagementRepository::new(db_pool.clone()),
    )));

    let supplier_qualification_engine = Arc::new(atlas_core::SupplierQualificationEngine::new(Arc::new(
        atlas_core::supplier_qualification::PostgresSupplierQualificationRepository::new(db_pool.clone()),
    )));

    let recurring_journal_engine = Arc::new(atlas_core::RecurringJournalEngine::new(Arc::new(
        atlas_core::recurring_journal::PostgresRecurringJournalRepository::new(db_pool.clone()),
    )));

    let manual_journal_engine = Arc::new(ManualJournalEngine::new(Arc::new(
        atlas_core::manual_journal::PostgresManualJournalRepository::new(db_pool.clone()),
    )));

    let dff_engine = Arc::new(atlas_core::DescriptiveFlexfieldEngine::new(Arc::new(
        atlas_core::descriptive_flexfield::PostgresDescriptiveFlexfieldRepository::new(db_pool.clone()),
    )));

    let cvr_engine = Arc::new(atlas_core::CrossValidationEngine::new(Arc::new(
        atlas_core::cross_validation::PostgresCrossValidationRepository::new(db_pool.clone()),
    )));

    let scheduled_process_engine = Arc::new(ScheduledProcessEngine::new(Arc::new(
        atlas_core::scheduled_process::PostgresScheduledProcessRepository::new(db_pool.clone()),
    )));

    let sod_engine = Arc::new(atlas_core::SegregationOfDutiesEngine::new(Arc::new(
        atlas_core::segregation_of_duties::PostgresSegregationOfDutiesRepository::new(db_pool.clone()),
    )));

    let allocation_engine = Arc::new(atlas_core::AllocationEngine::new(Arc::new(
        atlas_core::allocation::PostgresAllocationRepository::new(db_pool.clone()),
    )));

    let currency_revaluation_engine = Arc::new(atlas_core::CurrencyRevaluationEngine::new(Arc::new(
        atlas_core::currency_revaluation::PostgresCurrencyRevaluationRepository::new(db_pool.clone()),
    )));

    let purchase_requisition_engine = Arc::new(atlas_core::PurchaseRequisitionEngine::new(Arc::new(
        atlas_core::purchase_requisition::PostgresPurchaseRequisitionRepository::new(db_pool.clone()),
    )));

    let corporate_card_engine = Arc::new(atlas_core::CorporateCardEngine::new(Arc::new(
        atlas_core::corporate_card::PostgresCorporateCardRepository::new(db_pool.clone()),
    )));

    let benefits_engine = Arc::new(atlas_core::BenefitsEngine::new(Arc::new(
        atlas_core::benefits::PostgresBenefitsRepository::new(db_pool.clone()),
    )));

    let performance_engine = Arc::new(atlas_core::PerformanceEngine::new(Arc::new(
        atlas_core::performance::PostgresPerformanceRepository::new(db_pool.clone()),
    )));

    let credit_management_engine = Arc::new(atlas_core::CreditManagementEngine::new(Arc::new(
        atlas_core::credit_management::PostgresCreditManagementRepository::new(db_pool.clone()),
    )));

    let product_information_engine = Arc::new(atlas_core::ProductInformationEngine::new(Arc::new(
        atlas_core::product_information::PostgresProductInformationRepository::new(db_pool.clone()),
    )));

    let transfer_pricing_engine = Arc::new(atlas_core::TransferPricingEngine::new(Arc::new(
        atlas_core::transfer_pricing::PostgresTransferPricingRepository::new(db_pool.clone()),
    )));

    let approval_delegation_engine = Arc::new(ApprovalDelegationEngine::new(Arc::new(
        atlas_core::approval_delegation::PostgresApprovalDelegationRepository::new(db_pool.clone()),
    )));

    let state = atlas_gateway::AppState {
        db_pool: db_pool.clone(),
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
        approval_delegation_engine,
        order_management_engine: Arc::new(atlas_core::OrderManagementEngine::new(Arc::new(
            atlas_core::order_management::PostgresOrderManagementRepository::new(db_pool.clone()),
        ))),
        manufacturing_engine: Arc::new(atlas_core::ManufacturingEngine::new(Arc::new(
            atlas_core::manufacturing::PostgresManufacturingRepository::new(db_pool.clone()),
        ))),
        warehouse_management_engine: Arc::new(WarehouseManagementEngine::new(Arc::new(
            atlas_core::warehouse_management::PostgresWarehouseManagementRepository::new(db_pool.clone()),
        ))),
        absence_engine: Arc::new(atlas_core::AbsenceEngine::new(Arc::new(
            atlas_core::absence::PostgresAbsenceRepository::new(db_pool.clone()),
        ))),
        time_and_labor_engine: Arc::new(atlas_core::TimeAndLaborEngine::new(Arc::new(
            atlas_core::time_and_labor::PostgresTimeAndLaborRepository::new(db_pool.clone()),
        ))),
        approval_authority_engine: Arc::new(ApprovalAuthorityEngine::new(Arc::new(
            atlas_core::approval_authority::PostgresApprovalAuthorityRepository::new(db_pool.clone()),
        ))),
        data_archiving_engine: Arc::new(DataArchivingEngine::new(Arc::new(
            atlas_core::data_archiving::PostgresDataArchivingRepository::new(db_pool.clone()),
        ))),
        payroll_engine: Arc::new(atlas_core::PayrollEngine::new(Arc::new(
            atlas_core::payroll::PostgresPayrollRepository::new(db_pool.clone()),
        ))),
        compensation_engine: Arc::new(atlas_core::CompensationEngine::new(Arc::new(
            atlas_core::compensation::PostgresCompensationRepository::new(db_pool.clone()),
        ))),
        service_request_engine: Arc::new(atlas_core::ServiceRequestEngine::new(Arc::new(
            atlas_core::service_request::PostgresServiceRequestRepository::new(db_pool.clone()),
        ))),
        lead_opportunity_engine: Arc::new(atlas_core::LeadOpportunityEngine::new(Arc::new(
            atlas_core::lead_opportunity::PostgresLeadOpportunityRepository::new(db_pool.clone()),
        ))),
        demand_planning_engine: Arc::new(atlas_core::DemandPlanningEngine::new(Arc::new(
            atlas_core::demand_planning::PostgresDemandPlanningRepository::new(db_pool.clone()),
        ))),
        shipping_engine: Arc::new(atlas_core::ShippingEngine::new(Arc::new(
            atlas_core::shipping::PostgresShippingRepository::new(db_pool.clone()),
        ))),
        autoinvoice_engine: Arc::new(atlas_core::AutoInvoiceEngine::new(Arc::new(
            atlas_core::autoinvoice::PostgresAutoInvoiceRepository::new(db_pool.clone()),
        ))),
        recruiting_engine: Arc::new(RecruitingEngine::new(Arc::new(
            atlas_core::recruiting::PostgresRecruitingRepository::new(db_pool.clone()),
        ))),
        revenue_engine: Arc::new(atlas_core::RevenueEngine::new(Arc::new(
            atlas_core::revenue::PostgresRevenueRepository::new(db_pool.clone()),
        ))),
        marketing_engine: Arc::new(MarketingEngine::new(Arc::new(
            atlas_core::marketing::PostgresMarketingRepository::new(db_pool.clone()),
        ))),
        receiving_engine: Arc::new(ReceivingEngine::new(Arc::new(
            atlas_core::receiving::PostgresReceivingRepository::new(db_pool.clone()),
        ))),
        scorecard_engine: Arc::new(SupplierScorecardEngine::new(Arc::new(
            atlas_core::supplier_scorecard::PostgresScorecardRepository::new(db_pool.clone()),
        ))),
        kpi_engine: Arc::new(KpiEngine::new(Arc::new(
            atlas_core::kpi::PostgresKpiRepository::new(db_pool.clone()),
        ))),
        account_monitor_engine: Arc::new(AccountMonitorEngine::new(Arc::new(
            atlas_core::account_monitor::PostgresAccountMonitorRepository::new(db_pool.clone()),
        ))),
        event_bus,
        jwt_secret: TEST_JWT_SECRET.to_string(),
    };

    atlas_gateway::state::APP_STATE.set(Arc::new(state.clone())).ok();
    Arc::new(state)
}

pub fn test_entity_definition() -> EntityDefinition {
    EntityDefinition {
        id: Some(Uuid::new_v4()),
        name: "test_items".to_string(),
        label: "Test Item".to_string(),
        plural_label: "Test Items".to_string(),
        table_name: Some("test_items".to_string()),
        description: Some("A test entity for E2E tests".to_string()),
        fields: vec![
            FieldDefinition::new("name", "Name", FieldType::String { max_length: Some(200), pattern: None }),
            FieldDefinition::new("description", "Description", FieldType::String { max_length: None, pattern: None }),
            FieldDefinition::new("quantity", "Quantity", FieldType::Integer { min: Some(0), max: None }),
            FieldDefinition::new("price", "Price", FieldType::Decimal { precision: 12, scale: 2 }),
            FieldDefinition::new("status", "Status", FieldType::Enum {
                values: vec!["draft".to_string(), "active".to_string(), "closed".to_string()],
            }),
        ],
        indexes: vec![],
        workflow: Some(test_workflow_definition()),
        security: None,
        is_audit_enabled: true,
        is_soft_delete: true,
        icon: Some("test".to_string()),
        color: Some("#000000".to_string()),
        metadata: serde_json::Value::Null,
    }
}

pub fn test_workflow_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        id: Some(Uuid::new_v4()),
        name: "test_item_workflow".to_string(),
        initial_state: "draft".to_string(),
        states: vec![
            StateDefinition { name: "draft".into(), label: "Draft".into(), state_type: StateType::Initial,
                entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "submitted".into(), label: "Submitted".into(), state_type: StateType::Working,
                entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "approved".into(), label: "Approved".into(), state_type: StateType::Final,
                entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
            StateDefinition { name: "rejected".into(), label: "Rejected".into(), state_type: StateType::Final,
                entry_actions: vec![], exit_actions: vec![], metadata: serde_json::Value::Null },
        ],
        transitions: vec![
            TransitionDefinition { name: "submit".into(), from_state: "draft".into(), to_state: "submitted".into(),
                action: "submit".into(), action_label: Some("Submit".into()), guards: vec![],
                required_roles: vec![], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "approve".into(), from_state: "submitted".into(), to_state: "approved".into(),
                action: "approve".into(), action_label: Some("Approve".into()), guards: vec![],
                required_roles: vec!["manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
            TransitionDefinition { name: "reject".into(), from_state: "submitted".into(), to_state: "rejected".into(),
                action: "reject".into(), action_label: Some("Reject".into()), guards: vec![],
                required_roles: vec!["manager".into(), "admin".into()], entry_actions: vec![], metadata: serde_json::Value::Null },
        ],
        is_active: true,
    }
}

pub async fn setup_test_db(pool: &sqlx::PgPool) {
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS test_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            organization_id UUID,
            created_at TIMESTAMPTZ DEFAULT now(),
            updated_at TIMESTAMPTZ DEFAULT now(),
            created_by UUID, updated_by UUID, deleted_at TIMESTAMPTZ,
            workflow_state VARCHAR(100) DEFAULT 'draft',
            name TEXT, description TEXT, quantity BIGINT,
            price NUMERIC(12,2), status VARCHAR(100)
        )"#)
        .execute(pool).await.ok();
    sqlx::query("DELETE FROM test_items").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.audit_log WHERE entity_type = 'test_items'").execute(pool).await.ok();
}

pub async fn cleanup_test_db(pool: &sqlx::PgPool) {
    sqlx::query("DELETE FROM test_items").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.workflow_states WHERE entity_type = 'test_items'").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.audit_log WHERE entity_type = 'test_items'").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.entities WHERE name = 'test_items'").execute(pool).await.ok();
    // Clean currency test data
    sqlx::query("DELETE FROM _atlas.currency_conversions WHERE entity_type = 'test_items'").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.exchange_rates").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.currencies").execute(pool).await.ok();
    // Clean tax test data
    sqlx::query("DELETE FROM _atlas.tax_reports").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.tax_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.tax_determination_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.tax_rates").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.tax_jurisdictions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.tax_regimes").execute(pool).await.ok();
    // Clean reconciliation test data
    sqlx::query("DELETE FROM _atlas.reconciliation_matches").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.reconciliation_matching_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.reconciliation_summaries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.bank_statement_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.bank_statements").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.system_transactions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.bank_accounts").execute(pool).await.ok();
    // Clean expense test data
    sqlx::query("DELETE FROM _atlas.expense_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.expense_reports").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.expense_policies").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.expense_categories").execute(pool).await.ok();
    // Clean budget test data
    sqlx::query("DELETE FROM _atlas.budget_transfers").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.budget_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.budget_versions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.budget_definitions").execute(pool).await.ok();
    // Clean fixed asset test data
    sqlx::query("DELETE FROM _atlas.asset_depreciation_history").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.asset_retirements").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.asset_transfers").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.fixed_assets").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.asset_categories").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.asset_books").execute(pool).await.ok();
    // Clean encumbrance test data
    sqlx::query("DELETE FROM _atlas.encumbrance_liquidations").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.encumbrance_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.encumbrance_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.encumbrance_carry_forwards").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.encumbrance_types").execute(pool).await.ok();
    // Clean cash management test data
    sqlx::query("DELETE FROM _atlas.cash_forecast_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_forecasts").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_forecast_sources").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_forecast_templates").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cash_positions").execute(pool).await.ok();
    // Clean sourcing test data
    sqlx::query("DELETE FROM _atlas.sourcing_award_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sourcing_awards").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.response_scores").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scoring_criteria").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_response_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_responses").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sourcing_invites").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sourcing_event_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sourcing_events").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sourcing_templates").execute(pool).await.ok();
    // Clean financial reporting test data
    sqlx::query("DELETE FROM _atlas.financial_report_results").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.financial_report_runs").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.financial_report_favourites").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.financial_report_columns").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.financial_report_rows").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.financial_report_templates").execute(pool).await.ok();
    // Clean multi-book accounting test data
    sqlx::query("DELETE FROM _atlas.propagation_logs").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.book_journal_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.book_journal_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.account_mappings").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.accounting_books").execute(pool).await.ok();
    // Clean procurement contracts test data
    sqlx::query("DELETE FROM _atlas.procurement_contract_spend").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.procurement_contract_renewals").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.procurement_contract_milestones").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.procurement_contract_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.procurement_contracts").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.procurement_contract_types").execute(pool).await.ok();
    // Clean customer returns test data
    sqlx::query("DELETE FROM _atlas.credit_memos").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.return_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.return_authorizations").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.return_reasons").execute(pool).await.ok();
    // Clean sales commission test data
    sqlx::query("DELETE FROM _atlas.commission_payout_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.commission_transactions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.commission_payouts").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sales_quotas").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.plan_assignments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.commission_rate_tiers").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.commission_plans").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sales_reps").execute(pool).await.ok();
    // Clean treasury test data
    sqlx::query("DELETE FROM _atlas.treasury_settlements").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.treasury_deals").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.treasury_counterparties").execute(pool).await.ok();
    // Clean supplier qualification test data
    sqlx::query("DELETE FROM _atlas.supplier_qualification_responses").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_qualification_invitations").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_qualification_initiatives").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_certifications").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.qualification_questions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.qualification_areas").execute(pool).await.ok();
    // Clean recurring journal test data
    sqlx::query("DELETE FROM _atlas.recurring_journal_generation_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_journal_generations").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_journal_schedule_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.recurring_journal_schedules").execute(pool).await.ok();
    // Clean manual journal test data
    sqlx::query("DELETE FROM _atlas.journal_entry_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.journal_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.journal_batches").execute(pool).await.ok();
    // Clean descriptive flexfield test data
    sqlx::query("DELETE FROM _atlas.dff_data").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.dff_segments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.dff_contexts").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.dff_flexfields").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.dff_value_set_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.dff_value_sets").execute(pool).await.ok();
    // Clean cross-validation test data
    sqlx::query("DELETE FROM _atlas.cross_validation_rule_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.cross_validation_rules").execute(pool).await.ok();
    // Clean scheduled process test data
    sqlx::query("DELETE FROM _atlas.scheduled_process_logs").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scheduled_process_recurrences").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scheduled_processes").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scheduled_process_templates").execute(pool).await.ok();
    // Clean allocation test data
    sqlx::query("DELETE FROM _atlas.gl_allocation_run_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_runs").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_target_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_basis_details").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_bases").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_allocation_pools").execute(pool).await.ok();
    // Clean currency revaluation test data
    sqlx::query("DELETE FROM _atlas.currency_revaluation_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.currency_revaluation_runs").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.currency_revaluation_accounts").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.currency_revaluation_definitions").execute(pool).await.ok();
    // Clean purchase requisition test data
    sqlx::query("DELETE FROM _atlas.autocreate_links").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.requisition_approvals").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.requisition_distributions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.requisition_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.purchase_requisitions").execute(pool).await.ok();
    // Clean corporate card test data
    sqlx::query("DELETE FROM _atlas.corporate_card_limit_overrides").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.corporate_card_transactions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.corporate_card_statements").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.corporate_cards").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.corporate_card_programs").execute(pool).await.ok();
    // Clean benefits test data
    sqlx::query("DELETE FROM _atlas.benefits_deductions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.benefits_enrollments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.benefits_plans").execute(pool).await.ok();
    // Clean performance test data
    sqlx::query("DELETE FROM _atlas.performance_feedback").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_competency_assessments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_goals").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_documents").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_competencies").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_review_cycles").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_rating_models").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_dashboard").execute(pool).await.ok();
    // Clean credit management test data
    sqlx::query("DELETE FROM _atlas.credit_holds").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_reviews").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_exposure").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_limits").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_profiles").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_check_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.credit_scoring_models").execute(pool).await.ok();
    // Clean transfer pricing test data
    sqlx::query("DELETE FROM _atlas.transfer_pricing_comparables").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.transfer_pricing_documentation").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.transfer_pricing_transactions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.transfer_pricing_benchmarks").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.transfer_pricing_policies").execute(pool).await.ok();
    // Clean approval delegation test data
    sqlx::query("DELETE FROM _atlas.approval_delegation_history").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.approval_delegation_rules").execute(pool).await.ok();
    // Clean warehouse management test data
    sqlx::query("DELETE FROM _atlas.warehouse_tasks").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.pick_waves").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.put_away_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.warehouse_zones").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.warehouses").execute(pool).await.ok();
    // Clean absence management test data
    sqlx::query("DELETE FROM _atlas.absence_entry_history").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.absence_balances").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.absence_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.absence_plans").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.absence_types").execute(pool).await.ok();
    // Clean approval authority limits test data
    sqlx::query("DELETE FROM _atlas.approval_authority_check_audit").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.approval_authority_limits").execute(pool).await.ok();
    // Clean data archiving test data
    sqlx::query("DELETE FROM _atlas.archive_audit").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.archived_records").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.archive_batches").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.legal_hold_items").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.legal_holds").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.retention_policies").execute(pool).await.ok();
    // Clean compensation management test data
    sqlx::query("DELETE FROM _atlas.compensation_worksheet_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_worksheets").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_budget_pools").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_statements").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_cycles").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_components").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.compensation_plans").execute(pool).await.ok();
    // Clean service request test data
    sqlx::query("DELETE FROM _atlas.service_request_updates").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.service_request_assignments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.service_requests").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.service_categories").execute(pool).await.ok();
    // Clean autoinvoice test data
    sqlx::query("DELETE FROM _atlas.autoinvoice_result_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_results").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_batches").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_validation_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_grouping_rules").execute(pool).await.ok();
    // Clean shipping test data
    sqlx::query("DELETE FROM _atlas.packing_slip_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.packing_slips").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipment_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipments").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipping_methods").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipping_carriers").execute(pool).await.ok();
    // Clean recruiting test data
    sqlx::query("DELETE FROM _atlas.job_offers").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.interviews").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.job_applications").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.candidates").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.job_requisitions").execute(pool).await.ok();
    // Clean receiving test data
    sqlx::query("DELETE FROM _atlas.inspection_details").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_inspections").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_deliveries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_returns").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_headers").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receiving_locations").execute(pool).await.ok();
    // Clean subledger accounting test data
    sqlx::query("DELETE FROM _atlas.subledger_distributions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sla_events").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subledger_journal_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subledger_journal_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_transfer_log").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.accounting_derivation_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.accounting_methods").execute(pool).await.ok();
    // Clean supplier scorecard test data
    sqlx::query("DELETE FROM _atlas.review_action_items").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_performance_reviews").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scorecard_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supplier_scorecards").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scorecard_categories").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.scorecard_templates").execute(pool).await.ok();
    // Clean KPI analytics test data
    sqlx::query("DELETE FROM _atlas.kpi_dashboard_widgets").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.kpi_data_points").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.kpi_dashboards").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.kpi_definitions").execute(pool).await.ok();
    // Clean account monitor test data
    sqlx::query("DELETE FROM _atlas.saved_balance_inquiries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.balance_snapshots").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.account_group_members").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.account_groups").execute(pool).await.ok();
}
