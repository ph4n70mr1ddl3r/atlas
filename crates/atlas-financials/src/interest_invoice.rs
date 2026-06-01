use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_code: String,
    pub annual_rate: Decimal,
    pub compounding_frequency: String, // daily, monthly
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestOverdueInvoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_number: String,
    pub outstanding_amount: Decimal,
    pub due_date: NaiveDate,
}

pub struct InterestInvoiceService {
    schedules: Arc<RwLock<Vec<InterestSchedule>>>,
}

impl Default for InterestInvoiceService {
    fn default() -> Self {
        Self::new()
    }
}

impl InterestInvoiceService {
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_schedule(
        &self,
        organization_id: Uuid,
        code: String,
        rate: Decimal,
    ) -> Result<InterestSchedule, String> {
        let mut schedules = self.schedules.write().unwrap();
        if schedules
            .iter()
            .any(|s| s.organization_id == organization_id && s.schedule_code == code)
        {
            return Err("Schedule with this code already exists".to_string());
        }

        let schedule = InterestSchedule {
            id: Uuid::new_v4(),
            organization_id,
            schedule_code: code,
            annual_rate: rate,
            compounding_frequency: "daily".to_string(),
            status: "active".to_string(),
        };

        schedules.push(schedule.clone());
        Ok(schedule)
    }

    pub fn calculate_interest(
        &self,
        schedule_id: Uuid,
        amount: Decimal,
        days_overdue: i32,
    ) -> Decimal {
        let schedules = self.schedules.read().unwrap();
        let schedule = match schedules.iter().find(|s| s.id == schedule_id) {
            Some(s) => s,
            None => return Decimal::ZERO,
        };

        if days_overdue <= 0 {
            return Decimal::ZERO;
        }

        // Simple interest: Principal * Rate * (Days / 365)
        let daily_rate = schedule.annual_rate / Decimal::new(100, 0) / Decimal::new(365, 0);
        (amount * daily_rate * Decimal::new(days_overdue as i64, 0)).round_dp(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_simple_interest() {
        let service = InterestInvoiceService::new();
        let org_id = Uuid::new_v4();

        let s = service
            .create_schedule(org_id, "STD_10".to_string(), dec!(10.0))
            .unwrap();

        // Principal: 1000, 365 days overdue, 10% rate -> 100
        let interest = service.calculate_interest(s.id, dec!(1000), 365);
        assert_eq!(interest, dec!(100));
    }
}
