pub struct CreditMemoRequestService;

pub struct CreditMemoRequestResult {
    pub request_id: String,
    pub transaction_id: String,
    pub requested_amount: f64,
    pub reason_code: String,
    pub status: String,
}

pub struct ApprovalResult {
    pub request_id: String,
    pub approver_id: String,
    pub status: String,
}

impl CreditMemoRequestService {
    /// Submits a new Credit Memo Request.
    /// This is an Oracle Fusion Receivables feature that allows users to request
    /// a credit memo against a specific transaction, often routing for approval.
    #[must_use]
    pub fn submit_request(
        transaction_id: &str,
        requested_amount: f64,
        transaction_balance: f64,
        reason_code: &str,
    ) -> CreditMemoRequestResult {
        if requested_amount <= 0.0 {
            return CreditMemoRequestResult {
                request_id: "".to_string(),
                transaction_id: transaction_id.to_string(),
                requested_amount: 0.0,
                reason_code: reason_code.to_string(),
                status: "REJECTED_INVALID_AMOUNT".to_string(),
            };
        }

        if requested_amount > transaction_balance {
            return CreditMemoRequestResult {
                request_id: "".to_string(),
                transaction_id: transaction_id.to_string(),
                requested_amount,
                reason_code: reason_code.to_string(),
                status: "REJECTED_EXCEEDS_BALANCE".to_string(),
            };
        }

        if reason_code.is_empty() {
            return CreditMemoRequestResult {
                request_id: "".to_string(),
                transaction_id: transaction_id.to_string(),
                requested_amount,
                reason_code: reason_code.to_string(),
                status: "REJECTED_MISSING_REASON".to_string(),
            };
        }

        CreditMemoRequestResult {
            request_id: format!("CMR-{}-{}", transaction_id, requested_amount),
            transaction_id: transaction_id.to_string(),
            requested_amount,
            reason_code: reason_code.to_string(),
            status: "PENDING_APPROVAL".to_string(),
        }
    }

    /// Approves or Rejects a pending Credit Memo Request.
    #[must_use]
    pub fn process_approval(
        request_id: &str,
        approver_id: &str,
        is_approved: bool,
    ) -> ApprovalResult {
        if request_id.is_empty() {
            return ApprovalResult {
                request_id: "".to_string(),
                approver_id: approver_id.to_string(),
                status: "INVALID_REQUEST".to_string(),
            };
        }

        if approver_id.is_empty() {
            return ApprovalResult {
                request_id: request_id.to_string(),
                approver_id: "".to_string(),
                status: "MISSING_APPROVER".to_string(),
            };
        }

        ApprovalResult {
            request_id: request_id.to_string(),
            approver_id: approver_id.to_string(),
            status: if is_approved {
                "APPROVED".to_string()
            } else {
                "REJECTED".to_string()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_request_valid() {
        let result =
            CreditMemoRequestService::submit_request("TRX-100", 250.0, 1000.0, "RETURNED_GOODS");
        assert_eq!(result.transaction_id, "TRX-100");
        assert_eq!(result.requested_amount, 250.0);
        assert_eq!(result.reason_code, "RETURNED_GOODS");
        assert_eq!(result.request_id, "CMR-TRX-100-250");
        assert_eq!(result.status, "PENDING_APPROVAL");
    }

    #[test]
    fn test_submit_request_invalid_amount() {
        let result =
            CreditMemoRequestService::submit_request("TRX-101", -50.0, 1000.0, "PRICING_ERROR");
        assert_eq!(result.requested_amount, 0.0);
        assert_eq!(result.status, "REJECTED_INVALID_AMOUNT");
    }

    #[test]
    fn test_submit_request_exceeds_balance() {
        let result =
            CreditMemoRequestService::submit_request("TRX-102", 1500.0, 1000.0, "PRICING_ERROR");
        assert_eq!(result.status, "REJECTED_EXCEEDS_BALANCE");
    }

    #[test]
    fn test_submit_request_missing_reason() {
        let result = CreditMemoRequestService::submit_request("TRX-103", 250.0, 1000.0, "");
        assert_eq!(result.status, "REJECTED_MISSING_REASON");
    }

    #[test]
    fn test_process_approval_approved() {
        let result = CreditMemoRequestService::process_approval("CMR-TRX-100-250", "MGR-001", true);
        assert_eq!(result.status, "APPROVED");
        assert_eq!(result.request_id, "CMR-TRX-100-250");
        assert_eq!(result.approver_id, "MGR-001");
    }

    #[test]
    fn test_process_approval_rejected() {
        let result =
            CreditMemoRequestService::process_approval("CMR-TRX-100-250", "MGR-001", false);
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_process_approval_invalid_request() {
        let result = CreditMemoRequestService::process_approval("", "MGR-001", true);
        assert_eq!(result.status, "INVALID_REQUEST");
    }

    #[test]
    fn test_process_approval_missing_approver() {
        let result = CreditMemoRequestService::process_approval("CMR-TRX-100-250", "", true);
        assert_eq!(result.status, "MISSING_APPROVER");
    }
}
