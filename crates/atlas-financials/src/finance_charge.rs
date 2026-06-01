use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceChargeTerm {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub term_code: String,
    pub charge_type: String, // percentage, flat_fee
    pub charge_rate: f64,
    pub grace_period_days: i32,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceChargeRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub run_date: NaiveDate,
    pub total_charges_assessed: f64,
    pub status: String, // draft, submitted, approved
}

pub struct FinanceChargeService {
    terms: Arc<RwLock<Vec<FinanceChargeTerm>>>,
    runs: Arc<RwLock<Vec<FinanceChargeRun>>>,
}

impl Default for FinanceChargeService {
    fn default() -> Self {
        Self {
            terms: Arc::new(RwLock::new(Vec::new())),
            runs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl FinanceChargeService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_term(
        &self,
        organization_id: Uuid,
        code: String,
        charge_type: String,
        rate: f64,
        grace: i32,
        currency: String,
    ) -> Result<FinanceChargeTerm, String> {
        let mut terms = self.terms.write().unwrap();
        if terms
            .iter()
            .any(|t| t.organization_id == organization_id && t.term_code == code)
        {
            return Err("Term with this code already exists".to_string());
        }

        let term = FinanceChargeTerm {
            id: Uuid::new_v4(),
            organization_id,
            term_code: code,
            charge_type,
            charge_rate: rate,
            grace_period_days: grace,
            currency_code: currency,
        };

        terms.push(term.clone());
        Ok(term)
    }

    pub fn calculate_charge(&self, term_id: Uuid, amount: f64, days_overdue: i32) -> f64 {
        let terms = self.terms.read().unwrap();
        let term = match terms.iter().find(|t| t.id == term_id) {
            Some(t) => t,
            None => return 0.0,
        };

        if days_overdue <= term.grace_period_days {
            return 0.0;
        }

        match term.charge_type.as_str() {
            "percentage" => amount * (term.charge_rate / 100.0),
            "flat_fee" => term.charge_rate,
            _ => 0.0,
        }
    }

    pub fn create_run(
        &self,
        organization_id: Uuid,
        number: String,
        date: NaiveDate,
    ) -> Result<FinanceChargeRun, String> {
        let mut runs = self.runs.write().unwrap();
        if runs
            .iter()
            .any(|r| r.organization_id == organization_id && r.run_number == number)
        {
            return Err("Run with this number already exists".to_string());
        }

        let run = FinanceChargeRun {
            id: Uuid::new_v4(),
            organization_id,
            run_number: number,
            run_date: date,
            total_charges_assessed: 0.0,
            status: "draft".to_string(),
        };

        runs.push(run.clone());
        Ok(run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage_charge() {
        let service = FinanceChargeService::new();
        let org_id = Uuid::new_v4();

        let t = service
            .create_term(
                org_id,
                "LATE_2".to_string(),
                "percentage".to_string(),
                2.0,
                5,
                "USD".to_string(),
            )
            .unwrap();

        // 3 days overdue (<= 5 grace) -> 0
        assert_eq!(service.calculate_charge(t.id, 1000.0, 3), 0.0);

        // 10 days overdue (> 5 grace) -> 2% of 1000 = 20
        assert_eq!(service.calculate_charge(t.id, 1000.0, 10), 20.0);
    }

    #[test]
    fn test_calculate_flat_fee() {
        let service = FinanceChargeService::new();
        let org_id = Uuid::new_v4();

        let t = service
            .create_term(
                org_id,
                "FLAT_50".to_string(),
                "flat_fee".to_string(),
                50.0,
                0,
                "USD".to_string(),
            )
            .unwrap();

        assert_eq!(service.calculate_charge(t.id, 1000.0, 1), 50.0);
    }
}
