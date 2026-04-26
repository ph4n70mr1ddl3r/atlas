//! Demand Planning Engine
//!
//! Oracle Fusion SCM: Demand Management.
//! Manages demand forecast methods, demand schedules (forecasts),
//! schedule lines, historical demand data, forecast consumption,
//! accuracy measurement, and demand planning analytics.
//!
//! The process follows Oracle Fusion Demand Management workflow:
//! 1. Define forecast methods (moving average, exponential smoothing, etc.)
//! 2. Create demand schedules (forecasts) with date ranges and methods
//! 3. Add schedule lines (item/period forecasts)
//! 4. Submit and approve schedules
//! 5. Record actual demand (history)
//! 6. Consume forecast against actuals
//! 7. Measure forecast accuracy (MAPE, bias)
//! 8. Analyze via dashboard

use atlas_shared::AtlasError;
use super::DemandPlanningRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid forecast method types
const VALID_METHOD_TYPES: &[&str] = &[
    "moving_average", "exponential_smoothing", "weighted_average",
    "regression", "manual",
];

/// Valid schedule types
const VALID_SCHEDULE_TYPES: &[&str] = &[
    "daily", "weekly", "monthly", "quarterly",
];

/// Valid schedule statuses
const VALID_SCHEDULE_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "active", "closed", "cancelled",
];

/// Valid confidence levels
const VALID_CONFIDENCE_LEVELS: &[&str] = &[
    "low", "medium", "high",
];

/// Valid source types for demand history
const VALID_SOURCE_TYPES: &[&str] = &[
    "sales_order", "manual", "shipment", "import",
];

/// Demand Planning engine
pub struct DemandPlanningEngine {
    repository: Arc<dyn DemandPlanningRepository>,
}

impl DemandPlanningEngine {
    pub fn new(repository: Arc<dyn DemandPlanningRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Forecast Method Management
    // ========================================================================

    /// Create a forecast method
    pub async fn create_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        method_type: &str,
        parameters: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandForecastMethod> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Method code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Method name is required".to_string()));
        }
        if !VALID_METHOD_TYPES.contains(&method_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid method_type '{}'. Must be one of: {}", method_type, VALID_METHOD_TYPES.join(", ")
            )));
        }
        if self.repository.get_method_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Forecast method '{}' already exists", code)));
        }
        info!("Creating forecast method '{}' for org {}", code, org_id);
        self.repository.create_method(org_id, &code, name, description, method_type, parameters, created_by).await
    }

    /// Get a method by ID
    pub async fn get_method(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::DemandForecastMethod>> {
        self.repository.get_method(id).await
    }

    /// List methods
    pub async fn list_methods(&self, org_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandForecastMethod>> {
        self.repository.list_methods(org_id).await
    }

    /// Delete a method
    pub async fn delete_method(&self, org_id: Uuid, code: &str) -> atlas_shared::AtlasResult<()> {
        info!("Deleting forecast method '{}' for org {}", code, org_id);
        self.repository.delete_method(org_id, code).await
    }

    // ========================================================================
    // Demand Schedule Management
    // ========================================================================

    /// Create a demand schedule
    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        method_id: Option<Uuid>,
        schedule_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        currency_code: &str,
        confidence_level: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        if schedule_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule name is required".to_string()));
        }
        if !VALID_SCHEDULE_TYPES.contains(&schedule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule_type '{}'. Must be one of: {}", schedule_type, VALID_SCHEDULE_TYPES.join(", ")
            )));
        }
        if !VALID_CONFIDENCE_LEVELS.contains(&confidence_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid confidence_level '{}'. Must be one of: {}", confidence_level, VALID_CONFIDENCE_LEVELS.join(", ")
            )));
        }
        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed("Start date must be before end date".to_string()));
        }

        // Resolve method name if method_id provided
        let method_name = if let Some(mid) = method_id {
            self.repository.get_method(mid).await?.map(|m| m.name.clone())
        } else {
            None
        };

        if self.repository.get_schedule_by_number(org_id, schedule_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Schedule '{}' already exists", schedule_number)));
        }

        info!("Creating demand schedule '{}' for org {}", schedule_number, org_id);
        self.repository.create_schedule(
            org_id, schedule_number, name, description,
            method_id, method_name.as_deref(), schedule_type,
            start_date, end_date, currency_code, confidence_level,
            owner_id, owner_name, created_by,
        ).await
    }

    /// Get a schedule by ID
    pub async fn get_schedule(&self, id: Uuid) -> atlas_shared::AtlasResult<Option<atlas_shared::DemandSchedule>> {
        self.repository.get_schedule(id).await
    }

    /// List schedules with optional status filter
    pub async fn list_schedules(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandSchedule>> {
        if let Some(s) = status {
            if !VALID_SCHEDULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SCHEDULE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_schedules(org_id, status).await
    }

    /// Submit a schedule for approval
    pub async fn submit_schedule(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        let schedule = self.repository.get_schedule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit schedule in '{}' status. Must be 'draft'.", schedule.status
            )));
        }
        info!("Submitting schedule {} for approval", schedule.schedule_number);
        self.repository.update_schedule_status(id, "submitted").await
    }

    /// Approve a schedule
    pub async fn approve_schedule(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        let schedule = self.repository.get_schedule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        if schedule.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve schedule in '{}' status. Must be 'submitted'.", schedule.status
            )));
        }
        info!("Approving schedule {}", schedule.schedule_number);
        self.repository.approve_schedule(id, approved_by).await
    }

    /// Activate an approved schedule
    pub async fn activate_schedule(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        let schedule = self.repository.get_schedule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        if schedule.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate schedule in '{}' status. Must be 'approved'.", schedule.status
            )));
        }
        info!("Activating schedule {}", schedule.schedule_number);
        self.repository.update_schedule_status(id, "active").await
    }

    /// Close a schedule
    pub async fn close_schedule(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        let schedule = self.repository.get_schedule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close schedule in '{}' status. Must be 'active'.", schedule.status
            )));
        }
        info!("Closing schedule {}", schedule.schedule_number);
        self.repository.update_schedule_status(id, "closed").await
    }

    /// Cancel a schedule
    pub async fn cancel_schedule(&self, id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::DemandSchedule> {
        let schedule = self.repository.get_schedule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
        if schedule.status == "closed" || schedule.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel schedule in '{}' status.", schedule.status
            )));
        }
        info!("Cancelling schedule {}", schedule.schedule_number);
        self.repository.update_schedule_status(id, "cancelled").await
    }

    /// Delete a schedule
    pub async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_schedule(org_id, schedule_number).await
    }

    // ========================================================================
    // Schedule Line Management
    // ========================================================================

    /// Add a schedule line
    pub async fn add_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        item_category: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        forecast_quantity: &str,
        unit_price: &str,
        confidence_pct: &str,
        notes: Option<&str>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandScheduleLine> {
        if item_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Item code is required".to_string()));
        }

        let qty: f64 = forecast_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Forecast quantity must be a number".to_string())
        })?;
        if qty < 0.0 {
            return Err(AtlasError::ValidationFailed("Forecast quantity cannot be negative".to_string()));
        }

        let price: f64 = unit_price.parse().unwrap_or(0.0);
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed("Unit price cannot be negative".to_string()));
        }

        let conf: f64 = confidence_pct.parse().unwrap_or(0.0);
        if !(0.0..=100.0).contains(&conf) {
            return Err(AtlasError::ValidationFailed("Confidence must be 0-100".to_string()));
        }

        if period_start >= period_end {
            return Err(AtlasError::ValidationFailed("Period start must be before period end".to_string()));
        }

        // Verify schedule exists
        let schedule = self.repository.get_schedule(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to schedule in '{}' status. Must be 'draft'.", schedule.status
            )));
        }

        // Get next line number
        let existing = self.repository.list_schedule_lines(schedule_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding line {} (item: {}) to schedule {}", line_number, item_code, schedule.schedule_number);

        let line = self.repository.add_schedule_line(
            org_id, schedule_id, line_number, item_code, item_name,
            item_category, warehouse_code, region, customer_group,
            period_start, period_end, forecast_quantity, unit_price, confidence_pct, notes,
        ).await?;

        // Update schedule totals
        let _ = self.repository.update_schedule_totals(schedule_id).await;

        Ok(line)
    }

    /// List schedule lines
    pub async fn list_schedule_lines(&self, schedule_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandScheduleLine>> {
        self.repository.list_schedule_lines(schedule_id).await
    }

    /// Delete a schedule line
    pub async fn delete_schedule_line(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        let line = self.repository.get_schedule_line(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule line {} not found", id)))?;

        // Check schedule status
        let schedule = self.repository.get_schedule(line.schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Schedule not found".to_string()))?;
        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete lines from non-draft schedule".to_string(),
            ));
        }

        self.repository.delete_schedule_line(id).await?;
        let _ = self.repository.update_schedule_totals(line.schedule_id).await;
        Ok(())
    }

    // ========================================================================
    // Demand History
    // ========================================================================

    /// Record historical demand
    pub async fn create_history(
        &self,
        org_id: Uuid,
        item_code: &str,
        item_name: Option<&str>,
        warehouse_code: Option<&str>,
        region: Option<&str>,
        customer_group: Option<&str>,
        actual_date: chrono::NaiveDate,
        actual_quantity: &str,
        actual_value: &str,
        source_type: &str,
        source_id: Option<Uuid>,
        source_line_id: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandHistory> {
        if item_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Item code is required".to_string()));
        }
        if !VALID_SOURCE_TYPES.contains(&source_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source_type '{}'. Must be one of: {}", source_type, VALID_SOURCE_TYPES.join(", ")
            )));
        }
        let qty: f64 = actual_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Actual quantity must be a number".to_string())
        })?;
        if qty < 0.0 {
            return Err(AtlasError::ValidationFailed("Actual quantity cannot be negative".to_string()));
        }
        let val: f64 = actual_value.parse().unwrap_or(0.0);
        if val < 0.0 {
            return Err(AtlasError::ValidationFailed("Actual value cannot be negative".to_string()));
        }

        info!("Recording demand history for item {} on {}", item_code, actual_date);
        self.repository.create_history(
            org_id, item_code, item_name, warehouse_code, region,
            customer_group, actual_date, actual_quantity, actual_value,
            source_type, source_id, source_line_id,
        ).await
    }

    /// List demand history
    pub async fn list_history(
        &self,
        org_id: Uuid,
        item_code: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandHistory>> {
        self.repository.list_history(org_id, item_code, start_date, end_date).await
    }

    /// Delete demand history
    pub async fn delete_history(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_history(id).await
    }

    // ========================================================================
    // Forecast Consumption
    // ========================================================================

    /// Consume forecast against actual demand
    pub async fn consume_forecast(
        &self,
        org_id: Uuid,
        schedule_line_id: Uuid,
        history_id: Option<Uuid>,
        consumed_quantity: &str,
        consumed_date: chrono::NaiveDate,
        source_type: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandConsumption> {
        let line = self.repository.get_schedule_line(schedule_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule line {} not found", schedule_line_id)))?;

        let remaining: f64 = line.remaining_quantity.parse().unwrap_or(0.0);
        let consume_qty: f64 = consumed_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Consumed quantity must be a number".to_string())
        })?;
        if consume_qty <= 0.0 {
            return Err(AtlasError::ValidationFailed("Consumed quantity must be positive".to_string()));
        }
        if consume_qty > remaining {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot consume {} – only {} remaining", consume_qty, remaining
            )));
        }

        info!("Consuming {} from schedule line for item {}", consume_qty, line.item_code);
        self.repository.create_consumption(
            org_id, schedule_line_id, history_id,
            consumed_quantity, consumed_date, source_type, notes, created_by,
        ).await
    }

    /// List consumption entries for a schedule line
    pub async fn list_consumption(&self, schedule_line_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandConsumption>> {
        self.repository.list_consumption(schedule_line_id).await
    }

    /// Delete a consumption entry
    pub async fn delete_consumption(&self, id: Uuid) -> atlas_shared::AtlasResult<()> {
        self.repository.delete_consumption(id).await
    }

    // ========================================================================
    // Accuracy Measurement
    // ========================================================================

    /// Measure forecast accuracy for a schedule line
    pub async fn measure_accuracy(
        &self,
        org_id: Uuid,
        schedule_line_id: Uuid,
        actual_quantity: &str,
        measurement_date: Option<chrono::NaiveDate>,
    ) -> atlas_shared::AtlasResult<atlas_shared::DemandAccuracy> {
        let line = self.repository.get_schedule_line(schedule_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule line {} not found", schedule_line_id)))?;

        let forecast_qty: f64 = line.forecast_quantity.parse().unwrap_or(0.0);
        let actual_qty: f64 = actual_quantity.parse().map_err(|_| {
            AtlasError::ValidationFailed("Actual quantity must be a number".to_string())
        })?;

        let absolute_error = (forecast_qty - actual_qty).abs();
        let absolute_pct_error = if actual_qty != 0.0 {
            (absolute_error / actual_qty) * 100.0
        } else if forecast_qty != 0.0 {
            100.0 // 100% error if forecast was nonzero but actual was zero
        } else {
            0.0 // both zero = perfect
        };
        let bias = forecast_qty - actual_qty;

        let measurement_date = measurement_date.unwrap_or(chrono::Utc::now().date_naive());

        info!("Measuring accuracy for item {} – forecast: {}, actual: {}, MAPE: {:.1}%",
              line.item_code, forecast_qty, actual_qty, absolute_pct_error);

        self.repository.create_accuracy(
            org_id, line.schedule_id, Some(schedule_line_id),
            &line.item_code, line.period_start, line.period_end,
            &format!("{:.2}", forecast_qty),
            &format!("{:.2}", actual_qty),
            &format!("{:.2}", absolute_error),
            &format!("{:.4}", absolute_pct_error),
            &format!("{:.2}", bias),
            measurement_date,
        ).await
    }

    /// List accuracy measurements for a schedule
    pub async fn list_accuracy(&self, schedule_id: Uuid) -> atlas_shared::AtlasResult<Vec<atlas_shared::DemandAccuracy>> {
        self.repository.list_accuracy(schedule_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get demand planning dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> atlas_shared::AtlasResult<atlas_shared::DemandPlanningDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_method_types() {
        assert!(VALID_METHOD_TYPES.contains(&"moving_average"));
        assert!(VALID_METHOD_TYPES.contains(&"exponential_smoothing"));
        assert!(VALID_METHOD_TYPES.contains(&"weighted_average"));
        assert!(VALID_METHOD_TYPES.contains(&"regression"));
        assert!(VALID_METHOD_TYPES.contains(&"manual"));
        assert!(!VALID_METHOD_TYPES.contains(&"neural_network"));
    }

    #[test]
    fn test_valid_schedule_types() {
        assert!(VALID_SCHEDULE_TYPES.contains(&"daily"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"weekly"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"monthly"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"quarterly"));
        assert!(!VALID_SCHEDULE_TYPES.contains(&"yearly"));
    }

    #[test]
    fn test_valid_schedule_statuses() {
        assert!(VALID_SCHEDULE_STATUSES.contains(&"draft"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"submitted"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"approved"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"active"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"closed"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"cancelled"));
        assert!(!VALID_SCHEDULE_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_confidence_levels() {
        assert!(VALID_CONFIDENCE_LEVELS.contains(&"low"));
        assert!(VALID_CONFIDENCE_LEVELS.contains(&"medium"));
        assert!(VALID_CONFIDENCE_LEVELS.contains(&"high"));
        assert!(!VALID_CONFIDENCE_LEVELS.contains(&"very_high"));
    }

    #[test]
    fn test_valid_source_types() {
        assert!(VALID_SOURCE_TYPES.contains(&"sales_order"));
        assert!(VALID_SOURCE_TYPES.contains(&"manual"));
        assert!(VALID_SOURCE_TYPES.contains(&"shipment"));
        assert!(VALID_SOURCE_TYPES.contains(&"import"));
        assert!(!VALID_SOURCE_TYPES.contains(&"forecast"));
    }

    #[test]
    fn test_forecast_value_calculation() {
        let qty: f64 = 1000.0;
        let price: f64 = 25.50;
        let value = qty * price;
        assert!((value - 25500.0).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_mape_calculation() {
        let forecast: f64 = 100.0;
        let actual: f64 = 90.0;
        let abs_error = (forecast - actual).abs();
        let mape = if actual != 0.0 { (abs_error / actual) * 100.0 } else { 100.0 };
        assert!((mape - 11.1111).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_zero_actual() {
        let forecast: f64 = 100.0;
        let actual: f64 = 0.0;
        let abs_error = (forecast - actual).abs();
        let mape = if actual != 0.0 { (abs_error / actual) * 100.0 } else if forecast != 0.0 { 100.0 } else { 0.0 };
        assert!((mape - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_both_zero() {
        let forecast: f64 = 0.0;
        let actual: f64 = 0.0;
        let abs_error = (forecast - actual).abs();
        let mape = if actual != 0.0 { (abs_error / actual) * 100.0 } else if forecast != 0.0 { 100.0 } else { 0.0 };
        assert!((mape - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_bias_calculation() {
        let forecast: f64 = 100.0;
        let actual: f64 = 110.0;
        let bias = forecast - actual;
        assert!((bias - (-10.0)).abs() < 0.01);

        let forecast: f64 = 120.0;
        let actual: f64 = 100.0;
        let bias = forecast - actual;
        assert!((bias - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_accuracy_pct_from_mape() {
        let mape = 15.5;
        let accuracy = 100.0 - mape;
        assert!((accuracy - 84.5f64).abs() < 0.01);
    }

    #[test]
    fn test_confidence_range() {
        assert!(!(0.0..=100.0).contains(&-1.0));
        assert!((0.0..=100.0).contains(&0.0));
        assert!((0.0..=100.0).contains(&50.0));
        assert!((0.0..=100.0).contains(&100.0));
        assert!(!(0.0..=100.0).contains(&101.0));
    }
}
