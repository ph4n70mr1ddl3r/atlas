//! Helper functions and types for E2E tests

use std::sync::Arc;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};

use atlas_core::{
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine,
    SecurityEngine, AuditEngine,
    eventbus::NatsEventBus,
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
}
