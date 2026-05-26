pub struct JournalImportService;

pub struct JournalImportResult {
    pub import_request_id: String,
    pub source: String,
    pub ledger_id: String,
    pub total_lines: u32,
    pub total_debits: f64,
    pub total_credits: f64,
    pub status: String,
}

impl JournalImportService {
    /// Imports journal lines into the General Ledger.
    /// This is an Oracle Fusion General Ledger feature for transferring data 
    /// from subledgers or external feeder systems into the GL.
    #[must_use]
    pub fn import_journals(
        source: &str,
        ledger_id: &str,
        lines: &[(f64, f64)], // (debit, credit) pairs
    ) -> JournalImportResult {
        if source.is_empty() || ledger_id.is_empty() {
            return JournalImportResult {
                import_request_id: "".to_string(),
                source: source.to_string(),
                ledger_id: ledger_id.to_string(),
                total_lines: 0,
                total_debits: 0.0,
                total_credits: 0.0,
                status: "REJECTED_INVALID_PARAMETERS".to_string(),
            };
        }

        if lines.is_empty() {
            return JournalImportResult {
                import_request_id: "".to_string(),
                source: source.to_string(),
                ledger_id: ledger_id.to_string(),
                total_lines: 0,
                total_debits: 0.0,
                total_credits: 0.0,
                status: "REJECTED_NO_LINES".to_string(),
            };
        }

        let mut total_debits = 0.0;
        let mut total_credits = 0.0;

        for &(dr, cr) in lines {
            if dr < 0.0 || cr < 0.0 {
                return JournalImportResult {
                    import_request_id: "".to_string(),
                    source: source.to_string(),
                    ledger_id: ledger_id.to_string(),
                    total_lines: lines.len() as u32,
                    total_debits: 0.0,
                    total_credits: 0.0,
                    status: "REJECTED_NEGATIVE_AMOUNTS".to_string(),
                };
            }
            total_debits += dr;
            total_credits += cr;
        }

        // Check if journal is balanced
        let diff = (total_debits - total_credits).abs();
        let status = if diff < 0.01 {
            "IMPORTED_BALANCED".to_string()
        } else {
            "IMPORTED_UNBALANCED".to_string()
        };

        JournalImportResult {
            import_request_id: format!("REQ-{}-{}", source, ledger_id),
            source: source.to_string(),
            ledger_id: ledger_id.to_string(),
            total_lines: lines.len() as u32,
            total_debits,
            total_credits,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_journals_balanced() {
        let lines = vec![
            (1500.0, 0.0),
            (0.0, 1000.0),
            (0.0, 500.0),
        ];
        let result = JournalImportService::import_journals("PAYABLES", "LEDGER-US", &lines);
        assert_eq!(result.source, "PAYABLES");
        assert_eq!(result.ledger_id, "LEDGER-US");
        assert_eq!(result.total_lines, 3);
        assert_eq!(result.total_debits, 1500.0);
        assert_eq!(result.total_credits, 1500.0);
        assert_eq!(result.import_request_id, "REQ-PAYABLES-LEDGER-US");
        assert_eq!(result.status, "IMPORTED_BALANCED");
    }

    #[test]
    fn test_import_journals_unbalanced() {
        let lines = vec![
            (1500.0, 0.0),
            (0.0, 1000.0),
        ];
        let result = JournalImportService::import_journals("RECEIVABLES", "LEDGER-UK", &lines);
        assert_eq!(result.total_debits, 1500.0);
        assert_eq!(result.total_credits, 1000.0);
        assert_eq!(result.status, "IMPORTED_UNBALANCED");
    }

    #[test]
    fn test_import_journals_negative_amount() {
        let lines = vec![
            (-500.0, 0.0),
            (0.0, -500.0),
        ];
        let result = JournalImportService::import_journals("PAYABLES", "LEDGER-US", &lines);
        assert_eq!(result.status, "REJECTED_NEGATIVE_AMOUNTS");
    }

    #[test]
    fn test_import_journals_no_lines() {
        let lines: Vec<(f64, f64)> = vec![];
        let result = JournalImportService::import_journals("PAYABLES", "LEDGER-US", &lines);
        assert_eq!(result.status, "REJECTED_NO_LINES");
    }

    #[test]
    fn test_import_journals_invalid_parameters() {
        let lines = vec![(100.0, 100.0)];
        let result = JournalImportService::import_journals("", "LEDGER-US", &lines);
        assert_eq!(result.status, "REJECTED_INVALID_PARAMETERS");
    }
}
