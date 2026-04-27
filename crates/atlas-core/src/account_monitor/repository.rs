//! Account Monitor Repository
//!
//! PostgreSQL storage for account groups, members, balance snapshots,
//! and saved balance inquiries.

use atlas_shared::{
    AccountGroup, AccountGroupMember, BalanceSnapshot, SavedBalanceInquiry,
    AccountMonitorSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Account Monitor data storage
#[async_trait]
pub trait AccountMonitorRepository: Send + Sync {
    // Account Groups
    async fn create_account_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        owner_id: Option<Uuid>,
        is_shared: bool,
        threshold_warning_pct: Option<&str>,
        threshold_critical_pct: Option<&str>,
        comparison_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountGroup>;

    async fn get_account_group(&self, id: Uuid) -> AtlasResult<Option<AccountGroup>>;
    async fn get_account_group_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountGroup>>;
    async fn list_account_groups(&self, org_id: Uuid, owner_id: Option<Uuid>) -> AtlasResult<Vec<AccountGroup>>;
    async fn delete_account_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Group Members
    async fn add_group_member(
        &self,
        group_id: Uuid,
        account_segment: &str,
        account_label: Option<&str>,
        display_order: i32,
        include_children: bool,
    ) -> AtlasResult<AccountGroupMember>;

    async fn remove_group_member(&self, id: Uuid) -> AtlasResult<()>;
    async fn list_group_members(&self, group_id: Uuid) -> AtlasResult<Vec<AccountGroupMember>>;

    // Balance Snapshots
    #[allow(clippy::too_many_arguments)]
    async fn create_balance_snapshot(
        &self,
        org_id: Uuid,
        group_id: Uuid,
        member_id: Option<Uuid>,
        account_segment: &str,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        fiscal_year: i32,
        period_number: i32,
        beginning_balance: &str,
        total_debits: &str,
        total_credits: &str,
        net_activity: &str,
        ending_balance: &str,
        journal_entry_count: i32,
        comparison_balance: Option<&str>,
        comparison_period_name: Option<&str>,
        variance_amount: Option<&str>,
        variance_pct: Option<&str>,
        alert_status: &str,
    ) -> AtlasResult<BalanceSnapshot>;

    async fn get_group_snapshots(
        &self,
        group_id: Uuid,
        snapshot_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<BalanceSnapshot>>;

    async fn get_alert_snapshots(&self, org_id: Uuid) -> AtlasResult<Vec<BalanceSnapshot>>;
    async fn delete_snapshot(&self, id: Uuid) -> AtlasResult<()>;

    // Saved Inquiries
    #[allow(clippy::too_many_arguments)]
    async fn create_saved_inquiry(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        account_segments: serde_json::Value,
        period_from: &str,
        period_to: &str,
        currency_code: &str,
        amount_type: &str,
        include_zero_balances: bool,
        comparison_enabled: bool,
        comparison_type: Option<&str>,
        sort_by: &str,
        sort_direction: &str,
        is_shared: bool,
    ) -> AtlasResult<SavedBalanceInquiry>;

    async fn get_saved_inquiry(&self, id: Uuid) -> AtlasResult<Option<SavedBalanceInquiry>>;
    async fn list_saved_inquiries(&self, org_id: Uuid, user_id: Option<Uuid>) -> AtlasResult<Vec<SavedBalanceInquiry>>;
    async fn delete_saved_inquiry(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard Summary
    async fn get_monitor_summary(&self, org_id: Uuid) -> AtlasResult<AccountMonitorSummary>;
}

/// PostgreSQL implementation
pub struct PostgresAccountMonitorRepository {
    pool: PgPool,
}

impl PostgresAccountMonitorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_member(row: &sqlx::postgres::PgRow) -> AccountGroupMember {
    AccountGroupMember {
        id: row.try_get("id").unwrap_or_default(),
        group_id: row.try_get("group_id").unwrap_or_default(),
        account_segment: row.try_get("account_segment").unwrap_or_default(),
        account_label: row.try_get("account_label").unwrap_or_default(),
        display_order: row.try_get("display_order").unwrap_or(0),
        include_children: row.try_get("include_children").unwrap_or(true),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_snapshot(row: &sqlx::postgres::PgRow) -> BalanceSnapshot {
    BalanceSnapshot {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        account_group_id: row.try_get("account_group_id").unwrap_or_default(),
        member_id: row.try_get("member_id").unwrap_or_default(),
        account_segment: row.try_get("account_segment").unwrap_or_default(),
        period_name: row.try_get("period_name").unwrap_or_default(),
        period_start: row.try_get("period_start").unwrap_or_default(),
        period_end: row.try_get("period_end").unwrap_or_default(),
        fiscal_year: row.try_get("fiscal_year").unwrap_or_default(),
        period_number: row.try_get("period_number").unwrap_or_default(),
        beginning_balance: row.try_get::<String, _>("beginning_balance").unwrap_or_default(),
        total_debits: row.try_get::<String, _>("total_debits").unwrap_or_default(),
        total_credits: row.try_get::<String, _>("total_credits").unwrap_or_default(),
        net_activity: row.try_get::<String, _>("net_activity").unwrap_or_default(),
        ending_balance: row.try_get::<String, _>("ending_balance").unwrap_or_default(),
        journal_entry_count: row.try_get("journal_entry_count").unwrap_or(0),
        comparison_balance: row.try_get("comparison_balance").unwrap_or_default(),
        comparison_period_name: row.try_get("comparison_period_name").unwrap_or_default(),
        variance_amount: row.try_get("variance_amount").unwrap_or_default(),
        variance_pct: row.try_get("variance_pct").unwrap_or_default(),
        alert_status: row.try_get("alert_status").unwrap_or_else(|_| "none".to_string()),
        snapshot_date: row.try_get("snapshot_date").unwrap_or(chrono::Utc::now().date_naive()),
        computed_at: row.try_get("computed_at").unwrap_or(chrono::Utc::now()),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
    }
}

fn row_to_inquiry(row: &sqlx::postgres::PgRow) -> SavedBalanceInquiry {
    SavedBalanceInquiry {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        user_id: row.try_get("user_id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        account_segments: row.try_get("account_segments").unwrap_or(serde_json::json!([])),
        period_from: row.try_get("period_from").unwrap_or_default(),
        period_to: row.try_get("period_to").unwrap_or_default(),
        currency_code: row.try_get("currency_code").unwrap_or_else(|_| "USD".to_string()),
        amount_type: row.try_get("amount_type").unwrap_or_else(|_| "ending_balance".to_string()),
        include_zero_balances: row.try_get("include_zero_balances").unwrap_or(false),
        comparison_enabled: row.try_get("comparison_enabled").unwrap_or(false),
        comparison_type: row.try_get("comparison_type").unwrap_or_default(),
        sort_by: row.try_get("sort_by").unwrap_or_else(|_| "account_segment".to_string()),
        sort_direction: row.try_get("sort_direction").unwrap_or_else(|_| "asc".to_string()),
        is_shared: row.try_get("is_shared").unwrap_or(false),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}

#[async_trait]
impl AccountMonitorRepository for PostgresAccountMonitorRepository {
    async fn create_account_group(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        owner_id: Option<Uuid>,
        is_shared: bool,
        threshold_warning_pct: Option<&str>,
        threshold_critical_pct: Option<&str>,
        comparison_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountGroup> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.account_groups
                (organization_id, code, name, description, owner_id, is_shared,
                 threshold_warning_pct, threshold_critical_pct, comparison_type,
                 status, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::NUMERIC, $8::NUMERIC, $9, 'active', '{}'::jsonb, $10)
            RETURNING *"#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(owner_id).bind(is_shared)
        .bind(threshold_warning_pct).bind(threshold_critical_pct)
        .bind(comparison_type).bind(created_by)
        .fetch_one(&self.pool).await?;

        let group = AccountGroup {
            id: row.try_get("id").unwrap_or_default(),
            organization_id: row.try_get("organization_id").unwrap_or_default(),
            code: row.try_get("code").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            description: row.try_get("description").unwrap_or_default(),
            owner_id: row.try_get("owner_id").unwrap_or_default(),
            is_shared: row.try_get("is_shared").unwrap_or(false),
            threshold_warning_pct: row.try_get("threshold_warning_pct").ok(),
            threshold_critical_pct: row.try_get("threshold_critical_pct").ok(),
            comparison_type: row.try_get("comparison_type").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            members: vec![],
            created_by: row.try_get("created_by").unwrap_or_default(),
            created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
        };
        Ok(group)
    }

    async fn get_account_group(&self, id: Uuid) -> AtlasResult<Option<AccountGroup>> {
        let row = sqlx::query("SELECT * FROM _atlas.account_groups WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        match row {
            Some(r) => {
                let mut group = row_to_group_bare(&r);
                group.members = self.list_group_members(id).await.unwrap_or_default();
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn get_account_group_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountGroup>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.account_groups WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await?;
        match row {
            Some(r) => {
                let gid: Uuid = r.try_get("id").unwrap_or_default();
                let mut group = row_to_group_bare(&r);
                group.members = self.list_group_members(gid).await.unwrap_or_default();
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn list_account_groups(&self, org_id: Uuid, owner_id: Option<Uuid>) -> AtlasResult<Vec<AccountGroup>> {
        let rows = if let Some(oid) = owner_id {
            sqlx::query(
                "SELECT * FROM _atlas.account_groups WHERE organization_id = $1 AND (owner_id = $2 OR is_shared = true) ORDER BY created_at DESC"
            )
            .bind(org_id).bind(oid)
            .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.account_groups WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await?
        };

        let mut groups = Vec::new();
        for r in &rows {
            let mut g = row_to_group_bare(r);
            let gid: Uuid = r.try_get("id").unwrap_or_default();
            g.members = self.list_group_members(gid).await.unwrap_or_default();
            groups.push(g);
        }
        Ok(groups)
    }

    async fn delete_account_group(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.account_groups WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound(format!("Account group '{}' not found", code)));
        }
        Ok(())
    }

    async fn add_group_member(
        &self,
        group_id: Uuid,
        account_segment: &str,
        account_label: Option<&str>,
        display_order: i32,
        include_children: bool,
    ) -> AtlasResult<AccountGroupMember> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.account_group_members
                (group_id, account_segment, account_label, display_order, include_children, metadata)
            VALUES ($1, $2, $3, $4, $5, '{}'::jsonb)
            RETURNING *"#
        )
        .bind(group_id).bind(account_segment).bind(account_label)
        .bind(display_order).bind(include_children)
        .fetch_one(&self.pool).await?;
        Ok(row_to_member(&row))
    }

    async fn remove_group_member(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.account_group_members WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Member not found".to_string()));
        }
        Ok(())
    }

    async fn list_group_members(&self, group_id: Uuid) -> AtlasResult<Vec<AccountGroupMember>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.account_group_members WHERE group_id = $1 ORDER BY display_order, account_segment"
        )
        .bind(group_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_member).collect())
    }

    async fn create_balance_snapshot(
        &self,
        org_id: Uuid,
        group_id: Uuid,
        member_id: Option<Uuid>,
        account_segment: &str,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        fiscal_year: i32,
        period_number: i32,
        beginning_balance: &str,
        total_debits: &str,
        total_credits: &str,
        net_activity: &str,
        ending_balance: &str,
        journal_entry_count: i32,
        comparison_balance: Option<&str>,
        comparison_period_name: Option<&str>,
        variance_amount: Option<&str>,
        variance_pct: Option<&str>,
        alert_status: &str,
    ) -> AtlasResult<BalanceSnapshot> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.balance_snapshots
                (organization_id, account_group_id, member_id, account_segment,
                 period_name, period_start, period_end, fiscal_year, period_number,
                 beginning_balance, total_debits, total_credits, net_activity,
                 ending_balance, journal_entry_count,
                 comparison_balance, comparison_period_name,
                 variance_amount, variance_pct, alert_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::NUMERIC, $11::NUMERIC, $12::NUMERIC, $13::NUMERIC,
                    $14::NUMERIC, $15,
                    $16::NUMERIC, $17,
                    $18::NUMERIC, $19::NUMERIC, $20)
            RETURNING *"#
        )
        .bind(org_id).bind(group_id).bind(member_id).bind(account_segment)
        .bind(period_name).bind(period_start).bind(period_end)
        .bind(fiscal_year).bind(period_number)
        .bind(beginning_balance).bind(total_debits).bind(total_credits)
        .bind(net_activity).bind(ending_balance).bind(journal_entry_count)
        .bind(comparison_balance).bind(comparison_period_name)
        .bind(variance_amount).bind(variance_pct).bind(alert_status)
        .fetch_one(&self.pool).await?;
        Ok(row_to_snapshot(&row))
    }

    async fn get_group_snapshots(
        &self,
        group_id: Uuid,
        snapshot_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> AtlasResult<Vec<BalanceSnapshot>> {
        let rows = if let Some(sd) = snapshot_date {
            sqlx::query(
                "SELECT * FROM _atlas.balance_snapshots WHERE account_group_id = $1 AND snapshot_date = $2 ORDER BY account_segment, period_name LIMIT $3 OFFSET $4"
            )
            .bind(group_id).bind(sd).bind(limit).bind(offset)
            .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.balance_snapshots WHERE account_group_id = $1 ORDER BY snapshot_date DESC, account_segment LIMIT $2 OFFSET $3"
            )
            .bind(group_id).bind(limit).bind(offset)
            .fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_snapshot).collect())
    }

    async fn get_alert_snapshots(&self, org_id: Uuid) -> AtlasResult<Vec<BalanceSnapshot>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.balance_snapshots WHERE organization_id = $1 AND alert_status != 'none' ORDER BY computed_at DESC LIMIT 50"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_snapshot).collect())
    }

    async fn delete_snapshot(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.balance_snapshots WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Snapshot not found".to_string()));
        }
        Ok(())
    }

    async fn create_saved_inquiry(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        name: &str,
        description: Option<&str>,
        account_segments: serde_json::Value,
        period_from: &str,
        period_to: &str,
        currency_code: &str,
        amount_type: &str,
        include_zero_balances: bool,
        comparison_enabled: bool,
        comparison_type: Option<&str>,
        sort_by: &str,
        sort_direction: &str,
        is_shared: bool,
    ) -> AtlasResult<SavedBalanceInquiry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.saved_balance_inquiries
                (organization_id, user_id, name, description, account_segments,
                 period_from, period_to, currency_code, amount_type,
                 include_zero_balances, comparison_enabled, comparison_type,
                 sort_by, sort_direction, is_shared, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, '{}'::jsonb)
            RETURNING *"#
        )
        .bind(org_id).bind(user_id).bind(name).bind(description)
        .bind(&account_segments).bind(period_from).bind(period_to)
        .bind(currency_code).bind(amount_type)
        .bind(include_zero_balances).bind(comparison_enabled).bind(comparison_type)
        .bind(sort_by).bind(sort_direction).bind(is_shared)
        .fetch_one(&self.pool).await?;
        Ok(row_to_inquiry(&row))
    }

    async fn get_saved_inquiry(&self, id: Uuid) -> AtlasResult<Option<SavedBalanceInquiry>> {
        let row = sqlx::query("SELECT * FROM _atlas.saved_balance_inquiries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(row_to_inquiry))
    }

    async fn list_saved_inquiries(&self, org_id: Uuid, user_id: Option<Uuid>) -> AtlasResult<Vec<SavedBalanceInquiry>> {
        let rows = if let Some(uid) = user_id {
            sqlx::query(
                "SELECT * FROM _atlas.saved_balance_inquiries WHERE organization_id = $1 AND (user_id = $2 OR is_shared = true) ORDER BY created_at DESC"
            )
            .bind(org_id).bind(uid)
            .fetch_all(&self.pool).await?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.saved_balance_inquiries WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await?
        };
        Ok(rows.iter().map(row_to_inquiry).collect())
    }

    async fn delete_saved_inquiry(&self, id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query("DELETE FROM _atlas.saved_balance_inquiries WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Saved inquiry not found".to_string()));
        }
        Ok(())
    }

    async fn get_monitor_summary(&self, org_id: Uuid) -> AtlasResult<AccountMonitorSummary> {
        let groups = self.list_account_groups(org_id, None).await?;
        let total_groups = groups.len() as i32;
        let active_groups = groups.iter().filter(|g| g.status == "active").count() as i32;
        let total_members: i32 = groups.iter().map(|g| g.members.len() as i32).sum();

        // Count alerts
        let alert_rows = sqlx::query(
            "SELECT alert_status, COUNT(*) as cnt FROM _atlas.balance_snapshots WHERE organization_id = $1 GROUP BY alert_status"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut warning = 0i32;
        let mut critical = 0i32;
        let mut on_track = 0i32;
        for r in &alert_rows {
            let status: String = r.try_get("alert_status").unwrap_or_default();
            let cnt: i64 = r.try_get("cnt").unwrap_or(0);
            match status.as_str() {
                "warning" => warning = cnt as i32,
                "critical" => critical = cnt as i32,
                "none" => on_track = cnt as i32,
                _ => {}
            }
        }

        // Latest snapshot date
        let latest: Option<chrono::NaiveDate> = sqlx::query_scalar(
            "SELECT MAX(snapshot_date) FROM _atlas.balance_snapshots WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(None);

        // Recent alerts
        let alerts = self.get_alert_snapshots(org_id).await.unwrap_or_default();
        let recent_alerts: Vec<serde_json::Value> = alerts.iter().take(10).map(|a| {
            serde_json::json!({
                "id": a.id,
                "accountSegment": a.account_segment,
                "periodName": a.period_name,
                "endingBalance": a.ending_balance,
                "alertStatus": a.alert_status,
                "variancePct": a.variance_pct,
                "computedAt": a.computed_at,
            })
        }).collect();

        Ok(AccountMonitorSummary {
            total_groups,
            active_groups,
            total_members,
            snapshots_with_warning: warning,
            snapshots_with_critical: critical,
            snapshots_on_track: on_track,
            latest_snapshot_date: latest,
            recent_alerts: serde_json::json!(recent_alerts),
        })
    }
}

fn row_to_group_bare(row: &sqlx::postgres::PgRow) -> AccountGroup {
    AccountGroup {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        code: row.try_get("code").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        owner_id: row.try_get("owner_id").unwrap_or_default(),
        is_shared: row.try_get("is_shared").unwrap_or(false),
        threshold_warning_pct: row.try_get("threshold_warning_pct").ok(),
        threshold_critical_pct: row.try_get("threshold_critical_pct").ok(),
        comparison_type: row.try_get("comparison_type").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_else(|_| "active".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        members: vec![],
        created_by: row.try_get("created_by").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(chrono::Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or(chrono::Utc::now()),
    }
}
