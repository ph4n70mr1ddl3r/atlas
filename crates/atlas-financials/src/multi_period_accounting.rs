pub struct MultiPeriodAccountingService;

pub struct MpaScheduleResult {
    pub schedule_id: String,
    pub source_document_id: String,
    pub total_amount: f64,
    pub periods: u32,
    pub amount_per_period: f64,
    pub status: String,
}

pub struct PeriodRecognitionResult {
    pub schedule_id: String,
    pub period_number: u32,
    pub recognized_amount: f64,
    pub status: String,
}

impl MultiPeriodAccountingService {
    /// Creates a Multi-Period Accounting (MPA) schedule.
    /// This is an Oracle Fusion Financials feature that enables users to recognize 
    /// expenses or revenues across multiple GL periods (amortization).
    #[must_use]
    pub fn create_schedule(source_document_id: &str, total_amount: f64, periods: u32) -> MpaScheduleResult {
        if periods == 0 || total_amount <= 0.0 {
            return MpaScheduleResult {
                schedule_id: "".to_string(),
                source_document_id: source_document_id.to_string(),
                total_amount: if total_amount > 0.0 { total_amount } else { 0.0 },
                periods,
                amount_per_period: 0.0,
                status: "REJECTED".to_string(),
            };
        }

        // Simple straight-line amortization
        let amount_per_period = (total_amount / periods as f64 * 100.0).round() / 100.0;
        
        MpaScheduleResult {
            schedule_id: format!("MPA-{}-{}", source_document_id, periods),
            source_document_id: source_document_id.to_string(),
            total_amount,
            periods,
            amount_per_period,
            status: "ACTIVE".to_string(),
        }
    }

    /// Recognizes the accounting entry for a specific period in the schedule.
    #[must_use]
    pub fn recognize_period(schedule_id: &str, period_number: u32, amount_per_period: f64) -> PeriodRecognitionResult {
        if schedule_id.is_empty() || period_number == 0 {
            return PeriodRecognitionResult {
                schedule_id: schedule_id.to_string(),
                period_number,
                recognized_amount: 0.0,
                status: "INVALID_PERIOD".to_string(),
            };
        }

        PeriodRecognitionResult {
            schedule_id: schedule_id.to_string(),
            period_number,
            recognized_amount: amount_per_period,
            status: "RECOGNIZED".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_schedule_valid() {
        let result = MultiPeriodAccountingService::create_schedule("INV-1001", 12000.0, 12);
        assert_eq!(result.source_document_id, "INV-1001");
        assert_eq!(result.total_amount, 12000.0);
        assert_eq!(result.periods, 12);
        assert_eq!(result.amount_per_period, 1000.0);
        assert_eq!(result.schedule_id, "MPA-INV-1001-12");
        assert_eq!(result.status, "ACTIVE");
    }

    #[test]
    fn test_create_schedule_rounding() {
        let result = MultiPeriodAccountingService::create_schedule("INV-1002", 1000.0, 3);
        assert_eq!(result.periods, 3);
        assert_eq!(result.amount_per_period, 333.33); // 1000 / 3 = 333.333... rounded to 2 decimals
        assert_eq!(result.status, "ACTIVE");
    }

    #[test]
    fn test_create_schedule_invalid_periods() {
        let result = MultiPeriodAccountingService::create_schedule("INV-1003", 5000.0, 0);
        assert_eq!(result.amount_per_period, 0.0);
        assert_eq!(result.schedule_id, "");
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_create_schedule_invalid_amount() {
        let result = MultiPeriodAccountingService::create_schedule("INV-1004", -100.0, 5);
        assert_eq!(result.amount_per_period, 0.0);
        assert_eq!(result.schedule_id, "");
        assert_eq!(result.status, "REJECTED");
    }

    #[test]
    fn test_recognize_period_valid() {
        let result = MultiPeriodAccountingService::recognize_period("MPA-INV-1001-12", 1, 1000.0);
        assert_eq!(result.schedule_id, "MPA-INV-1001-12");
        assert_eq!(result.period_number, 1);
        assert_eq!(result.recognized_amount, 1000.0);
        assert_eq!(result.status, "RECOGNIZED");
    }

    #[test]
    fn test_recognize_period_invalid() {
        let result = MultiPeriodAccountingService::recognize_period("", 1, 1000.0);
        assert_eq!(result.status, "INVALID_PERIOD");
        assert_eq!(result.recognized_amount, 0.0);
    }
}
