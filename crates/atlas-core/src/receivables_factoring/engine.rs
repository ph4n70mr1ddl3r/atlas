//! Receivables Factoring Engine
//!
//! Manages the full lifecycle of receivables factoring:
//! - Factor Company CRUD with activate/deactivate
//! - Factoring Agreement lifecycle: draft → active → suspended → terminated
//! - Factoring Request workflow: draft → submitted → approved → funded → settled
//! - Request lines for individual receivables
//! - Settlement processing when customers pay the factor
//! - Dashboard with factoring summary and metrics
//!
//! Oracle Fusion equivalent: Financials > Treasury > Receivables Factoring

use super::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Receivables Factoring Engine
pub struct ReceivablesFactoringEngine {
    repository: Arc<dyn ReceivablesFactoringRepository>,
}

impl ReceivablesFactoringEngine {
    pub fn new(repository: Arc<dyn ReceivablesFactoringRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Factor Companies
    // ========================================================================

    /// Create a new factor company
    pub async fn create_factor_company(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        bank_name: Option<&str>,
        bank_account_number: Option<&str>,
        advance_rate: &str,
        fee_rate: &str,
        recourse_type: &str,
        minimum_invoice_amount: Option<&str>,
        maximum_invoice_amount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FactorCompany> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Code is required".into()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Name is required".into()));
        }
        if !VALID_RECOURSE_TYPES.contains(&recourse_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recourse type '{}'. Must be one of: {}", recourse_type, VALID_RECOURSE_TYPES.join(", ")
            )));
        }

        // Validate numeric fields
        advance_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid advance rate".into()))?;
        fee_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid fee rate".into()))?;

        if let Some(min) = minimum_invoice_amount {
            if min.parse::<f64>().unwrap_or(-1.0) < 0.0 {
                return Err(AtlasError::ValidationFailed("Minimum invoice amount must be non-negative".into()));
            }
        }
        if let Some(max) = maximum_invoice_amount {
            if max.parse::<f64>().unwrap_or(-1.0) < 0.0 {
                return Err(AtlasError::ValidationFailed("Maximum invoice amount must be non-negative".into()));
            }
        }

        // Check duplicate code
        if self.repository.get_factor_company_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Factor company code '{}' already exists", code)));
        }

        info!("Creating factor company '{}' for org {}", code, org_id);
        self.repository.create_factor_company(
            org_id, code, name, description,
            contact_name, contact_email, contact_phone,
            bank_name, bank_account_number,
            advance_rate, fee_rate, recourse_type,
            minimum_invoice_amount, maximum_invoice_amount,
            created_by,
        ).await
    }

    /// Get a factor company by ID
    pub async fn get_factor_company(&self, id: Uuid) -> AtlasResult<Option<FactorCompany>> {
        self.repository.get_factor_company(id).await
    }

    /// Get a factor company by code
    pub async fn get_factor_company_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FactorCompany>> {
        self.repository.get_factor_company_by_code(org_id, code).await
    }

    /// List factor companies for an organization
    pub async fn list_factor_companies(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FactorCompany>> {
        self.repository.list_factor_companies(org_id, is_active).await
    }

    /// Deactivate a factor company
    pub async fn deactivate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany> {
        let fc = self.repository.get_factor_company(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Factor company {} not found", id)))?;
        if !fc.is_active {
            return Err(AtlasError::ValidationFailed("Factor company is already inactive".into()));
        }
        info!("Deactivating factor company '{}'", fc.code);
        self.repository.deactivate_factor_company(id).await
    }

    /// Activate a factor company
    pub async fn activate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany> {
        let fc = self.repository.get_factor_company(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Factor company {} not found", id)))?;
        if fc.is_active {
            return Err(AtlasError::ValidationFailed("Factor company is already active".into()));
        }
        info!("Activating factor company '{}'", fc.code);
        self.repository.activate_factor_company(id).await
    }

    // ========================================================================
    // Factoring Agreements
    // ========================================================================

    /// Create a new factoring agreement
    #[allow(clippy::too_many_arguments)]
    pub async fn create_agreement(
        &self,
        org_id: Uuid,
        agreement_number: &str,
        factor_company_id: Uuid,
        agreement_name: &str,
        description: Option<&str>,
        agreement_type: &str,
        recourse_type: &str,
        advance_rate: &str,
        factoring_fee_rate: &str,
        late_fee_rate: &str,
        reserve_rate: &str,
        minimum_fee: &str,
        currency_code: &str,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
        credit_limit: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringAgreement> {
        // Validate required fields
        if agreement_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Agreement number is required".into()));
        }
        if agreement_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Agreement name is required".into()));
        }
        if !VALID_AGREEMENT_TYPES.contains(&agreement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid agreement type '{}'. Must be one of: {}", agreement_type, VALID_AGREEMENT_TYPES.join(", ")
            )));
        }
        if !VALID_RECOURSE_TYPES.contains(&recourse_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recourse type '{}'. Must be one of: {}", recourse_type, VALID_RECOURSE_TYPES.join(", ")
            )));
        }

        // Validate rate fields
        advance_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid advance rate".into()))?;
        factoring_fee_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid factoring fee rate".into()))?;
        late_fee_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid late fee rate".into()))?;
        reserve_rate.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid reserve rate".into()))?;
        minimum_fee.parse::<f64>().map_err(|_| AtlasError::ValidationFailed("Invalid minimum fee".into()))?;

        // Validate date range
        if let Some(end) = end_date {
            if end < start_date {
                return Err(AtlasError::ValidationFailed("End date must be after start date".into()));
            }
        }

        // Verify factor company exists
        let fc = self.repository.get_factor_company(factor_company_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Factor company {} not found", factor_company_id)
            ))?;
        if !fc.is_active {
            return Err(AtlasError::ValidationFailed("Factor company is not active".into()));
        }

        info!("Creating factoring agreement '{}' with factor '{}'", agreement_number, fc.code);
        self.repository.create_agreement(
            org_id, agreement_number, factor_company_id,
            agreement_name, description, agreement_type, recourse_type,
            advance_rate, factoring_fee_rate, late_fee_rate, reserve_rate,
            minimum_fee, currency_code, start_date, end_date,
            credit_limit, created_by,
        ).await
    }

    /// Get an agreement by ID
    pub async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<FactoringAgreement>> {
        self.repository.get_agreement(id).await
    }

    /// List agreements with optional filters
    pub async fn list_agreements(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        factor_company_id: Option<Uuid>,
    ) -> AtlasResult<Vec<FactoringAgreement>> {
        if let Some(s) = status {
            if !VALID_AGREEMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_AGREEMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_agreements(org_id, status, factor_company_id).await
    }

    /// Activate a draft agreement
    pub async fn activate_agreement(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<FactoringAgreement> {
        let a = self.repository.get_agreement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", id)))?;
        if a.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot activate agreement in '{}' status. Must be 'draft'.", a.status)
            ));
        }
        info!("Activating factoring agreement '{}'", a.agreement_number);
        self.repository.update_agreement_status(id, "active", approved_by).await
    }

    /// Suspend an active agreement
    pub async fn suspend_agreement(&self, id: Uuid) -> AtlasResult<FactoringAgreement> {
        let a = self.repository.get_agreement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", id)))?;
        if a.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot suspend agreement in '{}' status. Must be 'active'.", a.status)
            ));
        }
        info!("Suspending factoring agreement '{}'", a.agreement_number);
        self.repository.update_agreement_status(id, "suspended", None).await
    }

    /// Terminate an agreement
    pub async fn terminate_agreement(&self, id: Uuid) -> AtlasResult<FactoringAgreement> {
        let a = self.repository.get_agreement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", id)))?;
        if a.status != "active" && a.status != "suspended" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot terminate agreement in '{}' status. Must be 'active' or 'suspended'.", a.status)
            ));
        }
        info!("Terminating factoring agreement '{}'", a.agreement_number);
        self.repository.update_agreement_status(id, "terminated", None).await
    }

    // ========================================================================
    // Factoring Requests
    // ========================================================================

    /// Create a new factoring request
    pub async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        agreement_id: Uuid,
        request_date: chrono::NaiveDate,
        recourse_type: &str,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringRequest> {
        if request_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Request number is required".into()));
        }
        if !VALID_RECOURSE_TYPES.contains(&recourse_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recourse type '{}'. Must be one of: {}", recourse_type, VALID_RECOURSE_TYPES.join(", ")
            )));
        }

        // Verify agreement exists and is active
        let agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", agreement_id)))?;
        if agreement.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot create requests for non-active agreement".into()
            ));
        }

        info!("Creating factoring request '{}' against agreement '{}'", request_number, agreement.agreement_number);
        self.repository.create_request(
            org_id, request_number, agreement_id, request_date,
            recourse_type, currency_code, notes, created_by,
        ).await
    }

    /// Get a request by ID
    pub async fn get_request(&self, id: Uuid) -> AtlasResult<Option<FactoringRequest>> {
        self.repository.get_request(id).await
    }

    /// List requests with optional filters
    pub async fn list_requests(
        &self,
        org_id: Uuid,
        agreement_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<FactoringRequest>> {
        if let Some(s) = status {
            if !VALID_REQUEST_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_REQUEST_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_requests(org_id, agreement_id, status).await
    }

    /// Submit a draft request for approval
    pub async fn submit_request(&self, id: Uuid) -> AtlasResult<FactoringRequest> {
        let r = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", id)))?;
        if r.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot submit request in '{}' status. Must be 'draft'.", r.status)
            ));
        }
        info!("Submitting factoring request '{}' for approval", r.request_number);
        self.repository.update_request_status(id, "submitted").await
    }

    /// Approve a submitted request
    pub async fn approve_request(&self, id: Uuid) -> AtlasResult<FactoringRequest> {
        let r = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", id)))?;
        if r.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot approve request in '{}' status. Must be 'submitted'.", r.status)
            ));
        }
        info!("Approving factoring request '{}'", r.request_number);
        self.repository.update_request_status(id, "approved").await
    }

    /// Fund an approved request (advance payment from factor)
    pub async fn fund_request(&self, id: Uuid) -> AtlasResult<FactoringRequest> {
        let r = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", id)))?;
        if r.status != "approved" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot fund request in '{}' status. Must be 'approved'.", r.status)
            ));
        }
        info!("Funding factoring request '{}'", r.request_number);
        self.repository.update_request_funding(id, None).await
    }

    /// Settle a funded request
    pub async fn settle_request(&self, id: Uuid) -> AtlasResult<FactoringRequest> {
        let r = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", id)))?;
        if r.status != "funded" && r.status != "partially_settled" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot settle request in '{}' status. Must be 'funded' or 'partially_settled'.", r.status)
            ));
        }
        info!("Settling factoring request '{}'", r.request_number);
        self.repository.update_request_settlement(id, None).await
    }

    /// Cancel a request (draft or submitted only)
    pub async fn cancel_request(&self, id: Uuid) -> AtlasResult<FactoringRequest> {
        let r = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", id)))?;
        if r.status != "draft" && r.status != "submitted" {
            return Err(AtlasError::ValidationFailed(
                format!("Cannot cancel request in '{}' status. Must be 'draft' or 'submitted'.", r.status)
            ));
        }
        info!("Cancelling factoring request '{}'", r.request_number);
        self.repository.update_request_status(id, "cancelled").await
    }

    // ========================================================================
    // Request Lines
    // ========================================================================

    /// Add a line to a factoring request
    #[allow(clippy::too_many_arguments)]
    pub async fn add_request_line(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        line_number: i32,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>,
        invoice_amount: &str,
        eligible_amount: &str,
        days_outstanding: Option<i32>,
        days_overdue: Option<i32>,
    ) -> AtlasResult<FactoringRequestLine> {
        // Verify request exists and is in a mutable state
        let request = self.repository.get_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Request {} not found", request_id)))?;
        if request.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Cannot add lines to request in non-draft status".into()
            ));
        }

        let inv_amt: f64 = invoice_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid invoice amount".into()))?;
        let elig_amt: f64 = eligible_amount.parse().map_err(|_| AtlasError::ValidationFailed("Invalid eligible amount".into()))?;
        if inv_amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Invoice amount must be positive".into()));
        }
        if elig_amt < 0.0 || elig_amt > inv_amt {
            return Err(AtlasError::ValidationFailed("Eligible amount must be between 0 and invoice amount".into()));
        }
        if line_number < 1 {
            return Err(AtlasError::ValidationFailed("Line number must be positive".into()));
        }

        info!("Adding line {} to factoring request '{}'", line_number, request.request_number);
        self.repository.add_request_line(
            org_id, request_id, line_number, transaction_id, transaction_number,
            customer_id, customer_number, customer_name,
            invoice_date, invoice_due_date, invoice_amount, eligible_amount,
            days_outstanding, days_overdue,
        ).await
    }

    /// List lines for a request
    pub async fn list_request_lines(&self, request_id: Uuid) -> AtlasResult<Vec<FactoringRequestLine>> {
        self.repository.list_request_lines(request_id).await
    }

    // ========================================================================
    // Settlements
    // ========================================================================

    /// Create a new settlement
    pub async fn create_settlement(
        &self,
        org_id: Uuid,
        settlement_number: &str,
        agreement_id: Uuid,
        request_id: Option<Uuid>,
        settlement_date: chrono::NaiveDate,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringSettlement> {
        if settlement_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Settlement number is required".into()));
        }

        // Verify agreement exists
        let _agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", agreement_id)))?;

        info!("Creating factoring settlement '{}'", settlement_number);
        self.repository.create_settlement(
            org_id, settlement_number, agreement_id, request_id,
            settlement_date, currency_code, notes, created_by,
        ).await
    }

    /// List settlements with optional filters
    pub async fn list_settlements(
        &self,
        org_id: Uuid,
        agreement_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<FactoringSettlement>> {
        if let Some(s) = status {
            if !VALID_SETTLEMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SETTLEMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_settlements(org_id, agreement_id, status).await
    }

    /// Process a settlement
    pub async fn process_settlement(&self, id: Uuid, processed_by: Option<Uuid>) -> AtlasResult<FactoringSettlement> {
        // Delegate to the repository which validates the settlement exists
        // and is in a valid status for processing.
        info!("Processing factoring settlement {}", id);
        self.repository.update_settlement_status(id, "processed", processed_by).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the factoring dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FactoringDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        companies: std::sync::Mutex<Vec<FactorCompany>>,
        agreements: std::sync::Mutex<Vec<FactoringAgreement>>,
        requests: std::sync::Mutex<Vec<FactoringRequest>>,
        lines: std::sync::Mutex<Vec<FactoringRequestLine>>,
        settlements: std::sync::Mutex<Vec<FactoringSettlement>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                companies: std::sync::Mutex::new(vec![]),
                agreements: std::sync::Mutex::new(vec![]),
                requests: std::sync::Mutex::new(vec![]),
                lines: std::sync::Mutex::new(vec![]),
                settlements: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl ReceivablesFactoringRepository for MockRepo {
        async fn create_factor_company(
            &self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>,
            cn: Option<&str>, ce: Option<&str>, cp: Option<&str>,
            bn: Option<&str>, ba: Option<&str>,
            ar: &str, fr: &str, rt: &str,
            min_amt: Option<&str>, max_amt: Option<&str>,
            cb: Option<Uuid>,
        ) -> AtlasResult<FactorCompany> {
            let fc = FactorCompany {
                id: Uuid::new_v4(), organization_id: org_id,
                code: code.into(), name: name.into(), description: desc.map(Into::into),
                contact_name: cn.map(Into::into), contact_email: ce.map(Into::into),
                contact_phone: cp.map(Into::into), bank_name: bn.map(Into::into),
                bank_account_number: ba.map(Into::into),
                default_advance_rate: ar.into(), default_fee_rate: fr.into(),
                default_recourse_type: rt.into(),
                minimum_invoice_amount: min_amt.map(Into::into),
                maximum_invoice_amount: max_amt.map(Into::into),
                is_active: true, metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.companies.lock().unwrap().push(fc.clone());
            Ok(fc)
        }

        async fn get_factor_company(&self, id: Uuid) -> AtlasResult<Option<FactorCompany>> {
            Ok(self.companies.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn get_factor_company_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FactorCompany>> {
            Ok(self.companies.lock().unwrap().iter().find(|c| c.organization_id == org_id && c.code == code).cloned())
        }

        async fn list_factor_companies(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FactorCompany>> {
            Ok(self.companies.lock().unwrap().iter()
                .filter(|c| c.organization_id == org_id)
                .filter(|c| is_active.is_none_or(|a| c.is_active == a))
                .cloned().collect())
        }

        async fn deactivate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany> {
            let mut cs = self.companies.lock().unwrap();
            let c = cs.iter_mut().find(|c| c.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            c.is_active = false; c.updated_at = chrono::Utc::now(); Ok(c.clone())
        }

        async fn activate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany> {
            let mut cs = self.companies.lock().unwrap();
            let c = cs.iter_mut().find(|c| c.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            c.is_active = true; c.updated_at = chrono::Utc::now(); Ok(c.clone())
        }

        async fn create_agreement(
            &self, org_id: Uuid, num: &str, fc_id: Uuid, name: &str, desc: Option<&str>,
            at: &str, rt: &str, ar: &str, ffr: &str, lfr: &str, rr: &str, mf: &str,
            cc: &str, sd: chrono::NaiveDate, ed: Option<chrono::NaiveDate>,
            cl: Option<&str>, cb: Option<Uuid>,
        ) -> AtlasResult<FactoringAgreement> {
            let a = FactoringAgreement {
                id: Uuid::new_v4(), organization_id: org_id,
                agreement_number: num.into(), factor_company_id: fc_id,
                factor_company_code: None, agreement_name: name.into(),
                description: desc.map(Into::into), agreement_type: at.into(),
                recourse_type: rt.into(), advance_rate: ar.into(),
                factoring_fee_rate: ffr.into(), late_fee_rate: lfr.into(),
                reserve_rate: rr.into(), minimum_fee: mf.into(),
                currency_code: cc.into(), start_date: sd, end_date: ed,
                credit_limit: cl.map(Into::into),
                total_factored_amount: "0.00".into(), total_advance_amount: "0.00".into(),
                total_fee_amount: "0.00".into(), total_reserve_amount: "0.00".into(),
                total_settled_amount: "0.00".into(), status: "draft".into(),
                approved_by: None, approved_at: None, metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.agreements.lock().unwrap().push(a.clone());
            Ok(a)
        }

        async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<FactoringAgreement>> {
            Ok(self.agreements.lock().unwrap().iter().find(|a| a.id == id).cloned())
        }

        async fn list_agreements(&self, org_id: Uuid, status: Option<&str>, _fc_id: Option<Uuid>) -> AtlasResult<Vec<FactoringAgreement>> {
            Ok(self.agreements.lock().unwrap().iter()
                .filter(|a| a.organization_id == org_id)
                .filter(|a| status.is_none_or(|s| a.status == s))
                .cloned().collect())
        }

        async fn update_agreement_status(&self, id: Uuid, status: &str, ab: Option<Uuid>) -> AtlasResult<FactoringAgreement> {
            let mut as_ = self.agreements.lock().unwrap();
            let a = as_.iter_mut().find(|a| a.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            a.status = status.into();
            if status == "active" { a.approved_by = ab; a.approved_at = Some(chrono::Utc::now()); }
            a.updated_at = chrono::Utc::now();
            Ok(a.clone())
        }

        async fn create_request(
            &self, org_id: Uuid, num: &str, agr_id: Uuid, rd: chrono::NaiveDate,
            rt: &str, cc: &str, notes: Option<&str>, cb: Option<Uuid>,
        ) -> AtlasResult<FactoringRequest> {
            let r = FactoringRequest {
                id: Uuid::new_v4(), organization_id: org_id,
                request_number: num.into(), agreement_id: agr_id,
                agreement_number: None, factor_company_id: None, factor_company_name: None,
                request_date: rd, funding_date: None, settlement_date: None,
                total_invoice_amount: "0.00".into(), eligible_amount: "0.00".into(),
                advance_rate: "0.8000".into(), advance_amount: "0.00".into(),
                factoring_fee_rate: "0.0150".into(), factoring_fee_amount: "0.00".into(),
                reserve_rate: "0.0500".into(), reserve_amount: "0.00".into(),
                recourse_type: rt.into(), currency_code: cc.into(),
                status: "draft".into(),
                approved_by: None, approved_at: None,
                funded_by: None, funded_at: None,
                settled_by: None, settled_at: None,
                notes: notes.map(Into::into), metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.requests.lock().unwrap().push(r.clone());
            Ok(r)
        }

        async fn get_request(&self, id: Uuid) -> AtlasResult<Option<FactoringRequest>> {
            Ok(self.requests.lock().unwrap().iter().find(|r| r.id == id).cloned())
        }

        async fn list_requests(&self, org_id: Uuid, agr_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FactoringRequest>> {
            Ok(self.requests.lock().unwrap().iter()
                .filter(|r| r.organization_id == org_id)
                .filter(|r| agr_id.is_none_or(|id| r.agreement_id == id))
                .filter(|r| status.is_none_or(|s| r.status == s))
                .cloned().collect())
        }

        async fn update_request_status(&self, id: Uuid, status: &str) -> AtlasResult<FactoringRequest> {
            let mut rs = self.requests.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.status = status.into(); r.updated_at = chrono::Utc::now(); Ok(r.clone())
        }

        async fn update_request_funding(&self, id: Uuid, fb: Option<Uuid>) -> AtlasResult<FactoringRequest> {
            let mut rs = self.requests.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.status = "funded".into(); r.funded_by = fb; r.funded_at = Some(chrono::Utc::now()); r.updated_at = chrono::Utc::now(); Ok(r.clone())
        }

        async fn update_request_settlement(&self, id: Uuid, sb: Option<Uuid>) -> AtlasResult<FactoringRequest> {
            let mut rs = self.requests.lock().unwrap();
            let r = rs.iter_mut().find(|r| r.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            r.status = "settled".into(); r.settled_by = sb; r.settled_at = Some(chrono::Utc::now()); r.updated_at = chrono::Utc::now(); Ok(r.clone())
        }

        async fn add_request_line(
            &self, org_id: Uuid, req_id: Uuid, ln: i32,
            tid: Option<Uuid>, tnum: Option<&str>,
            cid: Option<Uuid>, cnum: Option<&str>, cname: Option<&str>,
            idate: Option<chrono::NaiveDate>, idd: Option<chrono::NaiveDate>,
            iam: &str, eam: &str, do_: Option<i32>, dov: Option<i32>,
        ) -> AtlasResult<FactoringRequestLine> {
            let line = FactoringRequestLine {
                id: Uuid::new_v4(), organization_id: org_id, request_id: req_id,
                line_number: ln, transaction_id: tid, transaction_number: tnum.map(Into::into),
                customer_id: cid, customer_number: cnum.map(Into::into),
                customer_name: cname.map(Into::into),
                invoice_date: idate, invoice_due_date: idd,
                invoice_amount: iam.into(), eligible_amount: eam.into(),
                days_outstanding: do_, days_overdue: dov,
                advance_amount: "0.00".into(), factoring_fee_amount: "0.00".into(),
                reserve_amount: "0.00".into(), settlement_amount: "0.00".into(),
                is_eligible: true, exclusion_reason: None,
                status: "pending".into(), settled_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.lines.lock().unwrap().push(line.clone());
            Ok(line)
        }

        async fn list_request_lines(&self, req_id: Uuid) -> AtlasResult<Vec<FactoringRequestLine>> {
            Ok(self.lines.lock().unwrap().iter().filter(|l| l.request_id == req_id).cloned().collect())
        }

        async fn create_settlement(
            &self, org_id: Uuid, num: &str, agr_id: Uuid, req_id: Option<Uuid>,
            sd: chrono::NaiveDate, cc: &str, notes: Option<&str>, cb: Option<Uuid>,
        ) -> AtlasResult<FactoringSettlement> {
            let s = FactoringSettlement {
                id: Uuid::new_v4(), organization_id: org_id,
                settlement_number: num.into(), agreement_id: agr_id, request_id: req_id,
                settlement_date: sd, total_settled: "0.00".into(),
                total_reserve_released: "0.00".into(), total_chargebacks: "0.00".into(),
                total_late_fees: "0.00".into(), net_to_customer: "0.00".into(),
                currency_code: cc.into(), status: "draft".into(),
                processed_by: None, processed_at: None,
                notes: notes.map(Into::into), metadata: serde_json::json!({}),
                created_by: cb, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.settlements.lock().unwrap().push(s.clone());
            Ok(s)
        }

        async fn list_settlements(&self, org_id: Uuid, agr_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FactoringSettlement>> {
            Ok(self.settlements.lock().unwrap().iter()
                .filter(|s| s.organization_id == org_id)
                .filter(|s| agr_id.is_none_or(|id| s.agreement_id == id))
                .filter(|s| status.is_none_or(|st| s.status == st))
                .cloned().collect())
        }

        async fn update_settlement_status(&self, id: Uuid, status: &str, pb: Option<Uuid>) -> AtlasResult<FactoringSettlement> {
            let mut ss = self.settlements.lock().unwrap();
            let s = ss.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            s.status = status.into();
            if status == "processed" { s.processed_by = pb; s.processed_at = Some(chrono::Utc::now()); }
            s.updated_at = chrono::Utc::now();
            Ok(s.clone())
        }

        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FactoringDashboard> {
            let companies = self.companies.lock().unwrap();
            let agreements = self.agreements.lock().unwrap();
            let requests = self.requests.lock().unwrap();
            let settlements = self.settlements.lock().unwrap();

            let org_companies: Vec<_> = companies.iter().filter(|c| c.organization_id == org_id).collect();
            let org_agreements: Vec<_> = agreements.iter().filter(|a| a.organization_id == org_id).collect();
            let org_requests: Vec<_> = requests.iter().filter(|r| r.organization_id == org_id).collect();
            let org_settlements: Vec<_> = settlements.iter().filter(|s| s.organization_id == org_id).collect();

            Ok(FactoringDashboard {
                organization_id: org_id,
                total_factor_companies: org_companies.len() as i64,
                active_factor_companies: org_companies.iter().filter(|c| c.is_active).count() as i64,
                total_agreements: org_agreements.len() as i64,
                active_agreements: org_agreements.iter().filter(|a| a.status == "active").count() as i64,
                total_requests: org_requests.len() as i64,
                draft_requests: org_requests.iter().filter(|r| r.status == "draft").count() as i64,
                approved_requests: org_requests.iter().filter(|r| r.status == "approved").count() as i64,
                funded_requests: org_requests.iter().filter(|r| r.status == "funded").count() as i64,
                settled_requests: org_requests.iter().filter(|r| r.status == "settled").count() as i64,
                total_invoices_factored: "0.00".into(),
                total_advances: "0.00".into(),
                total_fees: "0.00".into(),
                total_reserves: "0.00".into(),
                total_settled: org_settlements.iter()
                    .filter(|s| s.status == "processed")
                    .map(|s| s.total_settled.parse::<f64>().unwrap_or(0.0))
                    .sum::<f64>().to_string(),
            })
        }
    }

    fn eng() -> ReceivablesFactoringEngine {
        ReceivablesFactoringEngine::new(Arc::new(MockRepo::new()))
    }

    async fn make_active_factor(eng: &ReceivablesFactoringEngine, org_id: Uuid) -> FactorCompany {
        let fc = eng.create_factor_company(
            org_id, "FC-001", "Acme Factoring", None, None, None, None, None, None,
            "0.8500", "0.0200", "recourse", None, None, None,
        ).await.unwrap();
        // Activate the agreement is done via agreement; company is active by default
        fc
    }

    async fn make_active_agreement(eng: &ReceivablesFactoringEngine, org_id: Uuid, fc_id: Uuid) -> FactoringAgreement {
        let a = eng.create_agreement(
            org_id, "AGR-001", fc_id, "Main Agreement", None,
            "spot", "recourse", "0.8500", "0.0200", "0.0050", "0.0500", "0.00",
            "USD", chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None, None,
        ).await.unwrap();
        eng.activate_agreement(a.id, None).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_factor_company() {
        let fc = eng().create_factor_company(
            Uuid::new_v4(), "FC-001", "Acme Factoring", None, None, None, None, None, None,
            "0.8500", "0.0200", "recourse", None, None, None,
        ).await.unwrap();
        assert_eq!(fc.code, "FC-001");
        assert!(fc.is_active);
    }

    #[tokio::test]
    async fn test_create_factor_company_empty_code_fails() {
        let r = eng().create_factor_company(
            Uuid::new_v4(), "", "Name", None, None, None, None, None, None,
            "0.8", "0.01", "recourse", None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_factor_company_invalid_recourse_fails() {
        let r = eng().create_factor_company(
            Uuid::new_v4(), "FC-X", "Name", None, None, None, None, None, None,
            "0.8", "0.01", "invalid", None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_factor_company_duplicate_code_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_factor_company(org, "FC-DUP", "Name", None, None, None, None, None, None, "0.8", "0.01", "recourse", None, None, None).await.unwrap();
        let r = e.create_factor_company(org, "FC-DUP", "Name 2", None, None, None, None, None, None, "0.8", "0.01", "recourse", None, None, None).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_deactivate_activate_factor_company() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = e.create_factor_company(org, "FC-DA", "Name", None, None, None, None, None, None, "0.8", "0.01", "recourse", None, None, None).await.unwrap();
        let fc = e.deactivate_factor_company(fc.id).await.unwrap();
        assert!(!fc.is_active);
        let fc = e.activate_factor_company(fc.id).await.unwrap();
        assert!(fc.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = e.create_factor_company(org, "FC-DI", "Name", None, None, None, None, None, None, "0.8", "0.01", "recourse", None, None, None).await.unwrap();
        e.deactivate_factor_company(fc.id).await.unwrap();
        let r = e.deactivate_factor_company(fc.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_agreement() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = e.create_agreement(
            org, "AGR-001", fc.id, "Main Agreement", None,
            "spot", "recourse", "0.8500", "0.0200", "0.0050", "0.0500", "0.00",
            "USD", chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None, None,
        ).await.unwrap();
        assert_eq!(a.agreement_number, "AGR-001");
        assert_eq!(a.status, "draft");
    }

    #[tokio::test]
    async fn test_agreement_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        assert_eq!(a.status, "active");

        let a = e.suspend_agreement(a.id).await.unwrap();
        assert_eq!(a.status, "suspended");

        let a = e.terminate_agreement(a.id).await.unwrap();
        assert_eq!(a.status, "terminated");
    }

    #[tokio::test]
    async fn test_request_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;

        let r = e.create_request(
            org, "REQ-001", a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            "recourse", "USD", None, None,
        ).await.unwrap();
        assert_eq!(r.status, "draft");

        let r = e.submit_request(r.id).await.unwrap();
        assert_eq!(r.status, "submitted");

        let r = e.approve_request(r.id).await.unwrap();
        assert_eq!(r.status, "approved");

        let r = e.fund_request(r.id).await.unwrap();
        assert_eq!(r.status, "funded");

        let r = e.settle_request(r.id).await.unwrap();
        assert_eq!(r.status, "settled");
    }

    #[tokio::test]
    async fn test_cancel_request() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        let r = e.create_request(
            org, "REQ-CAN", a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            "recourse", "USD", None, None,
        ).await.unwrap();
        let r = e.cancel_request(r.id).await.unwrap();
        assert_eq!(r.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_funded_request_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        let r = e.create_request(
            org, "REQ-CF", a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            "recourse", "USD", None, None,
        ).await.unwrap();
        e.submit_request(r.id).await.unwrap();
        e.approve_request(r.id).await.unwrap();
        e.fund_request(r.id).await.unwrap();
        let res = e.cancel_request(r.id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_add_request_line() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        let r = e.create_request(
            org, "REQ-LN", a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            "recourse", "USD", None, None,
        ).await.unwrap();

        let line = e.add_request_line(
            org, r.id, 1, None, Some("INV-001"),
            None, Some("CUST-001"), Some("Acme Customer"),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap()),
            "10000.00", "10000.00", Some(30), Some(0),
        ).await.unwrap();
        assert_eq!(line.invoice_amount, "10000.00");
        assert_eq!(line.line_number, 1);
    }

    #[tokio::test]
    async fn test_add_line_to_submitted_request_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        let r = e.create_request(
            org, "REQ-LNS", a.id,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            "recourse", "USD", None, None,
        ).await.unwrap();
        e.submit_request(r.id).await.unwrap();
        let res = e.add_request_line(org, r.id, 1, None, None, None, None, None, None, None, "1000.00", "1000.00", None, None).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_settlement_lifecycle() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;

        let s = e.create_settlement(
            org, "STL-001", a.id, None,
            chrono::NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            "USD", None, None,
        ).await.unwrap();
        assert_eq!(s.status, "draft");

        let s = e.process_settlement(s.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(s.status, "processed");
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = make_active_agreement(&e, org, fc.id).await;
        let r = e.create_request(org, "REQ-D", a.id, chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(), "recourse", "USD", None, None).await.unwrap();
        e.submit_request(r.id).await.unwrap();
        e.approve_request(r.id).await.unwrap();
        e.fund_request(r.id).await.unwrap();

        let d = e.get_dashboard(org).await.unwrap();
        assert_eq!(d.total_factor_companies, 1);
        assert_eq!(d.active_factor_companies, 1);
        assert_eq!(d.total_requests, 1);
        assert_eq!(d.funded_requests, 1);
    }

    #[tokio::test]
    async fn test_create_agreement_inactive_company_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = e.create_factor_company(org, "FC-IN", "Name", None, None, None, None, None, None, "0.8", "0.01", "recourse", None, None, None).await.unwrap();
        e.deactivate_factor_company(fc.id).await.unwrap();
        let r = e.create_agreement(
            org, "AGR-IN", fc.id, "Test", None,
            "spot", "recourse", "0.8", "0.02", "0.005", "0.05", "0.00",
            "USD", chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_request_non_active_agreement_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let fc = make_active_factor(&e, org).await;
        let a = e.create_agreement(
            org, "AGR-NA", fc.id, "Test", None,
            "spot", "recourse", "0.85", "0.02", "0.005", "0.05", "0.00",
            "USD", chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None, None,
        ).await.unwrap();
        // agreement is in 'draft' status, not active
        let r = e.create_request(org, "REQ-NA", a.id, chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(), "recourse", "USD", None, None).await;
        assert!(r.is_err());
    }
}
