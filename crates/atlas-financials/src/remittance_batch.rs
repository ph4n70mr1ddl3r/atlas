pub struct RemittanceBatchService;

pub struct RemittanceBatchResult {
    pub batch_id: String,
    pub bank_account_id: String,
    pub total_receipts: u32,
    pub total_amount: f64,
    pub status: String,
}

pub struct RemittanceClearanceResult {
    pub batch_id: String,
    pub cleared_amount: f64,
    pub status: String,
}

impl RemittanceBatchService {
    /// Creates a new Remittance Batch.
    /// This is an Oracle Fusion Receivables feature that groups receipts
    /// together to remit them to a bank for clearing.
    #[must_use]
    pub fn create_batch(bank_account_id: &str, receipt_amounts: &[f64]) -> RemittanceBatchResult {
        if bank_account_id.is_empty() {
            return RemittanceBatchResult {
                batch_id: "".to_string(),
                bank_account_id: bank_account_id.to_string(),
                total_receipts: 0,
                total_amount: 0.0,
                status: "REJECTED_INVALID_BANK".to_string(),
            };
        }

        if receipt_amounts.is_empty() {
            return RemittanceBatchResult {
                batch_id: "".to_string(),
                bank_account_id: bank_account_id.to_string(),
                total_receipts: 0,
                total_amount: 0.0,
                status: "REJECTED_NO_RECEIPTS".to_string(),
            };
        }

        let mut total_amount = 0.0;
        let mut total_receipts = 0;

        for &amount in receipt_amounts {
            if amount <= 0.0 {
                return RemittanceBatchResult {
                    batch_id: "".to_string(),
                    bank_account_id: bank_account_id.to_string(),
                    total_receipts: receipt_amounts.len() as u32,
                    total_amount: 0.0,
                    status: "REJECTED_INVALID_RECEIPT_AMOUNT".to_string(),
                };
            }
            total_amount += amount;
            total_receipts += 1;
        }

        RemittanceBatchResult {
            batch_id: format!("REMIT-{}-{}", bank_account_id, total_receipts),
            bank_account_id: bank_account_id.to_string(),
            total_receipts,
            total_amount,
            status: "CREATED".to_string(),
        }
    }

    /// Clears a remitted batch.
    #[must_use]
    pub fn clear_batch(batch_id: &str, cleared_amount: f64) -> RemittanceClearanceResult {
        if batch_id.is_empty() {
            return RemittanceClearanceResult {
                batch_id: batch_id.to_string(),
                cleared_amount: 0.0,
                status: "INVALID_BATCH_ID".to_string(),
            };
        }

        if cleared_amount <= 0.0 {
            return RemittanceClearanceResult {
                batch_id: batch_id.to_string(),
                cleared_amount,
                status: "REJECTED_INVALID_CLEARANCE".to_string(),
            };
        }

        RemittanceClearanceResult {
            batch_id: batch_id.to_string(),
            cleared_amount,
            status: "CLEARED".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_batch_valid() {
        let amounts = vec![1500.0, 500.0, 2000.0];
        let result = RemittanceBatchService::create_batch("BANK-123", &amounts);
        assert_eq!(result.bank_account_id, "BANK-123");
        assert_eq!(result.total_receipts, 3);
        assert_eq!(result.total_amount, 4000.0);
        assert_eq!(result.batch_id, "REMIT-BANK-123-3");
        assert_eq!(result.status, "CREATED");
    }

    #[test]
    fn test_create_batch_no_receipts() {
        let amounts: Vec<f64> = vec![];
        let result = RemittanceBatchService::create_batch("BANK-124", &amounts);
        assert_eq!(result.status, "REJECTED_NO_RECEIPTS");
    }

    #[test]
    fn test_create_batch_invalid_bank() {
        let amounts = vec![100.0];
        let result = RemittanceBatchService::create_batch("", &amounts);
        assert_eq!(result.status, "REJECTED_INVALID_BANK");
    }

    #[test]
    fn test_create_batch_negative_amount() {
        let amounts = vec![1000.0, -50.0];
        let result = RemittanceBatchService::create_batch("BANK-125", &amounts);
        assert_eq!(result.status, "REJECTED_INVALID_RECEIPT_AMOUNT");
    }

    #[test]
    fn test_clear_batch_valid() {
        let result = RemittanceBatchService::clear_batch("REMIT-BANK-123-3", 4000.0);
        assert_eq!(result.batch_id, "REMIT-BANK-123-3");
        assert_eq!(result.cleared_amount, 4000.0);
        assert_eq!(result.status, "CLEARED");
    }

    #[test]
    fn test_clear_batch_invalid_id() {
        let result = RemittanceBatchService::clear_batch("", 1000.0);
        assert_eq!(result.status, "INVALID_BATCH_ID");
    }

    #[test]
    fn test_clear_batch_invalid_amount() {
        let result = RemittanceBatchService::clear_batch("REMIT-BANK-123-3", 0.0);
        assert_eq!(result.status, "REJECTED_INVALID_CLEARANCE");
    }
}
