//! Tax Reporting Repository
//!
//! Storage interface for tax reporting data.

use atlas_shared::{
    TaxReturnTemplate, TaxReturnTemplateLine, TaxReturn, TaxReturnLine,
    TaxFilingCalendarEntry, TaxReportingDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for tax reporting data storage
#[async_trait]
pub trait TaxReportingRepository: Send + Sync {
    // Templates
    async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        tax_type: &str,
        jurisdiction_code: Option<&str>,
        filing_frequency: &str,
        return_form_number: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturnTemplate>;

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<TaxReturnTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxReturnTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplate>>;

    // Template Lines
    async fn create_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        line_number: i32,
        box_code: &str,
        box_name: &str,
        description: Option<&str>,
        line_type: &str,
        calculation_formula: Option<&str>,
        account_code_filter: Option<&str>,
        tax_rate_code_filter: Option<&str>,
        is_debit: bool,
        display_order: i32,
    ) -> AtlasResult<TaxReturnTemplateLine>;

    async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplateLine>>;

    // Tax Returns
    async fn create_return(
        &self,
        org_id: Uuid,
        return_number: &str,
        template_id: Uuid,
        template_name: Option<&str>,
        tax_type: Option<&str>,
        jurisdiction_code: Option<&str>,
        filing_period_start: chrono::NaiveDate,
        filing_period_end: chrono::NaiveDate,
        filing_due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn>;

    async fn get_return(&self, id: Uuid) -> AtlasResult<Option<TaxReturn>>;
    async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TaxReturn>>;
    async fn update_return_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        filed_by: Option<Uuid>,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn>;
    async fn update_return_totals(
        &self,
        id: Uuid,
        total_tax_amount: &str,
        total_taxable_amount: &str,
        total_exempt_amount: &str,
        total_input_tax: &str,
        total_output_tax: &str,
        net_tax_due: &str,
        total_amount_due: &str,
    ) -> AtlasResult<()>;
    async fn update_return_filing(
        &self,
        id: Uuid,
        status: &str,
        filing_method: &str,
        filing_reference: Option<&str>,
        filed_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn>;
    async fn update_return_payment(
        &self,
        id: Uuid,
        status: &str,
        payment_amount: &str,
        payment_reference: Option<&str>,
    ) -> AtlasResult<TaxReturn>;

    // Return Lines
    async fn create_return_line(
        &self,
        org_id: Uuid,
        tax_return_id: Uuid,
        template_line_id: Option<Uuid>,
        line_number: i32,
        box_code: &str,
        box_name: Option<&str>,
        line_type: &str,
        amount: &str,
        calculated_amount: &str,
        override_amount: Option<&str>,
        final_amount: &str,
        description: Option<&str>,
        source_count: i32,
    ) -> AtlasResult<TaxReturnLine>;

    async fn get_return_line(&self, id: Uuid) -> AtlasResult<Option<TaxReturnLine>>;
    async fn list_return_lines(&self, tax_return_id: Uuid) -> AtlasResult<Vec<TaxReturnLine>>;
    async fn update_return_line(
        &self,
        id: Uuid,
        amount: &str,
        override_amount: Option<&str>,
        final_amount: &str,
    ) -> AtlasResult<TaxReturnLine>;

    // Filing Calendar
    async fn list_filing_calendar(&self, org_id: Uuid) -> AtlasResult<Vec<TaxFilingCalendarEntry>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<TaxReportingDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresTaxReportingRepository {
    pool: PgPool,
}

impl PostgresTaxReportingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_template(row: &sqlx::postgres::PgRow) -> TaxReturnTemplate {
    TaxReturnTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        tax_type: row.get("tax_type"),
        jurisdiction_code: row.get("jurisdiction_code"),
        filing_frequency: row.get("filing_frequency"),
        return_form_number: row.get("return_form_number"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_template_line(row: &sqlx::postgres::PgRow) -> TaxReturnTemplateLine {
    TaxReturnTemplateLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        line_number: row.get("line_number"),
        box_code: row.get("box_code"),
        box_name: row.get("box_name"),
        description: row.get("description"),
        line_type: row.get("line_type"),
        calculation_formula: row.get("calculation_formula"),
        account_code_filter: row.get("account_code_filter"),
        tax_rate_code_filter: row.get("tax_rate_code_filter"),
        is_debit: row.get("is_debit"),
        display_order: row.get("display_order"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_return(row: &sqlx::postgres::PgRow) -> TaxReturn {
    TaxReturn {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        return_number: row.get("return_number"),
        template_id: row.get("template_id"),
        template_name: row.get("template_name"),
        tax_type: row.get("tax_type"),
        jurisdiction_code: row.get("jurisdiction_code"),
        filing_period_start: row.get("filing_period_start"),
        filing_period_end: row.get("filing_period_end"),
        filing_due_date: row.get("filing_due_date"),
        total_tax_amount: row.try_get("total_tax_amount").unwrap_or("0".to_string()),
        total_taxable_amount: row.try_get("total_taxable_amount").unwrap_or("0".to_string()),
        total_exempt_amount: row.try_get("total_exempt_amount").unwrap_or("0".to_string()),
        total_input_tax: row.try_get("total_input_tax").unwrap_or("0".to_string()),
        total_output_tax: row.try_get("total_output_tax").unwrap_or("0".to_string()),
        net_tax_due: row.try_get("net_tax_due").unwrap_or("0".to_string()),
        penalty_amount: row.try_get("penalty_amount").unwrap_or("0".to_string()),
        interest_amount: row.try_get("interest_amount").unwrap_or("0".to_string()),
        total_amount_due: row.try_get("total_amount_due").unwrap_or("0".to_string()),
        payment_amount: row.try_get("payment_amount").unwrap_or("0".to_string()),
        refund_amount: row.try_get("refund_amount").unwrap_or("0".to_string()),
        status: row.get("status"),
        filing_method: row.get("filing_method"),
        filing_reference: row.get("filing_reference"),
        filing_date: row.get("filing_date"),
        payment_date: row.get("payment_date"),
        payment_reference: row.get("payment_reference"),
        amendment_reason: row.get("amendment_reason"),
        notes: row.get("notes"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        filed_by: row.get("filed_by"),
        filed_at: row.get("filed_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_return_line(row: &sqlx::postgres::PgRow) -> TaxReturnLine {
    TaxReturnLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        tax_return_id: row.get("tax_return_id"),
        template_line_id: row.get("template_line_id"),
        line_number: row.get("line_number"),
        box_code: row.get("box_code"),
        box_name: row.get("box_name"),
        line_type: row.get("line_type"),
        amount: row.try_get("amount").unwrap_or("0".to_string()),
        calculated_amount: row.try_get("calculated_amount").unwrap_or("0".to_string()),
        override_amount: row.try_get("override_amount").unwrap_or(None),
        final_amount: row.try_get("final_amount").unwrap_or("0".to_string()),
        description: row.get("description"),
        source_count: row.try_get("source_count").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_filing_calendar(row: &sqlx::postgres::PgRow) -> TaxFilingCalendarEntry {
    TaxFilingCalendarEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        due_date: row.get("due_date"),
        filing_status: row.get("filing_status"),
        return_id: row.get("return_id"),
        extension_filed: row.get("extension_filed"),
        extension_due_date: row.get("extension_due_date"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl TaxReportingRepository for PostgresTaxReportingRepository {
    async fn create_template(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        tax_type: &str, jurisdiction_code: Option<&str>, filing_frequency: &str,
        return_form_number: Option<&str>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturnTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_return_templates
                (organization_id, code, name, description, tax_type, jurisdiction_code,
                 filing_frequency, return_form_number, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        ).bind(org_id).bind(code).bind(name).bind(description)
        .bind(tax_type).bind(jurisdiction_code).bind(filing_frequency)
        .bind(return_form_number).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template(&row))
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<TaxReturnTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.tax_return_templates WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TaxReturnTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.tax_return_templates WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_return_templates WHERE organization_id = $1 ORDER BY tax_type, code"
        ).bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn create_template_line(
        &self,
        org_id: Uuid, template_id: Uuid, line_number: i32,
        box_code: &str, box_name: &str, description: Option<&str>,
        line_type: &str, calculation_formula: Option<&str>,
        account_code_filter: Option<&str>, tax_rate_code_filter: Option<&str>,
        is_debit: bool, display_order: i32,
    ) -> AtlasResult<TaxReturnTemplateLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_return_template_lines
                (organization_id, template_id, line_number, box_code, box_name, description,
                 line_type, calculation_formula, account_code_filter, tax_rate_code_filter,
                 is_debit, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        ).bind(org_id).bind(template_id).bind(line_number).bind(box_code).bind(box_name)
        .bind(description).bind(line_type).bind(calculation_formula)
        .bind(account_code_filter).bind(tax_rate_code_filter)
        .bind(is_debit).bind(display_order)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_template_line(&row))
    }

    async fn list_template_lines(&self, template_id: Uuid) -> AtlasResult<Vec<TaxReturnTemplateLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_return_template_lines WHERE template_id = $1 ORDER BY display_order, line_number"
        ).bind(template_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template_line).collect())
    }

    async fn create_return(
        &self,
        org_id: Uuid, return_number: &str, template_id: Uuid,
        template_name: Option<&str>, tax_type: Option<&str>,
        jurisdiction_code: Option<&str>,
        filing_period_start: chrono::NaiveDate, filing_period_end: chrono::NaiveDate,
        filing_due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_returns
                (organization_id, return_number, template_id, template_name, tax_type,
                 jurisdiction_code, filing_period_start, filing_period_end, filing_due_date, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        ).bind(org_id).bind(return_number).bind(template_id).bind(template_name)
        .bind(tax_type).bind(jurisdiction_code)
        .bind(filing_period_start).bind(filing_period_end).bind(filing_due_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }

    async fn get_return(&self, id: Uuid) -> AtlasResult<Option<TaxReturn>> {
        let row = sqlx::query("SELECT * FROM _atlas.tax_returns WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_return(&r)))
    }

    async fn list_returns(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TaxReturn>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.tax_returns
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY filing_period_end DESC"#,
        ).bind(org_id).bind(status).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_return).collect())
    }

    async fn update_return_status(
        &self, id: Uuid, status: &str, submitted_by: Option<Uuid>,
        filed_by: Option<Uuid>, approved_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.tax_returns
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                filed_by = COALESCE($4, filed_by),
                filed_at = CASE WHEN $2 = 'filed' THEN now() ELSE filed_at END,
                approved_by = COALESCE($5, approved_by),
                approved_at = CASE WHEN $2 IN ('submitted','filed') THEN now() ELSE approved_at END,
                updated_at = now()
            WHERE id = $1 RETURNING *
            "#,
        ).bind(id).bind(status).bind(submitted_by).bind(filed_by).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }

    async fn update_return_totals(
        &self, id: Uuid, total_tax_amount: &str, total_taxable_amount: &str,
        total_exempt_amount: &str, total_input_tax: &str, total_output_tax: &str,
        net_tax_due: &str, total_amount_due: &str,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.tax_returns
               SET total_tax_amount = $2::decimal, total_taxable_amount = $3::decimal,
                   total_exempt_amount = $4::decimal, total_input_tax = $5::decimal,
                   total_output_tax = $6::decimal, net_tax_due = $7::decimal,
                   total_amount_due = $8::decimal, updated_at = now()
               WHERE id = $1"#,
        ).bind(id).bind(total_tax_amount).bind(total_taxable_amount)
        .bind(total_exempt_amount).bind(total_input_tax).bind(total_output_tax)
        .bind(net_tax_due).bind(total_amount_due)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_return_filing(
        &self, id: Uuid, status: &str, filing_method: &str,
        filing_reference: Option<&str>, filed_by: Option<Uuid>,
    ) -> AtlasResult<TaxReturn> {
        let row = sqlx::query(
            r#"UPDATE _atlas.tax_returns
               SET status = $2, filing_method = $3, filing_reference = $4,
                   filed_by = COALESCE($5, filed_by), filed_at = now(),
                   filing_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(filing_method).bind(filing_reference).bind(filed_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }

    async fn update_return_payment(
        &self, id: Uuid, status: &str, payment_amount: &str, payment_reference: Option<&str>,
    ) -> AtlasResult<TaxReturn> {
        let row = sqlx::query(
            r#"UPDATE _atlas.tax_returns
               SET status = $2, payment_amount = $3::decimal, payment_reference = $4,
                   payment_date = CURRENT_DATE, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(status).bind(payment_amount).bind(payment_reference)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return(&row))
    }

    async fn create_return_line(
        &self,
        org_id: Uuid, tax_return_id: Uuid, template_line_id: Option<Uuid>,
        line_number: i32, box_code: &str, box_name: Option<&str>,
        line_type: &str, amount: &str, calculated_amount: &str,
        override_amount: Option<&str>, final_amount: &str,
        description: Option<&str>, source_count: i32,
    ) -> AtlasResult<TaxReturnLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.tax_return_lines
                (organization_id, tax_return_id, template_line_id, line_number,
                 box_code, box_name, line_type, amount, calculated_amount,
                 override_amount, final_amount, description, source_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::decimal, $9::decimal,
                    $10::decimal, $11::decimal, $12, $13)
            RETURNING *
            "#,
        ).bind(org_id).bind(tax_return_id).bind(template_line_id).bind(line_number)
        .bind(box_code).bind(box_name).bind(line_type).bind(amount).bind(calculated_amount)
        .bind(override_amount).bind(final_amount).bind(description).bind(source_count)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return_line(&row))
    }

    async fn get_return_line(&self, id: Uuid) -> AtlasResult<Option<TaxReturnLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.tax_return_lines WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_return_line(&r)))
    }

    async fn list_return_lines(&self, tax_return_id: Uuid) -> AtlasResult<Vec<TaxReturnLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_return_lines WHERE tax_return_id = $1 ORDER BY line_number"
        ).bind(tax_return_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_return_line).collect())
    }

    async fn update_return_line(
        &self, id: Uuid, amount: &str, override_amount: Option<&str>, final_amount: &str,
    ) -> AtlasResult<TaxReturnLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.tax_return_lines
               SET amount = $2::decimal, override_amount = $3::decimal,
                   final_amount = $4::decimal, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(amount).bind(override_amount).bind(final_amount)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_return_line(&row))
    }

    async fn list_filing_calendar(&self, org_id: Uuid) -> AtlasResult<Vec<TaxFilingCalendarEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.tax_filing_calendar WHERE organization_id = $1 ORDER BY due_date"
        ).bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_filing_calendar).collect())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<TaxReportingDashboardSummary> {
        let tmpl_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active) as active
            FROM _atlas.tax_return_templates WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let ret_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft,
                COUNT(*) FILTER (WHERE status = 'filed' OR status = 'paid') as filed,
                COUNT(*) FILTER (WHERE status = 'draft' AND filing_due_date < CURRENT_DATE) as overdue,
                COALESCE(SUM(payment_amount), 0) as total_paid,
                COALESCE(SUM(net_tax_due) - SUM(payment_amount), 0) as total_due,
                COALESCE(SUM(refund_amount), 0) as total_refunds
            FROM _atlas.tax_returns WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let upcoming = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.tax_filing_calendar
               WHERE organization_id = $1 AND filing_status IN ('upcoming', 'due_soon')"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(TaxReportingDashboardSummary {
            total_templates: tmpl_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            active_templates: tmpl_row.try_get::<i64, _>("active").unwrap_or(0) as i32,
            total_returns: ret_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            draft_returns: ret_row.try_get::<i64, _>("draft").unwrap_or(0) as i32,
            filed_returns: ret_row.try_get::<i64, _>("filed").unwrap_or(0) as i32,
            overdue_returns: ret_row.try_get::<i64, _>("overdue").unwrap_or(0) as i32,
            total_tax_paid: format!("{:.2}", ret_row.try_get::<f64, _>("total_paid").unwrap_or(0.0)),
            total_tax_due: format!("{:.2}", ret_row.try_get::<f64, _>("total_due").unwrap_or(0.0)),
            total_refunds: format!("{:.2}", ret_row.try_get::<f64, _>("total_refunds").unwrap_or(0.0)),
            upcoming_filings: upcoming.try_get::<i64, _>("cnt").unwrap_or(0) as i32,
        })
    }
}
