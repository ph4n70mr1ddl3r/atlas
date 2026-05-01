//! Asset Depreciation Engine
//!
//! Implements the three standard depreciation methods used in Oracle Fusion:
//! 1. Straight-Line (SL): Even depreciation across useful life
//! 2. Declining Balance (DB): Accelerated depreciation with a multiplier
//! 3. Sum-of-Years-Digits (SYD): Accelerated depreciation with declining fraction
//!
//! Also supports:
//! - Generating full depreciation schedules for an asset
//! - Running single-period depreciation
//! - Calculating net book value at any point
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Fixed Assets > Depreciation

use atlas_shared::{
    DepreciationResult, DepreciationSchedule, AssetDepreciationHistory, FixedAsset,
    AtlasError, AtlasResult,
};
use super::AssetDepreciationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use chrono::Datelike;

/// Valid depreciation methods
const VALID_DEPRECIATION_METHODS: &[&str] = &[
    "straight_line", "declining_balance", "sum_of_years_digits",
];

/// Asset Depreciation Engine
pub struct AssetDepreciationEngine {
    repository: Arc<dyn AssetDepreciationRepository>,
}

impl AssetDepreciationEngine {
    pub fn new(repository: Arc<dyn AssetDepreciationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Depreciation Calculation
    // ========================================================================

    /// Calculate depreciation for a single period using the specified method
    pub fn calculate_period_depreciation(
        &self,
        method: &str,
        depreciable_basis: f64,
        salvage_value: f64,
        useful_life_months: i32,
        periods_depreciated: i32,
        declining_balance_rate: Option<f64>,
    ) -> AtlasResult<f64> {
        if !VALID_DEPRECIATION_METHODS.contains(&method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid depreciation method '{}'. Must be one of: {}",
                method, VALID_DEPRECIATION_METHODS.join(", ")
            )));
        }
        if depreciable_basis < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Depreciable basis must be non-negative".to_string(),
            ));
        }
        if useful_life_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Useful life must be positive".to_string(),
            ));
        }
        if periods_depreciated >= useful_life_months {
            return Ok(0.0); // Fully depreciated
        }

        let depreciable_amount = depreciable_basis - salvage_value;
        if depreciable_amount <= 0.0 {
            return Ok(0.0);
        }

        let depreciation = match method {
            "straight_line" => {
                let monthly_depreciation = depreciable_amount / (useful_life_months as f64);
                // Adjust last period for rounding
                let remaining_periods = useful_life_months - periods_depreciated;
                if remaining_periods == 1 {
                    // Last period gets the remainder
                    let total_so_far = monthly_depreciation * (periods_depreciated as f64);
                    depreciable_amount - total_so_far
                } else {
                    monthly_depreciation
                }
            }
            "declining_balance" => {
                let rate = declining_balance_rate.unwrap_or(2.0); // Default: double declining
                let monthly_rate = rate / (useful_life_months as f64);
                let book_value = depreciable_basis - (depreciable_amount / (useful_life_months as f64) * (periods_depreciated as f64));
                let book_value_for_calc = book_value.max(salvage_value);
                let mut dep = book_value_for_calc * monthly_rate;
                // Don't depreciate below salvage value
                let current_book = depreciable_basis
                    - (depreciable_amount / (useful_life_months as f64) * (periods_depreciated as f64));
                if current_book - dep < salvage_value {
                    dep = (current_book - salvage_value).max(0.0);
                }
                dep
            }
            "sum_of_years_digits" => {
                let useful_life_years = (useful_life_months as f64) / 12.0;
                let years = Self::round_up(useful_life_years);
                let sum_of_years = (years * (years + 1.0)) / 2.0;
                if sum_of_years <= 0.0 {
                    return Ok(0.0);
                }
                // Convert periods_depreciated to years (fractional)
                let current_year = ((periods_depreciated as f64) / 12.0) + 1.0;
                let remaining_years = years - current_year + 1.0;
                let annual_depreciation = depreciable_amount * (remaining_years / sum_of_years);
                let monthly = annual_depreciation / 12.0;
                monthly.max(0.0)
            }
            _ => return Err(AtlasError::ValidationFailed(
                format!("Unknown depreciation method: {}", method)
            )),
        };

        Ok(depreciation.max(0.0))
    }

    /// Generate a complete depreciation schedule for an asset
    pub fn generate_schedule(
        &self,
        asset_id: Uuid,
        asset_number: &str,
        asset_name: &str,
        original_cost: f64,
        salvage_value: f64,
        useful_life_months: i32,
        depreciation_method: &str,
        declining_balance_rate: Option<f64>,
        start_date: chrono::NaiveDate,
    ) -> AtlasResult<DepreciationSchedule> {
        if useful_life_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Useful life must be positive".to_string(),
            ));
        }

        let depreciable_basis = original_cost - salvage_value;

        let mut periods = Vec::new();
        let mut accumulated = 0.0;
        let mut current_date = start_date;

        for period in 0..useful_life_months {
            let dep = self.calculate_period_depreciation(
                depreciation_method,
                original_cost,
                salvage_value,
                useful_life_months,
                period,
                declining_balance_rate,
            )?;

            accumulated += dep;
            let nbv = (original_cost - accumulated).max(salvage_value);

            periods.push(DepreciationResult {
                asset_id,
                fiscal_year: current_date.year(),
                period_number: period + 1,
                depreciation_date: current_date,
                depreciation_amount: format!("{:.2}", dep),
                accumulated_depreciation: format!("{:.2}", accumulated),
                net_book_value: format!("{:.2}", nbv),
                depreciation_method: depreciation_method.to_string(),
            });

            // Advance to next month
            current_date = add_one_month(current_date);
        }

        let total_depreciation = depreciable_basis;

        Ok(DepreciationSchedule {
            asset_id,
            asset_number: asset_number.to_string(),
            asset_name: asset_name.to_string(),
            original_cost: format!("{:.2}", original_cost),
            salvage_value: format!("{:.2}", salvage_value),
            depreciable_basis: format!("{:.2}", depreciable_basis),
            useful_life_months,
            depreciation_method: depreciation_method.to_string(),
            declining_balance_rate: declining_balance_rate.map(|r| format!("{:.4}", r)),
            periods,
            total_depreciation: format!("{:.2}", total_depreciation),
        })
    }

    /// Run depreciation for a single asset for one period and persist the result
    pub async fn run_depreciation(
        &self,
        asset_id: Uuid,
        fiscal_year: i32,
        period_number: i32,
        depreciation_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDepreciationHistory> {
        let asset = self.repository.get_asset(asset_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Asset {} not found", asset_id)
            ))?;

        if asset.status != "in_service" && asset.status != "acquired" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot depreciate asset in '{}' status", asset.status)
            ));
        }

        let original_cost: f64 = asset.original_cost.parse().unwrap_or(0.0);
        let salvage_value: f64 = asset.salvage_value.parse().unwrap_or(0.0);
        let periods_depreciated = asset.periods_depreciated;
        let useful_life = asset.useful_life_months;
        let db_rate: Option<f64> = asset.declining_balance_rate.as_ref().and_then(|s| s.parse().ok());

        let dep_amount = self.calculate_period_depreciation(
            &asset.depreciation_method,
            original_cost,
            salvage_value,
            useful_life,
            periods_depreciated,
            db_rate,
        )?;

        let prev_accumulated: f64 = asset.accumulated_depreciation.parse().unwrap_or(0.0);
        let new_accumulated = prev_accumulated + dep_amount;
        let nbv = (original_cost - new_accumulated).max(salvage_value);

        info!("Running depreciation for asset {} period {}/{}: amount={:.2}, nbv={:.2}",
            asset.asset_number, period_number, fiscal_year, dep_amount, nbv);

        let history = self.repository.create_depreciation_history(
            asset.organization_id,
            asset_id,
            fiscal_year,
            period_number,
            None,
            depreciation_date,
            &format!("{:.2}", dep_amount),
            &format!("{:.2}", new_accumulated),
            &format!("{:.2}", nbv),
            &asset.depreciation_method,
            created_by,
        ).await?;

        // Update asset record
        self.repository.update_asset_depreciation(
            asset_id,
            &format!("{:.2}", new_accumulated),
            &format!("{:.2}", nbv),
            &format!("{:.2}", dep_amount),
            periods_depreciated + 1,
            depreciation_date,
        ).await?;

        Ok(history)
    }

    /// Get depreciation history for an asset
    pub async fn get_depreciation_history(&self, asset_id: Uuid) -> AtlasResult<Vec<AssetDepreciationHistory>> {
        self.repository.list_depreciation_history(asset_id).await
    }

    /// Get net book value of an asset at a given date
    pub fn calculate_net_book_value(
        &self,
        original_cost: f64,
        accumulated_depreciation: f64,
        salvage_value: f64,
    ) -> f64 {
        (original_cost - accumulated_depreciation).max(salvage_value)
    }

    /// Round up to nearest integer
    fn round_up(v: f64) -> f64 {
        if v == v.floor() { v } else { v.floor() + 1.0 }
    }
}

/// Add one month to a date
fn add_one_month(date: chrono::NaiveDate) -> chrono::NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + 1;
    if month > 12 {
        month = 1;
        year += 1;
    }
    let day = date.day().min(days_in_month(year, month as u32));
    chrono::NaiveDate::from_ymd_opt(year, month as u32, day).unwrap_or(date)
}

/// Days in month helper
fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        let dec1 = chrono::NaiveDate::from_ymd_opt(year, 12, 1).unwrap();
        let jan1 = chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap();
        (jan1 - dec1).num_days() as u32
    } else {
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next = chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap();
        (next - first).num_days() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_depreciation_methods() {
        assert!(VALID_DEPRECIATION_METHODS.contains(&"straight_line"));
        assert!(VALID_DEPRECIATION_METHODS.contains(&"declining_balance"));
        assert!(VALID_DEPRECIATION_METHODS.contains(&"sum_of_years_digits"));
        assert_eq!(VALID_DEPRECIATION_METHODS.len(), 3);
    }

    #[test]
    fn test_straight_line_depreciation() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // $12,000 asset, $0 salvage, 12 months = $1,000/month
        let dep = engine.calculate_period_depreciation(
            "straight_line", 12000.0, 0.0, 12, 0, None,
        ).unwrap();
        assert!((dep - 1000.0).abs() < 0.01, "Expected ~1000.0, got {}", dep);

        // $10,000 asset, $1,000 salvage, 36 months = $250/month
        let dep = engine.calculate_period_depreciation(
            "straight_line", 10000.0, 1000.0, 36, 0, None,
        ).unwrap();
        assert!((dep - 250.0).abs() < 0.01, "Expected ~250.0, got {}", dep);
    }

    #[test]
    fn test_straight_line_last_period() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // 12 months, already depreciated 11 periods, last period gets remainder
        let dep = engine.calculate_period_depreciation(
            "straight_line", 10000.0, 0.0, 12, 11, None,
        ).unwrap();
        // Remaining = 10000 - 11*833.33 = 10000 - 9166.63 = 833.37
        assert!(dep > 0.0, "Last period should have positive depreciation");
        assert!(dep < 1200.0, "Last period should be close to normal period");
    }

    #[test]
    fn test_straight_line_fully_depreciated() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // Already fully depreciated
        let dep = engine.calculate_period_depreciation(
            "straight_line", 10000.0, 0.0, 12, 12, None,
        ).unwrap();
        assert_eq!(dep, 0.0, "Fully depreciated asset should return 0");
    }

    #[test]
    fn test_declining_balance_depreciation() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // Double declining balance: $10,000, 60 months
        let dep = engine.calculate_period_depreciation(
            "declining_balance", 10000.0, 0.0, 60, 0, Some(2.0),
        ).unwrap();
        // First period: 10000 * (2.0 / 60) = 333.33
        assert!(dep > 0.0, "Should have positive depreciation");
        assert!(dep < 500.0, "First period should be reasonable");
    }

    #[test]
    fn test_sum_of_years_digits() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // $15,000, $0 salvage, 60 months (5 years)
        // Sum of years = 5+4+3+2+1 = 15
        // Year 1 depreciation = 15000 * (5/15) = 5000, monthly = 416.67
        let dep = engine.calculate_period_depreciation(
            "sum_of_years_digits", 15000.0, 0.0, 60, 0, None,
        ).unwrap();
        assert!(dep > 300.0, "SYD first year monthly should be > 300");
        assert!(dep < 500.0, "SYD first year monthly should be < 500");
    }

    #[test]
    fn test_zero_depreciable_basis() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        // Cost = salvage value
        let dep = engine.calculate_period_depreciation(
            "straight_line", 5000.0, 5000.0, 60, 0, None,
        ).unwrap();
        assert_eq!(dep, 0.0, "No depreciation when cost equals salvage");
    }

    #[test]
    fn test_invalid_method() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        let result = engine.calculate_period_depreciation(
            "invalid_method", 10000.0, 0.0, 60, 0, None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_useful_life() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        let result = engine.calculate_period_depreciation(
            "straight_line", 10000.0, 0.0, 0, 0, None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_schedule_straight_line() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));
        let asset_id = Uuid::new_v4();
        let start = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let schedule = engine.generate_schedule(
            asset_id, "ASSET-001", "Test Asset",
            12000.0, 0.0, 12, "straight_line", None, start,
        ).unwrap();

        assert_eq!(schedule.periods.len(), 12);
        assert_eq!(schedule.asset_number, "ASSET-001");
        assert_eq!(schedule.asset_name, "Test Asset");

        // First period
        let first = &schedule.periods[0];
        assert!((first.depreciation_amount.parse::<f64>().unwrap() - 1000.0).abs() < 0.01);

        // Last period should bring accumulated to total
        let last = &schedule.periods.last().unwrap();
        let total_accum: f64 = last.accumulated_depreciation.parse().unwrap();
        assert!((total_accum - 12000.0).abs() < 1.0, "Total accumulated should be ~12000, got {}", total_accum);
    }

    #[test]
    fn test_generate_schedule_with_salvage() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));
        let asset_id = Uuid::new_v4();
        let start = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let schedule = engine.generate_schedule(
            asset_id, "ASSET-002", "Salvage Asset",
            10000.0, 1000.0, 36, "straight_line", None, start,
        ).unwrap();

        assert_eq!(schedule.periods.len(), 36);

        // Total depreciation should be ~9000 (10000-1000)
        let total_dep: f64 = schedule.periods.iter()
            .map(|p| p.depreciation_amount.parse::<f64>().unwrap())
            .sum();
        assert!((total_dep - 9000.0).abs() < 1.0, "Total depreciation should be ~9000, got {}", total_dep);

        // Net book value at end should be >= salvage
        let last_nbv: f64 = schedule.periods.last().unwrap().net_book_value.parse().unwrap();
        assert!(last_nbv >= 999.0, "Final NBV should be >= salvage value");
    }

    #[test]
    fn test_calculate_net_book_value() {
        let engine = AssetDepreciationEngine::new(Arc::new(crate::mock_repos::MockAssetDepreciationRepository));

        let nbv = engine.calculate_net_book_value(10000.0, 6000.0, 0.0);
        assert!((nbv - 4000.0).abs() < 0.01);

        // Should not go below salvage
        let nbv = engine.calculate_net_book_value(10000.0, 9500.0, 1000.0);
        assert!((nbv - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_add_one_month() {
        let jan1 = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let feb1 = add_one_month(jan1);
        assert_eq!(feb1, chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());

        let jan31 = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let feb29 = add_one_month(jan31);
        assert_eq!(feb29, chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()); // Leap year

        let dec15 = chrono::NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let jan15 = add_one_month(dec15);
        assert_eq!(jan15, chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }
}
