use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AverageBalanceBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub book_code: String,
    pub book_name: String,
    pub period_type: String, // daily, monthly, quarterly, yearly
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AverageBalanceCalculation {
    pub id: Uuid,
    pub book_id: Uuid,
    pub account_id: Uuid,
    pub period_start_date: NaiveDate,
    pub period_end_date: NaiveDate,
    pub average_balance: Decimal,
    pub peak_balance: Decimal,
    pub status: String,
}

pub struct AverageBalanceProcessingService {
    books: Arc<RwLock<Vec<AverageBalanceBook>>>,
    calculations: Arc<RwLock<Vec<AverageBalanceCalculation>>>,
}

impl AverageBalanceProcessingService {
    pub fn new() -> Self {
        Self {
            books: Arc::new(RwLock::new(Vec::new())),
            calculations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_book(
        &self,
        organization_id: Uuid,
        code: String,
        name: String,
        period_type: String,
    ) -> Result<AverageBalanceBook, String> {
        let mut books = self.books.write().unwrap();
        if books.iter().any(|b| b.organization_id == organization_id && b.book_code == code) {
            return Err("Book with this code already exists".to_string());
        }

        let book = AverageBalanceBook {
            id: Uuid::new_v4(),
            organization_id,
            book_code: code,
            book_name: name,
            period_type,
            status: "active".to_string(),
        };

        books.push(book.clone());
        Ok(book)
    }

    pub fn run_calculation(
        &self,
        book_id: Uuid,
        account_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
        daily_balances: &[Decimal],
    ) -> Result<AverageBalanceCalculation, String> {
        if daily_balances.is_empty() {
            return Err("No daily balances provided for calculation".to_string());
        }

        let sum: Decimal = daily_balances.iter().sum();
        let count = Decimal::new(daily_balances.len() as i64, 0);
        let average = (sum / count).round_dp(2);
        let peak = *daily_balances.iter().max().unwrap_or(&Decimal::ZERO);

        let calc = AverageBalanceCalculation {
            id: Uuid::new_v4(),
            book_id,
            account_id,
            period_start_date: start,
            period_end_date: end,
            average_balance: average,
            peak_balance: peak,
            status: "calculated".to_string(),
        };

        let mut calculations = self.calculations.write().unwrap();
        calculations.push(calc.clone());
        Ok(calc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_book_and_run_calculation() {
        let service = AverageBalanceProcessingService::new();
        let org_id = Uuid::new_v4();
        
        let book = service.create_book(org_id, "CORP_ADB".to_string(), "Corp Average Balance".to_string(), "daily".to_string()).unwrap();
        
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 3).unwrap();
        let account_id = Uuid::new_v4();
        
        let balances = vec![dec!(100), dec!(200), dec!(150)];
        let calc = service.run_calculation(book.id, account_id, start, end, &balances).unwrap();

        assert_eq!(calc.average_balance, dec!(150));
        assert_eq!(calc.peak_balance, dec!(200));
    }
}
