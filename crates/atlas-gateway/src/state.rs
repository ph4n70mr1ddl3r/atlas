//! Application State
//! 
//! Shared state for all request handlers.

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine, FormulaEngine, SecurityEngine, AuditEngine, MockSchemaRepository, MockAuditRepository};
use sqlx::PgPool;
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
            .await?;
        
        info!("Connected to database");
        
        // Create in-memory schema engine
        let schema_engine = Arc::new(SchemaEngine::new(Arc::new(MockSchemaRepository)));
        
        // Initialize other engines
        let workflow_engine = Arc::new(WorkflowEngine::new());
        let validation_engine = Arc::new(ValidationEngine::new());
        let formula_engine = Arc::new(FormulaEngine::new());
        let security_engine = Arc::new(SecurityEngine::new());
        let audit_engine = Arc::new(AuditEngine::new(Arc::new(MockAuditRepository)));
        
        // Load JWT secret from environment
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET environment variable must be set");
        
        // Validate JWT secret strength
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters long");
        }
        
        // Warn if using weak secrets
        if jwt_secret.contains("secret") || jwt_secret.contains("password") || jwt_secret.contains("change") {
            tracing::warn!("JWT_SECRET appears to be a weak or placeholder value. Use a cryptographically random secret in production.");
        }
        
        let state = Self {
            db_pool,
            schema_engine,
            workflow_engine,
            validation_engine,
            formula_engine,
            security_engine,
            audit_engine,
            jwt_secret,
        };
        
        // Register global state
        APP_STATE.set(Arc::new(state.clone())).ok();
        
        Ok(state)
    }
}
