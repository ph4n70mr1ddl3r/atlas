use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOffsetTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_code: String,
    pub template_name: String,
    pub description: Option<String>,
    pub balancing_segment: String,
    pub generation_method: String,
    pub default_offset_account: String,
    pub enable_intra_entity: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOffsetTemplateLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub line_number: i32,
    pub balancing_segment_value: String,
    pub due_to_account: String,
    pub due_from_account: String,
    pub priority: i32,
}

pub struct AutomaticOffsetTemplateService {
    templates: Arc<RwLock<Vec<AutoOffsetTemplate>>>,
    lines: Arc<RwLock<Vec<AutoOffsetTemplateLine>>>,
}

impl Default for AutomaticOffsetTemplateService {
    fn default() -> Self {
        Self::new()
    }
}

impl AutomaticOffsetTemplateService {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(Vec::new())),
            lines: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_template(
        &self,
        organization_id: Uuid,
        template_code: String,
        template_name: String,
        description: Option<String>,
        balancing_segment: String,
        generation_method: String,
        default_offset_account: String,
        enable_intra_entity: bool,
    ) -> Result<AutoOffsetTemplate, String> {
        let valid_segments = ["entity", "department", "cost_center", "location", "intercompany"];
        if !valid_segments.contains(&balancing_segment.as_str()) {
            return Err("Invalid balancing segment".to_string());
        }

        let valid_methods = ["single_entry", "multi_entry", "net_zero"];
        if !valid_methods.contains(&generation_method.as_str()) {
            return Err("Invalid generation method".to_string());
        }

        let mut templates = self.templates.write().unwrap();
        if templates.iter().any(|t| t.organization_id == organization_id && t.template_code == template_code) {
            return Err("Template with this code already exists for the organization".to_string());
        }

        let template = AutoOffsetTemplate {
            id: Uuid::new_v4(),
            organization_id,
            template_code,
            template_name,
            description,
            balancing_segment,
            generation_method,
            default_offset_account,
            enable_intra_entity,
            is_active: true,
        };

        templates.push(template.clone());
        Ok(template)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_template_line(
        &self,
        organization_id: Uuid,
        template_id: Uuid,
        line_number: i32,
        balancing_segment_value: String,
        due_to_account: String,
        due_from_account: String,
        priority: i32,
    ) -> Result<AutoOffsetTemplateLine, String> {
        let templates = self.templates.read().unwrap();
        if !templates.iter().any(|t| t.id == template_id) {
            return Err("Template not found".to_string());
        }

        let mut lines = self.lines.write().unwrap();
        if lines.iter().any(|l| l.template_id == template_id && l.line_number == line_number) {
            return Err("Line number already exists in this template".to_string());
        }

        let line = AutoOffsetTemplateLine {
            id: Uuid::new_v4(),
            organization_id,
            template_id,
            line_number,
            balancing_segment_value,
            due_to_account,
            due_from_account,
            priority,
        };

        lines.push(line.clone());
        Ok(line)
    }

    pub fn resolve_offset_accounts(&self, template_id: Uuid, target_segment_value: &str) -> Option<(String, String)> {
        let templates = self.templates.read().unwrap();
        let default_acct = templates.iter().find(|t| t.id == template_id).map(|t| t.default_offset_account.clone());

        let lines = self.lines.read().unwrap();
        let mut matches: Vec<&AutoOffsetTemplateLine> = lines.iter()
            .filter(|l| l.template_id == template_id && l.balancing_segment_value == target_segment_value)
            .collect();
            
        matches.sort_by_key(|l| std::cmp::Reverse(l.priority));
        
        if let Some(best_match) = matches.first() {
            Some((best_match.due_to_account.clone(), best_match.due_from_account.clone()))
        } else {
            default_acct.map(|def_acct| (def_acct.clone(), def_acct))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template() {
        let service = AutomaticOffsetTemplateService::new();
        let org_id = Uuid::new_v4();
        
        let result = service.create_template(
            org_id,
            "DEF_OFFSET".to_string(),
            "Default Offsets".to_string(),
            None,
            "entity".to_string(),
            "single_entry".to_string(),
            "9999-IC-ACCT".to_string(),
            false,
        );

        assert!(result.is_ok());
        let t = result.unwrap();
        assert_eq!(t.template_code, "DEF_OFFSET");
    }

    #[test]
    fn test_invalid_segment_method() {
        let service = AutomaticOffsetTemplateService::new();
        let org_id = Uuid::new_v4();
        
        let res = service.create_template(org_id, "CODE".to_string(), "N".to_string(), None, "invalid".to_string(), "single_entry".to_string(), "AC".to_string(), false);
        assert!(res.is_err());
        
        let res2 = service.create_template(org_id, "CODE".to_string(), "N".to_string(), None, "entity".to_string(), "invalid".to_string(), "AC".to_string(), false);
        assert!(res2.is_err());
    }

    #[test]
    fn test_add_template_line_and_resolve() {
        let service = AutomaticOffsetTemplateService::new();
        let org_id = Uuid::new_v4();
        
        let t = service.create_template(
            org_id, "DEF".to_string(), "Def".to_string(), None, "entity".to_string(), "single_entry".to_string(), "DEF-ACCT".to_string(), false
        ).unwrap();

        service.add_template_line(
            org_id, t.id, 1, "US-ENT".to_string(), "US-DUE-TO".to_string(), "US-DUE-FROM".to_string(), 100
        ).unwrap();

        // Specific match
        let (dt, df) = service.resolve_offset_accounts(t.id, "US-ENT").unwrap();
        assert_eq!(dt, "US-DUE-TO");
        assert_eq!(df, "US-DUE-FROM");

        // Fallback to default
        let (dt_def, df_def) = service.resolve_offset_accounts(t.id, "UK-ENT").unwrap();
        assert_eq!(dt_def, "DEF-ACCT");
        assert_eq!(df_def, "DEF-ACCT");
    }
}
