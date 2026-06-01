use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chargeback {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub chargeback_number: String,
    pub customer_id: Option<Uuid>,
    pub amount: f64,
    pub reason_code: String,
    pub status: String, // open, under_review, resolved, cancelled
    pub open_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargebackLine {
    pub id: Uuid,
    pub chargeback_id: Uuid,
    pub line_number: i32,
    pub amount: f64,
    pub reason_code: Option<String>,
}

pub struct ChargebackService {
    chargebacks: Arc<RwLock<Vec<Chargeback>>>,
    lines: Arc<RwLock<Vec<ChargebackLine>>>,
}

impl Default for ChargebackService {
    fn default() -> Self {
        Self::new()
    }
}

impl ChargebackService {
    pub fn new() -> Self {
        Self {
            chargebacks: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_chargeback(
        &self,
        organization_id: Uuid,
        number: String,
        amount: f64,
        reason: String,
    ) -> Result<Chargeback, String> {
        let mut chargebacks = self.chargebacks.write().unwrap();
        if chargebacks.iter().any(|c| c.organization_id == organization_id && c.chargeback_number == number) {
            return Err("Chargeback with this number already exists".to_string());
        }

        let cb = Chargeback {
            id: Uuid::new_v4(),
            organization_id,
            chargeback_number: number,
            customer_id: None,
            amount,
            reason_code: reason,
            status: "open".to_string(),
            open_amount: amount,
        };

        chargebacks.push(cb.clone());
        Ok(cb)
    }

    pub fn add_line(
        &self,
        chargeback_id: Uuid,
        amount: f64,
        reason: Option<String>,
    ) -> Result<ChargebackLine, String> {
        let mut chargebacks = self.chargebacks.write().unwrap();
        let cb = chargebacks.iter_mut().find(|c| c.id == chargeback_id)
            .ok_or_else(|| "Chargeback not found".to_string())?;

        if cb.status != "open" && cb.status != "under_review" {
            return Err("Lines can only be added to open or under_review chargebacks".to_string());
        }

        let mut lines = self.lines.write().unwrap();
        let line_number = (lines.iter().filter(|l| l.chargeback_id == chargeback_id).count() as i32) + 1;

        let line = ChargebackLine {
            id: Uuid::new_v4(),
            chargeback_id,
            line_number,
            amount,
            reason_code: reason,
        };

        lines.push(line.clone());
        Ok(line)
    }

    pub fn resolve_chargeback(&self, id: Uuid, _notes: String) -> Result<(), String> {
        let mut chargebacks = self.chargebacks.write().unwrap();
        let cb = chargebacks.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Chargeback not found".to_string())?;
        
        cb.status = "resolved".to_string();
        cb.open_amount = 0.0;
        // Logic to create credit memo or adjust invoice would go here
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_resolve_chargeback() {
        let service = ChargebackService::new();
        let org_id = Uuid::new_v4();
        
        let cb = service.create_chargeback(org_id, "CB-101".to_string(), 500.0, "DAMAGED_GOODS".to_string()).unwrap();
        assert_eq!(cb.status, "open");

        service.add_line(cb.id, 250.0, Some("Box crushed".to_string())).unwrap();
        service.add_line(cb.id, 250.0, Some("Item missing".to_string())).unwrap();

        service.resolve_chargeback(cb.id, "Validated with warehouse".to_string()).unwrap();

        let chargebacks = service.chargebacks.read().unwrap();
        let updated = chargebacks.iter().find(|c| c.id == cb.id).unwrap();
        assert_eq!(updated.status, "resolved");
        assert_eq!(updated.open_amount, 0.0);
    }

    #[test]
    fn test_duplicate_chargeback() {
        let service = ChargebackService::new();
        let org_id = Uuid::new_v4();
        service.create_chargeback(org_id, "CB-1".to_string(), 100.0, "R".to_string()).unwrap();
        let res = service.create_chargeback(org_id, "CB-1".to_string(), 200.0, "R".to_string());
        assert!(res.is_err());
    }
}
