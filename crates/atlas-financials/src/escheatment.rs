//! Oracle Fusion Financial Feature: Escheatment (Unclaimed Property)
//! Manages the identification and transfer of uncashed payments to a legal authority.

use chrono::NaiveDate;

pub struct EscheatmentService;

#[derive(Debug, PartialEq, Clone)]
pub struct EscheatmentAuthority {
    pub authority_id: String,
    pub name: String,
    pub jurisdiction: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ApPayment {
    pub payment_id: String,
    pub amount: f64,
    pub payment_date: NaiveDate,
    pub status: String, // e.g., "NEGOTIABLE", "CLEARED", "ESCHEATED"
}

#[derive(Debug, PartialEq)]
pub struct EscheatedPaymentRecord {
    pub payment_id: String,
    pub authority_id: String,
    pub escheated_amount: f64,
    pub escheatment_date: NaiveDate,
    pub status: String, // "INITIATED", "REMITTED"
}

impl EscheatmentService {
    /// Identifies payments that have not been cashed and are older than the specified days threshold.
    #[must_use]
    pub fn identify_stale_payments(
        payments: &[ApPayment],
        as_of_date: NaiveDate,
        stale_days_threshold: i64,
    ) -> Vec<ApPayment> {
        payments
            .iter()
            .filter(|p| {
                p.status == "NEGOTIABLE" 
                && (as_of_date - p.payment_date).num_days() >= stale_days_threshold
            })
            .cloned()
            .collect()
    }

    /// Processes the escheatment of a specific payment to the designated authority.
    /// Returns the updated payment and the escheatment record, or an error if invalid.
    pub fn process_escheatment(
        payment: &ApPayment,
        authority: &EscheatmentAuthority,
        escheatment_date: NaiveDate,
    ) -> Result<(ApPayment, EscheatedPaymentRecord), String> {
        if payment.status != "NEGOTIABLE" {
            return Err("Only NEGOTIABLE payments can be escheated".to_string());
        }

        let updated_payment = ApPayment {
            payment_id: payment.payment_id.clone(),
            amount: payment.amount,
            payment_date: payment.payment_date,
            status: "ESCHEATED".to_string(),
        };

        let record = EscheatedPaymentRecord {
            payment_id: payment.payment_id.clone(),
            authority_id: authority.authority_id.clone(),
            escheated_amount: payment.amount,
            escheatment_date,
            status: "INITIATED".to_string(),
        };

        Ok((updated_payment, record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_stale_payments() {
        let payments = vec![
            ApPayment {
                payment_id: "PAY-1".to_string(),
                amount: 1000.0,
                payment_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                status: "NEGOTIABLE".to_string(),
            },
            ApPayment {
                payment_id: "PAY-2".to_string(),
                amount: 2000.0,
                payment_date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
                status: "NEGOTIABLE".to_string(), // not stale yet
            },
            ApPayment {
                payment_id: "PAY-3".to_string(),
                amount: 3000.0,
                payment_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                status: "CLEARED".to_string(), // already cleared
            },
        ];

        let as_of_date = NaiveDate::from_ymd_opt(2025, 11, 1).unwrap();
        // Assume stale threshold is 180 days
        let stale_payments = EscheatmentService::identify_stale_payments(&payments, as_of_date, 180);

        assert_eq!(stale_payments.len(), 1);
        assert_eq!(stale_payments[0].payment_id, "PAY-1");
    }

    #[test]
    fn test_process_escheatment_success() {
        let payment = ApPayment {
            payment_id: "PAY-1".to_string(),
            amount: 1500.0,
            payment_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            status: "NEGOTIABLE".to_string(),
        };

        let authority = EscheatmentAuthority {
            authority_id: "AUTH-DELAWARE".to_string(),
            name: "State of Delaware".to_string(),
            jurisdiction: "DE".to_string(),
        };

        let escheatment_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        let result = EscheatmentService::process_escheatment(&payment, &authority, escheatment_date);
        
        assert!(result.is_ok());
        let (updated_payment, record) = result.unwrap();

        assert_eq!(updated_payment.status, "ESCHEATED");
        assert_eq!(record.authority_id, "AUTH-DELAWARE");
        assert_eq!(record.escheated_amount, 1500.0);
        assert_eq!(record.status, "INITIATED");
    }

    #[test]
    fn test_process_escheatment_invalid_status() {
        let payment = ApPayment {
            payment_id: "PAY-2".to_string(),
            amount: 500.0,
            payment_date: NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
            status: "CLEARED".to_string(),
        };

        let authority = EscheatmentAuthority {
            authority_id: "AUTH-NY".to_string(),
            name: "New York State".to_string(),
            jurisdiction: "NY".to_string(),
        };

        let result = EscheatmentService::process_escheatment(&payment, &authority, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Only NEGOTIABLE payments can be escheated");
    }
}
