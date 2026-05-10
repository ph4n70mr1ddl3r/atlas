//! Transaction Calendar Engine
//!
//! Manages transaction calendar lifecycle, business day calculations,
//! holiday management, and date calculation audit trail.
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Transaction Calendars

use atlas_shared::{
    TransactionCalendar, CalendarException, CalendarDateCalculation,
    TransactionCalendarDashboard,
    AtlasError, AtlasResult,
};
use super::TransactionCalendarRepository;
use chrono::{Datelike, Duration, NaiveDate};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid exception types
const VALID_EXCEPTION_TYPES: &[&str] = &["holiday", "non_working", "special_working"];

/// Valid calendar statuses
#[allow(dead_code)]
const VALID_STATUSES: &[&str] = &["active", "inactive"];

/// Default working days: Monday through Friday
const DEFAULT_WORKING_DAYS: &[i32] = &[1, 2, 3, 4, 5];

/// Transaction Calendar Engine
pub struct TransactionCalendarEngine {
    repository: Arc<dyn TransactionCalendarRepository>,
}

impl TransactionCalendarEngine {
    pub fn new(repository: Arc<dyn TransactionCalendarRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Calendar CRUD
    // ========================================================================

    /// Create a new transaction calendar
    pub async fn create_calendar(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        working_days: Option<&serde_json::Value>,
        effective_from: Option<NaiveDate>,
        effective_to: Option<NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionCalendar> {
        // Validate inputs
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Calendar code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Calendar name is required".to_string()));
        }

        // Validate and normalize working days
        let working_days = working_days.cloned().unwrap_or_else(|| {
            serde_json::Value::Array(
                DEFAULT_WORKING_DAYS.iter().map(|d| serde_json::Value::from(*d)).collect()
            )
        });
        Self::validate_working_days(&working_days)?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        // Check uniqueness
        if self.repository.get_calendar(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Transaction calendar with code '{}' already exists", code
            )));
        }

        info!("Creating transaction calendar {} ({}) for org {}", code, name, org_id);

        self.repository.create_calendar(
            org_id, code, name, description, &working_days,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a calendar by code
    pub async fn get_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TransactionCalendar>> {
        self.repository.get_calendar(org_id, code).await
    }

    /// Get a calendar by ID
    pub async fn get_calendar_by_id(&self, id: Uuid) -> AtlasResult<Option<TransactionCalendar>> {
        self.repository.get_calendar_by_id(id).await
    }

    /// List calendars with optional status filter
    pub async fn list_calendars(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TransactionCalendar>> {
        self.repository.list_calendars(org_id, status).await
    }

    /// Activate a calendar
    pub async fn activate_calendar(&self, id: Uuid) -> AtlasResult<TransactionCalendar> {
        let cal = self.get_calendar_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Calendar {} not found", id)))?;

        if cal.status == "active" {
            return Err(AtlasError::WorkflowError("Calendar is already active".to_string()));
        }

        info!("Activated transaction calendar {}", cal.code);
        self.repository.update_calendar_status(id, "active").await
    }

    /// Deactivate a calendar
    pub async fn deactivate_calendar(&self, id: Uuid) -> AtlasResult<TransactionCalendar> {
        let cal = self.get_calendar_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Calendar {} not found", id)))?;

        if cal.status == "inactive" {
            return Err(AtlasError::WorkflowError("Calendar is already inactive".to_string()));
        }

        info!("Deactivated transaction calendar {}", cal.code);
        self.repository.update_calendar_status(id, "inactive").await
    }

    /// Delete a calendar (only if no exceptions exist)
    pub async fn delete_calendar(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let cal = self.repository.get_calendar(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Calendar '{}' not found", code)))?;

        // Check for exceptions
        let exceptions = self.repository.list_exceptions(cal.id).await?;
        if !exceptions.is_empty() {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot delete calendar '{}' - it has {} exception(s). Remove exceptions first.",
                code, exceptions.len()
            )));
        }

        info!("Deleted transaction calendar {}", code);
        self.repository.delete_calendar(org_id, code).await
    }

    // ========================================================================
    // Exception Management
    // ========================================================================

    /// Add a calendar exception (holiday, non-working day, or special working day)
    pub async fn create_exception(
        &self,
        org_id: Uuid,
        calendar_id: Uuid,
        exception_date: NaiveDate,
        exception_type: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CalendarException> {
        // Validate
        if !VALID_EXCEPTION_TYPES.contains(&exception_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid exception type '{}'. Must be one of: {}",
                exception_type, VALID_EXCEPTION_TYPES.join(", ")
            )));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Exception name is required".to_string()));
        }

        // Verify calendar exists and is active
        let cal = self.repository.get_calendar_by_id(calendar_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Calendar {} not found", calendar_id
            )))?;

        if cal.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add exceptions to inactive calendar '{}'", cal.code
            )));
        }

        // Check for duplicate exception date
        if self.repository.get_exception(calendar_id, exception_date).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "An exception already exists for calendar '{}' on date {}",
                cal.code, exception_date
            )));
        }

        info!(
            "Adding {} exception '{}' for calendar {} on {}",
            exception_type, name, cal.code, exception_date
        );

        self.repository.create_exception(
            org_id, calendar_id, exception_date, exception_type,
            name, description, created_by,
        ).await
    }

    /// Get an exception by calendar and date
    pub async fn get_exception(
        &self,
        calendar_id: Uuid,
        exception_date: NaiveDate,
    ) -> AtlasResult<Option<CalendarException>> {
        self.repository.get_exception(calendar_id, exception_date).await
    }

    /// List exceptions for a calendar
    pub async fn list_exceptions(
        &self,
        calendar_id: Uuid,
    ) -> AtlasResult<Vec<CalendarException>> {
        self.repository.list_exceptions(calendar_id).await
    }

    /// List exceptions for a calendar within a date range
    pub async fn list_exceptions_range(
        &self,
        calendar_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> AtlasResult<Vec<CalendarException>> {
        self.repository.list_exceptions_range(calendar_id, from_date, to_date).await
    }

    /// Delete an exception
    pub async fn delete_exception(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted calendar exception {}", id);
        self.repository.delete_exception(id).await
    }

    // ========================================================================
    // Business Day Calculations
    // ========================================================================

    /// Check if a date is a business day according to the calendar.
    /// A business day is:
    /// - A working day (as defined by the calendar's working_days)
    /// - NOT a holiday or non-working exception
    /// - OR IS a special_working exception (overrides non-working day)
    pub async fn is_business_day(
        &self,
        calendar_id: Uuid,
        date: NaiveDate,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<bool> {
        let cal = self.repository.get_calendar_by_id(calendar_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Calendar {} not found", calendar_id
            )))?;

        let result = Self::calculate_is_business_day(&cal, date, &self.repository).await?;

        // Log the calculation
        let _ = self.repository.create_calculation(
            cal.organization_id, calendar_id, &cal.code,
            "is_business_day", date, None, Some(result), None,
            reference_type, reference_id, calculated_by,
        ).await;

        Ok(result)
    }

    /// Get the next business day after the given date.
    pub async fn next_business_day(
        &self,
        calendar_id: Uuid,
        date: NaiveDate,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<NaiveDate> {
        let cal = self.repository.get_calendar_by_id(calendar_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Calendar {} not found", calendar_id
            )))?;

        let mut current = date + Duration::days(1);
        // Safety limit to prevent infinite loops
        for _ in 0..366 {
            if Self::calculate_is_business_day(&cal, current, &self.repository).await? {
                let _ = self.repository.create_calculation(
                    cal.organization_id, calendar_id, &cal.code,
                    "next_business_day", date, Some(current), None, None,
                    reference_type, reference_id, calculated_by,
                ).await;
                return Ok(current);
            }
            current += Duration::days(1);
        }

        Err(AtlasError::WorkflowError(
            "Could not find next business day within 366 days".to_string()
        ))
    }

    /// Get the previous business day before the given date.
    pub async fn previous_business_day(
        &self,
        calendar_id: Uuid,
        date: NaiveDate,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<NaiveDate> {
        let cal = self.repository.get_calendar_by_id(calendar_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Calendar {} not found", calendar_id
            )))?;

        let mut current = date - Duration::days(1);
        for _ in 0..366 {
            if Self::calculate_is_business_day(&cal, current, &self.repository).await? {
                let _ = self.repository.create_calculation(
                    cal.organization_id, calendar_id, &cal.code,
                    "previous_business_day", date, Some(current), None, None,
                    reference_type, reference_id, calculated_by,
                ).await;
                return Ok(current);
            }
            current -= Duration::days(1);
        }

        Err(AtlasError::WorkflowError(
            "Could not find previous business day within 366 days".to_string()
        ))
    }

    /// Add N business days to a date.
    pub async fn add_business_days(
        &self,
        calendar_id: Uuid,
        start_date: NaiveDate,
        days: i32,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        calculated_by: Option<Uuid>,
    ) -> AtlasResult<NaiveDate> {
        if days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Days must be non-negative. Use subtract_business_days for negative offsets.".to_string()
            ));
        }

        let cal = self.repository.get_calendar_by_id(calendar_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Calendar {} not found", calendar_id
            )))?;

        let mut current = start_date;
        let mut remaining = days;

        for _ in 0..(days as i64 + 366) {
            if remaining == 0 {
                break;
            }
            current += Duration::days(1);
            if Self::calculate_is_business_day(&cal, current, &self.repository).await? {
                remaining -= 1;
            }
        }

        let _ = self.repository.create_calculation(
            cal.organization_id, calendar_id, &cal.code,
            "add_business_days", start_date, Some(current), None, Some(days),
            reference_type, reference_id, calculated_by,
        ).await;

        Ok(current)
    }

    // ========================================================================
    // Audit Trail
    // ========================================================================

    /// List calculation audit entries
    pub async fn list_calculations(
        &self,
        org_id: Uuid,
        calendar_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<CalendarDateCalculation>> {
        self.repository.list_calculations(org_id, calendar_id, limit).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<TransactionCalendarDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Internal business day check that combines working day pattern and exceptions.
    async fn calculate_is_business_day(
        cal: &TransactionCalendar,
        date: NaiveDate,
        repo: &Arc<dyn TransactionCalendarRepository>,
    ) -> AtlasResult<bool> {
        let weekday = date.weekday().num_days_from_monday() as i32 + 1; // 1=Mon, 7=Sun

        // Check working_days pattern
        let is_working_day = cal.working_days.as_array()
            .map(|arr| arr.iter().any(|v| v.as_i64() == Some(weekday as i64)))
            .unwrap_or(false);

        // Check for exceptions on this date
        if let Some(exc) = repo.get_exception(cal.id, date).await? {
            match exc.exception_type.as_str() {
                "holiday" | "non_working" => return Ok(false),
                "special_working" => return Ok(true),
                _ => {}
            }
        }

        Ok(is_working_day)
    }

    /// Validate working_days JSON value
    pub fn validate_working_days(working_days: &serde_json::Value) -> AtlasResult<()> {
        match working_days.as_array() {
            Some(arr) => {
                if arr.is_empty() {
                    return Err(AtlasError::ValidationFailed(
                        "Working days cannot be empty".to_string(),
                    ));
                }
                for v in arr {
                    match v.as_i64() {
                        Some(d) if (1..=7).contains(&d) => {}
                        _ => return Err(AtlasError::ValidationFailed(format!(
                            "Invalid working day value: {}. Must be 1-7 (Mon-Sun)", v
                        ))),
                    }
                }
                Ok(())
            }
            None => Err(AtlasError::ValidationFailed(
                "Working days must be a JSON array".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_working_days_valid() {
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([1, 2, 3, 4, 5])
        ).is_ok());
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([1, 2, 3, 4, 5, 6])
        ).is_ok());
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([1, 7])
        ).is_ok());
    }

    #[test]
    fn test_validate_working_days_empty() {
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([])
        ).is_err());
    }

    #[test]
    fn test_validate_working_days_invalid_value() {
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([0, 1, 2])
        ).is_err());
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!([8])
        ).is_err());
    }

    #[test]
    fn test_validate_working_days_non_array() {
        assert!(TransactionCalendarEngine::validate_working_days(
            &serde_json::json!("Mon-Fri")
        ).is_err());
    }

    #[test]
    fn test_valid_exception_types() {
        assert!(VALID_EXCEPTION_TYPES.contains(&"holiday"));
        assert!(VALID_EXCEPTION_TYPES.contains(&"non_working"));
        assert!(VALID_EXCEPTION_TYPES.contains(&"special_working"));
    }
}
