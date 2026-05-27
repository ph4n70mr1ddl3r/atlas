pub struct ReceivableFactoringService;

pub struct FactoringResult {
    pub invoice_id: String,
    pub factor_id: String,
    pub original_amount: f64,
    pub discount_rate: f64,
    pub advance_amount: f64,
    pub fee_amount: f64,
    pub status: String,
}

impl ReceivableFactoringService {
    /// Processes receivable factoring for an invoice.
    /// This is an Oracle Fusion Financials feature for managing factored receivables.
    #[must_use]
    pub fn process_factoring(invoice_id: &str, factor_id: &str, amount: f64, discount_rate: f64) -> FactoringResult {
        if amount <= 0.0 || !(0.0..1.0).contains(&discount_rate) {
            return FactoringResult {
                invoice_id: invoice_id.to_string(),
                factor_id: factor_id.to_string(),
                original_amount: amount,
                discount_rate,
                advance_amount: 0.0,
                fee_amount: 0.0,
                status: "REJECTED".to_string(),
            };
        }
        
        let fee_amount = amount * discount_rate;
        let advance_amount = amount - fee_amount;

        FactoringResult {
            invoice_id: invoice_id.to_string(),
            factor_id: factor_id.to_string(),
            original_amount: amount,
            discount_rate,
            advance_amount,
            fee_amount,
            status: "PROCESSED".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_factoring_valid() {
        let result = ReceivableFactoringService::process_factoring("INV-1001", "FACTOR-A", 10000.0, 0.02);
        assert_eq!(result.invoice_id, "INV-1001");
        assert_eq!(result.factor_id, "FACTOR-A");
        assert_eq!(result.original_amount, 10000.0);
        assert_eq!(result.discount_rate, 0.02);
        assert_eq!(result.fee_amount, 200.0);
        assert_eq!(result.advance_amount, 9800.0);
        assert_eq!(result.status, "PROCESSED");
    }

    #[test]
    fn test_process_factoring_invalid_amount() {
        let result = ReceivableFactoringService::process_factoring("INV-1002", "FACTOR-B", -500.0, 0.05);
        assert_eq!(result.status, "REJECTED");
        assert_eq!(result.advance_amount, 0.0);
    }

    #[test]
    fn test_process_factoring_invalid_rate() {
        let result = ReceivableFactoringService::process_factoring("INV-1003", "FACTOR-C", 5000.0, 1.5);
        assert_eq!(result.status, "REJECTED");
        assert_eq!(result.advance_amount, 0.0);
    }
}
