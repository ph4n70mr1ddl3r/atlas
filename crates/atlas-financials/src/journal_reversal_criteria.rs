use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{NaiveDate, Datelike, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalReversalCriteriaSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReversalPeriod {
    NextPeriod,
    NextDay,
    SamePeriod,
    SameDay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReversalMethod {
    SwitchDrCr,
    SignReverse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalReversalCriteriaRule {
    pub id: Uuid,
    pub criteria_set_id: Uuid,
    pub journal_category: String,
    pub reversal_period: ReversalPeriod,
    pub reversal_method: ReversalMethod,
    pub is_automatic_reversal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversalAction {
    pub reversal_date: NaiveDate,
    pub method: ReversalMethod,
    pub is_automatic: bool,
}

pub struct JournalReversalCriteriaService {
    sets: Arc<RwLock<Vec<JournalReversalCriteriaSet>>>,
    rules: Arc<RwLock<Vec<JournalReversalCriteriaRule>>>,
}

impl Default for JournalReversalCriteriaService {
    fn default() -> Self {
        Self {
            sets: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl JournalReversalCriteriaService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_criteria_set(
        &self,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<JournalReversalCriteriaSet, String> {
        let set = JournalReversalCriteriaSet {
            id: Uuid::new_v4(),
            organization_id,
            name: name.clone(),
            description,
            is_active: true,
        };

        let mut sets = self.sets.write().unwrap();
        if sets.iter().any(|s| s.organization_id == organization_id && s.name == name) {
            return Err("Criteria set with this name already exists".to_string());
        }

        sets.push(set.clone());
        Ok(set)
    }

    pub fn add_rule(
        &self,
        criteria_set_id: Uuid,
        journal_category: String,
        reversal_period: ReversalPeriod,
        reversal_method: ReversalMethod,
        is_automatic_reversal: bool,
    ) -> Result<JournalReversalCriteriaRule, String> {
        let mut rules = self.rules.write().unwrap();
        
        if rules.iter().any(|r| r.criteria_set_id == criteria_set_id && r.journal_category == journal_category) {
            return Err("Rule for this journal category already exists in the criteria set".to_string());
        }

        let rule = JournalReversalCriteriaRule {
            id: Uuid::new_v4(),
            criteria_set_id,
            journal_category,
            reversal_period,
            reversal_method,
            is_automatic_reversal,
        };

        rules.push(rule.clone());
        Ok(rule)
    }

    pub fn get_reversal_action(
        &self,
        criteria_set_id: Uuid,
        journal_category: &str,
        accounting_date: NaiveDate,
    ) -> Option<ReversalAction> {
        let rules = self.rules.read().unwrap();
        if let Some(rule) = rules.iter().find(|r| r.criteria_set_id == criteria_set_id && r.journal_category == journal_category) {
            let reversal_date = match rule.reversal_period {
                ReversalPeriod::SameDay | ReversalPeriod::SamePeriod => accounting_date,
                ReversalPeriod::NextDay => accounting_date + Duration::days(1),
                ReversalPeriod::NextPeriod => {
                    // Simplistic mock for next period: Add 1 month, set to day 1
                    let mut year = accounting_date.year();
                    let mut month = accounting_date.month() + 1;
                    if month > 12 {
                        month = 1;
                        year += 1;
                    }
                    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(accounting_date)
                }
            };

            return Some(ReversalAction {
                reversal_date,
                method: rule.reversal_method.clone(),
                is_automatic: rule.is_automatic_reversal,
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_criteria_set() {
        let service = JournalReversalCriteriaService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_criteria_set(
            org_id,
            "Standard Reversals".to_string(),
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_rule() {
        let service = JournalReversalCriteriaService::new();
        let org_id = Uuid::new_v4();
        
        let set = service.create_criteria_set(org_id, "Set A".to_string(), None).unwrap();
        
        let result = service.add_rule(
            set.id,
            "Accrual".to_string(),
            ReversalPeriod::NextPeriod,
            ReversalMethod::SwitchDrCr,
            true,
        );

        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.journal_category, "Accrual");
    }

    #[test]
    fn test_duplicate_rule() {
        let service = JournalReversalCriteriaService::new();
        let org_id = Uuid::new_v4();
        
        let set = service.create_criteria_set(org_id, "Set A".to_string(), None).unwrap();
        
        service.add_rule(
            set.id,
            "Accrual".to_string(),
            ReversalPeriod::NextPeriod,
            ReversalMethod::SwitchDrCr,
            true,
        ).unwrap();

        let result = service.add_rule(
            set.id,
            "Accrual".to_string(),
            ReversalPeriod::SameDay,
            ReversalMethod::SignReverse,
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_reversal_action_next_period() {
        let service = JournalReversalCriteriaService::new();
        let org_id = Uuid::new_v4();
        
        let set = service.create_criteria_set(org_id, "Set A".to_string(), None).unwrap();
        
        service.add_rule(
            set.id,
            "Accrual".to_string(),
            ReversalPeriod::NextPeriod,
            ReversalMethod::SwitchDrCr,
            true,
        ).unwrap();

        let accounting_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        
        let action = service.get_reversal_action(set.id, "Accrual", accounting_date);
        assert!(action.is_some());
        
        let action = action.unwrap();
        assert_eq!(action.reversal_date, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(action.method, ReversalMethod::SwitchDrCr);
        assert!(action.is_automatic);
    }

    #[test]
    fn test_get_reversal_action_next_day() {
        let service = JournalReversalCriteriaService::new();
        let org_id = Uuid::new_v4();
        
        let set = service.create_criteria_set(org_id, "Set A".to_string(), None).unwrap();
        
        service.add_rule(
            set.id,
            "Daily Accrual".to_string(),
            ReversalPeriod::NextDay,
            ReversalMethod::SignReverse,
            false,
        ).unwrap();

        let accounting_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        
        let action = service.get_reversal_action(set.id, "Daily Accrual", accounting_date);
        assert!(action.is_some());
        
        let action = action.unwrap();
        assert_eq!(action.reversal_date, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(action.method, ReversalMethod::SignReverse);
        assert!(!action.is_automatic);
    }
}
