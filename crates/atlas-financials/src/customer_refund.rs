pub struct CustomerRefundService;

pub struct RefundResult {
    pub customer_id: String,
    pub amount_refunded: f64,
    pub ap_invoice_id: String,
    pub status: String,
}

impl CustomerRefundService {
    /// Processes a customer refund by creating an AP invoice for the credit balance.
    /// This is an Oracle Fusion Financials feature for AR to AP refunds.
    #[must_use]
    pub fn process_refund(customer_id: &str, credit_balance: f64) -> RefundResult {
        let status = if credit_balance > 0.0 {
            "PROCESSED".to_string()
        } else {
            "REJECTED".to_string()
        };
        
        RefundResult {
            customer_id: customer_id.to_string(),
            amount_refunded: if credit_balance > 0.0 { credit_balance } else { 0.0 },
            ap_invoice_id: if credit_balance > 0.0 { format!("REFUND-AP-{}", customer_id) } else { "".to_string() },
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_refund_valid() {
        let result = CustomerRefundService::process_refund("CUST-123", 500.0);
        assert_eq!(result.customer_id, "CUST-123");
        assert_eq!(result.amount_refunded, 500.0);
        assert_eq!(result.ap_invoice_id, "REFUND-AP-CUST-123");
        assert_eq!(result.status, "PROCESSED");
    }

    #[test]
    fn test_process_refund_zero_balance() {
        let result = CustomerRefundService::process_refund("CUST-456", 0.0);
        assert_eq!(result.customer_id, "CUST-456");
        assert_eq!(result.amount_refunded, 0.0);
        assert_eq!(result.ap_invoice_id, "");
        assert_eq!(result.status, "REJECTED");
    }
}
