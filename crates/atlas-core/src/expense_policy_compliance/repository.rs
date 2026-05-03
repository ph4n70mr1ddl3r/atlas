//! Expense Policy Compliance Repository
//!
//! PostgreSQL storage for expense policy rules, compliance audits, and violations.

use atlas_shared::{
    ExpensePolicyRule, ExpenseComplianceAudit, ExpenseComplianceViolation,
    ExpenseComplianceDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for expense policy compliance data storage
#[async_trait]
pub trait ExpensePolicyComplianceRepository: Send + Sync {
    // Policy Rules
    async fn create_rule(
        &self, org_id: Uuid, rule_code: &str, name: &str, description: Option<&str>,
        rule_type: &str, expense_category: &str, severity: &str, evaluation_scope: &str,
        threshold_amount: Option<&str>, maximum_amount: Option<&str>,
        threshold_days: i32, requires_receipt: bool, requires_justification: bool,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        applies_to_department: Option<&str>, applies_to_cost_center: Option<&str>,
        created_by_id: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicyRule>;
    async fn get_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<Option<ExpensePolicyRule>>;
    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpensePolicyRule>>;
    async fn list_rules(&self, org_id: Uuid, status: Option<&str>, rule_type: Option<&str>) -> AtlasResult<Vec<ExpensePolicyRule>>;
    async fn update_rule_status(&self, id: Uuid, status: &str) -> AtlasResult<ExpensePolicyRule>;
    async fn delete_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<()>;

    // Compliance Audits
    async fn create_audit(
        &self, org_id: Uuid, audit_number: &str, report_id: Uuid,
        report_number: Option<&str>, employee_id: Option<Uuid>,
        employee_name: Option<&str>, department_id: Option<Uuid>,
        audit_date: chrono::NaiveDate, audit_trigger: &str,
    ) -> AtlasResult<ExpenseComplianceAudit>;
    async fn get_audit(&self, id: Uuid) -> AtlasResult<Option<ExpenseComplianceAudit>>;
    async fn get_audit_by_number(&self, org_id: Uuid, audit_number: &str) -> AtlasResult<Option<ExpenseComplianceAudit>>;
    async fn list_audits(&self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>) -> AtlasResult<Vec<ExpenseComplianceAudit>>;
    async fn update_audit_results(
        &self, id: Uuid, total_lines: i32, violations_count: i32, warnings_count: i32,
        blocks_count: i32, compliance_score: &str, risk_level: &str,
        total_flagged_amount: &str, total_approved_amount: &str,
        requires_manager_review: bool, requires_finance_review: bool,
    ) -> AtlasResult<ExpenseComplianceAudit>;
    async fn update_audit_review(
        &self, id: Uuid, status: &str, reviewed_by_id: Option<Uuid>, review_notes: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceAudit>;
    async fn get_latest_audit_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Violations
    async fn create_violation(
        &self, org_id: Uuid, audit_id: Uuid, report_id: Uuid,
        report_line_id: Option<Uuid>, policy_rule_id: Option<Uuid>,
        rule_code: &str, rule_name: Option<&str>, rule_type: &str,
        severity: &str, violation_description: Option<&str>,
        expense_amount: Option<&str>, threshold_amount: Option<&str>,
        excess_amount: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceViolation>;
    async fn list_violations(&self, audit_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>>;
    async fn get_violation_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpenseComplianceViolation>>;
    async fn update_violation_resolution(
        &self, id: Uuid, resolution_status: &str, justification: Option<&str>,
        resolved_by_id: Option<Uuid>, resolution_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ExpenseComplianceViolation>;
    async fn list_open_violations(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ExpenseComplianceDashboard>;
}

/// Helper to read optional numeric text from a row
fn get_opt_numeric(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(col).unwrap_or(None)
}

/// Column list for policy rules with numeric casts
const RULE_COLS: &str = "\
    id, org_id, rule_code, name, description, rule_type, expense_category,\
    severity, evaluation_scope,\
    threshold_amount::text as threshold_amount,\
    maximum_amount::text as maximum_amount,\
    threshold_days, requires_receipt, requires_justification, is_active,\
    effective_from, effective_to, applies_to_department, applies_to_cost_center,\
    created_by_id, status, created_at, updated_at";

/// RETURNING clause for rule inserts/updates
const RULE_RETURNING: &str = "\
    RETURNING id, org_id, rule_code, name, description, rule_type, expense_category,\
    severity, evaluation_scope,\
    threshold_amount::text as threshold_amount,\
    maximum_amount::text as maximum_amount,\
    threshold_days, requires_receipt, requires_justification, is_active,\
    effective_from, effective_to, applies_to_department, applies_to_cost_center,\
    created_by_id, status, created_at, updated_at";

/// Column list for audits with numeric casts
const AUDIT_COLS: &str = "\
    id, org_id, audit_number, report_id, report_number,\
    employee_id, employee_name, department_id, audit_date, audit_trigger,\
    total_lines, violations_count, warnings_count, blocks_count,\
    compliance_score::text as compliance_score,\
    risk_level,\
    total_flagged_amount::text as total_flagged_amount,\
    total_approved_amount::text as total_approved_amount,\
    requires_manager_review, requires_finance_review,\
    status, reviewed_by_id, review_notes, created_at, updated_at";

/// RETURNING clause for audit updates
const AUDIT_RETURNING: &str = "\
    RETURNING id, org_id, audit_number, report_id, report_number,\
    employee_id, employee_name, department_id, audit_date, audit_trigger,\
    total_lines, violations_count, warnings_count, blocks_count,\
    compliance_score::text as compliance_score,\
    risk_level,\
    total_flagged_amount::text as total_flagged_amount,\
    total_approved_amount::text as total_approved_amount,\
    requires_manager_review, requires_finance_review,\
    status, reviewed_by_id, review_notes, created_at, updated_at";

/// Column list for violations with numeric casts
const VIOLATION_COLS: &str = "\
    id, org_id, audit_id, report_id, report_line_id, policy_rule_id,\
    rule_code, rule_name, rule_type, severity, violation_description,\
    expense_amount::text as expense_amount,\
    threshold_amount::text as threshold_amount,\
    excess_amount::text as excess_amount,\
    resolution_status, justification, resolved_by_id, resolution_date,\
    created_at, updated_at";

/// RETURNING clause for violation inserts/updates
const VIOLATION_RETURNING: &str = "\
    RETURNING id, org_id, audit_id, report_id, report_line_id, policy_rule_id,\
    rule_code, rule_name, rule_type, severity, violation_description,\
    expense_amount::text as expense_amount,\
    threshold_amount::text as threshold_amount,\
    excess_amount::text as excess_amount,\
    resolution_status, justification, resolved_by_id, resolution_date,\
    created_at, updated_at";

fn row_to_rule(row: &sqlx::postgres::PgRow) -> ExpensePolicyRule {
    ExpensePolicyRule {
        id: row.get("id"),
        org_id: row.get("org_id"),
        rule_code: row.get("rule_code"),
        name: row.get("name"),
        description: row.get("description"),
        rule_type: row.get("rule_type"),
        expense_category: row.get("expense_category"),
        severity: row.get("severity"),
        evaluation_scope: row.get("evaluation_scope"),
        threshold_amount: get_opt_numeric(row, "threshold_amount"),
        maximum_amount: get_opt_numeric(row, "maximum_amount"),
        threshold_days: row.get("threshold_days"),
        requires_receipt: row.get("requires_receipt"),
        requires_justification: row.get("requires_justification"),
        is_active: row.get("is_active"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        applies_to_department: row.get("applies_to_department"),
        applies_to_cost_center: row.get("applies_to_cost_center"),
        created_by_id: row.get("created_by_id"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_audit(row: &sqlx::postgres::PgRow) -> ExpenseComplianceAudit {
    ExpenseComplianceAudit {
        id: row.get("id"),
        org_id: row.get("org_id"),
        audit_number: row.get("audit_number"),
        report_id: row.get("report_id"),
        report_number: row.get("report_number"),
        employee_id: row.get("employee_id"),
        employee_name: row.get("employee_name"),
        department_id: row.get("department_id"),
        audit_date: row.get("audit_date"),
        audit_trigger: row.get("audit_trigger"),
        total_lines: row.get("total_lines"),
        violations_count: row.get("violations_count"),
        warnings_count: row.get("warnings_count"),
        blocks_count: row.get("blocks_count"),
        compliance_score: get_opt_numeric(row, "compliance_score"),
        risk_level: row.get("risk_level"),
        total_flagged_amount: get_opt_numeric(row, "total_flagged_amount"),
        total_approved_amount: get_opt_numeric(row, "total_approved_amount"),
        requires_manager_review: row.get("requires_manager_review"),
        requires_finance_review: row.get("requires_finance_review"),
        status: row.get("status"),
        reviewed_by_id: row.get("reviewed_by_id"),
        review_notes: row.get("review_notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_violation(row: &sqlx::postgres::PgRow) -> ExpenseComplianceViolation {
    ExpenseComplianceViolation {
        id: row.get("id"),
        org_id: row.get("org_id"),
        audit_id: row.get("audit_id"),
        report_id: row.get("report_id"),
        report_line_id: row.get("report_line_id"),
        policy_rule_id: row.get("policy_rule_id"),
        rule_code: row.get("rule_code"),
        rule_name: row.get("rule_name"),
        rule_type: row.get("rule_type"),
        severity: row.get("severity"),
        violation_description: row.get("violation_description"),
        expense_amount: get_opt_numeric(row, "expense_amount"),
        threshold_amount: get_opt_numeric(row, "threshold_amount"),
        excess_amount: get_opt_numeric(row, "excess_amount"),
        resolution_status: row.get("resolution_status"),
        justification: row.get("justification"),
        resolved_by_id: row.get("resolved_by_id"),
        resolution_date: row.get("resolution_date"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// PostgreSQL implementation
pub struct PostgresExpensePolicyComplianceRepository {
    pool: PgPool,
}

impl PostgresExpensePolicyComplianceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExpensePolicyComplianceRepository for PostgresExpensePolicyComplianceRepository {
    // ========================================================================
    // Policy Rules
    // ========================================================================

    async fn create_rule(
        &self, org_id: Uuid, rule_code: &str, name: &str, description: Option<&str>,
        rule_type: &str, expense_category: &str, severity: &str, evaluation_scope: &str,
        threshold_amount: Option<&str>, maximum_amount: Option<&str>,
        threshold_days: i32, requires_receipt: bool, requires_justification: bool,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        applies_to_department: Option<&str>, applies_to_cost_center: Option<&str>,
        created_by_id: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicyRule> {
        let sql = format!(
            "INSERT INTO fin_expense_policy_rules \
            (org_id, rule_code, name, description, rule_type, expense_category, \
             severity, evaluation_scope, threshold_amount, maximum_amount, \
             threshold_days, requires_receipt, requires_justification, \
             effective_from, effective_to, applies_to_department, \
             applies_to_cost_center, created_by_id, status) \
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,$11,$12,$13,$14,$15,$16,$17,$18,'active') \
            {RULE_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(org_id).bind(rule_code).bind(name).bind(description)
        .bind(rule_type).bind(expense_category).bind(severity)
        .bind(evaluation_scope).bind(threshold_amount).bind(maximum_amount)
        .bind(threshold_days).bind(requires_receipt).bind(requires_justification)
        .bind(effective_from).bind(effective_to).bind(applies_to_department)
        .bind(applies_to_cost_center).bind(created_by_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn get_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<Option<ExpensePolicyRule>> {
        let sql = format!("SELECT {RULE_COLS} FROM fin_expense_policy_rules WHERE org_id=$1 AND rule_code=$2");
        let row = sqlx::query(&sql)
        .bind(org_id).bind(rule_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpensePolicyRule>> {
        let sql = format!("SELECT {RULE_COLS} FROM fin_expense_policy_rules WHERE id=$1");
        let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_rule(&r)))
    }

    async fn list_rules(&self, org_id: Uuid, status: Option<&str>, rule_type: Option<&str>) -> AtlasResult<Vec<ExpensePolicyRule>> {
        let sql = format!(
            "SELECT {RULE_COLS} FROM fin_expense_policy_rules \
            WHERE org_id=$1 AND ($2::text IS NULL OR status=$2) \
                AND ($3::text IS NULL OR rule_type=$3) \
            ORDER BY rule_code"
        );
        let rows = sqlx::query(&sql)
        .bind(org_id).bind(status).bind(rule_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_rule).collect())
    }

    async fn update_rule_status(&self, id: Uuid, status: &str) -> AtlasResult<ExpensePolicyRule> {
        let sql = format!(
            "UPDATE fin_expense_policy_rules SET status=$2, is_active = ($2 = 'active'), \
            updated_at=now() WHERE id=$1 {RULE_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_rule(&row))
    }

    async fn delete_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM fin_expense_policy_rules WHERE org_id=$1 AND rule_code=$2"
        )
        .bind(org_id).bind(rule_code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Compliance Audits
    // ========================================================================

    async fn create_audit(
        &self, org_id: Uuid, audit_number: &str, report_id: Uuid,
        report_number: Option<&str>, employee_id: Option<Uuid>,
        employee_name: Option<&str>, department_id: Option<Uuid>,
        audit_date: chrono::NaiveDate, audit_trigger: &str,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        let sql = format!(
            "INSERT INTO fin_expense_compliance_audits \
            (org_id, audit_number, report_id, report_number, \
             employee_id, employee_name, department_id, \
             audit_date, audit_trigger) \
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) \
            {AUDIT_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(org_id).bind(audit_number).bind(report_id).bind(report_number)
        .bind(employee_id).bind(employee_name).bind(department_id)
        .bind(audit_date).bind(audit_trigger)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_audit(&row))
    }

    async fn get_audit(&self, id: Uuid) -> AtlasResult<Option<ExpenseComplianceAudit>> {
        let sql = format!("SELECT {AUDIT_COLS} FROM fin_expense_compliance_audits WHERE id=$1");
        let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_audit(&r)))
    }

    async fn get_audit_by_number(&self, org_id: Uuid, audit_number: &str) -> AtlasResult<Option<ExpenseComplianceAudit>> {
        let sql = format!("SELECT {AUDIT_COLS} FROM fin_expense_compliance_audits WHERE org_id=$1 AND audit_number=$2");
        let row = sqlx::query(&sql)
        .bind(org_id).bind(audit_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_audit(&r)))
    }

    async fn list_audits(&self, org_id: Uuid, status: Option<&str>, risk_level: Option<&str>) -> AtlasResult<Vec<ExpenseComplianceAudit>> {
        let sql = format!(
            "SELECT {AUDIT_COLS} FROM fin_expense_compliance_audits \
            WHERE org_id=$1 AND ($2::text IS NULL OR status=$2) \
                AND ($3::text IS NULL OR risk_level=$3) \
            ORDER BY audit_date DESC"
        );
        let rows = sqlx::query(&sql)
        .bind(org_id).bind(status).bind(risk_level)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_audit).collect())
    }

    async fn update_audit_results(
        &self, id: Uuid, total_lines: i32, violations_count: i32, warnings_count: i32,
        blocks_count: i32, compliance_score: &str, risk_level: &str,
        total_flagged_amount: &str, total_approved_amount: &str,
        requires_manager_review: bool, requires_finance_review: bool,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        let sql = format!(
            "UPDATE fin_expense_compliance_audits \
            SET total_lines=$2, violations_count=$3, warnings_count=$4, \
                blocks_count=$5, compliance_score=$6::numeric, risk_level=$7, \
                total_flagged_amount=$8::numeric, total_approved_amount=$9::numeric, \
                requires_manager_review=$10, requires_finance_review=$11, \
                updated_at=now() \
            WHERE id=$1 {AUDIT_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(id).bind(total_lines).bind(violations_count).bind(warnings_count)
        .bind(blocks_count).bind(compliance_score).bind(risk_level)
        .bind(total_flagged_amount).bind(total_approved_amount)
        .bind(requires_manager_review).bind(requires_finance_review)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_audit(&row))
    }

    async fn update_audit_review(
        &self, id: Uuid, status: &str, reviewed_by_id: Option<Uuid>, review_notes: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        let sql = format!(
            "UPDATE fin_expense_compliance_audits \
            SET status=$2, reviewed_by_id=COALESCE($3, reviewed_by_id), \
                review_notes=COALESCE($4, review_notes), updated_at=now() \
            WHERE id=$1 {AUDIT_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(id).bind(status).bind(reviewed_by_id).bind(review_notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_audit(&row))
    }

    async fn get_latest_audit_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(audit_number FROM 5) AS INTEGER)), 0) as max_num \
            FROM fin_expense_compliance_audits WHERE org_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max: i32 = row.try_get("max_num").unwrap_or(0);
        Ok(max)
    }

    // ========================================================================
    // Violations
    // ========================================================================

    async fn create_violation(
        &self, org_id: Uuid, audit_id: Uuid, report_id: Uuid,
        report_line_id: Option<Uuid>, policy_rule_id: Option<Uuid>,
        rule_code: &str, rule_name: Option<&str>, rule_type: &str,
        severity: &str, violation_description: Option<&str>,
        expense_amount: Option<&str>, threshold_amount: Option<&str>,
        excess_amount: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceViolation> {
        let sql = format!(
            "INSERT INTO fin_expense_compliance_violations \
            (org_id, audit_id, report_id, report_line_id, policy_rule_id, \
             rule_code, rule_name, rule_type, severity, violation_description, \
             expense_amount, threshold_amount, excess_amount) \
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::numeric,$12::numeric,$13::numeric) \
            {VIOLATION_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(org_id).bind(audit_id).bind(report_id).bind(report_line_id)
        .bind(policy_rule_id).bind(rule_code).bind(rule_name)
        .bind(rule_type).bind(severity).bind(violation_description)
        .bind(expense_amount).bind(threshold_amount).bind(excess_amount)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation(&row))
    }

    async fn list_violations(&self, audit_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>> {
        let sql = format!("SELECT {VIOLATION_COLS} FROM fin_expense_compliance_violations WHERE audit_id=$1 ORDER BY created_at");
        let rows = sqlx::query(&sql)
        .bind(audit_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_violation).collect())
    }

    async fn get_violation_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpenseComplianceViolation>> {
        let sql = format!("SELECT {VIOLATION_COLS} FROM fin_expense_compliance_violations WHERE id=$1");
        let row = sqlx::query(&sql)
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_violation(&r)))
    }

    async fn update_violation_resolution(
        &self, id: Uuid, resolution_status: &str, justification: Option<&str>,
        resolved_by_id: Option<Uuid>, resolution_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ExpenseComplianceViolation> {
        let sql = format!(
            "UPDATE fin_expense_compliance_violations \
            SET resolution_status=$2, justification=COALESCE($3, justification), \
                resolved_by_id=COALESCE($4, resolved_by_id), \
                resolution_date=COALESCE($5, resolution_date), updated_at=now() \
            WHERE id=$1 {VIOLATION_RETURNING}"
        );
        let row = sqlx::query(&sql)
        .bind(id).bind(resolution_status).bind(justification)
        .bind(resolved_by_id).bind(resolution_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_violation(&row))
    }

    async fn list_open_violations(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>> {
        let sql = format!("SELECT {VIOLATION_COLS} FROM fin_expense_compliance_violations WHERE org_id=$1 AND resolution_status='open' ORDER BY created_at DESC");
        let rows = sqlx::query(&sql)
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_violation).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ExpenseComplianceDashboard> {
        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM fin_expense_policy_rules WHERE org_id = $1 AND is_active = true) as active_rules,
                (SELECT COUNT(*) FROM fin_expense_compliance_audits
                    WHERE org_id = $1 AND EXTRACT(YEAR FROM audit_date) = EXTRACT(YEAR FROM CURRENT_DATE)) as audits_period,
                (SELECT COUNT(*) FROM fin_expense_compliance_violations v
                    JOIN fin_expense_compliance_audits a ON v.audit_id = a.id
                    WHERE a.org_id = $1 AND v.severity = 'violation'
                    AND EXTRACT(YEAR FROM v.created_at) = EXTRACT(YEAR FROM CURRENT_DATE)) as violations_period,
                (SELECT COUNT(*) FROM fin_expense_compliance_violations v
                    JOIN fin_expense_compliance_audits a ON v.audit_id = a.id
                    WHERE a.org_id = $1 AND v.severity = 'warning'
                    AND EXTRACT(YEAR FROM v.created_at) = EXTRACT(YEAR FROM CURRENT_DATE)) as warnings_period,
                (SELECT COUNT(*) FROM fin_expense_compliance_violations v
                    JOIN fin_expense_compliance_audits a ON v.audit_id = a.id
                    WHERE a.org_id = $1 AND v.severity = 'block'
                    AND EXTRACT(YEAR FROM v.created_at) = EXTRACT(YEAR FROM CURRENT_DATE)) as blocks_period,
                (SELECT COALESCE(AVG(compliance_score::numeric), 0)::text FROM fin_expense_compliance_audits
                    WHERE org_id = $1 AND EXTRACT(YEAR FROM audit_date) = EXTRACT(YEAR FROM CURRENT_DATE)) as avg_score,
                (SELECT COALESCE(SUM(total_flagged_amount::numeric), 0)::text FROM fin_expense_compliance_audits
                    WHERE org_id = $1 AND EXTRACT(YEAR FROM audit_date) = EXTRACT(YEAR FROM CURRENT_DATE)) as flagged_amt,
                (SELECT COUNT(*) FROM fin_expense_compliance_audits
                    WHERE org_id = $1 AND risk_level IN ('high', 'critical')
                    AND EXTRACT(YEAR FROM audit_date) = EXTRACT(YEAR FROM CURRENT_DATE)) as high_risk,
                (SELECT COUNT(*) FROM fin_expense_compliance_violations v
                    JOIN fin_expense_compliance_audits a ON v.audit_id = a.id
                    WHERE a.org_id = $1 AND v.resolution_status = 'open') as open_violations"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_rules: i64 = row.try_get("active_rules").unwrap_or(0);
        let audits_period: i64 = row.try_get("audits_period").unwrap_or(0);
        let violations_period: i64 = row.try_get("violations_period").unwrap_or(0);
        let warnings_period: i64 = row.try_get("warnings_period").unwrap_or(0);
        let blocks_period: i64 = row.try_get("blocks_period").unwrap_or(0);
        let avg_score: String = row.try_get("avg_score").unwrap_or_else(|_| "100".to_string());
        let flagged_amt: String = row.try_get("flagged_amt").unwrap_or_else(|_| "0".to_string());
        let high_risk: i64 = row.try_get("high_risk").unwrap_or(0);
        let open_violations: i64 = row.try_get("open_violations").unwrap_or(0);

        Ok(ExpenseComplianceDashboard {
            total_active_rules: active_rules as i32,
            total_audits_period: audits_period as i32,
            total_violations_period: violations_period as i32,
            total_warnings_period: warnings_period as i32,
            total_blocks_period: blocks_period as i32,
            avg_compliance_score: avg_score,
            total_flagged_amount: flagged_amt,
            high_risk_audits: high_risk as i32,
            open_violations: open_violations as i32,
        })
    }
}
