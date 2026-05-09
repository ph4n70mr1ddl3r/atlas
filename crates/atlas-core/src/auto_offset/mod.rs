//! Automatic Offsets (Intercompany Balancing) Module
//!
//! Oracle Fusion Cloud ERP: Financials > General Ledger > Automatic Offsets
//!
//! Manages the automatic generation of intercompany offset (due-to / due-from)
//! entries when a single journal entry's lines span multiple balancing segments
//! (legal entities). Ensures each balancing segment is self-balancing.
//!
//! Key components:
//!   - Offset Templates: Define offset accounts per balancing segment pair
//!   - Offset Template Lines: Per-entity due-to / due-from account mappings
//!   - Offset Generations: Audit trail of generated offset entries
//!   - Offset Lines: Individual due-to / due-from lines generated
//!   - Offset Activities: Full audit trail of all offset operations

mod engine;

pub use engine::{AutoOffsetEngine, JournalLine};

use atlas_shared::{
    AutoOffsetTemplate, AutoOffsetTemplateLine,
    AutoOffsetGeneration, AutoOffsetLine,
    AutoOffsetActivity, AutoOffsetDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for Automatic Offsets
#[async_trait]
pub trait AutoOffsetRepository: Send + Sync {
    // Template CRUD
    async fn create_template(
        &self, org_id: Uuid, template_code: &str, template_name: &str,
        description: Option<&str>, balancing_segment: &str,
        intercompany_segment: Option<&str>, generation_method: &str,
        default_offset_account: &str, default_offset_account_description: Option<&str>,
        enable_intra_entity: bool, intra_entity_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoOffsetTemplate>;

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<AutoOffsetTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AutoOffsetTemplate>>;
    async fn list_templates(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<AutoOffsetTemplate>>;
    async fn update_template_status(&self, id: Uuid, is_active: bool) -> AtlasResult<AutoOffsetTemplate>;
    async fn delete_template(&self, id: Uuid) -> AtlasResult<()>;

    // Template Lines
    async fn add_template_line(
        &self, org_id: Uuid, template_id: Uuid, line_number: i32,
        balancing_segment_value: &str,
        due_to_account: &str, due_to_account_description: Option<&str>,
        due_from_account: &str, due_from_account_description: Option<&str>,
        clearing_account: Option<&str>, priority: i32,
    ) -> AtlasResult<AutoOffsetTemplateLine>;

    async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<AutoOffsetTemplateLine>>;
    async fn delete_template_line(&self, line_id: Uuid) -> AtlasResult<()>;

    // Offset Generation
    async fn create_generation(
        &self, org_id: Uuid, generation_number: &str,
        template_id: Uuid, source_type: &str, source_id: Option<Uuid>,
        source_number: Option<&str>, fiscal_year: i32, period_name: &str,
        generation_date: chrono::NaiveDate, currency_code: &str,
        total_source_lines: i32, balancing_segments_affected: i32,
        total_offset_lines: i32, total_debit_amount: f64, total_credit_amount: f64,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoOffsetGeneration>;

    async fn get_generation(&self, id: Uuid) -> AtlasResult<Option<AutoOffsetGeneration>>;
    async fn get_generation_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AutoOffsetGeneration>>;
    async fn list_generations(&self, org_id: Uuid, status: Option<&str>, source_type: Option<&str>) -> AtlasResult<Vec<AutoOffsetGeneration>>;
    async fn update_generation_status(&self, id: Uuid, status: &str, posted_by: Option<Uuid>) -> AtlasResult<AutoOffsetGeneration>;

    // Offset Lines
    async fn add_offset_line(
        &self, org_id: Uuid, generation_id: Uuid, line_number: i32,
        from_segment_value: &str, to_segment_value: &str,
        offset_type: &str, account_code: &str, account_description: Option<&str>,
        amount: f64, currency_code: &str,
    ) -> AtlasResult<AutoOffsetLine>;

    async fn list_offset_lines(&self, generation_id: Uuid) -> AtlasResult<Vec<AutoOffsetLine>>;

    // Activities
    async fn log_activity(
        &self, org_id: Uuid, generation_id: Uuid, line_id: Option<Uuid>,
        activity_type: &str, description: Option<&str>,
        old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>,
        details: serde_json::Value,
    ) -> AtlasResult<AutoOffsetActivity>;

    async fn list_activities(&self, generation_id: Uuid) -> AtlasResult<Vec<AutoOffsetActivity>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AutoOffsetDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresAutoOffsetRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresAutoOffsetRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl AutoOffsetRepository for PostgresAutoOffsetRepository {
    async fn create_template(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: bool, _: Option<&str>, _: Option<chrono::NaiveDate>, _: Option<chrono::NaiveDate>, _: Option<Uuid>) -> AtlasResult<AutoOffsetTemplate> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_template(&self, _: Uuid) -> AtlasResult<Option<AutoOffsetTemplate>> { Ok(None) }
    async fn get_template_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<AutoOffsetTemplate>> { Ok(None) }
    async fn list_templates(&self, _: Uuid, _: Option<bool>) -> AtlasResult<Vec<AutoOffsetTemplate>> { Ok(vec![]) }
    async fn update_template_status(&self, _: Uuid, _: bool) -> AtlasResult<AutoOffsetTemplate> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn delete_template(&self, _: Uuid) -> AtlasResult<()> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn add_template_line(&self, _: Uuid, _: Uuid, _: i32, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: i32) -> AtlasResult<AutoOffsetTemplateLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_template_lines(&self, _: Uuid) -> AtlasResult<Vec<AutoOffsetTemplateLine>> { Ok(vec![]) }
    async fn delete_template_line(&self, _: Uuid) -> AtlasResult<()> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_generation(&self, _: Uuid, _: &str, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: i32, _: &str, _: chrono::NaiveDate, _: &str, _: i32, _: i32, _: i32, _: f64, _: f64, _: Option<Uuid>) -> AtlasResult<AutoOffsetGeneration> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_generation(&self, _: Uuid) -> AtlasResult<Option<AutoOffsetGeneration>> { Ok(None) }
    async fn get_generation_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<AutoOffsetGeneration>> { Ok(None) }
    async fn list_generations(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<Vec<AutoOffsetGeneration>> { Ok(vec![]) }
    async fn update_generation_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<AutoOffsetGeneration> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn add_offset_line(&self, _: Uuid, _: Uuid, _: i32, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: f64, _: &str) -> AtlasResult<AutoOffsetLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_offset_lines(&self, _: Uuid) -> AtlasResult<Vec<AutoOffsetLine>> { Ok(vec![]) }
    async fn log_activity(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: serde_json::Value) -> AtlasResult<AutoOffsetActivity> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_activities(&self, _: Uuid) -> AtlasResult<Vec<AutoOffsetActivity>> { Ok(vec![]) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<AutoOffsetDashboard> {
        Ok(AutoOffsetDashboard {
            total_templates: 0,
            active_templates: 0,
            total_generations: 0,
            generated_count: 0,
            posted_count: 0,
            reversed_count: 0,
            total_offset_lines: 0,
            total_offset_amount: 0.0,
        })
    }
}
