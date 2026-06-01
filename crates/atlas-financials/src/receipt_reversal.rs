//! Oracle Fusion Financial Feature: AR Receipt Reversal
//! Manages the reversal of customer cash receipts and the reopening of associated invoices.

use chrono::NaiveDate;

pub struct ReceiptReversalService;

#[derive(Debug, PartialEq, Clone)]
pub struct AppliedInvoice {
    pub invoice_id: String,
    pub applied_amount: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CustomerReceipt {
    pub receipt_id: String,
    pub receipt_number: String,
    pub amount: f64,
    pub status: String, // 'APPLIED', 'UNAPPLIED', 'CLEARED', 'REVERSED'
    pub applications: Vec<AppliedInvoice>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReversalRequest {
    pub reversal_category: String, // e.g., "NSF", "STOP_PAYMENT"
    pub reversal_reason: String,
    pub reversal_date: NaiveDate,
    pub comments: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct InvoiceBalanceUpdate {
    pub invoice_id: String,
    pub amount_to_reopen: f64,
}

#[derive(Debug, PartialEq)]
pub struct ReversalResult {
    pub receipt_id: String,
    pub new_receipt_status: String,
    pub is_successful: bool,
    pub reopened_invoices: Vec<InvoiceBalanceUpdate>,
    pub error_message: Option<String>,
}

impl ReceiptReversalService {
    /// Reverses a customer receipt. This changes the receipt status to REVERSED
    /// and generates balance updates to reopen any invoices the receipt was previously applied to.
    #[must_use]
    pub fn process_reversal(
        receipt: &CustomerReceipt,
        request: &ReversalRequest,
    ) -> ReversalResult {
        // Validation: Cannot reverse a receipt that is already reversed
        if receipt.status == "REVERSED" {
            return ReversalResult {
                receipt_id: receipt.receipt_id.clone(),
                new_receipt_status: receipt.status.clone(),
                is_successful: false,
                reopened_invoices: vec![],
                error_message: Some("Receipt is already reversed.".to_string()),
            };
        }

        // Validation: Ensure valid category (similar to Oracle standard categories)
        let valid_categories = [
            "NSF",
            "STOP_PAYMENT",
            "DATA_ENTRY_ERROR",
            "PAYMENT_REVERSAL",
        ];
        if !valid_categories.contains(&request.reversal_category.as_str()) {
            return ReversalResult {
                receipt_id: receipt.receipt_id.clone(),
                new_receipt_status: receipt.status.clone(),
                is_successful: false,
                reopened_invoices: vec![],
                error_message: Some(format!(
                    "Invalid reversal category: {}",
                    request.reversal_category
                )),
            };
        }

        // Unapply all associated invoices so they can be reopened
        let mut reopened_invoices = Vec::new();
        for application in &receipt.applications {
            reopened_invoices.push(InvoiceBalanceUpdate {
                invoice_id: application.invoice_id.clone(),
                amount_to_reopen: application.applied_amount,
            });
        }

        ReversalResult {
            receipt_id: receipt.receipt_id.clone(),
            new_receipt_status: "REVERSED".to_string(),
            is_successful: true,
            reopened_invoices,
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_reversal_with_applications() {
        let receipt = CustomerReceipt {
            receipt_id: "RCPT-100".to_string(),
            receipt_number: "CHK-9922".to_string(),
            amount: 1500.0,
            status: "APPLIED".to_string(),
            applications: vec![
                AppliedInvoice {
                    invoice_id: "INV-10".to_string(),
                    applied_amount: 1000.0,
                },
                AppliedInvoice {
                    invoice_id: "INV-11".to_string(),
                    applied_amount: 500.0,
                },
            ],
        };

        let request = ReversalRequest {
            reversal_category: "NSF".to_string(),
            reversal_reason: "Insufficient Funds".to_string(),
            reversal_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            comments: Some("Bank returned check".to_string()),
        };

        let result = ReceiptReversalService::process_reversal(&receipt, &request);

        assert!(result.is_successful);
        assert_eq!(result.new_receipt_status, "REVERSED");
        assert_eq!(result.reopened_invoices.len(), 2);

        let inv10 = result
            .reopened_invoices
            .iter()
            .find(|i| i.invoice_id == "INV-10")
            .unwrap();
        assert_eq!(inv10.amount_to_reopen, 1000.0);

        let inv11 = result
            .reopened_invoices
            .iter()
            .find(|i| i.invoice_id == "INV-11")
            .unwrap();
        assert_eq!(inv11.amount_to_reopen, 500.0);
    }

    #[test]
    fn test_successful_reversal_unapplied_receipt() {
        // Reversing a receipt that was never applied to any invoice
        let receipt = CustomerReceipt {
            receipt_id: "RCPT-101".to_string(),
            receipt_number: "CHK-9923".to_string(),
            amount: 300.0,
            status: "UNAPPLIED".to_string(),
            applications: vec![],
        };

        let request = ReversalRequest {
            reversal_category: "DATA_ENTRY_ERROR".to_string(),
            reversal_reason: "Wrong Customer".to_string(),
            reversal_date: NaiveDate::from_ymd_opt(2025, 6, 2).unwrap(),
            comments: None,
        };

        let result = ReceiptReversalService::process_reversal(&receipt, &request);

        assert!(result.is_successful);
        assert_eq!(result.new_receipt_status, "REVERSED");
        assert_eq!(result.reopened_invoices.len(), 0);
    }

    #[test]
    fn test_fail_reversal_already_reversed() {
        let receipt = CustomerReceipt {
            receipt_id: "RCPT-102".to_string(),
            receipt_number: "CHK-9924".to_string(),
            amount: 50.0,
            status: "REVERSED".to_string(),
            applications: vec![],
        };

        let request = ReversalRequest {
            reversal_category: "STOP_PAYMENT".to_string(),
            reversal_reason: "Customer stopped".to_string(),
            reversal_date: NaiveDate::from_ymd_opt(2025, 6, 3).unwrap(),
            comments: None,
        };

        let result = ReceiptReversalService::process_reversal(&receipt, &request);

        assert!(!result.is_successful);
        assert_eq!(
            result.error_message,
            Some("Receipt is already reversed.".to_string())
        );
        assert_eq!(result.new_receipt_status, "REVERSED"); // Unchanged
    }

    #[test]
    fn test_fail_invalid_category() {
        let receipt = CustomerReceipt {
            receipt_id: "RCPT-103".to_string(),
            receipt_number: "CHK-9925".to_string(),
            amount: 100.0,
            status: "APPLIED".to_string(),
            applications: vec![],
        };

        let request = ReversalRequest {
            reversal_category: "INVALID_CAT".to_string(),
            reversal_reason: "Just testing".to_string(),
            reversal_date: NaiveDate::from_ymd_opt(2025, 6, 4).unwrap(),
            comments: None,
        };

        let result = ReceiptReversalService::process_reversal(&receipt, &request);

        assert!(!result.is_successful);
        assert_eq!(
            result.error_message,
            Some("Invalid reversal category: INVALID_CAT".to_string())
        );
        assert_eq!(result.new_receipt_status, "APPLIED"); // Unchanged
    }
}
