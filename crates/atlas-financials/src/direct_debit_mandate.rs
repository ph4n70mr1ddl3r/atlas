pub struct DirectDebitMandateService;

pub struct MandateResult {
    pub mandate_id: String,
    pub customer_id: String,
    pub bank_account_id: String,
    pub amount_limit: f64,
    pub status: String,
}

impl DirectDebitMandateService {
    /// Creates a new Direct Debit Mandate.
    /// This is an Oracle Fusion Financials feature for Payments/Receivables,
    /// authorizing a creditor to collect funds directly from a debtor's bank account.
    #[must_use]
    pub fn create_mandate(
        customer_id: &str,
        bank_account_id: &str,
        amount_limit: f64,
    ) -> MandateResult {
        if amount_limit <= 0.0 {
            return MandateResult {
                mandate_id: "".to_string(),
                customer_id: customer_id.to_string(),
                bank_account_id: bank_account_id.to_string(),
                amount_limit: 0.0,
                status: "REJECTED".to_string(),
            };
        }

        MandateResult {
            mandate_id: format!("DDM-{}-{}", customer_id, bank_account_id),
            customer_id: customer_id.to_string(),
            bank_account_id: bank_account_id.to_string(),
            amount_limit,
            status: "PENDING_ACTIVATION".to_string(),
        }
    }

    /// Activates a Direct Debit Mandate.
    #[must_use]
    pub fn activate_mandate(mandate_id: &str) -> String {
        if mandate_id.is_empty() {
            "INVALID_MANDATE".to_string()
        } else {
            "ACTIVE".to_string()
        }
    }

    /// Revokes a Direct Debit Mandate.
    #[must_use]
    pub fn revoke_mandate(mandate_id: &str) -> String {
        if mandate_id.is_empty() {
            "INVALID_MANDATE".to_string()
        } else {
            "REVOKED".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mandate_valid() {
        let result = DirectDebitMandateService::create_mandate("CUST1", "BANK1", 5000.0);
        assert_eq!(result.customer_id, "CUST1");
        assert_eq!(result.bank_account_id, "BANK1");
        assert_eq!(result.amount_limit, 5000.0);
        assert_eq!(result.mandate_id, "DDM-CUST1-BANK1");
        assert_eq!(result.status, "PENDING_ACTIVATION");
    }

    #[test]
    fn test_create_mandate_invalid_amount() {
        let result = DirectDebitMandateService::create_mandate("CUST2", "BANK2", -50.0);
        assert_eq!(result.amount_limit, 0.0);
        assert_eq!(result.mandate_id, "");
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_activate_mandate_valid() {
        let result = DirectDebitMandateService::activate_mandate("DDM-CUST1-BANK1");
        assert_eq!(result, "ACTIVE");
    }

    #[test]
    fn test_activate_mandate_invalid() {
        let result = DirectDebitMandateService::activate_mandate("");
        assert_eq!(result, "INVALID_MANDATE");
    }

    #[test]
    fn test_revoke_mandate_valid() {
        let result = DirectDebitMandateService::revoke_mandate("DDM-CUST1-BANK1");
        assert_eq!(result, "REVOKED");
    }

    #[test]
    fn test_revoke_mandate_invalid() {
        let result = DirectDebitMandateService::revoke_mandate("");
        assert_eq!(result, "INVALID_MANDATE");
    }
}
