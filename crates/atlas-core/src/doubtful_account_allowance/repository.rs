//! Doubtful Account Allowance Repository
//!
//! Storage interface for doubtful account provision data.

use atlas_shared::{
    DoubtfulAccountPolicy, AgingBucketDefinition, ProvisionRun,
    ProvisionRunDetail, ProvisionRunActivity, DoubtfulAccountDashboard,
    AtlasResult, AtlasError,
};
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Repository trait for doubtful account allowance data storage
#[async_trait]
pub trait DoubtfulAccountAllowanceRepository: Send + Sync {
    async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        policy_name: &str,
        description: Option<&str>,
        calculation_method: &str,
        flat_percentage: &str,
        default_provision_account: Option<&str>,
        default_expense_account: Option<&str>,
        currency_code: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DoubtfulAccountPolicy>;

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<DoubtfulAccountPolicy>>;
    async fn get_policy_by_code(&self, org_id: Uuid, policy_code: &str) -> AtlasResult<Option<DoubtfulAccountPolicy>>;
    async fn list_policies(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DoubtfulAccountPolicy>>;
    async fn update_policy_status(&self, id: Uuid, status: &str) -> AtlasResult<DoubtfulAccountPolicy>;

    async fn create_aging_bucket(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        bucket_name: &str,
        from_days: i32,
        to_days: Option<i32>,
        provision_percentage: &str,
        display_order: i32,
    ) -> AtlasResult<AgingBucketDefinition>;

    async fn list_aging_buckets(&self, policy_id: Uuid) -> AtlasResult<Vec<AgingBucketDefinition>>;

    async fn create_provision_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        policy_id: Uuid,
        policy_code: &str,
        run_date: chrono::NaiveDate,
        as_of_date: chrono::NaiveDate,
        calculation_method: &str,
        currency_code: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun>;

    async fn get_provision_run(&self, id: Uuid) -> AtlasResult<Option<ProvisionRun>>;
    async fn get_provision_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<ProvisionRun>>;
    async fn list_provision_runs(&self, org_id: Uuid, policy_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ProvisionRun>>;

    async fn update_provision_run_results(
        &self,
        id: Uuid,
        total_outstanding_amount: &str,
        total_provision_amount: &str,
        total_prior_provision: &str,
        incremental_provision: &str,
        customer_count: i32,
        transaction_count: i32,
        status: &str,
    ) -> AtlasResult<ProvisionRun>;

    async fn update_provision_run_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
        journal_entry_number: Option<&str>,
    ) -> AtlasResult<ProvisionRun>;

    async fn create_provision_detail(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        bucket_id: Option<Uuid>,
        bucket_name: &str,
        from_days: i32,
        to_days: Option<i32>,
        provision_percentage: &str,
        outstanding_amount: &str,
        transaction_count: i32,
        customer_count: i32,
        provision_amount: &str,
    ) -> AtlasResult<ProvisionRunDetail>;

    async fn list_provision_details(&self, run_id: Uuid) -> AtlasResult<Vec<ProvisionRunDetail>>;

    async fn create_activity(
        &self,
        org_id: Uuid,
        run_id: Option<Uuid>,
        policy_id: Option<Uuid>,
        action: &str,
        description: Option<&str>,
        performed_by: Option<Uuid>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> AtlasResult<ProvisionRunActivity>;

    async fn list_activities(&self, run_id: Uuid) -> AtlasResult<Vec<ProvisionRunActivity>>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DoubtfulAccountDashboard>;
    async fn get_next_run_number(&self, org_id: Uuid) -> AtlasResult<i32>;
}

/// PostgreSQL row types for query_as
#[derive(Debug, sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    organization_id: Uuid,
    policy_code: String,
    policy_name: String,
    description: Option<String>,
    calculation_method: String,
    flat_percentage: f64,
    default_provision_account: Option<String>,
    default_expense_account: Option<String>,
    currency_code: String,
    effective_from: chrono::NaiveDate,
    effective_to: Option<chrono::NaiveDate>,
    is_active: bool,
    status: String,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PolicyRow> for DoubtfulAccountPolicy {
    fn from(r: PolicyRow) -> Self {
        DoubtfulAccountPolicy {
            id: r.id,
            organization_id: r.organization_id,
            policy_code: r.policy_code,
            policy_name: r.policy_name,
            description: r.description,
            calculation_method: r.calculation_method,
            flat_percentage: r.flat_percentage.to_string(),
            default_provision_account: r.default_provision_account,
            default_expense_account: r.default_expense_account,
            currency_code: r.currency_code,
            effective_from: r.effective_from,
            effective_to: r.effective_to,
            is_active: r.is_active,
            status: r.status,
            aging_buckets: vec![],
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct BucketRow {
    id: Uuid,
    organization_id: Uuid,
    policy_id: Uuid,
    bucket_name: String,
    from_days: i32,
    to_days: Option<i32>,
    provision_percentage: f64,
    display_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<BucketRow> for AgingBucketDefinition {
    fn from(r: BucketRow) -> Self {
        AgingBucketDefinition {
            id: r.id,
            organization_id: r.organization_id,
            policy_id: r.policy_id,
            bucket_name: r.bucket_name,
            from_days: r.from_days,
            to_days: r.to_days,
            provision_percentage: r.provision_percentage.to_string(),
            display_order: r.display_order,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RunRow {
    id: Uuid,
    organization_id: Uuid,
    run_number: String,
    policy_id: Uuid,
    policy_code: String,
    run_date: chrono::NaiveDate,
    as_of_date: chrono::NaiveDate,
    calculation_method: String,
    status: String,
    total_outstanding_amount: f64,
    total_provision_amount: f64,
    total_prior_provision: f64,
    incremental_provision: f64,
    currency_code: String,
    journal_batch_id: Option<Uuid>,
    journal_entry_number: Option<String>,
    description: Option<String>,
    customer_count: i32,
    transaction_count: i32,
    posted_by: Option<Uuid>,
    posted_at: Option<DateTime<Utc>>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RunRow> for ProvisionRun {
    fn from(r: RunRow) -> Self {
        ProvisionRun {
            id: r.id,
            organization_id: r.organization_id,
            run_number: r.run_number,
            policy_id: r.policy_id,
            policy_code: r.policy_code,
            run_date: r.run_date,
            as_of_date: r.as_of_date,
            calculation_method: r.calculation_method,
            status: r.status,
            total_outstanding_amount: r.total_outstanding_amount.to_string(),
            total_provision_amount: r.total_provision_amount.to_string(),
            total_prior_provision: r.total_prior_provision.to_string(),
            incremental_provision: r.incremental_provision.to_string(),
            currency_code: r.currency_code,
            journal_batch_id: r.journal_batch_id,
            journal_entry_number: r.journal_entry_number,
            description: r.description,
            customer_count: r.customer_count,
            transaction_count: r.transaction_count,
            posted_by: r.posted_by,
            posted_at: r.posted_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DetailRow {
    id: Uuid,
    organization_id: Uuid,
    run_id: Uuid,
    bucket_id: Option<Uuid>,
    bucket_name: String,
    from_days: i32,
    to_days: Option<i32>,
    provision_percentage: f64,
    outstanding_amount: f64,
    transaction_count: i32,
    customer_count: i32,
    provision_amount: f64,
    created_at: DateTime<Utc>,
}

impl From<DetailRow> for ProvisionRunDetail {
    fn from(r: DetailRow) -> Self {
        ProvisionRunDetail {
            id: r.id,
            organization_id: r.organization_id,
            run_id: r.run_id,
            bucket_id: r.bucket_id,
            bucket_name: r.bucket_name,
            from_days: r.from_days,
            to_days: r.to_days,
            provision_percentage: r.provision_percentage.to_string(),
            outstanding_amount: r.outstanding_amount.to_string(),
            transaction_count: r.transaction_count,
            customer_count: r.customer_count,
            provision_amount: r.provision_amount.to_string(),
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ActivityRow {
    id: Uuid,
    organization_id: Uuid,
    run_id: Option<Uuid>,
    policy_id: Option<Uuid>,
    action: String,
    description: Option<String>,
    performed_by: Option<Uuid>,
    performed_at: DateTime<Utc>,
    old_status: Option<String>,
    new_status: Option<String>,
    metadata: Option<serde_json::Value>,
}

impl From<ActivityRow> for ProvisionRunActivity {
    fn from(r: ActivityRow) -> Self {
        ProvisionRunActivity {
            id: r.id,
            organization_id: r.organization_id,
            run_id: r.run_id,
            policy_id: r.policy_id,
            action: r.action,
            description: r.description,
            performed_by: r.performed_by,
            performed_at: r.performed_at,
            old_status: r.old_status,
            new_status: r.new_status,
            metadata: r.metadata,
        }
    }
}

/// PostgreSQL implementation
pub struct PostgresDoubtfulAccountAllowanceRepository {
    pool: sqlx::PgPool,
}

impl PostgresDoubtfulAccountAllowanceRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    async fn load_buckets(&self, policy_id: Uuid) -> Vec<AgingBucketDefinition> {
        self.list_aging_buckets(policy_id).await.unwrap_or_default()
    }
}

#[async_trait]
impl DoubtfulAccountAllowanceRepository for PostgresDoubtfulAccountAllowanceRepository {
    async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        policy_name: &str,
        description: Option<&str>,
        calculation_method: &str,
        flat_percentage: &str,
        default_provision_account: Option<&str>,
        default_expense_account: Option<&str>,
        currency_code: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DoubtfulAccountPolicy> {
        let row: PolicyRow = sqlx::query_as(
            r#"
            INSERT INTO _atlas.doubtful_account_policies
                (organization_id, policy_code, policy_name, description,
                 calculation_method, flat_percentage,
                 default_provision_account, default_expense_account,
                 currency_code, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(policy_code)
        .bind(policy_name)
        .bind(description)
        .bind(calculation_method)
        .bind(flat_percentage.parse::<f64>().unwrap_or(0.0))
        .bind(default_provision_account)
        .bind(default_expense_account)
        .bind(currency_code)
        .bind(effective_from)
        .bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut policy = DoubtfulAccountPolicy::from(row);
        policy.aging_buckets = self.load_buckets(policy.id).await;
        Ok(policy)
    }

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<DoubtfulAccountPolicy>> {
        let row: Option<PolicyRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_policies WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let buckets = self.load_buckets(r.id).await;
                let mut policy = DoubtfulAccountPolicy::from(r);
                policy.aging_buckets = buckets;
                Ok(Some(policy))
            }
            None => Ok(None),
        }
    }

    async fn get_policy_by_code(&self, org_id: Uuid, policy_code: &str) -> AtlasResult<Option<DoubtfulAccountPolicy>> {
        let row: Option<PolicyRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_policies WHERE organization_id = $1 AND policy_code = $2",
        )
        .bind(org_id)
        .bind(policy_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let buckets = self.load_buckets(r.id).await;
                let mut policy = DoubtfulAccountPolicy::from(r);
                policy.aging_buckets = buckets;
                Ok(Some(policy))
            }
            None => Ok(None),
        }
    }

    async fn list_policies(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DoubtfulAccountPolicy>> {
        let rows: Vec<PolicyRow> = if let Some(s) = status {
            sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_policies WHERE organization_id = $1 AND status = $2 ORDER BY policy_code",
            )
            .bind(org_id)
            .bind(s)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_policies WHERE organization_id = $1 ORDER BY policy_code",
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut policies = Vec::new();
        for r in rows {
            let buckets = self.load_buckets(r.id).await;
            let mut policy = DoubtfulAccountPolicy::from(r);
            policy.aging_buckets = buckets;
            policies.push(policy);
        }
        Ok(policies)
    }

    async fn update_policy_status(&self, id: Uuid, status: &str) -> AtlasResult<DoubtfulAccountPolicy> {
        sqlx::query(
            "UPDATE _atlas.doubtful_account_policies SET status = $2, is_active = ($2 = 'active'), updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        self.get_policy(id).await?.ok_or_else(|| AtlasError::EntityNotFound("Policy not found".to_string()))
    }

    async fn create_aging_bucket(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        bucket_name: &str,
        from_days: i32,
        to_days: Option<i32>,
        provision_percentage: &str,
        display_order: i32,
    ) -> AtlasResult<AgingBucketDefinition> {
        let row: BucketRow = sqlx::query_as(
            r#"
            INSERT INTO _atlas.doubtful_account_aging_buckets
                (organization_id, policy_id, bucket_name, from_days, to_days,
                 provision_percentage, display_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(policy_id)
        .bind(bucket_name)
        .bind(from_days)
        .bind(to_days)
        .bind(provision_percentage.parse::<f64>().unwrap_or(0.0))
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(AgingBucketDefinition::from(row))
    }

    async fn list_aging_buckets(&self, policy_id: Uuid) -> AtlasResult<Vec<AgingBucketDefinition>> {
        let rows: Vec<BucketRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_aging_buckets WHERE policy_id = $1 ORDER BY from_days",
        )
        .bind(policy_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(AgingBucketDefinition::from).collect())
    }

    async fn create_provision_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        policy_id: Uuid,
        policy_code: &str,
        run_date: chrono::NaiveDate,
        as_of_date: chrono::NaiveDate,
        calculation_method: &str,
        currency_code: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProvisionRun> {
        let row: RunRow = sqlx::query_as(
            r#"
            INSERT INTO _atlas.doubtful_account_provision_runs
                (organization_id, run_number, policy_id, policy_code,
                 run_date, as_of_date, calculation_method,
                 currency_code, description, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(run_number)
        .bind(policy_id)
        .bind(policy_code)
        .bind(run_date)
        .bind(as_of_date)
        .bind(calculation_method)
        .bind(currency_code)
        .bind(description)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(ProvisionRun::from(row))
    }

    async fn get_provision_run(&self, id: Uuid) -> AtlasResult<Option<ProvisionRun>> {
        let row: Option<RunRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(ProvisionRun::from))
    }

    async fn get_provision_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<ProvisionRun>> {
        let row: Option<RunRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND run_number = $2",
        )
        .bind(org_id)
        .bind(run_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(ProvisionRun::from))
    }

    async fn list_provision_runs(&self, org_id: Uuid, policy_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<ProvisionRun>> {
        let rows: Vec<RunRow> = match (policy_id, status) {
            (Some(pid), Some(s)) => sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND policy_id = $2 AND status = $3 ORDER BY run_date DESC",
            ).bind(org_id).bind(pid).bind(s).fetch_all(&self.pool).await,
            (Some(pid), None) => sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND policy_id = $2 ORDER BY run_date DESC",
            ).bind(org_id).bind(pid).fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND status = $2 ORDER BY run_date DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await,
            (None, None) => sqlx::query_as(
                "SELECT * FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 ORDER BY run_date DESC",
            ).bind(org_id).fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(ProvisionRun::from).collect())
    }

    async fn update_provision_run_results(
        &self,
        id: Uuid,
        total_outstanding_amount: &str,
        total_provision_amount: &str,
        total_prior_provision: &str,
        incremental_provision: &str,
        customer_count: i32,
        transaction_count: i32,
        status: &str,
    ) -> AtlasResult<ProvisionRun> {
        sqlx::query(
            r#"
            UPDATE _atlas.doubtful_account_provision_runs
            SET total_outstanding_amount = $2,
                total_provision_amount = $3,
                total_prior_provision = $4,
                incremental_provision = $5,
                customer_count = $6,
                transaction_count = $7,
                status = $8,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(total_outstanding_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_provision_amount.parse::<f64>().unwrap_or(0.0))
        .bind(total_prior_provision.parse::<f64>().unwrap_or(0.0))
        .bind(incremental_provision.parse::<f64>().unwrap_or(0.0))
        .bind(customer_count)
        .bind(transaction_count)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        self.get_provision_run(id).await?.ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))
    }

    async fn update_provision_run_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
        journal_entry_number: Option<&str>,
    ) -> AtlasResult<ProvisionRun> {
        sqlx::query(
            r#"
            UPDATE _atlas.doubtful_account_provision_runs
            SET status = $2,
                posted_by = COALESCE($3, posted_by),
                posted_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE posted_at END,
                journal_entry_number = COALESCE($4, journal_entry_number),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(posted_by)
        .bind(journal_entry_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        self.get_provision_run(id).await?.ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))
    }

    async fn create_provision_detail(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        bucket_id: Option<Uuid>,
        bucket_name: &str,
        from_days: i32,
        to_days: Option<i32>,
        provision_percentage: &str,
        outstanding_amount: &str,
        transaction_count: i32,
        customer_count: i32,
        provision_amount: &str,
    ) -> AtlasResult<ProvisionRunDetail> {
        let row: DetailRow = sqlx::query_as(
            r#"
            INSERT INTO _atlas.doubtful_account_provision_details
                (organization_id, run_id, bucket_id, bucket_name,
                 from_days, to_days, provision_percentage,
                 outstanding_amount, transaction_count, customer_count,
                 provision_amount)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(run_id)
        .bind(bucket_id)
        .bind(bucket_name)
        .bind(from_days)
        .bind(to_days)
        .bind(provision_percentage.parse::<f64>().unwrap_or(0.0))
        .bind(outstanding_amount.parse::<f64>().unwrap_or(0.0))
        .bind(transaction_count)
        .bind(customer_count)
        .bind(provision_amount.parse::<f64>().unwrap_or(0.0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(ProvisionRunDetail::from(row))
    }

    async fn list_provision_details(&self, run_id: Uuid) -> AtlasResult<Vec<ProvisionRunDetail>> {
        let rows: Vec<DetailRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_provision_details WHERE run_id = $1 ORDER BY from_days",
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(ProvisionRunDetail::from).collect())
    }

    async fn create_activity(
        &self,
        org_id: Uuid,
        run_id: Option<Uuid>,
        policy_id: Option<Uuid>,
        action: &str,
        description: Option<&str>,
        performed_by: Option<Uuid>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> AtlasResult<ProvisionRunActivity> {
        let row: ActivityRow = sqlx::query_as(
            r#"
            INSERT INTO _atlas.doubtful_account_provision_activities
                (organization_id, run_id, policy_id, action,
                 description, performed_by, old_status, new_status, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(run_id)
        .bind(policy_id)
        .bind(action)
        .bind(description)
        .bind(performed_by)
        .bind(old_status)
        .bind(new_status)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(ProvisionRunActivity::from(row))
    }

    async fn list_activities(&self, run_id: Uuid) -> AtlasResult<Vec<ProvisionRunActivity>> {
        let rows: Vec<ActivityRow> = sqlx::query_as(
            "SELECT * FROM _atlas.doubtful_account_provision_activities WHERE run_id = $1 ORDER BY performed_at",
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(ProvisionRunActivity::from).collect())
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DoubtfulAccountDashboard> {
        let policy_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.doubtful_account_policies WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.doubtful_account_policies WHERE organization_id = $1 AND status = 'active'",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let draft_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND status = 'draft'",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let posted_runs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.doubtful_account_provision_runs WHERE organization_id = $1 AND status = 'posted'",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        #[derive(Debug, sqlx::FromRow)]
        struct LatestRow {
            total_provision_amount: f64,
            run_date: chrono::NaiveDate,
            total_outstanding_amount: f64,
        }

        let latest: Option<LatestRow> = sqlx::query_as(
            r#"
            SELECT total_provision_amount, run_date, total_outstanding_amount
            FROM _atlas.doubtful_account_provision_runs
            WHERE organization_id = $1 AND status IN ('calculated', 'posted')
            ORDER BY run_date DESC LIMIT 1
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let latest_provision = latest
            .as_ref()
            .map(|l| l.total_provision_amount.to_string())
            .unwrap_or_else(|| "0.00".to_string());

        let latest_run_date = latest
            .as_ref()
            .map(|l| l.run_date.to_string())
            .unwrap_or_default();

        let total_outstanding = latest
            .as_ref()
            .map(|l| l.total_outstanding_amount.to_string())
            .unwrap_or_else(|| "0.00".to_string());

        let outstanding_f64: f64 = total_outstanding.parse().unwrap_or(0.0);
        let provision_f64: f64 = latest_provision.parse().unwrap_or(0.0);
        let rate = if outstanding_f64 > 0.0 {
            (provision_f64 / outstanding_f64) * 100.0
        } else {
            0.0
        };

        Ok(DoubtfulAccountDashboard {
            organization_id: org_id,
            total_policies: policy_count as i32,
            active_policies: active_count as i32,
            total_runs: total_runs as i32,
            draft_runs: draft_runs as i32,
            posted_runs: posted_runs as i32,
            latest_provision_amount: latest_provision,
            latest_run_date: if latest_run_date.is_empty() { None } else { Some(latest_run_date) },
            total_outstanding_ar: total_outstanding,
            overall_provision_rate: format!("{:.4}", rate),
        })
    }

    async fn get_next_run_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let max: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM _atlas.doubtful_account_provision_runs
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(max.unwrap_or(0) as i32 + 1)
    }
}
