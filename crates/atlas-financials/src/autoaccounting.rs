//! Oracle Fusion Financial Feature: AutoAccounting Rules (Receivables)
//! Dynamically derives General Ledger account segments for AR transactions.

pub struct AutoAccountingService;

#[derive(Debug, PartialEq, Clone)]
pub struct AutoAccountingRule {
    pub account_class: String, // e.g., "REVENUE", "RECEIVABLE", "FREIGHT"
    pub segment_name: String,  // e.g., "COMPANY", "COST_CENTER", "ACCOUNT"
    pub source_type: String,   // "CONSTANT", "TRANSACTION_TYPE", "SALESPERSON", "STANDARD_LINE"
    pub constant_value: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransactionContext {
    pub transaction_type_id: String,
    pub salesperson_id: Option<String>,
    pub standard_line_id: Option<String>,
}

/// Simulated reference data lookups. In a real system, these would be DB queries.
pub trait ReferenceDataLookup {
    fn get_segment_from_trx_type(&self, trx_type_id: &str, segment_name: &str) -> Option<String>;
    fn get_segment_from_salesperson(
        &self,
        salesperson_id: &str,
        segment_name: &str,
    ) -> Option<String>;
    fn get_segment_from_standard_line(
        &self,
        std_line_id: &str,
        segment_name: &str,
    ) -> Option<String>;
}

impl AutoAccountingService {
    /// Derives the GL account combination string for a specific account class (e.g., REVENUE)
    /// based on the configured rules and the transaction context.
    pub fn derive_account<T: ReferenceDataLookup>(
        account_class: &str,
        segments_to_build: &[&str], // e.g., ["COMPANY", "COST_CENTER", "ACCOUNT"]
        rules: &[AutoAccountingRule],
        context: &TransactionContext,
        lookup: &T,
    ) -> Result<String, String> {
        let mut combination = Vec::new();

        for &segment in segments_to_build {
            // Find the rule for this account class and segment
            let rule = rules
                .iter()
                .find(|r| r.account_class == account_class && r.segment_name == segment);

            let segment_value = match rule {
                Some(r) => match r.source_type.as_str() {
                    "CONSTANT" => r.constant_value.clone(),
                    "TRANSACTION_TYPE" => {
                        lookup.get_segment_from_trx_type(&context.transaction_type_id, segment)
                    }
                    "SALESPERSON" => {
                        if let Some(ref rep_id) = context.salesperson_id {
                            lookup.get_segment_from_salesperson(rep_id, segment)
                        } else {
                            None
                        }
                    }
                    "STANDARD_LINE" => {
                        if let Some(ref line_id) = context.standard_line_id {
                            lookup.get_segment_from_standard_line(line_id, segment)
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                None => {
                    return Err(format!(
                        "No AutoAccounting rule found for class '{}', segment '{}'",
                        account_class, segment
                    ))
                }
            };

            match segment_value {
                Some(val) => combination.push(val),
                None => {
                    return Err(format!(
                        "Failed to derive segment '{}' for class '{}'",
                        segment, account_class
                    ))
                }
            }
        }

        Ok(combination.join("-"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLookup {
        trx_type_data: HashMap<(String, String), String>,
        salesperson_data: HashMap<(String, String), String>,
        std_line_data: HashMap<(String, String), String>,
    }

    impl ReferenceDataLookup for MockLookup {
        fn get_segment_from_trx_type(
            &self,
            trx_type_id: &str,
            segment_name: &str,
        ) -> Option<String> {
            self.trx_type_data
                .get(&(trx_type_id.to_string(), segment_name.to_string()))
                .cloned()
        }
        fn get_segment_from_salesperson(
            &self,
            salesperson_id: &str,
            segment_name: &str,
        ) -> Option<String> {
            self.salesperson_data
                .get(&(salesperson_id.to_string(), segment_name.to_string()))
                .cloned()
        }
        fn get_segment_from_standard_line(
            &self,
            std_line_id: &str,
            segment_name: &str,
        ) -> Option<String> {
            self.std_line_data
                .get(&(std_line_id.to_string(), segment_name.to_string()))
                .cloned()
        }
    }

    #[test]
    fn test_derive_revenue_account() {
        // Setup mock data mapping
        let mut trx_data = HashMap::new();
        trx_data.insert(
            ("TRX-INV".to_string(), "COMPANY".to_string()),
            "01".to_string(),
        );

        let mut rep_data = HashMap::new();
        rep_data.insert(
            ("REP-500".to_string(), "COST_CENTER".to_string()),
            "200".to_string(),
        );

        let mut line_data = HashMap::new();
        line_data.insert(
            ("ITEM-A".to_string(), "ACCOUNT".to_string()),
            "4000".to_string(),
        );

        let lookup = MockLookup {
            trx_type_data: trx_data,
            salesperson_data: rep_data,
            std_line_data: line_data,
        };

        let rules = vec![
            AutoAccountingRule {
                account_class: "REVENUE".to_string(),
                segment_name: "COMPANY".to_string(),
                source_type: "TRANSACTION_TYPE".to_string(),
                constant_value: None,
            },
            AutoAccountingRule {
                account_class: "REVENUE".to_string(),
                segment_name: "COST_CENTER".to_string(),
                source_type: "SALESPERSON".to_string(),
                constant_value: None,
            },
            AutoAccountingRule {
                account_class: "REVENUE".to_string(),
                segment_name: "ACCOUNT".to_string(),
                source_type: "STANDARD_LINE".to_string(),
                constant_value: None,
            },
        ];

        let context = TransactionContext {
            transaction_type_id: "TRX-INV".to_string(),
            salesperson_id: Some("REP-500".to_string()),
            standard_line_id: Some("ITEM-A".to_string()),
        };

        let segments = ["COMPANY", "COST_CENTER", "ACCOUNT"];
        let result =
            AutoAccountingService::derive_account("REVENUE", &segments, &rules, &context, &lookup);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "01-200-4000");
    }

    #[test]
    fn test_derive_receivable_account_with_constant() {
        let mut trx_data = HashMap::new();
        trx_data.insert(
            ("TRX-INV".to_string(), "COMPANY".to_string()),
            "01".to_string(),
        );
        trx_data.insert(
            ("TRX-INV".to_string(), "COST_CENTER".to_string()),
            "000".to_string(),
        );

        let lookup = MockLookup {
            trx_type_data: trx_data,
            salesperson_data: HashMap::new(),
            std_line_data: HashMap::new(),
        };

        let rules = vec![
            AutoAccountingRule {
                account_class: "RECEIVABLE".to_string(),
                segment_name: "COMPANY".to_string(),
                source_type: "TRANSACTION_TYPE".to_string(),
                constant_value: None,
            },
            AutoAccountingRule {
                account_class: "RECEIVABLE".to_string(),
                segment_name: "COST_CENTER".to_string(),
                source_type: "TRANSACTION_TYPE".to_string(), // Usually defaults to 000 for AR
                constant_value: None,
            },
            AutoAccountingRule {
                account_class: "RECEIVABLE".to_string(),
                segment_name: "ACCOUNT".to_string(),
                source_type: "CONSTANT".to_string(),
                constant_value: Some("1200".to_string()), // Constant AR account
            },
        ];

        let context = TransactionContext {
            transaction_type_id: "TRX-INV".to_string(),
            salesperson_id: None,
            standard_line_id: None,
        };

        let segments = ["COMPANY", "COST_CENTER", "ACCOUNT"];
        let result = AutoAccountingService::derive_account(
            "RECEIVABLE",
            &segments,
            &rules,
            &context,
            &lookup,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "01-000-1200");
    }

    #[test]
    fn test_missing_rule_error() {
        let lookup = MockLookup {
            trx_type_data: HashMap::new(),
            salesperson_data: HashMap::new(),
            std_line_data: HashMap::new(),
        };

        let rules = vec![
            AutoAccountingRule {
                account_class: "FREIGHT".to_string(),
                segment_name: "COMPANY".to_string(),
                source_type: "CONSTANT".to_string(),
                constant_value: Some("01".to_string()),
            },
            // Missing ACCOUNT rule
        ];

        let context = TransactionContext {
            transaction_type_id: "TRX-INV".to_string(),
            salesperson_id: None,
            standard_line_id: None,
        };

        let segments = ["COMPANY", "ACCOUNT"];
        let result =
            AutoAccountingService::derive_account("FREIGHT", &segments, &rules, &context, &lookup);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "No AutoAccounting rule found for class 'FREIGHT', segment 'ACCOUNT'"
        );
    }

    #[test]
    fn test_failed_to_derive_segment() {
        let lookup = MockLookup {
            trx_type_data: HashMap::new(), // Empty lookup data
            salesperson_data: HashMap::new(),
            std_line_data: HashMap::new(),
        };

        let rules = vec![AutoAccountingRule {
            account_class: "REVENUE".to_string(),
            segment_name: "COMPANY".to_string(),
            source_type: "TRANSACTION_TYPE".to_string(),
            constant_value: None,
        }];

        let context = TransactionContext {
            transaction_type_id: "TRX-INV".to_string(), // Has type, but lookup returns None
            salesperson_id: None,
            standard_line_id: None,
        };

        let segments = ["COMPANY"];
        let result =
            AutoAccountingService::derive_account("REVENUE", &segments, &rules, &context, &lookup);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Failed to derive segment 'COMPANY' for class 'REVENUE'"
        );
    }
}
