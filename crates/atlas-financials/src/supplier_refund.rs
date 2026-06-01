pub struct SupplierRefundService;

pub struct SupplierRefundResult {
    pub refund_id: String,
    pub supplier_id: String,
    pub refund_amount: f64,
    pub bank_account_id: String,
    pub status: String,
}

impl SupplierRefundService {
    /// Processes a refund received from a supplier.
    /// This is an Oracle Fusion Financials feature for Accounts Payable that handles
    /// refunds from suppliers for credit balances, overpayments, or returned goods.
    #[must_use]
    pub fn process_refund(
        supplier_id: &str,
        refund_amount: f64,
        bank_account_id: &str,
    ) -> SupplierRefundResult {
        if refund_amount <= 0.0 {
            return SupplierRefundResult {
                refund_id: "".to_string(),
                supplier_id: supplier_id.to_string(),
                refund_amount: 0.0,
                bank_account_id: bank_account_id.to_string(),
                status: "REJECTED_INVALID_AMOUNT".to_string(),
            };
        }

        if bank_account_id.is_empty() {
            return SupplierRefundResult {
                refund_id: "".to_string(),
                supplier_id: supplier_id.to_string(),
                refund_amount,
                bank_account_id: bank_account_id.to_string(),
                status: "REJECTED_INVALID_BANK".to_string(),
            };
        }

        SupplierRefundResult {
            refund_id: format!("SR-{}-{}", supplier_id, refund_amount),
            supplier_id: supplier_id.to_string(),
            refund_amount,
            bank_account_id: bank_account_id.to_string(),
            status: "PROCESSED".to_string(),
        }
    }

    /// Voids a previously processed supplier refund.
    #[must_use]
    pub fn void_refund(refund_id: &str) -> String {
        if refund_id.is_empty() {
            "INVALID_REFUND_ID".to_string()
        } else {
            "VOIDED".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_refund_valid() {
        let result = SupplierRefundService::process_refund("SUPP-123", 2500.0, "BANK-789");
        assert_eq!(result.supplier_id, "SUPP-123");
        assert_eq!(result.refund_amount, 2500.0);
        assert_eq!(result.bank_account_id, "BANK-789");
        assert_eq!(result.refund_id, "SR-SUPP-123-2500");
        assert_eq!(result.status, "PROCESSED");
    }

    #[test]
    fn test_process_refund_invalid_amount() {
        let result = SupplierRefundService::process_refund("SUPP-124", -100.0, "BANK-789");
        assert_eq!(result.refund_amount, 0.0);
        assert_eq!(result.refund_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_AMOUNT");
    }

    #[test]
    fn test_process_refund_invalid_bank() {
        let result = SupplierRefundService::process_refund("SUPP-125", 500.0, "");
        assert_eq!(result.refund_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_BANK");
    }

    #[test]
    fn test_void_refund_valid() {
        let result = SupplierRefundService::void_refund("SR-SUPP-123-2500");
        assert_eq!(result, "VOIDED");
    }

    #[test]
    fn test_void_refund_invalid() {
        let result = SupplierRefundService::void_refund("");
        assert_eq!(result, "INVALID_REFUND_ID");
    }
}
