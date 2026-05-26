pub struct CustomerDisputeService;

pub struct CustomerDisputeResult {
    pub dispute_id: String,
    pub invoice_id: String,
    pub customer_id: String,
    pub dispute_amount: f64,
    pub reason_code: String,
    pub status: String,
}

impl CustomerDisputeService {
    /// Creates a new customer dispute for an invoice.
    /// This is an Oracle Fusion Receivables/Advanced Collections feature that allows
    /// tracking when a customer disputes a transaction balance.
    #[must_use]
    pub fn create_dispute(
        customer_id: &str,
        invoice_id: &str,
        dispute_amount: f64,
        invoice_balance: f64,
        reason_code: &str,
    ) -> CustomerDisputeResult {
        if dispute_amount <= 0.0 {
            return CustomerDisputeResult {
                dispute_id: "".to_string(),
                invoice_id: invoice_id.to_string(),
                customer_id: customer_id.to_string(),
                dispute_amount: 0.0,
                reason_code: reason_code.to_string(),
                status: "REJECTED_INVALID_AMOUNT".to_string(),
            };
        }

        if dispute_amount > invoice_balance {
            return CustomerDisputeResult {
                dispute_id: "".to_string(),
                invoice_id: invoice_id.to_string(),
                customer_id: customer_id.to_string(),
                dispute_amount,
                reason_code: reason_code.to_string(),
                status: "REJECTED_EXCEEDS_BALANCE".to_string(),
            };
        }

        if reason_code.is_empty() {
            return CustomerDisputeResult {
                dispute_id: "".to_string(),
                invoice_id: invoice_id.to_string(),
                customer_id: customer_id.to_string(),
                dispute_amount,
                reason_code: reason_code.to_string(),
                status: "REJECTED_MISSING_REASON".to_string(),
            };
        }
        
        CustomerDisputeResult {
            dispute_id: format!("DISP-{}-{}", customer_id, invoice_id),
            invoice_id: invoice_id.to_string(),
            customer_id: customer_id.to_string(),
            dispute_amount,
            reason_code: reason_code.to_string(),
            status: "SUBMITTED".to_string(),
        }
    }

    /// Resolves an open customer dispute.
    #[must_use]
    pub fn resolve_dispute(dispute_id: &str, resolution_code: &str) -> String {
        if dispute_id.is_empty() {
            "INVALID_DISPUTE_ID".to_string()
        } else if resolution_code.is_empty() {
            "MISSING_RESOLUTION_CODE".to_string()
        } else {
            "RESOLVED".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dispute_valid() {
        let result = CustomerDisputeService::create_dispute("CUST-001", "INV-100", 500.0, 1000.0, "PRICING_ERROR");
        assert_eq!(result.customer_id, "CUST-001");
        assert_eq!(result.invoice_id, "INV-100");
        assert_eq!(result.dispute_amount, 500.0);
        assert_eq!(result.reason_code, "PRICING_ERROR");
        assert_eq!(result.dispute_id, "DISP-CUST-001-INV-100");
        assert_eq!(result.status, "SUBMITTED");
    }

    #[test]
    fn test_create_dispute_invalid_amount() {
        let result = CustomerDisputeService::create_dispute("CUST-002", "INV-101", -50.0, 1000.0, "MISSING_GOODS");
        assert_eq!(result.dispute_amount, 0.0);
        assert_eq!(result.dispute_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_AMOUNT");
    }

    #[test]
    fn test_create_dispute_exceeds_balance() {
        let result = CustomerDisputeService::create_dispute("CUST-003", "INV-102", 1500.0, 1000.0, "MISSING_GOODS");
        assert_eq!(result.dispute_id, "");
        assert_eq!(result.status, "REJECTED_EXCEEDS_BALANCE");
    }

    #[test]
    fn test_create_dispute_missing_reason() {
        let result = CustomerDisputeService::create_dispute("CUST-004", "INV-103", 500.0, 1000.0, "");
        assert_eq!(result.dispute_id, "");
        assert_eq!(result.status, "REJECTED_MISSING_REASON");
    }

    #[test]
    fn test_resolve_dispute_valid() {
        let result = CustomerDisputeService::resolve_dispute("DISP-CUST-001-INV-100", "CREDIT_MEMO_ISSUED");
        assert_eq!(result, "RESOLVED");
    }

    #[test]
    fn test_resolve_dispute_invalid_id() {
        let result = CustomerDisputeService::resolve_dispute("", "CREDIT_MEMO_ISSUED");
        assert_eq!(result, "INVALID_DISPUTE_ID");
    }

    #[test]
    fn test_resolve_dispute_missing_code() {
        let result = CustomerDisputeService::resolve_dispute("DISP-CUST-001-INV-100", "");
        assert_eq!(result, "MISSING_RESOLUTION_CODE");
    }
}
