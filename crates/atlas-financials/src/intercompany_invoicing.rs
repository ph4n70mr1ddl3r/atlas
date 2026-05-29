//! Oracle Fusion Financial Feature: Intercompany Invoicing
//! Automatically generates corresponding AR and AP invoices for cross-charge transactions
//! between distinct legal entities or organizations.

use chrono::NaiveDate;

pub struct IntercompanyInvoicingService;

#[derive(Debug, PartialEq, Clone)]
pub struct IntercompanyTransaction {
    pub transaction_id: String,
    pub provider_org_id: String,
    pub receiver_org_id: String,
    pub transaction_date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub status: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GeneratedInvoice {
    pub invoice_id: String,
    pub invoice_type: String, // "AR" or "AP"
    pub organization_id: String, // Org that owns the invoice
    pub counterpart_id: String,  // The other org acting as Customer (for AR) or Supplier (for AP)
    pub amount: f64,
}

#[derive(Debug, PartialEq)]
pub struct InvoicingResult {
    pub is_successful: bool,
    pub new_status: String,
    pub ar_invoice: Option<GeneratedInvoice>,
    pub ap_invoice: Option<GeneratedInvoice>,
    pub error_message: Option<String>,
}

impl IntercompanyInvoicingService {
    /// Evaluates an intercompany transaction. If valid, generates a Receivables invoice
    /// for the provider organization and a Payables invoice for the receiver organization.
    #[must_use]
    pub fn generate_invoices(transaction: &IntercompanyTransaction) -> InvoicingResult {
        // Validation: Provider and Receiver must be different
        if transaction.provider_org_id == transaction.receiver_org_id {
            return InvoicingResult {
                is_successful: false,
                new_status: "FAILED".to_string(),
                ar_invoice: None,
                ap_invoice: None,
                error_message: Some("Provider and Receiver organizations must be distinct for intercompany invoicing.".to_string()),
            };
        }

        // Validation: Amount must be positive
        if transaction.amount <= 0.0 {
            return InvoicingResult {
                is_successful: false,
                new_status: "FAILED".to_string(),
                ar_invoice: None,
                ap_invoice: None,
                error_message: Some("Intercompany transaction amount must be greater than zero.".to_string()),
            };
        }

        // Validation: Ensure transaction isn't already processed
        if transaction.status == "INVOICED" {
            return InvoicingResult {
                is_successful: false,
                new_status: transaction.status.clone(),
                ar_invoice: None,
                ap_invoice: None,
                error_message: Some("Transaction has already been invoiced.".to_string()),
            };
        }

        // Generate AR Invoice for the Provider Org (Receiver acts as Customer)
        let ar_invoice = GeneratedInvoice {
            invoice_id: format!("AR-IC-{}", transaction.transaction_id),
            invoice_type: "AR".to_string(),
            organization_id: transaction.provider_org_id.clone(),
            counterpart_id: transaction.receiver_org_id.clone(), // Receiver is the customer
            amount: transaction.amount,
        };

        // Generate AP Invoice for the Receiver Org (Provider acts as Supplier)
        let ap_invoice = GeneratedInvoice {
            invoice_id: format!("AP-IC-{}", transaction.transaction_id),
            invoice_type: "AP".to_string(),
            organization_id: transaction.receiver_org_id.clone(),
            counterpart_id: transaction.provider_org_id.clone(), // Provider is the supplier
            amount: transaction.amount,
        };

        InvoicingResult {
            is_successful: true,
            new_status: "INVOICED".to_string(),
            ar_invoice: Some(ar_invoice),
            ap_invoice: Some(ap_invoice),
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_intercompany_invoicing() {
        let transaction = IntercompanyTransaction {
            transaction_id: "IC-TRX-100".to_string(),
            provider_org_id: "ORG-US".to_string(),
            receiver_org_id: "ORG-UK".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: 5000.0,
            description: "Cross-charge for IT services".to_string(),
            status: "NEW".to_string(),
        };

        let result = IntercompanyInvoicingService::generate_invoices(&transaction);

        assert!(result.is_successful);
        assert_eq!(result.new_status, "INVOICED");
        assert!(result.error_message.is_none());

        // Validate AR Invoice
        let ar = result.ar_invoice.unwrap();
        assert_eq!(ar.invoice_type, "AR");
        assert_eq!(ar.organization_id, "ORG-US"); // Provider owns AR
        assert_eq!(ar.counterpart_id, "ORG-UK");  // Receiver is the customer
        assert_eq!(ar.amount, 5000.0);

        // Validate AP Invoice
        let ap = result.ap_invoice.unwrap();
        assert_eq!(ap.invoice_type, "AP");
        assert_eq!(ap.organization_id, "ORG-UK"); // Receiver owns AP
        assert_eq!(ap.counterpart_id, "ORG-US");  // Provider is the supplier
        assert_eq!(ap.amount, 5000.0);
    }

    #[test]
    fn test_fail_same_organization() {
        let transaction = IntercompanyTransaction {
            transaction_id: "IC-TRX-101".to_string(),
            provider_org_id: "ORG-US".to_string(),
            receiver_org_id: "ORG-US".to_string(), // Same org!
            transaction_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: 1000.0,
            description: "Internal charge".to_string(),
            status: "NEW".to_string(),
        };

        let result = IntercompanyInvoicingService::generate_invoices(&transaction);

        assert!(!result.is_successful);
        assert_eq!(result.new_status, "FAILED");
        assert_eq!(result.error_message.unwrap(), "Provider and Receiver organizations must be distinct for intercompany invoicing.");
        assert!(result.ar_invoice.is_none());
        assert!(result.ap_invoice.is_none());
    }

    #[test]
    fn test_fail_zero_amount() {
        let transaction = IntercompanyTransaction {
            transaction_id: "IC-TRX-102".to_string(),
            provider_org_id: "ORG-US".to_string(),
            receiver_org_id: "ORG-FR".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: 0.0, // Invalid amount
            description: "Zero value cross-charge".to_string(),
            status: "NEW".to_string(),
        };

        let result = IntercompanyInvoicingService::generate_invoices(&transaction);

        assert!(!result.is_successful);
        assert_eq!(result.new_status, "FAILED");
        assert_eq!(result.error_message.unwrap(), "Intercompany transaction amount must be greater than zero.");
    }

    #[test]
    fn test_fail_already_invoiced() {
        let transaction = IntercompanyTransaction {
            transaction_id: "IC-TRX-103".to_string(),
            provider_org_id: "ORG-US".to_string(),
            receiver_org_id: "ORG-CA".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: 250.0,
            description: "Already processed".to_string(),
            status: "INVOICED".to_string(), // Already done
        };

        let result = IntercompanyInvoicingService::generate_invoices(&transaction);

        assert!(!result.is_successful);
        assert_eq!(result.new_status, "INVOICED");
        assert_eq!(result.error_message.unwrap(), "Transaction has already been invoiced.");
    }
}
