//! Dunning Letter Management Module
//!
//! Oracle Fusion Financials-inspired Dunning Letter Management for Accounts Receivable.
//! Manages escalating payment reminder letters for overdue customer invoices:
//! - Create dunning letter sets with multiple severity levels
//! - Configure customer dunning profiles
//! - Generate batch dunning letter runs
//! - Track dunning letter history and delivery status
//!
//! Letter set statuses: draft → active → inactive
//! Profile statuses: enabled → disabled/hold
//! Run statuses: draft → submitted → completed/cancelled
//! Result statuses: pending → generated/sent/failed/skipped
//!
//! Oracle Fusion equivalent: Financials > Receivables > Dunning Letters

mod engine;

pub use engine::DunningLetterManagementEngine;

use async_trait::async_trait;
use atlas_shared::{AtlasError, AtlasResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Dunning letter set (collection of escalating severity letters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub set_name: String,
    pub description: Option<String>,
    pub status: String,
    pub number_of_levels: i32,
    pub minimum_overdue_days: i32,
    pub currency_code: String,
    pub include_finance_charges: bool,
    pub include_unapplied_receipts: bool,
    pub aging_basis: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Dunning letter set line (individual level within a set)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterSetLine {
    pub id: Uuid,
    pub set_id: Uuid,
    pub level_number: i32,
    pub level_name: String,
    pub min_days_overdue: i32,
    pub max_days_overdue: Option<i32>,
    pub minimum_amount: String,
    pub letter_template: Option<String>,
    pub cc_to: Option<String>,
    pub delivery_method: String,
    pub apply_credit_hold: bool,
    pub assess_finance_charges: bool,
    pub letter_text: Option<String>,
    pub escalation_days: Option<i32>,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Customer dunning profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub letter_set_id: Option<Uuid>,
    pub dunning_status: String,
    pub minimum_overdue_amount: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_address: Option<String>,
    pub last_dunning_level: Option<i32>,
    pub last_dunning_date: Option<chrono::NaiveDate>,
    pub dunning_letter_count: i32,
    pub preferred_delivery_method: Option<String>,
    pub hold_reason: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Dunning letter run (batch generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub description: Option<String>,
    pub letter_set_id: Option<Uuid>,
    pub run_date: chrono::NaiveDate,
    pub aging_as_of_date: chrono::NaiveDate,
    pub status: String,
    pub currency_code: String,
    pub minimum_amount_filter: Option<String>,
    pub specific_level: Option<i32>,
    pub customer_id_filter: Option<Uuid>,
    pub total_customers: i32,
    pub total_letters_generated: i32,
    pub total_overdue_amount: String,
    pub error_count: i32,
    pub warning_count: i32,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Dunning letter run result (per-customer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningLetterRunResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_number: Option<String>,
    pub profile_id: Option<Uuid>,
    pub dunning_level: i32,
    pub level_name: Option<String>,
    pub number_of_overdue_items: i32,
    pub total_overdue_amount: String,
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    pub days_overdue: i32,
    pub finance_charge_amount: String,
    pub letter_template: Option<String>,
    pub status: String,
    pub delivery_method: Option<String>,
    pub sent_date: Option<chrono::NaiveDate>,
    pub delivery_confirmation: Option<String>,
    pub reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait DunningLetterManagementRepository: Send + Sync {
    // Letter Sets
    async fn create_letter_set(
        &self,
        org_id: Uuid,
        set_name: &str,
        description: Option<&str>,
        number_of_levels: i32,
        minimum_overdue_days: i32,
        currency_code: &str,
        include_finance_charges: bool,
        include_unapplied_receipts: bool,
        aging_basis: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterSet>;
    async fn get_letter_set(&self, id: Uuid) -> AtlasResult<Option<DunningLetterSet>>;
    async fn get_letter_set_by_name(
        &self,
        org_id: Uuid,
        name: &str,
    ) -> AtlasResult<Option<DunningLetterSet>>;
    async fn list_letter_sets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningLetterSet>>;
    async fn update_letter_set_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<DunningLetterSet>;

    // Letter Set Lines
    async fn add_letter_set_line(
        &self,
        set_id: Uuid,
        level_number: i32,
        level_name: &str,
        min_days_overdue: i32,
        max_days_overdue: Option<i32>,
        minimum_amount: &str,
        letter_template: Option<&str>,
        delivery_method: &str,
        apply_credit_hold: bool,
        assess_finance_charges: bool,
        letter_text: Option<&str>,
        escalation_days: Option<i32>,
    ) -> AtlasResult<DunningLetterSetLine>;
    async fn list_letter_set_lines(&self, set_id: Uuid) -> AtlasResult<Vec<DunningLetterSetLine>>;

    // Dunning Profiles
    async fn create_profile(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        letter_set_id: Option<Uuid>,
        minimum_overdue_amount: &str,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        preferred_delivery_method: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningProfile>;
    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<DunningProfile>>;
    async fn get_profile_by_customer(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
    ) -> AtlasResult<Option<DunningProfile>>;
    async fn list_profiles(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningProfile>>;
    async fn update_profile_status(
        &self,
        id: Uuid,
        status: &str,
        reason: Option<&str>,
    ) -> AtlasResult<DunningProfile>;
    async fn update_profile_dunning_sent(
        &self,
        id: Uuid,
        level: i32,
        sent_date: chrono::NaiveDate,
    ) -> AtlasResult<DunningProfile>;

    // Dunning Runs
    async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        description: Option<&str>,
        letter_set_id: Option<Uuid>,
        run_date: chrono::NaiveDate,
        aging_as_of_date: chrono::NaiveDate,
        currency_code: &str,
        minimum_amount_filter: Option<&str>,
        specific_level: Option<i32>,
        customer_id_filter: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterRun>;
    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRun>>;
    async fn get_run_by_number(
        &self,
        org_id: Uuid,
        number: &str,
    ) -> AtlasResult<Option<DunningLetterRun>>;
    async fn list_runs(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningLetterRun>>;
    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<DunningLetterRun>;
    async fn update_run_stats(
        &self,
        id: Uuid,
        total_customers: i32,
        total_letters_generated: i32,
        total_overdue_amount: &str,
    ) -> AtlasResult<DunningLetterRun>;

    // Run Results
    async fn create_run_result(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        customer_number: Option<&str>,
        profile_id: Option<Uuid>,
        dunning_level: i32,
        level_name: Option<&str>,
        number_of_overdue_items: i32,
        total_overdue_amount: &str,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        days_overdue: i32,
        finance_charge_amount: &str,
        letter_template: Option<&str>,
        delivery_method: Option<&str>,
        status: &str,
        reason: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult>;
    async fn list_run_results(&self, run_id: Uuid) -> AtlasResult<Vec<DunningLetterRunResult>>;
    async fn get_run_result(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRunResult>>;
    async fn update_run_result_status(
        &self,
        id: Uuid,
        status: &str,
        sent_date: Option<chrono::NaiveDate>,
        delivery_confirmation: Option<&str>,
        reason: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult>;
}

// ============================================================================
// Row conversion helpers
// ============================================================================

fn row_to_letter_set(row: &sqlx::postgres::PgRow) -> DunningLetterSet {
    DunningLetterSet {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        set_name: row.get("set_name"),
        description: row.get("description"),
        status: row.get("status"),
        number_of_levels: row.get("number_of_levels"),
        minimum_overdue_days: row.get("minimum_overdue_days"),
        currency_code: row.get("currency_code"),
        include_finance_charges: row.get("include_finance_charges"),
        include_unapplied_receipts: row.get("include_unapplied_receipts"),
        aging_basis: row.get("aging_basis"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_letter_set_line(row: &sqlx::postgres::PgRow) -> DunningLetterSetLine {
    DunningLetterSetLine {
        id: row.get("id"),
        set_id: row.get("set_id"),
        level_number: row.get("level_number"),
        level_name: row.get("level_name"),
        min_days_overdue: row.get("min_days_overdue"),
        max_days_overdue: row.get("max_days_overdue"),
        minimum_amount: row.get("minimum_amount"),
        letter_template: row.get("letter_template"),
        cc_to: row.get("cc_to"),
        delivery_method: row.get("delivery_method"),
        apply_credit_hold: row.get("apply_credit_hold"),
        assess_finance_charges: row.get("assess_finance_charges"),
        letter_text: row.get("letter_text"),
        escalation_days: row.get("escalation_days"),
        display_order: row.get("display_order"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}

fn row_to_profile(row: &sqlx::postgres::PgRow) -> DunningProfile {
    DunningProfile {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        letter_set_id: row.get("letter_set_id"),
        dunning_status: row.get("dunning_status"),
        minimum_overdue_amount: row.get("minimum_overdue_amount"),
        contact_name: row.get("contact_name"),
        contact_email: row.get("contact_email"),
        contact_address: row.get("contact_address"),
        last_dunning_level: row.get("last_dunning_level"),
        last_dunning_date: row.get("last_dunning_date"),
        dunning_letter_count: row.get("dunning_letter_count"),
        preferred_delivery_method: row.get("preferred_delivery_method"),
        hold_reason: row.get("hold_reason"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_run(row: &sqlx::postgres::PgRow) -> DunningLetterRun {
    DunningLetterRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_number: row.get("run_number"),
        description: row.get("description"),
        letter_set_id: row.get("letter_set_id"),
        run_date: row.get("run_date"),
        aging_as_of_date: row.get("aging_as_of_date"),
        status: row.get("status"),
        currency_code: row.get("currency_code"),
        minimum_amount_filter: row.get("minimum_amount_filter"),
        specific_level: row.get("specific_level"),
        customer_id_filter: row.get("customer_id_filter"),
        total_customers: row.get("total_customers"),
        total_letters_generated: row.get("total_letters_generated"),
        total_overdue_amount: row.get("total_overdue_amount"),
        error_count: row.get("error_count"),
        warning_count: row.get("warning_count"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_run_result(row: &sqlx::postgres::PgRow) -> DunningLetterRunResult {
    DunningLetterRunResult {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_id: row.get("run_id"),
        customer_id: row.get("customer_id"),
        customer_name: row.get("customer_name"),
        customer_number: row.get("customer_number"),
        profile_id: row.get("profile_id"),
        dunning_level: row.get("dunning_level"),
        level_name: row.get("level_name"),
        number_of_overdue_items: row.get("number_of_overdue_items"),
        total_overdue_amount: row.get("total_overdue_amount"),
        oldest_overdue_date: row.get("oldest_overdue_date"),
        days_overdue: row.get("days_overdue"),
        finance_charge_amount: row.get("finance_charge_amount"),
        letter_template: row.get("letter_template"),
        status: row.get("status"),
        delivery_method: row.get("delivery_method"),
        sent_date: row.get("sent_date"),
        delivery_confirmation: row.get("delivery_confirmation"),
        reason: row.get("reason"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

// ============================================================================
// PostgreSQL implementation
// ============================================================================

pub struct PostgresDunningLetterManagementRepository {
    pool: PgPool,
}

impl PostgresDunningLetterManagementRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DunningLetterManagementRepository for PostgresDunningLetterManagementRepository {
    // ========================================================================
    // Letter Sets
    // ========================================================================

    async fn create_letter_set(
        &self,
        org_id: Uuid,
        set_name: &str,
        description: Option<&str>,
        number_of_levels: i32,
        minimum_overdue_days: i32,
        currency_code: &str,
        include_finance_charges: bool,
        include_unapplied_receipts: bool,
        aging_basis: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterSet> {
        let row = sqlx::query(
            "INSERT INTO financials.dunning_letter_sets
                (organization_id, set_name, description, number_of_levels, minimum_overdue_days,
                 currency_code, include_finance_charges, include_unapplied_receipts, aging_basis, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING *"
        )
        .bind(org_id).bind(set_name).bind(description).bind(number_of_levels)
        .bind(minimum_overdue_days).bind(currency_code).bind(include_finance_charges)
        .bind(include_unapplied_receipts).bind(aging_basis).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_letter_set(&row))
    }

    async fn get_letter_set(&self, id: Uuid) -> AtlasResult<Option<DunningLetterSet>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_letter_sets WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_letter_set(&r)))
    }

    async fn get_letter_set_by_name(
        &self,
        org_id: Uuid,
        name: &str,
    ) -> AtlasResult<Option<DunningLetterSet>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_letter_sets WHERE organization_id = $1 AND set_name = $2")
            .bind(org_id).bind(name)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_letter_set(&r)))
    }

    async fn list_letter_sets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningLetterSet>> {
        let rows = if let Some(s) = status {
            sqlx::query("SELECT * FROM financials.dunning_letter_sets WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC")
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM financials.dunning_letter_sets WHERE organization_id = $1 ORDER BY created_at DESC")
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_letter_set).collect())
    }

    async fn update_letter_set_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<DunningLetterSet> {
        let row = sqlx::query(
            "UPDATE financials.dunning_letter_sets SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_letter_set(&row))
    }

    // ========================================================================
    // Letter Set Lines
    // ========================================================================

    async fn add_letter_set_line(
        &self,
        set_id: Uuid,
        level_number: i32,
        level_name: &str,
        min_days_overdue: i32,
        max_days_overdue: Option<i32>,
        minimum_amount: &str,
        letter_template: Option<&str>,
        delivery_method: &str,
        apply_credit_hold: bool,
        assess_finance_charges: bool,
        letter_text: Option<&str>,
        escalation_days: Option<i32>,
    ) -> AtlasResult<DunningLetterSetLine> {
        let row = sqlx::query(
            "INSERT INTO financials.dunning_letter_set_lines
                (set_id, level_number, level_name, min_days_overdue, max_days_overdue,
                 minimum_amount, letter_template, delivery_method, apply_credit_hold,
                 assess_finance_charges, letter_text, escalation_days, display_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $2)
             RETURNING *",
        )
        .bind(set_id)
        .bind(level_number)
        .bind(level_name)
        .bind(min_days_overdue)
        .bind(max_days_overdue)
        .bind(minimum_amount)
        .bind(letter_template)
        .bind(delivery_method)
        .bind(apply_credit_hold)
        .bind(assess_finance_charges)
        .bind(letter_text)
        .bind(escalation_days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_letter_set_line(&row))
    }

    async fn list_letter_set_lines(&self, set_id: Uuid) -> AtlasResult<Vec<DunningLetterSetLine>> {
        let rows = sqlx::query(
            "SELECT * FROM financials.dunning_letter_set_lines WHERE set_id = $1 ORDER BY level_number"
        ).bind(set_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_letter_set_line).collect())
    }

    // ========================================================================
    // Dunning Profiles
    // ========================================================================

    async fn create_profile(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        letter_set_id: Option<Uuid>,
        minimum_overdue_amount: &str,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        preferred_delivery_method: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningProfile> {
        let row = sqlx::query(
            "INSERT INTO financials.dunning_profiles
                (organization_id, customer_id, customer_name, letter_set_id,
                 minimum_overdue_amount, contact_name, contact_email,
                 preferred_delivery_method, notes, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING *",
        )
        .bind(org_id)
        .bind(customer_id)
        .bind(customer_name)
        .bind(letter_set_id)
        .bind(minimum_overdue_amount)
        .bind(contact_name)
        .bind(contact_email)
        .bind(preferred_delivery_method)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_profile(&row))
    }

    async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<DunningProfile>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_profiles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_profile(&r)))
    }

    async fn get_profile_by_customer(
        &self,
        org_id: Uuid,
        customer_id: Uuid,
    ) -> AtlasResult<Option<DunningProfile>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_profiles WHERE organization_id = $1 AND customer_id = $2")
            .bind(org_id).bind(customer_id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_profile(&r)))
    }

    async fn list_profiles(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningProfile>> {
        let rows = if let Some(s) = status {
            sqlx::query("SELECT * FROM financials.dunning_profiles WHERE organization_id = $1 AND dunning_status = $2 ORDER BY created_at DESC")
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM financials.dunning_profiles WHERE organization_id = $1 ORDER BY created_at DESC")
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_profile).collect())
    }

    async fn update_profile_status(
        &self,
        id: Uuid,
        status: &str,
        reason: Option<&str>,
    ) -> AtlasResult<DunningProfile> {
        let row = sqlx::query(
            "UPDATE financials.dunning_profiles SET dunning_status = $2, hold_reason = $3, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_profile(&row))
    }

    async fn update_profile_dunning_sent(
        &self,
        id: Uuid,
        level: i32,
        sent_date: chrono::NaiveDate,
    ) -> AtlasResult<DunningProfile> {
        let row = sqlx::query(
            "UPDATE financials.dunning_profiles
             SET last_dunning_level = $2, last_dunning_date = $3,
                 dunning_letter_count = dunning_letter_count + 1, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(level)
        .bind(sent_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_profile(&row))
    }

    // ========================================================================
    // Dunning Runs
    // ========================================================================

    async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        description: Option<&str>,
        letter_set_id: Option<Uuid>,
        run_date: chrono::NaiveDate,
        aging_as_of_date: chrono::NaiveDate,
        currency_code: &str,
        minimum_amount_filter: Option<&str>,
        specific_level: Option<i32>,
        customer_id_filter: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DunningLetterRun> {
        let row = sqlx::query(
            "INSERT INTO financials.dunning_letter_runs
                (organization_id, run_number, description, letter_set_id, run_date,
                 aging_as_of_date, currency_code, minimum_amount_filter, specific_level,
                 customer_id_filter, notes, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *",
        )
        .bind(org_id)
        .bind(run_number)
        .bind(description)
        .bind(letter_set_id)
        .bind(run_date)
        .bind(aging_as_of_date)
        .bind(currency_code)
        .bind(minimum_amount_filter)
        .bind(specific_level)
        .bind(customer_id_filter)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRun>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_letter_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn get_run_by_number(
        &self,
        org_id: Uuid,
        number: &str,
    ) -> AtlasResult<Option<DunningLetterRun>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_letter_runs WHERE organization_id = $1 AND run_number = $2")
            .bind(org_id).bind(number)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn list_runs(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<DunningLetterRun>> {
        let rows = if let Some(s) = status {
            sqlx::query("SELECT * FROM financials.dunning_letter_runs WHERE organization_id = $1 AND status = $2 ORDER BY run_date DESC")
                .bind(org_id).bind(s)
                .fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM financials.dunning_letter_runs WHERE organization_id = $1 ORDER BY run_date DESC")
                .bind(org_id)
                .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run).collect())
    }

    async fn update_run_status(&self, id: Uuid, status: &str) -> AtlasResult<DunningLetterRun> {
        let row = sqlx::query(
            "UPDATE financials.dunning_letter_runs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    async fn update_run_stats(
        &self,
        id: Uuid,
        total_customers: i32,
        total_letters_generated: i32,
        total_overdue_amount: &str,
    ) -> AtlasResult<DunningLetterRun> {
        let row = sqlx::query(
            "UPDATE financials.dunning_letter_runs
             SET total_customers = $2, total_letters_generated = $3,
                 total_overdue_amount = $4, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(total_customers)
        .bind(total_letters_generated)
        .bind(total_overdue_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    // ========================================================================
    // Run Results
    // ========================================================================

    async fn create_run_result(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        customer_id: Uuid,
        customer_name: Option<&str>,
        customer_number: Option<&str>,
        profile_id: Option<Uuid>,
        dunning_level: i32,
        level_name: Option<&str>,
        number_of_overdue_items: i32,
        total_overdue_amount: &str,
        oldest_overdue_date: Option<chrono::NaiveDate>,
        days_overdue: i32,
        finance_charge_amount: &str,
        letter_template: Option<&str>,
        delivery_method: Option<&str>,
        status: &str,
        reason: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult> {
        let row = sqlx::query(
            "INSERT INTO financials.dunning_letter_run_results
                (organization_id, run_id, customer_id, customer_name, customer_number,
                 profile_id, dunning_level, level_name, number_of_overdue_items,
                 total_overdue_amount, oldest_overdue_date, days_overdue, finance_charge_amount,
                 letter_template, delivery_method, status, reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
             RETURNING *",
        )
        .bind(org_id)
        .bind(run_id)
        .bind(customer_id)
        .bind(customer_name)
        .bind(customer_number)
        .bind(profile_id)
        .bind(dunning_level)
        .bind(level_name)
        .bind(number_of_overdue_items)
        .bind(total_overdue_amount)
        .bind(oldest_overdue_date)
        .bind(days_overdue)
        .bind(finance_charge_amount)
        .bind(letter_template)
        .bind(delivery_method)
        .bind(status)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run_result(&row))
    }

    async fn list_run_results(&self, run_id: Uuid) -> AtlasResult<Vec<DunningLetterRunResult>> {
        let rows = sqlx::query(
            "SELECT * FROM financials.dunning_letter_run_results WHERE run_id = $1 ORDER BY customer_name"
        ).bind(run_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run_result).collect())
    }

    async fn get_run_result(&self, id: Uuid) -> AtlasResult<Option<DunningLetterRunResult>> {
        let row = sqlx::query("SELECT * FROM financials.dunning_letter_run_results WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run_result(&r)))
    }

    async fn update_run_result_status(
        &self,
        id: Uuid,
        status: &str,
        sent_date: Option<chrono::NaiveDate>,
        delivery_confirmation: Option<&str>,
        reason: Option<&str>,
    ) -> AtlasResult<DunningLetterRunResult> {
        let row = sqlx::query(
            "UPDATE financials.dunning_letter_run_results
             SET status = $2, sent_date = $3, delivery_confirmation = $4, reason = $5, updated_at = now()
             WHERE id = $1 RETURNING *"
        ).bind(id).bind(status).bind(sent_date).bind(delivery_confirmation).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run_result(&row))
    }
}
