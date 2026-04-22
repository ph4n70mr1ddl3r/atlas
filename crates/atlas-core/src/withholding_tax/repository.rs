//! Withholding Tax Repository
//!
//! PostgreSQL storage for withholding tax codes, groups, supplier assignments,
//! certificates, and withholding tax lines.

use atlas_shared::{
    WithholdingTaxCode, WithholdingTaxGroup, WithholdingTaxGroupMember,
    SupplierWithholdingAssignment, WithholdingCertificate,
    WithholdingTaxLine, AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for withholding tax data storage
#[async_trait]
pub trait WithholdingTaxRepository: Send + Sync {
    // Tax Codes
    async fn create_tax_code(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        rate_percentage: &str,
        threshold_amount: &str,
        threshold_is_cumulative: bool,
        withholding_account_code: Option<&str>,
        expense_account_code: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxCode>;

    async fn get_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxCode>>;
    async fn get_tax_code_by_id(&self, id: Uuid) -> AtlasResult<Option<WithholdingTaxCode>>;
    async fn list_tax_codes(&self, org_id: Uuid, tax_type: Option<&str>) -> AtlasResult<Vec<WithholdingTaxCode>>;
    async fn delete_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Tax Groups
    async fn create_tax_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxGroup>;

    async fn get_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxGroup>>;
    async fn get_tax_group_by_id(&self, id: Uuid) -> AtlasResult<Option<WithholdingTaxGroup>>;
    async fn list_tax_groups(&self, org_id: Uuid) -> AtlasResult<Vec<WithholdingTaxGroup>>;
    async fn delete_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Tax Group Members
    async fn add_group_member(
        &self,
        group_id: Uuid,
        tax_code_id: Uuid,
        rate_override: Option<&str>,
        display_order: i32,
    ) -> AtlasResult<WithholdingTaxGroupMember>;

    async fn list_group_members(&self, group_id: Uuid) -> AtlasResult<Vec<WithholdingTaxGroupMember>>;
    async fn remove_group_member(&self, id: Uuid) -> AtlasResult<()>;

    // Supplier Assignments
    async fn create_supplier_assignment(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_group_id: Uuid,
        is_exempt: bool,
        exemption_reason: Option<&str>,
        exemption_certificate: Option<&str>,
        exemption_valid_until: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierWithholdingAssignment>;

    async fn get_supplier_assignment(&self, org_id: Uuid, supplier_id: Uuid) -> AtlasResult<Option<SupplierWithholdingAssignment>>;
    async fn list_supplier_assignments(&self, org_id: Uuid) -> AtlasResult<Vec<SupplierWithholdingAssignment>>;
    async fn delete_supplier_assignment(&self, id: Uuid) -> AtlasResult<()>;

    // Withholding Tax Lines
    async fn create_withholding_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        payment_number: Option<&str>,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        tax_code_id: Uuid,
        tax_code: &str,
        tax_code_name: Option<&str>,
        tax_type: &str,
        rate_percentage: &str,
        taxable_amount: &str,
        withheld_amount: &str,
        withholding_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxLine>;

    async fn get_withholding_lines_by_payment(&self, payment_id: Uuid) -> AtlasResult<Vec<WithholdingTaxLine>>;
    async fn get_withholding_lines_by_supplier(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<WithholdingTaxLine>>;
    async fn update_withholding_line_status(&self, id: Uuid, status: &str, remittance_date: Option<chrono::NaiveDate>, remittance_reference: Option<&str>) -> AtlasResult<WithholdingTaxLine>;

    // Certificates
    async fn create_certificate(
        &self,
        org_id: Uuid,
        certificate_number: &str,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_type: &str,
        tax_code_id: Uuid,
        tax_code: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        total_invoice_amount: &str,
        total_withheld_amount: &str,
        rate_percentage: &str,
        payment_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingCertificate>;

    async fn get_certificate(&self, id: Uuid) -> AtlasResult<Option<WithholdingCertificate>>;
    async fn get_certificate_by_number(&self, org_id: Uuid, certificate_number: &str) -> AtlasResult<Option<WithholdingCertificate>>;
    async fn list_certificates(&self, org_id: Uuid, supplier_id: Option<Uuid>) -> AtlasResult<Vec<WithholdingCertificate>>;
    async fn update_certificate_status(&self, id: Uuid, status: &str) -> AtlasResult<WithholdingCertificate>;
}

/// PostgreSQL implementation
pub struct PostgresWithholdingTaxRepository {
    pool: PgPool,
}

impl PostgresWithholdingTaxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_tax_code(&self, row: &sqlx::postgres::PgRow) -> WithholdingTaxCode {
        let rate: serde_json::Value = row.try_get("rate_percentage").unwrap_or(serde_json::json!("0"));
        let threshold: serde_json::Value = row.try_get("threshold_amount").unwrap_or(serde_json::json!("0"));

        WithholdingTaxCode {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            tax_type: row.get("tax_type"),
            rate_percentage: rate.to_string(),
            threshold_amount: threshold.to_string(),
            threshold_is_cumulative: row.get("threshold_is_cumulative"),
            withholding_account_code: row.get("withholding_account_code"),
            expense_account_code: row.get("expense_account_code"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_tax_group(&self, row: &sqlx::postgres::PgRow) -> WithholdingTaxGroup {
        WithholdingTaxGroup {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            tax_codes: vec![], // Loaded separately
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_group_member(&self, row: &sqlx::postgres::PgRow) -> WithholdingTaxGroupMember {
        let rate_override: Option<serde_json::Value> = row.try_get("rate_override").ok().flatten();
        WithholdingTaxGroupMember {
            id: row.get("id"),
            group_id: row.get("group_id"),
            tax_code_id: row.get("tax_code_id"),
            tax_code: row.get("tax_code"),
            tax_code_name: row.get("tax_code_name"),
            rate_override: rate_override.map(|v| v.to_string()),
            is_active: row.get("is_active"),
            display_order: row.get("display_order"),
        }
    }

    fn row_to_supplier_assignment(&self, row: &sqlx::postgres::PgRow) -> SupplierWithholdingAssignment {
        SupplierWithholdingAssignment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            supplier_id: row.get("supplier_id"),
            supplier_number: row.get("supplier_number"),
            supplier_name: row.get("supplier_name"),
            tax_group_id: row.get("tax_group_id"),
            tax_group_code: row.get("tax_group_code"),
            tax_group_name: row.get("tax_group_name"),
            is_exempt: row.get("is_exempt"),
            exemption_reason: row.get("exemption_reason"),
            exemption_certificate: row.get("exemption_certificate"),
            exemption_valid_until: row.get("exemption_valid_until"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_certificate(&self, row: &sqlx::postgres::PgRow) -> WithholdingCertificate {
        let total_invoice: serde_json::Value = row.try_get("total_invoice_amount").unwrap_or(serde_json::json!("0"));
        let total_withheld: serde_json::Value = row.try_get("total_withheld_amount").unwrap_or(serde_json::json!("0"));
        let rate: serde_json::Value = row.try_get("rate_percentage").unwrap_or(serde_json::json!("0"));

        WithholdingCertificate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            certificate_number: row.get("certificate_number"),
            supplier_id: row.get("supplier_id"),
            supplier_number: row.get("supplier_number"),
            supplier_name: row.get("supplier_name"),
            tax_type: row.get("tax_type"),
            tax_code_id: row.get("tax_code_id"),
            tax_code: row.get("tax_code"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            total_invoice_amount: total_invoice.to_string(),
            total_withheld_amount: total_withheld.to_string(),
            rate_percentage: rate.to_string(),
            payment_ids: row.get("payment_ids"),
            status: row.get("status"),
            issued_at: row.get("issued_at"),
            acknowledged_at: row.get("acknowledged_at"),
            notes: row.get("notes"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_withholding_line(&self, row: &sqlx::postgres::PgRow) -> WithholdingTaxLine {
        let taxable: serde_json::Value = row.try_get("taxable_amount").unwrap_or(serde_json::json!("0"));
        let withheld: serde_json::Value = row.try_get("withheld_amount").unwrap_or(serde_json::json!("0"));
        let rate: serde_json::Value = row.try_get("rate_percentage").unwrap_or(serde_json::json!("0"));

        WithholdingTaxLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            payment_id: row.get("payment_id"),
            payment_number: row.get("payment_number"),
            invoice_id: row.get("invoice_id"),
            invoice_number: row.get("invoice_number"),
            supplier_id: row.get("supplier_id"),
            supplier_name: row.get("supplier_name"),
            tax_code_id: row.get("tax_code_id"),
            tax_code: row.get("tax_code"),
            tax_code_name: row.get("tax_code_name"),
            tax_type: row.get("tax_type"),
            rate_percentage: rate.to_string(),
            taxable_amount: taxable.to_string(),
            withheld_amount: withheld.to_string(),
            withholding_account_code: row.get("withholding_account_code"),
            status: row.get("status"),
            remittance_date: row.get("remittance_date"),
            remittance_reference: row.get("remittance_reference"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl WithholdingTaxRepository for PostgresWithholdingTaxRepository {
    // ========================================================================
    // Tax Codes
    // ========================================================================

    async fn create_tax_code(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        rate_percentage: &str,
        threshold_amount: &str,
        threshold_is_cumulative: bool,
        withholding_account_code: Option<&str>,
        expense_account_code: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxCode> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.withholding_tax_codes
                (organization_id, code, name, description, tax_type,
                 rate_percentage, threshold_amount, threshold_is_cumulative,
                 withholding_account_code, expense_account_code,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, tax_type = $5,
                    rate_percentage = $6::numeric, threshold_amount = $7::numeric,
                    threshold_is_cumulative = $8,
                    withholding_account_code = $9, expense_account_code = $10,
                    effective_from = $11, effective_to = $12, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(tax_type)
        .bind(rate_percentage).bind(threshold_amount).bind(threshold_is_cumulative)
        .bind(withholding_account_code).bind(expense_account_code)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_tax_code(&row))
    }

    async fn get_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxCode>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_codes WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_tax_code(&r)))
    }

    async fn get_tax_code_by_id(&self, id: Uuid) -> AtlasResult<Option<WithholdingTaxCode>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_codes WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_tax_code(&r)))
    }

    async fn list_tax_codes(&self, org_id: Uuid, tax_type: Option<&str>) -> AtlasResult<Vec<WithholdingTaxCode>> {
        let rows = match tax_type {
            Some(tt) => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_codes WHERE organization_id = $1 AND tax_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(tt)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_codes WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_tax_code(r)).collect())
    }

    async fn delete_tax_code(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.withholding_tax_codes SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Tax Groups
    // ========================================================================

    async fn create_tax_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxGroup> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.withholding_tax_groups
                (organization_id, code, name, description, created_by)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_tax_group(&row))
    }

    async fn get_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WithholdingTaxGroup>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_groups WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut group = self.row_to_tax_group(&r);
                group.tax_codes = self.list_group_members(group.id).await?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn get_tax_group_by_id(&self, id: Uuid) -> AtlasResult<Option<WithholdingTaxGroup>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_groups WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut group = self.row_to_tax_group(&r);
                group.tax_codes = self.list_group_members(group.id).await?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn list_tax_groups(&self, org_id: Uuid) -> AtlasResult<Vec<WithholdingTaxGroup>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_groups WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            let mut group = self.row_to_tax_group(&row);
            group.tax_codes = self.list_group_members(group.id).await?;
            groups.push(group);
        }
        Ok(groups)
    }

    async fn delete_tax_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.withholding_tax_groups SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Tax Group Members
    // ========================================================================

    async fn add_group_member(
        &self,
        group_id: Uuid,
        tax_code_id: Uuid,
        rate_override: Option<&str>,
        display_order: i32,
    ) -> AtlasResult<WithholdingTaxGroupMember> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.withholding_tax_group_members
                (group_id, tax_code_id, rate_override, display_order)
            VALUES ($1, $2, $3::numeric, $4)
            RETURNING *
            "#,
        )
        .bind(group_id).bind(tax_code_id).bind(rate_override).bind(display_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_group_member(&row))
    }

    async fn list_group_members(&self, group_id: Uuid) -> AtlasResult<Vec<WithholdingTaxGroupMember>> {
        let rows = sqlx::query(
            r#"
            SELECT m.*, c.code as tax_code, c.name as tax_code_name
            FROM _atlas.withholding_tax_group_members m
            JOIN _atlas.withholding_tax_codes c ON c.id = m.tax_code_id
            WHERE m.group_id = $1 AND m.is_active = true
            ORDER BY m.display_order
            "#
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_group_member(r)).collect())
    }

    async fn remove_group_member(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.withholding_tax_group_members SET is_active = false WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Supplier Assignments
    // ========================================================================

    async fn create_supplier_assignment(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_group_id: Uuid,
        is_exempt: bool,
        exemption_reason: Option<&str>,
        exemption_certificate: Option<&str>,
        exemption_valid_until: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierWithholdingAssignment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.supplier_withholding_assignments
                (organization_id, supplier_id, supplier_number, supplier_name,
                 tax_group_id, is_exempt, exemption_reason, exemption_certificate,
                 exemption_valid_until, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, supplier_id) DO UPDATE
                SET tax_group_id = $5, is_exempt = $6, exemption_reason = $7,
                    exemption_certificate = $8, exemption_valid_until = $9,
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(supplier_id).bind(supplier_number).bind(supplier_name)
        .bind(tax_group_id).bind(is_exempt).bind(exemption_reason)
        .bind(exemption_certificate).bind(exemption_valid_until).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_supplier_assignment(&row))
    }

    async fn get_supplier_assignment(&self, org_id: Uuid, supplier_id: Uuid) -> AtlasResult<Option<SupplierWithholdingAssignment>> {
        let row = sqlx::query(
            r#"
            SELECT a.*, g.code as tax_group_code, g.name as tax_group_name
            FROM _atlas.supplier_withholding_assignments a
            JOIN _atlas.withholding_tax_groups g ON g.id = a.tax_group_id
            WHERE a.organization_id = $1 AND a.supplier_id = $2 AND a.is_active = true
            "#
        )
        .bind(org_id).bind(supplier_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_supplier_assignment(&r)))
    }

    async fn list_supplier_assignments(&self, org_id: Uuid) -> AtlasResult<Vec<SupplierWithholdingAssignment>> {
        let rows = sqlx::query(
            r#"
            SELECT a.*, g.code as tax_group_code, g.name as tax_group_name
            FROM _atlas.supplier_withholding_assignments a
            JOIN _atlas.withholding_tax_groups g ON g.id = a.tax_group_id
            WHERE a.organization_id = $1 AND a.is_active = true
            ORDER BY a.supplier_name
            "#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_supplier_assignment(r)).collect())
    }

    async fn delete_supplier_assignment(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.supplier_withholding_assignments SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Withholding Tax Lines
    // ========================================================================

    async fn create_withholding_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        payment_number: Option<&str>,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        tax_code_id: Uuid,
        tax_code: &str,
        tax_code_name: Option<&str>,
        tax_type: &str,
        rate_percentage: &str,
        taxable_amount: &str,
        withheld_amount: &str,
        withholding_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingTaxLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.withholding_tax_lines
                (organization_id, payment_id, payment_number,
                 invoice_id, invoice_number,
                 supplier_id, supplier_name,
                 tax_code_id, tax_code, tax_code_name, tax_type,
                 rate_percentage, taxable_amount, withheld_amount,
                 withholding_account_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                    $12::numeric, $13::numeric, $14::numeric, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(payment_id).bind(payment_number)
        .bind(invoice_id).bind(invoice_number)
        .bind(supplier_id).bind(supplier_name)
        .bind(tax_code_id).bind(tax_code).bind(tax_code_name).bind(tax_type)
        .bind(rate_percentage).bind(taxable_amount).bind(withheld_amount)
        .bind(withholding_account_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_withholding_line(&row))
    }

    async fn get_withholding_lines_by_payment(&self, payment_id: Uuid) -> AtlasResult<Vec<WithholdingTaxLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.withholding_tax_lines WHERE payment_id = $1 ORDER BY created_at"
        )
        .bind(payment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_withholding_line(r)).collect())
    }

    async fn get_withholding_lines_by_supplier(
        &self,
        org_id: Uuid,
        supplier_id: Uuid,
        from_date: Option<chrono::NaiveDate>,
        to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<WithholdingTaxLine>> {
        let rows = match (from_date, to_date) {
            (Some(from), Some(to)) => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_lines WHERE organization_id = $1 AND supplier_id = $2 AND created_at::date >= $3 AND created_at::date <= $4 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(supplier_id).bind(from).bind(to)
            .fetch_all(&self.pool).await,
            (Some(from), None) => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_lines WHERE organization_id = $1 AND supplier_id = $2 AND created_at::date >= $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(supplier_id).bind(from)
            .fetch_all(&self.pool).await,
            (None, Some(to)) => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_lines WHERE organization_id = $1 AND supplier_id = $2 AND created_at::date <= $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(supplier_id).bind(to)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.withholding_tax_lines WHERE organization_id = $1 AND supplier_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(supplier_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_withholding_line(r)).collect())
    }

    async fn update_withholding_line_status(
        &self,
        id: Uuid,
        status: &str,
        remittance_date: Option<chrono::NaiveDate>,
        remittance_reference: Option<&str>,
    ) -> AtlasResult<WithholdingTaxLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.withholding_tax_lines
            SET status = $2, remittance_date = $3, remittance_reference = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(remittance_date).bind(remittance_reference)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_withholding_line(&row))
    }

    // ========================================================================
    // Certificates
    // ========================================================================

    async fn create_certificate(
        &self,
        org_id: Uuid,
        certificate_number: &str,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        tax_type: &str,
        tax_code_id: Uuid,
        tax_code: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        total_invoice_amount: &str,
        total_withheld_amount: &str,
        rate_percentage: &str,
        payment_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<WithholdingCertificate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.withholding_certificates
                (organization_id, certificate_number,
                 supplier_id, supplier_number, supplier_name,
                 tax_type, tax_code_id, tax_code,
                 period_start, period_end,
                 total_invoice_amount, total_withheld_amount, rate_percentage,
                 payment_ids, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, $13::numeric, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(certificate_number)
        .bind(supplier_id).bind(supplier_number).bind(supplier_name)
        .bind(tax_type).bind(tax_code_id).bind(tax_code)
        .bind(period_start).bind(period_end)
        .bind(total_invoice_amount).bind(total_withheld_amount).bind(rate_percentage)
        .bind(payment_ids).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_certificate(&row))
    }

    async fn get_certificate(&self, id: Uuid) -> AtlasResult<Option<WithholdingCertificate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_certificates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_certificate(&r)))
    }

    async fn get_certificate_by_number(&self, org_id: Uuid, certificate_number: &str) -> AtlasResult<Option<WithholdingCertificate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.withholding_certificates WHERE organization_id = $1 AND certificate_number = $2"
        )
        .bind(org_id).bind(certificate_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_certificate(&r)))
    }

    async fn list_certificates(&self, org_id: Uuid, supplier_id: Option<Uuid>) -> AtlasResult<Vec<WithholdingCertificate>> {
        let rows = match supplier_id {
            Some(sid) => sqlx::query(
                "SELECT * FROM _atlas.withholding_certificates WHERE organization_id = $1 AND supplier_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(sid)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.withholding_certificates WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_certificate(r)).collect())
    }

    async fn update_certificate_status(&self, id: Uuid, status: &str) -> AtlasResult<WithholdingCertificate> {
        let issued_at_expr = if status == "issued" { "CASE WHEN issued_at IS NULL THEN now() ELSE issued_at END" } else { "issued_at" };
        let query_str = format!(
            r#"UPDATE _atlas.withholding_certificates SET status = $2, issued_at = {}, updated_at = now() WHERE id = $1 RETURNING *"#,
            issued_at_expr
        );
        let row = sqlx::query(&query_str)
            .bind(id).bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_certificate(&row))
    }
}
