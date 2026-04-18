//! Customer Returns Repository
//!
//! PostgreSQL storage for return reason codes, RMAs, return lines,
//! and credit memos.

use atlas_shared::{
    ReturnReason, ReturnAuthorization, ReturnLine, CreditMemo,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for customer returns data storage
#[async_trait]
pub trait CustomerReturnsRepository: Send + Sync {
    // Return Reasons
    async fn create_return_reason(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        return_type: &str,
        default_disposition: Option<&str>,
        requires_approval: bool,
        credit_issued_automatically: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnReason>;

    async fn get_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReturnReason>>;
    async fn list_return_reasons(&self, org_id: Uuid, return_type: Option<&str>) -> AtlasResult<Vec<ReturnReason>>;
    async fn delete_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Return Authorizations (RMAs)
    async fn create_rma(
        &self,
        org_id: Uuid,
        rma_number: &str,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        return_type: &str,
        reason_code: Option<&str>,
        reason_name: Option<&str>,
        original_order_number: Option<&str>,
        original_order_id: Option<Uuid>,
        customer_contact: Option<&str>,
        customer_email: Option<&str>,
        customer_phone: Option<&str>,
        return_date: chrono::NaiveDate,
        expected_receipt_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnAuthorization>;

    async fn get_rma(&self, id: Uuid) -> AtlasResult<Option<ReturnAuthorization>>;
    async fn get_rma_by_number(&self, org_id: Uuid, rma_number: &str) -> AtlasResult<Option<ReturnAuthorization>>;
    async fn list_rmas(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        return_type: Option<&str>,
    ) -> AtlasResult<Vec<ReturnAuthorization>>;
    async fn update_rma_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<ReturnAuthorization>;
    async fn update_rma_totals(
        &self,
        id: Uuid,
        total_quantity: &str,
        total_amount: &str,
        total_credit_amount: &str,
    ) -> AtlasResult<()>;
    async fn update_rma_credit_memo(
        &self,
        id: Uuid,
        credit_memo_id: Uuid,
        credit_memo_number: &str,
    ) -> AtlasResult<()>;

    // Return Lines
    async fn create_return_line(
        &self,
        org_id: Uuid,
        rma_id: Uuid,
        line_number: i32,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        original_line_id: Option<Uuid>,
        original_quantity: &str,
        return_quantity: &str,
        unit_price: &str,
        return_amount: &str,
        credit_amount: &str,
        reason_code: Option<&str>,
        disposition: Option<&str>,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        condition: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnLine>;

    async fn get_return_line(&self, id: Uuid) -> AtlasResult<Option<ReturnLine>>;
    async fn list_return_lines_by_rma(&self, rma_id: Uuid) -> AtlasResult<Vec<ReturnLine>>;
    async fn update_return_line_receipt(
        &self,
        id: Uuid,
        received_quantity: &str,
        received_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ReturnLine>;
    async fn update_return_line_inspection(
        &self,
        id: Uuid,
        inspection_status: &str,
        inspection_notes: Option<&str>,
        disposition: Option<&str>,
    ) -> AtlasResult<ReturnLine>;
    async fn update_return_line_credit_status(
        &self,
        id: Uuid,
        credit_status: &str,
    ) -> AtlasResult<ReturnLine>;

    // Credit Memos
    async fn create_credit_memo(
        &self,
        org_id: Uuid,
        credit_memo_number: &str,
        rma_id: Option<Uuid>,
        rma_number: Option<&str>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        gl_account_code: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditMemo>;

    async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<CreditMemo>>;
    async fn get_credit_memo_by_number(&self, org_id: Uuid, credit_memo_number: &str) -> AtlasResult<Option<CreditMemo>>;
    async fn list_credit_memos(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CreditMemo>>;
    async fn update_credit_memo_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<CreditMemo>;
}

/// PostgreSQL implementation
pub struct PostgresCustomerReturnsRepository {
    pool: PgPool,
}

impl PostgresCustomerReturnsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_return_reason(&self, row: &sqlx::postgres::PgRow) -> ReturnReason {
        ReturnReason {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            return_type: row.get("return_type"),
            default_disposition: row.get("default_disposition"),
            requires_approval: row.get("requires_approval"),
            credit_issued_automatically: row.get("credit_issued_automatically"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_rma(&self, row: &sqlx::postgres::PgRow) -> ReturnAuthorization {
        let total_qty: serde_json::Value = row.try_get("total_quantity").unwrap_or(serde_json::json!("0"));
        let total_amt: serde_json::Value = row.try_get("total_amount").unwrap_or(serde_json::json!("0"));
        let total_credit: serde_json::Value = row.try_get("total_credit_amount").unwrap_or(serde_json::json!("0"));

        ReturnAuthorization {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rma_number: row.get("rma_number"),
            customer_id: row.get("customer_id"),
            customer_number: row.get("customer_number"),
            customer_name: row.get("customer_name"),
            return_type: row.get("return_type"),
            status: row.get("status"),
            reason_code: row.get("reason_code"),
            reason_name: row.get("reason_name"),
            original_order_number: row.get("original_order_number"),
            original_order_id: row.get("original_order_id"),
            customer_contact: row.get("customer_contact"),
            customer_email: row.get("customer_email"),
            customer_phone: row.get("customer_phone"),
            return_date: row.get("return_date"),
            expected_receipt_date: row.get("expected_receipt_date"),
            total_quantity: total_qty.to_string(),
            total_amount: total_amt.to_string(),
            total_credit_amount: total_credit.to_string(),
            currency_code: row.get("currency_code"),
            notes: row.get("notes"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            credit_memo_id: row.get("credit_memo_id"),
            credit_memo_number: row.get("credit_memo_number"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_return_line(&self, row: &sqlx::postgres::PgRow) -> ReturnLine {
        let orig_qty: serde_json::Value = row.try_get("original_quantity").unwrap_or(serde_json::json!("0"));
        let ret_qty: serde_json::Value = row.try_get("return_quantity").unwrap_or(serde_json::json!("0"));
        let unit_price: serde_json::Value = row.try_get("unit_price").unwrap_or(serde_json::json!("0"));
        let ret_amt: serde_json::Value = row.try_get("return_amount").unwrap_or(serde_json::json!("0"));
        let cr_amt: serde_json::Value = row.try_get("credit_amount").unwrap_or(serde_json::json!("0"));
        let recv_qty: serde_json::Value = row.try_get("received_quantity").unwrap_or(serde_json::json!("0"));

        ReturnLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            rma_id: row.get("rma_id"),
            line_number: row.get("line_number"),
            item_id: row.get("item_id"),
            item_code: row.get("item_code"),
            item_description: row.get("item_description"),
            original_line_id: row.get("original_line_id"),
            original_quantity: orig_qty.to_string(),
            return_quantity: ret_qty.to_string(),
            unit_price: unit_price.to_string(),
            return_amount: ret_amt.to_string(),
            credit_amount: cr_amt.to_string(),
            reason_code: row.get("reason_code"),
            disposition: row.get("disposition"),
            lot_number: row.get("lot_number"),
            serial_number: row.get("serial_number"),
            condition: row.get("condition"),
            received_quantity: recv_qty.to_string(),
            received_date: row.get("received_date"),
            inspection_status: row.get("inspection_status"),
            inspection_notes: row.get("inspection_notes"),
            credit_status: row.get("credit_status"),
            notes: row.get("notes"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_credit_memo(&self, row: &sqlx::postgres::PgRow) -> CreditMemo {
        let amount: serde_json::Value = row.try_get("amount").unwrap_or(serde_json::json!("0"));
        let applied: serde_json::Value = row.try_get("applied_amount").unwrap_or(serde_json::json!("0"));
        let remaining: serde_json::Value = row.try_get("remaining_amount").unwrap_or(serde_json::json!("0"));

        CreditMemo {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            credit_memo_number: row.get("credit_memo_number"),
            rma_id: row.get("rma_id"),
            rma_number: row.get("rma_number"),
            customer_id: row.get("customer_id"),
            customer_number: row.get("customer_number"),
            customer_name: row.get("customer_name"),
            amount: amount.to_string(),
            currency_code: row.get("currency_code"),
            status: row.get("status"),
            applied_amount: applied.to_string(),
            remaining_amount: remaining.to_string(),
            issue_date: row.get("issue_date"),
            applied_to_invoice_id: row.get("applied_to_invoice_id"),
            applied_to_invoice_number: row.get("applied_to_invoice_number"),
            gl_account_code: row.get("gl_account_code"),
            notes: row.get("notes"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl CustomerReturnsRepository for PostgresCustomerReturnsRepository {
    // ========================================================================
    // Return Reasons
    // ========================================================================

    async fn create_return_reason(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        return_type: &str,
        default_disposition: Option<&str>,
        requires_approval: bool,
        credit_issued_automatically: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnReason> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.return_reasons
                (organization_id, code, name, description, return_type,
                 default_disposition, requires_approval, credit_issued_automatically, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, return_type = $5,
                    default_disposition = $6, requires_approval = $7,
                    credit_issued_automatically = $8, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(return_type)
        .bind(default_disposition).bind(requires_approval).bind(credit_issued_automatically)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_return_reason(&row))
    }

    async fn get_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ReturnReason>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.return_reasons WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_return_reason(&r)))
    }

    async fn list_return_reasons(&self, org_id: Uuid, return_type: Option<&str>) -> AtlasResult<Vec<ReturnReason>> {
        let rows = match return_type {
            Some(rt) => sqlx::query(
                "SELECT * FROM _atlas.return_reasons WHERE organization_id = $1 AND return_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(rt)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.return_reasons WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_return_reason(&r)).collect())
    }

    async fn delete_return_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.return_reasons SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Return Authorizations (RMAs)
    // ========================================================================

    async fn create_rma(
        &self,
        org_id: Uuid,
        rma_number: &str,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        return_type: &str,
        reason_code: Option<&str>,
        reason_name: Option<&str>,
        original_order_number: Option<&str>,
        original_order_id: Option<Uuid>,
        customer_contact: Option<&str>,
        customer_email: Option<&str>,
        customer_phone: Option<&str>,
        return_date: chrono::NaiveDate,
        expected_receipt_date: Option<chrono::NaiveDate>,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnAuthorization> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.return_authorizations
                (organization_id, rma_number, customer_id, customer_number, customer_name,
                 return_type, reason_code, reason_name, original_order_number, original_order_id,
                 customer_contact, customer_email, customer_phone,
                 return_date, expected_receipt_date, currency_code, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rma_number).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(return_type).bind(reason_code).bind(reason_name)
        .bind(original_order_number).bind(original_order_id)
        .bind(customer_contact).bind(customer_email).bind(customer_phone)
        .bind(return_date).bind(expected_receipt_date)
        .bind(currency_code).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_rma(&row))
    }

    async fn get_rma(&self, id: Uuid) -> AtlasResult<Option<ReturnAuthorization>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.return_authorizations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_rma(&r)))
    }

    async fn get_rma_by_number(&self, org_id: Uuid, rma_number: &str) -> AtlasResult<Option<ReturnAuthorization>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.return_authorizations WHERE organization_id = $1 AND rma_number = $2"
        )
        .bind(org_id).bind(rma_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_rma(&r)))
    }

    async fn list_rmas(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
        return_type: Option<&str>,
    ) -> AtlasResult<Vec<ReturnAuthorization>> {
        let mut query = String::from(
            "SELECT * FROM _atlas.return_authorizations WHERE organization_id = $1"
        );
        let mut bind_idx = 2;

        let status_val;
        let customer_val;
        let return_type_val;

        if status.is_some() {
            query.push_str(&format!(" AND status = ${}", bind_idx));
            bind_idx += 1;
        }
        if customer_id.is_some() {
            query.push_str(&format!(" AND customer_id = ${}", bind_idx));
            bind_idx += 1;
        }
        if return_type.is_some() {
            query.push_str(&format!(" AND return_type = ${}", bind_idx));
            bind_idx += 1;
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);

        if let Some(s) = status {
            status_val = s.to_string();
            q = q.bind(&status_val);
        }
        if let Some(c) = customer_id {
            customer_val = c;
            q = q.bind(customer_val);
        }
        if let Some(rt) = return_type {
            return_type_val = rt.to_string();
            q = q.bind(&return_type_val);
        }

        let rows = q.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_rma(&r)).collect())
    }

    async fn update_rma_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<ReturnAuthorization> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.return_authorizations
            SET status = $2, approved_by = $3, approved_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE approved_at END,
                rejected_reason = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_rma(&row))
    }

    async fn update_rma_totals(
        &self,
        id: Uuid,
        total_quantity: &str,
        total_amount: &str,
        total_credit_amount: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.return_authorizations
            SET total_quantity = $2::numeric, total_amount = $3::numeric,
                total_credit_amount = $4::numeric, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_quantity).bind(total_amount).bind(total_credit_amount)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_rma_credit_memo(
        &self,
        id: Uuid,
        credit_memo_id: Uuid,
        credit_memo_number: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.return_authorizations
            SET credit_memo_id = $2, credit_memo_number = $3, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(credit_memo_id).bind(credit_memo_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Return Lines
    // ========================================================================

    async fn create_return_line(
        &self,
        org_id: Uuid,
        rma_id: Uuid,
        line_number: i32,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        original_line_id: Option<Uuid>,
        original_quantity: &str,
        return_quantity: &str,
        unit_price: &str,
        return_amount: &str,
        credit_amount: &str,
        reason_code: Option<&str>,
        disposition: Option<&str>,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        condition: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReturnLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.return_lines
                (organization_id, rma_id, line_number, item_id, item_code, item_description,
                 original_line_id, original_quantity, return_quantity, unit_price,
                 return_amount, credit_amount, reason_code, disposition,
                 lot_number, serial_number, condition, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric, $10::numeric,
                    $11::numeric, $12::numeric, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(rma_id).bind(line_number).bind(item_id).bind(item_code).bind(item_description)
        .bind(original_line_id).bind(original_quantity).bind(return_quantity).bind(unit_price)
        .bind(return_amount).bind(credit_amount).bind(reason_code).bind(disposition)
        .bind(lot_number).bind(serial_number).bind(condition).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_return_line(&row))
    }

    async fn get_return_line(&self, id: Uuid) -> AtlasResult<Option<ReturnLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.return_lines WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_return_line(&r)))
    }

    async fn list_return_lines_by_rma(&self, rma_id: Uuid) -> AtlasResult<Vec<ReturnLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.return_lines WHERE rma_id = $1 ORDER BY line_number"
        )
        .bind(rma_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_return_line(&r)).collect())
    }

    async fn update_return_line_receipt(
        &self,
        id: Uuid,
        received_quantity: &str,
        received_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ReturnLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.return_lines
            SET received_quantity = $2::numeric, received_date = $3, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(received_quantity).bind(received_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_return_line(&row))
    }

    async fn update_return_line_inspection(
        &self,
        id: Uuid,
        inspection_status: &str,
        inspection_notes: Option<&str>,
        disposition: Option<&str>,
    ) -> AtlasResult<ReturnLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.return_lines
            SET inspection_status = $2, inspection_notes = $3, disposition = COALESCE($4, disposition), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(inspection_status).bind(inspection_notes).bind(disposition)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_return_line(&row))
    }

    async fn update_return_line_credit_status(
        &self,
        id: Uuid,
        credit_status: &str,
    ) -> AtlasResult<ReturnLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.return_lines
            SET credit_status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(credit_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_return_line(&row))
    }

    // ========================================================================
    // Credit Memos
    // ========================================================================

    async fn create_credit_memo(
        &self,
        org_id: Uuid,
        credit_memo_number: &str,
        rma_id: Option<Uuid>,
        rma_number: Option<&str>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        gl_account_code: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CreditMemo> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.credit_memos
                (organization_id, credit_memo_number, rma_id, rma_number,
                 customer_id, customer_number, customer_name,
                 amount, remaining_amount, currency_code, gl_account_code, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $8::numeric, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(credit_memo_number).bind(rma_id).bind(rma_number)
        .bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(amount).bind(currency_code).bind(gl_account_code).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_credit_memo(&row))
    }

    async fn get_credit_memo(&self, id: Uuid) -> AtlasResult<Option<CreditMemo>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_memos WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_credit_memo(&r)))
    }

    async fn get_credit_memo_by_number(&self, org_id: Uuid, credit_memo_number: &str) -> AtlasResult<Option<CreditMemo>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.credit_memos WHERE organization_id = $1 AND credit_memo_number = $2"
        )
        .bind(org_id).bind(credit_memo_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_credit_memo(&r)))
    }

    async fn list_credit_memos(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CreditMemo>> {
        let rows = match (customer_id, status) {
            (Some(cid), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.credit_memos WHERE organization_id = $1 AND customer_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(cid).bind(s)
            .fetch_all(&self.pool).await,
            (Some(cid), None) => sqlx::query(
                "SELECT * FROM _atlas.credit_memos WHERE organization_id = $1 AND customer_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(cid)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.credit_memos WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.credit_memos WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_credit_memo(&r)).collect())
    }

    async fn update_credit_memo_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<CreditMemo> {
        let issued_at_expr = if status == "issued" { "CASE WHEN issue_date IS NULL THEN CURRENT_DATE ELSE issue_date END" } else { "issue_date" };
        let query_str = format!(
            r#"UPDATE _atlas.credit_memos SET status = $2, issue_date = {}, updated_at = now() WHERE id = $1 RETURNING *"#,
            issued_at_expr
        );
        let row = sqlx::query(&query_str)
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_credit_memo(&row))
    }
}
