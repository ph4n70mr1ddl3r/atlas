//! Revenue Recognition Repository
//!
//! PostgreSQL storage for revenue recognition data:
//! revenue policies, revenue contracts, performance obligations,
//! revenue schedule lines, and contract modifications.

use atlas_shared::{
    RevenuePolicy, RevenueContract, PerformanceObligation,
    RevenueScheduleLine, RevenueModification,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for revenue recognition data storage
#[async_trait]
pub trait RevenueRepository: Send + Sync {
    // Revenue Policies
    async fn create_policy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        recognition_method: &str,
        over_time_method: Option<&str>,
        allocation_basis: &str,
        default_selling_price: Option<&str>,
        constrain_variable_consideration: bool,
        constraint_threshold_percent: Option<&str>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        contra_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenuePolicy>;

    async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RevenuePolicy>>;
    async fn get_policy_by_id(&self, id: Uuid) -> AtlasResult<Option<RevenuePolicy>>;
    async fn list_policies(&self, org_id: Uuid) -> AtlasResult<Vec<RevenuePolicy>>;
    async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Revenue Contracts
    async fn create_contract(
        &self,
        org_id: Uuid,
        contract_number: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        contract_date: Option<chrono::NaiveDate>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        total_transaction_price: &str,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueContract>;

    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<RevenueContract>>;
    async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<RevenueContract>>;
    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<RevenueContract>>;
    async fn update_contract_status(
        &self,
        id: Uuid,
        status: Option<&str>,
        step1_contract_identified: Option<bool>,
        step2_obligations_identified: Option<bool>,
        step3_price_determined: Option<bool>,
        step4_price_allocated: Option<bool>,
        step5_recognition_scheduled: Option<bool>,
        total_allocated_revenue: Option<&str>,
        total_recognized_revenue: Option<&str>,
        total_deferred_revenue: Option<&str>,
        total_transaction_price: Option<&str>,
        notes: Option<Option<&str>>,
    ) -> AtlasResult<RevenueContract>;

    // Performance Obligations
    async fn create_obligation(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        line_number: i32,
        description: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        source_line_id: Option<Uuid>,
        revenue_policy_id: Option<Uuid>,
        recognition_method: Option<&str>,
        over_time_method: Option<&str>,
        standalone_selling_price: &str,
        allocated_transaction_price: &str,
        satisfaction_method: &str,
        recognition_start_date: Option<chrono::NaiveDate>,
        recognition_end_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceObligation>;

    async fn get_obligation(&self, id: Uuid) -> AtlasResult<Option<PerformanceObligation>>;
    async fn list_obligations(&self, contract_id: Uuid) -> AtlasResult<Vec<PerformanceObligation>>;
    async fn update_obligation_allocation(
        &self,
        id: Uuid,
        allocated_transaction_price: &str,
        deferred_revenue: &str,
    ) -> AtlasResult<PerformanceObligation>;
    async fn update_obligation_status(
        &self,
        id: Uuid,
        status: &str,
        recognition_start_date: Option<&str>,
        recognition_end_date: Option<&str>,
    ) -> AtlasResult<PerformanceObligation>;
    async fn update_obligation_recognition(
        &self,
        id: Uuid,
        total_recognized_revenue: &str,
        deferred_revenue: &str,
        percent_complete: &str,
        status: &str,
    ) -> AtlasResult<PerformanceObligation>;

    // Revenue Schedule Lines
    async fn create_schedule_line(
        &self,
        org_id: Uuid,
        obligation_id: Uuid,
        contract_id: Uuid,
        line_number: i32,
        recognition_date: chrono::NaiveDate,
        amount: &str,
        percent_of_total: &str,
        recognition_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueScheduleLine>;

    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<RevenueScheduleLine>>;
    async fn list_schedule_lines(&self, obligation_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>>;
    async fn list_schedule_lines_by_contract(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>>;
    async fn update_schedule_line_status(
        &self,
        id: Uuid,
        status: &str,
        recognized_amount: Option<&str>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<RevenueScheduleLine>;

    // Contract Modifications
    async fn create_modification(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        modification_number: i32,
        modification_type: &str,
        description: Option<&str>,
        previous_transaction_price: &str,
        new_transaction_price: &str,
        previous_end_date: Option<chrono::NaiveDate>,
        new_end_date: Option<chrono::NaiveDate>,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueModification>;

    async fn list_modifications(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueModification>>;
}

/// PostgreSQL implementation
pub struct PostgresRevenueRepository {
    pool: PgPool,
}

impl PostgresRevenueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_policy(row: &sqlx::postgres::PgRow) -> RevenuePolicy {
    RevenuePolicy {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        recognition_method: row.get("recognition_method"),
        over_time_method: row.get("over_time_method"),
        allocation_basis: row.get("allocation_basis"),
        default_selling_price: row.try_get("default_selling_price")
            .map(|v: serde_json::Value| v.to_string()).ok(),
        constrain_variable_consideration: row.get("constrain_variable_consideration"),
        constraint_threshold_percent: row.try_get("constraint_threshold_percent")
            .map(|v: serde_json::Value| v.to_string()).ok(),
        revenue_account_code: row.get("revenue_account_code"),
        deferred_revenue_account_code: row.get("deferred_revenue_account_code"),
        contra_revenue_account_code: row.get("contra_revenue_account_code"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_contract(row: &sqlx::postgres::PgRow) -> RevenueContract {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    RevenueContract {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_number: row.get("contract_number"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        contract_date: row.get("contract_date"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        total_transaction_price: get_num(row, "total_transaction_price"),
        total_allocated_revenue: get_num(row, "total_allocated_revenue"),
        total_recognized_revenue: get_num(row, "total_recognized_revenue"),
        total_deferred_revenue: get_num(row, "total_deferred_revenue"),
        status: row.get("status"),
        step1_contract_identified: row.get("step1_contract_identified"),
        step2_obligations_identified: row.get("step2_obligations_identified"),
        step3_price_determined: row.get("step3_price_determined"),
        step4_price_allocated: row.get("step4_price_allocated"),
        step5_recognition_scheduled: row.get("step5_recognition_scheduled"),
        currency_code: row.get("currency_code"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_obligation(row: &sqlx::postgres::PgRow) -> PerformanceObligation {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    PerformanceObligation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        line_number: row.get("line_number"),
        description: row.get("description"),
        product_id: row.get("product_id"),
        product_name: row.get("product_name"),
        source_line_id: row.get("source_line_id"),
        revenue_policy_id: row.get("revenue_policy_id"),
        recognition_method: row.get("recognition_method"),
        over_time_method: row.get("over_time_method"),
        standalone_selling_price: get_num(row, "standalone_selling_price"),
        allocated_transaction_price: get_num(row, "allocated_transaction_price"),
        total_recognized_revenue: get_num(row, "total_recognized_revenue"),
        deferred_revenue: get_num(row, "deferred_revenue"),
        recognition_start_date: row.get("recognition_start_date"),
        recognition_end_date: row.get("recognition_end_date"),
        percent_complete: row.try_get("percent_complete")
            .map(|v: serde_json::Value| v.to_string()).ok(),
        satisfaction_method: row.get("satisfaction_method"),
        status: row.get("status"),
        revenue_account_code: row.get("revenue_account_code"),
        deferred_revenue_account_code: row.get("deferred_revenue_account_code"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_schedule_line(row: &sqlx::postgres::PgRow) -> RevenueScheduleLine {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    RevenueScheduleLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        obligation_id: row.get("obligation_id"),
        contract_id: row.get("contract_id"),
        line_number: row.get("line_number"),
        recognition_date: row.get("recognition_date"),
        amount: get_num(row, "amount"),
        recognized_amount: get_num(row, "recognized_amount"),
        status: row.get("status"),
        recognition_method: row.get("recognition_method"),
        percent_of_total: row.try_get("percent_of_total")
            .map(|v: serde_json::Value| v.to_string()).ok(),
        journal_entry_id: row.get("journal_entry_id"),
        recognized_at: row.get("recognized_at"),
        reversed_by_id: row.get("reversed_by_id"),
        reversal_reason: row.get("reversal_reason"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_modification(row: &sqlx::postgres::PgRow) -> RevenueModification {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    RevenueModification {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        modification_number: row.get("modification_number"),
        modification_type: row.get("modification_type"),
        description: row.get("description"),
        previous_transaction_price: get_num(row, "previous_transaction_price"),
        new_transaction_price: get_num(row, "new_transaction_price"),
        previous_end_date: row.get("previous_end_date"),
        new_end_date: row.get("new_end_date"),
        effective_date: row.get("effective_date"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl RevenueRepository for PostgresRevenueRepository {
    // ========================================================================
    // Revenue Policies
    // ========================================================================

    async fn create_policy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        recognition_method: &str,
        over_time_method: Option<&str>,
        allocation_basis: &str,
        default_selling_price: Option<&str>,
        constrain_variable_consideration: bool,
        constraint_threshold_percent: Option<&str>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        contra_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenuePolicy> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.revenue_policies
                (organization_id, code, name, description,
                 recognition_method, over_time_method, allocation_basis,
                 default_selling_price, constrain_variable_consideration,
                 constraint_threshold_percent,
                 revenue_account_code, deferred_revenue_account_code,
                 contra_revenue_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9, $10::numeric,
                    $11, $12, $13, $14)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4,
                    recognition_method = $5, over_time_method = $6,
                    allocation_basis = $7, default_selling_price = $8::numeric,
                    constrain_variable_consideration = $9,
                    constraint_threshold_percent = $10::numeric,
                    revenue_account_code = $11, deferred_revenue_account_code = $12,
                    contra_revenue_account_code = $13, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(recognition_method).bind(over_time_method).bind(allocation_basis)
        .bind(default_selling_price).bind(constrain_variable_consideration)
        .bind(constraint_threshold_percent)
        .bind(revenue_account_code).bind(deferred_revenue_account_code)
        .bind(contra_revenue_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_policy(&row))
    }

    async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RevenuePolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.revenue_policies WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn get_policy_by_id(&self, id: Uuid) -> AtlasResult<Option<RevenuePolicy>> {
        let row = sqlx::query("SELECT * FROM _atlas.revenue_policies WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn list_policies(&self, org_id: Uuid) -> AtlasResult<Vec<RevenuePolicy>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.revenue_policies WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_policy(&r)).collect())
    }

    async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.revenue_policies SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Revenue Contracts
    // ========================================================================

    async fn create_contract(
        &self,
        org_id: Uuid,
        contract_number: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        contract_date: Option<chrono::NaiveDate>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        total_transaction_price: &str,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueContract> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.revenue_contracts
                (organization_id, contract_number,
                 source_type, source_id, source_number,
                 customer_id, customer_number, customer_name,
                 contract_date, start_date, end_date,
                 total_transaction_price, total_allocated_revenue,
                 total_recognized_revenue, total_deferred_revenue,
                 currency_code, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12::numeric, 0, 0, $12::numeric,
                    $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_number)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(contract_date).bind(start_date).bind(end_date)
        .bind(total_transaction_price)
        .bind(currency_code).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_contract(&row))
    }

    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<RevenueContract>> {
        let row = sqlx::query("SELECT * FROM _atlas.revenue_contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract(&r)))
    }

    async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<RevenueContract>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.revenue_contracts WHERE organization_id = $1 AND contract_number = $2"
        )
        .bind(org_id).bind(contract_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract(&r)))
    }

    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<RevenueContract>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.revenue_contracts
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR customer_id = $3)
            ORDER BY contract_date DESC NULLS LAST, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_contract(&r)).collect())
    }

    async fn update_contract_status(
        &self,
        id: Uuid,
        status: Option<&str>,
        step1_contract_identified: Option<bool>,
        step2_obligations_identified: Option<bool>,
        step3_price_determined: Option<bool>,
        step4_price_allocated: Option<bool>,
        step5_recognition_scheduled: Option<bool>,
        total_allocated_revenue: Option<&str>,
        total_recognized_revenue: Option<&str>,
        total_deferred_revenue: Option<&str>,
        total_transaction_price: Option<&str>,
        notes: Option<Option<&str>>,
    ) -> AtlasResult<RevenueContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.revenue_contracts
            SET status = COALESCE($2, status),
                step1_contract_identified = COALESCE($3, step1_contract_identified),
                step2_obligations_identified = COALESCE($4, step2_obligations_identified),
                step3_price_determined = COALESCE($5, step3_price_determined),
                step4_price_allocated = COALESCE($6, step4_price_allocated),
                step5_recognition_scheduled = COALESCE($7, step5_recognition_scheduled),
                total_allocated_revenue = COALESCE($8::numeric, total_allocated_revenue),
                total_recognized_revenue = COALESCE($9::numeric, total_recognized_revenue),
                total_deferred_revenue = COALESCE($10::numeric, total_deferred_revenue),
                total_transaction_price = COALESCE($11::numeric, total_transaction_price),
                notes = COALESCE($12, notes),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .bind(step1_contract_identified).bind(step2_obligations_identified)
        .bind(step3_price_determined).bind(step4_price_allocated)
        .bind(step5_recognition_scheduled)
        .bind(total_allocated_revenue).bind(total_recognized_revenue)
        .bind(total_deferred_revenue).bind(total_transaction_price)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_contract(&row))
    }

    // ========================================================================
    // Performance Obligations
    // ========================================================================

    async fn create_obligation(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        line_number: i32,
        description: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        source_line_id: Option<Uuid>,
        revenue_policy_id: Option<Uuid>,
        recognition_method: Option<&str>,
        over_time_method: Option<&str>,
        standalone_selling_price: &str,
        allocated_transaction_price: &str,
        satisfaction_method: &str,
        recognition_start_date: Option<chrono::NaiveDate>,
        recognition_end_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceObligation> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.performance_obligations
                (organization_id, contract_id, line_number,
                 description, product_id, product_name, source_line_id,
                 revenue_policy_id, recognition_method, over_time_method,
                 standalone_selling_price, allocated_transaction_price,
                 total_recognized_revenue, deferred_revenue,
                 satisfaction_method, recognition_start_date, recognition_end_date,
                 revenue_account_code, deferred_revenue_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, 0, $12::numeric,
                    $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(line_number)
        .bind(description).bind(product_id).bind(product_name).bind(source_line_id)
        .bind(revenue_policy_id).bind(recognition_method).bind(over_time_method)
        .bind(standalone_selling_price).bind(allocated_transaction_price)
        .bind(satisfaction_method).bind(recognition_start_date).bind(recognition_end_date)
        .bind(revenue_account_code).bind(deferred_revenue_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_obligation(&row))
    }

    async fn get_obligation(&self, id: Uuid) -> AtlasResult<Option<PerformanceObligation>> {
        let row = sqlx::query("SELECT * FROM _atlas.performance_obligations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_obligation(&r)))
    }

    async fn list_obligations(&self, contract_id: Uuid) -> AtlasResult<Vec<PerformanceObligation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.performance_obligations WHERE contract_id = $1 AND status != 'cancelled' ORDER BY line_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_obligation(&r)).collect())
    }

    async fn update_obligation_allocation(
        &self,
        id: Uuid,
        allocated_transaction_price: &str,
        deferred_revenue: &str,
    ) -> AtlasResult<PerformanceObligation> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.performance_obligations
            SET allocated_transaction_price = $2::numeric,
                deferred_revenue = $3::numeric,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(allocated_transaction_price).bind(deferred_revenue)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_obligation(&row))
    }

    async fn update_obligation_status(
        &self,
        id: Uuid,
        status: &str,
        recognition_start_date: Option<&str>,
        recognition_end_date: Option<&str>,
    ) -> AtlasResult<PerformanceObligation> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.performance_obligations
            SET status = $2,
                recognition_start_date = COALESCE($3::date, recognition_start_date),
                recognition_end_date = COALESCE($4::date, recognition_end_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(recognition_start_date).bind(recognition_end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_obligation(&row))
    }

    async fn update_obligation_recognition(
        &self,
        id: Uuid,
        total_recognized_revenue: &str,
        deferred_revenue: &str,
        percent_complete: &str,
        status: &str,
    ) -> AtlasResult<PerformanceObligation> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.performance_obligations
            SET total_recognized_revenue = $2::numeric,
                deferred_revenue = $3::numeric,
                percent_complete = $4::numeric,
                status = $5,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(total_recognized_revenue).bind(deferred_revenue)
        .bind(percent_complete).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_obligation(&row))
    }

    // ========================================================================
    // Revenue Schedule Lines
    // ========================================================================

    async fn create_schedule_line(
        &self,
        org_id: Uuid,
        obligation_id: Uuid,
        contract_id: Uuid,
        line_number: i32,
        recognition_date: chrono::NaiveDate,
        amount: &str,
        percent_of_total: &str,
        recognition_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueScheduleLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.revenue_schedule_lines
                (organization_id, obligation_id, contract_id, line_number,
                 recognition_date, amount, recognized_amount,
                 status, percent_of_total, recognition_method, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, 0,
                    'planned', $7::numeric, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(obligation_id).bind(contract_id).bind(line_number)
        .bind(recognition_date).bind(amount)
        .bind(percent_of_total).bind(recognition_method).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_schedule_line(&row))
    }

    async fn get_schedule_line(&self, id: Uuid) -> AtlasResult<Option<RevenueScheduleLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.revenue_schedule_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_schedule_line(&r)))
    }

    async fn list_schedule_lines(&self, obligation_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.revenue_schedule_lines WHERE obligation_id = $1 ORDER BY line_number"
        )
        .bind(obligation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_schedule_line(&r)).collect())
    }

    async fn list_schedule_lines_by_contract(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.revenue_schedule_lines WHERE contract_id = $1 ORDER BY recognition_date, line_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_schedule_line(&r)).collect())
    }

    async fn update_schedule_line_status(
        &self,
        id: Uuid,
        status: &str,
        recognized_amount: Option<&str>,
        reversal_reason: Option<&str>,
    ) -> AtlasResult<RevenueScheduleLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.revenue_schedule_lines
            SET status = $2,
                recognized_amount = COALESCE($3::numeric, recognized_amount),
                recognized_at = CASE WHEN $2 = 'recognized' THEN now() ELSE recognized_at END,
                reversal_reason = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(recognized_amount).bind(reversal_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_schedule_line(&row))
    }

    // ========================================================================
    // Contract Modifications
    // ========================================================================

    async fn create_modification(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        modification_number: i32,
        modification_type: &str,
        description: Option<&str>,
        previous_transaction_price: &str,
        new_transaction_price: &str,
        previous_end_date: Option<chrono::NaiveDate>,
        new_end_date: Option<chrono::NaiveDate>,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueModification> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.revenue_modifications
                (organization_id, contract_id, modification_number,
                 modification_type, description,
                 previous_transaction_price, new_transaction_price,
                 previous_end_date, new_end_date, effective_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric,
                    $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(modification_number)
        .bind(modification_type).bind(description)
        .bind(previous_transaction_price).bind(new_transaction_price)
        .bind(previous_end_date).bind(new_end_date).bind(effective_date)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_modification(&row))
    }

    async fn list_modifications(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueModification>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.revenue_modifications WHERE contract_id = $1 ORDER BY modification_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_modification(&r)).collect())
    }
}
