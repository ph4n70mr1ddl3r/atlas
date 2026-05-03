//! Letter of Credit Engine
//!
//! Manages letter of credit lifecycle: draft → issued → advised → confirmed →
//! accepted → paid / expired / cancelled, with amendments, document tracking,
//! shipments, presentations, and dashboard reporting.
//!
//! Oracle Fusion equivalent: Treasury > Trade Finance > Letters of Credit

use atlas_shared::{
    LetterOfCredit, LcAmendment, LcRequiredDocument, LcShipment,
    LcPresentation, LcPresentationDocument, LcDashboard,
    AtlasError, AtlasResult,
};
use super::LetterOfCreditRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid LC types
const VALID_LC_TYPES: &[&str] = &[
    "import", "export", "back_to_back", "transferable",
    "standby", "revolving", "red_clause", "green_clause",
];

/// Valid LC forms
const VALID_LC_FORMS: &[&str] = &[
    "irrevocable", "revocable", "irrevocable_confirmed", "irrevocable_unconfirmed",
];

/// Valid LC statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "issued", "advised", "confirmed", "accepted",
    "paid", "expired", "cancelled",
];

/// Valid amendment types
const VALID_AMENDMENT_TYPES: &[&str] = &[
    "amount_increase", "amount_decrease", "expiry_extension",
    "expiry_reduction", "terms_change", "beneficiary_change",
    "document_change", "shipment_change", "other",
];

/// Valid amendment statuses
const VALID_AMENDMENT_STATUSES: &[&str] = &[
    "draft", "pending_approval", "approved", "rejected", "applied",
];

/// Valid shipment statuses
const VALID_SHIPMENT_STATUSES: &[&str] = &[
    "pending", "shipped", "in_transit", "arrived", "delivered", "cancelled",
];

/// Valid presentation statuses
const VALID_PRESENTATION_STATUSES: &[&str] = &[
    "submitted", "under_review", "compliant", "discrepant",
    "accepted", "paid", "rejected", "returned",
];

/// Valid available_by values
const VALID_AVAILABLE_BY: &[&str] = &[
    "payment", "acceptance", "negotiation", "deferred_payment", "mixed",
];

/// Letter of Credit engine
pub struct LetterOfCreditEngine {
    repository: Arc<dyn LetterOfCreditRepository>,
}

impl LetterOfCreditEngine {
    pub fn new(repository: Arc<dyn LetterOfCreditRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Letter of Credit CRUD
    // ========================================================================

    /// Create a new letter of credit (in draft status)
    pub async fn create_lc(
        &self,
        org_id: Uuid,
        lc_number: &str,
        lc_type: &str,
        lc_form: &str,
        description: Option<&str>,
        applicant_name: &str,
        applicant_address: Option<&str>,
        applicant_bank_name: &str,
        applicant_bank_swift: Option<&str>,
        beneficiary_name: &str,
        beneficiary_address: Option<&str>,
        beneficiary_bank_name: Option<&str>,
        beneficiary_bank_swift: Option<&str>,
        advising_bank_name: Option<&str>,
        advising_bank_swift: Option<&str>,
        confirming_bank_name: Option<&str>,
        confirming_bank_swift: Option<&str>,
        lc_amount: &str,
        currency_code: &str,
        tolerance_plus: &str,
        tolerance_minus: &str,
        available_with: Option<&str>,
        available_by: &str,
        draft_at: Option<&str>,
        expiry_date: chrono::NaiveDate,
        place_of_expiry: Option<&str>,
        partial_shipments: &str,
        transshipment: &str,
        port_of_loading: Option<&str>,
        port_of_discharge: Option<&str>,
        shipment_period: Option<chrono::NaiveDate>,
        latest_shipment_date: Option<chrono::NaiveDate>,
        goods_description: Option<&str>,
        incoterms: Option<&str>,
        additional_conditions: Option<&str>,
        bank_charges: &str,
        reference_po_number: Option<&str>,
        reference_contract_number: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LetterOfCredit> {
        // Validate
        if lc_number.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("LC number is required".to_string()));
        }
        if !VALID_LC_TYPES.contains(&lc_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid LC type: {}", lc_type)));
        }
        if !VALID_LC_FORMS.contains(&lc_form) {
            return Err(AtlasError::ValidationFailed(format!("Invalid LC form: {}", lc_form)));
        }
        if !VALID_AVAILABLE_BY.contains(&available_by) {
            return Err(AtlasError::ValidationFailed(format!("Invalid available_by: {}", available_by)));
        }
        let amount_val: f64 = lc_amount.parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid LC amount".to_string()))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed("LC amount must be positive".to_string()));
        }
        if applicant_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Applicant name is required".to_string()));
        }
        if applicant_bank_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Applicant bank name is required".to_string()));
        }
        if beneficiary_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Beneficiary name is required".to_string()));
        }

        // Check uniqueness
        if let Some(_) = self.repository.get_lc_by_number(org_id, lc_number).await? {
            return Err(AtlasError::ValidationFailed(format!("LC number already exists: {}", lc_number)));
        }

        let lc = LetterOfCredit {
            id: Uuid::new_v4(),
            org_id,
            lc_number: lc_number.to_string(),
            lc_type: lc_type.to_string(),
            lc_form: lc_form.to_string(),
            description: description.map(|s| s.to_string()),
            applicant_name: applicant_name.to_string(),
            applicant_address: applicant_address.map(|s| s.to_string()),
            applicant_bank_name: applicant_bank_name.to_string(),
            applicant_bank_swift: applicant_bank_swift.map(|s| s.to_string()),
            beneficiary_name: beneficiary_name.to_string(),
            beneficiary_address: beneficiary_address.map(|s| s.to_string()),
            beneficiary_bank_name: beneficiary_bank_name.map(|s| s.to_string()),
            beneficiary_bank_swift: beneficiary_bank_swift.map(|s| s.to_string()),
            advising_bank_name: advising_bank_name.map(|s| s.to_string()),
            advising_bank_swift: advising_bank_swift.map(|s| s.to_string()),
            confirming_bank_name: confirming_bank_name.map(|s| s.to_string()),
            confirming_bank_swift: confirming_bank_swift.map(|s| s.to_string()),
            lc_amount: lc_amount.to_string(),
            currency_code: currency_code.to_string(),
            tolerance_plus: tolerance_plus.to_string(),
            tolerance_minus: tolerance_minus.to_string(),
            available_with: available_with.map(|s| s.to_string()),
            available_by: available_by.to_string(),
            draft_at: draft_at.map(|s| s.to_string()),
            issue_date: None,
            expiry_date,
            place_of_expiry: place_of_expiry.map(|s| s.to_string()),
            partial_shipments: partial_shipments.to_string(),
            transshipment: transshipment.to_string(),
            port_of_loading: port_of_loading.map(|s| s.to_string()),
            port_of_discharge: port_of_discharge.map(|s| s.to_string()),
            shipment_period,
            latest_shipment_date,
            goods_description: goods_description.map(|s| s.to_string()),
            incoterms: incoterms.map(|s| s.to_string()),
            additional_conditions: additional_conditions.map(|s| s.to_string()),
            bank_charges: bank_charges.to_string(),
            status: "draft".to_string(),
            amendment_count: 0,
            latest_amendment_number: None,
            reference_po_number: reference_po_number.map(|s| s.to_string()),
            reference_contract_number: reference_contract_number.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by_id: created_by,
            approved_by_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        info!("Creating letter of credit {} for org {}", lc_number, org_id);
        self.repository.create_lc(&lc).await
    }

    /// Get LC by number
    pub async fn get_lc_by_number(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<Option<LetterOfCredit>> {
        self.repository.get_lc_by_number(org_id, lc_number).await
    }

    /// Get LC by ID
    pub async fn get_lc_by_id(&self, id: Uuid) -> AtlasResult<Option<LetterOfCredit>> {
        self.repository.get_lc_by_id(id).await
    }

    /// List LCs with optional filters
    pub async fn list_lcs(&self, org_id: Uuid, status: Option<&str>, lc_type: Option<&str>) -> AtlasResult<Vec<LetterOfCredit>> {
        self.repository.list_lcs(org_id, status, lc_type).await
    }

    /// Delete a draft LC
    pub async fn delete_lc(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<()> {
        self.repository.delete_lc(org_id, lc_number).await
    }

    // ========================================================================
    // LC Lifecycle Actions
    // ========================================================================

    /// Issue a draft LC (set issue date, move to issued status)
    pub async fn issue_lc(&self, id: Uuid, issue_date: chrono::NaiveDate) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if lc.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!("Cannot issue LC in status '{}'. Must be in 'draft' status.", lc.status)));
        }

        self.repository.update_lc_issue(id, issue_date).await?;
        let updated = self.repository.update_lc_status(id, "issued", None).await?;
        info!("Issued letter of credit {} (id: {})", lc.lc_number, id);
        Ok(updated)
    }

    /// Advise an issued LC
    pub async fn advise_lc(&self, id: Uuid) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if lc.status != "issued" {
            return Err(AtlasError::ValidationFailed(format!("Cannot advise LC in status '{}'. Must be in 'issued' status.", lc.status)));
        }

        let updated = self.repository.update_lc_status(id, "advised", None).await?;
        info!("Advised letter of credit {}", lc.lc_number);
        Ok(updated)
    }

    /// Confirm an advised LC
    pub async fn confirm_lc(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if lc.status != "advised" {
            return Err(AtlasError::ValidationFailed(format!("Cannot confirm LC in status '{}'. Must be in 'advised' status.", lc.status)));
        }

        let updated = self.repository.update_lc_status(id, "confirmed", Some(approved_by)).await?;
        info!("Confirmed letter of credit {}", lc.lc_number);
        Ok(updated)
    }

    /// Accept an LC presentation
    pub async fn accept_lc(&self, id: Uuid) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if !["issued", "advised", "confirmed"].contains(&lc.status.as_str()) {
            return Err(AtlasError::ValidationFailed(format!("Cannot accept LC in status '{}'.", lc.status)));
        }

        let updated = self.repository.update_lc_status(id, "accepted", None).await?;
        info!("Accepted letter of credit {}", lc.lc_number);
        Ok(updated)
    }

    /// Pay an accepted LC
    pub async fn pay_lc(&self, id: Uuid) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if lc.status != "accepted" {
            return Err(AtlasError::ValidationFailed(format!("Cannot pay LC in status '{}'. Must be in 'accepted' status.", lc.status)));
        }

        let updated = self.repository.update_lc_status(id, "paid", None).await?;
        info!("Paid letter of credit {}", lc.lc_number);
        Ok(updated)
    }

    /// Cancel a draft LC
    pub async fn cancel_lc(&self, id: Uuid) -> AtlasResult<LetterOfCredit> {
        let lc = self.repository.get_lc_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if ["paid", "expired", "cancelled"].contains(&lc.status.as_str()) {
            return Err(AtlasError::ValidationFailed(format!("Cannot cancel LC in terminal status '{}'.", lc.status)));
        }

        let updated = self.repository.update_lc_status(id, "cancelled", None).await?;
        info!("Cancelled letter of credit {}", lc.lc_number);
        Ok(updated)
    }

    /// Process expired LCs
    pub async fn process_expired(&self, org_id: Uuid, as_of_date: chrono::NaiveDate) -> AtlasResult<Vec<LetterOfCredit>> {
        let lcs = self.repository.list_lcs(org_id, None, None).await?;
        let mut expired = Vec::new();
        for lc in lcs {
            if lc.expiry_date <= as_of_date && !["paid", "expired", "cancelled", "draft"].contains(&lc.status.as_str()) {
                let updated = self.repository.update_lc_status(lc.id, "expired", None).await?;
                expired.push(updated);
            }
        }
        if !expired.is_empty() {
            info!("Processed {} expired LCs for org {}", expired.len(), org_id);
        }
        Ok(expired)
    }

    // ========================================================================
    // Amendments
    // ========================================================================

    /// Create an amendment
    pub async fn create_amendment(
        &self,
        org_id: Uuid,
        lc_id: Uuid,
        amendment_type: &str,
        previous_amount: Option<&str>,
        new_amount: Option<&str>,
        previous_expiry_date: Option<chrono::NaiveDate>,
        new_expiry_date: Option<chrono::NaiveDate>,
        previous_terms: Option<&str>,
        new_terms: Option<&str>,
        reason: Option<&str>,
        bank_reference: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LcAmendment> {
        let lc = self.repository.get_lc_by_id(lc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if ["draft", "cancelled", "expired", "paid"].contains(&lc.status.as_str()) {
            return Err(AtlasError::ValidationFailed(format!("Cannot amend LC in status '{}'.", lc.status)));
        }

        if !VALID_AMENDMENT_TYPES.contains(&amendment_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid amendment type: {}", amendment_type)));
        }

        let amendment_number = format!("AMD-{:03}", lc.amendment_count + 1);

        let amendment = LcAmendment {
            id: Uuid::new_v4(),
            org_id,
            lc_id,
            lc_number: lc.lc_number.clone(),
            amendment_number: amendment_number.clone(),
            amendment_type: amendment_type.to_string(),
            previous_amount: previous_amount.map(|s| s.to_string()),
            new_amount: new_amount.map(|s| s.to_string()),
            previous_expiry_date,
            new_expiry_date,
            previous_terms: previous_terms.map(|s| s.to_string()),
            new_terms: new_terms.map(|s| s.to_string()),
            reason: reason.map(|s| s.to_string()),
            bank_reference: bank_reference.map(|s| s.to_string()),
            status: "draft".to_string(),
            effective_date,
            approved_by_id: None,
            created_by_id: created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created = self.repository.create_amendment(&amendment).await?;
        info!("Created amendment {} for LC {}", amendment_number, lc.lc_number);
        Ok(created)
    }

    /// Approve an amendment (applies changes to the LC)
    pub async fn approve_amendment(&self, amendment_id: Uuid, approved_by: Uuid) -> AtlasResult<LcAmendment> {
        let amendment = self.repository.get_amendment_by_id(amendment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Amendment not found".to_string()))?;

        if amendment.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!("Cannot approve amendment in status '{}'.", amendment.status)));
        }

        // Update amendment status
        let updated = self.repository.update_amendment_status(amendment_id, "approved", Some(approved_by)).await?;

        // Apply amendment to LC
        self.repository.update_lc_from_amendment(
            amendment.lc_id,
            amendment.new_amount.as_deref(),
            amendment.new_expiry_date,
        ).await?;

        self.repository.increment_amendment_count(amendment.lc_id, &amendment.amendment_number).await?;

        info!("Approved amendment {} for LC {}", amendment.amendment_number, amendment.lc_number);
        Ok(updated)
    }

    /// Reject an amendment
    pub async fn reject_amendment(&self, amendment_id: Uuid) -> AtlasResult<LcAmendment> {
        let amendment = self.repository.get_amendment_by_id(amendment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Amendment not found".to_string()))?;

        if amendment.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!("Cannot reject amendment in status '{}'.", amendment.status)));
        }

        let updated = self.repository.update_amendment_status(amendment_id, "rejected", None).await?;
        info!("Rejected amendment {} for LC {}", amendment.amendment_number, amendment.lc_number);
        Ok(updated)
    }

    /// List amendments for an LC
    pub async fn list_amendments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcAmendment>> {
        self.repository.list_amendments(lc_id).await
    }

    // ========================================================================
    // Required Documents
    // ========================================================================

    /// Add a required document to an LC
    pub async fn add_required_document(
        &self,
        org_id: Uuid,
        lc_id: Uuid,
        document_type: &str,
        document_code: Option<&str>,
        description: Option<&str>,
        original_copies: i32,
        copy_count: i32,
        is_mandatory: bool,
        special_instructions: Option<&str>,
    ) -> AtlasResult<LcRequiredDocument> {
        let _lc = self.repository.get_lc_by_id(lc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if document_type.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Document type is required".to_string()));
        }

        let doc = LcRequiredDocument {
            id: Uuid::new_v4(),
            org_id,
            lc_id,
            document_type: document_type.to_string(),
            document_code: document_code.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            original_copies,
            copy_count,
            is_mandatory,
            special_instructions: special_instructions.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create_required_document(&doc).await
    }

    /// List required documents for an LC
    pub async fn list_required_documents(&self, lc_id: Uuid) -> AtlasResult<Vec<LcRequiredDocument>> {
        self.repository.list_required_documents(lc_id).await
    }

    /// Delete a required document
    pub async fn delete_required_document(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_required_document(id).await
    }

    // ========================================================================
    // Shipments
    // ========================================================================

    /// Create a shipment for an LC
    pub async fn create_shipment(
        &self,
        org_id: Uuid,
        lc_id: Uuid,
        shipment_number: &str,
        vessel_name: Option<&str>,
        voyage_number: Option<&str>,
        bill_of_lading_number: Option<&str>,
        carrier_name: Option<&str>,
        port_of_loading: Option<&str>,
        port_of_discharge: Option<&str>,
        shipment_date: Option<chrono::NaiveDate>,
        expected_arrival_date: Option<chrono::NaiveDate>,
        goods_description: Option<&str>,
        quantity: Option<&str>,
        unit_price: Option<&str>,
        shipment_amount: &str,
        currency_code: &str,
        notes: Option<&str>,
    ) -> AtlasResult<LcShipment> {
        let _lc = self.repository.get_lc_by_id(lc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if shipment_number.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Shipment number is required".to_string()));
        }
        let amt: f64 = shipment_amount.parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid shipment amount".to_string()))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Shipment amount must be positive".to_string()));
        }

        let shipment = LcShipment {
            id: Uuid::new_v4(),
            org_id,
            lc_id,
            shipment_number: shipment_number.to_string(),
            vessel_name: vessel_name.map(|s| s.to_string()),
            voyage_number: voyage_number.map(|s| s.to_string()),
            bill_of_lading_number: bill_of_lading_number.map(|s| s.to_string()),
            carrier_name: carrier_name.map(|s| s.to_string()),
            port_of_loading: port_of_loading.map(|s| s.to_string()),
            port_of_discharge: port_of_discharge.map(|s| s.to_string()),
            shipment_date,
            expected_arrival_date,
            actual_arrival_date: None,
            shipping_marks: None,
            container_numbers: None,
            goods_description: goods_description.map(|s| s.to_string()),
            quantity: quantity.map(|s| s.to_string()),
            unit_price: unit_price.map(|s| s.to_string()),
            shipment_amount: shipment_amount.to_string(),
            currency_code: currency_code.to_string(),
            status: "pending".to_string(),
            notes: notes.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        info!("Creating shipment {} for LC", shipment_number);
        self.repository.create_shipment(&shipment).await
    }

    /// List shipments for an LC
    pub async fn list_shipments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcShipment>> {
        self.repository.list_shipments(lc_id).await
    }

    /// Update shipment status
    pub async fn update_shipment_status(&self, shipment_id: Uuid, status: &str) -> AtlasResult<LcShipment> {
        if !VALID_SHIPMENT_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!("Invalid shipment status: {}", status)));
        }
        self.repository.get_shipment_by_id(shipment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Shipment not found".to_string()))?;
        self.repository.update_shipment_status(shipment_id, status).await
    }

    // ========================================================================
    // Presentations
    // ========================================================================

    /// Create a presentation for an LC
    pub async fn create_presentation(
        &self,
        org_id: Uuid,
        lc_id: Uuid,
        presentation_number: &str,
        shipment_id: Option<Uuid>,
        presentation_date: chrono::NaiveDate,
        presenting_bank_name: Option<&str>,
        total_amount: &str,
        currency_code: &str,
        discrepant: bool,
        discrepancies: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LcPresentation> {
        let lc = self.repository.get_lc_by_id(lc_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Letter of credit not found".to_string()))?;

        if ["draft", "cancelled", "expired", "paid"].contains(&lc.status.as_str()) {
            return Err(AtlasError::ValidationFailed(format!("Cannot create presentation for LC in status '{}'.", lc.status)));
        }

        if presentation_number.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Presentation number is required".to_string()));
        }
        let amt: f64 = total_amount.parse()
            .map_err(|_| AtlasError::ValidationFailed("Invalid presentation amount".to_string()))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Presentation amount must be positive".to_string()));
        }

        let presentation = LcPresentation {
            id: Uuid::new_v4(),
            org_id,
            lc_id,
            presentation_number: presentation_number.to_string(),
            shipment_id,
            presentation_date,
            presenting_bank_name: presenting_bank_name.map(|s| s.to_string()),
            total_amount: total_amount.to_string(),
            currency_code: currency_code.to_string(),
            document_count: 0,
            discrepant,
            discrepancies: discrepancies.map(|s| s.to_string()),
            bank_response: None,
            response_date: None,
            payment_due_date: None,
            payment_date: None,
            paid_amount: None,
            status: "submitted".to_string(),
            notes: notes.map(|s| s.to_string()),
            created_by_id: created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        info!("Creating presentation {} for LC {}", presentation_number, lc.lc_number);
        self.repository.create_presentation(&presentation).await
    }

    /// List presentations for an LC
    pub async fn list_presentations(&self, lc_id: Uuid) -> AtlasResult<Vec<LcPresentation>> {
        self.repository.list_presentations(lc_id).await
    }

    /// Accept a presentation (compliant documents)
    pub async fn accept_presentation(&self, id: Uuid) -> AtlasResult<LcPresentation> {
        let pres = self.repository.get_presentation_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Presentation not found".to_string()))?;

        if pres.status != "submitted" && pres.status != "under_review" {
            return Err(AtlasError::ValidationFailed(format!("Cannot accept presentation in status '{}'.", pres.status)));
        }

        let updated = self.repository.update_presentation_status(id, "accepted").await?;
        info!("Accepted presentation {} for LC", pres.presentation_number);
        Ok(updated)
    }

    /// Pay a presentation
    pub async fn pay_presentation(&self, id: Uuid, paid_amount: &str, payment_date: chrono::NaiveDate) -> AtlasResult<LcPresentation> {
        let pres = self.repository.get_presentation_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Presentation not found".to_string()))?;

        if pres.status != "accepted" {
            return Err(AtlasError::ValidationFailed(format!("Cannot pay presentation in status '{}'. Must be 'accepted'.", pres.status)));
        }

        self.repository.update_presentation_payment(id, paid_amount, payment_date).await?;
        let updated = self.repository.update_presentation_status(id, "paid").await?;
        info!("Paid presentation {} amount {}", pres.presentation_number, paid_amount);
        Ok(updated)
    }

    /// Reject a presentation
    pub async fn reject_presentation(&self, id: Uuid) -> AtlasResult<LcPresentation> {
        let pres = self.repository.get_presentation_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Presentation not found".to_string()))?;

        if pres.status != "submitted" && pres.status != "under_review" {
            return Err(AtlasError::ValidationFailed(format!("Cannot reject presentation in status '{}'.", pres.status)));
        }

        let updated = self.repository.update_presentation_status(id, "rejected").await?;
        info!("Rejected presentation {}", pres.presentation_number);
        Ok(updated)
    }

    // ========================================================================
    // Presentation Documents
    // ========================================================================

    /// Add a document to a presentation
    pub async fn add_presentation_document(
        &self,
        org_id: Uuid,
        presentation_id: Uuid,
        required_document_id: Option<Uuid>,
        document_type: &str,
        document_reference: Option<&str>,
        description: Option<&str>,
        original_copies: i32,
        copy_count: i32,
        is_compliant: bool,
        discrepancies: Option<&str>,
    ) -> AtlasResult<LcPresentationDocument> {
        let _pres = self.repository.get_presentation_by_id(presentation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Presentation not found".to_string()))?;

        if document_type.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Document type is required".to_string()));
        }

        let doc = LcPresentationDocument {
            id: Uuid::new_v4(),
            org_id,
            presentation_id,
            required_document_id,
            document_type: document_type.to_string(),
            document_reference: document_reference.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            original_copies,
            copy_count,
            is_compliant,
            discrepancies: discrepancies.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repository.create_presentation_document(&doc).await
    }

    /// List documents for a presentation
    pub async fn list_presentation_documents(&self, presentation_id: Uuid) -> AtlasResult<Vec<LcPresentationDocument>> {
        self.repository.list_presentation_documents(presentation_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get LC dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LcDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}
