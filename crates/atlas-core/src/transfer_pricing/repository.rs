//! Transfer Pricing Repository
//!
//! PostgreSQL storage for transfer pricing policies, transactions,
//! benchmark studies, comparables, and documentation packages.

use atlas_shared::{
    TransferPricingPolicy, TransferPriceTransaction,
    BenchmarkStudy, BenchmarkComparable,
    TransferPricingDocumentation, TransferPricingDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Repository trait for transfer pricing data storage
#[async_trait]
pub trait TransferPricingRepository: Send + Sync {
    // Policies
    async fn create_policy(
        &self, org_id: Uuid, policy_code: &str, name: &str, description: Option<&str>,
        pricing_method: &str, from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        product_category: Option<&str>, item_id: Option<Uuid>, item_code: Option<&str>,
        geography: Option<&str>, tax_jurisdiction: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        arm_length_range_low: Option<&str>, arm_length_range_mid: Option<&str>,
        arm_length_range_high: Option<&str>, margin_pct: Option<&str>,
        cost_base: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingPolicy>;
    async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransferPricingPolicy>>;
    async fn get_policy_by_id(&self, id: Uuid) -> AtlasResult<Option<TransferPricingPolicy>>;
    async fn list_policies(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TransferPricingPolicy>>;
    async fn update_policy_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<TransferPricingPolicy>;
    async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Transactions
    async fn create_transaction(
        &self, org_id: Uuid, transaction_number: &str, policy_id: Option<Uuid>,
        from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
        quantity: &str, unit_cost: &str, transfer_price: &str,
        total_amount: &str, currency_code: &str, transaction_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        margin_applied: Option<&str>, margin_amount: Option<&str>,
        is_arm_length_compliant: Option<bool>, compliance_notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPriceTransaction>;
    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<TransferPriceTransaction>>;
    async fn list_transactions(&self, org_id: Uuid, status: Option<&str>, policy_id: Option<Uuid>) -> AtlasResult<Vec<TransferPriceTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, submitted_at: Option<DateTime<Utc>>, approved_by: Option<Uuid>) -> AtlasResult<TransferPriceTransaction>;

    // Benchmarks
    async fn create_benchmark(
        &self, org_id: Uuid, study_number: &str, title: &str, description: Option<&str>,
        policy_id: Option<Uuid>, analysis_method: &str, fiscal_year: Option<i32>,
        from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        product_category: Option<&str>, tested_party: Option<&str>,
        prepared_by: Option<Uuid>, prepared_by_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenchmarkStudy>;
    async fn get_benchmark(&self, id: Uuid) -> AtlasResult<Option<BenchmarkStudy>>;
    async fn list_benchmarks(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BenchmarkStudy>>;
    async fn update_benchmark_status(&self, id: Uuid, status: &str, reviewed_by: Option<Uuid>, reviewed_by_name: Option<&str>) -> AtlasResult<BenchmarkStudy>;
    async fn delete_benchmark(&self, id: Uuid) -> AtlasResult<()>;

    // Comparables
    async fn add_comparable(
        &self, org_id: Uuid, benchmark_id: Uuid, comparable_number: i32,
        company_name: &str, country: Option<&str>,
        industry_code: Option<&str>, industry_description: Option<&str>,
        fiscal_year: Option<i32>, revenue: Option<&str>,
        operating_income: Option<&str>, operating_margin_pct: Option<&str>,
        net_income: Option<&str>, total_assets: Option<&str>,
        employees: Option<i32>, data_source: Option<&str>,
    ) -> AtlasResult<BenchmarkComparable>;
    async fn list_comparables(&self, benchmark_id: Uuid) -> AtlasResult<Vec<BenchmarkComparable>>;
    async fn update_comparable_inclusion(&self, id: Uuid, included: bool, reason: Option<&str>) -> AtlasResult<BenchmarkComparable>;

    // Documentation
    async fn create_documentation(
        &self, org_id: Uuid, doc_number: &str, title: &str, doc_type: &str,
        fiscal_year: i32, country: Option<&str>,
        reporting_entity_id: Option<Uuid>, reporting_entity_name: Option<&str>,
        description: Option<&str>, content_summary: Option<&str>,
        filing_deadline: Option<chrono::NaiveDate>, responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingDocumentation>;
    async fn get_documentation(&self, id: Uuid) -> AtlasResult<Option<TransferPricingDocumentation>>;
    async fn list_documentation(&self, org_id: Uuid, doc_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<TransferPricingDocumentation>>;
    async fn update_documentation_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, filed_at: Option<DateTime<Utc>>) -> AtlasResult<TransferPricingDocumentation>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransferPricingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresTransferPricingRepository {
    pool: PgPool,
}

impl PostgresTransferPricingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_policy(row: &sqlx::postgres::PgRow) -> TransferPricingPolicy {
    TransferPricingPolicy {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        policy_code: row.get("policy_code"),
        name: row.get("name"),
        description: row.get("description"),
        pricing_method: row.get("pricing_method"),
        from_entity_id: row.get("from_entity_id"),
        from_entity_name: row.get("from_entity_name"),
        to_entity_id: row.get("to_entity_id"),
        to_entity_name: row.get("to_entity_name"),
        product_category: row.get("product_category"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        geography: row.get("geography"),
        tax_jurisdiction: row.get("tax_jurisdiction"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        arm_length_range_low: row_to_numeric(row, "arm_length_range_low"),
        arm_length_range_mid: row_to_numeric(row, "arm_length_range_mid"),
        arm_length_range_high: row_to_numeric(row, "arm_length_range_high"),
        margin_pct: row_to_numeric(row, "margin_pct"),
        cost_base: row.get("cost_base"),
        status: row.get("status"),
        version: row.get("version"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> TransferPriceTransaction {
    TransferPriceTransaction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        transaction_number: row.get("transaction_number"),
        policy_id: row.get("policy_id"),
        from_entity_id: row.get("from_entity_id"),
        from_entity_name: row.get("from_entity_name"),
        to_entity_id: row.get("to_entity_id"),
        to_entity_name: row.get("to_entity_name"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        quantity: row_to_numeric(row, "quantity"),
        unit_cost: row_to_numeric(row, "unit_cost"),
        transfer_price: row_to_numeric(row, "transfer_price"),
        total_amount: row_to_numeric(row, "total_amount"),
        currency_code: row.get("currency_code"),
        transaction_date: row.get("transaction_date"),
        gl_date: row.get("gl_date"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        margin_applied: row.try_get("margin_applied").ok().map(|v: serde_json::Value| v.to_string()),
        margin_amount: row.try_get("margin_amount").ok().map(|v: serde_json::Value| v.to_string()),
        is_arm_length_compliant: row.get("is_arm_length_compliant"),
        compliance_notes: row.get("compliance_notes"),
        status: row.get("status"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_benchmark(row: &sqlx::postgres::PgRow) -> BenchmarkStudy {
    BenchmarkStudy {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        study_number: row.get("study_number"),
        title: row.get("title"),
        description: row.get("description"),
        policy_id: row.get("policy_id"),
        analysis_method: row.get("analysis_method"),
        fiscal_year: row.get("fiscal_year"),
        from_entity_id: row.get("from_entity_id"),
        from_entity_name: row.get("from_entity_name"),
        to_entity_id: row.get("to_entity_id"),
        to_entity_name: row.get("to_entity_name"),
        product_category: row.get("product_category"),
        tested_party: row.get("tested_party"),
        interquartile_range_low: row_to_numeric(row, "interquartile_range_low"),
        interquartile_range_mid: row_to_numeric(row, "interquartile_range_mid"),
        interquartile_range_high: row_to_numeric(row, "interquartile_range_high"),
        tested_result: row_to_numeric(row, "tested_result"),
        is_within_range: row.get("is_within_range"),
        conclusion: row.get("conclusion"),
        prepared_by: row.get("prepared_by"),
        prepared_by_name: row.get("prepared_by_name"),
        reviewed_by: row.get("reviewed_by"),
        reviewed_by_name: row.get("reviewed_by_name"),
        status: row.get("status"),
        approved_at: row.get("approved_at"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_comparable(row: &sqlx::postgres::PgRow) -> BenchmarkComparable {
    BenchmarkComparable {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        benchmark_id: row.get("benchmark_id"),
        comparable_number: row.get("comparable_number"),
        company_name: row.get("company_name"),
        country: row.get("country"),
        industry_code: row.get("industry_code"),
        industry_description: row.get("industry_description"),
        fiscal_year: row.get("fiscal_year"),
        revenue: row_to_numeric(row, "revenue"),
        operating_income: row_to_numeric(row, "operating_income"),
        operating_margin_pct: row_to_numeric(row, "operating_margin_pct"),
        net_income: row_to_numeric(row, "net_income"),
        total_assets: row_to_numeric(row, "total_assets"),
        employees: row.get("employees"),
        data_source: row.get("data_source"),
        is_included: row.get("is_included"),
        exclusion_reason: row.get("exclusion_reason"),
        relevance_score: row.try_get("relevance_score").ok().map(|v: serde_json::Value| v.to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_documentation(row: &sqlx::postgres::PgRow) -> TransferPricingDocumentation {
    TransferPricingDocumentation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        doc_number: row.get("doc_number"),
        title: row.get("title"),
        doc_type: row.get("doc_type"),
        fiscal_year: row.get("fiscal_year"),
        country: row.get("country"),
        reporting_entity_id: row.get("reporting_entity_id"),
        reporting_entity_name: row.get("reporting_entity_name"),
        description: row.get("description"),
        content_summary: row.get("content_summary"),
        policy_ids: row.try_get("policy_ids").ok(),
        benchmark_ids: row.try_get("benchmark_ids").ok(),
        filing_date: row.get("filing_date"),
        filing_deadline: row.get("filing_deadline"),
        responsible_party: row.get("responsible_party"),
        status: row.get("status"),
        reviewed_by: row.get("reviewed_by"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        filed_at: row.get("filed_at"),
        created_by: row.get("created_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl TransferPricingRepository for PostgresTransferPricingRepository {
    // ========================================================================
    // Policies
    // ========================================================================

    async fn create_policy(
        &self, org_id: Uuid, policy_code: &str, name: &str, description: Option<&str>,
        pricing_method: &str, from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        product_category: Option<&str>, item_id: Option<Uuid>, item_code: Option<&str>,
        geography: Option<&str>, tax_jurisdiction: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        arm_length_range_low: Option<&str>, arm_length_range_mid: Option<&str>,
        arm_length_range_high: Option<&str>, margin_pct: Option<&str>,
        cost_base: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingPolicy> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transfer_pricing_policies
                (organization_id, policy_code, name, description, pricing_method,
                 from_entity_id, from_entity_name, to_entity_id, to_entity_name,
                 product_category, item_id, item_code, geography, tax_jurisdiction,
                 effective_from, effective_to,
                 arm_length_range_low, arm_length_range_mid, arm_length_range_high,
                 margin_pct, cost_base, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,
                    $17::numeric,$18::numeric,$19::numeric,$20::numeric,$21,$22)
            RETURNING *"#,
        )
        .bind(org_id).bind(policy_code).bind(name).bind(description)
        .bind(pricing_method)
        .bind(from_entity_id).bind(from_entity_name)
        .bind(to_entity_id).bind(to_entity_name)
        .bind(product_category).bind(item_id).bind(item_code)
        .bind(geography).bind(tax_jurisdiction)
        .bind(effective_from).bind(effective_to)
        .bind(arm_length_range_low).bind(arm_length_range_mid).bind(arm_length_range_high)
        .bind(margin_pct).bind(cost_base).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_policy(&row))
    }

    async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransferPricingPolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_policies WHERE organization_id=$1 AND policy_code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn get_policy_by_id(&self, id: Uuid) -> AtlasResult<Option<TransferPricingPolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_policies WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn list_policies(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<TransferPricingPolicy>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.transfer_pricing_policies WHERE organization_id=$1 AND status=$2 ORDER BY policy_code"
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.transfer_pricing_policies WHERE organization_id=$1 ORDER BY policy_code"
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_policy(&r)).collect())
    }

    async fn update_policy_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<TransferPricingPolicy> {
        let row = sqlx::query(
            r#"UPDATE _atlas.transfer_pricing_policies SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $2='active' AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_policy(&row))
    }

    async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.transfer_pricing_policies WHERE organization_id=$1 AND policy_code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    async fn create_transaction(
        &self, org_id: Uuid, transaction_number: &str, policy_id: Option<Uuid>,
        from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        item_id: Option<Uuid>, item_code: Option<&str>, item_description: Option<&str>,
        quantity: &str, unit_cost: &str, transfer_price: &str,
        total_amount: &str, currency_code: &str, transaction_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        margin_applied: Option<&str>, margin_amount: Option<&str>,
        is_arm_length_compliant: Option<bool>, compliance_notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPriceTransaction> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transfer_pricing_transactions
                (organization_id, transaction_number, policy_id,
                 from_entity_id, from_entity_name, to_entity_id, to_entity_name,
                 item_id, item_code, item_description,
                 quantity, unit_cost, transfer_price, total_amount,
                 currency_code, transaction_date,
                 source_type, source_id, source_number,
                 margin_applied, margin_amount,
                 is_arm_length_compliant, compliance_notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                    $11::numeric,$12::numeric,$13::numeric,$14::numeric,
                    $15,$16,$17,$18,$19,
                    $20::numeric,$21::numeric,
                    $22,$23,$24)
            RETURNING *"#,
        )
        .bind(org_id).bind(transaction_number).bind(policy_id)
        .bind(from_entity_id).bind(from_entity_name)
        .bind(to_entity_id).bind(to_entity_name)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(quantity).bind(unit_cost).bind(transfer_price).bind(total_amount)
        .bind(currency_code).bind(transaction_date)
        .bind(source_type).bind(source_id).bind(source_number)
        .bind(margin_applied).bind(margin_amount)
        .bind(is_arm_length_compliant).bind(compliance_notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<TransferPriceTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_transactions WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn list_transactions(&self, org_id: Uuid, status: Option<&str>, policy_id: Option<Uuid>) -> AtlasResult<Vec<TransferPriceTransaction>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.transfer_pricing_transactions
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::uuid IS NULL OR policy_id=$3)
            ORDER BY transaction_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(policy_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_transaction(&r)).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, submitted_at: Option<DateTime<Utc>>, approved_by: Option<Uuid>) -> AtlasResult<TransferPriceTransaction> {
        let row = sqlx::query(
            r#"UPDATE _atlas.transfer_pricing_transactions SET status=$2,
                submitted_at=COALESCE($3, submitted_at),
                approved_by=COALESCE($4, approved_by),
                approved_at=CASE WHEN $2='approved' AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(submitted_at).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transaction(&row))
    }

    // ========================================================================
    // Benchmarks
    // ========================================================================

    async fn create_benchmark(
        &self, org_id: Uuid, study_number: &str, title: &str, description: Option<&str>,
        policy_id: Option<Uuid>, analysis_method: &str, fiscal_year: Option<i32>,
        from_entity_id: Option<Uuid>, from_entity_name: Option<&str>,
        to_entity_id: Option<Uuid>, to_entity_name: Option<&str>,
        product_category: Option<&str>, tested_party: Option<&str>,
        prepared_by: Option<Uuid>, prepared_by_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BenchmarkStudy> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transfer_pricing_benchmarks
                (organization_id, study_number, title, description, policy_id,
                 analysis_method, fiscal_year,
                 from_entity_id, from_entity_name, to_entity_id, to_entity_name,
                 product_category, tested_party,
                 prepared_by, prepared_by_name, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            RETURNING *"#,
        )
        .bind(org_id).bind(study_number).bind(title).bind(description).bind(policy_id)
        .bind(analysis_method).bind(fiscal_year)
        .bind(from_entity_id).bind(from_entity_name)
        .bind(to_entity_id).bind(to_entity_name)
        .bind(product_category).bind(tested_party)
        .bind(prepared_by).bind(prepared_by_name).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_benchmark(&row))
    }

    async fn get_benchmark(&self, id: Uuid) -> AtlasResult<Option<BenchmarkStudy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_benchmarks WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_benchmark(&r)))
    }

    async fn list_benchmarks(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BenchmarkStudy>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.transfer_pricing_benchmarks WHERE organization_id=$1 AND status=$2 ORDER BY created_at DESC"
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.transfer_pricing_benchmarks WHERE organization_id=$1 ORDER BY created_at DESC"
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_benchmark(&r)).collect())
    }

    async fn update_benchmark_status(&self, id: Uuid, status: &str, reviewed_by: Option<Uuid>, reviewed_by_name: Option<&str>) -> AtlasResult<BenchmarkStudy> {
        let row = sqlx::query(
            r#"UPDATE _atlas.transfer_pricing_benchmarks SET status=$2,
                reviewed_by=COALESCE($3, reviewed_by),
                reviewed_by_name=COALESCE($4, reviewed_by_name),
                approved_at=CASE WHEN $2='approved' AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(reviewed_by).bind(reviewed_by_name)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_benchmark(&row))
    }

    async fn delete_benchmark(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.transfer_pricing_benchmarks WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Comparables
    // ========================================================================

    async fn add_comparable(
        &self, org_id: Uuid, benchmark_id: Uuid, comparable_number: i32,
        company_name: &str, country: Option<&str>,
        industry_code: Option<&str>, industry_description: Option<&str>,
        fiscal_year: Option<i32>, revenue: Option<&str>,
        operating_income: Option<&str>, operating_margin_pct: Option<&str>,
        net_income: Option<&str>, total_assets: Option<&str>,
        employees: Option<i32>, data_source: Option<&str>,
    ) -> AtlasResult<BenchmarkComparable> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transfer_pricing_comparables
                (organization_id, benchmark_id, comparable_number, company_name,
                 country, industry_code, industry_description, fiscal_year,
                 revenue, operating_income, operating_margin_pct,
                 net_income, total_assets, employees, data_source)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,$11::numeric,$12::numeric,$13::numeric,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(benchmark_id).bind(comparable_number).bind(company_name)
        .bind(country).bind(industry_code).bind(industry_description).bind(fiscal_year)
        .bind(revenue).bind(operating_income).bind(operating_margin_pct)
        .bind(net_income).bind(total_assets).bind(employees).bind(data_source)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_comparable(&row))
    }

    async fn list_comparables(&self, benchmark_id: Uuid) -> AtlasResult<Vec<BenchmarkComparable>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_comparables WHERE benchmark_id=$1 ORDER BY comparable_number"
        )
        .bind(benchmark_id)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_comparable(&r)).collect())
    }

    async fn update_comparable_inclusion(&self, id: Uuid, included: bool, reason: Option<&str>) -> AtlasResult<BenchmarkComparable> {
        let row = sqlx::query(
            r#"UPDATE _atlas.transfer_pricing_comparables SET
                is_included=$2, exclusion_reason=$3,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(included).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_comparable(&row))
    }

    // ========================================================================
    // Documentation
    // ========================================================================

    async fn create_documentation(
        &self, org_id: Uuid, doc_number: &str, title: &str, doc_type: &str,
        fiscal_year: i32, country: Option<&str>,
        reporting_entity_id: Option<Uuid>, reporting_entity_name: Option<&str>,
        description: Option<&str>, content_summary: Option<&str>,
        filing_deadline: Option<chrono::NaiveDate>, responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransferPricingDocumentation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transfer_pricing_documentation
                (organization_id, doc_number, title, doc_type, fiscal_year,
                 country, reporting_entity_id, reporting_entity_name,
                 description, content_summary, filing_deadline, responsible_party, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(doc_number).bind(title).bind(doc_type).bind(fiscal_year)
        .bind(country).bind(reporting_entity_id).bind(reporting_entity_name)
        .bind(description).bind(content_summary).bind(filing_deadline).bind(responsible_party)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_documentation(&row))
    }

    async fn get_documentation(&self, id: Uuid) -> AtlasResult<Option<TransferPricingDocumentation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transfer_pricing_documentation WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_documentation(&r)))
    }

    async fn list_documentation(&self, org_id: Uuid, doc_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<TransferPricingDocumentation>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.transfer_pricing_documentation
            WHERE organization_id=$1 AND ($2::text IS NULL OR doc_type=$2)
            AND ($3::text IS NULL OR status=$3)
            ORDER BY fiscal_year DESC, created_at DESC"#,
        )
        .bind(org_id).bind(doc_type).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_documentation(&r)).collect())
    }

    async fn update_documentation_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, filed_at: Option<DateTime<Utc>>) -> AtlasResult<TransferPricingDocumentation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.transfer_pricing_documentation SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $2='approved' AND approved_at IS NULL THEN now() ELSE approved_at END,
                filed_at=CASE WHEN $2='filed' THEN COALESCE($4, now()) ELSE filed_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by).bind(filed_at)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_documentation(&row))
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransferPricingDashboard> {
        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_policies WHERE organization_id=$1) as total_policies,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_policies WHERE organization_id=$1 AND status='active') as active_policies,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_transactions WHERE organization_id=$1) as total_transactions,
                (SELECT COALESCE(SUM(total_amount),0) FROM _atlas.transfer_pricing_transactions WHERE organization_id=$1) as total_transaction_value,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_transactions WHERE organization_id=$1 AND status IN ('draft','submitted')) as pending_transactions,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_transactions WHERE organization_id=$1 AND is_arm_length_compliant=false) as non_compliant_transactions,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_benchmarks WHERE organization_id=$1) as total_benchmarks,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_benchmarks WHERE organization_id=$1 AND status='approved') as active_benchmarks,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_benchmarks WHERE organization_id=$1 AND is_within_range=true) as benchmarks_within_range,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_documentation WHERE organization_id=$1) as total_documentation,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_documentation WHERE organization_id=$1 AND status IN ('draft','in_review','approved')) as pending_filings,
                (SELECT COUNT(*) FROM _atlas.transfer_pricing_documentation WHERE organization_id=$1 AND status != 'filed' AND filing_deadline < now()::date) as overdue_filings"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let total_policies: i64 = row.try_get("total_policies").unwrap_or(0);
        let active_policies: i64 = row.try_get("active_policies").unwrap_or(0);
        let total_transactions: i64 = row.try_get("total_transactions").unwrap_or(0);
        let total_tx_value: serde_json::Value = row.try_get("total_transaction_value").unwrap_or(serde_json::json!("0"));
        let pending_transactions: i64 = row.try_get("pending_transactions").unwrap_or(0);
        let non_compliant: i64 = row.try_get("non_compliant_transactions").unwrap_or(0);
        let total_benchmarks: i64 = row.try_get("total_benchmarks").unwrap_or(0);
        let active_benchmarks: i64 = row.try_get("active_benchmarks").unwrap_or(0);
        let within_range: i64 = row.try_get("benchmarks_within_range").unwrap_or(0);
        let total_docs: i64 = row.try_get("total_documentation").unwrap_or(0);
        let pending_filings: i64 = row.try_get("pending_filings").unwrap_or(0);
        let overdue_filings: i64 = row.try_get("overdue_filings").unwrap_or(0);

        let total_with_compliance = total_transactions - non_compliant;
        let compliance_rate = if total_transactions > 0 {
            (total_with_compliance as f64 / total_transactions as f64) * 100.0
        } else {
            0.0
        };

        Ok(TransferPricingDashboard {
            total_policies: total_policies as i32,
            active_policies: active_policies as i32,
            total_transactions: total_transactions as i32,
            total_transaction_value: total_tx_value.to_string(),
            pending_transactions: pending_transactions as i32,
            non_compliant_transactions: non_compliant as i32,
            compliance_rate_pct: format!("{:.1}", compliance_rate),
            total_benchmarks: total_benchmarks as i32,
            active_benchmarks: active_benchmarks as i32,
            benchmarks_within_range: within_range as i32,
            total_documentation: total_docs as i32,
            pending_filings: pending_filings as i32,
            overdue_filings: overdue_filings as i32,
            transactions_by_method: serde_json::json!({}),
            transactions_by_status: serde_json::json!({}),
        })
    }
}
