//! Lease Accounting Repository
//!
//! PostgreSQL storage for lease contracts, payment schedules,
//! modifications, and terminations.

use atlas_shared::{
    LeaseContract, LeasePayment, LeaseModification, LeaseTermination,
    LeaseDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for lease accounting data storage
#[async_trait]
pub trait LeaseAccountingRepository: Send + Sync {
    // Lease Contracts
    async fn create_lease(
        &self,
        org_id: Uuid,
        lease_number: &str,
        title: &str,
        description: Option<&str>,
        classification: &str,
        lessor_id: Option<Uuid>,
        lessor_name: Option<&str>,
        asset_description: Option<&str>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        commencement_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        lease_term_months: i32,
        purchase_option_exists: bool,
        purchase_option_likely: bool,
        renewal_option_exists: bool,
        renewal_option_months: Option<i32>,
        renewal_option_likely: bool,
        discount_rate: &str,
        currency_code: &str,
        payment_frequency: &str,
        annual_payment_amount: &str,
        escalation_rate: Option<&str>,
        escalation_frequency_months: Option<i32>,
        total_lease_payments: &str,
        initial_lease_liability: &str,
        initial_rou_asset_value: &str,
        residual_guarantee_amount: Option<&str>,
        current_lease_liability: &str,
        current_rou_asset_value: &str,
        accumulated_rou_depreciation: &str,
        total_payments_made: &str,
        periods_elapsed: i32,
        rou_asset_account_code: Option<&str>,
        rou_depreciation_account_code: Option<&str>,
        lease_liability_account_code: Option<&str>,
        lease_expense_account_code: Option<&str>,
        interest_expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseContract>;

    async fn get_lease(&self, id: Uuid) -> AtlasResult<Option<LeaseContract>>;
    async fn get_lease_by_number(&self, org_id: Uuid, lease_number: &str) -> AtlasResult<Option<LeaseContract>>;
    async fn list_leases(&self, org_id: Uuid, status: Option<&str>, classification: Option<&str>) -> AtlasResult<Vec<LeaseContract>>;
    async fn update_lease_status(
        &self,
        id: Uuid,
        status: &str,
        impairment_amount: Option<&str>,
        impairment_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LeaseContract>;
    async fn update_lease_balances(
        &self,
        id: Uuid,
        current_liability: &str,
        current_rou: &str,
        accumulated_depreciation: &str,
        total_payments_made: &str,
        periods_elapsed: i32,
    ) -> AtlasResult<()>;

    // Lease Payments
    async fn create_payment(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        period_number: i32,
        payment_date: chrono::NaiveDate,
        payment_amount: &str,
        interest_amount: &str,
        principal_amount: &str,
        remaining_liability: &str,
        rou_asset_value: &str,
        rou_depreciation: &str,
        accumulated_depreciation: &str,
        lease_expense: &str,
        is_paid: bool,
        payment_reference: Option<&str>,
        journal_entry_id: Option<Uuid>,
        status: &str,
    ) -> AtlasResult<LeasePayment>;

    async fn get_payment_by_period(&self, lease_id: Uuid, period_number: i32) -> AtlasResult<Option<LeasePayment>>;
    async fn list_payments(&self, lease_id: Uuid) -> AtlasResult<Vec<LeasePayment>>;
    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        is_paid: bool,
        payment_reference: Option<&str>,
        journal_entry_id: Option<Uuid>,
    ) -> AtlasResult<LeasePayment>;

    // Lease Modifications
    async fn create_modification(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        modification_number: i32,
        modification_type: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        previous_term_months: Option<i32>,
        new_term_months: Option<i32>,
        previous_end_date: Option<chrono::NaiveDate>,
        new_end_date: Option<chrono::NaiveDate>,
        previous_discount_rate: Option<&str>,
        new_discount_rate: Option<&str>,
        liability_adjustment: &str,
        rou_asset_adjustment: &str,
        status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseModification>;

    async fn get_next_modification_number(&self, lease_id: Uuid) -> AtlasResult<i32>;
    async fn list_modifications(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseModification>>;

    // Lease Terminations
    async fn create_termination(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        termination_type: &str,
        termination_date: chrono::NaiveDate,
        reason: Option<&str>,
        remaining_liability: &str,
        remaining_rou_asset: &str,
        termination_penalty: &str,
        gain_loss_amount: &str,
        gain_loss_type: Option<&str>,
        journal_entry_id: Option<Uuid>,
        status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseTermination>;

    async fn list_terminations(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseTermination>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<LeaseDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresLeaseAccountingRepository {
    pool: PgPool,
}

impl PostgresLeaseAccountingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_lease(&self, row: &sqlx::postgres::PgRow) -> LeaseContract {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        LeaseContract {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            lease_number: row.get("lease_number"),
            title: row.get("title"),
            description: row.get("description"),
            classification: row.get("classification"),
            lessor_id: row.get("lessor_id"),
            lessor_name: row.get("lessor_name"),
            asset_description: row.get("asset_description"),
            location: row.get("location"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            commencement_date: row.get("commencement_date"),
            end_date: row.get("end_date"),
            lease_term_months: row.get("lease_term_months"),
            purchase_option_exists: row.get("purchase_option_exists"),
            purchase_option_likely: row.get("purchase_option_likely"),
            renewal_option_exists: row.get("renewal_option_exists"),
            renewal_option_months: row.get("renewal_option_months"),
            renewal_option_likely: row.get("renewal_option_likely"),
            discount_rate: get_num(row, "discount_rate"),
            currency_code: row.get("currency_code"),
            payment_frequency: row.get("payment_frequency"),
            escalation_rate: row.try_get("escalation_rate").unwrap_or(None),
            escalation_frequency_months: row.get("escalation_frequency_months"),
            total_lease_payments: get_num(row, "total_lease_payments"),
            initial_lease_liability: get_num(row, "initial_lease_liability"),
            initial_rou_asset_value: get_num(row, "initial_rou_asset_value"),
            residual_guarantee_amount: row.try_get("residual_guarantee_amount").unwrap_or(None),
            current_lease_liability: get_num(row, "current_lease_liability"),
            current_rou_asset_value: get_num(row, "current_rou_asset_value"),
            accumulated_rou_depreciation: get_num(row, "accumulated_rou_depreciation"),
            total_payments_made: get_num(row, "total_payments_made"),
            periods_elapsed: row.get("periods_elapsed"),
            rou_asset_account_code: row.get("rou_asset_account_code"),
            rou_depreciation_account_code: row.get("rou_depreciation_account_code"),
            lease_liability_account_code: row.get("lease_liability_account_code"),
            lease_expense_account_code: row.get("lease_expense_account_code"),
            interest_expense_account_code: row.get("interest_expense_account_code"),
            status: row.get("status"),
            impairment_amount: row.try_get("impairment_amount").unwrap_or(None),
            impairment_date: row.get("impairment_date"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_payment(&self, row: &sqlx::postgres::PgRow) -> LeasePayment {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        LeasePayment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            lease_id: row.get("lease_id"),
            period_number: row.get("period_number"),
            payment_date: row.get("payment_date"),
            payment_amount: get_num(row, "payment_amount"),
            interest_amount: get_num(row, "interest_amount"),
            principal_amount: get_num(row, "principal_amount"),
            remaining_liability: get_num(row, "remaining_liability"),
            rou_asset_value: get_num(row, "rou_asset_value"),
            rou_depreciation: get_num(row, "rou_depreciation"),
            accumulated_depreciation: get_num(row, "accumulated_depreciation"),
            lease_expense: get_num(row, "lease_expense"),
            is_paid: row.get("is_paid"),
            payment_reference: row.get("payment_reference"),
            journal_entry_id: row.get("journal_entry_id"),
            status: row.get("status"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_modification(&self, row: &sqlx::postgres::PgRow) -> LeaseModification {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        LeaseModification {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            lease_id: row.get("lease_id"),
            modification_number: row.get("modification_number"),
            modification_type: row.get("modification_type"),
            description: row.get("description"),
            effective_date: row.get("effective_date"),
            previous_term_months: row.get("previous_term_months"),
            new_term_months: row.get("new_term_months"),
            previous_end_date: row.get("previous_end_date"),
            new_end_date: row.get("new_end_date"),
            previous_discount_rate: row.try_get("previous_discount_rate").unwrap_or(None),
            new_discount_rate: row.try_get("new_discount_rate").unwrap_or(None),
            liability_adjustment: get_num(row, "liability_adjustment"),
            rou_asset_adjustment: get_num(row, "rou_asset_adjustment"),
            status: row.get("status"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_termination(&self, row: &sqlx::postgres::PgRow) -> LeaseTermination {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        LeaseTermination {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            lease_id: row.get("lease_id"),
            termination_type: row.get("termination_type"),
            termination_date: row.get("termination_date"),
            reason: row.get("reason"),
            remaining_liability: get_num(row, "remaining_liability"),
            remaining_rou_asset: get_num(row, "remaining_rou_asset"),
            termination_penalty: get_num(row, "termination_penalty"),
            gain_loss_amount: get_num(row, "gain_loss_amount"),
            gain_loss_type: row.get("gain_loss_type"),
            journal_entry_id: row.get("journal_entry_id"),
            status: row.get("status"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl LeaseAccountingRepository for PostgresLeaseAccountingRepository {
    async fn create_lease(
        &self,
        org_id: Uuid,
        lease_number: &str,
        title: &str,
        description: Option<&str>,
        classification: &str,
        lessor_id: Option<Uuid>,
        lessor_name: Option<&str>,
        asset_description: Option<&str>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        commencement_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        lease_term_months: i32,
        purchase_option_exists: bool,
        purchase_option_likely: bool,
        renewal_option_exists: bool,
        renewal_option_months: Option<i32>,
        renewal_option_likely: bool,
        discount_rate: &str,
        currency_code: &str,
        payment_frequency: &str,
        annual_payment_amount: &str,
        escalation_rate: Option<&str>,
        escalation_frequency_months: Option<i32>,
        total_lease_payments: &str,
        initial_lease_liability: &str,
        initial_rou_asset_value: &str,
        residual_guarantee_amount: Option<&str>,
        current_lease_liability: &str,
        current_rou_asset_value: &str,
        accumulated_rou_depreciation: &str,
        total_payments_made: &str,
        periods_elapsed: i32,
        rou_asset_account_code: Option<&str>,
        rou_depreciation_account_code: Option<&str>,
        lease_liability_account_code: Option<&str>,
        lease_expense_account_code: Option<&str>,
        interest_expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseContract> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.lease_contracts
                (organization_id, lease_number, title, description, classification,
                 lessor_id, lessor_name, asset_description, location,
                 department_id, department_name,
                 commencement_date, end_date, lease_term_months,
                 purchase_option_exists, purchase_option_likely,
                 renewal_option_exists, renewal_option_months, renewal_option_likely,
                 discount_rate, currency_code, payment_frequency,
                 annual_payment_amount, escalation_rate, escalation_frequency_months,
                 total_lease_payments, initial_lease_liability, initial_rou_asset_value,
                 residual_guarantee_amount,
                 current_lease_liability, current_rou_asset_value,
                 accumulated_rou_depreciation, total_payments_made, periods_elapsed,
                 rou_asset_account_code, rou_depreciation_account_code,
                 lease_liability_account_code, lease_expense_account_code,
                 interest_expense_account_code,
                 status, created_by)
            VALUES ($1, $2, $3, $4, $5,
                    $6, $7, $8, $9, $10, $11,
                    $12, $13, $14,
                    $15, $16, $17, $18, $19,
                    $20::numeric, $21, $22,
                    $23::numeric, $24::numeric, $25,
                    $26::numeric, $27::numeric, $28::numeric,
                    $29::numeric,
                    $30::numeric, $31::numeric,
                    $32::numeric, $33::numeric, $34,
                    $35, $36, $37, $38, $39,
                    'draft', $40)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(lease_number).bind(title).bind(description).bind(classification)
        .bind(lessor_id).bind(lessor_name).bind(asset_description).bind(location)
        .bind(department_id).bind(department_name)
        .bind(commencement_date).bind(end_date).bind(lease_term_months)
        .bind(purchase_option_exists).bind(purchase_option_likely)
        .bind(renewal_option_exists).bind(renewal_option_months).bind(renewal_option_likely)
        .bind(discount_rate).bind(currency_code).bind(payment_frequency)
        .bind(annual_payment_amount).bind(escalation_rate).bind(escalation_frequency_months)
        .bind(total_lease_payments).bind(initial_lease_liability).bind(initial_rou_asset_value)
        .bind(residual_guarantee_amount)
        .bind(current_lease_liability).bind(current_rou_asset_value)
        .bind(accumulated_rou_depreciation).bind(total_payments_made).bind(periods_elapsed)
        .bind(rou_asset_account_code).bind(rou_depreciation_account_code)
        .bind(lease_liability_account_code).bind(lease_expense_account_code)
        .bind(interest_expense_account_code)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_lease(&row))
    }

    async fn get_lease(&self, id: Uuid) -> AtlasResult<Option<LeaseContract>> {
        let row = sqlx::query("SELECT * FROM _atlas.lease_contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_lease(&r)))
    }

    async fn get_lease_by_number(&self, org_id: Uuid, lease_number: &str) -> AtlasResult<Option<LeaseContract>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.lease_contracts WHERE organization_id = $1 AND lease_number = $2"
        )
        .bind(org_id).bind(lease_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_lease(&r)))
    }

    async fn list_leases(&self, org_id: Uuid, status: Option<&str>, classification: Option<&str>) -> AtlasResult<Vec<LeaseContract>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.lease_contracts
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR classification = $3)
            ORDER BY lease_number
            "#,
        )
        .bind(org_id).bind(status).bind(classification)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_lease(&r)).collect())
    }

    async fn update_lease_status(
        &self,
        id: Uuid,
        status: &str,
        impairment_amount: Option<&str>,
        impairment_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LeaseContract> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.lease_contracts
            SET status = $2,
                impairment_amount = COALESCE($3::numeric, impairment_amount),
                impairment_date = COALESCE($4, impairment_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(impairment_amount).bind(impairment_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_lease(&row))
    }

    async fn update_lease_balances(
        &self,
        id: Uuid,
        current_liability: &str,
        current_rou: &str,
        accumulated_depreciation: &str,
        total_payments_made: &str,
        periods_elapsed: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.lease_contracts
            SET current_lease_liability = $2::numeric,
                current_rou_asset_value = $3::numeric,
                accumulated_rou_depreciation = $4::numeric,
                total_payments_made = $5::numeric,
                periods_elapsed = $6,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(current_liability).bind(current_rou)
        .bind(accumulated_depreciation).bind(total_payments_made).bind(periods_elapsed)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_payment(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        period_number: i32,
        payment_date: chrono::NaiveDate,
        payment_amount: &str,
        interest_amount: &str,
        principal_amount: &str,
        remaining_liability: &str,
        rou_asset_value: &str,
        rou_depreciation: &str,
        accumulated_depreciation: &str,
        lease_expense: &str,
        is_paid: bool,
        payment_reference: Option<&str>,
        journal_entry_id: Option<Uuid>,
        status: &str,
    ) -> AtlasResult<LeasePayment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.lease_payments
                (organization_id, lease_id, period_number, payment_date,
                 payment_amount, interest_amount, principal_amount,
                 remaining_liability, rou_asset_value,
                 rou_depreciation, accumulated_depreciation,
                 lease_expense, is_paid, payment_reference,
                 journal_entry_id, status)
            VALUES ($1, $2, $3, $4,
                    $5::numeric, $6::numeric, $7::numeric,
                    $8::numeric, $9::numeric,
                    $10::numeric, $11::numeric,
                    $12::numeric, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(lease_id).bind(period_number).bind(payment_date)
        .bind(payment_amount).bind(interest_amount).bind(principal_amount)
        .bind(remaining_liability).bind(rou_asset_value)
        .bind(rou_depreciation).bind(accumulated_depreciation)
        .bind(lease_expense).bind(is_paid).bind(payment_reference)
        .bind(journal_entry_id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_payment(&row))
    }

    async fn get_payment_by_period(&self, lease_id: Uuid, period_number: i32) -> AtlasResult<Option<LeasePayment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.lease_payments WHERE lease_id = $1 AND period_number = $2"
        )
        .bind(lease_id).bind(period_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_payment(&r)))
    }

    async fn list_payments(&self, lease_id: Uuid) -> AtlasResult<Vec<LeasePayment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.lease_payments WHERE lease_id = $1 ORDER BY period_number"
        )
        .bind(lease_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_payment(&r)).collect())
    }

    async fn update_payment_status(
        &self,
        id: Uuid,
        status: &str,
        is_paid: bool,
        payment_reference: Option<&str>,
        journal_entry_id: Option<Uuid>,
    ) -> AtlasResult<LeasePayment> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.lease_payments
            SET status = $2, is_paid = $3,
                payment_reference = COALESCE($4, payment_reference),
                journal_entry_id = COALESCE($5, journal_entry_id),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(is_paid).bind(payment_reference).bind(journal_entry_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_payment(&row))
    }

    async fn create_modification(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        modification_number: i32,
        modification_type: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        previous_term_months: Option<i32>,
        new_term_months: Option<i32>,
        previous_end_date: Option<chrono::NaiveDate>,
        new_end_date: Option<chrono::NaiveDate>,
        previous_discount_rate: Option<&str>,
        new_discount_rate: Option<&str>,
        liability_adjustment: &str,
        rou_asset_adjustment: &str,
        status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseModification> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.lease_modifications
                (organization_id, lease_id, modification_number, modification_type,
                 description, effective_date,
                 previous_term_months, new_term_months,
                 previous_end_date, new_end_date,
                 previous_discount_rate, new_discount_rate,
                 liability_adjustment, rou_asset_adjustment,
                 status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10,
                    $11::numeric, $12::numeric,
                    $13::numeric, $14::numeric,
                    $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(lease_id).bind(modification_number).bind(modification_type)
        .bind(description).bind(effective_date)
        .bind(previous_term_months).bind(new_term_months)
        .bind(previous_end_date).bind(new_end_date)
        .bind(previous_discount_rate).bind(new_discount_rate)
        .bind(liability_adjustment).bind(rou_asset_adjustment)
        .bind(status).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_modification(&row))
    }

    async fn get_next_modification_number(&self, lease_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(modification_number), 0) + 1 as next_num FROM _atlas.lease_modifications WHERE lease_id = $1"
        )
        .bind(lease_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let next: i32 = row.get("next_num");
        Ok(next)
    }

    async fn list_modifications(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseModification>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.lease_modifications WHERE lease_id = $1 ORDER BY modification_number"
        )
        .bind(lease_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_modification(&r)).collect())
    }

    async fn create_termination(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        termination_type: &str,
        termination_date: chrono::NaiveDate,
        reason: Option<&str>,
        remaining_liability: &str,
        remaining_rou_asset: &str,
        termination_penalty: &str,
        gain_loss_amount: &str,
        gain_loss_type: Option<&str>,
        journal_entry_id: Option<Uuid>,
        status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseTermination> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.lease_terminations
                (organization_id, lease_id, termination_type, termination_date,
                 reason, remaining_liability, remaining_rou_asset,
                 termination_penalty, gain_loss_amount, gain_loss_type,
                 journal_entry_id, status, created_by)
            VALUES ($1, $2, $3, $4, $5,
                    $6::numeric, $7::numeric,
                    $8::numeric, $9::numeric, $10,
                    $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(lease_id).bind(termination_type).bind(termination_date)
        .bind(reason).bind(remaining_liability).bind(remaining_rou_asset)
        .bind(termination_penalty).bind(gain_loss_amount).bind(gain_loss_type)
        .bind(journal_entry_id).bind(status).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_termination(&row))
    }

    async fn list_terminations(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseTermination>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.lease_terminations WHERE lease_id = $1 ORDER BY created_at DESC"
        )
        .bind(lease_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_termination(&r)).collect())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<LeaseDashboardSummary> {
        let rows = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status IN ('active', 'modified', 'impaired')) as total_active,
                COALESCE(SUM(current_lease_liability) FILTER (WHERE status IN ('active', 'modified', 'impaired')), 0) as total_liability,
                COALESCE(SUM(current_rou_asset_value) FILTER (WHERE status IN ('active', 'modified', 'impaired')), 0) as total_rou,
                COALESCE(SUM(accumulated_rou_depreciation) FILTER (WHERE status IN ('active', 'modified', 'impaired')), 0) as total_depreciation,
                COALESCE(SUM(total_payments_made) FILTER (WHERE status IN ('active', 'modified', 'impaired')), 0) as total_payments,
                COUNT(*) FILTER (WHERE classification = 'operating' AND status IN ('active', 'modified', 'impaired')) as operating_count,
                COUNT(*) FILTER (WHERE classification = 'finance' AND status IN ('active', 'modified', 'impaired')) as finance_count,
                COUNT(*) FILTER (WHERE end_date <= CURRENT_DATE + INTERVAL '90 days' AND status IN ('active', 'modified', 'impaired')) as expiring_90
            FROM _atlas.lease_contracts
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active: i64 = rows.try_get("total_active").unwrap_or(0);
        let operating: i64 = rows.try_get("operating_count").unwrap_or(0);
        let finance: i64 = rows.try_get("finance_count").unwrap_or(0);
        let expiring: i64 = rows.try_get("expiring_90").unwrap_or(0);

        let liability: serde_json::Value = rows.try_get("total_liability").unwrap_or(serde_json::json!(0));
        let rou: serde_json::Value = rows.try_get("total_rou").unwrap_or(serde_json::json!(0));
        let dep: serde_json::Value = rows.try_get("total_depreciation").unwrap_or(serde_json::json!(0));
        let payments: serde_json::Value = rows.try_get("total_payments").unwrap_or(serde_json::json!(0));

        Ok(LeaseDashboardSummary {
            total_active_leases: active as i32,
            total_lease_liability: liability.to_string(),
            total_rou_assets: rou.to_string(),
            total_rou_depreciation: dep.to_string(),
            total_net_rou_assets: format!("{:.2}", rou.as_f64().unwrap_or(0.0) - dep.as_f64().unwrap_or(0.0)),
            total_payments_made: payments.to_string(),
            operating_lease_count: operating as i32,
            finance_lease_count: finance as i32,
            upcoming_payments_count: 0,
            upcoming_payments_amount: "0".to_string(),
            leases_expiring_90_days: expiring as i32,
            leases_by_classification: serde_json::json!({}),
            leases_by_status: serde_json::json!({}),
            liability_by_period: serde_json::json!({}),
        })
    }
}
