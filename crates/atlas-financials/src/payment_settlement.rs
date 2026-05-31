use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub status: String, // draft, submitted, approved, settled, cancelled
    pub currency_code: String,
    pub total_settled_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementLine {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub invoice_id: Uuid,
    pub amount_paid: f64,
    pub discount_taken: f64,
    pub net_settlement: f64,
    pub status: String, // pending, settled, cancelled, error
}

pub struct PaymentSettlementService {
    batches: Arc<RwLock<Vec<SettlementBatch>>>,
    lines: Arc<RwLock<Vec<SettlementLine>>>,
}

impl PaymentSettlementService {
    pub fn new() -> Self {
        Self {
            batches: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_batch(
        &self,
        organization_id: Uuid,
        number: String,
        currency: String,
    ) -> Result<SettlementBatch, String> {
        let mut batches = self.batches.write().unwrap();
        if batches.iter().any(|b| b.organization_id == organization_id && b.batch_number == number) {
            return Err("Settlement batch with this number already exists".to_string());
        }

        let batch = SettlementBatch {
            id: Uuid::new_v4(),
            organization_id,
            batch_number: number,
            status: "draft".to_string(),
            currency_code: currency,
            total_settled_amount: 0.0,
        };

        batches.push(batch.clone());
        Ok(batch)
    }

    pub fn add_settlement_line(
        &self,
        batch_id: Uuid,
        invoice_id: Uuid,
        amount: f64,
        discount: f64,
    ) -> Result<SettlementLine, String> {
        let mut batches = self.batches.write().unwrap();
        let batch = batches.iter_mut().find(|b| b.id == batch_id)
            .ok_or_else(|| "Batch not found".to_string())?;

        if batch.status != "draft" {
            return Err("Lines can only be added to draft batches".to_string());
        }

        let line = SettlementLine {
            id: Uuid::new_v4(),
            batch_id,
            invoice_id,
            amount_paid: amount,
            discount_taken: discount,
            net_settlement: amount - discount,
            status: "pending".to_string(),
        };

        batch.total_settled_amount += line.net_settlement;

        let mut lines = self.lines.write().unwrap();
        lines.push(line.clone());
        Ok(line)
    }

    pub fn settle_batch(&self, id: Uuid) -> Result<(), String> {
        let mut batches = self.batches.write().unwrap();
        let batch = batches.iter_mut().find(|b| b.id == id)
            .ok_or_else(|| "Batch not found".to_string())?;
        
        if batch.status != "approved" && batch.status != "submitted" && batch.status != "draft" {
            return Err("Batch is not in a settleable status".to_string());
        }

        batch.status = "settled".to_string();

        let mut lines = self.lines.write().unwrap();
        for line in lines.iter_mut().filter(|l| l.batch_id == id) {
            line.status = "settled".to_string();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_settle_batch() {
        let service = PaymentSettlementService::new();
        let org_id = Uuid::new_v4();
        
        let batch = service.create_batch(org_id, "SET-001".to_string(), "USD".to_string()).unwrap();
        assert_eq!(batch.status, "draft");

        service.add_settlement_line(batch.id, Uuid::new_v4(), 1000.0, 50.0).unwrap();
        service.add_settlement_line(batch.id, Uuid::new_v4(), 500.0, 0.0).unwrap();

        {
            let batches = service.batches.read().unwrap();
            let updated = batches.iter().find(|b| b.id == batch.id).unwrap();
            assert_eq!(updated.total_settled_amount, 1450.0);
        }

        service.settle_batch(batch.id).unwrap();

        let batches = service.batches.read().unwrap();
        let settled = batches.iter().find(|b| b.id == batch.id).unwrap();
        assert_eq!(settled.status, "settled");

        let lines = service.lines.read().unwrap();
        assert!(lines.iter().all(|l| l.status == "settled"));
    }

    #[test]
    fn test_duplicate_batch_number() {
        let service = PaymentSettlementService::new();
        let org_id = Uuid::new_v4();
        service.create_batch(org_id, "B1".to_string(), "USD".to_string()).unwrap();
        let res = service.create_batch(org_id, "B1".to_string(), "USD".to_string());
        assert!(res.is_err());
    }
}
