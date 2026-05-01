//! Netting Repository
//!
//! Storage interface for AP/AR netting data.

use atlas_shared::{
    NettingAgreement, NettingBatch, NettingTransactionLine,
    NettingDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for netting data storage
#[async_trait]
pub trait NettingRepository: Send + Sync {
    // Agreements
    async fn create_agreement(
        &self,
        org_id: Uuid,
        agreement_number: &str,
        name: &str,
        description: Option<&str>,
        partner_id: Uuid,
        partner_number: Option<&str>,
        partner_name: Option<&str>,
        currency_code: &str,
        netting_direction: &str,
        settlement_method: &str,
        minimum_netting_amount: &str,
        maximum_netting_amount: Option<&str>,
        auto_select_transactions: bool,
        selection_criteria: serde_json::Value,
        netting_clearing_account: Option<&str>,
        ap_clearing_account: Option<&str>,
        ar_clearing_account: Option<&str>,
        approval_required: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingAgreement>;

    async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<NettingAgreement>>;
    async fn get_agreement_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<NettingAgreement>>;
    async fn list_agreements(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<NettingAgreement>>;
    async fn update_agreement_status(&self, id: Uuid, status: &str) -> AtlasResult<NettingAgreement>;

    // Batches
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        agreement_id: Uuid,
        netting_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        partner_id: Uuid,
        partner_name: Option<&str>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingBatch>;

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<NettingBatch>>;
    async fn list_batches(&self, org_id: Uuid, agreement_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<NettingBatch>>;
    async fn update_batch_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<NettingBatch>;
    async fn update_batch_totals(
        &self, id: Uuid,
        total_payables: &str, total_receivables: &str,
        net_difference: &str, settlement_direction: &str,
        payable_count: i32, receivable_count: i32,
    ) -> AtlasResult<()>;

    // Transaction Lines
    async fn create_transaction_line(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        line_number: i32,
        source_type: &str,
        source_id: Uuid,
        source_number: Option<&str>,
        source_date: Option<chrono::NaiveDate>,
        original_amount: &str,
        netting_amount: &str,
        remaining_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingTransactionLine>;

    async fn list_batch_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<NettingTransactionLine>>;
    async fn update_line_status(&self, id: Uuid, status: &str) -> AtlasResult<NettingTransactionLine>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<NettingDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresNettingRepository {
    pool: PgPool,
}

impl PostgresNettingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_agreement(row: &sqlx::postgres::PgRow) -> NettingAgreement {
    NettingAgreement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        agreement_number: row.get("agreement_number"),
        name: row.get("name"),
        description: row.get("description"),
        partner_id: row.get("partner_id"),
        partner_number: row.get("partner_number"),
        partner_name: row.get("partner_name"),
        currency_code: row.get("currency_code"),
        netting_direction: row.get("netting_direction"),
        settlement_method: row.get("settlement_method"),
        minimum_netting_amount: row.try_get("minimum_netting_amount").unwrap_or("0".to_string()),
        maximum_netting_amount: row.try_get("maximum_netting_amount").unwrap_or(None),
        auto_select_transactions: row.get("auto_select_transactions"),
        selection_criteria: row.try_get("selection_criteria").unwrap_or(serde_json::json!({})),
        netting_clearing_account: row.get("netting_clearing_account"),
        ap_clearing_account: row.get("ap_clearing_account"),
        ar_clearing_account: row.get("ar_clearing_account"),
        approval_required: row.get("approval_required"),
        status: row.get("status"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_batch(row: &sqlx::postgres::PgRow) -> NettingBatch {
    NettingBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        agreement_id: row.get("agreement_id"),
        netting_date: row.get("netting_date"),
        gl_date: row.get("gl_date"),
        partner_id: row.get("partner_id"),
        partner_name: row.get("partner_name"),
        currency_code: row.get("currency_code"),
        total_payables_amount: row.try_get("total_payables_amount").unwrap_or("0".to_string()),
        total_receivables_amount: row.try_get("total_receivables_amount").unwrap_or("0".to_string()),
        net_difference: row.try_get("net_difference").unwrap_or("0".to_string()),
        settlement_direction: row.try_get("settlement_direction").unwrap_or("zero".to_string()),
        status: row.get("status"),
        payable_transaction_count: row.try_get("payable_transaction_count").unwrap_or(0),
        receivable_transaction_count: row.try_get("receivable_transaction_count").unwrap_or(0),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        rejected_reason: row.get("rejected_reason"),
        settlement_payment_id: row.get("settlement_payment_id"),
        settlement_receipt_id: row.get("settlement_receipt_id"),
        journal_entry_id: row.get("journal_entry_id"),
        settled_at: row.get("settled_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transaction_line(row: &sqlx::postgres::PgRow) -> NettingTransactionLine {
    NettingTransactionLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_id: row.get("batch_id"),
        line_number: row.get("line_number"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        source_date: row.get("source_date"),
        original_amount: row.try_get("original_amount").unwrap_or("0".to_string()),
        netting_amount: row.try_get("netting_amount").unwrap_or("0".to_string()),
        remaining_amount: row.try_get("remaining_amount").unwrap_or("0".to_string()),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl NettingRepository for PostgresNettingRepository {
    async fn create_agreement(
        &self,
        org_id: Uuid, agreement_number: &str, name: &str, description: Option<&str>,
        partner_id: Uuid, partner_number: Option<&str>, partner_name: Option<&str>,
        currency_code: &str, netting_direction: &str, settlement_method: &str,
        minimum_netting_amount: &str, maximum_netting_amount: Option<&str>,
        auto_select_transactions: bool, selection_criteria: serde_json::Value,
        netting_clearing_account: Option<&str>, ap_clearing_account: Option<&str>,
        ar_clearing_account: Option<&str>, approval_required: bool,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingAgreement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.netting_agreements
                (organization_id, agreement_number, name, description,
                 partner_id, partner_number, partner_name,
                 currency_code, netting_direction, settlement_method,
                 minimum_netting_amount, maximum_netting_amount,
                 auto_select_transactions, selection_criteria,
                 netting_clearing_account, ap_clearing_account, ar_clearing_account,
                 approval_required, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::double precision, $12::double precision,
                    $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(agreement_number).bind(name).bind(description)
        .bind(partner_id).bind(partner_number).bind(partner_name)
        .bind(currency_code).bind(netting_direction).bind(settlement_method)
        .bind(minimum_netting_amount).bind(maximum_netting_amount)
        .bind(auto_select_transactions).bind(selection_criteria)
        .bind(netting_clearing_account).bind(ap_clearing_account).bind(ar_clearing_account)
        .bind(approval_required).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_agreement(&row))
    }

    async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<NettingAgreement>> {
        let row = sqlx::query("SELECT * FROM _atlas.netting_agreements WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_agreement(&r)))
    }

    async fn get_agreement_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<NettingAgreement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.netting_agreements WHERE organization_id = $1 AND agreement_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_agreement(&r)))
    }

    async fn list_agreements(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<NettingAgreement>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.netting_agreements
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_agreement).collect())
    }

    async fn update_agreement_status(&self, id: Uuid, status: &str) -> AtlasResult<NettingAgreement> {
        let row = sqlx::query(
            "UPDATE _atlas.netting_agreements SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_agreement(&row))
    }

    async fn create_batch(
        &self,
        org_id: Uuid, batch_number: &str, agreement_id: Uuid,
        netting_date: chrono::NaiveDate, gl_date: Option<chrono::NaiveDate>,
        partner_id: Uuid, partner_name: Option<&str>,
        currency_code: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<NettingBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.netting_batches
                (organization_id, batch_number, agreement_id,
                 netting_date, gl_date, partner_id, partner_name,
                 currency_code, total_payables_amount, total_receivables_amount,
                 net_difference, settlement_direction, status,
                 payable_transaction_count, receivable_transaction_count, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 0, 0, 'zero', 'draft', 0, 0, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_number).bind(agreement_id)
        .bind(netting_date).bind(gl_date).bind(partner_id).bind(partner_name)
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<NettingBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.netting_batches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn list_batches(&self, org_id: Uuid, agreement_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<NettingBatch>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.netting_batches
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR agreement_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY netting_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(agreement_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_batch).collect())
    }

    async fn update_batch_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<NettingBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.netting_batches
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                settled_at = CASE WHEN $2 = 'settled' THEN now() ELSE settled_at END,
                rejected_reason = COALESCE($5, rejected_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(submitted_by).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn update_batch_totals(
        &self, id: Uuid,
        total_payables: &str, total_receivables: &str,
        net_difference: &str, settlement_direction: &str,
        payable_count: i32, receivable_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.netting_batches
            SET total_payables_amount = $2::double precision,
                total_receivables_amount = $3::double precision,
                net_difference = $4::double precision,
                settlement_direction = $5,
                payable_transaction_count = $6,
                receivable_transaction_count = $7,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_payables).bind(total_receivables)
        .bind(net_difference).bind(settlement_direction)
        .bind(payable_count).bind(receivable_count)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_transaction_line(
        &self,
        org_id: Uuid, batch_id: Uuid, line_number: i32,
        source_type: &str, source_id: Uuid,
        source_number: Option<&str>, source_date: Option<chrono::NaiveDate>,
        original_amount: &str, netting_amount: &str, remaining_amount: &str,
        currency_code: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<NettingTransactionLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.netting_transaction_lines
                (organization_id, batch_id, line_number,
                 source_type, source_id, source_number, source_date,
                 original_amount, netting_amount, remaining_amount,
                 currency_code, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::double precision, $9::double precision, $10::double precision,
                    $11, 'selected', $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_id).bind(line_number)
        .bind(source_type).bind(source_id).bind(source_number).bind(source_date)
        .bind(original_amount).bind(netting_amount).bind(remaining_amount)
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction_line(&row))
    }

    async fn list_batch_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<NettingTransactionLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.netting_transaction_lines WHERE batch_id = $1 ORDER BY line_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transaction_line).collect())
    }

    async fn update_line_status(&self, id: Uuid, status: &str) -> AtlasResult<NettingTransactionLine> {
        let row = sqlx::query(
            "UPDATE _atlas.netting_transaction_lines SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction_line(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<NettingDashboardSummary> {
        // Query agreement counts
        let aggr_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.netting_agreements WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_agreements: i64 = aggr_row.try_get("total").unwrap_or(0);
        let active_agreements: i64 = aggr_row.try_get("active").unwrap_or(0);

        // Query batch counts and totals
        let batch_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft,
                COUNT(*) FILTER (WHERE status = 'submitted') as pending,
                COUNT(*) FILTER (WHERE status = 'settled') as settled,
                COALESCE(SUM(total_payables_amount) FILTER (WHERE status = 'settled'), 0) as total_payables,
                COALESCE(SUM(total_receivables_amount) FILTER (WHERE status = 'settled'), 0) as total_receivables,
                COALESCE(SUM(ABS(net_difference)) FILTER (WHERE status = 'settled'), 0) as total_net
            FROM _atlas.netting_batches WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(NettingDashboardSummary {
            total_agreements: total_agreements as i32,
            active_agreements: active_agreements as i32,
            total_batches: batch_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            draft_batches: batch_row.try_get::<i64, _>("draft").unwrap_or(0) as i32,
            pending_approval_batches: batch_row.try_get::<i64, _>("pending").unwrap_or(0) as i32,
            settled_batches: batch_row.try_get::<i64, _>("settled").unwrap_or(0) as i32,
            total_payables_netted: format!("{:.2}", batch_row.try_get::<f64, _>("total_payables").unwrap_or(0.0)),
            total_receivables_netted: format!("{:.2}", batch_row.try_get::<f64, _>("total_receivables").unwrap_or(0.0)),
            total_net_difference_settled: format!("{:.2}", batch_row.try_get::<f64, _>("total_net").unwrap_or(0.0)),
        })
    }
}
