use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub status: String, // draft, confirmed, closed
    pub total_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCashReceipt {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_number: String,
    pub customer_id: Uuid,
    pub amount: Decimal,
    pub applied_amount: Decimal,
    pub status: String, // unidentified, identified, applied, partially_applied, reversed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptApplication {
    pub id: Uuid,
    pub receipt_id: Uuid,
    pub invoice_id: Uuid,
    pub amount_applied: Decimal,
}

pub struct CashReceiptService {
    batches: Arc<RwLock<Vec<ReceiptBatch>>>,
    receipts: Arc<RwLock<Vec<CustomerCashReceipt>>>,
    applications: Arc<RwLock<Vec<ReceiptApplication>>>,
}

impl Default for CashReceiptService {
    fn default() -> Self {
        Self::new()
    }
}

impl CashReceiptService {
    pub fn new() -> Self {
        Self {
            batches: Arc::new(RwLock::new(Vec::new())),
            receipts: Arc::new(RwLock::new(Vec::new())),
            applications: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_batch(
        &self,
        organization_id: Uuid,
        number: String,
    ) -> Result<ReceiptBatch, String> {
        let mut batches = self.batches.write().unwrap();
        if batches
            .iter()
            .any(|b| b.organization_id == organization_id && b.batch_number == number)
        {
            return Err("Batch already exists".to_string());
        }
        let batch = ReceiptBatch {
            id: Uuid::new_v4(),
            organization_id,
            batch_number: number,
            status: "draft".to_string(),
            total_amount: Decimal::ZERO,
        };
        batches.push(batch.clone());
        Ok(batch)
    }

    pub fn create_receipt(
        &self,
        organization_id: Uuid,
        customer_id: Uuid,
        number: String,
        amount: Decimal,
    ) -> Result<CustomerCashReceipt, String> {
        let mut receipts = self.receipts.write().unwrap();
        if receipts
            .iter()
            .any(|r| r.organization_id == organization_id && r.receipt_number == number)
        {
            return Err("Receipt already exists".to_string());
        }
        let receipt = CustomerCashReceipt {
            id: Uuid::new_v4(),
            organization_id,
            receipt_number: number,
            customer_id,
            amount,
            applied_amount: Decimal::ZERO,
            status: "identified".to_string(),
        };
        receipts.push(receipt.clone());
        Ok(receipt)
    }

    pub fn apply_receipt(
        &self,
        receipt_id: Uuid,
        invoice_id: Uuid,
        amount: Decimal,
    ) -> Result<(), String> {
        let mut receipts = self.receipts.write().unwrap();
        let receipt = receipts
            .iter_mut()
            .find(|r| r.id == receipt_id)
            .ok_or_else(|| "Receipt not found".to_string())?;

        let remaining = receipt.amount - receipt.applied_amount;
        if amount > remaining {
            return Err("Applied amount exceeds remaining receipt balance".to_string());
        }

        let app = ReceiptApplication {
            id: Uuid::new_v4(),
            receipt_id,
            invoice_id,
            amount_applied: amount,
        };

        receipt.applied_amount += amount;
        receipt.status = if receipt.applied_amount == receipt.amount {
            "applied".to_string()
        } else {
            "partially_applied".to_string()
        };

        let mut applications = self.applications.write().unwrap();
        applications.push(app);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_and_apply_receipt() {
        let service = CashReceiptService::new();
        let org_id = Uuid::new_v4();
        let customer_id = Uuid::new_v4();

        let receipt = service
            .create_receipt(org_id, customer_id, "REC-001".to_string(), dec!(1000))
            .unwrap();
        assert_eq!(receipt.status, "identified");

        let invoice_id = Uuid::new_v4();
        service
            .apply_receipt(receipt.id, invoice_id, dec!(600))
            .unwrap();

        {
            let receipts = service.receipts.read().unwrap();
            let updated = receipts.iter().find(|r| r.id == receipt.id).unwrap();
            assert_eq!(updated.applied_amount, dec!(600));
            assert_eq!(updated.status, "partially_applied");
        }

        service
            .apply_receipt(receipt.id, Uuid::new_v4(), dec!(400))
            .unwrap();

        {
            let updated2 = service.receipts.read().unwrap();
            let final_receipt = updated2.iter().find(|r| r.id == receipt.id).unwrap();
            assert_eq!(final_receipt.status, "applied");
        }
    }

    #[test]
    fn test_apply_exceeds_amount() {
        let service = CashReceiptService::new();
        let r = service
            .create_receipt(Uuid::new_v4(), Uuid::new_v4(), "R".to_string(), dec!(100))
            .unwrap();
        let err = service.apply_receipt(r.id, Uuid::new_v4(), dec!(150));
        assert!(err.is_err());
    }
}
