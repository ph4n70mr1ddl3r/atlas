//! Contract Lifecycle Engine
//!
//! Business logic for Oracle Fusion Enterprise Contracts management.

use atlas_shared::{
    ClmContractType, ClmClause, ClmTemplate, ClmTemplateClause,
    ClmContract, ClmContractParty, ClmContractClause, ClmMilestone,
    ClmDeliverable, ClmAmendment, ClmRisk, ClmDashboard,
    AtlasError, AtlasResult,
};
use super::ContractLifecycleRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ── Valid enum values ──────────────────────────────────────────────────

const VALID_CATEGORIES: &[&str] = &[
    "sales", "procurement", "service", "nda", "partnership", "employment", "general",
];
const VALID_STATUSES: &[&str] = &[
    "draft", "in_review", "pending_approval", "approved", "active",
    "suspended", "expired", "terminated", "completed", "cancelled",
];
const VALID_PRIORITIES: &[&str] = &["low", "normal", "high", "critical"];
const VALID_CLAUSE_TYPES: &[&str] = &["standard", "optional", "mandatory", "boilerplate"];
const VALID_CLAUSE_CATEGORIES: &[&str] = &[
    "indemnification", "confidentiality", "termination", "payment", "liability", "general",
];
const VALID_PARTY_TYPES: &[&str] = &["internal", "external"];
const VALID_PARTY_ROLES: &[&str] = &[
    "initiator", "counterparty", "beneficiary", "guarantor", "approver", "reviewer",
];
const VALID_MILESTONE_TYPES: &[&str] = &[
    "event", "payment", "delivery", "review", "approval", "renewal", "termination",
];
const VALID_DELIVERABLE_TYPES: &[&str] = &[
    "document", "product", "service", "report", "payment",
];
const VALID_AMENDMENT_TYPES: &[&str] = &[
    "modification", "extension", "renewal", "termination", "scope_change",
];
const VALID_RISK_CATEGORIES: &[&str] = &[
    "financial", "legal", "operational", "compliance", "reputational",
];
const VALID_PROBABILITY: &[&str] = &["low", "medium", "high", "very_high"];
const VALID_IMPACT: &[&str] = &["low", "medium", "high", "critical"];
const VALID_RENEWAL_TYPES: &[&str] = &["none", "automatic", "manual", "with_notice"];

/// Contract Lifecycle Management Engine
pub struct ContractLifecycleEngine {
    repository: Arc<dyn ContractLifecycleRepository>,
}

impl ContractLifecycleEngine {
    pub fn new(repository: Arc<dyn ContractLifecycleRepository>) -> Self {
        Self { repository }
    }

    // ══════════════════════════════════════════════════════════════════════
    // Contract Types
    // ══════════════════════════════════════════════════════════════════════

    pub async fn create_contract_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        contract_category: &str,
        default_duration_days: Option<i32>,
        requires_approval: bool,
        is_auto_renew: bool,
        risk_scoring_enabled: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ClmContractType> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper, "Contract type code")?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Contract type name is required".into()));
        }
        validate_enum(contract_category, VALID_CATEGORIES, "contract_category")?;

        if self.repository.get_contract_type_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Contract type '{}' already exists", code_upper)));
        }

        info!(code = %code_upper, "Creating contract type");
        self.repository.create_contract_type(
            org_id, &code_upper, name, description, contract_category,
            default_duration_days, requires_approval, is_auto_renew,
            risk_scoring_enabled, created_by,
        ).await
    }

    pub async fn get_contract_type(&self, id: Uuid) -> AtlasResult<Option<ClmContractType>> {
        self.repository.get_contract_type(id).await
    }

    pub async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ClmContractType>> {
        self.repository.list_contract_types(org_id).await
    }

    pub async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!(code, "Deleting contract type");
        self.repository.delete_contract_type(org_id, code).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Clause Library
    // ══════════════════════════════════════════════════════════════════════

    pub async fn create_clause(
        &self,
        org_id: Uuid,
        code: &str,
        title: &str,
        body: &str,
        clause_type: &str,
        clause_category: &str,
        applicability: &str,
        is_locked: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ClmClause> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper, "Clause code")?;
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Clause title is required".into()));
        }
        if body.is_empty() {
            return Err(AtlasError::ValidationFailed("Clause body is required".into()));
        }
        validate_enum(clause_type, VALID_CLAUSE_TYPES, "clause_type")?;
        validate_enum(clause_category, VALID_CLAUSE_CATEGORIES, "clause_category")?;

        if self.repository.get_clause_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Clause '{}' already exists", code_upper)));
        }

        info!(code = %code_upper, "Creating clause");
        self.repository.create_clause(
            org_id, &code_upper, title, body, clause_type,
            clause_category, applicability, is_locked, created_by,
        ).await
    }

    pub async fn get_clause(&self, id: Uuid) -> AtlasResult<Option<ClmClause>> {
        self.repository.get_clause(id).await
    }

    pub async fn list_clauses(&self, org_id: Uuid, category: Option<&str>) -> AtlasResult<Vec<ClmClause>> {
        self.repository.list_clauses(org_id, category).await
    }

    pub async fn delete_clause(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!(code, "Deleting clause");
        self.repository.delete_clause(org_id, code).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Templates
    // ══════════════════════════════════════════════════════════════════════

    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        contract_type_id: Option<Uuid>,
        default_currency: &str,
        default_duration_days: Option<i32>,
        terms_and_conditions: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ClmTemplate> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper, "Template code")?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".into()));
        }

        if self.repository.get_template_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Template '{}' already exists", code_upper)));
        }

        info!(code = %code_upper, "Creating contract template");
        self.repository.create_template(
            org_id, &code_upper, name, description, contract_type_id,
            default_currency, default_duration_days, terms_and_conditions, created_by,
        ).await
    }

    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<ClmTemplate>> {
        self.repository.get_template(id).await
    }

    pub async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<ClmTemplate>> {
        self.repository.list_templates(org_id).await
    }

    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!(code, "Deleting template");
        self.repository.delete_template(org_id, code).await
    }

    pub async fn add_template_clause(
        &self,
        template_id: Uuid,
        clause_id: Uuid,
        section: Option<&str>,
        display_order: i32,
        is_required: bool,
    ) -> AtlasResult<ClmTemplateClause> {
        let _tmpl = self.repository.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Template {} not found", template_id)))?;
        let _clause = self.repository.get_clause(clause_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Clause {} not found", clause_id)))?;

        self.repository.add_template_clause(template_id, clause_id, section, display_order, is_required).await
    }

    pub async fn list_template_clauses(&self, template_id: Uuid) -> AtlasResult<Vec<ClmTemplateClause>> {
        self.repository.list_template_clauses(template_id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Contracts
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn create_contract(
        &self,
        org_id: Uuid,
        contract_number: &str,
        title: &str,
        description: Option<&str>,
        contract_type_id: Option<Uuid>,
        template_id: Option<Uuid>,
        contract_category: &str,
        currency: &str,
        total_value: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        priority: &str,
        renewal_type: &str,
        auto_renew_months: Option<i32>,
        renewal_notice_days: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ClmContract> {
        let cn_upper = contract_number.to_uppercase();
        validate_code(&cn_upper, "Contract number")?;
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Contract title is required".into()));
        }
        validate_enum(contract_category, VALID_CATEGORIES, "contract_category")?;
        validate_enum(priority, VALID_PRIORITIES, "priority")?;
        validate_enum(renewal_type, VALID_RENEWAL_TYPES, "renewal_type")?;
        if total_value.parse::<f64>().is_err() {
            return Err(AtlasError::ValidationFailed("Total value must be a valid number".into()));
        }

        if self.repository.get_contract_by_number(org_id, &cn_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Contract '{}' already exists", cn_upper)));
        }

        info!(number = %cn_upper, "Creating contract");
        self.repository.create_contract(
            org_id, &cn_upper, title, description, contract_type_id, template_id,
            contract_category, currency, total_value, start_date, end_date,
            priority, renewal_type, auto_renew_months, renewal_notice_days, created_by,
        ).await
    }

    pub async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ClmContract>> {
        self.repository.get_contract(id).await
    }

    pub async fn get_contract_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ClmContract>> {
        self.repository.get_contract_by_number(org_id, number).await
    }

    pub async fn list_contracts(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        category: Option<&str>,
    ) -> AtlasResult<Vec<ClmContract>> {
        self.repository.list_contracts(org_id, status, category).await
    }

    pub async fn transition_contract(&self, id: Uuid, new_status: &str, user_id: Option<Uuid>) -> AtlasResult<ClmContract> {
        validate_enum(new_status, VALID_STATUSES, "status")?;

        let contract = self.repository.get_contract(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", id)))?;

        validate_transition(&contract.status, new_status)?;

        info!(id = %id, from = %contract.status, to = %new_status, "Transitioning contract");

        self.repository.update_contract_status(id, new_status, user_id).await
    }

    pub async fn delete_contract(&self, org_id: Uuid, number: &str) -> AtlasResult<()> {
        info!(number, "Deleting contract");
        self.repository.delete_contract(org_id, number).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Contract Parties
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn add_contract_party(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        party_type: &str,
        party_role: &str,
        party_name: &str,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        entity_reference: Option<&str>,
        is_primary: bool,
    ) -> AtlasResult<ClmContractParty> {
        validate_enum(party_type, VALID_PARTY_TYPES, "party_type")?;
        validate_enum(party_role, VALID_PARTY_ROLES, "party_role")?;
        if party_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Party name is required".into()));
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        self.repository.add_contract_party(
            org_id, contract_id, party_type, party_role, party_name,
            contact_name, contact_email, contact_phone, entity_reference, is_primary,
        ).await
    }

    pub async fn list_contract_parties(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractParty>> {
        self.repository.list_contract_parties(contract_id).await
    }

    pub async fn remove_contract_party(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_contract_party(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Contract Clauses
    // ══════════════════════════════════════════════════════════════════════

    pub async fn add_contract_clause(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        clause_id: Option<Uuid>,
        section: Option<&str>,
        title: &str,
        body: &str,
        clause_type: &str,
        display_order: i32,
    ) -> AtlasResult<ClmContractClause> {
        validate_enum(clause_type, VALID_CLAUSE_TYPES, "clause_type")?;
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Clause title is required".into()));
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        let original_body = if let Some(cid) = clause_id {
            let clause = self.repository.get_clause(cid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Clause {} not found", cid)))?;
            if clause.is_locked && clause.body != body {
                return Err(AtlasError::ValidationFailed(
                    format!("Clause '{}' is locked and cannot be modified", clause.code)
                ));
            }
            if clause.body != body { Some(clause.body.clone()) } else { None }
        } else {
            None
        };

        self.repository.add_contract_clause(
            org_id, contract_id, clause_id, section, title, body,
            clause_type, display_order, original_body,
        ).await
    }

    pub async fn list_contract_clauses(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmContractClause>> {
        self.repository.list_contract_clauses(contract_id).await
    }

    pub async fn remove_contract_clause(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_contract_clause(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Milestones
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn create_milestone(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        name: &str,
        description: Option<&str>,
        milestone_type: &str,
        due_date: Option<chrono::NaiveDate>,
        amount: Option<&str>,
        currency: &str,
    ) -> AtlasResult<ClmMilestone> {
        validate_enum(milestone_type, VALID_MILESTONE_TYPES, "milestone_type")?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Milestone name is required".into()));
        }
        if let Some(a) = amount {
            if a.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed("Milestone amount must be a valid number".into()));
            }
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        self.repository.create_milestone(
            org_id, contract_id, name, description, milestone_type,
            due_date, amount, currency,
        ).await
    }

    pub async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmMilestone>> {
        self.repository.list_milestones(contract_id).await
    }

    pub async fn complete_milestone(&self, id: Uuid) -> AtlasResult<ClmMilestone> {
        let milestone = self.repository.get_milestone(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Milestone {} not found", id)))?;
        if milestone.status != "pending" && milestone.status != "in_progress" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot complete milestone in '{}' status", milestone.status)
            ));
        }
        self.repository.update_milestone_status(id, "completed").await
    }

    pub async fn delete_milestone(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_milestone(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Deliverables
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn create_deliverable(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        milestone_id: Option<Uuid>,
        name: &str,
        description: Option<&str>,
        deliverable_type: &str,
        quantity: &str,
        unit_of_measure: &str,
        due_date: Option<chrono::NaiveDate>,
        amount: Option<&str>,
        currency: &str,
    ) -> AtlasResult<ClmDeliverable> {
        validate_enum(deliverable_type, VALID_DELIVERABLE_TYPES, "deliverable_type")?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Deliverable name is required".into()));
        }
        if quantity.parse::<f64>().is_err() {
            return Err(AtlasError::ValidationFailed("Quantity must be a valid number".into()));
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        if let Some(mid) = milestone_id {
            let _m = self.repository.get_milestone(mid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Milestone {} not found", mid)))?;
        }

        self.repository.create_deliverable(
            org_id, contract_id, milestone_id, name, description,
            deliverable_type, quantity, unit_of_measure, due_date, amount, currency,
        ).await
    }

    pub async fn list_deliverables(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmDeliverable>> {
        self.repository.list_deliverables(contract_id).await
    }

    pub async fn accept_deliverable(&self, id: Uuid, accepted_by: Option<Uuid>) -> AtlasResult<ClmDeliverable> {
        let d = self.repository.get_deliverable(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deliverable {} not found", id)))?;
        if d.status != "submitted" && d.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot accept deliverable in '{}' status", d.status)
            ));
        }
        self.repository.update_deliverable_status(id, "accepted", accepted_by).await
    }

    pub async fn reject_deliverable(&self, id: Uuid) -> AtlasResult<ClmDeliverable> {
        let d = self.repository.get_deliverable(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Deliverable {} not found", id)))?;
        if d.status != "submitted" && d.status != "under_review" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot reject deliverable in '{}' status", d.status)
            ));
        }
        self.repository.update_deliverable_status(id, "rejected", None).await
    }

    pub async fn delete_deliverable(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_deliverable(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Amendments
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn create_amendment(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        amendment_number: &str,
        title: &str,
        description: Option<&str>,
        amendment_type: &str,
        previous_value: Option<&str>,
        new_value: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ClmAmendment> {
        validate_enum(amendment_type, VALID_AMENDMENT_TYPES, "amendment_type")?;
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Amendment title is required".into()));
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        info!(number = amendment_number, "Creating amendment");
        self.repository.create_amendment(
            org_id, contract_id, amendment_number, title, description,
            amendment_type, previous_value, new_value, effective_date, created_by,
        ).await
    }

    pub async fn list_amendments(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmAmendment>> {
        self.repository.list_amendments(contract_id).await
    }

    pub async fn approve_amendment(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ClmAmendment> {
        let a = self.repository.get_amendment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Amendment {} not found", id)))?;
        if a.status != "pending_approval" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot approve amendment in '{}' status", a.status)
            ));
        }
        self.repository.update_amendment_status(id, "approved", approved_by).await
    }

    pub async fn reject_amendment(&self, id: Uuid) -> AtlasResult<ClmAmendment> {
        let a = self.repository.get_amendment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Amendment {} not found", id)))?;
        if a.status != "pending_approval" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot reject amendment in '{}' status", a.status)
            ));
        }
        self.repository.update_amendment_status(id, "rejected", None).await
    }

    pub async fn delete_amendment(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_amendment(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Risk Assessments
    // ══════════════════════════════════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub async fn create_risk(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        risk_category: &str,
        risk_description: &str,
        probability: &str,
        impact: &str,
        mitigation_strategy: Option<&str>,
        assessed_by: Option<Uuid>,
    ) -> AtlasResult<ClmRisk> {
        validate_enum(risk_category, VALID_RISK_CATEGORIES, "risk_category")?;
        validate_enum(probability, VALID_PROBABILITY, "probability")?;
        validate_enum(impact, VALID_IMPACT, "impact")?;
        if risk_description.is_empty() {
            return Err(AtlasError::ValidationFailed("Risk description is required".into()));
        }

        let _contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Contract {} not found", contract_id)))?;

        self.repository.create_risk(
            org_id, contract_id, risk_category, risk_description,
            probability, impact, mitigation_strategy, assessed_by,
        ).await
    }

    pub async fn list_risks(&self, contract_id: Uuid) -> AtlasResult<Vec<ClmRisk>> {
        self.repository.list_risks(contract_id).await
    }

    pub async fn delete_risk(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_risk(id).await
    }

    // ══════════════════════════════════════════════════════════════════════
    // Dashboard
    // ══════════════════════════════════════════════════════════════════════

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ClmDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ── Validation helpers ─────────────────────────────────────────────────

fn validate_code(code: &str, label: &str) -> AtlasResult<()> {
    if code.is_empty() || code.len() > 100 {
        return Err(AtlasError::ValidationFailed(
            format!("{} must be 1-100 characters", label)
        ));
    }
    Ok(())
}

fn validate_enum(value: &str, valid: &[&str], field: &str) -> AtlasResult<()> {
    if !valid.contains(&value) {
        return Err(AtlasError::ValidationFailed(
            format!("Invalid {} '{}'. Must be one of: {}", field, value, valid.join(", "))
        ));
    }
    Ok(())
}

fn validate_transition(from: &str, to: &str) -> AtlasResult<()> {
    let allowed: &[&[&str]] = &[
        &["draft", "in_review"],
        &["draft", "cancelled"],
        &["in_review", "pending_approval"],
        &["in_review", "draft"],
        &["pending_approval", "approved"],
        &["pending_approval", "cancelled"],
        &["approved", "active"],
        &["active", "suspended"],
        &["active", "completed"],
        &["active", "terminated"],
        &["active", "expired"],
        &["suspended", "active"],
        &["suspended", "terminated"],
    ];
    if !allowed.iter().any(|pair| pair[0] == from && pair[1] == to) {
        return Err(AtlasError::InvalidStateTransition(from.to_string(), to.to_string()));
    }
    Ok(())
}
