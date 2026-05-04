//! Contract Lifecycle Repository
//!
//! PostgreSQL storage for contract lifecycle management data.

use atlas_shared::{
    ClmContractType, ClmClause, ClmTemplate, ClmTemplateClause,
    ClmContract, ClmContractParty, ClmContractClause, ClmMilestone,
    ClmDeliverable, ClmAmendment, ClmRisk, ClmDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

#[async_trait]
pub trait ContractLifecycleRepository: Send + Sync {
    async fn create_contract_type(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, contract_category: &str, default_duration_days: Option<i32>, requires_approval: bool, is_auto_renew: bool, risk_scoring_enabled: bool, created_by: Option<Uuid>) -> AtlasResult<ClmContractType>;
    async fn get_contract_type(&self, id: Uuid) -> AtlasResult<Option<ClmContractType>>;
    async fn get_contract_type_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmContractType>>;
    async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ClmContractType>>;
    async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn create_clause(&self, org_id: Uuid, code: &str, title: &str, body: &str, clause_type: &str, clause_category: &str, applicability: &str, is_locked: bool, created_by: Option<Uuid>) -> AtlasResult<ClmClause>;
    async fn get_clause(&self, id: Uuid) -> AtlasResult<Option<ClmClause>>;
    async fn get_clause_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmClause>>;
    async fn list_clauses(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<ClmClause>>;
    async fn delete_clause(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, contract_type_id: Option<Uuid>, default_currency: &str, default_duration_days: Option<i32>, terms_and_conditions: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<ClmTemplate>;
    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ClmTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<ClmTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn add_template_clause(&self, template_id: Uuid, clause_id: Uuid, section: Option<&str>, display_order: i32, is_required: bool) -> AtlasResult<ClmTemplateClause>;
    async fn list_template_clauses(&self, template_id: Uuid) -> AtlasResult<Vec<ClmTemplateClause>>;
    async fn create_contract(&self, org_id: Uuid, contract_number: &str, title: &str, description: Option<&str>, contract_type_id: Option<Uuid>, template_id: Option<Uuid>, contract_category: &str, currency: &str, total_value: &str, start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>, priority: &str, renewal_type: &str, auto_renew_months: Option<i32>, renewal_notice_days: i32, created_by: Option<Uuid>) -> AtlasResult<ClmContract>;
    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ClmContract>>;
    async fn get_contract_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ClmContract>>;
    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, category: Option<&str>) -> AtlasResult<Vec<ClmContract>>;
    async fn update_contract_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ClmContract>;
    async fn delete_contract(&self, org_id: Uuid, number: &str) -> AtlasResult<()>;
    async fn add_contract_party(&self, org_id: Uuid, contract_id: Uuid, party_type: &str, party_role: &str, party_name: &str, contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>, entity_reference: Option<&str>, is_primary: bool) -> AtlasResult<ClmContractParty>;
    async fn list_contract_parties(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractParty>>;
    async fn remove_contract_party(&self, id: Uuid) -> AtlasResult<()>;
    async fn add_contract_clause(&self, org_id: Uuid, contract_id: Uuid, clause_id: Option<Uuid>, section: Option<&str>, title: &str, body: &str, clause_type: &str, display_order: i32, original_body: Option<String>) -> AtlasResult<ClmContractClause>;
    async fn list_contract_clauses(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractClause>>;
    async fn remove_contract_clause(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_milestone(&self, org_id: Uuid, contract_id: Uuid, name: &str, description: Option<&str>, milestone_type: &str, due_date: Option<chrono::NaiveDate>, amount: Option<&str>, currency: &str) -> AtlasResult<ClmMilestone>;
    async fn get_milestone(&self, id: Uuid) -> AtlasResult<Option<ClmMilestone>>;
    async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmMilestone>>;
    async fn update_milestone_status(&self, id: Uuid, status: &str) -> AtlasResult<ClmMilestone>;
    async fn delete_milestone(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_deliverable(&self, org_id: Uuid, contract_id: Uuid, milestone_id: Option<Uuid>, name: &str, description: Option<&str>, deliverable_type: &str, quantity: &str, unit_of_measure: &str, due_date: Option<chrono::NaiveDate>, amount: Option<&str>, currency: &str) -> AtlasResult<ClmDeliverable>;
    async fn get_deliverable(&self, id: Uuid) -> AtlasResult<Option<ClmDeliverable>>;
    async fn list_deliverables(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmDeliverable>>;
    async fn update_deliverable_status(&self, id: Uuid, status: &str, accepted_by: Option<Uuid>) -> AtlasResult<ClmDeliverable>;
    async fn delete_deliverable(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_amendment(&self, org_id: Uuid, contract_id: Uuid, amendment_number: &str, title: &str, description: Option<&str>, amendment_type: &str, previous_value: Option<&str>, new_value: Option<&str>, effective_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<ClmAmendment>;
    async fn get_amendment(&self, id: Uuid) -> AtlasResult<Option<ClmAmendment>>;
    async fn list_amendments(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmAmendment>>;
    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ClmAmendment>;
    async fn delete_amendment(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_risk(&self, org_id: Uuid, contract_id: Uuid, risk_category: &str, risk_description: &str, probability: &str, impact: &str, mitigation_strategy: Option<&str>, assessed_by: Option<Uuid>) -> AtlasResult<ClmRisk>;
    async fn list_risks(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmRisk>>;
    async fn delete_risk(&self, id: Uuid) -> AtlasResult<()>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ClmDashboard>;
}

// Row mappers
fn row_to_contract_type(row: &sqlx::postgres::PgRow) -> ClmContractType {
    ClmContractType { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), code: row.try_get("code").unwrap_or_default(), name: row.try_get("name").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), contract_category: row.try_get("contract_category").unwrap_or_default(), default_duration_days: row.try_get("default_duration_days").unwrap_or_default(), requires_approval: row.try_get("requires_approval").unwrap_or(true), is_auto_renew: row.try_get("is_auto_renew").unwrap_or(false), risk_scoring_enabled: row.try_get("risk_scoring_enabled").unwrap_or(false), status: row.try_get("status").unwrap_or_else(|_| "active".to_string()), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_by: row.try_get("created_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_clause(row: &sqlx::postgres::PgRow) -> ClmClause {
    ClmClause { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), code: row.try_get("code").unwrap_or_default(), title: row.try_get("title").unwrap_or_default(), body: row.try_get("body").unwrap_or_default(), clause_type: row.try_get("clause_type").unwrap_or_else(|_| "standard".to_string()), clause_category: row.try_get("clause_category").unwrap_or_else(|_| "general".to_string()), applicability: row.try_get("applicability").unwrap_or_else(|_| "all".to_string()), is_locked: row.try_get("is_locked").unwrap_or(false), version: row.try_get("version").unwrap_or(1), status: row.try_get("status").unwrap_or_else(|_| "active".to_string()), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_by: row.try_get("created_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_template(row: &sqlx::postgres::PgRow) -> ClmTemplate {
    ClmTemplate { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), code: row.try_get("code").unwrap_or_default(), name: row.try_get("name").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), contract_type_id: row.try_get("contract_type_id").unwrap_or_default(), default_currency: row.try_get("default_currency").unwrap_or_else(|_| "USD".to_string()), default_duration_days: row.try_get("default_duration_days").unwrap_or_default(), terms_and_conditions: row.try_get("terms_and_conditions").unwrap_or_default(), is_standard: row.try_get("is_standard").unwrap_or(false), status: row.try_get("status").unwrap_or_else(|_| "active".to_string()), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_by: row.try_get("created_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_template_clause(row: &sqlx::postgres::PgRow) -> ClmTemplateClause {
    ClmTemplateClause { id: row.try_get("id").unwrap_or_default(), template_id: row.try_get("template_id").unwrap_or_default(), clause_id: row.try_get("clause_id").unwrap_or_default(), section: row.try_get("section").unwrap_or_default(), display_order: row.try_get("display_order").unwrap_or(0), is_required: row.try_get("is_required").unwrap_or(true), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_contract(row: &sqlx::postgres::PgRow) -> ClmContract {
    ClmContract { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_number: row.try_get("contract_number").unwrap_or_default(), title: row.try_get("title").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), contract_type_id: row.try_get("contract_type_id").unwrap_or_default(), template_id: row.try_get("template_id").unwrap_or_default(), contract_category: row.try_get("contract_category").unwrap_or_else(|_| "general".to_string()), currency: row.try_get("currency").unwrap_or_else(|_| "USD".to_string()), total_value: row.try_get::<String, _>("total_value").unwrap_or_else(|_| "0".to_string()), start_date: row.try_get("start_date").unwrap_or_default(), end_date: row.try_get("end_date").unwrap_or_default(), status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()), priority: row.try_get("priority").unwrap_or_else(|_| "normal".to_string()), risk_score: row.try_get("risk_score").unwrap_or_default(), risk_level: row.try_get("risk_level").unwrap_or_default(), parent_contract_id: row.try_get("parent_contract_id").unwrap_or_default(), renewal_type: row.try_get("renewal_type").unwrap_or_else(|_| "none".to_string()), auto_renew_months: row.try_get("auto_renew_months").unwrap_or_default(), renewal_notice_days: row.try_get("renewal_notice_days").unwrap_or(30), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_by: row.try_get("created_by").unwrap_or_default(), approved_by: row.try_get("approved_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_party(row: &sqlx::postgres::PgRow) -> ClmContractParty {
    ClmContractParty { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), party_type: row.try_get("party_type").unwrap_or_else(|_| "external".to_string()), party_role: row.try_get("party_role").unwrap_or_else(|_| "counterparty".to_string()), party_name: row.try_get("party_name").unwrap_or_default(), contact_name: row.try_get("contact_name").unwrap_or_default(), contact_email: row.try_get("contact_email").unwrap_or_default(), contact_phone: row.try_get("contact_phone").unwrap_or_default(), entity_reference: row.try_get("entity_reference").unwrap_or_default(), is_primary: row.try_get("is_primary").unwrap_or(false), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_contract_clause(row: &sqlx::postgres::PgRow) -> ClmContractClause {
    ClmContractClause { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), clause_id: row.try_get("clause_id").unwrap_or_default(), section: row.try_get("section").unwrap_or_default(), title: row.try_get("title").unwrap_or_default(), body: row.try_get("body").unwrap_or_default(), clause_type: row.try_get("clause_type").unwrap_or_else(|_| "standard".to_string()), display_order: row.try_get("display_order").unwrap_or(0), is_modified: row.try_get("is_modified").unwrap_or(false), original_body: row.try_get("original_body").unwrap_or_default(), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_milestone(row: &sqlx::postgres::PgRow) -> ClmMilestone {
    ClmMilestone { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), name: row.try_get("name").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), milestone_type: row.try_get("milestone_type").unwrap_or_else(|_| "event".to_string()), due_date: row.try_get("due_date").unwrap_or_default(), completed_date: row.try_get("completed_date").unwrap_or_default(), amount: Some(row.try_get::<String, _>("amount").unwrap_or_default()), currency: row.try_get("currency").unwrap_or_else(|_| "USD".to_string()), status: row.try_get("status").unwrap_or_else(|_| "pending".to_string()), responsible_party_id: row.try_get("responsible_party_id").unwrap_or_default(), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_deliverable(row: &sqlx::postgres::PgRow) -> ClmDeliverable {
    ClmDeliverable { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), milestone_id: row.try_get("milestone_id").unwrap_or_default(), name: row.try_get("name").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), deliverable_type: row.try_get("deliverable_type").unwrap_or_else(|_| "document".to_string()), quantity: row.try_get::<String, _>("quantity").unwrap_or_else(|_| "1".to_string()), unit_of_measure: row.try_get("unit_of_measure").unwrap_or_else(|_| "each".to_string()), due_date: row.try_get("due_date").unwrap_or_default(), completed_date: row.try_get("completed_date").unwrap_or_default(), acceptance_date: row.try_get("acceptance_date").unwrap_or_default(), amount: Some(row.try_get::<String, _>("amount").unwrap_or_default()), currency: row.try_get("currency").unwrap_or_else(|_| "USD".to_string()), status: row.try_get("status").unwrap_or_else(|_| "pending".to_string()), accepted_by: row.try_get("accepted_by").unwrap_or_default(), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_amendment(row: &sqlx::postgres::PgRow) -> ClmAmendment {
    ClmAmendment { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), amendment_number: row.try_get("amendment_number").unwrap_or_default(), title: row.try_get("title").unwrap_or_default(), description: row.try_get("description").unwrap_or_default(), amendment_type: row.try_get("amendment_type").unwrap_or_else(|_| "modification".to_string()), previous_value: row.try_get("previous_value").unwrap_or_default(), new_value: row.try_get("new_value").unwrap_or_default(), effective_date: row.try_get("effective_date").unwrap_or_default(), status: row.try_get("status").unwrap_or_else(|_| "draft".to_string()), approved_by: row.try_get("approved_by").unwrap_or_default(), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), created_by: row.try_get("created_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}
fn row_to_risk(row: &sqlx::postgres::PgRow) -> ClmRisk {
    ClmRisk { id: row.try_get("id").unwrap_or_default(), organization_id: row.try_get("organization_id").unwrap_or_default(), contract_id: row.try_get("contract_id").unwrap_or_default(), risk_category: row.try_get("risk_category").unwrap_or_default(), risk_description: row.try_get("risk_description").unwrap_or_default(), probability: row.try_get("probability").unwrap_or_else(|_| "medium".to_string()), impact: row.try_get("impact").unwrap_or_else(|_| "medium".to_string()), mitigation_strategy: row.try_get("mitigation_strategy").unwrap_or_default(), residual_risk: row.try_get("residual_risk").unwrap_or_default(), owner_id: row.try_get("owner_id").unwrap_or_default(), status: row.try_get("status").unwrap_or_else(|_| "identified".to_string()), metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})), assessed_by: row.try_get("assessed_by").unwrap_or_default(), created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()), updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()) }
}

#[allow(dead_code)]
pub struct PostgresContractLifecycleRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresContractLifecycleRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

fn check_del(r: sqlx::postgres::PgQueryResult, l: &str, id: &str) -> AtlasResult<()> {
    if r.rows_affected() == 0 { Err(AtlasError::EntityNotFound(format!("{} '{}' not found", l, id))) } else { Ok(()) }
}

#[async_trait]
impl ContractLifecycleRepository for PostgresContractLifecycleRepository {
    async fn create_contract_type(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, contract_category: &str, default_duration_days: Option<i32>, requires_approval: bool, is_auto_renew: bool, risk_scoring_enabled: bool, created_by: Option<Uuid>) -> AtlasResult<ClmContractType> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_types (organization_id,code,name,description,contract_category,default_duration_days,requires_approval,is_auto_renew,risk_scoring_enabled,metadata,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'{}'::jsonb,$10) RETURNING *").bind(org_id).bind(code).bind(name).bind(description).bind(contract_category).bind(default_duration_days).bind(requires_approval).bind(is_auto_renew).bind(risk_scoring_enabled).bind(created_by).fetch_one(&self.pool).await?;
        Ok(row_to_contract_type(&row))
    }
    async fn get_contract_type(&self, id: Uuid) -> AtlasResult<Option<ClmContractType>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_types WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_contract_type)) }
    async fn get_contract_type_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmContractType>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_types WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).fetch_optional(&self.pool).await?.as_ref().map(row_to_contract_type)) }
    async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ClmContractType>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_types WHERE organization_id=$1 AND status='active' ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await?.iter().map(row_to_contract_type).collect()) }
    async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_types WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).execute(&self.pool).await?, "Contract type", code) }
    async fn create_clause(&self, org_id: Uuid, code: &str, title: &str, body: &str, clause_type: &str, clause_category: &str, applicability: &str, is_locked: bool, created_by: Option<Uuid>) -> AtlasResult<ClmClause> {
        let row = sqlx::query("INSERT INTO _atlas.clm_clauses (organization_id,code,title,body,clause_type,clause_category,applicability,is_locked,metadata,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'{}'::jsonb,$9) RETURNING *").bind(org_id).bind(code).bind(title).bind(body).bind(clause_type).bind(clause_category).bind(applicability).bind(is_locked).bind(created_by).fetch_one(&self.pool).await?;
        Ok(row_to_clause(&row))
    }
    async fn get_clause(&self, id: Uuid) -> AtlasResult<Option<ClmClause>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_clauses WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_clause)) }
    async fn get_clause_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmClause>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_clauses WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).fetch_optional(&self.pool).await?.as_ref().map(row_to_clause)) }
    async fn list_clauses(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<ClmClause>> {
        let rows = if let Some(cat) = category { sqlx::query("SELECT * FROM _atlas.clm_clauses WHERE organization_id=$1 AND clause_category=$2 AND status='active' ORDER BY created_at DESC").bind(org_id).bind(cat).fetch_all(&self.pool).await? } else { sqlx::query("SELECT * FROM _atlas.clm_clauses WHERE organization_id=$1 AND status='active' ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await? };
        Ok(rows.iter().map(row_to_clause).collect())
    }
    async fn delete_clause(&self, org_id: Uuid, code: &str) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_clauses WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).execute(&self.pool).await?, "Clause", code) }
    async fn create_template(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, contract_type_id: Option<Uuid>, default_currency: &str, default_duration_days: Option<i32>, terms_and_conditions: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<ClmTemplate> {
        let row = sqlx::query("INSERT INTO _atlas.clm_templates (organization_id,code,name,description,contract_type_id,default_currency,default_duration_days,terms_and_conditions,metadata,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'{}'::jsonb,$9) RETURNING *").bind(org_id).bind(code).bind(name).bind(description).bind(contract_type_id).bind(default_currency).bind(default_duration_days).bind(terms_and_conditions).bind(created_by).fetch_one(&self.pool).await?;
        Ok(row_to_template(&row))
    }
    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ClmTemplate>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_templates WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_template)) }
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ClmTemplate>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_templates WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).fetch_optional(&self.pool).await?.as_ref().map(row_to_template)) }
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<ClmTemplate>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_templates WHERE organization_id=$1 AND status='active' ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await?.iter().map(row_to_template).collect()) }
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_templates WHERE organization_id=$1 AND code=$2").bind(org_id).bind(code).execute(&self.pool).await?, "Template", code) }
    async fn add_template_clause(&self, template_id: Uuid, clause_id: Uuid, section: Option<&str>, display_order: i32, is_required: bool) -> AtlasResult<ClmTemplateClause> {
        let row = sqlx::query("INSERT INTO _atlas.clm_template_clauses (template_id,clause_id,section,display_order,is_required) VALUES ($1,$2,$3,$4,$5) RETURNING *").bind(template_id).bind(clause_id).bind(section).bind(display_order).bind(is_required).fetch_one(&self.pool).await?;
        Ok(row_to_template_clause(&row))
    }
    async fn list_template_clauses(&self, template_id: Uuid) -> AtlasResult<Vec<ClmTemplateClause>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_template_clauses WHERE template_id=$1 ORDER BY display_order").bind(template_id).fetch_all(&self.pool).await?.iter().map(row_to_template_clause).collect()) }
    async fn create_contract(&self, org_id: Uuid, contract_number: &str, title: &str, description: Option<&str>, contract_type_id: Option<Uuid>, template_id: Option<Uuid>, contract_category: &str, currency: &str, total_value: &str, start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>, priority: &str, renewal_type: &str, auto_renew_months: Option<i32>, renewal_notice_days: i32, created_by: Option<Uuid>) -> AtlasResult<ClmContract> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contracts (organization_id,contract_number,title,description,contract_type_id,template_id,contract_category,currency,total_value,start_date,end_date,priority,renewal_type,auto_renew_months,renewal_notice_days,metadata,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,'{}'::jsonb,$16) RETURNING *").bind(org_id).bind(contract_number).bind(title).bind(description).bind(contract_type_id).bind(template_id).bind(contract_category).bind(currency).bind(total_value).bind(start_date).bind(end_date).bind(priority).bind(renewal_type).bind(auto_renew_months).bind(renewal_notice_days).bind(created_by).fetch_one(&self.pool).await?;
        Ok(row_to_contract(&row))
    }
    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ClmContract>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_contract)) }
    async fn get_contract_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ClmContract>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE organization_id=$1 AND contract_number=$2").bind(org_id).bind(number).fetch_optional(&self.pool).await?.as_ref().map(row_to_contract)) }
    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, category: Option<&str>) -> AtlasResult<Vec<ClmContract>> {
        let rows = match (status, category) {
            (Some(s), Some(c)) => sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE organization_id=$1 AND status=$2 AND contract_category=$3 ORDER BY created_at DESC").bind(org_id).bind(s).bind(c).fetch_all(&self.pool).await?,
            (Some(s), None) => sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE organization_id=$1 AND status=$2 ORDER BY created_at DESC").bind(org_id).bind(s).fetch_all(&self.pool).await?,
            (None, Some(c)) => sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE organization_id=$1 AND contract_category=$2 ORDER BY created_at DESC").bind(org_id).bind(c).fetch_all(&self.pool).await?,
            _ => sqlx::query("SELECT * FROM _atlas.clm_contracts WHERE organization_id=$1 ORDER BY created_at DESC").bind(org_id).fetch_all(&self.pool).await?,
        };
        Ok(rows.iter().map(row_to_contract).collect())
    }
    async fn update_contract_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ClmContract> {
        let row = sqlx::query("UPDATE _atlas.clm_contracts SET status=$2, approved_by=COALESCE($3,approved_by), updated_at=now() WHERE id=$1 RETURNING *").bind(id).bind(status).bind(approved_by).fetch_one(&self.pool).await?;
        Ok(row_to_contract(&row))
    }
    async fn delete_contract(&self, org_id: Uuid, number: &str) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contracts WHERE organization_id=$1 AND contract_number=$2").bind(org_id).bind(number).execute(&self.pool).await?, "Contract", number) }
    async fn add_contract_party(&self, org_id: Uuid, contract_id: Uuid, party_type: &str, party_role: &str, party_name: &str, contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>, entity_reference: Option<&str>, is_primary: bool) -> AtlasResult<ClmContractParty> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_parties (organization_id,contract_id,party_type,party_role,party_name,contact_name,contact_email,contact_phone,entity_reference,is_primary,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'{}'::jsonb) RETURNING *").bind(org_id).bind(contract_id).bind(party_type).bind(party_role).bind(party_name).bind(contact_name).bind(contact_email).bind(contact_phone).bind(entity_reference).bind(is_primary).fetch_one(&self.pool).await?;
        Ok(row_to_party(&row))
    }
    async fn list_contract_parties(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractParty>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_parties WHERE contract_id=$1 ORDER BY created_at").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_party).collect()) }
    async fn remove_contract_party(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_parties WHERE id=$1").bind(id).execute(&self.pool).await?, "Party", &id.to_string()) }
    async fn add_contract_clause(&self, org_id: Uuid, contract_id: Uuid, clause_id: Option<Uuid>, section: Option<&str>, title: &str, body: &str, clause_type: &str, display_order: i32, original_body: Option<String>) -> AtlasResult<ClmContractClause> {
        let is_modified = original_body.is_some();
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_clauses (organization_id,contract_id,clause_id,section,title,body,clause_type,display_order,is_modified,original_body,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'{}'::jsonb) RETURNING *").bind(org_id).bind(contract_id).bind(clause_id).bind(section).bind(title).bind(body).bind(clause_type).bind(display_order).bind(is_modified).bind(original_body).fetch_one(&self.pool).await?;
        Ok(row_to_contract_clause(&row))
    }
    async fn list_contract_clauses(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractClause>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_clauses WHERE contract_id=$1 ORDER BY display_order").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_contract_clause).collect()) }
    async fn remove_contract_clause(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_clauses WHERE id=$1").bind(id).execute(&self.pool).await?, "Clause", &id.to_string()) }
    async fn create_milestone(&self, org_id: Uuid, contract_id: Uuid, name: &str, description: Option<&str>, milestone_type: &str, due_date: Option<chrono::NaiveDate>, amount: Option<&str>, currency: &str) -> AtlasResult<ClmMilestone> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_milestones (organization_id,contract_id,name,description,milestone_type,due_date,amount,currency,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'{}'::jsonb) RETURNING *").bind(org_id).bind(contract_id).bind(name).bind(description).bind(milestone_type).bind(due_date).bind(amount).bind(currency).fetch_one(&self.pool).await?;
        Ok(row_to_milestone(&row))
    }
    async fn get_milestone(&self, id: Uuid) -> AtlasResult<Option<ClmMilestone>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_milestones WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_milestone)) }
    async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmMilestone>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_milestones WHERE contract_id=$1 ORDER BY due_date NULLS LAST").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_milestone).collect()) }
    async fn update_milestone_status(&self, id: Uuid, status: &str) -> AtlasResult<ClmMilestone> {
        let completed = if status == "completed" { Some(chrono::Utc::now().date_naive()) } else { None };
        let row = sqlx::query("UPDATE _atlas.clm_contract_milestones SET status=$2, completed_date=COALESCE($3,completed_date), updated_at=now() WHERE id=$1 RETURNING *").bind(id).bind(status).bind(completed).fetch_one(&self.pool).await?;
        Ok(row_to_milestone(&row))
    }
    async fn delete_milestone(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_milestones WHERE id=$1").bind(id).execute(&self.pool).await?, "Milestone", &id.to_string()) }
    async fn create_deliverable(&self, org_id: Uuid, contract_id: Uuid, milestone_id: Option<Uuid>, name: &str, description: Option<&str>, deliverable_type: &str, quantity: &str, unit_of_measure: &str, due_date: Option<chrono::NaiveDate>, amount: Option<&str>, currency: &str) -> AtlasResult<ClmDeliverable> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_deliverables (organization_id,contract_id,milestone_id,name,description,deliverable_type,quantity,unit_of_measure,due_date,amount,currency,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'{}'::jsonb) RETURNING *").bind(org_id).bind(contract_id).bind(milestone_id).bind(name).bind(description).bind(deliverable_type).bind(quantity).bind(unit_of_measure).bind(due_date).bind(amount).bind(currency).fetch_one(&self.pool).await?;
        Ok(row_to_deliverable(&row))
    }
    async fn get_deliverable(&self, id: Uuid) -> AtlasResult<Option<ClmDeliverable>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_deliverables WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_deliverable)) }
    async fn list_deliverables(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmDeliverable>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_deliverables WHERE contract_id=$1 ORDER BY due_date NULLS LAST").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_deliverable).collect()) }
    async fn update_deliverable_status(&self, id: Uuid, status: &str, accepted_by: Option<Uuid>) -> AtlasResult<ClmDeliverable> {
        let acceptance = if status == "accepted" { Some(chrono::Utc::now().date_naive()) } else { None };
        let row = sqlx::query("UPDATE _atlas.clm_contract_deliverables SET status=$2, accepted_by=$3, acceptance_date=$4, updated_at=now() WHERE id=$1 RETURNING *").bind(id).bind(status).bind(accepted_by).bind(acceptance).fetch_one(&self.pool).await?;
        Ok(row_to_deliverable(&row))
    }
    async fn delete_deliverable(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_deliverables WHERE id=$1").bind(id).execute(&self.pool).await?, "Deliverable", &id.to_string()) }
    async fn create_amendment(&self, org_id: Uuid, contract_id: Uuid, amendment_number: &str, title: &str, description: Option<&str>, amendment_type: &str, previous_value: Option<&str>, new_value: Option<&str>, effective_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<ClmAmendment> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_amendments (organization_id,contract_id,amendment_number,title,description,amendment_type,previous_value,new_value,effective_date,metadata,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'{}'::jsonb,$10) RETURNING *").bind(org_id).bind(contract_id).bind(amendment_number).bind(title).bind(description).bind(amendment_type).bind(previous_value).bind(new_value).bind(effective_date).bind(created_by).fetch_one(&self.pool).await?;
        Ok(row_to_amendment(&row))
    }
    async fn get_amendment(&self, id: Uuid) -> AtlasResult<Option<ClmAmendment>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_amendments WHERE id=$1").bind(id).fetch_optional(&self.pool).await?.as_ref().map(row_to_amendment)) }
    async fn list_amendments(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmAmendment>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_amendments WHERE contract_id=$1 ORDER BY created_at DESC").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_amendment).collect()) }
    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<ClmAmendment> {
        let row = sqlx::query("UPDATE _atlas.clm_contract_amendments SET status=$2, approved_by=$3, updated_at=now() WHERE id=$1 RETURNING *").bind(id).bind(status).bind(approved_by).fetch_one(&self.pool).await?;
        Ok(row_to_amendment(&row))
    }
    async fn delete_amendment(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_amendments WHERE id=$1").bind(id).execute(&self.pool).await?, "Amendment", &id.to_string()) }
    async fn create_risk(&self, org_id: Uuid, contract_id: Uuid, risk_category: &str, risk_description: &str, probability: &str, impact: &str, mitigation_strategy: Option<&str>, assessed_by: Option<Uuid>) -> AtlasResult<ClmRisk> {
        let row = sqlx::query("INSERT INTO _atlas.clm_contract_risks (organization_id,contract_id,risk_category,risk_description,probability,impact,mitigation_strategy,metadata,assessed_by) VALUES ($1,$2,$3,$4,$5,$6,$7,'{}'::jsonb,$8) RETURNING *").bind(org_id).bind(contract_id).bind(risk_category).bind(risk_description).bind(probability).bind(impact).bind(mitigation_strategy).bind(assessed_by).fetch_one(&self.pool).await?;
        Ok(row_to_risk(&row))
    }
    async fn list_risks(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmRisk>> { Ok(sqlx::query("SELECT * FROM _atlas.clm_contract_risks WHERE contract_id=$1 ORDER BY created_at DESC").bind(contract_id).fetch_all(&self.pool).await?.iter().map(row_to_risk).collect()) }
    async fn delete_risk(&self, id: Uuid) -> AtlasResult<()> { check_del(sqlx::query("DELETE FROM _atlas.clm_contract_risks WHERE id=$1").bind(id).execute(&self.pool).await?, "Risk", &id.to_string()) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ClmDashboard> {
        let contracts = self.list_contracts(org_id, None, None).await.unwrap_or_default();
        let total = contracts.len() as i32;
        let active = contracts.iter().filter(|c| c.status == "active").count() as i32;
        let draft = contracts.iter().filter(|c| c.status == "draft").count() as i32;
        let high_risk = contracts.iter().filter(|c| c.risk_level.as_deref() == Some("high") || c.risk_level.as_deref() == Some("critical")).count() as i32;
        let total_value: f64 = contracts.iter().filter_map(|c| c.total_value.parse::<f64>().ok()).sum();
        let mut by_status: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        let mut by_category: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for c in &contracts { *by_status.entry(c.status.clone()).or_insert(0) += 1; *by_category.entry(c.contract_category.clone()).or_insert(0) += 1; }
        let today = chrono::Utc::now().date_naive();
        let thirty = today + chrono::Duration::days(30);
        let expiring = contracts.iter().filter(|c| c.end_date.is_some_and(|d| d <= thirty && d >= today && c.status == "active")).count() as i32;
        let recent: Vec<serde_json::Value> = contracts.iter().take(10).map(|c| serde_json::json!({"id":c.id,"contractNumber":c.contract_number,"title":c.title,"status":c.status,"contractCategory":c.contract_category,"totalValue":c.total_value,"currency":c.currency})).collect();
        Ok(ClmDashboard { total_contracts: total, active_contracts: active, draft_contracts: draft, expiring_contracts: expiring, total_contract_value: format!("{:.2}", total_value), contracts_by_category: serde_json::to_value(&by_category).unwrap_or(serde_json::json!({})), contracts_by_status: serde_json::to_value(&by_status).unwrap_or(serde_json::json!({})), high_risk_contracts: high_risk, pending_milestones: 0, pending_deliverables: 0, pending_amendments: 0, recent_contracts: serde_json::json!(recent) })
    }
}
