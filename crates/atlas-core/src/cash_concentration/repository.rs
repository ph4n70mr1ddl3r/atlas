//! Cash Concentration Repository
//!
//! PostgreSQL storage for cash pools, participants, sweep rules, and sweep runs.

use atlas_shared::{
    CashPool, CashPoolParticipant, CashPoolSweepRule,
    CashPoolSweepRun, CashPoolSweepRunLine, CashPoolDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for cash concentration data storage
#[async_trait]
pub trait CashConcentrationRepository: Send + Sync {
    // Cash Pools
    async fn create_pool(&self, params: &PoolCreateParams) -> AtlasResult<CashPool>;
    async fn get_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<Option<CashPool>>;
    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPool>>;
    async fn list_pools(&self, org_id: Uuid, status: Option<&str>, pool_type: Option<&str>) -> AtlasResult<Vec<CashPool>>;
    async fn update_pool_status(&self, id: Uuid, status: &str) -> AtlasResult<CashPool>;
    async fn delete_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<()>;

    // Participants
    async fn create_participant(&self, params: &ParticipantCreateParams) -> AtlasResult<CashPoolParticipant>;
    async fn get_participant(&self, pool_id: Uuid, participant_code: &str) -> AtlasResult<Option<CashPoolParticipant>>;
    async fn get_participant_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPoolParticipant>>;
    async fn list_participants(&self, pool_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CashPoolParticipant>>;
    async fn update_participant_status(&self, id: Uuid, status: &str) -> AtlasResult<CashPoolParticipant>;
    async fn update_participant_balance(&self, id: Uuid, balance: &str) -> AtlasResult<CashPoolParticipant>;

    // Sweep Rules
    async fn create_sweep_rule(&self, params: &SweepRuleCreateParams) -> AtlasResult<CashPoolSweepRule>;
    async fn list_sweep_rules(&self, pool_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRule>>;
    async fn delete_sweep_rule(&self, pool_id: Uuid, rule_code: &str) -> AtlasResult<()>;

    // Sweep Runs
    async fn create_sweep_run(&self, params: &SweepRunCreateParams) -> AtlasResult<CashPoolSweepRun>;
    async fn get_sweep_run(&self, id: Uuid) -> AtlasResult<Option<CashPoolSweepRun>>;
    async fn list_sweep_runs(&self, pool_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRun>>;
    async fn update_sweep_run_status(&self, id: Uuid, status: &str, total_swept: Option<&str>, total_txns: Option<i32>, successful: Option<i32>, failed: Option<i32>) -> AtlasResult<CashPoolSweepRun>;
    async fn create_sweep_run_line(&self, params: &SweepRunLineCreateParams) -> AtlasResult<CashPoolSweepRunLine>;
    async fn list_sweep_run_lines(&self, sweep_run_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRunLine>>;
    async fn get_latest_run_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashPoolDashboard>;
}

/// Parameters for creating a cash pool
pub struct PoolCreateParams {
    pub org_id: Uuid,
    pub pool_code: String,
    pub pool_name: String,
    pub pool_type: String,
    pub concentration_account_id: Option<Uuid>,
    pub concentration_account_name: Option<String>,
    pub currency_code: String,
    pub sweep_frequency: Option<String>,
    pub sweep_time: Option<String>,
    pub minimum_transfer_amount: Option<String>,
    pub maximum_transfer_amount: Option<String>,
    pub target_balance: Option<String>,
    pub interest_allocation_method: Option<String>,
    pub interest_rate: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub termination_date: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a participant
pub struct ParticipantCreateParams {
    pub org_id: Uuid,
    pub pool_id: Uuid,
    pub participant_code: String,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub bank_name: Option<String>,
    pub account_number: Option<String>,
    pub participant_type: String,
    pub sweep_direction: String,
    pub priority: Option<i32>,
    pub minimum_balance: Option<String>,
    pub maximum_balance: Option<String>,
    pub threshold_amount: Option<String>,
    pub current_balance: Option<String>,
    pub entity_id: Option<Uuid>,
    pub entity_name: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a sweep rule
pub struct SweepRuleCreateParams {
    pub org_id: Uuid,
    pub pool_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub sweep_type: String,
    pub participant_id: Option<Uuid>,
    pub direction: String,
    pub trigger_condition: Option<String>,
    pub threshold_amount: Option<String>,
    pub target_balance: Option<String>,
    pub minimum_transfer: Option<String>,
    pub maximum_transfer: Option<String>,
    pub priority: Option<i32>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a sweep run
pub struct SweepRunCreateParams {
    pub org_id: Uuid,
    pub pool_id: Uuid,
    pub run_number: String,
    pub run_date: chrono::NaiveDate,
    pub run_type: String,
    pub initiated_by: Option<Uuid>,
    pub notes: Option<String>,
}

/// Parameters for creating a sweep run line
pub struct SweepRunLineCreateParams {
    pub organization_id: Uuid,
    pub sweep_run_id: Uuid,
    pub pool_id: Uuid,
    pub participant_id: Uuid,
    pub participant_code: Option<String>,
    pub bank_account_name: Option<String>,
    pub sweep_rule_id: Option<Uuid>,
    pub direction: String,
    pub pre_sweep_balance: Option<String>,
    pub sweep_amount: String,
    pub post_sweep_balance: Option<String>,
    pub status: String,
}

// Helper functions
fn get_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<f64, _>(col)
        .map(|v| format!("{:.2}", v))
        .unwrap_or_else(|_| "0.00".to_string())
}

fn get_optional_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
    row.try_get::<Option<f64>, _>(col)
        .ok()
        .flatten()
        .map(|v| format!("{:.2}", v))
}

fn row_to_pool(row: &sqlx::postgres::PgRow) -> CashPool {
    CashPool {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        pool_code: row.get("pool_code"),
        pool_name: row.get("pool_name"),
        pool_type: row.get("pool_type"),
        concentration_account_id: row.get("concentration_account_id"),
        concentration_account_name: row.get("concentration_account_name"),
        currency_code: row.get("currency_code"),
        status: row.get("status"),
        effective_date: row.get("effective_date"),
        termination_date: row.get("termination_date"),
        sweep_frequency: row.get("sweep_frequency"),
        sweep_time: row.get("sweep_time"),
        minimum_transfer_amount: get_optional_numeric_text(row, "minimum_transfer_amount"),
        maximum_transfer_amount: get_optional_numeric_text(row, "maximum_transfer_amount"),
        target_balance: get_optional_numeric_text(row, "target_balance"),
        interest_allocation_method: row.get("interest_allocation_method"),
        interest_rate: get_optional_numeric_text(row, "interest_rate"),
        description: row.get("description"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_participant(row: &sqlx::postgres::PgRow) -> CashPoolParticipant {
    CashPoolParticipant {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        pool_id: row.get("pool_id"),
        participant_code: row.get("participant_code"),
        bank_account_id: row.get("bank_account_id"),
        bank_account_name: row.get("bank_account_name"),
        bank_name: row.get("bank_name"),
        account_number: row.get("account_number"),
        participant_type: row.get("participant_type"),
        sweep_direction: row.get("sweep_direction"),
        priority: row.try_get("priority").ok(),
        minimum_balance: Some(get_numeric_text(row, "minimum_balance")),
        maximum_balance: get_optional_numeric_text(row, "maximum_balance"),
        threshold_amount: Some(get_numeric_text(row, "threshold_amount")),
        current_balance: Some(get_numeric_text(row, "current_balance")),
        status: row.get("status"),
        effective_date: row.get("effective_date"),
        termination_date: row.get("termination_date"),
        entity_id: row.get("entity_id"),
        entity_name: row.get("entity_name"),
        description: row.get("description"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_sweep_rule(row: &sqlx::postgres::PgRow) -> CashPoolSweepRule {
    CashPoolSweepRule {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        pool_id: row.get("pool_id"),
        rule_code: row.get("rule_code"),
        rule_name: row.get("rule_name"),
        sweep_type: row.get("sweep_type"),
        participant_id: row.get("participant_id"),
        direction: row.get("direction"),
        trigger_condition: row.get("trigger_condition"),
        threshold_amount: get_optional_numeric_text(row, "threshold_amount"),
        target_balance: get_optional_numeric_text(row, "target_balance"),
        minimum_transfer: get_optional_numeric_text(row, "minimum_transfer"),
        maximum_transfer: get_optional_numeric_text(row, "maximum_transfer"),
        priority: row.try_get("priority").ok(),
        is_active: row.try_get("is_active").ok(),
        effective_date: row.get("effective_date"),
        termination_date: row.get("termination_date"),
        description: row.get("description"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
    }
}

fn row_to_sweep_run(row: &sqlx::postgres::PgRow) -> CashPoolSweepRun {
    CashPoolSweepRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        pool_id: row.get("pool_id"),
        run_number: row.get("run_number"),
        run_date: row.get("run_date"),
        run_type: row.get("run_type"),
        status: row.get("status"),
        total_swept_amount: Some(get_numeric_text(row, "total_swept_amount")),
        total_transactions: row.try_get("total_transactions").ok(),
        successful_transactions: row.try_get("successful_transactions").ok(),
        failed_transactions: row.try_get("failed_transactions").ok(),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        initiated_by: row.get("initiated_by"),
        notes: row.get("notes"),
        error_message: row.get("error_message"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_sweep_run_line(row: &sqlx::postgres::PgRow) -> CashPoolSweepRunLine {
    CashPoolSweepRunLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        sweep_run_id: row.get("sweep_run_id"),
        pool_id: row.get("pool_id"),
        participant_id: row.get("participant_id"),
        participant_code: row.get("participant_code"),
        bank_account_name: row.get("bank_account_name"),
        sweep_rule_id: row.get("sweep_rule_id"),
        direction: row.get("direction"),
        pre_sweep_balance: Some(get_numeric_text(row, "pre_sweep_balance")),
        sweep_amount: Some(get_numeric_text(row, "sweep_amount")),
        post_sweep_balance: Some(get_numeric_text(row, "post_sweep_balance")),
        status: row.get("status"),
        reference_number: row.get("reference_number"),
        error_message: row.get("error_message"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// PostgreSQL implementation
pub struct PostgresCashConcentrationRepository {
    pool: PgPool,
}

impl PostgresCashConcentrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CashConcentrationRepository for PostgresCashConcentrationRepository {
    // ========================================================================
    // Cash Pools
    // ========================================================================

    async fn create_pool(&self, p: &PoolCreateParams) -> AtlasResult<CashPool> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cash_pools
                (organization_id, pool_code, pool_name, pool_type,
                 concentration_account_id, concentration_account_name, currency_code,
                 sweep_frequency, sweep_time, minimum_transfer_amount,
                 maximum_transfer_amount, target_balance,
                 interest_allocation_method, interest_rate,
                 effective_date, termination_date, description, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(&p.pool_code).bind(&p.pool_name).bind(&p.pool_type)
        .bind(p.concentration_account_id).bind(&p.concentration_account_name).bind(&p.currency_code)
        .bind(&p.sweep_frequency).bind(&p.sweep_time).bind(p.minimum_transfer_amount.as_deref().and_then(|s| s.parse::<f64>().ok()))
        .bind(p.maximum_transfer_amount.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.target_balance.as_deref().and_then(|s| s.parse::<f64>().ok()))
        .bind(&p.interest_allocation_method).bind(p.interest_rate.as_deref().and_then(|s| s.parse::<f64>().ok()))
        .bind(p.effective_date).bind(p.termination_date).bind(&p.description).bind(&p.notes).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_pool(&row))
    }

    async fn get_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<Option<CashPool>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_pools WHERE organization_id=$1 AND pool_code=$2"
        )
        .bind(org_id).bind(pool_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_pool(&r)))
    }

    async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPool>> {
        let row = sqlx::query("SELECT * FROM _atlas.cash_pools WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_pool(&r)))
    }

    async fn list_pools(&self, org_id: Uuid, status: Option<&str>, pool_type: Option<&str>) -> AtlasResult<Vec<CashPool>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.cash_pools
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            AND ($3::text IS NULL OR pool_type=$3)
            ORDER BY pool_code"#,
        )
        .bind(org_id).bind(status).bind(pool_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_pool).collect())
    }

    async fn update_pool_status(&self, id: Uuid, status: &str) -> AtlasResult<CashPool> {
        let row = sqlx::query(
            "UPDATE _atlas.cash_pools SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_pool(&row))
    }

    async fn delete_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.cash_pools WHERE organization_id=$1 AND pool_code=$2")
            .bind(org_id).bind(pool_code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Participants
    // ========================================================================

    async fn create_participant(&self, p: &ParticipantCreateParams) -> AtlasResult<CashPoolParticipant> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cash_pool_participants
                (organization_id, pool_id, participant_code,
                 bank_account_id, bank_account_name, bank_name, account_number,
                 participant_type, sweep_direction, priority,
                 minimum_balance, maximum_balance, threshold_amount, current_balance,
                 entity_id, entity_name, effective_date, description, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(p.pool_id).bind(&p.participant_code)
        .bind(p.bank_account_id).bind(&p.bank_account_name).bind(&p.bank_name).bind(&p.account_number)
        .bind(&p.participant_type).bind(&p.sweep_direction).bind(p.priority)
        .bind(p.minimum_balance.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.maximum_balance.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.threshold_amount.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.current_balance.as_deref().and_then(|s| s.parse::<f64>().ok()))
        .bind(p.entity_id).bind(&p.entity_name).bind(p.effective_date).bind(&p.description).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_participant(&row))
    }

    async fn get_participant(&self, pool_id: Uuid, participant_code: &str) -> AtlasResult<Option<CashPoolParticipant>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_pool_participants WHERE pool_id=$1 AND participant_code=$2"
        )
        .bind(pool_id).bind(participant_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_participant(&r)))
    }

    async fn get_participant_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPoolParticipant>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_pool_participants WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_participant(&r)))
    }

    async fn list_participants(&self, pool_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CashPoolParticipant>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.cash_pool_participants
            WHERE pool_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY priority, participant_code"#,
        )
        .bind(pool_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_participant).collect())
    }

    async fn update_participant_status(&self, id: Uuid, status: &str) -> AtlasResult<CashPoolParticipant> {
        let row = sqlx::query(
            "UPDATE _atlas.cash_pool_participants SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_participant(&row))
    }

    async fn update_participant_balance(&self, id: Uuid, balance: &str) -> AtlasResult<CashPoolParticipant> {
        let bal: f64 = balance.parse().unwrap_or(0.0);
        let row = sqlx::query(
            "UPDATE _atlas.cash_pool_participants SET current_balance=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(bal)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_participant(&row))
    }

    // ========================================================================
    // Sweep Rules
    // ========================================================================

    async fn create_sweep_rule(&self, p: &SweepRuleCreateParams) -> AtlasResult<CashPoolSweepRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cash_pool_sweep_rules
                (organization_id, pool_id, rule_code, rule_name, sweep_type,
                 participant_id, direction, trigger_condition,
                 threshold_amount, target_balance, minimum_transfer, maximum_transfer,
                 priority, effective_date, description, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            RETURNING *"#,
        )
        .bind(p.org_id).bind(p.pool_id).bind(&p.rule_code).bind(&p.rule_name).bind(&p.sweep_type)
        .bind(p.participant_id).bind(&p.direction).bind(&p.trigger_condition)
        .bind(p.threshold_amount.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.target_balance.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.minimum_transfer.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.maximum_transfer.as_deref().and_then(|s| s.parse::<f64>().ok()))
        .bind(p.priority).bind(p.effective_date).bind(&p.description).bind(p.created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sweep_rule(&row))
    }

    async fn list_sweep_rules(&self, pool_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_pool_sweep_rules WHERE pool_id=$1 ORDER BY priority, rule_code"
        )
        .bind(pool_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_sweep_rule).collect())
    }

    async fn delete_sweep_rule(&self, pool_id: Uuid, rule_code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.cash_pool_sweep_rules WHERE pool_id=$1 AND rule_code=$2"
        )
        .bind(pool_id).bind(rule_code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Sweep Runs
    // ========================================================================

    async fn create_sweep_run(&self, p: &SweepRunCreateParams) -> AtlasResult<CashPoolSweepRun> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cash_pool_sweep_runs
                (organization_id, pool_id, run_number, run_date, run_type,
                 status, initiated_by, notes, started_at)
            VALUES ($1,$2,$3,$4,$5,'in_progress',$6,$7,now())
            RETURNING *"#,
        )
        .bind(p.org_id).bind(p.pool_id).bind(&p.run_number).bind(p.run_date)
        .bind(&p.run_type).bind(p.initiated_by).bind(&p.notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sweep_run(&row))
    }

    async fn get_sweep_run(&self, id: Uuid) -> AtlasResult<Option<CashPoolSweepRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.cash_pool_sweep_runs WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_sweep_run(&r)))
    }

    async fn list_sweep_runs(&self, pool_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRun>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_pool_sweep_runs WHERE pool_id=$1 ORDER BY run_date DESC, run_number DESC"
        )
        .bind(pool_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_sweep_run).collect())
    }

    async fn update_sweep_run_status(
        &self, id: Uuid, status: &str,
        total_swept: Option<&str>, total_txns: Option<i32>,
        successful: Option<i32>, failed: Option<i32>,
    ) -> AtlasResult<CashPoolSweepRun> {
        let row = sqlx::query(
            r#"UPDATE _atlas.cash_pool_sweep_runs
            SET status=$2,
                total_swept_amount=COALESCE($3, total_swept_amount),
                total_transactions=COALESCE($4, total_transactions),
                successful_transactions=COALESCE($5, successful_transactions),
                failed_transactions=COALESCE($6, failed_transactions),
                completed_at=CASE WHEN $2 IN ('completed','partially_completed','failed','cancelled')
                    THEN now() ELSE completed_at END,
                updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .bind(total_swept.and_then(|s| s.parse::<f64>().ok())).bind(total_txns).bind(successful).bind(failed)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sweep_run(&row))
    }

    async fn create_sweep_run_line(&self, p: &SweepRunLineCreateParams) -> AtlasResult<CashPoolSweepRunLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.cash_pool_sweep_run_lines
                (organization_id, sweep_run_id, pool_id, participant_id,
                 participant_code, bank_account_name, sweep_rule_id,
                 direction, pre_sweep_balance, sweep_amount,
                 post_sweep_balance, status)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING *"#,
        )
        .bind(p.organization_id).bind(p.sweep_run_id).bind(p.pool_id).bind(p.participant_id)
        .bind(&p.participant_code).bind(&p.bank_account_name).bind(p.sweep_rule_id)
        .bind(&p.direction).bind(p.pre_sweep_balance.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(p.sweep_amount.parse::<f64>().ok())
        .bind(p.post_sweep_balance.as_deref().and_then(|s| s.parse::<f64>().ok())).bind(&p.status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sweep_run_line(&row))
    }

    async fn list_sweep_run_lines(&self, sweep_run_id: Uuid) -> AtlasResult<Vec<CashPoolSweepRunLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_pool_sweep_run_lines WHERE sweep_run_id=$1 ORDER BY participant_code"
        )
        .bind(sweep_run_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_sweep_run_line).collect())
    }

    async fn get_latest_run_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(run_number FROM 7) AS INT)), 0) as max_num FROM _atlas.cash_pool_sweep_runs WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max_num: i64 = row.try_get("max_num").unwrap_or(0);
        Ok(max_num as i32)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashPoolDashboard> {
        let pool_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.cash_pools WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_pools: i64 = pool_row.try_get("total").unwrap_or(0);
        let active_pools: i64 = pool_row.try_get("active").unwrap_or(0);

        let part_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM _atlas.cash_pool_participants WHERE organization_id=$1 AND status='active'"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let total_participants: i64 = part_row.try_get("cnt").unwrap_or(0);

        let sweep_row = sqlx::query(
            r#"SELECT
                COALESCE(SUM(total_swept_amount), 0)::text as today_swept,
                COUNT(*) FILTER (WHERE status = 'pending' OR status = 'in_progress') as pending
            FROM _atlas.cash_pool_sweep_runs WHERE organization_id=$1 AND run_date = CURRENT_DATE"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let today_swept: String = sweep_row.try_get("today_swept").unwrap_or_else(|_| "0".to_string());
        let pending: i64 = sweep_row.try_get("pending").unwrap_or(0);

        // By pool type
        let type_rows = sqlx::query(
            r#"SELECT pool_type, COUNT(*) as count
            FROM _atlas.cash_pools WHERE organization_id = $1 GROUP BY pool_type"#
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_pool_type = serde_json::Map::new();
        for row in &type_rows {
            let pt: String = row.try_get("pool_type").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_pool_type.insert(pt, serde_json::json!(count));
        }

        // By currency
        let curr_rows = sqlx::query(
            r#"SELECT currency_code, COUNT(*) as count
            FROM _atlas.cash_pools WHERE organization_id = $1 GROUP BY currency_code"#
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_currency = serde_json::Map::new();
        for row in &curr_rows {
            let cc: String = row.try_get("currency_code").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            by_currency.insert(cc, serde_json::json!(count));
        }

        Ok(CashPoolDashboard {
            total_pools: total_pools as i32,
            active_pools: active_pools as i32,
            total_participants: total_participants as i32,
            total_concentrated_balance: "0.00".to_string(),
            total_swept_today: today_swept,
            pending_sweeps: pending as i32,
            by_pool_type: serde_json::Value::Object(by_pool_type),
            by_currency: serde_json::Value::Object(by_currency),
        })
    }
}
