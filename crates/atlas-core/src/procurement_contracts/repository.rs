//! Procurement Contracts Repository
//!
//! PostgreSQL storage for procurement contracts data:
//! contract types, contracts, contract lines, milestones,
//! renewals, and spend entries.

use atlas_shared::{
    ContractType, ProcurementContract, ContractLine, ContractMilestone,
    ContractRenewal, ContractSpend,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for procurement contracts data storage
#[async_trait]
pub trait ProcurementContractRepository: Send + Sync {
    // Contract Types
    async fn create_contract_type(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        contract_classification: &str, requires_approval: bool,
        default_duration_days: Option<i32>,
        allow_amount_commitment: bool, allow_quantity_commitment: bool,
        allow_line_additions: bool, allow_price_adjustment: bool,
        allow_renewal: bool, allow_termination: bool,
        max_renewals: Option<i32>,
        default_payment_terms_code: Option<&str>,
        default_currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractType>;

    async fn get_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ContractType>>;
    async fn get_contract_type_by_id(&self, id: Uuid) -> AtlasResult<Option<ContractType>>;
    async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ContractType>>;
    async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Contracts
    async fn create_contract(
        &self,
        org_id: Uuid, contract_number: &str, title: &str, description: Option<&str>,
        contract_type_code: Option<&str>, contract_classification: &str,
        supplier_id: Uuid, supplier_number: Option<&str>,
        supplier_name: Option<&str>, supplier_contact: Option<&str>,
        buyer_id: Option<Uuid>, buyer_name: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        total_committed_amount: &str, currency_code: &str,
        payment_terms_code: Option<&str>, price_type: &str,
        max_renewals: Option<i32>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProcurementContract>;

    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ProcurementContract>>;
    async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<ProcurementContract>>;
    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ProcurementContract>>;
    async fn update_contract_status(
        &self, id: Uuid, status: &str,
        approved_by: Option<Uuid>, rejection_reason: Option<&str>,
        terminated_by: Option<Uuid>, termination_reason: Option<&str>,
    ) -> AtlasResult<ProcurementContract>;
    async fn update_contract_totals(
        &self, id: Uuid,
        total_committed_amount: Option<&str>,
        total_released_amount: Option<&str>,
        total_invoiced_amount: Option<&str>,
        line_count: Option<i32>,
        milestone_count: Option<i32>,
    ) -> AtlasResult<ProcurementContract>;
    async fn update_contract_dates(
        &self, id: Uuid,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ProcurementContract>;
    async fn increment_renewal_count(&self, id: Uuid) -> AtlasResult<ProcurementContract>;

    // Contract Lines
    async fn create_contract_line(
        &self,
        org_id: Uuid, contract_id: Uuid, line_number: i32,
        item_description: &str, item_code: Option<&str>,
        category: Option<&str>, uom: Option<&str>,
        quantity_committed: Option<&str>, quantity_released: &str,
        unit_price: &str, line_amount: &str, amount_released: &str,
        delivery_date: Option<chrono::NaiveDate>,
        supplier_part_number: Option<&str>,
        account_code: Option<&str>, cost_center: Option<&str>,
        project_id: Option<Uuid>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractLine>;

    async fn get_contract_line(&self, id: Uuid) -> AtlasResult<Option<ContractLine>>;
    async fn list_contract_lines(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractLine>>;
    async fn delete_contract_line(&self, id: Uuid) -> AtlasResult<()>;

    // Milestones
    async fn create_milestone(
        &self,
        org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        milestone_number: i32, name: &str, description: Option<&str>,
        milestone_type: &str, target_date: chrono::NaiveDate,
        amount: &str, percent_of_total: &str,
        deliverable: Option<&str>, is_billable: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractMilestone>;

    async fn get_milestone(&self, id: Uuid) -> AtlasResult<Option<ContractMilestone>>;
    async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractMilestone>>;
    async fn update_milestone_status(
        &self, id: Uuid, status: &str, actual_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ContractMilestone>;

    // Renewals
    async fn create_renewal(
        &self,
        org_id: Uuid, contract_id: Uuid, renewal_number: i32,
        previous_end_date: chrono::NaiveDate, new_end_date: chrono::NaiveDate,
        renewal_type: &str, terms_changed: Option<&str>,
        renewed_by: Option<Uuid>, notes: Option<&str>,
    ) -> AtlasResult<ContractRenewal>;

    async fn list_renewals(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractRenewal>>;

    // Spend
    async fn create_spend_entry(
        &self,
        org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        source_type: &str, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate, amount: &str,
        quantity: Option<&str>, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractSpend>;

    async fn list_spend_entries(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractSpend>>;
}

/// PostgreSQL implementation
pub struct PostgresProcurementContractRepository {
    pool: PgPool,
}

impl PostgresProcurementContractRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_contract_type(row: &sqlx::postgres::PgRow) -> ContractType {
    ContractType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        contract_classification: row.get("contract_classification"),
        requires_approval: row.get("requires_approval"),
        default_duration_days: row.get("default_duration_days"),
        allow_amount_commitment: row.get("allow_amount_commitment"),
        allow_quantity_commitment: row.get("allow_quantity_commitment"),
        allow_line_additions: row.get("allow_line_additions"),
        allow_price_adjustment: row.get("allow_price_adjustment"),
        allow_renewal: row.get("allow_renewal"),
        allow_termination: row.get("allow_termination"),
        max_renewals: row.get("max_renewals"),
        default_payment_terms_code: row.get("default_payment_terms_code"),
        default_currency_code: row.get("default_currency_code"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_contract(row: &sqlx::postgres::PgRow) -> ProcurementContract {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    ProcurementContract {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_number: row.get("contract_number"),
        title: row.get("title"),
        description: row.get("description"),
        contract_type_code: row.get("contract_type_code"),
        contract_classification: row.get("contract_classification"),
        status: row.get("status"),
        supplier_id: row.get("supplier_id"),
        supplier_number: row.get("supplier_number"),
        supplier_name: row.get("supplier_name"),
        supplier_contact: row.get("supplier_contact"),
        buyer_id: row.get("buyer_id"),
        buyer_name: row.get("buyer_name"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        total_committed_amount: get_num(row, "total_committed_amount"),
        total_released_amount: get_num(row, "total_released_amount"),
        total_invoiced_amount: get_num(row, "total_invoiced_amount"),
        currency_code: row.get("currency_code"),
        payment_terms_code: row.get("payment_terms_code"),
        price_type: row.get("price_type"),
        renewal_count: row.get("renewal_count"),
        max_renewals: row.get("max_renewals"),
        line_count: row.get("line_count"),
        milestone_count: row.get("milestone_count"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        rejection_reason: row.get("rejection_reason"),
        termination_reason: row.get("termination_reason"),
        terminated_by: row.get("terminated_by"),
        terminated_at: row.get("terminated_at"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_contract_line(row: &sqlx::postgres::PgRow) -> ContractLine {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    ContractLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        line_number: row.get("line_number"),
        item_description: row.get("item_description"),
        item_code: row.get("item_code"),
        category: row.get("category"),
        uom: row.get("uom"),
        quantity_committed: row.get("quantity_committed"),
        quantity_released: get_num(row, "quantity_released"),
        unit_price: get_num(row, "unit_price"),
        line_amount: get_num(row, "line_amount"),
        amount_released: get_num(row, "amount_released"),
        delivery_date: row.get("delivery_date"),
        supplier_part_number: row.get("supplier_part_number"),
        account_code: row.get("account_code"),
        cost_center: row.get("cost_center"),
        project_id: row.get("project_id"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_milestone(row: &sqlx::postgres::PgRow) -> ContractMilestone {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    ContractMilestone {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        contract_line_id: row.get("contract_line_id"),
        milestone_number: row.get("milestone_number"),
        name: row.get("name"),
        description: row.get("description"),
        milestone_type: row.get("milestone_type"),
        target_date: row.get("target_date"),
        actual_date: row.get("actual_date"),
        status: row.get("status"),
        amount: get_num(row, "amount"),
        percent_of_total: get_num(row, "percent_of_total"),
        deliverable: row.get("deliverable"),
        is_billable: row.get("is_billable"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_renewal(row: &sqlx::postgres::PgRow) -> ContractRenewal {
    ContractRenewal {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        renewal_number: row.get("renewal_number"),
        previous_end_date: row.get("previous_end_date"),
        new_end_date: row.get("new_end_date"),
        renewal_type: row.get("renewal_type"),
        terms_changed: row.get("terms_changed"),
        renewed_by: row.get("renewed_by"),
        renewed_at: row.get("renewed_at"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
    }
}

fn row_to_spend(row: &sqlx::postgres::PgRow) -> ContractSpend {
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    ContractSpend {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        contract_id: row.get("contract_id"),
        contract_line_id: row.get("contract_line_id"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        transaction_date: row.get("transaction_date"),
        amount: get_num(row, "amount"),
        quantity: row.get("quantity"),
        description: row.get("description"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl ProcurementContractRepository for PostgresProcurementContractRepository {
    // ========================================================================
    // Contract Types
    // ========================================================================

    async fn create_contract_type(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        contract_classification: &str, requires_approval: bool,
        default_duration_days: Option<i32>,
        allow_amount_commitment: bool, allow_quantity_commitment: bool,
        allow_line_additions: bool, allow_price_adjustment: bool,
        allow_renewal: bool, allow_termination: bool,
        max_renewals: Option<i32>,
        default_payment_terms_code: Option<&str>,
        default_currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractType> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contract_types
                (organization_id, code, name, description, contract_classification,
                 requires_approval, default_duration_days,
                 allow_amount_commitment, allow_quantity_commitment,
                 allow_line_additions, allow_price_adjustment,
                 allow_renewal, allow_termination, max_renewals,
                 default_payment_terms_code, default_currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, contract_classification = $5,
                    requires_approval = $6, default_duration_days = $7,
                    allow_amount_commitment = $8, allow_quantity_commitment = $9,
                    allow_line_additions = $10, allow_price_adjustment = $11,
                    allow_renewal = $12, allow_termination = $13, max_renewals = $14,
                    default_payment_terms_code = $15, default_currency_code = $16,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(contract_classification).bind(requires_approval).bind(default_duration_days)
        .bind(allow_amount_commitment).bind(allow_quantity_commitment)
        .bind(allow_line_additions).bind(allow_price_adjustment)
        .bind(allow_renewal).bind(allow_termination).bind(max_renewals)
        .bind(default_payment_terms_code).bind(default_currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_contract_type(&row))
    }

    async fn get_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ContractType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_types WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract_type(&r)))
    }

    async fn get_contract_type_by_id(&self, id: Uuid) -> AtlasResult<Option<ContractType>> {
        let row = sqlx::query("SELECT * FROM _atlas.procurement_contract_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract_type(&r)))
    }

    async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ContractType>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_types WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_contract_type).collect())
    }

    async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.procurement_contract_types SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Contracts
    // ========================================================================

    async fn create_contract(
        &self,
        org_id: Uuid, contract_number: &str, title: &str, description: Option<&str>,
        contract_type_code: Option<&str>, contract_classification: &str,
        supplier_id: Uuid, supplier_number: Option<&str>,
        supplier_name: Option<&str>, supplier_contact: Option<&str>,
        buyer_id: Option<Uuid>, buyer_name: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        total_committed_amount: &str, currency_code: &str,
        payment_terms_code: Option<&str>, price_type: &str,
        max_renewals: Option<i32>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProcurementContract> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contracts
                (organization_id, contract_number, title, description,
                 contract_type_code, contract_classification, status,
                 supplier_id, supplier_number, supplier_name, supplier_contact,
                 buyer_id, buyer_name,
                 start_date, end_date,
                 total_committed_amount, total_released_amount, total_invoiced_amount,
                 currency_code, payment_terms_code, price_type,
                 renewal_count, max_renewals, line_count, milestone_count,
                 notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, 'draft',
                    $7, $8, $9, $10, $11, $12, $13, $14,
                    $15::numeric, 0, 0,
                    $16, $17, $18,
                    0, $19, 0, 0,
                    $20, $21)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_number).bind(title).bind(description)
        .bind(contract_type_code).bind(contract_classification)
        .bind(supplier_id).bind(supplier_number).bind(supplier_name).bind(supplier_contact)
        .bind(buyer_id).bind(buyer_name)
        .bind(start_date).bind(end_date)
        .bind(total_committed_amount)
        .bind(currency_code).bind(payment_terms_code).bind(price_type)
        .bind(max_renewals).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_contract(&row))
    }

    async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ProcurementContract>> {
        let row = sqlx::query("SELECT * FROM _atlas.procurement_contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract(&r)))
    }

    async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<ProcurementContract>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.procurement_contracts WHERE organization_id = $1 AND contract_number = $2"
        )
        .bind(org_id).bind(contract_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract(&r)))
    }

    async fn list_contracts(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<ProcurementContract>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.procurement_contracts
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::uuid IS NULL OR supplier_id = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_contract).collect())
    }

    async fn update_contract_status(
        &self, id: Uuid, status: &str,
        approved_by: Option<Uuid>, rejection_reason: Option<&str>,
        terminated_by: Option<Uuid>, termination_reason: Option<&str>,
    ) -> AtlasResult<ProcurementContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.procurement_contracts
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE approved_at END,
                rejection_reason = $4,
                terminated_by = $5,
                termination_reason = $6,
                terminated_at = CASE WHEN $5 IS NOT NULL THEN now() ELSE terminated_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejection_reason)
        .bind(terminated_by).bind(termination_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_contract(&row))
    }

    async fn update_contract_totals(
        &self, id: Uuid,
        total_committed_amount: Option<&str>,
        total_released_amount: Option<&str>,
        total_invoiced_amount: Option<&str>,
        line_count: Option<i32>,
        milestone_count: Option<i32>,
    ) -> AtlasResult<ProcurementContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.procurement_contracts
            SET total_committed_amount = COALESCE($2::numeric, total_committed_amount),
                total_released_amount = COALESCE($3::numeric, total_released_amount),
                total_invoiced_amount = COALESCE($4::numeric, total_invoiced_amount),
                line_count = COALESCE($5, line_count),
                milestone_count = COALESCE($6, milestone_count),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(total_committed_amount).bind(total_released_amount)
        .bind(total_invoiced_amount).bind(line_count).bind(milestone_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_contract(&row))
    }

    async fn update_contract_dates(
        &self, id: Uuid,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ProcurementContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.procurement_contracts
            SET start_date = COALESCE($2, start_date),
                end_date = COALESCE($3, end_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(start_date).bind(end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_contract(&row))
    }

    async fn increment_renewal_count(&self, id: Uuid) -> AtlasResult<ProcurementContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.procurement_contracts
            SET renewal_count = renewal_count + 1, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_contract(&row))
    }

    // ========================================================================
    // Contract Lines
    // ========================================================================

    async fn create_contract_line(
        &self,
        org_id: Uuid, contract_id: Uuid, line_number: i32,
        item_description: &str, item_code: Option<&str>,
        category: Option<&str>, uom: Option<&str>,
        quantity_committed: Option<&str>, quantity_released: &str,
        unit_price: &str, line_amount: &str, amount_released: &str,
        delivery_date: Option<chrono::NaiveDate>,
        supplier_part_number: Option<&str>,
        account_code: Option<&str>, cost_center: Option<&str>,
        project_id: Option<Uuid>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contract_lines
                (organization_id, contract_id, line_number,
                 item_description, item_code, category, uom,
                 quantity_committed, quantity_released,
                 unit_price, line_amount, amount_released,
                 delivery_date, supplier_part_number,
                 account_code, cost_center, project_id, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9::numeric,
                    $10::numeric, $11::numeric, $12::numeric,
                    $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(line_number)
        .bind(item_description).bind(item_code).bind(category).bind(uom)
        .bind(quantity_committed).bind(quantity_released)
        .bind(unit_price).bind(line_amount).bind(amount_released)
        .bind(delivery_date).bind(supplier_part_number)
        .bind(account_code).bind(cost_center).bind(project_id).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_contract_line(&row))
    }

    async fn get_contract_line(&self, id: Uuid) -> AtlasResult<Option<ContractLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.procurement_contract_lines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_contract_line(&r)))
    }

    async fn list_contract_lines(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_lines WHERE contract_id = $1 ORDER BY line_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_contract_line).collect())
    }

    async fn delete_contract_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.procurement_contract_lines WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Milestones
    // ========================================================================

    async fn create_milestone(
        &self,
        org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        milestone_number: i32, name: &str, description: Option<&str>,
        milestone_type: &str, target_date: chrono::NaiveDate,
        amount: &str, percent_of_total: &str,
        deliverable: Option<&str>, is_billable: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractMilestone> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contract_milestones
                (organization_id, contract_id, contract_line_id,
                 milestone_number, name, description, milestone_type,
                 target_date, status, amount, percent_of_total,
                 deliverable, is_billable, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8, 'pending', $9::numeric, $10::numeric,
                    $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(contract_line_id)
        .bind(milestone_number).bind(name).bind(description).bind(milestone_type)
        .bind(target_date).bind(amount).bind(percent_of_total)
        .bind(deliverable).bind(is_billable).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_milestone(&row))
    }

    async fn get_milestone(&self, id: Uuid) -> AtlasResult<Option<ContractMilestone>> {
        let row = sqlx::query("SELECT * FROM _atlas.procurement_contract_milestones WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_milestone(&r)))
    }

    async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractMilestone>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_milestones WHERE contract_id = $1 ORDER BY milestone_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_milestone).collect())
    }

    async fn update_milestone_status(
        &self, id: Uuid, status: &str, actual_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ContractMilestone> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.procurement_contract_milestones
            SET status = $2,
                actual_date = COALESCE($3, actual_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(actual_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_milestone(&row))
    }

    // ========================================================================
    // Renewals
    // ========================================================================

    async fn create_renewal(
        &self,
        org_id: Uuid, contract_id: Uuid, renewal_number: i32,
        previous_end_date: chrono::NaiveDate, new_end_date: chrono::NaiveDate,
        renewal_type: &str, terms_changed: Option<&str>,
        renewed_by: Option<Uuid>, notes: Option<&str>,
    ) -> AtlasResult<ContractRenewal> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contract_renewals
                (organization_id, contract_id, renewal_number,
                 previous_end_date, new_end_date, renewal_type,
                 terms_changed, renewed_by, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(renewal_number)
        .bind(previous_end_date).bind(new_end_date).bind(renewal_type)
        .bind(terms_changed).bind(renewed_by).bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_renewal(&row))
    }

    async fn list_renewals(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractRenewal>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_renewals WHERE contract_id = $1 ORDER BY renewal_number"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_renewal).collect())
    }

    // ========================================================================
    // Spend
    // ========================================================================

    async fn create_spend_entry(
        &self,
        org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        source_type: &str, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate, amount: &str,
        quantity: Option<&str>, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractSpend> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.procurement_contract_spend
                (organization_id, contract_id, contract_line_id,
                 source_type, source_id, source_number,
                 transaction_date, amount,
                 quantity, description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(contract_id).bind(contract_line_id)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(transaction_date).bind(amount)
        .bind(quantity).bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_spend(&row))
    }

    async fn list_spend_entries(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractSpend>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.procurement_contract_spend WHERE contract_id = $1 ORDER BY transaction_date"
        )
        .bind(contract_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_spend).collect())
    }
}
