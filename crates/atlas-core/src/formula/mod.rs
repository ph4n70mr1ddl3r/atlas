//! Formula Engine
//! 
//! Expression evaluation for computed fields and formulas.

mod engine;
mod parser;

pub use engine::FormulaEngine;
pub use parser::FormulaParser;

/// Result of formula evaluation
#[derive(Debug, Clone)]
pub enum FormulaValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<FormulaValue>),
    Object(serde_json::Map<String, serde_json::Value>),
}

impl From<serde_json::Value> for FormulaValue {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => FormulaValue::Null,
            serde_json::Value::Bool(b) => FormulaValue::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    FormulaValue::Number(f)
                } else {
                    FormulaValue::Number(n.to_string().parse().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => FormulaValue::String(s),
            serde_json::Value::Array(arr) => FormulaValue::Array(arr.into_iter().map(FormulaValue::from).collect()),
            serde_json::Value::Object(obj) => FormulaValue::Object(obj),
        }
    }
}

impl From<FormulaValue> for serde_json::Value {
    fn from(v: FormulaValue) -> Self {
        match v {
            FormulaValue::Null => serde_json::Value::Null,
            FormulaValue::Boolean(b) => serde_json::Value::Bool(b),
            FormulaValue::Number(n) => serde_json::json!(n),
            FormulaValue::String(s) => serde_json::Value::String(s),
            FormulaValue::Array(arr) => serde_json::Value::Array(arr.into_iter().map(serde_json::Value::from).collect()),
            FormulaValue::Object(obj) => serde_json::Value::Object(obj),
        }
    }
}

/// Context for formula evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current record data
    pub record: serde_json::Value,
    /// Related records (for one-to-many)
    pub related: std::collections::HashMap<String, Vec<serde_json::Value>>,
    /// Current user
    pub user_id: Option<uuid::Uuid>,
    /// Current organization
    pub organization_id: Option<uuid::Uuid>,
    /// Custom variables
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

impl EvaluationContext {
    pub fn new(record: serde_json::Value) -> Self {
        Self {
            record,
            related: std::collections::HashMap::new(),
            user_id: None,
            organization_id: None,
            variables: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_related(mut self, entity: &str, records: Vec<serde_json::Value>) -> Self {
        self.related.insert(entity.to_string(), records);
        self
    }
    
    pub fn get_field(&self, name: &str) -> Option<&serde_json::Value> {
        self.record.get(name)
    }
    
    pub fn get_related(&self, entity: &str) -> Option<&Vec<serde_json::Value>> {
        self.related.get(entity)
    }
}
