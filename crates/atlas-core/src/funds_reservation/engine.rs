//! Funds Reservation & Budgetary Control Engine
//!
//! Manages fund reservations, availability checks, consumption, and releases.
//!
//! Oracle Fusion Cloud equivalent: Financials > Budgetary Control > Funds Reservation

use atlas_shared::{
    FundReservation, FundReservationLine, FundAvailability,
    BudgetaryControlDashboard,
    AtlasError, AtlasResult,
};
use super::FundsReservationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_STATUSES: &[&str] = &[
    "draft", "active", "partially_consumed", "fully_consumed",
    "released", "expired", "cancelled",
];

const VALID_CONTROL_LEVELS: &[&str] = &[
    "advisory", "absolute",
];

const VALID_SOURCE_TYPES: &[&str] = &[
    "purchase_requisition", "purchase_order", "contract",
    "expense_report", "journal_entry", "manual", "other",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Funds Reservation & Budgetary Control Engine
pub struct FundsReservationEngine {
    repository: Arc<dyn FundsReservationRepository>,
}

impl FundsReservationEngine {
    pub fn new(repository: Arc<dyn FundsReservationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Fund Reservations
    // ========================================================================

    /// Create a fund reservation
    #[allow(clippy::too_many_arguments)]
    pub async fn create_reservation(
        &self,
        org_id: Uuid,
        reservation_number: &str,
        budget_id: Uuid,
        budget_code: &str,
        budget_version_id: Option<Uuid>,
        description: Option<&str>,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        reserved_amount: f64,
        currency_code: &str,
        reservation_date: chrono::NaiveDate,
        expiry_date: Option<chrono::NaiveDate>,
        control_level: &str,
        fiscal_year: Option<i32>,
        period_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FundReservation> {
        // Validate required fields
        if reservation_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Reservation number is required".to_string()));
        }
        if budget_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Budget code is required".to_string()));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }

        // Validate enum fields
        validate_enum("control_level", control_level, VALID_CONTROL_LEVELS)?;
        if let Some(st) = source_type {
            validate_enum("source_type", st, VALID_SOURCE_TYPES)?;
        }

        // Validate amounts
        if reserved_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Reserved amount must be positive".to_string(),
            ));
        }

        // Validate dates
        if let Some(exp) = expiry_date {
            if exp <= reservation_date {
                return Err(AtlasError::ValidationFailed(
                    "Expiry date must be after reservation date".to_string(),
                ));
            }
        }

        // Check for duplicate reservation number
        if self.repository.get_reservation_by_number(org_id, reservation_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Reservation '{}' already exists", reservation_number
            )));
        }

        // Perform fund availability check
        let fund_check = self.repository.check_fund_availability(
            org_id, budget_id, "*", // wildcard - check overall budget
            reservation_date, fiscal_year, period_name,
        ).await.unwrap_or(FundAvailability {
            organization_id: org_id,
            budget_id,
            budget_code: budget_code.to_string(),
            account_code: "*".to_string(),
            budget_amount: 0.0,
            total_reserved: 0.0,
            total_consumed: 0.0,
            total_released: 0.0,
            available_balance: reserved_amount, // assume available if no budget data
            check_passed: true,
            control_level: control_level.to_string(),
            message: "No budget data found - allowing reservation".to_string(),
            as_of_date: reservation_date,
            fiscal_year,
            period_name: period_name.map(String::from),
        });

        let fund_check_passed = fund_check.available_balance >= reserved_amount;
        let fund_check_message = if fund_check_passed {
            Some(format!("Fund check passed. Available: {:.2}, Requested: {:.2}",
                        fund_check.available_balance, reserved_amount))
        } else {
            Some(format!("Fund check failed. Available: {:.2}, Requested: {:.2}. Control: {}",
                        fund_check.available_balance, reserved_amount, control_level))
        };

        // For absolute control, block if insufficient funds
        if control_level == "absolute" && !fund_check_passed {
            return Err(AtlasError::ValidationFailed(format!(
                "Insufficient funds. Available: {:.2}, Requested: {:.2}. Budget control is set to absolute.",
                fund_check.available_balance, reserved_amount
            )));
        }

        info!("Creating fund reservation '{}' for org {} [budget={}, amount={:.2}, control={}]",
              reservation_number, org_id, budget_code, reserved_amount, control_level);

        self.repository.create_reservation(
            org_id, reservation_number,
            budget_id, budget_code, budget_version_id,
            description,
            source_type, source_id, source_number,
            reserved_amount, currency_code,
            reservation_date, expiry_date,
            "active", control_level,
            fiscal_year, period_name,
            department_id, department_name,
            fund_check_passed, fund_check_message.as_deref(),
            serde_json::json!({}), created_by,
        ).await
    }

    /// Get a reservation by ID
    pub async fn get_reservation(&self, id: Uuid) -> AtlasResult<Option<FundReservation>> {
        self.repository.get_reservation(id).await
    }

    /// Get a reservation by number
    pub async fn get_reservation_by_number(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<Option<FundReservation>> {
        self.repository.get_reservation_by_number(org_id, reservation_number).await
    }

    /// List reservations with optional filters
    pub async fn list_reservations(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        budget_id: Option<&Uuid>,
        department_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<FundReservation>> {
        self.repository.list_reservations(org_id, status, budget_id, department_id).await
    }

    /// Consume funds from a reservation (when actual expenditure occurs)
    pub async fn consume_reservation(
        &self,
        id: Uuid,
        consume_amount: f64,
    ) -> AtlasResult<FundReservation> {
        if consume_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Consume amount must be positive".to_string(),
            ));
        }

        let reservation = self.repository.get_reservation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;

        if reservation.status != "active" && reservation.status != "partially_consumed" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot consume from reservation in '{}' status. Must be 'active' or 'partially_consumed'.",
                reservation.status
            )));
        }

        if consume_amount > reservation.remaining_amount {
            return Err(AtlasError::ValidationFailed(format!(
                "Consume amount ({:.2}) exceeds remaining reservation amount ({:.2})",
                consume_amount, reservation.remaining_amount
            )));
        }

        let new_consumed = reservation.consumed_amount + consume_amount;
        let new_remaining = reservation.reserved_amount - new_consumed - reservation.released_amount;

        info!("Consuming {:.2} from reservation {} (remaining: {:.2} -> {:.2})",
              consume_amount, reservation.reservation_number, reservation.remaining_amount, new_remaining);

        self.repository.update_reservation_amounts(id, new_consumed, reservation.released_amount, new_remaining).await
    }

    /// Release funds from a reservation (partial or full release)
    pub async fn release_reservation(
        &self,
        id: Uuid,
        release_amount: f64,
    ) -> AtlasResult<FundReservation> {
        if release_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Release amount must be positive".to_string(),
            ));
        }

        let reservation = self.repository.get_reservation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;

        if reservation.status == "cancelled" || reservation.status == "fully_consumed" || reservation.status == "released" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot release from reservation in '{}' status.",
                reservation.status
            )));
        }

        if release_amount > reservation.remaining_amount {
            return Err(AtlasError::ValidationFailed(format!(
                "Release amount ({:.2}) exceeds remaining reservation amount ({:.2})",
                release_amount, reservation.remaining_amount
            )));
        }

        let new_released = reservation.released_amount + release_amount;
        let new_remaining = reservation.reserved_amount - reservation.consumed_amount - new_released;

        info!("Releasing {:.2} from reservation {} (remaining: {:.2} -> {:.2})",
              release_amount, reservation.reservation_number, reservation.remaining_amount, new_remaining);

        let result = self.repository.update_reservation_amounts(
            id, reservation.consumed_amount, new_released, new_remaining,
        ).await?;

        // If fully released, update status
        if new_remaining <= 0.0 {
            return self.repository.update_reservation_status(id, "released").await;
        }

        Ok(result)
    }

    /// Cancel a reservation
    pub async fn cancel_reservation(
        &self,
        id: Uuid,
        cancelled_by: Option<Uuid>,
        reason: Option<&str>,
    ) -> AtlasResult<FundReservation> {
        let reservation = self.repository.get_reservation(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Reservation {} not found", id)))?;

        if reservation.status == "cancelled" {
            return Err(AtlasError::ValidationFailed("Reservation is already cancelled".to_string()));
        }
        if reservation.status == "fully_consumed" {
            return Err(AtlasError::ValidationFailed("Cannot cancel a fully consumed reservation".to_string()));
        }

        info!("Cancelling reservation {} [reason: {}]",
              reservation.reservation_number, reason.unwrap_or("N/A"));

        self.repository.cancel_reservation(id, cancelled_by, reason).await
    }

    /// Update reservation status
    pub async fn update_reservation_status(&self, id: Uuid, status: &str) -> AtlasResult<FundReservation> {
        validate_enum("status", status, VALID_STATUSES)?;
        info!("Updating reservation {} status to {}", id, status);
        self.repository.update_reservation_status(id, status).await
    }

    /// Delete a reservation by number
    pub async fn delete_reservation(&self, org_id: Uuid, reservation_number: &str) -> AtlasResult<()> {
        info!("Deleting reservation '{}' for org {}", reservation_number, org_id);
        self.repository.delete_reservation(org_id, reservation_number).await
    }

    // ========================================================================
    // Fund Reservation Lines
    // ========================================================================

    /// Add a line to a reservation
    #[allow(clippy::too_many_arguments)]
    pub async fn create_reservation_line(
        &self,
        org_id: Uuid,
        reservation_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_description: Option<&str>,
        budget_line_id: Option<Uuid>,
        department_id: Option<Uuid>,
        project_id: Option<Uuid>,
        cost_center: Option<&str>,
        reserved_amount: f64,
    ) -> AtlasResult<FundReservationLine> {
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".to_string()));
        }
        if reserved_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed("Line reserved amount must be positive".to_string()));
        }

        // Verify reservation exists
        self.repository.get_reservation(reservation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Reservation {} not found", reservation_id
            )))?;

        info!("Adding line {} to reservation {} [account={}, amount={:.2}]",
              line_number, reservation_id, account_code, reserved_amount);

        self.repository.create_reservation_line(
            org_id, reservation_id, line_number,
            account_code, account_description,
            budget_line_id, department_id, project_id, cost_center,
            reserved_amount,
            serde_json::json!({}),
        ).await
    }

    /// List lines for a reservation
    pub async fn list_reservation_lines(&self, reservation_id: Uuid) -> AtlasResult<Vec<FundReservationLine>> {
        self.repository.list_reservation_lines(reservation_id).await
    }

    // ========================================================================
    // Fund Availability
    // ========================================================================

    /// Check fund availability for a specific account within a budget
    pub async fn check_fund_availability(
        &self,
        org_id: Uuid,
        budget_id: Uuid,
        account_code: &str,
        as_of_date: chrono::NaiveDate,
        fiscal_year: Option<i32>,
        period_name: Option<&str>,
    ) -> AtlasResult<FundAvailability> {
        info!("Checking fund availability for budget {} account {} as of {}",
              budget_id, account_code, as_of_date);
        self.repository.check_fund_availability(
            org_id, budget_id, account_code, as_of_date, fiscal_year, period_name,
        ).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the budgetary control dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BudgetaryControlDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"active"));
        assert!(VALID_STATUSES.contains(&"partially_consumed"));
        assert!(VALID_STATUSES.contains(&"fully_consumed"));
        assert!(VALID_STATUSES.contains(&"released"));
        assert!(VALID_STATUSES.contains(&"expired"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert!(!VALID_STATUSES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_control_levels() {
        assert!(VALID_CONTROL_LEVELS.contains(&"advisory"));
        assert!(VALID_CONTROL_LEVELS.contains(&"absolute"));
        assert!(!VALID_CONTROL_LEVELS.contains(&"none"));
    }

    #[test]
    fn test_valid_source_types() {
        assert!(VALID_SOURCE_TYPES.contains(&"purchase_requisition"));
        assert!(VALID_SOURCE_TYPES.contains(&"purchase_order"));
        assert!(VALID_SOURCE_TYPES.contains(&"contract"));
        assert!(VALID_SOURCE_TYPES.contains(&"expense_report"));
        assert!(VALID_SOURCE_TYPES.contains(&"journal_entry"));
        assert!(VALID_SOURCE_TYPES.contains(&"manual"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("control_level", "advisory", VALID_CONTROL_LEVELS).is_ok());
        assert!(validate_enum("control_level", "absolute", VALID_CONTROL_LEVELS).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("control_level", "permissive", VALID_CONTROL_LEVELS);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("control_level"));
                assert!(msg.contains("permissive"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("status", "", VALID_STATUSES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockFundsReservationRepository;
    use chrono::NaiveDate;

    fn create_engine() -> FundsReservationEngine {
        FundsReservationEngine::new(Arc::new(MockFundsReservationRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    fn test_budget_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap()
    }

    // --- Reservation Creation Tests ---

    #[tokio::test]
    async fn test_create_reservation_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            5000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_empty_budget_code() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "", None,
            None, None, None, None,
            5000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Budget code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_bad_control_level() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            5000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "permissive", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("control_level")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_bad_source_type() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            None, Some("invalid_source"), None, None,
            5000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("source_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_negative_amount() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            -500.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_expiry_before_reservation() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            5000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2024, 5, 1).unwrap()),
            "advisory", Some(2024), Some("Q2-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Expiry date")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_validation_empty_currency() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            5000.0, "",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), Some("Q1-2024"),
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Currency")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_success() {
        let engine = create_engine();
        let result = engine.create_reservation(
            test_org_id(), "FR-001", test_budget_id(), "BUD-001", None,
            Some("Reserve funds for Q1 IT purchases"),
            Some("purchase_requisition"), None, Some("PR-00123"),
            50000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            "advisory", Some(2024), Some("Q1-2024"),
            None, Some("IT Department"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let reservation = result.unwrap();
        assert_eq!(reservation.reservation_number, "FR-001");
        assert_eq!(reservation.budget_code, "BUD-001");
        assert_eq!(reservation.control_level, "advisory");
        assert!((reservation.reserved_amount - 50000.0).abs() < 0.01);
        assert_eq!(reservation.status, "active");
        assert!(reservation.fund_check_passed);
    }

    #[tokio::test]
    async fn test_create_reservation_duplicate_conflict() {
        let engine = create_engine();
        // First creation succeeds (mock returns it)
        engine.create_reservation(
            test_org_id(), "FR-DUP", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            1000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), None,
            None, None, None,
        ).await.unwrap();

        // Mock returns existing reservation for duplicate check
        let result = engine.create_reservation(
            test_org_id(), "FR-DUP", test_budget_id(), "BUD-001", None,
            None, None, None, None,
            2000.0, "USD",
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None,
            "advisory", Some(2024), None,
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::Conflict(msg) => assert!(msg.contains("FR-DUP")),
            _ => panic!("Expected Conflict"),
        }
    }

    // --- Consume/Release Tests ---

    #[tokio::test]
    async fn test_consume_reservation_validation_negative() {
        let engine = create_engine();
        let result = engine.consume_reservation(Uuid::new_v4(), -100.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_consume_reservation_not_found() {
        let engine = create_engine();
        let result = engine.consume_reservation(Uuid::new_v4(), 100.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_release_reservation_validation_negative() {
        let engine = create_engine();
        let result = engine.release_reservation(Uuid::new_v4(), -100.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_release_reservation_not_found() {
        let engine = create_engine();
        let result = engine.release_reservation(Uuid::new_v4(), 100.0).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    // --- Cancel Tests ---

    #[tokio::test]
    async fn test_cancel_reservation_not_found() {
        let engine = create_engine();
        let result = engine.cancel_reservation(Uuid::new_v4(), None, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    // --- Status Update Tests ---

    #[tokio::test]
    async fn test_update_reservation_status_bad_status() {
        let engine = create_engine();
        let result = engine.update_reservation_status(Uuid::new_v4(), "unknown").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("status")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    // --- Reservation Line Tests ---

    #[tokio::test]
    async fn test_create_reservation_line_validation_empty_account() {
        let engine = create_engine();
        let result = engine.create_reservation_line(
            test_org_id(), Uuid::new_v4(), 1,
            "", None, None, None, None, None,
            1000.0,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Account code")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_line_validation_negative_amount() {
        let engine = create_engine();
        let result = engine.create_reservation_line(
            test_org_id(), Uuid::new_v4(), 1,
            "1000-100", None, None, None, None, None,
            -1000.0,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_reservation_line_reservation_not_found() {
        let engine = create_engine();
        let result = engine.create_reservation_line(
            test_org_id(), Uuid::new_v4(), 1,
            "1000-100", Some("Cash"), None, None, None, None,
            1000.0,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }
}
