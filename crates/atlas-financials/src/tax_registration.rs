use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRegistration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub registration_number: String,
    pub registration_type: String, // tin, vat, gst, ein, pan, etc.
    pub country_code: String,
    pub status: String, // active, suspended, deregistered, expired, pending
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}

pub struct TaxRegistrationService {
    registrations: Arc<RwLock<Vec<TaxRegistration>>>,
}

impl TaxRegistrationService {
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_registration(
        &self,
        organization_id: Uuid,
        number: String,
        reg_type: String,
        country: String,
        effective_from: NaiveDate,
    ) -> Result<TaxRegistration, String> {
        let valid_types = ["tin", "vat", "gst", "ein", "sst", "pan", "cst", "sales_tax", "withholding_tax", "excise", "customs", "other"];
        if !valid_types.contains(&reg_type.as_str()) {
            return Err("Invalid registration type".to_string());
        }

        let mut registrations = self.registrations.write().unwrap();
        if registrations.iter().any(|r| r.organization_id == organization_id && r.registration_number == number) {
            return Err("Tax registration with this number already exists".to_string());
        }

        let reg = TaxRegistration {
            id: Uuid::new_v4(),
            organization_id,
            registration_number: number,
            registration_type: reg_type,
            country_code: country,
            status: "pending".to_string(),
            effective_from,
            effective_to: None,
        };

        registrations.push(reg.clone());
        Ok(reg)
    }

    pub fn activate_registration(&self, id: Uuid) -> Result<(), String> {
        let mut registrations = self.registrations.write().unwrap();
        let reg = registrations.iter_mut().find(|r| r.id == id)
            .ok_or_else(|| "Tax registration not found".to_string())?;
        
        reg.status = "active".to_string();
        Ok(())
    }

    pub fn is_active_on_date(&self, organization_id: Uuid, number: &str, date: NaiveDate) -> bool {
        let registrations = self.registrations.read().unwrap();
        if let Some(reg) = registrations.iter().find(|r| r.organization_id == organization_id && r.registration_number == number) {
            if reg.status != "active" {
                return false;
            }
            if date < reg.effective_from {
                return false;
            }
            if let Some(to) = reg.effective_to {
                if date > to {
                    return false;
                }
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_activate_registration() {
        let service = TaxRegistrationService::new();
        let org_id = Uuid::new_v4();
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        
        let reg = service.create_registration(org_id, "VAT-12345".to_string(), "vat".to_string(), "GB".to_string(), from).unwrap();
        assert_eq!(reg.status, "pending");

        assert!(!service.is_active_on_date(org_id, "VAT-12345", from));

        service.activate_registration(reg.id).unwrap();
        assert!(service.is_active_on_date(org_id, "VAT-12345", from));
        
        let future = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
        assert!(service.is_active_on_date(org_id, "VAT-12345", future));

        let past = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert!(!service.is_active_on_date(org_id, "VAT-12345", past));
    }

    #[test]
    fn test_invalid_reg_type() {
        let service = TaxRegistrationService::new();
        let res = service.create_registration(Uuid::new_v4(), "N".to_string(), "invalid".to_string(), "US".to_string(), NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert!(res.is_err());
    }
}
