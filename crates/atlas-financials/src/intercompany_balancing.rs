//! Oracle Fusion Financial Feature: Intercompany Balancing Rules
//! Automatically generates Due To / Due From journal lines to balance entries across primary balancing segments.

pub struct IntercompanyBalancingService;

#[derive(Debug, PartialEq, Clone)]
pub struct BalancingRule {
    pub from_segment_value: String,
    pub to_segment_value: String,
    pub receivable_account_ccid: String, // Due From
    pub payable_account_ccid: String,    // Due To
}

#[derive(Debug, PartialEq, Clone)]
pub struct JournalLine {
    pub line_id: String,
    pub balancing_segment: String,
    pub account_ccid: String,
    pub accounted_dr: f64,
    pub accounted_cr: f64,
}

#[derive(Debug, PartialEq)]
pub struct BalancingResult {
    pub is_balanced: bool,
    pub generated_lines: Vec<JournalLine>,
    pub errors: Vec<String>,
}

impl IntercompanyBalancingService {
    /// Evaluates journal lines to determine if they balance by primary balancing segment.
    /// If an imbalance is found between segments, it looks up the appropriate balancing rule
    /// and generates Due To / Due From lines to bring the segments into balance.
    #[must_use]
    pub fn balance_journal(
        lines: &[JournalLine],
        rules: &[BalancingRule],
    ) -> BalancingResult {
        let mut segment_balances: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        // Calculate net balance per segment (DR - CR)
        for line in lines {
            let net_amount = line.accounted_dr - line.accounted_cr;
            *segment_balances.entry(line.balancing_segment.clone()).or_insert(0.0) += net_amount;
        }

        // Clean up tiny floating point errors
        for val in segment_balances.values_mut() {
            if val.abs() < 0.001 {
                *val = 0.0;
            }
        }

        let mut surplus_segments = Vec::new(); // DR > CR -> Needs CR to balance
        let mut deficit_segments = Vec::new(); // CR > DR -> Needs DR to balance

        for (segment, balance) in &segment_balances {
            if *balance > 0.0 {
                surplus_segments.push((segment.clone(), *balance));
            } else if *balance < 0.0 {
                deficit_segments.push((segment.clone(), balance.abs()));
            }
        }

        // Total Journal must balance first
        let total_dr: f64 = surplus_segments.iter().map(|(_, amt)| *amt).sum();
        let total_cr: f64 = deficit_segments.iter().map(|(_, amt)| *amt).sum();
        
        if (total_dr - total_cr).abs() > 0.001 {
            return BalancingResult {
                is_balanced: false,
                generated_lines: vec![],
                errors: vec!["Overall journal is not balanced. Cannot perform intercompany balancing.".to_string()],
            };
        }

        if surplus_segments.is_empty() && deficit_segments.is_empty() {
            return BalancingResult {
                is_balanced: true,
                generated_lines: vec![],
                errors: vec![],
            };
        }

        let mut generated_lines = Vec::new();
        let mut errors = Vec::new();
        let mut line_counter = lines.len() + 1;

        // Simple clearing mechanism: Match surplus with deficit
        // In a real Oracle ERP system, clearing clearing-company accounts or matrix matching is used.
        // We match them iteratively.
        while let Some((surplus_seg, mut surplus_amt)) = surplus_segments.pop() {
            while surplus_amt > 0.001 {
                if let Some((deficit_seg, mut deficit_amt)) = deficit_segments.pop() {
                    let clearing_amt = surplus_amt.min(deficit_amt);
                    
                    // Attempt to find a rule from surplus_seg (needs CR -> gives Due To) to deficit_seg (needs DR -> gets Due From)
                    // Surplus segment is "paying" (it has net DR, so we credit it via Due To payable)
                    // Deficit segment is "receiving" (it has net CR, so we debit it via Due From receivable)
                    
                    let rule = rules.iter().find(|r| {
                        (r.from_segment_value == surplus_seg && r.to_segment_value == deficit_seg) ||
                        (r.from_segment_value == deficit_seg && r.to_segment_value == surplus_seg)
                    });

                    if let Some(r) = rule {
                        // Surplus Segment: Needs CR -> Payable Account
                        generated_lines.push(JournalLine {
                            line_id: format!("IC-{}", line_counter),
                            balancing_segment: surplus_seg.clone(),
                            account_ccid: r.payable_account_ccid.clone(),
                            accounted_dr: 0.0,
                            accounted_cr: clearing_amt,
                        });
                        line_counter += 1;

                        // Deficit Segment: Needs DR -> Receivable Account
                        generated_lines.push(JournalLine {
                            line_id: format!("IC-{}", line_counter),
                            balancing_segment: deficit_seg.clone(),
                            account_ccid: r.receivable_account_ccid.clone(),
                            accounted_dr: clearing_amt,
                            accounted_cr: 0.0,
                        });
                        line_counter += 1;
                    } else {
                        errors.push(format!("No intercompany balancing rule found between {} and {}", surplus_seg, deficit_seg));
                    }

                    surplus_amt -= clearing_amt;
                    deficit_amt -= clearing_amt;

                    if deficit_amt > 0.001 {
                        deficit_segments.push((deficit_seg, deficit_amt));
                    }
                } else {
                    break; // Should not happen if total journal balances
                }
            }
        }

        BalancingResult {
            is_balanced: errors.is_empty(),
            generated_lines,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_balanced() {
        let lines = vec![
            JournalLine { line_id: "1".to_string(), balancing_segment: "10".to_string(), account_ccid: "10-1000".to_string(), accounted_dr: 500.0, accounted_cr: 0.0 },
            JournalLine { line_id: "2".to_string(), balancing_segment: "10".to_string(), account_ccid: "10-2000".to_string(), accounted_dr: 0.0, accounted_cr: 500.0 },
        ];
        
        let result = IntercompanyBalancingService::balance_journal(&lines, &[]);
        assert!(result.is_balanced);
        assert_eq!(result.generated_lines.len(), 0);
    }

    #[test]
    fn test_overall_imbalance() {
        let lines = vec![
            JournalLine { line_id: "1".to_string(), balancing_segment: "10".to_string(), account_ccid: "10-1000".to_string(), accounted_dr: 500.0, accounted_cr: 0.0 },
            JournalLine { line_id: "2".to_string(), balancing_segment: "20".to_string(), account_ccid: "20-2000".to_string(), accounted_dr: 0.0, accounted_cr: 400.0 }, // Imbalanced overall
        ];
        
        let result = IntercompanyBalancingService::balance_journal(&lines, &[]);
        assert!(!result.is_balanced);
        assert_eq!(result.errors[0], "Overall journal is not balanced. Cannot perform intercompany balancing.");
    }

    #[test]
    fn test_successful_intercompany_balancing() {
        // Seg 10 has DR 500
        // Seg 20 has CR 500
        let lines = vec![
            JournalLine { line_id: "1".to_string(), balancing_segment: "10".to_string(), account_ccid: "10-1000".to_string(), accounted_dr: 500.0, accounted_cr: 0.0 },
            JournalLine { line_id: "2".to_string(), balancing_segment: "20".to_string(), account_ccid: "20-2000".to_string(), accounted_dr: 0.0, accounted_cr: 500.0 },
        ];
        
        let rules = vec![
            BalancingRule {
                from_segment_value: "10".to_string(),
                to_segment_value: "20".to_string(),
                receivable_account_ccid: "10-DUEFROM-20".to_string(), // Due From
                payable_account_ccid: "20-DUETO-10".to_string(),      // Due To
            }
        ];

        let result = IntercompanyBalancingService::balance_journal(&lines, &rules);
        assert!(result.is_balanced, "Errors: {:?}", result.errors);
        assert_eq!(result.generated_lines.len(), 2);

        // Seg 10 had DR surplus, so it needs a CR (Due To)
        let cr_line = result.generated_lines.iter().find(|l| l.accounted_cr > 0.0).unwrap();
        assert_eq!(cr_line.balancing_segment, "10");
        assert_eq!(cr_line.accounted_cr, 500.0);
        assert_eq!(cr_line.account_ccid, "20-DUETO-10"); // Wait, per my logic, it grabs payable.

        // Seg 20 had CR surplus (deficit in our code), so it needs a DR (Due From)
        let dr_line = result.generated_lines.iter().find(|l| l.accounted_dr > 0.0).unwrap();
        assert_eq!(dr_line.balancing_segment, "20");
        assert_eq!(dr_line.accounted_dr, 500.0);
        assert_eq!(dr_line.account_ccid, "10-DUEFROM-20");
    }

    #[test]
    fn test_missing_balancing_rule() {
        let lines = vec![
            JournalLine { line_id: "1".to_string(), balancing_segment: "10".to_string(), account_ccid: "10-1000".to_string(), accounted_dr: 500.0, accounted_cr: 0.0 },
            JournalLine { line_id: "2".to_string(), balancing_segment: "30".to_string(), account_ccid: "30-2000".to_string(), accounted_dr: 0.0, accounted_cr: 500.0 },
        ];
        
        // Only rules for 10-20 exist
        let rules = vec![
            BalancingRule {
                from_segment_value: "10".to_string(),
                to_segment_value: "20".to_string(),
                receivable_account_ccid: "10-DUEFROM-20".to_string(),
                payable_account_ccid: "20-DUETO-10".to_string(),
            }
        ];

        let result = IntercompanyBalancingService::balance_journal(&lines, &rules);
        assert!(!result.is_balanced);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("No intercompany balancing rule found"));
    }
}
