//! Intercompany Repository
//!
//! PostgreSQL storage for intercompany batches, transactions, settlements,
//! and balances.

use atlas_shared::{
    IntercompanyBatch, IntercompanyTransaction, IntercompanySettlement,
    IntercompanyBalance, AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for intercompany data storage
#[async_trait]
pub trait IntercompanyRepository: Send + Sync {
    // Batches
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyBatch>;

    async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<IntercompanyBatch>>;
    async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<IntercompanyBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<IntercompanyBatch>>;
    async fn update_batch_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        posted_at: Option<chrono::DateTime<chrono::Utc>>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<IntercompanyBatch>;
    async fn update_batch_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        total_debit: &str,
        total_credit: &str,
        transaction_count: i32,
    ) -> AtlasResult<()>;

    // Transactions
    async fn create_transaction(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        transaction_number: &str,
        transaction_type: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        from_debit_account: Option<&str>,
        from_credit_account: Option<&str>,
        to_debit_account: Option<&str>,
        to_credit_account: Option<&str>,
        from_ic_account: &str,
        to_ic_account: &str,
        transaction_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        source_entity_type: Option<&str>,
        source_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyTransaction>;

    async fn get_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<IntercompanyTransaction>>;
    async fn list_transactions_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<IntercompanyTransaction>>;
    async fn list_transactions_by_entity(
        &self,
        org_id: Uuid,
        entity_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<IntercompanyTransaction>>;
    async fn update_transaction_status(
        &self,
        id: Uuid,
        status: &str,
        settlement_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<IntercompanyTransaction>;

    // Settlements
    async fn create_settlement(
        &self,
        org_id: Uuid,
        settlement_number: &str,
        settlement_method: &str,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        settled_amount: &str,
        currency_code: &str,
        payment_reference: Option<&str>,
        transaction_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanySettlement>;

    async fn list_settlements(
        &self,
        org_id: Uuid,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<IntercompanySettlement>>;

    // Balances
    async fn get_balance(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
    ) -> AtlasResult<Option<IntercompanyBalance>>;

    async fn upsert_balance(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
        total_outstanding: &str,
        total_posted: &str,
        total_settled: &str,
        open_transaction_count: i32,
    ) -> AtlasResult<IntercompanyBalance>;

    async fn list_balances(&self, org_id: Uuid) -> AtlasResult<Vec<IntercompanyBalance>>;
}

/// PostgreSQL implementation
pub struct PostgresIntercompanyRepository {
    pool: PgPool,
}

impl PostgresIntercompanyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_batch(&self, row: &sqlx::postgres::PgRow) -> IntercompanyBatch {
        let total_amount: serde_json::Value = row.try_get::<serde_json::Value, _>("total_amount")
            .unwrap_or(serde_json::json!("0"));
        let total_debit: serde_json::Value = row.try_get::<serde_json::Value, _>("total_debit")
            .unwrap_or(serde_json::json!("0"));
        let total_credit: serde_json::Value = row.try_get::<serde_json::Value, _>("total_credit")
            .unwrap_or(serde_json::json!("0"));

        IntercompanyBatch {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            batch_number: row.get("batch_number"),
            description: row.get("description"),
            status: row.get("status"),
            from_entity_id: row.get("from_entity_id"),
            from_entity_name: row.get("from_entity_name"),
            to_entity_id: row.get("to_entity_id"),
            to_entity_name: row.get("to_entity_name"),
            currency_code: row.get("currency_code"),
            total_amount: total_amount.to_string(),
            total_debit: total_debit.to_string(),
            total_credit: total_credit.to_string(),
            transaction_count: row.get("transaction_count"),
            from_journal_id: row.get("from_journal_id"),
            to_journal_id: row.get("to_journal_id"),
            accounting_date: row.get("accounting_date"),
            posted_at: row.get("posted_at"),
            rejected_reason: row.get("rejected_reason"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            approved_by: row.get("approved_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_transaction(&self, row: &sqlx::postgres::PgRow) -> IntercompanyTransaction {
        let amount: serde_json::Value = row.try_get::<serde_json::Value, _>("amount")
            .unwrap_or(serde_json::json!("0"));
        let exchange_rate: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("exchange_rate")
            .ok().flatten();

        IntercompanyTransaction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            batch_id: row.get("batch_id"),
            transaction_number: row.get("transaction_number"),
            transaction_type: row.get("transaction_type"),
            description: row.get("description"),
            from_entity_id: row.get("from_entity_id"),
            from_entity_name: row.get("from_entity_name"),
            to_entity_id: row.get("to_entity_id"),
            to_entity_name: row.get("to_entity_name"),
            amount: amount.to_string(),
            currency_code: row.get("currency_code"),
            exchange_rate: exchange_rate.map(|v| v.to_string()),
            from_debit_account: row.get("from_debit_account"),
            from_credit_account: row.get("from_credit_account"),
            to_debit_account: row.get("to_debit_account"),
            to_credit_account: row.get("to_credit_account"),
            from_ic_account: row.get("from_ic_account"),
            to_ic_account: row.get("to_ic_account"),
            status: row.get("status"),
            transaction_date: row.get("transaction_date"),
            due_date: row.get("due_date"),
            settlement_date: row.get("settlement_date"),
            source_entity_type: row.get("source_entity_type"),
            source_entity_id: row.get("source_entity_id"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_settlement(&self, row: &sqlx::postgres::PgRow) -> IntercompanySettlement {
        let settled_amount: serde_json::Value = row.try_get::<serde_json::Value, _>("settled_amount")
            .unwrap_or(serde_json::json!("0"));

        IntercompanySettlement {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            settlement_number: row.get("settlement_number"),
            settlement_method: row.get("settlement_method"),
            from_entity_id: row.get("from_entity_id"),
            to_entity_id: row.get("to_entity_id"),
            settled_amount: settled_amount.to_string(),
            currency_code: row.get("currency_code"),
            payment_reference: row.get("payment_reference"),
            status: row.get("status"),
            settlement_date: row.get("settlement_date"),
            transaction_ids: row.get("transaction_ids"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_balance(&self, row: &sqlx::postgres::PgRow) -> IntercompanyBalance {
        let total_outstanding: serde_json::Value = row.try_get::<serde_json::Value, _>("total_outstanding")
            .unwrap_or(serde_json::json!("0"));
        let total_posted: serde_json::Value = row.try_get::<serde_json::Value, _>("total_posted")
            .unwrap_or(serde_json::json!("0"));
        let total_settled: serde_json::Value = row.try_get::<serde_json::Value, _>("total_settled")
            .unwrap_or(serde_json::json!("0"));

        IntercompanyBalance {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            from_entity_id: row.get("from_entity_id"),
            to_entity_id: row.get("to_entity_id"),
            currency_code: row.get("currency_code"),
            total_outstanding: total_outstanding.to_string(),
            total_posted: total_posted.to_string(),
            total_settled: total_settled.to_string(),
            open_transaction_count: row.get("open_transaction_count"),
            as_of_date: row.get("as_of_date"),
            metadata: row.get("metadata"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl IntercompanyRepository for PostgresIntercompanyRepository {
    // ========================================================================
    // Batches
    // ========================================================================

    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.intercompany_batches
                (organization_id, batch_number, description,
                 from_entity_id, from_entity_name, to_entity_id, to_entity_name,
                 currency_code, accounting_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, batch_number) DO UPDATE
                SET description = $3, from_entity_id = $4, from_entity_name = $5,
                    to_entity_id = $6, to_entity_name = $7, currency_code = $8,
                    accounting_date = $9, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_number).bind(description)
        .bind(from_entity_id).bind(from_entity_name)
        .bind(to_entity_id).bind(to_entity_name)
        .bind(currency_code).bind(accounting_date).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_batch(&row))
    }

    async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<IntercompanyBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.intercompany_batches WHERE organization_id = $1 AND batch_number = $2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_batch(&r)))
    }

    async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<IntercompanyBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.intercompany_batches WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_batch(&r)))
    }

    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<IntercompanyBatch>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.intercompany_batches WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.intercompany_batches WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_batch(r)).collect())
    }

    async fn update_batch_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        posted_at: Option<chrono::DateTime<chrono::Utc>>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<IntercompanyBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.intercompany_batches
            SET status = $2, approved_by = COALESCE($3, approved_by),
                posted_at = COALESCE($4, posted_at),
                rejected_reason = $5, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(posted_at).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_batch(&row))
    }

    async fn update_batch_totals(
        &self,
        id: Uuid,
        total_amount: &str,
        total_debit: &str,
        total_credit: &str,
        transaction_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.intercompany_batches
            SET total_amount = $2::numeric, total_debit = $3::numeric,
                total_credit = $4::numeric, transaction_count = $5,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_amount).bind(total_debit).bind(total_credit).bind(transaction_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    async fn create_transaction(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        transaction_number: &str,
        transaction_type: &str,
        description: Option<&str>,
        from_entity_id: Uuid,
        from_entity_name: &str,
        to_entity_id: Uuid,
        to_entity_name: &str,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        from_debit_account: Option<&str>,
        from_credit_account: Option<&str>,
        to_debit_account: Option<&str>,
        to_credit_account: Option<&str>,
        from_ic_account: &str,
        to_ic_account: &str,
        transaction_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        source_entity_type: Option<&str>,
        source_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanyTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.intercompany_transactions
                (organization_id, batch_id, transaction_number, transaction_type,
                 description, from_entity_id, from_entity_name,
                 to_entity_id, to_entity_name,
                 amount, currency_code, exchange_rate,
                 from_debit_account, from_credit_account,
                 to_debit_account, to_credit_account,
                 from_ic_account, to_ic_account,
                 transaction_date, due_date,
                 source_entity_type, source_entity_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::numeric, $11, $12::numeric,
                    $13, $14, $15, $16, $17, $18,
                    $19, $20, $21, $22, $23)
            ON CONFLICT (organization_id, transaction_number) DO UPDATE
                SET transaction_type = $4, description = $5, amount = $10::numeric,
                    exchange_rate = $12::numeric, from_debit_account = $13,
                    from_credit_account = $14, to_debit_account = $15,
                    to_credit_account = $16, from_ic_account = $17,
                    to_ic_account = $18, transaction_date = $19, due_date = $20,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_id).bind(transaction_number).bind(transaction_type)
        .bind(description)
        .bind(from_entity_id).bind(from_entity_name)
        .bind(to_entity_id).bind(to_entity_name)
        .bind(amount).bind(currency_code).bind(exchange_rate)
        .bind(from_debit_account).bind(from_credit_account)
        .bind(to_debit_account).bind(to_credit_account)
        .bind(from_ic_account).bind(to_ic_account)
        .bind(transaction_date).bind(due_date)
        .bind(source_entity_type).bind(source_entity_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transaction(&row))
    }

    async fn get_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<Option<IntercompanyTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.intercompany_transactions WHERE organization_id = $1 AND transaction_number = $2"
        )
        .bind(org_id).bind(transaction_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transaction(&r)))
    }

    async fn list_transactions_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<IntercompanyTransaction>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.intercompany_transactions WHERE batch_id = $1 ORDER BY transaction_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transaction(r)).collect())
    }

    async fn list_transactions_by_entity(
        &self,
        org_id: Uuid,
        entity_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<IntercompanyTransaction>> {
        let rows = match status {
            Some(s) => sqlx::query(
                r#"SELECT * FROM _atlas.intercompany_transactions
                WHERE organization_id = $1 AND (from_entity_id = $2 OR to_entity_id = $2) AND status = $3
                ORDER BY transaction_date DESC"#
            )
            .bind(org_id).bind(entity_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                r#"SELECT * FROM _atlas.intercompany_transactions
                WHERE organization_id = $1 AND (from_entity_id = $2 OR to_entity_id = $2)
                ORDER BY transaction_date DESC"#
            )
            .bind(org_id).bind(entity_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transaction(r)).collect())
    }

    async fn update_transaction_status(
        &self,
        id: Uuid,
        status: &str,
        settlement_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<IntercompanyTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.intercompany_transactions
            SET status = $2, settlement_date = COALESCE($3, settlement_date), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(settlement_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transaction(&row))
    }

    // ========================================================================
    // Settlements
    // ========================================================================

    async fn create_settlement(
        &self,
        org_id: Uuid,
        settlement_number: &str,
        settlement_method: &str,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        settled_amount: &str,
        currency_code: &str,
        payment_reference: Option<&str>,
        transaction_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<IntercompanySettlement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.intercompany_settlements
                (organization_id, settlement_number, settlement_method,
                 from_entity_id, to_entity_id, settled_amount,
                 currency_code, payment_reference, transaction_ids, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(settlement_number).bind(settlement_method)
        .bind(from_entity_id).bind(to_entity_id).bind(settled_amount)
        .bind(currency_code).bind(payment_reference).bind(transaction_ids).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_settlement(&row))
    }

    async fn list_settlements(
        &self,
        org_id: Uuid,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<IntercompanySettlement>> {
        let rows = match entity_id {
            Some(eid) => sqlx::query(
                r#"SELECT * FROM _atlas.intercompany_settlements
                WHERE organization_id = $1 AND (from_entity_id = $2 OR to_entity_id = $2)
                ORDER BY settlement_date DESC"#
            )
            .bind(org_id).bind(eid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.intercompany_settlements WHERE organization_id = $1 ORDER BY settlement_date DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_settlement(r)).collect())
    }

    // ========================================================================
    // Balances
    // ========================================================================

    async fn get_balance(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
    ) -> AtlasResult<Option<IntercompanyBalance>> {
        let today = chrono::Utc::now().date_naive();
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.intercompany_balances
            WHERE organization_id = $1 AND from_entity_id = $2
              AND to_entity_id = $3 AND currency_code = $4 AND as_of_date = $5"#
        )
        .bind(org_id).bind(from_entity_id).bind(to_entity_id).bind(currency_code).bind(today)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_balance(&r)))
    }

    async fn upsert_balance(
        &self,
        org_id: Uuid,
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        currency_code: &str,
        total_outstanding: &str,
        total_posted: &str,
        total_settled: &str,
        open_transaction_count: i32,
    ) -> AtlasResult<IntercompanyBalance> {
        let today = chrono::Utc::now().date_naive();
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.intercompany_balances
                (organization_id, from_entity_id, to_entity_id, currency_code,
                 total_outstanding, total_posted, total_settled,
                 open_transaction_count, as_of_date)
            VALUES ($1, $2, $3, $4, $5::numeric, $6::numeric, $7::numeric, $8, $9)
            ON CONFLICT (organization_id, from_entity_id, to_entity_id, currency_code, as_of_date)
            DO UPDATE SET
                total_outstanding = $5::numeric, total_posted = $6::numeric,
                total_settled = $7::numeric, open_transaction_count = $8,
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(from_entity_id).bind(to_entity_id).bind(currency_code)
        .bind(total_outstanding).bind(total_posted).bind(total_settled)
        .bind(open_transaction_count).bind(today)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_balance(&row))
    }

    async fn list_balances(&self, org_id: Uuid) -> AtlasResult<Vec<IntercompanyBalance>> {
        let today = chrono::Utc::now().date_naive();
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.intercompany_balances
            WHERE organization_id = $1 AND as_of_date = $2
            ORDER BY from_entity_id, to_entity_id"#
        )
        .bind(org_id).bind(today)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_balance(r)).collect())
    }
}
