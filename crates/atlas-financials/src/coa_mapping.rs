//! Oracle Fusion Financial Feature: GL Chart of Accounts (COA) Mapping
//! Translates account combinations from a Source Chart of Accounts to a Target Chart of Accounts.

use std::collections::HashMap;

pub struct CoaMappingService;

#[derive(Debug, PartialEq, Clone)]
pub struct CoaSegmentRule {
    pub target_segment_name: String,
    pub action_type: String, // 'COPY_FROM_SOURCE', 'ASSIGN_VALUE'
    pub source_segment_name: Option<String>,
    pub constant_value: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CoaMapping {
    pub mapping_name: String,
    pub target_coa_segments: Vec<String>, // The ordered structure of the target COA
    pub segment_rules: Vec<CoaSegmentRule>,
}

#[derive(Debug, PartialEq)]
pub struct MappingResult {
    pub is_successful: bool,
    pub target_account_combination: Option<String>,
    pub error_message: Option<String>,
}

impl CoaMappingService {
    /// Translates a source account combination (provided as a key-value map of segment names to values)
    /// into a target account combination string (segments joined by '-') based on the mapping rules.
    #[must_use]
    pub fn translate_account(
        mapping: &CoaMapping,
        source_segments: &HashMap<String, String>,
    ) -> MappingResult {
        let mut target_combination_parts = Vec::new();

        for target_segment in &mapping.target_coa_segments {
            // Find the rule for this target segment
            let rule_opt = mapping.segment_rules.iter().find(|r| r.target_segment_name == *target_segment);
            
            let rule = match rule_opt {
                Some(r) => r,
                None => {
                    return MappingResult {
                        is_successful: false,
                        target_account_combination: None,
                        error_message: Some(format!("No mapping rule found for target segment '{}'", target_segment)),
                    };
                }
            };

            let derived_value = match rule.action_type.as_str() {
                "ASSIGN_VALUE" => {
                    if let Some(ref val) = rule.constant_value {
                        val.clone()
                    } else {
                        return MappingResult {
                            is_successful: false,
                            target_account_combination: None,
                            error_message: Some(format!("Missing constant_value for ASSIGN_VALUE rule on '{}'", target_segment)),
                        };
                    }
                },
                "COPY_FROM_SOURCE" => {
                    if let Some(ref src_seg) = rule.source_segment_name {
                        if let Some(val) = source_segments.get(src_seg) {
                            val.clone()
                        } else {
                            return MappingResult {
                                is_successful: false,
                                target_account_combination: None,
                                error_message: Some(format!("Source segment '{}' not found in provided source combination", src_seg)),
                            };
                        }
                    } else {
                        return MappingResult {
                            is_successful: false,
                            target_account_combination: None,
                            error_message: Some(format!("Missing source_segment_name for COPY_FROM_SOURCE rule on '{}'", target_segment)),
                        };
                    }
                },
                _ => {
                    return MappingResult {
                        is_successful: false,
                        target_account_combination: None,
                        error_message: Some(format!("Invalid action_type '{}' on '{}'", rule.action_type, target_segment)),
                    };
                }
            };

            target_combination_parts.push(derived_value);
        }

        MappingResult {
            is_successful: true,
            target_account_combination: Some(target_combination_parts.join("-")),
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_translation() {
        let mapping = CoaMapping {
            mapping_name: "US_TO_GLOBAL".to_string(),
            // The global target COA has 3 segments: GlobalCompany-GlobalDept-GlobalAccount
            target_coa_segments: vec!["GlobalCompany".to_string(), "GlobalDept".to_string(), "GlobalAccount".to_string()],
            segment_rules: vec![
                CoaSegmentRule {
                    target_segment_name: "GlobalCompany".to_string(),
                    action_type: "ASSIGN_VALUE".to_string(),
                    source_segment_name: None,
                    constant_value: Some("99".to_string()), // Constant override
                },
                CoaSegmentRule {
                    target_segment_name: "GlobalDept".to_string(),
                    action_type: "COPY_FROM_SOURCE".to_string(),
                    source_segment_name: Some("US_CostCenter".to_string()), // Maps directly
                    constant_value: None,
                },
                CoaSegmentRule {
                    target_segment_name: "GlobalAccount".to_string(),
                    action_type: "COPY_FROM_SOURCE".to_string(),
                    source_segment_name: Some("US_Account".to_string()), // Maps directly
                    constant_value: None,
                },
            ],
        };

        // Incoming Source Combination: Company = 01, CostCenter = 500, Account = 4000
        let mut source_segs = HashMap::new();
        source_segs.insert("US_Company".to_string(), "01".to_string());
        source_segs.insert("US_CostCenter".to_string(), "500".to_string());
        source_segs.insert("US_Account".to_string(), "4000".to_string());

        let result = CoaMappingService::translate_account(&mapping, &source_segs);

        assert!(result.is_successful);
        assert_eq!(result.target_account_combination.unwrap(), "99-500-4000");
    }

    #[test]
    fn test_missing_rule_for_target_segment() {
        let mapping = CoaMapping {
            mapping_name: "INCOMPLETE_MAPPING".to_string(),
            target_coa_segments: vec!["Company".to_string(), "Account".to_string()],
            segment_rules: vec![
                CoaSegmentRule {
                    target_segment_name: "Company".to_string(),
                    action_type: "ASSIGN_VALUE".to_string(),
                    source_segment_name: None,
                    constant_value: Some("01".to_string()),
                },
                // Missing rule for "Account"
            ],
        };

        let source_segs = HashMap::new();
        let result = CoaMappingService::translate_account(&mapping, &source_segs);

        assert!(!result.is_successful);
        assert_eq!(result.error_message.unwrap(), "No mapping rule found for target segment 'Account'");
    }

    #[test]
    fn test_missing_source_segment_in_input() {
        let mapping = CoaMapping {
            mapping_name: "FAULTY_INPUT".to_string(),
            target_coa_segments: vec!["Account".to_string()],
            segment_rules: vec![
                CoaSegmentRule {
                    target_segment_name: "Account".to_string(),
                    action_type: "COPY_FROM_SOURCE".to_string(),
                    source_segment_name: Some("SourceAccount".to_string()),
                    constant_value: None,
                },
            ],
        };

        let mut source_segs = HashMap::new();
        source_segs.insert("DifferentAccount".to_string(), "4000".to_string()); // Wrong key provided

        let result = CoaMappingService::translate_account(&mapping, &source_segs);

        assert!(!result.is_successful);
        assert_eq!(result.error_message.unwrap(), "Source segment 'SourceAccount' not found in provided source combination");
    }

    #[test]
    fn test_invalid_action_type() {
        let mapping = CoaMapping {
            mapping_name: "INVALID_ACTION".to_string(),
            target_coa_segments: vec!["Account".to_string()],
            segment_rules: vec![
                CoaSegmentRule {
                    target_segment_name: "Account".to_string(),
                    action_type: "MAGIC_WAND".to_string(), // Invalid action
                    source_segment_name: None,
                    constant_value: None,
                },
            ],
        };

        let source_segs = HashMap::new();
        let result = CoaMappingService::translate_account(&mapping, &source_segs);

        assert!(!result.is_successful);
        assert_eq!(result.error_message.unwrap(), "Invalid action_type 'MAGIC_WAND' on 'Account'");
    }
}
