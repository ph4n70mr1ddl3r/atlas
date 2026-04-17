//! Mock repositories for testing and development

use atlas_shared::{EntityDefinition, AuditEntry};
use atlas_shared::errors::AtlasResult;
use async_trait::async_trait;
use uuid::Uuid;
use crate::schema::SchemaRepository;
use crate::audit::AuditRepository;
use crate::tax::TaxRepository;

/// Mock schema repository
pub struct MockSchemaRepository;

#[async_trait]
impl SchemaRepository for MockSchemaRepository {
    async fn get_all_entities(&self) -> AtlasResult<Vec<EntityDefinition>> { Ok(vec![]) }
    async fn get_entity(&self, _name: &str) -> AtlasResult<Option<EntityDefinition>> { Ok(None) }
    async fn upsert_entity(&self, _entity: &EntityDefinition) -> AtlasResult<()> { Ok(()) }
    async fn delete_entity(&self, _name: &str) -> AtlasResult<()> { Ok(()) }
    async fn get_entity_version(&self, _name: &str) -> AtlasResult<Option<i64>> { Ok(Some(1)) }
    async fn set_entity_version(&self, _name: &str, _version: i64) -> AtlasResult<()> { Ok(()) }
}

/// Mock audit repository
pub struct MockAuditRepository;

#[async_trait]
impl AuditRepository for MockAuditRepository {
    async fn insert(&self, _entry: &AuditEntry) -> AtlasResult<()> { Ok(()) }
    async fn query(&self, _query: &crate::audit::AuditQuery) -> AtlasResult<Vec<AuditEntry>> { Ok(vec![]) }
    async fn get_by_id(&self, _id: Uuid) -> AtlasResult<Option<AuditEntry>> { Ok(None) }
    async fn get_by_ids(&self, _ids: &[Uuid]) -> AtlasResult<Vec<AuditEntry>> { Ok(vec![]) }
}

/// Mock tax repository for testing
pub struct MockTaxRepository;

#[async_trait]
impl TaxRepository for MockTaxRepository {
    async fn create_regime(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _tax_type: &str, _default_inclusive: bool, _allows_recovery: bool,
        _rounding_rule: &str, _rounding_precision: i32,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxRegime> {
        Ok(atlas_shared::TaxRegime {
            id: Uuid::new_v4(), organization_id: _org_id, code: _code.to_string(),
            name: _name.to_string(), description: None, tax_type: _tax_type.to_string(),
            default_inclusive: _default_inclusive, allows_recovery: _allows_recovery,
            rounding_rule: _rounding_rule.to_string(), rounding_precision: _rounding_precision,
            is_active: true, effective_from: None, effective_to: None,
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_regime(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRegime>> { Ok(None) }
    async fn get_regime_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TaxRegime>> { Ok(None) }
    async fn list_regimes(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxRegime>> { Ok(vec![]) }
    async fn delete_regime(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_jurisdiction(
        &self, _org_id: Uuid, _regime_id: Uuid, _code: &str, _name: &str,
        _geographic_level: &str, _country_code: Option<&str>, _state_code: Option<&str>,
        _county: Option<&str>, _city: Option<&str>, _postal_code_pattern: Option<&str>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxJurisdiction> {
        Ok(atlas_shared::TaxJurisdiction {
            id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
            code: _code.to_string(), name: _name.to_string(),
            geographic_level: _geographic_level.to_string(),
            country_code: None, state_code: None, county: None, city: None,
            postal_code_pattern: None, is_active: true, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_jurisdiction(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxJurisdiction>> { Ok(None) }
    async fn list_jurisdictions(&self, _org_id: Uuid, _regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxJurisdiction>> { Ok(vec![]) }
    async fn delete_jurisdiction(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_tax_rate(
        &self, _org_id: Uuid, _regime_id: Uuid, _jurisdiction_id: Option<&Uuid>,
        _code: &str, _name: &str, _rate_percentage: &str, _rate_type: &str,
        _tax_account_code: Option<&str>, _recoverable: bool, _recovery_percentage: Option<&str>,
        _effective_from: chrono::NaiveDate, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxRate> {
        Ok(atlas_shared::TaxRate {
            id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
            jurisdiction_id: None, code: _code.to_string(), name: _name.to_string(),
            rate_percentage: _rate_percentage.to_string(), rate_type: _rate_type.to_string(),
            tax_account_code: None, recoverable: false, recovery_percentage: None,
            effective_from: _effective_from, effective_to: None, is_active: true,
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_tax_rate(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_tax_rate_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_tax_rate_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_effective_tax_rates(&self, _org_id: Uuid, _regime_id: Uuid, _on_date: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::TaxRate>> { Ok(vec![]) }
    async fn list_tax_rates(&self, _org_id: Uuid, _regime_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxRate>> { Ok(vec![]) }
    async fn delete_tax_rate(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_determination_rule(
        &self, _org_id: Uuid, _regime_id: Uuid, _name: &str, _description: Option<&str>,
        _priority: i32, _condition: serde_json::Value, _action: serde_json::Value,
        _stop_on_match: bool, _effective_from: Option<chrono::NaiveDate>,
        _effective_to: Option<chrono::NaiveDate>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxDeterminationRule> { Ok(atlas_shared::TaxDeterminationRule {
        id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
        name: _name.to_string(), description: None, priority: _priority,
        condition: _condition, action: _action, stop_on_match: _stop_on_match,
        is_active: true, effective_from: None, effective_to: None,
        created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    }) }
    async fn list_determination_rules(&self, _org_id: Uuid, _regime_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxDeterminationRule>> { Ok(vec![]) }
    async fn create_tax_line(
        &self, _org_id: Uuid, _entity_type: &str, _entity_id: Uuid, _line_id: Option<Uuid>,
        _regime_id: Option<Uuid>, _jurisdiction_id: Option<Uuid>, _tax_rate_id: Uuid,
        _taxable_amount: &str, _tax_rate_percentage: &str, _tax_amount: &str,
        _is_inclusive: bool, _original_amount: Option<&str>,
        _recoverable_amount: Option<&str>, _non_recoverable_amount: Option<&str>,
        _tax_account_code: Option<&str>, _determination_rule_id: Option<Uuid>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxLine> { Ok(atlas_shared::TaxLine {
        id: Uuid::new_v4(), organization_id: _org_id,
        entity_type: _entity_type.to_string(), entity_id: _entity_id, line_id: None,
        regime_id: None, jurisdiction_id: None, tax_rate_id: _tax_rate_id,
        taxable_amount: _taxable_amount.to_string(),
        tax_rate_percentage: _tax_rate_percentage.to_string(),
        tax_amount: _tax_amount.to_string(),
        is_inclusive: false, original_amount: None,
        recoverable_amount: None, non_recoverable_amount: None,
        tax_account_code: None, determination_rule_id: None,
        created_by: None, created_at: chrono::Utc::now(),
    }) }
    async fn get_tax_lines(&self, _entity_type: &str, _entity_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxLine>> { Ok(vec![]) }
    async fn generate_tax_report(
        &self, _org_id: Uuid, _regime_id: Uuid, _jurisdiction_id: Option<&Uuid>,
        _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxReport> { Ok(atlas_shared::TaxReport {
        id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
        jurisdiction_id: None, period_start: _period_start, period_end: _period_end,
        total_taxable_amount: "0".to_string(), total_tax_amount: "0".to_string(),
        total_recoverable_amount: "0".to_string(), total_non_recoverable_amount: "0".to_string(),
        transaction_count: 0, status: "draft".to_string(),
        filed_by: None, filed_at: None, metadata: serde_json::json!({}),
        created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    }) }
    async fn list_tax_reports(&self, _org_id: Uuid, _regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxReport>> { Ok(vec![]) }
}
