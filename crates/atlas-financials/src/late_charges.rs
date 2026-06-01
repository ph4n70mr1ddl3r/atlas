use atlas_shared::{AtlasError, AtlasResult, RecordId};
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Represents an overdue invoice that needs late charges calculated
#[derive(Debug, Clone)]
pub struct OverdueInvoice {
    pub invoice_id: RecordId,
    pub due_date: NaiveDate,
    pub outstanding_amount: Decimal,
}

/// Represents the calculated late charge for an invoice
#[derive(Debug, Clone, PartialEq)]
pub struct LateCharge {
    pub invoice_id: RecordId,
    pub charge_amount: Decimal,
    pub days_overdue: i64,
}

/// Service to calculate Late Charges for overdue invoices,
/// inspired by Oracle Fusion Financials Advanced Collections / Receivables features.
pub struct LateChargesService {
    // In a real implementation, this would connect to the database.
}

impl LateChargesService {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculates late charges for a list of overdue invoices based on a given annual interest rate.
    /// The calculation date is used to determine how many days the invoice is overdue.
    pub fn calculate_late_charges(
        &self,
        invoices: &[OverdueInvoice],
        annual_interest_rate: Decimal,
        calculation_date: NaiveDate,
        minimum_charge: Decimal,
    ) -> AtlasResult<Vec<LateCharge>> {
        if annual_interest_rate < Decimal::new(0, 0) {
            return Err(AtlasError::ValidationFailed(
                "Interest rate cannot be negative".to_string(),
            ));
        }

        if minimum_charge < Decimal::new(0, 0) {
            return Err(AtlasError::ValidationFailed(
                "Minimum charge cannot be negative".to_string(),
            ));
        }

        let daily_rate = annual_interest_rate / Decimal::new(365, 0);
        let mut late_charges = Vec::new();

        for invoice in invoices {
            if calculation_date <= invoice.due_date {
                continue; // Not overdue
            }

            let days_overdue = (calculation_date - invoice.due_date).num_days();

            // simple interest: outstanding_amount * daily_rate * days_overdue
            let mut charge_amount =
                invoice.outstanding_amount * daily_rate * Decimal::from(days_overdue);
            charge_amount = charge_amount.round_dp(2);

            if charge_amount < minimum_charge {
                charge_amount = minimum_charge;
            }

            if charge_amount > Decimal::new(0, 0) {
                late_charges.push(LateCharge {
                    invoice_id: invoice.invoice_id,
                    charge_amount,
                    days_overdue,
                });
            }
        }

        Ok(late_charges)
    }
}

impl Default for LateChargesService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[test]
    fn test_calculate_late_charges() {
        let service = LateChargesService::new();
        let calculation_date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let invoice1_id = Uuid::new_v4();
        let invoice2_id = Uuid::new_v4();

        let invoices = vec![
            OverdueInvoice {
                invoice_id: invoice1_id,
                due_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), // 31 days overdue
                outstanding_amount: dec!(1000.00),
            },
            OverdueInvoice {
                invoice_id: invoice2_id,
                due_date: NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(), // 17 days overdue
                outstanding_amount: dec!(500.00),
            },
        ];

        let annual_rate = dec!(0.10); // 10% annual
        let minimum_charge = dec!(5.00);

        let charges = service
            .calculate_late_charges(&invoices, annual_rate, calculation_date, minimum_charge)
            .unwrap();

        assert_eq!(charges.len(), 2);

        let daily_rate = dec!(0.10) / dec!(365);

        let expected_charge_1 = (dec!(1000.00) * daily_rate * dec!(31)).round_dp(2);

        assert_eq!(charges[0].invoice_id, invoice1_id);
        assert_eq!(charges[0].days_overdue, 31);
        assert_eq!(charges[0].charge_amount, expected_charge_1);

        assert_eq!(charges[1].invoice_id, invoice2_id);
        assert_eq!(charges[1].days_overdue, 17);
        assert_eq!(charges[1].charge_amount, dec!(5.00)); // Minimum charge applies
    }

    #[test]
    fn test_calculate_late_charges_not_overdue() {
        let service = LateChargesService::new();
        let calculation_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let invoice1_id = Uuid::new_v4();

        let invoices = vec![OverdueInvoice {
            invoice_id: invoice1_id,
            due_date: NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(), // Not overdue yet
            outstanding_amount: dec!(1000.00),
        }];

        let charges = service
            .calculate_late_charges(&invoices, dec!(0.10), calculation_date, dec!(5.00))
            .unwrap();

        assert_eq!(charges.len(), 0);
    }

    #[test]
    fn test_calculate_late_charges_negative_minimum_charge() {
        let service = LateChargesService::new();
        let calculation_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let invoices = vec![];

        let res =
            service.calculate_late_charges(&invoices, dec!(0.10), calculation_date, dec!(-5.00));

        assert!(res.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = res {
            assert!(msg.contains("Minimum charge cannot be negative"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }
}
