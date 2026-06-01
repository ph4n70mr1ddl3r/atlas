use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{NaiveDate, Datelike, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringInvoiceTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_number: String,
    pub template_name: String,
    pub supplier_id: Option<Uuid>,
    pub invoice_type: String, // standard, credit_memo, debit_memo, prepayment
    pub amount_type: String, // fixed, variable, adjusted
    pub recurrence_type: String, // daily, weekly, monthly, quarterly, semi_annual, annual
    pub recurrence_interval: i32,
    pub generation_day: i32,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub status: String, // draft, active, suspended, completed, cancelled
    pub next_generation_date: Option<NaiveDate>,
}

pub struct RecurringInvoiceService {
    templates: Arc<RwLock<Vec<RecurringInvoiceTemplate>>>,
}

impl Default for RecurringInvoiceService {
    fn default() -> Self {
        Self {
            templates: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl RecurringInvoiceService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_template(
        &self,
        organization_id: Uuid,
        number: String,
        name: String,
        recurrence: String,
        interval: i32,
        gen_day: i32,
        effective_from: NaiveDate,
    ) -> Result<RecurringInvoiceTemplate, String> {
        let valid_recurrences = ["daily", "weekly", "monthly", "quarterly", "semi_annual", "annual"];
        if !valid_recurrences.contains(&recurrence.as_str()) {
            return Err("Invalid recurrence type".to_string());
        }

        let mut templates = self.templates.write().unwrap();
        if templates.iter().any(|t| t.organization_id == organization_id && t.template_number == number) {
            return Err("Template with this number already exists".to_string());
        }

        let mut template = RecurringInvoiceTemplate {
            id: Uuid::new_v4(),
            organization_id,
            template_number: number,
            template_name: name,
            supplier_id: None,
            invoice_type: "standard".to_string(),
            amount_type: "fixed".to_string(),
            recurrence_type: recurrence,
            recurrence_interval: interval,
            generation_day: gen_day,
            effective_from,
            effective_to: None,
            status: "draft".to_string(),
            next_generation_date: None,
        };

        template.next_generation_date = self.calculate_next_date(&template, effective_from);

        templates.push(template.clone());
        Ok(template)
    }

    fn calculate_next_date(&self, template: &RecurringInvoiceTemplate, after_date: NaiveDate) -> Option<NaiveDate> {
        match template.recurrence_type.as_str() {
            "monthly" => {
                let mut year = after_date.year();
                let mut month = after_date.month() as i32 + template.recurrence_interval;
                while month > 12 {
                    month -= 12;
                    year += 1;
                }
                NaiveDate::from_ymd_opt(year, month as u32, template.generation_day as u32)
                    .or_else(|| NaiveDate::from_ymd_opt(year, month as u32, 28)) // Fallback for short months
            },
            "daily" => {
                after_date.checked_add_signed(Duration::days(template.recurrence_interval as i64))
            },
            _ => Some(after_date + Duration::days(30)), // Simplistic fallback
        }
    }

    pub fn activate_template(&self, id: Uuid) -> Result<(), String> {
        let mut templates = self.templates.write().unwrap();
        let t = templates.iter_mut().find(|t| t.id == id)
            .ok_or_else(|| "Template not found".to_string())?;
        
        t.status = "active".to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_recurring_template() {
        let service = RecurringInvoiceService::new();
        let org_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        
        let t = service.create_template(
            org_id,
            "RENT-001".to_string(),
            "Monthly Rent".to_string(),
            "monthly".to_string(),
            1,
            1,
            start,
        ).unwrap();

        assert_eq!(t.next_generation_date, Some(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()));
    }

    #[test]
    fn test_activate_template() {
        let service = RecurringInvoiceService::new();
        let org_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        
        let t = service.create_template(
            org_id, "T1".to_string(), "T1".to_string(), "daily".to_string(), 1, 1, start
        ).unwrap();

        assert_eq!(t.status, "draft");
        service.activate_template(t.id).unwrap();
        
        let templates = service.templates.read().unwrap();
        assert_eq!(templates[0].status, "active");
    }
}
