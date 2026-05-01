//! Funds Reservation Repository
//!
//! PostgreSQL storage for fund reservations, reservation lines,
//! fund availability checks, and budgetary control dashboard.

use atlas_shared::{
    FundReservation, FundReservationLine, FundAvailability,
    BudgetaryControlDashboard,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for funds reservation data storage
#[async_trait]
pub trait FundsReservationRepository: Send + Sync {
    // Fund Reservations
    async fn create_reservation(
        &self, org_id: Uuid, reservation_number: &str,
        budget_id: Uuid, budget_code: &str, budget_version_id: Option<Uuid>,
        description: Option<&str>,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        reserved_amount: f64, currency_code: &str,
        reservation_date: chrono::NaiveDate, expiry_date: Option<chrono::NaiveDate>,
        status: &str, control_level: &str,
        fiscal_year: Option<i32>, period_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        fund_check_passed: bool, fund_check_message: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<FundReservation>;

    async fn get_reservation(&self, id: Uuid) -> AtlasResult<Option<FundReservation>>;
    async fn get_reservation_by_number(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<Option<FundReservation>>;
    async fn list_reservations(
        &self, org_id: Uuid, status: Option<&str>, budget_id: Option<&Uuid>,
        department_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<FundReservation>>;
    async fn update_reservation_status(&self, id: Uuid, status: &str) -> AtlasResult<FundReservation>;
    async fn update_reservation_amounts(
        &self, id: Uuid, consumed_amount: f64, released_amount: f64, remaining_amount: f64,
    ) -> AtlasResult<FundReservation>;
    async fn cancel_reservation(
        &self, id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>,
    ) -> AtlasResult<FundReservation>;
    async fn delete_reservation(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<()>;

    // Fund Reservation Lines
    async fn create_reservation_line(
        &self, org_id: Uuid, reservation_id: Uuid, line_number: i32,
        account_code: &str, account_description: Option<&str>,
        budget_line_id: Option<Uuid>, department_id: Option<Uuid>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        reserved_amount: f64,
        metadata: serde_json::Value,
    ) -> AtlasResult<FundReservationLine>;
    async fn list_reservation_lines(&self, reservation_id: Uuid) -> AtlasResult<Vec<FundReservationLine>>;
    async fn update_reservation_line_amounts(
        &self, id: Uuid, consumed_amount: f64, released_amount: f64, remaining_amount: f64,
    ) -> AtlasResult<FundReservationLine>;

    // Fund Availability
    async fn check_fund_availability(
        &self, org_id: Uuid, budget_id: Uuid, account_code: &str,
        as_of_date: chrono::NaiveDate, fiscal_year: Option<i32>,
        period_name: Option<&str>,
    ) -> AtlasResult<FundAvailability>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetaryControlDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresFundsReservationRepository {
    pool: PgPool,
}

impl PostgresFundsReservationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Helper numeric decode
fn get_numeric(row: &sqlx::postgres::PgRow, column: &str) -> f64 {
    if let Ok(v) = row.try_get::<f64, _>(column) {
        return v;
    }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() {
            return n;
        }
        if let Some(s) = v.as_str() {
            if let Ok(n) = s.parse::<f64>() {
                return n;
            }
        }
    }
    if let Ok(s) = row.try_get::<String, _>(column) {
        return s.parse::<f64>().unwrap_or(0.0);
    }
    0.0
}

fn get_optional_numeric(row: &sqlx::postgres::PgRow, column: &str) -> Option<f64> {
    if let Ok(v) = row.try_get::<f64, _>(column) {
        return Some(v);
    }
    if let Ok(v) = row.try_get::<serde_json::Value, _>(column) {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(s) = v.as_str() {
            return s.parse::<f64>().ok();
        }
    }
    if let Ok(s) = row.try_get::<String, _>(column) {
        return s.parse::<f64>().ok();
    }
    None
}

fn row_to_reservation(row: &sqlx::postgres::PgRow) -> FundReservation {
    FundReservation {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        reservation_number: row.try_get("reservation_number").unwrap_or_default(),
        budget_id: row.try_get("budget_id").unwrap_or_default(),
        budget_code: row.try_get("budget_code").unwrap_or_default(),
        budget_version_id: row.try_get("budget_version_id").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        source_type: row.try_get("source_type").unwrap_or_default(),
        source_id: row.try_get("source_id").unwrap_or_default(),
        source_number: row.try_get("source_number").unwrap_or_default(),
        reserved_amount: get_numeric(row, "reserved_amount"),
        consumed_amount: get_numeric(row, "consumed_amount"),
        released_amount: get_numeric(row, "released_amount"),
        remaining_amount: get_numeric(row, "remaining_amount"),
        currency_code: row.try_get("currency_code").unwrap_or_default(),
        reservation_date: row.try_get("reservation_date").unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        expiry_date: row.try_get("expiry_date").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        control_level: row.try_get("control_level").unwrap_or_default(),
        fiscal_year: row.try_get("fiscal_year").unwrap_or_default(),
        period_name: row.try_get("period_name").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        department_name: row.try_get("department_name").unwrap_or_default(),
        fund_check_passed: row.try_get("fund_check_passed").unwrap_or(true),
        fund_check_message: row.try_get("fund_check_message").unwrap_or_default(),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.try_get("created_by").unwrap_or_default(),
        approved_by: row.try_get("approved_by").unwrap_or_default(),
        cancelled_by: row.try_get("cancelled_by").unwrap_or_default(),
        cancelled_at: row.try_get("cancelled_at").unwrap_or_default(),
        cancellation_reason: row.try_get("cancellation_reason").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_reservation_line(row: &sqlx::postgres::PgRow) -> FundReservationLine {
    FundReservationLine {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        reservation_id: row.try_get("reservation_id").unwrap_or_default(),
        line_number: row.try_get("line_number").unwrap_or(1),
        account_code: row.try_get("account_code").unwrap_or_default(),
        account_description: row.try_get("account_description").unwrap_or_default(),
        budget_line_id: row.try_get("budget_line_id").unwrap_or_default(),
        department_id: row.try_get("department_id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        cost_center: row.try_get("cost_center").unwrap_or_default(),
        reserved_amount: get_numeric(row, "reserved_amount"),
        consumed_amount: get_numeric(row, "consumed_amount"),
        released_amount: get_numeric(row, "released_amount"),
        remaining_amount: get_numeric(row, "remaining_amount"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl FundsReservationRepository for PostgresFundsReservationRepository {
    // ========================================================================
    // Fund Reservations
    // ========================================================================

    async fn create_reservation(
        &self, org_id: Uuid, reservation_number: &str,
        budget_id: Uuid, budget_code: &str, budget_version_id: Option<Uuid>,
        description: Option<&str>,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        reserved_amount: f64, currency_code: &str,
        reservation_date: chrono::NaiveDate, expiry_date: Option<chrono::NaiveDate>,
        status: &str, control_level: &str,
        fiscal_year: Option<i32>, period_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        fund_check_passed: bool, fund_check_message: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<FundReservation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.fund_reservations
                (organization_id, reservation_number,
                 budget_id, budget_code, budget_version_id,
                 description, source_type, source_id, source_number,
                 reserved_amount, consumed_amount, released_amount, remaining_amount,
                 currency_code, reservation_date, expiry_date,
                 status, control_level, fiscal_year, period_name,
                 department_id, department_name,
                 fund_check_passed, fund_check_message,
                 metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10, 0, 0, $10,
                    $11, $12, $13, $14, $15, $16, $17,
                    $18, $19, $20, $21, $22, $23)
            RETURNING *"#,
        )
        .bind(org_id).bind(reservation_number)
        .bind(budget_id).bind(budget_code).bind(budget_version_id)
        .bind(description).bind(source_type).bind(source_id).bind(source_number)
        .bind(reserved_amount)
        .bind(currency_code).bind(reservation_date).bind(expiry_date)
        .bind(status).bind(control_level).bind(fiscal_year).bind(period_name)
        .bind(department_id).bind(department_name)
        .bind(fund_check_passed).bind(fund_check_message)
        .bind(&metadata).bind(created_by)
        .fetch_one(&self.pool).await?;
        Ok(row_to_reservation(&row))
    }

    async fn get_reservation(&self, id: Uuid) -> AtlasResult<Option<FundReservation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.fund_reservations WHERE id = $1"
        ).bind(id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_reservation))
    }

    async fn get_reservation_by_number(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<Option<FundReservation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.fund_reservations WHERE organization_id = $1 AND reservation_number = $2"
        ).bind(org_id).bind(reservation_number).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_reservation))
    }

    async fn list_reservations(
        &self, org_id: Uuid, status: Option<&str>, budget_id: Option<&Uuid>,
        department_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<FundReservation>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.fund_reservations
               WHERE organization_id = $1
                 AND ($2::text IS NULL OR status = $2)
                 AND ($3::uuid IS NULL OR budget_id = $3)
                 AND ($4::uuid IS NULL OR department_id = $4)
               ORDER BY reservation_date DESC, created_at DESC"#,
        )
        .bind(org_id).bind(status).bind(budget_id.copied()).bind(department_id.copied())
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_reservation).collect())
    }

    async fn update_reservation_status(&self, id: Uuid, status: &str) -> AtlasResult<FundReservation> {
        let row = sqlx::query(
            "UPDATE _atlas.fund_reservations SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        ).bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|_| atlas_shared::AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;
        Ok(row_to_reservation(&row))
    }

    async fn update_reservation_amounts(
        &self, id: Uuid, consumed_amount: f64, released_amount: f64, remaining_amount: f64,
    ) -> AtlasResult<FundReservation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.fund_reservations
               SET consumed_amount = $2, released_amount = $3, remaining_amount = $4,
                   status = CASE
                       WHEN $2 >= (reserved_amount - $3) THEN 'fully_consumed'
                       WHEN $2 > 0 THEN 'partially_consumed'
                       ELSE status
                   END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(consumed_amount).bind(released_amount).bind(remaining_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| atlas_shared::AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;
        Ok(row_to_reservation(&row))
    }

    async fn cancel_reservation(
        &self, id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>,
    ) -> AtlasResult<FundReservation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.fund_reservations
               SET status = 'cancelled',
                   remaining_amount = 0,
                   released_amount = reserved_amount - consumed_amount,
                   cancelled_by = $2,
                   cancelled_at = now(),
                   cancellation_reason = $3,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(cancelled_by).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|_| atlas_shared::AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;
        Ok(row_to_reservation(&row))
    }

    async fn delete_reservation(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.fund_reservations WHERE organization_id = $1 AND reservation_number = $2"
        ).bind(org_id).bind(reservation_number).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(atlas_shared::AtlasError::EntityNotFound(
                format!("Reservation '{}' not found", reservation_number)
            ));
        }
        Ok(())
    }

    // ========================================================================
    // Fund Reservation Lines
    // ========================================================================

    async fn create_reservation_line(
        &self, org_id: Uuid, reservation_id: Uuid, line_number: i32,
        account_code: &str, account_description: Option<&str>,
        budget_line_id: Option<Uuid>, department_id: Option<Uuid>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        reserved_amount: f64,
        metadata: serde_json::Value,
    ) -> AtlasResult<FundReservationLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.fund_reservation_lines
                (organization_id, reservation_id, line_number,
                 account_code, account_description,
                 budget_line_id, department_id, project_id, cost_center,
                 reserved_amount, consumed_amount, released_amount, remaining_amount,
                 metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, 0, $10, $11)
            RETURNING *"#,
        )
        .bind(org_id).bind(reservation_id).bind(line_number)
        .bind(account_code).bind(account_description)
        .bind(budget_line_id).bind(department_id).bind(project_id).bind(cost_center)
        .bind(reserved_amount).bind(&metadata)
        .fetch_one(&self.pool).await?;
        Ok(row_to_reservation_line(&row))
    }

    async fn list_reservation_lines(&self, reservation_id: Uuid) -> AtlasResult<Vec<FundReservationLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.fund_reservation_lines WHERE reservation_id = $1 ORDER BY line_number"
        ).bind(reservation_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_reservation_line).collect())
    }

    async fn update_reservation_line_amounts(
        &self, id: Uuid, consumed_amount: f64, released_amount: f64, remaining_amount: f64,
    ) -> AtlasResult<FundReservationLine> {
        let row = sqlx::query(
            r#"UPDATE _atlas.fund_reservation_lines
               SET consumed_amount = $2, released_amount = $3, remaining_amount = $4,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(consumed_amount).bind(released_amount).bind(remaining_amount)
        .fetch_one(&self.pool).await
        .map_err(|_| atlas_shared::AtlasError::EntityNotFound(format!("Reservation line {} not found", id)))?;
        Ok(row_to_reservation_line(&row))
    }

    // ========================================================================
    // Fund Availability
    // ========================================================================

    async fn check_fund_availability(
        &self, org_id: Uuid, budget_id: Uuid, account_code: &str,
        as_of_date: chrono::NaiveDate, fiscal_year: Option<i32>,
        period_name: Option<&str>,
    ) -> AtlasResult<FundAvailability> {
        // Get budget amount from budget lines
        let budget_row = sqlx::query(
            r#"SELECT COALESCE(SUM(
                CASE
                    WHEN bl.budget_amount ~ '^[0-9]+\.?[0-9]*$' THEN bl.budget_amount::numeric
                    ELSE 0
                END
            ), 0) as budget_amount,
            COALESCE(bd.control_level, 'none') as control_level,
            COALESCE(bd.budget_code, '') as budget_code
            FROM _atlas.budget_lines bl
            JOIN _atlas.budget_definitions bd ON bd.id = bl.budget_definition_id
            WHERE bl.budget_definition_id = $1
              AND bl.account_code = $2
              AND bl.organization_id = $3"#,
        )
        .bind(budget_id).bind(account_code).bind(org_id)
        .fetch_optional(&self.pool).await?;

        let budget_amount: f64 = budget_row.as_ref()
            .and_then(|r| get_optional_numeric(r, "budget_amount"))
            .unwrap_or(0.0);
        let control_level = budget_row.as_ref()
            .and_then(|r| r.try_get::<String, _>("control_level").ok())
            .unwrap_or_else(|| "none".to_string());
        let budget_code = budget_row.as_ref()
            .and_then(|r| r.try_get::<String, _>("budget_code").ok())
            .unwrap_or_default();

        // Calculate total reserved, consumed, released from active reservations
        let summary_row = sqlx::query(
            r#"SELECT
                COALESCE(SUM(
                    CASE WHEN frl.reserved_amount ~ '^[0-9]+\.?[0-9]*$' THEN frl.reserved_amount::numeric ELSE 0 END
                ), 0) as total_reserved,
                COALESCE(SUM(
                    CASE WHEN frl.consumed_amount ~ '^[0-9]+\.?[0-9]*$' THEN frl.consumed_amount::numeric ELSE 0 END
                ), 0) as total_consumed,
                COALESCE(SUM(
                    CASE WHEN frl.released_amount ~ '^[0-9]+\.?[0-9]*$' THEN frl.released_amount::numeric ELSE 0 END
                ), 0) as total_released
            FROM _atlas.fund_reservation_lines frl
            JOIN _atlas.fund_reservations fr ON fr.id = frl.reservation_id
            WHERE fr.budget_id = $1
              AND frl.account_code = $2
              AND fr.organization_id = $3
              AND fr.status IN ('active', 'partially_consumed')"#,
        )
        .bind(budget_id).bind(account_code).bind(org_id)
        .fetch_one(&self.pool).await?;

        let total_reserved: f64 = get_numeric(&summary_row, "total_reserved");
        let total_consumed: f64 = get_numeric(&summary_row, "total_consumed");
        let total_released: f64 = get_numeric(&summary_row, "total_released");
        let available_balance = budget_amount - total_reserved + total_released - total_consumed;

        Ok(FundAvailability {
            organization_id: org_id,
            budget_id,
            budget_code,
            account_code: account_code.to_string(),
            budget_amount,
            total_reserved,
            total_consumed,
            total_released,
            available_balance,
            check_passed: available_balance >= 0.0,
            control_level,
            message: if available_balance >= 0.0 {
                format!("Funds available: {:.2}", available_balance)
            } else {
                format!("Insufficient funds: shortfall of {:.2}", available_balance.abs())
            },
            as_of_date,
            fiscal_year,
            period_name: period_name.map(String::from),
        })
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetaryControlDashboard> {
        let rows = sqlx::query(
            "SELECT status, reserved_amount, consumed_amount, released_amount FROM _atlas.fund_reservations WHERE organization_id = $1"
        ).bind(org_id).fetch_all(&self.pool).await.unwrap_or_default();

        let total_reservations = rows.len() as i64;
        let active_reservations = rows.iter()
            .filter(|r| {
                let s: String = r.try_get("status").unwrap_or_default();
                s == "active" || s == "partially_consumed"
            })
            .count() as i64;

        let total_reserved_amount: f64 = rows.iter()
            .map(|r| get_numeric(r, "reserved_amount"))
            .sum();
        let total_consumed_amount: f64 = rows.iter()
            .map(|r| get_numeric(r, "consumed_amount"))
            .sum();
        let total_released_amount: f64 = rows.iter()
            .map(|r| get_numeric(r, "released_amount"))
            .sum();

        // Build status summary
        let mut status_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in &rows {
            let s: String = row.try_get("status").unwrap_or_default();
            *status_counts.entry(s).or_insert(0) += 1;
        }

        let total_available_amount = total_reserved_amount - total_consumed_amount - total_released_amount;
        let budget_utilization_pct = if total_reserved_amount > 0.0 {
            (total_consumed_amount / total_reserved_amount * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        Ok(BudgetaryControlDashboard {
            organization_id: org_id,
            total_reservations,
            active_reservations,
            total_reserved_amount,
            total_consumed_amount,
            total_released_amount,
            total_available_amount,
            reservations_by_status: serde_json::to_value(&status_counts).unwrap_or(serde_json::json!({})),
            top_departments_by_reservation: serde_json::json!([]),
            budget_utilization_pct,
        })
    }
}
