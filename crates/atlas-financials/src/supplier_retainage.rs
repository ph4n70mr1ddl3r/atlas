pub struct SupplierRetainageService;

pub struct RetainageResult {
    pub invoice_id: String,
    pub original_amount: f64,
    pub retainage_amount: f64,
    pub payable_amount: f64,
    pub status: String,
}

impl SupplierRetainageService {
    /// Calculates supplier retainage for an invoice.
    /// This is a common feature in Oracle Fusion Financials for construction and project-based procurement,
    /// where a portion of the payment is withheld until the project or milestone is successfully completed.
    #[must_use]
    pub fn calculate_retainage(
        invoice_id: &str,
        invoice_amount: f64,
        retainage_rate: f64,
    ) -> RetainageResult {
        if invoice_amount <= 0.0 || !(0.0..=1.0).contains(&retainage_rate) {
            return RetainageResult {
                invoice_id: invoice_id.to_string(),
                original_amount: invoice_amount,
                retainage_amount: 0.0,
                payable_amount: invoice_amount,
                status: "INVALID_PARAMETERS".to_string(),
            };
        }

        let retainage_amount = invoice_amount * retainage_rate;
        let payable_amount = invoice_amount - retainage_amount;

        RetainageResult {
            invoice_id: invoice_id.to_string(),
            original_amount: invoice_amount,
            retainage_amount,
            payable_amount,
            status: "CALCULATED".to_string(),
        }
    }

    /// Releases a previously withheld retainage.
    #[must_use]
    pub fn release_retainage(invoice_id: &str, retainage_amount: f64) -> RetainageResult {
        if retainage_amount <= 0.0 {
            return RetainageResult {
                invoice_id: invoice_id.to_string(),
                original_amount: 0.0,
                retainage_amount: 0.0,
                payable_amount: 0.0,
                status: "INVALID_RELEASE_AMOUNT".to_string(),
            };
        }

        RetainageResult {
            invoice_id: invoice_id.to_string(),
            original_amount: retainage_amount,
            retainage_amount: 0.0,
            payable_amount: retainage_amount, // The retained amount is now payable
            status: "RELEASED".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_retainage_valid() {
        let result = SupplierRetainageService::calculate_retainage("INV-001", 1000.0, 0.10);
        assert_eq!(result.invoice_id, "INV-001");
        assert_eq!(result.original_amount, 1000.0);
        assert_eq!(result.retainage_amount, 100.0);
        assert_eq!(result.payable_amount, 900.0);
        assert_eq!(result.status, "CALCULATED");
    }

    #[test]
    fn test_calculate_retainage_invalid_rate() {
        let result = SupplierRetainageService::calculate_retainage("INV-002", 1000.0, 1.5);
        assert_eq!(result.status, "INVALID_PARAMETERS");
        assert_eq!(result.retainage_amount, 0.0);
    }

    #[test]
    fn test_calculate_retainage_negative_amount() {
        let result = SupplierRetainageService::calculate_retainage("INV-003", -500.0, 0.1);
        assert_eq!(result.status, "INVALID_PARAMETERS");
        assert_eq!(result.retainage_amount, 0.0);
    }

    #[test]
    fn test_release_retainage_valid() {
        let result = SupplierRetainageService::release_retainage("INV-001", 100.0);
        assert_eq!(result.invoice_id, "INV-001");
        assert_eq!(result.original_amount, 100.0);
        assert_eq!(result.retainage_amount, 0.0);
        assert_eq!(result.payable_amount, 100.0);
        assert_eq!(result.status, "RELEASED");
    }

    #[test]
    fn test_release_retainage_invalid_amount() {
        let result = SupplierRetainageService::release_retainage("INV-001", 0.0);
        assert_eq!(result.status, "INVALID_RELEASE_AMOUNT");
        assert_eq!(result.payable_amount, 0.0);
    }
}
