//! Application State
//! 
//! Shared state for all request handlers.

use atlas_core::{
    SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine, 
    SecurityEngine, AuditEngine,
    eventbus::NatsEventBus,
    schema::PostgresSchemaRepository,
    audit::PostgresAuditRepository,
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
            event_bus,
            jwt_secret,
        };
        
        Ok(state)
    }
}
