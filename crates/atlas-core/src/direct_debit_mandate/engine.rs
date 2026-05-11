//! Direct Debit Mandate Management Engine
//!
//! Manages the full lifecycle of direct debit mandates:
//! - Create mandates with customer bank details
//! - Activate mandates for collection
//! - Record collections against mandates
//! - Process collection completion, failures, and returns
//! - Revoke or cancel mandates
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Receivables > Direct Debit Mandates

use super::{DirectDebitMandateRepository, AtlasResult, DirectDebitMandate, AtlasError, MandateCollection, DirectDebitDashboard};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_MANDATE_TYPES: &[&str] = &["core", "b2b", "business_to_business", "ach", "one_off", "other"];
const VALID_MANDATE_STATUSES: &[&str] = &["draft", "active", "used", "cancelled", "expired", "revoked"];
const VALID_COLLECTION_TYPES: &[&str] = &["first", "recurring", "final", "one_off"];
const VALID_COLLECTION_STATUSES: &[&str] = &["pending", "submitted", "completed", "failed", "returned", "reversed"];

pub struct DirectDebitMandateEngine {
    repository: Arc<dyn DirectDebitMandateRepository>,
}

impl DirectDebitMandateEngine {
    pub fn new(repository: Arc<dyn DirectDebitMandateRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Mandates
    // ========================================================================

    /// Create a new direct debit mandate
    pub async fn create_mandate(
        &self,
        org_id: Uuid,
        mandate_number: &str,
        customer_id: Uuid,
        customer_name: Option<&str>,
        customer_account_number: Option<&str>,
        mandate_type: &str,
        bank_account_holder: Option<&str>,
        bank_account_number: Option<&str>,
        bank_account_number_type: Option<&str>,
        bank_code: Option<&str>,
        bank_code_type: Option<&str>,
        bank_name: Option<&str>,
        bank_branch: Option<&str>,
        creditor_scheme_id: Option<&str>,
        creditor_name: Option<&str>,
        mandate_reference: Option<&str>,
        mandate_date: chrono::NaiveDate,
        expiry_date: Option<chrono::NaiveDate>,
        max_collection_amount: Option<&str>,
        currency_code: &str,
        authorization_reference: Option<&str>,
        authorization_method: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DirectDebitMandate> {
        if mandate_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Mandate number is required".into()));
        }
        if !VALID_MANDATE_TYPES.contains(&mandate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid mandate type '{}'. Must be one of: {}", mandate_type, VALID_MANDATE_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() || currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if bank_account_number.is_none() || bank_account_number.unwrap().is_empty() {
            return Err(AtlasError::ValidationFailed("Bank account number is required".into()));
        }
        if bank_code.is_none() || bank_code.unwrap().is_empty() {
            return Err(AtlasError::ValidationFailed("Bank code (BIC/sort code) is required".into()));
        }
        if let Some(max_amt) = max_collection_amount {
            let val: f64 = max_amt.parse().unwrap_or(f64::NAN);
            if !val.is_nan() && val < 0.0 {
                return Err(AtlasError::ValidationFailed("Max collection amount cannot be negative".into()));
            }
        }
        if let Some(exp) = expiry_date {
            if exp <= mandate_date {
                return Err(AtlasError::ValidationFailed("Expiry date must be after mandate date".into()));
            }
        }

        if self.repository.get_mandate_by_number(org_id, mandate_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Mandate number '{mandate_number}' already exists")));
        }

        info!("Creating direct debit mandate '{}' for customer {}", mandate_number, customer_id);
        self.repository.create_mandate(
            org_id, mandate_number, customer_id, customer_name,
            customer_account_number, mandate_type, bank_account_holder,
            bank_account_number, bank_account_number_type, bank_code,
            bank_code_type, bank_name, bank_branch, creditor_scheme_id,
            creditor_name, mandate_reference, mandate_date, expiry_date,
            max_collection_amount, currency_code, authorization_reference,
            authorization_method, notes, created_by,
        ).await
    }

    /// Get a mandate by ID
    pub async fn get_mandate(&self, id: Uuid) -> AtlasResult<Option<DirectDebitMandate>> {
        self.repository.get_mandate(id).await
    }

    /// List mandates with optional filters
    pub async fn list_mandates(
        &self, org_id: Uuid, customer_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<DirectDebitMandate>> {
        if let Some(s) = status {
            if !VALID_MANDATE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_MANDATE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_mandates(org_id, customer_id, status).await
    }

    /// Activate a mandate (transition from draft to active)
    pub async fn activate_mandate(&self, mandate_id: Uuid) -> AtlasResult<DirectDebitMandate> {
        let mandate = self.repository.get_mandate(mandate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {mandate_id} not found")))?;

        if mandate.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate mandate in '{}' status. Must be 'draft'.", mandate.status)
            ));
        }

        info!("Activating direct debit mandate '{}'", mandate.mandate_number);
        let today = chrono::Utc::now().date_naive();
        self.repository.update_mandate_activation(mandate_id, today).await?;
        self.repository.update_mandate_status(mandate_id, "active", None).await
    }

    /// Cancel a mandate
    pub async fn cancel_mandate(&self, mandate_id: Uuid, reason: &str) -> AtlasResult<DirectDebitMandate> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Cancellation reason is required".into()));
        }

        let mandate = self.repository.get_mandate(mandate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {mandate_id} not found")))?;

        if mandate.status == "cancelled" || mandate.status == "revoked" || mandate.status == "expired" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel mandate in '{}' status.", mandate.status)
            ));
        }

        info!("Cancelling direct debit mandate '{}' (reason: {})", mandate.mandate_number, reason);
        self.repository.update_mandate_status(mandate_id, "cancelled", Some(reason)).await
    }

    /// Revoke a mandate (customer-initiated cancellation)
    pub async fn revoke_mandate(&self, mandate_id: Uuid, reason: &str) -> AtlasResult<DirectDebitMandate> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Revocation reason is required".into()));
        }

        let mandate = self.repository.get_mandate(mandate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {mandate_id} not found")))?;

        if mandate.status == "cancelled" || mandate.status == "revoked" || mandate.status == "expired" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot revoke mandate in '{}' status.", mandate.status)
            ));
        }

        info!("Revoking direct debit mandate '{}' (reason: {})", mandate.mandate_number, reason);
        self.repository.update_mandate_status(mandate_id, "revoked", Some(reason)).await
    }

    /// Set mandate expiry
    pub async fn expire_mandate(&self, mandate_id: Uuid, expiry_date: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate> {
        let mandate = self.repository.get_mandate(mandate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {mandate_id} not found")))?;

        if expiry_date <= mandate.mandate_date {
            return Err(AtlasError::ValidationFailed("Expiry date must be after mandate date".into()));
        }

        info!("Setting expiry for mandate '{}' to {}", mandate.mandate_number, expiry_date);
        self.repository.update_mandate_expiry(mandate_id, expiry_date).await
    }

    // ========================================================================
    // Collections
    // ========================================================================

    /// Create a new collection against a mandate
    pub async fn create_collection(
        &self,
        org_id: Uuid,
        mandate_id: Uuid,
        collection_number: &str,
        collection_type: &str,
        amount: &str,
        currency_code: &str,
        invoice_id: Option<Uuid>,
        invoice_number: Option<&str>,
        receipt_id: Option<Uuid>,
        scheduled_date: chrono::NaiveDate,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<MandateCollection> {
        if collection_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Collection number is required".into()));
        }
        if !VALID_COLLECTION_TYPES.contains(&collection_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid collection type '{}'. Must be one of: {}", collection_type, VALID_COLLECTION_TYPES.join(", ")
            )));
        }

        let amount_val: f64 = amount.parse().unwrap_or(f64::NAN);
        if amount_val.is_nan() || amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("Amount must be a positive number".into()));
        }

        let mandate = self.repository.get_mandate(mandate_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {mandate_id} not found")))?;

        if mandate.status != "active" && mandate.status != "used" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot create collection for mandate in '{}' status. Must be 'active' or 'used'.", mandate.status)
            ));
        }

        // Check max collection amount
        if let Some(max_str) = &mandate.max_collection_amount {
            let max_val: f64 = max_str.parse().unwrap_or(0.0);
            if max_val > 0.0 && amount_val > max_val {
                return Err(AtlasError::ValidationFailed(format!(
                    "Collection amount {amount} exceeds mandate max of {max_str}"
                )));
            }
        }

        // Validate collection type matches mandate's next expected type
        if mandate.next_collection_type.as_deref() == Some("first") && collection_type != "first" && collection_type != "one_off" {
            return Err(AtlasError::ValidationFailed(
                "First collection under this mandate must be of type 'first' or 'one_off'".into()
            ));
        }

        info!("Creating collection '{}' for mandate '{}' (amount: {})", collection_number, mandate.mandate_number, amount);
        self.repository.create_collection(
            org_id, mandate_id, collection_number, collection_type,
            amount, currency_code, invoice_id, invoice_number, receipt_id,
            scheduled_date, notes, created_by,
        ).await
    }

    /// Get a collection by ID
    pub async fn get_collection(&self, id: Uuid) -> AtlasResult<Option<MandateCollection>> {
        self.repository.get_collection(id).await
    }

    /// List collections for a mandate
    pub async fn list_collections(&self, mandate_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MandateCollection>> {
        if let Some(s) = status {
            if !VALID_COLLECTION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_COLLECTION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_collections(mandate_id, status).await
    }

    /// Submit a pending collection to the bank
    pub async fn submit_collection(&self, collection_id: Uuid) -> AtlasResult<MandateCollection> {
        let collection = self.repository.get_collection(collection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {collection_id} not found")))?;

        if collection.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit collection in '{}' status. Must be 'pending'.", collection.status)
            ));
        }

        info!("Submitting collection '{}' to bank", collection.collection_number);
        let today = chrono::Utc::now().date_naive();
        let c = self.repository.update_collection_status(collection_id, "submitted", None).await?;
        // Update execution date
        let _ = today; // In real impl, we'd store this
        Ok(c)
    }

    /// Complete a submitted collection
    pub async fn complete_collection(&self, collection_id: Uuid, bank_reference: Option<&str>) -> AtlasResult<MandateCollection> {
        let collection = self.repository.get_collection(collection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {collection_id} not found")))?;

        if collection.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete collection in '{}' status. Must be 'submitted'.", collection.status)
            ));
        }

        info!("Completing collection '{}'", collection.collection_number);
        let c = self.repository.update_collection_status(collection_id, "completed", bank_reference).await?;

        // Update mandate stats
        let today = chrono::Utc::now().date_naive();
        let total_collected = self.repository.sum_collected_for_mandate(collection.mandate_id).await?;
        let count = self.repository.count_collections_for_mandate(collection.mandate_id).await?;
        let total_f: f64 = total_collected.parse().unwrap_or(0.0);
        self.repository.update_mandate_collection_stats(
            collection.mandate_id,
            count as i32,
            &format!("{total_f:.2}"),
            Some(today),
            "recurring",
        ).await?;

        // Move mandate to 'used' status
        let mandate = self.repository.get_mandate(collection.mandate_id).await?;
        if let Some(m) = mandate {
            if m.status == "active" {
                self.repository.update_mandate_status(collection.mandate_id, "used", None).await?;
            }
        }

        Ok(c)
    }

    /// Mark a collection as failed
    pub async fn fail_collection(&self, collection_id: Uuid, reason_code: &str, reason_text: Option<&str>) -> AtlasResult<MandateCollection> {
        let collection = self.repository.get_collection(collection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {collection_id} not found")))?;

        if collection.status != "submitted" && collection.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot fail collection in '{}' status. Must be 'pending' or 'submitted'.", collection.status)
            ));
        }

        info!("Failing collection '{}' (reason: {})", collection.collection_number, reason_code);
        self.repository.update_collection_return(collection_id, reason_code, reason_text).await?;
        self.repository.update_collection_status(collection_id, "failed", None).await
    }

    /// Return a completed collection (chargeback/reversal)
    pub async fn return_collection(&self, collection_id: Uuid, reason_code: &str, reason_text: Option<&str>) -> AtlasResult<MandateCollection> {
        let collection = self.repository.get_collection(collection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {collection_id} not found")))?;

        if collection.status != "completed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot return collection in '{}' status. Must be 'completed'.", collection.status)
            ));
        }

        info!("Returning collection '{}' (reason: {})", collection.collection_number, reason_code);
        self.repository.update_collection_return(collection_id, reason_code, reason_text).await?;
        self.repository.update_collection_status(collection_id, "returned", None).await
    }

    /// Reverse a collection
    pub async fn reverse_collection(&self, collection_id: Uuid, reason_code: &str, reason_text: Option<&str>) -> AtlasResult<MandateCollection> {
        let collection = self.repository.get_collection(collection_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {collection_id} not found")))?;

        if collection.status != "completed" && collection.status != "returned" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse collection in '{}' status. Must be 'completed' or 'returned'.", collection.status)
            ));
        }

        info!("Reversing collection '{}' (reason: {})", collection.collection_number, reason_code);
        self.repository.update_collection_return(collection_id, reason_code, reason_text).await?;
        self.repository.update_collection_status(collection_id, "reversed", None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DirectDebitDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        mandates: std::sync::Mutex<Vec<DirectDebitMandate>>,
        collections: std::sync::Mutex<Vec<MandateCollection>>,
    }

    impl MockRepo {
        fn new() -> Self {
            MockRepo {
                mandates: std::sync::Mutex::new(vec![]),
                collections: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[allow(dead_code)]
    fn make_mandate(
        id: Uuid, org_id: Uuid, num: &str, customer_id: Uuid, mandate_type: &str,
    ) -> DirectDebitMandate {
        DirectDebitMandate {
            id, organization_id: org_id,
            mandate_number: num.into(), customer_id,
            customer_name: None, customer_account_number: None,
            mandate_type: mandate_type.into(),
            status: "draft".into(),
            bank_account_holder: None,
            bank_account_number: Some("DE89370400440532013000".into()),
            bank_account_number_type: Some("iban".into()),
            bank_code: Some("COBADEFFXXX".into()),
            bank_code_type: Some("bic".into()),
            bank_name: None, bank_branch: None,
            creditor_scheme_id: None, creditor_name: None,
            mandate_reference: None,
            mandate_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            activation_date: None, first_collection_date: None,
            last_collection_date: None, expiry_date: None,
            cancellation_date: None,
            collection_count: 0, total_collected: "0.00".into(),
            next_collection_type: Some("first".into()),
            max_collection_amount: Some("0".into()),
            currency_code: "USD".into(),
            status_reason: None,
            authorization_reference: None, authorization_method: None,
            notes: None, metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait::async_trait]
    impl DirectDebitMandateRepository for MockRepo {
        async fn create_mandate(
            &self, org_id: Uuid, num: &str, customer_id: Uuid,
            customer_name: Option<&str>, _customer_account_number: Option<&str>,
            mandate_type: &str, _bank_account_holder: Option<&str>,
            bank_account_number: Option<&str>, bank_account_number_type: Option<&str>,
            bank_code: Option<&str>, bank_code_type: Option<&str>,
            _bank_name: Option<&str>, _bank_branch: Option<&str>,
            _creditor_scheme_id: Option<&str>, _creditor_name: Option<&str>,
            _mandate_reference: Option<&str>, mandate_date: chrono::NaiveDate,
            expiry_date: Option<chrono::NaiveDate>,
            max_collection_amount: Option<&str>, currency_code: &str,
            _authorization_reference: Option<&str>, _authorization_method: Option<&str>,
            _notes: Option<&str>, _created_by: Option<Uuid>,
        ) -> AtlasResult<DirectDebitMandate> {
            let id = Uuid::new_v4();
            let m = DirectDebitMandate {
                id, organization_id: org_id, mandate_number: num.into(), customer_id,
                customer_name: customer_name.map(Into::into),
                customer_account_number: None,
                mandate_type: mandate_type.into(),
                status: "draft".into(),
                bank_account_holder: None,
                bank_account_number: bank_account_number.map(Into::into),
                bank_account_number_type: bank_account_number_type.map(Into::into),
                bank_code: bank_code.map(Into::into),
                bank_code_type: bank_code_type.map(Into::into),
                bank_name: None, bank_branch: None,
                creditor_scheme_id: None, creditor_name: None,
                mandate_reference: None,
                mandate_date,
                activation_date: None, first_collection_date: None,
                last_collection_date: None, expiry_date,
                cancellation_date: None,
                collection_count: 0, total_collected: "0.00".into(),
                next_collection_type: Some("first".into()),
                max_collection_amount: max_collection_amount.map(Into::into),
                currency_code: currency_code.into(),
                status_reason: None,
                authorization_reference: None, authorization_method: None,
                notes: None, metadata: serde_json::json!({}),
                created_by: None,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.mandates.lock().unwrap().push(m.clone());
            Ok(m)
        }

        async fn get_mandate(&self, id: Uuid) -> AtlasResult<Option<DirectDebitMandate>> {
            Ok(self.mandates.lock().unwrap().iter().find(|m| m.id == id).cloned())
        }

        async fn get_mandate_by_number(&self, org_id: Uuid, num: &str) -> AtlasResult<Option<DirectDebitMandate>> {
            Ok(self.mandates.lock().unwrap().iter()
                .find(|m| m.organization_id == org_id && m.mandate_number == num).cloned())
        }

        async fn list_mandates(&self, org_id: Uuid, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<DirectDebitMandate>> {
            Ok(self.mandates.lock().unwrap().iter()
                .filter(|m| m.organization_id == org_id)
                .filter(|m| customer_id.is_none_or(|c| m.customer_id == c))
                .filter(|m| status.is_none_or(|s| m.status == s))
                .cloned().collect())
        }

        async fn update_mandate_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<DirectDebitMandate> {
            let mut ms = self.mandates.lock().unwrap();
            let m = ms.iter_mut().find(|m| m.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {} not found", id)))?;
            m.status = status.into();
            m.status_reason = reason.map(Into::into);
            m.updated_at = chrono::Utc::now();
            Ok(m.clone())
        }

        async fn update_mandate_activation(&self, id: Uuid, activation_date: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate> {
            let mut ms = self.mandates.lock().unwrap();
            let m = ms.iter_mut().find(|m| m.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {} not found", id)))?;
            m.activation_date = Some(activation_date);
            m.updated_at = chrono::Utc::now();
            Ok(m.clone())
        }

        async fn update_mandate_collection_stats(&self, id: Uuid, count: i32, total: &str, last_date: Option<chrono::NaiveDate>, next_type: &str) -> AtlasResult<()> {
            let mut ms = self.mandates.lock().unwrap();
            if let Some(m) = ms.iter_mut().find(|m| m.id == id) {
                m.collection_count = count;
                m.total_collected = total.into();
                m.last_collection_date = last_date;
                m.next_collection_type = Some(next_type.into());
            }
            Ok(())
        }

        async fn update_mandate_expiry(&self, id: Uuid, expiry_date: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate> {
            let mut ms = self.mandates.lock().unwrap();
            let m = ms.iter_mut().find(|m| m.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Mandate {} not found", id)))?;
            m.expiry_date = Some(expiry_date);
            m.updated_at = chrono::Utc::now();
            Ok(m.clone())
        }

        async fn create_collection(
            &self, org_id: Uuid, mandate_id: Uuid, num: &str,
            collection_type: &str, amount: &str, currency_code: &str,
            invoice_id: Option<Uuid>, invoice_number: Option<&str>,
            receipt_id: Option<Uuid>, scheduled_date: chrono::NaiveDate,
            notes: Option<&str>, _created_by: Option<Uuid>,
        ) -> AtlasResult<MandateCollection> {
            let c = MandateCollection {
                id: Uuid::new_v4(), organization_id: org_id, mandate_id,
                collection_number: num.into(), collection_type: collection_type.into(),
                status: "pending".into(), amount: amount.into(),
                currency_code: currency_code.into(),
                invoice_id, invoice_number: invoice_number.map(Into::into),
                receipt_id,
                scheduled_date, execution_date: None, settlement_date: None,
                return_reason_code: None, return_reason_text: None,
                bank_reference: None, notes: notes.map(Into::into),
                metadata: serde_json::json!({}), created_by: None,
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            };
            self.collections.lock().unwrap().push(c.clone());
            Ok(c)
        }

        async fn get_collection(&self, id: Uuid) -> AtlasResult<Option<MandateCollection>> {
            Ok(self.collections.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn list_collections(&self, mandate_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MandateCollection>> {
            Ok(self.collections.lock().unwrap().iter()
                .filter(|c| c.mandate_id == mandate_id)
                .filter(|c| status.is_none_or(|s| c.status == s))
                .cloned().collect())
        }

        async fn update_collection_status(&self, id: Uuid, status: &str, bank_reference: Option<&str>) -> AtlasResult<MandateCollection> {
            let mut cs = self.collections.lock().unwrap();
            let c = cs.iter_mut().find(|c| c.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {} not found", id)))?;
            c.status = status.into();
            c.bank_reference = bank_reference.map(Into::into);
            c.updated_at = chrono::Utc::now();
            Ok(c.clone())
        }

        async fn update_collection_return(&self, id: Uuid, reason_code: &str, reason_text: Option<&str>) -> AtlasResult<MandateCollection> {
            let mut cs = self.collections.lock().unwrap();
            let c = cs.iter_mut().find(|c| c.id == id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Collection {} not found", id)))?;
            c.return_reason_code = Some(reason_code.into());
            c.return_reason_text = reason_text.map(Into::into);
            c.updated_at = chrono::Utc::now();
            Ok(c.clone())
        }

        async fn sum_collected_for_mandate(&self, mandate_id: Uuid) -> AtlasResult<String> {
            let cs = self.collections.lock().unwrap();
            let sum: f64 = cs.iter()
                .filter(|c| c.mandate_id == mandate_id && c.status == "completed")
                .map(|c| c.amount.parse::<f64>().unwrap_or(0.0))
                .sum();
            Ok(format!("{:.2}", sum))
        }

        async fn count_collections_for_mandate(&self, mandate_id: Uuid) -> AtlasResult<i64> {
            let cs = self.collections.lock().unwrap();
            Ok(cs.iter().filter(|c| c.mandate_id == mandate_id && c.status == "completed").count() as i64)
        }

        async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<DirectDebitDashboard> {
            Ok(DirectDebitDashboard {
                total_mandates: 0, draft_mandates: 0, active_mandates: 0, used_mandates: 0,
                cancelled_mandates: 0, expired_mandates: 0, revoked_mandates: 0,
                total_collections: 0, completed_collections: 0, failed_collections: 0,
                pending_collections: 0, total_collected_amount: "0.00".into(),
                pending_amount: "0.00".into(),
            })
        }
    }

    fn eng() -> DirectDebitMandateEngine {
        DirectDebitMandateEngine::new(Arc::new(MockRepo::new()))
    }

    // ========================================================================
    // Mandate Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_mandate() {
        let m = eng().create_mandate(
            Uuid::new_v4(), "MANDATE-001", Uuid::new_v4(),
            Some("Acme Corp"), Some("CUST-001"),
            "core", Some("Acme Corp"),
            Some("DE89370400440532013000"), Some("iban"),
            Some("COBADEFFXXX"), Some("bic"),
            Some("Commerzbank"), None,
            Some("CRED-SCHEME-001"), Some("Our Company"),
            Some("MANDATE-REF-001"),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, Some("10000.00"), "EUR",
            Some("AUTH-001"), Some("paper"),
            None, None,
        ).await.unwrap();
        assert_eq!(m.mandate_number, "MANDATE-001");
        assert_eq!(m.status, "draft");
        assert_eq!(m.mandate_type, "core");
    }

    #[tokio::test]
    async fn test_create_mandate_empty_number_fails() {
        let r = eng().create_mandate(
            Uuid::new_v4(), "", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_mandate_invalid_type_fails() {
        let r = eng().create_mandate(
            Uuid::new_v4(), "M-001", Uuid::new_v4(), None, None,
            "crypto", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_mandate_no_bank_account_fails() {
        let r = eng().create_mandate(
            Uuid::new_v4(), "M-002", Uuid::new_v4(), None, None,
            "core", None, None, None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_mandate_no_bank_code_fails() {
        let r = eng().create_mandate(
            Uuid::new_v4(), "M-003", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, None, None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_mandate_duplicate_number_fails() {
        let org = Uuid::new_v4();
        let e = eng();
        let _ = e.create_mandate(
            org, "DUP-MAND", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        let r = e.create_mandate(
            org, "DUP-MAND", Uuid::new_v4(), None, None,
            "ach", None, Some("IBAN456"), None, Some("BIC456"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await;
        assert!(matches!(r, Err(AtlasError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_activate_mandate() {
        let e = eng();
        let m = e.create_mandate(
            Uuid::new_v4(), "M-ACT", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        assert_eq!(m.status, "draft");

        let m = e.activate_mandate(m.id).await.unwrap();
        assert_eq!(m.status, "active");
        assert!(m.activation_date.is_some());
    }

    #[tokio::test]
    async fn test_activate_non_draft_fails() {
        let e = eng();
        let m = e.create_mandate(
            Uuid::new_v4(), "M-ACT2", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_mandate(m.id).await.unwrap();
        let r = e.activate_mandate(m.id).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_cancel_mandate() {
        let e = eng();
        let m = e.create_mandate(
            Uuid::new_v4(), "M-CANC", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        let m = e.cancel_mandate(m.id, "Customer request").await.unwrap();
        assert_eq!(m.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_mandate_no_reason_fails() {
        let e = eng();
        let m = e.create_mandate(
            Uuid::new_v4(), "M-CANC2", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        let r = e.cancel_mandate(m.id, "").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_revoke_mandate() {
        let e = eng();
        let m = e.create_mandate(
            Uuid::new_v4(), "M-REV", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        let m = e.revoke_mandate(m.id, "Customer revoked authorization").await.unwrap();
        assert_eq!(m.status, "revoked");
    }

    // ========================================================================
    // Collection Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_collection() {
        let e = eng();
        let org = Uuid::new_v4();
        let m = e.create_mandate(
            org, "M-COLL", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_mandate(m.id).await.unwrap();

        let c = e.create_collection(
            org, m.id, "COLL-001", "first", "5000.00", "USD",
            Some(Uuid::new_v4()), Some("INV-001"), None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await.unwrap();
        assert_eq!(c.collection_number, "COLL-001");
        assert_eq!(c.status, "pending");
        assert_eq!(c.collection_type, "first");
    }

    #[tokio::test]
    async fn test_create_collection_inactive_mandate_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let m = e.create_mandate(
            org, "M-COLL2", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        // Still in draft status
        let r = e.create_collection(
            org, m.id, "COLL-BAD", "first", "1000.00", "USD",
            None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_create_collection_wrong_type_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let m = e.create_mandate(
            org, "M-COLL3", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_mandate(m.id).await.unwrap();
        // Should be 'first' but we try 'recurring'
        let r = e.create_collection(
            org, m.id, "COLL-BAD2", "recurring", "1000.00", "USD",
            None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_submit_and_complete_collection() {
        let e = eng();
        let org = Uuid::new_v4();
        let m = e.create_mandate(
            org, "M-SUB", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_mandate(m.id).await.unwrap();

        let c = e.create_collection(
            org, m.id, "COLL-SUB", "first", "2500.00", "USD",
            None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await.unwrap();

        let c = e.submit_collection(c.id).await.unwrap();
        assert_eq!(c.status, "submitted");

        let c = e.complete_collection(c.id, Some("BANK-REF-001")).await.unwrap();
        assert_eq!(c.status, "completed");
        assert_eq!(c.bank_reference, Some("BANK-REF-001".into()));
    }

    #[tokio::test]
    async fn test_full_collection_workflow() {
        let e = eng();
        let org = Uuid::new_v4();
        let customer_id = Uuid::new_v4();

        // 1. Create mandate
        let m = e.create_mandate(
            org, "M-WF", customer_id, Some("Acme Corp"), None,
            "core", Some("Acme Corp"),
            Some("DE89370400440532013000"), Some("iban"),
            Some("COBADEFFXXX"), Some("bic"),
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None, None, "EUR", None, None, None, None,
        ).await.unwrap();

        // 2. Activate
        let m = e.activate_mandate(m.id).await.unwrap();
        assert_eq!(m.status, "active");

        // 3. Create first collection
        let c1 = e.create_collection(
            org, m.id, "COLL-WF-1", "first", "3000.00", "EUR",
            Some(Uuid::new_v4()), Some("INV-001"), None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await.unwrap();

        // 4. Submit and complete
        e.submit_collection(c1.id).await.unwrap();
        let c1 = e.complete_collection(c1.id, Some("BANK-001")).await.unwrap();
        assert_eq!(c1.status, "completed");

        // 5. Verify mandate is now 'used'
        let m = e.get_mandate(m.id).await.unwrap().unwrap();
        assert_eq!(m.status, "used");

        // 6. Create recurring collection
        let c2 = e.create_collection(
            org, m.id, "COLL-WF-2", "recurring", "1500.00", "EUR",
            Some(Uuid::new_v4()), Some("INV-002"), None,
            chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            None, None,
        ).await.unwrap();
        e.submit_collection(c2.id).await.unwrap();
        let c2 = e.complete_collection(c2.id, Some("BANK-002")).await.unwrap();
        assert_eq!(c2.status, "completed");

        // 7. Return the second collection
        let c2 = e.return_collection(c2.id, "R01", Some("Insufficient funds")).await.unwrap();
        assert_eq!(c2.status, "returned");
        assert_eq!(c2.return_reason_code, Some("R01".into()));
    }

    #[tokio::test]
    async fn test_fail_collection() {
        let e = eng();
        let org = Uuid::new_v4();
        let m = e.create_mandate(
            org, "M-FAIL", Uuid::new_v4(), None, None,
            "core", None, Some("IBAN123"), None, Some("BIC123"), None,
            None, None, None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None, None, "USD", None, None, None, None,
        ).await.unwrap();
        e.activate_mandate(m.id).await.unwrap();

        let c = e.create_collection(
            org, m.id, "COLL-FAIL", "first", "1000.00", "USD",
            None, None, None,
            chrono::NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None, None,
        ).await.unwrap();

        let c = e.fail_collection(c.id, "R04", Some("Account closed")).await.unwrap();
        assert_eq!(c.status, "failed");
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let d = eng().get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(d.total_mandates, 0);
        assert_eq!(d.total_collected_amount, "0.00");
    }
}
