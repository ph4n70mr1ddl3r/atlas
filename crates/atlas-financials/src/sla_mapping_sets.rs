//! Oracle Fusion Financial Feature: Subledger Accounting (SLA) Mapping Sets
//! Maps transaction input attributes to specific General Ledger segment values.

use chrono::NaiveDate;

pub struct SlaMappingSetService;

#[derive(Debug, PartialEq, Clone)]
pub struct MappingSetRule {
    pub input_value: String,
    pub output_value: String,
    pub effective_start_date: NaiveDate,
    pub effective_end_date: Option<NaiveDate>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MappingSet {
    pub mapping_set_code: String,
    pub output_segment: String,
    pub use_default_value: bool,
    pub default_output_value: Option<String>,
    pub rules: Vec<MappingSetRule>,
}

#[derive(Debug, PartialEq)]
pub struct MappingResult {
    pub mapping_set_code: String,
    pub output_segment: String,
    pub mapped_value: Option<String>,
    pub status: String, // 'MAPPED_VIA_RULE', 'MAPPED_VIA_DEFAULT', 'UNMAPPED'
}

impl SlaMappingSetService {
    /// Evaluates a transaction's input value against an SLA Mapping Set to derive a GL segment.
    #[must_use]
    pub fn map_segment_value(
        mapping_set: &MappingSet,
        transaction_input: &str,
        transaction_date: NaiveDate,
    ) -> MappingResult {
        // Find a matching rule that is active on the transaction date
        let matching_rule = mapping_set.rules.iter().find(|rule| {
            let matches_input = rule.input_value == transaction_input;
            let is_started = transaction_date >= rule.effective_start_date;
            let is_not_expired = match rule.effective_end_date {
                Some(end) => transaction_date <= end,
                None => true,
            };

            matches_input && is_started && is_not_expired
        });

        if let Some(rule) = matching_rule {
            return MappingResult {
                mapping_set_code: mapping_set.mapping_set_code.clone(),
                output_segment: mapping_set.output_segment.clone(),
                mapped_value: Some(rule.output_value.clone()),
                status: "MAPPED_VIA_RULE".to_string(),
            };
        }

        // Fallback to default if configured
        if mapping_set.use_default_value {
            if let Some(ref default_val) = mapping_set.default_output_value {
                return MappingResult {
                    mapping_set_code: mapping_set.mapping_set_code.clone(),
                    output_segment: mapping_set.output_segment.clone(),
                    mapped_value: Some(default_val.clone()),
                    status: "MAPPED_VIA_DEFAULT".to_string(),
                };
            }
        }

        MappingResult {
            mapping_set_code: mapping_set.mapping_set_code.clone(),
            output_segment: mapping_set.output_segment.clone(),
            mapped_value: None,
            status: "UNMAPPED".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_sample_mapping_set() -> MappingSet {
        MappingSet {
            mapping_set_code: "ITEM_CATEGORY_TO_ACCOUNT".to_string(),
            output_segment: "ACCOUNT".to_string(),
            use_default_value: true,
            default_output_value: Some("9999".to_string()),
            rules: vec![
                MappingSetRule {
                    input_value: "HARDWARE".to_string(),
                    output_value: "1200".to_string(),
                    effective_start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    effective_end_date: None,
                },
                MappingSetRule {
                    input_value: "SOFTWARE".to_string(),
                    output_value: "1300".to_string(),
                    effective_start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    effective_end_date: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
                },
                MappingSetRule {
                    input_value: "SOFTWARE".to_string(),
                    output_value: "1350".to_string(), // New account from 2025 onwards
                    effective_start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    effective_end_date: None,
                },
            ],
        }
    }

    #[test]
    fn test_mapped_via_rule_hardware() {
        let mapping_set = get_sample_mapping_set();
        let trx_date = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        
        let result = SlaMappingSetService::map_segment_value(&mapping_set, "HARDWARE", trx_date);
        
        assert_eq!(result.status, "MAPPED_VIA_RULE");
        assert_eq!(result.mapped_value, Some("1200".to_string()));
    }

    #[test]
    fn test_mapped_via_rule_software_old_date() {
        let mapping_set = get_sample_mapping_set();
        let trx_date = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        
        let result = SlaMappingSetService::map_segment_value(&mapping_set, "SOFTWARE", trx_date);
        
        assert_eq!(result.status, "MAPPED_VIA_RULE");
        assert_eq!(result.mapped_value, Some("1300".to_string()));
    }

    #[test]
    fn test_mapped_via_rule_software_new_date() {
        let mapping_set = get_sample_mapping_set();
        let trx_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        
        let result = SlaMappingSetService::map_segment_value(&mapping_set, "SOFTWARE", trx_date);
        
        assert_eq!(result.status, "MAPPED_VIA_RULE");
        assert_eq!(result.mapped_value, Some("1350".to_string()));
    }

    #[test]
    fn test_mapped_via_default_value() {
        let mapping_set = get_sample_mapping_set();
        let trx_date = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        
        // Input "SERVICES" has no specific rule, should fallback to default "9999"
        let result = SlaMappingSetService::map_segment_value(&mapping_set, "SERVICES", trx_date);
        
        assert_eq!(result.status, "MAPPED_VIA_DEFAULT");
        assert_eq!(result.mapped_value, Some("9999".to_string()));
    }

    #[test]
    fn test_unmapped_when_no_default() {
        let mut mapping_set = get_sample_mapping_set();
        mapping_set.use_default_value = false; // Turn off defaults
        mapping_set.default_output_value = None;
        
        let trx_date = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        
        // Input "SERVICES" has no specific rule, and no default allowed -> UNMAPPED
        let result = SlaMappingSetService::map_segment_value(&mapping_set, "SERVICES", trx_date);
        
        assert_eq!(result.status, "UNMAPPED");
        assert_eq!(result.mapped_value, None);
    }
}
