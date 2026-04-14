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

    let state = atlas_gateway::AppState {
        db_pool: db_pool.clone(),
        schema_engine,
        workflow_engine,
        validation_engine,
        formula_engine,
        security_engine,
        audit_engine,
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
    sqlx::query("DELETE FROM _atlas.audit_log WHERE entity_type = 'test_items'").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.entities WHERE name = 'test_items'").execute(pool).await.ok();
}
