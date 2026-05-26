pub struct ThirdPartyPaymentService;

pub struct ThirdPartyPaymentResult {
    pub payment_id: String,
    pub original_payee_id: String,
    pub third_party_payee_id: String,
    pub amount: f64,
    pub status: String,
}

impl ThirdPartyPaymentService {
    /// Processes a third-party payment.
    /// This is an Oracle Fusion Financials feature that allows payments to be 
    /// routed to a third-party payee on behalf of the original supplier or employee.
    #[must_use]
    pub fn process_payment(
        original_payee_id: &str,
        third_party_payee_id: &str,
        amount: f64,
        is_relationship_active: bool,
    ) -> ThirdPartyPaymentResult {
        if amount <= 0.0 {
            return ThirdPartyPaymentResult {
                payment_id: "".to_string(),
                original_payee_id: original_payee_id.to_string(),
                third_party_payee_id: third_party_payee_id.to_string(),
                amount: 0.0,
                status: "REJECTED_INVALID_AMOUNT".to_string(),
            };
        }

        if !is_relationship_active {
            return ThirdPartyPaymentResult {
                payment_id: "".to_string(),
                original_payee_id: original_payee_id.to_string(),
                third_party_payee_id: third_party_payee_id.to_string(),
                amount,
                status: "REJECTED_INACTIVE_RELATIONSHIP".to_string(),
            };
        }
        
        ThirdPartyPaymentResult {
            payment_id: format!("TPP-{}-{}", original_payee_id, third_party_payee_id),
            original_payee_id: original_payee_id.to_string(),
            third_party_payee_id: third_party_payee_id.to_string(),
            amount,
            status: "PROCESSED".to_string(),
        }
    }

    /// Validates a third-party payment relationship.
    #[must_use]
    pub fn validate_relationship(original_payee_id: &str, third_party_payee_id: &str) -> bool {
        if original_payee_id.is_empty() || third_party_payee_id.is_empty() {
            return false;
        }
        // In a real system, this would query a database to check the relationship validity
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_payment_valid() {
        let result = ThirdPartyPaymentService::process_payment("SUPP-001", "SUPP-002-FACTOR", 10000.0, true);
        assert_eq!(result.original_payee_id, "SUPP-001");
        assert_eq!(result.third_party_payee_id, "SUPP-002-FACTOR");
        assert_eq!(result.amount, 10000.0);
        assert_eq!(result.payment_id, "TPP-SUPP-001-SUPP-002-FACTOR");
        assert_eq!(result.status, "PROCESSED");
    }

    #[test]
    fn test_process_payment_invalid_amount() {
        let result = ThirdPartyPaymentService::process_payment("SUPP-003", "SUPP-004", -500.0, true);
        assert_eq!(result.amount, 0.0);
        assert_eq!(result.payment_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_AMOUNT");
    }

    #[test]
    fn test_process_payment_inactive_relationship() {
        let result = ThirdPartyPaymentService::process_payment("SUPP-005", "SUPP-006", 5000.0, false);
        assert_eq!(result.amount, 5000.0);
        assert_eq!(result.payment_id, "");
        assert_eq!(result.status, "REJECTED_INACTIVE_RELATIONSHIP");
    }

    #[test]
    fn test_validate_relationship_valid() {
        let is_valid = ThirdPartyPaymentService::validate_relationship("EMP-100", "GARNISHMENT-ORG");
        assert!(is_valid);
    }

    #[test]
    fn test_validate_relationship_invalid() {
        let is_valid = ThirdPartyPaymentService::validate_relationship("", "GARNISHMENT-ORG");
        assert!(!is_valid);
    }
}
