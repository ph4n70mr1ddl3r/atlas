//! Dynamic Query Builder
//!
//! Builds SQL queries dynamically based on entity definitions.

use atlas_shared::errors::{AtlasError, AtlasResult};
use atlas_shared::{FilterOperator, QueryFilter, QueryRequest, SortDirection, SortOrder};
use std::collections::HashMap;

/// Sanitize a SQL identifier by rejecting characters that could allow injection.
/// Only allows alphanumeric and underscores. Dots are rejected to prevent
/// cross-schema references in dynamic queries.
fn sanitize_sql_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Validate a SQL JOIN ON clause.
///
/// Only allows the pattern `identifier.identifier = identifier.identifier`
/// (optionally qualified with a table alias). Rejects anything containing
/// OR, AND, semicolons, or other SQL keywords that could widen the clause.
fn validate_join_on(on: &str) -> AtlasResult<()> {
    // Allow only: alphanumeric, underscore, dot, equals, and spaces
    if !on
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '=' || c == ' ')
    {
        return Err(AtlasError::ValidationFailed(
            "JOIN ON clause contains disallowed characters".into(),
        ));
    }
    // Must contain exactly one '='
    let parts: Vec<&str> = on.split('=').collect();
    if parts.len() != 2 {
        return Err(AtlasError::ValidationFailed(
            "JOIN ON clause must contain exactly one '='".into(),
        ));
    }
    // Each side must look like identifier.identifier or just identifier
    for part in &parts {
        let trimmed = part.trim();
        for segment in trimmed.split('.') {
            if segment.is_empty() || !segment.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(AtlasError::ValidationFailed(
                    "JOIN ON clause has invalid identifier".into(),
                ));
            }
        }
    }
    Ok(())
}

/// Escape SQL LIKE wildcard characters (`%`, `_`, `\`) so that a user-provided
/// string is matched literally inside a `LIKE` pattern.
fn escape_like_wildcards(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '%' => out.push_str("\\%"),
            '_' => out.push_str("\\_"),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out
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
    #[must_use]
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

    #[must_use]
    pub fn select(mut self, fields: Vec<&str>) -> Self {
        self.select_fields = fields
            .into_iter()
            .map(|s| {
                let safe = sanitize_sql_identifier(s);
                if s == "*" {
                    "*".to_string()
                } else {
                    format!("\"{safe}\"")
                }
            })
            .collect();
        self
    }

    #[must_use]
    pub fn filter(mut self, filter: QueryFilter) -> Self {
        self.filters.push(filter);
        self
    }

    #[must_use]
    pub fn filters(mut self, filters: Vec<QueryFilter>) -> Self {
        self.filters.extend(filters);
        self
    }

    #[must_use]
    pub fn sort(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort.push(SortOrder {
            field: field.to_string(),
            direction,
        });
        self
    }

    #[must_use]
    pub const fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    #[must_use]
    pub const fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub const fn paginate(mut self, page: u64, page_size: u64) -> Self {
        self.offset = Some((page * page_size) as i64);
        self.limit = Some(page_size as i64);
        self
    }

    /// Add a JOIN clause.
    ///
    /// The ON clause is validated to prevent injection — it must match the
    /// pattern `identifier.identifier = identifier.identifier`.
    pub fn join(
        mut self,
        alias: &str,
        join_type: JoinType,
        table: &str,
        on: &str,
    ) -> AtlasResult<Self> {
        validate_join_on(on)?;
        let safe_alias = sanitize_sql_identifier(alias);
        let safe_table = sanitize_sql_identifier(table);
        // Re-derive the safe ON from validated input
        let safe_on: String = on
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '=' || *c == ' ')
            .collect();
        self.joins.insert(
            safe_alias,
            JoinDef {
                join_type,
                table: safe_table,
                on: safe_on,
            },
        );
        Ok(self)
    }

    /// Build the SELECT query (parameterized).
    ///
    /// Returns `(sql, values)` where `values` is a list of JSON values that
    /// should be bound positionally to the query.  Filter values are **never**
    /// interpolated into the SQL string, eliminating SQL-injection risk.
    ///
    /// Sort fields and identifiers are sanitized through `sanitize_sql_identifier`.
    pub fn build_select(&self) -> (String, Vec<serde_json::Value>) {
        let mut sql = String::from("SELECT ");
        let mut values: Vec<serde_json::Value> = Vec::new();
        let mut param_idx = 0;

        // Select clause
        sql.push_str(&self.select_fields.join(", "));
        sql.push_str(" FROM ");
        sql.push_str(&format!("\"{}\"", self.table_name));

        // Joins
        for (alias, join) in &self.joins {
            let join_keyword = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
            };
            sql.push_str(&format!(
                " {} \"{}\" AS \"{}\" ON {}",
                join_keyword, join.table, alias, join.on
            ));
        }

        // Where clause (parameterized)
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .filters
                .iter()
                .map(|f| {
                    let (cond, mut vals) = self.filter_to_sql_param(f, &mut param_idx);
                    values.append(&mut vals);
                    cond
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // Order by (sanitize sort fields)
        if !self.sort.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self
                .sort
                .iter()
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
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        (sql, values)
    }

    /// Build the COUNT query (parameterized).
    ///
    /// Returns `(sql, values)` mirroring the parameterised approach of
    /// `build_select` so that filter values are bound, never interpolated.
    pub fn build_count(&self) -> (String, Vec<serde_json::Value>) {
        let mut sql = String::from("SELECT COUNT(*) FROM ");
        let mut values: Vec<serde_json::Value> = Vec::new();
        let mut param_idx = 0;

        sql.push_str(&format!("\"{}\"", self.table_name));

        // Joins for count
        for (alias, join) in &self.joins {
            let join_keyword = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
            };
            sql.push_str(&format!(
                " {} \"{}\" AS \"{}\" ON {}",
                join_keyword, join.table, alias, join.on
            ));
        }

        // Where clause (parameterized)
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .filters
                .iter()
                .map(|f| {
                    let (cond, mut vals) = self.filter_to_sql_param(f, &mut param_idx);
                    values.append(&mut vals);
                    cond
                })
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        (sql, values)
    }

    /// Build the INSERT query (parameterized).
    ///
    /// Returns `(sql, values)` where `values` should be bound positionally.
    pub fn build_insert(
        &self,
        data: &serde_json::Value,
    ) -> AtlasResult<(String, Vec<serde_json::Value>)> {
        if let Some(obj) = data.as_object() {
            let fields: Vec<String> = obj
                .keys()
                .map(|k| format!("\"{}\"", sanitize_sql_identifier(k)))
                .collect();
            let placeholders: Vec<String> = (1..=obj.len()).map(|i| format!("${i}")).collect();
            let values: Vec<serde_json::Value> = obj.values().cloned().collect();

            let sql = format!(
                "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
                self.table_name,
                fields.join(", "),
                placeholders.join(", ")
            );

            Ok((sql, values))
        } else {
            Err(AtlasError::ValidationFailed(
                "Expected object for insert".to_string(),
            ))
        }
    }

    /// Build the UPDATE query
    pub fn build_update(
        &self,
        id: &uuid::Uuid,
        data: &serde_json::Value,
    ) -> AtlasResult<(String, Vec<serde_json::Value>)> {
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
                "UPDATE \"{}\" SET {} WHERE id = ${} RETURNING *",
                self.table_name,
                set_clauses.join(", "),
                values.len() + 1
            );

            values.push(serde_json::json!(id.to_string()));

            Ok((sql, values))
        } else {
            Err(AtlasError::ValidationFailed(
                "Expected object for update".to_string(),
            ))
        }
    }

    /// Build the SOFT DELETE query
    ///
    /// The record ID must be bound as `$1` (UUID) by the caller.
    #[must_use]
    pub fn build_soft_delete(&self) -> String {
        format!(
            "UPDATE \"{}\" SET deleted_at = now() WHERE id = $1",
            self.table_name
        )
    }

    /// Build the HARD DELETE query
    ///
    /// The record ID must be bound as `$1` (UUID) by the caller.
    #[must_use]
    pub fn build_hard_delete(&self) -> String {
        format!("DELETE FROM \"{}\" WHERE id = $1", self.table_name)
    }

    /// Parameterized version of `filter_to_sql`.
    ///
    /// Returns `(sql_fragment, bind_values)`.  The caller is responsible for
    /// appending values to the overall bind list in the same order.
    fn filter_to_sql_param(
        &self,
        filter: &QueryFilter,
        param_idx: &mut usize,
    ) -> (String, Vec<serde_json::Value>) {
        // Sanitize field name to prevent injection
        let field = format!("\"{}\"", sanitize_sql_identifier(&filter.field));
        let value = &filter.value;

        match filter.operator {
            FilterOperator::Eq => {
                *param_idx += 1;
                (format!("{} = ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::Ne => {
                *param_idx += 1;
                (format!("{} != ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::Gt => {
                *param_idx += 1;
                (format!("{} > ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::Gte => {
                *param_idx += 1;
                (format!("{} >= ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::Lt => {
                *param_idx += 1;
                (format!("{} < ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::Lte => {
                *param_idx += 1;
                (format!("{} <= ${}", field, *param_idx), vec![value.clone()])
            }
            FilterOperator::In => {
                *param_idx += 1;
                (
                    format!("{} = ANY(${})", field, *param_idx),
                    vec![value.clone()],
                )
            }
            FilterOperator::NotIn => {
                *param_idx += 1;
                (
                    format!("NOT {} = ANY(${})", field, *param_idx),
                    vec![value.clone()],
                )
            }
            FilterOperator::Contains => {
                let v = value.as_str().unwrap_or("");
                let escaped = escape_like_wildcards(v);
                *param_idx += 1;
                (
                    format!("{} LIKE ${}", field, *param_idx),
                    vec![serde_json::json!(format!("%{}%", escaped))],
                )
            }
            FilterOperator::StartsWith => {
                let v = value.as_str().unwrap_or("");
                let escaped = escape_like_wildcards(v);
                *param_idx += 1;
                (
                    format!("{} LIKE ${}", field, *param_idx),
                    vec![serde_json::json!(format!("{}%", escaped))],
                )
            }
            FilterOperator::EndsWith => {
                let v = value.as_str().unwrap_or("");
                let escaped = escape_like_wildcards(v);
                *param_idx += 1;
                (
                    format!("{} LIKE ${}", field, *param_idx),
                    vec![serde_json::json!(format!("%{}", escaped))],
                )
            }
            FilterOperator::IsNull => (format!("{field} IS NULL"), vec![]),
            FilterOperator::IsNotNull => (format!("{field} IS NOT NULL"), vec![]),
            FilterOperator::Between => {
                if let Some(arr) = value.as_array() {
                    if arr.len() == 2 {
                        *param_idx += 1;
                        let p1 = *param_idx;
                        *param_idx += 1;
                        let p2 = *param_idx;
                        return (
                            format!("{} BETWEEN ${} AND ${}", field, p1, p2),
                            vec![arr[0].clone(), arr[1].clone()],
                        );
                    }
                }
                // Fallback: degenerate BETWEEN that matches nothing
                ("1=0".to_string(), vec![])
            }
            _ => ("1=1".to_string(), vec![]),
        }
    }
}

impl From<&QueryRequest> for DynamicQuery {
    fn from(req: &QueryRequest) -> Self {
        let mut query = Self::new(&req.entity);

        if !req.fields.is_empty() {
            query = query.select(req.fields.iter().map(std::string::String::as_str).collect());
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

        let (sql, values) = query.build_select();
        assert!(sql.contains("SELECT \"id\", \"name\", \"email\""));
        assert!(sql.contains("FROM \"employees\""));
        // Parameterized – no inline 'active'
        assert!(sql.contains("WHERE \"status\" = $1"));
        assert!(sql.contains("ORDER BY \"created_at\" DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert_eq!(values, vec![serde_json::json!("active")]);
    }

    #[test]
    fn test_count_query() {
        let query = DynamicQuery::new("orders").filter(QueryFilter {
            field: "customer_id".to_string(),
            operator: FilterOperator::Eq,
            value: serde_json::json!("123"),
        });

        let (count_sql, values) = query.build_count();
        assert!(count_sql.contains("SELECT COUNT(*) FROM \"orders\""));
        // Parameterized – no inline '123'
        assert!(count_sql.contains("WHERE \"customer_id\" = $1"));
        assert_eq!(values, vec![serde_json::json!("123")]);
    }

    #[test]
    fn test_pagination() {
        let query = DynamicQuery::new("products").paginate(2, 25); // Page 2, 25 per page

        let (sql, _values) = query.build_select();
        assert!(sql.contains("OFFSET 50"));
        assert!(sql.contains("LIMIT 25"));
    }

    #[test]
    fn test_filter_operators() {
        // IsNull
        let query = DynamicQuery::new("tasks").filter(QueryFilter {
            field: "completed_at".to_string(),
            operator: FilterOperator::IsNull,
            value: serde_json::Value::Null,
        });

        let (sql, values) = query.build_select();
        assert!(sql.contains("\"completed_at\" IS NULL"));
        assert!(values.is_empty());

        // Contains
        let query2 = DynamicQuery::new("products").filter(QueryFilter {
            field: "name".to_string(),
            operator: FilterOperator::Contains,
            value: serde_json::json!("widget"),
        });

        let (sql2, values2) = query2.build_select();
        // Parameterized LIKE – pattern in bind value, not SQL string
        assert!(sql2.contains("\"name\" LIKE $1"));
        assert_eq!(values2, vec![serde_json::json!("%widget%")]);

        // Between
        let query3 = DynamicQuery::new("orders").filter(QueryFilter {
            field: "amount".to_string(),
            operator: FilterOperator::Between,
            value: serde_json::json!([100, 500]),
        });

        let (sql3, values3) = query3.build_select();
        assert!(sql3.contains("\"amount\" BETWEEN $1 AND $2"));
        assert_eq!(
            values3,
            vec![serde_json::json!(100), serde_json::json!(500)]
        );
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

        assert!(sql.contains("INSERT INTO \"users\""));
        assert!(sql.contains("\"name\""));
        assert!(sql.contains("\"email\""));
        assert!(sql.contains("\"age\""));
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_like_wildcard_escaping() {
        // Ensure user-provided LIKE wildcards are escaped in the bind value
        let query = DynamicQuery::new("products").filter(QueryFilter {
            field: "name".to_string(),
            operator: FilterOperator::Contains,
            value: serde_json::json!("100%_real"),
        });

        let (_sql, values) = query.build_select();
        // The % and _ inside the user value should be escaped in the bind param
        assert_eq!(values.len(), 1);
        let pattern = values[0].as_str().unwrap();
        assert!(pattern.contains("\\%"));
        assert!(pattern.contains("\\_"));
    }

    #[test]
    fn test_escape_like_wildcards_function() {
        assert_eq!(escape_like_wildcards("normal"), "normal");
        assert_eq!(escape_like_wildcards("100%"), "100\\%");
        assert_eq!(escape_like_wildcards("a_b"), "a\\_b");
        assert_eq!(escape_like_wildcards("a\\b"), "a\\\\b");
        assert_eq!(escape_like_wildcards("%_\\"), "\\%\\_\\\\");
    }

    #[test]
    fn test_sql_injection_in_identifier() {
        let malicious = "users; DROP TABLE users--";
        let safe = sanitize_sql_identifier(malicious);
        assert!(!safe.contains(';'));
        assert!(!safe.contains('-'));
        assert!(!safe.contains(' '));
    }

    #[test]
    fn test_sql_injection_in_filter_value() {
        // With parameterized queries, injection values end up as bind params
        let query = DynamicQuery::new("users").filter(QueryFilter {
            field: "name".to_string(),
            operator: FilterOperator::Eq,
            value: serde_json::json!("Robert'; DROP TABLE students;--"),
        });
        let (sql, values) = query.build_select();
        // The SQL should use $1, not inline the value
        assert!(sql.contains("WHERE \"name\" = $1"));
        // The malicious value is safely in the bind parameters
        assert_eq!(
            values[0],
            serde_json::json!("Robert'; DROP TABLE students;--")
        );
        // Crucially: no raw SQL injection in the query string
        assert!(!sql.contains("DROP TABLE"));
    }

    #[test]
    fn test_sql_injection_in_field_name() {
        let query = DynamicQuery::new("users").filter(QueryFilter {
            field: "id; DROP TABLE users--".to_string(),
            operator: FilterOperator::Eq,
            value: serde_json::json!(1),
        });
        let (sql, _values) = query.build_select();
        // The field name should be sanitized - no semicolons, no dashes, no spaces
        assert!(!sql.contains(';'));
        assert!(!sql.contains("--"));
        // The malicious SQL keywords should not appear as executable SQL
        assert!(!sql.contains("; DROP"));
        assert!(!sql.contains("--"));
    }

    #[test]
    fn test_validate_join_on_rejects_malicious() {
        assert!(validate_join_on("a.id = b.id OR 1=1").is_err());
        assert!(validate_join_on("a.id = b.id; DROP TABLE users").is_err());
        assert!(validate_join_on("a.id = b.id").is_ok());
        assert!(validate_join_on("a.id = b.user_id").is_ok());
        assert!(validate_join_on("a.id == b.id").is_err());
    }
}
