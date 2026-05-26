pub struct LetterOfCreditService;

pub struct LetterOfCreditResult {
    pub lc_number: String,
    pub applicant_id: String,
    pub beneficiary_id: String,
    pub amount: f64,
    pub status: String,
}

impl LetterOfCreditService {
    /// Issues a new Letter of Credit (LC).
    /// This is an Oracle Fusion Financials trade finance feature providing a guarantee 
    /// from a bank that a buyer's payment to a seller will be received on time and for the correct amount.
    #[must_use]
    pub fn issue_lc(
        applicant_id: &str,
        beneficiary_id: &str,
        amount: f64,
        days_valid: u32,
    ) -> LetterOfCreditResult {
        let status = if amount > 0.0 && days_valid > 0 {
            "ISSUED".to_string()
        } else {
            "REJECTED".to_string()
        };
        
        let lc_number = if status == "ISSUED" {
            format!("LC-{}-{}", applicant_id, beneficiary_id)
        } else {
            "".to_string()
        };
        
        LetterOfCreditResult {
            lc_number,
            applicant_id: applicant_id.to_string(),
            beneficiary_id: beneficiary_id.to_string(),
            amount: if status == "ISSUED" { amount } else { 0.0 },
            status,
        }
    }

    /// Amends an existing Letter of Credit to a new amount.
    #[must_use]
    pub fn amend_lc(lc_number: &str, new_amount: f64) -> LetterOfCreditResult {
        if lc_number.is_empty() || new_amount <= 0.0 {
            LetterOfCreditResult {
                lc_number: lc_number.to_string(),
                applicant_id: "".to_string(),
                beneficiary_id: "".to_string(),
                amount: 0.0,
                status: "AMENDMENT_REJECTED".to_string(),
            }
        } else {
            LetterOfCreditResult {
                lc_number: lc_number.to_string(),
                applicant_id: "UNKNOWN".to_string(), // In a real system, we'd fetch the LC details
                beneficiary_id: "UNKNOWN".to_string(),
                amount: new_amount,
                status: "AMENDED".to_string(),
            }
        }
    }
    
    /// Cancels an existing Letter of Credit.
    #[must_use]
    pub fn cancel_lc(lc_number: &str) -> String {
        if lc_number.is_empty() {
            "INVALID_LC".to_string()
        } else {
            "CANCELLED".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_lc_valid() {
        let result = LetterOfCreditService::issue_lc("BUYER1", "SELLER1", 50000.0, 90);
        assert_eq!(result.applicant_id, "BUYER1");
        assert_eq!(result.beneficiary_id, "SELLER1");
        assert_eq!(result.amount, 50000.0);
        assert_eq!(result.lc_number, "LC-BUYER1-SELLER1");
        assert_eq!(result.status, "ISSUED");
    }

    #[test]
    fn test_issue_lc_invalid_amount() {
        let result = LetterOfCreditService::issue_lc("BUYER2", "SELLER2", -100.0, 30);
        assert_eq!(result.amount, 0.0);
        assert_eq!(result.lc_number, "");
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_amend_lc_valid() {
        let result = LetterOfCreditService::amend_lc("LC-BUYER1-SELLER1", 60000.0);
        assert_eq!(result.lc_number, "LC-BUYER1-SELLER1");
        assert_eq!(result.amount, 60000.0);
        assert_eq!(result.status, "AMENDED");
    }

    #[test]
    fn test_amend_lc_invalid() {
        let result = LetterOfCreditService::amend_lc("LC-BUYER1-SELLER1", 0.0);
        assert_eq!(result.status, "AMENDMENT_REJECTED");
    }

    #[test]
    fn test_cancel_lc_valid() {
        let result = LetterOfCreditService::cancel_lc("LC-BUYER1-SELLER1");
        assert_eq!(result, "CANCELLED");
    }

    #[test]
    fn test_cancel_lc_invalid() {
        let result = LetterOfCreditService::cancel_lc("");
        assert_eq!(result, "INVALID_LC");
    }
}
