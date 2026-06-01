pub struct PositivePayService;

pub struct PositivePayFileResult {
    pub file_id: String,
    pub bank_account_id: String,
    pub total_records: u32,
    pub total_amount: f64,
    pub status: String,
}

pub struct PositivePayTransmissionResult {
    pub file_id: String,
    pub transmission_status: String,
    pub confirmation_code: String,
}

impl PositivePayService {
    /// Generates a Positive Pay file for a specific bank account.
    /// Positive Pay is an automated fraud detection tool in Oracle Fusion Cash Management
    /// that matches the account number, check number, and dollar amount of each check presented for payment
    /// against a list of checks previously authorized and issued.
    #[must_use]
    pub fn generate_file(
        bank_account_id: &str,
        payment_records: &[(String, f64)],
    ) -> PositivePayFileResult {
        if bank_account_id.is_empty() {
            return PositivePayFileResult {
                file_id: "".to_string(),
                bank_account_id: "".to_string(),
                total_records: 0,
                total_amount: 0.0,
                status: "REJECTED_INVALID_BANK_ACCOUNT".to_string(),
            };
        }

        if payment_records.is_empty() {
            return PositivePayFileResult {
                file_id: "".to_string(),
                bank_account_id: bank_account_id.to_string(),
                total_records: 0,
                total_amount: 0.0,
                status: "REJECTED_NO_RECORDS".to_string(),
            };
        }

        if payment_records
            .iter()
            .any(|(chk, amt)| chk.is_empty() || *amt <= 0.0)
        {
            return PositivePayFileResult {
                file_id: "".to_string(),
                bank_account_id: bank_account_id.to_string(),
                total_records: 0,
                total_amount: 0.0,
                status: "REJECTED_INVALID_RECORD".to_string(),
            };
        }

        let total_records = payment_records.len() as u32;
        let total_amount: f64 = payment_records
            .iter()
            .map(|(_, amt)| if *amt > 0.0 { *amt } else { 0.0 })
            .sum();

        PositivePayFileResult {
            file_id: format!("PPF-{}-{}", bank_account_id, total_records),
            bank_account_id: bank_account_id.to_string(),
            total_records,
            total_amount,
            status: "GENERATED".to_string(),
        }
    }

    /// Transmits a generated Positive Pay file to the bank.
    #[must_use]
    pub fn transmit_file(file_id: &str) -> PositivePayTransmissionResult {
        if file_id.is_empty() {
            PositivePayTransmissionResult {
                file_id: file_id.to_string(),
                transmission_status: "FAILED".to_string(),
                confirmation_code: "".to_string(),
            }
        } else {
            PositivePayTransmissionResult {
                file_id: file_id.to_string(),
                transmission_status: "TRANSMITTED".to_string(),
                confirmation_code: format!("CONF-{}", file_id),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_valid() {
        let records = vec![
            ("CHK-1001".to_string(), 1500.0),
            ("CHK-1002".to_string(), 250.0),
            ("CHK-1003".to_string(), 5000.0),
        ];
        let result = PositivePayService::generate_file("BANK-001", &records);
        assert_eq!(result.bank_account_id, "BANK-001");
        assert_eq!(result.total_records, 3);
        assert_eq!(result.total_amount, 6750.0);
        assert_eq!(result.file_id, "PPF-BANK-001-3");
        assert_eq!(result.status, "GENERATED");
    }

    #[test]
    fn test_generate_file_no_records() {
        let records: Vec<(String, f64)> = vec![];
        let result = PositivePayService::generate_file("BANK-002", &records);
        assert_eq!(result.total_records, 0);
        assert_eq!(result.total_amount, 0.0);
        assert_eq!(result.status, "REJECTED_NO_RECORDS");
    }

    #[test]
    fn test_generate_file_invalid_bank() {
        let records = vec![("CHK-1004".to_string(), 100.0)];
        let result = PositivePayService::generate_file("", &records);
        assert_eq!(result.status, "REJECTED_INVALID_BANK_ACCOUNT");
    }

    #[test]
    fn test_transmit_file_valid() {
        let result = PositivePayService::transmit_file("PPF-BANK-001-3");
        assert_eq!(result.file_id, "PPF-BANK-001-3");
        assert_eq!(result.transmission_status, "TRANSMITTED");
        assert_eq!(result.confirmation_code, "CONF-PPF-BANK-001-3");
    }

    #[test]
    fn test_transmit_file_invalid() {
        let result = PositivePayService::transmit_file("");
        assert_eq!(result.transmission_status, "FAILED");
        assert_eq!(result.confirmation_code, "");
    }

    #[test]
    fn test_generate_file_negative_amount() {
        let records = vec![
            ("CHK-1001".to_string(), 1500.0),
            ("CHK-1002".to_string(), -10.0),
        ];
        let result = PositivePayService::generate_file("BANK-001", &records);
        assert_eq!(result.status, "REJECTED_INVALID_RECORD");
        assert_eq!(result.total_records, 0);
        assert_eq!(result.total_amount, 0.0);
    }

    #[test]
    fn test_generate_file_empty_check_number() {
        let records = vec![("".to_string(), 1500.0)];
        let result = PositivePayService::generate_file("BANK-001", &records);
        assert_eq!(result.status, "REJECTED_INVALID_RECORD");
        assert_eq!(result.total_records, 0);
        assert_eq!(result.total_amount, 0.0);
    }
}
