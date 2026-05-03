//! Bank Guarantee Repository
//!
//! PostgreSQL storage for bank guarantees, amendments, and dashboard queries.

use atlas_shared::{
    BankGuarantee, BankGuaranteeAmendment,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Repository trait for bank guarantee data storage
#[async_trait]
pub trait BankGuaranteeRepository: Send + Sync {
    // Guarantees
    async fn create_guarantee(
        &self, org_id: Uuid, guarantee_number: &str, guarantee_type: &str,
        description: Option<&str>,
        beneficiary_name: &str, beneficiary_code: Option<&str>,
        applicant_name: &str, applicant_code: Option<&str>,
        issuing_bank_name: &str, issuing_bank_code: Option<&str>,
        bank_account_number: Option<&str>,
        guarantee_amount: &str, currency_code: &str,
        margin_percentage: &str, margin_amount: &str,
        commission_rate: &str, commission_amount: &str,
        issue_date: Option<chrono::NaiveDate>, effective_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>,
        claim_expiry_date: Option<chrono::NaiveDate>, renewal_date: Option<chrono::NaiveDate>,
        auto_renew: bool,
        reference_contract_number: Option<&str>, reference_purchase_order: Option<&str>,
        purpose: Option<&str>,
        collateral_type: Option<&str>, collateral_amount: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuarantee>;
    async fn get_guarantee(&self, org_id: Uuid, guarantee_number: &str) -> AtlasResult<Option<BankGuarantee>>;
    async fn get_guarantee_by_id(&self, id: Uuid) -> AtlasResult<Option<BankGuarantee>>;
    async fn list_guarantees(&self, org_id: Uuid, status: Option<&str>, guarantee_type: Option<&str>) -> AtlasResult<Vec<BankGuarantee>>;
    async fn update_guarantee_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<BankGuarantee>;
    async fn update_guarantee_amounts(&self, id: Uuid, guarantee_amount: &str, margin_amount: &str, commission_amount: &str) -> AtlasResult<()>;
    async fn update_guarantee_expiry(&self, id: Uuid, expiry_date: chrono::NaiveDate) -> AtlasResult<()>;
    async fn increment_amendment_count(&self, id: Uuid, latest_amendment_number: &str) -> AtlasResult<()>;
    async fn delete_guarantee(&self, org_id: Uuid, guarantee_number: &str) -> AtlasResult<()>;

    // Amendments
    async fn create_amendment(
        &self, org_id: Uuid, guarantee_id: Uuid, guarantee_number: &str,
        amendment_number: &str, amendment_type: &str,
        previous_amount: Option<&str>, new_amount: Option<&str>,
        previous_expiry_date: Option<chrono::NaiveDate>, new_expiry_date: Option<chrono::NaiveDate>,
        previous_terms: Option<&str>, new_terms: Option<&str>,
        reason: Option<&str>, effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuaranteeAmendment>;
    async fn get_amendment_by_id(&self, id: Uuid) -> AtlasResult<Option<BankGuaranteeAmendment>>;
    async fn list_amendments(&self, guarantee_id: Uuid) -> AtlasResult<Vec<BankGuaranteeAmendment>>;
    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<BankGuaranteeAmendment>;
    async fn count_pending_amendments(&self, org_id: Uuid) -> AtlasResult<i64>;
}

/// Guarantee columns with NUMERIC cast to text for sqlx compatibility
const BG_SELECT: &str = "id, org_id, guarantee_number, guarantee_type, description, \
    beneficiary_name, beneficiary_code, applicant_name, applicant_code, \
    issuing_bank_name, issuing_bank_code, bank_account_number, \
    guarantee_amount::text, currency_code, \
    margin_percentage::text, margin_amount::text, \
    commission_rate::text, commission_amount::text, \
    issue_date, effective_date, expiry_date, \
    claim_expiry_date, renewal_date, auto_renew, \
    reference_contract_number, reference_purchase_order, \
    purpose, collateral_type, collateral_amount::text, \
    status, amendment_count, latest_amendment_number, \
    created_by_id, approved_by_id, notes, created_at, updated_at";

/// Amendment columns with NUMERIC cast to text
const AMD_SELECT: &str = "id, org_id, guarantee_id, guarantee_number, \
    amendment_number, amendment_type, \
    previous_amount::text, new_amount::text, \
    previous_expiry_date, new_expiry_date, \
    previous_terms, new_terms, reason, \
    status, effective_date, approved_by_id, created_at, updated_at";

/// PostgreSQL implementation of BankGuaranteeRepository
pub struct PostgresBankGuaranteeRepository {
    pool: PgPool,
}

impl PostgresBankGuaranteeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_guarantee(row: &sqlx::postgres::PgRow) -> BankGuarantee {
        BankGuarantee {
            id: row.get("id"),
            org_id: row.get("org_id"),
            guarantee_number: row.get("guarantee_number"),
            guarantee_type: row.get("guarantee_type"),
            description: row.get("description"),
            beneficiary_name: row.get("beneficiary_name"),
            beneficiary_code: row.get("beneficiary_code"),
            applicant_name: row.get("applicant_name"),
            applicant_code: row.get("applicant_code"),
            issuing_bank_name: row.get("issuing_bank_name"),
            issuing_bank_code: row.get("issuing_bank_code"),
            bank_account_number: row.get("bank_account_number"),
            guarantee_amount: row.get::<String, _>("guarantee_amount"),
            currency_code: row.get("currency_code"),
            margin_percentage: row.get::<String, _>("margin_percentage"),
            margin_amount: row.get::<String, _>("margin_amount"),
            commission_rate: row.get::<String, _>("commission_rate"),
            commission_amount: row.get::<String, _>("commission_amount"),
            issue_date: row.get("issue_date"),
            effective_date: row.get("effective_date"),
            expiry_date: row.get("expiry_date"),
            claim_expiry_date: row.get("claim_expiry_date"),
            renewal_date: row.get("renewal_date"),
            auto_renew: row.get("auto_renew"),
            reference_contract_number: row.get("reference_contract_number"),
            reference_purchase_order: row.get("reference_purchase_order"),
            purpose: row.get("purpose"),
            collateral_type: row.get("collateral_type"),
            collateral_amount: row.get("collateral_amount"),
            status: row.get("status"),
            amendment_count: row.get("amendment_count"),
            latest_amendment_number: row.get("latest_amendment_number"),
            created_by_id: row.get("created_by_id"),
            approved_by_id: row.get("approved_by_id"),
            notes: row.get("notes"),
            created_at: row.get::<DateTime<Utc>, _>("created_at"),
            updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
        }
    }

    fn row_to_amendment(row: &sqlx::postgres::PgRow) -> BankGuaranteeAmendment {
        BankGuaranteeAmendment {
            id: row.get("id"),
            org_id: row.get("org_id"),
            guarantee_id: row.get("guarantee_id"),
            guarantee_number: row.get("guarantee_number"),
            amendment_number: row.get("amendment_number"),
            amendment_type: row.get("amendment_type"),
            previous_amount: row.get("previous_amount"),
            new_amount: row.get("new_amount"),
            previous_expiry_date: row.get("previous_expiry_date"),
            new_expiry_date: row.get("new_expiry_date"),
            previous_terms: row.get("previous_terms"),
            new_terms: row.get("new_terms"),
            reason: row.get("reason"),
            status: row.get("status"),
            effective_date: row.get("effective_date"),
            approved_by_id: row.get("approved_by_id"),
            created_at: row.get::<DateTime<Utc>, _>("created_at"),
            updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
        }
    }
}

#[async_trait]
impl BankGuaranteeRepository for PostgresBankGuaranteeRepository {
    async fn create_guarantee(
        &self,
        org_id: Uuid, guarantee_number: &str, guarantee_type: &str,
        description: Option<&str>,
        beneficiary_name: &str, beneficiary_code: Option<&str>,
        applicant_name: &str, applicant_code: Option<&str>,
        issuing_bank_name: &str, issuing_bank_code: Option<&str>,
        bank_account_number: Option<&str>,
        guarantee_amount: &str, currency_code: &str,
        margin_percentage: &str, margin_amount: &str,
        commission_rate: &str, commission_amount: &str,
        issue_date: Option<chrono::NaiveDate>, effective_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>,
        claim_expiry_date: Option<chrono::NaiveDate>, renewal_date: Option<chrono::NaiveDate>,
        auto_renew: bool,
        reference_contract_number: Option<&str>, reference_purchase_order: Option<&str>,
        purpose: Option<&str>,
        collateral_type: Option<&str>, collateral_amount: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuarantee> {
        let sql = format!(
            "INSERT INTO fin_bank_guarantees (\
                org_id, guarantee_number, guarantee_type, description, \
                beneficiary_name, beneficiary_code, \
                applicant_name, applicant_code, \
                issuing_bank_name, issuing_bank_code, bank_account_number, \
                guarantee_amount, currency_code, \
                margin_percentage, margin_amount, \
                commission_rate, commission_amount, \
                issue_date, effective_date, expiry_date, \
                claim_expiry_date, renewal_date, auto_renew, \
                reference_contract_number, reference_purchase_order, \
                purpose, collateral_type, collateral_amount, \
                notes, created_by_id) \
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12::numeric,$13,$14::numeric,$15::numeric,$16::numeric,$17::numeric,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28::numeric,$29,$30) \
            RETURNING {}",
            BG_SELECT
        );

        let row = sqlx::query(&sql)
            .bind(org_id)
            .bind(guarantee_number)
            .bind(guarantee_type)
            .bind(description)
            .bind(beneficiary_name)
            .bind(beneficiary_code)
            .bind(applicant_name)
            .bind(applicant_code)
            .bind(issuing_bank_name)
            .bind(issuing_bank_code)
            .bind(bank_account_number)
            .bind(guarantee_amount)
            .bind(currency_code)
            .bind(margin_percentage)
            .bind(margin_amount)
            .bind(commission_rate)
            .bind(commission_amount)
            .bind(issue_date)
            .bind(effective_date)
            .bind(expiry_date)
            .bind(claim_expiry_date)
            .bind(renewal_date)
            .bind(auto_renew)
            .bind(reference_contract_number)
            .bind(reference_purchase_order)
            .bind(purpose)
            .bind(collateral_type)
            .bind(collateral_amount)
            .bind(notes)
            .bind(created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(Self::row_to_guarantee(&row))
    }

    async fn get_guarantee(&self, org_id: Uuid, guarantee_number: &str) -> AtlasResult<Option<BankGuarantee>> {
        let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE org_id = $1 AND guarantee_number = $2", BG_SELECT);
        let row = sqlx::query(&sql)
            .bind(org_id)
            .bind(guarantee_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.as_ref().map(Self::row_to_guarantee))
    }

    async fn get_guarantee_by_id(&self, id: Uuid) -> AtlasResult<Option<BankGuarantee>> {
        let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE id = $1", BG_SELECT);
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.as_ref().map(Self::row_to_guarantee))
    }

    async fn list_guarantees(&self, org_id: Uuid, status: Option<&str>, guarantee_type: Option<&str>) -> AtlasResult<Vec<BankGuarantee>> {
        let rows = match (status, guarantee_type) {
            (Some(s), Some(t)) => {
                let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE org_id = $1 AND status = $2 AND guarantee_type = $3 ORDER BY created_at DESC", BG_SELECT);
                sqlx::query(&sql)
                    .bind(org_id).bind(s).bind(t)
                    .fetch_all(&self.pool).await
            }
            (Some(s), None) => {
                let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE org_id = $1 AND status = $2 ORDER BY created_at DESC", BG_SELECT);
                sqlx::query(&sql)
                    .bind(org_id).bind(s)
                    .fetch_all(&self.pool).await
            }
            (None, Some(t)) => {
                let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE org_id = $1 AND guarantee_type = $2 ORDER BY created_at DESC", BG_SELECT);
                sqlx::query(&sql)
                    .bind(org_id).bind(t)
                    .fetch_all(&self.pool).await
            }
            (None, None) => {
                let sql = format!("SELECT {} FROM fin_bank_guarantees WHERE org_id = $1 ORDER BY created_at DESC", BG_SELECT);
                sqlx::query(&sql)
                    .bind(org_id)
                    .fetch_all(&self.pool).await
            }
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(Self::row_to_guarantee).collect())
    }

    async fn update_guarantee_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<BankGuarantee> {
        let sql = format!(
            "UPDATE fin_bank_guarantees SET status = $2, approved_by_id = COALESCE($3, approved_by_id), updated_at = now() WHERE id = $1 RETURNING {}",
            BG_SELECT
        );
        let row = sqlx::query(&sql)
            .bind(id).bind(status).bind(approved_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(Self::row_to_guarantee(&row))
    }

    async fn update_guarantee_amounts(&self, id: Uuid, guarantee_amount: &str, margin_amount: &str, commission_amount: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE fin_bank_guarantees SET guarantee_amount = $2::numeric, margin_amount = $3::numeric, commission_amount = $4::numeric, updated_at = now() WHERE id = $1"
        )
            .bind(id).bind(guarantee_amount).bind(margin_amount).bind(commission_amount)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_guarantee_expiry(&self, id: Uuid, expiry_date: chrono::NaiveDate) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE fin_bank_guarantees SET expiry_date = $2, updated_at = now() WHERE id = $1"
        )
            .bind(id).bind(expiry_date)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn increment_amendment_count(&self, id: Uuid, latest_amendment_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE fin_bank_guarantees SET amendment_count = amendment_count + 1, latest_amendment_number = $2, updated_at = now() WHERE id = $1"
        )
            .bind(id).bind(latest_amendment_number)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_guarantee(&self, org_id: Uuid, guarantee_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM fin_bank_guarantees WHERE org_id = $1 AND guarantee_number = $2 AND status = 'draft'"
        )
            .bind(org_id).bind(guarantee_number)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!(
                "Draft guarantee '{}' not found", guarantee_number
            )));
        }
        Ok(())
    }

    // Amendments

    async fn create_amendment(
        &self,
        org_id: Uuid, guarantee_id: Uuid, guarantee_number: &str,
        amendment_number: &str, amendment_type: &str,
        previous_amount: Option<&str>, new_amount: Option<&str>,
        previous_expiry_date: Option<chrono::NaiveDate>, new_expiry_date: Option<chrono::NaiveDate>,
        previous_terms: Option<&str>, new_terms: Option<&str>,
        reason: Option<&str>, effective_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankGuaranteeAmendment> {
        let sql = format!(
            "INSERT INTO fin_bank_guarantee_amendments (\
                org_id, guarantee_id, guarantee_number, \
                amendment_number, amendment_type, \
                previous_amount, new_amount, \
                previous_expiry_date, new_expiry_date, \
                previous_terms, new_terms, \
                reason, effective_date, created_by_id) \
            VALUES ($1,$2,$3,$4,$5,$6::numeric,$7::numeric,$8,$9,$10,$11,$12,$13,$14) \
            RETURNING {}",
            AMD_SELECT
        );

        let row = sqlx::query(&sql)
            .bind(org_id)
            .bind(guarantee_id)
            .bind(guarantee_number)
            .bind(amendment_number)
            .bind(amendment_type)
            .bind(previous_amount)
            .bind(new_amount)
            .bind(previous_expiry_date)
            .bind(new_expiry_date)
            .bind(previous_terms)
            .bind(new_terms)
            .bind(reason)
            .bind(effective_date)
            .bind(created_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(Self::row_to_amendment(&row))
    }

    async fn get_amendment_by_id(&self, id: Uuid) -> AtlasResult<Option<BankGuaranteeAmendment>> {
        let sql = format!("SELECT {} FROM fin_bank_guarantee_amendments WHERE id = $1", AMD_SELECT);
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.as_ref().map(Self::row_to_amendment))
    }

    async fn list_amendments(&self, guarantee_id: Uuid) -> AtlasResult<Vec<BankGuaranteeAmendment>> {
        let sql = format!("SELECT {} FROM fin_bank_guarantee_amendments WHERE guarantee_id = $1 ORDER BY created_at DESC", AMD_SELECT);
        let rows = sqlx::query(&sql)
            .bind(guarantee_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(Self::row_to_amendment).collect())
    }

    async fn update_amendment_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<BankGuaranteeAmendment> {
        let sql = format!(
            "UPDATE fin_bank_guarantee_amendments SET status = $2, approved_by_id = COALESCE($3, approved_by_id), updated_at = now() WHERE id = $1 RETURNING {}",
            AMD_SELECT
        );
        let row = sqlx::query(&sql)
            .bind(id).bind(status).bind(approved_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(Self::row_to_amendment(&row))
    }

    async fn count_pending_amendments(&self, org_id: Uuid) -> AtlasResult<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM fin_bank_guarantee_amendments WHERE org_id = $1 AND status = 'pending_approval'"
        )
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.get("count"))
    }
}
