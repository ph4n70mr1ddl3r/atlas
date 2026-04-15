//! Formula Engine Implementation

use crate::formula::{FormulaValue, EvaluationContext};
use atlas_shared::{AtlasError, AtlasResult};
use std::collections::HashMap;
use tracing::warn;

/// Built-in functions available in formulas
#[derive(Clone)]
pub struct FormulaFunction {
    #[allow(dead_code)] // Used for debugging
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    pub handler: fn(&[FormulaValue]) -> FormulaValue,
}

/// Formula engine for evaluating expressions
pub struct FormulaEngine {
    functions: HashMap<String, FormulaFunction>,
}

impl FormulaEngine {
    pub fn new() -> Self {
        let mut engine = Self { functions: HashMap::new() };
        engine.register_builtin_functions();
        engine
    }
    
    /// Register a custom function
    pub fn register_function(&mut self, name: &str, min_args: usize, max_args: usize, handler: fn(&[FormulaValue]) -> FormulaValue) {
        self.functions.insert(name.to_lowercase(), FormulaFunction {
            name: name.to_string(),
            min_args,
            max_args,
            handler,
        });
    }
    
    /// Evaluate a formula expression
    pub fn evaluate(&self, expression: &str, ctx: &EvaluationContext) -> AtlasResult<FormulaValue> {
        let expression = expression.trim();
        
        // Handle simple field reference
        if !expression.contains('(') && !expression.contains(' ') {
            return self.get_field_value(expression, ctx);
        }
        
        // Parse and evaluate
        match self.parse_and_evaluate(expression, ctx) {
            Ok(value) => Ok(value),
            Err(e) => {
                warn!("Formula evaluation error: {}", e);
                Err(AtlasError::Internal(format!("Formula error: {}", e)))
            }
        }
    }
    
    fn get_field_value(&self, field: &str, ctx: &EvaluationContext) -> AtlasResult<FormulaValue> {
        // Check if it's a related field (e.g., "lines.amount")
        if field.contains('.') {
            let parts: Vec<&str> = field.split('.').collect();
            if parts.len() == 2 {
                let _entity = parts[0];
                let _field_name = parts[1];
                
                if let Some(_records) = ctx.get_related(_entity) {
                    // Aggregate function needed
                    return Err(AtlasError::NotImplemented(
                        format!("Related field {} requires aggregation", field)
                    ));
                }
            }
        }
        
        // Get from record
        match ctx.get_field(field) {
            Some(value) => Ok(FormulaValue::from(value.clone())),
            None => Ok(FormulaValue::Null),
        }
    }
    
    fn parse_and_evaluate(&self, expression: &str, ctx: &EvaluationContext) -> Result<FormulaValue, String> {
        let expr = expression.trim();
        
        // Handle string literals
        if expr.starts_with('"') && expr.ends_with('"') {
            return Ok(FormulaValue::String(expr[1..expr.len()-1].to_string()));
        }
        
        // Handle function calls
        if let Some(paren_idx) = expr.find('(') {
            let func_name = expr[..paren_idx].trim().to_lowercase();
            let args_str = &expr[paren_idx+1..expr.len()-1];
            
            if let Some(func) = self.functions.get(&func_name) {
                let args = self.parse_arguments(args_str, ctx)?;
                
                if args.len() < func.min_args || args.len() > func.max_args {
                    return Err(format!("Function {} expects {} arguments", func_name, func.min_args));
                }
                
                return Ok((func.handler)(&args));
            }
            
            return Err(format!("Unknown function: {}", func_name));
        }
        
        // Handle operators
        if expr.contains('+') && !expr.contains('"') {
            if let Some(result) = self.evaluate_binary_op(expr, '+', ctx)? {
                return Ok(result);
            }
        }
        
        if expr.contains('-') {
            if let Some(result) = self.evaluate_binary_op(expr, '-', ctx)? {
                return Ok(result);
            }
        }
        
        if expr.contains('*') {
            if let Some(result) = self.evaluate_binary_op(expr, '*', ctx)? {
                return Ok(result);
            }
        }
        
        if expr.contains('/') {
            if let Some(result) = self.evaluate_binary_op(expr, '/', ctx)? {
                return Ok(result);
            }
        }
        
        // Handle comparison operators — check longer operators first to avoid
        // partial matches (e.g. ">=" must be tried before ">" ).
        for op in [">=", "<=", "==", "!=", ">", "<"].iter() {
            if expr.contains(op) {
                if let Some(result) = self.evaluate_compare(expr, op, ctx)? {
                    return Ok(result);
                }
            }
        }
        
        // Handle logical operators
        if expr.contains("AND") {
            if let Some(result) = self.evaluate_logical(expr, "AND", ctx)? {
                return Ok(result);
            }
        }
        
        if expr.contains("OR") {
            if let Some(result) = self.evaluate_logical(expr, "OR", ctx)? {
                return Ok(result);
            }
        }
        
        // Try to parse as number
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(FormulaValue::Number(num));
        }
        
        // Try to parse as boolean
        if expr == "true" {
            return Ok(FormulaValue::Boolean(true));
        }
        if expr == "false" {
            return Ok(FormulaValue::Boolean(false));
        }
        
        // Get field value
        Ok(self.get_field_value(expr, ctx).unwrap_or(FormulaValue::Null))
    }
    
    fn parse_arguments(&self, args_str: &str, ctx: &EvaluationContext) -> Result<Vec<FormulaValue>, String> {
        let mut args = vec![];
        let mut depth = 0;
        let mut current = String::new();
        
        for ch in args_str.chars() {
            match ch {
                '(' | '[' | '{' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    let arg = current.trim();
                    if !arg.is_empty() {
                        let value = self.parse_and_evaluate(arg, ctx)?;
                        args.push(value);
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }
        
        let arg = current.trim();
        if !arg.is_empty() {
            let value = self.parse_and_evaluate(arg, ctx)?;
            args.push(value);
        }
        
        Ok(args)
    }
    
    fn evaluate_binary_op(&self, expr: &str, op: char, ctx: &EvaluationContext) -> Result<Option<FormulaValue>, String> {
        // Find the operator (not inside quotes, parens, or identifiers)
        // For '-', skip matches that are preceded by an alphanumeric or '_' (e.g. field names like 'start_date' vs 'a - b')
        let mut depth = 0;
        let mut in_string = false;
        let mut op_idx = None;
        let chars: Vec<char> = expr.chars().collect();
        
        for i in 0..chars.len() {
            let ch = chars[i];
            match ch {
                '"' => in_string = !in_string,
                '(' | '[' | '{' if !in_string => depth += 1,
                ')' | ']' | '}' if !in_string => depth -= 1,
                c if c == op && depth == 0 && !in_string => {
                    // For '-', reject if preceded by alphanumeric or underscore
                    // (part of an identifier like 'start_date'), or if at start
                    if op == '-' {
                        if i == 0 {
                            continue; // unary minus at start, not binary
                        }
                        let prev = chars[i - 1];
                        if prev.is_alphanumeric() || prev == '_' {
                            continue; // part of identifier
                        }
                    }
                    // Must have whitespace or non-identifier chars around the operator
                    // to distinguish from field names
                    if op == '+' || op == '-' || op == '*' || op == '/' {
                        // Check that there's at least something on both sides
                        if i > 0 && i < chars.len() - 1 {
                            op_idx = Some(i);
                            break;
                        }
                    } else {
                        op_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(idx) = op_idx {
            // Convert char index back to byte index
            let byte_idx = chars[..idx].iter().collect::<String>().len();
            let left = expr[..byte_idx].trim();
            let right = expr[byte_idx + 1..].trim(); // operators are 1-byte ASCII
            
            let left_val = self.parse_and_evaluate(left, ctx)?;
            let right_val = self.parse_and_evaluate(right, ctx)?;
            
            let result = match op {
                '+' => add_values(&left_val, &right_val),
                '-' => subtract_values(&left_val, &right_val),
                '*' => multiply_values(&left_val, &right_val),
                '/' => divide_values(&left_val, &right_val)?,
                _ => return Ok(None),
            };
            
            return Ok(Some(result));
        }
        
        Ok(None)
    }
    
    fn evaluate_compare(&self, expr: &str, op: &str, ctx: &EvaluationContext) -> Result<Option<FormulaValue>, String> {
        // Find the FIRST occurrence of the operator (not inside quotes or parens)
        let mut depth = 0;
        let mut in_string = false;
        let mut op_idx = None;
        let chars: Vec<char> = expr.chars().collect();
        let op_len = op.len();

        for i in 0..chars.len().saturating_sub(op_len) {
            let ch = chars[i];
            match ch {
                '"' => in_string = !in_string,
                '(' | '[' | '{' if !in_string => depth += 1,
                ')' | ']' | '}' if !in_string => depth -= 1,
                _ if !in_string && depth == 0 => {
                    // Check if the slice starting at char index i matches op
                    let slice: String = chars[i..i + op_len].iter().collect();
                    if slice == op {
                        op_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(idx) = op_idx {
            // Convert char index to byte index for string slicing
            let byte_idx = chars[..idx].iter().collect::<String>().len();
            let byte_end = chars[..idx + op_len].iter().collect::<String>().len();
            let left = expr[..byte_idx].trim();
            let right = expr[byte_end..].trim();
            
            let left_val = self.parse_and_evaluate(left, ctx)?;
            let right_val = self.parse_and_evaluate(right, ctx)?;
            
            let result = match op {
                "==" => compare_values(&left_val, &right_val) == 0,
                "!=" => compare_values(&left_val, &right_val) != 0,
                ">" => compare_values(&left_val, &right_val) > 0,
                ">=" => compare_values(&left_val, &right_val) >= 0,
                "<" => compare_values(&left_val, &right_val) < 0,
                "<=" => compare_values(&left_val, &right_val) <= 0,
                _ => false,
            };
            
            return Ok(Some(FormulaValue::Boolean(result)));
        }
        
        Ok(None)
    }
    
    fn evaluate_logical(&self, expr: &str, op: &str, ctx: &EvaluationContext) -> Result<Option<FormulaValue>, String> {
        let parts: Vec<&str> = expr.split(op).collect();
        if parts.len() == 2 {
            let left = self.parse_and_evaluate(parts[0].trim(), ctx)?;
            let right = self.parse_and_evaluate(parts[1].trim(), ctx)?;
            
            let left_bool = to_bool(&left);
            let right_bool = to_bool(&right);
            
            let result = match op {
                "AND" => left_bool && right_bool,
                "OR" => left_bool || right_bool,
                _ => false,
            };
            
            return Ok(Some(FormulaValue::Boolean(result)));
        }
        
        Ok(None)
    }
    
    fn register_builtin_functions(&mut self) {
        // Math functions
        self.register_function("SUM", 1, 1, |args| {
            if let FormulaValue::Array(arr) = &args[0] {
                let sum: f64 = arr.iter()
                    .filter_map(|v| {
                        if let FormulaValue::Number(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum();
                FormulaValue::Number(sum)
            } else {
                FormulaValue::Null
            }
        });
        
        self.register_function("AVG", 1, 1, |args| {
            if let FormulaValue::Array(arr) = &args[0] {
                let sum: f64 = arr.iter()
                    .filter_map(|v| {
                        if let FormulaValue::Number(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum();
                let count = arr.iter().filter(|v| matches!(v, FormulaValue::Number(_))).count();
                if count > 0 {
                    FormulaValue::Number(sum / count as f64)
                } else {
                    FormulaValue::Null
                }
            } else {
                FormulaValue::Null
            }
        });
        
        self.register_function("COUNT", 1, 1, |args| {
            if let FormulaValue::Array(arr) = &args[0] {
                FormulaValue::Number(arr.len() as f64)
            } else {
                FormulaValue::Number(0.0)
            }
        });
        
        self.register_function("MIN", 1, usize::MAX, |args| {
            let nums: Vec<f64> = args.iter()
                .filter_map(|v| {
                    if let FormulaValue::Number(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            nums.iter().cloned().reduce(f64::min).map(FormulaValue::Number).unwrap_or(FormulaValue::Null)
        });
        
        self.register_function("MAX", 1, usize::MAX, |args| {
            let nums: Vec<f64> = args.iter()
                .filter_map(|v| {
                    if let FormulaValue::Number(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            nums.iter().cloned().reduce(f64::max).map(FormulaValue::Number).unwrap_or(FormulaValue::Null)
        });
        
        // String functions
        self.register_function("CONCAT", 1, usize::MAX, |args| {
            let s: String = args.iter()
                .map(value_to_string)
                .collect();
            FormulaValue::String(s)
        });
        
        self.register_function("UPPER", 1, 1, |args| {
            if let FormulaValue::String(s) = &args[0] {
                FormulaValue::String(s.to_uppercase())
            } else {
                FormulaValue::Null
            }
        });
        
        self.register_function("LOWER", 1, 1, |args| {
            if let FormulaValue::String(s) = &args[0] {
                FormulaValue::String(s.to_lowercase())
            } else {
                FormulaValue::Null
            }
        });
        
        self.register_function("TRIM", 1, 1, |args| {
            if let FormulaValue::String(s) = &args[0] {
                FormulaValue::String(s.trim().to_string())
            } else {
                FormulaValue::Null
            }
        });
        
        // Logic functions
        self.register_function("IF", 3, 3, |args| {
            let condition = to_bool(&args[0]);
            if condition { args[1].clone() } else { args[2].clone() }
        });
        
        self.register_function("COALESCE", 1, usize::MAX, |args| {
            for arg in args {
                if !matches!(arg, FormulaValue::Null) {
                    return arg.clone();
                }
            }
            FormulaValue::Null
        });
        
        // Date functions
        self.register_function("NOW", 0, 0, |_args| {
            FormulaValue::String(chrono::Utc::now().to_rfc3339())
        });
        
        self.register_function("TODAY", 0, 0, |_args| {
            FormulaValue::String(chrono::Utc::now().format("%Y-%m-%d").to_string())
        });
        
        // Aggregation for related records
        self.register_function("SUM_CHILDREN", 2, 2, |args| {
            if let (FormulaValue::String(_entity), FormulaValue::String(_field)) = (&args[0], &args[1]) {
                // This would need context to resolve - placeholder
                FormulaValue::Null
            } else {
                FormulaValue::Null
            }
        });
    }
}

fn add_values(a: &FormulaValue, b: &FormulaValue) -> FormulaValue {
    match (a, b) {
        (FormulaValue::Number(n1), FormulaValue::Number(n2)) => FormulaValue::Number(n1 + n2),
        (FormulaValue::String(s1), FormulaValue::String(s2)) => FormulaValue::String(format!("{}{}", s1, s2)),
        _ => FormulaValue::Null,
    }
}

fn subtract_values(a: &FormulaValue, b: &FormulaValue) -> FormulaValue {
    match (a, b) {
        (FormulaValue::Number(n1), FormulaValue::Number(n2)) => FormulaValue::Number(n1 - n2),
        _ => FormulaValue::Null,
    }
}

fn multiply_values(a: &FormulaValue, b: &FormulaValue) -> FormulaValue {
    match (a, b) {
        (FormulaValue::Number(n1), FormulaValue::Number(n2)) => FormulaValue::Number(n1 * n2),
        _ => FormulaValue::Null,
    }
}

fn divide_values(a: &FormulaValue, b: &FormulaValue) -> Result<FormulaValue, String> {
    match (a, b) {
        (FormulaValue::Number(n1), FormulaValue::Number(n2)) => {
            if *n2 == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(FormulaValue::Number(n1 / n2))
            }
        }
        _ => Ok(FormulaValue::Null),
    }
}

fn compare_values(a: &FormulaValue, b: &FormulaValue) -> i32 {
    match (a, b) {
        (FormulaValue::Null, FormulaValue::Null) => 0,
        (FormulaValue::Null, _) => -1,
        (_, FormulaValue::Null) => 1,
        (FormulaValue::Number(n1), FormulaValue::Number(n2)) => {
            if n1 < n2 { -1 } else if n1 > n2 { 1 } else { 0 }
        }
        (FormulaValue::String(s1), FormulaValue::String(s2)) => s1.cmp(s2) as i32,
        (FormulaValue::Boolean(b1), FormulaValue::Boolean(b2)) => {
            if b1 == b2 { 0 } else if *b1 { 1 } else { -1 }
        }
        _ => 0,
    }
}

fn to_bool(v: &FormulaValue) -> bool {
    match v {
        FormulaValue::Boolean(b) => *b,
        FormulaValue::Number(n) => *n != 0.0,
        FormulaValue::String(s) => !s.is_empty(),
        FormulaValue::Null => false,
        _ => false,
    }
}

fn value_to_string(v: &FormulaValue) -> String {
    match v {
        FormulaValue::String(s) => s.clone(),
        FormulaValue::Number(n) => n.to_string(),
        FormulaValue::Boolean(b) => b.to_string(),
        FormulaValue::Null => String::new(),
        FormulaValue::Array(arr) => format!("[{}]", arr.iter().map(value_to_string).collect::<Vec<_>>().join(", ")),
        FormulaValue::Object(obj) => format!("object with {} fields", obj.len()),
    }
}

impl Default for FormulaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_context() -> EvaluationContext {
        EvaluationContext::new(serde_json::json!({
            "name": "Test",
            "quantity": 10,
            "price": 25.50,
            "discount": 0.1,
            "active": true
        }))
    }
    
    #[test]
    fn test_simple_field() {
        let engine = FormulaEngine::new();
        let ctx = create_context();
        
        let result = engine.evaluate("name", &ctx).unwrap();
        assert!(matches!(result, FormulaValue::String(s) if s == "Test"));
    }
    
    #[test]
    fn test_arithmetic() {
        let engine = FormulaEngine::new();
        let ctx = create_context();
        
        let result = engine.evaluate("quantity * price", &ctx).unwrap();
        if let FormulaValue::Number(n) = result {
            assert!((n - 255.0).abs() < 0.01);
        } else {
            panic!("Expected number");
        }
    }
    
    #[test]
    fn test_function() {
        let engine = FormulaEngine::new();
        let ctx = EvaluationContext::new(serde_json::json!({
            "values": [10, 20, 30]
        }));
        
        let result = engine.evaluate("SUM(values)", &ctx).unwrap();
        if let FormulaValue::Number(n) = result {
            assert!((n - 60.0).abs() < 0.01);
        } else {
            panic!("Expected number");
        }
    }
    
    #[test]
    fn test_string_concat() {
        let engine = FormulaEngine::new();
        let ctx = create_context();
        
        let result = engine.evaluate("CONCAT(name, \" World\")", &ctx).unwrap();
        assert!(matches!(result, FormulaValue::String(s) if s == "Test World"));
    }
    
    #[test]
    fn test_if_function() {
        let engine = FormulaEngine::new();
        let ctx = create_context();
        
        let result = engine.evaluate("IF(active, \"Active\", \"Inactive\")", &ctx).unwrap();
        assert!(matches!(result, FormulaValue::String(s) if s == "Active"));
    }
    
    #[test]
    fn test_comparison() {
        let engine = FormulaEngine::new();
        let ctx = create_context();
        
        let result = engine.evaluate("quantity > 5", &ctx).unwrap();
        assert!(matches!(result, FormulaValue::Boolean(true)));
    }
}
