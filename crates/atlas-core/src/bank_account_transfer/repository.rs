//! Bank Account Transfer Repository
//!
//! Storage interface for bank account transfer data.

use atlas_shared::{
    BankTransferType, BankAccountTransfer, BankTransferDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for bank account transfer data storage
#[async_trait]
pub trait BankAccountTransferRepository: Send + Sync {
    // Transfer Types
    async fn create_transfer_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        settlement_method: &str,
        requires_approval: bool,
        approval_threshold: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankTransferType>;

    async fn list_transfer_types(&self, org_id: Uuid) -> AtlasResult<Vec<BankTransferType>>;

    // Transfers
    async fn create_transfer(
        &self,
        org_id: Uuid,
        transfer_number: &str,
        transfer_type_id: Option<Uuid>,
        from_bank_account_id: Uuid,
        from_bank_account_number: Option<&str>,
        from_bank_name: Option<&str>,
        to_bank_account_id: Uuid,
        to_bank_account_number: Option<&str>,
        to_bank_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        from_currency: Option<&str>,
        to_currency: Option<&str>,
        transferred_amount: Option<&str>,
        transfer_date: chrono::NaiveDate,
        value_date: Option<chrono::NaiveDate>,
        settlement_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        description: Option<&str>,
        purpose: Option<&str>,
        status: &str,
        priority: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccountTransfer>;

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<BankAccountTransfer>>;
    async fn list_transfers(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BankAccountTransfer>>;
    async fn update_transfer_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        completed_by: Option<Uuid>,
        cancelled_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BankAccountTransfer>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<BankTransferDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresBankAccountTransferRepository {
    pool: PgPool,
}

impl PostgresBankAccountTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_transfer_type(row: &sqlx::postgres::PgRow) -> BankTransferType {
    BankTransferType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        settlement_method: row.get("settlement_method"),
        requires_approval: row.get("requires_approval"),
        approval_threshold: row.try_get("approval_threshold").unwrap_or(None),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transfer(row: &sqlx::postgres::PgRow) -> BankAccountTransfer {
    BankAccountTransfer {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        transfer_number: row.get("transfer_number"),
        transfer_type_id: row.get("transfer_type_id"),
        from_bank_account_id: row.get("from_bank_account_id"),
        from_bank_account_number: row.get("from_bank_account_number"),
        from_bank_name: row.get("from_bank_name"),
        to_bank_account_id: row.get("to_bank_account_id"),
        to_bank_account_number: row.get("to_bank_account_number"),
        to_bank_name: row.get("to_bank_name"),
        amount: row.try_get("amount").unwrap_or("0".to_string()),
        currency_code: row.get("currency_code"),
        exchange_rate: row.try_get("exchange_rate").unwrap_or(None),
        from_currency: row.get("from_currency"),
        to_currency: row.get("to_currency"),
        transferred_amount: row.try_get("transferred_amount").unwrap_or(None),
        transfer_date: row.get("transfer_date"),
        value_date: row.get("value_date"),
        settlement_date: row.get("settlement_date"),
        reference_number: row.get("reference_number"),
        description: row.get("description"),
        purpose: row.get("purpose"),
        status: row.get("status"),
        priority: row.get("priority"),
        from_journal_id: row.get("from_journal_id"),
        to_journal_id: row.get("to_journal_id"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        completed_by: row.get("completed_by"),
        completed_at: row.get("completed_at"),
        cancelled_by: row.get("cancelled_by"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        failure_reason: row.get("failure_reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl BankAccountTransferRepository for PostgresBankAccountTransferRepository {
    async fn create_transfer_type(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        settlement_method: &str, requires_approval: bool,
        approval_threshold: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<BankTransferType> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.bank_transfer_types
                (organization_id, code, name, description, settlement_method,
                 requires_approval, approval_threshold, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::decimal, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(settlement_method).bind(requires_approval)
        .bind(approval_threshold).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transfer_type(&row))
    }

    async fn list_transfer_types(&self, org_id: Uuid) -> AtlasResult<Vec<BankTransferType>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.bank_transfer_types WHERE organization_id = $1 AND is_active = true ORDER BY code"
        ).bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transfer_type).collect())
    }

    async fn create_transfer(
        &self,
        org_id: Uuid, transfer_number: &str, transfer_type_id: Option<Uuid>,
        from_bank_account_id: Uuid, from_bank_account_number: Option<&str>,
        from_bank_name: Option<&str>,
        to_bank_account_id: Uuid, to_bank_account_number: Option<&str>,
        to_bank_name: Option<&str>,
        amount: &str, currency_code: &str, exchange_rate: Option<&str>,
        from_currency: Option<&str>, to_currency: Option<&str>,
        transferred_amount: Option<&str>,
        transfer_date: chrono::NaiveDate, value_date: Option<chrono::NaiveDate>,
        settlement_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>, description: Option<&str>,
        purpose: Option<&str>, status: &str, priority: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccountTransfer> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.bank_account_transfers
                (organization_id, transfer_number, transfer_type_id,
                 from_bank_account_id, from_bank_account_number, from_bank_name,
                 to_bank_account_id, to_bank_account_number, to_bank_name,
                 amount, currency_code, exchange_rate, from_currency, to_currency,
                 transferred_amount, transfer_date, value_date, settlement_date,
                 reference_number, description, purpose, status, priority, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::decimal, $11, $12::decimal, $13, $14, $15::decimal,
                    $16, $17, $18, $19, $20, $21, $22, $23, $24)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transfer_number).bind(transfer_type_id)
        .bind(from_bank_account_id).bind(from_bank_account_number).bind(from_bank_name)
        .bind(to_bank_account_id).bind(to_bank_account_number).bind(to_bank_name)
        .bind(amount).bind(currency_code).bind(exchange_rate).bind(from_currency).bind(to_currency)
        .bind(transferred_amount).bind(transfer_date).bind(value_date).bind(settlement_date)
        .bind(reference_number).bind(description).bind(purpose).bind(status).bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transfer(&row))
    }

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<BankAccountTransfer>> {
        let row = sqlx::query("SELECT * FROM _atlas.bank_account_transfers WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transfer(&r)))
    }

    async fn list_transfers(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BankAccountTransfer>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.bank_account_transfers
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY transfer_date DESC, created_at DESC"#,
        ).bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transfer).collect())
    }

    async fn update_transfer_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
        completed_by: Option<Uuid>, cancelled_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<BankAccountTransfer> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.bank_account_transfers
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                completed_by = COALESCE($5, completed_by),
                completed_at = CASE WHEN $2 = 'completed' THEN now() ELSE completed_at END,
                cancelled_by = COALESCE($6, cancelled_by),
                cancelled_at = CASE WHEN $2 = 'cancelled' THEN now() ELSE cancelled_at END,
                cancellation_reason = COALESCE($7, cancellation_reason),
                updated_at = now()
            WHERE id = $1 RETURNING *
            "#,
        ).bind(id).bind(status).bind(submitted_by).bind(approved_by)
        .bind(completed_by).bind(cancelled_by).bind(cancellation_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transfer(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<BankTransferDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('draft', 'submitted', 'approved', 'in_transit')) as pending,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled,
                COALESCE(SUM(amount) FILTER (WHERE status = 'completed'), 0) as total_amount,
                COALESCE(AVG(amount) FILTER (WHERE status = 'completed'), 0) as avg_amount
            FROM _atlas.bank_account_transfers WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let type_count = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.bank_transfer_types WHERE organization_id = $1 AND is_active = true"
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(BankTransferDashboardSummary {
            total_transfers: row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            pending_transfers: row.try_get::<i64, _>("pending").unwrap_or(0) as i32,
            completed_transfers: row.try_get::<i64, _>("completed").unwrap_or(0) as i32,
            cancelled_transfers: row.try_get::<i64, _>("cancelled").unwrap_or(0) as i32,
            total_amount_transferred: format!("{:.2}", row.try_get::<f64, _>("total_amount").unwrap_or(0.0)),
            average_transfer_amount: format!("{:.2}", row.try_get::<f64, _>("avg_amount").unwrap_or(0.0)),
            total_transfer_types: type_count.try_get::<i64, _>("cnt").unwrap_or(0) as i32,
        })
    }
}
