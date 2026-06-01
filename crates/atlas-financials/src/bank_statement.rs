use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_number: String,
    pub bank_account_id: Uuid,
    pub statement_date: NaiveDate,
    pub closing_balance: Decimal,
    pub status: String, // imported, validating, validated, reconciling, reconciled, exception, cancelled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementLine {
    pub id: Uuid,
    pub statement_id: Uuid,
    pub line_number: i32,
    pub transaction_date: NaiveDate,
    pub amount: Decimal,
    pub transaction_type: String, // credit, debit
    pub match_status: String, // unmatched, matched, partially_matched, exception
}

pub struct BankStatementService {
    statements: Arc<RwLock<Vec<BankStatement>>>,
    lines: Arc<RwLock<Vec<BankStatementLine>>>,
}

impl Default for BankStatementService {
    fn default() -> Self {
        Self::new()
    }
}

impl BankStatementService {
    pub fn new() -> Self {
        Self {
            statements: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_statement(
        &self,
        organization_id: Uuid,
        number: String,
        bank_account_id: Uuid,
        date: NaiveDate,
        closing_balance: Decimal,
    ) -> Result<BankStatement, String> {
        let mut statements = self.statements.write().unwrap();
        if statements.iter().any(|s| {
            s.organization_id == organization_id && 
            s.bank_account_id == bank_account_id && 
            s.statement_number == number && 
            s.statement_date == date
        }) {
            return Err("Statement already exists for this account and date".to_string());
        }

        let stmt = BankStatement {
            id: Uuid::new_v4(),
            organization_id,
            statement_number: number,
            bank_account_id,
            statement_date: date,
            closing_balance,
            status: "imported".to_string(),
        };

        statements.push(stmt.clone());
        Ok(stmt)
    }

    pub fn add_line(
        &self,
        statement_id: Uuid,
        amount: Decimal,
        date: NaiveDate,
        tx_type: String,
    ) -> Result<BankStatementLine, String> {
        let statements = self.statements.read().unwrap();
        if !statements.iter().any(|s| s.id == statement_id) {
            return Err("Statement not found".to_string());
        }

        let mut lines = self.lines.write().unwrap();
        let line_number = (lines.iter().filter(|l| l.statement_id == statement_id).count() as i32) + 1;

        let line = BankStatementLine {
            id: Uuid::new_v4(),
            statement_id,
            line_number,
            transaction_date: date,
            amount,
            transaction_type: tx_type,
            match_status: "unmatched".to_string(),
        };

        lines.push(line.clone());
        Ok(line)
    }

    pub fn auto_match_simple(&self, statement_id: Uuid, system_transactions: &[(Uuid, Decimal, String)]) -> usize {
        let mut lines = self.lines.write().unwrap();
        let mut matched_count = 0;

        for line in lines.iter_mut().filter(|l| l.statement_id == statement_id && l.match_status == "unmatched") {
            // Very simple exact match logic for demo
            if let Some((_sys_id, _, _)) = system_transactions.iter().find(|(_, amt, _)| *amt == line.amount) {

                line.match_status = "matched".to_string();
                // In a real system, we'd record the matched_transaction_id
                matched_count += 1;
            }
        }

        matched_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_and_match_statement() {
        let service = BankStatementService::new();
        let org_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        
        let stmt = service.create_statement(org_id, "ST-001".to_string(), account_id, date, dec!(5000)).unwrap();
        
        service.add_line(stmt.id, dec!(100), date, "credit".to_string()).unwrap();
        service.add_line(stmt.id, dec!(250), date, "debit".to_string()).unwrap();

        // System transactions: (id, amount, type)
        let sys_txs = vec![
            (Uuid::new_v4(), dec!(100), "receipt".to_string()),
        ];

        let matched = service.auto_match_simple(stmt.id, &sys_txs);
        assert_eq!(matched, 1);

        let lines = service.lines.read().unwrap();
        let matched_line = lines.iter().find(|l| l.amount == dec!(100)).unwrap();
        assert_eq!(matched_line.match_status, "matched");

        let unmatched_line = lines.iter().find(|l| l.amount == dec!(250)).unwrap();
        assert_eq!(unmatched_line.match_status, "unmatched");
    }
}
