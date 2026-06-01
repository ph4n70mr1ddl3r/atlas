//! Oracle Fusion Financial Feature: Evaluated Receipt Settlement (ERS)
//! Also known as "Pay on Receipt". Automatically generates AP invoices from PO receipts.

pub struct EvaluatedReceiptSettlementService;

#[derive(Debug, PartialEq, Clone)]
pub struct SupplierErsProfile {
    pub supplier_id: String,
    pub ers_enabled: bool,
    pub default_payment_terms: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PoReceipt {
    pub receipt_id: String,
    pub purchase_order_id: String,
    pub supplier_id: String,
    pub quantity_received: f64,
    pub unit_price: f64,
    pub tax_rate: f64,
}

#[derive(Debug, PartialEq)]
pub struct ErsInvoiceResult {
    pub receipt_id: String,
    pub generated_invoice_id: Option<String>,
    pub invoice_amount: f64,
    pub payment_terms: String,
    pub status: String,
    pub error_reason: Option<String>,
}

impl EvaluatedReceiptSettlementService {
    /// Evaluates a receipt to determine if an AP invoice should be auto-generated.
    /// If the supplier is ERS-enabled, it calculates the invoice amount and generates it.
    #[must_use]
    pub fn process_receipt(
        receipt: &PoReceipt,
        supplier_profile: Option<&SupplierErsProfile>,
    ) -> ErsInvoiceResult {
        let profile = match supplier_profile {
            Some(p) => p,
            None => {
                return ErsInvoiceResult {
                    receipt_id: receipt.receipt_id.clone(),
                    generated_invoice_id: None,
                    invoice_amount: 0.0,
                    payment_terms: "".to_string(),
                    status: "FAILED".to_string(),
                    error_reason: Some("Supplier ERS profile not found".to_string()),
                };
            }
        };

        if !profile.ers_enabled {
            return ErsInvoiceResult {
                receipt_id: receipt.receipt_id.clone(),
                generated_invoice_id: None,
                invoice_amount: 0.0,
                payment_terms: "".to_string(),
                status: "SKIPPED".to_string(),
                error_reason: Some("Supplier is not ERS enabled".to_string()),
            };
        }

        if receipt.quantity_received <= 0.0 || receipt.unit_price < 0.0 {
            return ErsInvoiceResult {
                receipt_id: receipt.receipt_id.clone(),
                generated_invoice_id: None,
                invoice_amount: 0.0,
                payment_terms: "".to_string(),
                status: "FAILED".to_string(),
                error_reason: Some("Invalid receipt quantities or price".to_string()),
            };
        }

        let subtotal = receipt.quantity_received * receipt.unit_price;
        let tax_amount = subtotal * receipt.tax_rate;
        let total_invoice_amount = subtotal + tax_amount;

        ErsInvoiceResult {
            receipt_id: receipt.receipt_id.clone(),
            generated_invoice_id: Some(format!("ERS-INV-{}", receipt.receipt_id)),
            invoice_amount: total_invoice_amount,
            payment_terms: profile.default_payment_terms.clone(),
            status: "PROCESSED".to_string(),
            error_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_receipt_ers_enabled() {
        let profile = SupplierErsProfile {
            supplier_id: "SUPP-001".to_string(),
            ers_enabled: true,
            default_payment_terms: "NET 30".to_string(),
        };

        let receipt = PoReceipt {
            receipt_id: "RCPT-100".to_string(),
            purchase_order_id: "PO-500".to_string(),
            supplier_id: "SUPP-001".to_string(),
            quantity_received: 10.0,
            unit_price: 50.0,
            tax_rate: 0.10, // 10% tax
        };

        let result = EvaluatedReceiptSettlementService::process_receipt(&receipt, Some(&profile));

        assert_eq!(result.status, "PROCESSED");
        assert_eq!(
            result.generated_invoice_id,
            Some("ERS-INV-RCPT-100".to_string())
        );
        // 10 * 50 = 500. 500 + 10% tax = 550.
        assert_eq!(result.invoice_amount, 550.0);
        assert_eq!(result.payment_terms, "NET 30");
        assert!(result.error_reason.is_none());
    }

    #[test]
    fn test_process_receipt_ers_disabled() {
        let profile = SupplierErsProfile {
            supplier_id: "SUPP-002".to_string(),
            ers_enabled: false,
            default_payment_terms: "IMMEDIATE".to_string(),
        };

        let receipt = PoReceipt {
            receipt_id: "RCPT-101".to_string(),
            purchase_order_id: "PO-501".to_string(),
            supplier_id: "SUPP-002".to_string(),
            quantity_received: 5.0,
            unit_price: 100.0,
            tax_rate: 0.0,
        };

        let result = EvaluatedReceiptSettlementService::process_receipt(&receipt, Some(&profile));

        assert_eq!(result.status, "SKIPPED");
        assert_eq!(result.generated_invoice_id, None);
        assert_eq!(
            result.error_reason,
            Some("Supplier is not ERS enabled".to_string())
        );
    }

    #[test]
    fn test_process_receipt_missing_profile() {
        let receipt = PoReceipt {
            receipt_id: "RCPT-102".to_string(),
            purchase_order_id: "PO-502".to_string(),
            supplier_id: "SUPP-003".to_string(),
            quantity_received: 1.0,
            unit_price: 100.0,
            tax_rate: 0.0,
        };

        let result = EvaluatedReceiptSettlementService::process_receipt(&receipt, None);

        assert_eq!(result.status, "FAILED");
        assert_eq!(
            result.error_reason,
            Some("Supplier ERS profile not found".to_string())
        );
    }

    #[test]
    fn test_process_receipt_invalid_quantity() {
        let profile = SupplierErsProfile {
            supplier_id: "SUPP-004".to_string(),
            ers_enabled: true,
            default_payment_terms: "NET 15".to_string(),
        };

        let receipt = PoReceipt {
            receipt_id: "RCPT-103".to_string(),
            purchase_order_id: "PO-503".to_string(),
            supplier_id: "SUPP-004".to_string(),
            quantity_received: -5.0, // Invalid quantity
            unit_price: 20.0,
            tax_rate: 0.0,
        };

        let result = EvaluatedReceiptSettlementService::process_receipt(&receipt, Some(&profile));

        assert_eq!(result.status, "FAILED");
        assert_eq!(
            result.error_reason,
            Some("Invalid receipt quantities or price".to_string())
        );
    }
}
