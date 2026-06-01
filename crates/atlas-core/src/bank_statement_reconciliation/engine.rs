//! Bank Statement Auto-Reconciliation Engine
//!
//! Provides the core matching algorithms and reconciliation logic for
//! automatically reconciling bank statement lines against system transactions
//! (receipts, payments, journal entries).
//!
//! Oracle Fusion Cloud ERP equivalent: Cash Management > Bank Statements > Auto-Reconciliation
//!
//! Key capabilities:
//! - Statement import validation (MT940, BAI2, OFX format simulation)
//! - Configurable matching rules (amount, reference, date, combined)
//! - Auto-matching engine with confidence scoring
//! - Exception detection and management
//! - Reconciliation audit trail
//! - Dashboard reporting

use super::BankStatementReconciliationRepository;
use atlas_shared::AtlasResult;
use std::sync::Arc;
// tracing macros used when adding logging in future

// ============================================================================
// Constants — Valid values for validation
// ============================================================================

/// Valid bank statement statuses
pub const VALID_STATEMENT_STATUSES: &[&str] = &[
    "imported",
    "validating",
    "validated",
    "reconciling",
    "reconciled",
    "exception",
    "cancelled",
];

/// Valid import sources
pub const VALID_IMPORT_SOURCES: &[&str] = &["mt940", "bai2", "ofx", "csv", "manual", "api"];

/// Valid line match statuses
pub const VALID_LINE_MATCH_STATUSES: &[&str] = &[
    "unmatched",
    "matched",
    "partially_matched",
    "exception",
    "manually_matched",
    "excluded",
];

/// Valid match strategies for reconciliation rules
pub const VALID_MATCH_STRATEGIES: &[&str] = &[
    "exact_amount",
    "amount_tolerance",
    "reference_match",
    "date_range",
    "combined_amount_reference",
    "combined_amount_date",
    "fuzzy_match",
];

/// Valid exception types
pub const VALID_EXCEPTION_TYPES: &[&str] = &[
    "unmatched",
    "multiple_match",
    "amount_mismatch",
    "date_out_of_range",
];

/// Valid exception resolution statuses
pub const VALID_RESOLUTION_STATUSES: &[&str] = &[
    "open",
    "resolved_matched",
    "resolved_write_off",
    "resolved_excluded",
    "resolved_adjustment",
];

/// Valid audit actions
pub const VALID_AUDIT_ACTIONS: &[&str] = &[
    "imported",
    "validated",
    "auto_matched",
    "manually_matched",
    "exception_created",
    "exception_resolved",
    "reconciliation_completed",
    "reconciliation_reversed",
];

/// Valid target transaction types for matching rules
pub const VALID_TARGET_TRANSACTION_TYPES: &[&str] =
    &["receipt", "payment", "journal_entry", "bank_charge", "all"];

// ============================================================================
// Engine
// ============================================================================

/// Bank Statement Reconciliation Engine
///
/// Orchestrates the auto-reconciliation workflow:
/// 1. Import/validate bank statement
/// 2. Load active matching rules (sorted by priority)
/// 3. For each unmatched statement line, attempt matching
/// 4. Generate exceptions for unmatched or ambiguous items
/// 5. Produce reconciliation summary
pub struct BankStatementReconciliationEngine {
    #[allow(dead_code)]
    repository: Arc<dyn BankStatementReconciliationRepository>,
}

impl BankStatementReconciliationEngine {
    pub fn new(repository: Arc<dyn BankStatementReconciliationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Statement Validation
    // ========================================================================

    /// Validate a bank statement header for consistency
    ///
    /// Oracle Fusion: Cash Management > Bank Statements > Validate
    pub fn validate_statement(
        opening_balance: f64,
        closing_balance: f64,
        total_credits: f64,
        total_debits: f64,
    ) -> AtlasResult<()> {
        let expected_closing = opening_balance + total_credits - total_debits;
        let difference = (closing_balance - expected_closing).abs();

        if difference > 0.01 {
            return Err(atlas_shared::AtlasError::ValidationFailed(
                format!(
                    "Statement balance inconsistency: expected closing {expected_closing:.2} but got {closing_balance:.2} (diff {difference:.2})"
                ),
            ));
        }

        Ok(())
    }

    /// Validate an MT940-format statement reference (basic check)
    pub fn validate_mt940_reference(reference: &str) -> AtlasResult<()> {
        if reference.is_empty() {
            return Err(atlas_shared::AtlasError::ValidationFailed(
                "MT940 reference cannot be empty".to_string(),
            ));
        }
        if reference.len() > 16 {
            return Err(atlas_shared::AtlasError::ValidationFailed(
                "MT940 field 61 reference must be ≤16 characters".to_string(),
            ));
        }
        Ok(())
    }

    // ========================================================================
    // Matching Algorithms
    // ========================================================================

    /// Match a single statement line against system transactions using
    /// an exact amount match strategy.
    ///
    /// Returns the index of the matching system transaction, if any.
    #[must_use]
    pub fn match_exact_amount(
        statement_amount: f64,
        statement_type: &str, // "credit" or "debit"
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
        // (amount, txn_type, reference, date)
    ) -> Option<usize> {
        for (i, (sys_amount, _txn_type, _reference, _date)) in
            system_transactions.iter().enumerate()
        {
            if (statement_amount - sys_amount).abs() < 0.005 {
                // Credit on statement = inflow = receipt; Debit = outflow = payment
                let expected_type = if statement_type == "credit" {
                    "receipt"
                } else {
                    "payment"
                };
                if *_txn_type == expected_type || *_txn_type == "all" {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Match a statement line against system transactions using
    /// amount tolerance matching.
    ///
    /// Returns all candidate matches within the tolerance band.
    #[must_use]
    pub fn match_amount_tolerance(
        statement_amount: f64,
        tolerance_pct: f64,
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
    ) -> Vec<(usize, f64)> {
        let mut matches = Vec::new();
        for (i, (sys_amount, _txn_type, _ref, _date)) in system_transactions.iter().enumerate() {
            if *sys_amount > 0.0 {
                let pct_diff = ((statement_amount - sys_amount).abs() / *sys_amount) * 100.0;
                if pct_diff <= tolerance_pct {
                    matches.push((i, pct_diff));
                }
            }
        }
        matches.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Match by customer/bank reference string
    #[must_use]
    pub fn match_reference(
        statement_reference: &str,
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
        case_sensitive: bool,
    ) -> Vec<usize> {
        let mut matches = Vec::new();
        let search_ref = if case_sensitive {
            statement_reference.to_string()
        } else {
            statement_reference.to_lowercase()
        };

        for (i, (_amount, _txn_type, sys_ref, _date)) in system_transactions.iter().enumerate() {
            if let Some(reference) = sys_ref {
                let comp_ref = if case_sensitive {
                    reference.to_string()
                } else {
                    reference.to_lowercase()
                };
                if comp_ref.contains(&search_ref) || search_ref.contains(&comp_ref) {
                    matches.push(i);
                }
            }
        }
        matches
    }

    /// Match by date within a tolerance range
    #[must_use]
    pub fn match_date_range(
        statement_date: chrono::NaiveDate,
        tolerance_days: i32,
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
    ) -> Vec<usize> {
        let mut matches = Vec::new();
        for (i, (_amount, _txn_type, _ref, sys_date)) in system_transactions.iter().enumerate() {
            if let Some(date) = sys_date {
                let days_diff = (*date - statement_date).num_days().abs();
                if days_diff <= i64::from(tolerance_days) {
                    matches.push(i);
                }
            }
        }
        matches
    }

    /// Combined match: amount tolerance AND reference
    #[must_use]
    pub fn match_combined_amount_reference(
        statement_amount: f64,
        statement_reference: &str,
        tolerance_pct: f64,
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
    ) -> Vec<(usize, f64)> {
        let amount_matches =
            Self::match_amount_tolerance(statement_amount, tolerance_pct, system_transactions);
        let ref_matches = Self::match_reference(statement_reference, system_transactions, false);

        // Intersection: present in both sets
        amount_matches
            .into_iter()
            .filter(|(idx, _score)| ref_matches.contains(idx))
            .collect()
    }

    /// Combined match: amount tolerance AND date range
    #[must_use]
    pub fn match_combined_amount_date(
        statement_amount: f64,
        statement_date: chrono::NaiveDate,
        tolerance_pct: f64,
        tolerance_days: i32,
        system_transactions: &[(f64, &str, Option<&str>, Option<chrono::NaiveDate>)],
    ) -> Vec<(usize, f64)> {
        let amount_matches =
            Self::match_amount_tolerance(statement_amount, tolerance_pct, system_transactions);
        let date_matches =
            Self::match_date_range(statement_date, tolerance_days, system_transactions);

        amount_matches
            .into_iter()
            .filter(|(idx, _)| date_matches.contains(idx))
            .collect()
    }

    // ========================================================================
    // Reconciliation Run
    // ========================================================================

    /// Execute the full auto-reconciliation run for a set of statement lines
    /// against system transactions using the provided matching rules.
    ///
    /// Returns a `ReconciliationRunResult` summarising the outcome.
    #[must_use]
    pub fn run_auto_reconciliation(
        statement_lines: &[StatementLineInput],
        system_transactions: &[SystemTransactionInput],
        rules: &[MatchingRuleConfig],
    ) -> ReconciliationRunResult {
        let total_lines = statement_lines.len();
        let mut matched_count = 0usize;
        let mut partially_matched_count = 0usize;
        let mut unmatched_count = 0usize;
        let mut exception_count = 0usize;
        let mut line_results: Vec<LineMatchResult> = Vec::with_capacity(total_lines);

        // Track which system transactions have been consumed
        let mut consumed: Vec<bool> = vec![false; system_transactions.len()];

        // Sort rules by priority (ascending = highest priority first)
        let mut sorted_rules: Vec<&MatchingRuleConfig> = rules.iter().collect();
        sorted_rules.sort_by_key(|r| r.priority);

        for line in statement_lines {
            let available_txns: Vec<(f64, &str, Option<&str>, Option<chrono::NaiveDate>)> =
                system_transactions
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !consumed[*i])
                    .map(|(_, t)| {
                        (
                            t.amount,
                            t.txn_type.as_str(),
                            t.reference.as_deref(),
                            t.date,
                        )
                    })
                    .collect();

            // Build a mapping from filtered index → original index
            let index_map: Vec<usize> = system_transactions
                .iter()
                .enumerate()
                .filter(|(i, _)| !consumed[*i])
                .map(|(i, _)| i)
                .collect();

            let mut best_match: Option<(usize, f64, &str)> = None; // (original_idx, confidence, rule_name)

            for rule in &sorted_rules {
                if !rule.is_active {
                    continue;
                }

                let matches = match rule.match_strategy.as_str() {
                    "exact_amount" => Self::match_exact_amount(
                        line.amount,
                        &line.transaction_type,
                        &available_txns,
                    )
                    .map(|i| vec![(i, 0.0)])
                    .unwrap_or_default(),
                    "amount_tolerance" => {
                        let tolerance = rule.tolerance_pct.unwrap_or(1.0);
                        Self::match_amount_tolerance(line.amount, tolerance, &available_txns)
                    }
                    "reference_match" => {
                        if let Some(ref line_ref) = line.reference {
                            Self::match_reference(line_ref, &available_txns, false)
                                .into_iter()
                                .map(|i| (i, 10.0))
                                .collect()
                        } else {
                            vec![]
                        }
                    }
                    "date_range" => {
                        let days = rule.date_tolerance_days.unwrap_or(3);
                        Self::match_date_range(line.date, days, &available_txns)
                            .into_iter()
                            .map(|i| (i, 20.0))
                            .collect()
                    }
                    "combined_amount_reference" => {
                        let tolerance = rule.tolerance_pct.unwrap_or(1.0);
                        if let Some(ref line_ref) = line.reference {
                            Self::match_combined_amount_reference(
                                line.amount,
                                line_ref,
                                tolerance,
                                &available_txns,
                            )
                        } else {
                            vec![]
                        }
                    }
                    "combined_amount_date" => {
                        let tolerance = rule.tolerance_pct.unwrap_or(1.0);
                        let days = rule.date_tolerance_days.unwrap_or(3);
                        Self::match_combined_amount_date(
                            line.amount,
                            line.date,
                            tolerance,
                            days,
                            &available_txns,
                        )
                    }
                    _ => vec![],
                };

                if !matches.is_empty() {
                    // Take the best (first) match from this rule
                    let (filtered_idx, score) = matches[0];
                    let original_idx = index_map[filtered_idx];
                    let confidence = 100.0 - score; // invert: lower score = higher confidence

                    // Only accept if better than what we have
                    if best_match.is_none() || confidence > best_match.as_ref().unwrap().1 {
                        best_match = Some((original_idx, confidence, rule.rule_name.as_str()));
                    }

                    // If we have multiple matches from a single rule, flag as exception
                    if matches.len() > 1 {
                        // Multiple potential matches — mark as partial/exception
                        // but keep the best match
                        if confidence > 50.0 {
                            best_match =
                                Some((original_idx, confidence.min(70.0), rule.rule_name.as_str()));
                        }
                    }
                }
            }

            match best_match {
                Some((orig_idx, confidence, rule_name)) if confidence >= 70.0 => {
                    consumed[orig_idx] = true;
                    matched_count += 1;
                    line_results.push(LineMatchResult {
                        line_number: line.line_number,
                        status: "matched".to_string(),
                        matched_transaction_index: Some(orig_idx),
                        match_rule: Some(rule_name.to_string()),
                        confidence,
                    });
                }
                Some((orig_idx, confidence, rule_name)) if confidence >= 40.0 => {
                    // Partial match — needs review
                    partially_matched_count += 1;
                    exception_count += 1;
                    line_results.push(LineMatchResult {
                        line_number: line.line_number,
                        status: "partially_matched".to_string(),
                        matched_transaction_index: Some(orig_idx),
                        match_rule: Some(rule_name.to_string()),
                        confidence,
                    });
                }
                _ => {
                    // No match found
                    unmatched_count += 1;
                    exception_count += 1;
                    line_results.push(LineMatchResult {
                        line_number: line.line_number,
                        status: "unmatched".to_string(),
                        matched_transaction_index: None,
                        match_rule: None,
                        confidence: 0.0,
                    });
                }
            }
        }

        ReconciliationRunResult {
            total_lines,
            matched_count,
            partially_matched_count,
            unmatched_count,
            exception_count,
            line_results,
        }
    }

    // ========================================================================
    // Reconciliation Difference Calculation
    // ========================================================================

    /// Calculate the reconciliation difference between bank and book
    #[must_use]
    pub fn calculate_reconciliation_difference(
        bank_closing_balance: f64,
        book_balance: f64,
        deposits_in_transit: f64,
        outstanding_withdrawals: f64,
        bank_charges: f64,
        bank_interest: f64,
        errors_adjustments: f64,
    ) -> f64 {
        let adjusted_bank = bank_closing_balance;
        let adjusted_book =
            book_balance + deposits_in_transit - outstanding_withdrawals - bank_charges
                + bank_interest
                + errors_adjustments;
        adjusted_bank - adjusted_book
    }

    // ========================================================================
    // MT940 Parsing Helpers
    // ========================================================================

    /// Parse an MT940-style amount string (e.g., "100,00" or "100.00")
    /// MT940 uses comma as decimal separator in European formats
    pub fn parse_mt940_amount(raw: &str) -> AtlasResult<f64> {
        let cleaned = raw.trim().replace(',', ".");
        cleaned.parse::<f64>().map_err(|_| {
            atlas_shared::AtlasError::ValidationFailed(format!("Invalid MT940 amount: '{raw}'"))
        })
    }

    /// Extract the debit/credit indicator from MT940 field 61
    /// Returns "credit" or "debit"
    pub fn parse_mt940_credit_debit(indicator: char) -> AtlasResult<String> {
        match indicator {
            'C' | 'c' => Ok("credit".to_string()),
            'D' | 'd' => Ok("debit".to_string()),
            _ => Err(atlas_shared::AtlasError::ValidationFailed(format!(
                "Invalid MT940 credit/debit indicator: '{indicator}'"
            ))),
        }
    }

    /// Validate a BAI2 format record type code
    pub fn validate_bai2_record_type(code: &str) -> AtlasResult<()> {
        match code {
            "01" | "02" | "03" | "16" | "49" | "88" | "98" | "99" => Ok(()),
            _ => Err(atlas_shared::AtlasError::ValidationFailed(format!(
                "Invalid BAI2 record type: '{code}'"
            ))),
        }
    }
}

// ============================================================================
// Input / Output types for reconciliation run
// ============================================================================

/// Input representation of a bank statement line for matching
#[derive(Debug, Clone)]
pub struct StatementLineInput {
    pub line_number: i32,
    pub amount: f64,
    pub transaction_type: String,
    pub date: chrono::NaiveDate,
    pub reference: Option<String>,
    pub description: Option<String>,
}

/// Input representation of a system transaction (receipt, payment, journal)
#[derive(Debug, Clone)]
pub struct SystemTransactionInput {
    pub amount: f64,
    pub txn_type: String,
    pub reference: Option<String>,
    pub date: Option<chrono::NaiveDate>,
}

/// Configuration for a single matching rule
#[derive(Debug, Clone)]
pub struct MatchingRuleConfig {
    pub rule_name: String,
    pub priority: i32,
    pub match_strategy: String,
    pub is_active: bool,
    pub tolerance_pct: Option<f64>,
    pub date_tolerance_days: Option<i32>,
}

/// Result of a single line match attempt
#[derive(Debug, Clone)]
pub struct LineMatchResult {
    pub line_number: i32,
    pub status: String,
    pub matched_transaction_index: Option<usize>,
    pub match_rule: Option<String>,
    pub confidence: f64,
}

/// Aggregate result of an auto-reconciliation run
#[derive(Debug, Clone)]
pub struct ReconciliationRunResult {
    pub total_lines: usize,
    pub matched_count: usize,
    pub partially_matched_count: usize,
    pub unmatched_count: usize,
    pub exception_count: usize,
    pub line_results: Vec<LineMatchResult>,
}
