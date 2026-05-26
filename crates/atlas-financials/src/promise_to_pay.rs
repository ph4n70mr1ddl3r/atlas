pub struct PromiseToPayService;

pub struct PromiseToPayResult {
    pub ptp_id: String,
    pub customer_id: String,
    pub invoice_id: String,
    pub promise_amount: f64,
    pub promise_date: String,
    pub status: String,
}

pub struct PtpFulfillmentResult {
    pub ptp_id: String,
    pub fulfilled_amount: f64,
    pub remaining_amount: f64,
    pub status: String,
}

impl PromiseToPayService {
    /// Creates a Promise to Pay for a delinquent invoice.
    /// This is an Oracle Fusion Advanced Collections feature where a customer
    /// commits to paying a specific amount on a specific date.
    #[must_use]
    pub fn create_promise(
        customer_id: &str,
        invoice_id: &str,
        promise_amount: f64,
        days_from_now: u32,
    ) -> PromiseToPayResult {
        if promise_amount <= 0.0 {
            return PromiseToPayResult {
                ptp_id: "".to_string(),
                customer_id: customer_id.to_string(),
                invoice_id: invoice_id.to_string(),
                promise_amount: 0.0,
                promise_date: "".to_string(),
                status: "REJECTED".to_string(),
            };
        }
        
        let promise_date = format!("T+{} days", days_from_now);
        
        PromiseToPayResult {
            ptp_id: format!("PTP-{}-{}", customer_id, invoice_id),
            customer_id: customer_id.to_string(),
            invoice_id: invoice_id.to_string(),
            promise_amount,
            promise_date,
            status: "OPEN".to_string(),
        }
    }

    /// Records a payment against an open Promise to Pay.
    #[must_use]
    pub fn fulfill_promise(
        ptp_id: &str,
        promise_amount: f64,
        payment_amount: f64,
    ) -> PtpFulfillmentResult {
        if ptp_id.is_empty() || payment_amount <= 0.0 {
            return PtpFulfillmentResult {
                ptp_id: ptp_id.to_string(),
                fulfilled_amount: 0.0,
                remaining_amount: promise_amount,
                status: "INVALID_PAYMENT".to_string(),
            };
        }

        let remaining = if payment_amount >= promise_amount {
            0.0
        } else {
            promise_amount - payment_amount
        };

        let status = if remaining == 0.0 {
            "FULFILLED".to_string()
        } else {
            "PARTIALLY_FULFILLED".to_string()
        };

        PtpFulfillmentResult {
            ptp_id: ptp_id.to_string(),
            fulfilled_amount: payment_amount,
            remaining_amount: remaining,
            status,
        }
    }

    /// Marks a Promise to Pay as broken.
    #[must_use]
    pub fn break_promise(ptp_id: &str) -> String {
        if ptp_id.is_empty() {
            "INVALID_PTP".to_string()
        } else {
            "BROKEN".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_promise_valid() {
        let result = PromiseToPayService::create_promise("CUST123", "INV-999", 1500.0, 15);
        assert_eq!(result.customer_id, "CUST123");
        assert_eq!(result.invoice_id, "INV-999");
        assert_eq!(result.promise_amount, 1500.0);
        assert_eq!(result.ptp_id, "PTP-CUST123-INV-999");
        assert_eq!(result.promise_date, "T+15 days");
        assert_eq!(result.status, "OPEN");
    }

    #[test]
    fn test_create_promise_invalid_amount() {
        let result = PromiseToPayService::create_promise("CUST124", "INV-998", -50.0, 7);
        assert_eq!(result.promise_amount, 0.0);
        assert_eq!(result.ptp_id, "");
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_fulfill_promise_full() {
        let result = PromiseToPayService::fulfill_promise("PTP-CUST123-INV-999", 1500.0, 1500.0);
        assert_eq!(result.status, "FULFILLED");
        assert_eq!(result.remaining_amount, 0.0);
        assert_eq!(result.fulfilled_amount, 1500.0);
    }

    #[test]
    fn test_fulfill_promise_partial() {
        let result = PromiseToPayService::fulfill_promise("PTP-CUST123-INV-999", 1500.0, 500.0);
        assert_eq!(result.status, "PARTIALLY_FULFILLED");
        assert_eq!(result.remaining_amount, 1000.0);
        assert_eq!(result.fulfilled_amount, 500.0);
    }

    #[test]
    fn test_fulfill_promise_invalid() {
        let result = PromiseToPayService::fulfill_promise("", 1500.0, 500.0);
        assert_eq!(result.status, "INVALID_PAYMENT");
    }

    #[test]
    fn test_break_promise_valid() {
        let result = PromiseToPayService::break_promise("PTP-CUST123-INV-999");
        assert_eq!(result, "BROKEN");
    }

    #[test]
    fn test_break_promise_invalid() {
        let result = PromiseToPayService::break_promise("");
        assert_eq!(result, "INVALID_PTP");
    }
}
