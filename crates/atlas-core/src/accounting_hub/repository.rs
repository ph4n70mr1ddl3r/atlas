//! Accounting Hub Repository
//!
//! PostgreSQL storage for external systems, accounting events, and mapping rules.

use atlas_shared::{
    ExternalSystem, AccountingEvent, TransactionMappingRule, AccountingHubDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Accounting Hub data storage
#[async_trait]
pub trait AccountingHubRepository: Send + Sync {
    // External Systems
    async fn create_external_system(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        system_type: &str, connection_config: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExternalSystem>;

    async fn get_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExternalSystem>>;
    async fn get_external_system_by_id(&self, id: Uuid) -> AtlasResult<Option<ExternalSystem>>;
    async fn list_external_systems(&self, org_id: Uuid) -> AtlasResult<Vec<ExternalSystem>>;
    async fn delete_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn update_system_stats(&self, id: Uuid, events_received: i32, events_processed: i32, events_failed: i32) -> AtlasResult<()>;

    // Accounting Events
    async fn create_accounting_event(
        &self,
        org_id: Uuid, event_number: &str, external_system_id: Uuid,
        external_system_code: Option<&str>, event_type: &str, event_class: &str,
        source_event_id: &str, payload: serde_json::Value,
        transaction_attributes: serde_json::Value, accounting_method_id: Option<Uuid>,
        status: &str, event_date: chrono::NaiveDate, accounting_date: Option<chrono::NaiveDate>,
        currency_code: &str, total_amount: Option<&str>, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingEvent>;

    async fn get_accounting_event(&self, id: Uuid) -> AtlasResult<Option<AccountingEvent>>;
    async fn get_event_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AccountingEvent>>;
    async fn list_accounting_events(
        &self, org_id: Uuid, status: Option<&str>,
        external_system_id: Option<Uuid>, event_type: Option<&str>,
    ) -> AtlasResult<Vec<AccountingEvent>>;
    async fn update_event_status(
        &self, id: Uuid, status: &str, error_message: Option<&str>,
        transaction_attributes: Option<serde_json::Value>,
        journal_entry_id: Option<Uuid>, processed_by: Option<Uuid>,
    ) -> AtlasResult<AccountingEvent>;

    // Mapping Rules
    async fn create_mapping_rule(
        &self,
        org_id: Uuid, external_system_id: Uuid, code: &str, name: &str,
        description: Option<&str>, event_type: &str, event_class: &str,
        priority: i32, conditions: serde_json::Value, field_mappings: serde_json::Value,
        accounting_method_id: Option<Uuid>, stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionMappingRule>;

    async fn get_mapping_rule(&self, org_id: Uuid, external_system_id: Uuid, code: &str) -> AtlasResult<Option<TransactionMappingRule>>;
    async fn list_mapping_rules(&self, org_id: Uuid, external_system_id: Option<Uuid>) -> AtlasResult<Vec<TransactionMappingRule>>;
    async fn list_active_mapping_rules(&self, org_id: Uuid, external_system_id: Uuid, event_type: &str) -> AtlasResult<Vec<TransactionMappingRule>>;
    async fn delete_mapping_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<AccountingHubDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresAccountingHubRepository {
    pool: PgPool,
}

impl PostgresAccountingHubRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

macro_rules! row_to_system {
    ($row:expr) => {{
        ExternalSystem {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            system_type: $row.get("system_type"),
            connection_config: $row.get("connection_config"),
            is_active: $row.get("is_active"),
            last_event_received: $row.get("last_event_received"),
            total_events_received: $row.try_get::<i32, _>("total_events_received").unwrap_or(0),
            total_events_processed: $row.try_get::<i32, _>("total_events_processed").unwrap_or(0),
            total_events_failed: $row.try_get::<i32, _>("total_events_failed").unwrap_or(0),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_event {
    ($row:expr) => {{
        AccountingEvent {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            event_number: $row.get("event_number"),
            external_system_id: $row.get("external_system_id"),
            external_system_code: $row.get("external_system_code"),
            event_type: $row.get("event_type"),
            event_class: $row.get("event_class"),
            source_event_id: $row.get("source_event_id"),
            payload: $row.get("payload"),
            transaction_attributes: $row.get("transaction_attributes"),
            accounting_method_id: $row.get("accounting_method_id"),
            status: $row.get("status"),
            error_message: $row.get("error_message"),
            journal_entry_id: $row.get("journal_entry_id"),
            event_date: $row.get("event_date"),
            accounting_date: $row.get("accounting_date"),
            currency_code: $row.get("currency_code"),
            total_amount: $row.try_get::<f64, _>("total_amount").ok().map(|v| format!("{:.2}", v)),
            description: $row.get("description"),
            processed_by: $row.get("processed_by"),
            processed_at: $row.get("processed_at"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_mapping {
    ($row:expr) => {{
        TransactionMappingRule {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            external_system_id: $row.get("external_system_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            event_type: $row.get("event_type"),
            event_class: $row.get("event_class"),
            priority: $row.get("priority"),
            conditions: $row.get("conditions"),
            field_mappings: $row.get("field_mappings"),
            accounting_method_id: $row.get("accounting_method_id"),
            stop_on_match: $row.get("stop_on_match"),
            is_active: $row.get("is_active"),
            effective_from: $row.get("effective_from"),
            effective_to: $row.get("effective_to"),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

#[async_trait]
impl AccountingHubRepository for PostgresAccountingHubRepository {
    async fn create_external_system(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        system_type: &str, connection_config: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExternalSystem> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.external_systems
                (organization_id, code, name, description, system_type, connection_config, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(system_type).bind(&connection_config).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_system!(row))
    }

    async fn get_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExternalSystem>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.external_systems WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_system!(r)))
    }

    async fn get_external_system_by_id(&self, id: Uuid) -> AtlasResult<Option<ExternalSystem>> {
        let row = sqlx::query("SELECT * FROM _atlas.external_systems WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_system!(r)))
    }

    async fn list_external_systems(&self, org_id: Uuid) -> AtlasResult<Vec<ExternalSystem>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.external_systems WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_system!(r)).collect())
    }

    async fn delete_external_system(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.external_systems SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_system_stats(&self, id: Uuid, events_received: i32, events_processed: i32, events_failed: i32) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.external_systems
            SET total_events_received = $1, total_events_processed = $2, total_events_failed = $3,
                last_event_received = now(), updated_at = now()
            WHERE id = $4"#,
        )
        .bind(events_received).bind(events_processed).bind(events_failed).bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_accounting_event(
        &self,
        org_id: Uuid, event_number: &str, external_system_id: Uuid,
        external_system_code: Option<&str>, event_type: &str, event_class: &str,
        source_event_id: &str, payload: serde_json::Value,
        transaction_attributes: serde_json::Value, accounting_method_id: Option<Uuid>,
        status: &str, event_date: chrono::NaiveDate, accounting_date: Option<chrono::NaiveDate>,
        currency_code: &str, total_amount: Option<&str>, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingEvent> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.accounting_events
                (organization_id, event_number, external_system_id, external_system_code,
                 event_type, event_class, source_event_id, payload, transaction_attributes,
                 accounting_method_id, status, event_date, accounting_date,
                 currency_code, total_amount, description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *"#,
        )
        .bind(org_id).bind(event_number).bind(external_system_id).bind(external_system_code)
        .bind(event_type).bind(event_class).bind(source_event_id).bind(&payload)
        .bind(&transaction_attributes).bind(accounting_method_id)
        .bind(status).bind(event_date).bind(accounting_date)
        .bind(currency_code)
        .bind(total_amount.and_then(|v| v.parse::<f64>().ok()))
        .bind(description).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_event!(row))
    }

    async fn get_accounting_event(&self, id: Uuid) -> AtlasResult<Option<AccountingEvent>> {
        let row = sqlx::query("SELECT * FROM _atlas.accounting_events WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_event!(r)))
    }

    async fn get_event_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AccountingEvent>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_events WHERE organization_id = $1 AND event_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_event!(r)))
    }

    async fn list_accounting_events(
        &self, org_id: Uuid, status: Option<&str>,
        external_system_id: Option<Uuid>, event_type: Option<&str>,
    ) -> AtlasResult<Vec<AccountingEvent>> {
        let mut query = String::from("SELECT * FROM _atlas.accounting_events WHERE organization_id = $1");
        let mut param_idx = 2;
        if status.is_some() { query.push_str(&format!(" AND status = ${}", param_idx)); param_idx += 1; }
        if external_system_id.is_some() { query.push_str(&format!(" AND external_system_id = ${}", param_idx)); param_idx += 1; }
        if event_type.is_some() { query.push_str(&format!(" AND event_type = ${}", param_idx)); }
        query.push_str(" ORDER BY event_date DESC, created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(e) = external_system_id { q = q.bind(e); }
        if let Some(e) = event_type { q = q.bind(e); }

        let rows = q.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_event!(r)).collect())
    }

    async fn update_event_status(
        &self, id: Uuid, status: &str, error_message: Option<&str>,
        transaction_attributes: Option<serde_json::Value>,
        journal_entry_id: Option<Uuid>, processed_by: Option<Uuid>,
    ) -> AtlasResult<AccountingEvent> {
        let row = sqlx::query(
            r#"UPDATE _atlas.accounting_events
            SET status = $1, error_message = $2, transaction_attributes = COALESCE($3, transaction_attributes),
                journal_entry_id = COALESCE($4, journal_entry_id),
                processed_by = COALESCE($5, processed_by),
                processed_at = CASE WHEN $1 IN ('accounted', 'posted', 'transferred') THEN now() ELSE processed_at END,
                updated_at = now()
            WHERE id = $6
            RETURNING *"#,
        )
        .bind(status).bind(error_message)
        .bind(transaction_attributes).bind(journal_entry_id).bind(processed_by).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_event!(row))
    }

    async fn create_mapping_rule(
        &self,
        org_id: Uuid, external_system_id: Uuid, code: &str, name: &str,
        description: Option<&str>, event_type: &str, event_class: &str,
        priority: i32, conditions: serde_json::Value, field_mappings: serde_json::Value,
        accounting_method_id: Option<Uuid>, stop_on_match: bool,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionMappingRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.transaction_mapping_rules
                (organization_id, external_system_id, code, name, description,
                 event_type, event_class, priority, conditions, field_mappings,
                 accounting_method_id, stop_on_match, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *"#,
        )
        .bind(org_id).bind(external_system_id).bind(code).bind(name).bind(description)
        .bind(event_type).bind(event_class).bind(priority).bind(&conditions).bind(&field_mappings)
        .bind(accounting_method_id).bind(stop_on_match).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_mapping!(row))
    }

    async fn get_mapping_rule(&self, org_id: Uuid, external_system_id: Uuid, code: &str) -> AtlasResult<Option<TransactionMappingRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.transaction_mapping_rules WHERE organization_id = $1 AND external_system_id = $2 AND code = $3 AND is_active = true"
        )
        .bind(org_id).bind(external_system_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_mapping!(r)))
    }

    async fn list_mapping_rules(&self, org_id: Uuid, external_system_id: Option<Uuid>) -> AtlasResult<Vec<TransactionMappingRule>> {
        let rows = if let Some(sid) = external_system_id {
            sqlx::query(
                "SELECT * FROM _atlas.transaction_mapping_rules WHERE organization_id = $1 AND external_system_id = $2 AND is_active = true ORDER BY priority"
            )
            .bind(org_id).bind(sid)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.transaction_mapping_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_mapping!(r)).collect())
    }

    async fn list_active_mapping_rules(&self, org_id: Uuid, external_system_id: Uuid, event_type: &str) -> AtlasResult<Vec<TransactionMappingRule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.transaction_mapping_rules
            WHERE organization_id = $1 AND external_system_id = $2 AND event_type = $3
              AND is_active = true
              AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
              AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
            ORDER BY priority"#,
        )
        .bind(org_id).bind(external_system_id).bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_mapping!(r)).collect())
    }

    async fn delete_mapping_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.transaction_mapping_rules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<AccountingHubDashboardSummary> {
        let systems = sqlx::query(
            "SELECT * FROM _atlas.external_systems WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let events = sqlx::query(
            "SELECT * FROM _atlas.accounting_events WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_systems = systems.len() as i32;
        let active_systems = systems.iter().filter(|s| s.get::<bool, _>("is_active")).count() as i32;
        let total_events = events.len() as i32;
        let received_events = events.iter().filter(|e| e.get::<String, _>("status") == "received").count() as i32;
        let accounted_events = events.iter().filter(|e| e.get::<String, _>("status") == "accounted").count() as i32;
        let posted_events = events.iter().filter(|e| e.get::<String, _>("status") == "posted" || e.get::<String, _>("status") == "transferred").count() as i32;
        let error_events = events.iter().filter(|e| e.get::<String, _>("status") == "error").count() as i32;

        let total_amount: f64 = events.iter()
            .filter_map(|e| e.try_get::<f64, _>("total_amount").ok())
            .sum();

        let by_system: serde_json::Value = systems.iter().map(|s| {
            let code: String = s.get("code");
            let count = events.iter().filter(|e| e.get::<Uuid, _>("external_system_id") == s.get::<Uuid, _>("id")).count();
            serde_json::json!({"system": code, "count": count})
        }).collect();

        let by_type: serde_json::Value = events.iter()
            .map(|e| e.get::<String, _>("event_type"))
            .fold(std::collections::HashMap::<String, i32>::new(), |mut acc, t| {
                *acc.entry(t).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(k, v)| serde_json::json!({"event_type": k, "count": v}))
            .collect();

        Ok(AccountingHubDashboardSummary {
            total_systems,
            active_systems,
            total_events,
            received_events,
            accounted_events,
            posted_events,
            error_events,
            total_amount_processed: format!("{:.2}", total_amount),
            events_by_system: by_system,
            events_by_type: by_type,
        })
    }
}
