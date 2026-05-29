//! Oracle Fusion Financial Feature: Customer Tax Exemptions
//! Evaluates whether a customer is exempt from specific taxes based on active certificates.

use chrono::NaiveDate;

pub struct TaxExemptionService;

#[derive(Debug, PartialEq, Clone)]
pub struct TaxExemptionCertificate {
    pub customer_id: String,
    pub tax_regime_code: String,
    pub tax_code: Option<String>, // If None, applies to whole regime
    pub exemption_percentage: f64,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TaxCalculationRequest {
    pub customer_id: String,
    pub tax_regime_code: String,
    pub tax_code: String,
    pub transaction_date: NaiveDate,
    pub original_tax_amount: f64,
}

#[derive(Debug, PartialEq)]
pub struct ExemptionResult {
    pub is_exempt: bool,
    pub exemption_percentage_applied: f64,
    pub final_tax_amount: f64,
    pub reason: Option<String>,
}

impl TaxExemptionService {
    /// Evaluates if a given tax calculation should be exempted (partly or fully) based
    /// on active customer tax exemptions.
    #[must_use]
    pub fn apply_exemptions(
        request: &TaxCalculationRequest,
        certificates: &[TaxExemptionCertificate],
    ) -> ExemptionResult {
        // Find valid active certificates for this customer on the transaction date
        let active_certs: Vec<&TaxExemptionCertificate> = certificates
            .iter()
            .filter(|c| {
                let is_customer = c.customer_id == request.customer_id;
                let is_active_status = c.status == "PRIMARY" || c.status == "MANUAL";
                let is_started = request.transaction_date >= c.start_date;
                let is_not_expired = match c.end_date {
                    Some(end) => request.transaction_date <= end,
                    None => true,
                };
                
                is_customer && is_active_status && is_started && is_not_expired
            })
            .collect();

        if active_certs.is_empty() {
            return ExemptionResult {
                is_exempt: false,
                exemption_percentage_applied: 0.0,
                final_tax_amount: request.original_tax_amount,
                reason: None,
            };
        }

        // Look for the most specific certificate:
        // 1. Matches regime AND specific tax code
        // 2. Matches regime (and tax code is None)
        let specific_match = active_certs.iter().find(|c| {
            c.tax_regime_code == request.tax_regime_code && c.tax_code.as_deref() == Some(request.tax_code.as_str())
        });

        let regime_match = active_certs.iter().find(|c| {
            c.tax_regime_code == request.tax_regime_code && c.tax_code.is_none()
        });

        let applied_cert = specific_match.or(regime_match);

        if let Some(cert) = applied_cert {
            // Apply exemption
            let percent_to_exempt = cert.exemption_percentage;
            let exempted_amount = request.original_tax_amount * (percent_to_exempt / 100.0);
            let final_amount = request.original_tax_amount - exempted_amount;

            ExemptionResult {
                is_exempt: true,
                exemption_percentage_applied: percent_to_exempt,
                final_tax_amount: final_amount.max(0.0), // Prevent negative tax
                reason: Some(format!("Applied exemption from regime {}", cert.tax_regime_code)),
            }
        } else {
            ExemptionResult {
                is_exempt: false,
                exemption_percentage_applied: 0.0,
                final_tax_amount: request.original_tax_amount,
                reason: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_exemption_found() {
        let request = TaxCalculationRequest {
            customer_id: "CUST-1".to_string(),
            tax_regime_code: "US-SALES".to_string(),
            tax_code: "CA-STATE-TAX".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
            original_tax_amount: 100.0,
        };

        let result = TaxExemptionService::apply_exemptions(&request, &[]);
        assert!(!result.is_exempt);
        assert_eq!(result.final_tax_amount, 100.0);
    }

    #[test]
    fn test_full_regime_exemption() {
        let certs = vec![
            TaxExemptionCertificate {
                customer_id: "CUST-1".to_string(),
                tax_regime_code: "US-SALES".to_string(),
                tax_code: None, // Applies to all taxes under US-SALES
                exemption_percentage: 100.0,
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: None,
                status: "PRIMARY".to_string(),
            }
        ];

        let request = TaxCalculationRequest {
            customer_id: "CUST-1".to_string(),
            tax_regime_code: "US-SALES".to_string(),
            tax_code: "NY-CITY-TAX".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            original_tax_amount: 50.0,
        };

        let result = TaxExemptionService::apply_exemptions(&request, &certs);
        assert!(result.is_exempt);
        assert_eq!(result.exemption_percentage_applied, 100.0);
        assert_eq!(result.final_tax_amount, 0.0); // Fully exempted
    }

    #[test]
    fn test_specific_tax_exemption_prioritized() {
        let certs = vec![
            TaxExemptionCertificate {
                customer_id: "CUST-2".to_string(),
                tax_regime_code: "EU-VAT".to_string(),
                tax_code: None,
                exemption_percentage: 50.0, // 50% off the regime generally
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: None,
                status: "PRIMARY".to_string(),
            },
            TaxExemptionCertificate {
                customer_id: "CUST-2".to_string(),
                tax_regime_code: "EU-VAT".to_string(),
                tax_code: Some("FR-VAT-STANDARD".to_string()),
                exemption_percentage: 100.0, // 100% off this specific tax
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: None,
                status: "PRIMARY".to_string(),
            }
        ];

        let request = TaxCalculationRequest {
            customer_id: "CUST-2".to_string(),
            tax_regime_code: "EU-VAT".to_string(),
            tax_code: "FR-VAT-STANDARD".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            original_tax_amount: 200.0,
        };

        let result = TaxExemptionService::apply_exemptions(&request, &certs);
        assert!(result.is_exempt);
        // The specific 100% rule should have been applied instead of the 50% rule
        assert_eq!(result.exemption_percentage_applied, 100.0);
        assert_eq!(result.final_tax_amount, 0.0);
    }

    #[test]
    fn test_expired_certificate() {
        let certs = vec![
            TaxExemptionCertificate {
                customer_id: "CUST-3".to_string(),
                tax_regime_code: "UK-VAT".to_string(),
                tax_code: None,
                exemption_percentage: 100.0,
                start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                end_date: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()), // Expired
                status: "PRIMARY".to_string(),
            }
        ];

        let request = TaxCalculationRequest {
            customer_id: "CUST-3".to_string(),
            tax_regime_code: "UK-VAT".to_string(),
            tax_code: "STANDARD".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(), // Past expiration
            original_tax_amount: 20.0,
        };

        let result = TaxExemptionService::apply_exemptions(&request, &certs);
        assert!(!result.is_exempt);
        assert_eq!(result.final_tax_amount, 20.0); // Tax is still charged
    }

    #[test]
    fn test_partial_exemption() {
        let certs = vec![
            TaxExemptionCertificate {
                customer_id: "CUST-4".to_string(),
                tax_regime_code: "CA-GST".to_string(),
                tax_code: None,
                exemption_percentage: 25.0, // Only 25% exempt
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: None,
                status: "PRIMARY".to_string(),
            }
        ];

        let request = TaxCalculationRequest {
            customer_id: "CUST-4".to_string(),
            tax_regime_code: "CA-GST".to_string(),
            tax_code: "GST".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            original_tax_amount: 100.0,
        };

        let result = TaxExemptionService::apply_exemptions(&request, &certs);
        assert!(result.is_exempt);
        assert_eq!(result.exemption_percentage_applied, 25.0);
        // 25% of 100 is 25. 100 - 25 = 75 final tax
        assert_eq!(result.final_tax_amount, 75.0);
    }
}
