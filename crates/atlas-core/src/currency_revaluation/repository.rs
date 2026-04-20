//! Currency Revaluation Repository
//!
//! PostgreSQL storage for currency revaluation definitions, accounts, and runs.

use atlas_shared::{
    CurrencyRevaluationDefinition, CurrencyRevaluationAccount,
    CurrencyRevaluationRun, CurrencyRevaluationLine,
    CurrencyRevaluationDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for currency revaluation storage
#[async_trait]
pub trait CurrencyRevaluationRepository: Send + Sync {
    // ── Definitions ────────────────────────────────────────────
    async fn create_definition(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        revaluation_type: &str, currency_code: &str, rate_type: &str,
        gain_account_code: &str, loss_account_code: &str,
        unrealized_gain_account_code: Option<&str>,
        unrealized_loss_account_code: Option<&str>,
        account_range_from: Option<&str>, account_range_to: Option<&str>,
        include_subledger: bool, auto_reverse: bool, reversal_period_offset: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationDefinition>;
    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationDefinition>>;
    async fn get_definition_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyRevaluationDefinition>>;
    async fn list_definitions(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CurrencyRevaluationDefinition>>;
    async fn update_definition_active(&self, id: Uuid, is_active: bool) -> AtlasResult<CurrencyRevaluationDefinition>;
    async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Accounts ───────────────────────────────────────────────
    async fn add_account(
        &self, org_id: Uuid, definition_id: Uuid, account_code: &str,
        account_name: Option<&str>, account_type: &str, is_included: bool,
    ) -> AtlasResult<CurrencyRevaluationAccount>;
    async fn list_accounts_by_definition(&self, definition_id: Uuid) -> AtlasResult<Vec<CurrencyRevaluationAccount>>;
    async fn delete_account(&self, id: Uuid) -> AtlasResult<()>;

    // ── Runs ────────────────────────────────────────────────────
    async fn create_run(
        &self, org_id: Uuid, run_number: &str,
        definition_id: Uuid, definition_code: &str, definition_name: &str,
        period_name: &str, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, revaluation_date: chrono::NaiveDate,
        currency_code: &str, rate_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationRun>;
    async fn get_run_by_id(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CurrencyRevaluationRun>>;
    async fn update_run_status(
        &self, id: Uuid, status: &str, acted_by: Option<Uuid>,
        reversed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<CurrencyRevaluationRun>;
    async fn update_run_totals(
        &self, id: Uuid, total_revalued: &str, total_gain: &str,
        total_loss: &str, total_entries: i32,
    ) -> AtlasResult<CurrencyRevaluationRun>;
    async fn update_run_reversal(&self, id: Uuid, reversal_run_id: Uuid) -> AtlasResult<()>;

    // ── Run Lines ───────────────────────────────────────────────
    async fn create_run_line(
        &self, org_id: Uuid, run_id: Uuid, line_number: i32,
        account_code: &str, account_name: Option<&str>, account_type: &str,
        original_amount: &str, original_currency: &str,
        original_exchange_rate: &str, original_base_amount: &str,
        revalued_exchange_rate: &str, revalued_base_amount: &str,
        gain_loss_amount: &str, gain_loss_type: &str,
        gain_loss_account_code: &str,
    ) -> AtlasResult<CurrencyRevaluationLine>;
    async fn update_line_reversal(&self, line_id: Uuid, reversal_run_id: Uuid) -> AtlasResult<()>;

    // ── Dashboard ───────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CurrencyRevaluationDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresCurrencyRevaluationRepository {
    pool: PgPool,
}

impl PostgresCurrencyRevaluationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_definition(&self, row: &sqlx::postgres::PgRow) -> CurrencyRevaluationDefinition {
        CurrencyRevaluationDefinition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            revaluation_type: row.get("revaluation_type"),
            currency_code: row.get("currency_code"),
            rate_type: row.get("rate_type"),
            gain_account_code: row.get("gain_account_code"),
            loss_account_code: row.get("loss_account_code"),
            unrealized_gain_account_code: row.get("unrealized_gain_account_code"),
            unrealized_loss_account_code: row.get("unrealized_loss_account_code"),
            account_range_from: row.get("account_range_from"),
            account_range_to: row.get("account_range_to"),
            include_subledger: row.get("include_subledger"),
            auto_reverse: row.get("auto_reverse"),
            reversal_period_offset: row.get("reversal_period_offset"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            accounts: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_account(&self, row: &sqlx::postgres::PgRow) -> CurrencyRevaluationAccount {
        CurrencyRevaluationAccount {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            definition_id: row.get("definition_id"),
            account_code: row.get("account_code"),
            account_name: row.get("account_name"),
            account_type: row.get("account_type"),
            is_included: row.get("is_included"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_run(&self, row: &sqlx::postgres::PgRow) -> CurrencyRevaluationRun {
        CurrencyRevaluationRun {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            run_number: row.get("run_number"),
            definition_id: row.get("definition_id"),
            definition_code: row.get("definition_code"),
            definition_name: row.get("definition_name"),
            period_name: row.get("period_name"),
            period_start_date: row.get("period_start_date"),
            period_end_date: row.get("period_end_date"),
            revaluation_date: row.get("revaluation_date"),
            currency_code: row.get("currency_code"),
            rate_type: row.get("rate_type"),
            total_revalued_amount: row.get("total_revalued_amount"),
            total_gain_amount: row.get("total_gain_amount"),
            total_loss_amount: row.get("total_loss_amount"),
            total_entries: row.get("total_entries"),
            status: row.get("status"),
            reversal_run_id: row.get("reversal_run_id"),
            original_run_id: row.get("original_run_id"),
            reversed_at: row.get("reversed_at"),
            posted_at: row.get("posted_at"),
            posted_by: row.get("posted_by"),
            lines: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> CurrencyRevaluationLine {
        CurrencyRevaluationLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            run_id: row.get("run_id"),
            line_number: row.get("line_number"),
            account_code: row.get("account_code"),
            account_name: row.get("account_name"),
            account_type: row.get("account_type"),
            original_amount: row.get("original_amount"),
            original_currency: row.get("original_currency"),
            original_exchange_rate: row.get("original_exchange_rate"),
            original_base_amount: row.get("original_base_amount"),
            revalued_exchange_rate: row.get("revalued_exchange_rate"),
            revalued_base_amount: row.get("revalued_base_amount"),
            gain_loss_amount: row.get("gain_loss_amount"),
            gain_loss_type: row.get("gain_loss_type"),
            gain_loss_account_code: row.get("gain_loss_account_code"),
            reversal_line_id: row.get("reversal_line_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl CurrencyRevaluationRepository for PostgresCurrencyRevaluationRepository {
    // ── Definitions ────────────────────────────────────────────

    async fn create_definition(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        revaluation_type: &str, currency_code: &str, rate_type: &str,
        gain_account_code: &str, loss_account_code: &str,
        unrealized_gain_account_code: Option<&str>,
        unrealized_loss_account_code: Option<&str>,
        account_range_from: Option<&str>, account_range_to: Option<&str>,
        include_subledger: bool, auto_reverse: bool, reversal_period_offset: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationDefinition> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.currency_revaluation_definitions
                (organization_id, code, name, description, revaluation_type,
                 currency_code, rate_type, gain_account_code, loss_account_code,
                 unrealized_gain_account_code, unrealized_loss_account_code,
                 account_range_from, account_range_to, include_subledger,
                 auto_reverse, reversal_period_offset,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(revaluation_type).bind(currency_code).bind(rate_type)
        .bind(gain_account_code).bind(loss_account_code)
        .bind(unrealized_gain_account_code).bind(unrealized_loss_account_code)
        .bind(account_range_from).bind(account_range_to)
        .bind(include_subledger).bind(auto_reverse).bind(reversal_period_offset)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_definition(&row))
    }

    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationDefinition>> {
        let row = sqlx::query("SELECT * FROM _atlas.currency_revaluation_definitions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let def_id: Uuid = r.get("id");
                let mut def = self.row_to_definition(&r);
                let accounts = sqlx::query(
                    "SELECT * FROM _atlas.currency_revaluation_accounts WHERE definition_id = $1"
                ).bind(def_id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                def.accounts = accounts.iter().map(|a| self.row_to_account(a)).collect();
                Ok(Some(def))
            }
            None => Ok(None),
        }
    }

    async fn get_definition_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyRevaluationDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let def_id: Uuid = r.get("id");
                let mut def = self.row_to_definition(&r);
                let accounts = sqlx::query(
                    "SELECT * FROM _atlas.currency_revaluation_accounts WHERE definition_id = $1"
                ).bind(def_id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                def.accounts = accounts.iter().map(|a| self.row_to_account(a)).collect();
                Ok(Some(def))
            }
            None => Ok(None),
        }
    }

    async fn list_definitions(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CurrencyRevaluationDefinition>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 AND is_active = true ORDER BY code"
            ).bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 ORDER BY code"
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut defs = Vec::new();
        for row in rows {
            let def_id: Uuid = row.get("id");
            let mut def = self.row_to_definition(&row);
            let accounts = sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_accounts WHERE definition_id = $1"
            ).bind(def_id).fetch_all(&self.pool).await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            def.accounts = accounts.iter().map(|a| self.row_to_account(a)).collect();
            defs.push(def);
        }
        Ok(defs)
    }

    async fn update_definition_active(&self, id: Uuid, is_active: bool) -> AtlasResult<CurrencyRevaluationDefinition> {
        let row = sqlx::query(
            "UPDATE _atlas.currency_revaluation_definitions SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_definition(&row))
    }

    async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        // Delete accounts first
        sqlx::query(
            r#"DELETE FROM _atlas.currency_revaluation_accounts WHERE definition_id IN
               (SELECT id FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 AND code = $2)"#
        ).bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 AND code = $2")
            .bind(org_id).bind(code)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Accounts ───────────────────────────────────────────────

    async fn add_account(
        &self, org_id: Uuid, definition_id: Uuid, account_code: &str,
        account_name: Option<&str>, account_type: &str, is_included: bool,
    ) -> AtlasResult<CurrencyRevaluationAccount> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.currency_revaluation_accounts
                (organization_id, definition_id, account_code, account_name, account_type, is_included)
            VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING *"#,
        )
        .bind(org_id).bind(definition_id).bind(account_code)
        .bind(account_name).bind(account_type).bind(is_included)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_account(&row))
    }

    async fn list_accounts_by_definition(&self, definition_id: Uuid) -> AtlasResult<Vec<CurrencyRevaluationAccount>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.currency_revaluation_accounts WHERE definition_id = $1"
        ).bind(definition_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_account(r)).collect())
    }

    async fn delete_account(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.currency_revaluation_accounts WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Runs ────────────────────────────────────────────────────

    async fn create_run(
        &self, org_id: Uuid, run_number: &str,
        definition_id: Uuid, definition_code: &str, definition_name: &str,
        period_name: &str, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, revaluation_date: chrono::NaiveDate,
        currency_code: &str, rate_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationRun> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.currency_revaluation_runs
                (organization_id, run_number, definition_id, definition_code, definition_name,
                 period_name, period_start_date, period_end_date, revaluation_date,
                 currency_code, rate_type,
                 total_revalued_amount, total_gain_amount, total_loss_amount, total_entries,
                 status, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,0,0,0,0,'draft',$12)
            RETURNING *"#,
        )
        .bind(org_id).bind(run_number).bind(definition_id)
        .bind(definition_code).bind(definition_name)
        .bind(period_name).bind(period_start_date).bind(period_end_date)
        .bind(revaluation_date).bind(currency_code).bind(rate_type)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_run(&row))
    }

    async fn get_run_by_id(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.currency_revaluation_runs WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut run = self.row_to_run(&r);
                let lines = sqlx::query(
                    "SELECT * FROM _atlas.currency_revaluation_lines WHERE run_id = $1 ORDER BY line_number"
                ).bind(id).fetch_all(&self.pool).await
                    .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                run.lines = lines.iter().map(|l| self.row_to_line(l)).collect();
                Ok(Some(run))
            }
            None => Ok(None),
        }
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CurrencyRevaluationRun>> {
        let rows = if status.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_runs WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(status)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_runs WHERE organization_id = $1 ORDER BY created_at DESC"
            ).bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut runs = Vec::new();
        for row in rows {
            let run_id: Uuid = row.get("id");
            let mut run = self.row_to_run(&row);
            let lines = sqlx::query(
                "SELECT * FROM _atlas.currency_revaluation_lines WHERE run_id = $1 ORDER BY line_number"
            ).bind(run_id).fetch_all(&self.pool).await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            run.lines = lines.iter().map(|l| self.row_to_line(l)).collect();
            runs.push(run);
        }
        Ok(runs)
    }

    async fn update_run_status(
        &self, id: Uuid, status: &str, acted_by: Option<Uuid>,
        reversed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<CurrencyRevaluationRun> {
        let row = if status == "posted" {
            sqlx::query(
                r#"UPDATE _atlas.currency_revaluation_runs SET status = $2, posted_at = now(), posted_by = $3, updated_at = now() WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).bind(acted_by)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else if status == "reversed" {
            sqlx::query(
                r#"UPDATE _atlas.currency_revaluation_runs SET status = $2, reversed_at = $3, updated_at = now() WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).bind(reversed_at)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "UPDATE _atlas.currency_revaluation_runs SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(self.row_to_run(&row))
    }

    async fn update_run_totals(
        &self, id: Uuid, total_revalued: &str, total_gain: &str,
        total_loss: &str, total_entries: i32,
    ) -> AtlasResult<CurrencyRevaluationRun> {
        let row = sqlx::query(
            r#"UPDATE _atlas.currency_revaluation_runs SET
                total_revalued_amount = $2, total_gain_amount = $3, total_loss_amount = $4,
                total_entries = $5, updated_at = now()
            WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(total_revalued).bind(total_gain).bind(total_loss).bind(total_entries)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_run(&row))
    }

    async fn update_run_reversal(&self, id: Uuid, reversal_run_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.currency_revaluation_runs SET reversal_run_id = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(reversal_run_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Run Lines ───────────────────────────────────────────────

    async fn create_run_line(
        &self, org_id: Uuid, run_id: Uuid, line_number: i32,
        account_code: &str, account_name: Option<&str>, account_type: &str,
        original_amount: &str, original_currency: &str,
        original_exchange_rate: &str, original_base_amount: &str,
        revalued_exchange_rate: &str, revalued_base_amount: &str,
        gain_loss_amount: &str, gain_loss_type: &str,
        gain_loss_account_code: &str,
    ) -> AtlasResult<CurrencyRevaluationLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.currency_revaluation_lines
                (organization_id, run_id, line_number,
                 account_code, account_name, account_type,
                 original_amount, original_currency,
                 original_exchange_rate, original_base_amount,
                 revalued_exchange_rate, revalued_base_amount,
                 gain_loss_amount, gain_loss_type, gain_loss_account_code)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING *"#,
        )
        .bind(org_id).bind(run_id).bind(line_number)
        .bind(account_code).bind(account_name).bind(account_type)
        .bind(original_amount).bind(original_currency)
        .bind(original_exchange_rate).bind(original_base_amount)
        .bind(revalued_exchange_rate).bind(revalued_base_amount)
        .bind(gain_loss_amount).bind(gain_loss_type).bind(gain_loss_account_code)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_line(&row))
    }

    async fn update_line_reversal(&self, line_id: Uuid, reversal_run_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.currency_revaluation_lines SET reversal_line_id = $2 WHERE id = $1"
        ).bind(line_id).bind(reversal_run_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Dashboard ───────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CurrencyRevaluationDashboardSummary> {
        let def_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE is_active) as active FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let run_row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(*) FILTER (WHERE status = 'posted') as posted, COUNT(*) FILTER (WHERE status = 'draft') as draft, COUNT(*) FILTER (WHERE status = 'reversed') as reversed, COALESCE(SUM(total_gain_amount::numeric), 0) as total_gain, COALESCE(SUM(total_loss_amount::numeric), 0) as total_loss FROM _atlas.currency_revaluation_runs WHERE organization_id = $1"
        ).bind(org_id).fetch_one(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let type_rows = sqlx::query(
            "SELECT revaluation_type, COUNT(*) as cnt FROM _atlas.currency_revaluation_definitions WHERE organization_id = $1 GROUP BY revaluation_type"
        ).bind(org_id).fetch_all(&self.pool).await.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut definitions_by_type = serde_json::Map::new();
        for r in &type_rows {
            let rt: String = r.get("revaluation_type");
            let cnt: i64 = r.get("cnt");
            definitions_by_type.insert(rt, serde_json::json!(cnt));
        }

        let total_defs: i64 = def_row.try_get("total").unwrap_or(0);
        let active_defs: i64 = def_row.try_get("active").unwrap_or(0);
        let total_runs: i64 = run_row.try_get("total").unwrap_or(0);
        let posted_runs: i64 = run_row.try_get("posted").unwrap_or(0);
        let draft_runs: i64 = run_row.try_get("draft").unwrap_or(0);
        let reversed_runs: i64 = run_row.try_get("reversed").unwrap_or(0);
        let total_gain: serde_json::Value = run_row.try_get("total_gain").unwrap_or(serde_json::json!("0"));
        let total_loss: serde_json::Value = run_row.try_get("total_loss").unwrap_or(serde_json::json!("0"));

        Ok(CurrencyRevaluationDashboardSummary {
            total_definitions: total_defs as i32,
            active_definitions: active_defs as i32,
            total_runs: total_runs as i32,
            posted_runs: posted_runs as i32,
            draft_runs: draft_runs as i32,
            reversed_runs: reversed_runs as i32,
            total_gain_amount: total_gain.to_string(),
            total_loss_amount: total_loss.to_string(),
            definitions_by_type: serde_json::Value::Object(definitions_by_type),
        })
    }
}