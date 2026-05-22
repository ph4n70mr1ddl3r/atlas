use atlas_shared::{AtlasResult, AtlasError, RecordId};
use chrono::NaiveDate;
use std::collections::BTreeMap;
use rust_decimal::Decimal;

/// Represents a daily balance record for an account.
#[derive(Debug, Clone)]
pub struct DailyBalance {
    pub date: NaiveDate,
    pub end_of_day_balance: Decimal,
}

/// Service to calculate Average Daily Balances (ADB), 
/// inspired by Oracle Fusion Financials General Ledger features.
pub struct AverageDailyBalanceService {
    // In a real implementation, this would connect to the database.
    // For this demonstration, we'll keep it simple.
}

impl AverageDailyBalanceService {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculates the Average Daily Balance for a given period.
    /// It takes a list of daily balances and the start and end dates of the period.
    /// If there are missing days in the list, it assumes the balance carries over from the last known day.
    pub fn calculate_average_balance(
        &self,
        _account_id: RecordId,
        period_start: NaiveDate,
        period_end: NaiveDate,
        daily_balances: &[DailyBalance],
        opening_balance: Decimal,
    ) -> AtlasResult<Decimal> {
        if period_end < period_start {
            return Err(AtlasError::ValidationFailed(
                "period_end cannot be before period_start".to_string(),
            ));
        }

        let mut current_balance = opening_balance;
        let mut sum_of_balances = Decimal::new(0, 0);
        let mut days_in_period = 0;

        // Build a map for quick lookup of balances by date
        let balance_map: BTreeMap<NaiveDate, Decimal> = daily_balances
            .iter()
            .map(|b| (b.date, b.end_of_day_balance))
            .collect();

        let mut current_date = period_start;

        while current_date <= period_end {
            if let Some(&balance) = balance_map.get(&current_date) {
                current_balance = balance;
            }
            sum_of_balances += current_balance;
            days_in_period += 1;
            
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        if days_in_period == 0 {
            return Ok(Decimal::new(0, 0));
        }

        let avg_balance = sum_of_balances / Decimal::from(days_in_period);
        Ok(avg_balance.round_dp(2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[test]
    fn test_calculate_average_balance() {
        let service = AverageDailyBalanceService::new();
        let account_id = Uuid::new_v4();
        let period_start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2023, 1, 5).unwrap(); // 5 days

        let daily_balances = vec![
            DailyBalance {
                date: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
                end_of_day_balance: dec!(150.00),
            },
            DailyBalance {
                date: NaiveDate::from_ymd_opt(2023, 1, 4).unwrap(),
                end_of_day_balance: dec!(200.00),
            },
        ];

        let opening_balance = dec!(100.00);

        // Day 1: 100.00
        // Day 2: 150.00
        // Day 3: 150.00 (carried over)
        // Day 4: 200.00
        // Day 5: 200.00 (carried over)
        // Sum = 100 + 150 + 150 + 200 + 200 = 800
        // Average = 800 / 5 = 160.00

        let avg = service
            .calculate_average_balance(
                account_id,
                period_start,
                period_end,
                &daily_balances,
                opening_balance,
            )
            .unwrap();

        assert_eq!(avg, dec!(160.00));
    }

    #[test]
    fn test_calculate_average_balance_invalid_period() {
        let service = AverageDailyBalanceService::new();
        let account_id = Uuid::new_v4();
        let period_start = NaiveDate::from_ymd_opt(2023, 1, 5).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        let res = service.calculate_average_balance(
            account_id,
            period_start,
            period_end,
            &[],
            dec!(100.00),
        );

        assert!(res.is_err());
    }
}
