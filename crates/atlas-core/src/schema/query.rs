//! Dynamic Query Builder
//! 
//! Builds SQL queries dynamically based on entity definitions.

use atlas_shared::{QueryFilter, FilterOperator, QueryRequest, SortOrder, SortDirection};
use atlas_shared::errors::{AtlasError, AtlasResult};
use std::collections::HashMap;

/// Sanitize a SQL identifier by rejecting characters that could allow injection.
/// Only allows alphanumeric and underscores. Dots are rejected to prevent
/// cross-schema references in dynamic queries.
fn sanitize_sql_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Dynamic SQL query builder for entities
pub struct DynamicQuery {
    table_name: String,
    select_fields: Vec<String>,
    filters: Vec<QueryFilter>,
    sort: Vec<SortOrder>,
    offset: Option<i64>,
    limit: Option<i64>,
    joins: HashMap<String, JoinDef>,
}

struct JoinDef {
    join_type: JoinType,
    table: String,
    on: String,
}

#[derive(Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

impl DynamicQuery {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: sanitize_sql_identifier(table_name),
            select_fields: vec!["*".to_string()],
            filters: vec![],
            sort: vec![],
            offset: None,
            limit: None,
            joins: HashMap::new(),
        }
    }
    
    pub fn select(mut self, fields: Vec<&str>) -> Self {
        self.select_fields = fields.into_iter()
            .map(|s| {
                let safe = sanitize_sql_identifier(s);
                if s == "*" { "*".to_string() } else { format!("\"{}\"", safe) }
            })
            .collect();
        self
    }
    
    pub fn filter(mut self, filter: QueryFilter) -> Self {
        self.filters.push(filter);
        self
    }
    
    pub fn filters(mut self, filters: Vec<QueryFilter>) -> Self {
        self.filters.extend(filters);
        self
    }
    
    pub fn sort(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort.push(SortOrder { field: field.to_string(), direction });
        self
    }
    
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
    
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn paginate(mut self, page: u64, page_size: u64) -> Self {
        self.offset = Some((page * page_size) as i64);
        self.limit = Some(page_size as i64);
        self
    }
    
    pub fn join(mut self, alias: &str, join_type: JoinType, table: &str, on: &str) -> Self {
        let safe_alias = sanitize_sql_identifier(alias);
        let safe_table = sanitize_sql_identifier(table);
        // Sanitize ON clause field references (allow only identifier.identifier patterns)
        let safe_on: String = on.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '=')
            .collect();
        self.joins.insert(safe_alias, JoinDef {
            join_type,
            table: safe_table,
            on: safe_on,
        });
        self
    }
    
    /// Build the SELECT query.
    ///
    /// **WARNING**: Filter values are interpolated into the SQL string via
    /// `value_to_sql`, which performs basic escaping.  For user-facing code
    /// prefer building parameterized queries (as the gateway handlers do)
    /// rather than using this method directly with untrusted input.
    ///
    /// Sort fields are sanitized through `sanitize_sql_identifier`.
    pub fn build_select(&self) -> String {
        let mut sql = String::from("SELECT ");
        
        // Select clause
        sql.push_str(&self.select_fields.join(", "));
        sql.push_str(" FROM ");
        sql.push_str(&self.table_name);
        
        // Joins
        for (alias, join) in &self.joins {
            let join_keyword = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
            };
            sql.push_str(&format!(" {} {} AS {} ON {}", join_keyword, join.table, alias, join.on));
        }
        
        // Where clause
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self.filters.iter()
                .map(|f| self.filter_to_sql(f))
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }
        
        // Order by (sanitize sort fields)
        if !self.sort.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self.sort.iter()
                .map(|s| {
                    let dir = match s.direction {
                        SortDirection::Asc => "ASC",
                        SortDirection::Desc => "DESC",
                    };
                    format!("\"{}\" {}", sanitize_sql_identifier(&s.field), dir)
                })
                .collect();
            sql.push_str(&orders.join(", "));
        }
        
        // Pagination
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        sql
    }
    
    /// Build the COUNT query.
    ///
    /// **WARNING**: Same caveat as `build_select` – filter values are
    /// string-interpolated.  Use parameterized queries for untrusted input.
    pub fn build_count(&self) -> String {
        let mut sql = String::from("SELECT COUNT(*) FROM ");
        sql.push_str(&self.table_name);
        
        // Joins for count
        for (alias, join) in &self.joins {
            let join_keyword = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
            };
            sql.push_str(&format!(" {} {} AS {} ON {}", join_keyword, join.table, alias, join.on));
        }
        
        // Where clause
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self.filters.iter()
                .map(|f| self.filter_to_sql(f))
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }
        
        sql
    }
    
    /// Build the INSERT query (parameterized).
    ///
    /// Returns `(sql, values)` where `values` should be bound positionally.
    pub fn build_insert(&self, data: &serde_json::Value) -> AtlasResult<(String, Vec<serde_json::Value>)> {
        if let Some(obj) = data.as_object() {
            let fields: Vec<String> = obj.keys().map(|k| format!("\"{}\"", sanitize_sql_identifier(k))).collect();
            let placeholders: Vec<String> = (1..=obj.len()).map(|i| format!("${}", i)).collect();
            let values: Vec<serde_json::Value> = obj.values().cloned().collect();
            
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                self.table_name,
                fields.join(", "),
                placeholders.join(", ")
            );
            
            Ok((sql, values))
        } else {
            Err(AtlasError::ValidationFailed("Expected object for insert".to_string()))
        }
    }
    
    /// Build the UPDATE query
    pub fn build_update(&self, id: &uuid::Uuid, data: &serde_json::Value) -> AtlasResult<(String, Vec<serde_json::Value>)> {
        if let Some(obj) = data.as_object() {
            let mut set_clauses = vec![];
            let mut values = vec![];
            
            for (i, (key, value)) in obj.iter().enumerate() {
                let safe_key = sanitize_sql_identifier(key);
                set_clauses.push(format!("\"{}\" = ${}", safe_key, i + 1));
                values.push(value.clone());
            }
            
            // Add updated_at
            set_clauses.push("updated_at = now()".to_string());
            
            let sql = format!(
                "UPDATE {} SET {} WHERE id = ${} RETURNING *",
                self.table_name,
                set_clauses.join(", "),
                values.len() + 1
            );
            
            values.push(serde_json::json!(id.to_string()));
            
            Ok((sql, values))
        } else {
            Err(AtlasError::ValidationFailed("Expected object for update".to_string()))
        }
    }
    
    /// Build the SOFT DELETE query
    pub fn build_soft_delete(&self, _id: &uuid::Uuid) -> String {
        format!(
            "UPDATE {} SET deleted_at = now() WHERE id = $1",
            self.table_name
        )
    }
    
    /// Build the HARD DELETE query
    pub fn build_hard_delete(&self, _id: &uuid::Uuid) -> String {
        format!(
            "DELETE FROM {} WHERE id = $1",
            self.table_name
        )
    }
    
    fn filter_to_sql(&self, filter: &QueryFilter) -> String {
        // Sanitize field name to prevent injection
        let field = format!("\"{}\"", sanitize_sql_identifier(&filter.field));
        let value = &filter.value;
        
        match filter.operator {
            FilterOperator::Eq => format!("{} = {}", field, self.value_to_sql(value)),
            FilterOperator::Ne => format!("{} != {}", field, self.value_to_sql(value)),
            FilterOperator::Gt => format!("{} > {}", field, self.value_to_sql(value)),
            FilterOperator::Gte => format!("{} >= {}", field, self.value_to_sql(value)),
            FilterOperator::Lt => format!("{} < {}", field, self.value_to_sql(value)),
            FilterOperator::Lte => format!("{} <= {}", field, self.value_to_sql(value)),
            FilterOperator::In => format!("{} = ANY({})", field, self.value_to_sql(value)),
            FilterOperator::NotIn => format!("NOT {} = ANY({})", field, self.value_to_sql(value)),
            FilterOperator::Contains => {
                let v = value.as_str().unwrap_or("");
                format!("{} LIKE {}", field, self.value_to_sql(&serde_json::json!(format!("%{}%", v))))
            }
            FilterOperator::StartsWith => {
                let v = value.as_str().unwrap_or("");
                format!("{} LIKE {}", field, self.value_to_sql(&serde_json::json!(format!("{}%", v))))
            }
            FilterOperator::EndsWith => {
                let v = value.as_str().unwrap_or("");
                format!("{} LIKE {}", field, self.value_to_sql(&serde_json::json!(format!("%{}", v))))
            }
            FilterOperator::IsNull => format!("{} IS NULL", field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", field),
            FilterOperator::Between => {
                if let Some(arr) = value.as_array() {
                    if arr.len() == 2 {
                        return format!("{} BETWEEN {} AND {}", field, 
                            self.value_to_sql(&arr[0]), self.value_to_sql(&arr[1]));
                    }
                }
                format!("{} BETWEEN {} AND {}", field, value, value)
            }
            _ => format!("-- unsupported operator: {:?}", filter.operator),
        }
    }
    
    /// **WARNING – for internal / trusted use only.**
    ///
    /// Interpolates values directly into the SQL string with basic escaping.
    /// Callers should prefer parameterized queries for untrusted input.
    fn value_to_sql(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                // Escape backslashes and single quotes
                let escaped = s.replace('\\', "\\\\").replace('\'', "''");
                format!("'{}'", escaped)
            }
            serde_json::Value::Array(arr) => format!("ARRAY[{}]", arr.iter()
                .map(|v| self.value_to_sql(v))
                .collect::<Vec<_>>()
                .join(", ")),
            serde_json::Value::Object(_) => {
                let escaped = serde_json::to_string(value).unwrap_or_default()
                    .replace('\\', "\\\\").replace('\'', "''");
                format!("'{}'", escaped)
            }
        }
    }
}

impl From<&QueryRequest> for DynamicQuery {
    fn from(req: &QueryRequest) -> Self {
        let mut query = DynamicQuery::new(&req.entity);
        
        if !req.fields.is_empty() {
            query = query.select(req.fields.iter().map(|s| s.as_str()).collect());
        }
        
        if !req.filters.is_empty() {
            query = query.filters(req.filters.clone());
        }
        
        if !req.sort_by.is_empty() {
            for sort in &req.sort_by {
                query = query.sort(&sort.field, sort.direction.clone());
            }
        }
        
        if let Some(offset) = req.offset {
            query = query.offset(offset);
        }
        
        if let Some(limit) = req.limit {
            query = query.limit(limit);
        }
        
        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_select_query() {
        let query = DynamicQuery::new("employees")
            .select(vec!["id", "name", "email"])
            .filter(QueryFilter {
                field: "status".to_string(),
                operator: FilterOperator::Eq,
                value: serde_json::json!("active"),
            })
            .sort("created_at", SortDirection::Desc)
            .limit(10);
        
        let sql = query.build_select();
        assert!(sql.contains("SELECT \"id\", \"name\", \"email\""));
        assert!(sql.contains("FROM employees"));
        assert!(sql.contains("WHERE \"status\" = 'active'"));
        assert!(sql.contains("ORDER BY \"created_at\" DESC"));
        assert!(sql.contains("LIMIT 10"));
    }
    
    #[test]
    fn test_count_query() {
        let query = DynamicQuery::new("orders")
            .filter(QueryFilter {
                field: "customer_id".to_string(),
                operator: FilterOperator::Eq,
                value: serde_json::json!("123"),
            });
        
        let count_sql = query.build_count();
        assert!(count_sql.contains("SELECT COUNT(*) FROM orders"));
        assert!(count_sql.contains("WHERE \"customer_id\" = '123'"));
    }
    
    #[test]
    fn test_pagination() {
        let query = DynamicQuery::new("products")
            .paginate(2, 25); // Page 2, 25 per page
        
        let sql = query.build_select();
        assert!(sql.contains("OFFSET 50"));
        assert!(sql.contains("LIMIT 25"));
    }
    
    #[test]
    fn test_filter_operators() {
        // IsNull
        let query = DynamicQuery::new("tasks")
            .filter(QueryFilter {
                field: "completed_at".to_string(),
                operator: FilterOperator::IsNull,
                value: serde_json::Value::Null,
            });
        
        let sql = query.build_select();
        assert!(sql.contains("\"completed_at\" IS NULL"));
        
        // Contains
        let query2 = DynamicQuery::new("products")
            .filter(QueryFilter {
                field: "name".to_string(),
                operator: FilterOperator::Contains,
                value: serde_json::json!("widget"),
            });
        
        let sql2 = query2.build_select();
        assert!(sql2.contains("\"name\" LIKE '%widget%'"));
        
        // Between
        let query3 = DynamicQuery::new("orders")
            .filter(QueryFilter {
                field: "amount".to_string(),
                operator: FilterOperator::Between,
                value: serde_json::json!([100, 500]),
            });
        
        let sql3 = query3.build_select();
        assert!(sql3.contains("\"amount\" BETWEEN 100 AND 500"));
    }
    
    #[test]
    fn test_insert() {
        let query = DynamicQuery::new("users");
        let data = serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30
        });
        
        let (sql, values) = query.build_insert(&data).unwrap();
        
        assert!(sql.contains("INSERT INTO users"));
        assert!(sql.contains("\"name\""));
        assert!(sql.contains("\"email\""));
        assert!(sql.contains("\"age\""));
        assert_eq!(values.len(), 3);
    }
}
