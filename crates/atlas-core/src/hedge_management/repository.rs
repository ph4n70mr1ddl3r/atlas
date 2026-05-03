//! Hedge Management Repository
//!
//! PostgreSQL storage for derivative instruments, hedge relationships,
//! effectiveness tests, and hedge documentation.

use atlas_shared::{
    DerivativeInstrument, HedgeRelationship, HedgeEffectivenessTest,
    HedgeDocumentation, HedgeDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde_json::Value;

/// Repository trait for hedge management data storage
#[async_trait]
pub trait HedgeManagementRepository: Send + Sync {
    // Derivative Instruments
    async fn create_derivative(&self, derivative: &DerivativeCreateParams) -> AtlasResult<DerivativeInstrument>;
    async fn get_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<Option<DerivativeInstrument>>;
    async fn get_derivative_by_id(&self, id: Uuid) -> AtlasResult<Option<DerivativeInstrument>>;
    async fn list_derivatives(&self, org_id: Uuid, status: Option<&str>, instrument_type: Option<&str>) -> AtlasResult<Vec<DerivativeInstrument>>;
    async fn update_derivative_status(&self, id: Uuid, status: &str) -> AtlasResult<DerivativeInstrument>;
    async fn update_derivative_valuation(&self, id: Uuid, fair_value: &str, unrealized_gl: &str, valuation_method: Option<&str>, valuation_date: Option<chrono::NaiveDate>) -> AtlasResult<DerivativeInstrument>;
    async fn delete_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<()>;
    async fn get_latest_derivative_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Hedge Relationships
    async fn create_hedge_relationship(&self, params: &HedgeRelationshipCreateParams) -> AtlasResult<HedgeRelationship>;
    async fn get_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<Option<HedgeRelationship>>;
    async fn get_hedge_relationship_by_id(&self, id: Uuid) -> AtlasResult<Option<HedgeRelationship>>;
    async fn list_hedge_relationships(&self, org_id: Uuid, status: Option<&str>, hedge_type: Option<&str>) -> AtlasResult<Vec<HedgeRelationship>>;
    async fn update_hedge_relationship_status(&self, id: Uuid, status: &str) -> AtlasResult<HedgeRelationship>;
    async fn update_hedge_effectiveness(&self, id: Uuid, test_date: chrono::NaiveDate, result: &str) -> AtlasResult<HedgeRelationship>;
    async fn delete_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<()>;
    async fn get_latest_hedge_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Effectiveness Tests
    async fn create_effectiveness_test(&self, params: &EffectivenessTestCreateParams) -> AtlasResult<HedgeEffectivenessTest>;
    async fn get_effectiveness_test(&self, id: Uuid) -> AtlasResult<Option<HedgeEffectivenessTest>>;
    async fn list_effectiveness_tests(&self, hedge_relationship_id: Uuid) -> AtlasResult<Vec<HedgeEffectivenessTest>>;
    async fn update_effectiveness_test_status(&self, id: Uuid, status: &str) -> AtlasResult<HedgeEffectivenessTest>;
    async fn get_latest_test_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Documentation
    async fn create_hedge_documentation(&self, params: &DocumentationCreateParams) -> AtlasResult<HedgeDocumentation>;
    async fn get_hedge_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<Option<HedgeDocumentation>>;
    async fn get_hedge_documentation_by_id(&self, id: Uuid) -> AtlasResult<Option<HedgeDocumentation>>;
    async fn list_hedge_documentation(&self, org_id: Uuid, hedge_relationship_id: Option<Uuid>) -> AtlasResult<Vec<HedgeDocumentation>>;
    async fn update_documentation_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<HedgeDocumentation>;
    async fn delete_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<HedgeDashboard>;
}

/// Parameters for creating a derivative instrument
pub struct DerivativeCreateParams {
    pub org_id: Uuid,
    pub instrument_number: String,
    pub instrument_type: String,
    pub underlying_type: String,
    pub underlying_description: Option<String>,
    pub currency_code: String,
    pub counter_currency_code: Option<String>,
    pub notional_amount: String,
    pub strike_rate: Option<String>,
    pub forward_rate: Option<String>,
    pub spot_rate: Option<String>,
    pub option_type: Option<String>,
    pub premium_amount: Option<String>,
    pub trade_date: Option<chrono::NaiveDate>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub settlement_type: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_reference: Option<String>,
    pub portfolio_code: Option<String>,
    pub trading_book: Option<String>,
    pub accounting_treatment: Option<String>,
    pub risk_factor: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a hedge relationship
pub struct HedgeRelationshipCreateParams {
    pub org_id: Uuid,
    pub hedge_id: String,
    pub hedge_type: String,
    pub derivative_id: Option<Uuid>,
    pub derivative_number: Option<String>,
    pub hedged_item_description: Option<String>,
    pub hedged_item_id: Option<Uuid>,
    pub hedged_risk: String,
    pub hedge_strategy: Option<String>,
    pub hedged_item_reference: Option<String>,
    pub hedged_item_currency: Option<String>,
    pub hedged_amount: String,
    pub hedge_ratio: Option<String>,
    pub designated_start_date: Option<chrono::NaiveDate>,
    pub designated_end_date: Option<chrono::NaiveDate>,
    pub effectiveness_method: String,
    pub critical_terms_match: Option<String>,
    pub hedge_documentation_ref: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating an effectiveness test
pub struct EffectivenessTestCreateParams {
    pub org_id: Uuid,
    pub hedge_relationship_id: Uuid,
    pub hedge_id: Option<String>,
    pub test_type: String,
    pub effectiveness_method: String,
    pub test_date: chrono::NaiveDate,
    pub test_period_start: Option<chrono::NaiveDate>,
    pub test_period_end: Option<chrono::NaiveDate>,
    pub derivative_fair_value_change: Option<String>,
    pub hedged_item_fair_value_change: Option<String>,
    pub hedge_ratio_result: Option<String>,
    pub ratio_lower_bound: Option<String>,
    pub ratio_upper_bound: Option<String>,
    pub effectiveness_result: String,
    pub ineffective_amount: Option<String>,
    pub cumulative_gain_loss: Option<String>,
    pub regression_r_squared: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating hedge documentation
pub struct DocumentationCreateParams {
    pub org_id: Uuid,
    pub hedge_relationship_id: Option<Uuid>,
    pub hedge_id: Option<String>,
    pub document_number: String,
    pub hedge_type: String,
    pub risk_management_objective: Option<String>,
    pub hedging_strategy_description: Option<String>,
    pub hedged_item_description: Option<String>,
    pub hedged_risk_description: Option<String>,
    pub derivative_description: Option<String>,
    pub effectiveness_method_description: Option<String>,
    pub assessment_frequency: Option<String>,
    pub designation_date: Option<chrono::NaiveDate>,
    pub documentation_date: Option<chrono::NaiveDate>,
    pub prepared_by: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

// Helper functions for reading NUMERIC columns as text
fn get_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<String, _>(col).unwrap_or_else(|_| "0.00".to_string())
}

fn get_optional_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(col).unwrap_or(None)
}

fn row_to_derivative(row: &sqlx::postgres::PgRow) -> DerivativeInstrument {
    DerivativeInstrument {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        instrument_number: row.get("instrument_number"),
        instrument_type: row.get("instrument_type"),
        underlying_type: row.get("underlying_type"),
        underlying_description: row.get("underlying_description"),
        currency_code: row.get("currency_code"),
        counter_currency_code: row.get("counter_currency_code"),
        notional_amount: get_numeric_text(row, "notional_amount"),
        strike_rate: row.get("strike_rate"),
        forward_rate: row.get("forward_rate"),
        spot_rate: row.get("spot_rate"),
        option_type: row.get("option_type"),
        premium_amount: get_optional_numeric_text(row, "premium_amount"),
        trade_date: row.get("trade_date"),
        effective_date: row.get("effective_date"),
        maturity_date: row.get("maturity_date"),
        settlement_date: row.get("settlement_date"),
        settlement_type: row.get("settlement_type"),
        counterparty_name: row.get("counterparty_name"),
        counterparty_reference: row.get("counterparty_reference"),
        portfolio_code: row.get("portfolio_code"),
        trading_book: row.get("trading_book"),
        accounting_treatment: row.get("accounting_treatment"),
        fair_value: get_optional_numeric_text(row, "fair_value"),
        unrealized_gain_loss: get_optional_numeric_text(row, "unrealized_gain_loss"),
        realized_gain_loss: get_optional_numeric_text(row, "realized_gain_loss"),
        valuation_method: row.get("valuation_method"),
        last_valuation_date: row.get("last_valuation_date"),
        risk_factor: row.get("risk_factor"),
        status: row.get("status"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_hedge_relationship(row: &sqlx::postgres::PgRow) -> HedgeRelationship {
    HedgeRelationship {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        hedge_id: row.get("hedge_id"),
        hedge_type: row.get("hedge_type"),
        derivative_id: row.get("derivative_id"),
        derivative_number: row.get("derivative_number"),
        hedged_item_description: row.get("hedged_item_description"),
        hedged_item_id: row.get("hedged_item_id"),
        hedged_risk: row.get("hedged_risk"),
        hedge_strategy: row.get("hedge_strategy"),
        hedged_item_reference: row.get("hedged_item_reference"),
        hedged_item_currency: row.get("hedged_item_currency"),
        hedged_amount: get_numeric_text(row, "hedged_amount"),
        hedge_ratio: row.get("hedge_ratio"),
        designated_start_date: row.get("designated_start_date"),
        designated_end_date: row.get("designated_end_date"),
        effectiveness_method: row.get("effectiveness_method"),
        critical_terms_match: row.get("critical_terms_match"),
        prospective_effective: row.get("prospective_effective"),
        retrospective_effective: row.get("retrospective_effective"),
        hedge_documentation_ref: row.get("hedge_documentation_ref"),
        status: row.get("status"),
        last_effectiveness_test_date: row.get("last_effectiveness_test_date"),
        last_effectiveness_result: row.get("last_effectiveness_result"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_effectiveness_test(row: &sqlx::postgres::PgRow) -> HedgeEffectivenessTest {
    HedgeEffectivenessTest {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        hedge_relationship_id: row.get("hedge_relationship_id"),
        hedge_id: row.get("hedge_id"),
        test_type: row.get("test_type"),
        effectiveness_method: row.get("effectiveness_method"),
        test_date: row.get("test_date"),
        test_period_start: row.get("test_period_start"),
        test_period_end: row.get("test_period_end"),
        derivative_fair_value_change: get_optional_numeric_text(row, "derivative_fair_value_change"),
        hedged_item_fair_value_change: get_optional_numeric_text(row, "hedged_item_fair_value_change"),
        hedge_ratio_result: row.get("hedge_ratio_result"),
        ratio_lower_bound: row.get("ratio_lower_bound"),
        ratio_upper_bound: row.get("ratio_upper_bound"),
        effectiveness_result: row.get("effectiveness_result"),
        ineffective_amount: get_optional_numeric_text(row, "ineffective_amount"),
        cumulative_gain_loss: get_optional_numeric_text(row, "cumulative_gain_loss"),
        regression_r_squared: row.get("regression_r_squared"),
        notes: row.get("notes"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_documentation(row: &sqlx::postgres::PgRow) -> HedgeDocumentation {
    HedgeDocumentation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        hedge_relationship_id: row.get("hedge_relationship_id"),
        hedge_id: row.get("hedge_id"),
        document_number: row.get("document_number"),
        hedge_type: row.get("hedge_type"),
        risk_management_objective: row.get("risk_management_objective"),
        hedging_strategy_description: row.get("hedging_strategy_description"),
        hedged_item_description: row.get("hedged_item_description"),
        hedged_risk_description: row.get("hedged_risk_description"),
        derivative_description: row.get("derivative_description"),
        effectiveness_method_description: row.get("effectiveness_method_description"),
        assessment_frequency: row.get("assessment_frequency"),
        designation_date: row.get("designation_date"),
        documentation_date: row.get("documentation_date"),
        approval_date: row.get("approval_date"),
        approved_by: row.get("approved_by"),
        prepared_by: row.get("prepared_by"),
        status: row.get("status"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

/// PostgreSQL implementation
pub struct PostgresHedgeManagementRepository {
    pool: PgPool,
}

impl PostgresHedgeManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HedgeManagementRepository for PostgresHedgeManagementRepository {
    // ========================================================================
    // Derivative Instruments
    // ========================================================================

    async fn create_derivative(&self, p: &DerivativeCreateParams) -> AtlasResult<DerivativeInstrument> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.derivative_instruments
                (organization_id, instrument_number, instrument_type, underlying_type,
                 underlying_description, currency_code, counter_currency_code,
                 notional_amount, strike_rate, forward_rate, spot_rate,
                 option_type, premium_amount, trade_date, effective_date,
                 maturity_date, settlement_date, settlement_type,
                 counterparty_name, counterparty_reference, portfolio_code,
                 trading_book, accounting_treatment, risk_factor, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(&p.instrument_number).bind(&p.instrument_type).bind(&p.underlying_type)
        .bind(&p.underlying_description).bind(&p.currency_code).bind(&p.counter_currency_code)
        .bind(&p.notional_amount).bind(&p.strike_rate).bind(&p.forward_rate).bind(&p.spot_rate)
        .bind(&p.option_type).bind(&p.premium_amount).bind(p.trade_date).bind(p.effective_date)
        .bind(p.maturity_date).bind(p.settlement_date).bind(&p.settlement_type)
        .bind(&p.counterparty_name).bind(&p.counterparty_reference).bind(&p.portfolio_code)
        .bind(&p.trading_book).bind(&p.accounting_treatment).bind(&p.risk_factor).bind(&p.notes).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_derivative(&row))
    }

    async fn get_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<Option<DerivativeInstrument>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.derivative_instruments WHERE organization_id=$1 AND instrument_number=$2"
        )
        .bind(org_id).bind(instrument_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_derivative(&r)))
    }

    async fn get_derivative_by_id(&self, id: Uuid) -> AtlasResult<Option<DerivativeInstrument>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.derivative_instruments WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_derivative(&r)))
    }

    async fn list_derivatives(&self, org_id: Uuid, status: Option<&str>, instrument_type: Option<&str>) -> AtlasResult<Vec<DerivativeInstrument>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.derivative_instruments
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::text IS NULL OR instrument_type=$3)
            ORDER BY instrument_number"#,
        )
        .bind(org_id).bind(status).bind(instrument_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_derivative).collect())
    }

    async fn update_derivative_status(&self, id: Uuid, status: &str) -> AtlasResult<DerivativeInstrument> {
        let row = sqlx::query(
            "UPDATE _atlas.derivative_instruments SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_derivative(&row))
    }

    async fn update_derivative_valuation(&self, id: Uuid, fair_value: &str, unrealized_gl: &str, valuation_method: Option<&str>, valuation_date: Option<chrono::NaiveDate>) -> AtlasResult<DerivativeInstrument> {
        let row = sqlx::query(
            r#"UPDATE _atlas.derivative_instruments
            SET fair_value=$2, unrealized_gain_loss=$3,
                valuation_method=COALESCE($4, valuation_method),
                last_valuation_date=COALESCE($5, last_valuation_date),
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(fair_value).bind(unrealized_gl).bind(valuation_method).bind(valuation_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_derivative(&row))
    }

    async fn delete_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.derivative_instruments WHERE organization_id=$1 AND instrument_number=$2"
        )
        .bind(org_id).bind(instrument_number)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_latest_derivative_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.derivative_instruments WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count as i32)
    }

    // ========================================================================
    // Hedge Relationships
    // ========================================================================

    async fn create_hedge_relationship(&self, p: &HedgeRelationshipCreateParams) -> AtlasResult<HedgeRelationship> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.hedge_relationships
                (organization_id, hedge_id, hedge_type, derivative_id, derivative_number,
                 hedged_item_description, hedged_item_id, hedged_risk, hedge_strategy,
                 hedged_item_reference, hedged_item_currency, hedged_amount, hedge_ratio,
                 designated_start_date, designated_end_date, effectiveness_method,
                 critical_terms_match, hedge_documentation_ref, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(&p.hedge_id).bind(&p.hedge_type)
        .bind(p.derivative_id).bind(&p.derivative_number)
        .bind(&p.hedged_item_description).bind(p.hedged_item_id).bind(&p.hedged_risk)
        .bind(&p.hedge_strategy).bind(&p.hedged_item_reference).bind(&p.hedged_item_currency)
        .bind(&p.hedged_amount).bind(&p.hedge_ratio)
        .bind(p.designated_start_date).bind(p.designated_end_date).bind(&p.effectiveness_method)
        .bind(&p.critical_terms_match).bind(&p.hedge_documentation_ref).bind(&p.notes).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_hedge_relationship(&row))
    }

    async fn get_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<Option<HedgeRelationship>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.hedge_relationships WHERE organization_id=$1 AND hedge_id=$2"
        )
        .bind(org_id).bind(hedge_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_hedge_relationship(&r)))
    }

    async fn get_hedge_relationship_by_id(&self, id: Uuid) -> AtlasResult<Option<HedgeRelationship>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.hedge_relationships WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_hedge_relationship(&r)))
    }

    async fn list_hedge_relationships(&self, org_id: Uuid, status: Option<&str>, hedge_type: Option<&str>) -> AtlasResult<Vec<HedgeRelationship>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.hedge_relationships
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::text IS NULL OR hedge_type=$3)
            ORDER BY hedge_id"#,
        )
        .bind(org_id).bind(status).bind(hedge_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_hedge_relationship).collect())
    }

    async fn update_hedge_relationship_status(&self, id: Uuid, status: &str) -> AtlasResult<HedgeRelationship> {
        let row = sqlx::query(
            "UPDATE _atlas.hedge_relationships SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_hedge_relationship(&row))
    }

    async fn update_hedge_effectiveness(&self, id: Uuid, test_date: chrono::NaiveDate, result: &str) -> AtlasResult<HedgeRelationship> {
        let row = sqlx::query(
            r#"UPDATE _atlas.hedge_relationships SET last_effectiveness_test_date=$2,
                last_effectiveness_result=$3, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(test_date).bind(result)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_hedge_relationship(&row))
    }

    async fn delete_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.hedge_relationships WHERE organization_id=$1 AND hedge_id=$2"
        )
        .bind(org_id).bind(hedge_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_latest_hedge_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.hedge_relationships WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count as i32)
    }

    // ========================================================================
    // Effectiveness Tests
    // ========================================================================

    async fn create_effectiveness_test(&self, p: &EffectivenessTestCreateParams) -> AtlasResult<HedgeEffectivenessTest> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.hedge_effectiveness_tests
                (organization_id, hedge_relationship_id, hedge_id, test_type,
                 effectiveness_method, test_date, test_period_start, test_period_end,
                 derivative_fair_value_change, hedged_item_fair_value_change,
                 hedge_ratio_result, ratio_lower_bound, ratio_upper_bound,
                 effectiveness_result, ineffective_amount, cumulative_gain_loss,
                 regression_r_squared, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(p.hedge_relationship_id).bind(&p.hedge_id).bind(&p.test_type)
        .bind(&p.effectiveness_method).bind(p.test_date).bind(p.test_period_start).bind(p.test_period_end)
        .bind(&p.derivative_fair_value_change).bind(&p.hedged_item_fair_value_change)
        .bind(&p.hedge_ratio_result).bind(&p.ratio_lower_bound).bind(&p.ratio_upper_bound)
        .bind(&p.effectiveness_result).bind(&p.ineffective_amount).bind(&p.cumulative_gain_loss)
        .bind(&p.regression_r_squared).bind(&p.notes).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_effectiveness_test(&row))
    }

    async fn get_effectiveness_test(&self, id: Uuid) -> AtlasResult<Option<HedgeEffectivenessTest>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.hedge_effectiveness_tests WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_effectiveness_test(&r)))
    }

    async fn list_effectiveness_tests(&self, hedge_relationship_id: Uuid) -> AtlasResult<Vec<HedgeEffectivenessTest>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.hedge_effectiveness_tests WHERE hedge_relationship_id=$1 ORDER BY test_date DESC"
        )
        .bind(hedge_relationship_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_effectiveness_test).collect())
    }

    async fn update_effectiveness_test_status(&self, id: Uuid, status: &str) -> AtlasResult<HedgeEffectivenessTest> {
        let row = sqlx::query(
            "UPDATE _atlas.hedge_effectiveness_tests SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_effectiveness_test(&row))
    }

    async fn get_latest_test_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.hedge_documentation WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let count: i64 = row.try_get("cnt").unwrap_or(0);
        Ok(count as i32)
    }

    // ========================================================================
    // Documentation
    // ========================================================================

    async fn create_hedge_documentation(&self, p: &DocumentationCreateParams) -> AtlasResult<HedgeDocumentation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.hedge_documentation
                (organization_id, hedge_relationship_id, hedge_id, document_number,
                 hedge_type, risk_management_objective, hedging_strategy_description,
                 hedged_item_description, hedged_risk_description,
                 derivative_description, effectiveness_method_description,
                 assessment_frequency, designation_date, documentation_date,
                 prepared_by, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(p.hedge_relationship_id).bind(&p.hedge_id).bind(&p.document_number)
        .bind(&p.hedge_type).bind(&p.risk_management_objective).bind(&p.hedging_strategy_description)
        .bind(&p.hedged_item_description).bind(&p.hedged_risk_description)
        .bind(&p.derivative_description).bind(&p.effectiveness_method_description)
        .bind(&p.assessment_frequency).bind(p.designation_date).bind(p.documentation_date)
        .bind(&p.prepared_by).bind(&p.notes).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_documentation(&row))
    }

    async fn get_hedge_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<Option<HedgeDocumentation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.hedge_documentation WHERE organization_id=$1 AND document_number=$2"
        )
        .bind(org_id).bind(document_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_documentation(&r)))
    }

    async fn get_hedge_documentation_by_id(&self, id: Uuid) -> AtlasResult<Option<HedgeDocumentation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.hedge_documentation WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_documentation(&r)))
    }

    async fn list_hedge_documentation(&self, org_id: Uuid, hedge_relationship_id: Option<Uuid>) -> AtlasResult<Vec<HedgeDocumentation>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.hedge_documentation
            WHERE organization_id=$1 AND ($2::uuid IS NULL OR hedge_relationship_id=$2)
            ORDER BY document_number"#,
        )
        .bind(org_id).bind(hedge_relationship_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_documentation).collect())
    }

    async fn update_documentation_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<HedgeDocumentation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.hedge_documentation SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approval_date=CASE WHEN $3 IS NOT NULL THEN CURRENT_DATE ELSE approval_date END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_documentation(&row))
    }

    async fn delete_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.hedge_documentation WHERE organization_id=$1 AND document_number=$2"
        )
        .bind(org_id).bind(document_number)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<HedgeDashboard> {
        // Get derivative summary
        let deriv_row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_deriv,
                COALESCE(SUM(notional_amount::numeric) FILTER (WHERE status = 'active'), 0)::text as total_notional,
                COALESCE(SUM(ABS(unrealized_gain_loss::numeric)) FILTER (WHERE status = 'active'), 0)::text as total_ugl
            FROM _atlas.derivative_instruments WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_deriv: i64 = deriv_row.try_get("active_deriv").unwrap_or(0);
        let total_notional: String = deriv_row.try_get("total_notional").unwrap_or_else(|_| "0".to_string());
        let total_ugl: String = deriv_row.try_get("total_ugl").unwrap_or_else(|_| "0".to_string());

        // Get hedge relationship summary
        let hedge_row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'active') as active_hedges,
                COALESCE(SUM(hedged_amount::numeric) FILTER (WHERE status = 'active'), 0)::text as total_hedged,
                COUNT(*) FILTER (WHERE status = 'active' AND last_effectiveness_result = 'effective') as effective,
                COUNT(*) FILTER (WHERE status = 'active' AND last_effectiveness_result = 'ineffective') as ineffective
            FROM _atlas.hedge_relationships WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_hedges: i64 = hedge_row.try_get("active_hedges").unwrap_or(0);
        let total_hedged: String = hedge_row.try_get("total_hedged").unwrap_or_else(|_| "0".to_string());
        let effective: i64 = hedge_row.try_get("effective").unwrap_or(0);
        let ineffective: i64 = hedge_row.try_get("ineffective").unwrap_or(0);

        // Get documentation pending
        let doc_row = sqlx::query(
            "SELECT COUNT(*) as pending FROM _atlas.hedge_documentation WHERE organization_id = $1 AND status = 'draft'"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let pending_doc: i64 = doc_row.try_get("pending").unwrap_or(0);

        // By instrument type
        let type_rows = sqlx::query(
            r#"SELECT instrument_type, COUNT(*) as count, COALESCE(SUM(notional_amount::numeric), 0)::text as total
            FROM _atlas.derivative_instruments WHERE organization_id = $1 AND status = 'active'
            GROUP BY instrument_type"#
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_instrument_type = serde_json::Map::new();
        for row in &type_rows {
            let itype: String = row.try_get("instrument_type").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            let total: String = row.try_get("total").unwrap_or_else(|_| "0".to_string());
            by_instrument_type.insert(itype, serde_json::json!({"count": count, "total": total}));
        }

        // By hedge type
        let hedge_type_rows = sqlx::query(
            r#"SELECT hedge_type, COUNT(*) as count, COALESCE(SUM(hedged_amount::numeric), 0)::text as total
            FROM _atlas.hedge_relationships WHERE organization_id = $1 AND status = 'active'
            GROUP BY hedge_type"#
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_hedge_type = serde_json::Map::new();
        for row in &hedge_type_rows {
            let htype: String = row.try_get("hedge_type").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            let total: String = row.try_get("total").unwrap_or_else(|_| "0".to_string());
            by_hedge_type.insert(htype, serde_json::json!({"count": count, "total": total}));
        }

        Ok(HedgeDashboard {
            total_active_derivatives: active_deriv as i32,
            total_notional_amount: total_notional,
            total_active_hedges: active_hedges as i32,
            total_hedged_amount: total_hedged,
            total_effective_hedges: effective as i32,
            total_ineffective_hedges: ineffective as i32,
            total_pending_documentation: pending_doc as i32,
            total_unrealized_gain_loss: total_ugl,
            by_instrument_type: Value::Object(by_instrument_type),
            by_hedge_type: Value::Object(by_hedge_type),
        })
    }
}
