//! Supplier Qualification Repository
//!
//! PostgreSQL storage for qualification areas, questions, initiatives,
//! invitations, responses, and certifications.

use atlas_shared::{
    QualificationArea, QualificationQuestion,
    SupplierQualificationInitiative, SupplierQualificationInvitation,
    SupplierQualificationResponse, SupplierCertification,
    SupplierQualificationDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for supplier qualification data storage
#[async_trait]
pub trait SupplierQualificationRepository: Send + Sync {
    // Qualification Areas
    async fn create_area(
        &self, org_id: Uuid, area_code: &str, name: &str, description: Option<&str>,
        area_type: &str, scoring_model: &str, passing_score: &str,
        is_mandatory: bool, renewal_period_days: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<QualificationArea>;
    async fn get_area(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualificationArea>>;
    async fn get_area_by_id(&self, id: Uuid) -> AtlasResult<Option<QualificationArea>>;
    async fn list_areas(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualificationArea>>;
    async fn delete_area(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Qualification Questions
    async fn create_question(
        &self, org_id: Uuid, area_id: Uuid, question_number: i32,
        question_text: &str, description: Option<&str>, response_type: &str,
        choices: Option<serde_json::Value>, is_required: bool, weight: &str,
        max_score: &str, help_text: Option<&str>, display_order: i32,
    ) -> AtlasResult<QualificationQuestion>;
    async fn list_questions(&self, area_id: Uuid) -> AtlasResult<Vec<QualificationQuestion>>;
    async fn delete_question(&self, id: Uuid) -> AtlasResult<()>;

    // Initiatives
    async fn create_initiative(
        &self, org_id: Uuid, initiative_number: &str, name: &str,
        description: Option<&str>, area_id: Uuid, qualification_purpose: &str,
        deadline: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInitiative>;
    async fn get_initiative(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInitiative>>;
    async fn list_initiatives(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SupplierQualificationInitiative>>;
    async fn update_initiative_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierQualificationInitiative>;
    async fn update_initiative_counts(
        &self, id: Uuid, invited: i32, responded: i32, qualified: i32,
        disqualified: i32, pending: i32,
    ) -> AtlasResult<()>;

    // Invitations
    async fn create_invitation(
        &self, org_id: Uuid, initiative_id: Uuid, supplier_id: Uuid,
        supplier_name: &str, supplier_contact_name: Option<&str>,
        supplier_contact_email: Option<&str>, expiry_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInvitation>;
    async fn get_invitation(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInvitation>>;
    async fn list_invitations_by_initiative(&self, initiative_id: Uuid) -> AtlasResult<Vec<SupplierQualificationInvitation>>;
    async fn list_invitations_by_supplier(&self, org_id: Uuid, supplier_id: Uuid) -> AtlasResult<Vec<SupplierQualificationInvitation>>;
    async fn update_invitation_status(
        &self, id: Uuid, status: &str, response_date: Option<chrono::DateTime<chrono::Utc>>,
        evaluation_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<SupplierQualificationInvitation>;
    async fn update_invitation_scores(
        &self, id: Uuid, overall_score: &str, max_possible_score: &str,
        score_percentage: &str, qualified_by: Option<Uuid>,
        disqualified_reason: Option<&str>, evaluation_notes: Option<&str>,
    ) -> AtlasResult<SupplierQualificationInvitation>;

    // Responses
    async fn create_response(
        &self, org_id: Uuid, invitation_id: Uuid, question_id: Uuid,
        response_text: Option<&str>, response_value: Option<serde_json::Value>,
        file_reference: Option<&str>,
    ) -> AtlasResult<SupplierQualificationResponse>;
    async fn get_response(&self, invitation_id: Uuid, question_id: Uuid) -> AtlasResult<Option<SupplierQualificationResponse>>;
    async fn list_responses(&self, invitation_id: Uuid) -> AtlasResult<Vec<SupplierQualificationResponse>>;
    async fn score_response(
        &self, id: Uuid, score: &str, evaluator_notes: Option<&str>,
        evaluated_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationResponse>;

    // Certifications
    async fn create_certification(
        &self, org_id: Uuid, supplier_id: Uuid, supplier_name: &str,
        certification_type: &str, certification_name: &str,
        certifying_body: Option<&str>, certificate_number: Option<&str>,
        status: &str, issued_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>, renewal_date: Option<chrono::NaiveDate>,
        qualification_invitation_id: Option<Uuid>, document_reference: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierCertification>;
    async fn get_certification(&self, id: Uuid) -> AtlasResult<Option<SupplierCertification>>;
    async fn list_certifications(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierCertification>>;
    async fn update_certification_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierCertification>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SupplierQualificationDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresSupplierQualificationRepository {
    pool: PgPool,
}

impl PostgresSupplierQualificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn get_numeric(&self, row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
}

#[async_trait]
impl SupplierQualificationRepository for PostgresSupplierQualificationRepository {
    // ========================================================================
    // Qualification Areas
    // ========================================================================

    async fn create_area(
        &self, org_id: Uuid, area_code: &str, name: &str, description: Option<&str>,
        area_type: &str, scoring_model: &str, passing_score: &str,
        is_mandatory: bool, renewal_period_days: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<QualificationArea> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.qualification_areas
                (organization_id, area_code, name, description, area_type, scoring_model,
                 passing_score, is_mandatory, renewal_period_days, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7::numeric,$8,$9,$10) RETURNING *"#,
        )
        .bind(org_id).bind(area_code).bind(name).bind(description)
        .bind(area_type).bind(scoring_model).bind(passing_score)
        .bind(is_mandatory).bind(renewal_period_days).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(QualificationArea {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            area_code: row.get("area_code"),
            name: row.get("name"),
            description: row.get("description"),
            area_type: row.get("area_type"),
            scoring_model: row.get("scoring_model"),
            passing_score: self.get_numeric(&row, "passing_score"),
            is_mandatory: row.get("is_mandatory"),
            renewal_period_days: row.get("renewal_period_days"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_area(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualificationArea>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.qualification_areas WHERE organization_id=$1 AND area_code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| QualificationArea {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            area_code: r.get("area_code"),
            name: r.get("name"),
            description: r.get("description"),
            area_type: r.get("area_type"),
            scoring_model: r.get("scoring_model"),
            passing_score: self.get_numeric(&r, "passing_score"),
            is_mandatory: r.get("is_mandatory"),
            renewal_period_days: r.get("renewal_period_days"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn get_area_by_id(&self, id: Uuid) -> AtlasResult<Option<QualificationArea>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.qualification_areas WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| QualificationArea {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            area_code: r.get("area_code"),
            name: r.get("name"),
            description: r.get("description"),
            area_type: r.get("area_type"),
            scoring_model: r.get("scoring_model"),
            passing_score: self.get_numeric(&r, "passing_score"),
            is_mandatory: r.get("is_mandatory"),
            renewal_period_days: r.get("renewal_period_days"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_areas(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualificationArea>> {
        let rows = if active_only {
            sqlx::query(
                "SELECT * FROM _atlas.qualification_areas WHERE organization_id=$1 AND is_active=true ORDER BY area_code"
            ).bind(org_id).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.qualification_areas WHERE organization_id=$1 ORDER BY area_code"
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| QualificationArea {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            area_code: r.get("area_code"),
            name: r.get("name"),
            description: r.get("description"),
            area_type: r.get("area_type"),
            scoring_model: r.get("scoring_model"),
            passing_score: self.get_numeric(r, "passing_score"),
            is_mandatory: r.get("is_mandatory"),
            renewal_period_days: r.get("renewal_period_days"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_area(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.qualification_areas WHERE organization_id=$1 AND area_code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Qualification Questions
    // ========================================================================

    async fn create_question(
        &self, org_id: Uuid, area_id: Uuid, question_number: i32,
        question_text: &str, description: Option<&str>, response_type: &str,
        choices: Option<serde_json::Value>, is_required: bool, weight: &str,
        max_score: &str, help_text: Option<&str>, display_order: i32,
    ) -> AtlasResult<QualificationQuestion> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.qualification_questions
                (organization_id, area_id, question_number, question_text, description,
                 response_type, choices, is_required, weight, max_score, help_text, display_order)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,$11,$12) RETURNING *"#,
        )
        .bind(org_id).bind(area_id).bind(question_number).bind(question_text)
        .bind(description).bind(response_type).bind(choices).bind(is_required)
        .bind(weight).bind(max_score).bind(help_text).bind(display_order)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(QualificationQuestion {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            area_id: row.get("area_id"),
            question_number: row.get("question_number"),
            question_text: row.get("question_text"),
            description: row.get("description"),
            response_type: row.get("response_type"),
            choices: row.try_get("choices").unwrap_or(None),
            is_required: row.get("is_required"),
            weight: self.get_numeric(&row, "weight"),
            max_score: self.get_numeric(&row, "max_score"),
            help_text: row.get("help_text"),
            display_order: row.get("display_order"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_questions(&self, area_id: Uuid) -> AtlasResult<Vec<QualificationQuestion>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.qualification_questions WHERE area_id=$1 AND is_active=true ORDER BY question_number"
        )
        .bind(area_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| QualificationQuestion {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            area_id: r.get("area_id"),
            question_number: r.get("question_number"),
            question_text: r.get("question_text"),
            description: r.get("description"),
            response_type: r.get("response_type"),
            choices: r.try_get("choices").unwrap_or(None),
            is_required: r.get("is_required"),
            weight: self.get_numeric(r, "weight"),
            max_score: self.get_numeric(r, "max_score"),
            help_text: r.get("help_text"),
            display_order: r.get("display_order"),
            is_active: r.get("is_active"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn delete_question(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.qualification_questions WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Initiatives
    // ========================================================================

    async fn create_initiative(
        &self, org_id: Uuid, initiative_number: &str, name: &str,
        description: Option<&str>, area_id: Uuid, qualification_purpose: &str,
        deadline: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInitiative> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.supplier_qualification_initiatives
                (organization_id, initiative_number, name, description, area_id,
                 qualification_purpose, deadline, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"#,
        )
        .bind(org_id).bind(initiative_number).bind(name).bind(description)
        .bind(area_id).bind(qualification_purpose).bind(deadline).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(SupplierQualificationInitiative {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            initiative_number: row.get("initiative_number"),
            name: row.get("name"),
            description: row.get("description"),
            area_id: row.get("area_id"),
            qualification_purpose: row.get("qualification_purpose"),
            status: row.get("status"),
            deadline: row.get("deadline"),
            total_invited: row.get("total_invited"),
            total_responded: row.get("total_responded"),
            total_qualified: row.get("total_qualified"),
            total_disqualified: row.get("total_disqualified"),
            total_pending: row.get("total_pending"),
            completed_at: row.get("completed_at"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_initiative(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInitiative>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_initiatives WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| SupplierQualificationInitiative {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            initiative_number: r.get("initiative_number"),
            name: r.get("name"),
            description: r.get("description"),
            area_id: r.get("area_id"),
            qualification_purpose: r.get("qualification_purpose"),
            status: r.get("status"),
            deadline: r.get("deadline"),
            total_invited: r.get("total_invited"),
            total_responded: r.get("total_responded"),
            total_qualified: r.get("total_qualified"),
            total_disqualified: r.get("total_disqualified"),
            total_pending: r.get("total_pending"),
            completed_at: r.get("completed_at"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_initiatives(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SupplierQualificationInitiative>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.supplier_qualification_initiatives
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| SupplierQualificationInitiative {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            initiative_number: r.get("initiative_number"),
            name: r.get("name"),
            description: r.get("description"),
            area_id: r.get("area_id"),
            qualification_purpose: r.get("qualification_purpose"),
            status: r.get("status"),
            deadline: r.get("deadline"),
            total_invited: r.get("total_invited"),
            total_responded: r.get("total_responded"),
            total_qualified: r.get("total_qualified"),
            total_disqualified: r.get("total_disqualified"),
            total_pending: r.get("total_pending"),
            completed_at: r.get("completed_at"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn update_initiative_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierQualificationInitiative> {
        let row = sqlx::query(
            r#"UPDATE _atlas.supplier_qualification_initiatives SET status=$2,
                completed_at=CASE WHEN $2='completed' AND completed_at IS NULL THEN now() ELSE completed_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(SupplierQualificationInitiative {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            initiative_number: row.get("initiative_number"),
            name: row.get("name"),
            description: row.get("description"),
            area_id: row.get("area_id"),
            qualification_purpose: row.get("qualification_purpose"),
            status: row.get("status"),
            deadline: row.get("deadline"),
            total_invited: row.get("total_invited"),
            total_responded: row.get("total_responded"),
            total_qualified: row.get("total_qualified"),
            total_disqualified: row.get("total_disqualified"),
            total_pending: row.get("total_pending"),
            completed_at: row.get("completed_at"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn update_initiative_counts(
        &self, id: Uuid, invited: i32, responded: i32, qualified: i32,
        disqualified: i32, pending: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.supplier_qualification_initiatives SET
                total_invited=$2, total_responded=$3, total_qualified=$4,
                total_disqualified=$5, total_pending=$6, updated_at=now() WHERE id=$1"#,
        )
        .bind(id).bind(invited).bind(responded).bind(qualified)
        .bind(disqualified).bind(pending)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Invitations
    // ========================================================================

    async fn create_invitation(
        &self, org_id: Uuid, initiative_id: Uuid, supplier_id: Uuid,
        supplier_name: &str, supplier_contact_name: Option<&str>,
        supplier_contact_email: Option<&str>, expiry_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.supplier_qualification_invitations
                (organization_id, initiative_id, supplier_id, supplier_name,
                 supplier_contact_name, supplier_contact_email, expiry_date, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *"#,
        )
        .bind(org_id).bind(initiative_id).bind(supplier_id).bind(supplier_name)
        .bind(supplier_contact_name).bind(supplier_contact_email)
        .bind(expiry_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_invitation(&row))
    }

    async fn get_invitation(&self, id: Uuid) -> AtlasResult<Option<SupplierQualificationInvitation>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_invitations WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_invitation(&r)))
    }

    async fn list_invitations_by_initiative(&self, initiative_id: Uuid) -> AtlasResult<Vec<SupplierQualificationInvitation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_invitations WHERE initiative_id=$1 ORDER BY supplier_name"
        )
        .bind(initiative_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_invitation).collect())
    }

    async fn list_invitations_by_supplier(&self, org_id: Uuid, supplier_id: Uuid) -> AtlasResult<Vec<SupplierQualificationInvitation>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_invitations WHERE organization_id=$1 AND supplier_id=$2 ORDER BY created_at DESC"
        )
        .bind(org_id).bind(supplier_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_invitation).collect())
    }

    async fn update_invitation_status(
        &self, id: Uuid, status: &str, response_date: Option<chrono::DateTime<chrono::Utc>>,
        evaluation_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.supplier_qualification_invitations SET status=$2,
                response_date=COALESCE($3, response_date),
                evaluation_date=COALESCE($4, evaluation_date),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(response_date).bind(evaluation_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_invitation(&row))
    }

    async fn update_invitation_scores(
        &self, id: Uuid, overall_score: &str, max_possible_score: &str,
        score_percentage: &str, qualified_by: Option<Uuid>,
        disqualified_reason: Option<&str>, evaluation_notes: Option<&str>,
    ) -> AtlasResult<SupplierQualificationInvitation> {
        let row = sqlx::query(
            r#"UPDATE _atlas.supplier_qualification_invitations SET
                overall_score=$2::numeric, max_possible_score=$3::numeric,
                score_percentage=$4::numeric, qualified_by=COALESCE($5, qualified_by),
                disqualified_reason=COALESCE($6, disqualified_reason),
                evaluation_notes=COALESCE($7, evaluation_notes),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(overall_score).bind(max_possible_score).bind(score_percentage)
        .bind(qualified_by).bind(disqualified_reason).bind(evaluation_notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_invitation(&row))
    }

    // ========================================================================
    // Responses
    // ========================================================================

    async fn create_response(
        &self, org_id: Uuid, invitation_id: Uuid, question_id: Uuid,
        response_text: Option<&str>, response_value: Option<serde_json::Value>,
        file_reference: Option<&str>,
    ) -> AtlasResult<SupplierQualificationResponse> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.supplier_qualification_responses
                (organization_id, invitation_id, question_id, response_text,
                 response_value, file_reference)
            VALUES ($1,$2,$3,$4,$5,$6) RETURNING *"#,
        )
        .bind(org_id).bind(invitation_id).bind(question_id).bind(response_text)
        .bind(response_value).bind(file_reference)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_response(&row))
    }

    async fn get_response(&self, invitation_id: Uuid, question_id: Uuid) -> AtlasResult<Option<SupplierQualificationResponse>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_responses WHERE invitation_id=$1 AND question_id=$2"
        )
        .bind(invitation_id).bind(question_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_response(&r)))
    }

    async fn list_responses(&self, invitation_id: Uuid) -> AtlasResult<Vec<SupplierQualificationResponse>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.supplier_qualification_responses WHERE invitation_id=$1 ORDER BY created_at"
        )
        .bind(invitation_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_response).collect())
    }

    async fn score_response(
        &self, id: Uuid, score: &str, evaluator_notes: Option<&str>,
        evaluated_by: Option<Uuid>,
    ) -> AtlasResult<SupplierQualificationResponse> {
        let row = sqlx::query(
            r#"UPDATE _atlas.supplier_qualification_responses SET
                score=$2::numeric, evaluator_notes=COALESCE($3, evaluator_notes),
                evaluated_by=COALESCE($4, evaluated_by),
                evaluated_at=CASE WHEN evaluated_at IS NULL THEN now() ELSE evaluated_at END,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(score).bind(evaluator_notes).bind(evaluated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_response(&row))
    }

    // ========================================================================
    // Certifications
    // ========================================================================

    async fn create_certification(
        &self, org_id: Uuid, supplier_id: Uuid, supplier_name: &str,
        certification_type: &str, certification_name: &str,
        certifying_body: Option<&str>, certificate_number: Option<&str>,
        status: &str, issued_date: Option<chrono::NaiveDate>,
        expiry_date: Option<chrono::NaiveDate>, renewal_date: Option<chrono::NaiveDate>,
        qualification_invitation_id: Option<Uuid>, document_reference: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<SupplierCertification> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.supplier_certifications
                (organization_id, supplier_id, supplier_name, certification_type,
                 certification_name, certifying_body, certificate_number, status,
                 issued_date, expiry_date, renewal_date, qualification_invitation_id,
                 document_reference, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) RETURNING *"#,
        )
        .bind(org_id).bind(supplier_id).bind(supplier_name).bind(certification_type)
        .bind(certification_name).bind(certifying_body).bind(certificate_number)
        .bind(status).bind(issued_date).bind(expiry_date).bind(renewal_date)
        .bind(qualification_invitation_id).bind(document_reference).bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(SupplierCertification {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            supplier_id: row.get("supplier_id"),
            supplier_name: row.get("supplier_name"),
            certification_type: row.get("certification_type"),
            certification_name: row.get("certification_name"),
            certifying_body: row.get("certifying_body"),
            certificate_number: row.get("certificate_number"),
            status: row.get("status"),
            issued_date: row.get("issued_date"),
            expiry_date: row.get("expiry_date"),
            renewal_date: row.get("renewal_date"),
            qualification_invitation_id: row.get("qualification_invitation_id"),
            document_reference: row.get("document_reference"),
            notes: row.get("notes"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn get_certification(&self, id: Uuid) -> AtlasResult<Option<SupplierCertification>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.supplier_certifications WHERE id=$1"
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| SupplierCertification {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            supplier_id: r.get("supplier_id"),
            supplier_name: r.get("supplier_name"),
            certification_type: r.get("certification_type"),
            certification_name: r.get("certification_name"),
            certifying_body: r.get("certifying_body"),
            certificate_number: r.get("certificate_number"),
            status: r.get("status"),
            issued_date: r.get("issued_date"),
            expiry_date: r.get("expiry_date"),
            renewal_date: r.get("renewal_date"),
            qualification_invitation_id: r.get("qualification_invitation_id"),
            document_reference: r.get("document_reference"),
            notes: r.get("notes"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn list_certifications(&self, org_id: Uuid, supplier_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SupplierCertification>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.supplier_certifications
            WHERE organization_id=$1 AND ($2::uuid IS NULL OR supplier_id=$2)
            AND ($3::text IS NULL OR status=$3)
            ORDER BY certification_name"#,
        )
        .bind(org_id).bind(supplier_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| SupplierCertification {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            supplier_id: r.get("supplier_id"),
            supplier_name: r.get("supplier_name"),
            certification_type: r.get("certification_type"),
            certification_name: r.get("certification_name"),
            certifying_body: r.get("certifying_body"),
            certificate_number: r.get("certificate_number"),
            status: r.get("status"),
            issued_date: r.get("issued_date"),
            expiry_date: r.get("expiry_date"),
            renewal_date: r.get("renewal_date"),
            qualification_invitation_id: r.get("qualification_invitation_id"),
            document_reference: r.get("document_reference"),
            notes: r.get("notes"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    async fn update_certification_status(&self, id: Uuid, status: &str) -> AtlasResult<SupplierCertification> {
        let row = sqlx::query(
            r#"UPDATE _atlas.supplier_certifications SET status=$2, updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(SupplierCertification {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            supplier_id: row.get("supplier_id"),
            supplier_name: row.get("supplier_name"),
            certification_type: row.get("certification_type"),
            certification_name: row.get("certification_name"),
            certifying_body: row.get("certifying_body"),
            certificate_number: row.get("certificate_number"),
            status: row.get("status"),
            issued_date: row.get("issued_date"),
            expiry_date: row.get("expiry_date"),
            renewal_date: row.get("renewal_date"),
            qualification_invitation_id: row.get("qualification_invitation_id"),
            document_reference: row.get("document_reference"),
            notes: row.get("notes"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SupplierQualificationDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM _atlas.qualification_areas WHERE organization_id=$1 AND is_active=true) as active_areas,
                (SELECT COUNT(*) FROM _atlas.supplier_qualification_initiatives WHERE organization_id=$1 AND status IN ('active','pending_evaluations')) as active_initiatives,
                (SELECT COUNT(*) FROM _atlas.supplier_qualification_invitations WHERE organization_id=$1 AND status='initiated') as pending_invitations,
                (SELECT COUNT(*) FROM _atlas.supplier_qualification_invitations WHERE organization_id=$1 AND status='qualified') as qualified_suppliers,
                (SELECT COUNT(*) FROM _atlas.supplier_qualification_invitations WHERE organization_id=$1 AND status IN ('initiated','pending_response','under_evaluation')) as pending_suppliers,
                (SELECT COUNT(*) FROM _atlas.supplier_qualification_invitations WHERE organization_id=$1 AND status='disqualified') as disqualified_suppliers,
                (SELECT COUNT(*) FROM _atlas.supplier_certifications WHERE organization_id=$1 AND status='active') as active_certs,
                (SELECT COUNT(*) FROM _atlas.supplier_certifications WHERE organization_id=$1 AND status='active' AND expiry_date <= CURRENT_DATE + INTERVAL '30 days') as expiring_certs"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_areas: i64 = row.try_get("active_areas").unwrap_or(0);
        let active_initiatives: i64 = row.try_get("active_initiatives").unwrap_or(0);
        let pending_invitations: i64 = row.try_get("pending_invitations").unwrap_or(0);
        let qualified_suppliers: i64 = row.try_get("qualified_suppliers").unwrap_or(0);
        let pending_suppliers: i64 = row.try_get("pending_suppliers").unwrap_or(0);
        let disqualified_suppliers: i64 = row.try_get("disqualified_suppliers").unwrap_or(0);
        let active_certs: i64 = row.try_get("active_certs").unwrap_or(0);
        let expiring_certs: i64 = row.try_get("expiring_certs").unwrap_or(0);

        let total_evaluated = qualified_suppliers + disqualified_suppliers;
        let qual_rate = if total_evaluated > 0 {
            qualified_suppliers as f64 / total_evaluated as f64 * 100.0
        } else {
            0.0
        };

        Ok(SupplierQualificationDashboardSummary {
            total_active_areas: active_areas as i32,
            total_active_initiatives: active_initiatives as i32,
            total_suppliers_invited: pending_invitations as i32,
            total_suppliers_qualified: qualified_suppliers as i32,
            total_suppliers_pending: pending_suppliers as i32,
            total_suppliers_disqualified: disqualified_suppliers as i32,
            total_certifications_active: active_certs as i32,
            total_certifications_expiring_30_days: expiring_certs as i32,
            qualification_rate_percent: format!("{:.1}", qual_rate),
            initiatives_by_status: serde_json::json!({}),
            certifications_by_type: serde_json::json!({}),
        })
    }
}

// Helper functions to map rows

fn row_to_invitation(row: &sqlx::postgres::PgRow) -> SupplierQualificationInvitation {
    SupplierQualificationInvitation {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        initiative_id: row.get("initiative_id"),
        supplier_id: row.get("supplier_id"),
        supplier_name: row.get("supplier_name"),
        supplier_contact_name: row.get("supplier_contact_name"),
        supplier_contact_email: row.get("supplier_contact_email"),
        status: row.get("status"),
        invitation_date: row.get("invitation_date"),
        response_date: row.get("response_date"),
        evaluation_date: row.get("evaluation_date"),
        expiry_date: row.get("expiry_date"),
        overall_score: row_to_numeric(row, "overall_score"),
        max_possible_score: row_to_numeric(row, "max_possible_score"),
        score_percentage: row_to_numeric(row, "score_percentage"),
        qualified_by: row.get("qualified_by"),
        disqualified_reason: row.get("disqualified_reason"),
        evaluation_notes: row.get("evaluation_notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_response(row: &sqlx::postgres::PgRow) -> SupplierQualificationResponse {
    SupplierQualificationResponse {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        invitation_id: row.get("invitation_id"),
        question_id: row.get("question_id"),
        response_text: row.get("response_text"),
        response_value: row.try_get("response_value").unwrap_or(None),
        file_reference: row.get("file_reference"),
        score: row_to_numeric(row, "score"),
        max_score: row_to_numeric(row, "max_score"),
        evaluator_notes: row.get("evaluator_notes"),
        evaluated_by: row.get("evaluated_by"),
        evaluated_at: row.get("evaluated_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_numeric(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}
