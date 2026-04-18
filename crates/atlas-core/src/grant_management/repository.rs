//! Grant Management Repository
//!
//! PostgreSQL storage for grant sponsors, awards, budgets, expenditures,
//! billings, and compliance reports.

use atlas_shared::{
    GrantSponsor, GrantIndirectCostRate, GrantAward, GrantBudgetLine,
    GrantExpenditure, GrantBilling, GrantComplianceReport, GrantDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for grant management data storage
#[async_trait]
pub trait GrantManagementRepository: Send + Sync {
    // Sponsors
    async fn create_sponsor(
        &self, org_id: Uuid, sponsor_code: &str, name: &str, sponsor_type: &str,
        country_code: Option<&str>, taxpayer_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state_province: Option<&str>, postal_code: Option<&str>,
        payment_terms: Option<&str>, billing_frequency: &str, currency_code: &str,
        credit_limit: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantSponsor>;
    async fn get_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GrantSponsor>>;
    async fn list_sponsors(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantSponsor>>;
    async fn delete_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Indirect Cost Rates
    async fn create_indirect_cost_rate(
        &self, org_id: Uuid, rate_name: &str, rate_type: &str,
        rate_percentage: &str, base_type: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        negotiated_by: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantIndirectCostRate>;
    async fn list_indirect_cost_rates(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantIndirectCostRate>>;

    // Awards
    #[allow(clippy::too_many_arguments)]
    async fn create_award(
        &self, org_id: Uuid, award_number: &str, award_title: &str,
        sponsor_id: Uuid, sponsor_name: Option<&str>, sponsor_award_number: Option<&str>,
        award_type: &str, award_purpose: Option<&str>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        total_award_amount: &str, direct_costs_total: &str, indirect_costs_total: &str,
        cost_sharing_total: &str, currency_code: &str,
        indirect_cost_rate_id: Option<Uuid>, indirect_cost_rate: &str,
        cost_sharing_required: bool, cost_sharing_percent: &str,
        principal_investigator_id: Option<Uuid>, principal_investigator_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        gl_revenue_account: Option<&str>, gl_receivable_account: Option<&str>,
        gl_deferred_account: Option<&str>,
        billing_frequency: &str, billing_basis: &str,
        reporting_requirements: Option<&str>, compliance_notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GrantAward>;
    async fn get_award(&self, id: Uuid) -> AtlasResult<Option<GrantAward>>;
    async fn get_award_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GrantAward>>;
    async fn list_awards(&self, org_id: Uuid, status: Option<&str>, sponsor_id: Option<Uuid>) -> AtlasResult<Vec<GrantAward>>;
    async fn update_award_status(&self, id: Uuid, status: &str, closeout_date: Option<chrono::NaiveDate>, closeout_notes: Option<&str>) -> AtlasResult<GrantAward>;
    async fn update_award_totals(&self, id: Uuid, total_expenditures: &str, total_commitments: &str, total_billed: &str, total_collected: &str, available_balance: &str) -> AtlasResult<()>;

    // Budget Lines
    async fn create_budget_line(
        &self, org_id: Uuid, award_id: Uuid, line_number: i32,
        budget_category: &str, description: Option<&str>, account_code: Option<&str>,
        budget_amount: &str, period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>, fiscal_year: Option<i32>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBudgetLine>;
    async fn list_budget_lines(&self, award_id: Uuid) -> AtlasResult<Vec<GrantBudgetLine>>;
    async fn get_budget_line(&self, id: Uuid) -> AtlasResult<Option<GrantBudgetLine>>;
    async fn update_budget_line_amounts(&self, id: Uuid, committed: &str, expended: &str, billed: &str, available: &str) -> AtlasResult<()>;

    // Expenditures
    async fn create_expenditure(
        &self, org_id: Uuid, award_id: Uuid, expenditure_number: &str,
        expenditure_type: &str, expenditure_date: chrono::NaiveDate,
        description: Option<&str>, budget_line_id: Option<Uuid>,
        budget_category: Option<&str>, amount: &str, indirect_cost_amount: &str,
        total_amount: &str, cost_sharing_amount: &str,
        employee_id: Option<Uuid>, employee_name: Option<&str>,
        vendor_id: Option<Uuid>, vendor_name: Option<&str>,
        source_entity_type: Option<&str>, source_entity_id: Option<Uuid>,
        source_entity_number: Option<&str>,
        gl_debit_account: Option<&str>, gl_credit_account: Option<&str>,
        status: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantExpenditure>;
    async fn get_expenditure(&self, id: Uuid) -> AtlasResult<Option<GrantExpenditure>>;
    async fn list_expenditures(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantExpenditure>>;
    async fn update_expenditure_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<GrantExpenditure>;

    // Billings
    async fn create_billing(
        &self, org_id: Uuid, award_id: Uuid, invoice_number: &str,
        invoice_date: chrono::NaiveDate, period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        direct_costs_billed: &str, indirect_costs_billed: &str,
        cost_sharing_billed: &str, total_amount: &str,
        expenditure_ids: serde_json::Value, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBilling>;
    async fn get_billing(&self, id: Uuid) -> AtlasResult<Option<GrantBilling>>;
    async fn list_billings(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantBilling>>;
    async fn update_billing_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, payment_reference: Option<&str>) -> AtlasResult<GrantBilling>;

    // Compliance Reports
    async fn create_compliance_report(
        &self, org_id: Uuid, award_id: Uuid, report_type: &str,
        report_title: Option<&str>, reporting_period_start: chrono::NaiveDate,
        reporting_period_end: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        total_expenditures: &str, total_billed: &str, total_received: &str,
        cash_draws: &str, obligations: &str, content: serde_json::Value,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantComplianceReport>;
    async fn get_compliance_report(&self, id: Uuid) -> AtlasResult<Option<GrantComplianceReport>>;
    async fn list_compliance_reports(&self, award_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<GrantComplianceReport>>;
    async fn update_compliance_report_status(&self, id: Uuid, status: &str, reviewed_by: Option<Uuid>, approved_by: Option<Uuid>) -> AtlasResult<GrantComplianceReport>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<GrantDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresGrantManagementRepository {
    pool: PgPool,
}

impl PostgresGrantManagementRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn get_numeric(&self, row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }

    fn row_to_sponsor(&self, row: &sqlx::postgres::PgRow) -> GrantSponsor {
        GrantSponsor {
            id: row.get("id"), organization_id: row.get("organization_id"),
            sponsor_code: row.get("sponsor_code"), name: row.get("name"),
            sponsor_type: row.get("sponsor_type"), country_code: row.get("country_code"),
            taxpayer_id: row.get("taxpayer_id"), contact_name: row.get("contact_name"),
            contact_email: row.get("contact_email"), contact_phone: row.get("contact_phone"),
            address_line1: row.get("address_line1"), address_line2: row.get("address_line2"),
            city: row.get("city"), state_province: row.get("state_province"),
            postal_code: row.get("postal_code"), payment_terms: row.get("payment_terms"),
            billing_frequency: row.get("billing_frequency"),
            currency_code: row.get("currency_code"),
            credit_limit: row.try_get("credit_limit").unwrap_or(None),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_award(&self, row: &sqlx::postgres::PgRow) -> GrantAward {
        GrantAward {
            id: row.get("id"), organization_id: row.get("organization_id"),
            award_number: row.get("award_number"), award_title: row.get("award_title"),
            sponsor_id: row.get("sponsor_id"), sponsor_name: row.get("sponsor_name"),
            sponsor_award_number: row.get("sponsor_award_number"),
            status: row.get("status"), award_type: row.get("award_type"),
            award_purpose: row.get("award_purpose"),
            start_date: row.get("start_date"), end_date: row.get("end_date"),
            budget_start_date: row.get("budget_start_date"), budget_end_date: row.get("budget_end_date"),
            total_award_amount: self.get_numeric(row, "total_award_amount"),
            direct_costs_total: self.get_numeric(row, "direct_costs_total"),
            indirect_costs_total: self.get_numeric(row, "indirect_costs_total"),
            cost_sharing_total: self.get_numeric(row, "cost_sharing_total"),
            total_funded: self.get_numeric(row, "total_funded"),
            total_billed: self.get_numeric(row, "total_billed"),
            total_collected: self.get_numeric(row, "total_collected"),
            total_expenditures: self.get_numeric(row, "total_expenditures"),
            total_commitments: self.get_numeric(row, "total_commitments"),
            available_balance: self.get_numeric(row, "available_balance"),
            currency_code: row.get("currency_code"),
            indirect_cost_rate_id: row.get("indirect_cost_rate_id"),
            indirect_cost_rate: self.get_numeric(row, "indirect_cost_rate"),
            cost_sharing_required: row.get("cost_sharing_required"),
            cost_sharing_percent: self.get_numeric(row, "cost_sharing_percent"),
            principal_investigator_id: row.get("principal_investigator_id"),
            principal_investigator_name: row.get("principal_investigator_name"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            project_id: row.get("project_id"), cost_center: row.get("cost_center"),
            gl_revenue_account: row.get("gl_revenue_account"),
            gl_receivable_account: row.get("gl_receivable_account"),
            gl_deferred_account: row.get("gl_deferred_account"),
            billing_frequency: row.get("billing_frequency"),
            billing_basis: row.get("billing_basis"),
            reporting_requirements: row.get("reporting_requirements"),
            compliance_notes: row.get("compliance_notes"),
            closeout_date: row.get("closeout_date"), closeout_notes: row.get("closeout_notes"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl GrantManagementRepository for PostgresGrantManagementRepository {
    async fn create_sponsor(&self, org_id: Uuid, sponsor_code: &str, name: &str,
        sponsor_type: &str, country_code: Option<&str>, taxpayer_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state_province: Option<&str>, postal_code: Option<&str>,
        payment_terms: Option<&str>, billing_frequency: &str, currency_code: &str,
        credit_limit: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantSponsor> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_sponsors
                (organization_id, sponsor_code, name, sponsor_type, country_code,
                 taxpayer_id, contact_name, contact_email, contact_phone,
                 address_line1, address_line2, city, state_province, postal_code,
                 payment_terms, billing_frequency, currency_code, credit_limit, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18::numeric,$19)
            RETURNING *"#,
        ).bind(org_id).bind(sponsor_code).bind(name).bind(sponsor_type)
        .bind(country_code).bind(taxpayer_id).bind(contact_name).bind(contact_email)
        .bind(contact_phone).bind(address_line1).bind(address_line2)
        .bind(city).bind(state_province).bind(postal_code).bind(payment_terms)
        .bind(billing_frequency).bind(currency_code).bind(credit_limit).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_sponsor(&row))
    }

    async fn get_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GrantSponsor>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_sponsors WHERE organization_id=$1 AND sponsor_code=$2")
            .bind(org_id).bind(code).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_sponsor(&r)))
    }

    async fn list_sponsors(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantSponsor>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.grant_sponsors WHERE organization_id=$1 AND is_active=true ORDER BY sponsor_code")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.grant_sponsors WHERE organization_id=$1 ORDER BY sponsor_code")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_sponsor(&r)).collect())
    }

    async fn delete_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.grant_sponsors WHERE organization_id=$1 AND sponsor_code=$2")
            .bind(org_id).bind(code).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_indirect_cost_rate(&self, org_id: Uuid, rate_name: &str, rate_type: &str,
        rate_percentage: &str, base_type: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        negotiated_by: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantIndirectCostRate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_indirect_cost_rates
                (organization_id, rate_name, rate_type, rate_percentage, base_type,
                 effective_from, effective_to, negotiated_by, created_by)
            VALUES ($1,$2,$3,$4::numeric,$5,$6,$7,$8,$9) RETURNING *"#,
        ).bind(org_id).bind(rate_name).bind(rate_type).bind(rate_percentage)
        .bind(base_type).bind(effective_from).bind(effective_to).bind(negotiated_by).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(GrantIndirectCostRate {
            id: row.get("id"), organization_id: row.get("organization_id"),
            rate_name: row.get("rate_name"), rate_type: row.get("rate_type"),
            rate_percentage: self.get_numeric(&row, "rate_percentage"),
            base_type: row.get("base_type"), effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"), negotiated_by: row.get("negotiated_by"),
            approved_by: row.get("approved_by"), is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"), created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_indirect_cost_rates(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantIndirectCostRate>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.grant_indirect_cost_rates WHERE organization_id=$1 AND is_active=true ORDER BY effective_from DESC")
                .bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM _atlas.grant_indirect_cost_rates WHERE organization_id=$1 ORDER BY effective_from DESC")
                .bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| GrantIndirectCostRate {
            id: r.get("id"), organization_id: r.get("organization_id"),
            rate_name: r.get("rate_name"), rate_type: r.get("rate_type"),
            rate_percentage: self.get_numeric(r, "rate_percentage"),
            base_type: r.get("base_type"), effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"), negotiated_by: r.get("negotiated_by"),
            approved_by: r.get("approved_by"), is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_award(&self, org_id: Uuid, award_number: &str, award_title: &str,
        sponsor_id: Uuid, sponsor_name: Option<&str>, sponsor_award_number: Option<&str>,
        award_type: &str, award_purpose: Option<&str>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        total_award_amount: &str, direct_costs_total: &str, indirect_costs_total: &str,
        cost_sharing_total: &str, currency_code: &str,
        indirect_cost_rate_id: Option<Uuid>, indirect_cost_rate: &str,
        cost_sharing_required: bool, cost_sharing_percent: &str,
        principal_investigator_id: Option<Uuid>, principal_investigator_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        gl_revenue_account: Option<&str>, gl_receivable_account: Option<&str>,
        gl_deferred_account: Option<&str>,
        billing_frequency: &str, billing_basis: &str,
        reporting_requirements: Option<&str>, compliance_notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GrantAward> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_awards
                (organization_id, award_number, award_title, sponsor_id, sponsor_name,
                 sponsor_award_number, award_type, award_purpose, start_date, end_date,
                 total_award_amount, direct_costs_total, indirect_costs_total, cost_sharing_total,
                 currency_code, indirect_cost_rate_id, indirect_cost_rate,
                 cost_sharing_required, cost_sharing_percent,
                 principal_investigator_id, principal_investigator_name,
                 department_id, department_name, project_id, cost_center,
                 gl_revenue_account, gl_receivable_account, gl_deferred_account,
                 billing_frequency, billing_basis, reporting_requirements, compliance_notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                    $11::numeric,$12::numeric,$13::numeric,$14::numeric,
                    $15,$16,$17::numeric,$18,$19::numeric,
                    $20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32)
            RETURNING *"#,
        ).bind(org_id).bind(award_number).bind(award_title).bind(sponsor_id)
        .bind(sponsor_name).bind(sponsor_award_number).bind(award_type).bind(award_purpose)
        .bind(start_date).bind(end_date).bind(total_award_amount).bind(direct_costs_total)
        .bind(indirect_costs_total).bind(cost_sharing_total).bind(currency_code)
        .bind(indirect_cost_rate_id).bind(indirect_cost_rate).bind(cost_sharing_required)
        .bind(cost_sharing_percent).bind(principal_investigator_id).bind(principal_investigator_name)
        .bind(department_id).bind(department_name).bind(project_id).bind(cost_center)
        .bind(gl_revenue_account).bind(gl_receivable_account).bind(gl_deferred_account)
        .bind(billing_frequency).bind(billing_basis).bind(reporting_requirements)
        .bind(compliance_notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_award(&row))
    }

    async fn get_award(&self, id: Uuid) -> AtlasResult<Option<GrantAward>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_awards WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_award(&r)))
    }

    async fn get_award_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GrantAward>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_awards WHERE organization_id=$1 AND award_number=$2")
            .bind(org_id).bind(number).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_award(&r)))
    }

    async fn list_awards(&self, org_id: Uuid, status: Option<&str>, sponsor_id: Option<Uuid>) -> AtlasResult<Vec<GrantAward>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.grant_awards
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2) AND ($3::uuid IS NULL OR sponsor_id=$3)
            ORDER BY award_number"#,
        ).bind(org_id).bind(status).bind(sponsor_id)
        .fetch_all(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_award(&r)).collect())
    }

    async fn update_award_status(&self, id: Uuid, status: &str, closeout_date: Option<chrono::NaiveDate>, closeout_notes: Option<&str>) -> AtlasResult<GrantAward> {
        let row = sqlx::query(
            r#"UPDATE _atlas.grant_awards SET status=$2,
                closeout_date=COALESCE($3, closeout_date),
                closeout_notes=COALESCE($4, closeout_notes),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(closeout_date).bind(closeout_notes)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_award(&row))
    }

    async fn update_award_totals(&self, id: Uuid, total_expenditures: &str, total_commitments: &str, total_billed: &str, total_collected: &str, available_balance: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.grant_awards SET total_expenditures=$2::numeric, total_commitments=$3::numeric,
                total_billed=$4::numeric, total_collected=$5::numeric, available_balance=$6::numeric, updated_at=now() WHERE id=$1"#,
        ).bind(id).bind(total_expenditures).bind(total_commitments).bind(total_billed).bind(total_collected).bind(available_balance)
        .execute(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_budget_line(&self, org_id: Uuid, award_id: Uuid, line_number: i32,
        budget_category: &str, description: Option<&str>, account_code: Option<&str>,
        budget_amount: &str, period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>, fiscal_year: Option<i32>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBudgetLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_budget_lines
                (organization_id, award_id, line_number, budget_category, description,
                 account_code, budget_amount, period_start, period_end, fiscal_year, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8,$9,$10,$11,$12) RETURNING *"#,
        ).bind(org_id).bind(award_id).bind(line_number).bind(budget_category)
        .bind(description).bind(account_code).bind(budget_amount)
        .bind(period_start).bind(period_end).bind(fiscal_year).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_budget_line(&row))
    }

    async fn list_budget_lines(&self, award_id: Uuid) -> AtlasResult<Vec<GrantBudgetLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.grant_budget_lines WHERE award_id=$1 ORDER BY line_number"
        ).bind(award_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_budget_line(&r)).collect())
    }

    async fn get_budget_line(&self, id: Uuid) -> AtlasResult<Option<GrantBudgetLine>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_budget_lines WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_budget_line(&r)))
    }

    async fn update_budget_line_amounts(&self, id: Uuid, committed: &str, expended: &str, billed: &str, available: &str) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.grant_budget_lines SET committed_amount=$2::numeric, expended_amount=$3::numeric,
                billed_amount=$4::numeric, available_balance=$5::numeric, updated_at=now() WHERE id=$1"#,
        ).bind(id).bind(committed).bind(expended).bind(billed).bind(available)
        .execute(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_expenditure(&self, org_id: Uuid, award_id: Uuid, expenditure_number: &str,
        expenditure_type: &str, expenditure_date: chrono::NaiveDate,
        description: Option<&str>, budget_line_id: Option<Uuid>,
        budget_category: Option<&str>, amount: &str, indirect_cost_amount: &str,
        total_amount: &str, cost_sharing_amount: &str,
        employee_id: Option<Uuid>, employee_name: Option<&str>,
        vendor_id: Option<Uuid>, vendor_name: Option<&str>,
        source_entity_type: Option<&str>, source_entity_id: Option<Uuid>,
        source_entity_number: Option<&str>,
        gl_debit_account: Option<&str>, gl_credit_account: Option<&str>,
        status: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantExpenditure> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_expenditures
                (organization_id, award_id, expenditure_number, expenditure_type, expenditure_date,
                 description, budget_line_id, budget_category, amount, indirect_cost_amount,
                 total_amount, cost_sharing_amount, employee_id, employee_name,
                 vendor_id, vendor_name, source_entity_type, source_entity_id, source_entity_number,
                 gl_debit_account, gl_credit_account, status, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,
                    $11::numeric,$12::numeric,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24)
            RETURNING *"#,
        ).bind(org_id).bind(award_id).bind(expenditure_number).bind(expenditure_type)
        .bind(expenditure_date).bind(description).bind(budget_line_id).bind(budget_category)
        .bind(amount).bind(indirect_cost_amount).bind(total_amount).bind(cost_sharing_amount)
        .bind(employee_id).bind(employee_name).bind(vendor_id).bind(vendor_name)
        .bind(source_entity_type).bind(source_entity_id).bind(source_entity_number)
        .bind(gl_debit_account).bind(gl_credit_account).bind(status).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_expenditure(&row))
    }

    async fn get_expenditure(&self, id: Uuid) -> AtlasResult<Option<GrantExpenditure>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_expenditures WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_expenditure(&r)))
    }

    async fn list_expenditures(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantExpenditure>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.grant_expenditures
            WHERE award_id=$1 AND ($2::text IS NULL OR status=$2) ORDER BY expenditure_date"#,
        ).bind(award_id).bind(status).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_expenditure(&r)).collect())
    }

    async fn update_expenditure_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<GrantExpenditure> {
        let row = sqlx::query(
            r#"UPDATE _atlas.grant_expenditures SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_expenditure(&row))
    }

    async fn create_billing(&self, org_id: Uuid, award_id: Uuid, invoice_number: &str,
        invoice_date: chrono::NaiveDate, period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        direct_costs_billed: &str, indirect_costs_billed: &str,
        cost_sharing_billed: &str, total_amount: &str,
        expenditure_ids: serde_json::Value, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBilling> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_billings
                (organization_id, award_id, invoice_number, invoice_date, period_start, period_end,
                 due_date, direct_costs_billed, indirect_costs_billed, cost_sharing_billed,
                 total_amount, expenditure_ids, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10::numeric,$11::numeric,$12,$13,$14)
            RETURNING *"#,
        ).bind(org_id).bind(award_id).bind(invoice_number).bind(invoice_date)
        .bind(period_start).bind(period_end).bind(due_date)
        .bind(direct_costs_billed).bind(indirect_costs_billed).bind(cost_sharing_billed)
        .bind(total_amount).bind(expenditure_ids).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_billing(&row))
    }

    async fn get_billing(&self, id: Uuid) -> AtlasResult<Option<GrantBilling>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_billings WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_billing(&r)))
    }

    async fn list_billings(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantBilling>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.grant_billings
            WHERE award_id=$1 AND ($2::text IS NULL OR status=$2) ORDER BY invoice_date"#,
        ).bind(award_id).bind(status).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_billing(&r)).collect())
    }

    async fn update_billing_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, payment_reference: Option<&str>) -> AtlasResult<GrantBilling> {
        let row = sqlx::query(
            r#"UPDATE _atlas.grant_billings SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                paid_at=CASE WHEN $2='paid' AND paid_at IS NULL THEN now() ELSE paid_at END,
                payment_reference=COALESCE($4, payment_reference),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(approved_by).bind(payment_reference)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_billing(&row))
    }

    async fn create_compliance_report(&self, org_id: Uuid, award_id: Uuid, report_type: &str,
        report_title: Option<&str>, reporting_period_start: chrono::NaiveDate,
        reporting_period_end: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        total_expenditures: &str, total_billed: &str, total_received: &str,
        cash_draws: &str, obligations: &str, content: serde_json::Value,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantComplianceReport> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.grant_compliance_reports
                (organization_id, award_id, report_type, report_title,
                 reporting_period_start, reporting_period_end, due_date,
                 total_expenditures, total_billed, total_received, cash_draws, obligations,
                 content, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10::numeric,$11::numeric,$12::numeric,$13,$14,$15)
            RETURNING *"#,
        ).bind(org_id).bind(award_id).bind(report_type).bind(report_title)
        .bind(reporting_period_start).bind(reporting_period_end).bind(due_date)
        .bind(total_expenditures).bind(total_billed).bind(total_received).bind(cash_draws).bind(obligations)
        .bind(content).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_compliance_report(&row))
    }

    async fn get_compliance_report(&self, id: Uuid) -> AtlasResult<Option<GrantComplianceReport>> {
        let row = sqlx::query("SELECT * FROM _atlas.grant_compliance_reports WHERE id=$1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_compliance_report(&r)))
    }

    async fn list_compliance_reports(&self, award_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<GrantComplianceReport>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.grant_compliance_reports
            WHERE award_id=$1 AND ($2::text IS NULL OR report_type=$2) ORDER BY reporting_period_start"#,
        ).bind(award_id).bind(report_type).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_compliance_report(&r)).collect())
    }

    async fn update_compliance_report_status(&self, id: Uuid, status: &str, reviewed_by: Option<Uuid>, approved_by: Option<Uuid>) -> AtlasResult<GrantComplianceReport> {
        let row = sqlx::query(
            r#"UPDATE _atlas.grant_compliance_reports SET status=$2,
                reviewed_by=COALESCE($3, reviewed_by),
                approved_by=COALESCE($4, approved_by),
                submitted_at=CASE WHEN $2='submitted' AND submitted_at IS NULL THEN now() ELSE submitted_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        ).bind(id).bind(status).bind(reviewed_by).bind(approved_by)
        .fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_compliance_report(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<GrantDashboardSummary> {
        let rows = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COUNT(DISTINCT sponsor_id) FILTER (WHERE status = 'active') as sponsor_count,
                COALESCE(SUM(total_award_amount) FILTER (WHERE status = 'active'), 0) as total_value,
                COALESCE(SUM(total_funded) FILTER (WHERE status = 'active'), 0) as total_funded,
                COALESCE(SUM(total_expenditures) FILTER (WHERE status = 'active'), 0) as total_exp,
                COALESCE(SUM(available_balance) FILTER (WHERE status = 'active'), 0) as total_avail,
                COUNT(*) FILTER (WHERE status = 'active' AND end_date <= CURRENT_DATE + INTERVAL '30 days') as expiring_30,
                0 as pending_bills,
                0 as overdue_reports
            FROM _atlas.grant_awards WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = rows.try_get("active_count").unwrap_or(0);
        let sponsors: i64 = rows.try_get("sponsor_count").unwrap_or(0);
        let expiring: i64 = rows.try_get("expiring_30").unwrap_or(0);
        let pending: i64 = rows.try_get("pending_bills").unwrap_or(0);
        let overdue: i64 = rows.try_get("overdue_reports").unwrap_or(0);

        let total_value: serde_json::Value = rows.try_get("total_value").unwrap_or(serde_json::json!(0));
        let total_funded: serde_json::Value = rows.try_get("total_funded").unwrap_or(serde_json::json!(0));
        let total_exp: serde_json::Value = rows.try_get("total_exp").unwrap_or(serde_json::json!(0));
        let total_avail: serde_json::Value = rows.try_get("total_avail").unwrap_or(serde_json::json!(0));

        let exp_f64: f64 = total_exp.to_string().parse().unwrap_or(0.0);
        let val_f64: f64 = total_value.to_string().parse().unwrap_or(0.0);
        let utilization = if val_f64 > 0.0 { (exp_f64 / val_f64 * 100.0) } else { 0.0 };

        Ok(GrantDashboardSummary {
            total_active_awards: active as i32,
            total_sponsors: sponsors as i32,
            total_award_value: total_value.to_string(),
            total_funded: total_funded.to_string(),
            total_expenditures: total_exp.to_string(),
            total_available_balance: total_avail.to_string(),
            total_pending_billings: pending as i32,
            total_overdue_reports: overdue as i32,
            awards_expiring_30_days: expiring as i32,
            budget_utilization_percent: format!("{:.1}", utilization),
            awards_by_status: serde_json::json!({}),
            expenditures_by_category: serde_json::json!({}),
            top_sponsors: serde_json::json!({}),
        })
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_budget_line(row: &sqlx::postgres::PgRow) -> GrantBudgetLine {
    GrantBudgetLine {
        id: row.get("id"), organization_id: row.get("organization_id"),
        award_id: row.get("award_id"), line_number: row.get("line_number"),
        budget_category: row.get("budget_category"), description: row.get("description"),
        account_code: row.get("account_code"),
        budget_amount: get_num(row, "budget_amount"),
        committed_amount: get_num(row, "committed_amount"),
        expended_amount: get_num(row, "expended_amount"),
        billed_amount: get_num(row, "billed_amount"),
        available_balance: get_num(row, "available_balance"),
        period_start: row.get("period_start"), period_end: row.get("period_end"),
        fiscal_year: row.get("fiscal_year"), notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_expenditure(row: &sqlx::postgres::PgRow) -> GrantExpenditure {
    GrantExpenditure {
        id: row.get("id"), organization_id: row.get("organization_id"),
        award_id: row.get("award_id"), expenditure_number: row.get("expenditure_number"),
        expenditure_type: row.get("expenditure_type"), expenditure_date: row.get("expenditure_date"),
        description: row.get("description"), budget_line_id: row.get("budget_line_id"),
        budget_category: row.get("budget_category"),
        amount: get_num(row, "amount"), indirect_cost_amount: get_num(row, "indirect_cost_amount"),
        total_amount: get_num(row, "total_amount"), cost_sharing_amount: get_num(row, "cost_sharing_amount"),
        employee_id: row.get("employee_id"), employee_name: row.get("employee_name"),
        vendor_id: row.get("vendor_id"), vendor_name: row.get("vendor_name"),
        source_entity_type: row.get("source_entity_type"), source_entity_id: row.get("source_entity_id"),
        source_entity_number: row.get("source_entity_number"),
        gl_debit_account: row.get("gl_debit_account"), gl_credit_account: row.get("gl_credit_account"),
        status: row.get("status"), approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"), billed_at: row.get("billed_at"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_billing(row: &sqlx::postgres::PgRow) -> GrantBilling {
    GrantBilling {
        id: row.get("id"), organization_id: row.get("organization_id"),
        award_id: row.get("award_id"), invoice_number: row.get("invoice_number"),
        invoice_date: row.get("invoice_date"), period_start: row.get("period_start"),
        period_end: row.get("period_end"), due_date: row.get("due_date"),
        direct_costs_billed: get_num(row, "direct_costs_billed"),
        indirect_costs_billed: get_num(row, "indirect_costs_billed"),
        cost_sharing_billed: get_num(row, "cost_sharing_billed"),
        total_amount: get_num(row, "total_amount"),
        amount_received: get_num(row, "amount_received"),
        status: row.get("status"), expenditure_ids: row.try_get("expenditure_ids").unwrap_or(serde_json::json!([])),
        notes: row.get("notes"), submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"), approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"), paid_at: row.get("paid_at"),
        payment_reference: row.get("payment_reference"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_compliance_report(row: &sqlx::postgres::PgRow) -> GrantComplianceReport {
    GrantComplianceReport {
        id: row.get("id"), organization_id: row.get("organization_id"),
        award_id: row.get("award_id"), report_type: row.get("report_type"),
        report_title: row.get("report_title"),
        reporting_period_start: row.get("reporting_period_start"),
        reporting_period_end: row.get("reporting_period_end"),
        due_date: row.get("due_date"), status: row.get("status"),
        total_expenditures: get_num(row, "total_expenditures"),
        total_billed: get_num(row, "total_billed"),
        total_received: get_num(row, "total_received"),
        cash_draws: get_num(row, "cash_draws"),
        obligations: get_num(row, "obligations"),
        content: row.try_get("content").unwrap_or(serde_json::json!({})),
        notes: row.get("notes"), prepared_by: row.get("prepared_by"),
        reviewed_by: row.get("reviewed_by"), approved_by: row.get("approved_by"),
        submitted_at: row.get("submitted_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"), created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
