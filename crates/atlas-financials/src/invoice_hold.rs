pub struct InvoiceHoldService;

pub struct InvoiceHoldResult {
    pub hold_id: String,
    pub invoice_id: String,
    pub hold_reason_code: String,
    pub status: String,
}

pub struct HoldReleaseResult {
    pub hold_id: String,
    pub release_reason_code: String,
    pub status: String,
}

impl InvoiceHoldService {
    /// Places a hold on an Accounts Payable invoice.
    /// This is an Oracle Fusion Financials feature that prevents an invoice 
    /// from being selected for payment until the hold is released.
    #[must_use]
    pub fn place_hold(invoice_id: &str, hold_reason_code: &str) -> InvoiceHoldResult {
        if invoice_id.is_empty() {
            return InvoiceHoldResult {
                hold_id: "".to_string(),
                invoice_id: invoice_id.to_string(),
                hold_reason_code: hold_reason_code.to_string(),
                status: "REJECTED_INVALID_INVOICE".to_string(),
            };
        }

        if hold_reason_code.is_empty() {
            return InvoiceHoldResult {
                hold_id: "".to_string(),
                invoice_id: invoice_id.to_string(),
                hold_reason_code: hold_reason_code.to_string(),
                status: "REJECTED_MISSING_REASON".to_string(),
            };
        }

        InvoiceHoldResult {
            hold_id: format!("HLD-{}-{}", invoice_id, hold_reason_code),
            invoice_id: invoice_id.to_string(),
            hold_reason_code: hold_reason_code.to_string(),
            status: "PLACED".to_string(),
        }
    }

    /// Releases an existing hold on an invoice.
    #[must_use]
    pub fn release_hold(hold_id: &str, release_reason_code: &str) -> HoldReleaseResult {
        if hold_id.is_empty() {
            return HoldReleaseResult {
                hold_id: hold_id.to_string(),
                release_reason_code: release_reason_code.to_string(),
                status: "INVALID_HOLD_ID".to_string(),
            };
        }

        if release_reason_code.is_empty() {
            return HoldReleaseResult {
                hold_id: hold_id.to_string(),
                release_reason_code: release_reason_code.to_string(),
                status: "MISSING_RELEASE_REASON".to_string(),
            };
        }

        HoldReleaseResult {
            hold_id: hold_id.to_string(),
            release_reason_code: release_reason_code.to_string(),
            status: "RELEASED".to_string(),
        }
    }

    /// Checks if an invoice has any active holds that prevent payment.
    #[must_use]
    pub fn is_invoice_payable(invoice_id: &str, active_holds_count: u32) -> bool {
        if invoice_id.is_empty() {
            return false;
        }
        active_holds_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_hold_valid() {
        let result = InvoiceHoldService::place_hold("INV-500", "PRICE_DISCREPANCY");
        assert_eq!(result.invoice_id, "INV-500");
        assert_eq!(result.hold_reason_code, "PRICE_DISCREPANCY");
        assert_eq!(result.hold_id, "HLD-INV-500-PRICE_DISCREPANCY");
        assert_eq!(result.status, "PLACED");
    }

    #[test]
    fn test_place_hold_invalid_invoice() {
        let result = InvoiceHoldService::place_hold("", "QTY_RECEIVED");
        assert_eq!(result.hold_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_INVOICE");
    }

    #[test]
    fn test_place_hold_missing_reason() {
        let result = InvoiceHoldService::place_hold("INV-501", "");
        assert_eq!(result.hold_id, "");
        assert_eq!(result.status, "REJECTED_MISSING_REASON");
    }

    #[test]
    fn test_release_hold_valid() {
        let result = InvoiceHoldService::release_hold("HLD-INV-500-PRICE_DISCREPANCY", "DISCREPANCY_RESOLVED");
        assert_eq!(result.hold_id, "HLD-INV-500-PRICE_DISCREPANCY");
        assert_eq!(result.release_reason_code, "DISCREPANCY_RESOLVED");
        assert_eq!(result.status, "RELEASED");
    }

    #[test]
    fn test_release_hold_invalid_id() {
        let result = InvoiceHoldService::release_hold("", "RESOLVED");
        assert_eq!(result.status, "INVALID_HOLD_ID");
    }

    #[test]
    fn test_release_hold_missing_reason() {
        let result = InvoiceHoldService::release_hold("HLD-INV-502-TAX", "");
        assert_eq!(result.status, "MISSING_RELEASE_REASON");
    }

    #[test]
    fn test_is_invoice_payable_no_holds() {
        assert!(InvoiceHoldService::is_invoice_payable("INV-600", 0));
    }

    #[test]
    fn test_is_invoice_payable_with_holds() {
        assert!(!InvoiceHoldService::is_invoice_payable("INV-601", 1));
    }

    #[test]
    fn test_is_invoice_payable_invalid_invoice() {
        assert!(!InvoiceHoldService::is_invoice_payable("", 0));
    }
}
