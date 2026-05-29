//! Oracle Fusion Financial Feature: Payment Process Profile
//! Manages the configuration for how payments are formatted, grouped, and transmitted.

pub struct PaymentProcessProfileService;

#[derive(Debug, PartialEq, Clone)]
pub struct PaymentProcessProfile {
    pub profile_code: String,
    pub payment_format: String,
    pub group_by_due_date: bool,
    pub group_by_payee: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvoicePayment {
    pub invoice_id: String,
    pub payee_id: String,
    pub due_date: String,
    pub amount: f64,
}

#[derive(Debug, PartialEq)]
pub struct PaymentBatch {
    pub profile_used: String,
    pub payment_format: String,
    pub group_key: String,
    pub total_amount: f64,
    pub invoices: Vec<String>,
}

impl PaymentProcessProfileService {
    /// Applies grouping rules from the Payment Process Profile to a list of invoices.
    /// Returns a list of discrete payment batches that should be generated.
    #[must_use]
    pub fn apply_grouping_rules(
        profile: &PaymentProcessProfile,
        invoices: &[InvoicePayment],
    ) -> Vec<PaymentBatch> {
        let mut grouped: std::collections::HashMap<String, Vec<InvoicePayment>> = std::collections::HashMap::new();

        for inv in invoices {
            // Build a dynamic group key based on the profile's flags
            let mut key_parts = Vec::new();
            if profile.group_by_payee {
                key_parts.push(format!("PAYEE:{}", inv.payee_id));
            }
            if profile.group_by_due_date {
                key_parts.push(format!("DATE:{}", inv.due_date));
            }

            let group_key = if key_parts.is_empty() {
                "GROUP:ALL".to_string()
            } else {
                key_parts.join("|")
            };

            grouped.entry(group_key).or_default().push(inv.clone());
        }

        let mut batches = Vec::new();
        for (key, group_invoices) in grouped {
            let total: f64 = group_invoices.iter().map(|i| i.amount).sum();
            let invoice_ids: Vec<String> = group_invoices.into_iter().map(|i| i.invoice_id).collect();

            batches.push(PaymentBatch {
                profile_used: profile.profile_code.clone(),
                payment_format: profile.payment_format.clone(),
                group_key: key,
                total_amount: total,
                invoices: invoice_ids,
            });
        }

        // Sort batches by group_key to ensure deterministic test results
        batches.sort_by(|a, b| a.group_key.cmp(&b.group_key));
        batches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_payee_only() {
        let profile = PaymentProcessProfile {
            profile_code: "PPP_STANDARD_ACH".to_string(),
            payment_format: "NACHA".to_string(),
            group_by_due_date: false,
            group_by_payee: true, // Should group invoices for the same payee together
        };

        let invoices = vec![
            InvoicePayment { invoice_id: "INV1".to_string(), payee_id: "SUPP-A".to_string(), due_date: "2025-01-01".to_string(), amount: 100.0 },
            InvoicePayment { invoice_id: "INV2".to_string(), payee_id: "SUPP-A".to_string(), due_date: "2025-01-15".to_string(), amount: 200.0 },
            InvoicePayment { invoice_id: "INV3".to_string(), payee_id: "SUPP-B".to_string(), due_date: "2025-01-01".to_string(), amount: 500.0 },
        ];

        let batches = PaymentProcessProfileService::apply_grouping_rules(&profile, &invoices);

        assert_eq!(batches.len(), 2);
        
        let batch_a = batches.iter().find(|b| b.group_key == "PAYEE:SUPP-A").unwrap();
        assert_eq!(batch_a.total_amount, 300.0);
        assert_eq!(batch_a.invoices.len(), 2);
        assert!(batch_a.invoices.contains(&"INV1".to_string()));
        assert!(batch_a.invoices.contains(&"INV2".to_string()));

        let batch_b = batches.iter().find(|b| b.group_key == "PAYEE:SUPP-B").unwrap();
        assert_eq!(batch_b.total_amount, 500.0);
        assert_eq!(batch_b.invoices.len(), 1);
    }

    #[test]
    fn test_group_by_payee_and_date() {
        let profile = PaymentProcessProfile {
            profile_code: "PPP_STRICT".to_string(),
            payment_format: "SEPA".to_string(),
            group_by_due_date: true,
            group_by_payee: true, 
        };

        let invoices = vec![
            InvoicePayment { invoice_id: "INV1".to_string(), payee_id: "SUPP-A".to_string(), due_date: "2025-01-01".to_string(), amount: 100.0 },
            InvoicePayment { invoice_id: "INV2".to_string(), payee_id: "SUPP-A".to_string(), due_date: "2025-01-15".to_string(), amount: 200.0 },
        ];

        let batches = PaymentProcessProfileService::apply_grouping_rules(&profile, &invoices);

        // Even though it's the same payee, the due dates are different, so it should be 2 batches.
        assert_eq!(batches.len(), 2);
        let batch_1 = &batches[0]; // PAYEE:SUPP-A|DATE:2025-01-01
        assert_eq!(batch_1.group_key, "PAYEE:SUPP-A|DATE:2025-01-01");
        assert_eq!(batch_1.total_amount, 100.0);

        let batch_2 = &batches[1]; // PAYEE:SUPP-A|DATE:2025-01-15
        assert_eq!(batch_2.group_key, "PAYEE:SUPP-A|DATE:2025-01-15");
        assert_eq!(batch_2.total_amount, 200.0);
    }

    #[test]
    fn test_no_grouping() {
        let profile = PaymentProcessProfile {
            profile_code: "PPP_SINGLE".to_string(),
            payment_format: "SWIFT".to_string(),
            group_by_due_date: false,
            group_by_payee: false, 
        };

        let invoices = vec![
            InvoicePayment { invoice_id: "INV1".to_string(), payee_id: "SUPP-A".to_string(), due_date: "2025-01-01".to_string(), amount: 100.0 },
            InvoicePayment { invoice_id: "INV2".to_string(), payee_id: "SUPP-B".to_string(), due_date: "2025-01-15".to_string(), amount: 200.0 },
        ];

        let batches = PaymentProcessProfileService::apply_grouping_rules(&profile, &invoices);

        // Group everything into one massive batch (though normally impractical without grouping by currency/payee, useful for specific workflows)
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].group_key, "GROUP:ALL");
        assert_eq!(batches[0].total_amount, 300.0);
        assert_eq!(batches[0].invoices.len(), 2);
    }
}
