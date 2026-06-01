use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARReceiptReversal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub reversal_category: String, // NSF, STOP_PAYMENT, REVERSE_PAYMENT, UNAPPLIED
    pub reversal_reason_code: String,
    pub reversal_date: NaiveDate,
    pub reversal_comments: Option<String>,
    pub reversed_by: Option<Uuid>,
    pub status: String,
}

pub struct ARReceiptReversalService {
    reversals: Arc<RwLock<Vec<ARReceiptReversal>>>,
}

impl Default for ARReceiptReversalService {
    fn default() -> Self {
        Self {
            reversals: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl ARReceiptReversalService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reverse_receipt(
        &self,
        organization_id: Uuid,
        receipt_id: Uuid,
        category: String,
        reason_code: String,
        date: NaiveDate,
        comments: Option<String>,
        user_id: Option<Uuid>,
    ) -> Result<ARReceiptReversal, String> {
        let valid_categories = ["NSF", "STOP_PAYMENT", "REVERSE_PAYMENT", "UNAPPLIED"];
        if !valid_categories.contains(&category.as_str()) {
            return Err(format!(
                "Invalid reversal category: {}. Must be one of {:?}",
                category, valid_categories
            ));
        }

        let mut reversals = self.reversals.write().unwrap();

        // Ensure not already reversed
        if reversals.iter().any(|r| r.receipt_id == receipt_id) {
            return Err("Receipt has already been reversed".to_string());
        }

        let reversal = ARReceiptReversal {
            id: Uuid::new_v4(),
            organization_id,
            receipt_id,
            reversal_category: category,
            reversal_reason_code: reason_code,
            reversal_date: date,
            reversal_comments: comments,
            reversed_by: user_id,
            status: "COMPLETED".to_string(),
        };

        reversals.push(reversal.clone());

        // In a real implementation, this would trigger logic to:
        // 1. Unapply the receipt from all invoices
        // 2. Re-open the invoices (increase balance)
        // 3. Post reversal journal entries to GL

        Ok(reversal)
    }

    pub fn get_reversal_by_receipt(&self, receipt_id: Uuid) -> Option<ARReceiptReversal> {
        let reversals = self.reversals.read().unwrap();
        reversals
            .iter()
            .find(|r| r.receipt_id == receipt_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_receipt_success() {
        let service = ARReceiptReversalService::new();
        let org_id = Uuid::new_v4();
        let receipt_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();

        let result = service.reverse_receipt(
            org_id,
            receipt_id,
            "NSF".to_string(),
            "INSUFFICIENT_FUNDS".to_string(),
            today,
            Some("Customer check bounced".to_string()),
            None,
        );

        assert!(result.is_ok());
        let rev = result.unwrap();
        assert_eq!(rev.reversal_category, "NSF");
        assert_eq!(rev.receipt_id, receipt_id);
    }

    #[test]
    fn test_reverse_receipt_invalid_category() {
        let service = ARReceiptReversalService::new();
        let org_id = Uuid::new_v4();
        let receipt_id = Uuid::new_v4();

        let result = service.reverse_receipt(
            org_id,
            receipt_id,
            "WRONG".to_string(),
            "REASON".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid reversal category"));
    }

    #[test]
    fn test_prevent_duplicate_reversal() {
        let service = ARReceiptReversalService::new();
        let org_id = Uuid::new_v4();
        let receipt_id = Uuid::new_v4();
        let today = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();

        service
            .reverse_receipt(
                org_id,
                receipt_id,
                "NSF".to_string(),
                "R1".to_string(),
                today,
                None,
                None,
            )
            .unwrap();

        let result = service.reverse_receipt(
            org_id,
            receipt_id,
            "NSF".to_string(),
            "R2".to_string(),
            today,
            None,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Receipt has already been reversed");
    }
}
