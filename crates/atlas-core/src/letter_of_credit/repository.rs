//! Letter of Credit Repository
//!
//! PostgreSQL storage for letters of credit, amendments, required documents,
//! shipments, presentations, and presentation documents.

use atlas_shared::{
    LetterOfCredit, LcAmendment, LcRequiredDocument, LcShipment,
    LcPresentation, LcPresentationDocument, LcDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
// chrono::DateTime, chrono::Utc used implicitly via sqlx::FromRow

/// Repository trait for letter of credit data storage
#[async_trait]
pub trait LetterOfCreditRepository: Send + Sync {
    // LC CRUD
    async fn create_lc(&self, lc: &LetterOfCredit) -> AtlasResult<LetterOfCredit>;
    async fn get_lc_by_id(&self, id: Uuid) -> AtlasResult<Option<LetterOfCredit>>;
    async fn get_lc_by_number(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<Option<LetterOfCredit>>;
    async fn list_lcs(&self, org_id: Uuid, status: Option<&str>, lc_type: Option<&str>) -> AtlasResult<Vec<LetterOfCredit>>;
    async fn update_lc_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<LetterOfCredit>;
    async fn update_lc_issue(&self, id: Uuid, issue_date: chrono::NaiveDate) -> AtlasResult<()>;
    async fn increment_amendment_count(&self, id: Uuid, latest_amendment_number: &str) -> AtlasResult<()>;
    async fn update_lc_from_amendment(&self, id: Uuid, new_amount: Option<&str>, new_expiry: Option<chrono::NaiveDate>) -> AtlasResult<()>;
    async fn delete_lc(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<()>;

    // Amendments
    async fn create_amendment(&self, amendment: &LcAmendment) -> AtlasResult<LcAmendment>;
    async fn get_amendment_by_id(&self, id: Uuid) -> AtlasResult<Option<LcAmendment>>;
    async fn list_amendments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcAmendment>>;
    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<LcAmendment>;
    async fn count_pending_amendments(&self, org_id: Uuid) -> AtlasResult<i64>;

    // Required Documents
    async fn create_required_document(&self, doc: &LcRequiredDocument) -> AtlasResult<LcRequiredDocument>;
    async fn list_required_documents(&self, lc_id: Uuid) -> AtlasResult<Vec<LcRequiredDocument>>;
    async fn delete_required_document(&self, id: Uuid) -> AtlasResult<()>;

    // Shipments
    async fn create_shipment(&self, shipment: &LcShipment) -> AtlasResult<LcShipment>;
    async fn get_shipment_by_id(&self, id: Uuid) -> AtlasResult<Option<LcShipment>>;
    async fn list_shipments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcShipment>>;
    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<LcShipment>;

    // Presentations
    async fn create_presentation(&self, presentation: &LcPresentation) -> AtlasResult<LcPresentation>;
    async fn get_presentation_by_id(&self, id: Uuid) -> AtlasResult<Option<LcPresentation>>;
    async fn list_presentations(&self, lc_id: Uuid) -> AtlasResult<Vec<LcPresentation>>;
    async fn update_presentation_status(&self, id: Uuid, status: &str) -> AtlasResult<LcPresentation>;
    async fn update_presentation_payment(&self, id: Uuid, paid_amount: &str, payment_date: chrono::NaiveDate) -> AtlasResult<()>;

    // Presentation Documents
    async fn create_presentation_document(&self, doc: &LcPresentationDocument) -> AtlasResult<LcPresentationDocument>;
    async fn list_presentation_documents(&self, presentation_id: Uuid) -> AtlasResult<Vec<LcPresentationDocument>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LcDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresLetterOfCreditRepository {
    pool: PgPool,
}

impl PostgresLetterOfCreditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_lc(row: &sqlx::postgres::PgRow) -> LetterOfCredit {
        LetterOfCredit {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            lc_number: row.get("lc_number"),
            lc_type: row.get("lc_type"),
            lc_form: row.get("lc_form"),
            description: row.get("description"),
            applicant_name: row.get("applicant_name"),
            applicant_address: row.get("applicant_address"),
            applicant_bank_name: row.get("applicant_bank_name"),
            applicant_bank_swift: row.get("applicant_bank_swift"),
            beneficiary_name: row.get("beneficiary_name"),
            beneficiary_address: row.get("beneficiary_address"),
            beneficiary_bank_name: row.get("beneficiary_bank_name"),
            beneficiary_bank_swift: row.get("beneficiary_bank_swift"),
            advising_bank_name: row.get("advising_bank_name"),
            advising_bank_swift: row.get("advising_bank_swift"),
            confirming_bank_name: row.get("confirming_bank_name"),
            confirming_bank_swift: row.get("confirming_bank_swift"),
            lc_amount: row.get("lc_amount"),
            currency_code: row.get("currency_code"),
            tolerance_plus: row.get("tolerance_plus"),
            tolerance_minus: row.get("tolerance_minus"),
            available_with: row.get("available_with"),
            available_by: row.get("available_by"),
            draft_at: row.get("draft_at"),
            issue_date: row.get("issue_date"),
            expiry_date: row.get("expiry_date"),
            place_of_expiry: row.get("place_of_expiry"),
            partial_shipments: row.get("partial_shipments"),
            transshipment: row.get("transshipment"),
            port_of_loading: row.get("port_of_loading"),
            port_of_discharge: row.get("port_of_discharge"),
            shipment_period: row.get("shipment_period"),
            latest_shipment_date: row.get("latest_shipment_date"),
            goods_description: row.get("goods_description"),
            incoterms: row.get("incoterms"),
            additional_conditions: row.get("additional_conditions"),
            bank_charges: row.get("bank_charges"),
            status: row.get("status"),
            amendment_count: row.get("amendment_count"),
            latest_amendment_number: row.get("latest_amendment_number"),
            reference_po_number: row.get("reference_po_number"),
            reference_contract_number: row.get("reference_contract_number"),
            notes: row.get("notes"),
            created_by_id: row.get("created_by"),
            approved_by_id: row.get("approved_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_amendment(row: &sqlx::postgres::PgRow) -> LcAmendment {
        LcAmendment {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            lc_id: row.get("lc_id"),
            lc_number: row.get("lc_number"),
            amendment_number: row.get("amendment_number"),
            amendment_type: row.get("amendment_type"),
            previous_amount: row.get("previous_amount"),
            new_amount: row.get("new_amount"),
            previous_expiry_date: row.get("previous_expiry_date"),
            new_expiry_date: row.get("new_expiry_date"),
            previous_terms: row.get("previous_terms"),
            new_terms: row.get("new_terms"),
            reason: row.get("reason"),
            bank_reference: row.get("bank_reference"),
            status: row.get("status"),
            effective_date: row.get("effective_date"),
            approved_by_id: row.get("approved_by"),
            created_by_id: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_required_doc(row: &sqlx::postgres::PgRow) -> LcRequiredDocument {
        LcRequiredDocument {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            lc_id: row.get("lc_id"),
            document_type: row.get("document_type"),
            document_code: row.get("document_code"),
            description: row.get("description"),
            original_copies: row.get("original_copies"),
            copy_count: row.get("copy_count"),
            is_mandatory: row.get("is_mandatory"),
            special_instructions: row.get("special_instructions"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_shipment(row: &sqlx::postgres::PgRow) -> LcShipment {
        LcShipment {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            lc_id: row.get("lc_id"),
            shipment_number: row.get("shipment_number"),
            vessel_name: row.get("vessel_name"),
            voyage_number: row.get("voyage_number"),
            bill_of_lading_number: row.get("bill_of_lading_number"),
            carrier_name: row.get("carrier_name"),
            port_of_loading: row.get("port_of_loading"),
            port_of_discharge: row.get("port_of_discharge"),
            shipment_date: row.get("shipment_date"),
            expected_arrival_date: row.get("expected_arrival_date"),
            actual_arrival_date: row.get("actual_arrival_date"),
            shipping_marks: row.get("shipping_marks"),
            container_numbers: row.get("container_numbers"),
            goods_description: row.get("goods_description"),
            quantity: row.get("quantity"),
            unit_price: row.get("unit_price"),
            shipment_amount: row.get("shipment_amount"),
            currency_code: row.get("currency_code"),
            status: row.get("status"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_presentation(row: &sqlx::postgres::PgRow) -> LcPresentation {
        LcPresentation {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            lc_id: row.get("lc_id"),
            presentation_number: row.get("presentation_number"),
            shipment_id: row.get("shipment_id"),
            presentation_date: row.get("presentation_date"),
            presenting_bank_name: row.get("presenting_bank_name"),
            total_amount: row.get("total_amount"),
            currency_code: row.get("currency_code"),
            document_count: row.get("document_count"),
            discrepant: row.get("discrepant"),
            discrepancies: row.get("discrepancies"),
            bank_response: row.get("bank_response"),
            response_date: row.get("response_date"),
            payment_due_date: row.get("payment_due_date"),
            payment_date: row.get("payment_date"),
            paid_amount: row.get("paid_amount"),
            status: row.get("status"),
            notes: row.get("notes"),
            created_by_id: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_presentation_doc(row: &sqlx::postgres::PgRow) -> LcPresentationDocument {
        LcPresentationDocument {
            id: row.get("id"),
            org_id: row.get("organization_id"),
            presentation_id: row.get("presentation_id"),
            required_document_id: row.get("required_document_id"),
            document_type: row.get("document_type"),
            document_reference: row.get("document_reference"),
            description: row.get("description"),
            original_copies: row.get("original_copies"),
            copy_count: row.get("copy_count"),
            is_compliant: row.get("is_compliant"),
            discrepancies: row.get("discrepancies"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl LetterOfCreditRepository for PostgresLetterOfCreditRepository {
    async fn create_lc(&self, lc: &LetterOfCredit) -> AtlasResult<LetterOfCredit> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.letters_of_credit (
                id, organization_id, lc_number, lc_type, lc_form, description,
                applicant_name, applicant_address, applicant_bank_name, applicant_bank_swift,
                beneficiary_name, beneficiary_address, beneficiary_bank_name, beneficiary_bank_swift,
                advising_bank_name, advising_bank_swift, confirming_bank_name, confirming_bank_swift,
                lc_amount, currency_code, tolerance_plus, tolerance_minus,
                available_with, available_by, draft_at,
                issue_date, expiry_date, place_of_expiry,
                partial_shipments, transshipment,
                port_of_loading, port_of_discharge,
                shipment_period, latest_shipment_date,
                goods_description, incoterms, additional_conditions, bank_charges,
                status, reference_po_number, reference_contract_number,
                notes, created_by
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,
                $31,$32,$33,$34,$35,$36,$37,$38,$39,$40,
                $41,$42,$43
            ) RETURNING *
        "#)
            .bind(lc.id)
            .bind(lc.org_id)
            .bind(&lc.lc_number)
            .bind(&lc.lc_type)
            .bind(&lc.lc_form)
            .bind(&lc.description)
            .bind(&lc.applicant_name)
            .bind(&lc.applicant_address)
            .bind(&lc.applicant_bank_name)
            .bind(&lc.applicant_bank_swift)
            .bind(&lc.beneficiary_name)
            .bind(&lc.beneficiary_address)
            .bind(&lc.beneficiary_bank_name)
            .bind(&lc.beneficiary_bank_swift)
            .bind(&lc.advising_bank_name)
            .bind(&lc.advising_bank_swift)
            .bind(&lc.confirming_bank_name)
            .bind(&lc.confirming_bank_swift)
            .bind(&lc.lc_amount)
            .bind(&lc.currency_code)
            .bind(&lc.tolerance_plus)
            .bind(&lc.tolerance_minus)
            .bind(&lc.available_with)
            .bind(&lc.available_by)
            .bind(&lc.draft_at)
            .bind(lc.issue_date)
            .bind(lc.expiry_date)
            .bind(&lc.place_of_expiry)
            .bind(&lc.partial_shipments)
            .bind(&lc.transshipment)
            .bind(&lc.port_of_loading)
            .bind(&lc.port_of_discharge)
            .bind(lc.shipment_period)
            .bind(lc.latest_shipment_date)
            .bind(&lc.goods_description)
            .bind(&lc.incoterms)
            .bind(&lc.additional_conditions)
            .bind(&lc.bank_charges)
            .bind(&lc.status)
            .bind(&lc.reference_po_number)
            .bind(&lc.reference_contract_number)
            .bind(&lc.notes)
            .bind(lc.created_by_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_lc(&row))
    }

    async fn get_lc_by_id(&self, id: Uuid) -> AtlasResult<Option<LetterOfCredit>> {
        let row = sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_lc))
    }

    async fn get_lc_by_number(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<Option<LetterOfCredit>> {
        let row = sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 AND lc_number = $2")
            .bind(org_id)
            .bind(lc_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_lc))
    }

    async fn list_lcs(&self, org_id: Uuid, status: Option<&str>, lc_type: Option<&str>) -> AtlasResult<Vec<LetterOfCredit>> {
        let rows = match (status, lc_type) {
            (Some(s), Some(t)) => {
                sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 AND status = $2 AND lc_type = $3 ORDER BY created_at DESC")
                    .bind(org_id).bind(s).bind(t)
            }
            (Some(s), None) => {
                sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC")
                    .bind(org_id).bind(s)
            }
            (None, Some(t)) => {
                sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 AND lc_type = $2 ORDER BY created_at DESC")
                    .bind(org_id).bind(t)
            }
            (None, None) => {
                sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 ORDER BY created_at DESC")
                    .bind(org_id)
            }
        }
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_lc).collect())
    }

    async fn update_lc_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<LetterOfCredit> {
        let row = sqlx::query(
            "UPDATE _atlas.letters_of_credit SET status = $2, approved_by = $3, updated_at = now() WHERE id = $1 RETURNING *"
        )
            .bind(id).bind(status).bind(approved_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_lc(&row))
    }

    async fn update_lc_issue(&self, id: Uuid, issue_date: chrono::NaiveDate) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.letters_of_credit SET issue_date = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(issue_date)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn increment_amendment_count(&self, id: Uuid, latest_amendment_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.letters_of_credit SET amendment_count = amendment_count + 1, latest_amendment_number = $2, updated_at = now() WHERE id = $1"
        )
            .bind(id).bind(latest_amendment_number)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_lc_from_amendment(&self, id: Uuid, new_amount: Option<&str>, new_expiry: Option<chrono::NaiveDate>) -> AtlasResult<()> {
        match (new_amount, new_expiry) {
            (Some(amt), Some(exp)) => {
                sqlx::query("UPDATE _atlas.letters_of_credit SET lc_amount = $2, expiry_date = $3, updated_at = now() WHERE id = $1")
                    .bind(id).bind(amt).bind(exp)
            }
            (Some(amt), None) => {
                sqlx::query("UPDATE _atlas.letters_of_credit SET lc_amount = $2, updated_at = now() WHERE id = $1")
                    .bind(id).bind(amt)
            }
            (None, Some(exp)) => {
                sqlx::query("UPDATE _atlas.letters_of_credit SET expiry_date = $2, updated_at = now() WHERE id = $1")
                    .bind(id).bind(exp)
            }
            (None, None) => return Ok(()),
        }
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_lc(&self, org_id: Uuid, lc_number: &str) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.letters_of_credit WHERE organization_id = $1 AND lc_number = $2 AND status = 'draft'")
            .bind(org_id).bind(lc_number)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Letter of credit not found or not in draft status".to_string()));
        }
        Ok(())
    }

    // Amendments
    async fn create_amendment(&self, amendment: &LcAmendment) -> AtlasResult<LcAmendment> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.lc_amendments (
                id, organization_id, lc_id, lc_number, amendment_number, amendment_type,
                previous_amount, new_amount, previous_expiry_date, new_expiry_date,
                previous_terms, new_terms, reason, bank_reference,
                status, effective_date, created_by
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *
        "#)
            .bind(amendment.id)
            .bind(amendment.org_id)
            .bind(amendment.lc_id)
            .bind(&amendment.lc_number)
            .bind(&amendment.amendment_number)
            .bind(&amendment.amendment_type)
            .bind(&amendment.previous_amount)
            .bind(&amendment.new_amount)
            .bind(amendment.previous_expiry_date)
            .bind(amendment.new_expiry_date)
            .bind(&amendment.previous_terms)
            .bind(&amendment.new_terms)
            .bind(&amendment.reason)
            .bind(&amendment.bank_reference)
            .bind(&amendment.status)
            .bind(amendment.effective_date)
            .bind(amendment.created_by_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_amendment(&row))
    }

    async fn get_amendment_by_id(&self, id: Uuid) -> AtlasResult<Option<LcAmendment>> {
        let row = sqlx::query("SELECT * FROM _atlas.lc_amendments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_amendment))
    }

    async fn list_amendments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcAmendment>> {
        let rows = sqlx::query("SELECT * FROM _atlas.lc_amendments WHERE lc_id = $1 ORDER BY created_at DESC")
            .bind(lc_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_amendment).collect())
    }

    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<LcAmendment> {
        let row = sqlx::query(
            "UPDATE _atlas.lc_amendments SET status = $2, approved_by = $3, updated_at = now() WHERE id = $1 RETURNING *"
        )
            .bind(id).bind(status).bind(approved_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_amendment(&row))
    }

    async fn count_pending_amendments(&self, org_id: Uuid) -> AtlasResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM _atlas.lc_amendments WHERE organization_id = $1 AND status IN ('draft', 'pending_approval')")
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.get("cnt"))
    }

    // Required Documents
    async fn create_required_document(&self, doc: &LcRequiredDocument) -> AtlasResult<LcRequiredDocument> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.lc_required_documents (
                id, organization_id, lc_id, document_type, document_code,
                description, original_copies, copy_count, is_mandatory, special_instructions
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *
        "#)
            .bind(doc.id)
            .bind(doc.org_id)
            .bind(doc.lc_id)
            .bind(&doc.document_type)
            .bind(&doc.document_code)
            .bind(&doc.description)
            .bind(doc.original_copies)
            .bind(doc.copy_count)
            .bind(doc.is_mandatory)
            .bind(&doc.special_instructions)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_required_doc(&row))
    }

    async fn list_required_documents(&self, lc_id: Uuid) -> AtlasResult<Vec<LcRequiredDocument>> {
        let rows = sqlx::query("SELECT * FROM _atlas.lc_required_documents WHERE lc_id = $1 ORDER BY created_at")
            .bind(lc_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_required_doc).collect())
    }

    async fn delete_required_document(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.lc_required_documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Shipments
    async fn create_shipment(&self, shipment: &LcShipment) -> AtlasResult<LcShipment> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.lc_shipments (
                id, organization_id, lc_id, shipment_number,
                vessel_name, voyage_number, bill_of_lading_number, carrier_name,
                port_of_loading, port_of_discharge,
                shipment_date, expected_arrival_date, actual_arrival_date,
                shipping_marks, container_numbers, goods_description,
                quantity, unit_price, shipment_amount, currency_code,
                status, notes
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
            RETURNING *
        "#)
            .bind(shipment.id)
            .bind(shipment.org_id)
            .bind(shipment.lc_id)
            .bind(&shipment.shipment_number)
            .bind(&shipment.vessel_name)
            .bind(&shipment.voyage_number)
            .bind(&shipment.bill_of_lading_number)
            .bind(&shipment.carrier_name)
            .bind(&shipment.port_of_loading)
            .bind(&shipment.port_of_discharge)
            .bind(shipment.shipment_date)
            .bind(shipment.expected_arrival_date)
            .bind(shipment.actual_arrival_date)
            .bind(&shipment.shipping_marks)
            .bind(&shipment.container_numbers)
            .bind(&shipment.goods_description)
            .bind(&shipment.quantity)
            .bind(&shipment.unit_price)
            .bind(&shipment.shipment_amount)
            .bind(&shipment.currency_code)
            .bind(&shipment.status)
            .bind(&shipment.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_shipment(&row))
    }

    async fn get_shipment_by_id(&self, id: Uuid) -> AtlasResult<Option<LcShipment>> {
        let row = sqlx::query("SELECT * FROM _atlas.lc_shipments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_shipment))
    }

    async fn list_shipments(&self, lc_id: Uuid) -> AtlasResult<Vec<LcShipment>> {
        let rows = sqlx::query("SELECT * FROM _atlas.lc_shipments WHERE lc_id = $1 ORDER BY created_at")
            .bind(lc_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_shipment).collect())
    }

    async fn update_shipment_status(&self, id: Uuid, status: &str) -> AtlasResult<LcShipment> {
        let row = sqlx::query("UPDATE _atlas.lc_shipments SET status = $2, updated_at = now() WHERE id = $1 RETURNING *")
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_shipment(&row))
    }

    // Presentations
    async fn create_presentation(&self, presentation: &LcPresentation) -> AtlasResult<LcPresentation> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.lc_presentations (
                id, organization_id, lc_id, presentation_number, shipment_id,
                presentation_date, presenting_bank_name,
                total_amount, currency_code, document_count,
                discrepant, discrepancies, bank_response,
                response_date, payment_due_date, payment_date, paid_amount,
                status, notes, created_by
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
            RETURNING *
        "#)
            .bind(presentation.id)
            .bind(presentation.org_id)
            .bind(presentation.lc_id)
            .bind(&presentation.presentation_number)
            .bind(presentation.shipment_id)
            .bind(presentation.presentation_date)
            .bind(&presentation.presenting_bank_name)
            .bind(&presentation.total_amount)
            .bind(&presentation.currency_code)
            .bind(presentation.document_count)
            .bind(presentation.discrepant)
            .bind(&presentation.discrepancies)
            .bind(&presentation.bank_response)
            .bind(presentation.response_date)
            .bind(presentation.payment_due_date)
            .bind(presentation.payment_date)
            .bind(&presentation.paid_amount)
            .bind(&presentation.status)
            .bind(&presentation.notes)
            .bind(presentation.created_by_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_presentation(&row))
    }

    async fn get_presentation_by_id(&self, id: Uuid) -> AtlasResult<Option<LcPresentation>> {
        let row = sqlx::query("SELECT * FROM _atlas.lc_presentations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.as_ref().map(Self::row_to_presentation))
    }

    async fn list_presentations(&self, lc_id: Uuid) -> AtlasResult<Vec<LcPresentation>> {
        let rows = sqlx::query("SELECT * FROM _atlas.lc_presentations WHERE lc_id = $1 ORDER BY created_at DESC")
            .bind(lc_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_presentation).collect())
    }

    async fn update_presentation_status(&self, id: Uuid, status: &str) -> AtlasResult<LcPresentation> {
        let row = sqlx::query("UPDATE _atlas.lc_presentations SET status = $2, updated_at = now() WHERE id = $1 RETURNING *")
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_presentation(&row))
    }

    async fn update_presentation_payment(&self, id: Uuid, paid_amount: &str, payment_date: chrono::NaiveDate) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.lc_presentations SET paid_amount = $2, payment_date = $3, updated_at = now() WHERE id = $1")
            .bind(id).bind(paid_amount).bind(payment_date)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // Presentation Documents
    async fn create_presentation_document(&self, doc: &LcPresentationDocument) -> AtlasResult<LcPresentationDocument> {
        let row = sqlx::query(r#"
            INSERT INTO _atlas.lc_presentation_documents (
                id, organization_id, presentation_id, required_document_id,
                document_type, document_reference, description,
                original_copies, copy_count, is_compliant, discrepancies
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *
        "#)
            .bind(doc.id)
            .bind(doc.org_id)
            .bind(doc.presentation_id)
            .bind(doc.required_document_id)
            .bind(&doc.document_type)
            .bind(&doc.document_reference)
            .bind(&doc.description)
            .bind(doc.original_copies)
            .bind(doc.copy_count)
            .bind(doc.is_compliant)
            .bind(&doc.discrepancies)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(Self::row_to_presentation_doc(&row))
    }

    async fn list_presentation_documents(&self, presentation_id: Uuid) -> AtlasResult<Vec<LcPresentationDocument>> {
        let rows = sqlx::query("SELECT * FROM _atlas.lc_presentation_documents WHERE presentation_id = $1 ORDER BY created_at")
            .bind(presentation_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(Self::row_to_presentation_doc).collect())
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LcDashboard> {
        let active_rows = sqlx::query(
            "SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1 AND status IN ('issued', 'advised', 'confirmed', 'accepted')"
        )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: Vec<LetterOfCredit> = active_rows.iter().map(Self::row_to_lc).collect();
        let total_active_lcs = active.len() as i32;
        let total_lc_amount: String = active.iter()
            .filter_map(|lc| lc.lc_amount.parse::<f64>().ok())
            .sum::<f64>()
            .to_string();

        let total_pending_amendments = self.count_pending_amendments(org_id).await.unwrap_or(0) as i32;

        let pending_presentations: i64 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.lc_presentations p JOIN _atlas.letters_of_credit lc ON p.lc_id = lc.id WHERE lc.organization_id = $1 AND p.status IN ('submitted', 'under_review')"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0);

        let discrepant_presentations: i64 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.lc_presentations p JOIN _atlas.letters_of_credit lc ON p.lc_id = lc.id WHERE lc.organization_id = $1 AND p.discrepant = true"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0);

        let today = chrono::Utc::now().date_naive();
        let in_30 = today + chrono::Duration::days(30);
        let in_90 = today + chrono::Duration::days(90);

        let expiring_30: i64 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.letters_of_credit WHERE organization_id = $1 AND expiry_date <= $2 AND status NOT IN ('draft', 'cancelled', 'expired')"
        )
            .bind(org_id).bind(in_30)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0);

        let expiring_90: i64 = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.letters_of_credit WHERE organization_id = $1 AND expiry_date <= $2 AND status NOT IN ('draft', 'cancelled', 'expired')"
        )
            .bind(org_id).bind(in_90)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.get("cnt"))
            .unwrap_or(0);

        // by_type / by_currency / by_status
        let all_rows = sqlx::query("SELECT * FROM _atlas.letters_of_credit WHERE organization_id = $1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let all: Vec<LetterOfCredit> = all_rows.iter().map(Self::row_to_lc).collect();

        let mut by_type: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let mut by_currency: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let mut by_status: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for lc in &all {
            *by_type.entry(lc.lc_type.clone()).or_insert(0) += 1;
            *by_currency.entry(lc.currency_code.clone()).or_insert(0) += 1;
            *by_status.entry(lc.status.clone()).or_insert(0) += 1;
        }

        let by_type_json: serde_json::Map<String, serde_json::Value> = by_type.into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v))).collect();
        let by_currency_json: serde_json::Map<String, serde_json::Value> = by_currency.into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v))).collect();
        let by_status_json: serde_json::Map<String, serde_json::Value> = by_status.into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v))).collect();

        Ok(LcDashboard {
            total_active_lcs,
            total_lc_amount,
            total_pending_amendments,
            total_presentations_pending: pending_presentations as i32,
            total_discrepant_presentations: discrepant_presentations as i32,
            expiring_within_30_days: expiring_30 as i32,
            expiring_within_90_days: expiring_90 as i32,
            by_type: serde_json::Value::Object(by_type_json),
            by_currency: serde_json::Value::Object(by_currency_json),
            by_status: serde_json::Value::Object(by_status_json),
        })
    }
}
