use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankGuarantee {
    pub id: Uuid,
    pub org_id: Uuid,
    pub guarantee_number: String,
    pub guarantee_type: String, // bid_bond, performance, advance_payment
    pub beneficiary_name: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub expiry_date: Option<NaiveDate>,
    pub status: String, // draft, active, expired, cancelled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankGuaranteeAmendment {
    pub id: Uuid,
    pub guarantee_id: Uuid,
    pub amendment_number: String,
    pub amendment_type: String, // extension, amount_increase, amount_decrease
    pub previous_amount: Option<Decimal>,
    pub new_amount: Option<Decimal>,
    pub status: String,
}

pub struct BankGuaranteeService {
    guarantees: Arc<RwLock<Vec<BankGuarantee>>>,
    amendments: Arc<RwLock<Vec<BankGuaranteeAmendment>>>,
}

impl Default for BankGuaranteeService {
    fn default() -> Self {
        Self::new()
    }
}

impl BankGuaranteeService {
    pub fn new() -> Self {
        Self {
            guarantees: Arc::new(RwLock::new(Vec::new())),
            amendments: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_guarantee(
        &self,
        org_id: Uuid,
        number: String,
        guarantee_type: String,
        beneficiary: String,
        amount: Decimal,
        currency: String,
        expiry: Option<NaiveDate>,
    ) -> Result<BankGuarantee, String> {
        let mut guarantees = self.guarantees.write().unwrap();
        if guarantees.iter().any(|g| g.org_id == org_id && g.guarantee_number == number) {
            return Err("Bank guarantee with this number already exists".to_string());
        }

        let bg = BankGuarantee {
            id: Uuid::new_v4(),
            org_id,
            guarantee_number: number,
            guarantee_type,
            beneficiary_name: beneficiary,
            amount,
            currency_code: currency,
            expiry_date: expiry,
            status: "draft".to_string(),
        };

        guarantees.push(bg.clone());
        Ok(bg)
    }

    pub fn activate_guarantee(&self, id: Uuid) -> Result<(), String> {
        let mut guarantees = self.guarantees.write().unwrap();
        let bg = guarantees.iter_mut().find(|g| g.id == id)
            .ok_or_else(|| "Bank guarantee not found".to_string())?;
        
        if bg.status != "draft" {
            return Err("Only draft guarantees can be activated".to_string());
        }

        bg.status = "active".to_string();
        Ok(())
    }

    pub fn add_amendment(
        &self,
        guarantee_id: Uuid,
        amendment_type: String,
        new_amount: Option<Decimal>,
    ) -> Result<BankGuaranteeAmendment, String> {
        let mut guarantees = self.guarantees.write().unwrap();
        let bg = guarantees.iter_mut().find(|g| g.id == guarantee_id)
            .ok_or_else(|| "Bank guarantee not found".to_string())?;

        if bg.status != "active" {
            return Err("Amendments can only be added to active guarantees".to_string());
        }

        let mut amendments = self.amendments.write().unwrap();
        let count = amendments.iter().filter(|a| a.guarantee_id == guarantee_id).count() + 1;
        let number = format!("{}-AMD-{}", bg.guarantee_number, count);

        let amd = BankGuaranteeAmendment {
            id: Uuid::new_v4(),
            guarantee_id,
            amendment_number: number,
            amendment_type,
            previous_amount: Some(bg.amount),
            new_amount,
            status: "pending_approval".to_string(),
        };

        amendments.push(amd.clone());
        Ok(amd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_and_activate_guarantee() {
        let service = BankGuaranteeService::new();
        let org_id = Uuid::new_v4();
        
        let bg = service.create_guarantee(
            org_id,
            "BG-2026-001".to_string(),
            "performance".to_string(),
            "Global Build LLC".to_string(),
            dec!(50000),
            "USD".to_string(),
            None,
        ).unwrap();

        assert_eq!(bg.status, "draft");
        
        service.activate_guarantee(bg.id).unwrap();
        
        let guarantees = service.guarantees.read().unwrap();
        assert_eq!(guarantees[0].status, "active");
    }

    #[test]
    fn test_add_amendment() {
        let service = BankGuaranteeService::new();
        let org_id = Uuid::new_v4();
        
        let bg = service.create_guarantee(
            org_id, "BG-1".to_string(), "bid".to_string(), "B".to_string(), dec!(1000), "USD".to_string(), None
        ).unwrap();

        // Error: not active
        let err = service.add_amendment(bg.id, "increase".to_string(), Some(dec!(2000)));
        assert!(err.is_err());

        service.activate_guarantee(bg.id).unwrap();

        let amd = service.add_amendment(bg.id, "increase".to_string(), Some(dec!(2000))).unwrap();
        assert_eq!(amd.previous_amount, Some(dec!(1000)));
        assert_eq!(amd.new_amount, Some(dec!(2000)));
    }
}
