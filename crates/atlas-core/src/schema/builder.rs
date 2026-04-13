//! Schema Builder
//! 
//! Fluent API for building entity definitions programmatically.

use atlas_shared::{EntityDefinition, FieldDefinition, FieldType, IndexDefinition};
use atlas_shared::{WorkflowDefinition, StateDefinition, StateType, TransitionDefinition};
use atlas_shared::SecurityPolicy;
use uuid::Uuid;


/// Fluent builder for EntityDefinition
pub struct SchemaBuilder {
    name: String,
    label: String,
    plural_label: String,
    table_name: Option<String>,
    description: Option<String>,
    fields: Vec<FieldDefinition>,
    indexes: Vec<IndexDefinition>,
    workflow: Option<WorkflowDefinition>,
    security: Option<SecurityPolicy>,
    is_audit_enabled: bool,
    is_soft_delete: bool,
    icon: Option<String>,
    color: Option<String>,
}

impl SchemaBuilder {
    pub fn new(name: &str, label: &str) -> Self {
        let plural_label = if label.ends_with('s') {
            format!("{}es", label)
        } else {
            format!("{}s", label)
        };
        
        Self {
            name: name.to_string(),
            label: label.to_string(),
            plural_label,
            table_name: None,
            description: None,
            fields: vec![],
            indexes: vec![],
            workflow: None,
            security: None,
            is_audit_enabled: true,
            is_soft_delete: true,
            icon: None,
            color: None,
        }
    }
    
    pub fn plural_label(mut self, plural: &str) -> Self {
        self.plural_label = plural.to_string();
        self
    }
    
    pub fn table_name(mut self, name: &str) -> Self {
        self.table_name = Some(name.to_string());
        self
    }
    
    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
    
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
    
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }
    
    pub fn audit_disabled(mut self) -> Self {
        self.is_audit_enabled = false;
        self
    }
    
    pub fn hard_delete(mut self) -> Self {
        self.is_soft_delete = false;
        self
    }
    
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }
    
    pub fn add_index(mut self, index: IndexDefinition) -> Self {
        self.indexes.push(index);
        self
    }
    
    pub fn workflow(mut self, workflow: WorkflowDefinition) -> Self {
        self.workflow = Some(workflow);
        self
    }
    
    pub fn security(mut self, policy: SecurityPolicy) -> Self {
        self.security = Some(policy);
        self
    }
    
    /// Add a string field
    pub fn string(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::String {
            max_length: None,
            pattern: None,
        }));
        self
    }
    
    /// Add a required string field
    pub fn required_string(mut self, name: &str, label: &str) -> Self {
        let mut field = FieldDefinition::new(name, label, FieldType::String {
            max_length: None,
            pattern: None,
        });
        field.is_required = true;
        self.fields.push(field);
        self
    }
    
    /// Add an integer field
    pub fn integer(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Integer {
            min: None,
            max: None,
        }));
        self
    }
    
    /// Add a required integer field
    pub fn required_integer(mut self, name: &str, label: &str) -> Self {
        let mut field = FieldDefinition::new(name, label, FieldType::Integer {
            min: None,
            max: None,
        });
        field.is_required = true;
        self.fields.push(field);
        self
    }
    
    /// Add a decimal field
    pub fn decimal(mut self, name: &str, label: &str, precision: u8, scale: u8) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Decimal {
            precision,
            scale,
        }));
        self
    }
    
    /// Add a boolean field
    pub fn boolean(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Boolean));
        self
    }
    
    /// Add a date field
    pub fn date(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Date));
        self
    }
    
    /// Add a datetime field
    pub fn datetime(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::DateTime));
        self
    }
    
    /// Add an enum field
    pub fn enumeration(mut self, name: &str, label: &str, values: Vec<&str>) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Enum {
            values: values.into_iter().map(|s| s.to_string()).collect(),
        }));
        self
    }
    
    /// Add a reference field
    pub fn reference(mut self, name: &str, label: &str, entity: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Reference {
            entity: entity.to_string(),
            field: None,
        }));
        self
    }
    
    /// Add a currency field
    pub fn currency(mut self, name: &str, label: &str, code: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Currency {
            code: code.to_string(),
        }));
        self
    }
    
    /// Add an email field
    pub fn email(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Email));
        self
    }
    
    /// Add a phone field
    pub fn phone(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Phone));
        self
    }
    
    /// Add a rich text field
    pub fn rich_text(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::RichText));
        self
    }
    
    /// Add an address field
    pub fn address(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Address));
        self
    }
    
    /// Add a URL field
    pub fn url(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Url));
        self
    }
    
    /// Add a JSON field
    pub fn json(mut self, name: &str, label: &str) -> Self {
        self.fields.push(FieldDefinition::new(name, label, FieldType::Json));
        self
    }
    
    /// Add a boolean field with a default value
    pub fn boolean_default(mut self, name: &str, label: &str, default: bool) -> Self {
        let mut field = FieldDefinition::new(name, label, FieldType::Boolean);
        field.default_value = Some(serde_json::json!(default));
        self.fields.push(field);
        self
    }
    
    pub fn build(self) -> EntityDefinition {
        // Auto-generate display_order for fields
        let fields: Vec<_> = self.fields.into_iter()
            .enumerate()
            .map(|(i, mut f)| {
                f.display_order = i as i32;
                f
            })
            .collect();
        
        EntityDefinition {
            id: Some(Uuid::new_v4()),
            name: self.name,
            label: self.label,
            plural_label: self.plural_label,
            table_name: self.table_name,
            description: self.description,
            fields,
            indexes: self.indexes,
            workflow: self.workflow,
            security: self.security,
            is_audit_enabled: self.is_audit_enabled,
            is_soft_delete: self.is_soft_delete,
            icon: self.icon,
            color: self.color,
            metadata: serde_json::Value::Null,
        }
    }
}

/// Builder for workflow definitions
pub struct WorkflowBuilder {
    name: String,
    initial_state: String,
    states: Vec<StateDefinition>,
    transitions: Vec<TransitionDefinition>,
}

impl WorkflowBuilder {
    pub fn new(name: &str, initial_state: &str) -> Self {
        Self {
            name: name.to_string(),
            initial_state: initial_state.to_string(),
            states: vec![],
            transitions: vec![],
        }
    }
    
    pub fn add_state(mut self, name: &str, label: &str, state_type: StateType) -> Self {
        self.states.push(StateDefinition {
            name: name.to_string(),
            label: label.to_string(),
            state_type,
            entry_actions: vec![],
            exit_actions: vec![],
            metadata: serde_json::Value::Null,
        });
        self
    }
    
    pub fn initial_state(mut self, name: &str, label: &str) -> Self {
        self.states.push(StateDefinition {
            name: name.to_string(),
            label: label.to_string(),
            state_type: StateType::Initial,
            entry_actions: vec![],
            exit_actions: vec![],
            metadata: serde_json::Value::Null,
        });
        self.initial_state = name.to_string();
        self
    }
    
    pub fn working_state(mut self, name: &str, label: &str) -> Self {
        self.states.push(StateDefinition {
            name: name.to_string(),
            label: label.to_string(),
            state_type: StateType::Working,
            entry_actions: vec![],
            exit_actions: vec![],
            metadata: serde_json::Value::Null,
        });
        self
    }
    
    pub fn final_state(mut self, name: &str, label: &str) -> Self {
        self.states.push(StateDefinition {
            name: name.to_string(),
            label: label.to_string(),
            state_type: StateType::Final,
            entry_actions: vec![],
            exit_actions: vec![],
            metadata: serde_json::Value::Null,
        });
        self
    }
    
    pub fn transition(mut self, from: &str, to: &str, action: &str) -> Self {
        self.transitions.push(TransitionDefinition {
            name: format!("{}:{}", from, to),
            from_state: from.to_string(),
            to_state: to.to_string(),
            action: action.to_string(),
            action_label: None,
            guards: vec![],
            required_roles: vec![],
            entry_actions: vec![],
            metadata: serde_json::Value::Null,
        });
        self
    }
    
    pub fn build(self) -> WorkflowDefinition {
        WorkflowDefinition {
            id: Some(Uuid::new_v4()),
            name: self.name,
            initial_state: self.initial_state,
            states: self.states,
            transitions: self.transitions,
            is_active: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_builder() {
        let entity = SchemaBuilder::new("employees", "Employee")
            .plural_label("People")
            .table_name("hr_employees")
            .description("Employee records")
            .icon("user")
            .required_string("employee_number", "Employee Number")
            .required_string("first_name", "First Name")
            .required_string("last_name", "Last Name")
            .email("work_email", "Work Email")
            .date("hire_date", "Hire Date")
            .decimal("salary", "Salary", 12, 2)
            .enumeration("status", "Status", vec!["active", "inactive", "terminated"])
            .reference("department_id", "Department", "departments")
            .build();
        
        assert_eq!(entity.name, "employees");
        assert_eq!(entity.label, "Employee");
        assert_eq!(entity.plural_label, "People");
        assert_eq!(entity.fields.len(), 8);
        assert!(entity.fields[0].is_required); // employee_number
        assert!(entity.fields[1].is_required); // first_name
    }
    
    #[test]
    fn test_workflow_builder() {
        let workflow = WorkflowBuilder::new("purchase_order_approval", "draft")
            .initial_state("draft", "Draft")
            .working_state("submitted", "Submitted for Approval")
            .final_state("approved", "Approved")
            .final_state("rejected", "Rejected")
            .transition("draft", "submitted", "submit")
            .transition("submitted", "approved", "approve")
            .transition("submitted", "rejected", "reject")
            .build();
        
        assert_eq!(workflow.initial_state, "draft");
        assert_eq!(workflow.states.len(), 4);
        assert_eq!(workflow.transitions.len(), 3);
    }
}
