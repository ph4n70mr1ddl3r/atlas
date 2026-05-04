//! Payment Risk & Fraud Detection Repository
//!
//! PostgreSQL storage for risk profiles, fraud alerts, sanctions screening results,
//! and supplier risk assessments.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Payment risk profile record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RiskProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub profile_type: String,
    pub default_risk_level: String,
    pub duplicate_amount_tolerance_pct: Option<String>,
    pub duplicate_date_tolerance_days: Option<String>,
    pub velocity_daily_limit: Option<String>,
    pub velocity_weekly_limit: Option<String>,
    pub amount_anomaly_std_dev: Option<String>,
    pub enable_sanctions_screening: bool,
    pub enable_duplicate_detection: bool,
    pub enable_velocity_checks: bool,
    pub enable_amount_anomaly: bool,
    pub enable_behavioral_analysis: bool,
    pub auto_block_critical: bool,
    pub auto_block_high: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<Uuid>,
}

/// Payment fraud alert record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FraudAlert {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub alert_number: String,
    pub alert_type: String,
    pub severity: String,
    pub status: String,
    pub payment_id: Option<Uuid>,
    pub invoice_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub amount: Option<String>,
    pub currency_code: Option<String>,
    pub risk_score: Option<String>,
    pub detection_rule: Option<String>,
    pub description: Option<String>,
    pub evidence: Option<String>,
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
    pub detected_date: Option<chrono::NaiveDate>,
    pub resolution_date: Option<chrono::NaiveDate>,
    pub resolution_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub related_alert_ids: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<Uuid>,
}

/// Sanctions screening result record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SanctionsScreeningResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub screening_id: String,
    pub screening_type: String,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub payment_id: Option<Uuid>,
    pub screened_list: String,
    pub match_name: Option<String>,
    pub match_type: String,
    pub match_score: Option<String>,
    pub match_status: String,
    pub sanctions_list_entry: Option<String>,
    pub sanctions_list_program: Option<String>,
    pub match_details: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_date: Option<chrono::NaiveDate>,
    pub review_notes: Option<String>,
    pub action_taken: Option<String>,
    pub screening_date: Option<chrono::NaiveDate>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<Uuid>,
}

/// Supplier risk assessment record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SupplierRiskAssessment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub assessment_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub assessment_date: Option<chrono::NaiveDate>,
    pub assessment_type: String,
    pub overall_risk_level: String,
    pub financial_risk_score: Option<String>,
    pub operational_risk_score: Option<String>,
    pub compliance_risk_score: Option<String>,
    pub payment_history_score: Option<String>,
    pub overall_risk_score: Option<String>,
    pub years_in_business: Option<i32>,
    pub has_financial_statements: bool,
    pub has_audit_reports: bool,
    pub has_insurance: bool,
    pub is_sanctions_clear: bool,
    pub is_aml_clear: bool,
    pub is_pep_clear: bool,
    pub payment_behavior_rating: Option<String>,
    pub total_historical_payments: Option<i32>,
    pub total_historical_amount: Option<String>,
    pub fraud_alerts_count: Option<i32>,
    pub duplicate_payments_count: Option<i32>,
    pub assessed_by: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub status: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<Uuid>,
}

// ============================================================================
// Create Parameters
// ============================================================================

/// Parameters for creating a risk profile
pub struct RiskProfileCreateParams {
    pub org_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub profile_type: String,
    pub default_risk_level: String,
    pub duplicate_amount_tolerance_pct: Option<String>,
    pub duplicate_date_tolerance_days: Option<String>,
    pub velocity_daily_limit: Option<String>,
    pub velocity_weekly_limit: Option<String>,
    pub amount_anomaly_std_dev: Option<String>,
    pub enable_sanctions_screening: bool,
    pub enable_duplicate_detection: bool,
    pub enable_velocity_checks: bool,
    pub enable_amount_anomaly: bool,
    pub enable_behavioral_analysis: bool,
    pub auto_block_critical: bool,
    pub auto_block_high: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a fraud alert
pub struct FraudAlertCreateParams {
    pub org_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub payment_id: Option<Uuid>,
    pub invoice_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub amount: Option<String>,
    pub currency_code: Option<String>,
    pub risk_score: Option<String>,
    pub detection_rule: Option<String>,
    pub description: Option<String>,
    pub evidence: Option<String>,
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
    pub related_alert_ids: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a sanctions screening result
pub struct SanctionsScreeningCreateParams {
    pub org_id: Uuid,
    pub screening_type: String,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub payment_id: Option<Uuid>,
    pub screened_list: String,
    pub match_name: Option<String>,
    pub match_type: String,
    pub match_score: Option<String>,
    pub match_status: String,
    pub sanctions_list_entry: Option<String>,
    pub sanctions_list_program: Option<String>,
    pub match_details: Option<String>,
    pub action_taken: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a supplier risk assessment
pub struct SupplierRiskAssessmentCreateParams {
    pub org_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub assessment_type: String,
    pub financial_risk_score: Option<String>,
    pub operational_risk_score: Option<String>,
    pub compliance_risk_score: Option<String>,
    pub payment_history_score: Option<String>,
    pub years_in_business: Option<i32>,
    pub has_financial_statements: bool,
    pub has_audit_reports: bool,
    pub has_insurance: bool,
    pub is_sanctions_clear: bool,
    pub is_aml_clear: bool,
    pub is_pep_clear: bool,
    pub total_historical_payments: Option<i32>,
    pub total_historical_amount: Option<String>,
    pub fraud_alerts_count: Option<i32>,
    pub duplicate_payments_count: Option<i32>,
    pub assessed_by: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub created_by: Option<Uuid>,
}

// ============================================================================
// Repository Trait
// ============================================================================

/// Repository trait for payment risk & fraud detection data storage
#[async_trait]
pub trait PaymentRiskRepository: Send + Sync {
    // Risk Profiles
    async fn create_risk_profile(&self, params: &RiskProfileCreateParams) -> AtlasResult<RiskProfile>;
    async fn get_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskProfile>>;
    async fn get_risk_profile_by_id(&self, id: Uuid) -> AtlasResult<Option<RiskProfile>>;
    async fn list_risk_profiles(&self, org_id: Uuid, profile_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<RiskProfile>>;
    async fn update_risk_profile_status(&self, id: Uuid, is_active: bool) -> AtlasResult<RiskProfile>;
    async fn delete_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;
    async fn get_next_profile_sequence(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Fraud Alerts
    async fn create_fraud_alert(&self, params: &FraudAlertCreateParams) -> AtlasResult<FraudAlert>;
    async fn get_fraud_alert(&self, org_id: Uuid, alert_number: &str) -> AtlasResult<Option<FraudAlert>>;
    async fn get_fraud_alert_by_id(&self, id: Uuid) -> AtlasResult<Option<FraudAlert>>;
    async fn list_fraud_alerts(&self, org_id: Uuid, status: Option<&str>, alert_type: Option<&str>, severity: Option<&str>) -> AtlasResult<Vec<FraudAlert>>;
    async fn update_fraud_alert_status(&self, id: Uuid, status: &str, resolution_notes: Option<&str>, resolved_by: Option<Uuid>) -> AtlasResult<FraudAlert>;
    async fn assign_fraud_alert(&self, id: Uuid, assigned_to: Option<&str>, assigned_team: Option<&str>) -> AtlasResult<FraudAlert>;
    async fn get_next_alert_sequence(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Sanctions Screening
    async fn create_screening_result(&self, params: &SanctionsScreeningCreateParams) -> AtlasResult<SanctionsScreeningResult>;
    async fn get_screening_result(&self, org_id: Uuid, screening_id: &str) -> AtlasResult<Option<SanctionsScreeningResult>>;
    async fn list_screening_results(&self, org_id: Uuid, supplier_id: Option<Uuid>, match_status: Option<&str>) -> AtlasResult<Vec<SanctionsScreeningResult>>;
    async fn review_screening_result(&self, id: Uuid, reviewed_by: &str, review_notes: Option<&str>, action_taken: &str) -> AtlasResult<SanctionsScreeningResult>;
    async fn get_next_screening_sequence(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Supplier Risk Assessments
    async fn create_assessment(&self, params: &SupplierRiskAssessmentCreateParams) -> AtlasResult<SupplierRiskAssessment>;
    async fn get_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<Option<SupplierRiskAssessment>>;
    async fn get_assessment_by_id(&self, id: Uuid) -> AtlasResult<Option<SupplierRiskAssessment>>;
    async fn list_assessments(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierRiskAssessment>>;
    async fn update_assessment_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierRiskAssessment>;
    async fn delete_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<()>;
    async fn get_next_assessment_sequence(&self, org_id: Uuid) -> AtlasResult<i32>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

/// PostgreSQL-backed payment risk repository
pub struct PostgresPaymentRiskRepository {
    pool: PgPool,
}

impl PostgresPaymentRiskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRiskRepository for PostgresPaymentRiskRepository {
    // Risk Profiles
    async fn create_risk_profile(&self, params: &RiskProfileCreateParams) -> AtlasResult<RiskProfile> {
        let row = sqlx::query_as::<_, RiskProfile>(
            r#"INSERT INTO _atlas.payment_risk_profiles
               (organization_id, code, name, description, profile_type, default_risk_level,
                duplicate_amount_tolerance_pct, duplicate_date_tolerance_days,
                velocity_daily_limit, velocity_weekly_limit, amount_anomaly_std_dev,
                enable_sanctions_screening, enable_duplicate_detection, enable_velocity_checks,
                enable_amount_anomaly, enable_behavioral_analysis,
                auto_block_critical, auto_block_high,
                effective_from, effective_to, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(&params.code)
        .bind(&params.name)
        .bind(&params.description)
        .bind(&params.profile_type)
        .bind(&params.default_risk_level)
        .bind(&params.duplicate_amount_tolerance_pct)
        .bind(&params.duplicate_date_tolerance_days)
        .bind(&params.velocity_daily_limit)
        .bind(&params.velocity_weekly_limit)
        .bind(&params.amount_anomaly_std_dev)
        .bind(params.enable_sanctions_screening)
        .bind(params.enable_duplicate_detection)
        .bind(params.enable_velocity_checks)
        .bind(params.enable_amount_anomaly)
        .bind(params.enable_behavioral_analysis)
        .bind(params.auto_block_critical)
        .bind(params.auto_block_high)
        .bind(params.effective_from)
        .bind(params.effective_to)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RiskProfile>> {
        let row = sqlx::query_as::<_, RiskProfile>(
            "SELECT * FROM _atlas.payment_risk_profiles WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_risk_profile_by_id(&self, id: Uuid) -> AtlasResult<Option<RiskProfile>> {
        let row = sqlx::query_as::<_, RiskProfile>(
            "SELECT * FROM _atlas.payment_risk_profiles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_risk_profiles(&self, org_id: Uuid, profile_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<RiskProfile>> {
        let rows = sqlx::query_as::<_, RiskProfile>(
            "SELECT * FROM _atlas.payment_risk_profiles WHERE organization_id = $1 AND ($2::text IS NULL OR profile_type = $2) AND ($3::bool IS NULL OR is_active = $3) ORDER BY code",
        )
        .bind(org_id)
        .bind(profile_type)
        .bind(is_active)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_risk_profile_status(&self, id: Uuid, is_active: bool) -> AtlasResult<RiskProfile> {
        let row = sqlx::query_as::<_, RiskProfile>(
            "UPDATE _atlas.payment_risk_profiles SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn delete_risk_profile(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.payment_risk_profiles WHERE organization_id = $1 AND code = $2",
        )
        .bind(org_id)
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Risk profile not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_profile_sequence(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(code FROM 'RP-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.payment_risk_profiles WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Fraud Alerts
    async fn create_fraud_alert(&self, params: &FraudAlertCreateParams) -> AtlasResult<FraudAlert> {
        let row = sqlx::query_as::<_, FraudAlert>(
            r#"INSERT INTO _atlas.payment_fraud_alerts
               (organization_id, alert_type, severity, status,
                payment_id, invoice_id, supplier_id, supplier_number, supplier_name,
                amount, currency_code, risk_score, detection_rule,
                description, evidence, assigned_to, assigned_team,
                detected_date, related_alert_ids, created_by)
               VALUES ($1,$2,$3,'open',$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,CURRENT_DATE,$17,$18)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(&params.alert_type)
        .bind(&params.severity)
        .bind(params.payment_id)
        .bind(params.invoice_id)
        .bind(params.supplier_id)
        .bind(&params.supplier_number)
        .bind(&params.supplier_name)
        .bind(&params.amount)
        .bind(&params.currency_code)
        .bind(&params.risk_score)
        .bind(&params.detection_rule)
        .bind(&params.description)
        .bind(&params.evidence)
        .bind(&params.assigned_to)
        .bind(&params.assigned_team)
        .bind(&params.related_alert_ids)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_fraud_alert(&self, org_id: Uuid, alert_number: &str) -> AtlasResult<Option<FraudAlert>> {
        let row = sqlx::query_as::<_, FraudAlert>(
            "SELECT * FROM _atlas.payment_fraud_alerts WHERE organization_id = $1 AND alert_number = $2",
        )
        .bind(org_id)
        .bind(alert_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_fraud_alert_by_id(&self, id: Uuid) -> AtlasResult<Option<FraudAlert>> {
        let row = sqlx::query_as::<_, FraudAlert>(
            "SELECT * FROM _atlas.payment_fraud_alerts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_fraud_alerts(&self, org_id: Uuid, status: Option<&str>, alert_type: Option<&str>, severity: Option<&str>) -> AtlasResult<Vec<FraudAlert>> {
        let rows = sqlx::query_as::<_, FraudAlert>(
            "SELECT * FROM _atlas.payment_fraud_alerts WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2) AND ($3::text IS NULL OR alert_type = $3) AND ($4::text IS NULL OR severity = $4) ORDER BY detected_date DESC",
        )
        .bind(org_id)
        .bind(status)
        .bind(alert_type)
        .bind(severity)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_fraud_alert_status(&self, id: Uuid, status: &str, resolution_notes: Option<&str>, resolved_by: Option<Uuid>) -> AtlasResult<FraudAlert> {
        let row = sqlx::query_as::<_, FraudAlert>(
            r#"UPDATE _atlas.payment_fraud_alerts
               SET status = $2, resolution_notes = COALESCE($3, resolution_notes),
                   resolution_date = CASE WHEN $2 IN ('confirmed_fraud','false_positive','closed') THEN CURRENT_DATE ELSE resolution_date END,
                   resolved_by = COALESCE($4, resolved_by),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(resolution_notes)
        .bind(resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn assign_fraud_alert(&self, id: Uuid, assigned_to: Option<&str>, assigned_team: Option<&str>) -> AtlasResult<FraudAlert> {
        let row = sqlx::query_as::<_, FraudAlert>(
            r#"UPDATE _atlas.payment_fraud_alerts
               SET assigned_to = $2, assigned_team = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(assigned_to)
        .bind(assigned_team)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_next_alert_sequence(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(alert_number FROM 'FA-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.payment_fraud_alerts WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Sanctions Screening
    async fn create_screening_result(&self, params: &SanctionsScreeningCreateParams) -> AtlasResult<SanctionsScreeningResult> {
        let row = sqlx::query_as::<_, SanctionsScreeningResult>(
            r#"INSERT INTO _atlas.sanctions_screening_results
               (organization_id, screening_type, supplier_id, supplier_name, payment_id,
                screened_list, match_name, match_type, match_score, match_status,
                sanctions_list_entry, sanctions_list_program, match_details,
                action_taken, screening_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,CURRENT_DATE,$15)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(&params.screening_type)
        .bind(params.supplier_id)
        .bind(&params.supplier_name)
        .bind(params.payment_id)
        .bind(&params.screened_list)
        .bind(&params.match_name)
        .bind(&params.match_type)
        .bind(&params.match_score)
        .bind(&params.match_status)
        .bind(&params.sanctions_list_entry)
        .bind(&params.sanctions_list_program)
        .bind(&params.match_details)
        .bind(&params.action_taken)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_screening_result(&self, org_id: Uuid, screening_id: &str) -> AtlasResult<Option<SanctionsScreeningResult>> {
        let row = sqlx::query_as::<_, SanctionsScreeningResult>(
            "SELECT * FROM _atlas.sanctions_screening_results WHERE organization_id = $1 AND screening_id = $2",
        )
        .bind(org_id)
        .bind(screening_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_screening_results(&self, org_id: Uuid, supplier_id: Option<Uuid>, match_status: Option<&str>) -> AtlasResult<Vec<SanctionsScreeningResult>> {
        let rows = sqlx::query_as::<_, SanctionsScreeningResult>(
            "SELECT * FROM _atlas.sanctions_screening_results WHERE organization_id = $1 AND ($2::uuid IS NULL OR supplier_id = $2) AND ($3::text IS NULL OR match_status = $3) ORDER BY screening_date DESC",
        )
        .bind(org_id)
        .bind(supplier_id)
        .bind(match_status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn review_screening_result(&self, id: Uuid, reviewed_by: &str, review_notes: Option<&str>, action_taken: &str) -> AtlasResult<SanctionsScreeningResult> {
        let row = sqlx::query_as::<_, SanctionsScreeningResult>(
            r#"UPDATE _atlas.sanctions_screening_results
               SET reviewed_by = $2, review_notes = $3, action_taken = $4, reviewed_date = CURRENT_DATE
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(reviewed_by)
        .bind(review_notes)
        .bind(action_taken)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_next_screening_sequence(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(screening_id FROM 'SC-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.sanctions_screening_results WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Supplier Risk Assessments
    async fn create_assessment(&self, params: &SupplierRiskAssessmentCreateParams) -> AtlasResult<SupplierRiskAssessment> {
        let row = sqlx::query_as::<_, SupplierRiskAssessment>(
            r#"INSERT INTO _atlas.supplier_risk_assessments
               (organization_id, supplier_id, supplier_name, assessment_date, assessment_type,
                overall_risk_level, status,
                financial_risk_score, operational_risk_score, compliance_risk_score,
                payment_history_score, overall_risk_score,
                years_in_business, has_financial_statements, has_audit_reports, has_insurance,
                is_sanctions_clear, is_aml_clear, is_pep_clear,
                total_historical_payments, total_historical_amount,
                fraud_alerts_count, duplicate_payments_count,
                assessed_by, findings, recommendations, created_by)
               VALUES ($1,$2,$3,CURRENT_DATE,$4,'medium','pending',
                       $5,$6,$7,$8,'0.00',
                       $9,$10,$11,$12,$13,$14,$15,
                       $16,$17,$18,$19,$20,$21,$22,$23)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(params.supplier_id)
        .bind(&params.supplier_name)
        .bind(&params.assessment_type)
        .bind(&params.financial_risk_score)
        .bind(&params.operational_risk_score)
        .bind(&params.compliance_risk_score)
        .bind(&params.payment_history_score)
        .bind(params.years_in_business)
        .bind(params.has_financial_statements)
        .bind(params.has_audit_reports)
        .bind(params.has_insurance)
        .bind(params.is_sanctions_clear)
        .bind(params.is_aml_clear)
        .bind(params.is_pep_clear)
        .bind(params.total_historical_payments)
        .bind(&params.total_historical_amount)
        .bind(params.fraud_alerts_count)
        .bind(params.duplicate_payments_count)
        .bind(&params.assessed_by)
        .bind(&params.findings)
        .bind(&params.recommendations)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<Option<SupplierRiskAssessment>> {
        let row = sqlx::query_as::<_, SupplierRiskAssessment>(
            "SELECT * FROM _atlas.supplier_risk_assessments WHERE organization_id = $1 AND assessment_number = $2",
        )
        .bind(org_id)
        .bind(assessment_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_assessment_by_id(&self, id: Uuid) -> AtlasResult<Option<SupplierRiskAssessment>> {
        let row = sqlx::query_as::<_, SupplierRiskAssessment>(
            "SELECT * FROM _atlas.supplier_risk_assessments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_assessments(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierRiskAssessment>> {
        let rows = sqlx::query_as::<_, SupplierRiskAssessment>(
            "SELECT * FROM _atlas.supplier_risk_assessments WHERE organization_id = $1 AND ($2::uuid IS NULL OR supplier_id = $2) AND ($3::text IS NULL OR status = $3) ORDER BY assessment_date DESC",
        )
        .bind(org_id)
        .bind(supplier_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_assessment_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierRiskAssessment> {
        let row = sqlx::query_as::<_, SupplierRiskAssessment>(
            "UPDATE _atlas.supplier_risk_assessments SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn delete_assessment(&self, org_id: Uuid, assessment_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.supplier_risk_assessments WHERE organization_id = $1 AND assessment_number = $2",
        )
        .bind(org_id)
        .bind(assessment_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Assessment not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_assessment_sequence(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(assessment_number FROM 'RA-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.supplier_risk_assessments WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }
}
