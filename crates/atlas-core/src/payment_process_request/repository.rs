//! Payment Process Request Repository
//!
//! `PostgreSQL` storage for PPR headers, selected documents, and activity audit trail.

use async_trait::async_trait;
use atlas_shared::{AtlasError, AtlasResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// PPR header record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentProcessRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub request_name: String,
    pub description: Option<String>,
    pub payment_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub payment_method: String,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub selection_criteria: String,
    pub due_date_from: Option<chrono::NaiveDate>,
    pub due_date_to: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub pay_group: Option<String>,
    pub minimum_amount: Option<f64>,
    pub maximum_amount: Option<f64>,
    pub include_on_hold: bool,
    pub take_discount: bool,
    pub pay_only_due: bool,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub payment_document: Option<String>,
    pub total_documents: i32,
    pub total_invoice_amount: f64,
    pub total_discount_taken: f64,
    pub total_payment_amount: f64,
    pub total_currency_adjustment: f64,
    pub status: String,
    pub processing_time_ms: Option<i32>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub selection_completed_by: Option<Uuid>,
    pub selection_completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub formatted_by: Option<Uuid>,
    pub formatted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancel_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PPR selected document (invoice selected for payment)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PprSelectedDocument {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ppr_id: Uuid,
    pub line_number: i32,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_amount: f64,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub original_amount: f64,
    pub amount_due: f64,
    pub amount_to_pay: f64,
    pub discount_available: f64,
    pub discount_taken: f64,
    pub discount_date: Option<chrono::NaiveDate>,
    pub currency_code: Option<String>,
    pub net_payment: f64,
    pub remaining_balance: f64,
    pub liability_account: Option<String>,
    pub discount_account: Option<String>,
    pub cash_account: Option<String>,
    pub selected_for_payment: bool,
    pub exclude_reason: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PPR activity log record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PprActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ppr_id: Uuid,
    pub document_id: Option<Uuid>,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub details: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PPR dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PprDashboard {
    pub total_requests: i64,
    pub draft_count: i64,
    pub submitted_count: i64,
    pub selection_complete_count: i64,
    pub formatted_count: i64,
    pub confirmed_count: i64,
    pub cancelled_count: i64,
    pub total_payment_amount: f64,
    pub total_documents_processed: i64,
    pub by_payment_method: serde_json::Value,
    pub by_selection_criteria: serde_json::Value,
}

// ============================================================================
// Trait
// ============================================================================

#[async_trait]
pub trait PaymentProcessRequestRepository: Send + Sync {
    async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        request_name: &str,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        selection_criteria: &str,
        due_date_from: Option<chrono::NaiveDate>,
        due_date_to: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        pay_group: Option<&str>,
        minimum_amount: Option<f64>,
        maximum_amount: Option<f64>,
        include_on_hold: bool,
        take_discount: bool,
        pay_only_due: bool,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentProcessRequest>;

    async fn get_request(&self, org_id: Uuid, id: Uuid) -> AtlasResult<PaymentProcessRequest>;
    async fn get_request_by_number(
        &self,
        org_id: Uuid,
        request_number: &str,
    ) -> AtlasResult<PaymentProcessRequest>;
    async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PaymentProcessRequest>>;
    async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentProcessRequest>;
    async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()>;

    async fn add_document(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_amount: f64,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        original_amount: f64,
        amount_due: f64,
        amount_to_pay: f64,
        discount_available: f64,
        discount_taken: f64,
        discount_date: Option<chrono::NaiveDate>,
        currency_code: Option<&str>,
        liability_account: Option<&str>,
        discount_account: Option<&str>,
        cash_account: Option<&str>,
    ) -> AtlasResult<PprSelectedDocument>;

    async fn list_documents(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprSelectedDocument>>;
    async fn remove_document(&self, ppr_id: Uuid, document_id: Uuid) -> AtlasResult<()>;
    async fn mark_documents_paid(&self, ppr_id: Uuid) -> AtlasResult<()>;

    async fn recalculate_totals(&self, ppr_id: Uuid) -> AtlasResult<PaymentProcessRequest>;

    async fn set_submitted(
        &self,
        id: Uuid,
        submitted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest>;
    async fn set_selection_complete(
        &self,
        id: Uuid,
        completed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest>;
    async fn set_formatted(
        &self,
        id: Uuid,
        formatted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest>;
    async fn set_confirmed(
        &self,
        id: Uuid,
        confirmed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest>;
    async fn set_cancelled(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<PaymentProcessRequest>;

    async fn log_activity(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        document_id: Option<Uuid>,
        activity_type: &str,
        description: Option<&str>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        performed_by: Option<Uuid>,
        performed_by_name: Option<&str>,
        details: serde_json::Value,
    ) -> AtlasResult<()>;

    async fn list_activities(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprActivity>>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresPaymentProcessRequestRepository {
    pool: PgPool,
}

impl PostgresPaymentProcessRequestRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentProcessRequestRepository for PostgresPaymentProcessRequestRepository {
    async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        request_name: &str,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        selection_criteria: &str,
        due_date_from: Option<chrono::NaiveDate>,
        due_date_to: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        pay_group: Option<&str>,
        minimum_amount: Option<f64>,
        maximum_amount: Option<f64>,
        include_on_hold: bool,
        take_discount: bool,
        pay_only_due: bool,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentProcessRequest> {
        let row = sqlx::query_as::<_, PaymentProcessRequest>(
            r"INSERT INTO _atlas.payment_process_requests (
                organization_id, request_number, request_name, description,
                payment_date, gl_date, payment_method, currency_code,
                exchange_rate_type, exchange_rate,
                selection_criteria, due_date_from, due_date_to,
                supplier_id, supplier_name, pay_group,
                minimum_amount, maximum_amount,
                include_on_hold, take_discount, pay_only_due,
                bank_account_id, bank_account_name, payment_document,
                created_by
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25)
            RETURNING *"
        )
            .bind(org_id)
            .bind(request_number)
            .bind(request_name)
            .bind(description)
            .bind(payment_date)
            .bind(gl_date)
            .bind(payment_method)
            .bind(currency_code)
            .bind(exchange_rate_type)
            .bind(exchange_rate)
            .bind(selection_criteria)
            .bind(due_date_from)
            .bind(due_date_to)
            .bind(supplier_id)
            .bind(supplier_name)
            .bind(pay_group)
            .bind(minimum_amount)
            .bind(maximum_amount)
            .bind(include_on_hold)
            .bind(take_discount)
            .bind(pay_only_due)
            .bind(bank_account_id)
            .bind(bank_account_name)
            .bind(payment_document)
            .bind(created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    async fn get_request(&self, org_id: Uuid, id: Uuid) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            "SELECT * FROM _atlas.payment_process_requests WHERE id = $1 AND organization_id = $2",
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AtlasError::EntityNotFound("Payment process request not found".to_string()))
    }

    async fn get_request_by_number(
        &self,
        org_id: Uuid,
        request_number: &str,
    ) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            "SELECT * FROM _atlas.payment_process_requests WHERE request_number = $1 AND organization_id = $2"
        )
            .bind(request_number).bind(org_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AtlasError::EntityNotFound("Payment process request not found".to_string()))
    }

    async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<PaymentProcessRequest>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, PaymentProcessRequest>(
                "SELECT * FROM _atlas.payment_process_requests WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query_as::<_, PaymentProcessRequest>(
                "SELECT * FROM _atlas.payment_process_requests WHERE organization_id = $1 ORDER BY created_at DESC"
            )
                .bind(org_id)
                .fetch_all(&self.pool).await
        };
        rows.map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            "UPDATE _atlas.payment_process_requests SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.payment_process_requests WHERE request_number = $1 AND organization_id = $2 AND status = 'draft'"
        )
            .bind(request_number).bind(org_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::ValidationFailed(
                "Cannot delete: request not found or not in draft status".to_string(),
            ));
        }
        Ok(())
    }

    async fn add_document(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_amount: f64,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        original_amount: f64,
        amount_due: f64,
        amount_to_pay: f64,
        discount_available: f64,
        discount_taken: f64,
        discount_date: Option<chrono::NaiveDate>,
        currency_code: Option<&str>,
        liability_account: Option<&str>,
        discount_account: Option<&str>,
        cash_account: Option<&str>,
    ) -> AtlasResult<PprSelectedDocument> {
        // Get next line number
        let max_line: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(line_number), 0) FROM _atlas.ppr_selected_documents WHERE ppr_id = $1"
        )
            .bind(ppr_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let net_payment = amount_to_pay - discount_taken;
        let remaining_balance = amount_due - amount_to_pay;

        let row = sqlx::query_as::<_, PprSelectedDocument>(
            r"INSERT INTO _atlas.ppr_selected_documents (
                organization_id, ppr_id, line_number,
                invoice_id, invoice_number, invoice_date, invoice_amount,
                supplier_id, supplier_number, supplier_name, supplier_site,
                original_amount, amount_due, amount_to_pay,
                discount_available, discount_taken, discount_date,
                currency_code, net_payment, remaining_balance,
                liability_account, discount_account, cash_account
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)
            RETURNING *"
        )
            .bind(org_id)
            .bind(ppr_id)
            .bind(max_line + 1)
            .bind(invoice_id)
            .bind(invoice_number)
            .bind(invoice_date)
            .bind(invoice_amount)
            .bind(supplier_id)
            .bind(supplier_number)
            .bind(supplier_name)
            .bind(supplier_site)
            .bind(original_amount)
            .bind(amount_due)
            .bind(amount_to_pay)
            .bind(discount_available)
            .bind(discount_taken)
            .bind(discount_date)
            .bind(currency_code)
            .bind(net_payment)
            .bind(remaining_balance)
            .bind(liability_account)
            .bind(discount_account)
            .bind(cash_account)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Recalculate PPR totals
        self.recalculate_totals(ppr_id).await?;

        Ok(row)
    }

    async fn list_documents(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprSelectedDocument>> {
        sqlx::query_as::<_, PprSelectedDocument>(
            "SELECT * FROM _atlas.ppr_selected_documents WHERE ppr_id = $1 ORDER BY line_number",
        )
        .bind(ppr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn remove_document(&self, ppr_id: Uuid, document_id: Uuid) -> AtlasResult<()> {
        let result =
            sqlx::query("DELETE FROM _atlas.ppr_selected_documents WHERE id = $1 AND ppr_id = $2")
                .bind(document_id)
                .bind(ppr_id)
                .execute(&self.pool)
                .await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Document not found".to_string()));
        }
        self.recalculate_totals(ppr_id).await?;
        Ok(())
    }

    async fn mark_documents_paid(&self, ppr_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.ppr_selected_documents SET status = 'paid', updated_at = now() WHERE ppr_id = $1 AND status = 'selected'"
        )
            .bind(ppr_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn recalculate_totals(&self, ppr_id: Uuid) -> AtlasResult<PaymentProcessRequest> {
        let totals = sqlx::query(
            r"SELECT
                COUNT(*) as total_documents,
                COALESCE(SUM(original_amount), 0) as total_invoice_amount,
                COALESCE(SUM(discount_taken), 0) as total_discount_taken,
                COALESCE(SUM(net_payment), 0) as total_payment_amount
            FROM _atlas.ppr_selected_documents
            WHERE ppr_id = $1 AND selected_for_payment = true",
        )
        .bind(ppr_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_docs: i64 = totals.get("total_documents");
        let total_invoice: f64 = totals.get("total_invoice_amount");
        let total_discount: f64 = totals.get("total_discount_taken");
        let total_payment: f64 = totals.get("total_payment_amount");

        sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET total_documents = $2, total_invoice_amount = $3,
                total_discount_taken = $4, total_payment_amount = $5,
                updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(ppr_id)
        .bind(total_docs as i32)
        .bind(total_invoice)
        .bind(total_discount)
        .bind(total_payment)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn set_submitted(
        &self,
        id: Uuid,
        submitted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET status = 'submitted', submitted_by = $2, submitted_at = now(), updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(submitted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn set_selection_complete(
        &self,
        id: Uuid,
        completed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        let start = std::time::Instant::now();
        let result = sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET status = 'selection_complete', selection_completed_by = $2,
                selection_completed_at = now(), processing_time_ms = $3, updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(completed_by)
        .bind(start.elapsed().as_millis() as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(result)
    }

    async fn set_formatted(
        &self,
        id: Uuid,
        formatted_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET status = 'formatted', formatted_by = $2, formatted_at = now(), updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(formatted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn set_confirmed(
        &self,
        id: Uuid,
        confirmed_by: Uuid,
    ) -> AtlasResult<PaymentProcessRequest> {
        // Mark all selected documents as paid
        self.mark_documents_paid(id).await?;

        sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET status = 'confirmed', confirmed_by = $2, confirmed_at = now(), updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(confirmed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn set_cancelled(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<PaymentProcessRequest> {
        sqlx::query_as::<_, PaymentProcessRequest>(
            r"UPDATE _atlas.payment_process_requests
            SET status = 'cancelled', cancelled_by = $2, cancelled_at = now(),
                cancel_reason = $3, updated_at = now()
            WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(cancelled_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn log_activity(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        document_id: Option<Uuid>,
        activity_type: &str,
        description: Option<&str>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        performed_by: Option<Uuid>,
        performed_by_name: Option<&str>,
        details: serde_json::Value,
    ) -> AtlasResult<()> {
        sqlx::query(
            r"INSERT INTO _atlas.ppr_activities (
                organization_id, ppr_id, document_id,
                activity_type, description, old_status, new_status,
                performed_by, performed_by_name, details
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(org_id)
        .bind(ppr_id)
        .bind(document_id)
        .bind(activity_type)
        .bind(description)
        .bind(old_status)
        .bind(new_status)
        .bind(performed_by)
        .bind(performed_by_name)
        .bind(details)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn list_activities(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprActivity>> {
        sqlx::query_as::<_, PprActivity>(
            "SELECT * FROM _atlas.ppr_activities WHERE ppr_id = $1 ORDER BY created_at",
        )
        .bind(ppr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard> {
        let stats = sqlx::query(
            r"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_count,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted_count,
                COUNT(*) FILTER (WHERE status = 'selection_complete') as selection_complete_count,
                COUNT(*) FILTER (WHERE status = 'formatted') as formatted_count,
                COUNT(*) FILTER (WHERE status = 'confirmed') as confirmed_count,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled_count,
                COALESCE(SUM(total_payment_amount), 0) as total_payment_amount,
                COALESCE(SUM(total_documents), 0) as total_documents_processed
            FROM _atlas.payment_process_requests WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_method = sqlx::query(
            r"SELECT payment_method, COUNT(*) as count, COALESCE(SUM(total_payment_amount), 0) as total
            FROM _atlas.payment_process_requests WHERE organization_id = $1 GROUP BY payment_method"
        )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_criteria = sqlx::query(
            r"SELECT selection_criteria, COUNT(*) as count, COALESCE(SUM(total_payment_amount), 0) as total
            FROM _atlas.payment_process_requests WHERE organization_id = $1 GROUP BY selection_criteria"
        )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_payment_method = serde_json::to_value(
            by_method
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "paymentMethod": r.get::<String, _>("payment_method"),
                        "count": r.get::<i64, _>("count"),
                        "total": r.get::<f64, _>("total"),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null);

        let by_selection_criteria = serde_json::to_value(
            by_criteria
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "selectionCriteria": r.get::<String, _>("selection_criteria"),
                        "count": r.get::<i64, _>("count"),
                        "total": r.get::<f64, _>("total"),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or(serde_json::Value::Null);

        Ok(PprDashboard {
            total_requests: stats.get("total"),
            draft_count: stats.get("draft_count"),
            submitted_count: stats.get("submitted_count"),
            selection_complete_count: stats.get("selection_complete_count"),
            formatted_count: stats.get("formatted_count"),
            confirmed_count: stats.get("confirmed_count"),
            cancelled_count: stats.get("cancelled_count"),
            total_payment_amount: stats.get("total_payment_amount"),
            total_documents_processed: stats.get("total_documents_processed"),
            by_payment_method,
            by_selection_criteria,
        })
    }
}
