//! Oracle Fusion Financial Feature: Cash Receipt AutoMatch Rules
//! Evaluates incoming cash receipts against open invoices to automatically apply payments.

pub struct AutoMatchService;

#[derive(Debug, PartialEq, Clone)]
pub struct AutoMatchRuleSet {
    pub rule_set_name: String,
    pub match_by_invoice_number: bool,
    pub match_by_purchase_order: bool,
    pub min_match_percentage: f64, // e.g., 100.0 means 100% of the invoice must be paid, or 100% of receipt used
    pub unapplied_handling: String, // "LEAVE_UNAPPLIED", "ON_ACCOUNT"
}

#[derive(Debug, PartialEq, Clone)]
pub struct CashReceipt {
    pub receipt_id: String,
    pub customer_id: String,
    pub receipt_amount: f64,
    // Optional references provided by the remitter (e.g., on a bank statement)
    pub reference_invoice_number: Option<String>,
    pub reference_po_number: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpenInvoice {
    pub invoice_id: String,
    pub invoice_number: String,
    pub po_number: Option<String>,
    pub customer_id: String,
    pub amount_due: f64,
}

#[derive(Debug, PartialEq)]
pub struct AutoMatchResult {
    pub receipt_id: String,
    pub matched_invoice_id: Option<String>,
    pub applied_amount: f64,
    pub unapplied_amount: f64,
    pub status: String, // 'APPLIED', 'ON_ACCOUNT', 'UNAPPLIED'
}

impl AutoMatchService {
    /// Attempts to automatically apply a cash receipt to a list of open invoices for the same customer
    /// based on the criteria defined in the AutoMatchRuleSet.
    #[must_use]
    pub fn execute_automatch(
        receipt: &CashReceipt,
        open_invoices: &[OpenInvoice],
        rule_set: &AutoMatchRuleSet,
    ) -> AutoMatchResult {
        let mut candidate_invoices: Vec<&OpenInvoice> = Vec::new();

        // Filter invoices by customer
        let customer_invoices: Vec<&OpenInvoice> = open_invoices
            .iter()
            .filter(|i| i.customer_id == receipt.customer_id)
            .collect();

        // 1. Evaluate Matching Rules
        for inv in &customer_invoices {
            let mut is_match = false;

            if rule_set.match_by_invoice_number {
                if let Some(ref ref_inv) = receipt.reference_invoice_number {
                    if ref_inv == &inv.invoice_number {
                        is_match = true;
                    }
                }
            }

            if !is_match && rule_set.match_by_purchase_order {
                if let (Some(ref ref_po), Some(ref inv_po)) = (&receipt.reference_po_number, &inv.po_number) {
                    if ref_po == inv_po {
                        is_match = true;
                    }
                }
            }

            if is_match {
                candidate_invoices.push(inv);
            }
        }

        // 2. Resolve Candidate Matches
        if candidate_invoices.is_empty() {
            return Self::handle_unapplied(receipt, rule_set);
        }

        // Simple resolution: If exactly one match, apply it.
        // If multiple matches, it's ambiguous, fallback to unapplied unless we have a perfect amount match.
        let target_invoice = if candidate_invoices.len() == 1 {
            candidate_invoices[0]
        } else {
            // Find an exact amount match among candidates
            let exact_amount_matches: Vec<&&OpenInvoice> = candidate_invoices
                .iter()
                .filter(|i| (i.amount_due - receipt.receipt_amount).abs() < 0.001)
                .collect();

            if exact_amount_matches.len() == 1 {
                exact_amount_matches[0]
            } else {
                return Self::handle_unapplied(receipt, rule_set);
            }
        };

        // 3. Evaluate Minimum Match Percentage
        let applied_amount = receipt.receipt_amount.min(target_invoice.amount_due);
        let match_percentage = (applied_amount / target_invoice.amount_due) * 100.0;

        if match_percentage < rule_set.min_match_percentage {
            // Did not meet minimum match threshold (e.g. they underpaid and we require 100%)
            return Self::handle_unapplied(receipt, rule_set);
        }

        let unapplied = receipt.receipt_amount - applied_amount;
        let mut final_status = "APPLIED".to_string();

        if unapplied > 0.001 && rule_set.unapplied_handling == "ON_ACCOUNT" {
            final_status = "APPLIED_WITH_ON_ACCOUNT".to_string();
        }

        AutoMatchResult {
            receipt_id: receipt.receipt_id.clone(),
            matched_invoice_id: Some(target_invoice.invoice_id.clone()),
            applied_amount,
            unapplied_amount: unapplied,
            status: final_status,
        }
    }

    fn handle_unapplied(receipt: &CashReceipt, rule_set: &AutoMatchRuleSet) -> AutoMatchResult {
        let status = if rule_set.unapplied_handling == "ON_ACCOUNT" {
            "ON_ACCOUNT"
        } else {
            "UNAPPLIED"
        };

        AutoMatchResult {
            receipt_id: receipt.receipt_id.clone(),
            matched_invoice_id: None,
            applied_amount: 0.0,
            unapplied_amount: receipt.receipt_amount,
            status: status.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_invoice_match() {
        let rule_set = AutoMatchRuleSet {
            rule_set_name: "Standard Invoice Match".to_string(),
            match_by_invoice_number: true,
            match_by_purchase_order: false,
            min_match_percentage: 100.0,
            unapplied_handling: "LEAVE_UNAPPLIED".to_string(),
        };

        let receipt = CashReceipt {
            receipt_id: "RCPT-001".to_string(),
            customer_id: "CUST-100".to_string(),
            receipt_amount: 500.0,
            reference_invoice_number: Some("INV-2025-01".to_string()),
            reference_po_number: None,
        };

        let open_invoices = vec![
            OpenInvoice {
                invoice_id: "ID-1".to_string(),
                invoice_number: "INV-2025-01".to_string(),
                po_number: None,
                customer_id: "CUST-100".to_string(),
                amount_due: 500.0,
            },
            OpenInvoice {
                invoice_id: "ID-2".to_string(),
                invoice_number: "INV-2025-02".to_string(),
                po_number: None,
                customer_id: "CUST-100".to_string(),
                amount_due: 300.0,
            }
        ];

        let result = AutoMatchService::execute_automatch(&receipt, &open_invoices, &rule_set);
        assert_eq!(result.status, "APPLIED");
        assert_eq!(result.matched_invoice_id, Some("ID-1".to_string()));
        assert_eq!(result.applied_amount, 500.0);
        assert_eq!(result.unapplied_amount, 0.0);
    }

    #[test]
    fn test_po_number_match() {
        let rule_set = AutoMatchRuleSet {
            rule_set_name: "PO Match".to_string(),
            match_by_invoice_number: false, // Turned off intentionally
            match_by_purchase_order: true,
            min_match_percentage: 100.0,
            unapplied_handling: "ON_ACCOUNT".to_string(),
        };

        let receipt = CashReceipt {
            receipt_id: "RCPT-002".to_string(),
            customer_id: "CUST-200".to_string(),
            receipt_amount: 1000.0,
            reference_invoice_number: Some("UNKNOWN".to_string()),
            reference_po_number: Some("PO-999".to_string()),
        };

        let open_invoices = vec![
            OpenInvoice {
                invoice_id: "ID-3".to_string(),
                invoice_number: "INV-10".to_string(),
                po_number: Some("PO-999".to_string()),
                customer_id: "CUST-200".to_string(),
                amount_due: 1000.0,
            }
        ];

        let result = AutoMatchService::execute_automatch(&receipt, &open_invoices, &rule_set);
        assert_eq!(result.status, "APPLIED");
        assert_eq!(result.matched_invoice_id, Some("ID-3".to_string()));
    }

    #[test]
    fn test_underpayment_rejected_by_threshold() {
        let rule_set = AutoMatchRuleSet {
            rule_set_name: "Strict Match".to_string(),
            match_by_invoice_number: true,
            match_by_purchase_order: false,
            min_match_percentage: 100.0, // Must pay full invoice
            unapplied_handling: "LEAVE_UNAPPLIED".to_string(),
        };

        let receipt = CashReceipt {
            receipt_id: "RCPT-003".to_string(),
            customer_id: "CUST-300".to_string(),
            receipt_amount: 200.0, // Underpaying the 500 invoice
            reference_invoice_number: Some("INV-50".to_string()),
            reference_po_number: None,
        };

        let open_invoices = vec![
            OpenInvoice {
                invoice_id: "ID-4".to_string(),
                invoice_number: "INV-50".to_string(),
                po_number: None,
                customer_id: "CUST-300".to_string(),
                amount_due: 500.0,
            }
        ];

        let result = AutoMatchService::execute_automatch(&receipt, &open_invoices, &rule_set);
        assert_eq!(result.status, "UNAPPLIED"); // Rejected because 200/500 is 40%, which is < 100%
        assert_eq!(result.matched_invoice_id, None);
        assert_eq!(result.unapplied_amount, 200.0);
    }

    #[test]
    fn test_overpayment_handled_on_account() {
        let rule_set = AutoMatchRuleSet {
            rule_set_name: "Overpayment On Account".to_string(),
            match_by_invoice_number: true,
            match_by_purchase_order: false,
            min_match_percentage: 100.0,
            unapplied_handling: "ON_ACCOUNT".to_string(),
        };

        let receipt = CashReceipt {
            receipt_id: "RCPT-004".to_string(),
            customer_id: "CUST-400".to_string(),
            receipt_amount: 600.0, // Overpaying
            reference_invoice_number: Some("INV-60".to_string()),
            reference_po_number: None,
        };

        let open_invoices = vec![
            OpenInvoice {
                invoice_id: "ID-5".to_string(),
                invoice_number: "INV-60".to_string(),
                po_number: None,
                customer_id: "CUST-400".to_string(),
                amount_due: 500.0,
            }
        ];

        let result = AutoMatchService::execute_automatch(&receipt, &open_invoices, &rule_set);
        assert_eq!(result.status, "APPLIED_WITH_ON_ACCOUNT");
        assert_eq!(result.matched_invoice_id, Some("ID-5".to_string()));
        assert_eq!(result.applied_amount, 500.0); // Only applies up to amount due
        assert_eq!(result.unapplied_amount, 100.0); // Remaining on account
    }

    #[test]
    fn test_ambiguous_match_amount_resolution() {
        let rule_set = AutoMatchRuleSet {
            rule_set_name: "PO Match".to_string(),
            match_by_invoice_number: false,
            match_by_purchase_order: true, // Matching by PO which has multiple invoices
            min_match_percentage: 100.0,
            unapplied_handling: "LEAVE_UNAPPLIED".to_string(),
        };

        let receipt = CashReceipt {
            receipt_id: "RCPT-005".to_string(),
            customer_id: "CUST-500".to_string(),
            receipt_amount: 300.0, 
            reference_invoice_number: None,
            reference_po_number: Some("PO-MULTI".to_string()),
        };

        let open_invoices = vec![
            OpenInvoice {
                invoice_id: "ID-6".to_string(),
                invoice_number: "INV-61".to_string(),
                po_number: Some("PO-MULTI".to_string()),
                customer_id: "CUST-500".to_string(),
                amount_due: 500.0,
            },
            OpenInvoice {
                invoice_id: "ID-7".to_string(),
                invoice_number: "INV-62".to_string(),
                po_number: Some("PO-MULTI".to_string()),
                customer_id: "CUST-500".to_string(),
                amount_due: 300.0, // This is the exact amount match
            }
        ];

        let result = AutoMatchService::execute_automatch(&receipt, &open_invoices, &rule_set);
        assert_eq!(result.status, "APPLIED");
        assert_eq!(result.matched_invoice_id, Some("ID-7".to_string())); // Resolved via exact amount
    }
}
