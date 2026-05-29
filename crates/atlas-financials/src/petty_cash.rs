//! Oracle Fusion Financial Feature: Petty Cash Management
//! Manages petty cash funds, custodians, and disbursements.

pub struct PettyCashService;

#[derive(Debug, PartialEq)]
pub struct PettyCashFund {
    pub fund_name: String,
    pub custodian_id: String,
    pub authorized_amount: f64,
    pub current_balance: f64,
}

#[derive(Debug, PartialEq)]
pub struct PettyCashDisbursementResult {
    pub fund_name: String,
    pub amount: f64,
    pub remaining_balance: f64,
    pub status: String,
    pub receipt_reference: Option<String>,
}

impl PettyCashService {
    /// Creates a new petty cash fund.
    #[must_use]
    pub fn create_fund(fund_name: &str, custodian_id: &str, authorized_amount: f64) -> PettyCashFund {
        PettyCashFund {
            fund_name: fund_name.to_string(),
            custodian_id: custodian_id.to_string(),
            authorized_amount,
            current_balance: authorized_amount, // initially funded
        }
    }

    /// Disburses cash from a petty cash fund if sufficient balance exists.
    #[must_use]
    pub fn disburse(fund: &PettyCashFund, amount: f64, receipt_reference: Option<String>) -> PettyCashDisbursementResult {
        if amount <= 0.0 {
            return PettyCashDisbursementResult {
                fund_name: fund.fund_name.clone(),
                amount: 0.0,
                remaining_balance: fund.current_balance,
                status: "REJECTED_INVALID_AMOUNT".to_string(),
                receipt_reference: None,
            };
        }

        if amount > fund.current_balance {
            return PettyCashDisbursementResult {
                fund_name: fund.fund_name.clone(),
                amount,
                remaining_balance: fund.current_balance,
                status: "REJECTED_INSUFFICIENT_FUNDS".to_string(),
                receipt_reference: None,
            };
        }

        PettyCashDisbursementResult {
            fund_name: fund.fund_name.clone(),
            amount,
            remaining_balance: fund.current_balance - amount,
            status: "CLEARED".to_string(),
            receipt_reference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fund() {
        let fund = PettyCashService::create_fund("HQ_CASH_01", "EMP-100", 500.0);
        assert_eq!(fund.fund_name, "HQ_CASH_01");
        assert_eq!(fund.custodian_id, "EMP-100");
        assert_eq!(fund.authorized_amount, 500.0);
        assert_eq!(fund.current_balance, 500.0);
    }

    #[test]
    fn test_disburse_success() {
        let fund = PettyCashService::create_fund("HQ_CASH_01", "EMP-100", 500.0);
        let result = PettyCashService::disburse(&fund, 150.0, Some("RCPT-001".to_string()));
        
        assert_eq!(result.status, "CLEARED");
        assert_eq!(result.amount, 150.0);
        assert_eq!(result.remaining_balance, 350.0);
        assert_eq!(result.receipt_reference, Some("RCPT-001".to_string()));
    }

    #[test]
    fn test_disburse_insufficient_funds() {
        let fund = PettyCashService::create_fund("HQ_CASH_01", "EMP-100", 100.0);
        let result = PettyCashService::disburse(&fund, 150.0, None);
        
        assert_eq!(result.status, "REJECTED_INSUFFICIENT_FUNDS");
        assert_eq!(result.remaining_balance, 100.0);
    }
    
    #[test]
    fn test_disburse_invalid_amount() {
        let fund = PettyCashService::create_fund("HQ_CASH_01", "EMP-100", 100.0);
        let result = PettyCashService::disburse(&fund, -50.0, None);
        
        assert_eq!(result.status, "REJECTED_INVALID_AMOUNT");
        assert_eq!(result.remaining_balance, 100.0);
    }
}